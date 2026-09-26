//! Serde-compatible JSON serializer/deserializer backed by jzon's SIMD string
//! escaping and zero-copy scanner.
//!
//! Works with **any** type that derives `serde::Serialize` / `serde::Deserialize`.
//! Available as `jzon::from_str` / `jzon::to_string` with the `serde` cargo
//! feature, or through the standalone [`jzon-rs-serde`](https://crates.io/crates/jzon-rs-serde)
//! crate which re-exports this module.
//!
//! # Usage
//!
//! ```rust,ignore
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

use serde::ser::{
    self as ser_trait, Serialize, SerializeMap, SerializeSeq, SerializeStruct,
    SerializeStructVariant, SerializeTuple, SerializeTupleStruct, SerializeTupleVariant,
};
use serde::de::{self as de_trait, DeserializeOwned, Visitor, MapAccess, SeqAccess, EnumAccess, VariantAccess};

use crate::ser::{write_escaped_str, write_u64, write_i64, write_u128, write_i128, ToJson};
use crate::{Scanner, JsonStr};

#[derive(Debug)]
pub enum Error {
    Custom(String),
    InvalidUtf8,
    Io(std::io::Error),
    Scanner(crate::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Custom(m)    => write!(f, "{m}"),
            Error::InvalidUtf8  => write!(f, "invalid UTF-8"),
            Error::Io(e)        => write!(f, "I/O error: {e}"),
            Error::Scanner(e)   => write!(f, "JSON parse error: {e}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e)      => Some(e),
            Error::Scanner(e) => Some(e),
            _                 => None,
        }
    }
}

impl ser_trait::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}

impl de_trait::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}

impl From<crate::Error> for Error {
    fn from(e: crate::Error) -> Self {
        Error::Scanner(e)
    }
}

// ── float helpers ─────────────────────────────────────────────────────────────
// serde_json always emits a decimal point for whole-number floats (e.g. 3.0 →
// "3.0").  jzon's core ToJson impl omits it to keep float rendering minimal.
// We add it back here in the serde layer so output matches serde_json.

#[inline]
fn ensure_decimal_point(buf_start: usize, w: &mut Vec<u8>) {
    let written = &w[buf_start..];
    let has_dot = written.contains(&b'.');
    let has_exp = written.contains(&b'e') || written.contains(&b'E');
    if !has_dot && !has_exp {
        w.extend_from_slice(b".0");
    }
}

/// Insert an explicit `+` in unsigned exponents (`1e21` → `1e+21`).
/// `serde_json` (zmij backend) always signs the exponent; the `ryu` backend
/// omits `+`. Rare path — only floats rendered with an exponent.
#[inline]
fn ensure_exponent_sign(buf_start: usize, w: &mut Vec<u8>) {
    let written = &w[buf_start..];
    for (i, &b) in written.iter().enumerate() {
        if b == b'e' || b == b'E' {
            match written.get(i + 1) {
                Some(b'+') | Some(b'-') | None => {}
                Some(_) => w.insert(buf_start + i + 1, b'+'),
            }
            return;
        }
    }
}

#[inline]
fn serialize_float64(v: f64, w: &mut Vec<u8>) {
    if !v.is_finite() {
        w.extend_from_slice(b"null");
        return;
    }
    if v == 0.0 {
        // serde_json preserves the sign of zero; the core backend
        // normalizes -0.0 to "0" (ECMA-262), so emit directly here.
        w.extend_from_slice(if v.is_sign_negative() { b"-0.0" } else { b"0.0" });
        return;
    }
    let start = w.len();
    v.json_write(w);
    ensure_decimal_point(start, w);
    ensure_exponent_sign(start, w);
}

#[inline]
fn serialize_float32(v: f32, w: &mut Vec<u8>) {
    if !v.is_finite() {
        w.extend_from_slice(b"null");
        return;
    }
    if v == 0.0 {
        w.extend_from_slice(if v.is_sign_negative() { b"-0.0" } else { b"0.0" });
        return;
    }
    let start = w.len();
    v.json_write(w);
    ensure_decimal_point(start, w);
    ensure_exponent_sign(start, w);
}

