// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Product-owned passive checkpoint and retained-history protocol, without scientific execution.
mod canonical;
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
use pse_diagnostics::observation::{
    ChildSnapshot, CoherentRecord, Coverage, OperationId, Outcome, ProducerId,
};
use pse_model::document::Version;
use schemars::{JsonSchema, Schema, generate::SchemaSettings};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
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
/// Passive requests only. Configured canonical connections never start a database or science.
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
}
impl Failure {
    fn capture(code: &str, error: &dyn std::error::Error) -> Self {
        let mut causes = Vec::new();
        let mut cause = error.source();
        while let Some(error) = cause {
            causes.push(error.to_string());
            cause = error.source();
        }
        Self {
            code: code.into(),
            message: error.to_string(),
            causes,
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
    },
    /// Native directional schema document; it validates representation, not scientific meaning.
    Schema {
        /// Product-owned schema identity.
        identity: &'static str,
        /// Actual directional schema document.
        #[schemars(with = "serde_json::Value")]
        document: Schema,
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
    /// Independent read/encoding failure and original owned-client drain failure.
    #[error("product reader operation failed: {operation}; client drain also failed: {cleanup}")]
    Settlement {
        /// Original read or serialization failure.
        #[source]
        operation: Box<ReaderError>,
        /// Original independently attempted client drain failure.
        cleanup: Box<ReaderError>,
    },
    /// A malformed selection, changed continuation, or unsupported bound.
    #[error("product reader selection: {0}")]
    Invalid(String),
}
pse_diagnostics::impl_diagnostic! {
    ReaderError,
    code(_this) { Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure) },
    forward(this) { match this { Self::Identity(error) => Some(error),Self::Canonical(error)=>Some(error.as_ref()),Self::Workflow(error)=>Some(error.as_ref()), _ => None } },
    help(_this) { None }, related(this) { match this {Self::Settlement{operation,cleanup}=>{let errors:[&dyn pse_diagnostics::TypedDiagnostic;2]=[operation.as_ref(),cleanup.as_ref()];Some(Box::new(errors.into_iter()))},_=>None} }, source(_this) { None }
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
fn handle(request: Request) -> Result<Payload, ReaderError> {
    match request {
        Request::Discover => Ok(Payload::Discovery {
            package_version: env!("CARGO_PKG_VERSION"),
            request_schema: REQUEST_SCHEMA,
            response_schema: RESPONSE_SCHEMA,
            operations: <Request as strum::VariantNames>::VARIANTS.to_vec(),
            request_bytes: REQUEST_BYTES,
            response_bytes: RESPONSE_BYTES,
        }),
        Request::Schema { contract } => Ok(Payload::Schema {
            identity: match contract {
                Contract::Request => REQUEST_SCHEMA,
                Contract::Response => RESPONSE_SCHEMA,
            },
            document: schema(contract),
        }),
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
        Request::Study { .. } | Request::Attempt { .. } => Err(invalid(
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
    let (category, payload) = match result {
        Ok(payload) => (Category::Success, payload),
        Err(error) => (
            Category::Refused,
            Payload::Refusal {
                failure: Failure::capture("request.refused", &error),
            },
        ),
    };
    ReaderResponse {
        protocol: Protocol::ProductReader,
        version: Version,
        correlation,
        schema: RESPONSE_SCHEMA,
        category,
        payload,
    }
}
/// Complete producer-owned frame after all required protected page/client owners settled.
pub struct EncodedResponse {
    frame: Vec<u8>,
    code: i32,
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
}
fn encode_response(response: &ReaderResponse) -> Result<EncodedResponse, ReaderError> {
    let mut frame = serde_json::to_vec(response)?;
    let mut code = if response.category == Category::Success {
        0
    } else {
        2
    };
    if frame.len().saturating_add(1) > RESPONSE_BYTES {
        let refusal = ReaderResponse {
            protocol: Protocol::ProductReader,
            version: Version,
            correlation: response.correlation.clone(),
            schema: RESPONSE_SCHEMA,
            category: Category::Refused,
            payload: Payload::Refusal {
                failure: Failure::capture(
                    "response.extent",
                    &invalid("whole product response exceeds its declared transport extent"),
                ),
            },
        };
        frame = serde_json::to_vec(&refusal)?;
        code = 2;
    }
    frame.push(b'\n');
    Ok(EncodedResponse { frame, code })
}
/// Decode one passive request and encode its owned native response before releasing custody.
/// Canonical pages remain typed and protected through encoding, then join owned client drain.
pub fn encode(bytes: &[u8]) -> Result<EncodedResponse, ReaderError> {
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
        ),
        Ok(ReaderRequest {
            correlation,
            request: Request::Attempt { canonical, request },
            ..
        }) => canonical::encode(correlation, canonical, canonical::Read::Attempt { request }),
        Ok(ReaderRequest {
            correlation,
            request,
            ..
        }) => encode_response(&response(correlation, handle(request))),
        Err(error) => encode_response(&response(correlation(bytes), Err(error))),
    }
}
/// One bounded stdin request and one privately prepared stdout frame. No partial serialization
/// can become evidence; an actual output I/O failure remains a transport failure.
pub fn serve(mut input: impl Read, mut output: impl Write) -> Result<i32, ReaderError> {
    let mut bytes = Vec::new();
    input
        .by_ref()
        .take((REQUEST_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    let encoded = encode(&bytes)?;
    output.write_all(encoded.bytes())?;
    Ok(encoded.exit_code())
}
#[cfg(test)]
mod tests;
