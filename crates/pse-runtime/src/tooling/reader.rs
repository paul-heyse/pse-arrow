// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Product-owned observations and explicit diagnostic investigation, without scientific execution.
mod canonical;
#[cfg(feature = "product-operators")]
mod catalog;
mod follow;
mod investigation;
use super::inspection::{
    self, DiagnosticListRequest, DiagnosticProcessPage, InspectionDocument, InspectionRequest,
};
use crate::{
    invocation::{InvocationEvidenceDocument, InvocationOutcome},
    observation::{
        CheckpointIdentity, ObservationError, ProcessLocator, ProcessResources,
        read_checkpoint_source,
    },
};
pub use canonical::CanonicalReadSelection;
pub use canonical::canonical_redaction_policy;
pub use canonical::project::{ProjectRead, ProjectReadSelection, ProjectReadView, ProjectTarget};
pub use follow::{Continuity as FollowContinuity, FollowCursor, FollowSample, ProducerPosition};
pub use investigation::{Request as InvestigationRequest, ResultView as InvestigationResult};
use pse_diagnostics::observation::{
    ChildSnapshot, CoherentRecord, Coverage, OperationId, Outcome, ProducerId,
};
use pse_host::investigation::NativePath;
use pse_model::document::Version;
use schemars::{JsonSchema, Schema, generate::SchemaSettings};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    io::Read,
    num::NonZeroUsize,
    os::unix::fs::MetadataExt as _,
    path::{Path, PathBuf},
};