pub struct Serializer {
    output: Vec<u8>,
}

pub fn to_string<T: Serialize>(v: &T) -> Result<String, Error> {
    let mut ser = Serializer { output: Vec::with_capacity(128) };
    v.serialize(&mut ser)?;
    String::from_utf8(ser.output).map_err(|_| Error::InvalidUtf8)
}

pub fn to_bytes<T: Serialize>(v: &T) -> Result<Vec<u8>, Error> {
    let mut ser = Serializer { output: Vec::with_capacity(128) };
    v.serialize(&mut ser)?;
    Ok(ser.output)
}

pub fn to_writer<W: std::io::Write, T: Serialize>(mut w: W, v: &T) -> Result<(), Error> {
    let bytes = to_bytes(v)?;
    w.write_all(&bytes).map_err(Error::Io)
}

impl<'a> ser_trait::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    type SerializeSeq            = SeqSerializer<'a>;
    type SerializeTuple          = SeqSerializer<'a>;
    type SerializeTupleStruct    = SeqSerializer<'a>;
    type SerializeTupleVariant   = SeqSerializer<'a>;
    type SerializeMap            = MapSerializer<'a>;
    type SerializeStruct         = StructSerializer<'a>;
    type SerializeStructVariant  = MapSerializer<'a>;

    #[inline]
    fn serialize_bool(self, v: bool) -> Result<(), Error> {
        self.output.extend_from_slice(if v { b"true" } else { b"false" });
        Ok(())
    }

    #[inline]
    fn serialize_i8(self, v: i8) -> Result<(), Error> { write_i64(v as i64, &mut self.output); Ok(()) }
    #[inline]
    fn serialize_i16(self, v: i16) -> Result<(), Error> { write_i64(v as i64, &mut self.output); Ok(()) }
    #[inline]
    fn serialize_i32(self, v: i32) -> Result<(), Error> { write_i64(v as i64, &mut self.output); Ok(()) }
    #[inline]
    fn serialize_i64(self, v: i64) -> Result<(), Error> { write_i64(v, &mut self.output); Ok(()) }

    #[inline]
    fn serialize_u8(self, v: u8) -> Result<(), Error> { write_u64(v as u64, &mut self.output); Ok(()) }
    #[inline]
    fn serialize_u16(self, v: u16) -> Result<(), Error> { write_u64(v as u64, &mut self.output); Ok(()) }
    #[inline]
    fn serialize_u32(self, v: u32) -> Result<(), Error> { write_u64(v as u64, &mut self.output); Ok(()) }
    #[inline]
    fn serialize_u64(self, v: u64) -> Result<(), Error> { write_u64(v, &mut self.output); Ok(()) }

    #[inline]
    fn serialize_f32(self, v: f32) -> Result<(), Error> {
        serialize_float32(v, &mut self.output);
        Ok(())
    }
    #[inline]
    fn serialize_f64(self, v: f64) -> Result<(), Error> {
        serialize_float64(v, &mut self.output);
        Ok(())
    }

    #[inline]
    fn serialize_char(self, v: char) -> Result<(), Error> {
        let mut buf = [0u8; 4];
        let s = v.encode_utf8(&mut buf);
        write_escaped_str(s, &mut self.output);
        Ok(())
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result<(), Error> {
        write_escaped_str(v, &mut self.output);
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<(), Error> {
        self.output.push(b'[');
        for (i, &b) in v.iter().enumerate() {
            if i > 0 { self.output.push(b','); }
            write_u64(b as u64, &mut self.output);
        }
        self.output.push(b']');
        Ok(())
    }

    #[inline]
    fn serialize_none(self) -> Result<(), Error> {
        self.output.extend_from_slice(b"null");
        Ok(())
    }

    #[inline]
    fn serialize_some<T: Serialize + ?Sized>(self, v: &T) -> Result<(), Error> {
        v.serialize(self)
    }

    #[inline]
    fn serialize_unit(self) -> Result<(), Error> {
        self.output.extend_from_slice(b"null");
        Ok(())
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<(), Error> {
        self.output.extend_from_slice(b"null");
        Ok(())
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<(), Error> {
        write_escaped_str(variant, &mut self.output);
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
        self.output.push(b'{');
        write_escaped_str(variant, &mut self.output);
        self.output.push(b':');
        value.serialize(&mut *self)?;
        self.output.push(b'}');
        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<SeqSerializer<'a>, Error> {
        self.output.push(b'[');
        Ok(SeqSerializer { ser: self, first: true })
    }

    fn serialize_tuple(self, len: usize) -> Result<SeqSerializer<'a>, Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<SeqSerializer<'a>, Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<SeqSerializer<'a>, Error> {
        self.output.push(b'{');
        write_escaped_str(variant, &mut self.output);
        self.output.push(b':');
        self.output.push(b'[');
        Ok(SeqSerializer { ser: self, first: true })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<MapSerializer<'a>, Error> {
        self.output.push(b'{');
        Ok(MapSerializer { ser: self, first: true, variant_wrap: false })
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<StructSerializer<'a>, Error> {
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
        Ok(StructSerializer::Map(self.serialize_map(Some(len))?))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<MapSerializer<'a>, Error> {
        self.output.push(b'{');
        write_escaped_str(variant, &mut self.output);
        self.output.push(b':');
        self.output.push(b'{');
        Ok(MapSerializer { ser: self, first: true, variant_wrap: true })
    }

    fn is_human_readable(&self) -> bool { true }
}

pub struct SeqSerializer<'a> {
    ser: &'a mut Serializer,
    first: bool,
}

impl<'a> SerializeSeq for SeqSerializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        if !self.first { self.ser.output.push(b','); }
        self.first = false;
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<(), Error> {
        self.ser.output.push(b']');
        Ok(())
    }
}

