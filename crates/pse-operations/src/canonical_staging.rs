// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Leased immutable source staging. Scientific payload blocks travel separately
//! from the closed metadata manifest activated by the head compare-and-set.

use crate::{
    canonical::{
        CanonicalError, CanonicalStore, ObjectEdit, PROTECTED_BEGIN, ProtectedSelection,
        bounded_query, protected_query, request,
    },
    canonical_codec,
    generated::surreal as wire,
};
use pse_ids::{Frame, derive_hash};
use pse_model::generated::runtime::{
    canonical_revisions::Row as Revision, canonical_versions::Row as ObjectVersion,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use surrealdb::types::{Object, QueryError, Value};

/// Each native request/response carries one source block below the protocol limit.
pub const SOURCE_BLOCK_BYTES: usize = 512 * 1024;
// Source payloads are separately blocked; edit count is not the wire byte limit.
// The closed manifest is an internal authoring inventory, never one wire payload.
// Its digest travels alone; per-object metadata, edges and blocks remain separate.
pub(crate) const MAX_EDITS: usize = 8_192;
const MANIFEST_BYTES: usize = 32 * 1024 * 1024;
pub(crate) const IDENTITY_BYTES: usize = 4096;
const RETRIES: usize = 8;
const STAGE_LIFETIME: Duration = Duration::from_secs(300);
// Four small edits, each with at most sixteen bounded references and 64 KiB
// of bytes, fit below the 3 MiB submission budget even at the identity limit.
// They use the same immutable guards and declarations as separately blocked
// versions, but share one durable transaction and transport round trip.
const SMALL_STAGE_EDITS: usize = 4;
const SMALL_STAGE_BYTES: usize = 64 * 1024;
const SMALL_STAGE_REFERENCES: usize = 16;

pub(crate) fn bounded_edit_sql(query: &str) -> String {
    query
        .replace("/* EDIT_SCAN_LIMIT */", &(MAX_EDITS + 1).to_string())
        .replace("/* EDIT_LIMIT */", &MAX_EDITS.to_string())
}

#[derive(Clone, Debug, Serialize)]
struct VersionDescription {
    key: String,
    logical: String,
    kind: String,
    interpretation: String,
    payload_digest: String,
    payload_len: u64,
    block_count: u64,
    reference_count: u64,
    content_digest: String,
}

#[derive(Clone, Debug, Serialize)]
struct ChangeDescription {
    logical: String,
    scope: String,
    name: String,
    version: Option<VersionDescription>,
}

struct EditPlan {
    changes: Vec<ChangeDescription>,
    request: Vec<u8>,
    request_digest: String,
}

/// Only a store-closed manifest may activate memberships. Its generation fences
/// handles from before a resume or abandoned-stage reclamation decision.
#[derive(Clone, Debug)]
pub(crate) struct ClosedStage {
    pub(crate) problem: String,
    pub(crate) expected: Option<String>,
    pub(crate) operation: String,
    pub(crate) generation: u64,
    pub(crate) request: Vec<u8>,
    pub(crate) request_digest: String,
    lifetime_micros: i64,
}

pub(crate) const PRODUCT_BLOB_KIND: &str = "pse.product-blob.v1";
const PRODUCT_BLOB_LIMIT: usize = 128 * 1024 * 1024;
const PRODUCT_PAYLOAD_LIMIT: usize = 65 * 1024 * 1024 + 4096;
const PRODUCT_DEPENDENCY_LIMIT: usize = 32 * 1024 * 1024;
const PRODUCT_BLOB_MAGIC: &[u8] = b"pse.product-blob.v1\0";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProductBlob {
    frame: String,
    pub(crate) version: String,
    pub(crate) digest: String,
    pub(crate) bytes: usize,
    pub(crate) payload_bytes: usize,
    pub(crate) dependency_bytes: usize,
}
impl ProductBlob {
    pub(crate) fn decode(bytes: &[u8]) -> Result<Self, CanonicalError> {
        if bytes.len() > 4096 {
            return Err(CanonicalError::PayloadLimit);
        }
        let value: Self = serde_json::from_slice(bytes).map_err(|error| {
            CanonicalError::Configuration(format!("product descriptor: {error}"))
        })?;
        let expected = PRODUCT_BLOB_MAGIC
            .len()
            .checked_add(16)
            .and_then(|n| n.checked_add(value.payload_bytes))
            .and_then(|n| n.checked_add(value.dependency_bytes));
        if value.frame != PRODUCT_BLOB_KIND
            || expected != Some(value.bytes)
            || value.bytes > PRODUCT_BLOB_LIMIT
            || value.payload_bytes > PRODUCT_PAYLOAD_LIMIT
            || value.dependency_bytes > PRODUCT_DEPENDENCY_LIMIT
            || value.version.len() != 64
            || value.digest.len() != 64
            || !value
                .version
                .bytes()
                .chain(value.digest.bytes())
                .all(|c| c.is_ascii_hexdigit())
        {
            return Err(CanonicalError::Configuration(
                "product blob descriptor invalid".into(),
            ));
        }
        Ok(value)
    }
    pub(crate) fn retained_bytes(&self) -> Result<usize, CanonicalError> {
        // Blob assembly, returned payload/dependency copies, and conservative JSON
        // syntax/string/tree expansion remain simultaneously charged by the caller.
        // The fixed 64 MiB allowance covers one 64-row flat membership page
        // under the 4 MiB protocol limit, SDK/value/typed-row copies, product
        // request bindings and the two block buffers even for an empty old scope.
        self.bytes
            .checked_add(self.payload_bytes)
            .and_then(|n| n.checked_add(self.dependency_bytes.checked_mul(64)?))
            .and_then(|n| n.checked_add(64 * 1024 * 1024))
            .ok_or(CanonicalError::PayloadLimit)
    }
    pub(crate) fn split(&self, bytes: &[u8]) -> Result<(Vec<u8>, Vec<u8>), CanonicalError> {
        let header = PRODUCT_BLOB_MAGIC.len();
        if bytes.len() != self.bytes
            || !bytes.starts_with(PRODUCT_BLOB_MAGIC)
            || payload_digest(bytes) != self.digest
            || derive_hash(Frame::CanonicalProductV1, &[bytes]).to_hex() != self.version
            || bytes.get(header..header + 8)
                != Some((self.payload_bytes as u64).to_le_bytes().as_slice())
            || bytes.get(header + 8..header + 16)
                != Some((self.dependency_bytes as u64).to_le_bytes().as_slice())
        {
            return Err(CanonicalError::Configuration(
                "product blob frame/digest mismatch".into(),
            ));
        }
        let start = header + 16;
        Ok((
            bytes[start..start + self.payload_bytes].to_vec(),
            bytes[start + self.payload_bytes..].to_vec(),
        ))
    }
}

pub(crate) fn product_blob(
    payload: &[u8],
    dependencies: &[u8],
) -> Result<(ProductBlob, ObjectEdit), CanonicalError> {
    if payload.len() > PRODUCT_PAYLOAD_LIMIT || dependencies.len() > PRODUCT_DEPENDENCY_LIMIT {
        return Err(CanonicalError::PayloadLimit);
    }
    let mut bytes =
        Vec::with_capacity(PRODUCT_BLOB_MAGIC.len() + 16 + payload.len() + dependencies.len());
    bytes.extend_from_slice(PRODUCT_BLOB_MAGIC);
    bytes.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    bytes.extend_from_slice(&(dependencies.len() as u64).to_le_bytes());
    bytes.extend_from_slice(payload);
    bytes.extend_from_slice(dependencies);
    let descriptor = ProductBlob {
        frame: PRODUCT_BLOB_KIND.into(),
        version: derive_hash(Frame::CanonicalProductV1, &[&bytes]).to_hex(),
        digest: payload_digest(&bytes),
        bytes: bytes.len(),
        payload_bytes: payload.len(),
        dependency_bytes: dependencies.len(),
    };
    let edit = ObjectEdit {
        logical: PRODUCT_BLOB_KIND.into(),
        scope: PRODUCT_BLOB_KIND.into(),
        name: PRODUCT_BLOB_KIND.into(),
        references: vec![],
        version: Some(ObjectVersion {
            key: descriptor.version.clone(),
            logical: PRODUCT_BLOB_KIND.into(),
            kind: PRODUCT_BLOB_KIND.into(),
            interpretation: wire::INTERPRETATION.into(),
            payload: bytes.into(),
        }),
    };
    Ok((descriptor, edit))
}

pub(crate) enum StageOutcome {
    Acknowledged(Revision),
    Closed(ClosedStage),
}

fn payload_digest(bytes: &[u8]) -> String {
    derive_hash(Frame::CanonicalPayloadV1, &[bytes]).to_hex()
}

fn plan(
    problem: &str,
    expected: Option<&str>,
    edits: &[ObjectEdit],
) -> Result<EditPlan, CanonicalError> {
    if edits.len() > MAX_EDITS
        || problem.len() > IDENTITY_BYTES
        || expected.is_some_and(|value| value.len() > IDENTITY_BYTES)
    {
        return Err(CanonicalError::PayloadLimit);
    }
    let mut logicals = std::collections::BTreeSet::new();
    let mut changes = Vec::with_capacity(edits.len());
    let mut metadata_bytes = 0_usize;
    for edit in edits {
        if [&edit.logical, &edit.scope, &edit.name]
            .iter()
            .any(|value| value.len() > IDENTITY_BYTES)
            || edit
                .references
                .iter()
                .any(|(scope, name)| scope.len() > IDENTITY_BYTES || name.len() > IDENTITY_BYTES)
        {
            return Err(CanonicalError::PayloadLimit);
        }
        if !logicals.insert(&edit.logical) || edit.logical.is_empty() || edit.name.is_empty() {
            return Err(CanonicalError::Configuration(
                "unique logical IDs and nonempty names required".into(),
            ));
        }
        if edit.version.is_none() && !edit.references.is_empty() {
            return Err(CanonicalError::Configuration(
                "removed objects cannot introduce structural references".into(),
            ));
        }
        let references = serde_json::to_vec(&edit.references)
            .map_err(|error| CanonicalError::Configuration(error.to_string()))?;
        metadata_bytes = metadata_bytes.saturating_add(references.len());
        let version = edit
            .version
            .as_ref()
            .map(|version| describe(version, edit, &references))
            .transpose()?;
        let change = ChangeDescription {
            logical: edit.logical.clone(),
            scope: edit.scope.clone(),
            name: edit.name.clone(),
            version,
        };
        metadata_bytes = metadata_bytes.saturating_add(
            serde_json::to_vec(&change)
                .map_err(|error| CanonicalError::Configuration(error.to_string()))?
                .len(),
        );
        if metadata_bytes > MANIFEST_BYTES {
            return Err(CanonicalError::PayloadLimit);
        }
        changes.push(change);
    }
    let encoded = serde_json::to_vec(&(problem, expected, &changes))
        .map_err(|error| CanonicalError::Configuration(error.to_string()))?;
    if encoded.len() > MANIFEST_BYTES {
        return Err(CanonicalError::PayloadLimit);
    }
    let hash = derive_hash(
        Frame::CanonicalEditV1,
        &[wire::INTERPRETATION.as_bytes(), &encoded],
    );
    Ok(EditPlan {
        changes,
        request: hash.as_bytes().to_vec(),
        request_digest: hash.to_hex(),
    })
}

fn conflict(error: &CanonicalError) -> bool {
    matches!(error, CanonicalError::Driver(error) if matches!(error.query_details(), Some(QueryError::TransactionConflict)))
}

// Replaying a staging transaction repeats immutable identities and exact values;
// its live generation/expiry fences are checked again inside the transaction.
// gRPC also maps some HTTP/2 transport failures to its generic internal domain.
fn retryable_stage(error: &CanonicalError) -> bool {
    conflict(error)
        || matches!(error, CanonicalError::Driver(error) if error.is_connection() || error.is_internal())
}

fn uint(value: usize) -> Result<Value, CanonicalError> {
    Ok(canonical_codec::encode_uint(
        u64::try_from(value).map_err(|_| CanonicalError::PayloadLimit)?,
    )?)
}

fn block_key(version: &str, ordinal: u64) -> String {
    format!("{version}:{ordinal}")
}
fn edge_key(version: &str, ordinal: u64) -> String {
    format!("{version}:{ordinal}")
}

fn metadata_bindings(
    token: &ClosedStage,
    ordinal: usize,
    change: &ChangeDescription,
) -> Result<Vec<(&'static str, Value)>, CanonicalError> {
    let row = pse_model::generated::runtime::canonical_staged_edits::Row {
        key: format!("{}:{ordinal}", token.operation),
        stage: token.operation.clone(),
        ordinal: u64::try_from(ordinal).map_err(|_| CanonicalError::PayloadLimit)?,
        logical: change.logical.clone(),
        scope: change.scope.clone(),
        name: change.name.clone(),
        version: change.version.as_ref().map(|version| version.key.clone()),
    };
    let mut bindings = vec![(
        "edit",
        Value::Object(wire::encode_canonical_staged_edits(&row)?),
    )];
    if let Some(version) = &change.version {
        let manifest = pse_model::generated::runtime::canonical_version_manifests::Row {
            key: version.key.clone(),
            logical: version.logical.clone(),
            kind: version.kind.clone(),
            interpretation: version.interpretation.clone(),
            payload_digest: version.payload_digest.clone(),
            payload_len: version.payload_len,
            block_count: version.block_count,
            reference_count: version.reference_count,
            creator_stage: token.operation.clone(),
            closed: false,
        };
        let receipt = pse_model::generated::runtime::canonical_version_receipts::Row {
            key: version.key.clone(),
            content_digest: version.content_digest.clone(),
        };
        bindings.push((
            "manifest",
            Value::Object(wire::encode_canonical_version_manifests(&manifest)?),
        ));
        bindings.push((
            "receipt",
            Value::Object(wire::encode_canonical_version_receipts(&receipt)?),
        ));
    }
    Ok(bindings)
}

fn source_block(version: &str, ordinal: u64, payload: &[u8]) -> Result<Value, CanonicalError> {
    let row = pse_model::generated::runtime::canonical_payload_blocks::Row {
        key: block_key(version, ordinal),
        version: version.into(),
        ordinal,
        payload: payload.to_vec().into(),
        digest: payload_digest(payload),
    };
    Ok(Value::Object(wire::encode_canonical_payload_blocks(&row)?))
}

fn source_edge(
    version: &str,
    ordinal: u64,
    scope: &str,
    name: &str,
) -> Result<Value, CanonicalError> {
    let row = pse_model::generated::runtime::canonical_edges::Row {
        key: edge_key(version, ordinal),
        source_version: version.into(),
        ordinal,
        target_scope: scope.into(),
        target_name: name.into(),
    };
    Ok(Value::Object(wire::encode_canonical_edges(&row)?))
}

fn staging_item(
    token: &ClosedStage,
    ordinal: usize,
    change: &ChangeDescription,
    edit: &ObjectEdit,
) -> Result<Option<Value>, CanonicalError> {
    if edit
        .version
        .as_ref()
        .is_some_and(|version| version.payload.len() > SMALL_STAGE_BYTES)
        || edit.references.len() > SMALL_STAGE_REFERENCES
    {
        return Ok(None);
    }
    let mut item = Object::new();
    for (key, value) in metadata_bindings(token, ordinal, change)? {
        item.insert(key, value);
    }
    if let Some(version) = &edit.version {
        let blocks = if version.payload.is_empty() {
            Vec::new()
        } else {
            vec![source_block(&version.key, 0, version.payload.as_slice())?]
        };
        let edges = edit
            .references
            .iter()
            .enumerate()
            .map(|(index, (scope, name))| {
                source_edge(
                    &version.key,
                    u64::try_from(index).map_err(|_| CanonicalError::PayloadLimit)?,
                    scope,
                    name,
                )
            })
            .collect::<Result<Vec<_>, CanonicalError>>()?;
        item.insert("blocks", Value::Array(blocks.into()));
        item.insert("edges", Value::Array(edges.into()));
    }
    Ok(Some(Value::Object(item)))
}

impl CanonicalStore {
    pub(crate) async fn stage_edits(
        &self,
        problem: &str,
        expected: Option<&str>,
        operation: &str,
        edits: &[ObjectEdit],
    ) -> Result<StageOutcome, CanonicalError> {
        if edits.iter().any(|edit| {
            edit.version
                .as_ref()
                .is_some_and(|version| version.kind == PRODUCT_BLOB_KIND)
        }) {
            return Err(CanonicalError::Configuration(
                "reserved product blobs cannot author source revisions".into(),
            ));
        }
        let plan = plan(problem, expected, edits)?;
        if let Some(revision) = self.revision(operation).await? {
            return if revision.request.as_slice() == plan.request.as_slice() {
                Ok(StageOutcome::Acknowledged(revision))
            } else {
                Err(CanonicalError::OperationReused)
            };
        }
        let token = match self
            .begin_stage(problem, expected, operation, &plan, STAGE_LIFETIME)
            .await
        {
            Ok(token) => token,
            Err(error) => {
                if let Ok(Some(revision)) = self.revision(operation).await {
                    if revision.request.as_slice() == plan.request.as_slice() {
                        return Ok(StageOutcome::Acknowledged(revision));
                    }
                    return Err(CanonicalError::OperationReused);
                }
                return Err(error);
            }
        };
        self.stage_plan(&token, &plan, edits).await?;
        Ok(StageOutcome::Closed(token))
    }

    pub(crate) async fn stage_product_blob(
        &self,
        problem: &str,
        operation: &str,
        edit: ObjectEdit,
    ) -> Result<ClosedStage, CanonicalError> {
        let edits = [edit];
        let plan = plan(problem, None, &edits)?;
        let token = self
            .begin_stage_inner(problem, None, operation, &plan, STAGE_LIFETIME, true)
            .await?;
        self.stage_plan(&token, &plan, &edits).await?;
        Ok(token)
    }

    async fn stage_plan(
        &self,
        token: &ClosedStage,
        plan: &EditPlan,
        edits: &[ObjectEdit],
    ) -> Result<(), CanonicalError> {
        let mut batch = Vec::with_capacity(SMALL_STAGE_EDITS);
        for (ordinal, (change, edit)) in plan.changes.iter().zip(edits).enumerate() {
            if let Some(item) = staging_item(token, ordinal, change, edit)? {
                batch.push(item);
                if batch.len() == SMALL_STAGE_EDITS {
                    self.stage_small_edits(token, std::mem::take(&mut batch))
                        .await?;
                }
                continue;
            }
            if !batch.is_empty() {
                self.stage_small_edits(token, std::mem::take(&mut batch))
                    .await?;
            }
            self.stage_metadata(token, ordinal, change).await?;
            if let (Some(description), Some(version)) = (&change.version, &edit.version) {
                self.stage_version(token, description, version, &edit.references)
                    .await?;
            }
        }
        if !batch.is_empty() {
            self.stage_small_edits(token, batch).await?;
        }
        self.stage_query(token, CLOSE_STAGE, Vec::new()).await?;
        Ok(())
    }

    async fn begin_stage(
        &self,
        problem: &str,
        expected: Option<&str>,
        operation: &str,
        plan: &EditPlan,
        lifetime: Duration,
    ) -> Result<ClosedStage, CanonicalError> {
        self.begin_stage_inner(problem, expected, operation, plan, lifetime, false)
            .await
    }

    async fn begin_stage_inner(
        &self,
        problem: &str,
        expected: Option<&str>,
        operation: &str,
        plan: &EditPlan,
        lifetime: Duration,
        product: bool,
    ) -> Result<ClosedStage, CanonicalError> {
        let lifetime_micros =
            i64::try_from(lifetime.as_micros()).map_err(|_| CanonicalError::PayloadLimit)?;
        if lifetime_micros <= 0 || problem.is_empty() || operation.is_empty() {
            return Err(CanonicalError::Configuration(
                "positive staging lease and nonempty identities required".into(),
            ));
        }
        if operation.len() > IDENTITY_BYTES {
            return Err(CanonicalError::PayloadLimit);
        }
        for attempt in 0..RETRIES {
            self.ensure_writes()?;
            let result = bounded_query(
                self.db
                    .query(BEGIN_STAGE)
                    .bind(("product_stage", product))
                    .bind(("problem", problem.to_owned()))
                    .bind(("expected", expected.map(str::to_owned)))
                    .bind(("operation", operation.to_owned()))
                    .bind(("request_digest", plan.request_digest.clone()))
                    .bind(("edit_count", uint(plan.changes.len())?))
                    .bind(("lifetime", lifetime_micros)),
            )
            .await;
            match result {
                Err(error) if retryable_stage(&error) && attempt + 1 < RETRIES => {
                    tokio::time::sleep(Duration::from_millis(10 << attempt.min(4))).await;
                }
                Err(error) => return Err(error),
                Ok(mut response) => {
                    let row = response
                        .take::<Option<Object>>(response.num_statements().saturating_sub(2))?
                        .ok_or_else(|| {
                            CanonicalError::Configuration(
                                "staging lease missing after acquisition".into(),
                            )
                        })?;
                    let row = wire::decode_canonical_stages(row)?;
                    return Ok(ClosedStage {
                        problem: problem.into(),
                        expected: expected.map(str::to_owned),
                        operation: operation.into(),
                        generation: row.generation,
                        request: plan.request.clone(),
                        request_digest: plan.request_digest.clone(),
                        lifetime_micros,
                    });
                }
            }
        }
        Err(CanonicalError::Configuration(
            "staging lease retries exhausted".into(),
        ))
    }

    async fn stage_query(
        &self,
        token: &ClosedStage,
        body: &str,
        bindings: Vec<(&str, Value)>,
    ) -> Result<surrealdb::IndexedResults, CanonicalError> {
        for attempt in 0..RETRIES {
            self.ensure_writes()?;
            let mut query = self
                .db
                .query(format!(
                    "{STAGE_BEGIN}\n{}\n{STAGE_END}",
                    bounded_edit_sql(body)
                ))
                .bind(("problem", token.problem.clone()))
                .bind(("operation", token.operation.clone()))
                .bind((
                    "generation",
                    canonical_codec::encode_uint(token.generation)?,
                ))
                .bind(("lifetime", token.lifetime_micros));
            for (key, value) in &bindings {
                query = query.bind(((*key).to_owned(), value.clone()));
            }
            match bounded_query(query).await {
                Err(error) if retryable_stage(&error) && attempt + 1 < RETRIES => {
                    tokio::time::sleep(Duration::from_millis(10 << attempt.min(4))).await;
                }
                other => return other,
            }
        }
        Err(CanonicalError::Configuration(
            "staging mutation retries exhausted".into(),
        ))
    }

    async fn stage_metadata(
        &self,
        token: &ClosedStage,
        ordinal: usize,
        change: &ChangeDescription,
    ) -> Result<(), CanonicalError> {
        let body = if change.version.is_some() {
            STAGE_METADATA_VERSION
        } else {
            STAGE_METADATA_REMOVAL
        };
        self.stage_query(token, body, metadata_bindings(token, ordinal, change)?)
            .await?;
        Ok(())
    }

    async fn stage_small_edits(
        &self,
        token: &ClosedStage,
        items: Vec<Value>,
    ) -> Result<(), CanonicalError> {
        let body = format!(
            "FOR $item IN $items {{\nLET $edit = $item.edit;\nIF $item.manifest = NONE {{\n{STAGE_METADATA_REMOVAL}\n}} ELSE {{\nLET $manifest = $item.manifest;\nLET $receipt = $item.receipt;\n{STAGE_METADATA_VERSION}\nIF $header = NONE OR $header.closed = false {{\nFOR $block IN $item.blocks {{\n{STAGE_BLOCK}\n}};\nLET $version = $manifest.key;\nLET $edges = $item.edges;\n{STAGE_EDGES}\n{CLOSE_VERSION}\n}};\n}};\n}};"
        );
        self.stage_query(token, &body, vec![("items", Value::Array(items.into()))])
            .await?;
        Ok(())
    }

    async fn stage_version(
        &self,
        token: &ClosedStage,
        description: &VersionDescription,
        version: &ObjectVersion,
        references: &[(String, String)],
    ) -> Result<(), CanonicalError> {
        let header: Option<Object> = request(
            self.db
                .select(("canonical_version_manifests", version.key.as_str())),
        )
        .await?;
        if header
            .map(wire::decode_canonical_version_manifests)
            .transpose()?
            .is_some_and(|header| header.closed)
        {
            // stage_metadata already checked the permanent content receipt and
            // acquired the live association under the global version guard.
            return Ok(());
        }
        for (ordinal, payload) in version
            .payload
            .as_slice()
            .chunks(SOURCE_BLOCK_BYTES)
            .enumerate()
        {
            let ordinal = u64::try_from(ordinal).map_err(|_| CanonicalError::PayloadLimit)?;
            self.stage_query(
                token,
                STAGE_BLOCK,
                vec![("block", source_block(&version.key, ordinal, payload)?)],
            )
            .await?;
        }
        for batch in references.chunks(64).enumerate() {
            let (batch_index, references) = batch;
            let rows = references
                .iter()
                .enumerate()
                .map(|(index, (scope, name))| {
                    let ordinal = u64::try_from(batch_index * 64 + index)
                        .map_err(|_| CanonicalError::PayloadLimit)?;
                    source_edge(&version.key, ordinal, scope, name)
                })
                .collect::<Result<Vec<_>, CanonicalError>>()?;
            self.stage_query(
                token,
                STAGE_EDGES,
                vec![
                    ("version", Value::String(version.key.clone())),
                    ("edges", Value::Array(rows.into())),
                ],
            )
            .await?;
        }
        self.stage_query(
            token,
            CLOSE_VERSION,
            vec![("version", Value::String(description.key.clone()))],
        )
        .await?;
        Ok(())
    }

    /// Assemble only this demanded immutable object. Production multi-message
    /// scientific reads use the protected variant below.
    #[cfg(test)]
    #[cfg(all(test, feature = "canonical-tests"))]
    pub(crate) async fn assemble_object(
        &self,
        version: &str,
    ) -> Result<Option<ObjectVersion>, CanonicalError> {
        self.assemble_source(None, version, None).await
    }

    /// Every header/block read validates the same exact selected revision and live
    /// protection. Expiry during transfer refuses completion rather than returning
    /// a partially assembled scientific object.
    pub(crate) async fn assemble_selected_object(
        &self,
        selection: &ProtectedSelection,
        version: &str,
    ) -> Result<Option<ObjectVersion>, CanonicalError> {
        self.assemble_source(Some(selection), version, None).await
    }

    /// Read the exact selected payload extent before allocating scientific bytes.
    /// The caller can reserve its payload and decode budget before assembly; every
    /// later block read still validates the same live protection.
    pub async fn selected_object_extent(
        &self,
        selection: &ProtectedSelection,
        version: &str,
    ) -> Result<Option<usize>, CanonicalError> {
        let Some(header) = self
            .source_record(
                "canonical_version_manifests",
                version,
                Some(selection),
                version,
            )
            .await?
        else {
            return Ok(None);
        };
        let header = wire::decode_canonical_version_manifests(header)?;
        if !header.closed {
            return Ok(None);
        }
        if header.key != version || header.interpretation != wire::INTERPRETATION {
            return Err(CanonicalError::Configuration(
                "immutable source manifest identity/interpretation mismatch".into(),
            ));
        }
        let length =
            usize::try_from(header.payload_len).map_err(|_| CanonicalError::PayloadLimit)?;
        let expected_blocks = u64::try_from(length.div_ceil(SOURCE_BLOCK_BYTES))
            .map_err(|_| CanonicalError::PayloadLimit)?;
        if expected_blocks != header.block_count {
            return Err(CanonicalError::Configuration(
                "immutable source block coverage mismatch".into(),
            ));
        }
        Ok(Some(length))
    }

    pub(crate) async fn validate_product_candidate(
        &self,
        selection: &ProtectedSelection,
        product: &pse_model::generated::runtime::canonical_products::Row,
        descriptor: &ProductBlob,
    ) -> Result<(), CanonicalError> {
        let row = self
            .authorized_record(
                "canonical_version_manifests",
                &descriptor.version,
                Some(selection),
                &descriptor.version,
                Some(product),
            )
            .await?
            .ok_or_else(|| CanonicalError::Configuration("product manifest absent".into()))?;
        let header = wire::decode_canonical_version_manifests(row)?;
        if !header.closed
            || header.key != descriptor.version
            || header.logical != PRODUCT_BLOB_KIND
            || header.kind != PRODUCT_BLOB_KIND
            || header.interpretation != wire::INTERPRETATION
            || header.reference_count != 0
            || header.payload_len != descriptor.bytes as u64
            || header.payload_digest != descriptor.digest
            || header.block_count != descriptor.bytes.div_ceil(SOURCE_BLOCK_BYTES) as u64
        {
            return Err(CanonicalError::Configuration(
                "product descriptor/header mismatch".into(),
            ));
        }
        Ok(())
    }

    pub(crate) async fn assemble_product_blob(
        &self,
        selection: &ProtectedSelection,
        product: &pse_model::generated::runtime::canonical_products::Row,
        descriptor: &ProductBlob,
    ) -> Result<Option<ObjectVersion>, CanonicalError> {
        self.assemble_source(
            Some(selection),
            &descriptor.version,
            Some((product, descriptor)),
        )
        .await
    }

    async fn source_record(
        &self,
        table: &str,
        key: &str,
        selection: Option<&ProtectedSelection>,
        version: &str,
    ) -> Result<Option<Object>, CanonicalError> {
        self.authorized_record(table, key, selection, version, None)
            .await
    }

    async fn authorized_record(
        &self,
        table: &str,
        key: &str,
        selection: Option<&ProtectedSelection>,
        version: &str,
        product: Option<&pse_model::generated::runtime::canonical_products::Row>,
    ) -> Result<Option<Object>, CanonicalError> {
        if let Some(product) = product {
            let selection = selection.ok_or_else(|| {
                CanonicalError::Configuration("product read requires protection".into())
            })?;
            let sql = format!(
                "{PROTECTED_BEGIN}\nLET $saved = SELECT * FROM ONLY type::record('canonical_products', $product.key);\nLET $root = SELECT * FROM ONLY type::record('canonical_roots', $product.key);\nLET $child = SELECT * FROM ONLY type::record('canonical_staged_edits', $product.key + ':0');\nLET $stage = SELECT * FROM ONLY type::record('canonical_stages', $product.key);\nIF $saved = NONE OR $saved.key != $product.key OR $saved.problem != $problem OR $saved.problem != $product.problem OR $saved.revision != $product.revision OR $saved.payload != $product.payload OR $saved.dependencies != $product.dependencies OR $saved.request != $product.request OR $saved.producer != $product.producer OR $saved.interpretation != $product.interpretation OR $root = NONE OR $root.problem != $saved.problem OR $root.revision != $saved.revision OR $root.owner_kind != 'product' OR $root.owner != $saved.key OR $child = NONE OR $child.stage != $saved.key OR $child.version != $version OR $child.logical != $reserved OR $child.scope != $reserved OR $child.name != $reserved OR $stage = NONE OR $stage.problem != $saved.problem OR $stage.closed = false OR $stage.activated = false OR $stage.abandoned {{ THROW 'product blob authorization unavailable'; }};\nRETURN SELECT * FROM ONLY type::record($table, $key);\nCOMMIT;"
            );
            let mut response = protected_query(|| {
                Ok(self
                    .db
                    .query(sql.clone())
                    .bind(("problem", selection.revision().problem.clone()))
                    .bind(("protection", selection.key().to_owned()))
                    .bind(("revision", selection.revision().key.clone()))
                    .bind((
                        "sequence",
                        canonical_codec::encode_uint(selection.revision().sequence)?,
                    ))
                    .bind(("product", wire::encode_canonical_products(product)?))
                    .bind(("reserved", PRODUCT_BLOB_KIND.to_owned()))
                    .bind(("version", version.to_owned()))
                    .bind(("table", table.to_owned()))
                    .bind(("key", key.to_owned())))
            })
            .await?;
            return Ok(response.take(response.num_statements().saturating_sub(2))?);
        }
        if let Some(selection) = selection {
            let mut response = protected_query(|| Ok(self.db.query(format!("{PROTECTED_BEGIN}\nLET $members = SELECT key FROM canonical_memberships WHERE problem = $problem AND version = $version AND from_sequence <= $sequence AND (to_sequence = NONE OR to_sequence > $sequence) LIMIT 1;\nLET $row = IF array::len($members) = 0 {{ RETURN NONE; }} ELSE {{ RETURN SELECT * FROM ONLY type::record($table, $key); }};\nRETURN $row;\nCOMMIT;"))
                .bind(("problem", selection.revision().problem.clone())).bind(("protection", selection.key().to_owned()))
                .bind(("revision", selection.revision().key.clone())).bind(("sequence", canonical_codec::encode_uint(selection.revision().sequence)?))
                .bind(("version", version.to_owned())).bind(("table", table.to_owned())).bind(("key", key.to_owned())))).await?;
            Ok(response.take(response.num_statements().saturating_sub(2))?)
        } else {
            Ok(request(self.db.select((table, key))).await?)
        }
    }

    async fn assemble_source(
        &self,
        selection: Option<&ProtectedSelection>,
        version: &str,
        product: Option<(
            &pse_model::generated::runtime::canonical_products::Row,
            &ProductBlob,
        )>,
    ) -> Result<Option<ObjectVersion>, CanonicalError> {
        let Some(header) = self
            .authorized_record(
                "canonical_version_manifests",
                version,
                selection,
                version,
                product.map(|(row, _)| row),
            )
            .await?
        else {
            return Ok(None);
        };
        let header = wire::decode_canonical_version_manifests(header)?;
        if !header.closed {
            return Ok(None);
        }
        if header.key != version || header.interpretation != wire::INTERPRETATION {
            return Err(CanonicalError::Configuration(
                "immutable source manifest identity/interpretation mismatch".into(),
            ));
        }
        let length =
            usize::try_from(header.payload_len).map_err(|_| CanonicalError::PayloadLimit)?;
        if let Some((_, descriptor)) = product
            && (header.logical != PRODUCT_BLOB_KIND
                || header.kind != PRODUCT_BLOB_KIND
                || header.reference_count != 0
                || header.payload_digest != descriptor.digest
                || header.payload_len != descriptor.bytes as u64
                || length > PRODUCT_BLOB_LIMIT)
        {
            return Err(CanonicalError::Configuration(
                "product manifest differs from descriptor".into(),
            ));
        }
        let expected_blocks = u64::try_from(length.div_ceil(SOURCE_BLOCK_BYTES))
            .map_err(|_| CanonicalError::PayloadLimit)?;
        if expected_blocks != header.block_count {
            return Err(CanonicalError::Configuration(
                "immutable source block coverage mismatch".into(),
            ));
        }
        let mut payload = Vec::new();
        payload.try_reserve_exact(length).map_err(|error| {
            CanonicalError::Configuration(format!("demanded source allocation: {error}"))
        })?;
        for ordinal in 0..header.block_count {
            let key = block_key(version, ordinal);
            let block = self
                .authorized_record(
                    "canonical_payload_blocks",
                    &key,
                    selection,
                    version,
                    product.map(|(row, _)| row),
                )
                .await?
                .ok_or_else(|| {
                    CanonicalError::Configuration(
                        "immutable source block missing during assembly".into(),
                    )
                })?;
            let block = wire::decode_canonical_payload_blocks(block)?;
            let expected_length = (length - payload.len()).min(SOURCE_BLOCK_BYTES);
            if block.key != key
                || block.version != version
                || block.ordinal != ordinal
                || block.payload.len() != expected_length
                || payload_digest(block.payload.as_slice()) != block.digest
            {
                return Err(CanonicalError::Configuration(
                    "immutable source block identity/length/digest mismatch".into(),
                ));
            }
            payload.extend_from_slice(block.payload.as_slice());
        }
        // The final protected header read closes an expiry race after the last
        // payload block, including an empty payload which has no block reads.
        if selection.is_some() {
            let final_header = self
                .authorized_record(
                    "canonical_version_manifests",
                    version,
                    selection,
                    version,
                    product.map(|(row, _)| row),
                )
                .await?
                .ok_or_else(|| {
                    CanonicalError::Configuration("selected manifest no longer visible".into())
                })?;
            if wire::decode_canonical_version_manifests(final_header)? != header {
                return Err(CanonicalError::Configuration(
                    "immutable manifest changed during assembly".into(),
                ));
            }
        }
        if payload.len() != length || payload_digest(&payload) != header.payload_digest {
            return Err(CanonicalError::Configuration(
                "immutable source payload length/digest mismatch".into(),
            ));
        }
        Ok(Some(ObjectVersion {
            key: header.key,
            logical: header.logical,
            kind: header.kind,
            interpretation: header.interpretation,
            payload: payload.into(),
        }))
    }
}

const BEGIN_STAGE: &str = r#"BEGIN;
LET $retention_guard = type::record('canonical_guards', 'retention:' + $problem);
SELECT * FROM $retention_guard FOR UPDATE;
LET $stage_guard = type::record('canonical_guards', 'stage:' + $operation);
SELECT * FROM $stage_guard FOR UPDATE;
LET $old = SELECT * FROM ONLY type::record('canonical_stages', $operation);
IF $old != NONE AND ($old.problem != $problem OR ($old.expected_head ?? NONE) != $expected OR $old.request_digest != $request_digest) { THROW 'staged operation identity reused'; };
IF $old.activated ?? false { THROW 'staged operation already activated; settle immutable acknowledgement'; };
LET $head = SELECT * FROM ONLY type::record('canonical_problems', $problem);
IF $product_stage = false AND ($head.head ?? NONE) != $expected { THROW 'head compare-and-set conflict before staging'; };
UPSERT type::record('canonical_stages', $operation) SET key = $operation, problem = $problem, expected_head = $expected, request_digest = $request_digest,
    expires_at = time::micros() + $lifetime, closed = false, activated = false, edit_count = $edit_count,
    generation = ($old.generation ?? 0dec) + 1dec, abandoned = false, cleanup_complete = false;
UPSERT $stage_guard SET key = 'stage:' + $operation, generation = (generation ?? 0dec) + 1dec;
UPSERT $retention_guard SET key = 'retention:' + $problem, generation = (generation ?? 0dec) + 1dec;
SELECT * FROM ONLY type::record('canonical_stages', $operation);
COMMIT;"#;

const STAGE_BEGIN: &str = r#"BEGIN;
LET $retention_guard = type::record('canonical_guards', 'retention:' + $problem);
SELECT * FROM $retention_guard FOR UPDATE;
LET $stage_guard = type::record('canonical_guards', 'stage:' + $operation);
SELECT * FROM $stage_guard FOR UPDATE;
LET $lease = SELECT * FROM ONLY type::record('canonical_stages', $operation);
IF $lease = NONE OR $lease.problem != $problem OR $lease.generation != $generation OR $lease.activated OR $lease.abandoned OR $lease.closed OR $lease.expires_at <= time::micros() { THROW 'staging lease fenced or expired'; };"#;

const STAGE_END: &str = r#"UPDATE type::record('canonical_stages', $operation) SET expires_at = time::micros() + $lifetime;
UPSERT $stage_guard SET key = 'stage:' + $operation, generation = (generation ?? 0dec) + 1dec;
UPSERT $retention_guard SET key = 'retention:' + $problem, generation = (generation ?? 0dec) + 1dec;
COMMIT;"#;

const STAGE_METADATA_VERSION: &str = r#"LET $version_guard = type::record('canonical_guards', 'version:' + $manifest.key);
SELECT * FROM $version_guard FOR UPDATE;
LET $identity = SELECT * FROM ONLY type::record('canonical_version_receipts', $receipt.key);
IF $identity != NONE AND $identity.content_digest != $receipt.content_digest { THROW 'immutable version identity reused'; };
IF $identity = NONE { CREATE type::record('canonical_version_receipts', $receipt.key) CONTENT $receipt; };
LET $header = SELECT * FROM ONLY type::record('canonical_version_manifests', $manifest.key);
IF $header = NONE { CREATE type::record('canonical_version_manifests', $manifest.key) CONTENT $manifest; }
ELSE IF $header.logical != $manifest.logical OR $header.kind != $manifest.kind OR $header.interpretation != $manifest.interpretation OR $header.payload_digest != $manifest.payload_digest OR $header.payload_len != $manifest.payload_len OR $header.block_count != $manifest.block_count OR $header.reference_count != $manifest.reference_count { THROW 'immutable source manifest reused'; };
LET $old_edit = SELECT * FROM ONLY type::record('canonical_staged_edits', $edit.key);
IF $old_edit = NONE { CREATE type::record('canonical_staged_edits', $edit.key) CONTENT $edit; }
ELSE IF $old_edit.stage != $edit.stage OR $old_edit.ordinal != $edit.ordinal OR $old_edit.logical != $edit.logical OR $old_edit.scope != $edit.scope OR $old_edit.name != $edit.name OR ($old_edit.version ?? NONE) != ($edit.version ?? NONE) { THROW 'immutable stage metadata reused'; };
UPSERT $version_guard SET key = 'version:' + $manifest.key, generation = (generation ?? 0dec) + 1dec;"#;

const STAGE_METADATA_REMOVAL: &str = r#"LET $old_edit = SELECT * FROM ONLY type::record('canonical_staged_edits', $edit.key);
IF $old_edit = NONE { CREATE type::record('canonical_staged_edits', $edit.key) CONTENT $edit; }
ELSE IF $old_edit.stage != $edit.stage OR $old_edit.ordinal != $edit.ordinal OR $old_edit.logical != $edit.logical OR $old_edit.scope != $edit.scope OR $old_edit.name != $edit.name OR ($old_edit.version ?? NONE) != NONE { THROW 'immutable stage metadata reused'; };"#;

const STAGE_BLOCK: &str = r#"LET $version_guard = type::record('canonical_guards', 'version:' + $block.version);
SELECT * FROM $version_guard FOR UPDATE;
LET $header = SELECT * FROM ONLY type::record('canonical_version_manifests', $block.version);
IF $header = NONE OR $block.ordinal >= $header.block_count OR bytes::len($block.payload) != math::min([524288dec, $header.payload_len - $block.ordinal * 524288dec]) { THROW 'source block outside declared manifest'; };
LET $old_block = SELECT * FROM ONLY type::record('canonical_payload_blocks', $block.key);
IF $old_block = NONE {
    IF $header.closed { THROW 'closed source block unavailable'; };
    CREATE type::record('canonical_payload_blocks', $block.key) CONTENT $block RETURN NONE;
} ELSE IF $old_block.version != $block.version OR $old_block.ordinal != $block.ordinal OR $old_block.digest != $block.digest OR $old_block.payload != $block.payload { THROW 'immutable source block reused'; };
UPSERT $version_guard SET key = 'version:' + $block.version, generation = (generation ?? 0dec) + 1dec;"#;

const STAGE_EDGES: &str = r#"LET $version_guard = type::record('canonical_guards', 'version:' + $version);
SELECT * FROM $version_guard FOR UPDATE;
LET $header = SELECT * FROM ONLY type::record('canonical_version_manifests', $version);
IF $header = NONE { THROW 'source manifest unavailable'; };
FOR $edge IN $edges {
    IF $edge.source_version != $version OR $edge.ordinal >= $header.reference_count { THROW 'reference outside declared manifest'; };
    LET $old_edge = SELECT * FROM ONLY type::record('canonical_edges', $edge.key);
    IF $old_edge = NONE {
        IF $header.closed { THROW 'closed structural reference unavailable'; };
        CREATE type::record('canonical_edges', $edge.key) CONTENT $edge;
    } ELSE IF $old_edge.source_version != $edge.source_version OR $old_edge.ordinal != $edge.ordinal OR $old_edge.target_scope != $edge.target_scope OR $old_edge.target_name != $edge.target_name { THROW 'immutable structural reference reused'; };
};
UPSERT $version_guard SET key = 'version:' + $version, generation = (generation ?? 0dec) + 1dec;"#;

const CLOSE_VERSION: &str = r#"LET $version_guard = type::record('canonical_guards', 'version:' + $version);
SELECT * FROM $version_guard FOR UPDATE;
LET $header = SELECT * FROM ONLY type::record('canonical_version_manifests', $version);
IF $header = NONE { THROW 'source manifest unavailable'; };
LET $blocks = SELECT count() AS n FROM canonical_payload_blocks WHERE version = $version GROUP ALL;
LET $references = SELECT count() AS n FROM canonical_edges WHERE source_version = $version GROUP ALL;
IF ($blocks[0].n ?? 0) != $header.block_count OR ($references[0].n ?? 0) != $header.reference_count { THROW 'immutable source manifest incomplete'; };
UPDATE type::record('canonical_version_manifests', $version) SET closed = true;
UPSERT $version_guard SET key = 'version:' + $version, generation = (generation ?? 0dec) + 1dec;"#;

const CLOSE_STAGE: &str = r#"LET $edits = SELECT * FROM canonical_staged_edits WHERE stage = $operation ORDER BY ordinal LIMIT /* EDIT_SCAN_LIMIT */;
IF array::len($edits) != $lease.edit_count OR array::len($edits) > /* EDIT_LIMIT */ { THROW 'staged metadata manifest incomplete'; };
FOR $edit IN $edits {
    IF $edit.version != NONE {
        LET $header = SELECT * FROM ONLY type::record('canonical_version_manifests', $edit.version);
        IF $header = NONE OR $header.closed = false { THROW 'staged source manifest incomplete'; };
    };
};
UPDATE type::record('canonical_stages', $operation) SET closed = true;"#;

/// Shared physical deletion decision. Caller supplies its enclosing retention
/// transaction and $version/$excluded_membership/$excluded_stage bindings. A named
/// version guard fences newly associated stages even across different problems.
pub(crate) const DELETE_VERSION: &str = r#"LET $version_guard = type::record('canonical_guards', 'version:' + $version);
SELECT * FROM $version_guard FOR UPDATE;
LET $members = SELECT key FROM canonical_memberships WHERE version = $version AND ($excluded_membership = NONE OR key != $excluded_membership) LIMIT 1;
LET $stages = SELECT key FROM canonical_staged_edits WHERE version = $version AND ($excluded_stage = NONE OR stage != $excluded_stage)
    AND type::record('canonical_stages', stage).activated = false AND type::record('canonical_stages', stage).abandoned = false
    AND type::record('canonical_stages', stage).expires_at > time::micros() LIMIT 1;
LET $products = SELECT key FROM canonical_staged_edits WHERE version = $version AND scope = 'pse.product-blob.v1' AND type::record('canonical_products', stage).key = stage AND type::record('canonical_roots', stage).owner_kind = 'product' AND type::record('canonical_roots', stage).owner = stage AND type::record('canonical_roots', stage).problem = type::record('canonical_products', stage).problem AND type::record('canonical_roots', stage).revision = type::record('canonical_products', stage).revision LIMIT 1;
LET $deleted = IF array::len($members) != 0 OR array::len($stages) != 0 OR array::len($products) != 0 {
    RETURN {version:false, blocks:0dec, edges:0dec, pending:false};
} ELSE {
    LET $manifest = SELECT * FROM ONLY type::record('canonical_version_manifests', $version);
    UPDATE type::record('canonical_version_manifests', $version) SET closed = false;
    LET $blocks = SELECT key FROM canonical_payload_blocks WHERE version = $version ORDER BY ordinal LIMIT 64;
    LET $edges = SELECT key FROM canonical_edges WHERE source_version = $version ORDER BY ordinal LIMIT 64;
    FOR $block IN $blocks { DELETE ONLY type::record('canonical_payload_blocks', $block.key) RETURN NONE; };
    FOR $edge IN $edges { DELETE ONLY type::record('canonical_edges', $edge.key); };
    LET $remaining_blocks = SELECT key FROM canonical_payload_blocks WHERE version = $version LIMIT 1;
    LET $remaining_edges = SELECT key FROM canonical_edges WHERE source_version = $version LIMIT 1;
    IF array::len($remaining_blocks) = 0 AND array::len($remaining_edges) = 0 {
        DELETE ONLY type::record('canonical_version_manifests', $version);
        RETURN {version:$manifest != NONE, blocks:<decimal>array::len($blocks), edges:<decimal>array::len($edges), pending:false};
    } ELSE {
        RETURN {version:false, blocks:<decimal>array::len($blocks), edges:<decimal>array::len($edges), pending:true};
    };
};
UPSERT $version_guard SET key = 'version:' + $version, generation = (generation ?? 0dec) + 1dec;"#;

/// One bounded abandoned/activated-stage cleanup call. Compact operation and
/// version identity receipts survive completion of all child frontiers.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StagingReclamationPage {
    /// Continue with this cursor; none completes the current pass.
    pub after: Option<String>,
    /// Staged metadata associations removed.
    pub edits: usize,
    /// Payload blocks physically removed.
    pub blocks: usize,
    /// Structural references physically removed.
    pub edges: usize,
    /// Version manifests physically removed.
    pub versions: usize,
}

