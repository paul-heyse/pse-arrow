// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One maintained cross-invocation question over explicitly supplied original members.
use crate::tooling::capture::evidence::developer as tables;
use datafusion::{
    arrow::{
        array::{
            Array, BooleanArray, Int64Array, ListArray, StringArray, StructArray, UInt64Array,
        },
        record_batch::RecordBatch,
    },
    datasource::MemTable,
    execution::{
        disk_manager::{DiskManagerBuilder, DiskManagerMode},
        runtime_env::RuntimeEnvBuilder,
    },
    physical_plan::SendableRecordBatchStream,
    prelude::{SessionConfig, SessionContext},
};
use futures_util::StreamExt as _;
use pse_columnar::{AllocationLease, CancellationToken, MemoryConsumer, MemoryPool};
use pse_host::{
    developer_evidence,
    envelope::CliInvocationId,
    test_workflow::execute::{ContentIdentity, EvidenceScope},
};
use pse_model::document::Version;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, io::Read as _, num::NonZeroUsize, path::PathBuf, sync::Arc};

/// Original acquisition, engine or selection refusal; native causes are retained.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Original host descriptor, interpretation or report-commitment refusal.
    #[error(transparent)]
    Evidence(#[from] Box<developer_evidence::EvidenceError>),
    /// Original explicit member I/O refusal.
    #[error("developer evidence query I/O: {0}")]
    Io(#[from] std::io::Error),
    /// Original DataFusion planning, allocation or execution refusal.
    #[error("developer evidence query engine: {0}")]
    Engine(#[from] datafusion::error::DataFusionError),
    /// Original typed Arrow projection refusal.
    #[error(transparent)]
    Projection(#[from] crate::observation_detail::CaptureError),
    /// Original native cancellation or columnar control cause.
    #[error(transparent)]
    Columnar(#[from] pse_columnar::CanonError),
    /// Explicit population, source identity or shape refusal.
    #[error("developer evidence query: {0}")]
    Invalid(String),
}
pse_diagnostics::impl_diagnostic! {
    Error,
    code(this) { match this {
        Self::Evidence(_) | Self::Projection(_) | Self::Columnar(_) => None,
        Self::Engine(datafusion::error::DataFusionError::ResourcesExhausted(_)) => Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit),
        _ => Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure),
    } },
    forward(this) { match this { Self::Evidence(error) => Some(error.as_ref()), Self::Projection(error) => Some(error), Self::Columnar(error) => Some(error), _ => None } },
    help(_this) { None }, related(_this) { None }, source(_this) { None }
}

/// Exact explicit supplied files. Embedded report paths are never followed.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Member {
    /// Complete original descriptor artifact for the selected passive family.
    pub descriptors: PathBuf,
    /// Report-only compact evidence exported by its existing native owner.
    pub payload: PathBuf,
}
/// Finite observer allocation and explicit original population, independent of host placement.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Request {
    /// Explicit selected members; at most 32. An empty population is valid and stays empty.
    pub members: Vec<Member>,
    /// Shared input/projection/query/response allocation, never a host workload cap.
    pub memory_bytes: NonZeroUsize,
}
/// Attribution of a verified supplied member, descriptive on subsequent offline reading.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Source {
    /// Original explicit member ordinal in this query.
    pub member: u64,
    /// Only these explicitly selected paths were read.
    pub input: Member,
    /// Digest of exact original descriptor bytes, without normalization.
    pub descriptor_sha256: String,
    /// Digest of the exact supplied compact payload bytes.
    pub payload_sha256: String,
    /// Host original invocation; never a scientific run/attempt.
    pub invocation: CliInvocationId,
    /// Terminal-bound exact original report commitment.
    pub report_sha256: String,
    /// Original acquisition, coverage and authenticity limitations.
    pub limitations: Vec<String>,
}
/// One grouping of original facts, not a new verdict or present control observation.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Pattern {
    /// Original native action.
    pub action: String,
    /// Original performing owner's reconciliation.
    pub outcome: String,
    /// Source failure-tree kind; absent means no supplied /failures entry.
    pub diagnostic_kind: Option<String>,
    /// Original native code, or unavailable.
    pub code: Option<String>,
    /// Original native class, or unavailable.
    pub class: Option<String>,
    /// Original helper capture coverage, independent of manager settlement.
    pub required_capture_complete: bool,
    /// Recorded-at-read manager observation; never current physical state.
    pub recorded_manager_settlement: Option<String>,
    /// Original recorded pending state, unavailable when not supplied.
    pub recorded_native_lifetime_pending: Option<bool>,
    /// Distinct original invocations represented in this group.
    pub invocations: u64,
    /// Exact observed selected-count range; missing counts are excluded, never zero-filled.
    pub selected_tests_min: Option<u64>,
    /// Exact upper bound of the observed selected-count range.
    pub selected_tests_max: Option<u64>,
    /// Source ordinals resolving every contributing original report.
    pub members: Vec<u64>,
    /// Exact paired member and original diagnostic pointer; absence preserves an empty cause.
    pub references: Vec<Reference>,
}
/// Literal source attribution within the exact committed original report.
#[derive(Debug, Serialize, schemars::JsonSchema, PartialEq, Eq, PartialOrd, Ord)]
pub struct Reference {
    /// Explicit supplied member ordinal, resolving through Document.sources.
    pub member: u64,
    /// Exact original JSON pointer; absent when no supplied /failures entry exists.
    pub pointer: Option<String>,
}
/// Answer to the maintained cause/capture/recorded-settlement pattern question.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Document {
    /// Answer codec, independent of source report interpretations.
    pub version: Version<1>,
    /// Literal maintained SQL actually executed, never caller-supplied commands.
    pub query: &'static str,
    /// Exact original sources; empty selection remains empty.
    pub sources: Vec<Source>,
    /// Ordered generic relational summaries; original native outcomes remain authoritative.
    pub patterns: Vec<Pattern>,
    /// Unavailable clocks/generations/associations stay unavailable.
    pub limitations: Vec<String>,
}
/// Response encoding retains its admitted observer allocation.
#[derive(Debug)]
pub struct Analysis {
    document: Document,
    _owners: Vec<Arc<AllocationLease>>,
}
impl Analysis {
    /// Borrow passive output without transferring its allocation owner.
    pub fn document(&self) -> &Document {
        &self.document
    }
}
/// Maintained left join preserves empty supplied failure trees and all unknown groups.
pub const SQL: &str = "SELECT i.action, i.outcome, c.kind AS diagnostic_kind, c.code, c.class, i.required_capture_complete, i.recorded_manager_settlement, i.recorded_native_lifetime_pending, COUNT(DISTINCT i.invocation) AS invocations, MIN(i.selected_tests) AS selected_tests_min, MAX(i.selected_tests) AS selected_tests_max, ARRAY_AGG(DISTINCT i.member) AS members, ARRAY_AGG(DISTINCT named_struct('member', i.member, 'pointer', c.pointer)) AS original_references FROM selected_invocations i LEFT JOIN selected_causes c ON i.invocation = c.invocation AND i.report_sha256 = c.report_sha256 GROUP BY i.action, i.outcome, c.kind, c.code, c.class, i.required_capture_complete, i.recorded_manager_settlement, i.recorded_native_lifetime_pending ORDER BY i.action, i.outcome, c.kind NULLS FIRST, c.code NULLS FIRST, c.class NULLS FIRST, i.required_capture_complete, i.recorded_manager_settlement NULLS FIRST, i.recorded_native_lifetime_pending NULLS FIRST";
fn invalid(error: impl std::fmt::Display) -> Error {
    Error::Invalid(error.to_string())
}
fn reserve(pool: &Arc<dyn MemoryPool>, bytes: usize) -> Result<Arc<AllocationLease>, Error> {
    let reservation = MemoryConsumer::new("developer-evidence:query").register(pool);
    reservation.try_grow(bytes).map_err(Error::Engine)?;
    Ok(AllocationLease::new(reservation))
}
/// Reject unknown routes/bounds before reading any input.
pub fn validate(request: &Request) -> Result<(), Error> {
    if request.members.len() > 32 {
        return Err(invalid(
            "developer evidence query accepts at most 32 explicit members",
        ));
    }
    if request
        .members
        .iter()
        .any(|m| !m.descriptors.is_absolute() || !m.payload.is_absolute())
    {
        return Err(invalid(
            "developer evidence query requires explicit absolute input paths",
        ));
    }
    Ok(())
}
fn acquire(
    path: &std::path::Path,
    pool: &Arc<dyn MemoryPool>,
    owners: &mut Vec<Arc<AllocationLease>>,
) -> Result<Vec<u8>, Error> {
    // Preserve explicit-path symlink following, but never wait for a FIFO peer
    // before the selected descriptor's regular-file validation can run.
    let file: std::fs::File = rustix::fs::open(
        path,
        rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::CLOEXEC | rustix::fs::OFlags::NONBLOCK,
        rustix::fs::Mode::empty(),
    )
    .map_err(std::io::Error::from)?
    .into();
    let metadata = file.metadata()?;
    let maximum =
        u64::try_from(developer_evidence::binary_options().max_message_size()).map_err(invalid)?;
    if !metadata.is_file() || metadata.len() > maximum {
        return Err(invalid(
            "developer query member must be a regular file within the portable evidence family's input bound",
        ));
    }
    let length = usize::try_from(metadata.len()).map_err(invalid)?;
    let bytes = length
        .checked_mul(8)
        .and_then(|n| n.checked_add(65536))
        .ok_or_else(|| invalid("developer query source extent overflow"))?;
    owners.push(reserve(pool, bytes)?);
    let mut bytes = Vec::new();
    file.take(metadata.len() + 1).read_to_end(&mut bytes)?;
    if bytes.len() != length {
        return Err(invalid(
            "developer query member extent changed during acquisition",
        ));
    }
    Ok(bytes)
}

