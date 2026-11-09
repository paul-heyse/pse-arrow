// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive views of original invocation evidence, without admission or stop authority.
use crate::{
    HostError,
    artifacts::Content,
    envelope::{
        ArtifactReference, CliInvocationId, CommandOutcome, Completeness, Envelope, ErrorRecord,
        OutputFormat, Presentation, ProviderContext, RedactionPolicy, ScientificReference,
        Settlement, SubjectState,
    },
    extraction::OutputStream,
    generation::PreparedGeneration,
    investigation::NativePath,
    invocation::{CompactObservation, InvocationStore},
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use schemars::JsonSchema;
use serde::Serialize;
use sha2::{Digest as _, Sha256};
use std::{
    fs,
    io::{Read, Seek, SeekFrom},
    num::NonZeroU64,
    path::Path,
};

/// One metadata-only row. Operation payloads and historical results are not acquired.
#[derive(Debug, Serialize, JsonSchema)]
pub struct HistoryRow {
    /// Original CLI identity, independent of this observation.
    pub invocation_id: CliInvocationId,
    /// Original creation timestamp; ordering is descending timestamp then identity.
    pub created_unix_ms: u64,
    /// Original owning verb.
    pub command: String,
    /// Explicit session association.
    pub session: Option<String>,
    /// Explicit parent association.
    pub parent: Option<CliInvocationId>,
    /// Recorded control identity, not permission to execute a recorded path.
    pub control: PreparedGeneration,
}
/// Bounded metadata presentation; collection failures remain mandatory envelope facts.
#[derive(Debug, Serialize, JsonSchema)]
pub struct HistoryPage {
    /// Requested session, or all checkout sessions.
    pub session: Option<String>,
    /// Metadata rows in this selected window.
    pub records: Vec<HistoryRow>,
    /// Successfully inspected rows matching this scope, excluding unreadable sources.
    pub known_matches: usize,
    /// Live collection membership is not an atomic snapshot or workload census.
    pub atomic_snapshot: bool,
}
/// Inspect newest metadata without loading stored operation or terminal payloads.
pub fn history(
    store: &InvocationStore,
    session: Option<&str>,
    limit: usize,
    policy: &RedactionPolicy,
) -> Result<Envelope<HistoryPage>, HostError> {
    let history = store.history::<serde::de::IgnoredAny>()?;
    let mut records: Vec<_> = history
        .records
        .into_iter()
        .filter(|record| {
            session.is_none_or(|session| record.selection.session.as_deref() == Some(session))
        })
        .map(|record| HistoryRow {
            invocation_id: record.selection.invocation_id,
            created_unix_ms: record.created_unix_ms,
            command: record.selection.command,
            session: record.selection.session,
            parent: record.selection.parent,
            control: record.selection.control,
        })
        .collect();
    records.reverse();
    let known_matches = records.len();
    records.truncate(limit);
    let mut response = Envelope::response(
        "history",
        format!(
            "{} of {known_matches} known invocation metadata rows",
            records.len()
        ),
        HistoryPage {
            session: session.map(str::to_owned),
            records,
            known_matches,
            atomic_snapshot: false,
        },
    );
    response.observation.scope = "checkout_invocation_metadata_scan".into();
    response.observation.limitations.push(
        "metadata only; terminal results and current workload state were not inspected".into(),
    );
    if known_matches > limit {
        response.view.truncated = true;
        response
            .observation
            .limitations
            .push("newest metadata window selected; increase --limit for older records".into());
    }
    for failure in history.failures {
        response
            .failures
            .push(ErrorRecord::from_typed(&failure, policy));
        response.observation.completeness = Completeness::Partial;
    }
    Ok(response)
}

/// The original full response is nested under the observing invocation's identity.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ResultView {
    /// Authenticated selected original identity.
    pub original_invocation: Option<CliInvocationId>,
    /// Original immutable response, including opaque owner payload and all original failures.
    #[schemars(with = "Option<Envelope<serde_json::Value>>")]
    pub response: Option<Envelope<Box<serde_json::value::RawValue>>>,
}
/// Absence of result evidence is unknown completion, never an inferred running or success state.
pub fn result(
    store: &InvocationStore,
    selector: &str,
    session: Option<&str>,
) -> Result<Envelope<ResultView>, HostError> {
    let selected = store.resolve(selector, session)?;
    let retained = selected
        .as_ref()
        .map(|id| store.result::<serde::de::IgnoredAny, Box<serde_json::value::RawValue>>(id))
        .transpose()?
        .flatten();
    let mut response = Envelope::response(
        "inspect invocation",
        if selected.is_none() {
            "no matching invocation"
        } else if retained.is_none() {
            "no terminal result observed; completion is unknown"
        } else {
            "original terminal evidence observed"
        },
        ResultView {
            original_invocation: selected,
            response: retained,
        },
    );
    response.subject_state = if response.payload.original_invocation.is_none() {
        SubjectState::NoMatch
    } else if response.payload.response.is_some() {
        SubjectState::Available
    } else {
        SubjectState::Unknown
    };
    response.observation.scope = "original_invocation_terminal_evidence".into();
    Ok(response)
}

