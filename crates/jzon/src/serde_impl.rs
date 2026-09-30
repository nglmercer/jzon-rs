//! Serde-compatible JSON serializer/deserializer backed by jzon's SIMD string
//! escaping and zero-copy scanner.
//!
//! Uses existing Serde traits. This native engine has its own errors and
//! incremental readers and streaming writers; assess its documented contract separately
//! from the delegated compatibility facade.
//! Available as `jzon::from_str` / `jzon::to_string` with the `serde` cargo
//! feature, or through the standalone [`jzon-rs-serde`](https://crates.io/crates/jzon-rs-serde)
//! crate which re-exports this module.
//!
//! # Usage
//!
//! ```rust
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize, Debug, PartialEq)]
//! struct Point { x: f64, y: f64 }
//!
//! let p = Point { x: 1.0, y: 2.0 };
//! let json = jzon::to_string(&p).unwrap();
//! let p2: Point = jzon::from_str(&json).unwrap();
//! assert_eq!(p, p2);
//! ```

use crate::__private::*;
#[cfg(feature = "std")]
use serde::de::DeserializeOwned;
use serde::de::{self as de_trait, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor};
use serde::ser::{
    self as ser_trait, Serialize, SerializeMap, SerializeSeq, SerializeStruct,
    SerializeStructVariant, SerializeTuple, SerializeTupleStruct, SerializeTupleVariant,
};

use crate::scanner::DecodedStr;
pub use crate::ser::SerializeSink;
#[cfg(feature = "std")]
pub use crate::ser::WriterSink;
use crate::ser::{
    try_write_escaped_key_sink as write_escaped_key,
    try_write_escaped_str_sink as write_escaped_str,
};
use crate::Scanner;
#[cfg(feature = "std")]
mod bridge;
#[cfg(feature = "std")]
mod reader;
#[cfg(feature = "std")]
pub use reader::{ReaderDeserializer, ReaderStream};

/// Classification of a native error, independent of upstream types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Io,
    Syntax,
    Data,
    Eof,
}

#[derive(Debug)]
pub enum Error {
    /// Located parse/visitor error. Unlocated serializer errors use the other variants.
    Positioned {
        source: Box<Error>,
        line: usize,
        column: usize,
    },
    Custom(String),
    InvalidUtf8,
    #[cfg(feature = "std")]
    Io(std::io::Error),
    Scanner(crate::Error),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Positioned {
                source,
                line,
                column,
            } => write!(f, "{source} at line {line} column {column}"),
            Error::Custom(m) => write!(f, "{m}"),
            Error::InvalidUtf8 => write!(f, "invalid UTF-8"),
            #[cfg(feature = "std")]
            Error::Io(e) => write!(f, "I/O error: {e}"),
            Error::Scanner(e) => write!(f, "JSON parse error: {e}"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            #[cfg(feature = "std")]
            Error::Io(e) => Some(e),
            Error::Scanner(e) => Some(e),
            Error::Positioned { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl Error {
    pub fn classify(&self) -> Category {
        match self {
            Self::Positioned { source, .. } => source.classify(),
            #[cfg(feature = "std")]
            Self::Io(_) => Category::Io,
            Self::Scanner(crate::Error::UnexpectedEof) => Category::Eof,
            Self::Scanner(_) | Self::InvalidUtf8 => Category::Syntax,
            Self::Custom(_) => Category::Data,
        }
    }
    pub fn line(&self) -> usize {
        match self {
            Self::Positioned { line, .. } => *line,
            _ => 0,
        }
    }
    pub fn column(&self) -> usize {
        match self {
            Self::Positioned { column, .. } => *column,
            _ => 0,
        }
    }
    pub fn is_io(&self) -> bool {
        self.classify() == Category::Io
    }
    pub fn is_syntax(&self) -> bool {
        self.classify() == Category::Syntax
    }
    pub fn is_data(&self) -> bool {
        self.classify() == Category::Data
    }
    pub fn is_eof(&self) -> bool {
        self.classify() == Category::Eof
    }
    pub fn cause(&self) -> &Self {
        match self {
            Self::Positioned { source, .. } => source.cause(),
            _ => self,
        }
    }
    pub(crate) fn at(self, line: usize, column: usize) -> Self {
        if self.line() != 0 {
            self
        } else {
            Self::Positioned {
                source: Box::new(self),
                line,
                column,
            }
        }
    }
}

impl ser_trait::Error for Error {
    fn custom<T: core::fmt::Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}

impl de_trait::Error for Error {
    fn custom<T: core::fmt::Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}

#[cfg(feature = "std")]
impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<crate::Error> for Error {
    fn from(e: crate::Error) -> Self {
        Error::Scanner(e)
    }
}

#[inline]
fn write_u128<S: SerializeSink>(v: u128, w: &mut S) -> Result<(), Error> {
    w.write_bytes(itoa::Buffer::new().format(v).as_bytes())?;
    Ok(())
}

#[inline]
fn write_i128<S: SerializeSink>(v: i128, w: &mut S) -> Result<(), Error> {
    w.write_bytes(itoa::Buffer::new().format(v).as_bytes())?;
    Ok(())
}

#[inline]
fn write_u64<S: SerializeSink>(v: u64, w: &mut S) -> Result<(), Error> {
    w.write_bytes(itoa::Buffer::new().format(v).as_bytes())?;
    Ok(())
}

#[inline]
fn write_i64<S: SerializeSink>(v: i64, w: &mut S) -> Result<(), Error> {
    w.write_bytes(itoa::Buffer::new().format(v).as_bytes())?;
    Ok(())
}

// ── float helpers ─────────────────────────────────────────────────────────────
// Native Serde output uses the same float formatter as the pinned upstream
// release. Mode A retains its separately selectable formatter policy.
#[inline]
fn serialize_float64<S: SerializeSink>(v: f64, w: &mut S) -> Result<(), Error> {
    if !v.is_finite() {
        w.write_bytes(b"null")?;
        return Ok(());
    }
    let mut buffer = zmij::Buffer::new();
    w.write_bytes(buffer.format_finite(v).as_bytes())?;
    Ok(())
}

#[inline]
fn serialize_float32<S: SerializeSink>(v: f32, w: &mut S) -> Result<(), Error> {
    if !v.is_finite() {
        w.write_bytes(b"null")?;
        return Ok(());
    }
    let mut buffer = zmij::Buffer::new();
    w.write_bytes(buffer.format_finite(v).as_bytes())?;
    Ok(())
}

pub struct Serializer<S: SerializeSink = Vec<u8>> {
    output: S,
}

impl Default for Serializer {
    fn default() -> Self {
        Self::new()
    }
}
impl Serializer {
    /// Fresh reusable byte serializer with a modest initial capacity.
    pub fn new() -> Self {
        Self::with_capacity(128)
    }
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            output: Vec::with_capacity(capacity),
        }
    }
    /// Reuse an owned Vec; subsequent serialization appends to its existing bytes.
    pub fn from_vec(output: Vec<u8>) -> Self {
        Self { output }
    }
    /// Reset output length, retaining capacity. Does not replay serialization.
    pub fn clear(&mut self) {
        self.output.clear();
    }
    pub fn buffer(&self) -> &[u8] {
        &self.output
    }
    pub fn capacity(&self) -> usize {
        self.output.capacity()
    }
}
#[cfg(feature = "std")]
impl<W: std::io::Write> Serializer<WriterSink<W>> {
    pub fn from_writer(writer: W) -> Self {
        Self {
            output: WriterSink::new(writer),
        }
    }
}
impl<S: SerializeSink> Serializer<S> {
    /// Append one value. Errors and panics preserve already emitted bytes.
    pub fn serialize<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        value.serialize(&mut *self)
    }
    pub fn into_inner(self) -> S {
        self.output
    }
}

pub fn to_string<T: Serialize + ?Sized>(v: &T) -> Result<String, Error> {
    let mut ser = Serializer {
        output: Vec::with_capacity(128),
    };
    v.serialize(&mut ser)?;
    String::from_utf8(ser.output).map_err(|_| Error::InvalidUtf8)
}

pub fn to_bytes<T: Serialize + ?Sized>(v: &T) -> Result<Vec<u8>, Error> {
    let mut ser = Serializer {
        output: Vec::with_capacity(128),
    };
    v.serialize(&mut ser)?;
    Ok(ser.output)
}

/// Append serialized JSON directly to an existing buffer. Call `clear()`
/// first for reset semantics. Errors retain partial output; unwinding restores
/// ownership of the buffer. No intermediate allocation or copy is performed.
pub fn to_bytes_in<T: Serialize + ?Sized>(v: &T, output: &mut Vec<u8>) -> Result<(), Error> {
    struct Restore<'a> {
        serializer: Serializer,
        output: &'a mut Vec<u8>,
    }
    impl Drop for Restore<'_> {
        fn drop(&mut self) {
            *self.output = core::mem::take(&mut self.serializer.output);
        }
    }
    let serializer = Serializer {
        output: core::mem::take(output),
    };
    let mut restore = Restore { serializer, output };
    v.serialize(&mut restore.serializer)
}

/// Stream native JSON directly to the writer, preserving partial output and
/// propagating each I/O failure before later user callbacks.
#[cfg(feature = "std")]
pub fn to_writer<W: std::io::Write, T: Serialize + ?Sized>(w: W, v: &T) -> Result<(), Error> {
    let mut serializer = Serializer::from_writer(w);
    v.serialize(&mut serializer)
}

/// Explicit buffered writer variant: completes user serialization before any
/// I/O, retaining the historical native helper's memory/error-ordering contract.
#[cfg(feature = "std")]
pub fn to_writer_buffered<W: std::io::Write, T: Serialize + ?Sized>(
    mut w: W,
    v: &T,
) -> Result<(), Error> {
    let bytes = to_bytes(v)?;
    w.write_all(&bytes)?;
    Ok(())
}

