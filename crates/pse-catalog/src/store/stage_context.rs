// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact durable semantic inputs. These values supplement, never replace, row admission.
use crate::session::SessionSemantics;
use pse_ids::SemanticId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A complete original source, including its package namespace and exact text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageSource {
    /// Owning declared package.
    pub package_id: SemanticId,
    /// Stable document identity.
    pub document_id: SemanticId,
    /// Exact package-relative path.
    pub path: String,
    /// Actual source text, independently reparsed before lookup.
    pub text: String,
}
/// One selected policy and its complete actual relation values.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StagePolicy {
    /// Selected actual policy key.
    pub policy_id: SemanticId,
    /// Qualified relation including schema version.
    pub relation: String,
    /// Complete rows using the existing lossless tagged Cell codec.
    pub rows: Vec<Vec<String>>,
}
/// Complete semantic values behind a durable lookup candidate.
/// Equality is direct value equality, including floating bits in tagged Cell values.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageContext {
    /// Complete self-describing registry relations in deterministic declared order.
    pub declarations: Vec<(String, Vec<Vec<String>>)>,
    /// Complete original source inventory, sorted by package, document and path.
    pub sources: Vec<StageSource>,
    /// Every explicitly selected policy, including selections that yield equal outputs.
    pub policies: BTreeMap<String, StagePolicy>,
    /// Actual sealed engine inputs, absent for passes that do not execute engine plans.
    pub engine: Option<SessionSemantics>,
}

/// Heap capacity retained by a complete stage context, including nested text and rows.
/// Vector and string capacities are counted directly; maps retain a conservative complete
/// node per occupied entry because their allocation capacity is not a public API.
/// This inspects existing values; producers must separately reserve their construction.
///
/// # Errors
/// Checked addressable-size arithmetic overflow.
pub fn heap_extent(value: &StageContext) -> Result<usize, crate::CatalogError> {
    super::stage_owned::context_extent(value)
}

/// Borrowed engine inputs, counted before their owned semantic snapshot is cloned.
pub(crate) fn engine_inputs_extent(
    engine_version_bytes: usize,
    arrow_version_bytes: usize,
    profile: &crate::session::EngineProfile,
    settings: &BTreeMap<String, Option<String>>,
    functions: &BTreeMap<String, Vec<String>>,
) -> Result<usize, crate::CatalogError> {
    super::stage_owned::engine_parts_extent(
        engine_version_bytes,
        arrow_version_bytes,
        profile,
        settings,
        functions,
    )
}
