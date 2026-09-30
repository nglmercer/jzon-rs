//! Incremental native reader: one lookahead byte, nesting frames and a scalar
//! token buffer. No document buffer or callback retries. RawValue necessarily
//! owns its requested raw value; ordinary/ignored containers do not retain it.
use super::bridge::{Bridge, BridgeVisitor};
use super::*;
use core::marker::PhantomData;
use serde::de::{DeserializeSeed, Deserializer as _};
use std::io::{self, Read};

pub struct ReaderDeserializer<R> {
    reader: R,
    peeked: Option<u8>,
    offset: usize,
    line: usize,
    column: usize,
    token: Vec<u8>,
    origin: (usize, usize),
    capture: Option<Vec<u8>>,
    depth: u8,
    disabled: bool,
    peak: usize,
    #[cfg(feature = "stats")]
    stats: crate::stats::ScannerStats,
}
impl<R: Read> ReaderDeserializer<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            peeked: None,
            offset: 0,
            line: 1,
            column: 0,
            token: Vec::new(),
            origin: (1, 0),
            capture: None,
            depth: crate::scanner::MAX_DEPTH,
            disabled: false,
            peak: 0,
            #[cfg(feature = "stats")]
            stats: crate::stats::ScannerStats::default(),
        }
    }
    pub fn byte_offset(&self) -> usize {
        self.offset
    }
    /// Largest scalar token length retained while parsing (excludes output and raw capture).
    pub fn peak_token_len(&self) -> usize {
        self.peak
    }
    /// Return the source and its already-read lookahead byte, preserving partial consumption.
    pub fn into_parts(self) -> (R, Option<u8>) {
        (self.reader, self.peeked)
    }
    pub fn get_ref(&self) -> &R {
        &self.reader
    }
    #[cfg(feature = "unbounded_depth")]
    pub fn disable_recursion_limit(&mut self) {
        self.disabled = true;
    }
    pub(super) fn located(&self, error: Error) -> Error {
        let (line, column) = if error.is_syntax() && self.peeked == Some(b'\n') {
            (self.line + 1, 0)
        } else {
            (self.line, self.column + usize::from(error.is_syntax()))
        };
        error.at(line, column)
    }
    fn syntax<T>(&self) -> Result<T, Error> {
        Err(self.located(Error::Scanner(crate::Error::UnexpectedToken)))
    }
    fn eof<T>(&self) -> Result<T, Error> {
        Err(self.located(Error::Scanner(crate::Error::UnexpectedEof)))
    }
    fn peek(&mut self) -> Result<Option<u8>, Error> {
        if self.peeked.is_none() {
            let mut byte = [0];
            loop {
                match self.reader.read(&mut byte) {
                    Ok(0) => return Ok(None),
                    Ok(_) => {
                        self.peeked = Some(byte[0]);
                        break;
                    }
                    Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(e) => return Err(self.located(Error::Io(e))),
                }
            }
        }
        Ok(self.peeked)
    }
    fn take(&mut self) -> Result<u8, Error> {
        let Some(byte) = self.peek()? else {
            return self.eof();
        };
        self.peeked = None;
        self.offset += 1;
        if byte == b'\n' {
            self.line += 1;
            self.column = 0;
        } else {
            self.column += 1;
        }
        if let Some(capture) = &mut self.capture {
            capture.push(byte);
        }
        Ok(byte)
    }
    fn ws(&mut self) -> Result<(), Error> {
        while matches!(self.peek()?, Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.take()?;
        }
        Ok(())
    }
    fn expect(&mut self, byte: u8) -> Result<(), Error> {
        self.ws()?;
        match self.peek()? {
            Some(b) if b == byte => {
                self.take()?;
                Ok(())
            }
            None => self.eof(),
            _ => self.syntax(),
        }
    }
    pub fn end(&mut self) -> Result<(), Error> {
        self.ws()?;
        if self.peek()?.is_none() {
            Ok(())
        } else {
            self.syntax()
        }
    }
    #[allow(clippy::should_implement_trait)] // T is chosen by the caller; IntoIterator cannot express this generic constructor.
    pub fn into_iter<T: DeserializeOwned>(self) -> ReaderStream<R, T> {
        ReaderStream {
            de: self,
            failed: false,
            marker: PhantomData,
        }
    }
    fn with_depth<T>(
        &mut self,
        operation: impl FnOnce(&mut Self) -> Result<T, Error>,
    ) -> Result<T, Error> {
        self.ws()?;
        if self.disabled {
            return operation(self);
        }
        if self.depth <= 1 {
            return Err(self.located(Error::Scanner(crate::Error::RecursionLimit)));
        }
        self.depth -= 1;
        struct Restore<'a, R>(&'a mut ReaderDeserializer<R>);
        impl<R> Drop for Restore<'_, R> {
            fn drop(&mut self) {
                self.0.depth += 1;
            }
        }
        let guard = Restore(self);
        operation(&mut *guard.0)
    }
    fn token(&mut self) -> Result<(), Error> {
        self.ws()?;
        self.origin = (self.line, self.column);
        self.token.clear();
        let Some(first) = self.peek()? else {
            return self.eof();
        };
        self.token.push(first);
        match first {
            b'"' => {
                self.take()?;
                loop {
                    let byte = self.take()?;
                    self.token.push(byte);
                    if byte == b'"' {
                        break;
                    }
                    if byte == b'\\' {
                        let escaped = self.take()?;
                        self.token.push(escaped);
                    }
                }
            }
            b't' | b'f' | b'n' => {
                self.take()?;
                let literal: &[u8] = match first {
                    b'f' => b"false",
                    b't' => b"true",
                    _ => b"null",
                };
                for &expected in &literal[1..] {
                    let byte = self.take()?;
                    self.token.push(byte);
                    if byte != expected {
                        return Err(Error::Scanner(crate::Error::UnexpectedToken)
                            .at(self.line, self.column));
                    }
                }
            }
            b'-' | b'0'..=b'9' => {
                self.take()?;
                while matches!(
                    self.peek()?,
                    Some(b'0'..=b'9' | b'-' | b'+' | b'.' | b'e' | b'E')
                ) {
                    let byte = self.take()?;
                    self.token.push(byte);
                }
            }
            _ => {}
        }
        self.peak = self.peak.max(self.token.len());
        Ok(())
    }
    #[cfg(feature = "stats")]
    pub fn stats(&self) -> &crate::stats::ScannerStats {
        &self.stats
    }
    fn scalar<'de, V: Visitor<'de>>(
        &mut self,
        visitor: V,
        operation: impl for<'short> FnOnce(
            &mut Deserializer<'short>,
            BridgeVisitor<'de, V>,
        ) -> Result<V::Value, Error>,
    ) -> Result<V::Value, Error> {
        self.token()?;
        struct Storage<'a>(&'a mut Vec<u8>);
        impl Drop for Storage<'_> {
            fn drop(&mut self) {
                if self.0.capacity() > 64 * 1024 {
                    *self.0 = Vec::new();
                }
            }
        }
        let result = {
            let storage = Storage(&mut self.token);
            let mut parser = Deserializer::from_slice(storage.0);
            let result =
                operation(&mut parser, BridgeVisitor(visitor, PhantomData)).and_then(|value| {
                    parser.end()?;
                    Ok(value)
                });
            #[cfg(feature = "stats")]
            let stats = parser.stats().clone();
            #[cfg(feature = "stats")]
            {
                self.stats.decoded_strings += stats.decoded_strings;
                self.stats.number_bytes_scanned += stats.number_bytes_scanned;
            }
            result
        };
        result.map_err(|error| self.rebase(error))
    }
    fn rebase(&self, error: Error) -> Error {
        if error.is_eof() && self.peeked.is_some() {
            return self.located(Error::Scanner(crate::Error::UnexpectedToken));
        }
        match error {
            Error::Positioned {
                source,
                line,
                column,
            } => Error::Positioned {
                source,
                line: self.origin.0 + line - 1,
                column: if line == 1 {
                    self.origin.1 + column
                } else {
                    column
                },
            },
            other => self.located(other),
        }
    }
    fn key<'de, S: DeserializeSeed<'de>>(&mut self, seed: S) -> Result<S::Value, Error> {
        self.ws()?;
        if self.peek()? != Some(b'"') {
            return if self.peek()?.is_none() {
                self.eof()
            } else {
                self.syntax()
            };
        }
        self.token()?;
        let mut parser = Deserializer::from_slice(&self.token);
        let result = seed.deserialize(Bridge(MapKeyDeserializer { de: &mut parser }, PhantomData));
        let result = result.map_err(|error| parser.position_error(error));
        let result = result
            .and_then(|value| {
                parser.end()?;
                Ok(value)
            })
            .map_err(|e| self.rebase(e));
        if self.token.capacity() > 64 * 1024 {
            self.token = Vec::new();
        }
        result
    }
    fn lexical_token(&mut self) -> Result<(), Error> {
        self.token()?;
        let mut parser = Deserializer::from_slice(&self.token);
        let result = parser
            .scanner
            .skip_value_serde()
            .and_then(|_| parser.scanner.expect_eof())
            .map_err(|error| parser.position_error(Error::Scanner(error)))
            .map_err(|error| self.rebase(error));
        if self.token.capacity() > 64 * 1024 {
            self.token = Vec::new();
        }
        result
    }
    // Iterative grammar for lexical ignored/raw values. Preserve their distinct
    // string/byte model and avoid recursive stack growth or document retention.
    fn skip(&mut self) -> Result<(), Error> {
        enum State {
            Value,
            ArrayFirst,
            ArrayNext,
            ArrayRest,
            ObjectFirst,
            ObjectNext,
            ObjectRest,
        }
        use State::*;
        let mut stack = vec![Value];
        while let Some(state) = stack.pop() {
            self.ws()?;
            let Some(byte) = self.peek()? else {
                return self.eof();
            };
            match state {
                Value => match byte {
                    b'[' => {
                        self.take()?;
                        stack.push(ArrayFirst);
                    }
                    b'{' => {
                        self.take()?;
                        stack.push(ObjectFirst);
                    }
                    _ => {
                        self.lexical_token()?;
                    }
                },
                ArrayFirst if byte == b']' => {
                    self.take()?;
                }
                ArrayFirst | ArrayNext => {
                    if byte == b']' {
                        return Err(self.located(Error::Scanner(crate::Error::TrailingComma)));
                    }
                    stack.push(ArrayRest);
                    stack.push(Value);
                }
                ArrayRest => match byte {
                    b']' => {
                        self.take()?;
                    }
                    b',' => {
                        self.take()?;
                        stack.push(ArrayNext);
                    }
                    _ => return self.syntax(),
                },
                ObjectFirst if byte == b'}' => {
                    self.take()?;
                }
                ObjectFirst | ObjectNext => {
                    if byte != b'"' {
                        return self.syntax();
                    }
                    self.lexical_token()?;
                    self.expect(b':')?;
                    stack.push(ObjectRest);
                    stack.push(Value);
                }
                ObjectRest => match byte {
                    b'}' => {
                        self.take()?;
                    }
                    b',' => {
                        self.take()?;
                        stack.push(ObjectNext);
                    }
                    _ => return self.syntax(),
                },
            }
        }
        Ok(())
    }
}
pub struct ReaderStream<R, T> {
    de: ReaderDeserializer<R>,
    failed: bool,
    marker: PhantomData<T>,
}
impl<R: Read, T: DeserializeOwned> ReaderStream<R, T> {
    pub fn byte_offset(&self) -> usize {
        self.de.byte_offset()
    }
    pub fn into_parts(self) -> (R, Option<u8>) {
        self.de.into_parts()
    }
}
impl<R: Read, T: DeserializeOwned> Iterator for ReaderStream<R, T> {
    type Item = Result<T, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.failed {
            return None;
        }
        let result = (|| {
            self.de.ws()?;
            if self.de.peek()?.is_none() {
                return Ok(None);
            }
            let self_delineated = matches!(self.de.peek()?, Some(b'[' | b'{' | b'"'));
            let value = T::deserialize(&mut self.de)?;
            if !self_delineated
                && !matches!(
                    self.de.peek()?,
                    None | Some(
                        b' ' | b'\n'
                            | b'\r'
                            | b'\t'
                            | b'"'
                            | b'['
                            | b']'
                            | b'{'
                            | b'}'
                            | b','
                            | b':'
                    )
                )
            {
                return self.de.syntax();
            }
            Ok(Some(value))
        })();
        match result {
            Ok(None) => None,
            Ok(Some(v)) => Some(Ok(v)),
            Err(e) => {
                self.failed = true;
                Some(Err(self.de.located(e)))
            }
        }
    }
}
impl<R: Read, T: DeserializeOwned> core::iter::FusedIterator for ReaderStream<R, T> {}