impl<'a, S: SerializeSink> ser_trait::Serializer for &'a mut Serializer<S> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = SeqSerializer<'a, S>;
    type SerializeTuple = SeqSerializer<'a, S>;
    type SerializeTupleStruct = SeqSerializer<'a, S>;
    type SerializeTupleVariant = SeqSerializer<'a, S>;
    type SerializeMap = MapSerializer<'a, S>;
    type SerializeStruct = StructSerializer<'a, S>;
    type SerializeStructVariant = StructVariantSerializer<'a, S>;

    #[inline]
    fn serialize_bool(self, v: bool) -> Result<(), Error> {
        self.output
            .write_bytes(if v { b"true" } else { b"false" })?;
        Ok(())
    }

    #[inline]
    fn serialize_i8(self, v: i8) -> Result<(), Error> {
        write_i64(v as i64, &mut self.output)?;
        Ok(())
    }
    #[inline]
    fn serialize_i16(self, v: i16) -> Result<(), Error> {
        write_i64(v as i64, &mut self.output)?;
        Ok(())
    }
    #[inline]
    fn serialize_i32(self, v: i32) -> Result<(), Error> {
        write_i64(v as i64, &mut self.output)?;
        Ok(())
    }
    #[inline]
    fn serialize_i64(self, v: i64) -> Result<(), Error> {
        write_i64(v, &mut self.output)?;
        Ok(())
    }

    #[inline]
    fn serialize_i128(self, v: i128) -> Result<(), Error> {
        write_i128(v, &mut self.output)
    }

    #[inline]
    fn serialize_u128(self, v: u128) -> Result<(), Error> {
        write_u128(v, &mut self.output)
    }

    #[inline]
    fn serialize_u8(self, v: u8) -> Result<(), Error> {
        write_u64(v as u64, &mut self.output)?;
        Ok(())
    }
    #[inline]
    fn serialize_u16(self, v: u16) -> Result<(), Error> {
        write_u64(v as u64, &mut self.output)?;
        Ok(())
    }
    #[inline]
    fn serialize_u32(self, v: u32) -> Result<(), Error> {
        write_u64(v as u64, &mut self.output)?;
        Ok(())
    }
    #[inline]
    fn serialize_u64(self, v: u64) -> Result<(), Error> {
        write_u64(v, &mut self.output)?;
        Ok(())
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> Result<(), Error> {
        serialize_float32(v, &mut self.output)?;
        Ok(())
    }
    #[inline]
    fn serialize_f64(self, v: f64) -> Result<(), Error> {
        serialize_float64(v, &mut self.output)?;
        Ok(())
    }

    #[inline]
    fn serialize_char(self, v: char) -> Result<(), Error> {
        let mut buf = [0u8; 4];
        let s = v.encode_utf8(&mut buf);
        write_escaped_str(s, &mut self.output)?;
        Ok(())
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result<(), Error> {
        write_escaped_str(v, &mut self.output)?;
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<(), Error> {
        self.output.push_byte(b'[')?;
        for (i, &b) in v.iter().enumerate() {
            if i > 0 {
                self.output.push_byte(b',')?;
            }
            write_u64(b as u64, &mut self.output)?;
        }
        self.output.push_byte(b']')?;
        Ok(())
    }

    #[inline]
    fn serialize_none(self) -> Result<(), Error> {
        self.output.write_bytes(b"null")?;
        Ok(())
    }

    #[inline]
    fn serialize_some<T: Serialize + ?Sized>(self, v: &T) -> Result<(), Error> {
        v.serialize(self)
    }

    #[inline]
    fn serialize_unit(self) -> Result<(), Error> {
        self.output.write_bytes(b"null")?;
        Ok(())
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<(), Error> {
        self.output.write_bytes(b"null")?;
        Ok(())
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<(), Error> {
        write_escaped_str(variant, &mut self.output)?;
        Ok(())
    }

    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        self.output.push_byte(b'{')?;
        write_escaped_str(variant, &mut self.output)?;
        self.output.push_byte(b':')?;
        value.serialize(&mut *self)?;
        self.output.push_byte(b'}')?;
        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<SeqSerializer<'a, S>, Error> {
        self.output.push_byte(b'[')?;
        Ok(SeqSerializer {
            ser: self,
            first: true,
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<SeqSerializer<'a, S>, Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<SeqSerializer<'a, S>, Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<SeqSerializer<'a, S>, Error> {
        self.output.push_byte(b'{')?;
        write_escaped_str(variant, &mut self.output)?;
        self.output.push_byte(b':')?;
        self.output.push_byte(b'[')?;
        Ok(SeqSerializer {
            ser: self,
            first: true,
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<MapSerializer<'a, S>, Error> {
        self.output.push_byte(b'{')?;
        Ok(MapSerializer {
            ser: self,
            first: true,
            variant_wrap: false,
        })
    }

    #[inline]
    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<StructSerializer<'a, S>, Error> {
        // serde_json token protocols: RawValue and arbitrary-precision
        // Number serialize as single-field structs whose payload must be
        // emitted raw (unquoted). Anything else is an ordinary struct.
        if name == RAW_VALUE_TOKEN {
            return Ok(StructSerializer::Raw(RawTokenStructSerializer {
                ser: self,
                token: RAW_VALUE_TOKEN,
                err_msg: "expected value",
            }));
        }
        if name == NUMBER_TOKEN {
            return Ok(StructSerializer::Raw(RawTokenStructSerializer {
                ser: self,
                token: NUMBER_TOKEN,
                err_msg: "invalid number",
            }));
        }
        let _ = len;
        self.output.push_byte(b'{')?;
        Ok(StructSerializer::Map(FieldsSerializer {
            ser: self,
            first: true,
        }))
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<StructVariantSerializer<'a, S>, Error> {
        self.output.push_byte(b'{')?;
        write_escaped_str(variant, &mut self.output)?;
        self.output.push_byte(b':')?;
        self.output.push_byte(b'{')?;
        Ok(StructVariantSerializer {
            fields: FieldsSerializer {
                ser: self,
                first: true,
            },
        })
    }

    fn is_human_readable(&self) -> bool {
        true
    }
}

pub struct SeqSerializer<'a, S: SerializeSink = Vec<u8>> {
    ser: &'a mut Serializer<S>,
    first: bool,
}

impl<'a, S: SerializeSink> SerializeSeq for SeqSerializer<'a, S> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        if !self.first {
            self.ser.output.push_byte(b',')?;
        }
        self.first = false;
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<(), Error> {
        self.ser.output.push_byte(b']')?;
        Ok(())
    }
}

macro_rules! delegate_to_seq {
    ($trait:ident, $method:ident) => {
        impl<'a, S: SerializeSink> $trait for SeqSerializer<'a, S> {
            type Ok = ();
            type Error = Error;
            fn $method<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
                SerializeSeq::serialize_element(self, value)
            }
            fn end(self) -> Result<(), Error> {
                SerializeSeq::end(self)
            }
        }
    };
}
delegate_to_seq!(SerializeTuple, serialize_element);
delegate_to_seq!(SerializeTupleStruct, serialize_field);

impl<'a, S: SerializeSink> SerializeTupleVariant for SeqSerializer<'a, S> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        SerializeSeq::serialize_element(self, value)
    }
    fn end(self) -> Result<(), Error> {
        self.ser.output.push_byte(b']')?;
        self.ser.output.push_byte(b'}')?;
        Ok(())
    }
}

/// Serializer for map keys: mirrors `serde_json` exactly — strings, numbers,
/// bools, chars and unit variants are stringified; anything structural is an
/// error (`collect_str` inherits serde's default, same as `serde_json`).
struct MapKeySerializer<'a, S: SerializeSink = Vec<u8>> {
    ser: &'a mut Serializer<S>,
}

#[inline]
fn key_must_be_a_string() -> Error {
    Error::Custom("key must be a string".to_owned())
}

#[inline]
fn float_key_must_be_finite() -> Error {
    Error::Custom("float key must be finite".to_owned())
}

macro_rules! quoted_int_key {
    ($method:ident, $ty:ty, $write:ident) => {
        fn $method(self, value: $ty) -> Result<(), Error> {
            self.ser.output.push_byte(b'"')?;
            $write(value as _, &mut self.ser.output)?;
            self.ser.output.push_byte(b'"')?;
            Ok(())
        }
    };
}

impl<'a, S: SerializeSink> ser_trait::Serializer for MapKeySerializer<'a, S> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = ser_trait::Impossible<(), Error>;
    type SerializeTuple = ser_trait::Impossible<(), Error>;
    type SerializeTupleStruct = ser_trait::Impossible<(), Error>;
    type SerializeTupleVariant = ser_trait::Impossible<(), Error>;
    type SerializeMap = ser_trait::Impossible<(), Error>;
    type SerializeStruct = ser_trait::Impossible<(), Error>;
    type SerializeStructVariant = ser_trait::Impossible<(), Error>;

    fn serialize_bool(self, value: bool) -> Result<(), Error> {
        self.ser.output.push_byte(b'"')?;
        self.ser
            .output
            .write_bytes(if value { b"true" } else { b"false" })?;
        self.ser.output.push_byte(b'"')?;
        Ok(())
    }

    quoted_int_key!(serialize_i8, i8, write_i64);
    quoted_int_key!(serialize_i16, i16, write_i64);
    quoted_int_key!(serialize_i32, i32, write_i64);
    quoted_int_key!(serialize_i64, i64, write_i64);
    quoted_int_key!(serialize_u8, u8, write_u64);
    quoted_int_key!(serialize_u16, u16, write_u64);
    quoted_int_key!(serialize_u32, u32, write_u64);
    quoted_int_key!(serialize_u64, u64, write_u64);
    quoted_int_key!(serialize_i128, i128, write_i128);
    quoted_int_key!(serialize_u128, u128, write_u128);

    fn serialize_f32(self, value: f32) -> Result<(), Error> {
        if !value.is_finite() {
            return Err(float_key_must_be_finite());
        }
        self.ser.output.push_byte(b'"')?;
        serialize_float32(value, &mut self.ser.output)?;
        self.ser.output.push_byte(b'"')?;
        Ok(())
    }

    fn serialize_f64(self, value: f64) -> Result<(), Error> {
        if !value.is_finite() {
            return Err(float_key_must_be_finite());
        }
        self.ser.output.push_byte(b'"')?;
        serialize_float64(value, &mut self.ser.output)?;
        self.ser.output.push_byte(b'"')?;
        Ok(())
    }

    fn serialize_char(self, value: char) -> Result<(), Error> {
        let mut buf = [0u8; 4];
        let s = value.encode_utf8(&mut buf);
        write_escaped_str(s, &mut self.ser.output)?;
        Ok(())
    }

    fn serialize_str(self, value: &str) -> Result<(), Error> {
        write_escaped_str(value, &mut self.ser.output)?;
        Ok(())
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<(), Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_none(self) -> Result<(), Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<(), Error> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<(), Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<(), Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<(), Error> {
        write_escaped_str(variant, &mut self.ser.output)?;
        Ok(())
    }

    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<(), Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Error> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Error> {
        Err(key_must_be_a_string())
    }

    fn is_human_readable(&self) -> bool {
        true
    }
}

