// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The publication catalog (ADR-0114 Outcomes 6 and 7; architecture §9.5).
//!
//! One transaction is the visibility boundary: it checks that the workspace head names the
//! expected parent, inserts the publication and its members, and advances the head. Readers
//! take short-lived lease rows; maintenance marks a publication `expiring`, waits for its
//! leases, and marks it `deleted` after `pse-catalog` removes the member files.
//! Transaction-scoped advisory locks serialize maintainers of one workspace. Members,
//! leases and heads are the registry rows the catalog stores.

use std::time::{Duration, Instant};

use pse_model::generated::enums::{RetentionPhase, SettlementOutcome};
use pse_operations_queries::client::Params as _;
use pse_operations_queries::queries::catalog as statements;
use tokio_postgres::types::ToSql;

use crate::attempts::{AttemptId, Tx, micros};
use crate::bulk::{Cells, copy_in};
use crate::error::{Classify, OperationsError, Target};
use crate::generated::copy;
use crate::lifecycle::AttemptState;
use crate::store::Store;
pub use pse_model::generated::identities::{
    PublicationId, ReaderLeaseId, SettlementId, WorkspaceId,
};
pub use pse_model::generated::runtime::operational_publication_members::RuntimeOperationalPublicationMembersRow;
pub use pse_model::generated::runtime::operational_reader_leases::RuntimeOperationalReaderLeasesRow;
/// A table version retention must keep: a projection of the members.
pub use pse_operations_queries::queries::catalog::ProtectedVersion;

/// A workspace to register: one publication history with one head.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Workspace {
    /// The workspace identity, minted by the runtime.
    pub workspace_id: WorkspaceId,
    /// A unique human name.
    pub name: String,
    /// The root of its Delta member tables.
    pub root_uri: String,
}

/// A publication to make visible.
#[derive(Clone, Debug)]
pub struct PublicationCommit {
    /// The publication identity, minted by the runtime before any effect.
    pub publication_id: PublicationId,
    /// The workspace.
    pub workspace_id: WorkspaceId,
    /// The finished attempt being published.
    pub attempt_id: AttemptId,
    /// The head this publication was prepared against; `None` for the first publication.
    pub expected_parent: Option<PublicationId>,
    /// The members, already written to attempt-scoped Delta locations; each names this
    /// publication.
    pub members: Vec<RuntimeOperationalPublicationMembersRow>,
}

/// A successful commit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Committed {
    /// This call made the publication visible and advanced the head.
    Advanced {
        /// The publication.
        publication_id: PublicationId,
    },
    /// The attempt was already published (publication is idempotent per attempt).
    AlreadyCommitted {
        /// The existing publication.
        publication_id: PublicationId,
    },
}

/// The outcome of settling an uncertain commit acknowledgement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Settlement {
    /// The attempt's publication is visible.
    Committed {
        /// The publication.
        publication_id: PublicationId,
    },
    /// No commit for the attempt was visible or in flight when settled.
    ProvedNoncommit,
}

/// What a reader wants to read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadTarget {
    /// An exact publication.
    Publication(PublicationId),
    /// The current head of a workspace, resolved and recorded in the lease.
    Head(WorkspaceId),
}

/// A granted reader lease with the members it protects.
#[derive(Clone, Debug)]
pub struct ReaderLease {
    /// The lease as stored.
    pub lease: RuntimeOperationalReaderLeasesRow,
    /// The members to read.
    pub members: Vec<RuntimeOperationalPublicationMembersRow>,
}

async fn members_of(
    tx: &Tx<'_>,
    target: &Target,
    publication: PublicationId,
) -> Result<Vec<RuntimeOperationalPublicationMembersRow>, OperationsError> {
    statements::members()
        .bind(tx, &publication)
        .all()
        .await
        .classify(target)
}

