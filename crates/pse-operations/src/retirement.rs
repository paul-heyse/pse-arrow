// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Completed external retirement export and atomic schema reset with lost-ack settlement.

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use pse_ids::{ContentHash, Frame, FramedHasher};
use pse_model::generated::enums::OrphanDisposition;
use pse_model::generated::identities::ResetId;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::Classify;
use crate::store::SchemaSession;
use crate::{OperationsError, Store, Target};

const PAGE: usize = 128;
const ROW_BYTES_MAX: u64 = 4 * 1024 * 1024;
/// Maximum completed inventory records accepted by one reset. Oversized inventories refuse.
pub const RESET_ROWS_MAX: u64 = 1_000_000;
const TABLES: &[&str] = &[
    "workspaces",
    "publication_intents",
    "publications",
    "publication_heads",
    "publication_members",
    "publication_windows",
    "reader_leases",
    "retention_marks",
    "settlements",
    "orphan_scans",
    "orphan_candidates",
    "reset_records",
    "retired_inventory",
    "schema_support_state",
    "catalog_schema_history",
    "operations_schema_history",
];

/// Completed immutable external manifest; reset verifies its content and live source again.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResetManifestReceipt {
    /// Caller-minted reset settlement identity.
    pub reset_id: ResetId,
    /// Digest over the exact versioned header and inventory records.
    pub manifest_digest: ContentHash,
    /// Exact source declaration exported.
    pub source_fingerprint: String,
    /// Completed local file outside every erased workspace/member root.
    pub destination: PathBuf,
    /// Complete inventory count; no truncation is accepted.
    pub inventory_rows: u64,
    /// Explicit bound applied to export and import.
    pub max_rows: u64,
}
/// Reset settlement receipt. A retry settles its identity before any repeat DROP.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResetReport {
    /// The settled reset.
    pub reset_id: ResetId,
    /// Exact completed manifest digest.
    pub manifest_digest: ContentHash,
    /// Whether the transaction had already committed, including lost acknowledgement.
    pub already_applied: bool,
    /// Number of completed exported records.
    pub inventory_rows: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Header {
    version: u32,
    reset_id: String,
    source: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct InventoryRecord {
    table: String,
    ordinal: u64,
    document: Value,
    workspace_id: Option<String>,
    root_uri: Option<String>,
    prefix: Option<String>,
    protections: Value,
    disposition: OrphanDisposition,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum ManifestLine {
    Header(Header),
    Record(InventoryRecord),
    Complete { rows: u64, digest: String },
}
fn invalid(reason: impl Into<String>) -> OperationsError {
    OperationsError::InvalidRequest {
        reason: reason.into(),
    }
}
/// A complete explicit reset retires its current catalog generation. Only authoritative
/// prefix records become attributable retirement; prior unresolved records are imported unchanged.
fn fresh_retirement_disposition(
    table: &str,
    document: &Value,
    prefix: Option<&str>,
    protections: &Value,
) -> OrphanDisposition {
    let authoritative = matches!(
        table,
        "publication_intents"
            | "publications"
            | "publication_heads"
            | "publication_members"
            | "publication_windows"
            | "reader_leases"
            | "retention_marks"
    ) || (table == "orphan_candidates"
        && document.get("ownership").and_then(Value::as_str) == Some("attributable"));
    if !authoritative || prefix.is_none() {
        return OrphanDisposition::Unresolved;
    }
    let Some(leases) = protections.as_array() else {
        return OrphanDisposition::Unresolved;
    };
    if leases.is_empty() {
        return OrphanDisposition::Discovered;
    }
    if leases.iter().any(|l| {
        l.get("expires_at")
            .and_then(Value::as_str)
            .is_none_or(|expiry| {
                time::OffsetDateTime::parse(expiry, &time::format_description::well_known::Rfc3339)
                    .is_err()
            })
    }) {
        return OrphanDisposition::Unresolved;
    }
    // The recorded deadlines are interpreted at fresh claim time; passage of time never
    // changes the completed export's bytes or digest.
    OrphanDisposition::Protected
}
fn io(error: impl std::fmt::Display) -> OperationsError {
    invalid(format!("retirement manifest: {error}"))
}
fn bound(max_rows: u64) -> Result<(), OperationsError> {
    if max_rows == 0 || max_rows > RESET_ROWS_MAX {
        Err(invalid("reset row bound must be 1..=1000000"))
    } else {
        Ok(())
    }
}
fn encoded(line: &ManifestLine) -> Result<Vec<u8>, OperationsError> {
    let mut bytes = serde_json::to_vec(line).map_err(io)?;
    bytes.push(b'\n');
    if bytes.len() as u64 > ROW_BYTES_MAX {
        return Err(invalid("retirement record exceeds 4 MiB bound"));
    }
    Ok(bytes)
}
fn header(reset_id: ResetId) -> Header {
    Header {
        version: 1,
        reset_id: reset_id.to_string(),
        source: crate::generated::SCHEMA_FINGERPRINT_HEX.to_owned(),
    }
}
async fn exclusive(store: &Store) -> Result<SchemaSession, OperationsError> {
    if store.has_schema_lease().await {
        return Err(OperationsError::MigrationRefused {
            reason: crate::MigrationRefusal::ActiveGeneration,
            detail: "close opened generations before reset/export".into(),
        });
    }
    let session = store.schema_session().await?;
    let locked: bool = session
        .client
        .query_one(
            "SELECT pg_try_advisory_lock($1)",
            &[&crate::schema::SCHEMA_LOCK],
        )
        .await
        .classify(store.target())?
        .try_get(0)
        .classify(store.target())?;
    if !locked {
        return Err(OperationsError::MigrationRefused {
            reason: crate::MigrationRefusal::ActiveGeneration,
            detail: "active generation or maintenance owns namespace".into(),
        });
    }
    Ok(session)
}
fn local_uri_path(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    if !rest.starts_with('/') {
        return None;
    }
    // Existing catalog file URIs use URL percent encoding; decode through the URL crate.
    url::Url::parse(uri).ok()?.to_file_path().ok()
}
fn canonical_existing_ancestor(path: &Path) -> Result<PathBuf, OperationsError> {
    let mut ancestor = path;
    let mut tail = Vec::new();
    while !ancestor.exists() {
        tail.push(
            ancestor
                .file_name()
                .ok_or_else(|| invalid("local inventory root has no existing ancestor"))?
                .to_owned(),
        );
        ancestor = ancestor
            .parent()
            .ok_or_else(|| invalid("local inventory root has no existing ancestor"))?;
    }
    let mut result = ancestor.canonicalize().map_err(io)?;
    for component in tail.into_iter().rev() {
        result.push(component);
    }
    Ok(result)
}
fn outside(destination: &Path, root: &str) -> Result<(), OperationsError> {
    if let Some(path) = local_uri_path(root) {
        let root = canonical_existing_ancestor(&path)?;
        if destination.starts_with(&root) {
            return Err(invalid(
                "retirement destination is inside an erased workspace/member root",
            ));
        }
    } else if root.starts_with("file:") {
        return Err(invalid("unverifiable local inventory root"));
    }
    Ok(())
}
fn destination(path: &Path) -> Result<PathBuf, OperationsError> {
    let parent = path
        .parent()
        .ok_or_else(|| invalid("manifest destination needs an existing parent"))?
        .canonicalize()
        .map_err(io)?;
    let name = path
        .file_name()
        .ok_or_else(|| invalid("manifest destination needs a filename"))?;
    let result = parent.join(name);
    if result.exists() {
        return Err(invalid("manifest destination already exists"));
    }
    Ok(result)
}
/// Query maps each original row to its established workspace/prefix and the complete active
/// reader/export expiry records. Unknown attribution remains explicit rather than fabricated.
fn export_query(table: &str) -> String {
    let (join, workspace, root, mut prefix, publication) = match table {
        "workspaces" => (
            "",
            "x.workspace_id::text",
            "x.root_uri",
            "NULL::text",
            "NULL::uuid",
        ),
        "publication_intents" => (
            "JOIN pse_ops.workspaces w USING(workspace_id)",
            "x.workspace_id::text",
            "w.root_uri",
            "x.member_prefix",
            "x.publication_id::uuid",
        ),
        "publications" => (
            "JOIN pse_ops.workspaces w USING(workspace_id) JOIN pse_ops.publication_intents i USING(publication_id)",
            "x.workspace_id::text",
            "w.root_uri",
            "i.member_prefix",
            "x.publication_id::uuid",
        ),
        "publication_members" | "publication_windows" | "reader_leases" | "retention_marks" => (
            "JOIN pse_ops.publications p USING(publication_id) JOIN pse_ops.workspaces w ON w.workspace_id=p.workspace_id JOIN pse_ops.publication_intents i USING(publication_id)",
            "p.workspace_id::text",
            "w.root_uri",
            "i.member_prefix",
            "x.publication_id::uuid",
        ),
        "publication_heads" | "orphan_scans" => (
            "JOIN pse_ops.workspaces w USING(workspace_id)",
            "x.workspace_id::text",
            "w.root_uri",
            "NULL::text",
            "NULL::uuid",
        ),
        "orphan_candidates" => (
            "JOIN pse_ops.orphan_scans s USING(scan_id)",
            "s.workspace_id::text",
            "s.root_uri",
            "x.prefix",
            "NULL::uuid",
        ),
        "retired_inventory" => (
            "",
            "x.workspace_id::text",
            "x.root_uri",
            "x.prefix",
            "NULL::uuid",
        ),
        _ => ("", "NULL::text", "NULL::text", "NULL::text", "NULL::uuid"),
    };
    if matches!(table, "publication_members" | "publication_windows") {
        prefix = "x.table_uri";
    }
    if table == "publication_heads" {
        prefix = "(SELECT i.member_prefix FROM pse_ops.publication_intents i WHERE i.publication_id=x.publication_id)";
    }
    let protections = if table == "retired_inventory" {
        "x.protections".to_owned()
    } else {
        format!(
            "COALESCE((SELECT jsonb_agg(to_jsonb(l) ORDER BY l.lease_id) FROM pse_ops.reader_leases l WHERE l.publication_id={publication} AND l.released_at IS NULL),'[]'::jsonb)"
        )
    };
    format!(
        "SELECT to_jsonb(x),{workspace},{root},{prefix},{protections} FROM pse_ops.{table} x {join} ORDER BY to_jsonb(x)::text"
    )
}
async fn inventory(
    client: &tokio_postgres::Client,
    target: &Target,
    reset_id: ResetId,
    max_rows: u64,
    destination: Option<&Path>,
    mut output: Option<&mut File>,
) -> Result<(ContentHash, u64), OperationsError> {
    let bytes = encoded(&ManifestLine::Header(header(reset_id)))?;
    let mut hash = FramedHasher::new(Frame::ResetManifestV1);
    hash.part(&bytes);
    if let Some(file) = output.as_mut() {
        file.write_all(&bytes).map_err(io)?;
    }
    let mut ordinal = 0u64;
    for table in TABLES {
        let exists: bool = client
            .query_one(
                "SELECT to_regclass($1) IS NOT NULL",
                &[&format!("pse_ops.{table}")],
            )
            .await
            .classify(target)?
            .try_get(0)
            .classify(target)?;
        if !exists {
            continue;
        }
        client
            .batch_execute(&format!(
                "DECLARE reset_inventory_cursor NO SCROLL CURSOR FOR {}",
                export_query(table)
            ))
            .await
            .classify(target)?;
        loop {
            let rows = client
                .query(&format!("FETCH {PAGE} FROM reset_inventory_cursor"), &[])
                .await
                .classify(target)?;
            if rows.is_empty() {
                break;
            }
            for row in rows {
                if ordinal >= max_rows {
                    return Err(invalid(
                        "complete retirement inventory exceeds explicit row bound",
                    ));
                }
                let mut record = InventoryRecord {
                    table: (*table).to_owned(),
                    ordinal,
                    document: row.try_get(0).classify(target)?,
                    workspace_id: row.try_get(1).classify(target)?,
                    root_uri: row.try_get(2).classify(target)?,
                    prefix: row.try_get(3).classify(target)?,
                    protections: row.try_get(4).classify(target)?,
                    disposition: OrphanDisposition::Unresolved,
                };
                record.disposition = fresh_retirement_disposition(
                    table,
                    &record.document,
                    record.prefix.as_deref(),
                    &record.protections,
                );
                if let Some(path) = destination {
                    for root in [
                        record.root_uri.as_deref(),
                        record.prefix.as_deref(),
                        record.document.get("table_uri").and_then(Value::as_str),
                    ]
                    .into_iter()
                    .flatten()
                    {
                        outside(path, root)?;
                    }
                }
                let bytes = encoded(&ManifestLine::Record(record))?;
                hash.part(&bytes);
                if let Some(file) = output.as_mut() {
                    file.write_all(&bytes).map_err(io)?;
                }
                ordinal += 1;
            }
        }
        client
            .batch_execute("CLOSE reset_inventory_cursor")
            .await
            .classify(target)?;
    }
    let digest = hash.finish_hash();
    if let Some(file) = output.as_mut() {
        file.write_all(&encoded(&ManifestLine::Complete {
            rows: ordinal,
            digest: digest.to_hex(),
        })?)
        .map_err(io)?;
    }
    Ok((digest, ordinal))
}
fn read_line(reader: &mut BufReader<File>) -> Result<Option<Vec<u8>>, OperationsError> {
    let mut line = Vec::new();
    // Take bounds allocation before parsing an untrusted external record.
    use std::io::Read;
    let count = reader
        .take(ROW_BYTES_MAX + 1)
        .read_until(b'\n', &mut line)
        .map_err(io)?;
    if count == 0 {
        return Ok(None);
    }
    if count as u64 > ROW_BYTES_MAX || line.last() != Some(&b'\n') {
        return Err(invalid("manifest record is oversized or truncated"));
    }
    Ok(Some(line))
}
fn verify_manifest(receipt: &ResetManifestReceipt) -> Result<(), OperationsError> {
    bound(receipt.max_rows)?;
    let mut reader = BufReader::new(File::open(&receipt.destination).map_err(io)?);
    let bytes = read_line(&mut reader)?.ok_or_else(|| invalid("manifest header missing"))?;
    let ManifestLine::Header(header) = serde_json::from_slice(&bytes).map_err(io)? else {
        return Err(invalid("manifest does not begin with header"));
    };
    if header.version != 1
        || header.reset_id != receipt.reset_id.to_string()
        || header.source != receipt.source_fingerprint
    {
        return Err(invalid(
            "manifest header conflicts with exact reset receipt",
        ));
    }
    let mut hash = FramedHasher::new(Frame::ResetManifestV1);
    hash.part(&bytes);
    let mut count = 0u64;
    loop {
        let bytes =
            read_line(&mut reader)?.ok_or_else(|| invalid("completed manifest trailer missing"))?;
        match serde_json::from_slice(&bytes).map_err(io)? {
            ManifestLine::Record(record) => {
                if count >= receipt.max_rows
                    || record.ordinal != count
                    || !TABLES.contains(&record.table.as_str())
                    || !record.protections.is_array()
                {
                    return Err(invalid("invalid or oversized retirement inventory record"));
                }
                if record.disposition
                    != fresh_retirement_disposition(
                        &record.table,
                        &record.document,
                        record.prefix.as_deref(),
                        &record.protections,
                    )
                {
                    return Err(invalid(
                        "retirement attribution/protection disposition is inconsistent",
                    ));
                }
                hash.part(&bytes);
                count += 1;
            }
            ManifestLine::Complete { rows, digest } => {
                let actual = hash.finish_hash();
                if rows != count
                    || count != receipt.inventory_rows
                    || actual != receipt.manifest_digest
                    || digest != actual.to_hex()
                    || read_line(&mut reader)?.is_some()
                {
                    return Err(invalid(
                        "manifest completion/count/digest conflicts with receipt",
                    ));
                }
                return Ok(());
            }
            ManifestLine::Header(_) => return Err(invalid("manifest contains a repeated header")),
        }
    }
}
impl Store {
    /// Inspect a completed versioned external export without connecting or mutating a store.
    /// # Errors
    /// Malformed, incomplete, oversized or tampered manifest.
    pub fn inspect_reset_manifest(
        path: &Path,
        max_rows: u64,
    ) -> Result<ResetManifestReceipt, OperationsError> {
        bound(max_rows)?;
        let destination = path.canonicalize().map_err(io)?;
        let mut reader = BufReader::new(File::open(&destination).map_err(io)?);
        let first = read_line(&mut reader)?.ok_or_else(|| invalid("manifest header missing"))?;
        let ManifestLine::Header(header) = serde_json::from_slice(&first).map_err(io)? else {
            return Err(invalid("manifest header missing"));
        };
        let reset_id = ResetId::from(pse_ids::SemanticId::parse_hex(&header.reset_id).map_err(io)?);
        let mut rows = 0u64;
        loop {
            let bytes =
                read_line(&mut reader)?.ok_or_else(|| invalid("manifest completion missing"))?;
            match serde_json::from_slice(&bytes).map_err(io)? {
                ManifestLine::Record(_) => {
                    rows += 1;
                    if rows > max_rows {
                        return Err(invalid("manifest exceeds explicit row bound"));
                    }
                }
                ManifestLine::Complete {
                    rows: declared,
                    digest,
                } => {
                    if rows != declared {
                        return Err(invalid("manifest row count mismatch"));
                    }
                    let receipt = ResetManifestReceipt {
                        reset_id,
                        manifest_digest: ContentHash::parse_hex(&digest).map_err(io)?,
                        source_fingerprint: header.source,
                        destination,
                        inventory_rows: rows,
                        max_rows,
                    };
                    verify_manifest(&receipt)?;
                    return Ok(receipt);
                }
                ManifestLine::Header(_) => return Err(invalid("repeated manifest header")),
            }
        }
    }

    /// Export complete catalog/control inventory to a new explicit file outside all erased roots.
    /// A dedicated exclusive namespace lease excludes every active generation during export.
    /// # Errors
    /// Unknown/drifted source, active generation, incomplete/oversized inventory or file failures.
    pub async fn export_reset_inventory(
        &self,
        reset_id: ResetId,
        path: &Path,
        max_rows: u64,
    ) -> Result<ResetManifestReceipt, OperationsError> {
        bound(max_rows)?;
        let destination = destination(path)?;
        let session = exclusive(self).await?;
        crate::schema::verify_ready(&session.client, self.target()).await?;
        session
            .client
            .batch_execute("BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY")
            .await
            .classify(self.target())?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&destination)
            .map_err(io)?;
        let (manifest_digest, inventory_rows) = inventory(
            &session.client,
            self.target(),
            reset_id,
            max_rows,
            Some(&destination),
            Some(&mut file),
        )
        .await?;
        file.sync_all().map_err(io)?;
        session
            .client
            .batch_execute("COMMIT")
            .await
            .classify(self.target())?;
        Ok(ResetManifestReceipt {
            reset_id,
            manifest_digest,
            source_fingerprint: crate::generated::SCHEMA_FINGERPRINT_HEX.to_owned(),
            destination,
            inventory_rows,
            max_rows,
        })
    }
    /// Settle or atomically reset against a completed exact retirement export. The schema DROP,
    /// CREATE, preserved retirement import, reset identity/digest and readiness are one transaction.
    /// # Errors
    /// Source/history drift, mismatched/incomplete export, active generation or driver/file failures.
    pub async fn reset(
        &self,
        receipt: &ResetManifestReceipt,
    ) -> Result<ResetReport, OperationsError> {
        if receipt.destination.canonicalize().map_err(io)? != receipt.destination {
            return Err(invalid(
                "reset manifest path is not an exact canonical external file",
            ));
        }
        verify_manifest(receipt)?;
        let session = exclusive(self).await?;
        crate::schema::verify_ready(&session.client, self.target()).await?;
        if let Some(row)=session.client.query_opt("SELECT manifest_digest,inventory_rows FROM pse_ops.reset_records WHERE reset_id=$1", &[&receipt.reset_id]).await.classify(self.target())? {
            let digest:ContentHash=row.try_get(0).classify(self.target())?;
            let rows:i64=row.try_get(1).classify(self.target())?;
            if digest!=receipt.manifest_digest || u64::try_from(rows).ok()!=Some(receipt.inventory_rows) {return Err(invalid("reset identity reused for another completed manifest"));}
            return Ok(ResetReport {reset_id:receipt.reset_id,manifest_digest:digest,already_applied:true,inventory_rows:receipt.inventory_rows});
        }
        if receipt.source_fingerprint != crate::generated::SCHEMA_FINGERPRINT_HEX {
            return Err(invalid(
                "reset source is not the exact supported build declaration",
            ));
        }
        session
            .client
            .batch_execute("BEGIN ISOLATION LEVEL REPEATABLE READ")
            .await
            .classify(self.target())?;
        let (live_digest, live_rows) = inventory(
            &session.client,
            self.target(),
            receipt.reset_id,
            receipt.max_rows,
            Some(&receipt.destination),
            None,
        )
        .await?;
        if live_digest != receipt.manifest_digest || live_rows != receipt.inventory_rows {
            return Err(invalid(
                "live catalog changed after completed export; export a new reset identity",
            ));
        }
        // All staged rows are bounded and verified again before production schema mutation.
        session.client.batch_execute("CREATE TEMP TABLE reset_inventory_stage(ordinal bigint PRIMARY KEY,kind text NOT NULL,document jsonb NOT NULL,workspace_id uuid,root_uri text,prefix text,protections jsonb NOT NULL,disposition text NOT NULL) ON COMMIT DROP").await.classify(self.target())?;
        let mut reader = BufReader::new(File::open(&receipt.destination).map_err(io)?);
        let _ = read_line(&mut reader)?;
        while let Some(bytes) = read_line(&mut reader)? {
            match serde_json::from_slice(&bytes).map_err(io)? {
                ManifestLine::Record(r) => {
                    let ordinal = i64::try_from(r.ordinal).map_err(io)?;
                    session.client.execute("INSERT INTO reset_inventory_stage VALUES($1,$2,$3,$4::text::uuid,$5,$6,$7,$8)", &[&ordinal,&r.table,&r.document,&r.workspace_id,&r.root_uri,&r.prefix,&r.protections,&r.disposition.as_str()]).await.classify(self.target())?;
                }
                ManifestLine::Complete { .. } => break,
                ManifestLine::Header(_) => return Err(invalid("repeated manifest header")),
            }
        }
        verify_manifest(receipt)?;
        session
            .client
            .batch_execute("DROP SCHEMA pse_ops CASCADE")
            .await
            .classify(self.target())?;
        for sql in [crate::generated::SCHEMA_SQL, crate::schema::PHYSICAL_SQL] {
            session
                .client
                .batch_execute(sql)
                .await
                .classify(self.target())?;
        }
        // Prior unresolved retirement rows and history retain original identities and raw documents.
        for table in ["workspaces", "reset_records", "retired_inventory"] {
            session.client.batch_execute(&format!("INSERT INTO pse_ops.{table} SELECT (jsonb_populate_record(NULL::pse_ops.{table},document)).* FROM reset_inventory_stage WHERE kind='{table}' ORDER BY ordinal")).await.classify(self.target())?;
        }
        let rows = i64::try_from(receipt.inventory_rows).map_err(io)?;
        let uri = receipt.destination.to_string_lossy().to_string();
        session.client.execute("INSERT INTO pse_ops.reset_records(reset_id,manifest_digest,manifest_uri,source_fingerprint,inventory_rows) VALUES($1,$2,$3,$4,$5)", &[&receipt.reset_id,&receipt.manifest_digest,&uri,&receipt.source_fingerprint,&rows]).await.classify(self.target())?;
        session.client.execute("INSERT INTO pse_ops.retired_inventory(reset_id,ordinal,workspace_id,root_uri,prefix,record_kind,document,protections,disposition) SELECT $1,ordinal,workspace_id,root_uri,prefix,kind,document,protections,disposition::pse_ops.orphan_disposition FROM reset_inventory_stage WHERE kind NOT IN ('retired_inventory','reset_records','workspaces') ORDER BY ordinal", &[&receipt.reset_id]).await.classify(self.target())?;
        session.client.batch_execute("INSERT INTO pse_ops.publication_heads(workspace_id,publication_id,advanced_at) SELECT workspace_id,NULL,now() FROM pse_ops.workspaces").await.classify(self.target())?;
        session.client.execute("INSERT INTO pse_ops.schema_support_state(history,shared_version,source,target,ready) VALUES('catalog',$1,'fresh',$2,true),('operations',$1,'fresh',$3,true)", &[&crate::generated::SHARED_VERSION,&crate::generated::CATALOG_FINGERPRINT_HEX,&crate::generated::OPERATIONS_FINGERPRINT_HEX]).await.classify(self.target())?;
        session
            .client
            .batch_execute(crate::generated::RECORD_SQL)
            .await
            .classify(self.target())?;
        crate::schema::verify_ready(&session.client, self.target()).await?;
        session
            .client
            .batch_execute("COMMIT")
            .await
            .classify(self.target())?;
        self.forget_statements();
        Ok(ResetReport {
            reset_id: receipt.reset_id,
            manifest_digest: receipt.manifest_digest,
            already_applied: false,
            inventory_rows: receipt.inventory_rows,
        })
    }
}

#[cfg(test)]
mod retirement_unit {
    use super::*;
    struct Temp(PathBuf);
    impl Temp {
        fn new() -> Self {
            let path =
                std::env::temp_dir().join(format!("pse-reset-unit-{}", uuid::Uuid::now_v7()));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    fn completed(path: &Path) -> ResetManifestReceipt {
        let reset_id = crate::mint_id();
        let h = header(reset_id);
        let record = InventoryRecord {
            table: "publication_intents".into(),
            ordinal: 0,
            document: serde_json::json!({"member_prefix":"file:///owned/member/"}),
            workspace_id: None,
            root_uri: Some("file:///owned/".into()),
            prefix: Some("file:///owned/member/".into()),
            protections: serde_json::json!([]),
            disposition: OrphanDisposition::Discovered,
        };
        let mut file = File::create(path).unwrap();
        let mut hash = FramedHasher::new(Frame::ResetManifestV1);
        for line in [ManifestLine::Header(h), ManifestLine::Record(record)] {
            let bytes = encoded(&line).unwrap();
            hash.part(&bytes);
            file.write_all(&bytes).unwrap();
        }
        let digest = hash.finish_hash();
        file.write_all(
            &encoded(&ManifestLine::Complete {
                rows: 1,
                digest: digest.to_hex(),
            })
            .unwrap(),
        )
        .unwrap();
        ResetManifestReceipt {
            reset_id,
            manifest_digest: digest,
            source_fingerprint: crate::generated::SCHEMA_FINGERPRINT_HEX.into(),
            destination: path.canonicalize().unwrap(),
            inventory_rows: 1,
            max_rows: 100,
        }
    }
    #[test]
    fn retirement_classification_retains_unknown_and_records_deadlines_without_time_dependent_bytes()
     {
        let empty = serde_json::json!([]);
        let prefix = Some("file:///owned/member/");
        assert_eq!(
            fresh_retirement_disposition("publication_intents", &Value::Null, prefix, &empty),
            OrphanDisposition::Discovered
        );
        for table in [
            "publications",
            "publication_members",
            "publication_windows",
            "reader_leases",
            "publication_heads",
            "retention_marks",
        ] {
            assert_eq!(
                fresh_retirement_disposition(table, &Value::Null, prefix, &empty),
                OrphanDisposition::Discovered
            );
        }
        let known = serde_json::json!({"ownership":"attributable"});
        let unknown = serde_json::json!({"ownership":"unattributable"});
        assert_eq!(
            fresh_retirement_disposition("orphan_candidates", &known, prefix, &empty),
            OrphanDisposition::Discovered
        );
        assert_eq!(
            fresh_retirement_disposition("orphan_candidates", &unknown, prefix, &empty),
            OrphanDisposition::Unresolved
        );
        assert_eq!(
            fresh_retirement_disposition(
                "retired_inventory",
                &serde_json::json!({"disposition":"unresolved"}),
                prefix,
                &empty
            ),
            OrphanDisposition::Unresolved
        );
        for expiry in ["2000-01-01T00:00:00Z", "2030-01-01T00:00:00Z"] {
            assert_eq!(
                fresh_retirement_disposition(
                    "publication_members",
                    &Value::Null,
                    prefix,
                    &serde_json::json!([{"expires_at":expiry,"holder":"export:destination"}])
                ),
                OrphanDisposition::Protected
            );
        }
        assert_eq!(
            fresh_retirement_disposition(
                "publication_members",
                &Value::Null,
                prefix,
                &serde_json::json!([{"holder":"unknown"}])
            ),
            OrphanDisposition::Unresolved
        );
        assert_eq!(
            fresh_retirement_disposition("publication_members", &Value::Null, None, &empty),
            OrphanDisposition::Unresolved
        );
    }
    #[test]
    fn completed_manifest_requires_exact_header_count_digest_and_completion() {
        let temp = Temp::new();
        let path = temp.0.join("inventory.jsonl");
        let receipt = completed(&path);
        assert!(verify_manifest(&receipt).is_ok());
        assert_eq!(Store::inspect_reset_manifest(&path, 100).unwrap(), receipt);
        let mut wrong = receipt.clone();
        wrong.inventory_rows = 0;
        assert!(verify_manifest(&wrong).is_err());
        let mut wrong = receipt.clone();
        wrong.reset_id = crate::mint_id();
        assert!(verify_manifest(&wrong).is_err());
        let mut wrong = receipt.clone();
        wrong.manifest_digest = ContentHash::from_bytes([0; 32]);
        assert!(verify_manifest(&wrong).is_err());
        let bytes = std::fs::read(&path).unwrap();
        let truncated = bytes
            .split(|b| *b == b'\n')
            .take(2)
            .flat_map(|line| line.iter().copied().chain(*b"\n"))
            .collect::<Vec<_>>();
        std::fs::write(&path, truncated).unwrap();
        assert!(Store::inspect_reset_manifest(&path, 100).is_err());
        assert!(bound(0).is_err());
        assert!(bound(RESET_ROWS_MAX + 1).is_err());
    }
    #[test]
    fn retirement_destination_stays_outside_existing_and_missing_descendant_roots() {
        let temp = Temp::new();
        let root = temp.0.join("members");
        std::fs::create_dir(&root).unwrap();
        let uri = url::Url::from_directory_path(&root).unwrap().to_string();
        assert!(outside(&root.join("inventory.jsonl"), &uri).is_err());
        assert!(outside(&temp.0.join("inventory.jsonl"), &uri).is_ok());
        #[cfg(unix)]
        {
            let alias = temp.0.join("alias");
            std::os::unix::fs::symlink(&root, &alias).unwrap();
            let uri = url::Url::from_directory_path(alias.join("missing"))
                .unwrap()
                .to_string();
            assert!(outside(&root.join("missing/inventory.jsonl"), &uri).is_err());
        }
    }
}

// These isolated PostgreSQL journeys are authored with the functional work and run in 25k.
#[cfg(test)]
mod retirement_journeys {
    use super::*;
    use crate::inventory::ObservedCandidate;
    use crate::testing::{Fault, FaultPoint, FaultProxy, TestDatabase};
    use pse_model::generated::identities::{ScanId, WorkspaceId};
    const WORKSPACE: &str = "00000000-0000-0000-0000-000000000004";
    const PREFIX: &str = "file:///retirement-owned/members/original/";
    async fn fixture() -> (TestDatabase, Store, PathBuf) {
        let db = TestDatabase::create().await.unwrap();
        let raw = db.session().await.unwrap();
        raw.execute(r#"
          INSERT INTO pse_ops.workspaces(workspace_id,name,root_uri,maintenance_epoch) VALUES('00000000-0000-0000-0000-000000000004','retirement','file:///retirement-owned/',9);
          INSERT INTO pse_ops.attempts(attempt_id,run_id,kind,operational_job_identity,state) VALUES('00000000-0000-0000-0000-000000000001','00000000-0000-0000-0000-000000000002','simulation',decode(repeat('17',32),'hex'),'completed');
          INSERT INTO pse_ops.publication_intents(publication_id,workspace_id,attempt_id,member_prefix) VALUES('00000000-0000-0000-0000-000000000005','00000000-0000-0000-0000-000000000004','00000000-0000-0000-0000-000000000001','file:///retirement-owned/members/original/');
          INSERT INTO pse_ops.publications(publication_id,workspace_id,attempt_id,kind) VALUES('00000000-0000-0000-0000-000000000005','00000000-0000-0000-0000-000000000004','00000000-0000-0000-0000-000000000001','relations');
          INSERT INTO pse_ops.publication_heads(workspace_id,publication_id) VALUES('00000000-0000-0000-0000-000000000004','00000000-0000-0000-0000-000000000005');
          INSERT INTO pse_ops.reader_leases(lease_id,publication_id,holder,expires_at) VALUES('00000000-0000-0000-0000-000000000006','00000000-0000-0000-0000-000000000005','export:original-destination',now()+interval '5 seconds');
          INSERT INTO pse_ops.reset_records(reset_id,manifest_digest,manifest_uri,source_fingerprint,inventory_rows) VALUES('00000000-0000-0000-0000-000000000099',decode(repeat('00',32),'hex'),'/retained/manifest','historical',1);
          INSERT INTO pse_ops.retired_inventory(reset_id,ordinal,workspace_id,root_uri,prefix,record_kind,document,protections,disposition) VALUES('00000000-0000-0000-0000-000000000099',0,'00000000-0000-0000-0000-000000000004','file:///retirement-owned/','file:///retirement-owned/members/unresolved/','publication_intents','{"unproven":true}','[]','unresolved');
        "#).await.unwrap();
        drop(raw);
        db.store().close();
        let admin = Store::connect_with(db.url(), &crate::StoreOptions::for_tests())
            .await
            .unwrap();
        let path =
            std::env::temp_dir().join(format!("pse-reset-journey-{}.jsonl", uuid::Uuid::now_v7()));
        (db, admin, path)
    }
    async fn export(admin: &Store, path: &Path) -> ResetManifestReceipt {
        for _ in 0..100 {
            match admin
                .export_reset_inventory(crate::mint_id(), path, 1000)
                .await
            {
                Ok(receipt) => return receipt,
                Err(OperationsError::MigrationRefused {
                    reason: crate::MigrationRefusal::ActiveGeneration,
                    ..
                }) => tokio::time::sleep(std::time::Duration::from_millis(10)).await,
                result => panic!("{result:?}"),
            }
        }
        panic!("closed generation did not release")
    }
    #[tokio::test]
    async fn reset_preserves_prior_unresolved_and_export_expiry_then_explicit_claim_rechecks() {
        let (db, admin, path) = fixture().await;
        let raw = db.session().await.unwrap();
        let original = raw
            .texts("SELECT to_jsonb(p)::text FROM pse_ops.publications p")
            .await
            .unwrap();
        let receipt = export(&admin, &path).await;
        let report = admin.reset(&receipt).await.unwrap();
        assert!(!report.already_applied);
        assert_eq!(raw.texts("SELECT document::text FROM pse_ops.retired_inventory WHERE reset_id<>'00000000-0000-0000-0000-000000000099' AND record_kind='publications'").await.unwrap(),original);
        // Raw preserved record is exact, not converted into a fresh disposition.
        assert_eq!(raw.count("SELECT count(*) FROM pse_ops.retired_inventory WHERE reset_id='00000000-0000-0000-0000-000000000099' AND disposition='unresolved'").await.unwrap(),1);
        let opened = Store::open_with(db.url(), &crate::StoreOptions::for_tests())
            .await
            .unwrap();
        let catalog = opened.catalog();
        let workspace = WorkspaceId::from(pse_ids::SemanticId::from_bytes(
            *uuid::Uuid::parse_str(WORKSPACE).unwrap().as_bytes(),
        ));
        let id: ScanId = crate::mint_id();
        let checkpoint = catalog.begin_orphan_scan(id, workspace).await.unwrap();
        catalog
            .record_orphan_page(
                id,
                checkpoint.generation,
                &[
                    ObservedCandidate {
                        prefix: PREFIX.into(),
                    },
                    ObservedCandidate {
                        prefix: "file:///retirement-owned/members/unresolved/".into(),
                    },
                    ObservedCandidate {
                        prefix: "file:///retirement-owned/members/unknown/".into(),
                    },
                ],
                true,
            )
            .await
            .unwrap();
        assert!(catalog.claim_orphan(id, PREFIX).await.is_err());
        assert!(
            catalog
                .claim_orphan(id, "file:///retirement-owned/members/unresolved/")
                .await
                .is_err()
        );
        tokio::time::sleep(std::time::Duration::from_secs(6)).await;
        let restarted = catalog.restart_orphan_scan(id).await.unwrap();
        assert_eq!(restarted.generation, checkpoint.generation + 1);
        assert!(
            catalog
                .record_orphan_page(id, checkpoint.generation, &[], true)
                .await
                .is_err()
        );
        catalog
            .record_orphan_page(
                id,
                restarted.generation,
                &[ObservedCandidate {
                    prefix: PREFIX.into(),
                }],
                true,
            )
            .await
            .unwrap();
        let claim = catalog.claim_orphan(id, PREFIX).await.unwrap();
        assert_eq!(claim.workspace_id(), workspace);
        claim.complete_deleted().await.unwrap();
        assert!(
            catalog
                .claim_orphan(id, "file:///retirement-owned/members/unresolved/")
                .await
                .is_err()
        );
        let candidates = catalog.orphan_candidates(id, None, 10).await.unwrap();
        assert_eq!(candidates.candidates.len(), 3);
        assert!(
            candidates
                .candidates
                .iter()
                .any(|c| c.prefix.ends_with("unknown/")
                    && c.ownership == pse_model::generated::enums::OrphanOwnership::Unattributable)
        );
        opened.close();
        drop(raw);
        db.remove().await.unwrap();
        std::fs::remove_file(path).unwrap();
    }
    #[tokio::test]
    async fn foreign_workspace_live_intent_protects_overlapping_owned_candidate() {
        let (db, _admin, path) = fixture().await;
        let raw = db.session().await.unwrap();
        raw.execute(r#"
          UPDATE pse_ops.publication_intents SET abandoned_at=clock_timestamp();
          DELETE FROM pse_ops.publication_heads;
          DELETE FROM pse_ops.reader_leases;
          DELETE FROM pse_ops.publications;
          INSERT INTO pse_ops.workspaces(workspace_id,name,root_uri,maintenance_epoch) VALUES('00000000-0000-0000-0000-000000000044','foreign','file:///retirement-owned/members/original/',0);
          INSERT INTO pse_ops.publication_intents(publication_id,workspace_id,attempt_id,member_prefix) VALUES('00000000-0000-0000-0000-000000000055','00000000-0000-0000-0000-000000000044','00000000-0000-0000-0000-000000000001','file:///retirement-owned/members/original/foreign/');
        "#).await.unwrap();
        let opened = Store::open_with(db.url(), &crate::StoreOptions::for_tests())
            .await
            .unwrap();
        let catalog = opened.catalog();
        let workspace = WorkspaceId::from(pse_ids::SemanticId::from_bytes(
            *uuid::Uuid::parse_str(WORKSPACE).unwrap().as_bytes(),
        ));
        let id: ScanId = crate::mint_id();
        let checkpoint = catalog.begin_orphan_scan(id, workspace).await.unwrap();
        catalog
            .record_orphan_page(
                id,
                checkpoint.generation,
                &[ObservedCandidate {
                    prefix: PREFIX.into(),
                }],
                true,
            )
            .await
            .unwrap();
        assert!(catalog.claim_orphan(id, PREFIX).await.is_err());
        raw.execute("UPDATE pse_ops.publication_intents SET abandoned_at=clock_timestamp() WHERE publication_id='00000000-0000-0000-0000-000000000055'").await.unwrap();
        catalog
            .claim_orphan(id, PREFIX)
            .await
            .unwrap()
            .complete_deleted()
            .await
            .unwrap();
        opened.close();
        drop(raw);
        db.remove().await.unwrap();
        assert!(!path.exists());
    }
    #[tokio::test]
    async fn reset_failed_and_lost_commit_ack_preserve_or_settle_before_repeat_drop() {
        for point in [FaultPoint::BeforeCommit, FaultPoint::AfterCommit] {
            let (db, admin, path) = fixture().await;
            let raw = db.session().await.unwrap();
            let receipt = export(&admin, &path).await;
            let before = raw
                .texts("SELECT to_jsonb(p)::text FROM pse_ops.publications p")
                .await
                .unwrap();
            let proxy = FaultProxy::start(db.url()).await.unwrap();
            let cut = Store::connect_with(proxy.url(), &crate::StoreOptions::for_tests())
                .await
                .unwrap();
            proxy.arm(Fault {
                marker: "DROP SCHEMA pse_ops CASCADE",
                point,
            });
            assert!(cut.reset(&receipt).await.is_err());
            assert!(proxy.fired());
            cut.close();
            proxy.shutdown().await.unwrap();
            if point == FaultPoint::BeforeCommit {
                assert_eq!(
                    raw.texts("SELECT to_jsonb(p)::text FROM pse_ops.publications p")
                        .await
                        .unwrap(),
                    before
                );
            }
            let settled = admin.reset(&receipt).await.unwrap();
            assert_eq!(settled.already_applied, point == FaultPoint::AfterCommit);
            assert!(admin.reset(&receipt).await.unwrap().already_applied);
            assert_eq!(
                raw.count("SELECT count(*) FROM pse_ops.reset_records")
                    .await
                    .unwrap(),
                2
            );
            drop(raw);
            db.remove().await.unwrap();
            std::fs::remove_file(path).unwrap();
        }
    }
    #[tokio::test]
    async fn reset_bound_incomplete_manifest_and_unknown_source_refuse_without_drop() {
        let (db, admin, path) = fixture().await;
        let receipt = export(&admin, &path).await;
        let raw = db.session().await.unwrap();
        let before = raw
            .texts("SELECT to_jsonb(p)::text FROM pse_ops.publications p")
            .await
            .unwrap();
        let too_small = path.with_extension("incomplete.jsonl");
        assert!(
            admin
                .export_reset_inventory(crate::mint_id(), &too_small, 1)
                .await
                .is_err()
        );
        assert!(Store::inspect_reset_manifest(&too_small, 100).is_err());
        let mut wrong = receipt.clone();
        wrong.inventory_rows += 1;
        assert!(admin.reset(&wrong).await.is_err());
        raw.execute("COMMENT ON SCHEMA pse_ops IS 'pse.ops.schema.v1 unknown'")
            .await
            .unwrap();
        assert!(admin.reset(&receipt).await.is_err());
        assert_eq!(
            raw.texts("SELECT to_jsonb(p)::text FROM pse_ops.publications p")
                .await
                .unwrap(),
            before
        );
        drop(raw);
        db.remove().await.unwrap();
        std::fs::remove_file(path).unwrap();
        std::fs::remove_file(too_small).unwrap();
    }
    #[tokio::test]
    async fn reader_renewal_queued_before_expiry_cannot_resurrect_after_global_claim_fence() {
        let (db, _admin, path) = fixture().await;
        let opened = Store::open_with(db.url(), &crate::StoreOptions::for_tests())
            .await
            .unwrap();
        let raw = db.session().await.unwrap();
        raw.execute(
            "UPDATE pse_ops.reader_leases SET expires_at=clock_timestamp()+interval '1 second'",
        )
        .await
        .unwrap();
        raw.execute(&format!(
            "SELECT pg_advisory_lock({})",
            crate::catalog::PROTECTION_LOCK
        ))
        .await
        .unwrap();
        let lease =
            pse_model::generated::identities::ReaderLeaseId::from(pse_ids::SemanticId::from_bytes(
                *uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000006")
                    .unwrap()
                    .as_bytes(),
            ));
        let renewed = tokio::spawn({
            let opened = opened.clone();
            async move {
                opened
                    .catalog()
                    .renew_reader_lease(lease, std::time::Duration::from_secs(10))
                    .await
            }
        });
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        assert!(!renewed.is_finished());
        raw.execute(&format!(
            "SELECT pg_advisory_unlock({})",
            crate::catalog::PROTECTION_LOCK
        ))
        .await
        .unwrap();
        assert!(matches!(
            renewed.await.unwrap(),
            Err(OperationsError::ReaderLeaseLapsed { .. })
        ));
        assert_eq!(
            raw.count(
                "SELECT count(*) FROM pse_ops.reader_leases WHERE expires_at>clock_timestamp()"
            )
            .await
            .unwrap(),
            0
        );
        opened.close();
        drop(raw);
        db.remove().await.unwrap();
        assert!(!path.exists());
    }
}