pub struct MapSerializer<'a, S: SerializeSink = Vec<u8>> {
    ser: &'a mut Serializer<S>,
    first: bool,
    /// True when this is a struct-variant that needs an extra closing `}`.
    variant_wrap: bool,
}

impl<'a, S: SerializeSink> SerializeMap for MapSerializer<'a, S> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<(), Error> {
        if !self.first {
            self.ser.output.push_byte(b',')?;
        }
        self.first = false;
        key.serialize(MapKeySerializer {
            ser: &mut *self.ser,
        })?;
        self.ser.output.push_byte(b':')?;
        Ok(())
    }

    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<(), Error> {
        self.ser.output.push_byte(b'}')?;
        if self.variant_wrap {
            self.ser.output.push_byte(b'}')?;
        }
        Ok(())
    }
}

impl<'a, S: SerializeSink> SerializeStruct for MapSerializer<'a, S> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        if !self.first {
            self.ser.output.push_byte(b',')?;
        }
        self.first = false;
        write_escaped_str(key, &mut self.ser.output)?;
        self.ser.output.push_byte(b':')?;
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<(), Error> {
        self.ser.output.push_byte(b'}')?;
        if self.variant_wrap {
            self.ser.output.push_byte(b'}')?;
        }
        Ok(())
    }
}

/// Ordinary struct fields avoid generic map-key dispatch and enum-wrapper state.
pub struct FieldsSerializer<'a, S: SerializeSink = Vec<u8>> {
    ser: &'a mut Serializer<S>,
    first: bool,
}
impl<S: SerializeSink> SerializeStruct for FieldsSerializer<'_, S> {
    type Ok = ();
    type Error = Error;
    #[inline]
    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        if !self.first {
            self.ser.output.push_byte(b',')?;
        }
        self.first = false;
        write_escaped_key(key, &mut self.ser.output)?;
        value.serialize(&mut *self.ser)
    }
    #[inline]
    fn end(self) -> Result<(), Error> {
        self.ser.output.push_byte(b'}')?;
        Ok(())
    }
}

pub struct StructVariantSerializer<'a, S: SerializeSink = Vec<u8>> {
    fields: FieldsSerializer<'a, S>,
}
impl<S: SerializeSink> SerializeStructVariant for StructVariantSerializer<'_, S> {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        self.fields.serialize_field(key, value)
    }
    fn end(self) -> Result<(), Error> {
        self.fields.ser.output.write_bytes(b"}}")?;
        Ok(())
    }
}

/// Token names of serde_json's private raw-passthrough protocols.
const RAW_VALUE_TOKEN: &str = "$serde_json::private::RawValue";
const NUMBER_TOKEN: &str = "$serde_json::private::Number";

/// Struct serializer dispatch: ordinary structs serialize as maps, while
/// serde_json token structs emit their payload raw (unquoted).
pub enum StructSerializer<'a, S: SerializeSink = Vec<u8>> {
    Map(FieldsSerializer<'a, S>),
    Raw(RawTokenStructSerializer<'a, S>),
}

impl<'a, S: SerializeSink> SerializeStruct for StructSerializer<'a, S> {
    type Ok = ();
    type Error = Error;

    #[inline(always)]
    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        match self {
            StructSerializer::Map(m) => SerializeStruct::serialize_field(m, key, value),
            StructSerializer::Raw(r) => SerializeStruct::serialize_field(r, key, value),
        }
    }

    #[inline(always)]
    fn end(self) -> Result<(), Error> {
        match self {
            StructSerializer::Map(m) => SerializeStruct::end(m),
            StructSerializer::Raw(r) => SerializeStruct::end(r),
        }
    }
}

pub struct RawTokenStructSerializer<'a, S: SerializeSink = Vec<u8>> {
    ser: &'a mut Serializer<S>,
    token: &'static str,
    err_msg: &'static str,
}

impl<'a, S: SerializeSink> SerializeStruct for RawTokenStructSerializer<'a, S> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        if key != self.token {
            return Err(Error::Custom(self.err_msg.to_owned()));
        }
        value.serialize(RawStrEmitter {
            ser: &mut *self.ser,
            err_msg: self.err_msg,
        })
    }

    fn end(self) -> Result<(), Error> {
        Ok(())
    }
}

/// Emits strings raw (unquoted); every other type is a protocol violation.
struct RawStrEmitter<'a, S: SerializeSink = Vec<u8>> {
    ser: &'a mut Serializer<S>,
    err_msg: &'static str,
}

macro_rules! raw_err {
    ($method:ident, $($arg:ident : $ty:ty),*) => {
        fn $method(self, $(_: $ty),*) -> Result<(), Error> {
            Err(Error::Custom(self.err_msg.to_owned()))
        }
    };
}

