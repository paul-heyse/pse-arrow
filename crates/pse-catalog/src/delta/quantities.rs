// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed scalar quantity references are checked against the exact selected type and
//! unit relations. No process-global quantity registry can satisfy a missing member.

use datafusion::{
    arrow::datatypes::Field,
    common::{Column, Result},
    execution::context::SessionContext,
    functions::core::expr_fn::get_field,
    functions_nested::expr_fn::array_length,
    logical_expr::{Expr, JoinType, LogicalPlan, LogicalPlanBuilder, Operator, binary_expr, lit},
};
use pse_relations::generated::runtime::publications;
use pse_schema::Registry;

pub(super) async fn plans(
    input: &LogicalPlan,
    record: &publications::Row,
    registry: &Registry,
    catalog: &str,
    context: &SessionContext,
) -> Result<Vec<LogicalPlan>> {
    let occurrences = super::nested_values::occurrences(
        input,
        input.schema().fields().iter().map(AsRef::as_ref),
        is_quantity,
    )?;
    if occurrences.is_empty() {
        return Ok(vec![]);
    }
    let types = selected(
        "reference.quantity_types",
        record,
        registry,
        catalog,
        context,
    )
    .await?;
    let units = selected("reference.units", record, registry, catalog, context).await?;
    occurrences
        .into_iter()
        .map(|occurrence| {
            let value = occurrence.value()?;
            let values = LogicalPlanBuilder::from(occurrence.input)
                .project(vec![
                    get_field(value.clone(), "quantity_type_id").alias("quantity"),
                    get_field(value, "unit_id").alias("unit"),
                ])?
                .build()?;
            let invalid = if let (Some(types), Some(units)) = (&types, &units) {
                incompatible(values, types, units)?
            } else {
                // With no selected definitions every visible quantity is unresolved.
                values
            };
            LogicalPlanBuilder::from(invalid)
                .project(vec![lit(format!(
            "quantity {} requires a selected scalar type and compatible representation unit",
            occurrence.path.join(".")
        )).alias("violation")])?
                .limit(0, Some(1))?
                .build()
        })
        .collect()
}

fn incompatible(
    values: LogicalPlan,
    types: &LogicalPlan,
    units: &LogicalPlan,
) -> Result<LogicalPlan> {
    let types = LogicalPlanBuilder::from(types.clone())
        .project(vec![
            column("quantity_type_id").alias("declared_quantity"),
            column("canonical_unit_id").alias("canonical_unit"),
            column("reference_state_id").alias("quantity_reference"),
            column("shape").alias("quantity_shape"),
        ])?
        .build()?;
    let values = join(values, types, "quantity", "declared_quantity")?;
    let values = join(
        values,
        unit_projection(units, "actual")?,
        "unit",
        "actual_unit",
    )?;
    let values = join(
        values,
        unit_projection(units, "canonical")?,
        "canonical_unit",
        "canonical_id",
    )?;
    LogicalPlanBuilder::from(values)
        .filter(
            column("declared_quantity")
                .is_null()
                .or(column("actual_unit").is_null())
                .or(column("canonical_id").is_null())
                .or(binary_expr(
                    column("actual_dimension"),
                    Operator::IsDistinctFrom,
                    column("canonical_dimension"),
                ))
                .or(array_length(column("quantity_shape")).not_eq(lit(0_i64)))
                .or(incompatible_reference("actual_reference"))
                .or(incompatible_reference("canonical_reference")),
        )?
        .build()
}

fn unit_projection(units: &LogicalPlan, role: &str) -> Result<LogicalPlan> {
    LogicalPlanBuilder::from(units.clone())
        .project(vec![
            column("unit_id").alias(if role == "actual" {
                "actual_unit"
            } else {
                "canonical_id"
            }),
            column("dimension").alias(format!("{role}_dimension")),
            column("reference_state_id").alias(format!("{role}_reference")),
        ])?
        .build()
}
fn incompatible_reference(name: &str) -> Expr {
    column(name).is_not_null().and(binary_expr(
        column(name),
        Operator::IsDistinctFrom,
        column("quantity_reference"),
    ))
}
fn join(left: LogicalPlan, right: LogicalPlan, source: &str, target: &str) -> Result<LogicalPlan> {
    LogicalPlanBuilder::from(left)
        .join(
            right,
            JoinType::Left,
            (
                vec![Column::from_name(source)],
                vec![Column::from_name(target)],
            ),
            None,
        )?
        .build()
}
async fn selected(
    name: &str,
    record: &publications::Row,
    registry: &Registry,
    catalog: &str,
    context: &SessionContext,
) -> Result<Option<LogicalPlan>> {
    if let Some(spec) = registry.relation(name) {
        super::admission::selected_table(record, spec.id, catalog, context).await
    } else {
        Ok(None)
    }
}
fn is_quantity(field: &Field) -> bool {
    field
        .metadata()
        .get(pse_schema::arrow::KEY_EXTENSION_NAME)
        .is_some_and(|name| name == "pse.quantity_value")
}
fn column(name: &str) -> Expr {
    Expr::Column(Column::from_name(name))
}

#[cfg(test)]
mod tests;