/// Optional owner detail selected separately from the compact result.
#[derive(Clone, Copy, Debug, clap::ValueEnum, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ResultDetail {
    /// Bounded original diagnostic trees.
    Failures,
    /// Originally captured environment and provider provenance.
    Provenance,
    /// References to owner-managed artifacts and scientific outputs.
    Outputs,
    /// Original workflow dependency ordinals for phase streams and other captured files.
    WorkflowFiles,
    /// Original discovery and execution phases, including elapsed and capture facts.
    WorkflowPhases,
    /// Bounded original campaign progress and case outcomes.
    Conformance,
}

/// Original outcome and settlement, without the owner's opaque payload.
#[derive(Debug, Serialize, serde::Deserialize, JsonSchema)]
pub struct ResultSummary {
    invocation_id: CliInvocationId,
    command: String,
    summary: String,
    outcome: CommandOutcome,
    subject_state: SubjectState,
    child_exit: Option<i32>,
    settlement: Settlement,
    failure_count: usize,
}

impl ResultSummary {
    pub(crate) fn from_terminal<T>(terminal: &Envelope<T>, policy: &RedactionPolicy) -> Self {
        Self {
            invocation_id: terminal.invocation_id.clone(),
            command: terminal.command.clone(),
            summary: policy.redact(&terminal.summary),
            outcome: terminal.outcome,
            subject_state: terminal.subject_state,
            child_exit: terminal.child_exit,
            settlement: terminal.settlement,
            failure_count: terminal.failures.len(),
        }
    }
}

/// Bounded selected result detail. Omitted entries remain distinguishable from absent evidence.
#[derive(Debug, Serialize, JsonSchema)]
pub struct DetailPage<T> {
    records: Vec<T>,
    omitted: usize,
}

impl<T: Serialize> DetailPage<T> {
    fn from_original(entries: Vec<T>) -> Result<Self, HostError> {
        const MAX_RECORDS: usize = 16;
        const MAX_BYTES: usize = 32 * 1024;
        let count = entries.len();
        let mut records = Vec::new();
        let mut remaining = MAX_BYTES;
        for entry in entries.into_iter().take(MAX_RECORDS) {
            let bytes = serde_json::to_vec(&entry).map_err(|error| {
                HostError::Ownership(format!("result detail encoding: {error}"))
            })?;
            // Preserve complete diagnostic relationships within each selected record.
            let Some(rest) = remaining.checked_sub(bytes.len()) else {
                break;
            };
            remaining = rest;
            records.push(entry);
        }
        Ok(Self {
            omitted: count - records.len(),
            records,
        })
    }
}