impl<'a, S: SerializeSink> ser_trait::Serializer for RawStrEmitter<'a, S> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = ser_trait::Impossible<(), Error>;
    type SerializeTuple = ser_trait::Impossible<(), Error>;
    type SerializeTupleStruct = ser_trait::Impossible<(), Error>;
    type SerializeTupleVariant = ser_trait::Impossible<(), Error>;
    type SerializeMap = ser_trait::Impossible<(), Error>;
    type SerializeStruct = ser_trait::Impossible<(), Error>;
    type SerializeStructVariant = ser_trait::Impossible<(), Error>;

    fn serialize_str(self, value: &str) -> Result<(), Error> {
        self.ser.output.write_bytes(value.as_bytes())?;
        Ok(())
    }

    raw_err!(serialize_bool, _v: bool);
    raw_err!(serialize_i8, _v: i8);
    raw_err!(serialize_i16, _v: i16);
    raw_err!(serialize_i32, _v: i32);
    raw_err!(serialize_i64, _v: i64);
    raw_err!(serialize_i128, _v: i128);
    raw_err!(serialize_u8, _v: u8);
    raw_err!(serialize_u16, _v: u16);
    raw_err!(serialize_u32, _v: u32);
    raw_err!(serialize_u64, _v: u64);
    raw_err!(serialize_u128, _v: u128);
    raw_err!(serialize_f32, _v: f32);
    raw_err!(serialize_f64, _v: f64);
    raw_err!(serialize_char, _v: char);
    raw_err!(serialize_bytes, _v: &[u8]);
    raw_err!(serialize_none,);
    raw_err!(serialize_unit,);
    raw_err!(serialize_unit_struct, _name: &'static str);

    fn serialize_some<T: Serialize + ?Sized>(self, _value: &T) -> Result<(), Error> {
        Err(Error::Custom(self.err_msg.to_owned()))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<(), Error> {
        Err(Error::Custom(self.err_msg.to_owned()))
    }

    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<(), Error> {
        Err(Error::Custom(self.err_msg.to_owned()))
    }

    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<(), Error> {
        Err(Error::Custom(self.err_msg.to_owned()))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Error> {
        Err(Error::Custom(self.err_msg.to_owned()))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Error> {
        Err(Error::Custom(self.err_msg.to_owned()))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Error> {
        Err(Error::Custom(self.err_msg.to_owned()))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Error> {
        Err(Error::Custom(self.err_msg.to_owned()))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Error> {
        Err(Error::Custom(self.err_msg.to_owned()))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Error> {
        Err(Error::Custom(self.err_msg.to_owned()))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Error> {
        Err(Error::Custom(self.err_msg.to_owned()))
    }

    fn is_human_readable(&self) -> bool {
        true
    }
}

impl<'a, S: SerializeSink> SerializeStructVariant for MapSerializer<'a, S> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        SerializeStruct::serialize_field(self, key, value)
    }

    fn end(self) -> Result<(), Error> {
        SerializeStruct::end(self)
    }
}

pub struct Deserializer<'de> {
    scanner: Scanner<'de>,
    scratch: Vec<u8>,
    remaining_depth: u8,
    depth_limit_disabled: bool,
}

impl<'de> Deserializer<'de> {
    /// Construct a native deserializer borrowing UTF-8 input.
    #[allow(clippy::should_implement_trait)] // Borrowing constructor, mirrors upstream; FromStr cannot express this lifetime.
    pub fn from_str(input: &'de str) -> Self {
        Self {
            scanner: Scanner::new_str(input),
            scratch: Vec::new(),
            remaining_depth: crate::scanner::MAX_DEPTH,
            depth_limit_disabled: false,
        }
    }

    /// Construct a native deserializer borrowing arbitrary input bytes.
    pub fn from_slice(input: &'de [u8]) -> Self {
        Self {
            scanner: Scanner::new(input),
            scratch: Vec::new(),
            remaining_depth: crate::scanner::MAX_DEPTH,
            depth_limit_disabled: false,
        }
    }

    #[inline]
    fn with_depth<T>(
        &mut self,
        operation: impl FnOnce(&mut Self) -> Result<T, Error>,
    ) -> Result<T, Error> {
        self.scanner.skip_whitespace();
        if self.depth_limit_disabled {
            return operation(self);
        }
        if self.remaining_depth <= 1 {
            return Err(Error::Scanner(crate::Error::RecursionLimit));
        }
        self.remaining_depth -= 1;
        // This private guard owns the exclusive borrow of the WHOLE parser.
        // Visitors only access it through that borrow, so there is no aliasing
        // of a scanner field with its enclosing deserializer. Drop is infallible.
        struct RestoreDepth<'a, 'de> {
            parser: &'a mut Deserializer<'de>,
        }
        impl Drop for RestoreDepth<'_, '_> {
            fn drop(&mut self) {
                self.parser.remaining_depth += 1;
            }
        }
        let guard = RestoreDepth { parser: self };
        operation(&mut *guard.parser)
    }

    #[inline]
    fn with_str<T>(
        &mut self,
        colon: bool,
        operation: impl for<'scratch> FnOnce(DecodedStr<'de, 'scratch>) -> Result<T, Error>,
    ) -> Result<T, Error> {
        // Keep typical decoded strings reusable, but release oversized storage
        // on success, error AND unwind. A transient reference cannot escape T.
        struct Scratch<'a>(&'a mut Vec<u8>);
        impl Drop for Scratch<'_> {
            fn drop(&mut self) {
                if self.0.capacity() > 64 * 1024 {
                    *self.0 = Vec::new();
                }
            }
        }
        let scratch = Scratch(&mut self.scratch);
        let value = self.scanner.read_str_with_scratch(scratch.0)?;
        if colon {
            self.scanner.skip_whitespace();
            self.scanner.expect_byte(b':')?;
        }
        operation(value)
    }

    fn run<T>(
        &mut self,
        operation: impl FnOnce(&mut Self) -> Result<T, Error>,
    ) -> Result<T, Error> {
        let result = operation(&mut *self);
        result.map_err(|error| self.position_error(error))
    }
    fn invalid_type<T, V: Visitor<'de>>(&mut self, visitor: &V) -> Result<T, Error> {
        use de_trait::Unexpected;
        let byte = self.scanner.peek_byte_after_ws()?;
        let unexpected = match byte {
            b'n' => {
                self.scanner.read_null()?;
                Unexpected::Unit
            }
            b't' | b'f' => Unexpected::Bool(self.scanner.read_bool()?),
            b'"' => {
                return self.with_str(false, |s| {
                    let text = match s {
                        DecodedStr::Borrowed(t) | DecodedStr::Transient(t) => t,
                    };
                    Err(de_trait::Error::invalid_type(
                        Unexpected::Str(text),
                        visitor,
                    ))
                })
            }
            b'-' | b'0'..=b'9' => {
                struct NumberError<'a, V>(&'a V);
                impl<'de, V: Visitor<'de>> Visitor<'de> for NumberError<'_, V> {
                    type Value = ();
                    fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                        self.0.expecting(f)
                    }
                    fn visit_i64<E: de_trait::Error>(self, v: i64) -> Result<(), E> {
                        Err(E::invalid_type(Unexpected::Signed(v), self.0))
                    }
                    fn visit_u64<E: de_trait::Error>(self, v: u64) -> Result<(), E> {
                        Err(E::invalid_type(Unexpected::Unsigned(v), self.0))
                    }
                    fn visit_f64<E: de_trait::Error>(self, v: f64) -> Result<(), E> {
                        Err(E::invalid_type(Unexpected::Float(v), self.0))
                    }
                }
                let bytes = self.scanner.read_number_bytes()?;
                return match visit_native_number(bytes, NumberError(visitor)) {
                    Err(e) => Err(e),
                    Ok(()) => unreachable!(),
                };
            }
            b'[' => Unexpected::Seq,
            b'{' => Unexpected::Map,
            _ => return Err(Error::Scanner(crate::Error::UnexpectedToken)),
        };
        let error: Error = de_trait::Error::invalid_type(unexpected, visitor);
        if matches!(byte, b'[' | b'{') {
            let (line, column) = self.scanner.location(self.scanner.pos());
            Err(error.at(line, column))
        } else {
            Err(error)
        }
    }
    fn position_error(&self, error: Error) -> Error {
        let offset = self
            .scanner
            .pos()
            .saturating_add(usize::from(error.is_syntax()));
        let (line, column) = self.scanner.location(offset);
        error.at(line, column)
    }

    /// Consume consecutive native JSON values without replaying callbacks.
    /// Borrowed outputs retain the original input lifetime.
    #[allow(clippy::should_implement_trait)] // Output type/lifetime is selected by this constructor, not IntoIterator.
    pub fn into_iter<T: serde::Deserialize<'de>>(self) -> StreamDeserializer<'de, T> {
        StreamDeserializer {
            de: self,
            failed: false,
            marker: core::marker::PhantomData,
        }
    }
    pub fn byte_offset(&self) -> usize {
        self.scanner.pos()
    }

    /// Release reusable decoding storage without changing input position.
    pub fn clear_scratch(&mut self) {
        self.scratch = Vec::new();
    }
    /// Retained native transient decoding capacity (not total allocations).
    pub fn scratch_capacity(&self) -> usize {
        self.scratch.capacity()
    }

    /// Require end of input after one value.
    pub fn end(&mut self) -> Result<(), Error> {
        self.scanner
            .expect_eof()
            .map_err(|e| self.position_error(Error::Scanner(e)))
    }

    /// Explicit opt-out; enabling the feature leaves the default limit intact.
    #[cfg(feature = "unbounded_depth")]
    pub fn disable_recursion_limit(&mut self) {
        self.depth_limit_disabled = true;
        self.scanner.disable_recursion_limit();
    }

    /// Borrow the underlying [`Scanner`](crate::Scanner) statistics.
    ///
    /// Available with the `stats` feature.
    #[cfg(feature = "stats")]
    pub fn stats(&self) -> &crate::stats::ScannerStats {
        &self.scanner.stats
    }

    /// Capture the next JSON value verbatim for the
    /// `$serde_json::private::RawValue` token protocol: skip one full value
    /// (validating it) while borrowing its exact input slice, then present it
    /// as a single-entry map `{ TOKEN: raw }`, mirroring serde_json's
    /// `deserialize_raw_value` + `BorrowedRawDeserializer`.
    fn deserialize_raw_token<V: Visitor<'de>>(&mut self, visitor: V) -> Result<V::Value, Error> {
        self.scanner.skip_whitespace();
        let start = self.scanner.pos();
        let rest = self.scanner.remaining_input();
        self.scanner.skip_value_serde()?;
        let raw = core::str::from_utf8(&rest[..self.scanner.pos() - start])
            .map_err(|_| Error::InvalidUtf8)?;
        visitor.visit_map(RawTokenMapAccess {
            token: RAW_VALUE_TOKEN,
            raw: Some(raw),
        })
    }
}

/// Fused native slice/string stream. Byte offsets describe actual parser consumption.
pub struct StreamDeserializer<'de, T> {
    de: Deserializer<'de>,
    failed: bool,
    marker: core::marker::PhantomData<T>,
}
impl<'de, T: serde::Deserialize<'de>> StreamDeserializer<'de, T> {
    pub fn byte_offset(&self) -> usize {
        self.de.byte_offset()
    }
    pub fn into_inner(self) -> Deserializer<'de> {
        self.de
    }
}
impl<'de, T: serde::Deserialize<'de>> Iterator for StreamDeserializer<'de, T> {
    type Item = Result<T, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.failed {
            return None;
        }
        self.de.scanner.skip_whitespace();
        if self.de.scanner.remaining_input().is_empty() {
            return None;
        }
        let self_delineated = matches!(self.de.scanner.peek_byte(), Ok(b'[' | b'{' | b'"'));
        let result = T::deserialize(&mut self.de)
            .and_then(|value| {
                if !self_delineated
                    && !matches!(
                        self.de.scanner.peek_byte(),
                        Err(_)
                            | Ok(b' '
                                | b'\n'
                                | b'\r'
                                | b'\t'
                                | b'"'
                                | b'['
                                | b']'
                                | b'{'
                                | b'}'
                                | b','
                                | b':')
                    )
                {
                    return Err(Error::Scanner(crate::Error::UnexpectedToken));
                }
                Ok(value)
            })
            .map_err(|error| self.de.position_error(error));
        if result.is_err() {
            self.failed = true;
        }
        Some(result)
    }
}
impl<'de, T: serde::Deserialize<'de>> core::iter::FusedIterator for StreamDeserializer<'de, T> {}

/// Single-entry `{ TOKEN: raw }` map for the RawValue token protocol.
struct RawTokenMapAccess<'de> {
    token: &'static str,
    raw: Option<&'de str>,
}

impl<'de> MapAccess<'de> for RawTokenMapAccess<'de> {
    type Error = Error;

    fn next_key_seed<K: de_trait::DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Error> {
        if self.raw.is_none() {
            return Ok(None);
        }
        seed.deserialize(de_trait::value::BorrowedStrDeserializer::new(self.token))
            .map(Some)
    }

    fn next_value_seed<V: de_trait::DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Error> {
        seed.deserialize(de_trait::value::BorrowedStrDeserializer::new(
            self.raw.take().expect("next_value without next_key"),
        ))
    }
}

pub fn from_str<'de, T: serde::Deserialize<'de>>(s: &'de str) -> Result<T, Error> {
    let mut de = Deserializer::from_str(s);
    let value = T::deserialize(&mut de).map_err(|e| de.position_error(e))?;
    // ECMA-404: a JSON text is exactly one value — reject trailing content.
    de.end()?;
    Ok(value)
}

pub fn from_slice<'de, T: serde::Deserialize<'de>>(b: &'de [u8]) -> Result<T, Error> {
    let mut de = Deserializer::from_slice(b);
    let value = T::deserialize(&mut de).map_err(|e| de.position_error(e))?;
    de.end()?;
    Ok(value)
}

#[cfg(feature = "std")]
fn from_reader_inner<'de, T: serde::Deserialize<'de>>(b: &'de [u8]) -> Result<T, Error> {
    from_slice(b)
}

#[cfg(feature = "std")]
pub fn from_reader_buffered<R: std::io::Read, T: DeserializeOwned>(mut r: R) -> Result<T, Error> {
    let mut buf = Vec::new();
    r.read_to_end(&mut buf).map_err(Error::Io)?;
    from_reader_inner(&buf)
}

/// Incremental native reader. Callbacks run once; containers are not buffered.
#[cfg(feature = "std")]
pub fn from_reader<R: std::io::Read, T: DeserializeOwned>(reader: R) -> Result<T, Error> {
    let mut de = ReaderDeserializer::new(reader);
    let value = T::deserialize(&mut de).map_err(|e| de.located(e))?;
    de.end()?;
    Ok(value)
}

/// Deserialize from `&str`, also returning the underlying [`Scanner`](crate::Scanner)
/// statistics (zero-copy borrows, heap allocations, bytes scanned).
///
/// Available with the `stats` feature.
#[cfg(feature = "stats")]
pub fn from_str_with_stats<'de, T: serde::Deserialize<'de>>(
    s: &'de str,
) -> Result<(T, crate::stats::ScannerStats), Error> {
    let mut de = Deserializer::from_str(s);
    let value = T::deserialize(&mut de).map_err(|e| de.position_error(e))?;
    de.end()?;
    Ok((value, de.scanner.stats))
}

