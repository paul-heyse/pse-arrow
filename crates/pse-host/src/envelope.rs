// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Command presentation, independent subject evidence and serialization contracts.
mod diagnostic;
mod presentation;
pub use diagnostic::{ErrorRecord, RedactionPolicy};
pub use presentation::{OutputFormat, Presentation, schema, write};
use schemars::JsonSchema;
use serde::Serialize;

/// CLI identity; never a scientific run or manager incarnation identity.
#[derive(Clone, Debug, Serialize, serde::Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(try_from = "String", into = "String")]
#[schemars(extend("format" = "uuid"))]
pub struct CliInvocationId(String);
impl CliInvocationId {
    /// Allocate an identity for precisely this command invocation.
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7().to_string())
    }
    /// Borrow the validated UUID spelling for storage and correlation.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
#[path = "envelope/tests.rs"]
mod tests;
impl Default for CliInvocationId {
    fn default() -> Self {
        Self::new()
    }
}
impl std::fmt::Display for CliInvocationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl From<CliInvocationId> for String {
    fn from(value: CliInvocationId) -> Self {
        value.0
    }
}
impl TryFrom<String> for CliInvocationId {
    type Error = uuid::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self(uuid::Uuid::parse_str(&value)?.to_string()))
    }
}

/// The command response, independently of the subject it observed.
#[derive(Clone, Copy, Debug, Serialize, serde::Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CommandOutcome {
    /// The requested response was produced.
    Success,
    /// A required command operation failed.
    Failure,
    /// The invocation was interrupted.
    Interrupted,
    /// Invalid arguments prevented dispatch.
    Usage,
    /// A boundary or tool premise was unavailable.
    BoundaryFailure,
}
/// Owner-reported subject state. Unknown is not a terminal assertion.
#[derive(Clone, Copy, Debug, Serialize, serde::Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SubjectState {
    /// This command makes no workload-state claim.
    NotApplicable,
    /// The subject's state is unknown.
    Unknown,
    /// Launch acknowledged; completion is unobserved.
    Launched,
    /// The owner reports ongoing work.
    Running,
    /// The owner reports terminal success.
    Succeeded,
    /// The owner reports terminal failure.
    Failed,
    /// The workload deadline expired, independently of observation deadlines.
    TimedOut,
    /// The owner reports cancellation.
    Cancelled,
    /// A requested provider is unavailable.
    Unavailable,
    /// Required explicit preparation is absent.
    Unprepared,
    /// Available evidence belongs to an older generation.
    Stale,
    /// A complete query found no matching subject.
    NoMatch,
}
/// Physical settlement, independently of command and workload outcomes.
#[derive(Clone, Copy, Debug, Serialize, serde::Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Settlement {
    /// This operation acquired no resource requiring reconciliation.
    NotRequired,
    /// The owner established actual completion and reconciliation.
    Complete,
    /// A required reconciliation remains incomplete.
    Incomplete,
    /// Required drain or release could not be established.
    Unknown,
}
/// Evidence coverage, independent of whether a response can be rendered.
#[derive(Clone, Copy, Debug, Serialize, serde::Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Completeness {
    /// All facts required by the named observation were obtained.
    Complete,
    /// Some requested facts are missing.
    Partial,
    /// Coverage has not been established.
    Unknown,
}
/// The actual observation's coverage and deadline, not a workload timeout.
#[derive(Clone, Debug, Serialize, serde::Deserialize, JsonSchema)]
pub struct Observation {
    /// Observing invocation or source identity, when supplied by its owner.
    pub reference: Option<String>,
    /// Named observed domain.
    pub scope: String,
    /// Established coverage.
    pub completeness: Completeness,
    /// Whether complete coverage is required to claim command success.
    pub required: bool,
    /// The observation deadline elapsed while the subject could remain active.
    pub timed_out: bool,
    /// Original unavailable facts or coverage limitations.
    pub limitations: Vec<String>,
}
/// Scientific references are supplied by the product, never synthesized from a CLI ID.
#[derive(Clone, Debug, Serialize, serde::Deserialize, JsonSchema)]
pub struct ScientificReference {
    /// Product-owned run identity.
    pub run_id: String,
    /// Product-owned attempt identity, when known.
    pub attempt_id: Option<String>,
}
/// Source coordinates retain their unit; byte and protocol positions cannot alias.
#[derive(Clone, Debug, Serialize, serde::Deserialize, JsonSchema)]
#[serde(tag = "encoding", rename_all = "snake_case")]
pub enum SourceCoordinates {
    /// UTF-8 byte offsets in a named source revision.
    Utf8Bytes {
        /// Source identity.
        source: String,
        /// Immutable revision, when known.
        revision: Option<String>,
        /// Inclusive byte offset.
        start: usize,
        /// Exclusive byte offset.
        end: usize,
    },
    /// Negotiated LSP line/character coordinates.
    Lsp {
        /// Source URI.
        uri: String,
        /// Negotiated position encoding.
        position_encoding: String,
        /// Zero-based line.
        line: u32,
        /// Zero-based character in the negotiated unit.
        character: u32,
    },
    /// CLI argument index, independent of source document locations.
    CliArgument {
        /// Zero-based argument position.
        index: usize,
    },
}
/// Provider attribution for progressive code/product queries.
#[derive(Clone, Debug, Serialize, serde::Deserialize, JsonSchema)]
pub struct ProviderContext {
    /// Actual provider name.
    pub provider: String,
    /// Selected crate/workspace or product context.
    pub context: Option<String>,
    /// Actual analysis generation; absent means not established.
    pub analysis_generation: Option<String>,
    /// Provider's explicit current availability.
    pub availability: SubjectState,
    /// Attributed source positions, in their original units.
    pub coordinates: Vec<SourceCoordinates>,
}
/// Retained full evidence or a larger paginated result.
#[derive(Clone, Debug, Serialize, serde::Deserialize, JsonSchema)]
pub struct ArtifactReference {
    /// Stable artifact purpose.
    pub kind: String,
    /// Owner-supplied path or URI.
    pub location: String,
    /// Content identity, when verified.
    pub digest: Option<String>,
}
/// A typed follow-up without manufacturing an executable shell string.
#[derive(Clone, Debug, Serialize, serde::Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NextAction {
    /// Another CLI query, with literal argument boundaries.
    Command {
        /// Literal argument vector, excluding the executable name.
        argv: Vec<String>,
        /// Why the action may be useful.
        reason: String,
    },
    /// Read already retained evidence.
    ReadArtifact {
        /// Artifact location.
        location: String,
        /// Why the evidence is useful.
        reason: String,
    },
}
/// Explicit omission metadata; presentation never changes mandatory evidence.
#[derive(Clone, Debug, Default, Serialize, serde::Deserialize, JsonSchema)]
pub struct DetailView {
    /// Payload fields selected by the caller; null means the full field set.
    pub fields: Option<Vec<String>>,
    /// Actual excluded payload fields.
    pub omitted_fields: Vec<String>,
    /// Record/detail allowance, never a serialized-byte ceiling.
    pub budget: Option<usize>,
    /// Number of payload records omitted by that allowance.
    pub omitted_records: usize,
    /// True precisely when this response omitted available detail.
    pub truncated: bool,
}
/// Versioned response. Declaration order makes the summary the first JSON member.
#[derive(Debug, Serialize, serde::Deserialize, JsonSchema)]
pub struct Envelope<T> {
    /// Short actionable response.
    pub summary: String,
    /// Contract version, independent of package versions.
    pub version: u32,
    /// Actual presented schema identity.
    pub schema: String,
    /// This invocation, independent of scientific/observation identities.
    pub invocation_id: CliInvocationId,
    /// Dispatched verb path.
    pub command: String,
    /// Command response outcome.
    pub outcome: CommandOutcome,
    /// Owner-reported subject state.
    pub subject_state: SubjectState,
    /// Original child status, including 128+signal; absent when no result is available.
    pub child_exit: Option<i32>,
    /// Required resource reconciliation.
    pub settlement: Settlement,
    /// Observation completeness and its independent deadline.
    pub observation: Observation,
    /// Actual product references, if the product supplied them.
    pub scientific: Vec<ScientificReference>,
    /// Actual query provider contexts, retained by every projected view.
    pub providers: Vec<ProviderContext>,
    /// All mandatory normalized failures, in occurrence order.
    pub failures: Vec<ErrorRecord>,
    /// Retained evidence beyond this presentation.
    pub artifacts: Vec<ArtifactReference>,
    /// Typed optional follow-ups.
    pub next: Vec<NextAction>,
    /// Explicit presentation omissions.
    pub view: DetailView,
    /// Verb-owned detail; only this field can be projected or budgeted.
    pub payload: T,
}
impl<T> Envelope<T> {
    /// Preserve every mandatory fact while replacing only the presentation payload.
    pub fn with_payload<U>(self, payload: U) -> Envelope<U> {
        Envelope {
            summary: self.summary,
            version: self.version,
            schema: self.schema,
            invocation_id: self.invocation_id,
            command: self.command,
            outcome: self.outcome,
            subject_state: self.subject_state,
            child_exit: self.child_exit,
            settlement: self.settlement,
            observation: self.observation,
            scientific: self.scientific,
            providers: self.providers,
            failures: self.failures,
            artifacts: self.artifacts,
            next: self.next,
            view: self.view,
            payload,
        }
    }
    /// A completed command response. This constructor asserts no scientific outcome.
    pub fn response(command: impl Into<String>, summary: impl Into<String>, payload: T) -> Self {
        let command = command.into();
        Self {
            summary: summary.into(),
            version: 1,
            schema: format!("pse.cli.v1/{command}/full"),
            invocation_id: CliInvocationId::new(),
            command,
            outcome: CommandOutcome::Success,
            subject_state: SubjectState::NotApplicable,
            child_exit: None,
            settlement: Settlement::NotRequired,
            observation: Observation {
                reference: None,
                scope: "command_response".into(),
                completeness: Completeness::Complete,
                required: true,
                timed_out: false,
                limitations: Vec::new(),
            },
            scientific: Vec::new(),
            providers: Vec::new(),
            failures: Vec::new(),
            artifacts: Vec::new(),
            next: Vec::new(),
            view: DetailView::default(),
            payload,
        }
    }
    /// Managed exit class. Exec adapters bypass this classification entirely.
    pub fn exit_code(&self) -> i32 {
        match self.outcome {
            CommandOutcome::Usage => 2,
            CommandOutcome::BoundaryFailure => 125,
            CommandOutcome::Interrupted => 130,
            CommandOutcome::Failure => 1,
            CommandOutcome::Success
                if matches!(
                    self.subject_state,
                    SubjectState::Failed | SubjectState::TimedOut | SubjectState::Cancelled
                ) || matches!(
                    self.settlement,
                    Settlement::Incomplete | Settlement::Unknown
                ) || (self.observation.required
                    && self.observation.completeness != Completeness::Complete) =>
            {
                1
            }
            CommandOutcome::Success => 0,
        }
    }
}
