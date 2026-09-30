use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc,
};

use crate::{simd, Error};

#[cold]
#[inline]
fn err_eof() -> Error {
    Error::UnexpectedEof
}

#[cold]
#[inline]
fn err_token() -> Error {
    Error::UnexpectedToken
}

/// A parsed JSON string: either a zero-copy borrow or a heap-allocated value.
///
/// The `BorrowedNoEsc` variant is returned by [`Scanner::read_str`] when no
/// escape sequences were present in the JSON input. Publicly constructed values
/// are still escaped by the serializer; this variant is not a trust boundary.
pub enum JsonStr<'de> {
    /// Zero-copy borrow from the input.  **No longer emitted by [`Scanner::read_str`]**
    /// (use [`JsonStr::BorrowedNoEsc`] instead); kept for API compatibility.
    /// `ToJson` will run `write_escaped_str` on this variant.
    Borrowed(&'de str),
    /// Zero-copy borrow whose content is **provably escape-free** (the scanner
    /// hit a closing `"` before any `\\`).  The serializer can bypass the
    /// `find_escape` scan and write the bytes directly.
    BorrowedNoEsc(&'de str),
    Owned(String),
}

impl<'de> JsonStr<'de> {
    #[inline]
    pub fn as_borrowed(&self) -> Option<&'de str> {
        match self {
            JsonStr::Borrowed(s) => Some(s),
            JsonStr::BorrowedNoEsc(s) => Some(s),
            JsonStr::Owned(_) => None,
        }
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        match self {
            JsonStr::Borrowed(s) => s,
            JsonStr::BorrowedNoEsc(s) => s,
            JsonStr::Owned(s) => s.as_str(),
        }
    }

    #[inline]
    pub fn into_owned(self) -> String {
        match self {
            JsonStr::Borrowed(s) => s.to_owned(),
            JsonStr::BorrowedNoEsc(s) => s.to_owned(),
            JsonStr::Owned(s) => s,
        }
    }
}

/// Strings decoded for native visitors preserve distinct input/scratch lifetimes.
#[cfg(feature = "serde")]
pub(crate) enum DecodedStr<'de, 'scratch> {
    Borrowed(&'de str),
    Transient(&'scratch str),
}

/// Levels of array/object nesting a single parse may enter, mirroring
/// `serde_json` (127 nested opens parse, the 128th errors).
pub const MAX_DEPTH: u8 = 128;

pub struct Scanner<'de> {
    input: &'de [u8],
    pos: usize,
    remaining_depth: Option<Arc<AtomicU8>>,
    depth_limit_disabled: bool,
    #[cfg(feature = "stats")]
    pub stats: crate::stats::ScannerStats,
}

/// Owns the depth counter independently of the scanner. Dropping or moving a
/// scanner while a guard exists is safe: the counter remains alive until the
/// last owner drops. Drop performs only infallible depth restoration, including
/// during unwinding. Leaking a guard can reduce the budget, never bypass it.
#[must_use = "bind the guard for the duration of the composite value"]
#[derive(Debug)]
pub struct DepthGuard {
    remaining: Option<Arc<AtomicU8>>,
}

impl Drop for DepthGuard {
    fn drop(&mut self) {
        if let Some(remaining) = &self.remaining {
            remaining.fetch_add(1, Ordering::Relaxed);
        }
    }
}

impl<'de> Scanner<'de> {
    #[inline]
    pub fn new(input: &'de [u8]) -> Self {
        Scanner {
            input,
            pos: 0,
            remaining_depth: None,
            depth_limit_disabled: false,
            #[cfg(feature = "stats")]
            stats: crate::stats::ScannerStats::default(),
        }
    }

    #[inline]
    pub fn new_str(s: &'de str) -> Self {
        Self::new(s.as_bytes())
    }

    #[inline]
    pub fn peek_byte(&self) -> Result<u8, Error> {
        self.input.get(self.pos).copied().ok_or_else(err_eof)
    }

    #[inline]
    pub fn advance(&mut self) {
        self.pos += 1;
    }

    /// Byte offset into the input slice — used by internally-tagged enum parsers to checkpoint and re-scan.
    #[inline]
    pub fn pos(&self) -> usize {
        self.pos
    }

    #[inline]
    pub fn set_pos(&mut self, saved_pos: usize) {
        self.pos = saved_pos;
    }

    #[inline]
    pub fn advance_by(&mut self, n: usize) {
        self.pos += n;
    }

    /// Remaining unprocessed input — used by single-pass float parsers (`fast_float2::parse_partial`).
    #[inline]
    pub fn remaining_input(&self) -> &'de [u8] {
        &self.input[self.pos..]
    }

