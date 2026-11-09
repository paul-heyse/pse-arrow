// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Fixed semantic operations over a reusable, originally owned rust-analyzer session.
mod context;
mod edit;
mod engine;
mod facts;
mod graph;
pub use graph::*;
mod investigation;
pub use investigation::*;
mod layout;
pub use layout::{LayoutAssociation, LayoutAssociationState, associate_layout};
mod native_usage_contract;
pub use native_usage_contract::dependency_usage_projection;
mod protocol_facts;
pub use facts::*;
#[cfg(feature = "native-hir")]
pub mod native;
pub(crate) mod rpc;
mod rpc_failure;
#[cfg(test)]
mod rpc_failure_tests;
pub use rpc_failure::{RpcDisposition, RpcFailure, RpcObservation};
pub(crate) mod service;
#[cfg(test)]
pub(crate) mod tests;
mod transport;
mod wire;
mod workers;
use crate::source_proposal::ChangeProposal;
pub use rpc::Client;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
pub use service::{
    SessionOverview, SessionRecord, SessionStatus, client, client_cancellable, load,
    preparation_lock, prepare_record, serve, socket_path, stop,
};
use std::path::PathBuf;

/// Current prepared semantic helper and selector protocol.
pub const SESSION_PROTOCOL_VERSION: u32 = 10;

pse_contract::projection_enum! {
/// Explicit context owner; immutable native snapshots are not a second live LSP world.
#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize, JsonSchema, clap::ValueEnum,
)]
#[serde(rename_all = "snake_case")]
pub enum Backend {
    projection: { name: "NativeAnalysisBackend", rust_path: crate::semantic::Backend, version: 1, direction: pse_contract::BoundaryDirection::Both, reserved_tags: [], reserved_names: [] };
    /// The explicitly prepared reusable rust-analyzer LSP session.
    #[default]
    Lsp => 1;
    /// An explicitly prepared immutable compatible-family HIR/IDE snapshot.
    NativeSnapshot => 2;
}
}

pse_contract::projection_struct! {
/// Selected analysis context; LSP cannot choose an exact Cargo compiler-unit instance.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    projection: { name: "NativeConfiguration", rust_path: crate::semantic::Configuration, version: 1, direction: pse_contract::BoundaryDirection::Both, reserved_tags: [], reserved_names: [] };
    /// Selected semantic context and provider authority.
    #[serde(default)]
    pub backend: Backend => { tag: 1, direction: pse_contract::BoundaryDirection::Both, rules: [] };
    /// Canonical Cargo project root, independent of the owning checkout.
    pub project: PathBuf => { tag: 2, direction: pse_contract::BoundaryDirection::Both, rules: [] };
    /// Explicit feature names passed to the analyzer's Cargo model.
    pub features: Vec<String> => { tag: 3, direction: pse_contract::BoundaryDirection::Both, rules: [] };
    /// Disable Cargo defaults in this analyzer model.
    pub no_default_features: bool => { tag: 4, direction: pse_contract::BoundaryDirection::Both, rules: [] };
    /// Explicit compilation target, when selected.
    pub target: Option<String> => { tag: 5, direction: pse_contract::BoundaryDirection::Both, rules: [] };
    /// Explicitly admit native build-script and proc-macro preparation through correctness Cargo.
    pub build_macros: bool => { tag: 6, direction: pse_contract::BoundaryDirection::Both, rules: [] };
    /// Explicitly include native cfg(test) semantics; never compiler-unit equivalence.
    #[serde(default)]
    pub test_context: bool => { tag: 7, direction: pse_contract::BoundaryDirection::Both, rules: [] };
}
}
impl Configuration {
    /// Validate and canonicalize before preparation; reads do not prepare a session.
    pub fn resolve(mut self) -> Result<Self> {
        if self.test_context && self.backend != Backend::NativeSnapshot {
            return Err(Error::unsupported(
                "explicit test_context is a native snapshot selection; live LSP retains its separately prepared configuration",
            ));
        }
        self.project = self.project.canonicalize()?;
        if !self.project.join("Cargo.toml").is_file() {
            return Err(Error::Unavailable(
                "selected project has no Cargo.toml".into(),
            ));
        }
        self.features.sort();
        self.features.dedup();
        Ok(self)
    }
    /// Content identity of explicitly selected context, independent of Git HEAD.
    pub fn identity(&self) -> Result<String> {
        Ok(crate::source_read::source_digest(&serde_json::to_string(
            self,
        )?))
    }
}
/// Fixed requests; clients cannot choose arbitrary protocol methods or executable arguments.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    /// Session state without a semantic query.
    Status {},
    /// Request original owner teardown; acknowledgement is not native drain evidence.
    Stop {},
    /// Selected semantic producer.
    Query(Query),
    /// Name-first workspace discovery in the existing explicitly prepared world.
    WorkspaceSymbols(WorkspaceQuery),
    /// One bounded relationship investigation in one original native snapshot.
    Graph(GraphQuery),
    /// Batch resolved workspace references to exact dependency packages.
    DependencyUsage(DependencyUsageQuery),
    /// Read selected package facts from the already loaded native Cargo/VFS world.
    PackageContext {
        /// Exact Cargo source-qualified package identity.
        package_id: String,
    },
}
pse_contract::projection_struct! {
/// Exact source-qualified Cargo dependency packages in one batch census.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DependencyUsageQuery {
    projection: { name: "NativeDependencyUsageQuery", rust_path: crate::semantic::DependencyUsageQuery, version: 1, direction: pse_contract::BoundaryDirection::Both, reserved_tags: [], reserved_names: [] };
    /// Exact source-qualified Cargo package identifiers.
    pub package_ids: Vec<String> => { tag: 1, direction: pse_contract::BoundaryDirection::Both, rules: [pse_contract::boundary_projection::LocalRule::Length { min: 1, max: u32::MAX as u64 }] };
}
}

