// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Authored generator-routing protocol, shared by producers and the independent host via `#[path]`.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::{Component, Path};

/// The aggregate generator owns this exact generated record, including its own entry.
pub const MAP_PATH: &str = "scripts/generated/generated-outputs.json";
/// Identity of the aggregate map's own producer.
pub const MAP_PRODUCER: &str = "generated-output-map";
/// The native ABI producer's existing exact output; also the missing-map protection fallback.
pub const IPOPT_BINDINGS: &str = "crates/pse-ipopt-sys/src/bindings.rs";
/// Current routing wire interpretation.
pub const VERSION: u32 = 1;

/// A file owns only itself; a tree also owns descendants at path-component boundaries.
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ScopeKind {
    /// Exactly one emitted file or source input.
    File,
    /// A declared directory and its descendants.
    Tree,
}
/// Relative path scope declared by the real producer.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    /// Normal repository-relative path, without symlink or filesystem inference.
    pub path: String,
    /// Exact file or directory-tree interpretation.
    pub kind: ScopeKind,
}
impl Scope {
    /// Match native path components, rather than string prefixes.
    pub fn contains(&self, path: &Path) -> bool {
        path == Path::new(&self.path)
            || (self.kind == ScopeKind::Tree && path.starts_with(&self.path))
    }
}
/// Normal repository-relative paths are the only allowed routing destinations.
pub fn normal_relative(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

/// One actual generation responsibility, including its declared conservative input reach.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GeneratorOwner {
    /// Stable producer identity, independent of individual emitted declarations.
    pub producer: String,
    /// Derived from the real GeneratedTree roots, never a separate output inventory.
    pub outputs: Vec<Scope>,
    /// Existing regeneration command as argv, not shell source or automatic execution.
    pub regenerate: Vec<String>,
    /// Conservative actual generator/registry/declaration source scopes.
    pub inputs: Vec<Scope>,
    /// Existing producer-owned contributor sidecar, not another provenance index.
    pub contributors: Option<String>,
}
/// Last complete aggregate generation's ownership declarations.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OutputMap {
    /// Exact generated-record interpretation.
    pub schema_version: u32,
    /// Generated-file marker; this record is emitted through the ordinary tree writer.
    #[serde(rename = "$comment")]
    pub comment: String,
    /// Actual aggregate producer population, including this map's self-owner.
    pub owners: Vec<GeneratorOwner>,
    /// Limits of source reach, completeness and freshness claims.
    pub coverage_limits: Vec<String>,
}

/// An actual declaration contributor to an output, rather than inferred consumer reach.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
pub struct SourceRelationship {
    /// Registry semantic key, or native Rust catalogue type.
    pub declaration_key: String,
    /// Which declaration namespace gives the key meaning.
    pub declaration_kind: String,
    /// Exact known source owner; absent when the registry retains no source locations.
    pub source_path: Option<String>,
    /// Actual type or declaration symbol when supplied by its owner.
    pub source_symbol: Option<String>,
    /// Whether a path names a declaration definition, registration, or supplemental input.
    pub source_role: String,
    /// Producer identity, not a claim about current checkout source bytes.
    pub source_identity: Option<String>,
    /// Actual emitter owner, relative to the checkout.
    pub generator_path: String,
    /// Actual emitter entry point.
    pub generator_symbol: String,
    /// Exact emitted destination, relative to the checkout.
    pub output_path: String,
    /// Direction only when explicitly declared by a catalogue.
    pub direction: Option<String>,
    /// Stable architectural sections explicitly asserted by this producer.
    #[serde(default)]
    pub contract_sections: Vec<String>,
    /// Producer declarations are distinct from language resolution and text matches.
    pub basis: String,
}
/// Existing sidecar shape, shared without changing its emitted bytes or meaning.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct RelationshipDocument {
    /// Existing generated marker.
    #[serde(rename = "$comment")]
    pub comment: String,
    /// Sidecar wire interpretation.
    pub schema_version: u32,
    /// Original producer identity.
    pub producer: String,
    /// Original producer's lineage and source-location limits.
    pub coverage_limits: Vec<String>,
    /// Sole existing declaration contributor records.
    pub relationships: Vec<SourceRelationship>,
}
