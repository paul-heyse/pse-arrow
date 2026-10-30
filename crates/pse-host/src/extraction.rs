// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Typed diagnostic extraction. Stream interpretation does not establish scientific acceptance.

use crate::envelope::RedactionPolicy;
use cargo_metadata::{Message, diagnostic::DiagnosticLevel};
use clap::ValueEnum;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashSet},
    num::NonZeroUsize,
};

/// Explicit interpretation selected by the command owner, never guessed from arbitrary logs.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ValueEnum,
)]
#[serde(rename_all = "kebab-case")]
pub enum RunKind {
    /// Retain command output without interpreting it as a test/compiler protocol.
    #[default]
    Generic,
    /// Cargo check messages.
    CargoCheck,
    /// Cargo build messages.
    CargoBuild,
    /// Cargo Clippy messages.
    Clippy,
    /// Nextest execution; actual listed selection and protocol are additional prerequisites.
    Nextest,
}

/// The original pipe, independently of stdout/stderr merged into a retained log.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OutputStream {
    /// Child standard output.
    Stdout,
    /// Child standard error.
    Stderr,
}
/// Attribution supplied by the byte-retention owner before presentation.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct RecordOrigin {
    /// Original pipe.
    pub stream: OutputStream,
    /// Starting byte in the original pipe, independent of log-container framing.
    pub byte_offset: u64,
    /// Original byte extent, including the record delimiter when present.
    pub byte_length: u64,
    /// Zero-based complete-record ordinal in that pipe.
    pub ordinal: u64,
}

/// Failure to interpret a required upstream record. Callers continue draining original pipes.
#[derive(Debug, thiserror::Error)]
pub enum ExtractionError {
    /// Original JSON error and its source chain are retained.
    #[error("invalid extraction protocol record: {0}")]
    Json(#[from] serde_json::Error),
    /// The captured selection or native record cannot support the requested interpretation.
    #[error("nextest evidence: {0}")]
    Nextest(String),
}
pse_diagnostics::impl_diagnostic! {
    ExtractionError,
    code(_this) { Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure) },
    forward(_this) { None }, help(_this) { None }, related(_this) { None }, source(_this) { None }
}

/// Native compiler coordinates retain both byte and one-based character positions.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct CompilerSpan {
    /// Native file/macro name after configured presentation redaction.
    pub file: String,
    /// Inclusive UTF-8 byte offset.
    pub byte_start: u32,
    /// Exclusive UTF-8 byte offset.
    pub byte_end: u32,
    /// One-based source line.
    pub line_start: usize,
    /// One-based source line.
    pub line_end: usize,
    /// One-based character offset, never a byte offset or LSP UTF-16 unit.
    pub character_start: usize,
    /// One-based character offset.
    pub character_end: usize,
    /// Native primary label when supplied.
    pub label: Option<String>,
}

/// Focused native diagnostic. The original whole record remains with the pipe/log owner.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct CompilerDiagnostic {
    /// Exact Cargo package identity.
    pub package_id: String,
    /// Actual compiler target.
    pub target: String,
    /// Native severity spelling.
    pub level: String,
    /// Original diagnostic code, for example E0308 or clippy::needless_borrow.
    pub code: Option<String>,
    /// Native message after configured redaction.
    pub message: String,
    /// All primary spans; multiple primaries are legitimate.
    pub primary: Vec<CompilerSpan>,
    /// Full rendered native context, retained independently of the one-line summary.
    pub rendered: Option<String>,
    /// Exact source attribution into retained evidence.
    pub origin: RecordOrigin,
    /// Presentation grouping only; original duplicate records are not discarded.
    pub duplicate: bool,
}
/// Decoded Cargo records. Unknown/malformed input is never silently downgraded to text.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CargoRecord {
    /// A compiler diagnostic, preserving original code/context and coordinates.
    Diagnostic(CompilerDiagnostic),
    /// Cargo's own terminal build assertion.
    Finished {
        /// Cargo's reported compilation success.
        success: bool,
        /// Original stream record.
        origin: RecordOrigin,
    },
    /// Known artifact/build-script record; original bytes stay in retained evidence.
    Other {
        /// Original stream record.
        origin: RecordOrigin,
    },
}

