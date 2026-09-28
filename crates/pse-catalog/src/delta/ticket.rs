// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Pre-effect publication tickets. A ticket is minted before any member write; its
//! commit outcome is settled by the operational catalog, never by Delta observation.
use super::attempt::MemberAttempt;
use pse_model::generated::identities::{AttemptId, PublicationId, WorkspaceId};
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
    pub const fn workspace_id(&self) -> WorkspaceId {
        self.candidate.workspace_id
    }
    /// The exact parent the publication was prepared against; never rebased.
    pub const fn parent_publication_id(&self) -> Option<PublicationId> {
        self.candidate.parent_publication_id
    }
    /// The planned candidate record; written members carry their actual versions only
    /// once the candidate executes.
    pub fn candidate(&self) -> &publication_manifests::Row {
        &self.candidate
    }
    /// Identity of the original attempt, not a new retry.
    ///
    /// The attempt and the publication it intends are distinct identities, so one does
    /// not stand in for the other:
    ///
    /// ```
    /// use pse_catalog::delta::ticket::PublicationTicket;
    /// use pse_model::generated::identities::{AttemptId, PublicationId};
    ///
    /// fn names(ticket: &PublicationTicket) -> (AttemptId, PublicationId) {
    ///     (ticket.attempt_id(), ticket.publication_id())
    /// }
    /// ```
    ///
    /// ```compile_fail,E0308
    /// use pse_catalog::delta::ticket::PublicationTicket;
    /// use pse_model::generated::identities::PublicationId;
    ///
    /// fn intended(ticket: &PublicationTicket) -> PublicationId {
    ///     // An attempt is not the publication it intends.
    ///     ticket.attempt_id()
    /// }
    /// ```
    pub const fn attempt_id(&self) -> AttemptId {
        self.candidate.attempt_id
    }
    /// Intended immutable publication identity.
    pub const fn publication_id(&self) -> PublicationId {
        super::manifest::publication_of(&self.candidate)
    }
    /// Complete planned member inventory; written members receive actual versions when
    /// the candidate executes.
    pub fn members(&self) -> &[pse_relations::generated::structures::MemberDescriptor] {
        &self.candidate.members
    }
}
