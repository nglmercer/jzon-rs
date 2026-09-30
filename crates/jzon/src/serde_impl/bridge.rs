//! Change only borrowed callbacks to transient callbacks for native reader tokens.
//! User seeds and visitors are invoked once; this adapter never retries.
use super::*;
use core::marker::PhantomData;
use serde::de::{DeserializeSeed, Visitor};
pub(super) struct Bridge<'short, D>(pub D, pub PhantomData<&'short ()>);
pub(super) struct BridgeVisitor<'de, V>(pub V, pub PhantomData<&'de ()>);
struct BridgeSeed<'de, S>(S, PhantomData<&'de ()>);
impl<'short, 'de, S: DeserializeSeed<'de>> DeserializeSeed<'short> for BridgeSeed<'de, S> {
    type Value = S::Value;
    fn deserialize<D: serde::Deserializer<'short>>(self, de: D) -> Result<Self::Value, D::Error> {
        self.0.deserialize(Bridge(de, PhantomData))
    }
}
impl<'short, 'de, D: serde::Deserializer<'short>> serde::Deserializer<'de> for Bridge<'short, D> {
    type Error = D::Error;
    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.0.deserialize_any(BridgeVisitor(visitor, PhantomData))
    }
    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.0.deserialize_bool(BridgeVisitor(visitor, PhantomData))
    }
    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.0.deserialize_i8(BridgeVisitor(visitor, PhantomData))
    }
    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.0.deserialize_i16(BridgeVisitor(visitor, PhantomData))
    }
    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.0.deserialize_i32(BridgeVisitor(visitor, PhantomData))
    }
    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.0.deserialize_i64(BridgeVisitor(visitor, PhantomData))
    }
    fn deserialize_i128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.0.deserialize_i128(BridgeVisitor(visitor, PhantomData))
    }
    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.0.deserialize_u8(BridgeVisitor(visitor, PhantomData))
    }
    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.0.deserialize_u16(BridgeVisitor(visitor, PhantomData))
    }
    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.0.deserialize_u32(BridgeVisitor(visitor, PhantomData))
    }
    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.0.deserialize_u64(BridgeVisitor(visitor, PhantomData))
    }
    fn deserialize_u128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.0.deserialize_u128(BridgeVisitor(visitor, PhantomData))
    }
    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.0.deserialize_f32(BridgeVisitor(visitor, PhantomData))
    }
    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.0.deserialize_f64(BridgeVisitor(visitor, PhantomData))
    }
    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.0.deserialize_char(BridgeVisitor(visitor, PhantomData))
    }
    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.0.deserialize_str(BridgeVisitor(visitor, PhantomData))
    }
    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.0
            .deserialize_string(BridgeVisitor(visitor, PhantomData))
    }
    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.0
            .deserialize_bytes(BridgeVisitor(visitor, PhantomData))
    }
    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.0
            .deserialize_byte_buf(BridgeVisitor(visitor, PhantomData))
    }
    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.0
            .deserialize_option(BridgeVisitor(visitor, PhantomData))
    }
    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.0.deserialize_unit(BridgeVisitor(visitor, PhantomData))
    }
    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, D::Error> {
        self.0
            .deserialize_unit_struct(name, BridgeVisitor(visitor, PhantomData))
    }
    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, D::Error> {
        self.0
            .deserialize_newtype_struct(name, BridgeVisitor(visitor, PhantomData))
    }
    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.0.deserialize_seq(BridgeVisitor(visitor, PhantomData))
    }
    fn deserialize_tuple<V: Visitor<'de>>(
        self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, D::Error> {
        self.0
            .deserialize_tuple(len, BridgeVisitor(visitor, PhantomData))
    }
    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, D::Error> {
        self.0
            .deserialize_tuple_struct(name, len, BridgeVisitor(visitor, PhantomData))
    }
    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.0.deserialize_map(BridgeVisitor(visitor, PhantomData))
    }
    fn deserialize_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, D::Error> {
        self.0
            .deserialize_struct(name, fields, BridgeVisitor(visitor, PhantomData))
    }
    fn deserialize_enum<V: Visitor<'de>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, D::Error> {
        self.0
            .deserialize_enum(name, variants, BridgeVisitor(visitor, PhantomData))
    }
    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.0
            .deserialize_identifier(BridgeVisitor(visitor, PhantomData))
    }
    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.0
            .deserialize_ignored_any(BridgeVisitor(visitor, PhantomData))
    }
    fn is_human_readable(&self) -> bool {
        self.0.is_human_readable()
    }
}
impl<'short, 'de, V: Visitor<'de>> Visitor<'short> for BridgeVisitor<'de, V> {
    type Value = V::Value;
    fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.0.expecting(f)
    }
    fn visit_bool<E: serde::de::Error>(self, value: bool) -> Result<Self::Value, E> {
        self.0.visit_bool(value)
    }
    fn visit_i8<E: serde::de::Error>(self, value: i8) -> Result<Self::Value, E> {
        self.0.visit_i8(value)
    }
    fn visit_i16<E: serde::de::Error>(self, value: i16) -> Result<Self::Value, E> {
        self.0.visit_i16(value)
    }
    fn visit_i32<E: serde::de::Error>(self, value: i32) -> Result<Self::Value, E> {
        self.0.visit_i32(value)
    }
    fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<Self::Value, E> {
        self.0.visit_i64(value)
    }
    fn visit_i128<E: serde::de::Error>(self, value: i128) -> Result<Self::Value, E> {
        self.0.visit_i128(value)
    }
    fn visit_u8<E: serde::de::Error>(self, value: u8) -> Result<Self::Value, E> {
        self.0.visit_u8(value)
    }
    fn visit_u16<E: serde::de::Error>(self, value: u16) -> Result<Self::Value, E> {
        self.0.visit_u16(value)
    }
    fn visit_u32<E: serde::de::Error>(self, value: u32) -> Result<Self::Value, E> {
        self.0.visit_u32(value)
    }
    fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<Self::Value, E> {
        self.0.visit_u64(value)
    }
    fn visit_u128<E: serde::de::Error>(self, value: u128) -> Result<Self::Value, E> {
        self.0.visit_u128(value)
    }
    fn visit_f32<E: serde::de::Error>(self, value: f32) -> Result<Self::Value, E> {
        self.0.visit_f32(value)
    }
    fn visit_f64<E: serde::de::Error>(self, value: f64) -> Result<Self::Value, E> {
        self.0.visit_f64(value)
    }
    fn visit_char<E: serde::de::Error>(self, value: char) -> Result<Self::Value, E> {
        self.0.visit_char(value)
    }
    fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
        self.0.visit_str(value)
    }
    fn visit_borrowed_str<E: serde::de::Error>(self, value: &'short str) -> Result<Self::Value, E> {
        self.0.visit_str(value)
    }
    fn visit_string<E: serde::de::Error>(self, value: String) -> Result<Self::Value, E> {
        self.0.visit_string(value)
    }
    fn visit_bytes<E: serde::de::Error>(self, value: &[u8]) -> Result<Self::Value, E> {
        self.0.visit_bytes(value)
    }
    fn visit_borrowed_bytes<E: serde::de::Error>(
        self,
        value: &'short [u8],
    ) -> Result<Self::Value, E> {
        self.0.visit_bytes(value)
    }
    fn visit_byte_buf<E: serde::de::Error>(self, value: Vec<u8>) -> Result<Self::Value, E> {
        self.0.visit_byte_buf(value)
    }
    fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
        self.0.visit_none()
    }
    fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
        self.0.visit_unit()
    }
    fn visit_some<D: serde::Deserializer<'short>>(self, de: D) -> Result<Self::Value, D::Error> {
        self.0.visit_some(Bridge(de, PhantomData))
    }
    fn visit_newtype_struct<D: serde::Deserializer<'short>>(
        self,
        de: D,
    ) -> Result<Self::Value, D::Error> {
        self.0.visit_newtype_struct(Bridge(de, PhantomData))
    }
    fn visit_seq<A: SeqAccess<'short>>(self, access: A) -> Result<Self::Value, A::Error> {
        self.0.visit_seq(Bridge(access, PhantomData))
    }
    fn visit_map<A: MapAccess<'short>>(self, access: A) -> Result<Self::Value, A::Error> {
        self.0.visit_map(Bridge(access, PhantomData))
    }
    fn visit_enum<A: EnumAccess<'short>>(self, access: A) -> Result<Self::Value, A::Error> {
        self.0.visit_enum(Bridge(access, PhantomData))
    }
}
impl<'short, 'de, A: SeqAccess<'short>> SeqAccess<'de> for Bridge<'short, A> {
    type Error = A::Error;
    fn next_element_seed<S: DeserializeSeed<'de>>(
        &mut self,
        seed: S,
    ) -> Result<Option<S::Value>, Self::Error> {
        self.0.next_element_seed(BridgeSeed(seed, PhantomData))
    }
    fn size_hint(&self) -> Option<usize> {
        self.0.size_hint()
    }
}
impl<'short, 'de, A: MapAccess<'short>> MapAccess<'de> for Bridge<'short, A> {
    type Error = A::Error;
    fn next_key_seed<S: DeserializeSeed<'de>>(
        &mut self,
        seed: S,
    ) -> Result<Option<S::Value>, Self::Error> {
        self.0.next_key_seed(BridgeSeed(seed, PhantomData))
    }
    fn next_value_seed<S: DeserializeSeed<'de>>(
        &mut self,
        seed: S,
    ) -> Result<S::Value, Self::Error> {
        self.0.next_value_seed(BridgeSeed(seed, PhantomData))
    }
    fn size_hint(&self) -> Option<usize> {
        self.0.size_hint()
    }
}
impl<'short, 'de, A: EnumAccess<'short>> EnumAccess<'de> for Bridge<'short, A> {
    type Error = A::Error;
    type Variant = Bridge<'short, A::Variant>;
    fn variant_seed<S: DeserializeSeed<'de>>(
        self,
        seed: S,
    ) -> Result<(S::Value, Self::Variant), Self::Error> {
        let (v, a) = self.0.variant_seed(BridgeSeed(seed, PhantomData))?;
        Ok((v, Bridge(a, PhantomData)))
    }
}
impl<'short, 'de, A: VariantAccess<'short>> VariantAccess<'de> for Bridge<'short, A> {
    type Error = A::Error;
    fn unit_variant(self) -> Result<(), Self::Error> {
        self.0.unit_variant()
    }
    fn newtype_variant_seed<S: DeserializeSeed<'de>>(
        self,
        seed: S,
    ) -> Result<S::Value, Self::Error> {
        self.0.newtype_variant_seed(BridgeSeed(seed, PhantomData))
    }
    fn tuple_variant<V: Visitor<'de>>(
        self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.0
            .tuple_variant(len, BridgeVisitor(visitor, PhantomData))
    }
    fn struct_variant<V: Visitor<'de>>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.0
            .struct_variant(fields, BridgeVisitor(visitor, PhantomData))
    }
}