impl CanonicalStore {
    /// Activated stages release metadata; expired stages are generation-fenced
    /// before cleanup. Live associations in any problem retain their payloads.
    pub async fn reclaim_staging_page(
        &self,
        problem: &str,
        after: &str,
    ) -> Result<StagingReclamationPage, CanonicalError> {
        self.ensure_writes()?;
        let mut response = bounded_query(self.db.query("SELECT key FROM canonical_stages WHERE problem = $problem AND key > $after AND cleanup_complete = false AND (activated = true OR abandoned = true OR expires_at <= time::micros()) ORDER BY key LIMIT 1;")
            .bind(("problem", problem.to_owned())).bind(("after", after.to_owned()))).await?;
        let candidates: Vec<Object> = response.take(0)?;
        let Some(mut candidate) = candidates.into_iter().next() else {
            return Ok(StagingReclamationPage::default());
        };
        let operation =
            canonical_codec::decode_string(canonical_codec::required(&mut candidate, "key")?)?;
        let mut response = bounded_query(self.db.query("SELECT key FROM canonical_staged_edits WHERE stage = $stage ORDER BY ordinal LIMIT 64;")
            .bind(("stage", operation.clone()))).await?;
        let children: Vec<Object> = response.take(0)?;
        let mut page = StagingReclamationPage::default();
        for mut child in children {
            let key =
                canonical_codec::decode_string(canonical_codec::required(&mut child, "key")?)?;
            let query = format!(
                "{CLEANUP_STAGE_BEGIN}\nLET $child = SELECT * FROM ONLY type::record('canonical_staged_edits', $child_key);\nLET $result = IF $eligible = false OR $child = NONE OR $child.stage != $operation {{ RETURN {{edit:false, version:false, blocks:0dec, edges:0dec, pending:false}}; }} ELSE {{\nLET $version = $child.version ?? NONE;\nLET $excluded_membership = NONE;\nLET $excluded_stage = $operation;\nLET $physical = IF $version = NONE {{ RETURN {{version:false, blocks:0dec, edges:0dec, pending:false}}; }} ELSE {{\n{DELETE_VERSION}\nRETURN $deleted;\n}};\nLET $root = SELECT * FROM ONLY type::record('canonical_roots', $operation);\nLET $keep = $child.scope = 'pse.product-blob.v1' AND $root != NONE AND $root.owner_kind = 'product' AND $root.owner = $operation AND $root.problem = $problem AND type::record('canonical_products', $operation).revision = $root.revision;\nIF $physical.pending = false AND $keep = false {{ DELETE ONLY type::record('canonical_staged_edits', $child_key); }};\nRETURN {{edit:!$physical.pending AND !$keep, version:$physical.version, blocks:$physical.blocks, edges:$physical.edges, pending:$physical.pending}};\n}};\n{CLEANUP_STAGE_END}\nRETURN $result;\nCOMMIT;"
            );
            let mut decision = self
                .cleanup_stage_query(problem, &operation, &query, Some(&key))
                .await?
                .ok_or_else(|| {
                    CanonicalError::Configuration("staging cleanup decision missing".into())
                })?;
            page.edits += usize::from(canonical_codec::decode_boolean(canonical_codec::required(
                &mut decision,
                "edit",
            )?)?);
            page.versions += usize::from(canonical_codec::decode_boolean(
                canonical_codec::required(&mut decision, "version")?,
            )?);
            page.blocks += usize::try_from(canonical_codec::decode_uint(
                canonical_codec::required(&mut decision, "blocks")?,
            )?)
            .map_err(|_| CanonicalError::PayloadLimit)?;
            page.edges += usize::try_from(canonical_codec::decode_uint(
                canonical_codec::required(&mut decision, "edges")?,
            )?)
            .map_err(|_| CanonicalError::PayloadLimit)?;
            if canonical_codec::decode_boolean(canonical_codec::required(
                &mut decision,
                "pending",
            )?)? {
                page.after = Some(after.into());
                return Ok(page);
            }
        }
        let query = format!(
            "{CLEANUP_STAGE_BEGIN}\n{CLEANUP_STAGE_END}\nLET $all_remaining = SELECT key FROM canonical_staged_edits WHERE stage = $operation LIMIT 1;\nLET $root = SELECT * FROM ONLY type::record('canonical_roots', $operation);\nLET $rooted_product = $root != NONE AND $root.owner_kind = 'product' AND $root.owner = $operation AND $root.problem = $problem AND type::record('canonical_products', $operation).revision = $root.revision;\nLET $remaining = $all_remaining.filter(|$child| $rooted_product = false);\nIF $eligible AND array::len($all_remaining) = 0 {{ UPDATE type::record('canonical_stages', $operation) SET cleanup_complete = true; }};\nRETURN {{pending:array::len($remaining) != 0}};\nCOMMIT;"
        );
        let mut decision = self
            .cleanup_stage_query(problem, &operation, &query, None)
            .await?
            .ok_or_else(|| {
                CanonicalError::Configuration("staging cleanup completion missing".into())
            })?;
        page.after = if canonical_codec::decode_boolean(canonical_codec::required(
            &mut decision,
            "pending",
        )?)? {
            Some(after.into())
        } else {
            Some(operation)
        };
        Ok(page)
    }

