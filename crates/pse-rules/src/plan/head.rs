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
    model::{Cell, FieldContract, QuantityContract, RuleSpec},
};
use std::sync::Arc;

pub(super) fn prepare(
    rule: &RuleSpec,
    output: Planned,
    columns: Vec<FieldContract>,
    registry: &Registry,
    keep_hidden: bool,
    session: &pse_catalog::session::SnapshotSession,
) -> Result<(datafusion_expr::LogicalPlan, Vec<Column>, SchemaRef), RuleError> {
    if output.columns.len() != columns.len() {
        return Err(internal("rule output has different head width"));
    }
    let mut projection = vec![];
    let mut contracts = vec![];
    let mut fields = vec![];
    for target in columns {
        let actual = expr::lookup(&output, target.name())?;
        let mut value = expr::column_expression(actual);
        let field = pse_schema::arrow::field_for(registry, &target)
            .map_err(|error| internal(error.to_string()))?;
        if let Some(literal) = &actual.literal {
            // A literal has no ambient physical contract. A quantity-bearing destination
            // still requires an explicit physical source expression.
            if target.quantity() != QuantityContract::None && !matches!(literal, Cell::Null) {
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
        } else if actual.spec.value_type() != target.value_type()
            && !actual.spec.nullable()
            && !target.nullable()
            && integer_type(&actual.spec.value_type())
            && integer_type(&target.value_type())
            && actual.spec.quantity() == target.quantity()
        {
            // Integer casts preserve exact values or fail/produce NULL. A nonnull
            // destination is required, so overflow can never become missing data.
            // An explicit target field carries the checked conversion's declaration;
            // reusing source metadata would incorrectly label UInt16 as UInt64.
            value = datafusion_expr::Expr::Cast(datafusion_expr::expr::Cast::new_from_field(
                Box::new(value),
                Arc::new(field.clone()),
            ));
        } else if actual.spec.value_type() != target.value_type()
            || actual.spec.quantity() != target.quantity()
            || (actual.spec.nullable() && !target.nullable())
        {
            return Err(mismatch(rule, &target, actual));
        }
        projection.push(value.alias(target.name()));
        fields.push(field);
        contracts.push(Column {
            name: target.name().to_owned().into(),
            spec: target,
            qualifier: None,
            physical: None,
            literal: actual.literal.clone(),
        });
    }
    if keep_hidden {
        projection.extend(output.hidden.iter().map(datafusion_expr::col));
    }
    let plan = LogicalPlanBuilder::from(output.plan)
        .project(projection)
        .map_err(engine)?
        .build()
        .map_err(engine)?;
    let schema = if matches!(rule.head, pse_schema::model::RuleHead::Relation(_)) && !keep_hidden {
        let spec = registry
            .relation(rule.head.relation())
            .ok_or_else(|| internal("head relation missing"))?;
        pse_schema::arrow::relation_schema(registry, spec)
            .map_err(|error| internal(error.to_string()))?
    } else {
        let mut fields = fields;
        for hidden in &output.hidden {
            if keep_hidden {
                fields.push(
                    plan.schema()
                        .field_with_unqualified_name(hidden)
                        .map_err(engine)?
                        .as_ref()
                        .clone(),
                );
            }
        }
        Schema::new(fields)
    };
    let schema = Arc::new(schema);
    let plan = declare_schema(plan, &schema, session)?;
    Ok((plan, contracts, schema))
}

/// Called only after the declaration frontend has established every expression's
/// complete field meaning. The target schema is part of the native plan before
/// preparation; execution never replaces metadata on a finished batch.
pub(super) fn declare_schema(
    plan: datafusion_expr::LogicalPlan,
    schema: &SchemaRef,
    session: &pse_catalog::session::SnapshotSession,
) -> Result<datafusion_expr::LogicalPlan, RuleError> {
    let plan = session.derive_plan_fields(plan, &pse_ids::CancellationToken::new())?;
    let expressions = schema
        .fields()
        .iter()
        .map(|field| {
            let (qualifier, source) = plan
                .schema()
                .qualified_field_with_unqualified_name(field.name())
                .map_err(engine)?;
            if source.data_type() != field.data_type()
                || (source.is_nullable() && !field.is_nullable())
            {
                return Err(internal(format!(
                    "native head field {} does not establish target storage/nullability: native={} nullable={}, declared={} nullable={}",
                    field.name(), source.data_type(), source.is_nullable(),
                    field.data_type(), field.is_nullable(),
                )));
            }
            Ok(
                datafusion_expr::Expr::Column(datafusion_common::Column::new(
                    qualifier.cloned(),
                    source.name(),
                ))
                .alias_with_metadata(
                    field.name(),
                    Some(datafusion_common::metadata::FieldMetadata::from(
                        field.as_ref(),
                    )),
                ),
            )
        })
        .collect::<Result<Vec<_>, RuleError>>()?;
    Ok(datafusion_expr::LogicalPlan::Projection(
        datafusion_expr::Projection::try_new_with_schema(
            expressions,
            Arc::new(plan),
            Arc::new(
                datafusion_common::DFSchema::try_from(schema.as_ref().clone()).map_err(engine)?,
            ),
        )
        .map_err(engine)?,
    ))
}
fn integer_type(ty: &FieldContract) -> bool {
    ty.extension().is_none()
        && matches!(
            ty.data_type(),
            DataType::UInt8
                | DataType::UInt16
                | DataType::UInt32
                | DataType::UInt64
                | DataType::Int32
                | DataType::Int64
        )
}

fn mismatch(rule: &RuleSpec, target: &FieldContract, actual: &Column) -> RuleError {
    RuleError::HeadSchemaMismatch {
        rule: rule.qualified_name(),
        column: target.name().to_owned(),
        expected: format!(
            "{:?} {:?} nullable={}",
            target.value_type(),
            target.quantity(),
            target.nullable()
        ),
        found: format!(
            "{:?} {:?} nullable={}",
            actual.spec.value_type(),
            actual.spec.quantity(),
            actual.spec.nullable()
        ),
    }
}
fn check_literal(
    rule: &RuleSpec,
    target: &FieldContract,
    value: &Cell,
    registry: &Registry,
) -> Result<(), RuleError> {
    use pse_schema::model::ExtensionUse as X;
    let valid = match (value, target.extension(), target.data_type()) {
        (Cell::Null, _, _) => target.nullable(),
        (Cell::Bool(_), None, DataType::Boolean)
        | (Cell::Text(_), None, DataType::Utf8)
        | (Cell::Id(_), Some(X::SemanticId), _)
        | (Cell::Hash(_), Some(X::ContentHash), _) => true,
        (Cell::Enum(name), Some(X::Enum(target)), _) => registry
            .enum_spec(target)
            .is_some_and(|spec| spec.members.iter().any(|member| member.name == *name)),
        (Cell::I64(value), None, DataType::Float64) => {
            pse_quantity::numeric::exact_f64_from_i64(*value).is_some()
        }
        (Cell::F64(value), None, DataType::Int64) => {
            pse_quantity::numeric::exact_i64_from_f64(*value).is_some()
        }
        (Cell::F64(value), None, DataType::Float64) => value.is_finite(),
        (Cell::I64(value), _, ty) => integer_in_range(i128::from(*value), &ty),
        (Cell::U64(value), _, ty) => integer_in_range(i128::from(*value), &ty),
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(RuleError::HeadSchemaMismatch {
            rule: rule.qualified_name(),
            column: target.name().to_owned(),
            expected: format!("{:?}", target.value_type()),
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
