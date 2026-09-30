use serde::de::{Deserializer, Visitor};
use serde::Deserialize;
use std::fmt;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub enum EnumKey {
    Unit,
    New(u8),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NewtypeKey(pub bool);
impl<'de> Deserialize<'de> for NewtypeKey {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        struct NewtypeVisitor;
        impl<'de> Visitor<'de> for NewtypeVisitor {
            type Value = NewtypeKey;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a newtype boolean key")
            }
            fn visit_newtype_struct<D: Deserializer<'de>>(
                self,
                de: D,
            ) -> Result<Self::Value, D::Error> {
                bool::deserialize(de).map(NewtypeKey)
            }
        }
        de.deserialize_newtype_struct("NewtypeKey", NewtypeVisitor)
    }
}

// Record which byte callback ran; accepting strings would hide R3.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ByteKey<const BUFFER: bool>(pub Vec<u8>, pub bool);
impl<'de, const BUFFER: bool> Deserialize<'de> for ByteKey<BUFFER> {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        struct BytesVisitor<const BUFFER: bool>;
        impl<'de, const BUFFER: bool> Visitor<'de> for BytesVisitor<BUFFER> {
            type Value = ByteKey<BUFFER>;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a byte key")
            }
            fn visit_borrowed_bytes<E: serde::de::Error>(
                self,
                v: &'de [u8],
            ) -> Result<Self::Value, E> {
                Ok(ByteKey(v.to_vec(), true))
            }
            fn visit_bytes<E: serde::de::Error>(self, v: &[u8]) -> Result<Self::Value, E> {
                Ok(ByteKey(v.to_vec(), false))
            }
        }
        if BUFFER {
            de.deserialize_byte_buf(BytesVisitor::<BUFFER>)
        } else {
            de.deserialize_bytes(BytesVisitor::<BUFFER>)
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnitKey<const STRUCT: bool = false>;
impl<'de, const STRUCT: bool> Deserialize<'de> for UnitKey<STRUCT> {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        struct UnitVisitor<const STRUCT: bool>;
        impl<'de, const STRUCT: bool> Visitor<'de> for UnitVisitor<STRUCT> {
            type Value = UnitKey<STRUCT>;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a string representing unit")
            }
            fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                Ok(UnitKey)
            }
            fn visit_str<E: serde::de::Error>(self, _: &str) -> Result<Self::Value, E> {
                Ok(UnitKey)
            }
        }
        if STRUCT {
            de.deserialize_unit_struct("UnitKey", UnitVisitor::<STRUCT>)
        } else {
            de.deserialize_unit(UnitVisitor::<STRUCT>)
        }
    }
}
