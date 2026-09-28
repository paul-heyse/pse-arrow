// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The publication catalog (ADR-0114 Outcomes 6, 7 and 20; Plan 22 X9, X10;
//! architecture §9.5).
//!
//! **Intents.** A publication is registered as an intent — its identity, its durable
//! attempt and the prefix its members are written under — before the first member write.
//! Unpublished members are therefore reclaimable through the catalog, and a settlement can
//! prove that nothing was committed before any effect.
//!
//! **Visibility.** One transaction commits an intent: it checks that every retained member
//! and input is an exact version a live publication selects (locking those publications
//! against retirement), compares the workspace head with the expected parent, inserts the
//! publication with its members and change windows, and advances the head. A lost race is
//! [`OperationsError::PublicationConflict`]; the publisher re-prepares and never rebases.
//!
//! **Readers** take a lease row in one short transaction, which also returns the complete
//! record and the workspace's maintenance epoch; they renew and release it in short
//! transactions and never hold a session while reading Delta files (finding T02).
//!
//! **Retention** is computed here (finding T16). Every maintenance transaction runs under
//! a transaction-scoped advisory lock on the workspace and advances its maintenance epoch
//! before any effect. Retirement is two-phase: mark expiring, wait for leases, remove the
//! tables no other undeleted publication selects, mark deleted.
//!
//! Identities of leases and settlements are minted by the caller (ADR-0114 Outcome 13).

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use pse_model::generated::enums::{MemberSelectionKind, RetentionPhase};
use pse_operations_queries::client::Params as _;
use pse_operations_queries::queries::catalog as statements;
use tokio_postgres::types::ToSql;

use crate::attempts::{AttemptId, Tx, micros};
use crate::bulk::{Cells, copy_in};
use crate::error::{Classify, OperationsError, Target};
use crate::generated::copy;
use crate::lifecycle::AttemptState;
use crate::store::Store;
pub use pse_model::generated::enums::{
    PublicationKind, PublicationMemberRole, RetentionReason, SettlementOutcome,
};
pub use pse_model::generated::identities::{
    PublicationId, ReaderLeaseId, SettlementId, WorkspaceId,
};
pub use pse_model::generated::runtime::operational_publication_intents::RuntimeOperationalPublicationIntentsRow;
pub use pse_model::generated::runtime::operational_publication_members::RuntimeOperationalPublicationMembersRow;
pub use pse_model::generated::runtime::operational_publications::RuntimeOperationalPublicationsRow;
pub use pse_model::generated::runtime::operational_reader_leases::RuntimeOperationalReaderLeasesRow;
pub use pse_model::generated::runtime::operational_workspaces::RuntimeOperationalWorkspacesRow;
pub use pse_model::generated::structures::{
    MemberDescriptor, MemberDescriptorSelection, MemberDescriptorSelectionRevision,
    MemberDescriptorSelectionSelected, VersionWindow,
};

/// A workspace to register: one publication history with one head.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewWorkspace {
    /// The workspace identity, minted by the runtime.
    pub workspace_id: WorkspaceId,
    /// A unique human name.
    pub name: String,
    /// The root of its Delta member tables; unique.
    pub root_uri: String,
}

/// A publication intent, registered before its first member write (Plan 22 X9).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewIntent {
    /// The publication identity, minted by the runtime.
    pub publication_id: PublicationId,
    /// The workspace it will be published in.
    pub workspace_id: WorkspaceId,
    /// The durable attempt being published.
    pub attempt_id: AttemptId,
    /// The prefix every member it writes lives under; unique.
    pub member_prefix: String,
}

/// A publication to make visible: the complete request.
#[derive(Clone, Debug, PartialEq)]
pub struct PublicationCommit {
    /// The registered intent's publication identity.
    pub publication_id: PublicationId,
    /// The workspace.
    pub workspace_id: WorkspaceId,
    /// The finished attempt being published.
    pub attempt_id: AttemptId,
    /// The head this publication was prepared against; `None` for the first publication.
    pub expected_parent: Option<PublicationId>,
    /// The artifact profile published.
    pub kind: PublicationKind,
    /// The members: every one written under the intent's prefix, or an exact version a
    /// live publication selects (retained).
    pub members: Vec<MemberDescriptor>,
    /// The exact inputs read; each an exact version a live publication selects.
    pub inputs: Vec<MemberDescriptor>,
    /// The change windows read.
    pub windows: Vec<VersionWindow>,
}

/// A successful commit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Committed {
    /// This call made the publication visible and advanced the head.
    Advanced {
        /// The publication.
        publication_id: PublicationId,
    },
    /// The attempt was already published with this exact request (publication is
    /// idempotent per attempt).
    AlreadyCommitted {
        /// The existing publication.
        publication_id: PublicationId,
    },
}

/// A committed publication as the catalog holds it.
#[derive(Clone, Debug, PartialEq)]
pub struct PublicationRecord {
    /// The publication row.
    pub publication: RuntimeOperationalPublicationsRow,
    /// The members, ordered by qualified name.
    pub members: Vec<MemberDescriptor>,
    /// The inputs, ordered by qualified name.
    pub inputs: Vec<MemberDescriptor>,
    /// The change windows, ordered by table and first version.
    pub windows: Vec<VersionWindow>,
}

/// What a settlement inquires about: the request of an uncertain commit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SettleRequest {
    /// The settlement identity, minted by the runtime.
    pub settlement_id: SettlementId,
    /// The attempt.
    pub attempt_id: AttemptId,
    /// The publication the request intended.
    pub publication_id: PublicationId,
    /// Its workspace.
    pub workspace_id: WorkspaceId,
    /// The parent it was prepared against.
    pub expected_parent: Option<PublicationId>,
}

/// The outcome of settling an uncertain commit acknowledgement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Settlement {
    /// The requested publication is visible.
    Committed {
        /// The publication.
        publication_id: PublicationId,
    },
    /// Nothing was committed for the request, and nothing is in flight; the same
    /// request may commit on a retry (the head is still its parent, or its intent was
    /// never registered).
    ProvedNoncommit,
    /// The request can never commit as prepared: the head moved, or the attempt was
    /// published as another publication.
    Conflict {
        /// Why.
        reason: String,
        /// The workspace head at settlement.
        head: Option<PublicationId>,
    },
}

