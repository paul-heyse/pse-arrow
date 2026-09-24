// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Translate native, resolved MERGE clauses to the pinned Delta builder.
use super::super::settlement::unresolved;
use super::execution::{Command, count, invalid};
use datafusion::{
    common::{
        Column, DFSchemaRef, Result,
        tree_node::{Transformed, TreeNode},
    },
    execution::context::SessionContext,
    logical_expr::{
        Expr, LogicalPlan,
        dml::{MergeIntoAction, MergeIntoClause, MergeIntoClauseKind},
    },
};
use deltalake::{
    DeltaTable,
    operations::merge::{DeleteBuilder, InsertBuilder, UpdateBuilder},
};

pub(super) fn bind(
    schema: &DFSchemaRef,
    target_width: usize,
    on: Expr,
    clauses: Vec<MergeIntoClause>,
) -> Result<Command> {
    let resolve = |expr: Expr| {
        // Native optimizers retain display aliases around rewritten ON predicates.
        // Delta persists a predicate expression, not a SELECT output name.
        expr.unalias_nested()
            .data
            .transform(|expr| {
                if let Expr::Column(column) = expr {
                    let index = schema.index_of_column(&column)?;
                    let alias = if index < target_width {
                        "target"
                    } else {
                        "source"
                    };
                    Ok(Transformed::yes(Expr::Column(Column::new(
                        Some(alias),
                        schema.field(index).name(),
                    ))))
                } else {
                    Ok(Transformed::no(expr))
                }
            })
            .map(|transformed| transformed.data)
    };
    let mut bound = vec![];
    for clause in clauses {
        let action = match clause.action {
            MergeIntoAction::Update(assignments) => MergeIntoAction::Update(
                assignments
                    .into_iter()
                    .map(|(name, expr)| Ok((name, resolve(expr)?)))
                    .collect::<Result<_>>()?,
            ),
            MergeIntoAction::Insert {
                mut columns,
                values,
            } => {
                if columns.is_empty() {
                    columns = schema.fields()[..target_width]
                        .iter()
                        .map(|f| f.name().to_owned())
                        .collect();
                }
                if columns.len() != values.len() {
                    return Err(invalid("MERGE INSERT columns and values differ"));
                }
                MergeIntoAction::Insert {
                    columns,
                    values: values.into_iter().map(&resolve).collect::<Result<_>>()?,
                }
            }
            MergeIntoAction::Delete => MergeIntoAction::Delete,
        };
        bound.push(MergeIntoClause {
            kind: clause.kind.canonical(),
            predicate: clause.predicate.map(&resolve).transpose()?,
            action,
        });
    }
    let on = resolve(on)?;
    deltalake::delta_datafusion::expr::fmt_expr_to_sql(&on)
        .map_err(|error| invalid(&format!("MERGE predicate {on:?}: {error}")))?;
    Ok(Command::Merge { on, clauses: bound })
}

pub(super) async fn execute(
    table: DeltaTable,
    context: &super::super::operation::DeltaOperationContext,
    input: LogicalPlan,
    on: Expr,
    clauses: Vec<MergeIntoClause>,
) -> Result<u64> {
    let state = &context.state;
    let source = SessionContext::new_with_state(state.as_ref().clone())
        .execute_logical_plan(input)
        .await?;
    let mut builder = context.merge(table, source, on);
    for clause in clauses {
        let predicate = clause.predicate;
        builder = match (clause.kind, clause.action) {
            (MergeIntoClauseKind::Matched, MergeIntoAction::Update(values)) => {
                builder.when_matched_update(|b| update(b, predicate, values))
            }
            (MergeIntoClauseKind::Matched, MergeIntoAction::Delete) => {
                builder.when_matched_delete(|b| delete(b, predicate))
            }
            (MergeIntoClauseKind::NotMatchedBySource, MergeIntoAction::Update(values)) => {
                builder.when_not_matched_by_source_update(|b| update(b, predicate, values))
            }
            (MergeIntoClauseKind::NotMatchedBySource, MergeIntoAction::Delete) => {
                builder.when_not_matched_by_source_delete(|b| delete(b, predicate))
            }
            (
                MergeIntoClauseKind::NotMatchedByTarget,
                MergeIntoAction::Insert { columns, values },
            ) => builder.when_not_matched_insert(|b| insert(b, predicate, columns, values)),
            _ => return Err(invalid("invalid MERGE clause kind/action combination")),
        }
        .map_err(|error| datafusion::common::DataFusionError::External(Box::new(error)))?;
    }
    let (after, metrics) = builder.await.map_err(unresolved)?;
    super::super::provider::committed(&after, state).await;
    let affected = metrics
        .num_target_rows_inserted
        .checked_add(metrics.num_target_rows_updated)
        .and_then(|n| n.checked_add(metrics.num_target_rows_deleted))
        .ok_or_else(|| invalid("MERGE affected count overflow"));
    super::super::settlement::observed(after.version(), affected.and_then(count))
}
fn update(
    mut builder: UpdateBuilder,
    predicate: Option<Expr>,
    values: Vec<(String, Expr)>,
) -> UpdateBuilder {
    if let Some(predicate) = predicate {
        builder = builder.predicate(predicate);
    }
    for (name, value) in values {
        builder = builder.update(Column::from_name(name), value);
    }
    builder
}
fn delete(builder: DeleteBuilder, predicate: Option<Expr>) -> DeleteBuilder {
    match predicate {
        Some(predicate) => builder.predicate(predicate),
        None => builder,
    }
}
fn insert(
    mut builder: InsertBuilder,
    predicate: Option<Expr>,
    columns: Vec<String>,
    values: Vec<Expr>,
) -> InsertBuilder {
    if let Some(predicate) = predicate {
        builder = builder.predicate(predicate);
    }
    for (name, value) in columns.into_iter().zip(values) {
        builder = builder.set(Column::from_name(name), value);
    }
    builder
}
