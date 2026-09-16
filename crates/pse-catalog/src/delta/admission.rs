// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Candidate checks expressed as native filters, grouping and anti joins.
#[cfg(test)]
mod tests;
use datafusion::{
    common::{Column, DataFusionError, Result, TableReference},
    execution::{context::SessionContext, session_state::SessionState},
    functions_aggregate::expr_fn::count,
    logical_expr::{Expr, JoinType, LogicalPlan, LogicalPlanBuilder, lit},
    physical_plan::execute_stream,
};
use futures_util::TryStreamExt;
use pse_relations::generated::runtime::publications;
use pse_schema::{
    Registry,
    model::{FieldContract, RelationSpec},
};
use std::sync::Arc;

/// Native violation queries for the exact candidate selection. No constraints are
/// advertised to the optimizer before these checks establish them.
/// # Errors
/// A contract/reference is missing or native query construction fails.
pub async fn violation_plans(
    record: &publications::Row,
    registry: Arc<Registry>,
    state: &SessionState,
) -> Result<Vec<LogicalPlan>> {
    let context = SessionContext::new_with_state(state.clone());
    let mut checks = vec![];
    for member in &record.members {
        let spec = registry
            .relation_by_id(member.relation_id)
            .ok_or_else(|| invalid("unknown candidate relation"))?;
        let reference = TableReference::full(
            member.catalog_name.clone(),
            member.schema_name.clone(),
            member.table_name.clone(),
        );
        let input = context.table(reference).await?.into_unoptimized_plan();
        checks.push(local_values(&registry, spec, input.clone(), state)?);
        let documents = if let Some(documents) = registry.relation("authored.documents") {
            selected_table(record, documents.id, &member.catalog_name, &context).await?
        } else {
            None
        };
        checks.extend(super::source_spans::plans(&input, documents.as_ref())?);
        checks.extend(
            super::quantities::plans(&input, record, &registry, &member.catalog_name, &context)
                .await?,
        );
        // Empty declared keys deliberately mean a singleton relation.
        let keys: Vec<_> = spec.primary_key.iter().map(|name| column(name)).collect();
        let duplicates = LogicalPlanBuilder::from(input.clone())
            .aggregate(keys, vec![count(lit(1i64)).alias("pse_key_count")])?
            .filter(column("pse_key_count").gt(lit(1i64)))?
            .build()?;
        checks.push(violation(duplicates, spec, "duplicate primary key")?);
        for occurrence in super::nested_values::occurrences(
            &input,
            spec.columns.iter().map(FieldContract::field),
            |field| FieldContract::from_field(field.clone()).fk().is_some(),
        )? {
            let field = FieldContract::from_field(occurrence.field);
            let fk = field
                .fk()
                .ok_or_else(|| invalid("missing reference declaration"))?;
            let target = registry
                .relation(fk.relation)
                .ok_or_else(|| invalid("undeclared reference target"))?;
            let target = selected_table(record, target.id, &member.catalog_name, &context).await?;
            let missing = if let Some(target) = target {
                LogicalPlanBuilder::from(occurrence.input)
                    .alias("candidate")?
                    .join(
                        LogicalPlanBuilder::from(target)
                            .alias("reference")?
                            .build()?,
                        JoinType::LeftAnti,
                        (
                            vec![Column::new(Some("candidate"), "value")],
                            vec![Column::new(Some("reference"), fk.column)],
                        ),
                        None,
                    )?
                    .build()?
            } else {
                // No target can satisfy a visible reference. Null/empty source
                // containers make no claim and do not require an invented table.
                occurrence.input
            };
            checks.push(violation(
                missing,
                spec,
                &format!("missing reference {}", occurrence.path.join(".")),
            )?);
        }
    }
    Ok(checks)
}
pub(super) async fn selected_table(
    record: &publications::Row,
    relation: pse_ids::SemanticId,
    catalog: &str,
    context: &SessionContext,
) -> Result<Option<LogicalPlan>> {
    let mut candidates = record
        .members
        .iter()
        .filter(|m| m.catalog_name == catalog && m.relation_id == relation);
    let Some(first) = candidates.next() else {
        return Ok(None);
    };
    if candidates.any(|m| {
        m.table_uri != first.table_uri
            || m.delta_version != first.delta_version
            || m.selection != first.selection
    }) {
        return Err(invalid("reference target has ambiguous selected revisions"));
    }
    Ok(Some(
        context
            .table(TableReference::full(
                first.catalog_name.clone(),
                first.schema_name.clone(),
                first.table_name.clone(),
            ))
            .await?
            .into_unoptimized_plan(),
    ))
}
fn local_values(
    registry: &Registry,
    spec: &RelationSpec,
    input: LogicalPlan,
    state: &SessionState,
) -> Result<LogicalPlan> {
    let schema = pse_schema::arrow::relation_schema(registry, spec).map_err(external)?;
    let mut predicates = super::row_checks::expressions(&schema, state)?;
    predicates.push(super::predicates::relation(registry, &schema)?);
    let predicate = super::predicates::combine(predicates);
    violation(
        LogicalPlanBuilder::from(input)
            .filter(predicate.is_not_true())?
            .build()?,
        spec,
        "local values",
    )
}
pub(super) async fn admit(
    record: &publications::Row,
    registry: Arc<Registry>,
    state: &SessionState,
) -> Result<()> {
    for check in violation_plans(record, registry, state).await? {
        let mut rows = execute_stream(state.create_physical_plan(&check).await?, state.task_ctx())?;
        while let Some(batch) = rows.try_next().await? {
            if batch.num_rows() != 0 {
                return Err(DataFusionError::Execution(format!(
                    "candidate violates its relation contract: {}",
                    check.display_indent()
                )));
            }
        }
    }
    Ok(())
}
fn violation(input: LogicalPlan, spec: &RelationSpec, reason: &str) -> Result<LogicalPlan> {
    LogicalPlanBuilder::from(input)
        .project(vec![
            lit(format!("{}: {reason}", spec.qualified_name())).alias("violation"),
        ])?
        .limit(0, Some(1))?
        .build()
}
fn column(name: &str) -> Expr {
    Expr::Column(Column::from_name(name))
}

fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(reason.into())
}
fn external(error: impl std::error::Error + Send + Sync + 'static) -> DataFusionError {
    DataFusionError::External(Box::new(error))
}
