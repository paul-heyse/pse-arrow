// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Closed typed configuration grammar; no callbacks or hash-based certificates.
use super::super::invalid;
use crate::CompilerError;
use pse_authoring::SourceSpan;
use pse_ids::SemanticId;
use pse_quantity::{QuantityRegistry, QuantityTypeId, ScaleKind, UnitId};
use pse_relations::generated::{
    enums::ConfigValueKind as Kind,
    extension_values::ExtensionQuantityValue,
    normalized::config_values::{self as cv, NormalizedConfigValuesFieldValue as Value},
};
use pse_schema::Registry;

pub(super) fn parse(
    text: &str,
    logical: &str,
    enum_id: Option<SemanticId>,
    registry: &Registry,
    physical: Option<&QuantityRegistry>,
) -> Result<Value, CompilerError> {
    let value = match logical {
        "bool" => Value::from_boolean(cv::NormalizedConfigValuesFieldValueBoolean {
            value: match text {
                "true" => true,
                "false" => false,
                _ => return Err(invalid("Boolean configuration requires true or false")),
            },
        }),
        "i32" | "i64" => {
            let n = text
                .parse::<i64>()
                .map_err(|_| invalid("invalid signed configuration integer"))?;
            if logical == "i32" && i32::try_from(n).is_err() {
                return Err(invalid("configuration integer exceeds declared i32 range"));
            }
            Value::from_signed(cv::NormalizedConfigValuesFieldValueSigned { value: n })
        }
        "u8" | "u16" | "u32" | "u64" => {
            let n = text
                .parse::<u64>()
                .map_err(|_| invalid("invalid unsigned configuration integer"))?;
            let valid = match logical {
                "u8" => u8::try_from(n).is_ok(),
                "u16" => u16::try_from(n).is_ok(),
                "u32" => u32::try_from(n).is_ok(),
                _ => true,
            };
            if !valid {
                return Err(invalid(
                    "configuration integer exceeds declared unsigned range",
                ));
            }
            Value::from_unsigned(cv::NormalizedConfigValuesFieldValueUnsigned { value: n })
        }
        "f64" => Value::from_real(cv::NormalizedConfigValuesFieldValueReal {
            value: finite(text)?,
        }),
        "text" => Value::from_text(cv::NormalizedConfigValuesFieldValueText {
            value: text.to_owned(),
        }),
        "semantic_id" => Value::from_semantic_id(cv::NormalizedConfigValuesFieldValueSemanticId {
            value: id(text)?,
        }),
        "index_tuple" => {
            let members: Vec<String> = serde_json::from_str(text).map_err(|_| {
                invalid("index configuration requires an array of semantic ID strings")
            })?;
            Value::from_index(cv::NormalizedConfigValuesFieldValueIndex {
                value: members
                    .iter()
                    .map(|member| id(member))
                    .collect::<Result<_, _>>()?,
            })
        }
        "quantity_value" => parse_quantity(text, physical)?,
        name if name.starts_with("enum:") || name == "enum" => {
            let declared = enum_id
                .and_then(|id| registry.enums().iter().find(|item| item.id == id))
                .ok_or_else(|| {
                    invalid("enum configuration lacks an actual registered dictionary")
                })?;
            if name != "enum" && name.strip_prefix("enum:") != Some(declared.name) {
                return Err(invalid(
                    "parameter enum identity disagrees with its logical type",
                ));
            }
            if !declared
                .members
                .iter()
                .any(|member| member.name == text && !member.deprecated)
            {
                return Err(invalid("configuration enum member is absent or deprecated"));
            }
            Value::from_enum(cv::NormalizedConfigValuesFieldValueEnumeration {
                enum_id: declared.id,
                member: text.to_owned(),
            })
        }
        _ => {
            return Err(invalid(
                "logical type has no declared scalar configuration grammar",
            ));
        }
    };
    if value.kind != Kind::Enum && enum_id.is_some() {
        return Err(invalid(
            "non-enum configuration declaration carries an enum dictionary",
        ));
    }
    Ok(value)
}

