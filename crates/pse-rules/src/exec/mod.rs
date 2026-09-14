// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Execute optimized relational plans and admit actual values against their derivation.
use crate::errmap::internal;
use crate::{RuleError, plan::CompiledRule};
use datafusion::arrow::array::RecordBatch;
use datafusion::arrow::datatypes::Schema;
use pse_catalog::session::{PlanObservation, SnapshotSession};
use pse_ids::{CancellationToken, SemanticId};
use pse_schema::Registry;
use std::sync::Arc;

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
    /// Engine plan rendering for attribution only; not a semantic cache input.
    pub explain_pgjson: String,
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
    for (plan, reason) in &rule.checks {
        let (rows, observed) = session
            .execute_rule_observed(plan.clone(), cancel)
            .await
            .map_err(|error| context(rule, "semantic precheck", error))?;
        plans.push(observed);
        if rows.iter().any(|batch| batch.num_rows() != 0) {
            return Err(internal(reason.clone()));
        }
    }
    let (raw, observed) = session
        .execute_rule_observed(rule.plan.clone(), cancel)
        .await
        .map_err(|error| context(rule, "decided head", error))?;
    let explain_pgjson = observed.explain_pgjson().to_owned();
    plans.push(observed);
    let mut head = Vec::with_capacity(raw.len());
    for batch in raw {
        head.push(admit(rule, registry, &batch, false)?);
    }
    let mut undecided = vec![];
    if let Some(plan) = &rule.undecided {
        let (rows, observed) = session
            .execute_rule_observed(plan.clone(), cancel)
            .await
            .map_err(|error| context(rule, "unknown candidates", error))?;
        plans.push(observed);
        for batch in rows {
            undecided.push(admit(rule, registry, &batch, true)?);
        }
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
        explain_pgjson,
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

fn admit(
    rule: &CompiledRule,
    registry: &Registry,
    batch: &RecordBatch,
    unknown: bool,
) -> Result<RecordBatch, RuleError> {
    let schema = if unknown {
        Arc::new(Schema::new(
            rule.key_columns
                .iter()
                .map(|name| {
                    rule.head_schema
                        .field_with_name(name)
                        .cloned()
                        .map_err(|error| internal(error.to_string()))
                })
                .collect::<Result<Vec<_>, _>>()?,
        ))
    } else {
        Arc::clone(&rule.head_schema)
    };
    if batch.num_columns() != schema.fields().len() {
        return Err(internal("engine head width differs from compiled contract"));
    }
    for (field, array) in schema.fields().iter().zip(batch.columns()) {
        let retained = rule
            .contracts
            .iter()
            .find(|column| column.spec.name == field.name())
            .ok_or_else(|| internal("head field has no retained typing derivation"))?;
        let expected = pse_schema::arrow::field_for(registry, &retained.spec)
            .map_err(|error| internal(error.to_string()))?;
        if &expected != field.as_ref() {
            return Err(internal(
                "head metadata differs from its retained typing derivation",
            ));
        }
        pse_relations::validate::validate_column(registry, field, array.as_ref()).map_err(
            |errors| RuleError::HeadSchemaMismatch {
                rule: rule.name.clone(),
                column: field.name().clone(),
                expected: format!("{:?}", field.data_type()),
                found: errors
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("; "),
            },
        )?;
    }
    // This shares the already reserved immutable arrays; no buffer allocation is hidden.
    RecordBatch::try_new_with_options(
        schema,
        batch.columns().to_vec(),
        &datafusion::arrow::array::RecordBatchOptions::new().with_row_count(Some(batch.num_rows())),
    )
    .map_err(|error| internal(error.to_string()))
}
