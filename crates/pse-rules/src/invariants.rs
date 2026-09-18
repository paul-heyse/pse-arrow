// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One native diagnostic program over exact complete candidate bindings.

mod program;

use crate::{RuleError, errmap::internal};
use datafusion::arrow::array::RecordBatch;
use pse_catalog::session::{CompletedComputation, SnapshotSession};
use pse_ids::{CancellationToken, SemanticId};
use pse_relations::{
    columnar::FieldCheckedBatch,
    generated::{enums::FindingSeverity, runtime::diagnostics_findings},
};
use pse_schema::{Registry, model::RelationKey};
use std::collections::{BTreeMap, BTreeSet};

/// Provider policies use the same registry-to-native lowering as bundle admission.
#[derive(Debug)]
pub struct RegistryRequirementPlanner;
#[async_trait::async_trait]
impl pse_catalog::session::policy::RequirementPlanner for RegistryRequirementPlanner {
    async fn plan(
        &self,
        session: &SnapshotSession,
        requirements: &BTreeSet<SemanticId>,
        cancel: &CancellationToken,
    ) -> Result<datafusion::logical_expr::LogicalPlan, pse_catalog::CatalogError> {
        let result = program::compile(
            &session.input_keys().collect(),
            session,
            session.registry(),
            InvariantScope::Required(requirements),
            cancel,
        )
        .await
        .map_err(|error| pse_catalog::CatalogError::Semantic(std::sync::Arc::new(error)))?;
        result
            .0
            .ok_or_else(|| pse_catalog::CatalogError::Admission {
                path: "provider.requirements".to_owned(),
                reason: "selected requirements produced no native obligation".to_owned(),
            })
    }
}

/// The complete candidate scope whose registry obligations are being discharged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InvariantScope<'a> {
    /// Authored/reference model facts.
    Model,
    /// Case facts together with their actual model context.
    Case,
    /// Registry declarations.
    Registry,
    /// Every explicitly supplied candidate relation.
    Candidate,
    /// Obligations reading any replaced candidate relation, including negative reads.
    /// All other dependencies remain available through exact admitted parent owners.
    Affected(&'a BTreeSet<RelationKey>),
    /// Local sidecar predicates; cross-artifact obligations require the enclosing scope.
    SidecarRelation,
    /// Exactly the invariant declarations selected by composed provider policies.
    Required(&'a BTreeSet<SemanticId>),
}

/// Actual immutable findings and the single computation that produced them.
/// It is a diagnostic result, not a caller-constructible validity certificate.
pub struct InvariantReport {
    completion: Option<CompletedComputation>,
    findings: Vec<FieldCheckedBatch>,
    error_count: usize,
    check_count: usize,
}

impl std::fmt::Debug for InvariantReport {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("InvariantReport")
            .field("check_count", &self.check_count)
            .field("error_count", &self.error_count)
            .field("findings", &self.findings)
            .finish_non_exhaustive()
    }
}

impl InvariantReport {
    /// Actual checked diagnostic batches, retained in Arrow.
    pub fn findings(&self) -> &[FieldCheckedBatch] {
        &self.findings
    }
    /// Number of actual error-severity findings, including undecided predicates.
    pub const fn error_count(&self) -> usize {
        self.error_count
    }
    /// Number of applicable registry obligations in the prepared program.
    pub const fn check_count(&self) -> usize {
        self.check_count
    }
    /// Actual prepared/executed program, source owners and optimizer observation.
    pub const fn completion(&self) -> Option<&CompletedComputation> {
        self.completion.as_ref()
    }
    /// Consume diagnostic batches at an explicit publication or reporting boundary.
    pub fn into_findings(self) -> Vec<RecordBatch> {
        self.findings
            .into_iter()
            .map(FieldCheckedBatch::into_batch)
            .collect()
    }
}

/// Prepare and execute all applicable registry obligations as one native diagnostic plan.
/// Candidate providers expose no unverified uniqueness or foreign-key constraints.
/// True and unknown violation predicates both retain their exact occurrence keys.
///
/// # Errors
/// Incomplete input bindings, construction/analysis failures, cancellation or resource errors.
pub async fn run_invariants(
    candidates: &BTreeMap<RelationKey, RecordBatch>,
    session: &SnapshotSession,
    registry: &Registry,
    scope: InvariantScope<'_>,
    cancel: &CancellationToken,
) -> Result<InvariantReport, RuleError> {
    session.validate_bindings(candidates)?;
    let (plan, check_count) = program::compile(
        &candidates.keys().copied().collect(),
        session,
        registry,
        scope,
        cancel,
    )
    .await?;
    let Some(plan) = plan else {
        return Ok(InvariantReport {
            completion: None,
            findings: Vec::new(),
            error_count: 0,
            check_count,
        });
    };
    let completion = session
        .prepare_rule_plan(plan, cancel)?
        .execute(cancel)
        .await?;
    let spec = diagnostics_findings::spec(registry)?;
    let mut findings = Vec::new();
    let mut error_count = 0_usize;
    for batch in completion.batches() {
        if batch.num_rows() == 0 {
            continue;
        }
        let owner = FieldCheckedBatch::admit_owned(registry, spec, batch.clone())?;
        let view = diagnostics_findings::View::from_checked(&owner)?;
        // This is an observation of the completed typed diagnostic output, not a second
        // interpretation of an invariant. Its predicates ran in the native program.
        let column = view.severity_column();
        for value in column.iter().flatten() {
            if value == FindingSeverity::Error.as_str() {
                error_count = error_count
                    .checked_add(1)
                    .ok_or_else(|| internal("diagnostic error count overflow"))?;
            }
        }
        findings.push(owner);
    }
    Ok(InvariantReport {
        completion: Some(completion),
        findings,
        error_count,
        check_count,
    })
}
