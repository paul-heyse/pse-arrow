// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Source provenance uses native nested projections, unnesting and joins to exact text.
#[cfg(test)]
mod tests;

use datafusion::{
    arrow::datatypes::{DataType, Field},
    common::{Column, Result},
    functions::{core::expr_fn::get_field, string::expr_fn::octet_length},
    logical_expr::{Expr, JoinType, LogicalPlan, LogicalPlanBuilder, cast, lit},
};

/// Derive checks from the declared field annotations, including nested list/struct
/// occurrences. Null containers carry no source claim. Offsets address UTF-8 bytes.
pub(super) fn plans(
    input: &LogicalPlan,
    documents: Option<&LogicalPlan>,
) -> Result<Vec<LogicalPlan>> {
    super::nested_values::occurrences(
        input,
        input.schema().fields().iter().map(AsRef::as_ref),
        is_span,
    )?
    .into_iter()
    .map(|occurrence| {
        let value = occurrence.value()?;
        let input = LogicalPlanBuilder::from(occurrence.input)
            .project(vec![
                get_field(value.clone(), "document_id").alias("source_document"),
                get_field(value.clone(), "start").alias("source_start"),
                get_field(value, "end").alias("source_end"),
            ])?
            .build()?;
        check(input, documents)
    })
    .collect()
}

fn is_span(field: &Field) -> bool {
    field
        .metadata()
        .get(pse_schema::arrow::KEY_EXTENSION_NAME)
        .is_some_and(|name| name == "pse.source_span")
}

fn check(input: LogicalPlan, documents: Option<&LogicalPlan>) -> Result<LogicalPlan> {
    let invalid = if let Some(documents) = documents {
        let documents = LogicalPlanBuilder::from(documents.clone())
            .project(vec![
                column("document_id").alias("declared_document"),
                cast(octet_length(column("source_text")), DataType::Int64).alias("source_bytes"),
            ])?
            .build()?;
        LogicalPlanBuilder::from(input)
            .join(
                documents,
                JoinType::Left,
                (
                    vec![Column::from_name("source_document")],
                    vec![Column::from_name("declared_document")],
                ),
                None,
            )?
            .filter(
                column("declared_document")
                    .is_null()
                    .or(column("source_start").lt(lit(0_i64)))
                    .or(column("source_start").gt(column("source_end")))
                    .or(column("source_end").gt(column("source_bytes"))),
            )?
    } else {
        // With no source relation every visible source claim is dangling. Empty or
        // entirely null spans are valid without manufacturing an empty document table.
        LogicalPlanBuilder::from(input)
    };
    invalid
        .project(vec![
            lit("source span lies outside its selected source text").alias("violation"),
        ])?
        .limit(0, Some(1))?
        .build()
}
fn column(name: &str) -> Expr {
    Expr::Column(Column::from_name(name))
}
