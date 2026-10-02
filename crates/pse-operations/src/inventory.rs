// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Bounded restartable orphan discovery. Listing is supplied by the storage owner;
//! attribution, protections and deletion admission remain catalog-owned.

use pse_model::generated::enums::{OrphanDisposition, OrphanOwnership};
use pse_model::generated::identities::{ScanId, WorkspaceId};
use serde_json::Value;

use crate::catalog::Catalog;
use crate::error::Classify;
use crate::store::SchemaSession;
use crate::{OperationsError, Store, Target};

/// Maximum candidates accepted or returned by one operation.
pub const INVENTORY_PAGE_MAX: usize = 256;
const EVIDENCE_RECORDS_MAX: usize = 256;
const PREFIX_BYTES_MAX: usize = 8192;

/// Durable enumeration checkpoint. A process restart starts a new generation from root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScanCheckpoint {
    /// Stable inventory identity.
    pub scan_id: ScanId,
    /// Established workspace owner.
    pub workspace_id: WorkspaceId,
    /// Established member root; supplied to the storage enumerator.
    pub root_uri: String,
    /// Workspace maintenance epoch at this checkpoint.
    pub maintenance_epoch: i64,
    /// Enumeration generation, incremented on restart.
    pub generation: i64,
    /// Number of observed entries in this generation (duplicates may be observed).
    pub listed_count: i64,
    /// Whether the current enumeration completed.
    pub complete: bool,
}
/// One storage observation; observation supplies no ownership or deletion authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedCandidate {
    /// Prefix below the established root, normalized by the storage owner.
    pub prefix: String,
}
/// Recorded attribution, protections and disposition of one observed prefix.
#[derive(Clone, Debug, PartialEq)]
pub struct OrphanCandidate {
    /// Exact observed prefix.
    pub prefix: String,
    /// Most recent observation generation.
    pub generation: i64,
    /// Maintenance epoch when reconciled.
    pub discovery_epoch: i64,
    /// Catalog-established ownership; absence remains unresolved.
    pub ownership: OrphanOwnership,
    /// Exact current/retired inventory establishing attribution.
    pub evidence: Value,
    /// Current publication/intent and reader/export protection records.
    pub protections: Value,
    /// Recorded disposition; discovery is never deletion permission.
    pub disposition: OrphanDisposition,
}
/// Bounded candidate report. Its keyset cursor orders persisted inventory, not provider listing.
#[derive(Clone, Debug, PartialEq)]
pub struct CandidatePage {
    /// At most the requested bound.
    pub candidates: Vec<OrphanCandidate>,
    /// Last returned prefix; continue with this value while it is present.
    pub next_prefix: Option<String>,
}

fn invalid(reason: impl Into<String>) -> OperationsError {
    OperationsError::InvalidRequest {
        reason: reason.into(),
    }
}
fn page_bound(limit: usize) -> Result<i64, OperationsError> {
    if !(1..=INVENTORY_PAGE_MAX).contains(&limit) {
        return Err(invalid("inventory page bound must be 1..=256"));
    }
    i64::try_from(limit).map_err(|_| invalid("inventory page bound overflow"))
}
/// URI-boundary containment; the storage owner additionally checks provider/canonical paths.
fn confined(root: &str, prefix: &str) -> bool {
    let root = format!("{}/", root.trim_end_matches('/'));
    prefix.starts_with(&root)
        && prefix.len() > root.len()
        && !prefix.split('/').any(|part| matches!(part, "." | ".."))
}
fn scan_row(row: &tokio_postgres::Row) -> Result<ScanCheckpoint, OperationsError> {
    Ok(ScanCheckpoint {
        scan_id: row.try_get(0).map_err(|e| invalid(e.to_string()))?,
        workspace_id: row.try_get(1).map_err(|e| invalid(e.to_string()))?,
        root_uri: row.try_get(2).map_err(|e| invalid(e.to_string()))?,
        maintenance_epoch: row.try_get(3).map_err(|e| invalid(e.to_string()))?,
        generation: row.try_get(4).map_err(|e| invalid(e.to_string()))?,
        listed_count: row.try_get(5).map_err(|e| invalid(e.to_string()))?,
        complete: row.try_get(6).map_err(|e| invalid(e.to_string()))?,
    })
}
const SCAN_COLUMNS: &str =
    "scan_id,workspace_id,root_uri,maintenance_epoch,generation,listed_count,complete";