macro_rules! delegate_to_seq {
    ($trait:ident, $method:ident) => {
        impl<'a> $trait for SeqSerializer<'a> {
            type Ok = ();
            type Error = Error;
            fn $method<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
                SerializeSeq::serialize_element(self, value)
            }
            fn end(self) -> Result<(), Error> { SerializeSeq::end(self) }
        }
    };
}
delegate_to_seq!(SerializeTuple, serialize_element);
delegate_to_seq!(SerializeTupleStruct, serialize_field);

impl<'a> SerializeTupleVariant for SeqSerializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        SerializeSeq::serialize_element(self, value)
    }
    fn end(self) -> Result<(), Error> {
        self.ser.output.push(b']');
        self.ser.output.push(b'}');
        Ok(())
    }
}

/// Serializer for map keys: mirrors `serde_json` exactly — strings, numbers,
/// bools, chars and unit variants are stringified; anything structural is an
/// error (`collect_str` inherits serde's default, same as `serde_json`).
struct MapKeySerializer<'a> {
    ser: &'a mut Serializer,
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
            self.ser.output.push(b'"');
            $write(value as _, &mut self.ser.output);
            self.ser.output.push(b'"');
            Ok(())
        }
    };
}

impl<'a> ser_trait::Serializer for MapKeySerializer<'a> {
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
        self.ser.output.push(b'"');
        self.ser.output.extend_from_slice(if value { b"true" } else { b"false" });
        self.ser.output.push(b'"');
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
        self.ser.output.push(b'"');
        serialize_float32(value, &mut self.ser.output);
        self.ser.output.push(b'"');
        Ok(())
    }

    fn serialize_f64(self, value: f64) -> Result<(), Error> {
        if !value.is_finite() {
            return Err(float_key_must_be_finite());
        }
        self.ser.output.push(b'"');
        serialize_float64(value, &mut self.ser.output);
        self.ser.output.push(b'"');
        Ok(())
    }

    fn serialize_char(self, value: char) -> Result<(), Error> {
        let mut buf = [0u8; 4];
        let s = value.encode_utf8(&mut buf);
        write_escaped_str(s, &mut self.ser.output);
        Ok(())
    }

    fn serialize_str(self, value: &str) -> Result<(), Error> {
        write_escaped_str(value, &mut self.ser.output);
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
        write_escaped_str(variant, &mut self.ser.output);
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

pub struct MapSerializer<'a> {
    ser: &'a mut Serializer,
    first: bool,
    /// True when this is a struct-variant that needs an extra closing `}`.
    variant_wrap: bool,
}

