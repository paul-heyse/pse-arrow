// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Typed transport for the existing native Shape owner; no inferred wire ontology.
use super::check::{child, enumeration, known_fields, text, uint, value};
use super::*;
use pse_contract::boundary_projection::{Alternative, Shape};

impl OptionEncoder {
    pub(super) fn native_shape(
        &self,
        shape: &Shape,
        package: &str,
    ) -> Result<DynamicMessage, MetadataError> {
        let (case, inner) = match shape {
            Shape::Record(fields) => (
                "record",
                self.native_message(
                    "NativeRecord",
                    &[("fields", self.shape_fields(fields, package)?)],
                )?,
            ),
            Shape::Alternatives(alternatives) => (
                "alternatives",
                self.native_message(
                    "NativeAlternatives",
                    &[(
                        "alternatives",
                        Value::List(
                            alternatives
                                .iter()
                                .map(|alternative| {
                                    self.native_message(
                                        "NativeAlternative",
                                        &[
                                            ("name", Value::String(alternative.name.clone())),
                                            ("tag", Value::U32(alternative.tag)),
                                            (
                                                "fields",
                                                self.shape_fields(&alternative.fields, package)?,
                                            ),
                                        ],
                                    )
                                    .map(Value::Message)
                                })
                                .collect::<Result<_, MetadataError>>()?,
                        ),
                    )],
                )?,
            ),
            Shape::StringEnum(values) => (
                "string_enum",
                self.native_message(
                    "NativeStringEnum",
                    &[(
                        "values",
                        Value::List(values.iter().cloned().map(Value::String).collect()),
                    )],
                )?,
            ),
        };
        self.native_message("NativeOwnerShape", &[(case, Value::Message(inner))])
    }
    fn shape_fields(&self, fields: &[Field], package: &str) -> Result<Value, MetadataError> {
        Ok(Value::List(
            fields
                .iter()
                .map(|field| {
                    self.native_message(
                        "NativeFieldDeclaration",
                        &[
                            ("name", Value::String(field.name.clone())),
                            ("tag", Value::U32(field.tag)),
                            (
                                "contract",
                                Value::Message(self.native_contract(
                                    &field.ty,
                                    &field.rules,
                                    package,
                                )?),
                            ),
                            ("direction", direction(field.direction)),
                        ],
                    )
                    .map(Value::Message)
                })
                .collect::<Result<_, _>>()?,
        ))
    }
}

pub(super) fn decode(metadata: &DynamicMessage) -> Result<Option<Shape>, MetadataError> {
    if uint(metadata, "metadata_version")? != Some(pse_contract::metadata::VERSION) {
        return Ok(None);
    }
    let Some(shape) = child(metadata, "native_shape")? else {
        return Ok(None);
    };
    known_fields(shape, &["record", "alternatives", "string_enum"])?;
    let mut kinds = ["record", "alternatives", "string_enum"]
        .into_iter()
        .filter(|name| value(shape, name).is_ok_and(|value| value.is_some()));
    let case = kinds
        .next()
        .ok_or_else(|| error("native shape kind is absent"))?;
    if kinds.next().is_some() {
        return Err(error("multiple native shape kinds"));
    }
    let shape = child(shape, case)?.ok_or_else(|| error("native shape payload is absent"))?;
    Ok(Some(match case {
        "record" => {
            known_fields(shape, &["fields"])?;
            Shape::Record(fields(shape)?)
        }
        "alternatives" => {
            known_fields(shape, &["alternatives"])?;
            let alternatives = list(shape, "alternatives")?
                .iter()
                .map(|value| {
                    let Value::Message(alternative) = value else {
                        return Err(error("native alternative is not a message"));
                    };
                    known_fields(alternative, &["name", "tag", "fields"])?;
                    Ok(Alternative {
                        name: name(alternative)?,
                        tag: tag(alternative)?,
                        fields: fields(alternative)?,
                    })
                })
                .collect::<Result<Vec<_>, MetadataError>>()?;
            if alternatives.is_empty() {
                return Err(error("native alternative requires at least one member"));
            }
            unique(
                alternatives
                    .iter()
                    .map(|value| (value.name.as_str(), value.tag)),
            )?;
            Shape::Alternatives(alternatives)
        }
        _ => {
            known_fields(shape, &["values"])?;
            let values = list(shape, "values")?
                .iter()
                .map(|value| match value {
                    Value::String(value) => Ok(value.clone()),
                    _ => Err(error("native enum member is not a string")),
                })
                .collect::<Result<Vec<_>, _>>()?;
            if values.is_empty()
                || values
                    .iter()
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    != values.len()
            {
                return Err(error("native string vocabulary is empty or duplicated"));
            }
            Shape::StringEnum(values)
        }
    }))
}
fn list<'a>(message: &'a DynamicMessage, name: &str) -> Result<&'a [Value], MetadataError> {
    match value(message, name)? {
        None => Ok(&[]),
        Some(Value::List(values)) => Ok(values),
        _ => Err(error("native shape list is not repeated")),
    }
}
fn name(message: &DynamicMessage) -> Result<String, MetadataError> {
    text(message, "name")?
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| error("native shape member name is absent/empty"))
}
fn tag(message: &DynamicMessage) -> Result<u32, MetadataError> {
    uint(message, "tag")?
        .filter(|value| *value > 0 && *value < (1 << 29) && !(19000..=19999).contains(value))
        .ok_or_else(|| error("native shape member tag is invalid"))
}
fn unique<'a>(members: impl Iterator<Item = (&'a str, u32)>) -> Result<(), MetadataError> {
    let mut names = std::collections::BTreeSet::new();
    let mut tags = std::collections::BTreeSet::new();
    for (name, tag) in members {
        if !names.insert(name) || !tags.insert(tag) {
            return Err(error("native shape member name/tag is duplicated"));
        }
    }
    Ok(())
}
fn fields(message: &DynamicMessage) -> Result<Vec<Field>, MetadataError> {
    let fields = list(message, "fields")?
        .iter()
        .map(|value| {
            let Value::Message(field) = value else {
                return Err(error("native shape field is not a message"));
            };
            known_fields(field, &["name", "tag", "contract", "direction"])?;
            let (ty, rules) = field_contract::decode_contract(
                child(field, "contract")?
                    .ok_or_else(|| error("native shape field contract is absent"))?,
            )?;
            let direction = match enumeration(field, "direction", |value| match value {
                1 => Some(Direction::Input),
                2 => Some(Direction::Output),
                3 => Some(Direction::Both),
                _ => None,
            })? {
                Some(pse_contract::metadata::EnumValue::Known(value)) => value,
                _ => return Err(unavailable("native shape field direction unavailable")),
            };
            Ok(Field {
                name: name(field)?,
                tag: tag(field)?,
                ty,
                direction,
                rules,
                named_facets: Default::default(),
            })
        })
        .collect::<Result<Vec<_>, MetadataError>>()?;
    unique(fields.iter().map(|field| (field.name.as_str(), field.tag)))?;
    Ok(fields)
}
