// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed, ordered operations produce an unpublished complete candidate (ADR-0053).

mod allocation;
mod apply;
pub(crate) mod base;
pub(crate) mod exact;
mod owned;
pub(crate) mod plans;
pub(crate) mod stage;

use crate::AuthoringError;
use arrow_array::RecordBatch;
use pse_ids::SemanticId;
use pse_relations::generated::authored;
use std::collections::BTreeMap;

pub use apply::apply_owned;
pub use owned::{OwnedCandidateSnapshot, OwnedChangeSet};
pub use stage::stage_batches;
/// Exact generated operation row; no handwritten schema mirror.
pub type ChangeOp = authored::change_ops::Row;

/// A columnar pre/post inventory bound to an explicit operation-role port.
#[derive(Clone, Debug)]
pub struct StagedMember {
    /// Exact registered relation.
    pub relation_id: SemanticId,
    /// Operation references select exact row ordinals; staging is not snapshot membership.
    pub batch: RecordBatch,
}

/// Complete typed operation envelope and its expected base revision.
#[derive(Clone, Debug)]
pub struct ChangeSet {
    data: std::sync::Arc<ChangeEnvelope>,
    source: Option<std::sync::Arc<SourceConstruction>>,
    completed: std::sync::Arc<plans::Completions>,
    // Candidate construction clones the envelope independently of its staging
    // handle. Those clones must keep every actual construction allocation alive.
    leases: Vec<std::sync::Arc<pse_ids::ReservationLease>>,
}

/// Immutable operation data. Only checked authoring methods can change the envelope.
#[derive(Clone, Debug)]
pub struct ChangeEnvelope {
    /// Generated change-set header.
    pub header: authored::change_sets::Row,
    /// Generated operations in exact ordinal order.
    pub ops: Vec<ChangeOp>,
    /// Every referenced staging port, with no extra hidden members.
    pub staged: BTreeMap<String, StagedMember>,
}

/// Explicit complete base context; an absent relation means an empty relation.
pub trait AuthoredReader: Send + Sync {
    /// The immutable base this reader names.
    fn revision_id(&self) -> SemanticId;
    /// Complete included authored/reference relations, all from the same base.
    ///
    /// # Errors
    /// Failure to read or admit any base member.
    fn relations(&self) -> Result<BTreeMap<SemanticId, RecordBatch>, AuthoringError>;
    /// Retained field-checked owners, when this reader already owns their admission.
    /// These are the relation authority when present; callers need not detach/re-admit them.
    fn checked_relations(&self) -> Option<&crate::document::Batches> {
        None
    }
    /// Complete original authoring bytes from this exact base, required for full rename.
    /// # Errors
    /// A reader without source artifacts explicitly refuses full rename.
    fn source_documents(&self) -> Result<BTreeMap<SemanticId, String>, AuthoringError> {
        Err(contract(
            "this base reader does not expose exact original document bytes",
        ))
    }
}

/// Applied rows remain private until P2 validates and the compiler publishes them.
#[derive(Clone, Debug)]
pub struct CandidateSnapshot {
    /// Complete candidate relation inventory.
    pub relations: BTreeMap<SemanticId, RecordBatch>,
    /// Exact source base identity for optimistic publication.
    pub base_revision_id: SemanticId,
    /// Admitted operation envelope, persisted as a sidecar on successful commit.
    pub changes: ChangeSet,
}

pub(crate) fn contract(reason: &str) -> AuthoringError {
    AuthoringError::Contract {
        at: None,
        reason: reason.to_owned(),
    }
}

#[derive(Debug)]
pub(crate) struct SourceConstruction {
    pub(crate) documents: crate::document::OwnedDocumentSet,
    pub(crate) candidate: crate::document::Batches,
    pub(crate) bindings: crate::document::binding::OwnedSourceBindings,
    pub(crate) edits: Vec<crate::document::DocumentEdit>,
    pub(crate) renamed: std::collections::BTreeSet<SemanticId>,
    pub(crate) completed: plans::Completions,
}
impl std::ops::Deref for ChangeSet {
    type Target = ChangeEnvelope;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl ChangeSet {
    /// Admit an external current-format operation envelope. Source construction
    /// cannot be supplied by a caller; before-images are checked during application.
    /// # Errors
    /// Invalid generated fields, ordinals, authorities or incomplete staged inventory.
    pub fn from_staged(
        header: authored::change_sets::Row,
        ops: Vec<ChangeOp>,
        staged: BTreeMap<String, StagedMember>,
        registry: &pse_schema::Registry,
    ) -> Result<Self, AuthoringError> {
        let changes = Self::from_parts(header, ops, staged);
        apply::validate_envelope(&changes, registry)?;
        Ok(changes)
    }
    pub(crate) fn from_parts(
        header: authored::change_sets::Row,
        ops: Vec<ChangeOp>,
        staged: BTreeMap<String, StagedMember>,
    ) -> Self {
        Self {
            data: std::sync::Arc::new(ChangeEnvelope {
                header,
                ops,
                staged,
            }),
            source: None,
            completed: std::sync::Arc::new(Vec::new()),
            leases: Vec::new(),
        }
    }
    pub(crate) fn data_mut(&mut self) -> &mut ChangeEnvelope {
        self.source = None;
        self.completed = std::sync::Arc::new(Vec::new());
        std::sync::Arc::make_mut(&mut self.data)
    }
    pub(crate) fn retain_completions(&mut self, mut completed: plans::Completions) {
        let mut seen = self
            .source_completions()
            .map(std::sync::Arc::as_ptr)
            .collect::<std::collections::BTreeSet<_>>();
        completed.retain(|value| seen.insert(std::sync::Arc::as_ptr(value)));
        self.completed = std::sync::Arc::new(completed);
    }
    /// Actual native computation pieces retained by this immutable change construction.
    /// These establish performed work; P2 and publication obligations remain separate.
    pub fn completions(
        &self,
    ) -> impl Iterator<Item = &std::sync::Arc<pse_catalog::session::CompletedComputation>> {
        self.completed.iter().chain(self.source_completions())
    }
    pub(crate) fn attach_source(&mut self, mut source: SourceConstruction) {
        let mut seen = self
            .completed
            .iter()
            .chain(source.bindings.completions())
            .map(std::sync::Arc::as_ptr)
            .collect::<std::collections::BTreeSet<_>>();
        source
            .completed
            .retain(|value| seen.insert(std::sync::Arc::as_ptr(value)));
        self.source = Some(std::sync::Arc::new(source));
    }
    pub(crate) fn source(&self) -> Option<&SourceConstruction> {
        self.source.as_deref()
    }
    /// Completed native source construction, including its binding computations.
    /// Empty for externally supplied envelopes until source admission succeeds.
    pub fn source_completions(
        &self,
    ) -> impl Iterator<Item = &std::sync::Arc<pse_catalog::session::CompletedComputation>> {
        self.source
            .iter()
            .flat_map(|source| source.completed.iter().chain(source.bindings.completions()))
    }
    /// Actual parsed source owner retained by local source construction.
    pub fn documents(&self) -> Option<&crate::document::OwnedDocumentSet> {
        self.source.as_ref().map(|source| &source.documents)
    }
    /// Complete source replacements produced by the retained source constructor.
    pub fn document_edits(&self) -> &[crate::document::DocumentEdit] {
        self.source
            .as_ref()
            .map_or(&[], |source| source.edits.as_slice())
    }
}