impl<'a> SerializeMap for MapSerializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<(), Error> {
        if !self.first { self.ser.output.push(b','); }
        self.first = false;
        key.serialize(MapKeySerializer { ser: &mut *self.ser })?;
        self.ser.output.push(b':');
        Ok(())
    }

    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<(), Error> {
        self.ser.output.push(b'}');
        if self.variant_wrap { self.ser.output.push(b'}'); }
        Ok(())
    }
}

impl<'a> SerializeStruct for MapSerializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        if !self.first { self.ser.output.push(b','); }
        self.first = false;
        write_escaped_str(key, &mut self.ser.output);
        self.ser.output.push(b':');
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<(), Error> {
        self.ser.output.push(b'}');
        if self.variant_wrap { self.ser.output.push(b'}'); }
        Ok(())
    }
}

/// Token names of serde_json's private raw-passthrough protocols.
const RAW_VALUE_TOKEN: &str = "$serde_json::private::RawValue";
const NUMBER_TOKEN: &str = "$serde_json::private::Number";

/// Struct serializer dispatch: ordinary structs serialize as maps, while
/// serde_json token structs emit their payload raw (unquoted).
pub enum StructSerializer<'a> {
    Map(MapSerializer<'a>),
    Raw(RawTokenStructSerializer<'a>),
}

impl<'a> SerializeStruct for StructSerializer<'a> {
    type Ok = ();
    type Error = Error;

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

    fn end(self) -> Result<(), Error> {
        match self {
            StructSerializer::Map(m) => SerializeStruct::end(m),
            StructSerializer::Raw(r) => SerializeStruct::end(r),
        }
    }
}

pub struct RawTokenStructSerializer<'a> {
    ser: &'a mut Serializer,
    token: &'static str,
    err_msg: &'static str,
}

impl<'a> SerializeStruct for RawTokenStructSerializer<'a> {
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
struct RawStrEmitter<'a> {
    ser: &'a mut Serializer,
    err_msg: &'static str,
}

macro_rules! raw_err {
    ($method:ident, $($arg:ident : $ty:ty),*) => {
        fn $method(self, $(_: $ty),*) -> Result<(), Error> {
            Err(Error::Custom(self.err_msg.to_owned()))
        }
    };
}

impl<'a> ser_trait::Serializer for RawStrEmitter<'a> {
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
        self.ser.output.extend_from_slice(value.as_bytes());
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

impl<'a> SerializeStructVariant for MapSerializer<'a> {
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
}

impl<'de> Deserializer<'de> {
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
        self.scanner.skip_value()?;
        let raw = core::str::from_utf8(&rest[..self.scanner.pos() - start])
            .map_err(|_| Error::InvalidUtf8)?;
        visitor.visit_map(RawTokenMapAccess {
            token: RAW_VALUE_TOKEN,
            raw: Some(raw),
        })
    }
}

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
    let scanner = Scanner::new_str(s);
    let mut de = Deserializer { scanner };
    let value = T::deserialize(&mut de)?;
    // ECMA-404: a JSON text is exactly one value — reject trailing content.
    de.scanner.expect_eof()?;
    Ok(value)
}

pub fn from_slice<'de, T: serde::Deserialize<'de>>(b: &'de [u8]) -> Result<T, Error> {
    let scanner = Scanner::new(b);
    let mut de = Deserializer { scanner };
    let value = T::deserialize(&mut de)?;
    de.scanner.expect_eof()?;
    Ok(value)
}

fn from_reader_inner<'de, T: serde::Deserialize<'de>>(b: &'de [u8]) -> Result<T, Error> {
    from_slice(b)
}

pub fn from_reader<R: std::io::Read, T: DeserializeOwned>(mut r: R) -> Result<T, Error> {
    let mut buf = Vec::new();
    r.read_to_end(&mut buf).map_err(Error::Io)?;
    from_reader_inner(&buf)
}