/// One typed subset of the original evidence.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InspectionDetail {
    /// Terminal diagnostics have their original relationships and producer redaction.
    Failures {
        /// Bounded diagnostic page.
        failures: DetailPage<ErrorRecord>,
    },
    /// Original selection provenance. Provider observations do not establish settings origins.
    Provenance {
        /// Original redacted environment identity, independent of actual values.
        environment_digest: String,
        /// Retained original provider observations.
        providers: Option<DetailPage<ProviderContext>>,
    },
    /// Observed product and artifact links do not grant an active lease.
    Outputs {
        /// Original artifact references.
        artifacts: DetailPage<ArtifactReference>,
        /// Original scientific references.
        scientific: DetailPage<ScientificReference>,
    },
    /// A bounded index of original workflow file commitments.
    WorkflowFiles {
        /// Exact ordinals are preserved even when this index is bounded.
        files: DetailPage<WorkflowFile>,
        /// Original optional Cargo report capture and omission observations.
        cargo_timings: DetailPage<crate::discovery::CargoTimingCapture>,
        /// Start the next read at this original commitment ordinal.
        next_offset: Option<usize>,
    },
    /// Original recorded phases, not reconstructed from raw output.
    WorkflowPhases {
        /// Bounded original timing, capture and process facts.
        phases: DetailPage<crate::test_workflow::execute::Phase>,
        /// Original phase offset for the next page.
        next_offset: Option<usize>,
    },
    /// Last published conformance observation; not a process or settlement verdict.
    Conformance {
        /// Original published case page.
        page: crate::cli::conformance::ConformancePage,
    },
    /// Required original evidence is absent for the selected detail.
    Unavailable {
        /// Detail requested by the caller.
        detail: ResultDetail,
    },
}

/// An ordinal and its original content or omission commitment.
#[derive(Debug, Serialize, JsonSchema)]
pub struct WorkflowFile {
    ordinal: usize,
    commitment: crate::test_workflow::execute::EvidenceFile,
}

/// Compact public inspection, retaining the original identity and verdict separately.
#[derive(Debug, Serialize, JsonSchema)]
pub struct InvocationInspection {
    /// Authenticated original identity, if resolved.
    pub original_invocation: Option<CliInvocationId>,
    /// Terminal summary if observed. Operation payloads are never returned here.
    pub response: Option<ResultSummary>,
    /// The owner released this compact presentation and its minimal witness is still retained.
    pub retired: bool,
    /// The selected detail, if requested.
    pub detail: Option<InspectionDetail>,
    /// Current build admission and phase, independent of the original terminal evidence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) build_activity: Option<crate::build_admission::Activity>,
    /// Last published nonterminal workflow observation, if one remains available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub live_workflow: Option<crate::test_workflow::execute::progress::LiveProgress>,
}

/// Observe only the selected invocation's compact summary or bounded owner detail.
pub fn inspect(
    store: &InvocationStore,
    selector: &str,
    session: Option<&str>,
    detail: Option<ResultDetail>,
    policy: &RedactionPolicy,
) -> Result<Envelope<InvocationInspection>, HostError> {
    inspect_at(store, selector, session, detail, policy, 0)
}

