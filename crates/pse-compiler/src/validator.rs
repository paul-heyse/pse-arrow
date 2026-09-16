// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One actual producer registration for local execution and external admission.
pub(crate) mod physical;
mod producer;
mod source_producer;
mod sources;
mod traversal;

use crate::{CompilerError, passes::registry::PassRegistry};
use pse_catalog::{
    Catalog, CatalogError,
    computation::StageProducer,
    store::membership::{AdmissionContext, SemanticValidator},
};
use pse_ids::{CancellationToken, SemanticId};
use pse_relations::RecordBatch;
use pse_schema::{Registry, model::RelationKey};
use std::{collections::BTreeMap, sync::Arc};

pub(crate) use producer::InvocationArguments;
pub(crate) use source_producer::SourceArguments;

/// Registered invariant execution and the single inventory of actual producers.
#[derive(Debug)]
pub struct CompilerValidator {
    invariants: Arc<dyn SemanticValidator>,
    passes: PassRegistry,
    producers: BTreeMap<SemanticId, Arc<dyn StageProducer>>,
    sources: Option<Arc<dyn pse_catalog::source_production::SourceProducer>>,
    sessions: Option<Arc<pse_catalog::session::SessionFactory>>,
}
impl CompilerValidator {
    /// Bind every available production implementation to its exact declaration once.
    /// # Errors
    /// A production implementation differs from its authoritative pass specification.
    pub fn new(
        invariants: Arc<dyn SemanticValidator>,
        registry: &Registry,
    ) -> Result<Self, CompilerError> {
        Ok(Self {
            invariants,
            passes: PassRegistry::production(registry)?,
            producers: BTreeMap::new(),
            sources: None,
            sessions: None,
        })
    }
    /// Seal the actual shared engine into every selected producer implementation.
    #[must_use]
    pub fn with_sessions(mut self, sessions: Arc<pse_catalog::session::SessionFactory>) -> Self {
        self.sources = Some(Arc::new(source_producer::RegisteredSources));
        self.producers = self
            .passes
            .implementations()
            .map(|pass| {
                let implementation: Arc<dyn StageProducer> = Arc::new(producer::RegisteredPass {
                    pass: Arc::clone(pass),
                    sessions: Arc::clone(&sessions),
                });
                (pass.spec().id, implementation)
            })
            .collect();
        self.sessions = Some(sessions);
        self
    }
    /// Register a custom implementation before sealing its runtime and opening a catalog.
    /// # Errors
    /// A runtime has already been sealed, or the implementation is duplicate/undeclared.
    pub fn register(
        &mut self,
        pass: Arc<dyn crate::Pass>,
        registry: &Registry,
    ) -> Result<(), CompilerError> {
        if self.sessions.is_some() {
            return Err(crate::passes::dag::invalid(
                "register producers before sealing the engine",
            ));
        }
        self.passes.register(pass, registry)
    }
}
impl SemanticValidator for CompilerValidator {
    fn validate_affected<'a>(
        &'a self,
        registry: &'a Registry,
        rows: &'a BTreeMap<RelationKey, pse_relations::columnar::FieldCheckedBatch>,
        changed: &'a std::collections::BTreeSet<RelationKey>,
        session: &'a pse_catalog::session::SnapshotSession,
        cancel: &'a CancellationToken,
    ) -> pse_catalog::BoxFut<'a, Result<(), CatalogError>> {
        self.invariants
            .validate_affected(registry, rows, changed, session, cancel)
    }
    fn source_producer(&self) -> Option<Arc<dyn pse_catalog::source_production::SourceProducer>> {
        self.sources.clone()
    }
    fn validate_checked<'a>(
        &'a self,
        registry: &'a Registry,
        rows: &'a BTreeMap<RelationKey, pse_relations::columnar::FieldCheckedBatch>,
        session: &'a pse_catalog::session::SnapshotSession,
        cancel: &'a CancellationToken,
    ) -> pse_catalog::BoxFut<'a, Result<(), CatalogError>> {
        self.invariants
            .validate_checked(registry, rows, session, cancel)
    }
    fn stage_producer(&self, pass: SemanticId) -> Option<Arc<dyn StageProducer>> {
        self.producers.get(&pass).cloned()
    }
    fn validate<'a>(
        &'a self,
        registry: &'a Registry,
        rows: &'a BTreeMap<RelationKey, RecordBatch>,
        session: &'a pse_catalog::session::SnapshotSession,
        cancel: &'a CancellationToken,
    ) -> pse_catalog::BoxFut<'a, Result<(), CatalogError>> {
        self.invariants.validate(registry, rows, session, cancel)
    }
    fn validate_sidecar<'a>(
        &'a self,
        registry: &'a Registry,
        rows: &'a BTreeMap<RelationKey, RecordBatch>,
        session: &'a pse_catalog::session::SnapshotSession,
        cancel: &'a CancellationToken,
    ) -> pse_catalog::BoxFut<'a, Result<(), CatalogError>> {
        self.invariants
            .validate_sidecar(registry, rows, session, cancel)
    }
    fn validate_snapshot_sources<'a>(
        &'a self,
        catalog: &'a Catalog,
        kind: pse_ids::SnapshotKind,
        context: &'a AdmissionContext,
        candidates: &'a BTreeMap<RelationKey, RecordBatch>,
        cancel: &'a CancellationToken,
    ) -> pse_catalog::BoxFut<'a, Result<(), CatalogError>> {
        Box::pin(async move {
            let sessions = self
                .sessions
                .as_ref()
                .ok_or_else(|| CatalogError::Admission {
                    path: "source admission".to_owned(),
                    reason: "shared engine has not been sealed".to_owned(),
                })?;
            sources::validate(catalog, kind, context, candidates, sessions, cancel)
                .await
                .map_err(|error| CatalogError::Semantic(Arc::new(error)))
        })
    }
}
