// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Pre-effect publication tickets. A ticket is minted before any member write; its
//! commit outcome is settled by the operational catalog, never by Delta observation.
use super::attempt::MemberAttempt;
use pse_relations::generated::runtime::publication_manifests;

/// Serializable complete publication request, minted before any native write.
/// Deserializing it grants no admission.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicationTicket {
    version: u32,
    candidate: publication_manifests::Row,
    attempts: Vec<MemberAttempt>,
}
impl PublicationTicket {
    pub(super) fn new(
        mut candidate: publication_manifests::Row,
        attempts: Vec<MemberAttempt>,
    ) -> Self {
        candidate.members.sort_by(|a, b| {
            (&a.catalog_name, &a.schema_name, &a.table_name).cmp(&(
                &b.catalog_name,
                &b.schema_name,
                &b.table_name,
            ))
        });
        Self {
            version: 2,
            candidate,
            attempts,
        }
    }
    /// The workspace the publication is prepared in.
    pub fn workspace_id(&self) -> pse_ids::SemanticId {
        self.candidate.workspace_id
    }
    /// The exact parent the publication was prepared against; never rebased.
    pub fn parent_publication_id(&self) -> Option<pse_ids::SemanticId> {
        self.candidate.parent_publication_id
    }
    /// The planned candidate record; written members carry their actual versions only
    /// once the candidate executes.
    pub fn candidate(&self) -> &publication_manifests::Row {
        &self.candidate
    }
    /// Identity of the original attempt, not a new retry.
    pub fn attempt_id(&self) -> pse_ids::SemanticId {
        self.candidate.attempt_id
    }
    /// Intended immutable publication identity.
    pub fn publication_id(&self) -> pse_ids::SemanticId {
        self.candidate.publication_id
    }
    /// Complete planned member inventory; written members receive actual versions when
    /// the candidate executes.
    pub fn members(&self) -> &[pse_relations::generated::structures::MemberDescriptor] {
        &self.candidate.members
    }
}