/// This is compiler evidence only, not selected-test reconciliation or scientific acceptance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CompilerOutcome {
    /// Successful child and one complete successful Cargo terminal record.
    Succeeded,
    /// A known child/compiler failure, even if some observations are incomplete.
    Failed,
    /// Required protocol/terminal evidence was unavailable.
    Incomplete,
}
/// Independent native outcome, coverage and presentation counts.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct CargoSummary {
    /// Original child status; this is not a CLI managed exit class.
    pub child_exit: i32,
    /// Compiler evidence outcome.
    pub outcome: CompilerOutcome,
    /// All required records decoded and exactly one terminal record observed.
    pub complete: bool,
    /// Original error occurrences, including unlocated compiler summaries.
    pub error_records: usize,
    /// Distinct located/coded errors; unknown after the bounded dedup cache fills.
    pub unique_primary_failures: Option<usize>,
    /// Unlocated/uncoded compiler errors, retained as context rather than invented source locations.
    pub unlocated_errors: usize,
    /// Original warning occurrences.
    pub warning_records: usize,
    /// Exact duplicate records detected before the cache limit.
    pub duplicates: usize,
    /// Invalid or unavailable required record occurrences.
    pub incomplete_records: usize,
}

/// State is bounded by an explicit diagnostic-grouping cache, independent of host RAM policy.
/// Every decoded record is returned, including duplicates and records beyond that cache.
#[derive(Debug)]
pub struct CargoExtractor {
    seen: HashSet<[u8; 32]>,
    capacity: NonZeroUsize,
    grouping_complete: bool,
    errors: usize,
    primary_errors: usize,
    unlocated_errors: usize,
    warnings: usize,
    duplicates: usize,
    incomplete: usize,
    terminal: Option<bool>,
    terminal_count: usize,
}
impl CargoExtractor {
    /// Bound only optional presentation grouping. Full source records are retained separately.
    pub fn new(capacity: NonZeroUsize) -> Self {
        Self {
            seen: HashSet::new(),
            capacity,
            grouping_complete: true,
            errors: 0,
            primary_errors: 0,
            unlocated_errors: 0,
            warnings: 0,
            duplicates: 0,
            incomplete: 0,
            terminal: None,
            terminal_count: 0,
        }
    }

    /// Mark oversized, truncated or lost required evidence reported by the framing/log owner.
    pub fn mark_incomplete(&mut self) {
        self.incomplete = self.incomplete.saturating_add(1);
    }