/// Deserialize from a byte slice, also returning [`Scanner`](crate::Scanner) statistics.
///
/// Available with the `stats` feature.
#[cfg(feature = "stats")]
pub fn from_slice_with_stats<'de, T: serde::Deserialize<'de>>(
    b: &'de [u8],
) -> Result<(T, crate::stats::ScannerStats), Error> {
    let mut de = Deserializer::from_slice(b);
    let value = T::deserialize(&mut de).map_err(|e| de.position_error(e))?;
    de.end()?;
    Ok((value, de.scanner.stats))
}

/// Deserialize from a reader, also returning [`Scanner`](crate::Scanner) statistics.
///
/// Available with the `stats` feature.
#[cfg(feature = "stats")]
#[cfg(feature = "std")]
pub fn from_reader_buffered_with_stats<R: std::io::Read, T: DeserializeOwned>(
    mut r: R,
) -> Result<(T, crate::stats::ScannerStats), Error> {
    let mut buf = Vec::new();
    r.read_to_end(&mut buf).map_err(Error::Io)?;
    from_slice_with_stats(&buf)
}

/// Incremental reader statistics count scalar native scanner events; no token-local borrow is reported as zero-copy output.
#[cfg(all(feature = "std", feature = "stats"))]
pub fn from_reader_with_stats<R: std::io::Read, T: DeserializeOwned>(
    reader: R,
) -> Result<(T, crate::stats::ScannerStats), Error> {
    let mut de = ReaderDeserializer::new(reader);
    let value = T::deserialize(&mut de).map_err(|e| de.located(e))?;
    de.end()?;
    Ok((value, de.stats().clone()))
}

macro_rules! deserialize_numeric {
    ($method:ident) => {
        fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
            let result = self.deserialize_typed_number(visitor);
            result.map_err(|error| self.position_error(error))
        }
    };
}

impl<'de> de_trait::Deserializer<'de> for &mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.run(|parser| {
            let b = parser.scanner.peek_byte_after_ws()?;
            match b {
                b'"' => parser.deserialize_str(visitor),
                b'{' => parser.deserialize_map(visitor),
                b'[' => parser.deserialize_seq(visitor),
                b't' | b'f' => parser.deserialize_bool(visitor),
                b'n' => {
                    parser.scanner.read_null()?;
                    visitor.visit_unit()
                }
                b'-' | b'0'..=b'9' => parser.deserialize_number(visitor),
                _ => Err(Error::Scanner(crate::Error::UnexpectedToken)),
            }
        })
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.run(|parser| {
            if !matches!(parser.scanner.peek_byte_after_ws()?, b't' | b'f') {
                return parser.invalid_type(&visitor);
            }
            let v = parser.scanner.read_bool()?;
            visitor.visit_bool(v)
        })
    }

    deserialize_numeric!(deserialize_i8);
    deserialize_numeric!(deserialize_i16);
    deserialize_numeric!(deserialize_i32);
    deserialize_numeric!(deserialize_i64);
    deserialize_numeric!(deserialize_u8);
    deserialize_numeric!(deserialize_u16);
    deserialize_numeric!(deserialize_u32);
    deserialize_numeric!(deserialize_u64);

    fn deserialize_i128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.run(|parser| {
            if !matches!(parser.scanner.peek_byte_after_ws()?, b'-' | b'0'..=b'9') {
                return parser.invalid_type(&visitor);
            }
            let bytes = parser.scanner.read_number_bytes()?;
            visitor.visit_i128(parse_i128(bytes)?)
        })
    }

    fn deserialize_u128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.run(|parser| {
            if !matches!(parser.scanner.peek_byte_after_ws()?, b'-' | b'0'..=b'9') {
                return parser.invalid_type(&visitor);
            }
            let bytes = parser.scanner.read_number_bytes()?;
            visitor.visit_u128(parse_u128(bytes)?)
        })
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.run(|parser| {
            #[cfg(feature = "float_roundtrip")]
            {
                if !matches!(parser.scanner.peek_byte_after_ws()?, b'-' | b'0'..=b'9') {
                    return parser.invalid_type(&visitor);
                }
                let bytes = parser.scanner.read_number_bytes()?;
                if !bytes.iter().any(|&b| matches!(b, b'.' | b'e' | b'E')) {
                    if bytes[0] == b'-' {
                        if let Ok(n) = parse_i64(bytes) {
                            if n != 0 {
                                return visitor.visit_i64(n);
                            }
                        }
                    } else if let Ok(n) = parse_u64(bytes) {
                        return visitor.visit_u64(n);
                    }
                }
                let value: f32 = fast_float2::parse(bytes)
                    .map_err(|_| Error::Scanner(crate::Error::InvalidNumber))?;
                if value.is_infinite() {
                    return Err(Error::Scanner(crate::Error::InvalidNumber));
                }
                visitor.visit_f64(value as f64)
            }
            #[cfg(not(feature = "float_roundtrip"))]
            parser.deserialize_typed_number(visitor)
        })
    }
    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.run(|parser| parser.deserialize_typed_number(visitor))
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.run(|parser| {
            // The standard char visitor validates exactly one scalar. Using the
            // string entry point also preserves upstream custom-visitor callbacks.
            parser.deserialize_str(visitor)
        })
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.run(|parser| {
            if parser.scanner.peek_byte_after_ws()? != b'"' {
                return parser.invalid_type(&visitor);
            }
            parser.with_str(false, |s| match s {
                DecodedStr::Borrowed(b) => visitor.visit_borrowed_str(b),
                DecodedStr::Transient(s) => visitor.visit_str(s),
            })
        })
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.run(|parser| parser.deserialize_str(visitor))
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.run(|parser| {
            if parser.scanner.peek_byte_after_ws()? == b'"' {
                match parser.scanner.read_byte_str()? {
                    alloc::borrow::Cow::Borrowed(bytes) => visitor.visit_borrowed_bytes(bytes),
                    alloc::borrow::Cow::Owned(bytes) => visitor.visit_bytes(&bytes),
                }
            } else {
                parser.deserialize_seq(visitor)
            }
        })
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.run(|parser| parser.deserialize_bytes(visitor))
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.run(|parser| {
            if parser.scanner.peek_null() {
                parser.scanner.read_null()?;
                visitor.visit_none()
            } else {
                visitor.visit_some(&mut *parser)
            }
        })
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.run(|parser| {
            if parser.scanner.peek_byte_after_ws()? != b'n' {
                return parser.invalid_type(&visitor);
            }
            parser.scanner.read_null()?;
            visitor.visit_unit()
        })
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.run(|parser| parser.deserialize_unit(visitor))
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.run(|parser| {
            // serde_json raw-value protocol: capture one full JSON value verbatim
            // and present it as a single-entry map { TOKEN: raw }.
            if name == RAW_VALUE_TOKEN {
                return parser.deserialize_raw_token(visitor);
            }
            visitor.visit_newtype_struct(&mut *parser)
        })
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.run(|parser| {
            if parser.scanner.peek_byte_after_ws()? != b'[' {
                return parser.invalid_type(&visitor);
            }
            parser.with_depth(|parser| {
                parser.scanner.skip_whitespace();
                parser.scanner.expect_byte(b'[')?;
                let mut access = JsonSeqAccess {
                    de: parser,
                    first: true,
                    done: false,
                };
                let value = visitor.visit_seq(&mut access)?;
                if !access.done {
                    access.de.scanner.skip_whitespace();
                    access.de.scanner.expect_byte(b']')?;
                }
                Ok(value)
            })
        })
    }

    fn deserialize_tuple<V: Visitor<'de>>(
        self,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.run(|parser| parser.deserialize_seq(visitor))
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.run(|parser| parser.deserialize_seq(visitor))
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.run(|parser| {
            if parser.scanner.peek_byte_after_ws()? != b'{' {
                return parser.invalid_type(&visitor);
            }
            parser.with_depth(|parser| {
                parser.scanner.skip_whitespace();
                parser.scanner.expect_byte(b'{')?;
                let mut access = JsonMapAccess {
                    de: parser,
                    first: true,
                    pending_value: false,
                    done: false,
                };
                let value = visitor.visit_map(&mut access)?;
                if access.pending_value {
                    return Err(Error::Custom("map value not consumed".into()));
                }
                if !access.done {
                    access.de.scanner.skip_whitespace();
                    access.de.scanner.expect_byte(b'}')?;
                }
                Ok(value)
            })
        })
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.run(|parser| {
            if parser.scanner.peek_byte_after_ws()? == b'[' {
                parser.deserialize_seq(visitor)
            } else {
                parser.deserialize_map(visitor)
            }
        })
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.run(|parser| {
            let b = parser.scanner.peek_byte_after_ws()?;
            if b == b'"' {
                parser.with_str(false, |s| visitor.visit_enum(StrDeserializer::new(s)))
            } else if b == b'{' {
                parser.with_depth(|parser| {
                    parser.scanner.skip_whitespace();
                    parser.scanner.expect_byte(b'{')?;
                    let value = visitor.visit_enum(JsonEnumAccess { de: parser })?;
                    parser.scanner.skip_whitespace();
                    parser.scanner.expect_byte(b'}')?;
                    Ok(value)
                })
            } else {
                parser.invalid_type(&visitor)
            }
        })
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.run(|parser| parser.deserialize_str(visitor))
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.run(|parser| {
            parser.scanner.skip_value_serde()?;
            visitor.visit_unit()
        })
    }

    fn is_human_readable(&self) -> bool {
        true
    }
}

fn invalid_number<T>() -> Result<T, Error> {
    Err(Error::Scanner(crate::Error::InvalidNumber))
}

fn parse_u64(bytes: &[u8]) -> Result<u64, Error> {
    let mut n = 0u64;
    for &b in bytes {
        if !b.is_ascii_digit() {
            return invalid_number();
        }
        let d = (b - b'0') as u64;
        n = n
            .checked_mul(10)
            .and_then(|v| v.checked_add(d))
            .ok_or(Error::Scanner(crate::Error::InvalidNumber))?;
    }
    Ok(n)
}

fn parse_u128(bytes: &[u8]) -> Result<u128, Error> {
    let mut n = 0u128;
    for &b in bytes {
        if !b.is_ascii_digit() {
            return invalid_number();
        }
        let d = (b - b'0') as u128;
        n = n
            .checked_mul(10)
            .and_then(|v| v.checked_add(d))
            .ok_or(Error::Scanner(crate::Error::InvalidNumber))?;
    }
    Ok(n)
}