pub(crate) fn inspect_at(
    store: &InvocationStore,
    selector: &str,
    session: Option<&str>,
    detail: Option<ResultDetail>,
    policy: &RedactionPolicy,
    file_offset: usize,
) -> Result<Envelope<InvocationInspection>, HostError> {
    if file_offset != 0 && !matches!(detail, Some(ResultDetail::WorkflowFiles | ResultDetail::WorkflowPhases | ResultDetail::Conformance)) {
        return Err(HostError::Ownership("--file-offset requires workflow-files, workflow-phases, or conformance detail".into()));
    }
    let mut original_invocation = store.resolve(selector, session)?;
    let (compact, _borrower) = match &original_invocation {
        Some(id) => store.observe_compact(id)?,
        None => (CompactObservation::Original, None),
    };
    let retired = matches!(compact, CompactObservation::Retired);
    if matches!(compact, CompactObservation::Expired) {
        original_invocation = None;
    }
    let terminal =
        if !retired && (detail.is_some() || matches!(compact, CompactObservation::Original)) {
            original_invocation
                .as_ref()
                .map(|id| store.result::<serde::de::IgnoredAny, serde::de::IgnoredAny>(id))
                .transpose()?
                .flatten()
        } else {
            None
        };
    let response = match compact {
        CompactObservation::Available(summary) => Some(summary),
        _ => terminal
            .as_ref()
            .map(|terminal| ResultSummary::from_terminal(terminal, policy)),
    };
    let detail = match (detail, terminal, original_invocation.as_ref()) {
        (Some(ResultDetail::Failures), Some(terminal), _) => Some(InspectionDetail::Failures {
            failures: DetailPage::from_original(terminal.failures)?,
        }),
        (Some(ResultDetail::Provenance), terminal, Some(id)) if !retired => {
            let metadata = store.metadata::<serde::de::IgnoredAny>(id)?;
            Some(InspectionDetail::Provenance {
                environment_digest: metadata.selection.environment_digest,
                providers: terminal
                    .map(|terminal| DetailPage::from_original(terminal.providers))
                    .transpose()?,
            })
        }
        (Some(ResultDetail::Outputs), Some(terminal), _) => Some(InspectionDetail::Outputs {
            artifacts: DetailPage::from_original(terminal.artifacts)?,
            scientific: DetailPage::from_original(terminal.scientific)?,
        }),
        (Some(ResultDetail::WorkflowFiles), Some(terminal), Some(id))
            if terminal.artifacts.iter().any(|artifact| artifact.kind == "workflow_report") =>
        {
            #[derive(serde::Deserialize)]
            struct RecordedTimings {
                #[serde(default)]
                cargo_timings: Vec<crate::discovery::CargoTimingCapture>,
            }
            let original = workflow_report(store, id.as_str(), session)?;
            let timings: RecordedTimings = serde_json::from_slice(&original.report)
                .map_err(std::io::Error::other)?;
            let commitments = original.observation.evidence()
                .map_err(|error| HostError::Workflow(Box::new(error)))?;
            if file_offset > commitments.len() {
                return Err(HostError::Ownership("file offset exceeds original commitments".into()));
            }
            let files = DetailPage::from_original(commitments.iter().enumerate().skip(file_offset)
                .map(|(ordinal, commitment)| WorkflowFile { ordinal, commitment: commitment.clone() })
                .collect())?;
            let next_offset = (!files.records.is_empty() && files.omitted != 0)
                .then_some(file_offset + files.records.len());
            Some(InspectionDetail::WorkflowFiles {
                files,
                cargo_timings: DetailPage::from_original(timings.cargo_timings)?,
                next_offset,
            })
        }
        (Some(ResultDetail::WorkflowPhases), Some(terminal), Some(id))
            if terminal.artifacts.iter().any(|artifact| artifact.kind == "workflow_report") =>
        {
            #[derive(serde::Deserialize)]
            struct RecordedPhases {
                phases: Vec<crate::test_workflow::execute::Phase>,
            }
            let original = workflow_report(store, id.as_str(), session)?;
            let recorded: RecordedPhases = serde_json::from_slice(&original.report)
                .map_err(std::io::Error::other)?;
            if file_offset > recorded.phases.len() {
                return Err(HostError::Ownership("phase offset exceeds original report".into()));
            }
            let phases = DetailPage::from_original(recorded.phases.into_iter().skip(file_offset).collect())?;
            let next_offset = (!phases.records.is_empty() && phases.omitted != 0)
                .then_some(file_offset + phases.records.len());
            Some(InspectionDetail::WorkflowPhases { phases, next_offset })
        }
        (Some(ResultDetail::Conformance), terminal, Some(id)) => {
            let directory = store.directory(id)?;
            let page = crate::cli::conformance::read_progress(
                &directory, terminal.as_ref().map(|record| record.artifacts.as_slice()), file_offset,
            )?;
            match page {
                Some(page) => Some(InspectionDetail::Conformance { page }),
                None => Some(InspectionDetail::Unavailable { detail: ResultDetail::Conformance }),
            }
        }
        (Some(detail), _, _) => Some(InspectionDetail::Unavailable { detail }),
        (None, _, _) => None,
    };
    let (summary, state) = match (&original_invocation, &response) {
        (None, _) => ("no matching invocation", SubjectState::NoMatch),
        (Some(_), None) if retired => ("compact result retired", SubjectState::Unavailable),
        (Some(_), None) => (
            "no terminal result observed; completion is unknown",
            SubjectState::Unknown,
        ),
        (Some(_), Some(_)) => (
            "original terminal evidence observed",
            SubjectState::Available,
        ),
    };
    let mut observed = Envelope::response(
        "inspect invocation",
        summary,
        InvocationInspection {
            original_invocation,
            response,
            retired,
            detail,
            build_activity: None,
            live_workflow: None,
        },
    );
    observed.subject_state = state;
    observed.observation.scope = "original_invocation_terminal_evidence".into();
    if observed.payload.response.is_none()
        && let Some(id) = &observed.payload.original_invocation
        && let Ok(directory) = store.directory(id)
        && let Some(action) = directory.workflow_action()
    {
        match crate::test_workflow::execute::progress::read(&directory, action) {
            Ok(progress) => observed.payload.live_workflow = progress,
            Err(error) => {
                observed.observation.completeness = Completeness::Partial;
                observed.observation.limitations.push(policy.redact(&error.to_string()));
            }
        }
    }
    Ok(observed)
}