/// Deserialize from `&str`, also returning the underlying [`Scanner`](crate::Scanner)
/// statistics (zero-copy borrows, heap allocations, bytes scanned).
///
/// Available with the `stats` feature.
#[cfg(feature = "stats")]
pub fn from_str_with_stats<'de, T: serde::Deserialize<'de>>(
    s: &'de str,
) -> Result<(T, crate::stats::ScannerStats), Error> {
    let scanner = Scanner::new_str(s);
    let mut de = Deserializer { scanner };
    let value = T::deserialize(&mut de)?;
    de.scanner.expect_eof()?;
    Ok((value, de.scanner.stats))
}

/// Deserialize from a byte slice, also returning [`Scanner`](crate::Scanner) statistics.
///
/// Available with the `stats` feature.
#[cfg(feature = "stats")]
pub fn from_slice_with_stats<'de, T: serde::Deserialize<'de>>(
    b: &'de [u8],
) -> Result<(T, crate::stats::ScannerStats), Error> {
    let scanner = Scanner::new(b);
    let mut de = Deserializer { scanner };
    let value = T::deserialize(&mut de)?;
    de.scanner.expect_eof()?;
    Ok((value, de.scanner.stats))
}

/// Deserialize from a reader, also returning [`Scanner`](crate::Scanner) statistics.
///
/// Available with the `stats` feature.
#[cfg(feature = "stats")]
pub fn from_reader_with_stats<R: std::io::Read, T: DeserializeOwned>(
    mut r: R,
) -> Result<(T, crate::stats::ScannerStats), Error> {
    let mut buf = Vec::new();
    r.read_to_end(&mut buf).map_err(Error::Io)?;
    from_slice_with_stats(&buf)
}

macro_rules! deserialize_signed_int {
    ($method:ident, $visit:ident, $num_ty:ty) => {
        fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
            let bytes = self.scanner.read_number_bytes()?;
            // serde_json parses `-0` as float -0.0 on every i64-family path,
            // so integer targets reject it ("invalid type: floating point").
            // (i128 has its own path and accepts `-0`; see below.)
            if bytes == b"-0" {
                return Err(Error::Scanner(crate::Error::InvalidNumber));
            }
            let n = parse_i64(bytes)?;
            if n < <$num_ty>::MIN as i64 || n > <$num_ty>::MAX as i64 {
                return Err(Error::Scanner(crate::Error::InvalidNumber));
            }
            visitor.$visit(n as $num_ty)
        }
    };
}

macro_rules! deserialize_unsigned_int {
    ($method:ident, $visit:ident, $num_ty:ty) => {
        fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
            let bytes = self.scanner.read_number_bytes()?;
            let n = parse_u64(bytes)?;
            if n > <$num_ty>::MAX as u64 {
                return Err(Error::Scanner(crate::Error::InvalidNumber));
            }
            visitor.$visit(n as $num_ty)
        }
    };
}

