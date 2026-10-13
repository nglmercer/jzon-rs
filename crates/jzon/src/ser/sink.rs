//! Sealed JSON output sinks for zero-allocation serialization paths.

use crate::__private::*;
#[cfg(feature = "std")]
use std::io;
#[cfg(not(feature = "std"))]
mod io {
    pub type Result<T> = core::result::Result<T, core::convert::Infallible>;
}
pub type SinkResult<T> = io::Result<T>;

mod sealed {
    pub trait Sealed {}
}

/// Output target for [`crate::ToJson::json_write_sink`].
///
/// External crates cannot implement this trait; only built-in sinks are supported
/// in this phase.
pub trait JsonSink: sealed::Sealed {
    /// Append one byte. After overflow/error, subsequent calls are no-ops.
    fn push(&mut self, b: u8);

    /// Append a byte slice. After overflow/error, subsequent calls are no-ops.
    fn extend(&mut self, bs: &[u8]);

    /// Whether all writes succeeded (no overflow / I/O error).
    #[inline]
    fn is_ok(&self) -> bool {
        true
    }

    /// Hint for upcoming writes; default is a no-op.
    #[inline]
    fn reserve(&mut self, _additional: usize) {}
}

/// Adapter so [`JsonSink`] methods can target a [`Vec<u8>`].
pub struct VecSink<'a>(pub &'a mut Vec<u8>);

impl sealed::Sealed for VecSink<'_> {}

impl<'a> JsonSink for VecSink<'a> {
    #[inline]
    fn push(&mut self, b: u8) {
        self.0.push(b);
    }

    #[inline]
    fn extend(&mut self, bs: &[u8]) {
        self.0.extend_from_slice(bs);
    }

    #[inline]
    fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }
}

/// Count serialized bytes without allocating output storage. The count
/// saturates at usize::MAX if a user implementation requests more bytes.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct LengthCounter {
    len: usize,
}

impl LengthCounter {
    #[inline]
    pub const fn new() -> Self {
        Self { len: 0 }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl sealed::Sealed for LengthCounter {}

impl JsonSink for LengthCounter {
    #[inline]
    fn push(&mut self, _b: u8) {
        self.len = self.len.saturating_add(1);
    }

    #[inline]
    fn extend(&mut self, bs: &[u8]) {
        self.len = self.len.saturating_add(bs.len());
    }
}

/// Adapter that writes JSON bytes to any [`io::Write`] target.
#[cfg(feature = "std")]
pub struct IoSink<'a, W: io::Write + ?Sized> {
    w: &'a mut W,
    ok: bool,
    err: Option<io::Error>,
}

#[cfg(feature = "std")]
impl<'a, W: io::Write + ?Sized> IoSink<'a, W> {
    #[inline]
    pub fn new(w: &'a mut W) -> Self {
        Self {
            w,
            ok: true,
            err: None,
        }
    }

    /// Complete the write and return the first I/O error, if any.
    #[inline]
    pub fn finish(self) -> io::Result<()> {
        match self.err {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }
}

#[cfg(feature = "std")]
impl<W: io::Write + ?Sized> sealed::Sealed for IoSink<'_, W> {}

#[cfg(feature = "std")]
impl<W: io::Write + ?Sized> IoSink<'_, W> {
    #[inline]
    fn record_err(&mut self, err: io::Error) {
        if self.ok {
            self.ok = false;
            self.err = Some(err);
        }
    }
}

#[cfg(feature = "std")]
impl<W: io::Write + ?Sized> JsonSink for IoSink<'_, W> {
    #[inline]
    fn push(&mut self, b: u8) {
        if !self.ok {
            return;
        }
        if let Err(e) = self.w.write_all(&[b]) {
            self.record_err(e);
        }
    }

    #[inline]
    fn extend(&mut self, bs: &[u8]) {
        if !self.ok {
            return;
        }
        if let Err(e) = self.w.write_all(bs) {
            self.record_err(e);
        }
    }

    #[inline]
    fn is_ok(&self) -> bool {
        self.ok
    }
}

impl<const N: usize> sealed::Sealed for crate::fixed::FixedBuf<N> {}

impl<const N: usize> JsonSink for crate::fixed::FixedBuf<N> {
    #[inline]
    fn push(&mut self, b: u8) {
        self.sink_push(b);
    }

    #[inline]
    fn extend(&mut self, bs: &[u8]) {
        self.sink_extend(bs);
    }

    #[inline]
    fn is_ok(&self) -> bool {
        self.sink_ok()
    }
}

/// Fallible, statically dispatched output used by the native Serde engine.
/// Sealed to preserve the behavior of the built-in buffer and writer sinks.
pub trait SerializeSink: sealed::Sealed {
    fn push_byte(&mut self, byte: u8) -> io::Result<()>;
    fn write_bytes(&mut self, bytes: &[u8]) -> io::Result<()>;
    fn reserve_bytes(&mut self, _additional: usize) {}
}

impl sealed::Sealed for Vec<u8> {}
impl SerializeSink for Vec<u8> {
    #[inline]
    fn push_byte(&mut self, byte: u8) -> io::Result<()> {
        self.push(byte);
        Ok(())
    }
    #[inline]
    fn write_bytes(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.extend_from_slice(bytes);
        Ok(())
    }
    #[inline]
    fn reserve_bytes(&mut self, additional: usize) {
        self.reserve(additional);
    }
}

/// A native streaming sink. Each operation propagates its writer error before
/// serialization continues; short and interrupted writes use `write_all`.
#[cfg(feature = "std")]
pub struct WriterSink<W> {
    writer: W,
}
#[cfg(feature = "std")]
impl<W> WriterSink<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }
    pub fn into_inner(self) -> W {
        self.writer
    }
}
#[cfg(feature = "std")]
impl<W: io::Write> sealed::Sealed for WriterSink<W> {}
#[cfg(feature = "std")]
impl<W: io::Write> SerializeSink for WriterSink<W> {
    #[inline]
    fn push_byte(&mut self, byte: u8) -> io::Result<()> {
        self.writer.write_all(&[byte])
    }
    #[inline]
    fn write_bytes(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.writer.write_all(bytes)
    }
}

pub(crate) struct InfallibleSink<'a, S>(pub &'a mut S);
impl<S: JsonSink> sealed::Sealed for InfallibleSink<'_, S> {}
impl<S: JsonSink> SerializeSink for InfallibleSink<'_, S> {
    #[inline]
    fn push_byte(&mut self, byte: u8) -> io::Result<()> {
        self.0.push(byte);
        Ok(())
    }
    #[inline]
    fn write_bytes(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.0.extend(bytes);
        Ok(())
    }
    #[inline]
    fn reserve_bytes(&mut self, additional: usize) {
        self.0.reserve(additional);
    }
}