/// What a reader wants to read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadTarget {
    /// An exact publication.
    Publication(PublicationId),
    /// The current head of a workspace, resolved and recorded in the lease.
    Head(WorkspaceId),
}

/// A granted reader lease with the complete record it protects.
#[derive(Clone, Debug, PartialEq)]
pub struct ReaderLease {
    /// The lease as stored; `publication_id` is the resolved publication.
    pub lease: RuntimeOperationalReaderLeasesRow,
    /// The publication to read.
    pub record: PublicationRecord,
    /// The workspace's maintenance epoch when the lease was granted: the reader's cache
    /// scope (Plan 22 X10).
    pub maintenance_epoch: i64,
}

/// A range of table versions retention must keep, and why.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProtectedRange {
    /// The table, or for [`RetentionReason::Attempt`] the member prefix of a live intent
    /// (every table under it keeps its whole history).
    pub table_uri: String,
    /// First protected version, inclusive.
    pub from_version: i64,
    /// Last protected version, inclusive.
    pub through_version: i64,
    /// Why the versions stay reachable.
    pub reason: RetentionReason,
}

/// What collection maintains in a workspace, fixed under its lock after its epoch
/// advanced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollectPlan {
    /// The epoch the workspace advanced to before any effect.
    pub maintenance_epoch: i64,
    /// Every maintained table with the ranges it must keep (possibly none).
    pub tables: BTreeMap<String, Vec<ProtectedRange>>,
}

/// An exact member version's selection, flattened into a catalog row.
/// A member's selection flattened into the store's columns: its kind and, for a revision,
/// the column and revision it selects.
pub(crate) fn flattened_selection(
    member: &MemberDescriptor,
) -> Result<(MemberSelectionKind, Option<String>, Option<pse_ids::SemanticId>), OperationsError> {
    Ok(match member
        .selection
        .selected()
        .map_err(|error| OperationsError::InvalidRequest {
            reason: format!(
                "member {}.{}.{}: {error}",
                member.catalog_name, member.schema_name, member.table_name
            ),
        })? {
        MemberDescriptorSelectionSelected::Full => (MemberSelectionKind::Full, None, None),
        MemberDescriptorSelectionSelected::Revision(revision) => (
            MemberSelectionKind::Revision,
            Some(revision.column.clone()),
            Some(revision.revision_id),
        ),
    })
}

/// The registry selection a stored member's flattened columns describe, or `None` when
/// they are inconsistent.
pub(crate) fn stored_selection(
    kind: MemberSelectionKind,
    column: Option<&String>,
    revision_id: Option<pse_ids::SemanticId>,
) -> Option<MemberDescriptorSelection> {
    match (kind, column, revision_id) {
        (MemberSelectionKind::Full, None, None) => Some(MemberDescriptorSelection::from_full()),
        (MemberSelectionKind::Revision, Some(column), Some(revision_id)) => Some(
            MemberDescriptorSelection::from_revision(MemberDescriptorSelectionRevision {
                column: column.clone(),
                revision_id,
            }),
        ),
        _ => None,
    }
}

fn member_row(
    publication_id: PublicationId,
    role: PublicationMemberRole,
    member: &MemberDescriptor,
) -> Result<RuntimeOperationalPublicationMembersRow, OperationsError> {
    let (selection_kind, revision_column, revision_id) = flattened_selection(member)?;
    Ok(RuntimeOperationalPublicationMembersRow {
        publication_id,
        role,
        catalog_name: member.catalog_name.clone(),
        schema_name: member.schema_name.clone(),
        table_name: member.table_name.clone(),
        relation_id: member.relation_id,
        relation_version: member.relation_version,
        contract_fingerprint: member.contract_fingerprint,
        table_uri: member.table_uri.clone(),
        delta_version: member.delta_version,
        selection_kind,
        revision_column,
        revision_id,
    })
}

/// The registry `MemberDescriptor` a stored member row flattens.
fn descriptor(
    row: &RuntimeOperationalPublicationMembersRow,
) -> Result<MemberDescriptor, OperationsError> {
    let selection = stored_selection(
        row.selection_kind,
        row.revision_column.as_ref(),
        row.revision_id,
    )
    .ok_or_else(|| OperationsError::CorruptValue {
        column: "selection_kind",
        detail: format!(
            "member {}.{}.{} of {} has an inconsistent selection",
            row.catalog_name, row.schema_name, row.table_name, row.publication_id
        ),
    })?;
    Ok(MemberDescriptor {
        catalog_name: row.catalog_name.clone(),
        schema_name: row.schema_name.clone(),
        table_name: row.table_name.clone(),
        relation_id: row.relation_id,
        relation_version: row.relation_version,
        contract_fingerprint: row.contract_fingerprint,
        table_uri: row.table_uri.clone(),
        delta_version: row.delta_version,
        selection,
    })
}

/// Members in the catalog's order: by qualified name.
fn ordered(members: &[MemberDescriptor]) -> Vec<MemberDescriptor> {
    let mut members = members.to_vec();
    members.sort_by(|left, right| {
        (&left.catalog_name, &left.schema_name, &left.table_name).cmp(&(
            &right.catalog_name,
            &right.schema_name,
            &right.table_name,
        ))
    });
    members
}

/// Windows in the catalog's order: by table and first version.
fn ordered_windows(windows: &[VersionWindow]) -> Vec<VersionWindow> {
    let mut windows = windows.to_vec();
    windows.sort_by(|left, right| {
        (&left.table_uri, left.from_version).cmp(&(&right.table_uri, right.from_version))
    });
    windows
}

