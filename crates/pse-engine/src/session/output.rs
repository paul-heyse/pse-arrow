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

/// Restore only relation-level metadata lost by native plan optimization. Every
/// physical field, including nested semantic annotations, must already agree.
/// The returned batch keeps the original buffers and allocation leases.
/// # Errors
/// A field differs from the declared relation or Arrow refuses the schema update.
pub fn restore_relation_metadata(
    batch: pse_columnar::owned_buffer::OwnedRecordBatch,
    schema: &datafusion::arrow::datatypes::Schema,
) -> Result<pse_columnar::owned_buffer::OwnedRecordBatch> {
    if batch.batch().schema().fields() != schema.fields() {
        return Err(invalid("executed fields differ from the declared relation"));
    }
    batch
        .with_schema_metadata(schema.metadata().clone())
        .map_err(pse_columnar::external)
}

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
        Ok(LogicalPlan::Projection(Projection::try_new(
            expressions,
            Arc::new(plan),
        )?))
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
    same_field_cases(schema, vec![(condition, yes)], no)
}

/// Build one flat native CASE over alternatives with the same value meaning.
/// Keeping alternatives in a single CASE avoids recursively resolving an expanding
/// tree of metadata carriers during native field derivation.
///
/// # Errors
/// Alternatives differ in storage or semantic metadata, or native CASE is invalid.
pub fn same_field_cases(schema: &DFSchema, cases: Vec<(Expr, Expr)>, no: Expr) -> Result<Expr> {
    let (_, no_field) = no.to_field(schema)?;
    let mut metadata = no_field.metadata().clone();
    let mut conditions = Vec::with_capacity(cases.len());
    let mut alternatives = Vec::with_capacity(cases.len());
    for (condition, value) in cases {
        let (_, field) = value.to_field(schema)?;
        if field.data_type() != no_field.data_type()
            || !super::admission::same_value_metadata(&field, &no_field)
        {
            return Err(invalid(
                "CASE alternatives do not establish the same field meaning",
            ));
        }
        metadata.retain(|key, value| field.metadata().get(key) == Some(value));
        conditions.push(condition);
        alternatives.push(value);
    }
    if conditions.is_empty() {
        return Ok(no);
    }
    let expression = datafusion::logical_expr::conditional_expressions::CaseBuilder::new(
        None,
        conditions,
        alternatives,
        Some(Box::new(no)),
    )
    .end()?;
    let mut arguments = vec![expression];
    for (key, value) in metadata
        .iter()
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
    registry.contract(spec).map_err(schema_error)?;
    let spec = registry
        .relation_by_id(spec.id)
        .ok_or_else(|| invalid("admitted output relation is absent"))?;
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
            pse_columnar::external(pse_relations::RelationError::Validation { errors })
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
    let field =
        Arc::new(pse_schema::arrow::field_for(registry, column).map_err(pse_columnar::external)?);
    let scalar = pse_schema::NativeLiteral::new(field, Arc::clone(array))
        .and_then(|literal| literal.scalar())
        .map_err(pse_columnar::external)?;
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
        pse_schema::arrow::KEY_ROW_KEY_ENCODING,
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
    pse_columnar::external(error)
}
fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(format!("declared relation output: {reason}"))
}

#[cfg(test)]
mod consolidation_unit {
    use super::*;
    use datafusion::logical_expr::{EmptyRelation, LogicalPlan};

    #[test]
    fn relation_metadata_restoration_preserves_buffers_and_refuses_changed_fields() -> Result<()> {
        use datafusion::arrow::{
            array::Float64Array, datatypes::Schema, record_batch::RecordBatch,
        };
        use pse_columnar::{
            CancellationToken, GreedyMemoryPool, MemoryPool, owned_buffer::OwnedRecordBatch,
        };
        let field = Field::new("value", DataType::Float64, false).with_metadata(
            std::collections::HashMap::from([("pse.quantity_type".into(), "physical-type".into())]),
        );
        let source = Arc::new(Schema::new(vec![field.clone()]));
        let declared = Schema::new_with_metadata(
            vec![field.clone()],
            std::collections::HashMap::from([("pse.relation".into(), "declared-output".into())]),
        );
        let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(1 << 20));
        let batch = OwnedRecordBatch::export(
            RecordBatch::try_new(source, vec![Arc::new(Float64Array::from(vec![1.0, -0.0]))])?,
            &pool,
            &CancellationToken::new(),
        )
        .map_err(pse_columnar::external)?;
        let before = pool.reserved();
        let restored = restore_relation_metadata(batch.clone(), &declared)?;
        assert_eq!(restored.batch().schema().metadata(), declared.metadata());
        assert!(Arc::ptr_eq(
            batch.batch().column(0),
            restored.batch().column(0)
        ));
        assert_eq!(pool.reserved(), before);
        let foreign = Schema::new(vec![
            field
                .clone()
                .with_metadata(std::collections::HashMap::new()),
        ]);
        assert!(restore_relation_metadata(batch.clone(), &foreign).is_err());
        let nullable = Schema::new(vec![field.with_nullable(true)]);
        assert!(restore_relation_metadata(batch.clone(), &nullable).is_err());
        drop(batch);
        assert_eq!(pool.reserved(), before);
        drop(restored);
        assert_eq!(pool.reserved(), 0);
        Ok(())
    }

    #[test]
    fn declared_output_rejects_forged_contract_without_executing_a_plan() -> Result<()> {
        let registry = crate::validation::registry().map_err(schema_error)?;
        let original = registry
            .relation("reference.dimensions")
            .ok_or_else(|| invalid("fixture relation missing"))?;
        let schema =
            pse_schema::arrow::relation_schema(registry, original).map_err(schema_error)?;
        let plan = LogicalPlan::EmptyRelation(EmptyRelation {
            produce_one_row: false,
            schema: Arc::new(DFSchema::try_from(schema)?),
        });
        assert!(declare_relation_output(plan.clone(), registry, original).is_ok());
        let mut changed = original.clone();
        changed.columns[0] = changed.columns[0]
            .clone()
            .with_nullable(!changed.columns[0].nullable());
        assert_eq!(changed.fingerprint, original.fingerprint);
        assert!(declare_relation_output(plan, registry, &changed).is_err());
        Ok(())
    }
}