macro_rules! scalar {
    ($name:ident $(, $arg:ident:$ty:ty)*)=>{
        fn $name<V:Visitor<'de>>(self,$($arg:$ty,)*visitor:V)->Result<V::Value,Error>{
            self.scalar(visitor,|parser,visitor|serde::Deserializer::$name(parser,$($arg,)*visitor))
        }
    }
}
impl<'de, R: Read> serde::Deserializer<'de> for &mut ReaderDeserializer<R> {
    type Error = Error;
    scalar!(deserialize_bool);
    scalar!(deserialize_i8);
    scalar!(deserialize_i16);
    scalar!(deserialize_i32);
    scalar!(deserialize_i64);
    scalar!(deserialize_i128);
    scalar!(deserialize_u8);
    scalar!(deserialize_u16);
    scalar!(deserialize_u32);
    scalar!(deserialize_u64);
    scalar!(deserialize_u128);
    scalar!(deserialize_f32);
    scalar!(deserialize_f64);
    scalar!(deserialize_char);
    scalar!(deserialize_str);
    scalar!(deserialize_string);
    scalar!(deserialize_unit);
    scalar!(deserialize_unit_struct,name:&'static str);
    scalar!(deserialize_identifier);
    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.ws()?;
        match self.peek()? {
            Some(b'[') => self.deserialize_seq(visitor),
            Some(b'{') => self.deserialize_map(visitor),
            _ => self.scalar(visitor, |parser, visitor| parser.deserialize_any(visitor)),
        }
    }
    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.ws()?;
        if self.peek()? == Some(b'[') {
            self.deserialize_seq(visitor)
        } else {
            self.scalar(visitor, |parser, visitor| parser.deserialize_bytes(visitor))
        }
    }
    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_bytes(visitor)
    }
    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.ws()?;
        if self.peek()? == Some(b'n') {
            self.scalar(visitor, |parser, visitor| {
                parser.deserialize_option(visitor)
            })
        } else {
            visitor.visit_some(&mut *self).map_err(|e| self.located(e))
        }
    }
    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        if name == RAW_VALUE_TOKEN {
            self.ws()?;
            self.origin = (self.line, self.column);
            self.capture = Some(Vec::new());
            let result = self.skip();
            let bytes = self.capture.take().unwrap();
            result?;
            let mut parser = Deserializer::from_slice(&bytes);
            parser
                .deserialize_newtype_struct(name, BridgeVisitor(visitor, PhantomData))
                .map_err(|e| self.located(e))
        } else {
            visitor
                .visit_newtype_struct(&mut *self)
                .map_err(|e| self.located(e))
        }
    }
    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.ws()?;
        if self.peek()? != Some(b'[') {
            return self.scalar(visitor, |parser, visitor| parser.deserialize_seq(visitor));
        }
        self.with_depth(|de| {
            de.expect(b'[')?;
            let mut access = Access {
                de,
                first: true,
                done: false,
                pending: false,
            };
            let value = visitor
                .visit_seq(&mut access)
                .map_err(|e| access.de.located(e))?;
            if !access.done {
                access.de.expect(b']')?;
            }
            Ok(value)
        })
    }
    fn deserialize_tuple<V: Visitor<'de>>(
        self,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.deserialize_seq(visitor)
    }
    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.deserialize_seq(visitor)
    }
    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.ws()?;
        if self.peek()? != Some(b'{') {
            return self.scalar(visitor, |parser, visitor| parser.deserialize_map(visitor));
        }
        self.with_depth(|de| {
            de.expect(b'{')?;
            let mut access = Access {
                de,
                first: true,
                done: false,
                pending: false,
            };
            let value = visitor
                .visit_map(&mut access)
                .map_err(|e| access.de.located(e))?;
            if access.pending {
                return Err(access
                    .de
                    .located(Error::Custom("map value not consumed".into())));
            }
            if !access.done {
                access.de.expect(b'}')?;
            }
            Ok(value)
        })
    }
    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.ws()?;
        if self.peek()? == Some(b'[') {
            self.deserialize_seq(visitor)
        } else {
            self.deserialize_map(visitor)
        }
    }
    fn deserialize_enum<V: Visitor<'de>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.ws()?;
        if self.peek()? != Some(b'{') {
            self.scalar(visitor, |parser, visitor| {
                parser.deserialize_enum(name, variants, visitor)
            })
        } else {
            self.with_depth(|de| {
                de.expect(b'{')?;
                let value = visitor
                    .visit_enum(ReaderEnum { de: &mut *de })
                    .map_err(|e| de.located(e))?;
                de.expect(b'}')?;
                Ok(value)
            })
        }
    }
    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.skip()?;
        visitor.visit_unit().map_err(|e| self.located(e))
    }
}
struct Access<'a, R> {
    de: &'a mut ReaderDeserializer<R>,
    first: bool,
    done: bool,
    pending: bool,
}
impl<R: Read> Access<'_, R> {
    fn next(&mut self, end: u8) -> Result<bool, Error> {
        if self.done {
            return Ok(false);
        }
        self.de.ws()?;
        if self.de.peek()? == Some(end) {
            self.de.take()?;
            self.done = true;
            return Ok(false);
        }
        if !self.first {
            self.de.expect(b',')?;
            self.de.ws()?;
            if self.de.peek()? == Some(end) {
                return Err(self.de.located(Error::Scanner(crate::Error::TrailingComma)));
            }
        }
        Ok(true)
    }
}
impl<'de, R: Read> SeqAccess<'de> for Access<'_, R> {
    type Error = Error;
    fn next_element_seed<S: DeserializeSeed<'de>>(
        &mut self,
        seed: S,
    ) -> Result<Option<S::Value>, Error> {
        if !self.next(b']')? {
            return Ok(None);
        }
        self.first = false;
        seed.deserialize(&mut *self.de).map(Some)
    }
}
impl<'de, R: Read> MapAccess<'de> for Access<'_, R> {
    type Error = Error;
    fn next_key_seed<S: DeserializeSeed<'de>>(
        &mut self,
        seed: S,
    ) -> Result<Option<S::Value>, Error> {
        if self.pending {
            return Err(self
                .de
                .located(Error::Custom("map key before value".into())));
        }
        if !self.next(b'}')? {
            return Ok(None);
        }
        // Check grammar before changing access state or invoking a user seed.
        if self.de.peek()? != Some(b'"') {
            return if self.de.peek()?.is_none() {
                self.de.eof()
            } else {
                self.de.syntax()
            };
        }
        self.first = false;
        let key = self.de.key(seed)?;
        self.de.expect(b':')?;
        self.pending = true;
        Ok(Some(key))
    }
    fn next_value_seed<S: DeserializeSeed<'de>>(&mut self, seed: S) -> Result<S::Value, Error> {
        if !self.pending {
            return Err(self
                .de
                .located(Error::Custom("map value without key".into())));
        }
        self.pending = false;
        seed.deserialize(&mut *self.de)
    }
}
struct ReaderEnum<'a, R> {
    de: &'a mut ReaderDeserializer<R>,
}
impl<'a, 'de, R: Read> EnumAccess<'de> for ReaderEnum<'a, R> {
    type Error = Error;
    type Variant = Self;
    fn variant_seed<S: DeserializeSeed<'de>>(self, seed: S) -> Result<(S::Value, Self), Error> {
        // Check grammar before changing access state or invoking a user seed.
        if self.de.peek()? != Some(b'"') {
            return if self.de.peek()?.is_none() {
                self.de.eof()
            } else {
                self.de.syntax()
            };
        }
        let key = self.de.key(seed)?;
        self.de.expect(b':')?;
        Ok((key, self))
    }
}
impl<'de, R: Read> VariantAccess<'de> for ReaderEnum<'_, R> {
    type Error = Error;
    fn unit_variant(self) -> Result<(), Error> {
        serde::Deserialize::deserialize(&mut *self.de)
    }
    fn newtype_variant_seed<S: DeserializeSeed<'de>>(self, seed: S) -> Result<S::Value, Error> {
        seed.deserialize(&mut *self.de)
    }
    fn tuple_variant<V: Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value, Error> {
        self.de.deserialize_tuple(len, visitor)
    }
    fn struct_variant<V: Visitor<'de>>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.de.deserialize_struct("", fields, visitor)
    }
}