async fn lock_workspace(
    client: &tokio_postgres::Client,
    target: &Target,
    workspace: WorkspaceId,
    exclusive: bool,
) -> Result<i64, OperationsError> {
    let fence = if exclusive {
        "SELECT pg_advisory_xact_lock($1)"
    } else {
        "SELECT pg_advisory_xact_lock_shared($1)"
    };
    client
        .query_one(fence, &[&crate::catalog::PROTECTION_LOCK])
        .await
        .classify(target)?;
    let key = format!("pse_ops.maintenance:{workspace}");
    client
        .query_one(
            "SELECT pg_advisory_xact_lock(hashtextextended($1,0))",
            &[&key],
        )
        .await
        .classify(target)?;
    let row = client.query_opt("UPDATE pse_ops.workspaces SET maintenance_epoch=maintenance_epoch+1 WHERE workspace_id=$1 RETURNING maintenance_epoch", &[&workspace]).await.classify(target)?
        .ok_or_else(|| OperationsError::NotFound { entity: "workspace", id: workspace.to_string() })?;
    row.try_get(0).classify(target)
}
async fn fenced_session(store: &Store) -> Result<SchemaSession, OperationsError> {
    // Ensure ordinary admission exists before opening a dedicated claim/session.
    drop(store.client().await?);
    let session = store.schema_session().await?;
    session
        .client
        .query_one(
            "SELECT pg_advisory_lock_shared($1)",
            &[&crate::schema::SCHEMA_LOCK],
        )
        .await
        .classify(store.target())?;
    crate::schema::verify_ready(&session.client, store.target()).await?;
    session
        .client
        .batch_execute("BEGIN")
        .await
        .classify(store.target())?;
    Ok(session)
}
async fn scan(
    client: &tokio_postgres::Client,
    target: &Target,
    id: ScanId,
) -> Result<ScanCheckpoint, OperationsError> {
    let row = client
        .query_opt(
            &format!("SELECT {SCAN_COLUMNS} FROM pse_ops.orphan_scans WHERE scan_id=$1"),
            &[&id],
        )
        .await
        .classify(target)?
        .ok_or_else(|| OperationsError::NotFound {
            entity: "orphan scan",
            id: id.to_string(),
        })?;
    scan_row(&row)
}
/// Reconcile exact overlapping current and retired prefixes. SQL operates on row contracts
/// owned by the registry; no observed path or UUID spelling is ownership evidence.
async fn reconcile(
    client: &tokio_postgres::Client,
    target: &Target,
    workspace: WorkspaceId,
    prefix: &str,
) -> Result<(OrphanOwnership, Value, Value), OperationsError> {
    let row = client.query_one(r#"
      WITH evidence AS (
        SELECT jsonb_build_object('kind','intent','publication_id',i.publication_id,'workspace_id',i.workspace_id) AS record
        FROM pse_ops.publication_intents i WHERE i.workspace_id=$1 AND (starts_with(rtrim($2,'/')||'/',rtrim(i.member_prefix,'/')||'/') OR starts_with(rtrim(i.member_prefix,'/')||'/',rtrim($2,'/')||'/'))
        UNION ALL SELECT jsonb_build_object('kind','retired','reset_id',r.reset_id,'ordinal',r.ordinal,'disposition',r.disposition)
        FROM pse_ops.retired_inventory r WHERE r.workspace_id=$1 AND r.prefix IS NOT NULL AND (r.record_kind<>'orphan_candidates' OR r.document->>'ownership'='attributable') AND (starts_with(rtrim($2,'/')||'/',rtrim(r.prefix,'/')||'/') OR starts_with(rtrim(r.prefix,'/')||'/',rtrim($2,'/')||'/'))
      ), protection AS (
        SELECT jsonb_build_object('kind','live_intent','publication_id',i.publication_id) AS record
        FROM pse_ops.publication_intents i WHERE i.abandoned_at IS NULL
        AND NOT EXISTS(SELECT 1 FROM pse_ops.publications p WHERE p.publication_id=i.publication_id)
        AND (starts_with(rtrim($2,'/')||'/',rtrim(i.member_prefix,'/')||'/') OR starts_with(rtrim(i.member_prefix,'/')||'/',rtrim($2,'/')||'/'))
        UNION ALL SELECT jsonb_build_object('kind','publication','publication_id',p.publication_id)
        FROM pse_ops.publications p JOIN pse_ops.publication_intents i USING(publication_id)
        LEFT JOIN pse_ops.retention_marks r USING(publication_id)
        WHERE (starts_with(rtrim($2,'/')||'/',rtrim(i.member_prefix,'/')||'/') OR starts_with(rtrim(i.member_prefix,'/')||'/',rtrim($2,'/')||'/')
          OR EXISTS(SELECT 1 FROM pse_ops.publication_members m WHERE m.publication_id=p.publication_id AND (starts_with(rtrim(m.table_uri,'/')||'/',rtrim($2,'/')||'/') OR starts_with(rtrim($2,'/')||'/',rtrim(m.table_uri,'/')||'/')))
          OR EXISTS(SELECT 1 FROM pse_ops.publication_windows w WHERE w.publication_id=p.publication_id AND (starts_with(rtrim(w.table_uri,'/')||'/',rtrim($2,'/')||'/') OR starts_with(rtrim($2,'/')||'/',rtrim(w.table_uri,'/')||'/'))))
        AND (r.publication_id IS NULL OR EXISTS(SELECT 1 FROM pse_ops.reader_leases l WHERE l.publication_id=p.publication_id AND l.released_at IS NULL AND l.expires_at>now()))
        UNION ALL SELECT jsonb_build_object('kind','retired_reader','reset_id',r.reset_id,'ordinal',r.ordinal,'disposition',r.disposition)
        FROM pse_ops.retired_inventory r WHERE r.prefix IS NOT NULL AND r.disposition<>'deleted'
        AND (starts_with(rtrim($2,'/')||'/',rtrim(r.prefix,'/')||'/') OR starts_with(rtrim(r.prefix,'/')||'/',rtrim($2,'/')||'/'))
        AND (r.disposition='unresolved' OR EXISTS(SELECT 1 FROM jsonb_array_elements(r.protections) l WHERE l->>'expires_at' IS NULL OR (l->>'expires_at')::timestamptz>now()))
      ), packed AS (SELECT COALESCE((SELECT jsonb_agg(record) FROM (SELECT record FROM evidence LIMIT 257) e),'[]'::jsonb) AS evidence, COALESCE((SELECT jsonb_agg(record) FROM (SELECT record FROM protection LIMIT 257) p),'[]'::jsonb) AS protections)
      SELECT CASE WHEN octet_length(evidence::text)>65536 THEN '[{"kind":"metadata_bound","limit_bytes":65536}]'::jsonb ELSE evidence END, CASE WHEN octet_length(protections::text)>65536 THEN '[{"kind":"metadata_bound","limit_bytes":65536}]'::jsonb ELSE protections END FROM packed
    "#, &[&workspace,&prefix]).await.classify(target)?;
    let evidence: Value = row.try_get(0).classify(target)?;
    let protections: Value = row.try_get(1).classify(target)?;
    Ok(bounded_reconciliation(evidence, protections))
}
fn bounded_reconciliation(
    mut evidence: Value,
    mut protections: Value,
) -> (OrphanOwnership, Value, Value) {
    let complete = evidence.as_array().is_some_and(|a| {
        a.len() <= EVIDENCE_RECORDS_MAX
            && !a
                .iter()
                .any(|r| r.get("kind").and_then(Value::as_str) == Some("metadata_bound"))
    });
    if let Some(records) = evidence.as_array_mut()
        && records.len() > EVIDENCE_RECORDS_MAX
    {
        records.truncate(EVIDENCE_RECORDS_MAX);
        records.push(serde_json::json!({"kind":"evidence_bound","limit":EVIDENCE_RECORDS_MAX}));
    }
    if let Some(records) = protections.as_array_mut()
        && records.len() > EVIDENCE_RECORDS_MAX
    {
        records.truncate(EVIDENCE_RECORDS_MAX);
        records.push(serde_json::json!({"kind":"protection_bound","limit":EVIDENCE_RECORDS_MAX}));
    }
    let owned = complete && evidence.as_array().is_some_and(|a| !a.is_empty());
    (
        if owned {
            OrphanOwnership::Attributable
        } else {
            OrphanOwnership::Unattributable
        },
        evidence,
        protections,
    )
}
fn disposition(ownership: OrphanOwnership, protections: &Value) -> OrphanDisposition {
    if ownership == OrphanOwnership::Unattributable {
        OrphanDisposition::Unresolved
    } else if protections.as_array().is_none_or(|a| !a.is_empty()) {
        OrphanDisposition::Protected
    } else {
        OrphanDisposition::Discovered
    }
}

impl Catalog<'_> {
    /// Start durable discovery at the workspace's established root.
    /// # Errors
    /// Unknown workspace, duplicate/conflicting scan identity or driver failures.
    pub async fn begin_orphan_scan(
        &self,
        scan_id: ScanId,
        workspace: WorkspaceId,
    ) -> Result<ScanCheckpoint, OperationsError> {
        let session = fenced_session(self.store).await?;
        let target = self.target();
        let epoch = lock_workspace(&session.client, target, workspace, false).await?;
        session.client.execute("INSERT INTO pse_ops.orphan_scans(scan_id,workspace_id,root_uri,maintenance_epoch,generation,listed_count,complete) SELECT $1,workspace_id,root_uri,$3,1,0,false FROM pse_ops.workspaces WHERE workspace_id=$2 ON CONFLICT(scan_id) DO NOTHING", &[&scan_id,&workspace,&epoch]).await.classify(target)?;
        let result = scan(&session.client, target, scan_id).await?;
        if result.workspace_id != workspace {
            return Err(invalid("scan identity belongs to another workspace"));
        }
        session
            .client
            .batch_execute("COMMIT")
            .await
            .classify(target)?;
        Ok(result)
    }
    /// Resume persisted reporting after restart while listing anew from root in a new generation.
    /// # Errors
    /// Unknown scan, generation overflow or driver failures.
    pub async fn restart_orphan_scan(
        &self,
        scan_id: ScanId,
    ) -> Result<ScanCheckpoint, OperationsError> {
        let session = fenced_session(self.store).await?;
        let old = scan(&session.client, self.target(), scan_id).await?;
        let epoch = lock_workspace(&session.client, self.target(), old.workspace_id, false).await?;
        session.client.execute("UPDATE pse_ops.orphan_scans SET generation=generation+1,listed_count=0,complete=false,maintenance_epoch=$2 WHERE scan_id=$1", &[&scan_id,&epoch]).await.classify(self.target())?;
        let checkpoint = scan(&session.client, self.target(), scan_id).await?;
        session
            .client
            .batch_execute("COMMIT")
            .await
            .classify(self.target())?;
        Ok(checkpoint)
    }
    /// Reconcile and persist one bounded storage batch, including unknown ownership.
    /// # Errors
    /// Invalid bounds/root, stale generation, already completed enumeration or driver failures.
    pub async fn record_orphan_page(
        &self,
        scan_id: ScanId,
        generation: i64,
        observations: &[ObservedCandidate],
        complete: bool,
    ) -> Result<ScanCheckpoint, OperationsError> {
        if observations.len() > INVENTORY_PAGE_MAX {
            return Err(invalid("inventory page exceeds 256 candidates"));
        }
        let session = fenced_session(self.store).await?;
        let initial = scan(&session.client, self.target(), scan_id).await?;
        let epoch =
            lock_workspace(&session.client, self.target(), initial.workspace_id, false).await?;
        let initial = scan(&session.client, self.target(), scan_id).await?;
        if initial.generation != generation || initial.complete {
            return Err(invalid("stale or completed enumeration generation"));
        }
        for item in observations {
            if item.prefix.len() > PREFIX_BYTES_MAX || !confined(&initial.root_uri, &item.prefix) {
                return Err(invalid(
                    "observed candidate is outside the established root",
                ));
            }
            let (ownership, evidence, protections) = reconcile(
                &session.client,
                self.target(),
                initial.workspace_id,
                &item.prefix,
            )
            .await?;
            let disposition = disposition(ownership, &protections);
            session.client.execute("INSERT INTO pse_ops.orphan_candidates(scan_id,prefix,generation,discovery_epoch,ownership,evidence,protections,disposition,claim_epoch) VALUES($1,$2,$3,$4,$5,$6,$7,$8,NULL) ON CONFLICT(scan_id,prefix) DO UPDATE SET generation=excluded.generation,discovery_epoch=excluded.discovery_epoch,ownership=excluded.ownership,evidence=excluded.evidence,protections=excluded.protections,disposition=excluded.disposition,claim_epoch=NULL", &[&scan_id,&item.prefix,&generation,&epoch,&ownership,&evidence,&protections,&disposition]).await.classify(self.target())?;
        }
        let count =
            i64::try_from(observations.len()).map_err(|_| invalid("inventory count overflow"))?;
        session.client.execute("UPDATE pse_ops.orphan_scans SET listed_count=listed_count+$2,complete=$3,maintenance_epoch=$4 WHERE scan_id=$1", &[&scan_id,&count,&complete,&epoch]).await.classify(self.target())?;
        let checkpoint = scan(&session.client, self.target(), scan_id).await?;
        session
            .client
            .batch_execute("COMMIT")
            .await
            .classify(self.target())?;
        Ok(checkpoint)
    }
    /// Read a bounded page of all durable candidates, including prior generations and unresolved entries.
    /// # Errors
    /// Invalid bound or classified driver failures.
    pub async fn orphan_candidates(
        &self,
        scan_id: ScanId,
        after_prefix: Option<&str>,
        limit: usize,
    ) -> Result<CandidatePage, OperationsError> {
        let bound = page_bound(limit)?;
        let client = self.store.client().await?;
        let _ = scan(&client, self.target(), scan_id).await?;
        let rows=client.query("SELECT prefix,generation,discovery_epoch,ownership,evidence,protections,disposition FROM pse_ops.orphan_candidates WHERE scan_id=$1 AND ($2::text IS NULL OR prefix>$2) ORDER BY prefix LIMIT $3", &[&scan_id,&after_prefix,&bound]).await.classify(self.target())?;
        let mut candidates = Vec::with_capacity(rows.len());
        for r in rows {
            candidates.push(OrphanCandidate {
                prefix: r.try_get(0).classify(self.target())?,
                generation: r.try_get(1).classify(self.target())?,
                discovery_epoch: r.try_get(2).classify(self.target())?,
                ownership: r.try_get(3).classify(self.target())?,
                evidence: r.try_get(4).classify(self.target())?,
                protections: r.try_get(5).classify(self.target())?,
                disposition: r.try_get(6).classify(self.target())?,
            });
        }
        let next_prefix = if candidates.len() == limit {
            candidates.last().map(|c| c.prefix.clone())
        } else {
            None
        };
        Ok(CandidatePage {
            candidates,
            next_prefix,
        })
    }
    /// Explicitly claim one recorded attributable prefix. The returned capability holds the
    /// schema lease and workspace maintenance/protection transaction until deletion completes.
    /// # Errors
    /// Unrecorded/unattributable/protected prefix, root mismatch or driver failures.
    pub async fn claim_orphan(
        &self,
        scan_id: ScanId,
        prefix: &str,
    ) -> Result<OrphanClaim, OperationsError> {
        let session = fenced_session(self.store).await?;
        let checkpoint = scan(&session.client, self.target(), scan_id).await?;
        let epoch = lock_workspace(
            &session.client,
            self.target(),
            checkpoint.workspace_id,
            true,
        )
        .await?;
        if prefix.len() > PREFIX_BYTES_MAX || !confined(&checkpoint.root_uri, prefix) {
            return Err(invalid("claim is outside established root"));
        }
        if session.client.query_opt("SELECT prefix FROM pse_ops.orphan_candidates WHERE scan_id=$1 AND prefix=$2 FOR UPDATE", &[&scan_id,&prefix]).await.classify(self.target())?.is_none() { return Err(invalid("candidate was not discovered")); }
        let (ownership, evidence, protections) = reconcile(
            &session.client,
            self.target(),
            checkpoint.workspace_id,
            prefix,
        )
        .await?;
        if disposition(ownership, &protections) != OrphanDisposition::Discovered {
            return Err(invalid(
                "candidate ownership or protection refuses deletion",
            ));
        }
        session.client.execute("UPDATE pse_ops.orphan_candidates SET ownership=$3,evidence=$4,protections=$5,disposition='claimed',claim_epoch=$6 WHERE scan_id=$1 AND prefix=$2", &[&scan_id,&prefix,&ownership,&evidence,&protections,&epoch]).await.classify(self.target())?;
        Ok(OrphanClaim {
            session,
            store: self.store.clone(),
            scan_id,
            prefix: prefix.to_owned(),
            workspace: checkpoint.workspace_id,
            epoch,
        })
    }
}
/// Exclusive candidate admission. Dropping before completion rolls back the claim and releases its fence.
#[derive(Debug)]
pub struct OrphanClaim {
    session: SchemaSession,
    store: Store,
    scan_id: ScanId,
    prefix: String,
    workspace: WorkspaceId,
    epoch: i64,
}
impl OrphanClaim {
    /// Prefix this capability permits deleting.
    pub fn prefix(&self) -> &str {
        &self.prefix
    }
    /// Established workspace whose maintenance fence this claim holds.
    pub const fn workspace_id(&self) -> WorkspaceId {
        self.workspace
    }
    /// Workspace epoch held through the storage operation.
    pub const fn maintenance_epoch(&self) -> i64 {
        self.epoch
    }
    /// Record completed idempotent storage deletion and release the protection fence.
    /// # Errors
    /// Classified driver failures; after a lost acknowledgement inspect durable candidate disposition.
    pub async fn complete_deleted(self) -> Result<(), OperationsError> {
        let client = &self.session.client;
        let target = self.store.target();
        client.execute("UPDATE pse_ops.orphan_candidates SET disposition='deleted' WHERE scan_id=$1 AND prefix=$2 AND disposition='claimed'", &[&self.scan_id,&self.prefix]).await.classify(target)?;
        client.execute("UPDATE pse_ops.retired_inventory SET disposition='deleted' WHERE workspace_id=$1 AND prefix IS NOT NULL AND starts_with(rtrim(prefix,'/')||'/',rtrim($2,'/')||'/')", &[&self.workspace,&self.prefix]).await.classify(target)?;
        client.batch_execute("COMMIT").await.classify(target)
    }
}

