// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The publication catalog (ADR-0114 Outcomes 6 and 7; architecture §9.5).
//!
//! One transaction is the visibility boundary: it checks that the workspace head names the
//! expected parent, inserts the publication and its members, and advances the head. Readers
//! take short-lived lease rows; maintenance marks a publication `expiring`, waits for its
//! leases, and marks it `deleted` after `pse-catalog` removes the member files.
//! Transaction-scoped advisory locks serialize maintainers of one workspace.

use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use pse_ids::{ContentHash, SemanticId};
use sqlx::postgres::PgRow;
use sqlx::{FromRow, PgConnection, Row};

use crate::codec;
use crate::error::{Classify, OperationsError, Target};
use crate::lifecycle::AttemptState;
use crate::store::Store;

/// A workspace: one publication history with one head.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Workspace {
    /// The workspace identity, minted by the runtime.
    pub workspace_id: SemanticId,
    /// A unique human name.
    pub name: String,
    /// The root of its Delta member tables.
    pub root_uri: String,
}

/// One member table of a publication, at an exact Delta version.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Member {
    /// The member (relation) name.
    pub member: String,
    /// The member table location.
    pub table_uri: String,
    /// The exact Delta version.
    pub delta_version: i64,
    /// The contract fingerprint the member was written under.
    pub contract_fingerprint: ContentHash,
}

impl FromRow<'_, PgRow> for Member {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            member: row.try_get("member")?,
            table_uri: row.try_get("table_uri")?,
            delta_version: row.try_get("delta_version")?,
            contract_fingerprint: codec::hash(row, "contract_fingerprint")?,
        })
    }
}

/// A publication to make visible.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicationCommit {
    /// The publication identity, minted by the runtime before any effect.
    pub publication_id: SemanticId,
    /// The workspace.
    pub workspace_id: SemanticId,
    /// The finished attempt being published.
    pub attempt_id: SemanticId,
    /// The head this publication was prepared against; `None` for the first publication.
    pub expected_parent: Option<SemanticId>,
    /// The members, already written to attempt-scoped Delta locations.
    pub members: Vec<Member>,
}

/// A successful commit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Committed {
    /// This call made the publication visible and advanced the head.
    Advanced {
        /// The publication.
        publication_id: SemanticId,
    },
    /// The attempt was already published (publication is idempotent per attempt).
    AlreadyCommitted {
        /// The existing publication.
        publication_id: SemanticId,
    },
}

/// The outcome of settling an uncertain commit acknowledgement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Settlement {
    /// The attempt's publication is visible.
    Committed {
        /// The publication.
        publication_id: SemanticId,
    },
    /// No commit for the attempt was visible or in flight when settled.
    ProvedNoncommit,
}

/// What a reader wants to read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadTarget {
    /// An exact publication.
    Publication(SemanticId),
    /// The current head of a workspace, resolved and recorded in the lease.
    Head(SemanticId),
}

/// A granted reader lease with the member locations it protects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReaderLease {
    /// The lease identity.
    pub lease_id: SemanticId,
    /// The resolved publication.
    pub publication_id: SemanticId,
    /// When the lease lapses unless released earlier.
    pub expires_at: DateTime<Utc>,
    /// The members to read.
    pub members: Vec<Member>,
}

/// A table version retention must keep.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProtectedVersion {
    /// The member table.
    pub table_uri: String,
    /// The Delta version.
    pub delta_version: i64,
}

impl FromRow<'_, PgRow> for ProtectedVersion {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            table_uri: row.try_get("table_uri")?,
            delta_version: row.try_get("delta_version")?,
        })
    }
}

async fn members_of(
    conn: &mut PgConnection,
    target: &Target,
    publication: SemanticId,
) -> Result<Vec<Member>, OperationsError> {
    sqlx::query_as(
        "SELECT member, table_uri, delta_version, contract_fingerprint \
         FROM pse_ops.publication_members WHERE publication_id = $1 ORDER BY member",
    )
    .bind(codec::uuid(publication))
    .fetch_all(&mut *conn)
    .await
    .classify(target)
}