    async fn cleanup_stage_query(
        &self,
        problem: &str,
        operation: &str,
        sql: &str,
        child: Option<&str>,
    ) -> Result<Option<Object>, CanonicalError> {
        for attempt in 0..RETRIES {
            self.ensure_writes()?;
            let result = bounded_query(
                self.db
                    .query(sql)
                    .bind(("problem", problem.to_owned()))
                    .bind(("operation", operation.to_owned()))
                    .bind(("child_key", child.map(str::to_owned))),
            )
            .await;
            match result {
                Err(error) if conflict(&error) && attempt + 1 < RETRIES => {
                    tokio::task::yield_now().await;
                }
                Err(error) => return Err(error),
                Ok(mut response) => {
                    return Ok(response.take(response.num_statements().saturating_sub(2))?);
                }
            }
        }
        Err(CanonicalError::Configuration(
            "staging cleanup retries exhausted".into(),
        ))
    }
}

const CLEANUP_STAGE_BEGIN: &str = r#"BEGIN;
LET $retention_guard = type::record('canonical_guards', 'retention:' + $problem);
SELECT * FROM $retention_guard FOR UPDATE;
LET $stage_guard = type::record('canonical_guards', 'stage:' + $operation);
SELECT * FROM $stage_guard FOR UPDATE;
LET $lease = SELECT * FROM ONLY type::record('canonical_stages', $operation);
LET $eligible = $lease != NONE AND $lease.problem = $problem AND ($lease.activated OR $lease.abandoned OR $lease.expires_at <= time::micros());
IF $eligible AND $lease.activated = false { UPDATE type::record('canonical_stages', $operation) SET abandoned = true, closed = false, generation = generation + 1dec; };"#;