    /// Record a field-hint cache hit. No-op unless the `stats` feature is on.
    ///
    /// Called by `#[derive(FromJson)]` generated code; always compiled so the
    /// proc-macro output stays identical across feature sets.
    #[inline]
    pub fn record_hint_hit(&mut self) {
        #[cfg(feature = "stats")]
        {
            self.stats.hint_hits += 1;
        }
    }

    /// Record a field-hint cache miss. No-op unless the `stats` feature is on.
    #[inline]
    pub fn record_hint_miss(&mut self) {
        #[cfg(feature = "stats")]
        {
            self.stats.hint_misses += 1;
        }
    }

    /// Reject a trailing comma when closing a composite: call with
    /// `after_comma` set if a `,` was just consumed. Always `Ok` without
    /// the `strict` feature (the default parser is a lenient superset).
    #[inline]
    pub fn check_trailing_comma(&self, after_comma: bool) -> Result<(), Error> {
        #[cfg(feature = "strict")]
        {
            if after_comma {
                return Err(Error::TrailingComma);
            }
        }
        #[cfg(not(feature = "strict"))]
        {
            let _ = after_comma;
        }
        Ok(())
    }

    /// Enter a nesting level. The default budget remains active with all
    /// features; the guard owns its counter and restores it on error or unwind.
    #[inline]
    pub fn enter_depth(&mut self) -> Result<DepthGuard, Error> {
        if self.depth_limit_disabled {
            return Ok(DepthGuard { remaining: None });
        }
        // Scalars and unescaped borrowed strings need no counter allocation.
        let counter = self
            .remaining_depth
            .get_or_insert_with(|| Arc::new(AtomicU8::new(MAX_DEPTH)));
        let mut remaining = counter.load(Ordering::Relaxed);
        loop {
            if remaining <= 1 {
                return Err(Error::RecursionLimit);
            }
            match counter.compare_exchange_weak(
                remaining,
                remaining - 1,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(current) => remaining = current,
            }
        }
        Ok(DepthGuard {
            remaining: Some(Arc::clone(counter)),
        })
    }

    /// Explicitly opt out of the nesting limit. The caller must ensure that
    /// recursive parsing and destruction cannot exhaust the stack.
    #[cfg(feature = "unbounded_depth")]
    pub fn disable_recursion_limit(&mut self) {
        self.depth_limit_disabled = true;
    }

    #[inline]
    pub fn expect_byte(&mut self, expected: u8) -> Result<(), Error> {
        match self.input.get(self.pos) {
            Some(&b) if b == expected => {
                self.pos += 1;
                Ok(())
            }
            _ => Err(err_token()),
        }
    }

    pub fn expect_bytes(&mut self, expected: &[u8]) -> Result<(), Error> {
        let end = self.pos + expected.len();
        if self.input.get(self.pos..end) == Some(expected) {
            self.pos = end;
            Ok(())
        } else {
            Err(err_token())
        }
    }

    #[inline(always)]
    pub fn skip_whitespace(&mut self) {
        // Fast path: compact JSON has no leading whitespace — skip the loop entirely.
        // All structural bytes are > b' ' (32), so this correctly identifies non-whitespace.
        if let Some(&b) = self.input.get(self.pos) {
            if b > b' ' {
                return;
            }
        } else {
            return;
        }
        self.skip_whitespace_swar();
    }

    /// JSON whitespace is exactly space, tab, LF and CR. Never skip other
    /// control bytes, including inside an eight-byte scanning block.
    #[inline]
    fn skip_whitespace_swar(&mut self) {
        while matches!(self.input.get(self.pos), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.pos += 1;
        }
    }

    #[inline(always)]
    pub fn peek_byte_after_ws(&mut self) -> Result<u8, Error> {
        self.skip_whitespace();
        self.peek_byte()
    }

    /// After parsing a top-level value, skip trailing whitespace and verify
    /// that no non-whitespace bytes remain (ECMA-404 requires a single value).
    #[inline]
    pub fn expect_eof(&mut self) -> Result<(), Error> {
        self.skip_whitespace();
        if self.pos < self.input.len() {
            Err(Error::UnexpectedToken)
        } else {
            Ok(())
        }
    }

    /// Read a JSON object key as a zero-copy `&'de [u8]`.
    /// Returns `Error::EscapedKey` if the key contains backslashes.
    pub fn read_key(&mut self) -> Result<&'de [u8], Error> {
        self.skip_whitespace();
        self.expect_byte(b'"')?;
        let start = self.pos;
        // When simd-intrinsics are active the find_escape kernels (NEON/AVX2)
        // fuse quote+backslash+ctrl<0x20 in a single SIMD pass — negligible extra
        // cost over find(). On scalar/SWAR paths, use find() + SWAR has_control_char
        // to avoid a byte-by-byte second pass.
        #[cfg(feature = "simd-intrinsics")]
        let stop = simd::find_escape(self.input, self.pos);
        #[cfg(not(feature = "simd-intrinsics"))]
        let stop = simd::find(self.input, self.pos);

