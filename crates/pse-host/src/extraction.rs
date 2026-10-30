// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Typed diagnostic extraction. Stream interpretation does not establish scientific acceptance.

use crate::envelope::RedactionPolicy;
use cargo_metadata::{Message, diagnostic::DiagnosticLevel};
use clap::ValueEnum;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::HashSet, num::NonZeroUsize};

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
    #[error("invalid compiler protocol record: {0}")]
    Json(#[from] serde_json::Error),
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

#[cfg(test)]
mod tests;
