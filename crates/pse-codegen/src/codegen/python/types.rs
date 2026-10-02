// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Python annotations and validators projected from logical and physical declarations.

use arrow_schema::DataType;

use crate::SchemaError;
use crate::model::{ExtensionUse, FieldContract};

use super::pascal;

pub(super) struct Type {
    pub annotation: String,
    pub validator: String,
    pub equality_key: String,
    pub item_key: Option<String>,
}

fn scalar(annotation: &str) -> Type {
    Type {
        item_key: None,
        equality_key: if annotation.starts_with("b.")
            || annotation.starts_with("e.")
            || annotation == "datetime"
            || annotation == "v.SemanticId"
            || annotation == "v.ContentHash"
        {
            "v.scalar_key".into()
        } else {
            "v.record_key".into()
        },
        annotation: annotation.to_owned(),
        validator: format!("attrs.validators.instance_of({annotation})"),
    }
}

fn integer(bits: u8, signed: bool) -> Type {
    let lower = if signed { -(1_i128 << (bits - 1)) } else { 0 };
    let upper = if signed {
        (1_i128 << (bits - 1)) - 1
    } else {
        (1_i128 << bits) - 1
    };
    Type {
        item_key: None,
        equality_key: "v.scalar_key".into(),
        annotation: "b.int".to_owned(),
        validator: format!("v.integer_range({lower}, {upper})"),
    }
}

pub(super) fn optional(mut ty: Type, nullable: bool) -> Type {
    if nullable {
        ty.equality_key = format!("v.optional_key({})", ty.equality_key);
        ty.annotation.push_str(" | None");
        ty.validator = format!("attrs.validators.optional({})", ty.validator);
    }
    ty
}

pub(super) fn structure(name: &str, fields: &[(String, Type)]) -> String {
    let attributes = super::identifiers::fields(fields.iter().map(|(name, _)| name.as_str()));
    let key_fields = fields
        .iter()
        .zip(&attributes)
        .map(|((_, ty), attribute)| format!("{}(self.{attribute})", ty.equality_key))
        .collect::<Vec<_>>()
        .join(", ");
    let key_tuple = if key_fields.is_empty() {
        "()".to_owned()
    } else {
        format!("({key_fields},)")
    };
    let fields = fields
        .iter()
        .zip(attributes)
        .map(|((name, ty), attribute)| {
            let rename = if name == &attribute {
                String::new()
            } else {
                format!(", metadata={{v.FIELD_NAME_METADATA: {name:?}}}")
            };
            let alias = if attribute.starts_with('_') {
                format!(", alias={attribute:?}")
            } else {
                String::new()
            };
            format!(
                "    {attribute}: {} = attrs.field(validator={}{rename}{alias})\n",
                ty.annotation, ty.validator
            )
        })
        .collect::<Vec<_>>()
        .concat();
    format!(
        "\n\n@attrs.frozen(kw_only=True)\nclass {name}:\n    \"\"\"Declared relation row or nested value.\"\"\"\n\n{fields}\n    def _pse_equality_key(self) -> v.EqualityKey:\n        return (type(self), {key_tuple})\n"
    )
}

fn list(child: &Type, width: Option<i32>) -> Type {
    let inner = format!(
        "attrs.validators.deep_iterable(member_validator={}, iterable_validator=attrs.validators.instance_of(b.tuple))",
        child.validator
    );
    let validator = width.map_or_else(|| inner.clone(), |width| format!("attrs.validators.and_({inner}, attrs.validators.min_len({width}), attrs.validators.max_len({width}))"));
    Type {
        item_key: Some(child.equality_key.clone()),
        equality_key: format!("v.sequence_key({})", child.equality_key),
        annotation: format!("b.tuple[{}, ...]", child.annotation),
        validator,
    }
}

