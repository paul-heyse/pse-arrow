// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original capture-to-invocation association within one explicitly supplied evidence set.
use crate::{
    invocation::InvocationOutcome,
    observation::ObservationError,
    tooling::{
        capture::{CaptureKind, CaptureLoss, CaptureStatus},
        frozen_inspection::{self, FrozenEvidenceSet, MAXIMUM_DOCUMENT_BYTES},
    },
    workflow::ReplayNativeRecordCoverage,
};
use datafusion::{
    arrow::{
        array::{Array, Int64Array, StringArray, UInt64Array},
        datatypes::{DataType, Field, Schema},
        record_batch::RecordBatch,
    },
    datasource::MemTable,
    error::DataFusionError,
    execution::{
        disk_manager::{DiskManagerBuilder, DiskManagerMode},
        runtime_env::RuntimeEnvBuilder,
    },
    prelude::{SessionConfig, SessionContext},
};
use futures_util::StreamExt as _;
use pse_columnar::{AllocationLease, CancellationToken, MemoryConsumer, MemoryPool};
use pse_diagnostics::observation::{OperationId, RecordAvailability};
use pse_model::{
    document::Version,
    generated::identities::{AttemptId, RunId},
};
use serde::{Deserialize, Serialize};
use std::{io::Read as _, num::NonZeroUsize, path::PathBuf, sync::Arc, time::Duration};

const MAXIMUM_MEMORY_BYTES: usize = 256 << 20;

/// Original supplied source, analytical engine, or selection refusal.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The explicitly selected frozen evidence member could not be read.
    #[error("scientific capture query I/O: {0}")]
    Io(#[from] std::io::Error),
    /// Existing frozen evidence codec or identity validation refused.
    #[error(transparent)]
    Frozen(#[from] ObservationError),
    /// The original allocation or reader cancellation cause.
    #[error(transparent)]
    Columnar(#[from] pse_columnar::CanonError),
    /// DataFusion's original planning or execution cause.
    #[error("scientific capture query engine: {0}")]
    Engine(#[from] DataFusionError),
    /// The requested extent or maintained relation shape is unsupported.
    #[error("scientific capture query: {0}")]
    Invalid(String),
}
pse_diagnostics::impl_diagnostic! {
    Error,
    code(this) { match this {
        Self::Frozen(_) | Self::Columnar(_) => None,
        Self::Engine(DataFusionError::ResourcesExhausted(_)) => Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit),
        _ => Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure),
    } },
    forward(this) { match this { Self::Frozen(error) => Some(error), Self::Columnar(error) => Some(error), _ => None } },
    help(_this) { None }, related(_this) { None }, source(_this) { None }
}

/// The exact source and finite observer allocation for this maintained question.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Request {
    /// Explicit absolute evidence-set v3 file; no path embedded in it is followed.
    pub evidence: PathBuf,
    /// Accounted working allocation for ingress, projections, execution and response.
    pub memory_bytes: NonZeroUsize,
}

/// What the supplied original operation identities establish.
#[derive(Debug, Serialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Association {
    /// One original invocation in the supplied set records this exact root operation.
    Linked {
        /// Index in `Document.invocations`, which preserves the original member order.
        invocation_member: u64,
    },
    /// The capture records no diagnostic root operation.
    OperationUnavailable,
    /// The root operation is known; no supplied invocation records it.
    InvocationUnavailable,
    /// Multiple supplied invocations record the root; no single run or attempt can be assigned.
    Ambiguous {
        /// Distinct original invocation members sharing this diagnostic root.
        candidates: u64,
    },
}

/// Original capture metadata and its attribution, independent of scientific outcome.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct CaptureFact {
    /// Exact original JSON pointer in the supplied image.
    pub pointer: String,
    /// Recorded diagnostic root, when supplied.
    pub operation: Option<OperationId>,
    /// Exact operation-key association, scoped to this image.
    pub association: Association,
    /// Original selected collection mechanism.
    pub kind: CaptureKind,
    /// Original exporter lifecycle, independent of the target outcome.
    pub status: CaptureStatus,
    /// Requested capture window length on the exporter clock.
    #[schemars(with = "pse_model::document::ClosedDuration")]
    pub requested_duration: Duration,
    /// Original exporter elapsed collection window; not an invocation interval.
    #[schemars(with = "pse_model::document::ClosedDuration")]
    pub observed_duration: Duration,
    /// Original loss and unknown coverage, including exact UInt64 counters.
    pub loss: CaptureLoss,
    /// Begun intervals with no observed end.
    pub open_intervals: usize,
}