const CLEANUP_STAGE_END: &str = r#"UPSERT $stage_guard SET key = 'stage:' + $operation, generation = (generation ?? 0dec) + 1dec;
UPSERT $retention_guard SET key = 'retention:' + $problem, generation = (generation ?? 0dec) + 1dec;"#;

fn describe(
    version: &ObjectVersion,
    edit: &ObjectEdit,
    references: &[u8],
) -> Result<VersionDescription, CanonicalError> {
    if version.key.len() > IDENTITY_BYTES || version.kind.len() > IDENTITY_BYTES {
        return Err(CanonicalError::PayloadLimit);
    }
    if version.logical != edit.logical
        || version.interpretation != wire::INTERPRETATION
        || version.key.is_empty()
    {
        return Err(CanonicalError::Configuration(
            "object version identity/interpretation mismatch".into(),
        ));
    }
    let digest = payload_digest(version.payload.as_slice());
    let len = u64::try_from(version.payload.len()).map_err(|_| CanonicalError::PayloadLimit)?;
    let blocks = u64::try_from(version.payload.len().div_ceil(SOURCE_BLOCK_BYTES))
        .map_err(|_| CanonicalError::PayloadLimit)?;
    let reference_count =
        u64::try_from(edit.references.len()).map_err(|_| CanonicalError::PayloadLimit)?;
    let content_digest = derive_hash(
        Frame::CanonicalVersionV1,
        &[
            version.logical.as_bytes(),
            version.kind.as_bytes(),
            version.interpretation.as_bytes(),
            &len.to_le_bytes(),
            digest.as_bytes(),
            references,
        ],
    )
    .to_hex();
    Ok(VersionDescription {
        key: version.key.clone(),
        logical: version.logical.clone(),
        kind: version.kind.clone(),
        interpretation: version.interpretation.clone(),
        payload_digest: digest,
        payload_len: len,
        block_count: blocks,
        reference_count,
        content_digest,
    })
}