fn parse_i64(bytes: &[u8]) -> Result<i64, Error> {
    let (neg, digits) = match bytes {
        [b'-', rest @ ..] => (true, rest),
        _ => (false, bytes),
    };
    if digits.is_empty() {
        return invalid_number();
    }

    let mut n = 0i64;
    for &b in digits {
        if !b.is_ascii_digit() {
            return invalid_number();
        }
        let d = (b - b'0') as i64;
        n = if neg {
            n.checked_mul(10).and_then(|v| v.checked_sub(d))
        } else {
            n.checked_mul(10).and_then(|v| v.checked_add(d))
        }
        .ok_or(Error::Scanner(crate::Error::InvalidNumber))?;
    }
    Ok(n)
}

fn parse_i128(bytes: &[u8]) -> Result<i128, Error> {
    let (neg, digits) = match bytes {
        [b'-', rest @ ..] => (true, rest),
        _ => (false, bytes),
    };
    if digits.is_empty() {
        return invalid_number();
    }

    let mut n = 0i128;
    for &b in digits {
        if !b.is_ascii_digit() {
            return invalid_number();
        }
        let d = (b - b'0') as i128;
        n = if neg {
            n.checked_mul(10).and_then(|v| v.checked_sub(d))
        } else {
            n.checked_mul(10).and_then(|v| v.checked_add(d))
        }
        .ok_or(Error::Scanner(crate::Error::InvalidNumber))?;
    }
    Ok(n)
}

fn parse_f64(bytes: &[u8]) -> Result<f64, Error> {
    #[cfg(not(feature = "float_roundtrip"))]
    {
        crate::native_number::parse(bytes)
    }
    #[cfg(feature = "float_roundtrip")]
    {
        let value: f64 =
            fast_float2::parse(bytes).map_err(|_| Error::Scanner(crate::Error::InvalidNumber))?;
        if value.is_infinite() {
            return Err(Error::Scanner(crate::Error::InvalidNumber));
        }
        Ok(value)
    }
}

fn visit_native_number<'de, V: Visitor<'de>>(bytes: &[u8], visitor: V) -> Result<V::Value, Error> {
    if bytes.iter().any(|&b| matches!(b, b'.' | b'e' | b'E')) {
        return visitor.visit_f64(parse_f64(bytes)?);
    }
    if bytes[0] == b'-' {
        match parse_i64(bytes) {
            Ok(0) => visitor.visit_f64(-0.0),
            Ok(n) => visitor.visit_i64(n),
            Err(_) => visitor.visit_f64(parse_f64(bytes)?),
        }
    } else {
        match parse_u64(bytes) {
            Ok(n) => visitor.visit_u64(n),
            Err(_) => visitor.visit_f64(parse_f64(bytes)?),
        }
    }
}

impl<'de> Deserializer<'de> {
    fn deserialize_typed_number<V: Visitor<'de>>(&mut self, visitor: V) -> Result<V::Value, Error> {
        if !matches!(self.scanner.peek_byte_after_ws()?, b'-' | b'0'..=b'9') {
            return self.invalid_type(&visitor);
        }
        let bytes = self.scanner.read_number_bytes()?;
        visit_native_number(bytes, visitor)
    }

    fn deserialize_number<V: Visitor<'de>>(&mut self, visitor: V) -> Result<V::Value, Error> {
        let bytes = self.scanner.read_number_bytes()?;
        let is_float = bytes.iter().any(|&b| b == b'.' || b == b'e' || b == b'E');
        // arbitrary_precision mirrors serde_json's `ParserNumber`: fitting
        // integers visit as ints; overflowed integers AND all floats visit as
        // a single-entry `{ NUMBER_TOKEN: digits }` map which downstream
        // `Number`/`Value` (compiled with serde_json's flag, forwarded from
        // ours) classify back into exact numbers. Never visit_f64 here: the
        // digits must survive verbatim.
        #[cfg(feature = "arbitrary_precision")]
        {
            if !is_float {
                if bytes[0] == b'-' {
                    if let Ok(n) = parse_i64(bytes) {
                        return visitor.visit_i64(n);
                    }
                } else if let Ok(n) = parse_u64(bytes) {
                    return visitor.visit_u64(n);
                }
            }
            let digits = core::str::from_utf8(bytes)
                .map_err(|_| Error::Scanner(crate::Error::InvalidNumber))?;
            visitor.visit_map(RawTokenMapAccess {
                token: NUMBER_TOKEN,
                raw: Some(digits),
            })
        }
        #[cfg(not(feature = "arbitrary_precision"))]
        {
            if is_float {
                let n = parse_f64(bytes)?;
                return visitor.visit_f64(n);
            }
            // serde_json degrades overflowing integers to f64 on untyped paths.
            if bytes[0] == b'-' {
                match parse_i64(bytes) {
                    // `-0` visits as f64 -0.0 ("Convert into a float if we
                    // underflow, or on `-0`"); -0 is the only negative int
                    // that parses to 0 since `-00` fails validation.
                    Ok(0) => visitor.visit_f64(-0.0),
                    Ok(n) => visitor.visit_i64(n),
                    Err(_) => visitor.visit_f64(parse_f64(bytes)?),
                }
            } else {
                match parse_u64(bytes) {
                    Ok(n) => visitor.visit_u64(n),
                    Err(_) => visitor.visit_f64(parse_f64(bytes)?),
                }
            }
        }
    }
}

struct JsonSeqAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    first: bool,
    /// Set to true once we have consumed the closing `]`.
    done: bool,
}

impl<'de, 'a> SeqAccess<'de> for JsonSeqAccess<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T: de_trait::DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Error> {
        if self.done {
            return Ok(None);
        }
        self.de.scanner.skip_whitespace();
        match self.de.scanner.peek_byte() {
            Ok(b']') => {
                self.de.scanner.advance();
                self.done = true;
                return Ok(None);
            }
            Err(_) => return Err(Error::Scanner(crate::Error::UnexpectedEof)),
            Ok(_) => {}
        }
        if !self.first {
            self.de.scanner.expect_byte(b',')?;
            self.de.scanner.skip_whitespace();
            if self.de.scanner.peek_byte() == Ok(b']') {
                // Serde paths are unconditionally strict: serde_json has no
                // leniency knob, so `[1,]` is always rejected here. (The core
                // `parse`/`FromJson` APIs keep their documented lenient
                // default behind the `strict` feature.)
                return Err(Error::Scanner(crate::Error::TrailingComma));
            }
        }
        self.first = false;
        let value = seed.deserialize(&mut *self.de)?;
        Ok(Some(value))
    }
}

struct JsonMapAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    first: bool,
    pending_value: bool,
    done: bool,
}

impl<'de, 'a> MapAccess<'de> for JsonMapAccess<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K: de_trait::DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Error> {
        if self.done {
            return Ok(None);
        }
        if self.pending_value {
            return Err(Error::Custom("map key before value".into()));
        }
        self.de.scanner.skip_whitespace();
        match self.de.scanner.peek_byte() {
            Ok(b'}') => {
                self.de.scanner.advance();
                self.done = true;
                return Ok(None);
            }
            Err(_) => return Err(Error::Scanner(crate::Error::UnexpectedEof)),
            Ok(_) => {}
        }
        if !self.first {
            self.de.scanner.expect_byte(b',')?;
            self.de.scanner.skip_whitespace();
            if self.de.scanner.peek_byte() == Ok(b'}') {
                // Unconditionally strict on serde paths (see seq above).
                return Err(Error::Scanner(crate::Error::TrailingComma));
            }
        }
        // All key types (including custom seeds, enums and raw tokens) must
        // enter through a quoted JSON key. Leave the quote for the key reader.
        match self.de.scanner.peek_byte() {
            Ok(b'"') => {}
            Ok(_) => return Err(Error::Scanner(crate::Error::UnexpectedToken)),
            Err(_) => return Err(Error::Scanner(crate::Error::UnexpectedEof)),
        }
        self.first = false;
        self.pending_value = true;
        let key = seed.deserialize(MapKeyDeserializer { de: &mut *self.de })?;
        self.de.scanner.skip_whitespace();
        self.de.scanner.expect_byte(b':')?;
        Ok(Some(key))
    }

    fn next_value_seed<V: de_trait::DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Error> {
        if !self.pending_value {
            return Err(Error::Custom("map value without key".into()));
        }
        self.pending_value = false;
        seed.deserialize(&mut *self.de)
    }
}

// Keys in JSON maps are always strings.
struct MapKeyDeserializer<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

macro_rules! numeric_key {
    ($method:ident) => {
        fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
            self.de.scanner.expect_byte(b'"')?;
            if !matches!(self.de.scanner.peek_byte()?, b'-' | b'0'..=b'9') {
                return Err(Error::Scanner(crate::Error::InvalidNumber));
            }
            let value = de_trait::Deserializer::$method(&mut *self.de, visitor)?;
            self.de.scanner.expect_byte(b'"')?;
            Ok(value)
        }
    };
}

impl<'de, 'a> de_trait::Deserializer<'de> for MapKeyDeserializer<'a, 'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_str(visitor)
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.de.with_str(false, |s| match s {
            DecodedStr::Borrowed(b) => visitor.visit_borrowed_str(b),
            DecodedStr::Transient(s) => visitor.visit_str(s),
        })
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_str(visitor)
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_str(visitor)
    }

    numeric_key!(deserialize_i8);
    numeric_key!(deserialize_i16);
    numeric_key!(deserialize_i32);
    numeric_key!(deserialize_i64);
    numeric_key!(deserialize_i128);
    numeric_key!(deserialize_u8);
    numeric_key!(deserialize_u16);
    numeric_key!(deserialize_u32);
    numeric_key!(deserialize_u64);
    numeric_key!(deserialize_u128);
    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let start = self.de.scanner.pos();
        self.de.scanner.expect_byte(b'"')?;
        // Boolean keys are lexical literals, not unescaped strings. Validate
        // the closing quote before invoking user code, as serde_json does.
        let value = match self.de.scanner.peek_byte()? {
            b't' => {
                self.de.scanner.expect_bytes(b"true\"")?;
                true
            }
            b'f' => {
                self.de.scanner.expect_bytes(b"false\"")?;
                false
            }
            _ => {
                self.de.scanner.set_pos(start);
                return self.de.invalid_type(&visitor);
            }
        };
        visitor.visit_bool(value)
    }
    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        de_trait::Deserializer::deserialize_bytes(&mut *self.de, visitor)
    }
    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        de_trait::Deserializer::deserialize_bytes(&mut *self.de, visitor)
    }
    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_some(self)
    }
    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_str(visitor)
    }
    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.deserialize_str(visitor)
    }
    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        // Map keys containing RawValue follow the same token protocol.
        if name == RAW_VALUE_TOKEN {
            return self.de.deserialize_raw_token(visitor);
        }
        visitor.visit_newtype_struct(self)
    }
    fn deserialize_seq<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Error> {
        Err(Error::Custom("seq key not supported".into()))
    }
    fn deserialize_tuple<V: Visitor<'de>>(self, _: usize, _visitor: V) -> Result<V::Value, Error> {
        Err(Error::Custom("tuple key not supported".into()))
    }
    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _: &'static str,
        _: usize,
        _visitor: V,
    ) -> Result<V::Value, Error> {
        Err(Error::Custom("tuple_struct key not supported".into()))
    }
    fn deserialize_map<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Error> {
        Err(Error::Custom("map key not supported".into()))
    }
    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Error> {
        Err(Error::Custom("struct key not supported".into()))
    }
    fn deserialize_enum<V: Visitor<'de>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        de_trait::Deserializer::deserialize_enum(&mut *self.de, name, variants, visitor)
    }
    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_str(visitor)
    }
    numeric_key!(deserialize_f32);
    numeric_key!(deserialize_f64);
    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_str(visitor)
    }
}

