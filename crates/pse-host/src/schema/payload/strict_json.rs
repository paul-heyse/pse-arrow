// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One token preflight before generated decoding or dynamic Any conversion.
use super::SchemaError;
use buffa_descriptor::{DescriptorPool, FieldKind, MessageDescriptor, SingularKind};
use serde::de::{self, DeserializeSeed, Visitor};
use std::{collections::BTreeSet, fmt};

/// Generated Serde readers ignore unknown names; only a matched compiled world
/// supplies descriptor key policy here. Acquired schemas keep their dynamic reader.
pub(super) fn check(
    bytes: &[u8],
    compiled: Option<(&DescriptorPool, &MessageDescriptor)>,
) -> Result<(), SchemaError> {
    let mut refusal = None;
    let mut decoder = serde_json::Deserializer::from_slice(bytes);
    Tokens {
        shape: compiled.map_or(Shape::Untyped, |(pool, message)| {
            Shape::message(pool, message)
        }),
        refusal: &mut refusal,
    }
    .deserialize(&mut decoder)?;
    decoder.end()?;
    // Preserve the original token preflight's precedence: malformed JSON,
    // literal duplicates and trailing input precede descriptor-key refusals.
    refusal.map_or(Ok(()), |reason| Err(SchemaError::Decode(reason)))
}

#[derive(Clone, Copy)]
enum Shape<'a> {
    Untyped,
    Message(&'a DescriptorPool, &'a MessageDescriptor),
    List(&'a DescriptorPool, SingularKind),
    Map(&'a DescriptorPool, SingularKind),
}
impl<'a> Shape<'a> {
    fn message(pool: &'a DescriptorPool, message: &'a MessageDescriptor) -> Self {
        match message.full_name() {
            // These types own arbitrary JSON rather than descriptor field names.
            "google.protobuf.Struct" | "google.protobuf.Value" | "google.protobuf.ListValue" => {
                Self::Untyped
            }
            _ => Self::Message(pool, message),
        }
    }
    fn singular(pool: &'a DescriptorPool, kind: SingularKind) -> Self {
        match kind {
            SingularKind::Message(index) => Self::message(pool, pool.message(index)),
            _ => Self::Untyped,
        }
    }
    fn field(pool: &'a DescriptorPool, kind: FieldKind) -> Self {
        match kind {
            FieldKind::Singular(kind) => Self::singular(pool, kind),
            FieldKind::List(kind) => Self::List(pool, kind),
            FieldKind::Map { value, .. } => Self::Map(pool, value),
        }
    }
}

struct Tokens<'a, 'r> {
    shape: Shape<'a>,
    refusal: &'r mut Option<String>,
}
impl<'de> DeserializeSeed<'de> for Tokens<'_, '_> {
    type Value = ();
    fn deserialize<D: de::Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        deserializer.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for Tokens<'_, '_> {
    type Value = ();
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("JSON without duplicate object members")
    }
    fn visit_bool<E: de::Error>(self, _: bool) -> Result<(), E> {
        Ok(())
    }
    fn visit_i64<E: de::Error>(self, _: i64) -> Result<(), E> {
        Ok(())
    }
    fn visit_u64<E: de::Error>(self, _: u64) -> Result<(), E> {
        Ok(())
    }
    fn visit_f64<E: de::Error>(self, _: f64) -> Result<(), E> {
        Ok(())
    }
    fn visit_str<E: de::Error>(self, _: &str) -> Result<(), E> {
        Ok(())
    }
    fn visit_unit<E: de::Error>(self) -> Result<(), E> {
        Ok(())
    }
    fn visit_seq<A: de::SeqAccess<'de>>(self, mut sequence: A) -> Result<(), A::Error> {
        let shape = match self.shape {
            Shape::List(pool, kind) => Shape::singular(pool, kind),
            _ => Shape::Untyped,
        };
        while sequence
            .next_element_seed(Tokens {
                shape,
                refusal: &mut *self.refusal,
            })?
            .is_some()
        {}
        Ok(())
    }
    fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<(), A::Error> {
        let mut keys = BTreeSet::new();
        let mut numbers = BTreeSet::new();
        while let Some(key) = map.next_key::<String>()? {
            if !keys.insert(key.clone()) {
                return Err(de::Error::custom(format!("duplicate JSON member: {key}")));
            }
            let shape = match self.shape {
                Shape::Message(pool, message) => {
                    if let Some(field) = message
                        .fields()
                        .iter()
                        .find(|field| field.name() == key || field.json_name() == key)
                    {
                        if !numbers.insert(field.number()) && self.refusal.is_none() {
                            *self.refusal = Some(format!(
                                "duplicate JSON field aliases: {}.{key}",
                                message.full_name()
                            ));
                        }
                        Shape::field(pool, field.kind())
                    } else {
                        if self.refusal.is_none() {
                            *self.refusal =
                                Some(format!("unknown JSON field {}.{key}", message.full_name()));
                        }
                        Shape::Untyped
                    }
                }
                Shape::Map(pool, kind) => Shape::singular(pool, kind),
                _ => Shape::Untyped,
            };
            map.next_value_seed(Tokens {
                shape,
                refusal: &mut *self.refusal,
            })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