pub(super) fn logical(
    ty: &FieldContract,
    stem: &str,
    declarations: &mut String,
) -> Result<Type, SchemaError> {
    super::super::native::render(
        &mut PythonPolicy { declarations },
        ty,
        stem,
        super::super::native::Mode::Domain,
    )
}
pub(super) fn storage(
    ty: &DataType,
    stem: &str,
    declarations: &mut String,
) -> Result<Type, SchemaError> {
    super::super::native::storage(&mut PythonPolicy { declarations }, ty, stem)
}
struct PythonPolicy<'a> {
    declarations: &'a mut String,
}
impl super::super::native::Policy for PythonPolicy<'_> {
    type Value = Type;
    fn nullable(&mut self, value: Type, nullable: bool) -> Type {
        optional(value, nullable)
    }
    fn leaf(
        &mut self,
        ty: &FieldContract,
        _stem: &str,
        mode: super::super::native::Mode,
    ) -> Result<Option<Type>, SchemaError> {
        if let Some(range) = crate::model::IntegerRange::from_field(ty.field())? {
            return Ok(Some(integer_domain(range)));
        }
        if let Some(name) = super::super::enum_name(ty.field()) {
            return Ok(Some(scalar(&format!("e.{}", pascal(name)))));
        }
        if matches!(mode, super::super::native::Mode::Domain) {
            // A named structure is declared once in `structures` (Plan 22 X11).
            if let Some(name) = ty.structure_name() {
                return Ok(Some(scalar(&format!("s.{name}"))));
            }
            // A nested identity value annotates as its alias and validates as its base
            // value, as an identity column does (ADR-0115).
            if let Some(identity) = ty.identity() {
                let base = if matches!(ty.extension(), Some(ExtensionUse::ContentHash)) {
                    "ContentHash"
                } else {
                    "SemanticId"
                };
                return Ok(Some(Type {
                    item_key: None,
                    equality_key: "v.scalar_key".into(),
                    annotation: format!(
                        "i.{}",
                        super::super::rust::identities::type_name(identity)
                    ),
                    validator: format!("attrs.validators.instance_of(v.{base})"),
                }));
            }
            if let Some(use_) = ty.extension() {
                if matches!(
                    use_,
                    ExtensionUse::DimensionVector
                        | ExtensionUse::QuantityValue
                        | ExtensionUse::Bound
                        | ExtensionUse::SourceSpan
                ) {
                    return Ok(Some(if matches!(use_, ExtensionUse::DimensionVector) {
                        list(&scalar("v.DimensionVectorItem"), Some(8))
                    } else {
                        scalar(&format!("v.{}", pascal(&use_.name())))
                    }));
                }
            } else if matches!(
                ty.data_type(),
                DataType::FixedSizeBinary(_) | DataType::Dictionary(..)
            ) {
                return Err(super::error(format!(
                    "no native language codec for {}; domain meaning requires an explicit declaration",
                    ty.data_type()
                )));
            }
        }
        Ok(Some(match ty.data_type() {
            DataType::Boolean => Type {
                item_key: None,
                equality_key: "v.scalar_key".into(),
                annotation: "b.bool".into(),
                validator: "v.exact_type(b.bool)".into(),
            },
            DataType::Int16 => integer(16, true),
            DataType::Int32 => integer(32, true),
            DataType::Int64 => integer(64, true),
            DataType::UInt8 => integer(8, false),
            DataType::UInt16 => integer(16, false),
            DataType::UInt32 => integer(32, false),
            DataType::UInt64 => integer(64, false),
            DataType::Float64 => Type {
                item_key: None,
                equality_key: "v.scalar_key".into(),
                annotation: "b.float".into(),
                validator: "v.finite_float".into(),
            },
            DataType::Utf8 => scalar("b.str"),
            DataType::Binary => scalar("b.bytes"),
            DataType::Timestamp(..) => Type {
                item_key: None,
                equality_key: "v.scalar_key".into(),
                annotation: "datetime".into(),
                validator: "v.utc_timestamp".into(),
            },
            DataType::FixedSizeBinary(16) => scalar("v.SemanticId"),
            DataType::FixedSizeBinary(32) => scalar("v.ContentHash"),
            DataType::List(_) | DataType::FixedSizeList(..) | DataType::Struct(_) => {
                return Ok(None);
            }
            other => return Err(super::error(format!("no Python type for {other}"))),
        }))
    }
    fn container(
        &mut self,
        ty: &FieldContract,
        stem: &str,
        children: Vec<Type>,
    ) -> Result<Type, SchemaError> {
        Ok(match ty.data_type() {
            DataType::List(_) => list(&children[0], None),
            DataType::FixedSizeList(_, width) => list(&children[0], Some(width)),
            DataType::Struct(fields) => {
                let fields = fields
                    .iter()
                    .zip(children)
                    .map(|(field, ty)| (field.name().to_owned(), ty))
                    .collect::<Vec<_>>();
                self.declarations.push_str(&structure(stem, &fields));
                self.declarations.push_str(&alternative_check(ty, &fields)?);
                scalar(stem)
            }
            other => return Err(super::error(format!("no Python container for {other}"))),
        })
    }
    fn decorate(&mut self, ty: &FieldContract, mut value: Type) -> Result<Type, SchemaError> {
        if let Some(collection) = crate::model::CollectionContract::from_field(ty.field())?
            && (collection.minimum != 0 || collection.maximum.is_some() || collection.unique)
        {
            let mut checks = vec![value.validator];
            if collection.minimum != 0 {
                checks.push(format!("attrs.validators.min_len({})", collection.minimum));
            }
            if let Some(maximum) = collection.maximum {
                checks.push(format!("attrs.validators.max_len({maximum})"));
            }
            if collection.unique {
                let key = value.item_key.as_deref().ok_or_else(|| {
                    super::error("unique collection lacks its declared element".into())
                })?;
                checks.push(format!("v.unique({key})"));
            }
            value.validator = format!("attrs.validators.and_({})", checks.join(", "));
        }
        Ok(value)
    }
}