impl<'de, 'a> de_trait::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let b = self.scanner.peek_byte_after_ws()?;
        match b {
            b'"'               => self.deserialize_str(visitor),
            b'{'               => self.deserialize_map(visitor),
            b'['               => self.deserialize_seq(visitor),
            b't' | b'f'        => self.deserialize_bool(visitor),
            b'n'               => { self.scanner.read_null()?; visitor.visit_unit() },
            b'-' | b'0'..=b'9' => self.deserialize_number(visitor),
            _                  => Err(Error::Scanner(crate::Error::UnexpectedToken)),
        }
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let v = self.scanner.read_bool()?;
        visitor.visit_bool(v)
    }

    deserialize_signed_int!(deserialize_i8,  visit_i8,  i8);
    deserialize_signed_int!(deserialize_i16, visit_i16, i16);
    deserialize_signed_int!(deserialize_i32, visit_i32, i32);
    deserialize_signed_int!(deserialize_i64, visit_i64, i64);
    deserialize_unsigned_int!(deserialize_u8,  visit_u8,  u8);
    deserialize_unsigned_int!(deserialize_u16, visit_u16, u16);
    deserialize_unsigned_int!(deserialize_u32, visit_u32, u32);
    deserialize_unsigned_int!(deserialize_u64, visit_u64, u64);

    fn deserialize_i128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let bytes = self.scanner.read_number_bytes()?;
        visitor.visit_i128(parse_i128(bytes)?)
    }

    fn deserialize_u128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let bytes = self.scanner.read_number_bytes()?;
        visitor.visit_u128(parse_u128(bytes)?)
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let bytes = self.scanner.read_number_bytes()?;
        let n: f32 = parse_f32(bytes)?;
        visitor.visit_f32(n)
    }
    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let bytes = self.scanner.read_number_bytes()?;
        let n: f64 = parse_f64(bytes)?;
        visitor.visit_f64(n)
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let s = self.scanner.read_str()?;
        let st = s.as_str();
        let mut chars = st.chars();
        let c = chars.next().ok_or_else(|| Error::Custom("expected char".into()))?;
        visitor.visit_char(c)
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let s = self.scanner.read_str()?;
        match s {
            JsonStr::Borrowed(b) | JsonStr::BorrowedNoEsc(b) => visitor.visit_borrowed_str(b),
            JsonStr::Owned(o)    => visitor.visit_string(o),
        }
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        if self.scanner.peek_null() {
            self.scanner.read_null()?;
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.scanner.read_null()?;
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        // serde_json raw-value protocol: capture one full JSON value verbatim
        // and present it as a single-entry map { TOKEN: raw }.
        if name == RAW_VALUE_TOKEN {
            return self.deserialize_raw_token(visitor);
        }
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let _depth = self.scanner.enter_depth()?;
        self.scanner.skip_whitespace();
        self.scanner.expect_byte(b'[')?;
        let value = visitor.visit_seq(JsonSeqAccess { de: self, first: true, done: false })?;
        Ok(value)
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
        let _depth = self.scanner.enter_depth()?;
        self.scanner.skip_whitespace();
        self.scanner.expect_byte(b'{')?;
        let value = visitor.visit_map(JsonMapAccess { de: self, first: true, pending_value: false })?;
        Ok(value)
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        let b = self.scanner.peek_byte_after_ws()?;
        if b == b'"' {
            let s = self.scanner.read_str()?;
            let variant = s.into_owned();
            visitor.visit_enum(StrDeserializer::new(variant))
        } else if b == b'{' {
            let _depth = self.scanner.enter_depth()?;
            self.scanner.skip_whitespace();
            self.scanner.expect_byte(b'{')?;
            let value = visitor.visit_enum(JsonEnumAccess { de: self })?;
            self.scanner.skip_whitespace();
            self.scanner.expect_byte(b'}')?;
            Ok(value)
        } else {
            Err(Error::Scanner(crate::Error::UnexpectedToken))
        }
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.scanner.skip_value()?;
        visitor.visit_unit()
    }

    fn is_human_readable(&self) -> bool { true }
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
    // fast-float2 parses integers and floats alike in a single pass.
    let value: f64 = fast_float2::parse(bytes)
        .map_err(|_| Error::Scanner(crate::Error::InvalidNumber))?;
    // serde_json errors on overflow ("number out of range") in every mode.
    if value.is_infinite() {
        return Err(Error::Scanner(crate::Error::InvalidNumber));
    }
    Ok(value)
}

fn parse_f32(bytes: &[u8]) -> Result<f32, Error> {
    // Parse directly as f32 (serde_json's `float_roundtrip` behavior);
    // overflow errors rather than saturating to infinity.
    let value: f32 = fast_float2::parse(bytes)
        .map_err(|_| Error::Scanner(crate::Error::InvalidNumber))?;
    if value.is_infinite() {
        return Err(Error::Scanner(crate::Error::InvalidNumber));
    }
    Ok(value)
}

impl<'de, 'a> Deserializer<'de> {
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
            return visitor.visit_map(RawTokenMapAccess {
                token: NUMBER_TOKEN,
                raw: Some(digits),
            });
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
        self.de.scanner.skip_whitespace();
        match self.de.scanner.peek_byte() {
            Ok(b']') => { self.de.scanner.advance(); self.done = true; return Ok(None); }
            Err(_)   => return Err(Error::Scanner(crate::Error::UnexpectedEof)),
            Ok(_)    => {}
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

impl<'a, 'de> Drop for JsonSeqAccess<'a, 'de> {
    fn drop(&mut self) {
        if !self.done {
            // The visitor stopped early without draining all elements.
            // Consume remaining array elements and the closing `]` so the
            // scanner is positioned correctly for the caller.
            let _ = self.de.scanner.skip_array_tail();
        }
    }
}

struct JsonMapAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    first: bool,
    pending_value: bool,
}