/// The complete record of a publication row.
async fn record(
    tx: &Tx<'_>,
    target: &Target,
    publication: RuntimeOperationalPublicationsRow,
) -> Result<PublicationRecord, OperationsError> {
    let rows = statements::members()
        .bind(tx, &publication.publication_id)
        .all()
        .await
        .classify(target)?;
    let mut members = Vec::new();
    let mut inputs = Vec::new();
    for row in &rows {
        match row.role {
            PublicationMemberRole::Output => members.push(descriptor(row)?),
            PublicationMemberRole::Input => inputs.push(descriptor(row)?),
        }
    }
    let windows = statements::windows()
        .bind(tx, &publication.publication_id)
        .all()
        .await
        .classify(target)?
        .into_iter()
        .map(|window| VersionWindow {
            table_uri: window.table_uri,
            from_version: window.from_version,
            through_version: window.through_version,
        })
        .collect();
    Ok(PublicationRecord {
        publication,
        members,
        inputs,
        windows,
    })
}

/// What differs between a stored publication and a request for the same attempt, if
/// anything.
fn difference(record: &PublicationRecord, commit: &PublicationCommit) -> Option<&'static str> {
    let publication = &record.publication;
    if publication.publication_id != commit.publication_id {
        Some("the attempt is published as another publication")
    } else if publication.workspace_id != commit.workspace_id {
        Some("another workspace")
    } else if publication.parent_publication != commit.expected_parent {
        Some("another parent")
    } else if publication.kind != commit.kind {
        Some("another kind")
    } else if record.members != ordered(&commit.members) {
        Some("other members")
    } else if record.inputs != ordered(&commit.inputs) {
        Some("other inputs")
    } else if record.windows != ordered_windows(&commit.windows) {
        Some("other change windows")
    } else {
        None
    }
}

/// Serialize maintainers of one workspace until the transaction ends.
async fn maintenance_lock(
    tx: &Tx<'_>,
    target: &Target,
    workspace: WorkspaceId,
) -> Result<(), OperationsError> {
    statements::maintenance_lock()
        .bind(tx, &format!("pse_ops.maintenance:{workspace}"))
        .one()
        .await
        .classify(target)?;
    Ok(())
}

/// Advance the workspace's maintenance epoch; returns the new epoch.
async fn advance_epoch(
    tx: &Tx<'_>,
    target: &Target,
    workspace: WorkspaceId,
) -> Result<i64, OperationsError> {
    statements::advance_epoch()
        .bind(tx, &workspace)
        .opt()
        .await
        .classify(target)?
        .ok_or_else(|| OperationsError::NotFound {
            entity: "workspace",
            id: workspace.to_string(),
        })
}

/// Lock a publication against new leases and return its workspace and retention phase.
async fn lock_publication(
    tx: &Tx<'_>,
    target: &Target,
    publication: PublicationId,
) -> Result<(WorkspaceId, Option<RetentionPhase>), OperationsError> {
    let locked = statements::lock_publication()
        .bind(tx, &publication)
        .opt()
        .await
        .classify(target)?
        .ok_or_else(|| OperationsError::NotFound {
            entity: "publication",
            id: publication.to_string(),
        })?;
    // A domain-typed result column arrives as its base type.
    Ok((WorkspaceId::from_id(locked.workspace_id), locked.phase))
}

async fn active_leases(
    tx: &Tx<'_>,
    target: &Target,
    publication: PublicationId,
) -> Result<i64, OperationsError> {
    statements::active_leases()
        .bind(tx, &publication)
        .one()
        .await
        .classify(target)
}

async fn head_of(
    tx: &Tx<'_>,
    target: &Target,
    workspace: WorkspaceId,
) -> Result<Option<PublicationId>, OperationsError> {
    statements::head()
        .bind(tx, &workspace)
        .opt()
        .await
        .classify(target)?
        .map(|head| head.publication_id)
        .ok_or_else(|| OperationsError::NotFound {
            entity: "workspace",
            id: workspace.to_string(),
        })
}

/// Require that a live publication selects the exact table version, and lock every
/// publication selecting it against being marked for deletion. A version no publication
/// selects is refused as a request defect; one only retiring publications select is
/// [`OperationsError::InputRetired`].
async fn require_live(
    tx: &Tx<'_>,
    target: &Target,
    member: &MemberDescriptor,
) -> Result<(), OperationsError> {
    // Lock first; read the marks in a new statement once the locks are held (a row that
    // was only locked is not rechecked within the statement that waited for it).
    let selecting = statements::share_selecting_publications()
        .params(
            tx,
            &statements::ShareSelectingPublicationsParams {
                table_uri: member.table_uri.as_str(),
                delta_version: member.delta_version,
            },
        )
        .all()
        .await
        .classify(target)?;
    if selecting.is_empty() {
        return Err(OperationsError::InvalidRequest {
            reason: format!(
                "{}@{} is neither written under the intent's member prefix nor selected by a publication",
                member.table_uri, member.delta_version
            ),
        });
    }
    let live = statements::live_selection()
        .params(
            tx,
            &statements::LiveSelectionParams {
                table_uri: member.table_uri.as_str(),
                delta_version: member.delta_version,
            },
        )
        .one()
        .await
        .classify(target)?;
    if live == 0 {
        return Err(OperationsError::InputRetired {
            table_uri: member.table_uri.clone(),
            delta_version: member.delta_version,
        });
    }
    Ok(())
}

/// Copy member rows of a publication, in one binary copy.
async fn copy_members(
    tx: &Tx<'_>,
    target: &Target,
    rows: &[RuntimeOperationalPublicationMembersRow],
) -> Result<(), OperationsError> {
    let mut cells: Vec<Cells<'_>> = Vec::with_capacity(rows.len());
    for row in rows {
        let publication: &(dyn ToSql + Sync) = &row.publication_id;
        cells.push(vec![
            publication,
            &row.role,
            &row.catalog_name,
            &row.schema_name,
            &row.table_name,
            &row.relation_id,
            &row.relation_version,
            &row.contract_fingerprint,
            &row.table_uri,
            &row.delta_version,
            &row.selection_kind,
            &row.revision_column,
            &row.revision_id,
        ]);
    }
    copy_in(tx, target, &copy::PUBLICATION_MEMBERS, &cells).await?;
    Ok(())
}

/// Copy the change windows of a publication, in one binary copy.
async fn copy_windows(
    tx: &Tx<'_>,
    target: &Target,
    publication: PublicationId,
    windows: &[VersionWindow],
) -> Result<(), OperationsError> {
    let mut cells: Vec<Cells<'_>> = Vec::with_capacity(windows.len());
    for window in windows {
        let publication: &(dyn ToSql + Sync) = &publication;
        cells.push(vec![
            publication,
            &window.table_uri,
            &window.from_version,
            &window.through_version,
        ]);
    }
    copy_in(tx, target, &copy::PUBLICATION_WINDOWS, &cells).await?;
    Ok(())
}

