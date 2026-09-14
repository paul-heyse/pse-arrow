// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Serde's value adapters decode hydrated syntax into generated document contracts.

use super::value::Value;
use serde::Deserializer;
use serde::de::{self, IntoDeserializer, Visitor};

type Error = de::value::Error;

impl IntoDeserializer<'_, Error> for Value {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self {
        self
    }
}

impl<'de> Deserializer<'de> for Value {
    type Error = Error;
    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self {
            Self::Null => visitor.visit_unit(),
            Self::Bool(value) => visitor.visit_bool(value),
            Self::I64(value) => visitor.visit_i64(value),
            Self::U64(value) => visitor.visit_u64(value),
            Self::F64(value) => visitor.visit_f64(value),
            Self::Text(value) => visitor.visit_string(value),
            Self::List(values) => visitor.visit_seq(de::value::SeqDeserializer::new(
                values.into_iter().map(|value| value.value),
            )),
            Self::Map(values) => visitor.visit_map(de::value::MapDeserializer::new(
                values
                    .into_iter()
                    .map(|(key, value)| (key.value, value.value)),
            )),
        }
    }
    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        if matches!(self, Self::Null) {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }
    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        match self {
            Self::Text(value) => visitor.visit_enum(value.into_deserializer()),
            value => value.deserialize_any(visitor),
        }
    }
    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        visitor.visit_newtype_struct(self)
    }
    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct seq tuple tuple_struct map struct identifier ignored_any
    }
}