/// Original scientific invocation facts, even when capture or timing is unavailable.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct InvocationFact {
    /// Exact original JSON pointer in the supplied image.
    pub pointer: String,
    /// Original run identity.
    pub run_id: RunId,
    /// Original durable attempt identity, when recorded.
    pub attempt_id: Option<AttemptId>,
    /// Original diagnostic operation, when recorded.
    pub operation: Option<OperationId>,
    /// Original recorded invocation outcome.
    pub outcome: InvocationOutcome,
    /// Original invocation clock scope; no mapping to the capture clock is inferred.
    pub clock_scope: String,
    /// Original timing availability cause, when supplied.
    pub timing_unavailable: Option<RecordAvailability>,
    /// Original interval omission count.
    pub omitted_intervals: usize,
    /// Original native attempt record coverage.
    pub native_record_coverage: ReplayNativeRecordCoverage,
    /// Original native attempt record omission count.
    pub omitted_native_attempts: usize,
}

/// Answer over one exact supplied image; success does not issue a verdict or grant access.
#[derive(Debug, Serialize, schemars::JsonSchema)]
#[schemars(rename = "ScientificCaptureAnalysisDocument")]
pub struct Document {
    /// Answer codec.
    pub version: Version<1>,
    /// The one file explicitly opened by this request.
    pub evidence: PathBuf,
    /// SHA-256 of the exact supplied image bytes, independent of producer authenticity.
    pub evidence_sha256: String,
    /// The maintained SQL actually executed.
    pub query: &'static str,
    /// One row per supplied capture, in original member order.
    pub captures: Vec<CaptureFact>,
    /// Supplied original invocation members, in original member order.
    pub invocations: Vec<InvocationFact>,
    /// Limits that apply to the whole answer.
    pub limitations: [&'static str; 3],
}

/// Retains the admitted result allocation through response encoding.
#[derive(Debug)]
pub struct Analysis {
    document: Document,
    _owner: Arc<AllocationLease>,
}
impl Analysis {
    /// Borrow the passive response without transferring its allocation owner.
    pub fn document(&self) -> &Document {
        &self.document
    }
}

// Count the candidate set before joining so shared roots never multiply capture rows.
const SQL: &str = "WITH invocation_roots AS (
    SELECT operation, COUNT(*) AS candidates, MIN(member) AS first_member
    FROM supplied_invocations WHERE operation IS NOT NULL GROUP BY operation
)
SELECT c.member, i.candidates,
    CASE WHEN i.candidates = 1 THEN i.first_member END AS invocation_member
FROM supplied_captures c LEFT JOIN invocation_roots i ON c.operation = i.operation
ORDER BY c.member";

fn invalid(error: impl std::fmt::Display) -> Error {
    Error::Invalid(error.to_string())
}

/// Refuse invalid paths and oversized allocations before source access.
pub fn validate(request: &Request) -> Result<(), Error> {
    if !request.evidence.is_absolute() {
        return Err(invalid("scientific capture query requires an explicit absolute input path"));
    }
    if request.memory_bytes.get() > MAXIMUM_MEMORY_BYTES {
        return Err(invalid("scientific capture query allocation must be at most 256 MiB"));
    }
    Ok(())
}

fn reserve(
    pool: &Arc<dyn MemoryPool>,
    name: &'static str,
    bytes: usize,
) -> Result<Arc<AllocationLease>, Error> {
    let reservation = MemoryConsumer::new(name).register(pool);
    reservation.try_grow(bytes)?;
    Ok(AllocationLease::new(reservation))
}

fn extent(rows: usize, bytes: usize) -> Result<usize, Error> {
    rows.checked_mul(bytes)
        .and_then(|n| n.checked_add(65536))
        .ok_or_else(|| invalid("scientific capture query allocation extent overflow"))
}