/// The ranges live publications, windows and intents protect on a workspace's tables.
async fn protections(
    tx: &Tx<'_>,
    target: &Target,
    workspace: WorkspaceId,
) -> Result<Vec<ProtectedRange>, OperationsError> {
    let mut ranges = Vec::new();
    for member in statements::protected_members()
        .params(
            tx,
            &statements::ProtectedMembersParams {
                workspace_id: workspace,
                expiring: RetentionPhase::Expiring,
            },
        )
        .all()
        .await
        .classify(target)?
    {
        ranges.push(ProtectedRange {
            table_uri: member.table_uri,
            from_version: member.delta_version,
            through_version: member.delta_version,
            reason: RetentionReason::Publication,
        });
    }
    for window in statements::protected_windows()
        .params(
            tx,
            &statements::ProtectedWindowsParams {
                workspace_id: workspace,
                expiring: RetentionPhase::Expiring,
            },
        )
        .all()
        .await
        .classify(target)?
    {
        ranges.push(ProtectedRange {
            table_uri: window.table_uri,
            from_version: window.from_version,
            through_version: window.through_version,
            reason: RetentionReason::Changes,
        });
    }
    for prefix in statements::live_intent_prefixes()
        .bind(tx, &workspace)
        .all()
        .await
        .classify(target)?
    {
        ranges.push(ProtectedRange {
            table_uri: prefix,
            from_version: 0,
            through_version: i64::MAX,
            reason: RetentionReason::Attempt,
        });
    }
    ranges.sort();
    Ok(ranges)
}

impl ProtectedRange {
    /// Whether the range applies to a table: an exact table match, or an attempt prefix
    /// the table lies under.
    pub fn covers(&self, table_uri: &str) -> bool {
        match self.reason {
            RetentionReason::Attempt => table_uri.starts_with(&self.table_uri),
            RetentionReason::Publication | RetentionReason::Changes => {
                self.table_uri == table_uri
            }
        }
    }
}

/// The catalog repository.
#[derive(Clone, Copy, Debug)]
pub struct Catalog<'s> {
    store: &'s Store,
}

impl<'s> Catalog<'s> {
    pub(crate) const fn new(store: &'s Store) -> Self {
        Self { store }
    }

    fn target(&self) -> &Target {
        self.store.target()
    }

    // ------------------------------------------------------------- workspaces --

    /// Register a workspace and its empty head.
    ///
    /// # Errors
    ///
    /// [`OperationsError::Duplicate`] for an existing identity, name or root; classified
    /// driver failures.
    pub async fn register_workspace(
        &self,
        workspace: &NewWorkspace,
    ) -> Result<RuntimeOperationalWorkspacesRow, OperationsError> {
        let target = self.target();
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(target)?;
        statements::insert_workspace()
            .params(
                &tx,
                &statements::InsertWorkspaceParams {
                    workspace_id: workspace.workspace_id,
                    name: workspace.name.as_str(),
                    root_uri: workspace.root_uri.as_str(),
                },
            )
            .await
            .classify(target)?;
        statements::insert_head()
            .bind(&tx, &workspace.workspace_id)
            .await
            .classify(target)?;
        let row = statements::workspace()
            .bind(&tx, &workspace.workspace_id)
            .one()
            .await
            .classify(target)?;
        tx.commit().await.classify(target)?;
        Ok(row)
    }

