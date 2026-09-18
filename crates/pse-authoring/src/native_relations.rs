// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native comparisons used inside finite source parsing and binding algorithms.
pub(crate) mod exact;
pub(crate) mod plans;
use crate::AuthoringError;
use pse_catalog::session::SnapshotSession;
use pse_ids::CancellationToken;
use pse_schema::model::RelationSpec;
pub(crate) fn contract(reason: &str) -> AuthoringError {
    AuthoringError::Contract {
        at: None,
        reason: reason.into(),
    }
}
pub(crate) async fn unique(
    session: &SnapshotSession,
    role: &str,
    spec: &RelationSpec,
    cancel: &CancellationToken,
) -> Result<std::sync::Arc<pse_catalog::session::CompletedComputation>, AuthoringError> {
    use datafusion::logical_expr::{LogicalPlanBuilder, col, lit};
    let keys = spec
        .primary_key
        .iter()
        .map(|name| col(*name))
        .collect::<Vec<_>>();
    if keys.is_empty() {
        return Err(contract("change relation has no primary key"));
    }
    let plan = LogicalPlanBuilder::from(session.scan_role(role)?)
        .aggregate(
            keys,
            vec![
                datafusion::functions_aggregate::expr_fn::count(lit(1_i64)).alias("__multiplicity"),
            ],
        )
        .and_then(|plan| plan.filter(col("__multiplicity").gt(lit(1_i64))))
        .and_then(|plan| plan.limit(0, Some(1)))
        .and_then(LogicalPlanBuilder::build)
        .map_err(plans::engine)?;
    let completed = std::sync::Arc::new(
        session
            .prepare_rule_plan(plan, cancel)?
            .execute(cancel)
            .await?,
    );
    if completed
        .batches()
        .iter()
        .any(|batch| batch.num_rows() != 0)
    {
        return Err(contract("duplicate primary key in change input"));
    }
    Ok(completed)
}