/// Receipt for one explicitly published inspection projection.
#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct ResultExport {
    original_invocation: Option<CliInvocationId>,
    retired: bool,
    published: Option<PublishedInspection>,
}

#[derive(Debug, Serialize, JsonSchema)]
struct PublishedInspection {
    destination: NativePath,
    bytes: usize,
    sha256: String,
}

pub(crate) fn export(
    store: &InvocationStore,
    selector: &str,
    session: Option<&str>,
    detail: Option<ResultDetail>,
    destination: &Path,
    policy: &RedactionPolicy,
    receipt_id: &CliInvocationId,
) -> Result<Envelope<ResultExport>, HostError> {
    const MAX_EXPORT_BYTES: usize = 128 * 1024;
    let mut inspected = inspect(store, selector, session, detail, policy)?;
    inspected.invocation_id = receipt_id.clone();
    let mut receipt = ResultExport {
        original_invocation: inspected.payload.original_invocation.clone(),
        retired: inspected.payload.retired,
        published: None,
    };
    if inspected.payload.response.is_none() {
        let mut response = Envelope::response("tools export-result", inspected.summary, receipt);
        response.invocation_id = receipt_id.clone();
        response.outcome = CommandOutcome::Failure;
        response.subject_state = inspected.subject_state;
        response.settlement = Settlement::Complete;
        return Ok(response);
    }

    let mut bytes = Vec::new();
    crate::envelope::write(
        &mut bytes,
        inspected,
        OutputFormat::Json,
        &Presentation::default(),
    )?;
    let (Some(parent), Some(name)) = (destination.parent(), destination.file_name()) else {
        return Err(HostError::Ownership(
            "export requires a fresh destination file".into(),
        ));
    };
    let parent = fs::canonicalize(parent)?;
    if parent.starts_with(fs::canonicalize(store.state_root())?) {
        return Err(HostError::Ownership(
            "export destination must be outside the invocation state".into(),
        ));
    }
    let destination = parent.join(name);
    crate::invocation::publish_bytes(&destination, &bytes, Some(MAX_EXPORT_BYTES), 0o600)?;
    receipt.published = Some(PublishedInspection {
        destination: policy.redact_path(destination.into()),
        bytes: bytes.len(),
        sha256: Sha256::digest(&bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
    });
    let mut response = Envelope::response(
        "tools export-result",
        "original inspection exported",
        receipt,
    );
    response.invocation_id = receipt_id.clone();
    response.subject_state = SubjectState::Available;
    response.settlement = Settlement::Complete;
    Ok(response)
}

/// One live raw-stream extent with exact original byte coordinates.
#[derive(Debug, Serialize, JsonSchema)]
pub struct LogView {
    /// Original owner identity.
    pub original_invocation: CliInvocationId,
    /// Exact stream; there is no inferred cross-stream producer ordering.
    pub stream: OutputStream,
    /// Original workflow dependency ordinal; absent for the outer invocation stream.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase_file: Option<usize>,
    /// Original mutable file size at this observation.
    pub observed_bytes: u64,
    /// Actual half-open selected range.
    pub byte_range: std::ops::Range<u64>,
    /// Exact bytes, encoding non-UTF-8 without loss.
    pub content: Content,
    /// Explicit continuation if the selected extent did not cover the observed remainder.
    pub next_offset: Option<u64>,
    /// A live ledger stream is not an immutable object or snapshot.
    pub atomic_snapshot: bool,
}

fn workflow_report(
    store: &InvocationStore,
    selector: &str,
    session: Option<&str>,
) -> Result<crate::test_inventory::CapturedReport, HostError> {
    crate::test_inventory::captured_report_at_limit(
        store, selector, session, Some(crate::developer_evidence::MAX_INPUT_BYTES),
    )
}

/// Select a bounded range of one authenticated file from the original workflow.
pub(crate) fn workflow_member(
    store: &InvocationStore,
    invocation: &str,
    ordinal: usize,
    offset: u64,
    length: NonZeroU64,
) -> Result<crate::artifacts::DescriptorRead, HostError> {
    if length.get() > 256 * 1024 {
        return Err(HostError::Ownership("workflow member reads support at most 262144 bytes per range".into()));
    }
    let report = workflow_report(store, invocation, None)?;
    let commitments = report.observation.evidence()
        .map_err(|error| HostError::Workflow(Box::new(error)))?;
    crate::test_workflow::execute::evidence::file_window(
        &report.parent, commitments, ordinal, None, offset, length, false,
    )
}
/// Select a bounded range in one original stream.
pub(crate) struct LogWindow {
    pub(crate) phase_file: Option<usize>,
    pub(crate) offset: u64,
    pub(crate) length: NonZeroU64,
    pub(crate) tail: bool,
}

/// Read a protected local raw stream directly. Generic object-store paths cannot establish
/// the local ledger's no-follow/ownership contract.
pub(crate) fn log(
    store: &InvocationStore,
    selector: &str,
    session: Option<&str>,
    stream: OutputStream,
    window: LogWindow,
) -> Result<Envelope<LogView>, HostError> {
    let LogWindow { phase_file, offset, length, tail } = window;
    if length.get() > 256 * 1024 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "log length exceeds 262144 bytes; use the returned offset for another window",
        )
        .into());
    }
    if let Some(ordinal) = phase_file {
        use crate::test_workflow::execute::{EvidenceRole, evidence};
        let report = workflow_report(store, selector, session)?;
        let commitments = report.observation.evidence()
            .map_err(|error| HostError::Workflow(Box::new(error)))?;
        let role = match stream {
            OutputStream::Stdout => EvidenceRole::Stdout,
            OutputStream::Stderr => EvidenceRole::Stderr,
        };
        let observed = evidence::phase_window(&report.parent, commitments, ordinal, role, offset, length, tail)?;
        let mut response = Envelope::response("inspect log", "original committed phase stream observed", LogView {
            original_invocation: report.id,
            stream,
            phase_file,
            observed_bytes: observed.identity.bytes,
            byte_range: observed.byte_range,
            content: observed.content,
            next_offset: observed.next_offset,
            atomic_snapshot: observed.atomic_snapshot,
        });
        response.observation.scope = "original_committed_workflow_stream".into();
        return Ok(response);
    }
    let id = store
        .resolve(selector, session)?
        .ok_or_else(|| HostError::Ownership("no matching invocation for log read".into()))?;
    let directory = store.directory(&id)?;
    let mut file = directory.stream(stream)?;
    let observed_bytes = file.metadata()?.len();
    let start = if tail {
        observed_bytes.saturating_sub(length.get())
    } else {
        offset
    };
    if start > observed_bytes {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "log offset exceeds observed bytes",
        )
        .into());
    }
    let end = start.saturating_add(length.get()).min(observed_bytes);
    file.seek(SeekFrom::Start(start))?;
    let mut bytes = Vec::new();
    file.take(end - start).read_to_end(&mut bytes)?;
    let actual_end = start + bytes.len() as u64;
    let content = match String::from_utf8(bytes) {
        Ok(text) => Content::Utf8(text),
        Err(error) => Content::Base64(STANDARD.encode(error.into_bytes())),
    };
    let mut response = Envelope::response(
        "inspect log",
        format!("original {stream:?} bytes {start}..{actual_end} of {observed_bytes} observed"),
        LogView {
            original_invocation: id,
            stream,
            phase_file: None,
            observed_bytes,
            byte_range: start..actual_end,
            content,
            next_offset: (actual_end < observed_bytes).then_some(actual_end),
            atomic_snapshot: false,
        },
    );
    response.observation.scope = "original_raw_stream_extent".into();
    if actual_end != end {
        response.observation.completeness = Completeness::Partial;
        response
            .observation
            .limitations
            .push("raw stream shortened during observation".into());
    }
    Ok(response)
}

#[cfg(test)]
mod tests;