/// Read the supplied population once, then execute only the maintained relational question.
pub async fn analyze(request: Request, cancel: &CancellationToken) -> Result<Analysis, Error> {
    cancel.checkpoint()?;
    validate(&request)?;
    let pool: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(
        request.memory_bytes.get(),
    ));
    let mut owners = Vec::new();
    let mut facts = Vec::new();
    let mut sources = Vec::new();
    let mut identities = BTreeSet::new();
    for (member, input) in request.members.into_iter().enumerate() {
        cancel.checkpoint()?;
        let descriptor = acquire(&input.descriptors, &pool, &mut owners)?;
        let payload = acquire(&input.payload, &pool, &mut owners)?;
        let options = developer_evidence::binary_options();
        let descriptors = options
            .decode_from_slice::<buffa_descriptor::generated::descriptor::FileDescriptorSet>(
                &descriptor,
            )
            .map_err(|e| Error::Evidence(Box::new(e.into())))?;
        let original = developer_evidence::inspect_facts(
            &descriptors,
            "pse.developer.evidence.v1.CapturedWorkflowReport",
            &payload,
            &options,
        )
        .map_err(|e| Error::Evidence(Box::new(e)))?;
        if original.readback.scope != EvidenceScope::Report {
            return Err(invalid(
                "developer query requires report-only original members; export with --scope report",
            ));
        }
        if !identities.insert(original.report.invocation.to_string()) {
            return Err(invalid(
                "duplicate original developer invocation identity, including conflicting report commitments",
            ));
        }
        sources.push(Source {
            member: member as u64,
            input,
            descriptor_sha256: ContentIdentity::of(&descriptor).sha256,
            payload_sha256: ContentIdentity::of(&payload).sha256,
            invocation: original.report.invocation.clone(),
            report_sha256: original.readback.report_sha256.clone(),
            limitations: original.readback.limitations.clone(),
        });
        facts.push(original);
    }
    let invocations = tables::invocations(&facts, &pool, cancel)?;
    let causes = tables::causes(&facts, &pool, cancel)?;
    let output_extent = invocations
        .get_array_memory_size()
        .checked_add(causes.get_array_memory_size())
        .and_then(|n| n.checked_mul(8))
        .and_then(|n| n.checked_add(65536))
        .ok_or_else(|| invalid("developer query response extent overflow"))?;
    owners.push(reserve(&pool, output_extent)?);
    let runtime = RuntimeEnvBuilder::new()
        .with_memory_pool(pool)
        .with_disk_manager_builder(
            DiskManagerBuilder::default().with_mode(DiskManagerMode::Disabled),
        )
        .build_arc()
        .map_err(Error::Engine)?;
    let context = SessionContext::new_with_config_rt(SessionConfig::new(), runtime);
    for (name, batch) in [
        ("selected_invocations", invocations),
        ("selected_causes", causes),
    ] {
        context
            .register_table(
                name,
                Arc::new(
                    MemTable::try_new(batch.schema(), vec![vec![batch]]).map_err(Error::Engine)?,
                ),
            )
            .map_err(Error::Engine)?;
    }
    cancel.checkpoint()?;
    let query = cancel.until_cancelled(context.sql(SQL)).await??;
    let stream = cancel.until_cancelled(query.execute_stream()).await??;
    let patterns = read_patterns(stream, cancel).await?;
    cancel.checkpoint()?;
    Ok(Analysis { document: Document {
        version: Version, query: SQL, sources, patterns,
        limitations: vec![
            "This finite query summarizes supplied original observations; portable agreement does not authenticate producers or issue native verdicts or action permission.".into(),
            "These original portable roots do not record producer generation or source-clock coordinates. No duration, causal order or scientific run/attempt association is inferred.".into(),
            "Recorded manager settlement and pending state describe the original recorded-at-read observation, never present physical control or current readiness.".into(),
            "This query projects the original failures tree. Pending prerequisite declarations and per-group test/compiler outcomes are outside this observation; an empty diagnostic join does not prove that the original report had no other recorded causes.".into(),
        ],
    }, _owners: owners })
}

