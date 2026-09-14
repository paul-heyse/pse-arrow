// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed, ordered operations produce an unpublished complete candidate (ADR-0053).

mod allocation;
mod apply;
mod owned;
pub(crate) mod proof;
mod stage;

use crate::AuthoringError;
use arrow_array::RecordBatch;
use pse_ids::SemanticId;
use pse_relations::generated::authored;
use std::collections::BTreeMap;

pub use apply::{apply, apply_owned};
pub use owned::{OwnedCandidateSnapshot, OwnedChangeSet};
pub use stage::stage_rows;
/// Exact generated operation row; no handwritten schema mirror.
pub type ChangeOp = authored::change_ops::Row;

/// One schema-valid row, bound to an explicit operation-role port.
#[derive(Clone, Debug)]
pub struct StagedMember {
    /// Exact registered relation.
    pub relation_id: SemanticId,
    /// Always one admitted row; staging is not snapshot membership.
    pub batch: RecordBatch,
}

/// Complete typed operation envelope and its expected base revision.
#[derive(Clone, Debug)]
pub struct ChangeSet {
    /// Generated change-set header.
    pub header: authored::change_sets::Row,
    /// Generated operations in exact ordinal order.
    pub ops: Vec<ChangeOp>,
    /// Every referenced staging port, with no extra hidden members.
    pub staged: BTreeMap<String, StagedMember>,
    rename_proof: Option<proof::RenameProof>,
}

/// Explicit complete base context; an absent relation means an empty relation.
pub trait AuthoredReader {
    /// The immutable base this reader names.
    fn revision_id(&self) -> SemanticId;
    /// Complete included authored/reference relations, all from the same base.
    ///
    /// # Errors
    /// Failure to read or admit any base member.
    fn relations(&self) -> Result<BTreeMap<SemanticId, RecordBatch>, AuthoringError>;
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

impl ChangeSet {
    /// Complete validated source replacements to publish atomically with this rename.
    pub fn document_edits(&self) -> &[crate::document::DocumentEdit] {
        self.rename_proof
            .as_ref()
            .map_or(&[], |proof| proof.edits.as_slice())
    }
}
