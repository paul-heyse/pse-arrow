// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Serde's value adapters decode hydrated syntax into generated document contracts.

use super::value::Value;
use serde::Deserializer;
use serde::de::{self, IntoDeserializer, Visitor};

type Error = de::value::Error;

impl<'de> IntoDeserializer<'de, Error> for &'de Value {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self {
        self
    }
}

impl<'de> Deserializer<'de> for &'de Value {
    type Error = Error;
    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self {
            Value::Null => visitor.visit_unit(),
            Value::Bool(value) => visitor.visit_bool(*value),
            Value::I64(value) => visitor.visit_i64(*value),
            Value::U64(value) => visitor.visit_u64(*value),
            Value::F64(value) => visitor.visit_f64(*value),
            Value::Text(value) => visitor.visit_borrowed_str(value),
            Value::List(values) => visitor.visit_seq(de::value::SeqDeserializer::new(
                values.iter().map(|value| &value.value),
            )),
            Value::Map(values) => visitor.visit_map(de::value::MapDeserializer::new(
                values
                    .iter()
                    .map(|(key, value)| (key.value.as_str(), &value.value)),
            )),
        }
    }
    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        if matches!(self, Value::Null) {
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
            Value::Text(value) => visitor.visit_enum(value.as_str().into_deserializer()),
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
