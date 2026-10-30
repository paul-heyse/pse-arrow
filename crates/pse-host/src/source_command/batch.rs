// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One coarse async bridge into the shared Rayon pool, with indexed per-file results.
use super::{FileOutline, Language, SourceCommandError};
use crate::{
    envelope::{CommandOutcome, Completeness, Envelope, ErrorRecord, RedactionPolicy},
    source_read::RustEdition,
};
use futures_util::FutureExt as _;
use rayon::prelude::*;
use schemars::JsonSchema;
use serde::Serialize;
use std::{ffi::OsString, path::PathBuf};
use tokio_util::sync::CancellationToken;

/// Each input remains represented, including independent read errors and cancellation.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Record {
    /// The same native source model used by the single-file command.
    Read {
        /// Exact immutable parse result.
        reading: FileOutline,
    },
    /// A file failure does not discard successful neighbors.
    Failed {
        /// Exact original native operand.
        file: OsString,
        /// Owned diagnostic from the native source reader.
        failure: ErrorRecord,
    },
    /// Cancellation was observed before this file began reading/parsing.
    Cancelled {
        /// Exact skipped operand, retaining its input position.
        file: OsString,
    },
}
/// Actual completed CPU observation; vector order is the original request order.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Report {
    /// Every requested file, without fail-fast truncation or completion-order sorting.
    pub records: Vec<Record>,
    /// Cancellation observed by the CPU owner before it returned.
    pub cancellation_requested: bool,
    /// True only after all submitted per-file jobs have actually returned.
    pub cpu_work_completed: bool,
}
/// Completion ownership survives cancellation of a borrowed settle future.
///
/// Dropping this owner requests cooperative cancellation but cannot stop a running
/// parse and supplies no completion proof. Keep it and await `settle` after cancel
/// when actual CPU completion is required.
#[must_use = "retain and settle the CPU operation; dropping an awaiter does not stop Rayon work"]
#[derive(Debug)]
pub struct BatchTask {
    completion: Option<tokio_rayon::AsyncRayonHandle<Envelope<Report>>>,
    cancellation: CancellationToken,
    redaction: RedactionPolicy,
}
impl BatchTask {
    /// Request that not-yet-started files be skipped; an in-progress parse finishes normally.
    pub fn cancel(&self) {
        self.cancellation.cancel();
    }
    /// Wait for actual completion without moving the worker handle into this future.
    /// Cancelling this borrow leaves the handle available for another settle call.
    pub async fn settle(&mut self) -> Result<Envelope<Report>, SourceCommandError> {
        let handle = self
            .completion
            .as_mut()
            .ok_or(SourceCommandError::AlreadySettled)?;
        let result = std::panic::AssertUnwindSafe(handle).catch_unwind().await;
        self.completion = None;
        result.map_err(|payload| {
            if let Some(message) = payload
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| payload.downcast_ref::<&'static str>().copied())
            {
                SourceCommandError::WorkerPanic(self.redaction.redact(message))
            } else {
                SourceCommandError::OpaqueWorkerPanic
            }
        })
    }
}
impl Drop for BatchTask {
    fn drop(&mut self) {
        if self.completion.is_some() {
            self.cancel();
        }
    }
}
fn submit<F>(files: Vec<PathBuf>, cancellation: CancellationToken, read: F) -> BatchTask
where
    F: Fn(&std::path::Path) -> Result<FileOutline, SourceCommandError> + Send + Sync + 'static,
{
    let cancellation = cancellation.child_token();
    let worker_token = cancellation.clone();
    let policy = RedactionPolicy::from_environment();
    let redaction = policy.clone();
    let completion = tokio_rayon::spawn(move || {
        // Vec's indexed iterator preserves request order while executing independently.
        // ASTs never cross this boundary: read/parse/extract all stay inside each worker.
        let records = files
            .into_par_iter()
            .map(|file| {
                if worker_token.is_cancelled() {
                    return Record::Cancelled {
                        file: file.into_os_string(),
                    };
                }
                match read(&file) {
                    Ok(reading) => Record::Read { reading },
                    Err(error) => Record::Failed {
                        file: file.into_os_string(),
                        failure: ErrorRecord::from_typed(&error, &policy),
                    },
                }
            })
            .collect::<Vec<_>>();
        let cancelled = worker_token.is_cancelled();
        let mut response = Envelope::response(
            "outline batch",
            format!(
                "{} source file observations; all submitted CPU work returned",
                records.len()
            ),
            Report {
                records,
                cancellation_requested: cancelled,
                cpu_work_completed: true,
            },
        );
        response.observation.scope = "requested_source_files_syntactic_reading".into();
        response.observation.limitations.push("per-file source snapshots are independent; written syntax does not establish active cfg, expansion, resolution or compiler acceptance".into());
        let mut parse_issues = false;
        let mut skipped = false;
        for record in &response.payload.records {
            match record {
                Record::Read { reading } => {
                    parse_issues |= !reading.outline.parse_issues.is_empty()
                }
                Record::Failed { failure, .. } => response.failures.push(failure.clone()),
                Record::Cancelled { .. } => skipped = true,
            }
        }
        if !response.failures.is_empty() {
            response.outcome = CommandOutcome::Failure;
            response.observation.completeness = Completeness::Partial;
        } else if parse_issues {
            response.observation.completeness = Completeness::Partial;
            response.observation.required = false;
        }
        if skipped {
            response.outcome = CommandOutcome::Interrupted;
            response.observation.completeness = Completeness::Partial;
            response.observation.required = true;
        }
        response
    });
    BatchTask {
        completion: Some(completion),
        cancellation,
        redaction,
    }
}
/// Eagerly submit selected files to the existing shared Rayon pool, without a new pool or cap.
pub fn start(
    files: Vec<PathBuf>,
    language: Option<Language>,
    edition: RustEdition,
    cancellation: CancellationToken,
) -> BatchTask {
    submit(files, cancellation, move |file| {
        super::outline(file, language, edition).map(|response| response.payload)
    })
}
/// Human batch reading retains each failure/skipped position as well as successful structures.
pub fn text(report: &Report) -> String {
    let mut text = String::new();
    for record in &report.records {
        match record {
            Record::Read { reading } => text.push_str(&super::outline_text(reading)),
            Record::Failed { file, failure } => {
                text.push_str(&format!("{file:?}: failed: {}\n", failure.message))
            }
            Record::Cancelled { file } => {
                text.push_str(&format!("{file:?}: cancelled before file read\n"))
            }
        }
    }
    text
}
#[cfg(test)]
mod tests;
