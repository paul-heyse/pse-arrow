// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Catalog publication admission through actual candidate rule execution.
use crate::RuleError;
use crate::invariants::{InvariantScope, run_invariants};
use datafusion::arrow::array::RecordBatch;
use datafusion::execution::runtime_env::RuntimeEnv;
use pse_catalog::session::{EngineProfile, build_candidate_session_with_cancel};
use pse_catalog::store::membership::SemanticValidator;
use pse_catalog::{CatalogError, ExecutionSettings, ThreadBudget};
use pse_ids::{CancellationToken, MemoryReserver};
use pse_schema::{Registry, model::RelationKey};
use std::{collections::BTreeMap, sync::Arc};

/// Required semantic validator bound to one actual immutable registry and runtime.
#[derive(Debug)]
pub struct InvariantValidator {
    registry: Arc<Registry>,
    runtime: Arc<RuntimeEnv>,
    reserver: Arc<dyn MemoryReserver>,
    settings: ExecutionSettings,
    budget: ThreadBudget,
    profile: EngineProfile,
}
impl InvariantValidator {
    /// Capture admitted execution configuration; candidate rows remain per-call inputs.
    pub fn new(
        registry: Arc<Registry>,
        runtime: Arc<RuntimeEnv>,
        reserver: Arc<dyn MemoryReserver>,
        settings: ExecutionSettings,
        budget: ThreadBudget,
        profile: EngineProfile,
    ) -> Self {
        Self {
            registry,
            runtime,
            reserver,
            settings,
            budget,
            profile,
        }
    }
}
impl SemanticValidator for InvariantValidator {
    fn validate<'a>(
        &'a self,
        reg: &'a Registry,
        rows: &'a BTreeMap<RelationKey, RecordBatch>,
        cancel: &'a CancellationToken,
    ) -> pse_catalog::BoxFut<'a, Result<(), CatalogError>> {
        self.validate_scope(reg, rows, InvariantScope::Candidate, cancel)
    }

    fn validate_sidecar<'a>(
        &'a self,
        reg: &'a Registry,
        rows: &'a BTreeMap<RelationKey, RecordBatch>,
        cancel: &'a CancellationToken,
    ) -> pse_catalog::BoxFut<'a, Result<(), CatalogError>> {
        self.validate_scope(reg, rows, InvariantScope::SidecarRelation, cancel)
    }
}
impl InvariantValidator {
    fn validate_scope<'a>(
        &'a self,
        reg: &'a Registry,
        rows: &'a BTreeMap<RelationKey, RecordBatch>,
        scope: InvariantScope,
        cancel: &'a CancellationToken,
    ) -> pse_catalog::BoxFut<'a, Result<(), CatalogError>> {
        Box::pin(async move {
            cancel.checkpoint()?;
            // This validator belongs to a concrete registry object; a caller cannot replace
            // its contracts by offering a matching fingerprint on another declaration set.
            if !std::ptr::eq(reg, self.registry.as_ref()) {
                return Err(CatalogError::Membership {
                    reason: "invariant validator belongs to a different registry instance".into(),
                });
            }
            let session = build_candidate_session_with_cancel(
                rows.clone(),
                Arc::clone(&self.registry),
                Arc::clone(&self.runtime),
                Arc::clone(&self.reserver),
                self.settings.clone(),
                self.budget,
                self.profile.clone(),
                cancel,
            )?;
            let report = run_invariants(rows, &session, reg, scope, None, cancel)
                .await
                .map_err(|error| CatalogError::Semantic(Arc::new(error)))?;
            if report.error_count != 0 {
                return Err(CatalogError::Semantic(Arc::new(
                    RuleError::InvariantViolations {
                        count: report.error_count,
                        findings: report.findings,
                    },
                )));
            }
            Ok(())
        })
    }
}
