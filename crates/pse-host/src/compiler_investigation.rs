// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original compiler evidence and explicitly selected native artifact investigations.
pub mod artifact;
pub(crate) mod facts;
use crate::{
    HostError,
    artifacts::{self, Content, DescriptorIdentity},
    correctness::{CompilerUnit, Resolution},
    envelope::{
        ArtifactReference, CliInvocationId, Completeness, Envelope, NextAction, ProviderContext,
        SourceCoordinates, SubjectState,
    },
    extraction::{CompilerSpan, OutputStream, RecordOrigin},
    generation::PreparedGeneration,
    invocation::InvocationStore,
    managed::result::StreamSummary,
    source_read::{SourceIdentity, SourceLocator, SourceRequest},
};
use cargo_metadata::{
    Message,
    diagnostic::{Applicability, Diagnostic, DiagnosticSpan},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use sha2::{Digest as _, Sha256};
use std::{
    ffi::OsString,
    num::{NonZeroU64, NonZeroUsize},
    path::{Component, Path, PathBuf},
};

const BACKEND: &str = "cargo_metadata/0.23.1;compiler-evidence-view/v2";

fn invalid(detail: impl Into<String>) -> HostError {
    HostError::Ownership(format!("compiler evidence: {}", detail.into()))
}
fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[derive(Deserialize)]
struct ReportSource {
    version: u32,
    invocation: CliInvocationId,
    action: crate::test_workflow::execute::Action,
    outcome: serde_json::Value,
    admission: Vec<Resolution>,
    phases: Vec<PhaseSource>,
    observations: Vec<GroupSource>,
}
#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum PhaseKind {
    Metadata,
    Discovery,
    GroupPolicy,
    Check,
    Test,
}
#[derive(Deserialize)]
struct PhaseSource {
    kind: PhaseKind,
    group: Option<usize>,
    stdout: String,
    child_exit: Option<i32>,
    capture_complete: bool,
    stdout_capture: Option<StreamSummary>,
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum GroupSource {
    Check {
        group: usize,
        observation: ObservationSource,
    },
    Test {},
}
#[derive(Deserialize)]
struct ObservationSource {
    failures: Vec<FailureSource>,
    omitted_failure_records: usize,
}
#[derive(Deserialize)]
struct FailureSource {
    package_id: String,
    target: String,
    code: Option<String>,
    primary: Vec<CompilerSpan>,
    message: String,
    origin: RecordOrigin,
    duplicate: bool,
}

/// Address of one original whole Cargo JSON record. This is observation data, never authority to
/// execute an arbitrary recorded path or reconstruct a compiler invocation.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticAddress {
    /// Original retained workflow invocation.
    pub invocation: CliInvocationId,
    /// Authenticated original performing-report bytes.
    pub report_sha256: String,
    /// Original phase index in that report, not a compiler query language.
    pub phase: usize,
    /// Original record's pipe, exact byte range and ordinal.
    pub origin: RecordOrigin,
}
/// Existing original failure reference, extended only with its authenticated artifact address.
#[derive(Debug, Serialize, JsonSchema)]
pub struct DiagnosticReference {
    /// Actual package identity from the original Cargo record.
    pub package_id: String,
    /// Original native target name.
    pub target: String,
    /// Original native diagnostic code.
    pub code: Option<String>,
    /// Original bounded presentation message, preserving capture-time redaction.
    pub message: String,
    /// Original primary locations; secondary/child detail stays in the raw addressed record.
    pub primary: Vec<CompilerSpan>,
    /// Exact original record address for the next observation.
    pub address: DiagnosticAddress,
    /// Original presentation grouping fact; duplicates remain original records.
    pub duplicate: bool,
}
/// Continuation bound to original performing-report bytes, not mutable current source.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticCursor {
    /// Original performing invocation.
    pub invocation: CliInvocationId,
    /// Exact original performing report.
    pub report_sha256: String,
    /// Whole original reference position.
    pub offset: usize,
}
/// Bounded original diagnostic references; this never scans compiler output for a new verdict.
#[derive(Debug, Serialize, JsonSchema)]
pub struct DiagnosticPage {
    /// Original operation, distinct from this observing invocation.
    pub original_invocation: CliInvocationId,
    /// Immutable original report byte identity.
    pub report_sha256: String,
    /// Total original references actually retained by B02.
    pub retained_references: usize,
    /// Native error occurrences omitted by B02's original presentation allowance.
    pub originally_omitted_references: usize,
    /// Whole retained references selected by this observer.
    pub records: Vec<DiagnosticReference>,
    /// Continuation within the original retained reference collection.
    pub next_cursor: Option<DiagnosticCursor>,
}
fn report(
    store: &InvocationStore,
    selector: &str,
    session: Option<&str>,
) -> Result<(CliInvocationId, String, PathBuf, ReportSource), HostError> {
    let (id, bytes, parent) = crate::test_inventory::captured_report(store, selector, session)?;
    let sha = digest(&bytes);
    let report: ReportSource = serde_json::from_slice(&bytes).map_err(std::io::Error::other)?;
    if report.version != 1 || report.invocation != id {
        return Err(invalid(
            "unsupported original report version or invocation association",
        ));
    }
    Ok((id, sha, parent, report))
}
/// Read original check-failure references through the existing authenticated report owner.
/// Missing raw/related detail is a next observation, never an automatic compilation.
pub fn diagnostics(
    store: &InvocationStore,
    selector: &str,
    session: Option<&str>,
    cursor: Option<&DiagnosticCursor>,
    limit: NonZeroUsize,
    root: &Path,
) -> Result<Envelope<DiagnosticPage>, HostError> {
    let (id, sha, _, report) = report(store, selector, session)?;
    let mut records = Vec::new();
    let mut omitted = 0;
    for group in report.observations {
        let GroupSource::Check { group, observation } = group else {
            continue;
        };
        omitted += observation.omitted_failure_records;
        let phases = report
            .phases
            .iter()
            .enumerate()
            .filter(|(_, phase)| phase.kind == PhaseKind::Check && phase.group == Some(group))
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        if phases.len() != 1 {
            return Err(invalid(
                "original compiler group has no unique check-output phase",
            ));
        }
        for failure in observation.failures {
            records.push(DiagnosticReference {
                package_id: failure.package_id,
                target: failure.target,
                code: failure.code,
                message: failure.message,
                primary: failure.primary,
                address: DiagnosticAddress {
                    invocation: id.clone(),
                    report_sha256: sha.clone(),
                    phase: phases[0],
                    origin: failure.origin,
                },
                duplicate: failure.duplicate,
            });
        }
    }
    let offset = match cursor {
        None => 0,
        Some(cursor)
            if cursor.invocation == id
                && cursor.report_sha256 == sha
                && cursor.offset <= records.len() =>
        {
            cursor.offset
        }
        Some(_) => {
            return Err(invalid(
                "diagnostic continuation refers to changed/unsupported original evidence",
            ));
        }
    };
    let total = records.len();
    let selected = records
        .into_iter()
        .skip(offset)
        .take(limit.get())
        .collect::<Vec<_>>();
    let next_offset = offset + selected.len();
    let next_cursor = (next_offset < total).then(|| DiagnosticCursor {
        invocation: id.clone(),
        report_sha256: sha.clone(),
        offset: next_offset,
    });
    let mut response = Envelope::response(
        "inspect diagnostic",
        "original retained compiler-failure references; no compilation started",
        DiagnosticPage {
            original_invocation: id,
            report_sha256: sha,
            retained_references: total,
            originally_omitted_references: omitted,
            records: selected,
            next_cursor,
        },
    );
    response.observation.scope = "original_b02_compiler_diagnostic_reference_page".into();
    response.observation.limitations.push("The page bounds retained references, not the original compiler's examined scope. B02 presentation omissions remain omissions; sole original phase output stays with its existing owner.".into());
    for record in &response.payload.records {
        if let (Some(root), Ok(address)) = (root.to_str(), serde_json::to_string(&record.address)) {
            response.next.push(NextAction::Command {argv:vec!["pse".into(),"-C".into(),root.into(),"inspect".into(),"diagnostic".into(),"--address".into(),address],reason:"read the original diagnostic's related spans, suggestions and source context without rebuilding".into()});
        }
    }
    response.providers.push(ProviderContext {
        provider: "B02/original-performing-report/v1".into(),
        context: Some(response.payload.original_invocation.to_string()),
        analysis_generation: Some(response.payload.report_sha256.clone()),
        availability: SubjectState::Available,
        coordinates: vec![],
    });
    if let Some(cursor) = &response.payload.next_cursor {
        if let Some(root) = root.to_str() {
            response.next.push(NextAction::Command {
                argv: vec![
                    "pse".into(),
                    "-C".into(),
                    root.into(),
                    "inspect".into(),
                    "diagnostic".into(),
                    response.payload.original_invocation.to_string(),
                    "--cursor".into(),
                    serde_json::to_string(cursor).map_err(std::io::Error::other)?,
                    "--budget".into(),
                    limit.to_string(),
                ],
                reason: "read the next retained original compiler-failure reference page".into(),
            });
        }
        response.observation.completeness = Completeness::Partial;
        response.observation.required = false;
    }
    if root.to_str().is_none() {
        response.observation.limitations.push("Typed command argv is unavailable for this non-UTF8 checkout path; native source/report identity remains retained.".into());
    }
    Ok(response)
}

