// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Head admission uses the retained derivation; storage agreement alone cannot add meaning.
use super::{Column, Planned, expr};
use crate::{
    RuleError,
    errmap::{engine, internal},
};
use datafusion::arrow::datatypes::{DataType, Schema, SchemaRef};
use datafusion_expr::LogicalPlanBuilder;
use pse_schema::{
    Registry,
    model::{Cell, ColumnSpec, LogicalType, QuantityContract, RuleSpec},
};
use std::sync::Arc;

pub(super) fn prepare(
    rule: &RuleSpec,
    output: Planned,
    columns: Vec<ColumnSpec>,
    registry: &Registry,
) -> Result<(datafusion_expr::LogicalPlan, Vec<Column>, SchemaRef), RuleError> {
    if output.columns.len() != columns.len() {
        return Err(internal("rule output has different head width"));
    }
    let mut projection = vec![];
    let mut contracts = vec![];
    let mut fields = vec![];
    for target in columns {
        let actual = expr::lookup(&output, target.name)?;
        let mut value = expr::column_expression(actual);
        let field = pse_schema::arrow::field_for(registry, &target)
            .map_err(|error| internal(error.to_string()))?;
        if let Some(literal) = &actual.literal {
            // A literal has no ambient physical contract. A quantity-bearing destination
            // still requires an explicit physical source expression.
            if target.quantity != QuantityContract::None && !matches!(literal, Cell::Null) {
                return Err(mismatch(rule, &target, actual));
            }
            check_literal(rule, &target, literal, registry)?;
            if matches!(literal, Cell::Null) {
                value = expr::typed_null(&target, registry)?;
            } else {
                // The literal's actual value has passed the exact conversion and
                // dictionary/quantity checks above. Materialize that checked scalar
                // under its destination contract; a column cast would retain the
                // original field metadata despite changing its storage type.
                let (datafusion_expr::Expr::Literal(scalar, _), _) =
                    expr::literal(literal, registry)?
                else {
                    return Err(internal("literal lowering did not produce a scalar"));
                };
                value = datafusion_expr::Expr::Literal(
                    scalar.cast_to(field.data_type()).map_err(engine)?,
                    Some(datafusion_common::metadata::FieldMetadata::from(&field)),
                );
            }
        } else if actual.spec.logical_type != target.logical_type
            || actual.spec.quantity != target.quantity
            || (actual.spec.nullable && !target.nullable)
        {
            return Err(mismatch(rule, &target, actual));
        }
        projection.push(value.alias(target.name));
        fields.push(field);
        contracts.push(Column {
            spec: target,
            qualifier: None,
            literal: actual.literal.clone(),
        });
    }
    let plan = LogicalPlanBuilder::from(output.plan)
        .project(projection)
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    let schema = if matches!(rule.head, pse_schema::model::RuleHead::Relation(_)) {
        let spec = registry
            .relation(rule.head.relation())
            .ok_or_else(|| internal("head relation missing"))?;
        pse_schema::arrow::relation_schema(registry, spec)
            .map_err(|error| internal(error.to_string()))?
    } else {
        Schema::new(fields)
    };
    Ok((plan, contracts, Arc::new(schema)))
}
fn mismatch(rule: &RuleSpec, target: &ColumnSpec, actual: &Column) -> RuleError {
    RuleError::HeadSchemaMismatch {
        rule: rule.qualified_name(),
        column: target.name.to_owned(),
        expected: format!(
            "{:?} {:?} nullable={}",
            target.logical_type, target.quantity, target.nullable
        ),
        found: format!(
            "{:?} {:?} nullable={}",
            actual.spec.logical_type, actual.spec.quantity, actual.spec.nullable
        ),
    }
}
fn check_literal(
    rule: &RuleSpec,
    target: &ColumnSpec,
    value: &Cell,
    registry: &Registry,
) -> Result<(), RuleError> {
    let valid = match (value, &target.logical_type) {
        (Cell::Null, _) => target.nullable,
        (Cell::Bool(_), LogicalType::Bool)
        | (Cell::Text(_), LogicalType::Text)
        | (Cell::Id(_), LogicalType::Ext(pse_schema::model::ExtensionUse::SemanticId))
        | (Cell::Hash(_), LogicalType::Ext(pse_schema::model::ExtensionUse::ContentHash)) => true,
        (Cell::Enum(name), LogicalType::Ext(pse_schema::model::ExtensionUse::Enum(target))) => {
            registry
                .enum_spec(target)
                .is_some_and(|spec| spec.members.iter().any(|member| member.name == *name))
        }
        (Cell::I64(value), LogicalType::F64) => {
            pse_quantity::numeric::exact_f64_from_i64(*value).is_some()
        }
        (Cell::F64(value), LogicalType::I64) => {
            pse_quantity::numeric::exact_i64_from_f64(*value).is_some()
        }
        (Cell::F64(value), LogicalType::F64) => value.is_finite(),
        (Cell::I64(value), ty) => integer_in_range(i128::from(*value), &ty.data_type()),
        (Cell::U64(value), ty) => integer_in_range(i128::from(*value), &ty.data_type()),
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(RuleError::HeadSchemaMismatch {
            rule: rule.qualified_name(),
            column: target.name.to_owned(),
            expected: format!("{:?}", target.logical_type),
            found: value.literal_spec(),
        })
    }
}
fn integer_in_range(value: i128, ty: &DataType) -> bool {
    match ty {
        DataType::Int32 => i32::try_from(value).is_ok(),
        DataType::Int64 => i64::try_from(value).is_ok(),
        DataType::UInt8 => u8::try_from(value).is_ok(),
        DataType::UInt16 => u16::try_from(value).is_ok(),
        DataType::UInt32 => u32::try_from(value).is_ok(),
        DataType::UInt64 => u64::try_from(value).is_ok(),
        _ => false,
    }
}