#[cfg(all(test, feature = "canonical-tests"))]
mod canonical_server_unit {
    use super::*;
    use crate::canonical::CanonicalOptions;
    use crate::canonical::checked;
    use std::path::Path;

    async fn fixture() -> (CanonicalStore, String) {
        let state =
            std::env::var("PSE_SURREAL_STATE").expect("canonical-test supplies supervised state");
        let mut options = CanonicalOptions::from_state(Path::new(&state)).unwrap();
        options.database = format!("canonical_staging_{}", uuid::Uuid::new_v4().simple());
        let store = CanonicalStore::connect(&options).await.unwrap();
        store.create().await.unwrap();
        (store, options.database)
    }

    fn edit(key: &str, logical: &str, length: usize) -> ObjectEdit {
        let mut payload = (0..length)
            .map(|index| u8::try_from(index % 251).unwrap())
            .collect::<Vec<_>>();
        if length >= 16 {
            payload[8..16].copy_from_slice(&(-0.0f64).to_bits().to_le_bytes());
        }
        ObjectEdit {
            logical: logical.into(),
            scope: "root".into(),
            name: logical.into(),
            references: Vec::new(),
            version: Some(ObjectVersion {
                key: key.into(),
                logical: logical.into(),
                kind: "test".into(),
                payload: payload.into(),
                interpretation: wire::INTERPRETATION.into(),
            }),
        }
    }

