// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact maintained CEL follows declared wire containers, not arbitrary record fields.

use super::{error, protobuf_map_key};
use pse_model::boundary_projection::{
    Float64Domain, NativeContainer, ProjectionGraph, Scalar, TypeRef,
};
use pse_schema::SchemaError;

pub(super) fn scalar_expression(domain: Float64Domain, value: &str) -> Result<String, SchemaError> {
    Ok(match domain {
        Float64Domain::Finite => format!(
            "{value} < {}u || ({value} >= {}u && {value} < {}u)",
            f64::INFINITY.to_bits(),
            (-0.0_f64).to_bits(),
            f64::NEG_INFINITY.to_bits()
        ),
        Float64Domain::NonnegativeFinite => format!(
            "{value} <= {}u || {value} == {}u",
            f64::MAX.to_bits(),
            (-0.0_f64).to_bits()
        ),
        Float64Domain::PositiveFinite | Float64Domain::PositiveUnit => {
            let maximum = domain.positive_maximum_bits().ok_or_else(|| {
                error("positive floating domain has no source-owned bit interval")
            })?;
            format!("{value} > 0u && {value} <= {maximum}u")
        }
    })
}

pub(super) fn collection_expression(
    graph: &ProjectionGraph,
    ty: &TypeRef,
    domain: Float64Domain,
) -> Result<String, SchemaError> {
    struct Access {
        value: String,
        presence: Option<String>,
    }
    fn field(parent: &str, name: &str) -> Access {
        let value = format!("{parent}.{name}");
        Access {
            presence: Some(format!("has({value})")),
            value,
        }
    }
    fn wrapped_value(ty: &TypeRef, parent: &str) -> Access {
        if matches!(ty, TypeRef::Scalar(_) | TypeRef::Named(_)) {
            Access {
                value: parent.into(),
                presence: None,
            }
        } else {
            field(parent, "value")
        }
    }
    fn expression(
        graph: &ProjectionGraph,
        ty: &TypeRef,
        domain: Float64Domain,
        access: Access,
        depth: usize,
    ) -> Result<String, SchemaError> {
        let value = &access.value;
        Ok(match ty {
            TypeRef::Scalar(Scalar::F64Bits) => scalar_expression(domain, value)?,
            TypeRef::Optional(inner) => {
                let inner_access =
                    if matches!(inner.as_ref(), TypeRef::Scalar(_) | TypeRef::Named(_)) {
                        Access {
                            value: value.clone(),
                            presence: None,
                        }
                    } else {
                        field(value, "value")
                    };
                let child = expression(graph, inner, domain, inner_access, depth + 1)?;
                access.presence.map_or(child.clone(), |present| {
                    format!("!({present}) || ({child})")
                })
            }
            TypeRef::Repeated(inner) | TypeRef::Set(inner) => {
                let element = format!("element_{depth}");
                let child = expression(
                    graph,
                    inner,
                    domain,
                    wrapped_value(inner, &element),
                    depth + 1,
                )?;
                format!("{value}.all({element}, ({child}))")
            }
            TypeRef::Map(key, inner) => {
                if protobuf_map_key(key) {
                    let key = format!("key_{depth}");
                    let child = expression(
                        graph,
                        inner,
                        domain,
                        wrapped_value(inner, &format!("{value}[{key}]")),
                        depth + 1,
                    )?;
                    format!("{value}.all({key}, ({child}))")
                } else {
                    let entry = format!("entry_{depth}");
                    let raw = field(&entry, "value");
                    let child_access =
                        if matches!(inner.as_ref(), TypeRef::Scalar(_) | TypeRef::Named(_)) {
                            raw
                        } else {
                            field(&raw.value, "value")
                        };
                    let child = expression(graph, inner, domain, child_access, depth + 1)?;
                    format!("{value}.all({entry}, ({child}))")
                }
            }
            TypeRef::Named(name) => {
                let Some(NativeContainer::FixedArray { element, .. }) = graph
                    .definitions()
                    .get(name)
                    .and_then(|owner| owner.native_container.as_ref())
                else {
                    return Err(error(
                        "floating domain requires its declared scalar/container owner",
                    ));
                };
                expression(
                    graph,
                    &TypeRef::Repeated(Box::new(element.clone())),
                    domain,
                    field(value, "values"),
                    depth + 1,
                )?
            }
            TypeRef::Scalar(_) => {
                return Err(error("floating domain requires actual binary64 leaves"));
            }
        })
    }
    expression(
        graph,
        ty,
        domain,
        Access {
            value: "this".into(),
            presence: None,
        },
        0,
    )
}
