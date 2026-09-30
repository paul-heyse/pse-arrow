// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Compatibility for encountered pairs from one exact selected physical closure.
use crate::{
    columnar::FieldCheckedBatch,
    native::{
        arrow::{array::{Array, FixedSizeBinaryArray, RecordBatch}, datatypes::{DataType, Field, Schema}},
        common::{Column, DFSchema, DataFusionError, Result, ScalarValue},
        functions::core::expr_fn::get_field,
        logical_expr::{EmptyRelation, Expr, LogicalPlan, LogicalPlanBuilder, lit},
    },
};
use super::RelationInputs;
use pse_columnar::CancellationToken;
use pse_ids::SemanticId;
use pse_quantity::{CanonicalConversionPlan, QuantityTypeId, UnitId};
use pse_schema::{Registry, model::RelationKey};
use std::{collections::{BTreeMap, BTreeSet}, sync::Arc};

/// The exact selected physical inputs and the compatible encountered value pairs.
/// Construction never accepts a process-global or caller-asserted quantity registry.
#[derive(Debug)]
pub struct QuantityCompatibility {
    selected: RelationInputs,
    pub(super) compatible: LogicalPlan,
}
impl QuantityCompatibility {
    /// Admit checked definitions and resolve only the distinct encountered pairs.
    /// The caller executes requests and physical selections in the same immutable context.
    /// # Errors
    /// Invalid/missing physical dependencies, wrong request storage or cancellation.
    pub fn admit(
        registry: &Registry,
        selected: &RelationInputs,
        definitions: &BTreeMap<RelationKey, FieldCheckedBatch>,
        requests: &[RecordBatch],
        cancel: &CancellationToken,
    ) -> Result<Self> {
        let (quantities, _) = crate::physical::inventory(definitions, registry, cancel, None)
            .map_err(pse_columnar::external)?;
        let mut accepted = BTreeSet::new();
        for batch in requests {
            let quantity = ids(batch, 0)?;
            let unit = ids(batch, 1)?;
            for row in 0..batch.num_rows() {
                cancel.checkpoint().map_err(pse_columnar::external)?;
                let quantity = identity(quantity, row)?;
                let unit = identity(unit, row)?;
                if CanonicalConversionPlan::registered(
                    &quantities, QuantityTypeId::from_id(quantity), UnitId::from_id(unit),
                ).is_ok() {
                    accepted.insert((quantity, unit));
                }
            }
        }
        let schema = Arc::new(DFSchema::try_from(Schema::new(vec![
            Field::new("admitted_quantity", DataType::FixedSizeBinary(16), false),
            Field::new("admitted_unit", DataType::FixedSizeBinary(16), false),
        ]))?);
        let compatible = if accepted.is_empty() {
            LogicalPlan::EmptyRelation(EmptyRelation { produce_one_row: false, schema })
        } else {
            let rows = accepted.into_iter().map(|(quantity, unit)| vec![
                lit(ScalarValue::FixedSizeBinary(16, Some(quantity.as_bytes().to_vec()))),
                lit(ScalarValue::FixedSizeBinary(16, Some(unit.as_bytes().to_vec()))),
            ]).collect();
            LogicalPlanBuilder::values(rows)?
                .project([
                    Expr::Column(Column::from_name("column1")).alias("admitted_quantity"),
                    Expr::Column(Column::from_name("column2")).alias("admitted_unit"),
                ])?.build()?
        };
        Ok(Self { selected: selected.clone(), compatible })
    }
    pub(super) fn require_selection(&self, selected: &RelationInputs) -> Result<()> {
        if &self.selected != selected {
            return Err(DataFusionError::Plan("physical compatibility belongs to different selected inputs".into()));
        }
        Ok(())
    }
}

/// Distinct quantity/unit requests, preserving absence: no occurrences means no inventory.
/// # Errors
/// Native projection or nested occurrence construction fails.
pub fn quantity_requests(inputs: &RelationInputs) -> Result<Option<LogicalPlan>> {
    let mut all: Option<LogicalPlan> = None;
    for input in inputs.values() {
        for occurrence in super::nested_values::occurrences(
            input, input.schema().fields().iter().map(AsRef::as_ref), super::quantities::is_quantity,
        )? {
            let value = occurrence.value()?;
            let pairs = LogicalPlanBuilder::from(occurrence.input).project([
                get_field(value.clone(), "quantity_type_id").alias("quantity"),
                get_field(value, "unit_id").alias("unit"),
            ])?.build()?;
            all = Some(match all {
                None => pairs,
                Some(previous) => LogicalPlanBuilder::from(previous).union(pairs)?.build()?,
            });
        }
    }
    all.map(|plan| LogicalPlanBuilder::from(plan).distinct()?.build()).transpose()
}
fn ids(batch: &RecordBatch, column: usize) -> Result<&FixedSizeBinaryArray> {
    batch.columns().get(column).and_then(|array| array.as_any().downcast_ref())
        .ok_or_else(|| DataFusionError::Plan("physical request needs two identity columns".into()))
}
fn identity(array: &FixedSizeBinaryArray, row: usize) -> Result<SemanticId> {
    if array.is_null(row) {
        return Err(DataFusionError::Plan("physical request identity cannot be null".into()));
    }
    Ok(SemanticId::from_bytes(array.value(row).try_into()
        .map_err(|_| DataFusionError::Plan("physical request identity has wrong width".into()))?))
}