    /// Decode one complete frame. Use the official Cargo message model with strict Serde
    /// decoding: Message::parse_stream intentionally falls back to text on malformed JSON,
    /// which cannot establish completeness of a required compiler-record stream.
    pub fn consume(
        &mut self,
        bytes: &[u8],
        origin: RecordOrigin,
        policy: &RedactionPolicy,
    ) -> Result<CargoRecord, ExtractionError> {
        let message = match serde_json::from_slice::<Message>(bytes) {
            Ok(message) => message,
            Err(error) => {
                self.mark_incomplete();
                return Err(error.into());
            }
        };
        if self.terminal.is_some() {
            self.mark_incomplete();
        }
        match message {
            Message::CompilerMessage(value) => {
                let diagnostic = value.message;
                let is_error = matches!(
                    diagnostic.level,
                    DiagnosticLevel::Ice | DiagnosticLevel::Error
                );
                let has_primary = diagnostic.code.is_some()
                    || diagnostic.spans.iter().any(|span| span.is_primary);
                self.errors = self.errors.saturating_add(usize::from(is_error));
                self.unlocated_errors = self
                    .unlocated_errors
                    .saturating_add(usize::from(is_error && !has_primary));
                self.warnings = self
                    .warnings
                    .saturating_add(usize::from(diagnostic.level == DiagnosticLevel::Warning));
                let key = serde_json::to_vec(&(&value.package_id, &value.target, &diagnostic))?;
                let key: [u8; 32] = Sha256::digest(&key).into();
                let duplicate = self.seen.contains(&key);
                if duplicate {
                    self.duplicates = self.duplicates.saturating_add(1);
                } else if self.seen.len() < self.capacity.get() {
                    self.seen.insert(key);
                } else {
                    self.grouping_complete = false;
                }
                self.primary_errors = self
                    .primary_errors
                    .saturating_add(usize::from(is_error && has_primary && !duplicate));
                let level = match diagnostic.level {
                    DiagnosticLevel::Ice => "error: internal compiler error",
                    DiagnosticLevel::Error => "error",
                    DiagnosticLevel::Warning => "warning",
                    DiagnosticLevel::FailureNote => "failure-note",
                    DiagnosticLevel::Note => "note",
                    DiagnosticLevel::Help => "help",
                    _ => {
                        self.mark_incomplete();
                        "unknown"
                    }
                }
                .to_owned();
                Ok(CargoRecord::Diagnostic(CompilerDiagnostic {
                    package_id: value.package_id.repr,
                    target: value.target.name,
                    level,
                    code: diagnostic.code.map(|code| code.code),
                    message: policy.redact(&diagnostic.message),
                    primary: diagnostic
                        .spans
                        .into_iter()
                        .filter(|span| span.is_primary)
                        .map(|span| CompilerSpan {
                            file: policy.redact(&span.file_name),
                            byte_start: span.byte_start,
                            byte_end: span.byte_end,
                            line_start: span.line_start,
                            line_end: span.line_end,
                            character_start: span.column_start,
                            character_end: span.column_end,
                            label: span.label.map(|label| policy.redact(&label)),
                        })
                        .collect(),
                    rendered: diagnostic.rendered.map(|rendered| policy.redact(&rendered)),
                    origin,
                    duplicate,
                }))
            }
            Message::BuildFinished(finished) => {
                self.terminal_count = self.terminal_count.saturating_add(1);
                self.terminal = Some(finished.success);
                Ok(CargoRecord::Finished {
                    success: finished.success,
                    origin,
                })
            }
            Message::CompilerArtifact(_) | Message::BuildScriptExecuted(_) => {
                Ok(CargoRecord::Other { origin })
            }
            _ => {
                self.mark_incomplete();
                Ok(CargoRecord::Other { origin })
            }
        }
    }

    /// Reconcile only compiler outcome; actual selected units, resource drain and science
    /// remain with their original owners. Failure wins over incomplete observation.
    pub fn finish(self, child_exit: i32) -> CargoSummary {
        let complete = self.incomplete == 0 && self.terminal_count == 1;
        let outcome = if child_exit != 0 || self.errors > 0 || self.terminal == Some(false) {
            CompilerOutcome::Failed
        } else if complete && self.terminal == Some(true) {
            CompilerOutcome::Succeeded
        } else {
            CompilerOutcome::Incomplete
        };
        CargoSummary {
            child_exit,
            outcome,
            complete,
            error_records: self.errors,
            unique_primary_failures: self.grouping_complete.then_some(self.primary_errors),
            unlocated_errors: self.unlocated_errors,
            warning_records: self.warnings,
            duplicates: self.duplicates,
            incomplete_records: self.incomplete,
        }
    }
}