    async fn remove(store: &CanonicalStore, database: &str) {
        store
            .db
            .query(format!("REMOVE DATABASE {database};"))
            .await
            .and_then(checked)
            .unwrap();
    }

    #[tokio::test]
    async fn declaration_inventory_above_old_edit_count_activates_atomically() {
        let (store, database) = fixture().await;
        let edits = (0..1050)
            .map(|ordinal| edit(&format!("version-{ordinal}"), &format!("name-{ordinal}"), 3))
            .collect::<Vec<_>>();
        let revision = store
            .edit("problem", None, "wide-inventory", &edits)
            .await
            .unwrap();
        assert_eq!(revision.sequence, 1);
        assert_eq!(
            store
                .edit("problem", None, "wide-inventory", &edits)
                .await
                .unwrap(),
            revision
        );
        let pin = store
            .protect(revision, Duration::from_secs(60))
            .await
            .unwrap();
        let mut after = String::new();
        let mut count = 0;
        loop {
            let page = store.membership_page(&pin, None, &after).await.unwrap();
            if page.is_empty() {
                break;
            }
            count += page.len();
            after = page.last().unwrap().key.clone();
        }
        assert_eq!(count, 1050);
        store.release(&pin).await.unwrap();
        remove(&store, &database).await;
    }

    #[test]
    fn declaration_count_and_metadata_extent_remain_bounded() {
        let edits = (0..=MAX_EDITS)
            .map(|ordinal| ObjectEdit {
                logical: format!("logical-{ordinal}"),
                scope: "scope".into(),
                name: format!("name-{ordinal}"),
                version: None,
                references: Vec::new(),
            })
            .collect::<Vec<_>>();
        assert!(plan("problem", None, &edits[..MAX_EDITS]).is_ok());
        assert!(matches!(
            plan("problem", None, &edits),
            Err(CanonicalError::PayloadLimit)
        ));
        let mut oversized = edits[..MAX_EDITS].to_vec();
        for edit in &mut oversized {
            edit.scope = "s".repeat(IDENTITY_BYTES);
            edit.name = "n".repeat(IDENTITY_BYTES);
        }
        assert!(matches!(
            plan("problem", None, &oversized),
            Err(CanonicalError::PayloadLimit)
        ));
        // Hash input spans many bounded RPCs, so crossing one message is allowed.
        let mut wide = edits[..1024].to_vec();
        for edit in &mut wide {
            edit.scope = "s".repeat(IDENTITY_BYTES);
        }
        assert!(plan("problem", None, &wide).is_ok());
    }