    /// A workspace by identity.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn workspace(
        &self,
        workspace: WorkspaceId,
    ) -> Result<Option<RuntimeOperationalWorkspacesRow>, OperationsError> {
        let client = self.store.client().await?;
        statements::workspace()
            .bind(&client, &workspace)
            .opt()
            .await
            .classify(self.target())
    }

    /// A workspace by name.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn workspace_by_name(
        &self,
        name: &str,
    ) -> Result<Option<RuntimeOperationalWorkspacesRow>, OperationsError> {
        let client = self.store.client().await?;
        statements::workspace_by_name()
            .bind(&client, &name)
            .opt()
            .await
            .classify(self.target())
    }

    /// The current head of a workspace; `None` before the first publication.
    ///
    /// # Errors
    ///
    /// [`OperationsError::NotFound`] for an unknown workspace; classified driver failures.
    pub async fn head(
        &self,
        workspace: WorkspaceId,
    ) -> Result<Option<PublicationId>, OperationsError> {
        let target = self.target();
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(target)?;
        let head = head_of(&tx, target, workspace).await?;
        tx.commit().await.classify(target)?;
        Ok(head)
    }

    // ---------------------------------------------------------------- intents --

    /// Register a publication intent before its first member write. Registering the same
    /// intent again returns it.
    ///
    /// # Errors
    ///
    /// [`OperationsError::PublicationIdentityReused`] when the identity is registered for
    /// another workspace, attempt or prefix; [`OperationsError::Duplicate`] when another
    /// intent owns the prefix; [`OperationsError::InvariantViolation`] for an unknown
    /// workspace or attempt; classified driver failures.
    pub async fn register_intent(
        &self,
        intent: &NewIntent,
    ) -> Result<RuntimeOperationalPublicationIntentsRow, OperationsError> {
        let target = self.target();
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(target)?;
        statements::insert_intent()
            .params(
                &tx,
                &statements::InsertIntentParams {
                    publication_id: intent.publication_id,
                    workspace_id: intent.workspace_id,
                    attempt_id: intent.attempt_id,
                    member_prefix: intent.member_prefix.as_str(),
                },
            )
            .await
            .classify(target)?;
        let stored = statements::intent()
            .bind(&tx, &intent.publication_id)
            .one()
            .await
            .classify(target)?;
        if stored.workspace_id != intent.workspace_id
            || stored.attempt_id != intent.attempt_id
            || stored.member_prefix != intent.member_prefix
        {
            return Err(OperationsError::PublicationIdentityReused {
                publication: intent.publication_id,
                reason: "the intent is registered for another workspace, attempt or prefix"
                    .to_owned(),
            });
        }
        tx.commit().await.classify(target)?;
        Ok(stored)
    }

    /// A registered intent.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn intent(
        &self,
        publication: PublicationId,
    ) -> Result<Option<RuntimeOperationalPublicationIntentsRow>, OperationsError> {
        let client = self.store.client().await?;
        statements::intent()
            .bind(&client, &publication)
            .opt()
            .await
            .classify(self.target())
    }

    /// Abandon an unpublished intent: it can never commit, and its members become
    /// reclaimable. Idempotent.
    ///
    /// # Errors
    ///
    /// [`OperationsError::InvalidRequest`] for a committed intent;
    /// [`OperationsError::NotFound`]; classified driver failures.
    pub async fn abandon_intent(&self, publication: PublicationId) -> Result<(), OperationsError> {
        let target = self.target();
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(target)?;
        statements::lock_intent()
            .bind(&tx, &publication)
            .opt()
            .await
            .classify(target)?
            .ok_or_else(|| OperationsError::NotFound {
                entity: "publication intent",
                id: publication.to_string(),
            })?;
        if statements::publication()
            .bind(&tx, &publication)
            .opt()
            .await
            .classify(target)?
            .is_some()
        {
            return Err(OperationsError::InvalidRequest {
                reason: format!("publication {publication} is committed; it cannot be abandoned"),
            });
        }
        statements::abandon_intent()
            .bind(&tx, &publication)
            .await
            .classify(target)?;
        tx.commit().await.classify(target)
    }

    /// Fence and return the workspace's reclaimable intents: unpublished intents that are
    /// abandoned, whose attempt is published as another publication, or whose attempt is
    /// stale or superseded. Each returned intent is abandoned first, so no commit can make
    /// its members visible while the caller removes them.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn claim_reclaimable(
        &self,
        workspace: WorkspaceId,
    ) -> Result<Vec<RuntimeOperationalPublicationIntentsRow>, OperationsError> {
        let target = self.target();
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(target)?;
        maintenance_lock(&tx, target, workspace).await?;
        let candidates = statements::reclaimable_intents()
            .params(
                &tx,
                &statements::ReclaimableIntentsParams {
                    workspace_id: workspace,
                    stale: AttemptState::Stale,
                    superseded: AttemptState::Superseded,
                },
            )
            .all()
            .await
            .classify(target)?;
        let mut claimed = Vec::with_capacity(candidates.len());
        for intent in candidates {
            // Re-read under the lock: a commit that held the intent may have published it.
            if statements::publication()
                .bind(&tx, &intent.publication_id)
                .opt()
                .await
                .classify(target)?
                .is_some()
            {
                continue;
            }
            statements::abandon_intent()
                .bind(&tx, &intent.publication_id)
                .await
                .classify(target)?;
            claimed.push(
                statements::intent()
                    .bind(&tx, &intent.publication_id)
                    .one()
                    .await
                    .classify(target)?,
            );
        }
        tx.commit().await.classify(target)?;
        Ok(claimed)
    }

    /// Record that an abandoned intent's members were removed. Idempotent.
    ///
    /// # Errors
    ///
    /// [`OperationsError::InvalidRequest`] for a committed intent;
    /// [`OperationsError::NotFound`]; classified driver failures.
    pub async fn mark_reclaimed(&self, publication: PublicationId) -> Result<(), OperationsError> {
        let target = self.target();
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(target)?;
        statements::lock_intent()
            .bind(&tx, &publication)
            .opt()
            .await
            .classify(target)?
            .ok_or_else(|| OperationsError::NotFound {
                entity: "publication intent",
                id: publication.to_string(),
            })?;
        if statements::publication()
            .bind(&tx, &publication)
            .opt()
            .await
            .classify(target)?
            .is_some()
        {
            return Err(OperationsError::InvalidRequest {
                reason: format!("publication {publication} is committed; it is never reclaimed"),
            });
        }
        statements::reclaim_intent()
            .bind(&tx, &publication)
            .await
            .classify(target)?;
        tx.commit().await.classify(target)
    }

    // ----------------------------------------------------------------- commit --

    /// Make a publication visible, in one transaction: the attempt must be finished, its
    /// intent registered and not abandoned; every written member lies under the intent's
    /// prefix and every retained member and input is an exact version a live publication
    /// selects (those publications are locked against retirement); the head must be the
    /// expected parent. The publication, its members and windows are inserted and the
    /// head advances.
    ///
    /// # Errors
    ///
    /// [`OperationsError::PublicationConflict`] when the head moved (re-prepare against
    /// the new head; never rebase); [`OperationsError::PublicationIdentityReused`] when
    /// the attempt or identity is committed with another request;
    /// [`OperationsError::IntentAbandoned`]; [`OperationsError::InputRetired`];
    /// [`OperationsError::InvalidRequest`] for an unfinished attempt or a written member
    /// outside the intent's prefix; [`OperationsError::NotFound`]; classified driver
    /// failures.
    pub async fn commit(&self, commit: &PublicationCommit) -> Result<Committed, OperationsError> {
        for (role, members) in [("member", &commit.members), ("input", &commit.inputs)] {
            let mut names = std::collections::BTreeSet::new();
            for member in members {
                if !names.insert((&member.catalog_name, &member.schema_name, &member.table_name)) {
                    return Err(OperationsError::InvalidRequest {
                        reason: format!(
                            "{role} {}.{}.{} is named twice",
                            member.catalog_name, member.schema_name, member.table_name
                        ),
                    });
                }
            }
        }
        let target = self.target();
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(target)?;
        let state = statements::share_attempt_state()
            .bind(&tx, &commit.attempt_id)
            .opt()
            .await
            .classify(target)?
            .ok_or_else(|| OperationsError::NotFound {
                entity: "attempt",
                id: commit.attempt_id.to_string(),
            })?;
        if !matches!(
            state,
            AttemptState::Completed
                | AttemptState::Partial
                | AttemptState::Failed
                | AttemptState::Cancelled
        ) {
            return Err(OperationsError::InvalidRequest {
                reason: format!(
                    "attempt {} is {}; only a finished attempt is published",
                    commit.attempt_id,
                    state.as_str()
                ),
            });
        }
        let intent = statements::lock_intent()
            .bind(&tx, &commit.publication_id)
            .opt()
            .await
            .classify(target)?
            .ok_or_else(|| OperationsError::NotFound {
                entity: "publication intent",
                id: commit.publication_id.to_string(),
            })?;
        if intent.workspace_id != commit.workspace_id || intent.attempt_id != commit.attempt_id {
            return Err(OperationsError::PublicationIdentityReused {
                publication: commit.publication_id,
                reason: "the intent is registered for another workspace or attempt".to_owned(),
            });
        }
        if intent.abandoned_at.is_some() {
            return Err(OperationsError::IntentAbandoned {
                publication: commit.publication_id,
            });
        }
        if let Some(published) = statements::publication_of_attempt()
            .bind(&tx, &commit.attempt_id)
            .opt()
            .await
            .classify(target)?
        {
            let stored = record(&tx, target, published).await?;
            return match difference(&stored, commit) {
                None => Ok(Committed::AlreadyCommitted {
                    publication_id: commit.publication_id,
                }),
                Some(reason) => Err(OperationsError::PublicationIdentityReused {
                    publication: commit.publication_id,
                    reason: reason.to_owned(),
                }),
            };
        }
        for member in &commit.members {
            if !member.table_uri.starts_with(&intent.member_prefix) {
                require_live(&tx, target, member).await?;
            }
        }
        for input in &commit.inputs {
            if input.table_uri.starts_with(&intent.member_prefix) {
                return Err(OperationsError::InvalidRequest {
                    reason: format!(
                        "input {} is a member this publication writes",
                        input.table_uri
                    ),
                });
            }
            require_live(&tx, target, input).await?;
        }
        let current = statements::lock_head()
            .bind(&tx, &commit.workspace_id)
            .opt()
            .await
            .classify(target)?
            .ok_or_else(|| OperationsError::NotFound {
                entity: "workspace",
                id: commit.workspace_id.to_string(),
            })?
            .publication_id;
        if current != commit.expected_parent {
            return Err(OperationsError::PublicationConflict {
                workspace: commit.workspace_id,
                expected: commit.expected_parent,
                current,
            });
        }
        let inserted = statements::insert_publication()
            .params(
                &tx,
                &statements::InsertPublicationParams {
                    publication_id: commit.publication_id,
                    workspace_id: commit.workspace_id,
                    parent_publication: commit.expected_parent,
                    attempt_id: commit.attempt_id,
                    kind: commit.kind,
                },
            )
            .await
            .classify(target);
        match inserted {
            // Another intent of this attempt committed concurrently: the attempt is
            // published once.
            Err(OperationsError::Duplicate { .. }) => {
                return Err(OperationsError::PublicationIdentityReused {
                    publication: commit.publication_id,
                    reason: "the attempt is published as another publication".to_owned(),
                });
            }
            other => {
                other?;
            }
        }
        let mut rows = Vec::with_capacity(commit.members.len() + commit.inputs.len());
        for member in ordered(&commit.members) {
            rows.push(member_row(
                commit.publication_id,
                PublicationMemberRole::Output,
                &member,
            )?);
        }
        for input in ordered(&commit.inputs) {
            rows.push(member_row(
                commit.publication_id,
                PublicationMemberRole::Input,
                &input,
            )?);
        }
        copy_members(&tx, target, &rows).await?;
        copy_windows(&tx, target, commit.publication_id, &commit.windows).await?;
        statements::advance_head()
            .params(
                &tx,
                &statements::AdvanceHeadParams {
                    publication_id: commit.publication_id,
                    workspace_id: commit.workspace_id,
                },
            )
            .await
            .classify(target)?;
        tx.commit().await.classify(target)?;
        Ok(Committed::Advanced {
            publication_id: commit.publication_id,
        })
    }

    /// A committed publication with its members and windows.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn publication(
        &self,
        publication: PublicationId,
    ) -> Result<Option<PublicationRecord>, OperationsError> {
        let target = self.target();
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(target)?;
        let found = match statements::publication()
            .bind(&tx, &publication)
            .opt()
            .await
            .classify(target)?
        {
            Some(row) => Some(record(&tx, target, row).await?),
            None => None,
        };
        tx.commit().await.classify(target)?;
        Ok(found)
    }

    // ------------------------------------------------------------- settlement --

    /// Settle an uncertain commit acknowledgement. Waits for any commit of the attempt
    /// still in flight, then reads the catalog; the result is recorded.
    ///
    /// # Errors
    ///
    /// [`OperationsError::NotFound`] for an unknown attempt or workspace; classified
    /// driver failures (the runtime reports an unreachable catalog as unresolved).
    pub async fn settle(&self, request: &SettleRequest) -> Result<Settlement, OperationsError> {
        let target = self.target();
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(target)?;
        statements::lock_attempt_row()
            .bind(&tx, &request.attempt_id)
            .opt()
            .await
            .classify(target)?
            .ok_or_else(|| OperationsError::NotFound {
                entity: "attempt",
                id: request.attempt_id.to_string(),
            })?;
        let head = head_of(&tx, target, request.workspace_id).await?;
        let published = statements::publication_of_attempt()
            .bind(&tx, &request.attempt_id)
            .opt()
            .await
            .classify(target)?;
        let settlement = match published {
            Some(publication) if publication.publication_id == request.publication_id => {
                Settlement::Committed {
                    publication_id: publication.publication_id,
                }
            }
            Some(publication) => Settlement::Conflict {
                reason: format!(
                    "attempt {} is published as publication {}",
                    request.attempt_id, publication.publication_id
                ),
                head,
            },
            None => {
                let intent = statements::intent()
                    .bind(&tx, &request.publication_id)
                    .opt()
                    .await
                    .classify(target)?;
                match intent {
                    // Never registered (D9) or abandoned: nothing can have been committed.
                    None => Settlement::ProvedNoncommit,
                    Some(intent) if intent.abandoned_at.is_some() => Settlement::ProvedNoncommit,
                    Some(_) if head == request.expected_parent => Settlement::ProvedNoncommit,
                    Some(_) => Settlement::Conflict {
                        reason: "the workspace head moved from the expected parent".to_owned(),
                        head,
                    },
                }
            }
        };
        let (outcome, publication_id, reason, conflict_head) = match &settlement {
            Settlement::Committed { publication_id } => {
                (SettlementOutcome::Committed, Some(*publication_id), None, None)
            }
            Settlement::ProvedNoncommit => (SettlementOutcome::ProvedNoncommit, None, None, None),
            Settlement::Conflict { reason, head } => (
                SettlementOutcome::Conflict,
                None,
                Some(reason.as_str()),
                *head,
            ),
        };
        statements::insert_settlement()
            .params(
                &tx,
                &statements::InsertSettlementParams {
                    settlement_id: request.settlement_id,
                    attempt_id: request.attempt_id,
                    outcome,
                    publication_id,
                    reason,
                    conflict_head,
                },
            )
            .await
            .classify(target)?;
        tx.commit().await.classify(target)?;
        Ok(settlement)
    }

    // ---------------------------------------------------------- reader leases --

    /// Take a reader lease in one short transaction. It returns the complete record and
    /// the workspace's maintenance epoch; the reader then reads the members without
    /// holding a database session, renews the lease before it expires and releases it
    /// when done.
    ///
    /// # Errors
    ///
    /// [`OperationsError::PublicationRetiring`] for an expiring or deleted publication;
    /// [`OperationsError::NotFound`] for an unknown publication, workspace or empty head;
    /// classified driver failures.
    pub async fn acquire_reader_lease(
        &self,
        lease_id: ReaderLeaseId,
        read: ReadTarget,
        holder: &str,
        ttl: Duration,
    ) -> Result<ReaderLease, OperationsError> {
        let target = self.target();
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(target)?;
        let (publication, head) = match read {
            ReadTarget::Publication(publication) => (publication, None),
            ReadTarget::Head(workspace) => (
                head_of(&tx, target, workspace)
                    .await?
                    .ok_or_else(|| OperationsError::NotFound {
                        entity: "workspace head",
                        id: workspace.to_string(),
                    })?,
                Some(workspace),
            ),
        };
        let row = statements::share_publication()
            .bind(&tx, &publication)
            .opt()
            .await
            .classify(target)?
            .ok_or_else(|| OperationsError::NotFound {
                entity: "publication",
                id: publication.to_string(),
            })?;
        // Read in a new statement once the share lock is held (see `require_live`).
        if let Some(phase) = statements::retention_phase()
            .bind(&tx, &publication)
            .opt()
            .await
            .classify(target)?
        {
            return Err(OperationsError::PublicationRetiring { publication, phase });
        }
        let lease = statements::insert_reader_lease()
            .params(
                &tx,
                &statements::InsertReaderLeaseParams {
                    lease_id,
                    publication_id: publication,
                    head_of: head,
                    holder,
                    ttl_us: micros(ttl),
                },
            )
            .one()
            .await
            .classify(target)?;
        let maintenance_epoch = statements::maintenance_epoch()
            .bind(&tx, &row.workspace_id)
            .one()
            .await
            .classify(target)?;
        let record = record(&tx, target, row).await?;
        tx.commit().await.classify(target)?;
        Ok(ReaderLease {
            lease,
            record,
            maintenance_epoch,
        })
    }

    /// Extend a live reader lease by `ttl` from now.
    ///
    /// # Errors
    ///
    /// [`OperationsError::ReaderLeaseLapsed`] when the lease expired or was released;
    /// classified driver failures.
    pub async fn renew_reader_lease(
        &self,
        lease: ReaderLeaseId,
        ttl: Duration,
    ) -> Result<RuntimeOperationalReaderLeasesRow, OperationsError> {
        let client = self.store.client().await?;
        statements::renew_reader_lease()
            .params(
                &client,
                &statements::RenewReaderLeaseParams {
                    ttl_us: micros(ttl),
                    lease_id: lease,
                },
            )
            .opt()
            .await
            .classify(self.target())?
            .ok_or(OperationsError::ReaderLeaseLapsed { lease })
    }

    /// A reader lease as stored.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn reader_lease(
        &self,
        lease: ReaderLeaseId,
    ) -> Result<Option<RuntimeOperationalReaderLeasesRow>, OperationsError> {
        let client = self.store.client().await?;
        statements::reader_lease()
            .bind(&client, &lease)
            .opt()
            .await
            .classify(self.target())
    }

    /// Release a reader lease. Returns whether it was still held.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn release_reader_lease(&self, lease: ReaderLeaseId) -> Result<bool, OperationsError> {
        let client = self.store.client().await?;
        let released = statements::release_reader_lease()
            .bind(&client, &lease)
            .await
            .classify(self.target())?;
        Ok(released == 1)
    }

    /// Live (unreleased, unexpired) reader leases on a publication.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn active_reader_leases(
        &self,
        publication: PublicationId,
    ) -> Result<i64, OperationsError> {
        let client = self.store.client().await?;
        statements::active_leases()
            .bind(&client, &publication)
            .one()
            .await
            .classify(self.target())
    }

    // ------------------------------------------------------------ maintenance --

    /// Phase one of retirement: mark a publication `expiring` so no new lease is granted,
    /// after advancing the workspace's maintenance epoch. The workspace head is protected.
    /// Idempotent for a publication already marked. Returns the new epoch.
    ///
    /// # Errors
    ///
    /// [`OperationsError::ProtectedPublication`] for the head;
    /// [`OperationsError::InvalidRequest`] for a publication of another workspace;
    /// [`OperationsError::NotFound`]; classified driver failures.
    pub async fn mark_expiring(
        &self,
        workspace: WorkspaceId,
        publication: PublicationId,
    ) -> Result<i64, OperationsError> {
        let target = self.target();
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(target)?;
        maintenance_lock(&tx, target, workspace).await?;
        let (owner, phase) = lock_publication(&tx, target, publication).await?;
        if owner != workspace {
            return Err(OperationsError::InvalidRequest {
                reason: format!("publication {publication} belongs to workspace {owner}"),
            });
        }
        if head_of(&tx, target, workspace).await? == Some(publication) {
            return Err(OperationsError::ProtectedPublication { publication });
        }
        let epoch = advance_epoch(&tx, target, workspace).await?;
        if phase.is_none() {
            statements::mark_expiring()
                .params(
                    &tx,
                    &statements::MarkExpiringParams {
                        publication_id: publication,
                        phase: RetentionPhase::Expiring,
                    },
                )
                .await
                .classify(target)?;
        }
        tx.commit().await.classify(target)?;
        Ok(epoch)
    }

    /// Poll until a publication has no live reader lease, or `timeout` passes.
    ///
    /// # Errors
    ///
    /// [`OperationsError::ReadersActive`] at the timeout; classified driver failures.
    pub async fn wait_for_readers(
        &self,
        publication: PublicationId,
        poll: Duration,
        timeout: Duration,
    ) -> Result<(), OperationsError> {
        let started = Instant::now();
        loop {
            let active = self.active_reader_leases(publication).await?;
            if active == 0 {
                return Ok(());
            }
            if started.elapsed() >= timeout {
                return Err(OperationsError::ReadersActive {
                    publication,
                    active,
                });
            }
            tokio::time::sleep(poll).await;
        }
    }

    /// The tables retiring an expiring publication removes: the tables it selects (its
    /// outputs, and inputs whose writer was already deleted) written under one of this
    /// workspace's intents, that no other undeleted publication selects and no undeleted
    /// publication's change window covers. Empty for a deleted publication.
    ///
    /// # Errors
    ///
    /// [`OperationsError::ReadersActive`] while a lease is live;
    /// [`OperationsError::InvalidRequest`] for a publication not marked in this
    /// workspace; [`OperationsError::NotFound`]; classified driver failures.
    pub async fn deletion_plan(
        &self,
        workspace: WorkspaceId,
        publication: PublicationId,
    ) -> Result<Vec<String>, OperationsError> {
        let target = self.target();
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(target)?;
        maintenance_lock(&tx, target, workspace).await?;
        let (owner, phase) = lock_publication(&tx, target, publication).await?;
        match (owner == workspace, phase) {
            (true, Some(RetentionPhase::Deleted)) => return Ok(Vec::new()),
            (true, Some(RetentionPhase::Expiring)) => {}
            _ => {
                return Err(OperationsError::InvalidRequest {
                    reason: format!(
                        "publication {publication} of workspace {owner} is not expiring in {workspace}"
                    ),
                });
            }
        }
        let active = active_leases(&tx, target, publication).await?;
        if active > 0 {
            return Err(OperationsError::ReadersActive {
                publication,
                active,
            });
        }
        let tables = statements::deletion_candidates()
            .params(
                &tx,
                &statements::DeletionCandidatesParams {
                    publication_id: publication,
                    workspace_id: workspace,
                    deleted: RetentionPhase::Deleted,
                },
            )
            .all()
            .await
            .classify(target)?;
        tx.commit().await.classify(target)?;
        Ok(tables)
    }

    /// Phase two of retirement, after `pse-catalog` removed the planned tables: mark an
    /// expiring publication `deleted`. Refused while any reader lease is live. Idempotent
    /// for a publication already deleted.
    ///
    /// # Errors
    ///
    /// [`OperationsError::ReadersActive`]; [`OperationsError::InvalidRequest`] for a
    /// publication not marked expiring; classified driver failures.
    pub async fn mark_deleted(
        &self,
        workspace: WorkspaceId,
        publication: PublicationId,
    ) -> Result<(), OperationsError> {
        let target = self.target();
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(target)?;
        maintenance_lock(&tx, target, workspace).await?;
        let (owner, phase) = lock_publication(&tx, target, publication).await?;
        match (owner == workspace, phase) {
            (true, Some(RetentionPhase::Deleted)) => return Ok(()),
            (true, Some(RetentionPhase::Expiring)) => {}
            _ => {
                return Err(OperationsError::InvalidRequest {
                    reason: format!(
                        "publication {publication} of workspace {owner} is not expiring in {workspace}"
                    ),
                });
            }
        }
        let active = active_leases(&tx, target, publication).await?;
        if active > 0 {
            return Err(OperationsError::ReadersActive {
                publication,
                active,
            });
        }
        statements::mark_deleted()
            .params(
                &tx,
                &statements::MarkDeletedParams {
                    phase: RetentionPhase::Deleted,
                    publication_id: publication,
                },
            )
            .await
            .classify(target)?;
        tx.commit().await.classify(target)
    }

    /// Begin collection in a workspace: advance its maintenance epoch, then fix the
    /// tables to maintain and every range each must keep.
    ///
    /// # Errors
    ///
    /// [`OperationsError::NotFound`] for an unknown workspace; classified driver failures.
    pub async fn begin_collect(&self, workspace: WorkspaceId) -> Result<CollectPlan, OperationsError> {
        let target = self.target();
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(target)?;
        maintenance_lock(&tx, target, workspace).await?;
        let maintenance_epoch = advance_epoch(&tx, target, workspace).await?;
        let tables = statements::collected_tables()
            .params(
                &tx,
                &statements::CollectedTablesParams {
                    workspace_id: workspace,
                    deleted: RetentionPhase::Deleted,
                },
            )
            .all()
            .await
            .classify(target)?;
        let ranges = protections(&tx, target, workspace).await?;
        tx.commit().await.classify(target)?;
        let tables = tables
            .into_iter()
            .map(|table| {
                let kept = ranges
                    .iter()
                    .filter(|range| range.covers(&table))
                    .cloned()
                    .collect();
                (table, kept)
            })
            .collect();
        Ok(CollectPlan {
            maintenance_epoch,
            tables,
        })
    }

    /// Every range retention must keep on the workspace's tables: the exact versions live
    /// publications select, the windows they read, and the whole history under the member
    /// prefixes of live intents. A publication is live while unmarked, or expiring with a
    /// live lease.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn protected_versions(
        &self,
        workspace: WorkspaceId,
    ) -> Result<Vec<ProtectedRange>, OperationsError> {
        let target = self.target();
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(target)?;
        let ranges = protections(&tx, target, workspace).await?;
        tx.commit().await.classify(target)?;
        Ok(ranges)
    }
}