/// Original record/context held once while borrowed detail pages are serialized.
#[derive(Debug)]
pub struct DiagnosticEvidence {
    address: DiagnosticAddress,
    report_context: Resolution,
    original_action: crate::test_workflow::execute::Action,
    original_outcome: serde_json::Value,
    original_host: PreparedGeneration,
    child_exit: Option<i32>,
    capture_complete: bool,
    artifact: Record,
    artifact_path: OsString,
    raw: Box<RawValue>,
    native: cargo_metadata::CompilerMessage,
    context_sha256: String,
}
#[derive(Debug)]
struct Record {
    identity: DescriptorIdentity,
    selected_sha256: String,
}
fn read_record(path: &Path, origin: &RecordOrigin) -> Result<(Record, Box<RawValue>), HostError> {
    if origin.byte_length == 0 {
        return Err(invalid("original compiler record extent is empty"));
    }
    let file = crate::test_inventory::open_private(path)?;
    let end = origin
        .byte_offset
        .checked_add(origin.byte_length)
        .ok_or_else(|| invalid("compiler record byte extent overflows"))?;
    let observed = artifacts::read_descriptor(
        file,
        origin.byte_offset,
        NonZeroU64::new(origin.byte_length),
        None,
    )?;
    if observed.byte_range.end != end {
        return Err(invalid("original raw record is missing or truncated"));
    }
    let Content::Utf8(text) = observed.content else {
        return Err(invalid("original Cargo JSON record is not UTF-8"));
    };
    let raw = serde_json::from_str(&text).map_err(std::io::Error::other)?;
    Ok((
        Record {
            identity: observed.identity,
            selected_sha256: observed.selected_sha256,
        },
        raw,
    ))
}
/// Read exactly one original JSON record from the existing owner's checked descriptor.
/// Decoding uses Cargo's native model; unsupported enums/shapes refuse instead of becoming text.
pub fn diagnostic(
    store: &InvocationStore,
    address: &DiagnosticAddress,
    max_record_bytes: NonZeroUsize,
) -> Result<DiagnosticEvidence, HostError> {
    if address.origin.stream != OutputStream::Stdout
        || address.origin.byte_length == 0
        || address.origin.byte_length > max_record_bytes.get() as u64
    {
        return Err(invalid(
            "selected compiler record has an unsupported pipe or exceeds the explicit whole-record decoding allowance",
        ));
    }
    let (id, sha, parent, report) = report(store, address.invocation.as_str(), None)?;
    if sha != address.report_sha256 || id != address.invocation {
        return Err(invalid(
            "original performing report changed since selecting the diagnostic",
        ));
    }
    let phase = report
        .phases
        .get(address.phase)
        .ok_or_else(|| invalid("original phase address is outside the report"))?;
    if phase.kind != PhaseKind::Check {
        return Err(invalid(
            "selected phase is not original retained check output",
        ));
    }
    let group = phase
        .group
        .ok_or_else(|| invalid("original compiler phase has no admitted group"))?;
    if report.phases.iter().filter(|phase|phase.kind==PhaseKind::Check && phase.group==Some(group)).count()!=1 || report.observations.iter().filter(|observation|matches!(observation,GroupSource::Check {group:observed,..} if *observed==group)).count()!=1 {return Err(invalid("original retained compiler group/phase association is ambiguous"));}
    let reference = report
        .observations
        .iter()
        .filter_map(|observation| match observation {
            GroupSource::Check {
                group: observed,
                observation,
            } if *observed == group => Some(&observation.failures),
            _ => None,
        })
        .flatten()
        .find(|failure| {
            failure.origin.stream == address.origin.stream
                && failure.origin.byte_offset == address.origin.byte_offset
                && failure.origin.byte_length == address.origin.byte_length
                && failure.origin.ordinal == address.origin.ordinal
        })
        .ok_or_else(|| {
            invalid("address does not identify an original retained failure reference")
        })?;
    let context = report
        .admission
        .get(group)
        .cloned()
        .ok_or_else(|| invalid("original admitted compiler context is absent"))?;
    let relative = Path::new(&phase.stdout);
    if relative.as_os_str().is_empty()
        || !relative
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
    {
        return Err(invalid(
            "original phase output is not a supported relative artifact path",
        ));
    }
    let end = address
        .origin
        .byte_offset
        .checked_add(address.origin.byte_length)
        .ok_or_else(|| invalid("compiler record byte extent overflows"))?;
    if let Some(capture) = &phase.stdout_capture
        && end > capture.bytes_retained
    {
        return Err(invalid(
            "compiler record lies beyond original retained stdout custody",
        ));
    }
    let path = parent.join(relative);
    let (artifact, raw) = read_record(&path, &address.origin)?;
    let message: Message = serde_json::from_str(raw.get()).map_err(std::io::Error::other)?;
    let Message::CompilerMessage(native) = message else {
        return Err(invalid(
            "selected original record is not a compiler diagnostic",
        ));
    };
    if reference.package_id != native.package_id.repr
        || reference.target != native.target.name
        || reference.code.as_deref() != native.message.code.as_ref().map(|code| code.code.as_str())
    {
        return Err(invalid(
            "native record differs from its retained original failure reference",
        ));
    }
    if !context.units.iter().any(|unit| {
        unit.package_id == native.package_id.repr && unit.target_name == native.target.name
    }) {
        return Err(invalid(
            "native diagnostic package/target does not belong to its original admitted compiler context",
        ));
    }
    let metadata: crate::invocation::InvocationMetadata<serde::de::IgnoredAny> =
        store.metadata(&id)?;
    related(&native.message, vec![], &mut |_, _, native_span| {
        if let Some(value) = native_span {
            span(value)?;
        }
        Ok(())
    })?;
    let context_sha256 = digest(
        &serde_json::to_vec(&(
            BACKEND,
            &address.report_sha256,
            &context,
            &metadata.selection.control,
        ))
        .map_err(std::io::Error::other)?,
    );
    Ok(DiagnosticEvidence {
        address: address.clone(),
        report_context: context,
        original_action: report.action,
        original_outcome: report.outcome,
        original_host: metadata.selection.control,
        child_exit: phase.child_exit,
        capture_complete: phase.capture_complete,
        artifact,
        artifact_path: path.into_os_string(),
        raw,
        native,
        context_sha256,
    })
}
/// Cursor binds original record bytes, report, native artifact validator and compiler context.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DetailCursor {
    /// Exact original diagnostic address.
    pub address: DiagnosticAddress,
    /// Exact selected original JSON bytes.
    pub record_sha256: String,
    /// Checked original descriptor metadata and selected-record digest.
    pub artifact: DescriptorIdentity,
    /// Original admitted compiler/prepared-host context.
    pub context_sha256: String,
    /// Whole related-span/note record position.
    pub offset: usize,
}
/// Native applicability retains its original spelling and grants no application authority.
#[derive(Clone, Copy, Debug, Serialize, JsonSchema)]
pub enum SuggestionApplicability {
    /// Native machine-applicable confidence.
    MachineApplicable,
    /// Native placeholder-bearing suggestion.
    HasPlaceholders,
    /// Native potentially incorrect suggestion.
    MaybeIncorrect,
    /// Native unspecified confidence.
    Unspecified,
}
fn applicability(value: &Applicability) -> Result<SuggestionApplicability, HostError> {
    Ok(match value {
        Applicability::MachineApplicable => SuggestionApplicability::MachineApplicable,
        Applicability::HasPlaceholders => SuggestionApplicability::HasPlaceholders,
        Applicability::MaybeIncorrect => SuggestionApplicability::MaybeIncorrect,
        Applicability::Unspecified => SuggestionApplicability::Unspecified,
        _ => {
            return Err(invalid(
                "native suggestion applicability is unsupported by this pinned decoder",
            ));
        }
    })
}
/// Original native source line, without guessed whole-file reconstruction.
#[derive(Debug, Serialize, JsonSchema)]
pub struct SpanLine<'a> {
    /// Exact original line text emitted by rustc.
    pub text: &'a str,
    /// One-based original Unicode scalar column.
    pub highlight_start: usize,
    /// Exclusive original Unicode scalar column.
    pub highlight_end: usize,
}
/// Borrowed original native span, including secondary spans, replacements and macro context.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Span<'a> {
    /// Original compiler filename, possibly virtual/remapped or unavailable locally.
    pub file_name: &'a str,
    /// Inclusive original UTF-8 byte coordinate.
    pub byte_start: u32,
    /// Exclusive original UTF-8 byte coordinate.
    pub byte_end: u32,
    /// Original one-based line extent.
    pub line_start: usize,
    /// Original inclusive final line.
    pub line_end: usize,
    /// Original one-based Unicode scalar column.
    pub column_start: usize,
    /// Original exclusive Unicode scalar column.
    pub column_end: usize,
    /// Original native primary/secondary distinction.
    pub is_primary: bool,
    /// Exact original line snippets, including macro-produced text when emitted.
    pub text: Vec<SpanLine<'a>>,
    /// Native label, including secondary borrow/move context.
    pub label: Option<&'a str>,
    /// Exact native suggested replacement; never applied by this reader.
    pub suggested_replacement: Option<&'a str>,
    /// Original native confidence, separate from source freshness or compilation success.
    pub suggestion_applicability: Option<SuggestionApplicability>,
    /// Original nested macro context, not expanded/resolved by the host.
    pub expansion: Option<Box<Expansion<'a>>>,
}
/// Original compiler expansion provenance.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Expansion<'a> {
    /// Exact native macro invocation span.
    pub span: Span<'a>,
    /// Original compiler macro declaration label.
    pub macro_decl_name: &'a str,
    /// Native definition-site location when supplied.
    pub def_site_span: Option<Span<'a>>,
}
fn span(value: &DiagnosticSpan) -> Result<Span<'_>, HostError> {
    if value.byte_start > value.byte_end
        || value.line_start == 0
        || value.line_start > value.line_end
        || value.column_start == 0
        || value.column_end == 0
        || (value.line_start == value.line_end && value.column_start > value.column_end)
        || value.text.iter().any(|line| {
            line.highlight_start == 0
                || line.highlight_start > line.highlight_end
                || line.highlight_end > line.text.chars().count() + 1
        })
    {
        return Err(invalid(
            "native diagnostic contains invalid source coordinates",
        ));
    }
    Ok(Span {
        file_name: &value.file_name,
        byte_start: value.byte_start,
        byte_end: value.byte_end,
        line_start: value.line_start,
        line_end: value.line_end,
        column_start: value.column_start,
        column_end: value.column_end,
        is_primary: value.is_primary,
        text: value
            .text
            .iter()
            .map(|line| SpanLine {
                text: &line.text,
                highlight_start: line.highlight_start,
                highlight_end: line.highlight_end,
            })
            .collect(),
        label: value.label.as_deref(),
        suggested_replacement: value.suggested_replacement.as_deref(),
        suggestion_applicability: value
            .suggestion_applicability
            .as_ref()
            .map(applicability)
            .transpose()?,
        expansion: value
            .expansion
            .as_ref()
            .map(|expansion| {
                Ok::<_, HostError>(Box::new(Expansion {
                    span: span(&expansion.span)?,
                    macro_decl_name: &expansion.macro_decl_name,
                    def_site_span: expansion.def_site_span.as_ref().map(span).transpose()?,
                }))
            })
            .transpose()?,
    })
}
/// A current-source observation never turns retained snippets into an original full-file hash.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum SourceComparison {
    /// Only the original visible lines agree; other bytes/generation remain unestablished.
    VisibleLinesMatch {
        /// Current source generation, independently acquired now.
        current: SourceIdentity,
        /// Range mapped from the original line/scalar coordinates into current bytes.
        current_locator: Option<SourceLocator>,
        /// Whether the original raw byte extent still equals those current coordinates.
        original_byte_coordinates_match: bool,
    },
    /// Current workspace-root candidate differs from retained native lines.
    VisibleLinesDiffer {
        /// Current source identity, never attributed to the old compilation.
        current: SourceIdentity,
        /// Explicit current line observations at the original line numbers.
        current_lines: Vec<String>,
    },
    /// The compiler supplied no original lines against which to compare.
    OriginalLinesUnavailable {
        /// Current independently acquired source when readable.
        current: Option<SourceIdentity>,
    },
    /// Virtual/remapped/missing sources retain their original diagnostic instead of disappearing.
    Unavailable {
        /// Original failed source observation.
        reason: String,
    },
}
fn compare_source(root: &Path, value: &DiagnosticSpan) -> SourceComparison {
    let name = Path::new(&value.file_name);
    let path = if name.is_absolute() {
        name.to_owned()
    } else {
        root.join(name)
    };
    let current = match (SourceRequest::Working { path }).acquire() {
        Ok(source) => source,
        Err(error) => {
            return SourceComparison::Unavailable {
                reason: error.to_string(),
            };
        }
    };
    if value.text.is_empty() {
        return SourceComparison::OriginalLinesUnavailable {
            current: Some(current.identity().clone()),
        };
    }
    let normalized_bom = current
        .text()
        .strip_prefix('\u{feff}')
        .unwrap_or(current.text());
    let bom_bytes = current.text().len() - normalized_bom.len();
    let observed = normalized_bom
        .split_inclusive('\n')
        .skip(value.line_start.saturating_sub(1))
        .take(value.text.len())
        .map(|line| {
            line.strip_suffix('\n')
                .map(|line| line.strip_suffix('\r').unwrap_or(line))
                .unwrap_or(line)
                .to_owned()
        })
        .collect::<Vec<_>>();
    let original = value
        .text
        .iter()
        .map(|line| line.text.as_str())
        .collect::<Vec<_>>();
    if observed.iter().map(String::as_str).collect::<Vec<_>>() != original {
        return SourceComparison::VisibleLinesDiffer {
            current: current.identity().clone(),
            current_lines: observed,
        };
    }
    let range =
        crate::source_read::byte_offset(normalized_bom, value.line_start, value.column_start)
            .and_then(|start| {
                crate::source_read::byte_offset(normalized_bom, value.line_end, value.column_end)
                    .map(|end| start + bom_bytes..end + bom_bytes)
            })
            .ok();
    let original_byte_coordinates_match = range.as_ref().is_some_and(|range| {
        range.start == value.byte_start as usize && range.end == value.byte_end as usize
    });
    let current_locator =
        range
            .filter(|range| range.start <= range.end)
            .map(|range| SourceLocator {
                source: current.identity().clone(),
                range: crate::source_read::location(
                    &crate::source_read::line_starts(current.text()),
                    range,
                ),
            });
    SourceComparison::VisibleLinesMatch {
        current: current.identity().clone(),
        current_locator,
        original_byte_coordinates_match,
    }
}
// This explicitly observes current code. Matching retained lines cannot establish the original
// compilation's whole-source generation, so neither the old hash nor an implicit preparation is
// supplied. Reacquisition also prevents converting a concurrently changed candidate into argv.
fn fresh_semantic_action(root: &Path, comparison: &SourceComparison) -> Option<NextAction> {
    let SourceComparison::VisibleLinesMatch {
        current_locator: Some(locator),
        ..
    } = comparison
    else {
        return None;
    };
    let source = (SourceRequest::Working {
        path: locator.source.path().to_owned(),
    })
    .acquire()
    .ok()?;
    locator.select(&source).ok()?;
    let prefix = source.text().get(..locator.range.byte_start)?;
    let column = prefix.rsplit('\n').next()?.chars().count() + 1;
    Some(NextAction::Command {
        argv: vec![
            "pse".into(), "-C".into(), root.to_str()?.into(), "inspect".into(),
            "entity".into(), source.identity().path().to_str()?.into(),
            "--line".into(), locator.range.line_start.to_string(),
            "--column".into(), column.to_string(), "--if-source".into(),
            source.identity().source_sha256().into(),
        ],
        reason: "request a separate semantic observation of this guarded current-source candidate; requires explicit session preparation and does not establish the original compilation's source generation".into(),
    })
}
/// Selected admitted compiler context; unrelated workspace units stay in the original report.
#[derive(Debug, Serialize, JsonSchema)]
pub struct AdmittedContext<'a> {
    /// Original admitted Cargo arguments, never inferred actual rustc argv.
    pub arguments: &'a [OsString],
    /// Original selected root packages.
    pub packages: &'a [String],
    /// Original native capabilities required by admission.
    pub native_capabilities: &'a std::collections::BTreeSet<String>,
    /// Matching native package/target contexts, including distinct host/target instances.
    pub matching_units: Vec<&'a CompilerUnit>,
    /// Original admitted group unit count.
    pub total_units: usize,
}
/// One whole related native span/note. Its source and applicability are indivisible detail.
#[derive(Debug, Serialize, JsonSchema)]
pub struct DetailRow<'a> {
    /// Child-index path within the original diagnostic; empty addresses its top message.
    pub diagnostic_path: Vec<usize>,
    /// Original message attached to this span/note.
    pub message: &'a str,
    /// Original native diagnostic level, serialized with its native names.
    pub level: String,
    /// Original primary/secondary/suggestion/macro span, absent for a spanless note.
    pub span: Option<Span<'a>>,
    /// Optional fresh source comparison, explicitly distinct from original compiler evidence.
    pub source: Option<SourceComparison>,
}
/// Original diagnostic with only its selected related-span/note page materialized.
#[derive(Debug, Serialize, JsonSchema)]
pub struct DetailPage<'a> {
    /// Exact original record/report association.
    pub address: &'a DiagnosticAddress,
    /// Original exact native JSON record digest.
    pub record_sha256: &'a str,
    /// Original compiler code when supplied.
    pub code: Option<&'a str>,
    /// Original diagnostic message.
    pub message: &'a str,
    /// Original rendered context; no compiler rerun is needed to obtain it.
    pub rendered: Option<&'a str>,
    /// Exact original admitted units/features/host-target contexts, not current Cargo inference.
    pub admitted: AdmittedContext<'a>,
    /// Original performing operation, independent of this observer.
    pub original_action: crate::test_workflow::execute::Action,
    /// Original workflow projection, not reconstructed into a native verdict capability.
    pub original_outcome: &'a serde_json::Value,
    /// Actual original compiler phase exit status, when waited.
    pub original_child_exit: Option<i32>,
    /// Original raw-capture completeness, separate from compiler success/physical settlement.
    pub original_capture_complete: bool,
    /// Actual original prepared host control identity; not an inferred rustc version.
    pub prepared_host: &'a PreparedGeneration,
    /// Original full-file identity is absent from B02; native snippets remain original evidence.
    pub original_source_sha256: Option<&'a str>,
    /// No compiler version is inferred from the observer's current toolchain.
    pub original_rustc_version: Option<&'a str>,
    /// All related native span/note records, including page omissions.
    pub total_related_records: usize,
    /// Whole selected records, preserving source/macro/suggestion detail.
    pub records: Vec<DetailRow<'a>>,
    /// Original-evidence-bound continuation, independent of mutable current source.
    pub next_cursor: Option<DetailCursor>,
    /// Original opaque JSON only when explicitly requested; lexemes/unknown fields stay intact.
    pub raw_record: Option<&'a RawValue>,
}
fn level(level: cargo_metadata::diagnostic::DiagnosticLevel) -> Result<String, HostError> {
    let value = serde_json::to_value(level).map_err(std::io::Error::other)?;
    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| invalid("native diagnostic level did not serialize as a string"))
}
fn related<'a>(
    node: &'a Diagnostic,
    path: Vec<usize>,
    visit: &mut impl FnMut(
        &'a Diagnostic,
        Vec<usize>,
        Option<&'a DiagnosticSpan>,
    ) -> Result<(), HostError>,
) -> Result<(), HostError> {
    if node.spans.is_empty() {
        visit(node, path.clone(), None)?;
    } else {
        for span in &node.spans {
            visit(node, path.clone(), Some(span))?;
        }
    }
    for (index, child) in node.children.iter().enumerate() {
        let mut child_path = path.clone();
        child_path.push(index);
        related(child, child_path, visit)?;
    }
    Ok(())
}
impl DiagnosticEvidence {
    /// Build only a selected related-span page while borrowing the original native diagnostic.
    /// Native snippets survive source edits; a current candidate is never original-source proof.
    pub fn page<'a>(
        &'a self,
        root: &Path,
        cursor: Option<&DetailCursor>,
        limit: NonZeroUsize,
        include_raw: bool,
        inspect_source: bool,
    ) -> Result<Envelope<DetailPage<'a>>, HostError> {
        let offset = match cursor {
            None => 0,
            Some(cursor)
                if serde_json::to_vec(&cursor.address).map_err(std::io::Error::other)?
                    == serde_json::to_vec(&self.address).map_err(std::io::Error::other)?
                    && cursor.record_sha256 == self.artifact.selected_sha256
                    && cursor.artifact == self.artifact.identity
                    && cursor.context_sha256 == self.context_sha256 =>
            {
                cursor.offset
            }
            Some(_) => {
                return Err(invalid(
                    "original diagnostic continuation context/artifact changed",
                ));
            }
        };
        let mut count = 0;
        let mut records = Vec::new();
        related(
            &self.native.message,
            vec![],
            &mut |node, path, native_span| {
                let index = count;
                count += 1;
                if index >= offset && records.len() < limit.get() {
                    records.push(DetailRow {
                        diagnostic_path: path,
                        message: &node.message,
                        level: level(node.level)?,
                        span: native_span.map(span).transpose()?,
                        source: inspect_source
                            .then(|| native_span.map(|span| compare_source(root, span)))
                            .flatten(),
                    });
                }
                Ok(())
            },
        )?;
        if offset > count {
            return Err(invalid(
                "related-record continuation position is outside original evidence",
            ));
        }
        let next = offset + records.len();
        let next_cursor = (next < count).then(|| DetailCursor {
            address: self.address.clone(),
            record_sha256: self.artifact.selected_sha256.clone(),
            artifact: self.artifact.identity.clone(),
            context_sha256: self.context_sha256.clone(),
            offset: next,
        });
        let admitted = AdmittedContext {
            arguments: &self.report_context.arguments,
            packages: &self.report_context.packages,
            native_capabilities: &self.report_context.native_capabilities,
            matching_units: self
                .report_context
                .units
                .iter()
                .filter(|unit| {
                    unit.package_id == self.native.package_id.repr
                        && unit.target_name == self.native.target.name
                })
                .collect(),
            total_units: self.report_context.units.len(),
        };
        let mut response = Envelope::response(
            "inspect diagnostic detail",
            "original native diagnostic detail; no compilation or automatic suggestion application",
            DetailPage {
                address: &self.address,
                record_sha256: &self.artifact.selected_sha256,
                code: self
                    .native
                    .message
                    .code
                    .as_ref()
                    .map(|code| code.code.as_str()),
                message: &self.native.message.message,
                rendered: self.native.message.rendered.as_deref(),
                admitted,
                original_action: self.original_action,
                original_outcome: &self.original_outcome,
                original_child_exit: self.child_exit,
                original_capture_complete: self.capture_complete,
                prepared_host: &self.original_host,
                original_source_sha256: None,
                original_rustc_version: None,
                total_related_records: count,
                records,
                next_cursor,
                raw_record: include_raw.then_some(self.raw.as_ref()),
            },
        );
        response.observation.scope =
            "one_original_rustc_diagnostic_and_selected_related_spans".into();
        response.observation.limitations.push("Original native snippets, spans and suggestion confidence are compiler evidence. B02 retained no full original-source digest or actual rustc version; matching visible lines does not prove an unchanged whole file. Relative source names are inspected as current candidates beneath the selected checkout root; remapped/other compiler cwd remains unestablished.".into());
        response.providers.push(ProviderContext {provider:BACKEND.into(),context:Some(serde_json::to_string(&(&self.address,&response.payload.admitted,&self.original_host,serde_json::json!({"original_full_source_identity":"unknown","original_rustc_version":"unknown","artifact":self.artifact_path}))).map_err(std::io::Error::other)?),analysis_generation:Some(self.context_sha256.clone()),availability:SubjectState::Available,coordinates:self.native.message.spans.iter().map(|span|SourceCoordinates::Utf8Bytes {source:span.file_name.clone(),revision:None,start:span.byte_start as usize,end:span.byte_end as usize}).collect()});
        let mut semantic_actions = std::collections::BTreeSet::new();
        for row in &response.payload.records {
            if let Some(NextAction::Command { argv, reason }) = row
                .source
                .as_ref()
                .and_then(|source| fresh_semantic_action(root, source))
                && semantic_actions.insert(argv.clone())
            {
                response.next.push(NextAction::Command { argv, reason });
            }
        }
        if let Some(root) = root.to_str() {
            let mut argv = vec![
                "pse".into(),
                "-C".into(),
                root.into(),
                "inspect".into(),
                "diagnostic".into(),
                "--address".into(),
                serde_json::to_string(&self.address).map_err(std::io::Error::other)?,
                "--source".into(),
            ];
            response.next.push(NextAction::Command {argv:argv.clone(),reason:"compare original native source snippets with independently acquired current source; never rebuild".into()});
            if let Some(cursor) = &response.payload.next_cursor {
                if !inspect_source {
                    argv.pop();
                }
                argv.extend([
                    "--cursor".into(),
                    serde_json::to_string(cursor).map_err(std::io::Error::other)?,
                    "--budget".into(),
                    limit.to_string(),
                ]);
                argv.extend([
                    "--max-record-bytes".into(),
                    self.address.origin.byte_length.to_string(),
                ]);
                if include_raw {
                    argv.push("--raw".into());
                }
                response.next.push(NextAction::Command {
                    argv,
                    reason:
                        "read the next original related-span page with the same evidence context"
                            .into(),
                });
            }
        } else {
            response.observation.limitations.push("Typed command argv is unavailable for a non-UTF8 checkout path; original byte-safe context remains retained.".into());
        }
        if let Some(path) = Path::new(&self.artifact_path).to_str() {
            response.artifacts.push(ArtifactReference {
                kind: "original-cargo-json-record".into(),
                location: path.into(),
                digest: None,
            });
            response.next.push(NextAction::Command {argv:vec!["pse".into(),"inspect".into(),"artifact".into(),path.into(),"--offset".into(),self.address.origin.byte_offset.to_string(),"--length".into(),self.address.origin.byte_length.to_string()],reason:"read the original retained JSON bytes; artifact extent and original compilation outcome remain separate".into()});
        }
        if next < count {
            response.observation.completeness = Completeness::Partial;
            response.observation.required = false;
        }
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::{fs, os::unix::fs::PermissionsExt as _, process::Command};
    fn allowance(value: usize) -> NonZeroUsize {
        NonZeroUsize::new(value).unwrap()
    }
    #[test]
    fn original_diagnostic_offers_only_guarded_current_semantics_and_preserves_bom_columns() {
        let directory = tempfile::tempdir().unwrap();
        fs::create_dir(directory.path().join("src")).unwrap();
        let file = directory.path().join("src/lib.rs");
        fs::write(&file, "\u{feff}α!\r\n").unwrap();
        let mut native: DiagnosticSpan = serde_json::from_value(native_span()).unwrap();
        native.line_start = 1;
        native.line_end = 1;
        let comparison = compare_source(directory.path(), &native);
        let NextAction::Command { argv, reason } =
            fresh_semantic_action(directory.path(), &comparison).unwrap()
        else {
            panic!("current semantic candidate")
        };
        assert_eq!(&argv[3..5], ["inspect", "entity"]);
        assert_eq!(argv[7], "1");
        assert_eq!(argv[9], "2"); // CLI scalars include the original BOM; compiler lines do not.
        assert_eq!(
            argv[11],
            (SourceRequest::Working { path: file.clone() })
                .acquire()
                .unwrap()
                .identity()
                .source_sha256()
        );
        assert!(reason.contains("current-source candidate"));
        assert!(reason.contains("does not establish"));
        fs::write(&file, "β!\r\n").unwrap();
        assert!(fresh_semantic_action(directory.path(), &comparison).is_none());
        assert!(
            fresh_semantic_action(directory.path(), &compare_source(directory.path(), &native))
                .is_none()
        );
    }
    fn native_span() -> serde_json::Value {
        json!({"file_name":"src/lib.rs","byte_start":4,"byte_end":6,"line_start":2,"line_end":2,"column_start":1,"column_end":2,"is_primary":true,"text":[{"text":"α!","highlight_start":1,"highlight_end":2}],"label":"original borrow","suggested_replacement":"β","suggestion_applicability":"MachineApplicable","expansion":null})
    }
    fn record() -> String {
        let mut secondary = native_span();
        secondary["is_primary"] = json!(false);
        secondary["suggested_replacement"] = json!(null);
        secondary["suggestion_applicability"] = json!(null);
        let mut primary = native_span();
        primary["expansion"] = json!({"span":secondary.clone(),"macro_decl_name":"original!","def_site_span":secondary.clone()});
        let native = json!({"reason":"compiler-message","package_id":"path+file:///fixture#case@0.0.0","target":{"name":"case","kind":["lib"],"crate_types":["lib"],"src_path":"/fixture/src/lib.rs","edition":"2024","doc":true,"doctest":true,"test":true},"message":{"message":"borrowed value moved","code":{"code":"E0382","explanation":null},"level":"error","spans":[primary,secondary],"children":[{"message":"clone before moving","code":null,"level":"help","spans":[native_span()],"children":[],"rendered":null},{"message":"original note","code":null,"level":"note","spans":[],"children":[],"rendered":null}],"rendered":"original full rendered diagnostic"}});
        let mut raw = serde_json::to_string(&native).unwrap();
        raw.pop();
        raw.push_str(",\"future\":1e+03,\"unknown\":1,\"unknown\":2}");
        raw
    }
    fn evidence(raw: &str) -> DiagnosticEvidence {
        let Message::CompilerMessage(native) = serde_json::from_str(raw).unwrap() else {
            panic!("native diagnostic")
        };
        DiagnosticEvidence {
            address: DiagnosticAddress {
                invocation: CliInvocationId::new(),
                report_sha256: "original-report".into(),
                phase: 0,
                origin: RecordOrigin {
                    stream: OutputStream::Stdout,
                    byte_offset: 0,
                    byte_length: raw.len() as u64,
                    ordinal: 0,
                },
            },
            report_context: Resolution {
                arguments: vec!["check".into()],
                packages: vec!["case".into()],
                units: vec![],
                native_capabilities: Default::default(),
            },
            original_action: crate::test_workflow::execute::Action::Check,
            original_outcome: json!("failed"),
            original_host: PreparedGeneration {
                generation: "generation.original".into(),
                protocol: 1,
                manifest_digest: "original-control".into(),
            },
            child_exit: Some(101),
            capture_complete: true,
            artifact: Record {
                identity: DescriptorIdentity {
                    device: 1,
                    inode: 2,
                    bytes: raw.len() as u64,
                    modified: (0, 0),
                    changed: (0, 0),
                },
                selected_sha256: digest(raw.as_bytes()),
            },
            artifact_path: "/original/stdout".into(),
            raw: serde_json::from_str(raw).unwrap(),
            native,
            context_sha256: "original-context".into(),
        }
    }
    #[test]
    fn related_pages_preserve_secondary_children_suggestions_macro_context_and_opaque_json() {
        let raw = record();
        let evidence = evidence(&raw);
        let first = evidence
            .page(Path::new("/fixture"), None, allowance(2), true, false)
            .unwrap();
        assert_eq!(first.exit_code(), 0);
        for action in &first.next {
            if let NextAction::Command { argv, .. } = action {
                use clap::Parser as _;
                crate::cli::Cli::try_parse_from(argv).unwrap_or_else(|error| {
                    panic!(
                        "original diagnostic action must use the current grammar: {argv:?}: {error}"
                    )
                });
            }
        }
        assert_eq!(first.payload.total_related_records, 4);
        assert!(first.payload.records[0].span.as_ref().unwrap().is_primary);
        assert!(!first.payload.records[1].span.as_ref().unwrap().is_primary);
        let primary = first.payload.records[0].span.as_ref().unwrap();
        assert_eq!(primary.suggested_replacement, Some("β"));
        assert_eq!(
            primary.expansion.as_ref().unwrap().macro_decl_name,
            "original!"
        );
        assert_eq!(first.payload.raw_record.unwrap().get(), raw);
        let wire = serde_json::to_string(&first).unwrap();
        assert!(wire.contains("MachineApplicable"));
        assert!(wire.contains("1e+03"));
        assert!(wire.contains("\"unknown\":1,\"unknown\":2"));
        let cursor = first.payload.next_cursor.as_ref().unwrap();
        let second = evidence
            .page(
                Path::new("/fixture"),
                Some(cursor),
                allowance(2),
                false,
                false,
            )
            .unwrap();
        assert_eq!(second.payload.records[0].diagnostic_path, [0]);
        assert_eq!(second.payload.records[1].message, "original note");
        assert!(second.payload.records[1].span.is_none());
        assert!(second.payload.next_cursor.is_none());
        let mut stale = cursor.clone();
        stale.artifact.inode += 1;
        assert!(
            evidence
                .page(
                    Path::new("/fixture"),
                    Some(&stale),
                    allowance(1),
                    false,
                    false
                )
                .is_err()
        );
        stale = cursor.clone();
        stale.context_sha256 = "changed-decoder".into();
        assert!(
            evidence
                .page(
                    Path::new("/fixture"),
                    Some(&stale),
                    allowance(1),
                    false,
                    false
                )
                .is_err()
        );
        stale = cursor.clone();
        stale.offset = 9;
        assert!(
            evidence
                .page(
                    Path::new("/fixture"),
                    Some(&stale),
                    allowance(1),
                    false,
                    false
                )
                .is_err()
        );
        let provider = first.providers.first().unwrap();
        assert!(
            provider
                .context
                .as_ref()
                .unwrap()
                .contains("original_full_source_identity")
        );
        assert!(
            provider
                .context
                .as_ref()
                .unwrap()
                .contains("original-control")
        );
    }
    #[test]
    fn checked_descriptor_read_is_bounded_and_refuses_symlink_truncation_and_writable_input() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("stdout");
        let raw = record();
        fs::write(&path, format!("prefix{raw}\ntail")).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        let origin = RecordOrigin {
            stream: OutputStream::Stdout,
            byte_offset: 6,
            byte_length: raw.len() as u64 + 1,
            ordinal: 3,
        };
        let (identity, selected) = read_record(&path, &origin).unwrap();
        assert_eq!(selected.get(), raw);
        assert_eq!(
            identity.selected_sha256,
            digest(format!("{raw}\n").as_bytes())
        );
        let alias = directory.path().join("alias");
        std::os::unix::fs::symlink(&path, &alias).unwrap();
        assert!(read_record(&alias, &origin).is_err());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o666)).unwrap();
        assert!(read_record(&path, &origin).is_err());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(&path, "prefix{}").unwrap();
        assert!(read_record(&path, &origin).is_err());
    }
    #[test]
    fn current_locator_maps_unicode_crlf_and_bom_without_reusing_original_byte_extent() {
        let directory = tempfile::tempdir().unwrap();
        fs::create_dir(directory.path().join("src")).unwrap();
        let mut native: DiagnosticSpan = serde_json::from_value(native_span()).unwrap();
        fs::write(directory.path().join("src/lib.rs"), "a\r\nα!\r\n").unwrap();
        let SourceComparison::VisibleLinesMatch {
            current_locator: Some(locator),
            original_byte_coordinates_match,
            ..
        } = compare_source(directory.path(), &native)
        else {
            panic!("visible match")
        };
        assert!(!original_byte_coordinates_match);
        assert_eq!((locator.range.byte_start, locator.range.byte_end), (3, 5));
        let source = SourceRequest::Working {
            path: directory.path().join("src/lib.rs"),
        }
        .acquire()
        .unwrap();
        assert_eq!(locator.select(&source).unwrap(), "α");
        fs::write(
            directory.path().join("src/lib.rs"),
            "much longer prefix\r\nα!\r\n",
        )
        .unwrap();
        let SourceComparison::VisibleLinesMatch {
            current_locator: Some(locator),
            original_byte_coordinates_match,
            ..
        } = compare_source(directory.path(), &native)
        else {
            panic!("same snippets after unrelated source edit")
        };
        assert!(!original_byte_coordinates_match);
        assert_eq!(locator.range.byte_start, "much longer prefix\r\n".len());
        fs::write(directory.path().join("src/lib.rs"), "prefix\r\nβ!\r\n").unwrap();
        assert!(matches!(
            compare_source(directory.path(), &native),
            SourceComparison::VisibleLinesDiffer { .. }
        ));
        native.line_start = 1;
        native.line_end = 1;
        native.byte_start = 3;
        native.byte_end = 5;
        fs::write(directory.path().join("src/lib.rs"), "\u{feff}α!\r\n").unwrap();
        let SourceComparison::VisibleLinesMatch {
            current_locator: Some(locator),
            original_byte_coordinates_match,
            ..
        } = compare_source(directory.path(), &native)
        else {
            panic!("BOM coordinate")
        };
        assert!(original_byte_coordinates_match);
        assert_eq!(locator.range.byte_start, 3);
        native.column_start = 0;
        assert!(span(&native).is_err());
        native.column_start = 1;
        native.text[0].highlight_end = 9;
        assert!(span(&native).is_err());
    }
    #[test]
    fn unknown_original_protocol_and_invalid_source_coordinates_refuse() {
        assert!(serde_json::from_value::<PhaseKind>(json!("future_phase")).is_err());
        assert!(serde_json::from_value::<GroupSource>(json!({"kind":"future_kind"})).is_err());
        let mut value = native_span();
        value["suggestion_applicability"] = json!("FutureConfidence");
        assert!(serde_json::from_value::<DiagnosticSpan>(value).is_err());
        let mut value: DiagnosticSpan = serde_json::from_value(native_span()).unwrap();
        value.byte_end = 1;
        assert!(span(&value).is_err());
    }
    #[test]
    fn actual_correctness_failure_retains_native_related_evidence_and_detects_later_source_edit() {
        use crate::{
            envelope::RedactionPolicy,
            managed::streams::ChunkObserver as _,
            selection::{
                self, Features, Filters, IgnoredPolicy, PackageIntent, Scope, TargetIntent,
            },
            test_workflow::{CheckIntent, CompilerObserver},
        };
        let scratch = tempfile::tempdir().unwrap();
        fs::create_dir(scratch.path().join("src")).unwrap();
        fs::write(
            scratch.path().join("Cargo.toml"),
            "[package]\nname=\"compiler-evidence-case\"\nversion=\"0.0.0\"\nedition=\"2024\"\n",
        )
        .unwrap();
        fs::write(scratch.path().join("src/lib.rs"),"pub fn case() {\n let text = String::from(\"value\");\n let moved = text;\n drop(text);\n drop(moved);\n}\n").unwrap();
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap();
        fs::copy(
            root.join("rust-toolchain.toml"),
            scratch.path().join("rust-toolchain.toml"),
        )
        .unwrap();
        let lock = Command::new(crate::correctness::real_cargo().unwrap())
            .current_dir(scratch.path())
            .args(["generate-lockfile", "--offline"])
            .output()
            .unwrap();
        assert!(lock.status.success(), "{lock:?}");
        let metadata = cargo_metadata::MetadataCommand::new()
            .current_dir(scratch.path())
            .no_deps()
            .exec()
            .unwrap();
        let intent = CheckIntent::new(
            metadata,
            selection::Request {
                operation: selection::Operation::Check,
                scope: Scope::Packages(vec![PackageIntent {
                    package: "compiler-evidence-case".into(),
                    target: Some(TargetIntent {
                        name: "compiler_evidence_case".into(),
                        kind: Some("lib".into()),
                    }),
                    features: Features::default(),
                }]),
                platform: None,
                filters: Filters::default(),
                ignored: IgnoredPolicy::Default,
            },
            std::env::vars_os().collect(),
        )
        .unwrap()
        .admit()
        .unwrap();
        let admitted = intent.resolved().admitted()[0].clone();
        assert!(admitted.native_capabilities.is_empty());
        let generation = crate::generation::prepared(root).unwrap().unwrap();
        let program = generation
            .executable(root, crate::generation::HostTool::Correctness)
            .unwrap();
        let output = Command::new(program)
            .current_dir(scratch.path())
            .args(&admitted.arguments)
            .arg("--message-format=json")
            .env_remove("CARGO_HACK_CARGO_SRC")
            .env_remove("PSE_CORRECTNESS_BUILD_CAPTURE")
            .env_remove("PSE_FIXTURE_CARGO_METADATA_CAPTURE")
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(101), "{output:?}");
        let mut observer = CompilerObserver::new(
            allowance(4 * 1024 * 1024),
            allowance(32),
            16,
            RedactionPolicy::default(),
        );
        observer
            .observe(OutputStream::Stdout, 0, &output.stdout)
            .unwrap();
        let (observation, error) = observer.finish(101, true);
        assert!(error.is_none());
        assert_eq!(
            observation.summary.outcome(),
            crate::extraction::CompilerOutcome::Failed
        );
        let failure = observation
            .failures
            .iter()
            .find(|failure| failure.code.as_deref() == Some("E0382"))
            .unwrap();
        let start = failure.origin.byte_offset as usize;
        let end = start + failure.origin.byte_length as usize;
        let raw = std::str::from_utf8(&output.stdout[start..end]).unwrap();
        let mut evidence = evidence(raw.trim_end());
        evidence.address.origin = failure.origin.clone();
        evidence.report_context = admitted;
        evidence.original_host = generation;
        evidence.original_outcome = serde_json::to_value(&observation.summary).unwrap();
        let original = evidence
            .page(scratch.path(), None, allowance(32), true, true)
            .unwrap();
        assert_eq!(original.payload.original_child_exit, Some(101));
        assert!(
            original
                .payload
                .records
                .iter()
                .any(|row| row.span.as_ref().is_some_and(|span| !span.is_primary))
        );
        assert!(original.payload.records.iter().any(|row| {
            row.span.as_ref().is_some_and(|span| {
                span.suggested_replacement.is_some() && span.suggestion_applicability.is_some()
            })
        }));
        assert!(original.payload.original_source_sha256.is_none());
        fs::write(scratch.path().join("src/lib.rs"), "pub fn case() {}\n").unwrap();
        let changed = evidence
            .page(scratch.path(), None, allowance(32), false, true)
            .unwrap();
        assert!(changed.payload.records.iter().any(|row| matches!(
            row.source,
            Some(SourceComparison::VisibleLinesDiffer { .. })
        )));
        assert_eq!(changed.payload.code, Some("E0382"));
        assert!(changed.next.iter().any(|action|matches!(action,NextAction::Command {argv,..} if argv.iter().any(|arg|arg=="--source"))));
    }
}