    #[tokio::test]
    async fn atomic_revision_exceeds_old_whole_payload_cap_and_has_native_graph_endpoints() {
        let (store, database) = fixture().await;
        let edits = [
            edit("large-x", "x", 2 * 1024 * 1024),
            edit("large-y", "y", 2 * 1024 * 1024),
        ];
        let planned = plan("problem", None, &edits).unwrap();
        let token = store
            .begin_stage("problem", None, "large-operation", &planned, STAGE_LIFETIME)
            .await
            .unwrap();
        for (ordinal, (change, edit)) in planned.changes.iter().zip(&edits).enumerate() {
            store.stage_metadata(&token, ordinal, change).await.unwrap();
            store
                .stage_version(
                    &token,
                    change.version.as_ref().unwrap(),
                    edit.version.as_ref().unwrap(),
                    &edit.references,
                )
                .await
                .unwrap();
            assert!(store.revision("large-operation").await.unwrap().is_none());
            let mut rows = store
                .db
                .query("SELECT key FROM canonical_memberships WHERE problem = 'problem';")
                .await
                .and_then(checked)
                .unwrap();
            assert!(rows.take::<Vec<Object>>(0).unwrap().is_empty());
        }
        store
            .stage_query(&token, CLOSE_STAGE, Vec::new())
            .await
            .unwrap();
        let revision = store
            .edit("problem", None, "large-operation", &edits)
            .await
            .unwrap();
        let pin = store
            .protect(revision.clone(), Duration::from_secs(60))
            .await
            .unwrap();
        assert_eq!(
            store.membership_page(&pin, None, "").await.unwrap().len(),
            2
        );
        for edit in &edits {
            let expected = edit.version.as_ref().unwrap();
            assert_eq!(
                store
                    .selected_object_extent(&pin, &expected.key)
                    .await
                    .unwrap(),
                Some(expected.payload.len())
            );
            let selected = store
                .selected_object(&pin, &expected.key)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(selected, *expected);
        }
        let mut traversal = store.db.query("SELECT VALUE ->canonical_memberships->canonical_version_manifests.key FROM ONLY canonical_problems:problem;")
            .await.and_then(checked).unwrap();
        let mut keys = traversal.take::<Vec<String>>(0).unwrap();
        keys.sort();
        assert_eq!(keys, ["large-x", "large-y"]);
        let mut physical = store
            .db
            .query("SELECT key FROM canonical_versions;")
            .await
            .and_then(checked)
            .unwrap();
        assert!(physical.take::<Vec<Object>>(0).unwrap().is_empty());
        assert_eq!(
            store
                .edit("problem", None, "large-operation", &edits)
                .await
                .unwrap(),
            revision
        );
        store.release(&pin).await.unwrap();
        remove(&store, &database).await;
    }