fn acquire(
    request: &Request,
    pool: &Arc<dyn MemoryPool>,
    cancel: &CancellationToken,
) -> Result<(FrozenEvidenceSet, String, Arc<AllocationLease>), Error> {
    // Open the supplied path once. NONBLOCK allows regular-file admission before any FIFO read.
    let mut file: std::fs::File = rustix::fs::open(
        &request.evidence,
        rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::CLOEXEC | rustix::fs::OFlags::NONBLOCK,
        rustix::fs::Mode::empty(),
    )
    .map_err(std::io::Error::from)?
    .into();
    let metadata = file.metadata()?;
    let length = usize::try_from(metadata.len()).map_err(invalid)?;
    if !metadata.is_file() || length > MAXIMUM_DOCUMENT_BYTES {
        return Err(invalid("scientific capture query requires a regular evidence file within 64 MiB"));
    }
    let owner = reserve(pool, "scientific-capture-query:source", extent(length, 8)?)?;
    let mut bytes = vec![0u8; length];
    for chunk in bytes.chunks_mut(64 << 10) {
        cancel.checkpoint()?;
        file.read_exact(chunk)?;
    }
    if file.read(&mut [0u8; 1])? != 0 {
        return Err(invalid("scientific capture evidence grew during acquisition"));
    }
    cancel.checkpoint()?;
    let digest = pse_host::test_workflow::execute::ContentIdentity::of(&bytes).sha256;
    let set: FrozenEvidenceSet = frozen_inspection::decode_document::<_, 3>(&bytes)?;
    set.validate()?;
    cancel.checkpoint()?;
    Ok((set, digest, owner))
}

fn table(
    relation: &'static str,
    operations: impl IntoIterator<Item = Option<OperationId>>,
) -> Result<Arc<MemTable>, Error> {
    let registry = pse_schema::registry().map_err(invalid)?;
    let declaration = registry
        .relation(relation)
        .ok_or_else(|| invalid("missing declared diagnostic view"))?;
    let declared = pse_schema::arrow::relation_schema_ref(registry, declaration).map_err(invalid)?;
    // Only the existing operation coordinate participates; member is a query-local row locator.
    let schema = Arc::new(Schema::new(vec![
        Field::new("member", DataType::UInt64, false),
        declared.field_with_name("operation").map_err(invalid)?.clone(),
    ]));
    let operations = StringArray::from_iter(
        operations.into_iter().map(|op| op.map(|op| op.to_string())),
    );
    let count = u64::try_from(operations.len()).map_err(invalid)?;
    let members = UInt64Array::from_iter_values(0..count);
    let batch = RecordBatch::try_new(schema.clone(), vec![Arc::new(members), Arc::new(operations)])
        .map_err(invalid)?;
    Ok(Arc::new(MemTable::try_new(schema, vec![vec![batch]])?))
}

