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
    let mut spans = vec![];
    for field in input.schema().fields() {
        if contains_span(field) {
            descend(
                field,
                LogicalPlanBuilder::from(input.clone())
                    .project(vec![column(field.name()).alias("value")])?
                    .build()?,
                &mut spans,
            )?;
        }
    }
    spans
        .into_iter()
        .map(|input| check(input, documents))
        .collect()
}

fn contains_span(field: &Field) -> bool {
    if is_span(field) {
        return true;
    }
    match field.data_type() {
        DataType::Struct(fields) => fields.iter().any(|f| contains_span(f)),
        DataType::List(child) | DataType::LargeList(child) | DataType::FixedSizeList(child, _) => {
            contains_span(child)
        }
        _ => false,
    }
}
fn is_span(field: &Field) -> bool {
    field
        .metadata()
        .get(pse_schema::arrow::KEY_EXTENSION_NAME)
        .is_some_and(|name| name == "pse.source_span")
}

fn descend(field: &Field, input: LogicalPlan, output: &mut Vec<LogicalPlan>) -> Result<()> {
    let input = LogicalPlanBuilder::from(input)
        .filter(column("value").is_not_null())?
        .build()?;
    if is_span(field) {
        output.push(
            LogicalPlanBuilder::from(input)
                .project(vec![
                    get_field(column("value"), "document_id").alias("source_document"),
                    get_field(column("value"), "start").alias("source_start"),
                    get_field(column("value"), "end").alias("source_end"),
                ])?
                .build()?,
        );
        return Ok(());
    }
    match field.data_type() {
        DataType::Struct(fields) => {
            for child in fields.iter().filter(|f| contains_span(f)) {
                descend(
                    child,
                    LogicalPlanBuilder::from(input.clone())
                        .project(vec![
                            get_field(column("value"), child.name()).alias("value"),
                        ])?
                        .build()?,
                    output,
                )?;
            }
        }
        DataType::List(child) | DataType::LargeList(child) | DataType::FixedSizeList(child, _) => {
            descend(
                child,
                LogicalPlanBuilder::from(input)
                    .unnest_column("value")?
                    .build()?,
                output,
            )?;
        }
        _ => {}
    }
    Ok(())
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