// DataFusion 55.1 DataFrame::execute_stream documents that dropping this stream
// aborts query execution and frees its resources. This is no external/native
// lifetime settlement claim. Cancellation retains its original CanonError.
async fn read_patterns(
    mut stream: SendableRecordBatchStream,
    cancel: &CancellationToken,
) -> Result<Vec<Pattern>, Error> {
    let mut patterns = Vec::new();
    loop {
        cancel.checkpoint()?;
        let Some(batch) = cancel.until_cancelled(stream.next()).await? else {
            break;
        };
        cancel.checkpoint()?;
        let batch = batch?;
        for row in 0..batch.num_rows() {
            cancel.checkpoint()?;
            patterns.push(pattern(&batch, row)?);
        }
    }
    // An empty batch/EOF can cancel in the same poll that completes the stream.
    cancel.checkpoint()?;
    Ok(patterns)
}
fn array<'a, T: Array + 'static>(batch: &'a RecordBatch, name: &str) -> Result<&'a T, Error> {
    batch
        .column_by_name(name)
        .and_then(|a| a.as_any().downcast_ref())
        .ok_or_else(|| invalid(format!("developer query returned unexpected column {name}")))
}
fn text(batch: &RecordBatch, name: &str, row: usize) -> Result<Option<String>, Error> {
    let values = array::<StringArray>(batch, name)?;
    Ok((!values.is_null(row)).then(|| values.value(row).to_owned()))
}
fn count(batch: &RecordBatch, name: &str, row: usize) -> Result<Option<u64>, Error> {
    let column = batch
        .column_by_name(name)
        .ok_or_else(|| invalid("missing developer query count"))?;
    if column.is_null(row) {
        return Ok(None);
    }
    if let Some(values) = column.as_any().downcast_ref::<UInt64Array>() {
        return Ok(Some(values.value(row)));
    }
    if let Some(values) = column.as_any().downcast_ref::<Int64Array>() {
        return Ok(Some(u64::try_from(values.value(row)).map_err(invalid)?));
    }
    Err(invalid("developer query count is not an exact integer"))
}
fn pattern(batch: &RecordBatch, row: usize) -> Result<Pattern, Error> {
    let values = array::<ListArray>(batch, "members")?.value(row);
    let values = values
        .as_any()
        .downcast_ref::<UInt64Array>()
        .ok_or_else(|| invalid("developer source membership is not UInt64"))?;
    let mut members = values.values().to_vec();
    members.sort_unstable();
    let reference_values = array::<ListArray>(batch, "original_references")?.value(row);
    let reference_values = reference_values
        .as_any()
        .downcast_ref::<StructArray>()
        .ok_or_else(|| invalid("developer attribution is not an original member/pointer struct"))?;
    let reference_members = reference_values
        .column_by_name("member")
        .and_then(|a| a.as_any().downcast_ref::<UInt64Array>())
        .ok_or_else(|| invalid("developer reference member is not UInt64"))?;
    let pointers = reference_values
        .column_by_name("pointer")
        .and_then(|a| a.as_any().downcast_ref::<StringArray>())
        .ok_or_else(|| invalid("developer reference pointer is not Utf8"))?;
    let mut references = (0..reference_values.len())
        .map(|index| {
            if reference_values.is_null(index) || reference_members.is_null(index) {
                return Err(invalid("original member attribution is missing"));
            }
            Ok(Reference {
                member: reference_members.value(index),
                pointer: (!pointers.is_null(index)).then(|| pointers.value(index).to_owned()),
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    references.sort();
    let pending = array::<BooleanArray>(batch, "recorded_native_lifetime_pending")?;
    Ok(Pattern {
        action: text(batch, "action", row)?.ok_or_else(|| invalid("missing original action"))?,
        outcome: text(batch, "outcome", row)?.ok_or_else(|| invalid("missing original outcome"))?,
        diagnostic_kind: text(batch, "diagnostic_kind", row)?,
        code: text(batch, "code", row)?,
        class: text(batch, "class", row)?,
        required_capture_complete: array::<BooleanArray>(batch, "required_capture_complete")?
            .value(row),
        recorded_manager_settlement: text(batch, "recorded_manager_settlement", row)?,
        recorded_native_lifetime_pending: (!pending.is_null(row)).then(|| pending.value(row)),
        invocations: count(batch, "invocations", row)?
            .ok_or_else(|| invalid("missing original invocation population"))?,
        selected_tests_min: count(batch, "selected_tests_min", row)?,
        selected_tests_max: count(batch, "selected_tests_max", row)?,
        members,
        references,
    })
}

#[cfg(test)]
mod tests;
