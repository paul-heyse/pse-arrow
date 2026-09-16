// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Declared local value domains lowered to ordinary DataFusion expressions.
//! Scalar predicates also become Delta SQL CHECK text. Collection expressions remain
//! native and use the narrowly scoped adapter required by Delta's pinned SQL parser.

use datafusion::{
    arrow::datatypes::{ArrowPrimitiveType, DataType, Field, Float16Type, Schema},
    common::{Column, DFSchema, DataFusionError, Result, ScalarValue},
    functions::{
        core::expr_fn::get_field, encoding::expr_fn::encode, math::expr_fn::gcd,
        string::expr_fn::octet_length,
    },
    functions_nested::expr_fn::{
        array_any_match, array_distinct, array_length, map_keys, map_values,
    },
    logical_expr::{
        Expr, cast,
        expr_fn::{lambda, lambda_var},
        lit,
    },
};
use pse_schema::{
    Registry,
    model::{FieldContract, IntegerRange},
};

pub(super) fn relation(registry: &Registry, schema: &Schema) -> Result<Expr> {
    let checks = schema
        .fields()
        .iter()
        .map(|field| field_value(registry, field, column(field.name()), 0))
        .collect::<Result<Vec<_>>>()?;
    Ok(combine(checks)
        .resolve_lambda_variables(&DFSchema::try_from(schema.clone())?)?
        .data)
}

pub(super) fn combine(checks: Vec<Expr>) -> Expr {
    all(checks)
}

fn column(name: &str) -> Expr {
    Expr::Column(Column::from_name(name))
}
fn all(checks: impl IntoIterator<Item = Expr>) -> Expr {
    checks
        .into_iter()
        .reduce(Expr::and)
        .unwrap_or_else(|| lit(true))
}

pub(super) fn field_value(
    registry: &Registry,
    field: &Field,
    value: Expr,
    depth: usize,
) -> Result<Expr> {
    let mut checks = vec![];
    if let Some(collection) =
        pse_schema::model::CollectionContract::from_field(field).map_err(external)?
    {
        let length = array_length(value.clone());
        checks.push(length.clone().gt_eq(lit(collection.minimum)));
        if let Some(maximum) = collection.maximum {
            checks.push(length.clone().lt_eq(lit(maximum)));
        }
        if collection.unique {
            checks.push(length.eq(array_length(array_distinct(value.clone()))));
        }
    }
    if let Some(alternative) =
        pse_schema::model::TaggedAlternative::from_field(field).map_err(external)?
    {
        let tag = get_field(value.clone(), &alternative.discriminator);
        let variants = alternative.arms.iter().map(|(name, selected)| {
            let presence = alternative.payloads().into_iter().map(|arm| {
                let payload = get_field(value.clone(), arm);
                if Some(arm) == selected.as_deref() {
                    payload.is_not_null()
                } else {
                    payload.is_null()
                }
            });
            tag.clone().eq(lit(name.clone())).and(all(presence))
        });
        checks.push(variants.reduce(Expr::or).unwrap_or_else(|| lit(false)));
    }
    if let Some(range) = IntegerRange::from_field(field).map_err(external)? {
        checks.push(
            value
                .clone()
                .between(lit(range.minimum), lit(range.maximum)),
        );
    }
    if let Some(name) = FieldContract::from_field(field.clone()).enum_name() {
        let domain = registry
            .enum_spec(name)
            .ok_or_else(|| DataFusionError::Plan(format!("unknown enum {name}")))?;
        checks.push(
            value.clone().in_list(
                domain
                    .members
                    .iter()
                    .map(|member| lit(member.name))
                    .collect(),
                false,
            ),
        );
    }
    checks.push(storage_value(
        registry,
        field.data_type(),
        value.clone(),
        depth,
    )?);
    match field
        .metadata()
        .get(pse_schema::arrow::KEY_EXTENSION_NAME)
        .map(String::as_str)
    {
        Some("pse.bound") => {
            let kind = get_field(value.clone(), "kind");
            let payload = get_field(value.clone(), "value");
            checks.push(
                kind.clone()
                    .eq(lit("finite"))
                    .and(payload.clone().is_not_null())
                    .or(kind.eq(lit("unbounded")).and(payload.is_null())),
            );
        }
        Some("pse.source_span") => {
            checks.push(get_field(value.clone(), "start").lt_eq(get_field(value.clone(), "end")));
        }
        Some("pse.dimension_vector") => {
            let parameter = format!("pse_exponent_{depth}");
            let member = lambda_var(&parameter);
            let numerator = cast(get_field(member.clone(), "num"), DataType::Int64);
            let denominator = cast(get_field(member, "den"), DataType::Int64);
            let valid = denominator
                .clone()
                .gt(lit(0_i64))
                .and(gcd(numerator, denominator).eq(lit(1_i64)));
            checks.push(!array_any_match(
                value.clone(),
                lambda([parameter], valid.is_not_true()),
            ));
        }
        _ => {}
    }
    let valid = all(checks).is_true();
    Ok(if field.is_nullable() {
        value.is_null().or(valid)
    } else {
        value.is_not_null().and(valid)
    })
}