struct StrDeserializer<'de, 'scratch> {
    value: DecodedStr<'de, 'scratch>,
}

impl<'de, 'scratch> StrDeserializer<'de, 'scratch> {
    fn new(value: DecodedStr<'de, 'scratch>) -> Self {
        StrDeserializer { value }
    }
}

impl<'de, 'scratch> de_trait::Deserializer<'de> for StrDeserializer<'de, 'scratch> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.value {
            DecodedStr::Borrowed(s) => visitor.visit_borrowed_str(s),
            DecodedStr::Transient(s) => visitor.visit_str(s),
        }
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.value {
            DecodedStr::Borrowed(s) => visitor.visit_borrowed_str(s),
            DecodedStr::Transient(s) => visitor.visit_str(s),
        }
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.value {
            DecodedStr::Borrowed(s) => visitor.visit_borrowed_str(s),
            DecodedStr::Transient(s) => visitor.visit_str(s),
        }
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.value {
            DecodedStr::Borrowed(s) => visitor.visit_borrowed_str(s),
            DecodedStr::Transient(s) => visitor.visit_str(s),
        }
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        visitor.visit_enum(self)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char bytes byte_buf option
        unit unit_struct newtype_struct seq tuple tuple_struct map struct
        ignored_any
    }
}

impl<'de, 'scratch> EnumAccess<'de> for StrDeserializer<'de, 'scratch> {
    type Error = Error;
    type Variant = UnitOnlyVariantAccess;

    fn variant_seed<V: de_trait::DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant), Error> {
        let val = seed.deserialize(StrDeserializer::new(self.value))?;
        Ok((val, UnitOnlyVariantAccess))
    }
}

struct UnitOnlyVariantAccess;

impl<'de> VariantAccess<'de> for UnitOnlyVariantAccess {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        Ok(())
    }

    fn newtype_variant_seed<T: de_trait::DeserializeSeed<'de>>(
        self,
        _seed: T,
    ) -> Result<T::Value, Error> {
        Err(Error::Custom("expected unit variant".into()))
    }

    fn tuple_variant<V: Visitor<'de>>(self, _len: usize, _visitor: V) -> Result<V::Value, Error> {
        Err(Error::Custom("expected unit variant".into()))
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Error> {
        Err(Error::Custom("expected unit variant".into()))
    }
}

struct JsonEnumAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'de, 'a> EnumAccess<'de> for JsonEnumAccess<'a, 'de> {
    type Error = Error;
    type Variant = JsonVariantAccess<'a, 'de>;

    fn variant_seed<V: de_trait::DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant), Error> {
        let val = self
            .de
            .with_str(true, |s| seed.deserialize(StrDeserializer::new(s)))?;
        Ok((val, JsonVariantAccess { de: self.de }))
    }
}