impl<'de, 'a> MapAccess<'de> for JsonMapAccess<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K: de_trait::DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Error> {
        self.de.scanner.skip_whitespace();
        match self.de.scanner.peek_byte() {
            Ok(b'}') => { self.de.scanner.advance(); return Ok(None); }
            Err(_)   => return Err(Error::Scanner(crate::Error::UnexpectedEof)),
            Ok(_)    => {}
        }
        if !self.first {
            self.de.scanner.expect_byte(b',')?;
            self.de.scanner.skip_whitespace();
            if self.de.scanner.peek_byte() == Ok(b'}') {
                // Unconditionally strict on serde paths (see seq above).
                return Err(Error::Scanner(crate::Error::TrailingComma));
            }
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
        self.pending_value = false;
        seed.deserialize(&mut *self.de)
    }
}

// Keys in JSON maps are always strings.
struct MapKeyDeserializer<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'de, 'a> de_trait::Deserializer<'de> for MapKeyDeserializer<'a, 'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_str(visitor)
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let s = self.de.scanner.read_str()?;
        match s {
            JsonStr::Borrowed(b) | JsonStr::BorrowedNoEsc(b) => visitor.visit_borrowed_str(b),
            JsonStr::Owned(o)    => visitor.visit_string(o),
        }
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_str(visitor)
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_str(visitor)
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let s = self.de.scanner.read_str()?;
        let n: i8 = s.as_str().parse().map_err(|_| Error::Scanner(crate::Error::InvalidNumber))?;
        visitor.visit_i8(n)
    }
    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let s = self.de.scanner.read_str()?;
        let n: i16 = s.as_str().parse().map_err(|_| Error::Scanner(crate::Error::InvalidNumber))?;
        visitor.visit_i16(n)
    }
    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let s = self.de.scanner.read_str()?;
        let n: i32 = s.as_str().parse().map_err(|_| Error::Scanner(crate::Error::InvalidNumber))?;
        visitor.visit_i32(n)
    }
    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let s = self.de.scanner.read_str()?;
        let n: i64 = s.as_str().parse().map_err(|_| Error::Scanner(crate::Error::InvalidNumber))?;
        visitor.visit_i64(n)
    }
    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let s = self.de.scanner.read_str()?;
        let n: u8 = s.as_str().parse().map_err(|_| Error::Scanner(crate::Error::InvalidNumber))?;
        visitor.visit_u8(n)
    }
    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let s = self.de.scanner.read_str()?;
        let n: u16 = s.as_str().parse().map_err(|_| Error::Scanner(crate::Error::InvalidNumber))?;
        visitor.visit_u16(n)
    }
    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let s = self.de.scanner.read_str()?;
        let n: u32 = s.as_str().parse().map_err(|_| Error::Scanner(crate::Error::InvalidNumber))?;
        visitor.visit_u32(n)
    }
    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let s = self.de.scanner.read_str()?;
        let n: u64 = s.as_str().parse().map_err(|_| Error::Scanner(crate::Error::InvalidNumber))?;
        visitor.visit_u64(n)
    }
    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let s = self.de.scanner.read_str()?;
        let b: bool = s.as_str().parse().map_err(|_| Error::Scanner(crate::Error::UnexpectedToken))?;
        visitor.visit_bool(b)
    }
    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> { self.deserialize_str(visitor) }
    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> { self.deserialize_str(visitor) }
    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> { visitor.visit_some(self) }
    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> { visitor.visit_unit() }
    fn deserialize_unit_struct<V: Visitor<'de>>(self, _: &'static str, visitor: V) -> Result<V::Value, Error> { visitor.visit_unit() }
    fn deserialize_newtype_struct<V: Visitor<'de>>(self, name: &'static str, visitor: V) -> Result<V::Value, Error> {
        // Map keys containing RawValue follow the same token protocol.
        if name == RAW_VALUE_TOKEN {
            return self.de.deserialize_raw_token(visitor);
        }
        visitor.visit_newtype_struct(self)
    }
    fn deserialize_seq<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Error> { Err(Error::Custom("seq key not supported".into())) }
    fn deserialize_tuple<V: Visitor<'de>>(self, _: usize, _visitor: V) -> Result<V::Value, Error> { Err(Error::Custom("tuple key not supported".into())) }
    fn deserialize_tuple_struct<V: Visitor<'de>>(self, _: &'static str, _: usize, _visitor: V) -> Result<V::Value, Error> { Err(Error::Custom("tuple_struct key not supported".into())) }
    fn deserialize_map<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Error> { Err(Error::Custom("map key not supported".into())) }
    fn deserialize_struct<V: Visitor<'de>>(self, _: &'static str, _: &'static [&'static str], _visitor: V) -> Result<V::Value, Error> { Err(Error::Custom("struct key not supported".into())) }
    fn deserialize_enum<V: Visitor<'de>>(self, _: &'static str, _: &'static [&'static str], visitor: V) -> Result<V::Value, Error> { self.deserialize_str(visitor) }
    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> { self.deserialize_str(visitor) }
    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let s = self.de.scanner.read_str()?;
        let n: f32 = s.as_str().parse().map_err(|_| Error::Scanner(crate::Error::InvalidNumber))?;
        visitor.visit_f32(n)
    }
    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        let s = self.de.scanner.read_str()?;
        let n: f64 = s.as_str().parse().map_err(|_| Error::Scanner(crate::Error::InvalidNumber))?;
        visitor.visit_f64(n)
    }
    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> { self.deserialize_str(visitor) }
}