fn column<'a, T: Array + 'static>(batch: &'a RecordBatch, name: &str) -> Result<&'a T, Error> {
    batch.column_by_name(name).and_then(|a| a.as_any().downcast_ref())
        .ok_or_else(|| invalid(format!("scientific capture query returned unexpected column {name}")))
}

/// Read exactly one supplied frozen set and execute the fixed original identity join.
pub async fn analyze(request: Request, cancel: &CancellationToken) -> Result<Analysis, Error> {
    cancel.checkpoint()?;
    validate(&request)?;
    let pool: Arc<dyn MemoryPool> =
        Arc::new(pse_columnar::GreedyMemoryPool::new(request.memory_bytes.get()));
    let (set, evidence_sha256, _source_owner) = acquire(&request, &pool, cancel)?;
    let rows = set.invocations.len() + set.captures.len();
    let _table_owner = reserve(&pool, "scientific-capture-query:tables", extent(rows, 512)?)?;
    let text_bytes = set.invocations.iter().try_fold(0usize, |n, source| {
        n.checked_add(source.clock_scope.len())
    }).ok_or_else(|| invalid("scientific capture response text extent overflow"))?;
    let response_bytes = extent(rows, 2048)?
        .checked_add(extent(text_bytes, 8)?)
        .ok_or_else(|| invalid("scientific capture response extent overflow"))?;
    let owner = reserve(&pool, "scientific-capture-query:response", response_bytes)?;
    let runtime = RuntimeEnvBuilder::new()
        .with_memory_pool(pool)
        .with_disk_manager_builder(DiskManagerBuilder::default().with_mode(DiskManagerMode::Disabled))
        .build_arc()?;
    let context = SessionContext::new_with_config_rt(SessionConfig::new(), runtime);
    context.register_table("supplied_invocations", table(
        "runtime.diagnostic_invocations",
        set.invocations.iter().map(|source| source.operation),
    )?)?;
    context.register_table("supplied_captures", table(
        "runtime.diagnostic_captures",
        set.captures.iter().map(|source| source.operation),
    )?)?;
    let frame = cancel.until_cancelled(context.sql(SQL)).await??;
    let mut stream = cancel.until_cancelled(frame.execute_stream()).await??;
    let mut captures = Vec::with_capacity(set.captures.len());
    while let Some(batch) = cancel.until_cancelled(stream.next()).await? {
        let batch = batch?;
        let members = column::<UInt64Array>(&batch, "member")?;
        let candidates = column::<Int64Array>(&batch, "candidates")?;
        let invocations = column::<UInt64Array>(&batch, "invocation_member")?;
        for row in 0..batch.num_rows() {
            cancel.checkpoint()?;
            let index = captures.len();
            if members.is_null(row) || usize::try_from(members.value(row)).map_err(invalid)? != index {
                return Err(invalid("unexpected scientific capture member order"));
            }
            let source = set.captures.get(index).ok_or_else(|| invalid("unknown scientific capture member"))?;
            let count = (!candidates.is_null(row)).then(|| candidates.value(row));
            let invocation = (!invocations.is_null(row)).then(|| invocations.value(row));
            let association = match (source.operation, count, invocation) {
                (None, None, None) => Association::OperationUnavailable,
                (Some(_), None, None) => Association::InvocationUnavailable,
                (Some(_), Some(1), Some(invocation_member)) => Association::Linked { invocation_member },
                (Some(_), Some(n), None) if n > 1 => Association::Ambiguous {
                    candidates: u64::try_from(n).map_err(invalid)?,
                },
                _ => return Err(invalid("inconsistent scientific capture association")),
            };
            captures.push(CaptureFact {
                pointer: format!("/captures/{index}"),
                operation: source.operation,
                association,
                kind: source.kind,
                status: source.status,
                requested_duration: source.requested_duration,
                observed_duration: source.observed_duration,
                loss: source.loss.clone(),
                open_intervals: source.open_intervals,
            });
        }
    }
    cancel.checkpoint()?;
    if captures.len() != set.captures.len() {
        return Err(invalid("incomplete scientific capture query result"));
    }
    let invocations = set.invocations.into_iter().enumerate().map(|(index, source)| InvocationFact {
        pointer: format!("/invocations/{index}"),
        run_id: source.run_id,
        attempt_id: source.attempt_id,
        operation: source.operation,
        outcome: source.outcome,
        clock_scope: source.clock_scope,
        timing_unavailable: source.timing_unavailable,
        omitted_intervals: source.omitted_intervals,
        native_record_coverage: source.attempts.native_record_coverage,
        omitted_native_attempts: source.attempts.omitted_native_attempts,
    }).collect();
    cancel.checkpoint()?;
    Ok(Analysis {
        document: Document {
            version: Version,
            evidence: request.evidence,
            evidence_sha256,
            query: SQL,
            captures,
            invocations,
            limitations: [
                "Association is scoped to this supplied image and uses only the exact recorded root operation. Missing matches do not prove that a capture or invocation never existed.",
                "Capture windows and invocation clocks have separate origins. This answer does not align clocks, order cross-clock events, or establish complete capture coverage.",
                "Recorded exporter state and invocation outcome grant no current control, rerun permission, scientific result access, or new verdict.",
            ],
        },
        _owner: owner,
    })
}