fn parse_quantity(text: &str, physical: Option<&QuantityRegistry>) -> Result<Value, CompilerError> {
    let physical = physical.ok_or_else(|| {
        invalid("quantity configuration requires actual physical registry bindings")
    })?;
    let json: serde_json::Value = serde_json::from_str(text)
        .map_err(|_| invalid("quantity configuration requires its exact JSON object"))?;
    let fields = json
        .as_object()
        .filter(|fields| fields.len() == 3)
        .ok_or_else(|| {
            invalid("quantity value must have value, quantity_type_id and unit_id only")
        })?;
    let number = fields
        .get("value")
        .and_then(serde_json::Value::as_f64)
        .filter(|n| n.is_finite())
        .ok_or_else(|| invalid("quantity value must be finite"))?;
    let quantity_id = id(fields
        .get("quantity_type_id")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| invalid("quantity_type_id must be an explicit semantic ID"))?)?;
    let unit_id = id(fields
        .get("unit_id")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| invalid("unit_id must be an explicit semantic ID"))?)?;
    let quantity = physical.quantity_type(QuantityTypeId::from_id(quantity_id))?;
    let unit = physical.unit(UnitId::from_id(unit_id))?;
    let canonical = physical.unit(quantity.canonical_unit)?;
    if unit.dimension != physical.kind(quantity.key.kind)?.dimension
        || unit.reference_state != canonical.reference_state
        || (quantity.key.scale_kind == ScaleKind::Difference && unit.is_affine)
    {
        return Err(invalid(
            "quantity configuration unit has incompatible dimension, datum or scale kind",
        ));
    }
    Ok(Value::from_quantity(ExtensionQuantityValue {
        value: number,
        quantity_type_id: quantity_id,
        unit_id,
    }))
}

/// Transfer an already parsed value under the child's actual declared type.
/// Enum identity, quantity payload and exact integer/float values survive binding.
pub(super) fn bind(
    value: &Value,
    logical: &str,
    enum_id: Option<SemanticId>,
    registry: &Registry,
) -> Result<Value, CompilerError> {
    let valid = match logical {
        "bool" => value.kind == Kind::Boolean,
        "i32" => {
            value.kind == Kind::Signed
                && value
                    .signed
                    .as_ref()
                    .map(|arm| arm.value)
                    .is_some_and(|n| i32::try_from(n).is_ok())
        }
        "i64" => value.kind == Kind::Signed,
        "u8" => {
            value.kind == Kind::Unsigned
                && value
                    .unsigned
                    .as_ref()
                    .map(|arm| arm.value)
                    .is_some_and(|n| u8::try_from(n).is_ok())
        }
        "u16" => {
            value.kind == Kind::Unsigned
                && value
                    .unsigned
                    .as_ref()
                    .map(|arm| arm.value)
                    .is_some_and(|n| u16::try_from(n).is_ok())
        }
        "u32" => {
            value.kind == Kind::Unsigned
                && value
                    .unsigned
                    .as_ref()
                    .map(|arm| arm.value)
                    .is_some_and(|n| u32::try_from(n).is_ok())
        }
        "u64" => value.kind == Kind::Unsigned,
        "f64" => value.kind == Kind::Real,
        "text" => value.kind == Kind::Text,
        "semantic_id" => value.kind == Kind::SemanticId,
        "index_tuple" => value.kind == Kind::Index,
        "quantity_value" => value.kind == Kind::Quantity,
        name if name.starts_with("enum:") || name == "enum" => {
            value.kind == Kind::Enum
                && enum_id.is_some()
                && value.enumeration.as_ref().map(|arm| arm.enum_id) == enum_id
                && registry.enums().iter().any(|declared| {
                    Some(declared.id) == enum_id
                        && (name == "enum" || name.strip_prefix("enum:") == Some(declared.name))
                })
        }
        _ => false,
    };
    if !valid || (value.kind != Kind::Enum && enum_id.is_some()) {
        return Err(invalid(
            "parent configuration value has a different declared child type",
        ));
    }
    Ok(value.clone())
}

