// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Catalog publication admission through actual candidate rule execution.
use crate::RuleError;
use crate::invariants::{InvariantScope, run_invariants};
use datafusion::arrow::array::RecordBatch;
use pse_catalog::CatalogError;
use pse_catalog::session::SnapshotSession;
use pse_catalog::store::membership::SemanticValidator;
use pse_ids::CancellationToken;
use pse_schema::{Registry, model::RelationKey};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

/// Required semantic validator bound to one actual immutable registry and runtime.
#[derive(Debug)]
pub struct InvariantValidator {
    registry: Arc<Registry>,
}
impl InvariantValidator {
    /// Bind the actual registry; every execution receives its operation's native scope.
    pub fn new(registry: Arc<Registry>) -> Self {
        Self { registry }
    }
}
impl SemanticValidator for InvariantValidator {
    fn validate_checked<'a>(
        &'a self,
        reg: &'a Registry,
        rows: &'a BTreeMap<RelationKey, pse_relations::columnar::FieldCheckedBatch>,
        session: &'a SnapshotSession,
        cancel: &'a CancellationToken,
    ) -> pse_catalog::BoxFut<'a, Result<(), CatalogError>> {
        Box::pin(async move {
            let changed = rows.keys().copied().collect();
            self.validate_affected(reg, rows, &changed, session, cancel)
                .await
        })
    }
    fn validate_affected<'a>(
        &'a self,
        reg: &'a Registry,
        rows: &'a BTreeMap<RelationKey, pse_relations::columnar::FieldCheckedBatch>,
        changed: &'a BTreeSet<RelationKey>,
        session: &'a SnapshotSession,
        cancel: &'a CancellationToken,
    ) -> pse_catalog::BoxFut<'a, Result<(), CatalogError>> {
        Box::pin(async move {
            if !std::ptr::eq(reg, self.registry.as_ref()) {
                return Err(CatalogError::Membership {
                    reason: "different actual registry instance".into(),
                });
            }
            let session = session
                .select_inputs(&BTreeSet::new(), cancel)?
                .with_checked_workspace(rows.clone(), cancel)?;
            let raw = rows
                .iter()
                .map(|(key, batch)| (*key, batch.batch().clone()))
                .collect();
            let report = run_invariants(
                &raw,
                &session,
                reg,
                InvariantScope::Affected(changed),
                None,
                cancel,
            )
            .await
            .map_err(|error| CatalogError::Semantic(Arc::new(error)))?;
            if report.error_count() != 0 {
                return Err(CatalogError::Semantic(Arc::new(
                    RuleError::InvariantViolations {
                        count: report.error_count(),
                        findings: report.into_findings(),
                    },
                )));
            }
            Ok(())
        })
    }
    fn validate<'a>(
        &'a self,
        reg: &'a Registry,
        rows: &'a BTreeMap<RelationKey, RecordBatch>,
        session: &'a SnapshotSession,
        cancel: &'a CancellationToken,
    ) -> pse_catalog::BoxFut<'a, Result<(), CatalogError>> {
        self.validate_scope(reg, rows, InvariantScope::Candidate, session, cancel)
    }

    fn validate_sidecar<'a>(
        &'a self,
        reg: &'a Registry,
        rows: &'a BTreeMap<RelationKey, RecordBatch>,
        session: &'a SnapshotSession,
        cancel: &'a CancellationToken,
    ) -> pse_catalog::BoxFut<'a, Result<(), CatalogError>> {
        self.validate_scope(reg, rows, InvariantScope::SidecarRelation, session, cancel)
    }
}
impl InvariantValidator {
    fn validate_scope<'a>(
        &'a self,
        reg: &'a Registry,
        rows: &'a BTreeMap<RelationKey, RecordBatch>,
        scope: InvariantScope<'a>,
        session: &'a SnapshotSession,
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
            let checked = rows
                .iter()
                .map(|(key, batch)| {
                    let spec =
                        reg.relation_by_key(*key)
                            .ok_or_else(|| CatalogError::Membership {
                                reason: "validation relation is undeclared".into(),
                            })?;
                    let batch = pse_relations::columnar::FieldCheckedBatch::admit_external(
                        reg,
                        spec,
                        batch,
                        session.reserver(),
                        cancel,
                    )?;
                    Ok((*key, batch))
                })
                .collect::<Result<BTreeMap<_, _>, CatalogError>>()?;
            let session = session
                .select_inputs(&BTreeSet::new(), cancel)?
                .with_checked_workspace(checked, cancel)?;
            let report = run_invariants(rows, &session, reg, scope, None, cancel)
                .await
                .map_err(|error| CatalogError::Semantic(Arc::new(error)))?;
            if report.error_count() != 0 {
                return Err(CatalogError::Semantic(Arc::new(
                    RuleError::InvariantViolations {
                        count: report.error_count(),
                        findings: report.into_findings(),
                    },
                )));
            }
            Ok(())
        })
    }
}
