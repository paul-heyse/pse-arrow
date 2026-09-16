// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Document parsing as a native grouped transform over exact typed source text.
mod parser;

use crate::ParseBudget;
use datafusion::{
    common::{Column, DataFusionError, Result, UnnestOptions},
    functions::core::expr_fn::{get_field, named_struct},
    functions_aggregate::expr_fn::array_agg,
    logical_expr::{Expr, ExprFunctionExt, LogicalPlan, LogicalPlanBuilder, ScalarUDF, lit},
};
use pse_ids::{CancellationToken, MemoryReserver, SemanticId};
use pse_schema::Registry;
use std::sync::Arc;

/// Derive one declared source relation from the complete document inventory of each package.
/// Grouping, ordering, unnesting and field projection are native operations. The UDF
/// owns only the declared document parser; it neither reads files nor publishes data.
/// Preparation constructs a plan and does not parse source text.
/// # Errors
/// An output is not produced by the source loader, a required source column is absent,
/// or the native expressions cannot be typed. Parse/resource failures occur in execution.
pub fn relation_plan(
    documents: LogicalPlan,
    relation: SemanticId,
    registry: &Arc<Registry>,
    budget: ParseBudget,
    reserver: Arc<dyn MemoryReserver>,
    cancel: CancellationToken,
) -> Result<LogicalPlan> {
    let spec = registry
        .relation_by_id(relation)
        .ok_or_else(|| invalid("source output relation is undeclared"))?;
    let source_relation = matches!(
        spec.qualified_name().as_str(),
        "authored.packages" | "authored.documents" | "authored.entities"
    ) || registry.documents().iter().any(|document| {
        document
            .sections
            .iter()
            .any(|section| section.relation == spec.qualified_name())
    });
    if !source_relation {
        return Err(invalid("relation has no declared document producer"));
    }
    let spec = spec.clone();
    let parser = ScalarUDF::from(parser::Parser::new(
        Arc::clone(registry),
        relation,
        budget,
        reserver,
        cancel,
    )?);
    let grouped = LogicalPlanBuilder::from(documents)
        .aggregate(
            vec![column("package_id")],
            vec![
                array_agg(named_struct(vec![
                    lit("document_id"),
                    column("document_id"),
                    lit("path"),
                    column("path"),
                    lit("source_text"),
                    column("source_text"),
                ]))
                .order_by(vec![column("path").sort(true, false)])
                .build()?
                .alias("documents"),
            ],
        )?
        .project(vec![
            parser
                .call(vec![column("package_id"), column("documents")])
                .alias("parsed"),
        ])?
        .unnest_column_with_options("parsed", UnnestOptions::new().with_preserve_nulls(false))?
        .project(spec.columns.iter().map(|field| {
            let value = get_field(column("parsed"), field.name());
            let value = if field.nullable() {
                value
            } else {
                pse_catalog::session::scalar::require_nonnull(value)
            };
            value.alias(field.name())
        }))?
        .build()?;
    pse_catalog::session::output::declare_relation_output(grouped, registry, &spec)
}

fn column(name: &str) -> Expr {
    Expr::Column(Column::from_name(name))
}
fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(reason.into())
}