struct StrDeserializer {
    value: String,
}

impl StrDeserializer {
    fn new(value: String) -> Self { StrDeserializer { value } }
}

impl<'de> de_trait::Deserializer<'de> for StrDeserializer {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_string(self.value)
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_string(self.value)
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_string(self.value)
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_string(self.value)
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

impl<'de> EnumAccess<'de> for StrDeserializer {
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

    fn unit_variant(self) -> Result<(), Error> { Ok(()) }

    fn newtype_variant_seed<T: de_trait::DeserializeSeed<'de>>(
        self,
        _seed: T,
    ) -> Result<T::Value, Error> {
        Err(Error::Custom("expected unit variant".into()))
    }

    fn tuple_variant<V: Visitor<'de>>(
        self,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, Error> {
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
        let s = self.de.scanner.read_str()?;
        let variant_name = s.into_owned();
        self.de.scanner.skip_whitespace();
        self.de.scanner.expect_byte(b':')?;
        let val = seed.deserialize(StrDeserializer::new(variant_name))?;
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

    fn tuple_variant<V: Visitor<'de>>(
        self,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Error> {
        de_trait::Deserializer::deserialize_seq(&mut *self.de, visitor)
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        de_trait::Deserializer::deserialize_map(&mut *self.de, visitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Point { x: f64, y: f64 }

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
    enum Color { Red, Green, Blue }

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
            from_str::<Vec<u64>>("[1,2,]").unwrap_err(),
            Error::Scanner(crate::Error::TrailingComma)
        ));
        assert!(matches!(
            from_str::<HashMap<String, u64>>(r#"{"a":1,}"#).unwrap_err(),
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
        // f32 overflow also errors (serde_json's `float_roundtrip` behavior).
        assert!(from_str::<f32>("1e100").is_err());
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
            from_str::<serde_json::Value>(&deep),
            Err(Error::Scanner(crate::Error::RecursionLimit))
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
            .spawn(move || from_str(&deep).unwrap())
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
        use std::collections::BTreeMap;
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
        let sj_err = serde_json::to_string(&StructKeyMap).unwrap_err().to_string();
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
            3.141592653589793,
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
            to_string(&FloatKeyMap(f64::INFINITY)).unwrap_err().to_string(),
            "float key must be finite"
        );
    }
}
