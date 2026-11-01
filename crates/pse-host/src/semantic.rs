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
mod protocol_facts;
pub use facts::*;
#[cfg(feature = "native-hir")]
pub mod native;
pub(crate) mod rpc;
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

/// Current prepared semantic helper/selector protocol; legacy selectors are administration-only.
pub const SESSION_PROTOCOL_VERSION: u32 = 6;

/// Explicit context owner; immutable native snapshots are not a second live LSP world.
#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize, JsonSchema, clap::ValueEnum,
)]
#[serde(rename_all = "snake_case")]
pub enum Backend {
    /// The explicitly prepared reusable rust-analyzer LSP session.
    #[default]
    Lsp,
    /// An explicitly prepared immutable compatible-family HIR/IDE snapshot.
    NativeSnapshot,
}

/// Selected analysis context; LSP cannot choose an exact Cargo compiler-unit instance.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    /// Selected semantic context and provider authority.
    #[serde(default)]
    pub backend: Backend,
    /// Canonical Cargo project root, independent of the owning checkout.
    pub project: PathBuf,
    /// Explicit feature names passed to the analyzer's Cargo model.
    pub features: Vec<String>,
    /// Disable Cargo defaults in this analyzer model.
    pub no_default_features: bool,
    /// Explicit compilation target, when selected.
    pub target: Option<String>,
    /// Explicitly admit native build-script and proc-macro preparation through correctness Cargo.
    pub build_macros: bool,
    /// Explicitly include native cfg(test) semantics; never compiler-unit equivalence.
    #[serde(default)]
    pub test_context: bool,
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
    /// Read selected package facts from the already loaded native Cargo/VFS world.
    PackageContext {
        /// Exact Cargo source-qualified package identity.
        package_id: String,
    },
}
/// Actual library crate context captured from the same prepared native database.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
pub struct NativeLibraryProfile {
    /// Original source-qualified Cargo package identifier.
    pub package_id: String,
    /// Original package manifest.
    pub manifest: PathBuf,
    /// Original package source root.
    pub source_root: PathBuf,
    /// Actual loaded library root.
    pub library_root: PathBuf,
    /// Effective feature atoms of this exact native crate.
    pub features: std::collections::BTreeSet<String>,
    /// Actual loaded target, absent for unqualified host contexts.
    pub target: Option<String>,
    /// Actual library/proc-macro/test role.
    pub role: String,
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
}
/// Supported backend operations, distinct from server-free syntax queries.
#[derive(Clone, Debug, Serialize, Deserialize, clap::ValueEnum, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Kind {
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
    /// Qualified loaded Cargo/VFS package context.
    PackageContext(PackageContextAnswer),
    /// Stop accepted by the session owner; teardown remains session-owned.
    Stopped,
}
impl Response {
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
}
/// Distinguish explicit preparation, unsupported capability, cancellation and revision invalidation.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FailureState {
    /// Explicit preparation is missing/pending or another selected premise is unavailable.
    Unavailable,
    /// Selected backend does not implement the requested operation.
    Unsupported,
    /// Abandoned request; session lifetime remains independent.
    Cancelled,
    /// Source/configuration changed; reacquire inputs.
    Stale,
    /// Original protocol, source or native owner failed.
    Failed,
}
/// Original failure boundary, separate from native availability/cancellation state.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FailurePhase {
    /// A schema representation could not be decoded or did not match its contract.
    Schema,
    /// The user's request violated a declared constraint or stream-control contract.
    Constraint,
    /// Original source/session/provider context is unavailable or stale.
    Context,
    /// Validator execution failed rather than rejecting the user's request.
    Validator,
    /// Original selected provider failure.
    Provider,
    /// Native result delivery or connection failed.
    Transport,
    /// Cancellation observation retains actual original work settlement.
    Settlement,
}
impl Failure {
    /// Preserve selected typed error categories at the serialization boundary.
    pub fn from_error(error: &Error) -> Self {
        if let Error::Refusal(failure) = error {
            return failure.clone();
        }
        let state = match error {
            Error::Stale => FailureState::Stale,
            Error::Cancelled | Error::CancelledObserved { .. } => FailureState::Cancelled,
            Error::Unavailable(_) => FailureState::Unavailable,
            _ => FailureState::Failed,
        };
        let phase = match error {
            Error::InvalidRequest(_) => FailurePhase::Constraint,
            Error::Validator(_) => FailurePhase::Validator,
            Error::Transport(_) | Error::CancellationUnobserved { .. } => FailurePhase::Transport,
            Error::Stale | Error::Unavailable(_) => FailurePhase::Context,
            Error::Json(_) => FailurePhase::Schema,
            Error::Schema(_) => FailurePhase::Schema,
            Error::Cancelled | Error::CancelledObserved { .. } => FailurePhase::Settlement,
            _ => FailurePhase::Provider,
        };
        Self {
            state,
            phase,
            message: crate::envelope::RedactionPolicy::from_environment()
                .redact(&error.to_string()),
            backend_code: None,
            backend_data: None,
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
    Failed(Failure),
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
    /// Validator fault, distinct from a rejected user request.
    #[error("semantic request validator: {0}")]
    Validator(String),
    /// Original helper connection or stream delivery failure.
    #[error("semantic RPC transport: {0}")]
    Transport(String),
    /// An original native refusal retains structured machine meaning across clients.
    #[error(transparent)]
    Refusal(Failure),
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
    /// Unsupported producer operation, distinct from missing explicit preparation.
    pub(super) fn unsupported(message: impl Into<String>) -> Self {
        Self::Refusal(Failure {
            state: FailureState::Unsupported,
            phase: FailurePhase::Provider,
            message: message.into(),
            backend_code: None,
            backend_data: None,
            observed_cancellation: None,
        })
    }
}

#[cfg(test)]
mod failure_tests {
    use super::*;
    #[test]
    fn cancellation_diagnostic_keeps_settlement_and_original_failure_typed() {
        let original = Failure {
            state: FailureState::Stale,
            phase: FailurePhase::Context,
            message: "original source changed".into(),
            backend_code: Some(-32801),
            backend_data: Some(serde_json::json!({"original":"provider"})),
            observed_cancellation: None,
        };
        let failure = Failure::from_error(&Error::CancelledObserved {
            completion: Settlement::CpuSettled,
            original: OriginalOutcome::Failed(original),
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