pse_contract::projection_struct! {
/// Actual library crate context captured from the same prepared native database.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
pub struct NativeLibraryProfile {
    projection: { name: "NativeLibraryProfile", rust_path: crate::semantic::NativeLibraryProfile, version: 1, direction: pse_contract::BoundaryDirection::Both, reserved_tags: [], reserved_names: [] };
    /// Original source-qualified Cargo package identifier.
    pub package_id: String => { tag: 1, direction: pse_contract::BoundaryDirection::Both, rules: [] };
    /// Original package manifest.
    pub manifest: PathBuf => { tag: 2, direction: pse_contract::BoundaryDirection::Both, rules: [] };
    /// Original package source root.
    pub source_root: PathBuf => { tag: 3, direction: pse_contract::BoundaryDirection::Both, rules: [] };
    /// Actual loaded library root.
    pub library_root: PathBuf => { tag: 4, direction: pse_contract::BoundaryDirection::Both, rules: [] };
    /// Effective feature atoms of this exact native crate.
    pub features: std::collections::BTreeSet<String> => { tag: 5, direction: pse_contract::BoundaryDirection::Both, rules: [] };
    /// Actual loaded target, absent for unqualified host contexts.
    pub target: Option<String> => { tag: 6, direction: pse_contract::BoundaryDirection::Both, rules: [] };
    /// Actual library/proc-macro/test role.
    pub role: String => { tag: 7, direction: pse_contract::BoundaryDirection::Both, rules: [] };
}
}
/// Actual library crate context captured from the same prepared native database.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum NativeDependencyExclusion {
    /// Actual native builtin sysroot crates have no ordinary Cargo documentation-unit node.
    Sysroot,
}
/// Actual library crate context captured from the same prepared native database.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct NativePackageContext {
    /// Edition of the actual loaded crate; absent only for an unmapped model.
    pub edition: Option<String>,
    /// Actual native cfg atoms, including key/value pairs.
    pub cfg: Vec<String>,
    /// Cargo source-qualified package identifier, never a bare crate spelling.
    pub package_id: String,
    /// Original package manifest from the loaded project model.
    pub manifest: PathBuf,
    /// Package source directory; may be a prepared dependency outside the project.
    pub source_root: PathBuf,
    /// Workspace root used by the loaded Cargo project model.
    pub workspace_root: PathBuf,
    /// Actual native Cargo library root in the loaded VFS.
    pub library_root: PathBuf,
    /// Actual feature atoms supplied to this loaded native crate.
    pub features: std::collections::BTreeSet<String>,
    /// Effective target fed to the native project loader; unknown host variants stay absent.
    pub target: Option<String>,
    /// Cargo library role: `lib`, `proc-macro`, or an explicitly distinct test context.
    pub role: String,
    /// Original loaded source bytes, checked by the immutable context owner.
    pub source_hashes: std::collections::BTreeMap<PathBuf, String>,
    /// Actual transitive non-sysroot library dependency profiles from this same HIR graph.
    pub dependencies: Vec<NativeLibraryProfile>,
    /// Unmapped/unsupported dependency contexts prevent inventing graph equivalence.
    pub dependency_issues: Vec<String>,
    /// Explicit successful projection exclusions, distinct from unqualified dependencies.
    pub dependency_exclusions: std::collections::BTreeSet<NativeDependencyExclusion>,
}
/// Package observations retain the same session/configuration/generation authority.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PackageContextAnswer {
    /// Actual prepared backend and configuration.
    pub session: AnalysisContext,
    /// Immutable native-world generation.
    pub generation: u64,
    /// All actual matching library contexts; ambiguity is not collapsed.
    pub packages: Vec<NativePackageContext>,
}
pse_contract::projection_struct! {
/// A source-qualified observed reference in the original native graph.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DependencyReference {
    projection: { name: "NativeDependencyReference", rust_path: crate::semantic::DependencyReference, version: 1, direction: pse_contract::BoundaryDirection::Both, reserved_tags: [], reserved_names: [] };
    /// Original declaration location in the selected VFS world.
    pub definition: NativeLocation => { tag: 1, direction: pse_contract::BoundaryDirection::Both, rules: [pse_contract::boundary_projection::LocalRule::Required] };
    /// All captured dependency profiles whose source root contains the declaration.
    pub profiles: Vec<NativeLibraryProfile> => { tag: 2, direction: pse_contract::BoundaryDirection::Both, rules: [] };
    /// Original written workspace occurrence.
    pub occurrence: NativeLocation => { tag: 3, direction: pse_contract::BoundaryDirection::Both, rules: [pse_contract::boundary_projection::LocalRule::Required] };
    /// Digest of the occurrence's original snapshot bytes.
    pub source_sha256: String => { tag: 4, direction: pse_contract::BoundaryDirection::Both, rules: [] };
    /// Upstream symbol category, retained for presentation only.
    pub symbol_kind: String => { tag: 5, direction: pse_contract::BoundaryDirection::Both, rules: [] };
    /// Resolved declaration label, when available.
    pub name: Option<String> => { tag: 6, direction: pse_contract::BoundaryDirection::Both, rules: [] };
    /// Syntactic occurrence role, not an execution classification.
    pub role: String => { tag: 7, direction: pse_contract::BoundaryDirection::Both, rules: [] };
}
}
pse_contract::projection_struct! {
/// One batch census in one explicitly prepared immutable native world.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DependencyUsageAnswer {
    projection: { name: "NativeDependencyUsageAnswer", rust_path: crate::semantic::DependencyUsageAnswer, version: 1, direction: pse_contract::BoundaryDirection::Both, reserved_tags: [], reserved_names: [] };
    /// Actual selected analysis context.
    pub session: AnalysisContext => { tag: 1, direction: pse_contract::BoundaryDirection::Both, rules: [pse_contract::boundary_projection::LocalRule::Required] };
    /// Immutable world generation.
    pub generation: u64 => { tag: 2, direction: pse_contract::BoundaryDirection::Both, rules: [] };
    /// Exact requested Cargo package IDs.
    pub package_ids: Vec<String> => { tag: 3, direction: pse_contract::BoundaryDirection::Both, rules: [] };
    /// Requested packages absent from the selected native graph, never zero-use verdicts.
    pub unavailable_package_ids: Vec<String> => { tag: 8, direction: pse_contract::BoundaryDirection::Both, rules: [] };
    /// Resolved reference observations, excluding definition occurrences.
    pub references: Vec<DependencyReference> => { tag: 4, direction: pse_contract::BoundaryDirection::Both, rules: [] };
    /// Number of source files indexed by the upstream batch operation.
    pub indexed_files: usize => { tag: 5, direction: pse_contract::BoundaryDirection::Both, rules: [] };
    /// Resolved symbols for which no original definition mapping was available.
    pub unmapped_definitions: usize => { tag: 6, direction: pse_contract::BoundaryDirection::Both, rules: [] };
    /// Explicit scope and interpretation limits.
    pub limitations: Vec<String> => { tag: 7, direction: pse_contract::BoundaryDirection::Both, rules: [] };
}
}
/// Policies directly supported by the selected native symbol index.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NativeNameMode {
    /// Library-owned fuzzy candidate matching.
    #[default]
    Fuzzy,
    /// Case-sensitive exact item matching and native module-path resolution.
    Exact,
    /// Case-sensitive item prefix matching and native module-path resolution.
    Prefix,
}
/// A name query does not require guessing a source file or position.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceQuery {
    /// Native analyzer name query; provider ranking/ambiguity remain visible.
    pub query: String,
    /// Actual native matching policy; display container labels never establish qualification.
    #[serde(default)]
    pub matching: NativeNameMode,
    /// Whole native candidate allowance.
    pub limit: usize,
    /// Include dependency-library roots from this same prepared native world.
    #[serde(default)]
    pub include_dependencies: bool,
    /// Optional original-session generation guard.
    pub if_generation: Option<u64>,
}
/// Native workspace candidates keep their selected configuration and generation.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct WorkspaceAnswer {
    /// Actual reusable provider and configuration, not source-only parser context.
    pub session: AnalysisContext,
    /// Actual synchronized session generation.
    pub generation: u64,
    /// Native candidates, without inventing typed fields from hover text.
    pub symbols: Section,
    /// Typed native candidates qualified by this original session and generation.
    pub observed: Vec<SymbolFact>,
    /// Unknown compiler-unit/exhaustiveness remains explicit.
    pub limitations: Vec<String>,
}
/// Explicit source question, with one-based Unicode scalar CLI coordinates.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Query {
    /// Fixed semantic operation.
    pub kind: Kind,
    /// Original working source, within the selected project.
    pub file: PathBuf,
    /// One-based line.
    pub line: usize,
    /// One-based Unicode scalar column, excluding a newline.
    pub column: usize,
    /// Optional precondition on original source bytes.
    pub if_source: Option<String>,
    /// Expected session generation, required for selected lazy handles.
    pub if_generation: Option<u64>,
    /// Exact discovery candidate fingerprint; indexes alone cannot identify a lazy action.
    pub if_candidate: Option<String>,
    /// Whole record limit; no records are recursively truncated.
    pub limit: usize,
    /// Optional inclusive final line for selected range annotations.
    pub end_line: Option<usize>,
    /// Selected occurrence domain; None leaves native visibility scope unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub occurrence_scope: Option<Vec<OccurrenceFile>>,
}
/// One same-world occurrence file with an optional contiguous UTF-8 byte range.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OccurrenceFile {
    /// Original file, resolved within the prepared world.
    pub file: PathBuf,
    /// None selects the entire file.
    pub range: Option<std::ops::Range<usize>>,
}
/// Supported backend operations, distinct from server-free syntax queries.
#[derive(Clone, Debug, Serialize, Deserialize, clap::ValueEnum, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Kind {
    /// Resolve authored Rust intra-documentation links in the selected native declaration.
    DocumentationLinks,
    /// Selected body observations from the same immutable snapshot.
    Body,
    /// Original format-string or derive provenance.
    #[value(skip)]
    Origins {
        /// Native original-source origin family.
        #[serde(rename = "origin_kind")]
        kind: OriginKind,
    },
    /// Compare actual types under a compatible generic owner.
    #[value(skip)]
    Compatibility {
        /// Original guarded operand in the same prepared world.
        against: ImportTarget,
        /// Session-local generation guard retained from the operand.
        against_generation: Option<u64>,
        /// Exact native operation requested by the caller.
        operation: CompatibilityOperation,
    },
    /// Native visibility from an original destination module.
    #[value(skip)]
    Visibility {
        /// Original guarded location whose module supplies the context.
        destination: ImportTarget,
        /// Session-local generation guard retained from the destination.
        destination_generation: Option<u64>,
    },
    /// Hover plus declaration/type targets.
    Explain,
    /// Fallible native target-qualified layout with immediate child detail.
    Layout,
    /// Semantic references, including declarations.
    Refs,
    /// One-hop incoming and outgoing call hierarchy.
    Calls,
    /// Implementations at the selected source position.
    Implementations,
    /// Selected macro expansion with invocation fallback.
    Macro,
    /// Related test candidates; never executes analyzer-produced argv.
    Tests,
    /// Contextual completion candidates, including unresolved snippets.
    Candidates,
    /// Document symbols from the semantic provider.
    Symbols,
    /// Selected native inlay annotations.
    Hints,
    /// Resolve a selected generation-bound inlay annotation.
    #[value(skip)]
    Hint {
        /// Original native hint index.
        index: usize,
    },
    /// Preview native semantic search and replace in its selected resolution context.
    #[value(skip)]
    Ssr {
        /// Native SSR rule including its ==>> replacement.
        rule: String,
    },
    /// Native path/visibility proof for an original target from the selected destination.
    #[value(skip)]
    Importable {
        /// Original native declaration range and exact source bytes.
        target: ImportTarget,
    },
    /// Native predicate question with distinct holds/not-proven/unsupported outcomes.
    #[value(skip)]
    Predicate {
        /// Rust predicate text resolved at the selected source context.
        text: String,
    },
    /// Resolve a selected source-bound completion candidate lazily.
    #[value(skip)]
    Candidate {
        /// Original completion candidate index.
        index: usize,
    },
    /// Rename preview, never filesystem application.
    #[value(skip)]
    Rename {
        /// New identifier supplied to native rename.
        name: String,
    },
    /// Native file-rename follow-up edits plus the explicit source-bound filesystem proposal.
    #[value(skip)]
    FileRename {
        /// Destination regular file; no file is moved by this operation.
        destination: PathBuf,
    },
    /// Discover code actions without resolving every edit.
    Assists,
    /// Resolve a selected native action and preview its edits.
    #[value(skip)]
    Assist {
        /// Zero-based candidate index in the original source-bound action set.
        index: usize,
    },
}
/// Plain guarded declaration coordinates; native AST/HIR handles never cross the boundary.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImportTarget {
    /// Canonical original declaration source in the same selected native world.
    pub file: PathBuf,
    /// Start of the original native declaration, in UTF-8 bytes.
    pub byte_start: usize,
    /// Exclusive end of the original native declaration, in UTF-8 bytes.
    pub byte_end: usize,
    /// SHA-256 of the exact original target file, checked against the native snapshot.
    pub source_sha256: String,
}
/// Native protocol result retained as opaque backend evidence, with whole-record budgeting.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Section {
    /// Actual native result category.
    pub category: String,
    /// Native fields retain their original backend meaning and coordinate encoding.
    pub evidence: serde_json::Value,
    /// Exact known records excluded from this presentation.
    pub omitted: usize,
    /// Source/generation-bound discovery handles for lazy selection; no permanent symbol identity.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub handles: Vec<CandidateHandle>,
}
/// Stateless identity of one original native discovery record.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CandidateHandle {
    /// Index in the original producer's complete native ordering.
    pub index: usize,
    /// Exact original native record fingerprint, checked after reacquisition.
    pub fingerprint: String,
}
/// One source-bound semantic response. Analysis is not compiler acceptance.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Answer {
    /// Actual session/context and backend initialization evidence.
    pub session: AnalysisContext,
    /// Exact input source identity.
    pub source: crate::source_read::SourceIdentity,
    /// Actual canonical backend URI, independent of the caller's operand spelling.
    pub source_uri: String,
    /// Exact selected input byte offset, in the identified original source.
    pub source_byte_offset: usize,
    /// A working provider can report no match within its explicitly selected model.
    pub availability: Availability,
    /// Session revision observed before and after the operation.
    pub generation: u64,
    /// Negotiated protocol position, distinct from CLI scalar columns.
    pub position: ProtocolPosition,
    /// Consumed native observations; opaque documents never drive caller decisions.
    pub observed: ObservedFacts,
    /// Exact known excluded relationship records, grouped by native operation.
    pub omitted_relationships: std::collections::BTreeMap<FactRelation, usize>,
    /// Native immutable-world type observations, when this provider supports them.
    pub native_types: Option<TypedFacts>,
    /// Whole native views, each retaining the provider's scope.
    pub sections: Vec<Section>,
    /// Exact original bytes for native source-linked result locations.
    pub location_sources: Vec<LocationSource>,
    /// Checked immutable source-bound edit proposal, when requested.
    pub proposal: Option<ChangeProposal>,
    /// Explicit interpretation/preparation limits.
    pub limitations: Vec<String>,
}
/// A provider-observed location source, retained separately from its native location payload.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct LocationSource {
    /// Canonical original path in the selected provider context.
    #[serde(with = "crate::investigation::native_path_serde")]
    #[schemars(with = "crate::investigation::NativePath")]
    pub path: PathBuf,
    /// Original byte digest used when the native result was produced.
    pub source_sha256: String,
}
/// Result presence is independent of backend preparation and compiler correctness.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Availability {
    /// Selected native evidence or edit candidates exist.
    Available,
    /// No match in this selected model; declared preparation/scope limits still apply.
    NoMatch,
}
/// Compact mandatory query context; full actual capabilities belong to session status.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct AnalysisContext {
    /// Actual analyzer identity.
    pub backend: String,
    /// Explicit project/features/target/preparation selection.
    pub configuration: Configuration,
    /// Negotiated native coordinate unit.
    pub position_encoding: String,
    /// Native load/health notification; absent remains unknown.
    pub backend_status: Option<serde_json::Value>,
    /// Actual supported preparation boundary.
    pub preparation: String,
}
/// Convert native positions through the existing negotiated coordinate owner.
pub fn native_byte_position(
    text: &str,
    position: &serde_json::Value,
    encoding: &str,
) -> Result<usize> {
    engine::byte_position(text, position, encoding)
}
/// Convert typed native protocol coordinates in the original negotiated encoding.
pub fn native_byte_protocol_position(
    text: &str,
    position: &ProtocolPosition,
    encoding: &str,
) -> Result<usize> {
    engine::byte_protocol_position(text, position, encoding)
}
/// Serialized service response; a failed query does not terminate the session.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "state", content = "answer", rename_all = "snake_case")]
pub enum Response {
    /// Actual prepared analyzer status.
    Status(SessionStatus),
    /// Settled selected source query.
    Query(Box<Answer>),
    /// Actual name-first native discovery.
    WorkspaceSymbols(WorkspaceAnswer),
    /// Settled bounded native relationship evidence.
    Graph(GraphAnswer),
    /// Settled batch reference observations.
    DependencyUsage(DependencyUsageAnswer),
    /// Qualified loaded Cargo/VFS package context.
    PackageContext(PackageContextAnswer),
    /// Stop accepted by the session owner; teardown remains session-owned.
    Stopped,
}
impl Response {
    /// Extract a batch dependency census.
    pub fn into_dependency_usage(self) -> Result<DependencyUsageAnswer> {
        match self {
            Self::DependencyUsage(value) => Ok(value),
            _ => Err(Error::Protocol("expected dependency usage response".into())),
        }
    }
    /// Extract a bounded relationship investigation.
    pub fn into_graph(self) -> Result<GraphAnswer> {
        match self {
            Self::Graph(value) => Ok(value),
            _ => Err(Error::Protocol("expected native graph response".into())),
        }
    }
    /// Extract the selected source answer, refusing a different operation result.
    pub fn into_query(self) -> Result<Answer> {
        match self {
            Self::Query(value) => Ok(*value),
            _ => Err(Error::Protocol("expected native query response".into())),
        }
    }
    /// Extract actual analyzer status.
    pub fn into_status(self) -> Result<SessionStatus> {
        match self {
            Self::Status(value) => Ok(value),
            _ => Err(Error::Protocol("expected native status response".into())),
        }
    }
    /// Extract actual native name candidates.
    pub fn into_workspace(self) -> Result<WorkspaceAnswer> {
        match self {
            Self::WorkspaceSymbols(value) => Ok(value),
            _ => Err(Error::Protocol("expected native workspace response".into())),
        }
    }
    /// Extract the selected native Cargo package observation.
    pub fn into_package_context(self) -> Result<PackageContextAnswer> {
        match self {
            Self::PackageContext(value) => Ok(value),
            _ => Err(Error::Protocol(
                "expected native package-context response".into(),
            )),
        }
    }
}
/// Query refusal preserves its semantic state across the private transport.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, thiserror::Error)]
#[error("{message}")]
pub struct Failure {
    /// Original diagnostic identity, independent of transport status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnostic_code: Option<String>,
    /// Native-owned contextual classification, independent of opaque backend evidence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<FailureContext>,
    /// Owner-classified refusal, never inferred by parsing hover/debug text.
    pub state: FailureState,
    /// Meaningful failure boundary, independent of native outcome state.
    pub phase: FailurePhase,
    /// Original explanatory message, under the normal redaction policy.
    pub message: String,
    /// Original JSON-RPC error code when the native peer supplied one.
    pub backend_code: Option<i32>,
    /// Original backend data, retained as opaque native evidence.
    pub backend_data: Option<serde_json::Value>,
    /// Typed observed settlement; original provider data never supplies this control fact.
    pub observed_cancellation: Option<Box<CancellationObservation>>,
    /// Schema-addressed correction derived from canonical details and origin disclosure.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation: Option<Box<crate::envelope::ValidationCorrection>>,
}
/// Typed native prerequisite or context failure.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct FailureContext {
    /// Original classification.
    pub reason: FailureReason,
    /// Operation that established the failure.
    pub operation: String,
    /// Original operand where known.
    pub operand: Option<crate::investigation::NativePath>,
}
pse_contract::projection_enum! {
/// Context meaning supplied by its actual native owner.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FailureReason {
    projection: { name: "FailureReason", rust_path: crate::semantic::FailureReason, version: 1, direction: pse_contract::BoundaryDirection::Output, reserved_tags: [], reserved_names: [] };
    /// Prepared tool is absent.
    ToolUnavailable => 1;
    /// Selected session is absent or drained.
    SessionUnavailable => 2;
    /// Prepared configuration changed.
    ConfigurationChanged => 3;
    /// Original source changed.
    SourceChanged => 4;
    /// File is outside the prepared world.
    OutsidePreparedWorld => 5;
    /// File has no active module.
    InactiveSource => 6;
    /// No precise original mapping exists.
    OriginalMappingUnavailable => 7;
    /// Selected fact is unsupported.
    UnsupportedFact => 8;
    /// Native provider failed.
    ProviderFailed => 9;
}
}
pse_contract::projection_enum! {
/// Distinguish explicit preparation, unsupported capability, cancellation and revision invalidation.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FailureState {
    projection: { name: "FailureState", rust_path: crate::semantic::FailureState, version: 1, direction: pse_contract::BoundaryDirection::Output, reserved_tags: [], reserved_names: [] };
    /// Explicit preparation is missing/pending or another selected premise is unavailable.
    Unavailable => 1;
    /// Selected backend does not implement the requested operation.
    Unsupported => 2;
    /// Abandoned request; session lifetime remains independent.
    Cancelled => 3;
    /// Source/configuration changed; reacquire inputs.
    Stale => 4;
    /// Original protocol, source or native owner failed.
    Failed => 5;
}
}
pse_contract::projection_enum! {
/// Original failure boundary, separate from native availability/cancellation state.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FailurePhase {
    projection: { name: "FailurePhase", rust_path: crate::semantic::FailurePhase, version: 1, direction: pse_contract::BoundaryDirection::Output, reserved_tags: [], reserved_names: [] };
    /// A schema representation could not be decoded or did not match its contract.
    Schema => 1;
    /// The user's request violated a declared constraint or stream-control contract.
    Constraint => 2;
    /// Original source/session/provider context is unavailable or stale.
    Context => 3;
    /// Validator execution failed rather than rejecting the user's request.
    Validator => 4;
    /// Original selected provider failure.
    Provider => 5;
    /// Native result delivery or connection failed.
    Transport => 6;
    /// Cancellation observation retains actual original work settlement.
    Settlement => 7;
}
}
impl Failure {
    /// Preserve selected typed error categories at the serialization boundary.
    pub fn from_error(error: &Error) -> Self {
        Self::from_error_with_policy(error, &crate::envelope::RedactionPolicy::from_environment())
    }
    fn from_error_with_policy(error: &Error, policy: &crate::envelope::RedactionPolicy) -> Self {
        if let Error::Rpc(failure) = error {
            return failure.failure(policy);
        }
        if let Error::Refusal(failure) = error {
            return failure.as_ref().clone().redacted(policy);
        }
        let state = match error {
            Error::Stale => FailureState::Stale,
            Error::Cancelled | Error::CancelledObserved { .. } => FailureState::Cancelled,
            Error::Unavailable(_) => FailureState::Unavailable,
            _ => FailureState::Failed,
        };
        let phase = error.phase();
        Self {
            diagnostic_code: pse_diagnostics::TypedDiagnostic::diagnostic_code(error)
                .map(|code| code.as_str().to_owned()),
            state,
            context: None,
            phase,
            message: error.to_string(),
            backend_code: None,
            backend_data: None,
            validation: None,
            observed_cancellation: match error {
                Error::CancelledObserved {
                    completion,
                    original,
                } => Some(Box::new(CancellationObservation {
                    completion: *completion,
                    original: original.clone(),
                })),
                _ => None,
            },
        }
        .redacted(policy)
    }
    fn redacted(mut self, policy: &crate::envelope::RedactionPolicy) -> Self {
        fn backend_data(value: &mut serde_json::Value, policy: &crate::envelope::RedactionPolicy) {
            match value {
                serde_json::Value::String(text) => *text = policy.redact(text),
                serde_json::Value::Array(values) => {
                    for value in values {
                        backend_data(value, policy);
                    }
                }
                serde_json::Value::Object(object) => {
                    let mut changed_keys = Vec::new();
                    for (key, mut value) in std::mem::take(object) {
                        backend_data(&mut value, policy);
                        let redacted = policy.redact(&key);
                        if redacted == key {
                            object.insert(key, value);
                        } else {
                            changed_keys.push((redacted, value));
                        }
                    }
                    // Reserve unchanged public keys and retain every colliding value. The
                    // suffix identifies a disclosure-generated name, never an original key.
                    for (redacted, value) in changed_keys {
                        let mut key = redacted.clone();
                        let mut collision = 1;
                        while object.contains_key(&key) {
                            key = format!("{redacted}<redacted-key-collision:{collision}>");
                            collision += 1;
                        }
                        object.insert(key, value);
                    }
                }
                _ => {}
            }
        }
        self.diagnostic_code = self.diagnostic_code.map(|code| policy.redact(&code));
        self.validation = self
            .validation
            .map(|value| Box::new(value.redacted(policy)));
        self.message = policy.redact(&self.message);
        if let Some(value) = &mut self.backend_data {
            backend_data(value, policy);
        }
        if let Some(context) = &mut self.context {
            context.operation = policy.redact(&context.operation);
            context.operand = context.operand.take().map(|path| policy.redact_path(path));
        }
        if let Some(observed) = &mut self.observed_cancellation
            && let OriginalOutcome::Failed(original) = &mut observed.original
        {
            **original = original.as_ref().clone().redacted(policy);
        }
        self
    }
}
/// Original native settlement, independent of response delivery.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Settlement {
    /// Actual CPU work was joined.
    CpuSettled,
    /// Live protocol work detached after cancellation; native completion is unknown.
    LspDetached,
    /// Original live protocol request returned.
    LspReturned,
}
/// Settled original result retained independently from a cancellation request.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "outcome", content = "failure", rename_all = "snake_case")]
pub enum OriginalOutcome {
    /// Original work completed successfully before cancellation settled.
    Succeeded,
    /// Original failure, including its typed native category.
    Failed(Box<Failure>),
}
/// Cancellation delivery observed with the original native settlement and outcome.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct CancellationObservation {
    /// Actual original native work ownership state.
    pub completion: Settlement,
    /// Original result, independent from the cancellation request.
    pub original: OriginalOutcome,
}
/// Semantic operation errors preserve input and native protocol causes.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Native local I/O.
    #[error("semantic I/O: {0}")]
    Io(#[from] std::io::Error),
    /// Fixed transport or typed payload conversion.
    #[error("semantic JSON: {0}")]
    Json(#[from] serde_json::Error),
    /// Exact source acquisition and position failures.
    #[error(transparent)]
    Source(#[from] crate::source_read::SourceReadError),
    /// Checked edit proposal failure.
    #[error(transparent)]
    Proposal(#[from] crate::source_proposal::ProposalError),
    /// Existing native ownership/admission failure.
    #[error("semantic native owner: {0}")]
    Host(#[from] Box<crate::HostError>),
    /// Missing explicit preparation or unsupported capability.
    #[error("semantic preparation/capability unavailable: {0}")]
    Unavailable(String),
    /// Native LSP failure; no fallback response is fabricated.
    #[error("semantic protocol: {0}")]
    Protocol(String),
    /// Invalid user request or unsatisfied declared request constraint.
    #[error("semantic request schema/constraint: {0}")]
    InvalidRequest(String),
    /// Original schema/decode mismatch, distinct from user constraint violations.
    #[error("semantic schema: {0}")]
    Schema(String),
    /// Original protobuf representation decode failure.
    #[error("semantic protobuf: {0}")]
    WireDecode(#[from] buffa::DecodeError),
    /// Validator fault, distinct from a rejected user request.
    #[error("semantic request validator: {0}")]
    Validator(String),
    /// Original helper connection or stream delivery failure.
    #[error("semantic RPC transport: {0}")]
    Transport(String),
    /// RPC detail interpretation retains the original transport error and local cause.
    #[error(transparent)]
    Rpc(#[from] Box<RpcFailure>),
    /// An original native refusal retains structured machine meaning across clients.
    #[error(transparent)]
    Refusal(Box<Failure>),
    /// Client request cancellation never implies session cancellation.
    #[error("semantic request canceled; inspect session status for original session availability")]
    Cancelled,
    /// Cancellation was observed with original work and native settlement retained.
    #[error("semantic request cancellation observed; original work settlement: {completion:?}")]
    CancelledObserved {
        /// Actual original work ownership state.
        completion: Settlement,
        /// Original work result, independent of the cancellation request.
        original: OriginalOutcome,
    },
    /// Cancellation was requested but the original completion reply could not be observed.
    #[error("native request cancellation requested; CPU settlement remains unobserved: {cause}")]
    CancellationUnobserved {
        /// Original transport/protocol failure, independent of the cancellation request.
        #[source]
        cause: Box<Error>,
    },
    /// An old handle/input cannot be joined with current analysis.
    #[error("semantic source/context changed; reacquire the current generation")]
    Stale,
}
pse_diagnostics::impl_diagnostic! { Error,
    code(_this) { Some(pse_diagnostics::DiagnosticCode::HostOwnership) },
    forward(_this) { None }, help(_this) { None }, related(_this) { None }, source(_this) { None }
}
/// Semantic owner result.
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub(crate) fn phase(&self) -> FailurePhase {
        match self {
            Error::Refusal(failure) => failure.phase,
            Error::Rpc(rpc) => rpc.phase(),
            Error::InvalidRequest(_) => FailurePhase::Constraint,
            Error::Validator(_) => FailurePhase::Validator,
            Error::Transport(_) | Error::CancellationUnobserved { .. } => FailurePhase::Transport,
            Error::Stale | Error::Unavailable(_) => FailurePhase::Context,
            Error::Json(_) => FailurePhase::Schema,
            Error::WireDecode(_) => FailurePhase::Schema,
            Error::Schema(_) => FailurePhase::Schema,
            Error::Cancelled | Error::CancelledObserved { .. } => FailurePhase::Settlement,
            _ => FailurePhase::Provider,
        }
    }
    /// Original authored refusal, distinct from an undecodable or absent RPC detail.
    pub(crate) fn native_failure(&self) -> Option<&Failure> {
        match self {
            Self::Refusal(failure) => Some(failure),
            Self::Rpc(rpc) => rpc.native_failure(),
            _ => None,
        }
    }
    /// Unsupported producer operation, distinct from missing explicit preparation.
    pub(super) fn unsupported(message: impl Into<String>) -> Self {
        Self::Refusal(Box::new(Failure {
            diagnostic_code: None,
            context: None,
            state: FailureState::Unsupported,
            phase: FailurePhase::Provider,
            message: message.into(),
            backend_code: None,
            backend_data: None,
            observed_cancellation: None,
            validation: None,
        }))
    }
}

#[cfg(test)]
mod failure_tests {
    use super::*;
    #[test]
    fn contextual_refusals_and_nested_cancellation_apply_native_path_disclosure() {
        use std::os::unix::ffi::OsStringExt as _;
        let marker = "test-disclosure-marker";
        let policy = crate::envelope::RedactionPolicy::new([("TEST_SECRET".into(), marker.into())]);
        for path in [
            PathBuf::from(format!("/selected/{marker}.rs")),
            PathBuf::from(std::ffi::OsString::from_vec(
                format!("/selected/{marker}")
                    .into_bytes()
                    .into_iter()
                    .chain([0xff])
                    .collect(),
            )),
        ] {
            let original = Failure {
                diagnostic_code: None,
                context: Some(FailureContext {
                    reason: FailureReason::SourceChanged,
                    operation: "source_guard".into(),
                    operand: Some(path.into()),
                }),
                state: FailureState::Stale,
                phase: FailurePhase::Context,
                message: format!("source changed: {marker}"),
                backend_code: None,
                backend_data: None,
                observed_cancellation: None,
                validation: None,
            };
            let refusal = Error::Refusal(Box::new(original.clone()));
            let disclosed = Failure::from_error_with_policy(&refusal, &policy);
            let encoded = serde_json::to_string(&disclosed).unwrap();
            assert!(!encoded.contains(marker), "{encoded}");
            assert!(encoded.contains("<redacted:TEST_SECRET>"));
            let wire = wire::failure_to_pb(disclosed).unwrap();
            let encoded = serde_json::to_string(&wire::failure_from_pb(wire).unwrap()).unwrap();
            assert!(!encoded.contains(marker));
            let cancelled = Error::CancelledObserved {
                completion: Settlement::CpuSettled,
                original: OriginalOutcome::Failed(Box::new(original)),
            };
            let encoded =
                serde_json::to_string(&Failure::from_error_with_policy(&cancelled, &policy))
                    .unwrap();
            assert!(!encoded.contains(marker), "{encoded}");
        }
    }
    #[test]
    fn cancellation_diagnostic_keeps_settlement_and_original_failure_typed() {
        let original = Failure {
            diagnostic_code: None,
            context: None,
            state: FailureState::Stale,
            phase: FailurePhase::Context,
            message: "original source changed".into(),
            backend_code: Some(-32801),
            backend_data: Some(serde_json::json!({"original":"provider"})),
            observed_cancellation: None,
            validation: None,
        };
        let failure = Failure::from_error(&Error::CancelledObserved {
            completion: Settlement::CpuSettled,
            original: OriginalOutcome::Failed(Box::new(original)),
        });
        assert!(matches!(failure.state, FailureState::Cancelled));
        assert!(matches!(failure.phase, FailurePhase::Settlement));
        assert!(failure.backend_data.is_none());
        let observed = failure.observed_cancellation.unwrap();
        assert!(matches!(observed.completion, Settlement::CpuSettled));
        let OriginalOutcome::Failed(original) = observed.original else {
            panic!("original native failure required")
        };
        assert_eq!(original.backend_code, Some(-32801));
        assert_eq!(original.backend_data.unwrap()["original"], "provider");
    }
    #[test]
    fn request_constraint_and_validator_execution_failures_remain_distinct() {
        assert!(matches!(
            Failure::from_error(&Error::InvalidRequest("line must be positive".into())).phase,
            FailurePhase::Constraint
        ));
        assert!(matches!(
            Failure::from_error(&Error::Validator("validator failed".into())).phase,
            FailurePhase::Validator
        ));
        assert!(matches!(
            Failure::from_error(&Error::Transport("stream reset".into())).phase,
            FailurePhase::Transport
        ));
    }
}
