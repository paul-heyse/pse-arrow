// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact companion joins supply the lexical axes of each actual source row.
use crate::{
    AuthoringError,
    document::{Batches, load::contract},
};
use datafusion::{
    arrow::array::{Array, BooleanArray, FixedSizeBinaryArray, ListArray, StringArray},
    common::Column,
    logical_expr::{Expr, JoinType, LogicalPlanBuilder, col, lit},
};
use pse_catalog::session::SnapshotSession;
use pse_ids::{CancellationToken, SemanticId};
use std::collections::BTreeMap;

pub(super) type IndexCompanions = BTreeMap<SemanticId, BTreeMap<pse_ids::ContentHash, Vec<String>>>;
fn field(alias: &str, name: &str) -> Expr {
    Expr::Column(Column::new(Some(alias), name))
}

pub(super) async fn load(
    inputs: &Batches,
    session: &SnapshotSession,
    cancel: &CancellationToken,
    completed: &mut crate::change_set::plans::Completions,
) -> Result<IndexCompanions, AuthoringError> {
    let registry = session.registry();
    let mut result = BTreeMap::new();
    for (source, companion, required) in [
        (
            "authored.template_contributions",
            "authored.template_contribution_contracts",
            true,
        ),
        (
            "authored.template_display",
            "authored.template_display_indices",
            false,
        ),
        (
            "authored.template_symbol_expressions",
            "authored.template_symbols",
            true,
        ),
    ] {
        let source = registry
            .relation(source)
            .ok_or_else(|| contract(None, "indexed source declaration absent"))?;
        let companion = registry
            .relation(companion)
            .ok_or_else(|| contract(None, "index companion declaration absent"))?;
        let Some(source_rows) = inputs
            .get(&source.id)
            .filter(|rows| rows.batch().num_rows() > 0)
        else {
            continue;
        };
        let companion_rows = inputs
            .get(&companion.id)
            .ok_or_else(|| contract(None, "indexed source companion input absent"))?;
        let bound = session.with_checked_role_inputs(
            BTreeMap::from([
                ("index_source".to_owned(), source_rows.clone()),
                ("index_companion".to_owned(), companion_rows.clone()),
            ]),
            cancel,
        )?;
        let plan = companion_join(&bound, source, companion)?;
        let mut values = BTreeMap::new();
        for batch in
            crate::change_set::plans::execute_recorded(&bound, plan, cancel, completed).await?
        {
            let keys = batch
                .column(0)
                .as_any()
                .downcast_ref::<FixedSizeBinaryArray>()
                .ok_or_else(|| contract(None, "companion key storage"))?;
            let matched = batch
                .column(1)
                .as_any()
                .downcast_ref::<BooleanArray>()
                .ok_or_else(|| contract(None, "companion match storage"))?;
            let axes = batch
                .column(2)
                .as_any()
                .downcast_ref::<ListArray>()
                .ok_or_else(|| contract(None, "companion axes storage"))?;
            for row in 0..batch.num_rows() {
                if matched.is_null(row) && required {
                    return Err(contract(
                        None,
                        "required source index companion missing or owner differs",
                    ));
                }
                let indices = if matched.is_null(row) {
                    Vec::new()
                } else {
                    string_list(axes, row)?
                };
                if values
                    .insert(
                        pse_ids::ContentHash::try_from_slice(keys.value(row))
                            .map_err(|error| contract(None, &error.to_string()))?,
                        indices,
                    )
                    .is_some()
                {
                    return Err(contract(None, "ambiguous source index companion"));
                }
            }
        }
        result.insert(source.id, values);
    }
    Ok(result)
}
fn string_list(array: &ListArray, row: usize) -> Result<Vec<String>, AuthoringError> {
    let values = array.value(row);
    let values =
        datafusion::arrow::compute::cast(&values, &datafusion::arrow::datatypes::DataType::Utf8)
            .map_err(|error| crate::change_set::plans::engine(error.into()))?;
    let values = values
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| contract(None, "index name storage"))?;
    values
        .iter()
        .map(|value| {
            value
                .map(str::to_owned)
                .ok_or_else(|| contract(None, "null index name"))
        })
        .collect()
}

fn companion_join(
    bound: &SnapshotSession,
    source: &pse_schema::model::RelationSpec,
    companion: &pse_schema::model::RelationSpec,
) -> Result<datafusion::logical_expr::LogicalPlan, AuthoringError> {
    let key = pse_catalog::session::scalar::key(
        source.id,
        source
            .primary_key
            .iter()
            .map(|name| (*name, col(*name)))
            .collect(),
    );
    let mut projection = source
        .columns
        .iter()
        .map(|column| col(column.name()))
        .collect::<Vec<_>>();
    projection.push(key.alias("source_key"));
    let source_plan = LogicalPlanBuilder::from(bound.scan_role("index_source")?)
        .project(projection)
        .and_then(|plan| plan.alias("source"))
        .map_err(crate::change_set::plans::engine)?;
    let mut projection = companion
        .columns
        .iter()
        .map(|column| col(column.name()))
        .collect::<Vec<_>>();
    projection.push(lit(true).alias("matched"));
    let target = LogicalPlanBuilder::from(bound.scan_role("index_companion")?)
        .project(projection)
        .and_then(|plan| plan.alias("companion"))
        .and_then(LogicalPlanBuilder::build)
        .map_err(crate::change_set::plans::engine)?;
    let mut conditions = source
        .primary_key
        .iter()
        .map(|name| field("source", name).eq(field("companion", name)))
        .collect::<Vec<_>>();
    if source.key.name == "template_symbol_expressions" {
        conditions.push(field("source", "template_id").eq(field("companion", "template_id")));
    }
    let plan = source_plan
        .join_on(target, JoinType::Left, conditions)
        .and_then(|plan| {
            plan.project(vec![
                field("source", "source_key"),
                field("companion", "matched"),
                field("companion", "indexed_by"),
            ])
        })
        .and_then(LogicalPlanBuilder::build)
        .map_err(crate::change_set::plans::engine)?;
    Ok(plan)
}
