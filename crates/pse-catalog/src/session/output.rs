// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit relation output declarations inside the observed native logical plan.

use datafusion::arrow::datatypes::{DataType, Field};
use datafusion::common::{DFSchema, DataFusionError, Result, ScalarValue, metadata::FieldMetadata};
use datafusion::logical_expr::{Expr, ExprSchemable, LogicalPlan, Projection};
use pse_schema::{
    Registry,
    model::{FieldContract, RelationSpec},
};
use std::sync::Arc;

/// Project values out of their source relation's role and foreign-key annotations.
/// Intrinsic Arrow/semantic fields, nullability and actual values are unchanged.
/// This is useful at recursive boundaries where rows come from different relations;
/// the original relation's obligations do not describe the recursive worktable.
///
/// # Errors
/// Native projection or field resolution fails.
pub fn forget_relation_annotations(plan: LogicalPlan) -> Result<LogicalPlan> {
    let mut changed = false;
    let expressions = plan
        .schema()
        .iter()
        .map(|(qualifier, field)| {
            let expression = Expr::Column(datafusion::common::Column::new(
                qualifier.cloned(),
                field.name(),
            ));
            if !field.metadata().contains_key(pse_schema::arrow::KEY_ROLE)
                && !field.metadata().contains_key(pse_schema::arrow::KEY_FK)
            {
                return expression;
            }
            changed = true;
            super::scalar::retain_metadata(
                expression,
                field
                    .metadata()
                    .keys()
                    .filter(|key| {
                        !matches!(
                            key.as_str(),
                            pse_schema::arrow::KEY_ROLE | pse_schema::arrow::KEY_FK
                        )
                    })
                    .cloned()
                    .collect::<std::collections::BTreeSet<_>>(),
            )
            .alias(field.name())
        })
        .collect::<Vec<_>>();
    if changed {
        datafusion::logical_expr::LogicalPlanBuilder::from(plan)
            .project(expressions)?
            .build()
    } else {
        Ok(plan)
    }
}

/// Build a native CASE whose alternatives establish the same field meaning.
/// DataFusion's CASE derives the value type and nullability but drops top-level
/// annotations. Its native `with_metadata` UDF carries their checked intersection.
///
/// # Errors
/// Alternatives differ in storage or semantic metadata, or native CASE is invalid.
pub fn same_field_case(schema: &DFSchema, condition: Expr, yes: Expr, no: Expr) -> Result<Expr> {
    let (_, yes_field) = yes.to_field(schema)?;
    let (_, no_field) = no.to_field(schema)?;
    if yes_field.data_type() != no_field.data_type()
        || !super::admission::same_value_metadata(&yes_field, &no_field)
    {
        return Err(invalid(
            "CASE alternatives do not establish the same field meaning",
        ));
    }
    let expression = datafusion::logical_expr::when(condition, yes).otherwise(no)?;
    let mut arguments = vec![expression];
    for (key, value) in yes_field
        .metadata()
        .iter()
        .filter(|(key, value)| no_field.metadata().get(*key) == Some(*value))
        .collect::<std::collections::BTreeMap<_, _>>()
    {
        arguments.push(datafusion::logical_expr::lit(key.clone()));
        arguments.push(datafusion::logical_expr::lit(value.clone()));
    }
    Ok(if arguments.len() == 1 {
        arguments.remove(0)
    } else {
        datafusion::functions::core::with_metadata().call(arguments)
    })
}

/// Projects named native outputs into an exact registry relation before preparation.
/// Source expressions must already establish semantic meaning; roles and FK annotations
/// declare outstanding relation obligations and never advertise optimizer constraints.
///
/// # Errors
/// Wrong output names/types/nullability, undeclared meaning or a different registry contract.
pub fn declare_relation_output(
    plan: LogicalPlan,
    registry: &Registry,
    spec: &RelationSpec,
) -> Result<LogicalPlan> {
    if plan.schema().fields().len() != spec.columns.len() {
        return Err(invalid(
            "native output width differs from the relation declaration",
        ));
    }
    declare_relation_projection(plan, registry, spec, Vec::new())
}