/// Maximum one protocol request, independent of scientific or host workload allocation.
pub const REQUEST_BYTES: usize = 1024 * 1024;
/// Maximum complete encoded response; failure emits a small refusal, never partial JSON.
pub const RESPONSE_BYTES: usize = 40 * 1024 * 1024;
/// Product-owned response contract identity.
pub const RESPONSE_SCHEMA: &str = "urn:pse:product-reader:v1:response";
/// Product-owned request contract identity.
pub const REQUEST_SCHEMA: &str = "urn:pse:product-reader:v1:request";
/// Exact transport family; its schema admits no other protocol spelling.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
pub enum Protocol {
    /// Passive product inspection transport.
    #[serde(rename = "pse.product-reader")]
    ProductReader,
}
/// Explicit effect class, independent of scientific outcome or result eligibility.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Access {
    /// Existing evidence only; capture and export refuse before effects.
    #[default]
    Passive,
    /// Caller explicitly admitted a diagnostic attachment or publication request.
    Investigation,
}
/// Product-owned schema generation direction, separate from domain validity.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Contract {
    /// Deserialization-direction accepted input.
    Request,
    /// Serialization-direction emitted output.
    Response,
}
/// A cursor binds both copied source bytes and the original reader selection.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PageCursor {
    /// Source image or directory inventory fingerprint issued by this reader.
    pub source: String,
    /// Exact original source/operation/section/limit selection.
    pub query: String,
    /// Next whole semantic record in that copied source.
    pub offset: usize,
    /// Exclusive checkpoint filename for process discovery, absent for image sections.
    pub after: Option<String>,
}
/// Current process selection and retained evidence are different read contracts.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "selection", rename_all = "snake_case", deny_unknown_fields)]
pub enum CheckpointSelection {
    /// Retrospective checkpoint read; no process is claimed to remain live.
    Retained {
        /// Explicit copied checkpoint path.
        checkpoint: PathBuf,
        /// Original supplied association, checked when present.
        expected: Option<CheckpointIdentity>,
    },
    /// Exact recorded process/start identity must still be current; PID reuse refuses.
    Current {
        /// Explicit published checkpoint path.
        checkpoint: PathBuf,
        /// Required original process/start association.
        expected: CheckpointIdentity,
    },
}
impl CheckpointSelection {
    fn path(&self) -> &Path {
        match self {
            Self::Retained { checkpoint, .. } | Self::Current { checkpoint, .. } => checkpoint,
        }
    }
    fn validate(&self, image: &crate::observation::ProcessCheckpoint) -> Result<(), ReaderError> {
        let expected = match self {
            Self::Retained { expected, .. } => expected.as_ref(),
            Self::Current { expected, .. } => Some(expected),
        };
        if let Some(expected) = expected {
            expected.validate(image).map_err(ReaderError::Identity)?;
        }
        if matches!(self, Self::Current { .. }) && !image.locator.is_current() {
            return Err(ReaderError::Invalid("selected process/start identity is no longer current; retained inspection must be selected explicitly".into()));
        }
        Ok(())
    }
}
/// Native whole-record sections, without recursive JSON projection or tuple truncation.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Section {
    /// Whole original related-operation links.
    Related,
    /// Whole producer-owned coherent numerical/phase records.
    Records,
}
/// Product observations and explicit diagnostic effects; canonical reads never start a database.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, strum::VariantNames)]
#[strum(serialize_all = "snake_case")]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    /// Discover this exact product reader's protocols and schema associations.
    Discover,
    /// Generate the actual directional Serde/Schemars contract from this product executable.
    Schema {
        /// Input or output direction to publish.
        contract: Contract,
    },
    /// Independently verify an existing project pack without a service connection.
    InspectProjectPack {
        /// Explicit absolute directory containing the project pack.
        input: NativePath,
        /// Optional bound on bytes in an individual manifest or document.
        maximum_document_bytes: Option<NonZeroUsize>,
    },
    /// Observe the exact original local publication without contacting its issuing service.
    ProjectExportStatus {
        /// Exact authored absolute native destination.
        destination: NativePath,
        /// Original publication key.
        key: String,
        /// Original authenticated issuing store identity.
        issuing_store: String,
    },
    /// Read bounded project metadata from an exact existing canonical deployment.
    Project {
        /// Existing private deployment and its exact selected database.
        canonical: ProjectReadSelection,
        /// Original typed project metadata, branch or draft query.
        query: ProjectRead,
    },
    /// Observe existing managed definitions under the exact selected issuing store.
    #[cfg(feature = "product-operators")]
    InspectCanonicalCatalog {
        /// Existing private deployment state directory.
        state: NativePath,
        /// Explicit existing database, or the deployment default.
        database: Option<String>,
        /// Required persistent canonical issuing-store UUID.
        expected_issuer: String,
        /// Bounded observer allowance in milliseconds.
        timeout_ms: Option<std::num::NonZeroU64>,
    },
    /// Existing checkpoint catalogue; only selected process images are decoded.
    Processes {
        /// Explicit existing diagnostic checkpoint directory.
        directory: PathBuf,
        /// Requested whole process rows, from one through 32.
        limit: usize,
        /// Original source- and query-bound continuation.
        cursor: Option<PageCursor>,
    },
    /// Native owner inspection and statements, including explicit original scientific evidence.
    Inspect {
        /// Exact copied source and current/retained semantics.
        source: CheckpointSelection,
        /// Original native operation/window/related selection.
        request: InspectionRequest,
        /// Optional explicitly supplied original invocation evidence document.
        invocation: Option<PathBuf>,
    },
    /// One passive follow sample; continuation is an opaque product-owned next request.
    Follow {
        /// Exact current or retained checkpoint selection.
        source: CheckpointSelection,
        /// Existing native inspection selection.
        request: InspectionRequest,
        /// Explicit original invocation evidence, when available.
        invocation: Option<PathBuf>,
        /// Original product-issued continuity coordinates.
        resume: Option<FollowCursor>,
    },
    /// A bounded related-operation or coherent-record page from one exact copied checkpoint.
    Page {
        /// Exact copied checkpoint selection.
        source: CheckpointSelection,
        /// Original selected diagnostic operation.
        operation_id: OperationId,
        /// Whole semantic record family.
        section: Section,
        /// Optional exact performing producer, for coherent records only.
        producer: Option<ProducerId>,
        /// Requested whole records, from one through 32.
        limit: usize,
        /// Original source/query continuation.
        cursor: Option<PageCursor>,
    },
    /// Exact selected study snapshot and retained per-attempt progress, without live attachment.
    Study {
        /// Existing connection and explicit observer allocation/allowance.
        canonical: CanonicalReadSelection,
        /// Existing native bounded study selection.
        request: crate::workflow::StudyObservationRequest,
        /// Exact existing native reconnect coordinates, never observer-issued scientific IDs.
        cursor: Option<crate::workflow::StudyObservationCursor>,
    },
    /// Exact original attempt lineage and protected terminal numerical history.
    Attempt {
        /// Existing connection and explicit observer allocation/allowance.
        canonical: CanonicalReadSelection,
        /// Original attempt and native source/batch reconnect position.
        request: crate::workflow::AttemptObservationRequest,
    },
    /// Explicit diagnostic investigation, separate from passive reading.
    Investigate {
        /// Product-owned bounded capture, supplied-evidence or export operation.
        request: investigation::Request,
    },
}
/// Correlation is an observer request nonce, never a RunId, AttemptId or cancellation capability.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReaderRequest {
    /// Fixed protocol family.
    pub protocol: Protocol,
    /// Transport version, checked by the native Version decoder.
    pub version: Version<1>,
    /// Opaque caller correlation; valid requests require one through 128 UTF-8 bytes.
    pub correlation: String,
    /// Explicit diagnostic-effect admission; absence retains passive behavior.
    #[serde(default)]
    pub access: Access,
    /// Product-owned read semantics.
    pub request: Request,
}
/// Command result only; successful observation cannot certify scientific success.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    /// A valid observation, possibly describing unavailable or failed original work.
    Success,
    /// Transport or checked reader selection was refused.
    Refused,
}
/// Original source availability/refusal, retaining native message and ordinary source chain.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Failure {
    /// Stable product refusal/source category.
    pub code: String,
    /// Original native explanation.
    pub message: String,
    /// Original ordinary error causes, in order.
    pub causes: Vec<String>,
    /// Source-owned typed diagnostic, including independent related failures.
    /// Absence retains historical coverage or a transport extent refusal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub native: Option<pse_host::envelope::ErrorRecord>,
}
impl Failure {
    fn capture(code: &str, error: &dyn std::error::Error) -> Self {
        let policy = pse_host::envelope::RedactionPolicy::from_environment();
        let mut causes = Vec::new();
        let mut cause = error.source();
        for _ in 0..16 {
            let Some(error) = cause else {
                break;
            };
            causes.push(policy.redact(&error.to_string()));
            cause = error.source();
        }
        Self {
            code: code.into(),
            message: policy.redact(&error.to_string()),
            causes,
            native: None,
        }
    }
}
/// Actual examined scope, independently of the number of returned records.
#[derive(Debug, Serialize, JsonSchema)]
pub struct SourceFacts {
    /// Original selected image/directory.
    pub path: PathBuf,
    /// Exact checkpoint bytes, or named mutable directory inventory fingerprint.
    pub identity: String,
    /// Actual decoded checkpoint extent, unavailable for catalogue metadata traversal.
    pub examined_bytes: Option<usize>,
    /// Examined directory entries or this one selected checkpoint.
    pub examined_sources: usize,
    /// Neither file reads nor directory traversal establish an atomic multi-source snapshot.
    pub atomic_snapshot: bool,
    /// Publication changed while listing; continuation is withheld rather than invented stable.
    pub changed_during_read: bool,
}
/// Original native process facts, never an operation-private resource charge.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ProcessFacts {
    #[serde(skip)]
    managed_invocation: Option<pse_diagnostics::process_facts::ManagedInvocationCandidate>,
    /// Native process-instance locator, not a scientific ID.
    pub process: String,
    /// Actual original PID/start/executable association.
    pub locator: ProcessLocator,
    /// Independent readback; absent start identity cannot become live=true.
    pub current_start_identity: Option<bool>,
    /// Native publication sequence, not a scientific attempt generation.
    pub publication_sequence: u64,
    /// Native publisher wall clock, distinct from producer monotonic clocks.
    pub published_unix_micros: u128,
    /// Original process-lifetime bytes and CPU ticks, with native field units preserved.
    pub resources: ProcessResources,
}
/// Only a validated original association; absent evidence never synthesizes scientific IDs.
#[derive(Debug, Serialize, JsonSchema)]
pub struct InvocationAssociation {
    /// Original scientific execution.
    pub run_id: pse_model::generated::identities::RunId,
    /// Original durable attempt, if supplied by its owner.
    pub attempt_id: Option<pse_model::generated::identities::AttemptId>,
    /// Exact matching diagnostic operation.
    pub operation: OperationId,
    /// Retained invocation owner outcome, never rewritten by successful observation.
    pub outcome: InvocationOutcome,
    /// Original acknowledged canonical locator, not permission to open it.
    pub canonical_run: Option<String>,
    /// Original acknowledged canonical attempt locator.
    pub canonical_attempt: Option<String>,
}
/// Whole owned records retain units, categories and enclosing operation context.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "section", content = "records", rename_all = "snake_case")]
pub enum Records {
    /// Original related-operation links, preserving relationship and clocks.
    Related(Vec<ChildSnapshot>),
    /// Original complete coherent records, preserving units and availability categories.
    Records(Vec<CoherentRecord>),
}
/// A semantically bounded source page; unknown total is distinct from observed prefix size.
#[derive(Debug, Serialize, JsonSchema)]
pub struct RecordPage {
    #[serde(skip)]
    managed_invocation: Option<pse_diagnostics::process_facts::ManagedInvocationCandidate>,
    /// Exact original diagnostic operation.
    pub operation: OperationId,
    /// Original producer-supplied operation description.
    pub operation_kind: String,
    /// Original operation outcome; successful observation never replaces it.
    pub outcome: Option<Outcome>,
    /// Original owner coverage, independent of bounded page length.
    pub coverage: Coverage,
    /// Original detail omission, independent of this consumer's continuation.
    pub detail_partial: bool,
    /// Original responsibility and expiry; neither is scientific result permission.
    pub summary: Option<pse_diagnostics::observation::SummaryPublication>,
    /// Native owner-selected freshness threshold.
    #[schemars(with = "pse_diagnostics::ClosedDuration")]
    pub stale_after: std::time::Duration,
    /// Original producer observation timestamp in its own monotonic clock.
    #[schemars(with = "Option<pse_diagnostics::ClosedDuration>")]
    pub observed_at: Option<std::time::Duration>,
    /// Actual native freshness assessment time, not a new numerical observation.
    #[schemars(with = "pse_diagnostics::ClosedDuration")]
    pub read_at: std::time::Duration,
    /// Original mapping for comparing this same-host clock, unavailable when not established.
    pub clock: Option<crate::observation::ClockMapping>,
    /// Original process publication sequence, separate from native numerical source sequence.
    pub publication_sequence: u64,
    /// Original process publication wall timestamp; it is not a source event ordering key.
    pub published_unix_micros: u128,
    /// Whole source-owned records, never arbitrary nested JSON fragments.
    pub records: Records,
    /// Number of records present in the examined copied source.
    pub known_records: usize,
    /// Full total only when original coverage establishes it.
    pub total: Option<usize>,
    /// Original source coverage is partial/curtailed, independently of visible page length.
    pub partial: bool,
    /// Source- and query-bound next page, if more known records remain.
    pub continuation: Option<PageCursor>,
    /// Actual source scope and original byte identity.
    pub source: SourceFacts,
}
/// Typed product results. Host forwarding treats this representation as opaque data.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Payload {
    /// Discovery belongs to this captured product image, not a host parser reconstruction.
    Discovery {
        /// Product package release, not executable content identity.
        package_version: &'static str,
        /// Actual accepted-input schema association.
        request_schema: &'static str,
        /// Actual emitted-output schema association.
        response_schema: &'static str,
        /// Operations implemented by this product decoder.
        operations: Vec<&'static str>,
        /// Maximum native request extent.
        request_bytes: usize,
        /// Maximum native complete response extent.
        response_bytes: usize,
        /// Fixed, separately admitted project-effect protocol in this captured executable.
        #[serde(skip_serializing_if = "Option::is_none")]
        project_perform: Option<u32>,
        /// Fixed exact project import issue, apply and recovery actions.
        #[serde(skip_serializing_if = "Option::is_none")]
        project_import: Option<u32>,
        /// Explicit protected source reopen for an existing completed import.
        #[serde(skip_serializing_if = "Option::is_none")]
        project_import_open: Option<u32>,
        /// Exact staged project and study publication protocol.
        #[serde(skip_serializing_if = "Option::is_none")]
        project_export: Option<u32>,
    },
    /// Native directional schema document; it validates representation, not scientific meaning.
    Schema {
        /// Product-owned schema identity.
        identity: &'static str,
        /// Actual directional schema document.
        #[schemars(with = "serde_json::Value")]
        document: Schema,
    },
    /// Offline verification over the product owner's exact pack membership.
    ProjectPack {
        /// Independently validated logical manifest.
        manifest: Box<crate::workflow::ProjectManifest>,
        /// Encoding checksum of the exact manifest bytes the reader verified.
        manifest_digest: String,
    },
    /// Original local project or study delivery settlement.
    ProjectExportStatus {
        /// Native attribution and observed publication state.
        status: Box<crate::host_operations::project::exchange::PublicationStatus>,
    },
    /// Bounded original canonical project read over the authenticated target.
    Project {
        /// Namespace, database and persistent issuing-store identity.
        target: ProjectTarget,
        /// Original typed metadata page or header.
        result: Box<ProjectReadView>,
    },
    /// Passive managed catalog and generated-definition comparison.
    #[cfg(feature = "product-operators")]
    CanonicalCatalog {
        /// Exact selected issuer, observed marker and redacted managed identities.
        observation: Box<pse_operations::canonical::maintenance::CatalogObservation>,
    },
    /// Existing native catalogue and explicit cursor/source boundaries.
    Processes {
        /// Native bounded catalogue, including independent unreadable rows.
        page: DiagnosticProcessPage,
        /// Actual examined source scope.
        source: SourceFacts,
        /// Bound next page; withheld when the catalogue changed during this read.
        continuation: Option<PageCursor>,
    },
    /// Existing owner interpretation, original process facts and optional validated association.
    Inspection {
        /// Original native interpretation and owner statements.
        document: Box<InspectionDocument>,
        /// Independent process-wide facts, never private operation charges.
        process: ProcessFacts,
        /// Exact examined checkpoint bytes.
        source: SourceFacts,
        /// Original supplied and validated scientific association, when requested.
        association: Option<InvocationAssociation>,
    },
    /// One original inspection and its explicit continuity interpretation.
    Follow {
        /// Product-owned source coordinates, reset/gap state and original inspection.
        sample: Box<FollowSample>,
    },
    /// Explicit diagnostic result, preserving its original typed owner's interpretation.
    Investigation {
        /// Capture, frozen inspection, analysis, specification or selected pack.
        result: investigation::ResultView,
    },
    /// Whole related/coherent-record page.
    Page {
        /// Whole semantic records and explicit source coverage.
        page: RecordPage,
    },
    /// Original protected study snapshot/history and its existing native cursor.
    Study {
        /// Typed native page retained through serialization.
        page: Box<crate::workflow::StudyObservationPage>,
    },
    /// Original exact attempt lineage and protected native progress page.
    Attempt {
        /// Typed native page retained through serialization.
        page: Box<crate::workflow::AttemptObservationPage>,
    },
    /// Valid read response describing unavailable evidence; not an empty successful population.
    Unavailable {
        /// Original explicit source selection.
        source: PathBuf,
        /// Original typed-owner explanation, not a scientific failure reinterpretation.
        failure: Failure,
    },
    /// Product decoder/domain refusal, distinct from an unavailable source response.
    Refusal {
        /// Native decoder/selection refusal.
        failure: Failure,
    },
}
/// Owner-constructed transport result; data projection never changes original scientific facts.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ReaderResponse {
    protocol: Protocol,
    version: Version<1>,
    correlation: String,
    schema: &'static str,
    category: Category,
    payload: Payload,
    /// Opaque transport continuation; only the product constructs its domain selection.
    #[schemars(with = "Option<Request>")]
    next_request: Option<Box<serde_json::value::RawValue>>,
    /// Candidate association only; native host custody and current membership require readback.
    managed_invocation: Option<pse_diagnostics::process_facts::ManagedInvocationCandidate>,
    /// The original native failure stays owned through encoding/delivery, never decoded from facts.
    #[serde(skip)]
    #[schemars(skip)]
    original_failure: Option<Box<ReaderError>>,
}
/// Concrete native protocol failure; read availability stays in a valid typed response.
#[derive(Debug, thiserror::Error)]
pub enum ReaderError {
    /// Actual transport or source I/O error.
    #[error("product reader I/O: {0}")]
    Io(#[from] std::io::Error),
    /// Original strict input/schema/document serialization error.
    #[error("product reader JSON: {0}")]
    Json(#[from] serde_json::Error),
    /// Original source identity mismatch.
    #[error(transparent)]
    Identity(ObservationError),
    /// Original canonical connection/protection/history owner failure.
    #[error(transparent)]
    Canonical(Box<pse_operations::canonical::CanonicalError>),
    /// Original retained-data validation/allocation/selection failure.
    #[error(transparent)]
    Workflow(Box<crate::workflow::WorkflowError>),
    /// Original explicit capture owner failure.
    #[error(transparent)]
    Capture(#[from] crate::observation_detail::CaptureError),
    /// Original supplied developer evidence, native projection or analytical query refusal.
    #[error(transparent)]
    Developer(#[from] super::review_query::developer::Error),
    /// Original caller-owned observation cancellation or allocation failure.
    #[error(transparent)]
    Columnar(#[from] pse_columnar::CanonError),
    /// Original selected evidence export owner failure.
    #[error(transparent)]
    Pack(#[from] super::review_pack::PackError),
    /// Independent read/encoding failure and original owned-client drain failure.
    #[error("product reader operation failed: {operation}; client drain also failed: {cleanup}")]
    Settlement {
        /// Original read or serialization failure.
        #[source]
        operation: Box<ReaderError>,
        /// Original independently attempted client drain failure.
        cleanup: Box<ReaderError>,
    },
    /// A native refusal and an independently failed delivery of its response.
    #[error(
        "product response retained its original refusal: {response}; delivery also failed: {transport}"
    )]
    Delivery {
        /// The original native operation/settlement failure, not reconstructed observations.
        #[source]
        response: Box<ReaderError>,
        /// Actual response encoding, cancellation or output error.
        transport: Box<ReaderError>,
    },
    /// A malformed selection, changed continuation, or unsupported bound.
    #[error("product reader selection: {0}")]
    Invalid(String),
}
pse_diagnostics::impl_diagnostic! {
    ReaderError,
    code(this) { match this { Self::Identity(_) | Self::Canonical(_) | Self::Workflow(_) | Self::Capture(_) | Self::Developer(_) | Self::Columnar(_) | Self::Pack(_) => None, _ => Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure) } },
    forward(this) { match this { Self::Identity(error) => Some(error),Self::Canonical(error)=>Some(error.as_ref()),Self::Workflow(error)=>Some(error.as_ref()),Self::Capture(error)=>Some(error),Self::Developer(error)=>Some(error),Self::Columnar(error)=>Some(error),Self::Pack(error)=>Some(error), _ => None } },
    help(_this) { None }, related(this) { match this {Self::Settlement{operation,cleanup} | Self::Delivery {response:operation,transport:cleanup} =>{let errors:[&dyn pse_diagnostics::TypedDiagnostic;2]=[operation.as_ref(),cleanup.as_ref()];Some(Box::new(errors.into_iter()))},_=>None} }, source(_this) { None }
}
fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
fn query<T: Serialize>(value: &T) -> Result<String, ReaderError> {
    Ok(hash(&serde_json::to_vec(value)?))
}
fn invalid(message: &str) -> ReaderError {
    ReaderError::Invalid(message.into())
}
fn limit(value: usize) -> Result<(), ReaderError> {
    if (1..=32).contains(&value) {
        Ok(())
    } else {
        Err(invalid("record page limit must be between one and 32"))
    }
}
fn absolute(path: &Path) -> Result<(), ReaderError> {
    if path.is_absolute() {
        Ok(())
    } else {
        Err(invalid("passive source selections must use absolute paths"))
    }
}
fn validate(request: &ReaderRequest) -> Result<(), ReaderError> {
    if request.correlation.is_empty() || request.correlation.len() > 128 {
        return Err(invalid(
            "correlation must contain one through 128 UTF-8 bytes",
        ));
    }
    match &request.request {
        Request::Discover | Request::Schema { .. } => {}
        Request::InspectProjectPack { input, .. } => absolute(&input.to_path_buf())?,
        Request::ProjectExportStatus { destination, key, issuing_store } => {
            absolute(&destination.to_path_buf())?;
            if !(1..=4096).contains(&key.len()) || !(1..=4096).contains(&issuing_store.len()) {
                return Err(invalid("original publication key and issuer must be nonempty and bounded"));
            }
        }
        Request::Project { canonical, .. } => canonical.validate()?,
        #[cfg(feature = "product-operators")]
        Request::InspectCanonicalCatalog {
            state, timeout_ms, ..
        } => {
            absolute(&state.to_path_buf())?;
            catalog::timeout(*timeout_ms)?;
        }
        Request::Processes {
            directory,
            limit: requested,
            cursor,
        } => {
            absolute(directory)?;
            limit(*requested)?;
            if cursor.as_ref().is_some_and(|cursor| {
                cursor.after.as_ref().is_none_or(|name| {
                    Path::new(name).components().count() != 1 || name == "." || name == ".."
                })
            }) {
                return Err(invalid("invalid checkpoint catalogue cursor"));
            }
        }
        Request::Inspect {
            source,
            request,
            invocation,
        }
        | Request::Follow {
            source,
            request,
            invocation,
            ..
        } => {
            absolute(source.path())?;
            if request.include_related {
                limit(request.related_limit)?;
            }
            if let Some(path) = invocation {
                absolute(path)?;
            }
        }
        Request::Page {
            source,
            section: Section::Related,
            producer: Some(_),
            ..
        } => {
            absolute(source.path())?;
            return Err(invalid("related-operation pages do not select a producer"));
        }
        Request::Page {
            source,
            limit: requested,
            ..
        } => {
            absolute(source.path())?;
            limit(*requested)?;
        }
        Request::Study {
            canonical, request, ..
        } => {
            canonical.validate()?;
            request
                .validate()
                .map_err(|error| ReaderError::Workflow(Box::new(error)))?;
        }
        Request::Attempt { canonical, request } => {
            canonical.validate()?;
            request
                .validate()
                .map_err(|error| ReaderError::Workflow(Box::new(error)))?;
        }
        Request::Investigate { request: selected } => {
            if selected.has_effects() && request.access != Access::Investigation {
                return Err(invalid(
                    "capture/export require explicit investigation admission; passive reads never attach or publish",
                ));
            }
            investigation::validate(selected)?;
        }
    }
    Ok(())
}
/// Actual product-owned directional schema. This never reads scientific data or prepares tools.
pub fn schema(contract: Contract) -> Schema {
    match contract {
        Contract::Request => {
            let mut schema = SchemaSettings::draft2020_12()
                .into_generator()
                .into_root_schema_for::<ReaderRequest>();
            schema.insert("$id".into(), REQUEST_SCHEMA.into());
            schema
        }
        Contract::Response => {
            let mut schema = SchemaSettings::draft2020_12()
                .for_serialize()
                .into_generator()
                .into_root_schema_for::<ReaderResponse>();
            schema.insert("$id".into(), RESPONSE_SCHEMA.into());
            schema
        }
    }
}
fn unavailable(path: &Path, code: &str, error: &dyn std::error::Error) -> Payload {
    Payload::Unavailable {
        source: path.to_owned(),
        failure: Failure::capture(code, error),
    }
}
fn directory_identity(directory: &Path) -> Result<(String, usize), std::io::Error> {
    let mut entries = std::collections::BTreeMap::new();
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        if entry
            .path()
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            let metadata = std::fs::symlink_metadata(entry.path())?;
            entries.insert(
                entry.file_name(),
                (
                    metadata.dev(),
                    metadata.ino(),
                    metadata.len(),
                    metadata.mtime(),
                    metadata.mtime_nsec(),
                    metadata.ctime(),
                    metadata.ctime_nsec(),
                ),
            );
        }
    }
    let count = entries.len();
    let mut digest = Sha256::new();
    for (name, metadata) in entries {
        digest.update((name.as_encoded_bytes().len() as u64).to_le_bytes());
        digest.update(name.as_encoded_bytes());
        digest.update(metadata.0.to_le_bytes());
        digest.update(metadata.1.to_le_bytes());
        digest.update(metadata.2.to_le_bytes());
        digest.update(metadata.3.to_le_bytes());
        digest.update(metadata.4.to_le_bytes());
        digest.update(metadata.5.to_le_bytes());
        digest.update(metadata.6.to_le_bytes());
    }
    Ok((
        digest
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
        count,
    ))
}
fn source_facts(
    path: &Path,
    identity: String,
    bytes: Option<usize>,
    examined: usize,
) -> SourceFacts {
    SourceFacts {
        path: path.to_owned(),
        identity,
        examined_bytes: bytes,
        examined_sources: examined,
        atomic_snapshot: false,
        changed_during_read: false,
    }
}
fn association(path: &Path, operation: OperationId) -> Result<InvocationAssociation, ReaderError> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take((REQUEST_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > REQUEST_BYTES {
        return Err(invalid(
            "supplied invocation evidence exceeds the reader bound",
        ));
    }
    let document: InvocationEvidenceDocument = serde_json::from_slice(&bytes)?;
    document.validate().map_err(ReaderError::Identity)?;
    if document.operation != Some(operation) {
        return Err(invalid(
            "supplied invocation evidence does not identify the selected diagnostic operation",
        ));
    }
    Ok(InvocationAssociation {
        run_id: document.run_id,
        attempt_id: document.attempt_id,
        operation,
        outcome: document.outcome,
        canonical_run: document.canonical_run,
        canonical_attempt: document.canonical_attempt,
    })
}
fn managed_candidate(
    image: &crate::observation::ProcessCheckpoint,
) -> Option<pse_diagnostics::process_facts::ManagedInvocationCandidate> {
    image
        .managed_invocation
        .as_ref()
        .filter(|candidate| {
            !candidate.invocation.is_empty()
                && candidate.invocation.len() <= 128
                && candidate.process.pid == image.locator.pid
                && Some(candidate.process.start_ticks) == image.locator.start_ticks
        })
        .cloned()
}
fn handle(
    request: Request,
    cancel: &pse_columnar::CancellationToken,
) -> Result<Payload, ReaderError> {
    cancel.checkpoint()?;
    match request {
        Request::Discover => Ok(Payload::Discovery {
            package_version: env!("CARGO_PKG_VERSION"),
            request_schema: REQUEST_SCHEMA,
            response_schema: RESPONSE_SCHEMA,
            operations: <Request as strum::VariantNames>::VARIANTS.to_vec(),
            request_bytes: REQUEST_BYTES,
            response_bytes: RESPONSE_BYTES,
            project_perform: cfg!(feature = "product-reader").then_some(1),
            project_import: cfg!(feature = "product-reader").then_some(1),
            project_import_open: cfg!(feature = "product-reader").then_some(1),
            project_export: cfg!(feature = "product-reader").then_some(1),
        }),
        Request::Schema { contract } => Ok(Payload::Schema {
            identity: match contract {
                Contract::Request => REQUEST_SCHEMA,
                Contract::Response => RESPONSE_SCHEMA,
            },
            document: schema(contract),
        }),
        Request::InspectProjectPack {
            input,
            maximum_document_bytes,
        } => {
            let maximum_document_bytes = maximum_document_bytes.map_or(
                crate::host_operations::project::PROJECT_DOCUMENT_BYTES,
                NonZeroUsize::get,
            );
            let bundle = crate::workflow::ProjectBundleReader::open(
                &input.to_path_buf(),
                maximum_document_bytes,
            )
            .map_err(|error| ReaderError::Workflow(Box::new(error)))?;
            cancel.checkpoint()?;
            let (manifest, manifest_digest) = bundle.into_inspection();
            Ok(Payload::ProjectPack {
                manifest: Box::new(manifest),
                manifest_digest,
            })
        }
        Request::ProjectExportStatus { destination, key, issuing_store } => {
            let status = crate::host_operations::project::exchange::publication_status(destination, &key, &issuing_store, cancel)
                .map_err(|error| ReaderError::Workflow(Box::new(error)))?;
            Ok(Payload::ProjectExportStatus { status: Box::new(status) })
        }
        Request::Follow {
            source,
            request,
            invocation,
            resume,
        } => follow::sample(source, request, invocation, resume),
        Request::Investigate { request } => investigation::handle(request, cancel),
        Request::Processes {
            directory,
            limit,
            cursor,
        } => {
            let (identity, examined) = match directory_identity(&directory) {
                Ok(value) => value,
                Err(error) => return Ok(unavailable(&directory, "source.unavailable", &error)),
            };
            let context = query(&(&directory, limit))?;
            if cursor
                .as_ref()
                .is_some_and(|cursor| cursor.source != identity || cursor.query != context)
            {
                return Err(invalid(
                    "checkpoint catalogue changed or cursor belongs to another reader selection",
                ));
            }
            let page = match inspection::list_processes(
                &directory,
                DiagnosticListRequest {
                    after: cursor.and_then(|cursor| cursor.after),
                    limit,
                },
            ) {
                Ok(page) => page,
                Err(error) => return Ok(unavailable(&directory, "source.unavailable", &error)),
            };
            let after = directory_identity(&directory).ok();
            let changed = after.as_ref().is_none_or(|(after, _)| *after != identity);
            let continuation = if changed {
                None
            } else {
                page.next.as_ref().map(|after| PageCursor {
                    source: identity.clone(),
                    query: context,
                    offset: 0,
                    after: Some(after.clone()),
                })
            };
            let mut source = source_facts(&directory, identity, None, examined);
            source.changed_during_read = changed;
            Ok(Payload::Processes {
                page,
                source,
                continuation,
            })
        }
        Request::Inspect {
            source,
            request,
            invocation,
        } => {
            let read = match read_checkpoint_source(source.path()) {
                Ok(read) => read,
                Err(error) => {
                    return Ok(unavailable(source.path(), "checkpoint.unavailable", &error));
                }
            };
            source.validate(&read.image)?;
            let association = invocation
                .as_ref()
                .map(|path| association(path, request.operation))
                .transpose()?;
            let process = ProcessFacts {
                managed_invocation: managed_candidate(&read.image),
                current_start_identity: read
                    .image
                    .locator
                    .start_ticks
                    .map(|_| read.image.locator.is_current()),
                process: read.image.process.clone(),
                locator: read.image.locator.clone(),
                publication_sequence: read.image.sequence,
                published_unix_micros: read.image.published_unix_micros,
                resources: read.image.resources.clone(),
            };
            let facts = source_facts(source.path(), read.sha256, Some(read.bytes), 1);
            let document = inspection::inspect_image(source.path(), request, read.image);
            Ok(Payload::Inspection {
                document: Box::new(document),
                process,
                source: facts,
                association,
            })
        }
        Request::Page {
            source,
            operation_id,
            section,
            producer,
            limit,
            cursor,
        } => {
            let read = match read_checkpoint_source(source.path()) {
                Ok(read) => read,
                Err(error) => {
                    return Ok(unavailable(source.path(), "checkpoint.unavailable", &error));
                }
            };
            source.validate(&read.image)?;
            let context = query(&(&source, operation_id, section, producer, limit))?;
            if cursor.as_ref().is_some_and(|cursor| {
                cursor.source != read.sha256 || cursor.query != context || cursor.after.is_some()
            }) {
                return Err(invalid(
                    "checkpoint changed or continuation belongs to another operation/section/reader selection",
                ));
            }
            let Some(operation) = read
                .image
                .operations
                .iter()
                .find(|operation| operation.id == operation_id)
            else {
                return Ok(unavailable(
                    source.path(),
                    "operation.unavailable",
                    &invalid("selected operation is absent from the copied checkpoint"),
                ));
            };
            let offset = cursor.map_or(0, |cursor| cursor.offset);
            // Assess freshness under the existing native policy, while retaining exact copied
            // row membership. A later wall clock cannot change a byte-bound page's offsets.
            let now = read
                .image
                .clock
                .as_ref()
                .and_then(|clock| clock.observation_now());
            let mut assessed = pse_diagnostics::inspection::select_window(
                operation.clone(),
                now.unwrap_or(operation.read_at),
                operation.requested_interval,
            );
            if now.is_none() {
                assessed.coverage.stale = true;
            }
            let (records, known, partial) = match section {
                Section::Related => (
                    Records::Related(
                        operation
                            .children
                            .iter()
                            .skip(offset)
                            .take(limit)
                            .cloned()
                            .collect(),
                    ),
                    operation.children.len(),
                    operation.coverage.curtailed
                        || operation.detail_partial
                        || read.image.omitted > 0,
                ),
                Section::Records => {
                    let selected: Vec<_> = operation
                        .producers
                        .iter()
                        .filter(|selected| producer.is_none_or(|id| selected.descriptor.id == id))
                        .collect();
                    if producer.is_some() && selected.is_empty() {
                        return Err(invalid("selected producer is absent from this operation"));
                    }
                    // Native terminal compaction may preserve current while releasing the
                    // optional recent ring. Preserve that whole current tuple as well.
                    let retained: Vec<_> = selected
                        .iter()
                        .flat_map(|selected| {
                            selected.recent.iter().chain(selected.current.iter().filter(
                                |current| {
                                    !selected.recent.iter().any(|record| {
                                        record.producer == current.producer
                                            && record.generation == current.generation
                                            && record.sequence == current.sequence
                                    })
                                },
                            ))
                        })
                        .collect();
                    let known = retained.len();
                    let partial = operation.coverage.curtailed
                        || operation.detail_partial
                        || selected.iter().any(|selected| {
                            selected.coverage.partial || selected.coverage.history_dropped > 0
                        });
                    (
                        Records::Records(
                            retained
                                .into_iter()
                                .skip(offset)
                                .take(limit)
                                .cloned()
                                .collect(),
                        ),
                        known,
                        partial,
                    )
                }
            };
            let count = match &records {
                Records::Related(records) => records.len(),
                Records::Records(records) => records.len(),
            };
            let next = offset.saturating_add(count);
            let continuation = (next < known).then(|| PageCursor {
                source: read.sha256.clone(),
                query: context,
                offset: next,
                after: None,
            });
            Ok(Payload::Page {
                page: RecordPage {
                    managed_invocation: managed_candidate(&read.image),
                    operation: operation_id,
                    operation_kind: operation.kind.clone(),
                    outcome: operation.outcome,
                    coverage: assessed.coverage,
                    detail_partial: operation.detail_partial,
                    summary: operation.summary.clone(),
                    stale_after: operation.stale_after,
                    observed_at: operation.observed_at,
                    read_at: assessed.read_at,
                    clock: read.image.clock.clone(),
                    publication_sequence: read.image.sequence,
                    published_unix_micros: read.image.published_unix_micros,
                    records,
                    known_records: known,
                    total: (!partial).then_some(known),
                    partial,
                    continuation,
                    source: source_facts(source.path(), read.sha256, Some(read.bytes), 1),
                },
            })
        }
        Request::Study { .. } | Request::Attempt { .. } | Request::Project { .. } => Err(invalid(
            "canonical requests require their owned encoding boundary",
        )),
        #[cfg(feature = "product-operators")]
        Request::InspectCanonicalCatalog { .. } => Err(invalid(
            "canonical requests require their owned encoding boundary",
        )),
    }
}
fn correlation(bytes: &[u8]) -> String {
    #[derive(Deserialize)]
    struct Correlation {
        correlation: String,
    }
    if bytes.len() > REQUEST_BYTES {
        String::new()
    } else {
        serde_json::from_slice::<Correlation>(bytes)
            .map(|value| value.correlation)
            .unwrap_or_default()
    }
}
fn decode(bytes: &[u8]) -> Result<ReaderRequest, ReaderError> {
    if bytes.len() > REQUEST_BYTES {
        return Err(invalid("request exceeds the reader extent bound"));
    }
    let request: ReaderRequest = serde_json::from_slice(bytes)?;
    validate(&request)?;
    Ok(request)
}
fn response(correlation: String, result: Result<Payload, ReaderError>) -> ReaderResponse {
    response_with_policy(
        correlation,
        result,
        &pse_host::envelope::RedactionPolicy::from_environment(),
    )
}

fn response_with_policy(
    correlation: String,
    result: Result<Payload, ReaderError>,
    policy: &pse_host::envelope::RedactionPolicy,
) -> ReaderResponse {
    let (category, payload, original_failure) = match result {
        Ok(payload) => (Category::Success, payload, None),
        Err(error) => {
            let native = pse_host::envelope::ErrorRecord::from_typed(&error, policy);
            let failure = Failure {
                code: "request.refused".into(),
                message: native.message.clone(),
                causes: native.causes.clone(),
                native: Some(native),
            };
            (
                Category::Refused,
                Payload::Refusal { failure },
                Some(Box::new(error)),
            )
        }
    };
    let next_request = match &payload {
        Payload::Follow { sample } => sample.next_request(),
        _ => None,
    };
    fn candidate(
        payload: &Payload,
    ) -> Option<pse_diagnostics::process_facts::ManagedInvocationCandidate> {
        match payload {
            Payload::Inspection { process, .. } => process.managed_invocation.clone(),
            Payload::Follow { sample } => candidate(&sample.observation),
            Payload::Page { page } => page.managed_invocation.clone(),
            _ => None,
        }
    }
    let managed_invocation = candidate(&payload);
    ReaderResponse {
        protocol: Protocol::ProductReader,
        version: Version,
        correlation,
        schema: RESPONSE_SCHEMA,
        category,
        payload,
        next_request,
        managed_invocation,
        original_failure,
    }
}
/// Complete producer-owned frame after all required protected page/client owners settled.
pub struct EncodedResponse {
    frame: Vec<u8>,
    code: i32,
    original_failure: Option<Box<ReaderError>>,
}
impl std::fmt::Debug for EncodedResponse {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("EncodedResponse")
            .field("frame_bytes", &self.frame.len())
            .field("code", &self.code)
            .finish()
    }
}
impl EncodedResponse {
    /// Complete exact encoded product response, including terminal newline.
    pub fn bytes(&self) -> &[u8] {
        &self.frame
    }
    /// Product command category status, independent of original scientific outcomes.
    pub fn exit_code(&self) -> i32 {
        self.code
    }
    /// Preserve a native refusal if its actual encoding/output/cancellation also fails.
    /// The original typed error is retained directly, never reconstructed from the wire frame.
    pub fn delivery_failed(self, transport: ReaderError) -> ReaderError {
        delivery_failure(self.original_failure, transport)
    }
    fn settlement_failed(self, cleanup: ReaderError) -> ReaderError {
        match self.original_failure {
            Some(operation) => ReaderError::Settlement {
                operation,
                cleanup: Box::new(cleanup),
            },
            None => cleanup,
        }
    }
}
fn delivery_failure(original: Option<Box<ReaderError>>, transport: ReaderError) -> ReaderError {
    match original {
        Some(response) => ReaderError::Delivery {
            response,
            transport: Box::new(transport),
        },
        None => transport,
    }
}
fn encode_response(mut response: ReaderResponse) -> Result<EncodedResponse, ReaderError> {
    let mut frame = match serde_json::to_vec(&response) {
        Ok(frame) => frame,
        Err(error) => {
            return Err(delivery_failure(
                response.original_failure.take(),
                error.into(),
            ));
        }
    };
    let mut code = if response.category == Category::Success {
        0
    } else {
        2
    };
    if frame.len().saturating_add(1) > RESPONSE_BYTES {
        let extent = invalid("whole product response exceeds its declared transport extent");
        let refusal = ReaderResponse {
            protocol: Protocol::ProductReader,
            version: Version,
            correlation: response.correlation.clone(),
            schema: RESPONSE_SCHEMA,
            category: Category::Refused,
            payload: Payload::Refusal {
                failure: Failure::capture("response.extent", &extent),
            },
            next_request: None,
            managed_invocation: None,
            original_failure: None,
        };
        frame = match serde_json::to_vec(&refusal) {
            Ok(frame) => frame,
            Err(error) => {
                return Err(delivery_failure(
                    response.original_failure.take(),
                    error.into(),
                ));
            }
        };
        response.original_failure = Some(Box::new(delivery_failure(
            response.original_failure.take(),
            extent,
        )));
        code = 2;
    }
    frame.push(b'\n');
    Ok(EncodedResponse {
        frame,
        code,
        original_failure: response.original_failure.take(),
    })
}
/// Decode one passive request and encode its owned native response before releasing custody.
/// Canonical pages remain typed and protected through encoding, then join owned client drain.
/// The caller's token stops only this observer's query/capture/read. This synchronous boundary
/// retains its native joins; an async process root must join its blocking task rather than drop it.
pub fn encode(
    bytes: &[u8],
    cancel: &pse_columnar::CancellationToken,
) -> Result<EncodedResponse, ReaderError> {
    if let Err(error) = cancel.checkpoint() {
        return encode_response(response(correlation(bytes), Err(error.into())));
    }
    match decode(bytes) {
        Ok(ReaderRequest {
            correlation,
            request:
                Request::Study {
                    canonical,
                    request,
                    cursor,
                },
            ..
        }) => canonical::encode(
            correlation,
            canonical,
            canonical::Read::Study { request, cursor },
            cancel,
        ),
        Ok(ReaderRequest {
            correlation,
            request: Request::Attempt { canonical, request },
            ..
        }) => canonical::encode(
            correlation,
            canonical,
            canonical::Read::Attempt { request },
            cancel,
        ),
        Ok(ReaderRequest {
            correlation,
            request: Request::Project { canonical, query },
            ..
        }) => canonical::project::encode(correlation, canonical, query, cancel),
        #[cfg(feature = "product-operators")]
        Ok(ReaderRequest {
            correlation,
            request:
                Request::InspectCanonicalCatalog {
                    state,
                    database,
                    expected_issuer,
                    timeout_ms,
                },
            ..
        }) => catalog::encode(
            correlation,
            state,
            database,
            &expected_issuer,
            timeout_ms,
            cancel,
        ),
        Ok(ReaderRequest {
            correlation,
            request,
            ..
        }) => encode_response(response(correlation, handle(request, cancel))),
        Err(error) => encode_response(response(correlation(bytes), Err(error))),
    }
}
#[cfg(test)]
mod tests;
