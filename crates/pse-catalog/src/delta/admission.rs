// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Candidate checks expressed as native filters, grouping and anti joins.
#[cfg(test)]
mod tests;
use datafusion::{
    common::{DataFusionError, Result, TableReference},
    execution::{context::SessionContext, session_state::SessionState},
    logical_expr::LogicalPlan,
    physical_plan::execute_stream,
};
use futures_util::TryStreamExt;
use pse_relations::generated::runtime::publications;
use pse_schema::Registry;
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
    let mut catalogs = std::collections::BTreeSet::new();
    let mut checks = Vec::new();
    for member in &record.members {
        catalogs.insert(member.catalog_name.clone());
    }
    for catalog in catalogs {
        let mut inputs = pse_relations::validate::obligations::RelationInputs::new();
        for member in record
            .members
            .iter()
            .filter(|member| member.catalog_name == catalog)
        {
            if inputs.contains_key(&member.relation_id) {
                continue;
            }
            let input = selected_table(record, member.relation_id, &catalog, &context)
                .await?
                .ok_or_else(|| invalid("selected member disappeared"))?;
            inputs.insert(member.relation_id, input);
        }
        let mut local = std::collections::BTreeSet::new();
        for member in record
            .members
            .iter()
            .filter(|member| member.catalog_name == catalog)
        {
            if super::write_evidence::establishes(state, &registry, record, member)? {
                local.insert(member.relation_id);
            }
        }
        checks.extend(
            pse_relations::validate::obligations::ObligationTemplates::new(&registry)
                .bind_required(
                    &inputs,
                    &pse_engine::validation::NativeValidation(state.clone()),
                    |relation, kind| {
                        kind != pse_relations::validate::obligations::ObligationKind::LocalValues
                            || !local.contains(&relation)
                    },
                )?
                .into_iter()
                .map(|check| check.plan),
        );
    }
    Ok(checks)
}

/// Validate a complete artifact's declared inventory before reading member values.
/// Empty relations remain required; a partial checkpoint uses the `relations` kind.
pub(super) fn admit_profile(record: &publications::Row, registry: &Registry) -> Result<()> {
    let present = record
        .members
        .iter()
        .map(|member| member.relation_id)
        .collect();
    pse_relations::validate::obligations::ObligationTemplates::new(registry)
        .require_profile(record.kind.as_str(), &present)
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
pub(super) async fn admit(
    record: &publications::Row,
    registry: Arc<Registry>,
    state: &SessionState,
) -> Result<()> {
    admit_profile(record, &registry)?;
    for check in violation_plans(record, registry, state).await? {
        let physical = state.create_physical_plan(&check).await.map_err(|error| {
            error.context(format!("publication obligation {}", check.display_indent()))
        })?;
        let mut rows = execute_stream(physical, state.task_ctx()).map_err(|error| {
            error.context(format!(
                "executing publication obligation {}",
                check.display_indent()
            ))
        })?;
        while let Some(batch) = rows.try_next().await.map_err(|error| {
            error.context(format!(
                "reading publication obligation {}",
                check.display_indent()
            ))
        })? {
            if batch.num_rows() != 0 {
                return Err(DataFusionError::External(Box::new(
                    crate::EngineError::Admission {
                        path: "publication.members".into(),
                        reason: format!(
                            "candidate violates its relation contract: {}",
                            check.display_indent()
                        ),
                    },
                )));
            }
        }
    }
    Ok(())
}
fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(reason.into())
}