pub(super) fn numeric_constraint(value: &Value, spec: &str) -> Result<bool, CompilerError> {
    if !(matches!(spec, "positive" | "nonnegative") || spec.starts_with("range[")) {
        return Ok(false);
    }
    let valid = match spec {
        "positive" => {
            floating_value(value).is_some_and(|n| n > 0.0)
                || value
                    .signed
                    .as_ref()
                    .map(|arm| arm.value)
                    .is_some_and(|n| n > 0)
                || value
                    .unsigned
                    .as_ref()
                    .map(|arm| arm.value)
                    .is_some_and(|n| n > 0)
        }
        "nonnegative" => {
            floating_value(value).is_some_and(|n| n >= 0.0)
                || value
                    .signed
                    .as_ref()
                    .map(|arm| arm.value)
                    .is_some_and(|n| n >= 0)
                || value.unsigned.is_some()
        }
        _ => range(value, spec)?,
    };
    if valid {
        Ok(true)
    } else {
        Err(invalid(
            "configuration violates its declared numeric domain",
        ))
    }
}
fn range(value: &Value, spec: &str) -> Result<bool, CompilerError> {
    let (lower, upper) = spec
        .strip_prefix("range[")
        .and_then(|s| s.strip_suffix(']'))
        .and_then(|s| s.split_once(','))
        .ok_or_else(|| invalid("range requires range[lower,upper]"))?;
    if let Some(number) = value.signed.as_ref().map(|arm| arm.value) {
        let (lower, upper) = (lower.parse::<i64>(), upper.parse::<i64>());
        return match (lower, upper) {
            (Ok(l), Ok(u)) if l <= u => Ok(l <= number && number <= u),
            _ => Err(invalid(
                "signed range endpoints must be ordered exact integers",
            )),
        };
    }
    if let Some(number) = value.unsigned.as_ref().map(|arm| arm.value) {
        let (lower, upper) = (lower.parse::<u64>(), upper.parse::<u64>());
        return match (lower, upper) {
            (Ok(l), Ok(u)) if l <= u => Ok(l <= number && number <= u),
            _ => Err(invalid(
                "unsigned range endpoints must be ordered exact integers",
            )),
        };
    }
    let (lower, upper) = (finite(lower)?, finite(upper)?);
    if lower > upper {
        return Err(invalid("range endpoints are reversed"));
    }
    Ok(floating_value(value).is_some_and(|number| lower <= number && number <= upper))
}
fn floating_value(value: &Value) -> Option<f64> {
    value
        .real
        .as_ref()
        .map(|arm| arm.value)
        .or_else(|| value.quantity.as_ref().map(|arm| arm.value))
}
fn finite(text: &str) -> Result<f64, CompilerError> {
    text.parse::<f64>()
        .ok()
        .filter(|n| n.is_finite())
        .ok_or_else(|| invalid("configuration number must be a finite f64"))
}
pub(super) fn id(text: &str) -> Result<SemanticId, CompilerError> {
    Ok(pse_authoring::ids::parse_id(
        text,
        SourceSpan {
            document_id: SemanticId::NIL,
            start: 0,
            end: 0,
        },
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_quantity::standard::{ids, standard_registry};
    #[test]
    fn typed_integer_constraints_preserve_values_above_float_precision() {
        let registry = pse_schema::registry().unwrap();
        let physical = standard_registry().unwrap();
        let value = parse("9007199254740993", "u64", None, registry, Some(&physical)).unwrap();
        assert_eq!(
            value.unsigned.as_ref().unwrap().value,
            9_007_199_254_740_993
        );
        assert!(numeric_constraint(&value, "range[9007199254740993,9007199254740993]").is_ok());
        assert!(numeric_constraint(&value, "range[9007199254740992,9007199254740992]").is_err());
        assert!(parse("256", "u8", None, registry, Some(&physical)).is_err());
        assert!(parse("-1", "u64", None, registry, Some(&physical)).is_err());
    }
    #[test]
    fn typed_configuration_rejects_nonfinite_and_preserves_signed_zero() {
        let registry = pse_schema::registry().unwrap();
        let physical = standard_registry().unwrap();
        for text in ["NaN", "inf", "-inf", "1e999"] {
            assert!(parse(text, "f64", None, registry, Some(&physical)).is_err());
        }
        assert_eq!(
            parse("-0.0", "f64", None, registry, Some(&physical))
                .unwrap()
                .real
                .unwrap()
                .value
                .to_bits(),
            (-0.0_f64).to_bits()
        );
        for text in ["inherit", "null", "0", "True"] {
            assert!(parse(text, "bool", None, registry, Some(&physical)).is_err());
        }
    }
    #[test]
    fn typed_configuration_checks_actual_enum_dictionary() {
        let registry = pse_schema::registry().unwrap();
        let physical = standard_registry().unwrap();
        let selection = registry.enum_spec("MaterialBalanceType").unwrap();
        assert!(
            parse(
                "componentTotal",
                "enum:MaterialBalanceType",
                Some(selection.id),
                registry,
                Some(&physical)
            )
            .is_ok()
        );
        assert!(
            parse(
                "invented",
                "enum:MaterialBalanceType",
                Some(selection.id),
                registry,
                Some(&physical)
            )
            .is_err()
        );
        assert!(
            parse(
                "componentTotal",
                "enum:EnergyBalanceType",
                Some(selection.id),
                registry,
                Some(&physical)
            )
            .is_err()
        );
        assert!(
            parse(
                "true",
                "bool",
                Some(selection.id),
                registry,
                Some(&physical)
            )
            .is_err()
        );
    }
    #[test]
    fn child_bindings_preserve_actual_types_and_exact_values() {
        let registry = pse_schema::registry().unwrap();
        let material = registry.enum_spec("MaterialBalanceType").unwrap();
        let energy = registry.enum_spec("EnergyBalanceType").unwrap();
        let value = parse("none", "enum", Some(material.id), registry, None).unwrap();
        assert!(bind(&value, "enum", Some(material.id), registry).is_ok());
        assert!(bind(&value, "enum", Some(energy.id), registry).is_err());
        assert!(bind(&value, "text", None, registry).is_err());
        let integer = parse("9007199254740993", "u64", None, registry, None).unwrap();
        assert_eq!(
            bind(&integer, "u64", None, registry)
                .unwrap()
                .unsigned
                .unwrap()
                .value,
            9_007_199_254_740_993
        );
        assert!(bind(&integer, "u32", None, registry).is_err());
        assert!(bind(&integer, "bool", None, registry).is_err());
        let negative_zero = parse("-0.0", "f64", None, registry, None).unwrap();
        assert_eq!(
            bind(&negative_zero, "f64", None, registry)
                .unwrap()
                .real
                .unwrap()
                .value
                .to_bits(),
            (-0.0_f64).to_bits()
        );
    }
    #[test]
    fn quantity_configuration_checks_actual_units_and_exact_fields() {
        let registry = pse_schema::registry().unwrap();
        let physical = standard_registry().unwrap();
        let quantity = physical
            .quantity_types()
            .find(|ty| ty.id == ids::quantity("temperature.point"))
            .unwrap();
        let input = serde_json::json!({"value":300.0,"quantity_type_id":quantity.id.as_id().to_hex(),"unit_id":quantity.canonical_unit.as_id().to_hex()});
        let value = parse(
            &input.to_string(),
            "quantity_value",
            None,
            registry,
            Some(&physical),
        )
        .unwrap();
        assert!(numeric_constraint(&value, "positive").is_ok());
        assert!(numeric_constraint(&value, "nonnegative").is_ok());
        assert!(numeric_constraint(&value, "range[299,301]").is_ok());
        assert!(numeric_constraint(&value, "range[301,302]").is_err());
        let mut wrong = input.clone();
        wrong["unit_id"] = serde_json::Value::String(ids::unit("m").as_id().to_hex());
        assert!(
            parse(
                &wrong.to_string(),
                "quantity_value",
                None,
                registry,
                Some(&physical)
            )
            .is_err()
        );
        let mut extra = input;
        extra["approved"] = serde_json::Value::Bool(true);
        assert!(
            parse(
                &extra.to_string(),
                "quantity_value",
                None,
                registry,
                Some(&physical)
            )
            .is_err()
        );
    }
}