async fn publication_of_attempt(
    tx: &Tx<'_>,
    target: &Target,
    attempt: AttemptId,
) -> Result<Option<PublicationId>, OperationsError> {
    Ok(statements::publication_of_attempt()
        .bind(tx, &attempt)
        .opt()
        .await
        .classify(target)?
        .map(|publication| publication.publication_id))
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

    /// Register a workspace and its empty head.
    ///
    /// # Errors
    ///
    /// [`OperationsError::Duplicate`] for an existing identity or name; classified driver
    /// failures.
    pub async fn register_workspace(&self, workspace: &Workspace) -> Result<(), OperationsError> {
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
        tx.commit().await.classify(target)
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
        let client = self.store.client().await?;
        statements::head()
            .bind(&client, &workspace)
            .opt()
            .await
            .classify(self.target())?
            .map(|head| head.publication_id)
            .ok_or_else(|| OperationsError::NotFound {
                entity: "workspace",
                id: workspace.to_string(),
            })
    }

    /// Make a publication visible: compare the head with the expected parent, insert the
    /// publication and its members, and advance the head, in one transaction.
    ///
    /// # Errors
    ///
    /// [`OperationsError::PublicationConflict`] when the head moved (re-prepare against the
    /// new head; never rebase); [`OperationsError::InvalidRequest`] when the attempt has not
    /// finished or a member names another publication; [`OperationsError::NotFound`];
    /// classified driver failures.
    pub async fn commit(&self, commit: &PublicationCommit) -> Result<Committed, OperationsError> {
        if let Some(member) = commit
            .members
            .iter()
            .find(|member| member.publication_id != commit.publication_id)
        {
            return Err(OperationsError::InvalidRequest {
                reason: format!(
                    "member {} names publication {}, not {}",
                    member.member, member.publication_id, commit.publication_id
                ),
            });
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
        if let Some(publication_id) =
            publication_of_attempt(&tx, target, commit.attempt_id).await?
        {
            return Ok(Committed::AlreadyCommitted { publication_id });
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
            tx.rollback().await.classify(target)?;
            // The head may have moved because this very attempt won a concurrent retry.
            let mut client = self.store.client().await?;
            let tx = client.transaction().await.classify(target)?;
            if let Some(publication_id) =
                publication_of_attempt(&tx, target, commit.attempt_id).await?
            {
                return Ok(Committed::AlreadyCommitted { publication_id });
            }
            return Err(OperationsError::PublicationConflict {
                workspace: commit.workspace_id,
                expected: commit.expected_parent,
                current,
            });
        }
        statements::insert_publication()
            .params(
                &tx,
                &statements::InsertPublicationParams {
                    publication_id: commit.publication_id,
                    workspace_id: commit.workspace_id,
                    parent_publication: commit.expected_parent,
                    attempt_id: commit.attempt_id,
                },
            )
            .await
            .classify(target)?;
        let mut rows: Vec<Cells<'_>> = Vec::with_capacity(commit.members.len());
        for member in &commit.members {
            let publication: &(dyn ToSql + Sync) = &member.publication_id;
            rows.push(vec![
                publication,
                &member.member,
                &member.table_uri,
                &member.delta_version,
                &member.contract_fingerprint,
            ]);
        }
        copy_in(&tx, target, &copy::PUBLICATION_MEMBERS, &rows).await?;
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

    /// The members of a publication.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn members(
        &self,
        publication: PublicationId,
    ) -> Result<Vec<RuntimeOperationalPublicationMembersRow>, OperationsError> {
        let client = self.store.client().await?;
        statements::members()
            .bind(&client, &publication)
            .all()
            .await
            .classify(self.target())
    }

    /// Settle an uncertain commit acknowledgement for `attempt`. Waits for any commit of
    /// that attempt still in flight, then reads the catalog; the result is recorded.
    ///
    /// # Errors
    ///
    /// [`OperationsError::NotFound`]; classified driver failures (the runtime reports an
    /// unreachable catalog as unresolved).
    pub async fn settle(&self, attempt: AttemptId) -> Result<Settlement, OperationsError> {
        let target = self.target();
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(target)?;
        statements::lock_attempt_row()
            .bind(&tx, &attempt)
            .opt()
            .await
            .classify(target)?
            .ok_or_else(|| OperationsError::NotFound {
                entity: "attempt",
                id: attempt.to_string(),
            })?;
        let publication = publication_of_attempt(&tx, target, attempt).await?;
        let (settlement, outcome) = match publication {
            Some(publication_id) => (
                Settlement::Committed { publication_id },
                SettlementOutcome::Committed,
            ),
            None => (Settlement::ProvedNoncommit, SettlementOutcome::ProvedNoncommit),
        };
        statements::insert_settlement()
            .params(
                &tx,
                &statements::InsertSettlementParams {
                    settlement_id: crate::mint_id(),
                    attempt_id: attempt,
                    outcome,
                    publication_id: publication,
                },
            )
            .await
            .classify(target)?;
        tx.commit().await.classify(target)?;
        Ok(settlement)
    }

    /// Take a reader lease in a short transaction. The reader then reads the returned
    /// members without holding a database session, and releases the lease when done.
    ///
    /// # Errors
    ///
    /// [`OperationsError::PublicationRetiring`] for an expiring or deleted publication;
    /// [`OperationsError::NotFound`] for an unknown publication, workspace or empty head;
    /// classified driver failures.
    pub async fn acquire_reader_lease(
        &self,
        read: ReadTarget,
        holder: &str,
        ttl: Duration,
    ) -> Result<ReaderLease, OperationsError> {
        let target = self.target();
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(target)?;
        let publication = match read {
            ReadTarget::Publication(publication) => publication,
            ReadTarget::Head(workspace) => statements::head()
                .bind(&tx, &workspace)
                .opt()
                .await
                .classify(target)?
                .and_then(|head| head.publication_id)
                .ok_or_else(|| OperationsError::NotFound {
                    entity: "workspace head",
                    id: workspace.to_string(),
                })?,
        };
        let phase = statements::share_publication()
            .bind(&tx, &publication)
            .opt()
            .await
            .classify(target)?
            .ok_or_else(|| OperationsError::NotFound {
                entity: "publication",
                id: publication.to_string(),
            })?;
        if let Some(phase) = phase {
            return Err(OperationsError::PublicationRetiring { publication, phase });
        }
        let lease = statements::insert_reader_lease()
            .params(
                &tx,
                &statements::InsertReaderLeaseParams {
                    lease_id: crate::mint_id(),
                    publication_id: publication,
                    holder,
                    ttl_us: micros(ttl),
                },
            )
            .one()
            .await
            .classify(target)?;
        let members = members_of(&tx, target, publication).await?;
        tx.commit().await.classify(target)?;
        Ok(ReaderLease { lease, members })
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

    /// Phase one of deletion: mark a publication `expiring` so no new lease is granted.
    /// The workspace head is protected. Idempotent for a publication already marked.
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
    ) -> Result<(), OperationsError> {
        let target = self.target();
        let mut client = self.store.client().await?;
        let tx = client.transaction().await.classify(target)?;
        maintenance_lock(&tx, target, workspace).await?;
        let (owner, _) = lock_publication(&tx, target, publication).await?;
        if owner != workspace {
            return Err(OperationsError::InvalidRequest {
                reason: format!("publication {publication} belongs to workspace {owner}"),
            });
        }
        let head = statements::head()
            .bind(&tx, &workspace)
            .one()
            .await
            .classify(target)?;
        if head.publication_id == Some(publication) {
            return Err(OperationsError::ProtectedPublication { publication });
        }
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
        tx.commit().await.classify(target)
    }

    /// Poll until an expiring publication has no live reader lease, or `timeout` passes.
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

    /// Phase two of deletion, after `pse-catalog` removed the member files: mark an
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

    /// The table versions retention must keep in a workspace: the members of every
    /// publication not marked for deletion, and of any publication with a live lease.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn protected_versions(
        &self,
        workspace: WorkspaceId,
    ) -> Result<Vec<ProtectedVersion>, OperationsError> {
        let client = self.store.client().await?;
        statements::protected_versions()
            .bind(&client, &workspace)
            .all()
            .await
            .classify(self.target())
    }
}
