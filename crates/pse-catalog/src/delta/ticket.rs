// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Pre-effect publication tickets. A ticket is minted before any member write; its
//! commit outcome is settled by the operational catalog, never by Delta observation.
use super::attempt::MemberAttempt;
use pse_model::generated::identities::{AttemptId, PublicationId, WorkspaceId};
use pse_relations::generated::runtime::publication_manifests;

/// Native member receipts observed for an exact pre-effect ticket. This establishes
/// written versions only; it grants no scientific assessment or catalog commit.
#[derive(Clone, Debug, PartialEq)]
pub struct MemberRecovery {
    /// Actual written members; absent/provisioned members are excluded.
    pub members: Vec<pse_relations::generated::structures::MemberDescriptor>,
    /// Every requested write has its exact native written receipt.
    pub complete: bool,
    /// At least one exact provisioning receipt exists without a completed data write.
    pub provisioned: bool,
}

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
    /// Inspect native receipts through the existing member-attempt owner. Failed or
    /// incomplete observation remains an error, never proof of absence or permission to replay.
    pub async fn recover_members(
        &self,
        state: &std::sync::Arc<datafusion::execution::session_state::SessionState>,
        cancel: &pse_columnar::CancellationToken,
    ) -> datafusion::common::Result<MemberRecovery> {
        use super::attempt::{MemberState, unresolved};
        self.validate_receipts()?;
        let mut provisioned = false;
        let mut members = Vec::with_capacity(self.attempts.len());
        for attempt in &self.attempts {
            cancel.checkpoint()?;
            let location = url::Url::parse(&attempt.member.table_uri).map_err(unresolved)?;
            let table = super::provider::table_builder(location.clone(), state)?
                .build()
                .map_err(unresolved)?;
            let observed = if table
                .verify_deltatable_existence()
                .await
                .map_err(unresolved)?
            {
                let opened = super::provider::open_native(
                    location,
                    None,
                    crate::cache_service::snapshot::LoadRequirement::Query,
                    state,
                )
                .await?;
                attempt.inspect(&opened.table, state).await?
            } else if attempt.base_version.is_some() {
                return Err(unresolved("member base version is unavailable"));
            } else {
                MemberState::Unpublished
            };
            if matches!(observed, MemberState::Provisioned(_)) {
                provisioned = true;
            }
            if let MemberState::Committed(version) = observed {
                let mut member = attempt.member.clone();
                member.delta_version = super::provider::signed_version(version)?;
                members.push(member);
            }
        }
        Ok(MemberRecovery {
            complete: members.len() == self.attempts.len(),
            members,
            provisioned,
        })
    }
    fn validate_receipts(&self) -> datafusion::common::Result<()> {
        use super::attempt::unresolved;
        if self.version != 2 {
            return Err(unresolved("unsupported publication ticket version"));
        }
        if self.attempts.len() != self.candidate.members.len() {
            return Err(unresolved("ticket inventory differs from member attempts"));
        }
        let mut names = std::collections::BTreeSet::new();
        for attempt in &self.attempts {
            if !names.insert((
                &attempt.member.catalog_name,
                &attempt.member.schema_name,
                &attempt.member.table_name,
            )) {
                return Err(unresolved("duplicate ticket member attempt"));
            }
            if attempt.attempt_id != self.attempt_id()
                || attempt.publication_id != self.publication_id()
                || attempt.workspace_id != self.workspace_id()
                || !self.candidate.members.contains(&attempt.member)
            {
                return Err(unresolved(
                    "ticket member identity differs from its candidate",
                ));
            }
        }
        Ok(())
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
        self.candidate.publication_id
    }
    /// Complete planned member inventory; written members receive actual versions when
    /// the candidate executes.
    pub fn members(&self) -> &[pse_relations::generated::structures::MemberDescriptor] {
        &self.candidate.members
    }
}

#[cfg(test)]
mod member_ticket_unit {
    use super::*;
    #[test]
    fn receipt_recovery_unit_refuses_inexact_or_duplicate_request_before_io() {
        use pse_relations::generated::structures::{MemberDescriptor, MemberDescriptorSelection};
        let member = MemberDescriptor {
            catalog_name: "point_7".into(),
            schema_name: "results".into(),
            table_name: "x".into(),
            relation_id: pse_ids::SemanticId::from_bytes([1; 16]),
            relation_version: 1,
            contract_fingerprint: pse_ids::ContentHash::from_bytes([1; 32]),
            table_uri: "memory:///point7/x".into(),
            delta_version: 0,
            selection: MemberDescriptorSelection::from_full(),
        };
        let header = publication_manifests::Row {
            publication_id: PublicationId::from_bytes([2; 16]),
            workspace_id: WorkspaceId::from_bytes([3; 16]),
            parent_publication_id: None,
            attempt_id: AttemptId::from_bytes([4; 16]),
            kind: pse_model::generated::enums::PublicationKind::Relations,
            inputs: vec![],
            members: vec![member.clone()],
            windows: vec![],
            exported_at: None,
            export_lease_id: None,
            export_expires_at: None,
            maintenance_epoch: None,
            store_fingerprint: None,
        };
        let attempt = MemberAttempt {
            operation_id: header.attempt_id.as_id(),
            workspace_id: header.workspace_id,
            publication_id: header.publication_id,
            attempt_id: header.attempt_id,
            member,
            inputs: vec![],
            dependencies: vec![],
            base_version: None,
        };
        let ticket = PublicationTicket::new(header, vec![attempt.clone()]);
        assert!(ticket.validate_receipts().is_ok());
        let mut wrong = ticket.clone();
        wrong.version = 99;
        assert!(wrong.validate_receipts().is_err());
        let mut wrong = ticket.clone();
        wrong.attempts[0].attempt_id = AttemptId::from_bytes([9; 16]);
        assert!(wrong.validate_receipts().is_err());
        let mut duplicate = ticket.clone();
        duplicate.attempts.push(attempt);
        duplicate
            .candidate
            .members
            .push(duplicate.candidate.members[0].clone());
        assert!(duplicate.validate_receipts().is_err());
        let mut missing = ticket;
        missing.attempts.clear();
        assert!(missing.validate_receipts().is_err());
    }
}