#[cfg(test)]
mod inventory_unit {
    use super::*;
    #[test]
    fn bounds_and_containment_do_not_admit_root_or_siblings() {
        for prefix in [
            "file:///root/",
            "file:///root2/x",
            "file:///root/../x",
            "file:///root/./x",
        ] {
            assert!(!confined("file:///root/", prefix));
        }
        assert!(confined("file:///root/", "file:///root/member/table/"));
        assert!(page_bound(0).is_err());
        assert!(page_bound(INVENTORY_PAGE_MAX + 1).is_err());
        assert_eq!(page_bound(1).unwrap(), 1);
    }
    #[test]
    fn attribution_bound_preserves_unresolved_evidence_instead_of_admitting_incomplete_proof() {
        let evidence = Value::Array(vec![
            serde_json::json!({"kind":"intent"});
            EVIDENCE_RECORDS_MAX + 1
        ]);
        let (ownership, evidence, protections) =
            bounded_reconciliation(evidence, serde_json::json!([]));
        assert_eq!(ownership, OrphanOwnership::Unattributable);
        assert_eq!(
            evidence.as_array().unwrap().last().unwrap()["kind"],
            "evidence_bound"
        );
        assert_eq!(
            disposition(ownership, &protections),
            OrphanDisposition::Unresolved
        );
    }
    #[test]
    fn absence_is_unresolved_and_any_protection_refuses_discovery_admission() {
        assert_eq!(
            disposition(OrphanOwnership::Unattributable, &serde_json::json!([])),
            OrphanDisposition::Unresolved
        );
        assert_eq!(
            disposition(
                OrphanOwnership::Attributable,
                &serde_json::json!([{"reader":"live"}])
            ),
            OrphanDisposition::Protected
        );
        assert_eq!(
            disposition(OrphanOwnership::Attributable, &Value::Null),
            OrphanDisposition::Protected
        );
        assert_eq!(
            disposition(OrphanOwnership::Attributable, &serde_json::json!([])),
            OrphanDisposition::Discovered
        );
    }
}