/// Native suite context. This spelling is not a Cargo package ID or nextest binary ID.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NextestSuiteName {
    /// Native package name, independent of its opaque package ID.
    #[serde(rename = "crate")]
    pub package: String,
    /// Native target name, which can collide across target kinds.
    pub test_binary: String,
    /// Target kind, present only on suite records.
    pub kind: String,
}
/// Native execution events describe the final retry result, not every attempt.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NextestTestEvent {
    /// Native test start (also emitted for ignored-policy mismatches).
    Started,
    /// Native final successful result, including native flaky-result policy.
    Ok,
    /// Native final failed result.
    Failed,
    /// Native ignored-policy mismatch, not an executed test.
    Ignored,
}
/// One normalized native event; original JSON bytes remain with the stream owner.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NextestRecord {
    /// Cargo output on the same stdout protocol.
    Cargo {
        /// Original typed Cargo record, with its own discriminator.
        record: CargoRecord,
    },
    /// Native suite start or terminal. Counters are preserved, not summed into a verdict.
    Suite {
        /// Original captured build/feature group.
        group: usize,
        /// Exact opaque Cargo package identity.
        package_id: String,
        /// Exact listed binary IDs compatible with the suite context (exactly one here).
        binary_id: String,
        /// Native name, retaining target kind.
        suite: NextestSuiteName,
        /// Native event spelling.
        event: String,
        /// Native suite-start count.
        test_count: Option<u64>,
        /// Native terminal passed count.
        passed: Option<u64>,
        /// Native terminal failed count.
        failed: Option<u64>,
        /// Native ignored declaration count, not a selected/skipped-test count.
        ignored: Option<u64>,
        /// Native measured count.
        measured: Option<u64>,
        /// Native counter, including the pinned runner's possible wrapping sentinel.
        filtered_out: Option<u64>,
        /// Original stream location.
        origin: RecordOrigin,
    },
    /// Exact captured binary/test identity, never a name-only rerun suggestion.
    Test {
        /// Original resolved selection identity.
        identity: crate::discovery::TestIdentity,
        /// Native full name, including any final retry suffix.
        native_name: String,
        /// Native event after exact identity resolution.
        event: NextestTestEvent,
        /// Whether this test actually matched the captured selection.
        selected: bool,
        /// Original listing's ignored flag.
        declared_ignored: bool,
        /// Native final attempt count; individual failed attempts are not emitted.
        attempts: Option<usize>,
        /// Native combined failure output, after presentation redaction.
        output: Option<String>,
        /// Native timeout/flaky-policy explanation, after presentation redaction.
        reason: Option<String>,
        /// Original stream location.
        origin: RecordOrigin,
    },
}
/// Native runner setup assertion. External fixture readiness and drain remain separate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NextestSetup {
    /// The pinned run's selected tests completed after native configured setup.
    SucceededByRunner,
    /// Native setup-script failure exit105.
    FailedByRunner,
    /// Required native execution evidence was insufficient.
    Unknown,
}
/// Workload reconciliation, independent of physical settlement and scientific acceptance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NextestOutcome {
    /// Nonempty selected population, native setup and complete final evidence agree.
    Succeeded,
    /// A known native test failure, even if other evidence is incomplete.
    TestFailed,
    /// Native setup failed, even if no test records were emitted.
    SetupFailed,
    /// Other nonzero runner status, retaining the exact child status separately.
    RunnerFailed,
    /// The captured selection contains no tests; never a workload pass.
    ZeroMatch,
    /// Required selected-test/protocol/terminal evidence is missing or contradictory.
    Incomplete,
}
/// Native run exit classification using nextest-metadata's documented constants.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NextestExitClass {
    /// Native run exited normally.
    Success,
    /// Native final test failure.
    TestsFailed,
    /// Native configured setup-script failure.
    SetupFailed,
    /// No selected tests were executed.
    NoTestsRun,
    /// A rerun chain still has outstanding tests, never a complete workload pass.
    OutstandingRerun,
    /// Compilation failed before execution.
    BuildFailed,
    /// Cargo metadata acquisition failed.
    MetadataFailed,
    /// Test enumeration failed.
    ListingFailed,
    /// Native output publication failed.
    OutputFailed,
    /// Other native status or signal encoding; exact value remains in child_exit.
    Other,
}
impl NextestExitClass {
    fn classify(exit: i32) -> Self {
        use nextest_metadata::NextestExitCode as Code;
        match exit {
            Code::OK => Self::Success,
            Code::TEST_RUN_FAILED => Self::TestsFailed,
            Code::SETUP_SCRIPT_FAILED => Self::SetupFailed,
            Code::NO_TESTS_RUN => Self::NoTestsRun,
            Code::RERUN_TESTS_OUTSTANDING => Self::OutstandingRerun,
            Code::BUILD_FAILED => Self::BuildFailed,
            Code::CARGO_METADATA_FAILED => Self::MetadataFailed,
            Code::TEST_LIST_CREATION_FAILED => Self::ListingFailed,
            Code::WRITE_OUTPUT_ERROR => Self::OutputFailed,
            _ => Self::Other,
        }
    }
}
/// Owner-constructed reconciliation. Fields are private so decoding cannot construct a pass.
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct NextestSummary {
    child_exit: i32,
    child_class: NextestExitClass,
    outcome: NextestOutcome,
    setup: NextestSetup,
    complete: bool,
    ignored_policy: crate::selection::IgnoredPolicy,
    selected: usize,
    completed: usize,
    failed: Vec<crate::discovery::TestIdentity>,
    incomplete_records: usize,
    cargo: Option<CargoSummary>,
}
impl NextestSummary {
    /// Reconciled workload result, not a child-exit synonym.
    pub fn outcome(&self) -> NextestOutcome {
        self.outcome
    }
    /// Whether every required protocol and selected-test record was observed.
    pub fn complete(&self) -> bool {
        self.complete
    }
    /// Exact original child status.
    pub fn child_exit(&self) -> i32 {
        self.child_exit
    }
    /// Native exit classification, independent of population reconciliation.
    pub fn child_class(&self) -> NextestExitClass {
        self.child_class
    }
    /// Native configured-setup interpretation only.
    pub fn setup(&self) -> NextestSetup {
        self.setup
    }
    /// Nonempty actual selected-test population required for success.
    pub fn selected(&self) -> usize {
        self.selected
    }
    /// Selected tests with a native final result.
    pub fn completed(&self) -> usize {
        self.completed
    }
    /// Exact selected failures for a policy-preserving rerun owner.
    pub fn failed(&self) -> &[crate::discovery::TestIdentity] {
        &self.failed
    }
}
#[derive(Deserialize)]
struct NextestTag {
    #[serde(rename = "type")]
    record_type: Option<String>,
    reason: Option<Box<serde_json::value::RawValue>>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SuiteWire {
    #[serde(rename = "type")]
    _record_type: String,
    event: String,
    nextest: NextestSuiteName,
    test_count: Option<u64>,
    passed: Option<u64>,
    failed: Option<u64>,
    ignored: Option<u64>,
    measured: Option<u64>,
    filtered_out: Option<u64>,
    exec_time: Option<f64>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TestWire {
    #[serde(rename = "type")]
    _record_type: String,
    event: NextestTestEvent,
    name: String,
    exec_time: Option<f64>,
    stdout: Option<String>,
    reason: Option<String>,
}
#[derive(Debug)]
struct SuiteEvidence {
    group: usize,
    package_id: String,
    name: NextestSuiteName,
    selected: bool,
    starts: usize,
    terminals: usize,
    last_terminal: Option<u64>,
    last_test: Option<u64>,
    block_records: usize,
}
#[derive(Debug)]
struct CaseEvidence {
    suite: String,
    selected: bool,
    ignored: bool,
    expect_ignored: bool,
    started: bool,
    terminal: Option<NextestTestEvent>,
}
/// Reconcile one captured ordinary nextest run/group. Stress and rerun-chain interpretation
/// are deliberately unsupported; observed extensions or outstanding native status cannot pass.
#[derive(Debug)]
pub struct NextestExtractor {
    suites: BTreeMap<String, SuiteEvidence>,
    cases: BTreeMap<crate::discovery::TestIdentity, CaseEvidence>,
    aliases: BTreeMap<String, Vec<crate::discovery::TestIdentity>>,
    ignored_policy: crate::selection::IgnoredPolicy,
    incomplete: usize,
    last_ordinal: Option<u64>,
    suite_failed: bool,
    test_failed: bool,
    cargo: CargoExtractor,
    cargo_seen: bool,
}
impl NextestExtractor {
    /// Bind the actual captured group/version/policy before consuming execution records.
    /// Execution still owns verifying/reusing this inventory and launching its ordinary run.
    pub fn new(
        inventory: &crate::discovery::Inventory,
        group: usize,
    ) -> Result<Self, ExtractionError> {
        if inventory
            .nextest_version
            .split_whitespace()
            .take(2)
            .collect::<Vec<_>>()
            != ["cargo-nextest", "0.9.146"]
        {
            return Err(ExtractionError::Nextest(
                "execution protocol is qualified only for captured nextest0.9.146".into(),
            ));
        }
        let listed = inventory
            .groups
            .get(group)
            .ok_or_else(|| ExtractionError::Nextest("captured group is absent".into()))?;
        Self::from_listing(group, &listed.listing, inventory.selection.ignored())
    }
    fn from_listing(
        group: usize,
        listing: &nextest_metadata::TestListSummary,
        ignored_policy: crate::selection::IgnoredPolicy,
    ) -> Result<Self, ExtractionError> {
        use nextest_metadata::{FilterMatch, MismatchReason, RustTestSuiteStatusSummary};
        let mut suites = BTreeMap::new();
        let mut cases = BTreeMap::new();
        let mut aliases: BTreeMap<String, Vec<crate::discovery::TestIdentity>> = BTreeMap::new();
        if listing.test_count
            != listing
                .rust_suites
                .values()
                .map(|suite| suite.test_cases.len())
                .sum::<usize>()
        {
            return Err(ExtractionError::Nextest(
                "listed population count differs from its actual records".into(),
            ));
        }
        for (binary_id, suite) in &listing.rust_suites {
            if *binary_id != suite.binary.binary_id
                || suite.status != RustTestSuiteStatusSummary::LISTED
            {
                return Err(ExtractionError::Nextest(
                    "selected inventory must have complete, identity-consistent listed suites"
                        .into(),
                ));
            }
            let mut selected_suite = false;
            for (name, case) in &suite.test_cases {
                let selected = case.filter_match.is_match();
                if selected
                    && match ignored_policy {
                        crate::selection::IgnoredPolicy::Default => case.ignored,
                        crate::selection::IgnoredPolicy::Only => !case.ignored,
                        crate::selection::IgnoredPolicy::All => false,
                    }
                {
                    return Err(ExtractionError::Nextest(
                        "ignored policy differs from the actual selected population".into(),
                    ));
                }
                selected_suite |= selected;
                let identity = crate::discovery::TestIdentity {
                    group,
                    package_id: suite.binary.package_id.clone(),
                    binary_id: binary_id.to_string(),
                    test_name: name.to_string(),
                };
                aliases
                    .entry(format!(
                        "{}::{}${}",
                        suite.package_name, suite.binary.binary_name, name
                    ))
                    .or_default()
                    .push(identity.clone());
                cases.insert(
                    identity,
                    CaseEvidence {
                        suite: binary_id.to_string(),
                        selected,
                        ignored: case.ignored,
                        expect_ignored: matches!(
                            case.filter_match,
                            FilterMatch::Mismatch {
                                reason: MismatchReason::Ignored
                            }
                        ),
                        started: false,
                        terminal: None,
                    },
                );
            }
            suites.insert(
                binary_id.to_string(),
                SuiteEvidence {
                    group,
                    package_id: suite.binary.package_id.clone(),
                    name: NextestSuiteName {
                        package: suite.package_name.clone(),
                        test_binary: suite.binary.binary_name.clone(),
                        kind: suite.binary.kind.to_string(),
                    },
                    selected: selected_suite,
                    starts: 0,
                    terminals: 0,
                    last_terminal: None,
                    last_test: None,
                    block_records: 0,
                },
            );
        }
        Ok(Self {
            suites,
            cases,
            aliases,
            ignored_policy,
            incomplete: 0,
            last_ordinal: None,
            suite_failed: false,
            test_failed: false,
            cargo: CargoExtractor::new(NonZeroUsize::MIN),
            cargo_seen: false,
        })
    }
    /// Framing loss, retention failure, truncation or an oversized required record prevents pass.
    pub fn mark_incomplete(&mut self) {
        self.incomplete = self.incomplete.saturating_add(1);
    }
    /// Decode one stdout record while the original raw streams continue draining independently.
    pub fn consume(
        &mut self,
        bytes: &[u8],
        origin: RecordOrigin,
        policy: &RedactionPolicy,
    ) -> Result<NextestRecord, ExtractionError> {
        let result = self.consume_record(bytes, origin, policy);
        if result.is_err() {
            self.mark_incomplete();
        }
        result
    }
    fn consume_record(
        &mut self,
        bytes: &[u8],
        origin: RecordOrigin,
        policy: &RedactionPolicy,
    ) -> Result<NextestRecord, ExtractionError> {
        if origin.stream != OutputStream::Stdout
            || self
                .last_ordinal
                .is_some_and(|previous| origin.ordinal <= previous)
        {
            return Err(ExtractionError::Nextest(
                "required records must retain ordered original stdout attribution".into(),
            ));
        }
        self.last_ordinal = Some(origin.ordinal);
        let tag: NextestTag = serde_json::from_slice(bytes)?;
        match tag.record_type.as_deref() {
            None if tag.reason.is_some() => {
                self.cargo_seen = true;
                self.cargo
                    .consume(bytes, origin, policy)
                    .map(|record| NextestRecord::Cargo { record })
            }
            Some("suite") => {
                let record: SuiteWire = serde_json::from_slice(bytes)?;
                let mut matches = self
                    .suites
                    .iter_mut()
                    .filter(|(_, suite)| suite.name == record.nextest);
                let (binary_id, suite) = matches.next().ok_or_else(|| {
                    ExtractionError::Nextest(
                        "suite context is absent from the original listing".into(),
                    )
                })?;
                if matches.next().is_some() {
                    return Err(ExtractionError::Nextest(
                        "native suite context has ambiguous package/binary identity".into(),
                    ));
                }
                match record.event.as_str() {
                    "started"
                        if record.test_count.is_some()
                            && record.passed.is_none()
                            && record.failed.is_none()
                            && record.ignored.is_none()
                            && record.measured.is_none()
                            && record.filtered_out.is_none()
                            && record.exec_time.is_none()
                            && suite.starts == suite.terminals =>
                    {
                        suite.starts += 1;
                        suite.block_records = 0;
                    }
                    "ok" | "failed"
                        if suite.starts == suite.terminals + 1
                            && suite.block_records > 0
                            && record.test_count.is_none()
                            && record.passed.is_some()
                            && record.failed.is_some()
                            && record.ignored.is_some()
                            && record.measured == Some(0)
                            && record.filtered_out.is_some()
                            && record.exec_time.is_some() =>
                    {
                        if (record.event == "failed")
                            != record.failed.is_some_and(|count| count > 0)
                        {
                            return Err(ExtractionError::Nextest(
                                "suite failure assertion contradicts its native failed counter"
                                    .into(),
                            ));
                        }
                        self.suite_failed |= record.event == "failed";
                        suite.terminals += 1;
                        suite.last_terminal = Some(origin.ordinal);
                    }
                    _ => {
                        return Err(ExtractionError::Nextest(
                            "missing, duplicated or unsupported native suite event".into(),
                        ));
                    }
                }
                Ok(NextestRecord::Suite {
                    group: suite.group,
                    package_id: suite.package_id.clone(),
                    binary_id: binary_id.clone(),
                    suite: record.nextest,
                    event: record.event,
                    test_count: record.test_count,
                    passed: record.passed,
                    failed: record.failed,
                    ignored: record.ignored,
                    measured: record.measured,
                    filtered_out: record.filtered_out,
                    origin,
                })
            }
            Some("test") => {
                let record: TestWire = serde_json::from_slice(bytes)?;
                self.test_failed |= record.event == NextestTestEvent::Failed;
                if (matches!(
                    record.event,
                    NextestTestEvent::Started | NextestTestEvent::Ignored
                ) && (record.exec_time.is_some()
                    || record.stdout.is_some()
                    || record.reason.is_some()))
                    || (record.event == NextestTestEvent::Ok
                        && (record.stdout.is_some() || record.reason.is_some()))
                {
                    return Err(ExtractionError::Nextest(
                        "native test event fields contradict its event kind".into(),
                    ));
                }
                let mut candidates: Vec<_> = self
                    .aliases
                    .get(&record.name)
                    .into_iter()
                    .flatten()
                    .map(|identity| (identity.clone(), None))
                    .collect();
                if matches!(
                    record.event,
                    NextestTestEvent::Ok | NextestTestEvent::Failed
                ) && let Some((base, suffix)) = record.name.rsplit_once('#')
                    && let Ok(attempts) = suffix.parse::<usize>()
                    && attempts > 1
                {
                    candidates.extend(
                        self.aliases
                            .get(base)
                            .into_iter()
                            .flatten()
                            .map(|identity| (identity.clone(), Some(attempts))),
                    );
                }
                if candidates.len() != 1 {
                    return Err(ExtractionError::Nextest("native test name is unknown or ambiguous across captured binaries/retry spellings".into()));
                }
                let (identity, attempts) = candidates.remove(0);
                let case = self.cases.get_mut(&identity).ok_or_else(|| {
                    ExtractionError::Nextest("captured test identity is absent".into())
                })?;
                let suite = self.suites.get_mut(&case.suite).ok_or_else(|| {
                    ExtractionError::Nextest("captured suite identity is absent".into())
                })?;
                if suite.starts != suite.terminals + 1 || (!case.selected && !case.expect_ignored) {
                    return Err(ExtractionError::Nextest(
                        "test event is outside its native suite or original selection policy"
                            .into(),
                    ));
                }
                match record.event {
                    NextestTestEvent::Started if !case.started && case.terminal.is_none() => {
                        case.started = true;
                    }
                    NextestTestEvent::Ignored
                        if case.started && case.expect_ignored && case.terminal.is_none() =>
                    {
                        case.terminal = Some(record.event);
                    }
                    NextestTestEvent::Ok | NextestTestEvent::Failed
                        if case.started
                            && case.selected
                            && case.terminal.is_none()
                            && record.exec_time.is_some() =>
                    {
                        case.terminal = Some(record.event);
                    }
                    _ => {
                        return Err(ExtractionError::Nextest(
                            "duplicate, missing-start or policy-incompatible test result".into(),
                        ));
                    }
                }
                suite.last_test = Some(origin.ordinal);
                suite.block_records += 1;
                Ok(NextestRecord::Test {
                    identity,
                    native_name: record.name,
                    event: record.event,
                    selected: case.selected,
                    declared_ignored: case.ignored,
                    attempts,
                    output: record.stdout.map(|value| policy.redact(&value)),
                    reason: record.reason.map(|value| policy.redact(&value)),
                    origin,
                })
            }
            _ => Err(ExtractionError::Nextest(
                "required record is neither native libtest0.1 nor Cargo JSON".into(),
            )),
        }
    }
    /// Reconcile native setup, exact population and terminal evidence. Runner exit alone never
    /// establishes a pass, external readiness, descendant drain or scientific acceptance.
    pub fn finish(self, child_exit: i32) -> NextestSummary {
        use nextest_metadata::NextestExitCode;
        let selected = self.cases.values().filter(|case| case.selected).count();
        let completed = self
            .cases
            .values()
            .filter(|case| case.selected && case.terminal.is_some())
            .count();
        let failed: Vec<_> = self
            .cases
            .iter()
            .filter(|(_, case)| case.selected && case.terminal == Some(NextestTestEvent::Failed))
            .map(|(identity, _)| identity.clone())
            .collect();
        let cargo = self.cargo_seen.then(|| {
            self.cargo
                .finish(if child_exit == NextestExitCode::BUILD_FAILED {
                    child_exit
                } else {
                    0
                })
        });
        let complete = self.incomplete == 0
            && completed == selected
            && self
                .cases
                .values()
                .all(|case| !case.started || case.terminal.is_some())
            && self.suites.values().all(|suite| {
                (!suite.selected && suite.starts == 0)
                    || (suite.terminals > 0
                        && suite.starts == suite.terminals
                        && suite.last_terminal > suite.last_test)
            })
            && cargo.as_ref().is_none_or(|summary| summary.complete);
        let setup = if child_exit == NextestExitCode::SETUP_SCRIPT_FAILED {
            NextestSetup::FailedByRunner
        } else if selected > 0
            && complete
            && matches!(
                child_exit,
                NextestExitCode::OK | NextestExitCode::TEST_RUN_FAILED
            )
        {
            NextestSetup::SucceededByRunner
        } else {
            NextestSetup::Unknown
        };
        let outcome = if child_exit == NextestExitCode::SETUP_SCRIPT_FAILED {
            NextestOutcome::SetupFailed
        } else if self.test_failed
            || self.suite_failed
            || child_exit == NextestExitCode::TEST_RUN_FAILED
        {
            NextestOutcome::TestFailed
        } else if selected == 0
            && matches!(
                child_exit,
                NextestExitCode::OK | NextestExitCode::NO_TESTS_RUN
            )
        {
            NextestOutcome::ZeroMatch
        } else if child_exit != NextestExitCode::OK
            || cargo
                .as_ref()
                .is_some_and(|summary| summary.outcome == CompilerOutcome::Failed)
        {
            NextestOutcome::RunnerFailed
        } else if selected > 0 && complete && setup == NextestSetup::SucceededByRunner {
            NextestOutcome::Succeeded
        } else {
            NextestOutcome::Incomplete
        };
        NextestSummary {
            child_exit,
            child_class: NextestExitClass::classify(child_exit),
            outcome,
            setup,
            complete,
            ignored_policy: self.ignored_policy,
            selected,
            completed,
            failed,
            incomplete_records: self.incomplete,
            cargo,
        }
    }
}

#[cfg(test)]
mod tests;