fn storage_value(registry: &Registry, ty: &DataType, value: Expr, depth: usize) -> Result<Expr> {
    Ok(match ty {
        DataType::Struct(fields) => all(fields
            .iter()
            .map(|child| {
                field_value(
                    registry,
                    child,
                    get_field(value.clone(), child.name()),
                    depth + 1,
                )
            })
            .collect::<Result<Vec<_>>>()?),
        DataType::List(child)
        | DataType::LargeList(child)
        | DataType::ListView(child)
        | DataType::LargeListView(child)
        | DataType::FixedSizeList(child, _) => {
            let parameter = format!("pse_item_{depth}");
            let valid = field_value(registry, child, lambda_var(&parameter), depth + 1)?;
            let members = !array_any_match(value.clone(), lambda([parameter], valid.is_not_true()));
            if let DataType::FixedSizeList(_, width) = ty {
                array_length(value).eq(lit(i64::from(*width))).and(members)
            } else {
                members
            }
        }
        DataType::FixedSizeBinary(width) => {
            octet_length(encode(value, lit("hex"))).eq(lit(i64::from(*width) * 2))
        }
        DataType::Map(entries, _) => {
            let DataType::Struct(fields) = entries.data_type() else {
                return Err(DataFusionError::Plan("map entries require a struct".into()));
            };
            let [key, item] = fields.as_ref() else {
                return Err(DataFusionError::Plan(
                    "map entries require key and value fields".into(),
                ));
            };
            let keys = map_keys(value.clone());
            let distinct =
                array_length(array_distinct(keys.clone())).eq(array_length(keys.clone()));
            let children = [keys, map_values(value)]
                .into_iter()
                .zip([key, item])
                .map(|(values, child)| {
                    let parameter = format!("pse_map_{depth}");
                    let valid = field_value(registry, child, lambda_var(&parameter), depth + 1)?;
                    Ok(!array_any_match(
                        values,
                        lambda([parameter], valid.is_not_true()),
                    ))
                })
                .collect::<Result<Vec<_>>>()?;
            distinct.and(all(children))
        }
        DataType::Float16 => {
            let maximum = f32::from(<Float16Type as ArrowPrimitiveType>::Native::MAX);
            cast(value, DataType::Float32).between(lit(-maximum), lit(maximum))
        }
        DataType::Float64 => value.between(lit(-f64::MAX), lit(f64::MAX)),
        DataType::Float32 => value.between(lit(-f32::MAX), lit(f32::MAX)),
        DataType::UInt8 => value.between(lit(0_i64), lit(i64::from(u8::MAX))),
        DataType::UInt16 => value.between(lit(0_i64), lit(i64::from(u16::MAX))),
        DataType::UInt32 => value.between(lit(0_i64), lit(i64::from(u32::MAX))),
        DataType::UInt64 => value.between(
            lit(0_i64),
            lit(ScalarValue::Decimal128(Some(i128::from(u64::MAX)), 20, 0)),
        ),
        DataType::Dictionary(_, values) => storage_value(registry, values, value, depth)?,
        _ => lit(true),
    })
}

#[cfg(test)]
mod tests;

fn external(error: impl std::error::Error + Send + Sync + 'static) -> DataFusionError {
    DataFusionError::External(Box::new(error))
}