struct JsonVariantAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'de, 'a> VariantAccess<'de> for JsonVariantAccess<'a, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        self.de.scanner.read_null()?;
        Ok(())
    }

    fn newtype_variant_seed<T: de_trait::DeserializeSeed<'de>>(
        self,
        seed: T,
    ) -> Result<T::Value, Error> {
        seed.deserialize(&mut *self.de)
    }

    fn tuple_variant<V: Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value, Error> {
        de_trait::Deserializer::deserialize_seq(&mut *self.de, visitor)
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        de_trait::Deserializer::deserialize_struct(&mut *self.de, "", _fields, visitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Point {
        x: f64,
        y: f64,
    }

    #[test]
    fn point_roundtrip() {
        let p = Point { x: 1.5, y: -2.0 };
        let json = to_string(&p).unwrap();
        let p2: Point = from_str(&json).unwrap();
        assert_eq!(p, p2);
    }

    #[test]
    fn string_zero_copy() {
        let json = r#"{"x":1.0,"y":2.5}"#;
        let p: Point = from_str(json).unwrap();
        assert!((p.x - 1.0).abs() < 1e-12);
        assert!((p.y - 2.5).abs() < 1e-12);
    }

    #[test]
    fn nested_map() {
        use std::collections::HashMap;
        let mut m = HashMap::new();
        m.insert("a".to_string(), vec![1u64, 2, 3]);
        let json = to_string(&m).unwrap();
        let m2: HashMap<String, Vec<u64>> = from_str(&json).unwrap();
        assert_eq!(m["a"], m2["a"]);
    }

    #[test]
    fn serde_json_compat() {
        let v = vec![1u64, 2, 3];
        assert_eq!(to_string(&v).unwrap(), serde_json::to_string(&v).unwrap());
    }

    #[test]
    fn bool_roundtrip() {
        assert_eq!(to_string(&true).unwrap(), "true");
        assert_eq!(to_string(&false).unwrap(), "false");
        let b: bool = from_str("true").unwrap();
        assert!(b);
    }

    #[test]
    fn option_roundtrip() {
        let v: Option<u64> = Some(42);
        let json = to_string(&v).unwrap();
        let v2: Option<u64> = from_str(&json).unwrap();
        assert_eq!(v, v2);

        let v: Option<u64> = None;
        let json = to_string(&v).unwrap();
        assert_eq!(json, "null");
        let v2: Option<u64> = from_str(&json).unwrap();
        assert_eq!(v, v2);
    }

    #[test]
    fn string_escape_roundtrip() {
        let s = "hello \"world\"\nnewline\ttab\\backslash";
        let json = to_string(&s.to_string()).unwrap();
        let s2: String = from_str(&json).unwrap();
        assert_eq!(s, s2);
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    enum Color {
        Red,
        Green,
        Blue,
    }

    #[test]
    fn unit_enum_roundtrip() {
        let c = Color::Green;
        let json = to_string(&c).unwrap();
        assert_eq!(json, r#""Green""#);
        let c2: Color = from_str(&json).unwrap();
        assert_eq!(c, c2);
    }

    #[cfg(feature = "strict")]
    #[test]
    fn strict_rejects_trailing_commas() {
        use std::collections::HashMap;
        assert!(matches!(
            from_str::<Vec<u64>>("[1,2,]").unwrap_err().cause(),
            Error::Scanner(crate::Error::TrailingComma)
        ));
        assert!(matches!(
            from_str::<HashMap<String, u64>>(r#"{"a":1,}"#)
                .unwrap_err()
                .cause(),
            Error::Scanner(crate::Error::TrailingComma)
        ));
        // Well-formed input still parses.
        assert_eq!(from_str::<Vec<u64>>("[1,2]").unwrap(), vec![1, 2]);
    }

    #[test]
    fn float_overflow_errors_like_serde_json() {
        for src in ["1e400", "-1e400", "1e309"] {
            assert!(from_str::<f64>(src).is_err(), "{src} as f64");
            // Without arbitrary_precision, overflow also errors through
            // `Value`; with it, serde_json preserves the exact digits
            // instead (covered by tests/arb_precision.rs).
            if !cfg!(feature = "arbitrary_precision") {
                assert!(
                    from_str::<serde_json::Value>(src).is_err(),
                    "{src} as Value"
                );
            }
        }
        // Default upstream narrows a finite f64 to f32 infinity; the
        // float_roundtrip feature instead rejects direct f32 overflow.
        let reference = serde_json::from_str::<f32>("1e100");
        let candidate = from_str::<f32>("1e100");
        assert_eq!(candidate.is_ok(), reference.is_ok());
        if let (Ok(a), Ok(b)) = (candidate, reference) {
            assert_eq!(a.to_bits(), b.to_bits());
        }
        // Underflow saturates to zero, preserving the sign.
        assert_eq!(from_str::<f64>("1e-999").unwrap(), 0.0);
        assert!(from_str::<f64>("-1e-999").unwrap().is_sign_negative());
    }

    #[test]
    fn neg_zero_matches_serde_json() {
        // Untyped `-0` visits as f64 -0.0 without arbitrary_precision...
        #[cfg(not(feature = "arbitrary_precision"))]
        {
            let v: serde_json::Value = from_str("-0").unwrap();
            assert_eq!(
                v,
                serde_json::Value::Number(serde_json::Number::from_f64(-0.0).unwrap())
            );
            assert!(v.as_f64().unwrap().is_sign_negative());
        }
        // ...but as i64 0 with it (serde_json's own quirk, probed).
        #[cfg(feature = "arbitrary_precision")]
        {
            let v: serde_json::Value = from_str("-0").unwrap();
            assert_eq!(v, serde_json::Value::Number(serde_json::Number::from(0)));
        }
        // Typed i64-family paths reject `-0` ("invalid type: floating
        // point") in both modes; i128 accepts; floats keep the sign.
        assert!(from_str::<i64>("-0").is_err());
        assert!(from_str::<i32>("-0").is_err());
        assert_eq!(from_str::<i128>("-0").unwrap(), 0);
        assert!(from_str::<u64>("-0").is_err());
        let f = from_str::<f64>("-0").unwrap();
        assert_eq!(f, 0.0);
        assert!(f.is_sign_negative());
    }

    #[test]
    fn invalid_float_heads_rejected_like_serde_json() {
        // The single-pass float reader must reject everything JSON rejects,
        // including inputs fast-float itself would accept.
        for src in [
            "+1", "+1.5", "01", "-01", "inf", "-inf", "nan", "1e", "1e+", "--1", "0x1", "", "-",
            ".", "e5", "1ee5",
        ] {
            assert!(from_str::<f64>(src).is_err(), "{src}");
            assert!(from_str::<f32>(src).is_err(), "{src} as f32");
            assert!(
                from_str::<serde_json::Value>(src).is_err(),
                "{src} as Value"
            );
        }
        assert_eq!(from_str::<f64>("1.5").unwrap(), 1.5);
        assert_eq!(
            from_str::<f64>("-0").unwrap().to_bits(),
            (-0.0f64).to_bits()
        );
    }

    #[test]
    fn leading_dot_numbers_rejected_like_serde_json() {
        // JSON numbers require an integer part: `.5` / `-.5` are invalid.
        for src in [".5", "-.5", "-.0", ".0", ".5E-7", "-.5e+3"] {
            assert!(from_str::<serde_json::Value>(src).is_err(), "{src}");
            assert!(from_str::<f64>(src).is_err(), "{src} as f64");
        }
        // Sanity: zero-prefixed forms still parse.
        assert_eq!(from_str::<f64>("0.5").unwrap(), 0.5);
        assert_eq!(from_str::<f64>("-0.5").unwrap(), -0.5);
    }

    #[test]
    fn lone_surrogates_rejected_like_serde_json() {
        // High surrogate followed by a non-low surrogate is an error...
        assert!(from_str::<String>(r#""\uD83D\uD00A""#).is_err());
        assert!(from_str::<String>(r#""\uD83D""#).is_err());
        assert!(from_str::<String>(r#""\uDC00""#).is_err());
        // ...while a valid pair decodes.
        assert_eq!(from_str::<String>(r#""\uD83D\uDE00""#).unwrap(), "😀");
    }

    #[test]
    fn int_boundaries_match_serde_json() {
        assert_eq!(from_str::<i64>("-9223372036854775808").unwrap(), i64::MIN);
        assert_eq!(from_str::<i64>("9223372036854775807").unwrap(), i64::MAX);
        assert_eq!(from_str::<u64>("18446744073709551615").unwrap(), u64::MAX);
        assert!(from_str::<i64>("-9223372036854775809").is_err());
        assert!(from_str::<i64>("9223372036854775808").is_err());
        assert!(from_str::<u64>("18446744073709551616").is_err());
        assert!(from_str::<u8>("256").is_err());
        assert!(from_str::<i8>("-129").is_err());
        assert_eq!(from_str::<u8>("0").unwrap(), 0);
    }

    #[test]
    fn integer_types() {
        assert_eq!(to_string(&42u8).unwrap(), "42");
        assert_eq!(to_string(&-1i32).unwrap(), "-1");
        assert_eq!(to_string(&u64::MAX).unwrap(), u64::MAX.to_string());
        let n: u64 = from_str("18446744073709551615").unwrap();
        assert_eq!(n, u64::MAX);
    }

    #[cfg(not(feature = "unbounded_depth"))]
    #[test]
    fn recursion_limit_matches_serde_json_boundary() {
        // serde_json allows 127 nested opens and errors on the 128th.
        for (depth, ok) in [(10usize, true), (127, true), (128, false), (129, false)] {
            let src = format!("{}{}", "[".repeat(depth), "]".repeat(depth));
            let res: Result<serde_json::Value, Error> = from_str(&src);
            assert_eq!(res.is_ok(), ok, "depth {depth}");
            let sj: Result<serde_json::Value, _> = serde_json::from_str(&src);
            assert_eq!(sj.is_ok(), ok, "serde_json depth {depth}");
        }
        let deep = format!("{}{}", "[".repeat(500), "]".repeat(500));
        assert!(matches!(
            from_str::<serde_json::Value>(&deep).unwrap_err().cause(),
            Error::Scanner(crate::Error::RecursionLimit)
        ));
    }

    #[cfg(feature = "unbounded_depth")]
    #[test]
    fn unbounded_depth_parses_deep_input() {
        // Unbounded parsing is still recursive, so deep inputs need stack
        // proportional to depth — serde_json itself aborts past ~1000
        // levels on a 2MB stack in debug builds (verified by probe). Run
        // with a grown stack to test the parser, not the thread default.
        let deep = format!("{}{}", "[".repeat(1000), "]".repeat(1000));
        let v: serde_json::Value = std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(move || {
                assert!(from_str::<serde_json::Value>(&deep).is_err());
                let mut de = Deserializer::from_str(&deep);
                de.disable_recursion_limit();
                let value = serde_json::Value::deserialize(&mut de).unwrap();
                de.end().unwrap();
                value
            })
            .unwrap()
            .join()
            .unwrap();
        assert!(v.is_array());
    }

    #[test]
    fn huge_integers_fall_back_to_f64_like_serde_json() {
        // Untyped (Value) parsing degrades overflowing integers to f64.
        for src in [
            "123456789012345678901234567890",
            "-123456789012345678901234567890",
        ] {
            let ours: serde_json::Value = from_str(src).unwrap();
            let theirs: serde_json::Value = serde_json::from_str(src).unwrap();
            assert_eq!(ours, theirs, "input {src}");
        }
        // Typed targets still reject overflow.
        assert!(from_str::<u64>("123456789012345678901234567890").is_err());
        assert!(from_str::<i64>("-123456789012345678901234567890").is_err());
        assert!(from_str::<u64>("-1").is_err());
    }

    #[test]
    fn map_keys_stringify_like_serde_json() {
        use alloc::collections::BTreeMap;
        // serde_json stringifies primitive keys instead of emitting invalid JSON.
        let mut m = BTreeMap::new();
        m.insert(1u64, "a");
        assert_eq!(to_string(&m).unwrap(), serde_json::to_string(&m).unwrap());
        assert_eq!(to_string(&m).unwrap(), r#"{"1":"a"}"#);

        let mut m = BTreeMap::new();
        m.insert(-5i32, "a");
        assert_eq!(to_string(&m).unwrap(), r#"{"-5":"a"}"#);

        let mut m = BTreeMap::new();
        m.insert(true, "a");
        assert_eq!(to_string(&m).unwrap(), r#"{"true":"a"}"#);

        // Unit-variant keys serialize as their name.
        #[derive(Serialize, PartialEq, Eq, PartialOrd, Ord)]
        enum K {
            Alpha,
        }
        let mut m = BTreeMap::new();
        m.insert(K::Alpha, 1u32);
        assert_eq!(to_string(&m).unwrap(), r#"{"Alpha":1}"#);
    }

    #[test]
    fn map_keys_reject_structured_keys_like_serde_json() {
        use serde::ser::SerializeMap;
        // Struct keys are not representable: serde_json errors "key must be a string".
        struct StructKeyMap;
        impl Serialize for StructKeyMap {
            fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                #[derive(Serialize)]
                struct K {
                    a: u32,
                }
                let mut m = s.serialize_map(Some(1))?;
                m.serialize_key(&K { a: 1 })?;
                m.serialize_value("a")?;
                m.end()
            }
        }
        let err = to_string(&StructKeyMap).unwrap_err().to_string();
        assert_eq!(err, "key must be a string");
        let sj_err = serde_json::to_string(&StructKeyMap)
            .unwrap_err()
            .to_string();
        assert_eq!(err, sj_err);

        // Unit keys also rejected.
        struct UnitKeyMap;
        impl Serialize for UnitKeyMap {
            fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                let mut m = s.serialize_map(Some(1))?;
                m.serialize_key(&())?;
                m.serialize_value("a")?;
                m.end()
            }
        }
        assert_eq!(
            to_string(&UnitKeyMap).unwrap_err().to_string(),
            "key must be a string"
        );
    }

    #[test]
    fn float_zero_sign_matches_serde_json() {
        assert_eq!(to_string(&0.0f64).unwrap(), "0.0");
        assert_eq!(to_string(&-0.0f64).unwrap(), "-0.0");
        assert_eq!(to_string(&0.0f32).unwrap(), "0.0");
        assert_eq!(to_string(&-0.0f32).unwrap(), "-0.0");
    }

    // Byte-exact float parity holds unconditionally now that ryu (or zmij)
    // backs every build.
    #[test]
    fn float_output_matches_serde_json() {
        let values: &[f64] = &[
            0.1 + 0.2,
            1e21,
            1e20,
            1e16,
            1e-3,
            1e-7,
            123456789.0,
            100.0,
            3.0,
            1.5,
            -1.5e-7,
            2.2250738585072014e-308,
            1.7976931348623157e308,
            f64::MIN_POSITIVE,
            5e-324,
            1e100,
            -1e100,
            std::f64::consts::PI,
            2.0 / 3.0,
            0.30000000000000004,
        ];
        for &v in values {
            assert_eq!(
                to_string(&v).unwrap(),
                serde_json::to_string(&v).unwrap(),
                "f64 {v:e}"
            );
        }
        let values32: &[f32] = &[
            0.1,
            1e21,
            1e10,
            1e-3,
            3.0,
            -0.0,
            f32::MAX,
            f32::MIN_POSITIVE,
            1e38,
            1.1754944e-38,
            1.5,
            100.0,
        ];
        for &v in values32 {
            assert_eq!(
                to_string(&v).unwrap(),
                serde_json::to_string(&v).unwrap(),
                "f32 {v:e}"
            );
        }
    }

    #[test]
    fn map_float_keys_stringify_and_reject_infinite() {
        use serde::ser::SerializeMap;
        struct FloatKeyMap(f64);
        impl Serialize for FloatKeyMap {
            fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                let mut m = s.serialize_map(Some(1))?;
                m.serialize_key(&self.0)?;
                m.serialize_value("a")?;
                m.end()
            }
        }
        assert_eq!(to_string(&FloatKeyMap(1.5)).unwrap(), r#"{"1.5":"a"}"#);
        assert_eq!(
            to_string(&FloatKeyMap(f64::INFINITY))
                .unwrap_err()
                .to_string(),
            "float key must be finite"
        );
    }
}

#[cfg(not(feature = "std"))]
impl From<core::convert::Infallible> for Error {
    fn from(value: core::convert::Infallible) -> Self {
        match value {}
    }
}
