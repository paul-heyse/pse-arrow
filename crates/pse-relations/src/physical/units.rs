// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared native reconciliation; consumers execute against their exact selected inputs.
use crate::native::{
    common::{Column, DataFusionError, Result},
    functions_aggregate::expr_fn::{count, count_distinct},
    logical_expr::{Expr, LogicalPlan, LogicalPlanBuilder, col, lit},
};
use crate::validate::obligations::RelationInputs;
use pse_schema::Registry;

/// Duplicate-source/contradictory-definition refusals and the exact merged unit rows.
#[derive(Debug)]
pub struct UnitReconciliation {
    /// Nonempty output refuses the inventory.
    pub conflicts: LogicalPlan,
    /// Distinct identical cross-source definitions in authoritative field order.
    pub merged: LogicalPlan,
}

/// Reconcile normalized/reference units using their full declared values.
/// # Errors
/// Missing declarations or incompatible native input fields.
pub fn reconcile_units(inputs: &RelationInputs, registry: &Registry) -> Result<Option<UnitReconciliation>> {
    let Some(normalized) = registry.relation("normalized.units")
        .and_then(|spec| inputs.get(&spec.id)) else { return Ok(None); };
    let reference = registry.relation("reference.units")
        .ok_or_else(|| DataFusionError::Plan("normalized units have no physical declaration".into()))?;
    let project = |input: &LogicalPlan| {
        LogicalPlanBuilder::from(input.clone())
            .project(reference.columns.iter().map(|field| col(field.name())))?.build()
    };
    let mut sources = vec![project(normalized)?];
    if let Some(input) = inputs.get(&reference.id) { sources.push(project(input)?); }
    let mut duplicates = Vec::new();
    for input in &sources {
        duplicates.push(LogicalPlanBuilder::from(input.clone())
            .aggregate([col("unit_id")], [count(lit(1_i64)).alias("__count")])?
            .filter(col("__count").gt(lit(1_i64)))?.project([col("unit_id")])?.build()?);
    }
    let mut input = sources.remove(0);
    for next in sources { input = LogicalPlanBuilder::from(input).union(next)?.build()?; }
    let definition = crate::identity::key(reference.id, reference.columns.iter()
        .map(|field| (field.name(), Expr::Column(Column::from_name(field.name())))).collect());
    let mut conflicts = LogicalPlanBuilder::from(input.clone())
        .project([col("unit_id"), definition.alias("__definition")])?
        .aggregate([col("unit_id")], [count_distinct(col("__definition")).alias("__count")])?
        .filter(col("__count").gt(lit(1_i64)))?.project([col("unit_id")])?.build()?;
    for duplicate in duplicates { conflicts = LogicalPlanBuilder::from(conflicts).union(duplicate)?.build()?; }
    let merged = LogicalPlanBuilder::from(input).distinct()?
        .sort([col("unit_id").sort(true, false)])?.build()?;
    Ok(Some(UnitReconciliation { conflicts, merged }))
}