fn integer_domain(range: crate::model::IntegerRange) -> Type {
    Type {
        item_key: None,
        equality_key: "v.scalar_key".into(),
        annotation: "b.int".to_owned(),
        validator: format!("v.integer_range({}, {})", range.minimum, range.maximum),
    }
}

fn alternative_check(ty: &FieldContract, fields: &[(String, Type)]) -> Result<String, SchemaError> {
    if let Some(alternative) = crate::model::TaggedAlternative::from_field(ty.field())? {
        let names = fields
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>();
        let attributes = names
            .iter()
            .copied()
            .zip(super::identifiers::fields(names.iter().copied()))
            .collect::<std::collections::BTreeMap<_, _>>();
        let discriminator = &attributes[alternative.discriminator.as_str()];
        let cases = alternative
            .arms
            .iter()
            .map(|(tag, selected)| {
                let checks = alternative
                    .payloads()
                    .into_iter()
                    .map(|name| {
                        let name_python = &attributes[name];
                        let present = if Some(name) == selected.as_deref() {
                            "not "
                        } else {
                            ""
                        };
                        format!("self.{name_python} is {present}None")
                    })
                    .collect::<Vec<_>>()
                    .join(" and ");
                if checks.is_empty() {
                    format!("self.{discriminator} == {tag:?}")
                } else {
                    format!("(self.{discriminator} == {tag:?} and {checks})")
                }
            })
            .collect::<Vec<_>>()
            .join(" or ");
        return Ok(format!(
            "\n    def __attrs_post_init__(self) -> None:\n        if not ({cases}):\n            message = \"tagged value requires exactly its selected arm\"\n            raise ValueError(message)\n"
        ));
    }
    Ok(String::new())
}

#[cfg(test)]
mod tests {
    use super::{scalar, structure};
    #[test]
    fn keyword_fields_emit_valid_aliases_with_exact_nested_wire_names() {
        let output = structure(
            "KeywordRow",
            &[
                ("class".to_owned(), scalar("b.str")),
                ("class_".to_owned(), scalar("b.str")),
                ("from".to_owned(), scalar("b.str")),
                ("_class".to_owned(), scalar("b.str")),
            ],
        );
        assert!(output.contains("    class__: b.str = attrs.field(validator=attrs.validators.instance_of(b.str), metadata={v.FIELD_NAME_METADATA: \"class\"})"));
        assert!(output.contains(
            "    class_: b.str = attrs.field(validator=attrs.validators.instance_of(b.str))"
        ));
        assert!(output.contains("metadata={v.FIELD_NAME_METADATA: \"from\"}"));
        assert!(output.contains("alias=\"_class\""));
        assert!(!output.contains("    class:"));
    }
}
