// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Strict inverse of the sole tagged Cell literal rendering.
use super::Cell;
use crate::{Registry, SchemaError};
use serde_json::Value;

pub(super) fn parse(text: &str, registry: &Registry) -> Result<Cell, SchemaError> {
    let value = serde_json::from_str(text).map_err(|error| invalid(error.to_string()))?;
    decode(&value, registry)
}
fn invalid(reason: impl Into<String>) -> SchemaError {
    SchemaError::InvalidDeclaration {
        context: "tagged cell literal".to_owned(),
        reason: reason.into(),
    }
}
fn decode(value: &Value, registry: &Registry) -> Result<Cell, SchemaError> {
    let values = value
        .as_array()
        .filter(|parts| parts.len() == 2)
        .ok_or_else(|| invalid("literal must contain exactly a tag and value"))?;
    let tag = values[0]
        .as_str()
        .ok_or_else(|| invalid("literal tag is not text"))?;
    let value = &values[1];
    let text = || {
        value
            .as_str()
            .ok_or_else(|| invalid("literal value must be text"))
    };
    Ok(match tag {
        "null" if value.is_null() => Cell::Null,
        "bool" => Cell::Bool(
            value
                .as_bool()
                .ok_or_else(|| invalid("Boolean value expected"))?,
        ),
        "i64" => Cell::I64(
            value
                .as_i64()
                .ok_or_else(|| invalid("exact signed 64-bit integer expected"))?,
        ),
        "u64" => Cell::U64(
            value
                .as_u64()
                .ok_or_else(|| invalid("exact unsigned 64-bit integer expected"))?,
        ),
        "f64" => {
            let bits = text()?;
            if bits.len() != 16
                || bits
                    .bytes()
                    .any(|byte| !byte.is_ascii_digit() && !(b'a'..=b'f').contains(&byte))
            {
                return Err(invalid(
                    "float requires exactly 16 lowercase hexadecimal bits",
                ));
            }
            Cell::F64(f64::from_bits(
                u64::from_str_radix(bits, 16).map_err(|error| invalid(error.to_string()))?,
            ))
        }
        "text" => Cell::text(text()?),
        "id" => Cell::Id(
            pse_ids::SemanticId::parse_hex(text()?).map_err(|error| invalid(error.to_string()))?,
        ),
        "hash" => Cell::Hash(
            pse_ids::ContentHash::parse_hex(text()?).map_err(|error| invalid(error.to_string()))?,
        ),
        "enum" => {
            let name = text()?;
            let member = registry
                .enums()
                .iter()
                .flat_map(|spec| &spec.members)
                .find(|member| member.name == name)
                .ok_or_else(|| {
                    invalid("enum member spelling is not declared in the supplied registry")
                })?;
            Cell::Enum(member.name)
        }
        "list" | "struct" => {
            let values = value
                .as_array()
                .ok_or_else(|| invalid("composite literal must be an array"))?
                .iter()
                .map(|value| decode(value, registry))
                .collect::<Result<Vec<_>, _>>()?;
            if tag == "list" {
                Cell::List(values)
            } else {
                Cell::Struct(values)
            }
        }
        _ => return Err(invalid("unknown literal tag or incompatible value")),
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        RegistryBuilder,
        model::{Cell, EnumDecl, EnumMember},
    };
    #[test]
    fn tagged_literals_round_trip_every_shape_and_float_bit_without_narrowing() {
        let mut builder = RegistryBuilder::new();
        builder.declare_enum(EnumDecl::platform(
            "Fixture",
            vec![EnumMember::new("member", "member")],
        ));
        let registry = builder.build().unwrap();
        let values = Cell::Struct(vec![
            Cell::Null,
            Cell::Bool(true),
            Cell::I64(i64::MIN),
            Cell::U64(u64::MAX),
            Cell::F64(-0.0),
            Cell::F64(f64::from_bits(0x7ff8_0000_0000_002a)),
            Cell::Text("quoted\"\\\n".into()),
            Cell::Id(pse_ids::SemanticId::NIL),
            Cell::Hash(pse_ids::ContentHash::from_bytes([0; 32])),
            Cell::Enum("member"),
            Cell::List(vec![Cell::U64(1)]),
        ]);
        let text = values.literal_spec();
        let decoded = Cell::from_literal_spec(&text, &registry).unwrap();
        assert_eq!(text, decoded.literal_spec());
        for text in [
            "[\"i64\",9223372036854775808]",
            "[\"u64\",-1]",
            "[\"f64\",\"0\"]",
            "[\"enum\",\"absent\"]",
            "[\"text\",3]",
            "[\"null\",false]",
            "[\"list\",{}]",
        ] {
            assert!(Cell::from_literal_spec(text, &registry).is_err());
        }
    }
}