        match self.input.get(stop) {
            Some(&b'"') => {
                #[cfg(not(feature = "simd-intrinsics"))]
                if simd::has_control_char(&self.input[start..stop]) {
                    return Err(Error::InvalidEscape);
                }
                self.pos = stop + 1;
                Ok(&self.input[start..stop])
            }
            Some(&b'\\') => Err(Error::EscapedKey),
            Some(_) => Err(Error::InvalidEscape), // control char from find_escape
            _ => Err(err_eof()),
        }
    }

    /// Read a JSON object key and the mandatory `:` separator in one call.
    #[inline]
    pub fn read_key_colon(&mut self) -> Result<&'de [u8], Error> {
        let key = self.read_key()?;
        // Fast path: ':' almost always immediately follows the closing '"' in compact JSON.
        if self.input.get(self.pos) == Some(&b':') {
            self.pos += 1;
        } else {
            self.skip_whitespace();
            self.expect_byte(b':')?;
        }
        Ok(key)
    }

    /// Read a JSON object key and the mandatory `:` separator, returning a
    /// validated UTF-8 key borrow.
    #[inline]
    pub(crate) fn read_str_key_colon(&mut self) -> Result<&'de str, Error> {
        let key = self.read_key_colon()?;
        core::str::from_utf8(key).map_err(|_| Error::InvalidUtf8)
    }

    /// Read a JSON string value.
    ///
    /// Returns [`JsonStr::BorrowedNoEsc`] when no escape sequences are present
    /// (zero allocation, provably escape-free), or [`JsonStr::Owned`] after
    /// unescaping.
    pub fn read_str(&mut self) -> Result<JsonStr<'de>, Error> {
        self.skip_whitespace();
        self.expect_byte(b'"')?;
        let start = self.pos;
        let (stop, _ascii_only) = simd::scan_string_run(self.input, start);

        match self.input.get(stop) {
            Some(&b'"') => {
                let s = core::str::from_utf8(&self.input[start..stop])
                    .map_err(|_| Error::InvalidUtf8)?;
                self.pos = stop + 1;

                #[cfg(feature = "stats")]
                {
                    self.stats.zero_copy_borrows += 1;
                }

                Ok(JsonStr::BorrowedNoEsc(s))
            }
            Some(&b'\\') => {
                self.pos = stop;
                let owned = self.unescape_from(start)?;

                #[cfg(feature = "stats")]
                {
                    self.stats.decoded_strings += 1;
                }

                Ok(JsonStr::Owned(owned))
            }
            Some(_) => Err(Error::InvalidEscape), // control char < 0x20
            None => Err(err_eof()),
        }
    }

    /// Native transient decoding: plain strings borrow the input; escaped
    /// strings borrow the caller's scratch only for the current visitor call.
    #[cfg(feature = "serde")]
    #[inline]
    pub(crate) fn read_str_with_scratch<'scratch>(
        &mut self,
        scratch: &'scratch mut Vec<u8>,
    ) -> Result<DecodedStr<'de, 'scratch>, Error> {
        self.skip_whitespace();
        self.expect_byte(b'"')?;
        let start = self.pos;
        let (stop, _) = simd::scan_string_run(self.input, start);
        match self.input.get(stop) {
            Some(b'"') => {
                let value = core::str::from_utf8(&self.input[start..stop])
                    .map_err(|_| Error::InvalidUtf8)?;
                self.pos = stop + 1;
                #[cfg(feature = "stats")]
                {
                    self.stats.zero_copy_borrows += 1;
                }
                Ok(DecodedStr::Borrowed(value))
            }
            Some(b'\\') => {
                self.pos = stop;
                self.unescape_into(start, scratch)?;
                let value = core::str::from_utf8(scratch).map_err(|_| Error::InvalidUtf8)?;
                #[cfg(feature = "stats")]
                {
                    self.stats.decoded_strings += 1;
                }
                Ok(DecodedStr::Transient(value))
            }
            Some(_) => Err(Error::InvalidEscape),
            None => Err(err_eof()),
        }
    }

    #[inline]
    fn scan_ascii_digits(&mut self) -> usize {
        let start = self.pos;

        // SWAR digit scan: for byte b, b is b'0'..=b'9' iff (b - 0x30) is 0..=9.
        // Two conditions: (1) sub has no high bits (rules out bytes >= 0xB0),
        // (2) sub + 0x76 has no high bits (rules out sub bytes 10..=0x7F).
        #[inline(always)]
        fn swar_all_digits(chunk: u64) -> bool {
            let sub = chunk.wrapping_sub(0x3030_3030_3030_3030_u64);
            if (sub & 0x8080_8080_8080_8080_u64) != 0 {
                return false;
            }
            let check = sub.wrapping_add(0x7676_7676_7676_7676_u64);
            (check & 0x8080_8080_8080_8080_u64) == 0
        }

        while self.pos + 8 <= self.input.len() {
            let chunk = u64::from_le_bytes(self.input[self.pos..self.pos + 8].try_into().unwrap());
            if swar_all_digits(chunk) {
                self.pos += 8;
            } else {
                break;
            }
        }
        while let Some(&b) = self.input.get(self.pos) {
            if b.is_ascii_digit() {
                self.pos += 1;
            } else {
                break;
            }
        }

        self.pos - start
    }

    /// Scan a JSON integer literal into a `u64` in a single pass.
    ///
    /// Strictly validates the integer grammar (`0` | `[1-9][0-9]*`, no sign,
    /// no fraction, no exponent) while accumulating, like serde_json's
    /// `parse_integer`. Rejects float syntax, leading zeros, and overflow —
    /// on any error the cursor position is unspecified (callers abort).
    pub fn read_u64_strict(&mut self) -> Result<u64, Error> {
        self.skip_whitespace();
        let mut n: u64;
        match self.input.get(self.pos) {
            Some(&b'0') => {
                self.pos += 1;
                // Leading zero: next byte must NOT be another digit.
                if matches!(self.input.get(self.pos), Some(b'0'..=b'9')) {
                    return Err(Error::InvalidNumber);
                }
                n = 0;
            }
            Some(&b) if b.is_ascii_digit() => {
                n = (b - b'0') as u64;
                self.pos += 1;
                while let Some(&d) = self.input.get(self.pos) {
                    if !d.is_ascii_digit() {
                        break;
                    }
                    n = n
                        .checked_mul(10)
                        .and_then(|v| v.checked_add((d - b'0') as u64))
                        .ok_or(Error::InvalidNumber)?;
                    self.pos += 1;
                }
            }
            _ => return Err(Error::InvalidNumber),
        }
        // Integer targets reject float syntax (`1.5`, `1e5`).
        if matches!(self.input.get(self.pos), Some(b'.' | b'e' | b'E')) {
            return Err(Error::InvalidNumber);
        }
        Ok(n)
    }

    /// Scan a JSON integer literal into an `i64` in a single pass.
    ///
    /// Same strictness as [`read_u64_strict`](Self::read_u64_strict), plus an
    /// optional `-` sign. Like serde_json, `-0` is treated as float `-0.0`
    /// and rejected for integer targets; `i64::MIN` is accepted.
    pub fn read_i64_strict(&mut self) -> Result<i64, Error> {
        self.skip_whitespace();
        let neg = self.input.get(self.pos) == Some(&b'-');
        if neg {
            self.pos += 1;
        }
        // Accumulate magnitude as u64 so i64::MIN (magnitude 2^63) fits.
        let mut mag: u64;
        match self.input.get(self.pos) {
            Some(&b'0') => {
                self.pos += 1;
                if matches!(self.input.get(self.pos), Some(b'0'..=b'9')) {
                    return Err(Error::InvalidNumber);
                }
                mag = 0;
            }
            Some(&b) if b.is_ascii_digit() => {
                mag = (b - b'0') as u64;
                self.pos += 1;
                while let Some(&d) = self.input.get(self.pos) {
                    if !d.is_ascii_digit() {
                        break;
                    }
                    mag = mag
                        .checked_mul(10)
                        .and_then(|v| v.checked_add((d - b'0') as u64))
                        .ok_or(Error::InvalidNumber)?;
                    self.pos += 1;
                }
            }
            _ => return Err(Error::InvalidNumber),
        }
        if matches!(self.input.get(self.pos), Some(b'.' | b'e' | b'E')) {
            return Err(Error::InvalidNumber);
        }
        if neg {
            // `-0` is float -0.0, rejected for integer targets.
            if mag == 0 {
                return Err(Error::InvalidNumber);
            }
            if mag > i64::MAX as u64 + 1 {
                return Err(Error::InvalidNumber);
            }
            Ok(mag.wrapping_neg() as i64)
        } else {
            if mag > i64::MAX as u64 {
                return Err(Error::InvalidNumber);
            }
            Ok(mag as i64)
        }
    }

    /// Scan a JSON number into an `f64` in a single pass.
    ///
    /// Validates the number head (`-?(0|[1-9]…)` — rejects `+`, `.5`, leading
    /// zeros, `inf`/`nan`) then delegates the digit scan to
    /// `fast_float2::parse_partial`, which reports bytes consumed. Overflow
    /// to infinity errors, like serde_json ("number out of range").
    pub fn read_f64(&mut self) -> Result<f64, Error> {
        self.skip_whitespace();
        let start = self.pos;
        self.check_number_head()?;
        let (val, consumed) = fast_float2::parse_partial::<f64, _>(self.remaining_input())
            .map_err(|_| Error::InvalidNumber)?;
        // parse_partial is lenient about trailing dots (`1.`, `1.e5`); every
        // other invalid shape either fails the head check or leaves the
        // cursor mid-token for a structural error. A `.` must be followed
        // by a digit — single memchr scan, exits at the first dot.
        let span = &self.input[start..start + consumed];
        if let Some(dot) = span.iter().position(|&b| b == b'.') {
            if !matches!(span.get(dot + 1), Some(b'0'..=b'9')) {
                return Err(Error::InvalidNumber);
            }
        }
        self.advance_by(consumed);
        if val.is_infinite() {
            return Err(Error::InvalidNumber);
        }
        Ok(val)
    }

    /// Scan a JSON number into an `f32` in a single pass (see
    /// [`read_f64`](Self::read_f64)). Parses directly as `f32`, so values
    /// overflowing `f32` error rather than saturating.
    pub fn read_f32(&mut self) -> Result<f32, Error> {
        self.skip_whitespace();
        let start = self.pos;
        self.check_number_head()?;
        let (val, consumed) = fast_float2::parse_partial::<f32, _>(self.remaining_input())
            .map_err(|_| Error::InvalidNumber)?;
        let span = &self.input[start..start + consumed];
        if let Some(dot) = span.iter().position(|&b| b == b'.') {
            if !matches!(span.get(dot + 1), Some(b'0'..=b'9')) {
                return Err(Error::InvalidNumber);
            }
        }
        self.advance_by(consumed);
        if val.is_infinite() {
            return Err(Error::InvalidNumber);
        }
        Ok(val)
    }

    /// Validate a JSON number head: optional `-`, then `0` (not followed by a
    /// digit) or `[1-9]`. Rejects everything `parse_partial` would wrongly
    /// accept (`+1`, `.5`, `01`, `inf`, `nan`, `-`, …).
    fn check_number_head(&self) -> Result<(), Error> {
        let mut i = self.pos;
        if self.input.get(i) == Some(&b'-') {
            i += 1;
        }
        match self.input.get(i) {
            Some(&b'0') => {
                if matches!(self.input.get(i + 1), Some(b'0'..=b'9')) {
                    return Err(Error::InvalidNumber);
                }
                Ok(())
            }
            Some(b'1'..=b'9') => Ok(()),
            _ => Err(Error::InvalidNumber),
        }
    }
}

