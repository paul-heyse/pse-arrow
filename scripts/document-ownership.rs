// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native document-owner export, shared with passive host consumers.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::ffi::OsString;

/// A stable section resolved by the native architecture reader.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Section {
    /// Native stable section identity.
    pub id: String,
    /// Authored heading.
    pub heading: String,
}
/// Existing edit workflow, advisory rather than edit authorization.
#[derive(Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Workflow {
    /// Decision/design route and blueprint revision row.
    ArchitectureDecision,
    /// Native proposed-decision workflow.
    ProposedDecision,
    /// Non-proposed decisions are immutable; use native supersession.
    SupersedeDecision,
    /// Follow the interpreted document owner's workflow.
    DocumentOwner,
}
/// Interpretation from the existing document lifecycle and section owners.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DocumentOwnership {
    /// Normalized native selected path, preserving platform bytes.
    pub path: OsString,
    /// Current document owner declared by native lifecycle interpretation.
    pub doc_owner: Option<String>,
    /// Historical reference, distinct from current ownership.
    pub historical_owner: Option<String>,
    /// Native interpreted role.
    pub doc_role: Option<String>,
    /// Native interpreted retention contract.
    pub doc_retention: Option<String>,
    /// Authored status, when declared.
    pub status: Option<String>,
    /// Stable sections owned by this exact source.
    pub sections: Vec<Section>,
    /// Workflow for the selected document's category.
    pub workflow: Workflow,
    /// Limits on the interpretation.
    pub coverage_limits: Vec<String>,
}

/// Original input identity used by the canonical document interpreter.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct InputEvidence {
    /// Repository-relative input path.
    pub path: String,
    /// Exact selected input bytes.
    pub sha256: String,
}
/// A citation is evidence of a written connection, not ownership or implementation.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Association {
    /// Native interpretation category.
    pub basis: String,
    /// Exact written link/citation.
    pub written: String,
    /// Native canonical target, absent when unresolved.
    pub resolved: Option<String>,
    /// Original rejection/unknown reason, without dropping the citation.
    pub limitation: Option<String>,
}
/// Passive canonical associations from one selected working/index/revision source universe.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AssociationExport {
    /// Original selected document path.
    pub path: OsString,
    /// Working, index or exact resolved commit identity.
    pub context: String,
    /// Selected document byte identity.
    pub source_sha256: String,
    /// Native interpreted document metadata/workflow, when all required inputs exist.
    pub document: Option<DocumentOwnership>,
    /// Native metadata provenance, including defaults/exceptions.
    pub metadata_origins: serde_json::Value,
    /// Explicit native scope-table records, not a copied status ledger.
    pub scope_items: Vec<serde_json::Value>,
    /// Explicit links/citations with their original basis and resolution.
    pub associations: Vec<Association>,
    /// Original selected policy/configuration/document inputs.
    pub inputs: Vec<InputEvidence>,
    /// Missing inputs/unsupported interpretation remain explicit.
    pub limitations: Vec<String>,
}
