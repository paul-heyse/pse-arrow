// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Library-owned structural differences, guarded against loss through an older decoder.
use super::super::{ApiError, Artifact};
use base64::Engine as _;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::io::Write as _;

/// Original input address and context, independent of numeric rustdoc item IDs.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Side {
    /// Exact source-qualified package identity.
    pub package_id: String,
    /// Original acquired artifact.
    pub artifact: std::path::PathBuf,
    /// Original source content identity.
    pub sha256: String,
    /// Full original acquisition-context identity.
    pub context_key: String,
}
impl Side {
    fn new(artifact: &Artifact) -> Result<Self, ApiError> {
        Ok(Self {
            package_id: artifact.context.package_id.clone(),
            artifact: artifact.path.clone(),
            sha256: artifact.sha256.clone(),
            context_key: artifact.context.key()?,
        })
    }
}
/// A whole native public-api result; it is not a bespoke compatibility verdict.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "change", rename_all = "snake_case")]
pub enum Difference {
    /// New public path/declaration under the selected native rendering policy.
    Added {
        /// Native declaration.
        declaration: String,
    },
    /// Former public path/declaration.
    Removed {
        /// Native declaration.
        declaration: String,
    },
    /// Same native public path with changed declaration/bounds.
    Changed {
        /// Original declaration.
        before: String,
        /// Current declaration.
        after: String,
    },
}
/// Producer-selected structural page over two original API artifacts.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Page {
    /// Exact performing normalization library and pin.
    pub backend: &'static str,
    /// Original baseline.
    pub baseline: Side,
    /// Original current side.
    pub current: Side,
    /// Extraction differences to assess before interpreting differences.
    pub differing_context_fields: Vec<&'static str>,
    /// Both selected typed documents survived the backend decoder without losing declaration data.
    pub lossless_typed_decoder_roundtrip: bool,
    /// Native unresolved item references, separately for each side.
    pub missing_items: [usize; 2],
    /// Number of native structural differences before paging.
    pub total_differences: usize,
    /// Whole producer-selected differences.
    pub differences: Vec<Difference>,
    /// Original differences not returned on this page.
    pub omitted_differences: usize,
    /// Resume this exact source/context/backend policy; changed premises refuse.
    pub continuation: Option<String>,
    /// Explicit native rendering and evidence boundaries.
    pub limitations: Vec<&'static str>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Cursor {
    version: u32,
    kind: String,
    identity: String,
    offset: usize,
}

fn native(artifact: &Artifact) -> Result<public_api::PublicApi, ApiError> {
    let original = super::super::artifact_bytes(artifact)?;
    let backend: public_api::rustdoc_types::Crate = serde_json::from_slice(&original)
        .map_err(|error| super::super::invalid(format!("structural backend cannot represent this format61 declaration: {error}; use original --declaration or native --compatibility")))?;
    let roundtrip = super::super::decode(&serde_json::to_vec(&backend)?).map_err(|error|
        super::super::invalid(format!("structural backend loses format61 declaration fields: {error}; use original --declaration or native --compatibility")))?;
    if roundtrip != artifact.document {
        return Err(super::super::invalid(
            "structural backend loses selected typed declaration data; use original --declaration or native --compatibility",
        ));
    }
    // The native builder accepts a filename, so supply private temporary captured bytes.
    // Original retained evidence and its digest stay separately addressed in the answer.
    let mut input = tempfile::NamedTempFile::new()?;
    input.write_all(&original)?;
    public_api::Builder::from_rustdoc_json(input.path())
        .omit_blanket_impls(true)
        .omit_auto_trait_impls(true)
        .omit_auto_derived_impls(true)
        .build()
        .map_err(|error| super::super::invalid(format!("native structural comparison: {error}")))
}
/// Compare with native path matching and normalization, never local item-ID equality.
pub fn inspect(
    baseline: &Artifact,
    current: &Artifact,
    limit: usize,
    cursor: Option<&str>,
) -> Result<Page, ApiError> {
    let before = Side::new(baseline)?;
    let after = Side::new(current)?;
    let identity = crate::probe::digest(&serde_json::to_vec(&(
        &before,
        &after,
        "public-api/0.52.2;omit-blanket,auto-trait,auto-derived",
    ))?);
    let offset = if let Some(cursor) = cursor {
        let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(cursor)
            .map_err(|_| super::super::invalid("invalid structural comparison cursor"))?;
        let cursor: Cursor = serde_json::from_slice(&bytes)?;
        if cursor.version != 1 || cursor.kind != "api_structural" || cursor.identity != identity {
            return Err(super::super::invalid(
                "structural comparison source/context/backend changed; restart against the original artifacts",
            ));
        }
        cursor.offset
    } else {
        0
    };
    let old = native(baseline)?;
    let new = native(current)?;
    let missing_items = [
        old.missing_item_ids().count(),
        new.missing_item_ids().count(),
    ];
    let diff = public_api::diff::PublicApiDiff::between(old, new);
    let total = diff.added.len() + diff.removed.len() + diff.changed.len();
    if offset > total {
        return Err(super::super::invalid(
            "structural comparison cursor exceeds the original result",
        ));
    }
    let differences: Vec<_> = diff
        .added
        .into_iter()
        .map(|item| Difference::Added {
            declaration: item.to_string(),
        })
        .chain(diff.removed.into_iter().map(|item| Difference::Removed {
            declaration: item.to_string(),
        }))
        .chain(diff.changed.into_iter().map(|item| Difference::Changed {
            before: item.old.to_string(),
            after: item.new.to_string(),
        }))
        .skip(offset)
        .take(limit)
        .collect();
    let next = offset + differences.len();
    let continuation = if limit > 0 && next < total {
        Some(
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(serde_json::to_vec(&Cursor {
                version: 1,
                kind: "api_structural".into(),
                identity,
                offset: next,
            })?),
        )
    } else {
        None
    };
    Ok(Page {
        backend: "public-api/0.52.2; decoder rustdoc-types/0.59.0 qualified per selected format61 document",
        baseline: before,
        current: after,
        differing_context_fields: super::differing_context_fields(
            &baseline.context,
            &current.context,
        ),
        lossless_typed_decoder_roundtrip: true,
        missing_items,
        total_differences: total,
        omitted_differences: total - differences.len(),
        differences,
        continuation,
        limitations: vec![
            "native renderer omits blanket, automatic-trait and automatically-derived impls; authored public declarations remain selected",
            "whole selected artifacts are decoded and normalized before paging; budgets bound whole response records, not source scan work",
            "structural differences are not a SemVer or behavioral compatibility verdict; explicit --compatibility uses separate native rules",
            "missing native item references make the structural account incomplete; no differences cannot establish compatibility",
        ],
    })
}
