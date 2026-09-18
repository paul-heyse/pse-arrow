// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Numerical results are meaningful only in the exact selected program's order.
//! Native joins and aggregates establish dimensions and complete sparse coordinates.

use datafusion::{
    common::{Column, Result},
    execution::context::SessionContext,
    functions::core::expr_fn::coalesce,
    functions_aggregate::expr_fn::count,
    functions_nested::expr_fn::array_length,
    logical_expr::{Expr, JoinType, LogicalPlan, LogicalPlanBuilder, lit},
};
use pse_relations::generated::runtime::publications;
use pse_schema::{Registry, model::RelationSpec};

pub(super) async fn plans(
    input: &LogicalPlan,
    spec: &RelationSpec,
    record: &publications::Row,
    registry: &Registry,
    catalog: &str,
    context: &SessionContext,
) -> Result<Vec<LogicalPlan>> {
    let name = spec.qualified_name();
    if name == "runtime.numerical_programs" {
        let coordinates = selected(
            "runtime.jacobian_coordinates",
            record,
            registry,
            catalog,
            context,
        )
        .await?;
        return coordinate_coverage(input, coordinates, spec).map(|plan| vec![plan]);
    }
    let valid = match name.as_str() {
        "runtime.numerical_evaluations" => column("residual_dimension")
            .eq(column("expected_residuals"))
            .and(column("variable_dimension").eq(array_length(column("expected_variables"))))
            .and(column("jacobian_dimension").eq(column("expected_jacobian"))),
        "runtime.solver_outcomes" => {
            column("variable_columns").eq(column("expected_variables"))
                // Subtract from the nonnegative program dimension to avoid overflow in
                // a forged maximum constraint count. A solver needs an objective row.
                .and(column("expected_residuals").gt(lit(0_i64)))
                .and(column("constraint_dimension").eq(column("expected_residuals") - lit(1_i64)))
        }
        "runtime.jacobian_coordinates" => column("ordinal")
            .lt(column("expected_jacobian"))
            .and(column("residual").lt(column("expected_residuals")))
            .and(column("variable").lt(array_length(column("expected_variables")))),
        _ => return Ok(vec![]),
    };
    let programs = selected(
        "runtime.numerical_programs",
        record,
        registry,
        catalog,
        context,
    )
    .await?;
    let invalid = if let Some(programs) = programs {
        let programs = LogicalPlanBuilder::from(programs)
            .project(vec![
                column("program_id").alias("expected_program"),
                column("residual_dimension").alias("expected_residuals"),
                column("variable_columns").alias("expected_variables"),
                column("jacobian_dimension").alias("expected_jacobian"),
            ])?
            .build()?;
        LogicalPlanBuilder::from(join(
            input.clone(),
            programs,
            "program_id",
            "expected_program",
        )?)
        .filter(column("expected_program").is_null().or(valid.is_not_true()))?
        .build()?
    } else {
        input.clone()
    };
    let mut plans = vec![super::admission::violation(
        invalid,
        spec,
        "selected numerical program mismatch",
    )?];
    if name == "runtime.jacobian_coordinates" {
        let repeated = LogicalPlanBuilder::from(input.clone())
            .aggregate(
                vec![column("program_id"), column("residual"), column("variable")],
                vec![count(lit(1_i64)).alias("coordinate_count")],
            )?
            .filter(column("coordinate_count").gt(lit(1_i64)))?
            .build()?;
        plans.push(super::admission::violation(
            repeated,
            spec,
            "repeated sparse coordinate",
        )?);
    }
    Ok(plans)
}

fn coordinate_coverage(
    programs: &LogicalPlan,
    coordinates: Option<LogicalPlan>,
    spec: &RelationSpec,
) -> Result<LogicalPlan> {
    let (input, count) = if let Some(coordinates) = coordinates {
        let counts = LogicalPlanBuilder::from(coordinates)
            .aggregate(
                vec![column("program_id").alias("coordinate_program")],
                vec![count(lit(1_i64)).alias("coordinate_count")],
            )?
            .build()?;
        (
            join(programs.clone(), counts, "program_id", "coordinate_program")?,
            coalesce(vec![column("coordinate_count"), lit(0_i64)]),
        )
    } else {
        (programs.clone(), lit(0_i64))
    };
    // Unique ordinal keys and bounds are admitted separately. Together with this
    // count they prove exactly 0..jacobian_dimension, including the zero case.
    let missing = LogicalPlanBuilder::from(input)
        .filter(count.not_eq(column("jacobian_dimension")))?
        .build()?;
    super::admission::violation(missing, spec, "incomplete sparse coordinate selection")
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
fn column(name: &str) -> Expr {
    Expr::Column(Column::from_name(name))
}

#[cfg(test)]
mod tests;