/// Project declared fields and ancillary algorithm arguments in the same native row.
/// Extra expressions remain ordinary native fields. The resulting wider schema does
/// not itself claim to be the declared relation; callers project its exact fields at
/// their typed algorithm boundary. This keeps source keys and selected values coupled
/// through sorting, repartitioning and volatile expression evaluation.
///
/// # Errors
/// Undeclared fields, incompatible semantic meaning or invalid extra expressions.
pub fn declare_relation_projection(
    plan: LogicalPlan,
    registry: &Registry,
    spec: &RelationSpec,
    extras: Vec<Expr>,
) -> Result<LogicalPlan> {
    let actual = registry
        .relation_by_id(spec.id)
        .ok_or_else(|| invalid("output relation is not declared"))?;
    if !std::ptr::eq(actual, spec)
        && registry
            .compiled_declaration(actual)
            .map_err(schema_error)?
            != registry.compiled_declaration(spec).map_err(schema_error)?
    {
        return Err(invalid(
            "output differs from its authoritative registry declaration",
        ));
    }
    let schema = pse_schema::arrow::relation_schema(registry, spec).map_err(schema_error)?;
    let mut expressions = Vec::with_capacity(spec.columns.len());
    for target in schema.fields() {
        let (qualifier, source) = plan
            .schema()
            .qualified_field_with_unqualified_name(target.name())?;
        let mut expression = Expr::Column(datafusion::common::Column::new(
            qualifier.cloned(),
            source.name(),
        ));
        let (_, source) = expression.to_field(plan.schema())?;
        check_field_output(&source, target)?;
        if source.data_type() != target.data_type() {
            // The recursive check establishes every child meaning first. Arrow's
            // native cast then carries primitive child labels in the actual result
            // datatype; no finished array is relabeled after materialization.
            expression = expression.cast_to(target.data_type(), plan.schema())?;
        }
        if target.is_nullable() && !source.is_nullable() {
            expression = super::scalar::nullable(expression);
        }
        if source
            .metadata()
            .keys()
            .any(|key| !target.metadata().contains_key(key))
        {
            expression = super::scalar::retain_metadata(
                expression,
                target
                    .metadata()
                    .keys()
                    .cloned()
                    .collect::<std::collections::BTreeSet<_>>(),
            );
        }
        // Use the pinned native metadata carrier so declaration transport remains
        // visible in the expression plan. The semantic check above establishes
        // compatibility first; with_metadata itself only carries annotations.
        if !target.metadata().is_empty() {
            let mut arguments = vec![expression];
            for (key, value) in target
                .metadata()
                .iter()
                .collect::<std::collections::BTreeMap<_, _>>()
            {
                arguments.push(datafusion::logical_expr::lit(key.clone()));
                arguments.push(datafusion::logical_expr::lit(value.clone()));
            }
            expression = datafusion::functions::core::with_metadata().call(arguments);
        }
        expressions.push(expression.alias(target.name()));
    }
    let mut output_fields = schema.fields().to_vec();
    for extra in extras {
        let (_, field) = extra.to_field(plan.schema())?;
        if output_fields
            .iter()
            .any(|previous| previous.name() == field.name())
        {
            return Err(invalid("ancillary output name collides with another field"));
        }
        output_fields.push(field);
        expressions.push(extra);
    }
    let schema = datafusion::arrow::datatypes::Schema::new_with_metadata(
        output_fields,
        schema.metadata().clone(),
    );
    Ok(LogicalPlan::Projection(Projection::try_new_with_schema(
        expressions,
        Arc::new(plan),
        Arc::new(DFSchema::try_from(schema)?),
    )?))
}

/// Constructs a scalar literal under a declared field after checking its actual value.
/// This is a construction-time scalar boundary, never a metadata-only cast of query rows.
///
/// # Errors
/// The scalar storage or local extension value differs from the declared field.
pub fn checked_literal(
    registry: &Registry,
    column: &FieldContract,
    scalar: ScalarValue,
) -> Result<Expr> {
    let field = pse_schema::arrow::field_for(registry, column).map_err(schema_error)?;
    if scalar.data_type() != *field.data_type() {
        return Err(invalid(&format!(
            "literal {} requires {}, but its scalar has {}",
            column.name(),
            field.data_type(),
            scalar.data_type(),
        )));
    }
    pse_relations::validate::validate_column(registry, &field, scalar.to_array()?.as_ref())
        .map_err(|errors| {
            DataFusionError::External(Box::new(pse_relations::RelationError::Validation {
                errors,
            }))
        })?;
    Ok(Expr::Literal(scalar, Some(FieldMetadata::from(&field))))
}

