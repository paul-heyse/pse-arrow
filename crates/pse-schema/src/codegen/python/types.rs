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
}

fn scalar(annotation: &str) -> Type {
    Type {
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
        annotation: "b.int".to_owned(),
        validator: format!("v.integer_range({lower}, {upper})"),
    }
}

pub(super) fn optional(mut ty: Type, nullable: bool) -> Type {
    if nullable {
        ty.annotation.push_str(" | None");
        ty.validator = format!("attrs.validators.optional({})", ty.validator);
    }
    ty
}

pub(super) fn structure(name: &str, fields: &[(String, Type)]) -> String {
    let attributes = super::identifiers::fields(fields.iter().map(|(name, _)| name.as_str()));
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
        "\n\n@attrs.frozen(kw_only=True)\nclass {name}:\n    \"\"\"Declared relation row or nested value.\"\"\"\n\n{fields}"
    )
}

fn list(child: &Type, width: Option<i32>) -> Type {
    let inner = format!(
        "attrs.validators.deep_iterable(member_validator={}, iterable_validator=attrs.validators.instance_of(b.tuple))",
        child.validator
    );
    let validator = width.map_or_else(|| inner.clone(), |width| format!("attrs.validators.and_({inner}, attrs.validators.min_len({width}), attrs.validators.max_len({width}))"));
    Type {
        annotation: format!("b.tuple[{}, ...]", child.annotation),
        validator,
    }
}

pub(super) fn logical(
    ty: &FieldContract,
    stem: &str,
    declarations: &mut String,
) -> Result<Type, SchemaError> {
    let mut value = logical_value(ty, stem, declarations)?;
    if let Some(collection) = crate::model::CollectionContract::from_field(ty.field())?
        && (collection.minimum != 0 || collection.maximum.is_some() || collection.unique)
    {
        let maximum = collection
            .maximum
            .map_or_else(|| "None".into(), |value| value.to_string());
        let unique = if collection.unique { "True" } else { "False" };
        value.validator = format!(
            "attrs.validators.and_({}, v.collection({}, {maximum}, unique={unique}))",
            value.validator, collection.minimum
        );
    }
    Ok(value)
}

fn logical_value(
    ty: &FieldContract,
    stem: &str,
    declarations: &mut String,
) -> Result<Type, SchemaError> {
    if let Some(range) = crate::model::IntegerRange::from_field(ty.field())? {
        return Ok(integer_domain(range));
    }
    Ok(match (ty.extension(), ty.data_type()) {
        (None, DataType::List(child)) => list(
            &optional(
                logical(
                    &FieldContract::from_field((*child).clone()),
                    &format!("{stem}Item"),
                    declarations,
                )?,
                child.is_nullable(),
            ),
            None,
        ),
        (None, DataType::FixedSizeList(child, width)) => list(
            &optional(
                logical(
                    &FieldContract::from_field((*child).clone()),
                    &format!("{stem}Item"),
                    declarations,
                )?,
                child.is_nullable(),
            ),
            Some(width),
        ),
        (None, DataType::Struct(children)) => {
            let fields = children
                .iter()
                .map(|field| {
                    let name = field.name();
                    let ty = FieldContract::from_field((**field).clone());
                    let nullable = field.is_nullable();
                    Ok((
                        name.to_owned(),
                        optional(
                            logical(&ty, &format!("{stem}{}", pascal(name)), declarations)?,
                            nullable,
                        ),
                    ))
                })
                .collect::<Result<Vec<_>, SchemaError>>()?;
            declarations.push_str(&structure(stem, &fields));
            declarations.push_str(&alternative_check(ty, &fields)?);
            scalar(stem)
        }
        (Some(ExtensionUse::Enum(name)), _) => scalar(&format!("e.{}", pascal(name))),
        (Some(use_), _)
            if matches!(
                use_,
                ExtensionUse::DimensionVector
                    | ExtensionUse::QuantityValue
                    | ExtensionUse::Bound
                    | ExtensionUse::SourceSpan
            ) =>
        {
            let name = pascal(&use_.name());
            if matches!(use_, ExtensionUse::DimensionVector) {
                list(&scalar("v.DimensionVectorItem"), Some(8))
            } else {
                scalar(&format!("v.{name}"))
            }
        }
        (None, DataType::FixedSizeBinary(_) | DataType::Dictionary(..)) => {
            return Err(super::error(format!(
                "no native language codec for {}; domain meaning requires an explicit declaration",
                ty.data_type()
            )));
        }
        _ => storage(&ty.data_type(), stem, declarations)?,
    })
}

pub(super) fn storage(
    ty: &DataType,
    stem: &str,
    declarations: &mut String,
) -> Result<Type, SchemaError> {
    Ok(match ty {
        DataType::Boolean => Type {
            annotation: "b.bool".to_owned(),
            validator: "v.exact_type(b.bool)".to_owned(),
        },
        DataType::Int16 => integer(16, true),
        DataType::Int32 => integer(32, true),
        DataType::Int64 => integer(64, true),
        DataType::UInt8 => integer(8, false),
        DataType::UInt16 => integer(16, false),
        DataType::UInt32 => integer(32, false),
        DataType::UInt64 => integer(64, false),
        DataType::Float64 => Type {
            annotation: "b.float".to_owned(),
            validator: "v.finite_float".to_owned(),
        },
        DataType::Utf8 => scalar("b.str"),
        DataType::Timestamp(..) => Type {
            annotation: "datetime".to_owned(),
            validator: "v.utc_timestamp".to_owned(),
        },
        DataType::FixedSizeBinary(16) => scalar("v.SemanticId"),
        DataType::FixedSizeBinary(32) => scalar("v.ContentHash"),
        DataType::List(child) => list(
            &storage(child.data_type(), &format!("{stem}Item"), declarations)?,
            None,
        ),
        DataType::FixedSizeList(child, width) => list(
            &storage(child.data_type(), &format!("{stem}Item"), declarations)?,
            Some(*width),
        ),
        DataType::Struct(children) => {
            let fields = children
                .iter()
                .map(|field| {
                    Ok((
                        field.name().to_owned(),
                        optional(
                            if let Some(range) = crate::model::IntegerRange::from_field(field)? {
                                integer_domain(range)
                            } else if let Some(name) = super::super::enum_name(field) {
                                scalar(&format!("e.{}", pascal(name)))
                            } else {
                                storage(
                                    field.data_type(),
                                    &format!("{stem}{}", pascal(field.name())),
                                    declarations,
                                )?
                            },
                            field.is_nullable(),
                        ),
                    ))
                })
                .collect::<Result<Vec<_>, SchemaError>>()?;
            declarations.push_str(&structure(stem, &fields));
            scalar(stem)
        }
        other => return Err(super::error(format!("no Python type for {other}"))),
    })
}

fn integer_domain(range: crate::model::IntegerRange) -> Type {
    Type {
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