async fn publication_of_attempt(
    conn: &mut PgConnection,
    target: &Target,
    attempt: SemanticId,
) -> Result<Option<SemanticId>, OperationsError> {
    let found: Option<uuid::Uuid> =
        sqlx::query_scalar("SELECT publication_id FROM pse_ops.publications WHERE attempt_id = $1")
            .bind(codec::uuid(attempt))
            .fetch_optional(&mut *conn)
            .await
            .classify(target)?;
    Ok(found.map(|id| SemanticId::from_bytes(id.into_bytes())))
}

/// Serialize maintainers of one workspace until the transaction ends.
async fn maintenance_lock(
    conn: &mut PgConnection,
    target: &Target,
    workspace: SemanticId,
) -> Result<(), OperationsError> {
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
        .bind(format!("pse_ops.maintenance:{workspace}"))
        .execute(&mut *conn)
        .await
        .classify(target)?;
    Ok(())
}

/// Lock a publication against new leases and return its workspace and retention phase.
async fn lock_publication(
    conn: &mut PgConnection,
    target: &Target,
    publication: SemanticId,
) -> Result<(SemanticId, Option<String>), OperationsError> {
    let row = sqlx::query(
        "SELECT p.workspace_id, r.phase::text AS phase FROM pse_ops.publications AS p \
         LEFT JOIN pse_ops.retention_marks AS r ON r.publication_id = p.publication_id \
         WHERE p.publication_id = $1 FOR NO KEY UPDATE OF p",
    )
    .bind(codec::uuid(publication))
    .fetch_optional(&mut *conn)
    .await
    .classify(target)?
    .ok_or_else(|| OperationsError::NotFound {
        entity: "publication",
        id: publication.to_string(),
    })?;
    Ok((
        codec::id(&row, "workspace_id").classify(target)?,
        row.try_get("phase").classify(target)?,
    ))
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
        let mut tx = self.store.pool().begin().await.classify(target)?;
        sqlx::query(
            "INSERT INTO pse_ops.workspaces (workspace_id, name, root_uri) VALUES ($1, $2, $3)",
        )
        .bind(codec::uuid(workspace.workspace_id))
        .bind(&workspace.name)
        .bind(&workspace.root_uri)
        .execute(&mut *tx)
        .await
        .classify(target)?;
        sqlx::query("INSERT INTO pse_ops.publication_heads (workspace_id) VALUES ($1)")
            .bind(codec::uuid(workspace.workspace_id))
            .execute(&mut *tx)
            .await
            .classify(target)?;
        tx.commit().await.classify(target)
    }

    /// The current head of a workspace; `None` before the first publication.
    ///
    /// # Errors
    ///
    /// [`OperationsError::NotFound`] for an unknown workspace; classified driver failures.
    pub async fn head(&self, workspace: SemanticId) -> Result<Option<SemanticId>, OperationsError> {
        let head: Option<Option<uuid::Uuid>> = sqlx::query_scalar(
            "SELECT publication_id FROM pse_ops.publication_heads WHERE workspace_id = $1",
        )
        .bind(codec::uuid(workspace))
        .fetch_optional(self.store.pool())
        .await
        .classify(self.target())?;
        head.map(|head| head.map(|id| SemanticId::from_bytes(id.into_bytes())))
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
    /// finished; [`OperationsError::NotFound`]; classified driver failures.
    pub async fn commit(&self, commit: &PublicationCommit) -> Result<Committed, OperationsError> {
        let target = self.target();
        let mut tx = self.store.pool().begin().await.classify(target)?;
        // FOR SHARE on the attempt: a concurrent settlement (FOR UPDATE) waits for us.
        let state: Option<String> = sqlx::query_scalar(
            "SELECT state::text AS state FROM pse_ops.attempts WHERE attempt_id = $1 FOR SHARE",
        )
        .bind(codec::uuid(commit.attempt_id))
        .fetch_optional(&mut *tx)
        .await
        .classify(target)?;
        let state: AttemptState = state
            .ok_or_else(|| OperationsError::NotFound {
                entity: "attempt",
                id: commit.attempt_id.to_string(),
            })?
            .parse()
            .map_err(
                |error: pse_model::ModelError| OperationsError::CorruptValue {
                    column: "attempts.state",
                    detail: error.to_string(),
                },
            )?;
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
            publication_of_attempt(&mut tx, target, commit.attempt_id).await?
        {
            return Ok(Committed::AlreadyCommitted { publication_id });
        }
        let current: Option<Option<uuid::Uuid>> = sqlx::query_scalar(
            "SELECT publication_id FROM pse_ops.publication_heads \
             WHERE workspace_id = $1 FOR UPDATE",
        )
        .bind(codec::uuid(commit.workspace_id))
        .fetch_optional(&mut *tx)
        .await
        .classify(target)?;
        let current = current
            .ok_or_else(|| OperationsError::NotFound {
                entity: "workspace",
                id: commit.workspace_id.to_string(),
            })?
            .map(|id| SemanticId::from_bytes(id.into_bytes()));
        if current != commit.expected_parent {
            tx.rollback().await.classify(target)?;
            // The head may have moved because this very attempt won a concurrent retry.
            let mut conn = self.store.pool().acquire().await.classify(target)?;
            if let Some(publication_id) =
                publication_of_attempt(&mut conn, target, commit.attempt_id).await?
            {
                return Ok(Committed::AlreadyCommitted { publication_id });
            }
            return Err(OperationsError::PublicationConflict {
                workspace: commit.workspace_id,
                expected: commit.expected_parent,
                current,
            });
        }
        sqlx::query(
            "INSERT INTO pse_ops.publications \
                 (publication_id, workspace_id, parent_publication, attempt_id) \
             VALUES ($1, $2, $3, $4)",
        )
        .bind(codec::uuid(commit.publication_id))
        .bind(codec::uuid(commit.workspace_id))
        .bind(commit.expected_parent.map(codec::uuid))
        .bind(codec::uuid(commit.attempt_id))
        .execute(&mut *tx)
        .await
        .classify(target)?;
        sqlx::query(
            "INSERT INTO pse_ops.publication_members \
                 (publication_id, member, table_uri, delta_version, contract_fingerprint) \
             SELECT $1, m.member, m.table_uri, m.delta_version, m.contract_fingerprint \
             FROM UNNEST($2::text[], $3::text[], $4::bigint[], $5::bytea[]) \
                 AS m (member, table_uri, delta_version, contract_fingerprint)",
        )
        .bind(codec::uuid(commit.publication_id))
        .bind(
            commit
                .members
                .iter()
                .map(|m| m.member.as_str())
                .collect::<Vec<_>>(),
        )
        .bind(
            commit
                .members
                .iter()
                .map(|m| m.table_uri.as_str())
                .collect::<Vec<_>>(),
        )
        .bind(
            commit
                .members
                .iter()
                .map(|m| m.delta_version)
                .collect::<Vec<_>>(),
        )
        .bind(
            commit
                .members
                .iter()
                .map(|m| codec::hash_bytes(&m.contract_fingerprint))
                .collect::<Vec<_>>(),
        )
        .execute(&mut *tx)
        .await
        .classify(target)?;
        sqlx::query(
            "UPDATE pse_ops.publication_heads SET publication_id = $2, advanced_at = now() \
             WHERE workspace_id = $1",
        )
        .bind(codec::uuid(commit.workspace_id))
        .bind(codec::uuid(commit.publication_id))
        .execute(&mut *tx)
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
    pub async fn members(&self, publication: SemanticId) -> Result<Vec<Member>, OperationsError> {
        let mut conn = self.store.pool().acquire().await.classify(self.target())?;
        members_of(&mut conn, self.target(), publication).await
    }

    /// Settle an uncertain commit acknowledgement for `attempt`. Waits for any commit of
    /// that attempt still in flight, then reads the catalog; the result is recorded.
    ///
    /// # Errors
    ///
    /// [`OperationsError::NotFound`]; classified driver failures (the runtime reports an
    /// unreachable catalog as unresolved).
    pub async fn settle(&self, attempt: SemanticId) -> Result<Settlement, OperationsError> {
        let target = self.target();
        let mut tx = self.store.pool().begin().await.classify(target)?;
        let exists: Option<i32> =
            sqlx::query_scalar("SELECT 1 FROM pse_ops.attempts WHERE attempt_id = $1 FOR UPDATE")
                .bind(codec::uuid(attempt))
                .fetch_optional(&mut *tx)
                .await
                .classify(target)?;
        if exists.is_none() {
            return Err(OperationsError::NotFound {
                entity: "attempt",
                id: attempt.to_string(),
            });
        }
        let publication = publication_of_attempt(&mut tx, target, attempt).await?;
        let (settlement, outcome) = match publication {
            Some(publication_id) => (Settlement::Committed { publication_id }, "committed"),
            None => (Settlement::ProvedNoncommit, "proved_noncommit"),
        };
        sqlx::query(
            "INSERT INTO pse_ops.settlements (settlement_id, attempt_id, outcome, publication_id) \
             VALUES ($4, $1, $2::pse_ops.settlement_outcome, $3)",
        )
        .bind(codec::uuid(attempt))
        .bind(outcome)
        .bind(publication.map(codec::uuid))
        .bind(codec::uuid(crate::mint_id()))
        .execute(&mut *tx)
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
        let mut tx = self.store.pool().begin().await.classify(target)?;
        let publication = match read {
            ReadTarget::Publication(publication) => publication,
            ReadTarget::Head(workspace) => {
                let head: Option<Option<uuid::Uuid>> = sqlx::query_scalar(
                    "SELECT publication_id FROM pse_ops.publication_heads WHERE workspace_id = $1",
                )
                .bind(codec::uuid(workspace))
                .fetch_optional(&mut *tx)
                .await
                .classify(target)?;
                head.flatten()
                    .map(|id| SemanticId::from_bytes(id.into_bytes()))
                    .ok_or_else(|| OperationsError::NotFound {
                        entity: "workspace head",
                        id: workspace.to_string(),
                    })?
            }
        };
        // FOR SHARE conflicts with maintenance's FOR NO KEY UPDATE: a lease is either
        // granted before a publication is marked expiring or refused after.
        let row = sqlx::query(
            "SELECT r.phase::text AS phase FROM pse_ops.publications AS p \
             LEFT JOIN pse_ops.retention_marks AS r ON r.publication_id = p.publication_id \
             WHERE p.publication_id = $1 FOR SHARE OF p",
        )
        .bind(codec::uuid(publication))
        .fetch_optional(&mut *tx)
        .await
        .classify(target)?
        .ok_or_else(|| OperationsError::NotFound {
            entity: "publication",
            id: publication.to_string(),
        })?;
        let phase: Option<String> = row.try_get("phase").classify(target)?;
        if let Some(phase) = phase {
            return Err(OperationsError::PublicationRetiring { publication, phase });
        }
        let lease = sqlx::query(
            "INSERT INTO pse_ops.reader_leases (lease_id, publication_id, holder, expires_at) \
             VALUES ($4, $1, $2, now() + $3) RETURNING lease_id, expires_at",
        )
        .bind(codec::uuid(publication))
        .bind(holder)
        .bind(codec::interval(ttl))
        .bind(codec::uuid(crate::mint_id()))
        .fetch_one(&mut *tx)
        .await
        .classify(target)?;
        let members = members_of(&mut tx, target, publication).await?;
        tx.commit().await.classify(target)?;
        Ok(ReaderLease {
            lease_id: codec::id(&lease, "lease_id").classify(target)?,
            publication_id: publication,
            expires_at: lease.try_get("expires_at").classify(target)?,
            members,
        })
    }

    /// Release a reader lease. Returns whether it was still held.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn release_reader_lease(&self, lease: SemanticId) -> Result<bool, OperationsError> {
        let done = sqlx::query(
            "UPDATE pse_ops.reader_leases SET released_at = now() \
             WHERE lease_id = $1 AND released_at IS NULL",
        )
        .bind(codec::uuid(lease))
        .execute(self.store.pool())
        .await
        .classify(self.target())?;
        Ok(done.rows_affected() == 1)
    }

    /// Live (unreleased, unexpired) reader leases on a publication.
    ///
    /// # Errors
    ///
    /// Classified driver failures.
    pub async fn active_reader_leases(
        &self,
        publication: SemanticId,
    ) -> Result<i64, OperationsError> {
        let mut conn = self.store.pool().acquire().await.classify(self.target())?;
        active_leases(&mut conn, self.target(), publication).await
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
        workspace: SemanticId,
        publication: SemanticId,
    ) -> Result<(), OperationsError> {
        let target = self.target();
        let mut tx = self.store.pool().begin().await.classify(target)?;
        maintenance_lock(&mut tx, target, workspace).await?;
        let (owner, _) = lock_publication(&mut tx, target, publication).await?;
        if owner != workspace {
            return Err(OperationsError::InvalidRequest {
                reason: format!("publication {publication} belongs to workspace {owner}"),
            });
        }
        let head: Option<uuid::Uuid> = sqlx::query_scalar(
            "SELECT publication_id FROM pse_ops.publication_heads WHERE workspace_id = $1",
        )
        .bind(codec::uuid(workspace))
        .fetch_one(&mut *tx)
        .await
        .classify(target)?;
        if head == Some(codec::uuid(publication)) {
            return Err(OperationsError::ProtectedPublication { publication });
        }
        sqlx::query(
            "INSERT INTO pse_ops.retention_marks (publication_id, phase) VALUES ($1, 'expiring') \
             ON CONFLICT (publication_id) DO NOTHING",
        )
        .bind(codec::uuid(publication))
        .execute(&mut *tx)
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
        publication: SemanticId,
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
        workspace: SemanticId,
        publication: SemanticId,
    ) -> Result<(), OperationsError> {
        let target = self.target();
        let mut tx = self.store.pool().begin().await.classify(target)?;
        maintenance_lock(&mut tx, target, workspace).await?;
        let (owner, phase) = lock_publication(&mut tx, target, publication).await?;
        match (owner == workspace, phase.as_deref()) {
            (true, Some("deleted")) => return Ok(()),
            (true, Some("expiring")) => {}
            _ => {
                return Err(OperationsError::InvalidRequest {
                    reason: format!(
                        "publication {publication} of workspace {owner} is not expiring in {workspace}"
                    ),
                });
            }
        }
        let active = active_leases(&mut tx, target, publication).await?;
        if active > 0 {
            return Err(OperationsError::ReadersActive {
                publication,
                active,
            });
        }
        sqlx::query(
            "UPDATE pse_ops.retention_marks SET phase = 'deleted', deleted_at = now() \
             WHERE publication_id = $1",
        )
        .bind(codec::uuid(publication))
        .execute(&mut *tx)
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
        workspace: SemanticId,
    ) -> Result<Vec<ProtectedVersion>, OperationsError> {
        sqlx::query_as(
            "SELECT DISTINCT m.table_uri, m.delta_version \
             FROM pse_ops.publication_members AS m \
             JOIN pse_ops.publications AS p ON p.publication_id = m.publication_id \
             LEFT JOIN pse_ops.retention_marks AS r ON r.publication_id = p.publication_id \
             WHERE p.workspace_id = $1 \
               AND (r.publication_id IS NULL OR EXISTS ( \
                   SELECT 1 FROM pse_ops.reader_leases AS l \
                   WHERE l.publication_id = p.publication_id \
                     AND l.released_at IS NULL AND l.expires_at > now())) \
             ORDER BY m.table_uri, m.delta_version",
        )
        .bind(codec::uuid(workspace))
        .fetch_all(self.store.pool())
        .await
        .classify(self.target())
    }
}

async fn active_leases(
    conn: &mut PgConnection,
    target: &Target,
    publication: SemanticId,
) -> Result<i64, OperationsError> {
    sqlx::query_scalar(
        "SELECT count(*) FROM pse_ops.reader_leases \
         WHERE publication_id = $1 AND released_at IS NULL AND expires_at > now()",
    )
    .bind(codec::uuid(publication))
    .fetch_one(&mut *conn)
    .await
    .classify(target)
}