impl<'de> Scanner<'de> {
    /// Scan a JSON number and return the raw byte slice (zero-copy).
    pub fn read_number_bytes(&mut self) -> Result<&'de [u8], Error> {
        self.skip_whitespace();
        let start = self.pos;
        if self.input.get(self.pos) == Some(&b'-') {
            self.pos += 1;
        }

        // Read the integer part.  If it starts with '0', the spec forbids any
        // further digit immediately following (leading zeros like "01" are invalid).
        // An integer part is REQUIRED: inputs like ".5" or "-.5" are invalid
        // JSON (serde_json rejects them) and must not slip through to the
        // fraction/exponent matchers below.
        match self.input.get(self.pos) {
            Some(&b'0') => {
                self.pos += 1;
                // Leading zero: next byte must NOT be another digit.
                if matches!(self.input.get(self.pos), Some(b'0'..=b'9')) {
                    return Err(Error::InvalidNumber);
                }
            }
            Some(&(b'1'..=b'9')) => {
                self.pos += 1;
                self.scan_ascii_digits();
            }
            _ => return Err(Error::InvalidNumber),
        }

        if self.input.get(self.pos) == Some(&b'.') {
            self.pos += 1;
            // At least one digit must follow the decimal point.
            if self.scan_ascii_digits() == 0 {
                // No digit after '.': "1." is invalid JSON.
                return Err(Error::InvalidNumber);
            }
        }
        if matches!(self.input.get(self.pos), Some(b'e') | Some(b'E')) {
            self.pos += 1;
            if matches!(self.input.get(self.pos), Some(b'+') | Some(b'-')) {
                self.pos += 1;
            }
            if self.scan_ascii_digits() == 0 {
                return Err(Error::InvalidNumber);
            }
        }
        let end = self.pos;
        if end == start || (end == start + 1 && self.input[start] == b'-') {
            return Err(Error::InvalidNumber);
        }

