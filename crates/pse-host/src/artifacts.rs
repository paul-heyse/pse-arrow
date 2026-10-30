// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Selected artifact reads through native object-store ranges and validators.
//! Object metadata is not process ownership or a cryptographic source identity.
use base64::{Engine as _, engine::general_purpose::STANDARD};
use object_store::{GetOptions, GetRange, ObjectStore, local::LocalFileSystem, path::Path};
use schemars::JsonSchema;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::num::NonZeroU64;

/// Explicit selected byte extent. `None` length requests all bytes from the offset.
#[derive(Clone, Debug)]
pub struct ReadRequest {
    /// Zero-based original object byte offset.
    pub offset: u64,
    /// Maximum selected bytes, or an explicit full remainder.
    pub length: Option<NonZeroU64>,
    /// Optional caller-supplied opaque validator from a prior observation.
    pub expected_etag: Option<String>,
}
impl Default for ReadRequest {
    fn default() -> Self {
        Self {
            offset: 0,
            length: NonZeroU64::new(4096),
            expected_etag: None,
        }
    }
}
/// Source bytes are never interpreted as compiler/test protocol by an artifact reader.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(tag = "encoding", content = "data", rename_all = "snake_case")]
pub enum Content {
    /// Exact valid UTF-8, retaining original whitespace and delimiters.
    Utf8(String),
    /// Exact bytes when UTF-8 decoding is unavailable.
    Base64(String),
}
/// A selected observation, with an explicit continuation rather than a silent full-file claim.
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct ArtifactRead {
    /// Backend-local validated Unicode object key, using the backend's native path contract.
    pub object_key: String,
    /// Source size reported by the actual object-store observation.
    pub object_bytes: u64,
    /// Opaque validator, not a content hash.
    pub etag: Option<String>,
    /// Provider-specific version when available.
    pub version: Option<String>,
    /// Actual half-open selected byte range.
    pub byte_range: std::ops::Range<u64>,
    /// Digest only of selected bytes, never silently attributed to the whole object.
    pub selected_sha256: String,
    /// Original selected bytes.
    pub content: Content,
    /// True only when this response contains every observed object byte.
    pub whole_object: bool,
    /// Offset for explicit continuation when bytes remain.
    pub next_offset: Option<u64>,
    /// Local mutable files have no snapshot-isolation guarantee, even with metadata validators.
    pub atomic_snapshot: bool,
}
/// Native source failures retain their error chains.
#[derive(Debug, thiserror::Error)]
pub enum ArtifactError {
    /// Original backend failure, including precondition/partial-stream errors.
    #[error(transparent)]
    Store(#[from] object_store::Error),
    /// Native filesystem key construction failed.
    #[error(transparent)]
    Path(#[from] object_store::path::Error),
    /// An invalid requested extent is not an empty artifact.
    #[error("artifact byte range is invalid: {0}")]
    Range(String),
}
pse_diagnostics::impl_diagnostic! {
    ArtifactError,
    code(_this) { Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure) },
    forward(_this) { None }, help(_this) { None }, related(_this) { None }, source(_this) { None }
}

/// Read one selected object extent. Remote backends can use the same native interface;
/// this function installs no provider, changes no authority and publishes no objects.
pub async fn read(
    store: &dyn ObjectStore,
    key: &Path,
    request: &ReadRequest,
) -> Result<ArtifactRead, ArtifactError> {
    let requested_end = request
        .length
        .map(|length| {
            request
                .offset
                .checked_add(length.get())
                .ok_or_else(|| ArtifactError::Range("offset plus length overflows".into()))
        })
        .transpose()?;
    let observed = store
        .get_opts(
            key,
            GetOptions {
                head: true,
                if_match: request.expected_etag.clone(),
                ..GetOptions::default()
            },
        )
        .await?;
    let mut meta = observed.meta;
    drop(observed.payload);
    if request.offset > meta.size {
        return Err(ArtifactError::Range(
            "offset exceeds the observed object size".into(),
        ));
    }
    let end = requested_end.unwrap_or(meta.size).min(meta.size);
    let (bytes, actual_range) = if end == request.offset {
        (bytes::Bytes::new(), request.offset..end)
    } else {
        let result = store
            .get_opts(
                key,
                GetOptions {
                    if_match: meta.e_tag.clone().or_else(|| request.expected_etag.clone()),
                    version: meta.version.clone(),
                    range: Some(GetRange::Bounded(request.offset..end)),
                    ..GetOptions::default()
                },
            )
            .await?;
        meta = result.meta.clone();
        let range = result.range.clone();
        (result.bytes().await?, range)
    };
    let selected_sha256 = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let content = match std::str::from_utf8(&bytes) {
        Ok(text) => Content::Utf8(text.into()),
        Err(_) => Content::Base64(STANDARD.encode(&bytes)),
    };
    Ok(ArtifactRead {
        object_key: key.to_string(),
        object_bytes: meta.size,
        etag: meta.e_tag,
        version: meta.version,
        whole_object: actual_range.start == 0 && actual_range.end == meta.size,
        next_offset: (actual_range.end < meta.size).then_some(actual_range.end),
        byte_range: actual_range,
        selected_sha256,
        content,
        atomic_snapshot: false,
    })
}

/// Observe an explicitly selected local file through the same object-store reader.
pub async fn read_file(
    source: &std::path::Path,
    request: &ReadRequest,
) -> Result<ArtifactRead, ArtifactError> {
    let key = Path::from_filesystem_path(source)?;
    read(&LocalFileSystem::new(), &key, request).await
}

#[cfg(test)]
mod tests;