/// Construct a checked literal from one actual Arrow value, retaining nested fields.
/// The pinned scalar extractor reconstructs list child fields without their metadata;
/// nested scalar variants instead retain the actual typed array and its buffers.
///
/// # Errors
/// More or fewer than one value, incompatible physical storage, or a declared field
/// or local-value violation.
pub fn checked_array_literal(
    registry: &Registry,
    column: &FieldContract,
    array: &datafusion::arrow::array::ArrayRef,
) -> Result<Expr> {
    use datafusion::arrow::array::{FixedSizeListArray, LargeListArray, ListArray, StructArray};
    if array.len() != 1 {
        return Err(invalid("an Arrow literal must contain exactly one value"));
    }
    macro_rules! nested {
        ($array:ty, $variant:ident) => {
            ScalarValue::$variant(Arc::new(
                array
                    .as_any()
                    .downcast_ref::<$array>()
                    .ok_or_else(|| invalid("nested literal storage differs from its data type"))?
                    .clone(),
            ))
        };
    }
    let scalar = match array.data_type() {
        DataType::List(_) => nested!(ListArray, List),
        DataType::LargeList(_) => nested!(LargeListArray, LargeList),
        DataType::FixedSizeList(..) => nested!(FixedSizeListArray, FixedSizeList),
        DataType::Struct(_) => nested!(StructArray, Struct),
        _ => ScalarValue::try_from_array(array, 0)?,
    };
    checked_literal(registry, column, scalar)
}

/// Check an actual native field against its expected output obligation. The target
/// does not establish semantics absent from the actual native field.
///
/// # Errors
/// Storage, nullability or complete semantic field meaning differs.
pub fn check_field_output(source: &Field, target: &Field) -> Result<()> {
    if source.is_nullable() && !target.is_nullable() {
        return Err(invalid(&format!(
            "field {} requires {} nullable={}, but native field {} derives {} nullable={}",
            target.name(),
            target.data_type(),
            target.is_nullable(),
            source.name(),
            source.data_type(),
            source.is_nullable(),
        )));
    }
    check_storage_output(source.data_type(), target.data_type())?;
    let semantic = [
        pse_schema::arrow::KEY_LOGICAL_TYPE,
        pse_schema::arrow::KEY_QUANTITY_TYPE,
        pse_schema::arrow::KEY_ENUM,
        pse_schema::arrow::KEY_EXTENSION_NAME,
        pse_schema::arrow::KEY_EXTENSION_METADATA,
    ];
    for key in semantic {
        let actual = source.metadata().get(key);
        let declared = target.metadata().get(key);
        if actual == declared {
            continue;
        }
        if key == pse_schema::arrow::KEY_LOGICAL_TYPE
            && actual.is_none()
            && (primitive(target.data_type())
                || matches!(
                    target.data_type(),
                    DataType::List(_)
                        | DataType::LargeList(_)
                        | DataType::FixedSizeList(..)
                        | DataType::Struct(_)
                ))
            && !target
                .metadata()
                .contains_key(pse_schema::arrow::KEY_EXTENSION_NAME)
        {
            continue;
        }
        return Err(invalid(&format!(
            "field {} semantic key {key} requires {declared:?}, but native field {} derives {actual:?}",
            target.name(),
            source.name(),
        )));
    }
    Ok(())
}

fn check_storage_output(source: &DataType, target: &DataType) -> Result<()> {
    match (source, target) {
        (DataType::List(source), DataType::List(target))
        | (DataType::LargeList(source), DataType::LargeList(target)) => {
            check_field_output(source, target)
        }
        (DataType::FixedSizeList(source, left), DataType::FixedSizeList(target, right))
            if left == right =>
        {
            check_field_output(source, target)
        }
        (DataType::Struct(source), DataType::Struct(target)) if source.len() == target.len() => {
            for (source, target) in source.iter().zip(target) {
                if source.name() != target.name() {
                    return Err(invalid("native struct child name differs from declaration"));
                }
                check_field_output(source, target)?;
            }
            Ok(())
        }
        _ if source == target => Ok(()),
        _ => Err(invalid(&format!(
            "native storage {source} differs from declared {target}"
        ))),
    }
}

fn primitive(kind: &DataType) -> bool {
    matches!(
        kind,
        DataType::Boolean
            | DataType::Int16
            | DataType::Int32
            | DataType::Int64
            | DataType::UInt8
            | DataType::UInt16
            | DataType::UInt32
            | DataType::UInt64
            | DataType::Float64
            | DataType::Utf8
            | DataType::Timestamp(..)
    )
}

fn schema_error(error: pse_schema::SchemaError) -> DataFusionError {
    DataFusionError::External(Box::new(error))
}
fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(format!("declared relation output: {reason}"))
}
