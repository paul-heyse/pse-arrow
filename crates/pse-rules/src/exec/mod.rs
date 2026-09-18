// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Execute prepared native rule plans whose output contracts were established at construction.
use crate::errmap::internal;
use crate::{RuleError, plan::CompiledRule};
use datafusion::arrow::array::RecordBatch;
use pse_catalog::session::{PlanObservation, SnapshotSession};
use pse_ids::{CancellationToken, SemanticId};
use pse_schema::Registry;

/// Observed rule execution, outside semantic memo identity.
#[derive(Clone, Debug)]
pub struct RuleFired {
    /// Exact registry rule identity.
    pub rule_id: SemanticId,
    /// Qualified rule name.
    pub name: String,
    /// Number of decided rows actually produced.
    pub rows: usize,
    /// Number of unknown candidates actually retained.
    pub unknown_rows: usize,
}
/// Bounded result batches retain their reservation owners and global primary-key order.
#[derive(Debug)]
pub struct RuleOutcome {
    /// Decided-true rows under the admitted head contract.
    pub head: Vec<RecordBatch>,
    /// Actual unknown candidate keys, never inserted into the head.
    pub undecided: Vec<RecordBatch>,
    /// Actual execution counters.
    pub rules_fired: Vec<RuleFired>,
    /// Retained actual optimizer observations for every executed plan.
    pub plans: Vec<PlanObservation>,
}

/// Execute every semantic precheck and then the rule's decided/unknown plans.
///
/// # Errors
/// Resource/cancellation errors keep their class; any unfinished recursion or rejected
/// null policy fails; incompatible values produce `rule::head_schema_mismatch`.
pub async fn execute(
    rule: &CompiledRule,
    session: &SnapshotSession,
    registry: &Registry,
    cancel: &CancellationToken,
) -> Result<RuleOutcome, RuleError> {
    let mut plans = vec![];
    let completed = session
        .prepare_rule_plan(rule.plan.clone(), cancel)
        .map_err(|error| context(rule, "head preparation", error))?
        .execute(cancel)
        .await
        .map_err(|error| context(rule, "decided head", error))?;
    let observed = completed.observation().clone();
    plans.push(observed);
    let head = {
        let name = &rule.head;
        let target = registry
            .relation(name)
            .ok_or_else(|| internal("head declaration absent"))?;
        vec![
            completed
                .into_checked_relation(registry, target, cancel)?
                .into_batch(),
        ]
    };
    let mut undecided = vec![];
    if let Some(plan) = &rule.undecided {
        let completed = session
            .prepare_rule_plan(plan.clone(), cancel)
            .map_err(|error| context(rule, "unknown preparation", error))?
            .execute(cancel)
            .await
            .map_err(|error| context(rule, "unknown candidates", error))?;
        plans.push(completed.observation().clone());
        undecided.extend(completed.into_batches());
    }
    let fired = RuleFired {
        rule_id: rule.rule_id,
        name: rule.name.clone(),
        rows: head.iter().map(RecordBatch::num_rows).sum(),
        unknown_rows: undecided.iter().map(RecordBatch::num_rows).sum(),
    };
    Ok(RuleOutcome {
        head,
        undecided,
        rules_fired: vec![fired],
        plans,
    })
}
fn context(
    rule: &CompiledRule,
    phase: &'static str,
    source: pse_catalog::CatalogError,
) -> RuleError {
    RuleError::Execution {
        rule: rule.name.clone(),
        phase,
        source: Box::new(RuleError::Catalog(source)),
    }
}
