// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Cooperating command protocol documents. A CPU-only launch is a separate API.
use pse_model::document::Version;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::Duration};

/// Explicit command ownership and preconstruction source request.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceRecordingLaunchRequest {
    /// Request codec.
    pub version: Version<1>,
    /// Explicit source request envelope consumed before scientific construction.
    pub request_file: PathBuf,
    /// Literal executable and arguments; no shell expansion.
    pub argv: Vec<String>,
    /// Explicit working directory, or the invoking directory.
    pub cwd: Option<PathBuf>,
    /// Finite acknowledgement window; does not grant source coverage.
    #[schemars(with = "pse_model::document::ClosedDuration")]
    pub acknowledgement_timeout: Duration,
    /// Finite command clock, independent of recording duration.
    #[schemars(with = "pse_model::document::ClosedDuration")]
    pub command_timeout: Duration,
    /// New output directory for bounded command logs and launch result.
    pub output: PathBuf,
    /// Combined retained stdout/stderr prefix allowance; drains continue after exhaustion.
    pub output_bytes: u64,
}
/// Independent actual command and source-activation outcomes.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceRecordingLaunchDocument {
    /// Document codec.
    pub version: Version<2>,
    /// Exact caller request.
    pub request: SourceRecordingLaunchRequest,
    /// Explicit cooperating owner acknowledgement; absent is not successful selection.
    pub acknowledgement: Option<crate::capture::SourceRecordingAcknowledgement>,
    /// Exact owned command status after join.
    pub command_status: Option<i32>,
    /// Command deadline, not recorder expiry.
    pub timed_out: bool,
    /// Caller cancelled the command, not just its recorder.
    pub cancelled: bool,
    /// Exact owned command group no longer exists after stop/join.
    pub drained: bool,
    /// Retained readable command stdout bytes.
    pub stdout_prefix_bytes: u64,
    /// Retained readable command stderr bytes.
    pub stderr_prefix_bytes: u64,
    /// Logs exceeded their finite prefixes; command pipes were still drained.
    pub logs_truncated: bool,
    /// Source selection is never inferred from ordinary command success.
    pub limitations: Vec<String>,
}