        #[cfg(feature = "stats")]
        {
            self.stats.number_bytes_scanned += (end - start) as u64;
        }

        Ok(&self.input[start..end])
    }

    /// Returns true if the next (non-whitespace) bytes are `null` — does NOT consume.
    #[inline]
    pub fn peek_null(&mut self) -> bool {
        self.skip_whitespace();
        self.input.get(self.pos..self.pos + 4) == Some(b"null")
    }

    pub fn read_null(&mut self) -> Result<(), Error> {
        self.skip_whitespace();
        self.expect_bytes(b"null")
    }

    pub fn read_bool(&mut self) -> Result<bool, Error> {
        self.skip_whitespace();
        match self.input.get(self.pos) {
            Some(&b't') => {
                self.pos += 4;
                if self.input.get(self.pos - 3..self.pos) == Some(b"rue") {
                    Ok(true)
                } else {
                    self.pos -= 4;
                    Err(err_token())
                }
            }
            Some(&b'f') => {
                self.pos += 5;
                if self.input.get(self.pos - 4..self.pos) == Some(b"alse") {
                    Ok(false)
                } else {
                    self.pos -= 5;
                    Err(err_token())
                }
            }
            _ => Err(err_token()),
        }
    }

    /// Skip over any JSON value — used for unknown fields.
    pub fn skip_value(&mut self) -> Result<(), Error> {
        self.skip_whitespace();
        match self.peek_byte()? {
            b'"' => self.skip_string(),
            b'{' => self.skip_object(),
            b'[' => self.skip_array(),
            b't' => self.expect_bytes(b"true"),
            b'f' => self.expect_bytes(b"false"),
            b'n' => self.expect_bytes(b"null"),
            b'-' | b'0'..=b'9' => {
                self.read_number_bytes()?;
                Ok(())
            }
            _ => Err(err_token()),
        }
    }

    /// Serde IgnoredAny/RawValue use lexical, iterative skipping, as upstream
    /// does. Typed recursion limits are not applied to this stack-safe path.
    #[cfg(feature = "serde")]
    pub(crate) fn skip_value_serde(&mut self) -> Result<(), Error> {
        struct Frame {
            close: u8,
            first: bool,
            after_value: bool,
        }
        let mut frames: Vec<Frame> = Vec::new();
        let mut need_value = true;
        loop {
            if !need_value && frames.is_empty() {
                return Ok(());
            }
            self.skip_whitespace();
            if need_value {
                match self.peek_byte()? {
                    b'"' => self.skip_string()?,
                    b't' => self.expect_bytes(b"true")?,
                    b'f' => self.expect_bytes(b"false")?,
                    b'n' => self.expect_bytes(b"null")?,
                    b'-' | b'0'..=b'9' => {
                        self.read_number_bytes()?;
                    }
                    open @ (b'[' | b'{') => {
                        self.pos += 1;
                        frames.push(Frame {
                            close: if open == b'[' { b']' } else { b'}' },
                            first: true,
                            after_value: false,
                        });
                    }
                    _ => return Err(err_token()),
                }
                need_value = false;
            }
            let frame = match frames.last_mut() {
                Some(frame) => frame,
                None => return Ok(()),
            };
            self.skip_whitespace();
            let byte = self.peek_byte()?;
            if byte == frame.close && (frame.first || frame.after_value) {
                self.pos += 1;
                frames.pop();
                continue;
            }
            if frame.after_value {
                self.expect_byte(b',')?;
                frame.first = false;
                frame.after_value = false;
                continue;
            }
            // Following a comma a closing delimiter cannot stand in for a
            // value/key. skip_value/read_string will reject it explicitly.
            if frame.close == b'}' {
                self.skip_string()?;
                self.skip_whitespace();
                self.expect_byte(b':')?;
            }
            frame.after_value = true;
            frame.first = false;
            need_value = true;
        }
    }

    fn read_hex_escape(&mut self) -> Result<u32, Error> {
        let hex = self
            .input
            .get(self.pos..self.pos + 4)
            .ok_or(Error::InvalidEscape)?;
        let mut value = 0;
        for &digit in hex {
            value = value * 16
                + match digit {
                    b'0'..=b'9' => (digit - b'0') as u32,
                    b'a'..=b'f' => (digit - b'a' + 10) as u32,
                    b'A'..=b'F' => (digit - b'A' + 10) as u32,
                    _ => return Err(Error::InvalidEscape),
                };
        }
        self.pos += 4;
        Ok(value)
    }

    /// Byte-oriented Serde strings accept non-UTF-8, literal controls and lone
    /// UTF-16 surrogates (encoded as WTF-8), matching the upstream byte model.
    #[cfg(feature = "serde")]
    pub(crate) fn read_byte_str(&mut self) -> Result<std::borrow::Cow<'de, [u8]>, Error> {
        use std::borrow::Cow;
        self.skip_whitespace();
        self.expect_byte(b'"')?;
        let start = self.pos;
        let mut decoded: Option<Vec<u8>> = None;
        loop {
            let byte = self.input.get(self.pos).copied().ok_or_else(err_eof)?;
            self.pos += 1;
            match byte {
                b'"' => {
                    return Ok(match decoded {
                        Some(buf) => Cow::Owned(buf),
                        None => Cow::Borrowed(&self.input[start..self.pos - 1]),
                    })
                }
                b'\\' => {
                    let buf =
                        decoded.get_or_insert_with(|| self.input[start..self.pos - 1].to_vec());
                    let escape = self.input.get(self.pos).copied().ok_or_else(err_eof)?;
                    self.pos += 1;
                    let byte = match escape {
                        b'"' | b'\\' | b'/' => escape,
                        b'b' => 8,
                        b'f' => 12,
                        b'n' => 10,
                        b'r' => 13,
                        b't' => 9,
                        b'u' => {
                            let mut code = self.read_hex_escape()?;
                            if (0xD800..=0xDBFF).contains(&code)
                                && self.input.get(self.pos..self.pos + 2) == Some(b"\\u")
                            {
                                let saved = self.pos;
                                self.pos += 2;
                                let low = self.read_hex_escape()?;
                                if (0xDC00..=0xDFFF).contains(&low) {
                                    code = 0x10000 + ((code - 0xD800) << 10) + low - 0xDC00;
                                } else {
                                    self.pos = saved;
                                }
                            }
                            if let Some(ch) = char::from_u32(code) {
                                let mut bytes = [0; 4];
                                buf.extend_from_slice(ch.encode_utf8(&mut bytes).as_bytes());
                            } else {
                                // A lone UTF-16 surrogate is encoded as WTF-8.
                                buf.extend_from_slice(&[
                                    (0xE0 | (code >> 12)) as u8,
                                    (0x80 | ((code >> 6) & 0x3F)) as u8,
                                    (0x80 | (code & 0x3F)) as u8,
                                ]);
                            }
                            continue;
                        }
                        _ => return Err(Error::InvalidEscape),
                    };
                    buf.push(byte);
                }
                byte => {
                    if let Some(buf) = &mut decoded {
                        buf.push(byte);
                    }
                }
            }
        }
    }

    fn skip_string(&mut self) -> Result<(), Error> {
        self.expect_byte(b'"')?;
        loop {
            let stop = simd::find_escape(self.input, self.pos);
            self.pos = stop;
            match self.input.get(stop).copied() {
                Some(b'"') => {
                    self.pos += 1;
                    return Ok(());
                }
                Some(b'\\') => {
                    self.pos += 1;
                    let escaped = self.input.get(self.pos).copied().ok_or_else(err_eof)?;
                    self.pos += 1;
                    match escaped {
                        b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => {}
                        b'u' => {
                            self.read_hex_escape()?;
                        }
                        _ => return Err(Error::InvalidEscape),
                    }
                }
                Some(_) => return Err(Error::InvalidEscape),
                None => return Err(err_eof()),
            }
        }
    }

    /// Validate and consume the tail after at least one array value was read.
    /// This helper does not suppress syntax errors or allow missing separators.
    pub fn skip_array_tail(&mut self) -> Result<(), Error> {
        loop {
            self.skip_whitespace();
            match self.peek_byte()? {
                b']' => {
                    self.pos += 1;
                    return Ok(());
                }
                b',' => {
                    self.pos += 1;
                    self.skip_value()?;
                }
                _ => return Err(err_token()),
            }
        }
    }

    /// Skip remaining fields of an already-opened object (cursor is just past `{`).
    /// Used by internally-tagged enum deserialization when the variant is unknown.
    pub fn skip_object_tail(&mut self) -> Result<(), Error> {
        let mut after_comma = false;
        loop {
            self.skip_whitespace();
            match self.peek_byte()? {
                b'}' => {
                    self.check_trailing_comma(after_comma)?;
                    self.pos += 1;
                    return Ok(());
                }
                b'"' => {
                    self.skip_string()?;
                    self.skip_whitespace();
                    self.expect_byte(b':')?;
                    self.skip_value()?;
                    self.skip_whitespace();
                    match self.peek_byte()? {
                        b',' => {
                            self.pos += 1;
                            after_comma = true;
                        }
                        b'}' => {
                            self.pos += 1;
                            return Ok(());
                        }
                        _ => return Err(err_token()),
                    }
                }
                _ => return Err(err_token()),
            }
        }
    }

    fn skip_object(&mut self) -> Result<(), Error> {
        let _depth = self.enter_depth()?;
        self.expect_byte(b'{')?;
        self.skip_whitespace();
        if self.input.get(self.pos) == Some(&b'}') {
            self.pos += 1;
            return Ok(());
        }
        loop {
            self.skip_string()?;
            self.skip_whitespace();
            self.expect_byte(b':')?;
            self.skip_value()?;
            self.skip_whitespace();
            match self.peek_byte()? {
                b',' => {
                    self.pos += 1;
                    self.skip_whitespace();
                }
                b'}' => {
                    self.pos += 1;
                    break;
                }
                _ => return Err(err_token()),
            }
        }
        Ok(())
    }

    fn skip_array(&mut self) -> Result<(), Error> {
        let _depth = self.enter_depth()?;
        self.expect_byte(b'[')?;
        self.skip_whitespace();
        if self.input.get(self.pos) == Some(&b']') {
            self.pos += 1;
            return Ok(());
        }
        loop {
            self.skip_value()?;
            self.skip_whitespace();
            match self.peek_byte()? {
                b',' => {
                    self.pos += 1;
                    self.skip_whitespace();
                }
                b']' => {
                    self.pos += 1;
                    break;
                }
                _ => return Err(err_token()),
            }
        }
        Ok(())
    }

    /// Unescape a JSON string whose content starts at `content_start` and whose
    /// first backslash is at `self.pos`.  Returns the fully decoded `String`.
    fn unescape_from(&mut self, content_start: usize) -> Result<String, Error> {
        let mut buf = Vec::new();
        self.unescape_into(content_start, &mut buf)?;
        String::from_utf8(buf).map_err(|_| Error::InvalidUtf8)
    }

    fn unescape_into(&mut self, content_start: usize, buf: &mut Vec<u8>) -> Result<(), Error> {
        // Reserve only the scanned prefix plus modest slack. Unrelated values
        // must not inflate the retained capacity of this string.
        buf.clear();
        buf.reserve(self.pos.saturating_sub(content_start).saturating_add(16));
        // The caller (read_str) already positioned self.pos at the first `\`;
        // find_escape already verified no control chars before that point,
        // so the prefix is clean and we copy it directly.
        buf.extend_from_slice(&self.input[content_start..self.pos]);

        loop {
            match self.input.get(self.pos) {
                Some(&b'"') => {
                    self.pos += 1;
                    break;
                }
                Some(&b'\\') => {
                    self.pos += 1;
                    let esc = self.input.get(self.pos).copied().ok_or_else(err_eof)?;
                    self.pos += 1;
                    match esc {
                        b'"' => buf.push(b'"'),
                        b'\\' => buf.push(b'\\'),
                        b'/' => buf.push(b'/'),
                        b'n' => buf.push(b'\n'),
                        b't' => buf.push(b'\t'),
                        b'r' => buf.push(b'\r'),
                        b'b' => buf.push(0x08),
                        b'f' => buf.push(0x0C),
                        b'u' => {
                            let code = self.read_hex_escape()?;
                            let c = if (0xD800..=0xDBFF).contains(&code) {
                                if self.input.get(self.pos..self.pos + 2) != Some(b"\\u") {
                                    return Err(Error::InvalidEscape);
                                }
                                self.pos += 2;
                                let lo = self.read_hex_escape()?;
                                // The second half must be a low surrogate
                                // (DC00–DFFF); anything else is a lone
                                // leading surrogate (serde_json rejects it).
                                if !(0xDC00..=0xDFFF).contains(&lo) {
                                    return Err(Error::InvalidEscape);
                                }
                                let combined = 0x10000 + ((code - 0xD800) << 10) + (lo - 0xDC00);
                                char::from_u32(combined).ok_or(Error::InvalidEscape)?
                            } else {
                                char::from_u32(code).ok_or(Error::InvalidEscape)?
                            };
                            let mut tmp = [0u8; 4];
                            buf.extend_from_slice(c.encode_utf8(&mut tmp).as_bytes());
                            continue;
                        }
                        _ => return Err(Error::InvalidEscape),
                    }
                }
                Some(_) => {
                    let seg_start = self.pos;
                    // Single pass: find_escape stops at `"`, `\`, or any byte < 0x20.
                    let stop = simd::find_escape(self.input, self.pos);
                    match self.input.get(stop) {
                        Some(&b'"') | Some(&b'\\') => {
                            buf.extend_from_slice(&self.input[seg_start..stop]);
                            self.pos = stop;
                        }
                        Some(_) => return Err(Error::InvalidEscape), // control char
                        None => return Err(err_eof()),
                    }
                }
                None => return Err(err_eof()),
            }
        }

        Ok(())
    }
}