    #[tokio::test]
    async fn individual_payload_above_message_limit_is_exact_and_corruption_refuses_completion() {
        let (store, database) = fixture().await;
        let edits = [edit("huge-version", "x", 5 * 1024 * 1024 + 37)];
        let revision = store
            .edit("problem", None, "huge-operation", &edits)
            .await
            .unwrap();
        let pin = store
            .protect(revision, Duration::from_secs(60))
            .await
            .unwrap();
        assert_eq!(
            store
                .selected_object_extent(&pin, "huge-version")
                .await
                .unwrap(),
            Some(5 * 1024 * 1024 + 37)
        );
        assert!(
            store
                .selected_object_extent(&pin, "not-selected")
                .await
                .unwrap()
                .is_none()
        );
        let selected = store
            .selected_object(&pin, "huge-version")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(selected.payload, edits[0].version.as_ref().unwrap().payload);
        let block = block_key("huge-version", 0);
        store
            .db
            .query("UPDATE type::record('canonical_payload_blocks', $key) SET digest = 'wrong';")
            .bind(("key", block))
            .await
            .and_then(checked)
            .unwrap();
        assert!(store.selected_object(&pin, "huge-version").await.is_err());
        store.release(&pin).await.unwrap();
        assert!(
            store
                .selected_object_extent(&pin, "huge-version")
                .await
                .is_err()
        );
        remove(&store, &database).await;
    }

    #[tokio::test]
    async fn interrupted_stage_resumes_and_fences_previous_writer_and_stale_head() {
        let (store, database) = fixture().await;
        let edits = [edit("resume-version", "x", 1024 * 1024 + 11)];
        let planned = plan("problem", None, &edits).unwrap();
        let old = store
            .begin_stage(
                "problem",
                None,
                "resume-operation",
                &planned,
                STAGE_LIFETIME,
            )
            .await
            .unwrap();
        store
            .stage_metadata(&old, 0, &planned.changes[0])
            .await
            .unwrap();
        let payload = &edits[0].version.as_ref().unwrap().payload.as_slice()[..SOURCE_BLOCK_BYTES];
        let block = pse_model::generated::runtime::canonical_payload_blocks::Row {
            key: block_key("resume-version", 0),
            version: "resume-version".into(),
            ordinal: 0,
            payload: payload.to_vec().into(),
            digest: payload_digest(payload),
        };
        store
            .stage_query(
                &old,
                STAGE_BLOCK,
                vec![(
                    "block",
                    Value::Object(wire::encode_canonical_payload_blocks(&block).unwrap()),
                )],
            )
            .await
            .unwrap();
        assert!(
            store
                .stage_query(&old, CLOSE_STAGE, Vec::new())
                .await
                .is_err()
        );
        let resumed = store
            .begin_stage(
                "problem",
                None,
                "resume-operation",
                &planned,
                STAGE_LIFETIME,
            )
            .await
            .unwrap();
        assert!(resumed.generation > old.generation);
        assert!(
            store
                .stage_query(
                    &old,
                    CLOSE_VERSION,
                    vec![("version", Value::String("resume-version".into()))]
                )
                .await
                .is_err()
        );
        let revision = store
            .edit("problem", None, "resume-operation", &edits)
            .await
            .unwrap();
        let pin = store
            .protect(revision.clone(), Duration::from_secs(60))
            .await
            .unwrap();
        assert_eq!(
            store
                .selected_object(&pin, "resume-version")
                .await
                .unwrap()
                .unwrap(),
            *edits[0].version.as_ref().unwrap()
        );
        store.release(&pin).await.unwrap();
        let future = [edit("future-version", "x", 32)];
        let future_plan = plan("problem", Some("resume-operation"), &future).unwrap();
        let token = store
            .begin_stage(
                "problem",
                Some("resume-operation"),
                "stale-operation",
                &future_plan,
                STAGE_LIFETIME,
            )
            .await
            .unwrap();
        store
            .stage_metadata(&token, 0, &future_plan.changes[0])
            .await
            .unwrap();
        store
            .stage_version(
                &token,
                future_plan.changes[0].version.as_ref().unwrap(),
                future[0].version.as_ref().unwrap(),
                &[],
            )
            .await
            .unwrap();
        store
            .stage_query(&token, CLOSE_STAGE, Vec::new())
            .await
            .unwrap();
        let latest = store
            .edit(
                "problem",
                Some("resume-operation"),
                "winning-operation",
                &[edit("winning-version", "x", 32)],
            )
            .await
            .unwrap();
        assert!(
            store
                .edit(
                    "problem",
                    Some("resume-operation"),
                    "stale-operation",
                    &future
                )
                .await
                .is_err()
        );
        assert!(store.revision("stale-operation").await.unwrap().is_none());
        assert_eq!(
            store.revision("winning-operation").await.unwrap(),
            Some(latest)
        );
        let first = store
            .edit(
                "tuple-identities",
                None,
                "a:b",
                &[edit("tuple-v1", "c", 16)],
            )
            .await
            .unwrap();
        let second = store
            .edit(
                "tuple-identities",
                Some("a:b"),
                "a",
                &[edit("tuple-v2", "b:c", 16)],
            )
            .await
            .unwrap();
        assert_eq!(first.sequence, 1);
        assert_eq!(second.sequence, 2);
        let pin = store
            .protect(second, Duration::from_secs(30))
            .await
            .unwrap();
        assert_eq!(
            store.membership_page(&pin, None, "").await.unwrap().len(),
            2
        );
        store.release(&pin).await.unwrap();
        remove(&store, &database).await;
    }

    #[tokio::test]
    async fn abandoned_blocks_reclaim_in_bounded_pages_and_permanent_identity_survives() {
        let (store, database) = fixture().await;
        let edits = [edit("orphan-version", "x", SOURCE_BLOCK_BYTES * 65 + 17)];
        let planned = plan("problem", None, &edits).unwrap();
        let token = store
            .begin_stage(
                "problem",
                None,
                "orphan-operation",
                &planned,
                Duration::from_secs(1),
            )
            .await
            .unwrap();
        store
            .stage_metadata(&token, 0, &planned.changes[0])
            .await
            .unwrap();
        store
            .stage_version(
                &token,
                planned.changes[0].version.as_ref().unwrap(),
                edits[0].version.as_ref().unwrap(),
                &[],
            )
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(1100)).await;
        let page = store.reclaim_staging_page("problem", "").await.unwrap();
        assert_eq!(page.blocks, 64);
        assert_eq!(page.edits, 0);
        assert_eq!(page.versions, 0);
        assert!(
            store
                .stage_query(&token, CLOSE_STAGE, Vec::new())
                .await
                .is_err()
        );
        let final_page = store
            .reclaim_staging_page("problem", page.after.as_deref().unwrap())
            .await
            .unwrap();
        assert_eq!(final_page.blocks, 2);
        assert_eq!(final_page.edits, 1);
        assert_eq!(final_page.versions, 1);
        assert!(store.object("orphan-version").await.unwrap().is_none());
        let receipt: Option<Object> = store
            .db
            .select(("canonical_version_receipts", "orphan-version"))
            .await
            .unwrap();
        assert!(receipt.is_some());
        let changed = [edit("orphan-version", "x", 16)];
        assert!(
            store
                .edit("problem", None, "changed-operation", &changed)
                .await
                .is_err()
        );
        assert!(
            store
                .edit("problem", None, "orphan-operation", &changed)
                .await
                .is_err()
        );
        let revision = store
            .edit("problem", None, "orphan-operation", &edits)
            .await
            .unwrap();
        assert_eq!(
            store.revision("orphan-operation").await.unwrap(),
            Some(revision)
        );
        let metadata = store.reclaim_staging_page("problem", "").await.unwrap();
        assert_eq!(metadata.edits, 1);
        assert_eq!(metadata.blocks, 0);
        assert!(store.object("orphan-version").await.unwrap().is_some());
        remove(&store, &database).await;
    }
}
