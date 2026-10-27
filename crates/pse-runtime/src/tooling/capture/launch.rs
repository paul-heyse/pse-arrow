// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit command launch with sampling enabled before its first scientific instruction.

use super::{
    CaptureDocument, CaptureKind, CaptureLoss, CaptureSession, CaptureStatus, CaptureSymbols,
    CaptureTarget, cpu, document, failure,
};
use crate::observation::ProcessLocator;
use crate::observation_detail::{CaptureBudget, CaptureError};
use pse_model::document::Version;
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    path::PathBuf,
    process::{Command, ExitStatus, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

fn startup_timeout() -> Duration {
    Duration::from_secs(10)
}
/// Structured launch request. A new command is independently owned from its CPU capture.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LaunchCaptureRequest {
    /// Request encoding version.
    pub version: Version<1>,
    /// Executable and arguments passed directly, never interpolated as shell source.
    pub argv: Vec<String>,
    /// Optional command working directory. It does not change the capture output location.
    pub cwd: Option<PathBuf>,
    /// Fresh directory exclusively owned by the optional capture.
    pub output: PathBuf,
    /// Finite sampling interval after acknowledged activation, at most one hour.
    #[serde(default = "super::default_duration")]
    #[schemars(with = "pse_model::document::ClosedDuration")]
    pub duration: Duration,
    /// Finite allowance for establishing acknowledged sampling, at most one minute.
    #[serde(default = "startup_timeout")]
    #[schemars(with = "pse_model::document::ClosedDuration")]
    pub startup_timeout: Duration,
    /// Existing finite capture/output allowances; never limits scientific execution.
    #[serde(default)]
    pub budget: CaptureBudget,
}

/// Command owner. Capture workers do not retain this handle or its cancellation authority.
#[derive(Debug)]
pub struct LaunchedCommand {
    child: pse_diagnostics::SupervisedChild,
    locator: ProcessLocator,
}
impl LaunchedCommand {
    pub(crate) fn take_stdout(&mut self) -> Option<std::process::ChildStdout> {
        self.child.take_stdout()
    }
    pub(crate) fn take_stderr(&mut self) -> Option<std::process::ChildStderr> {
        self.child.take_stderr()
    }
    pub(crate) fn retained_status(&self) -> Option<ExitStatus> {
        self.child.retained_status()
    }
    /// Exact launched process identity, retained independently of optional diagnostics.
    pub fn locator(&self) -> &ProcessLocator {
        &self.locator
    }
    /// Reap an already completed command after independently proving owned group drain.
    pub fn try_wait(&mut self) -> Result<Option<i32>, CaptureError> {
        self.child
            .try_wait()
            .map(|status| status.map(code))
            .map_err(supervision_error)
    }
    /// Wait on the verified leader descriptor using one original interval deadline.
    pub fn wait_ready(&mut self, interval: Duration) -> Result<Option<i32>, CaptureError> {
        if let Some(status) = self.try_wait()? {
            return Ok(Some(status));
        }
        let deadline = Instant::now()
            .checked_add(interval)
            .ok_or_else(|| CaptureError::Invalid("launch wait deadline extent".into()))?;
        match self.child.wait_ready(None, Some(deadline)) {
            Ok(()) => self.try_wait(),
            Err(pse_diagnostics::SupervisionError::Deadline) => Ok(None),
            Err(error) => Err(supervision_error(error)),
        }
    }
    /// Wait for leader completion and exact process-group drain.
    pub fn wait(&mut self) -> Result<i32, CaptureError> {
        self.child.wait().map(code).map_err(supervision_error)
    }
    /// Stop this owned command independently of CPU capture.
    pub fn cancel(&mut self) -> Result<i32, CaptureError> {
        self.child.cancel().map(code).map_err(supervision_error)
    }
    /// End command ownership; a live command is cancelled and reaped.
    pub fn close(&mut self) -> Result<i32, CaptureError> {
        self.cancel()
    }
}
fn supervision_error(error: pse_diagnostics::SupervisionError) -> CaptureError {
    match error {
        pse_diagnostics::SupervisionError::Io(error) => CaptureError::Io(error),
        error => CaptureError::Invalid(error.to_string()),
    }
}
fn code(status: ExitStatus) -> i32 {
    use std::os::unix::process::ExitStatusExt;
    status
        .code()
        .unwrap_or_else(|| 128 + status.signal().unwrap_or(0))
}
/// Establish shared verified group ownership with the existing diagnostic drain bound.
pub(crate) fn spawn_owned_command(command: &mut Command) -> Result<LaunchedCommand, CaptureError> {
    let child = pse_diagnostics::SupervisedChild::spawn(
        command,
        pse_diagnostics::DrainFailurePolicy::Refuse(Duration::from_secs(3)),
        None,
    )
    .map_err(supervision_error)?;
    let identity = child
        .identity()
        .ok_or_else(|| CaptureError::Invalid("verified launch identity absent".into()))?;
    let locator = ProcessLocator {
        pid: identity.pid,
        start_ticks: Some(identity.start_ticks),
        executable: std::fs::read_link(format!("/proc/{}/exe", identity.pid)).ok(),
    };
    Ok(LaunchedCommand { child, locator })
}

/// Launch CPU capture before exec. A refused/failed optional sampler still releases the command.
/// Invalid structured requests refuse before spawning; command and capture ownership stay separate.
pub fn launch_capture(
    request: LaunchCaptureRequest,
) -> Result<(LaunchedCommand, CaptureSession), CaptureError> {
    launch_with(request, |request, locator, stop, state, ready| {
        cpu::run_launched(request, locator, stop, state, ready)
    })
}
fn launch_with(
    mut request: LaunchCaptureRequest,
    collector: impl FnOnce(
        &LaunchCaptureRequest,
        &ProcessLocator,
        &AtomicBool,
        &super::Shared,
        &mpsc::SyncSender<Result<(), String>>,
    ) -> Result<(), CaptureError>
    + Send
    + 'static,
) -> Result<(LaunchedCommand, CaptureSession), CaptureError> {
    if request.argv.is_empty()
        || request.argv[0].is_empty()
        || request.argv.iter().any(|argument| argument.contains('\0'))
        || request.argv.len() > 4096
        || request
            .argv
            .iter()
            .try_fold(0_usize, |total, arg| total.checked_add(arg.len()))
            .is_none_or(|size| size > 1 << 20)
    {
        return Err(CaptureError::Invalid(
            "launch requires a bounded nonempty executable/argument vector without NUL".into(),
        ));
    }
    if request.duration.is_zero()
        || request.duration > Duration::from_secs(3600)
        || request.startup_timeout.is_zero()
        || request.startup_timeout > Duration::from_secs(60)
        || request.budget.memory_bytes < 4 << 20
        || request.budget.output_bytes < 16384
    {
        return Err(CaptureError::Invalid("launch capture needs finite positive duration/startup bounds, 4 MiB working and 16 KiB output allowances".into()));
    }
    std::fs::create_dir(&request.output)?;
    request.output = std::fs::canonicalize(&request.output)?;
    let mut command = Command::new("/bin/sh");
    command
        .args([
            "-c",
            "IFS= read -r gate || exit 125; [ \"$gate\" = exec ] || exit 125; exec \"$@\"",
            "pse-capture-launch",
        ])
        .args(&request.argv)
        .stdin(Stdio::piped());
    if let Some(cwd) = &request.cwd {
        command.current_dir(cwd);
    }
    let mut command = spawn_owned_command(&mut command)?;
    let gate = command
        .child
        .take_stdin()
        .ok_or_else(|| CaptureError::Invalid("launch exec gate unavailable".into()))?;
    let locator = command.locator.clone();
    let stop = Arc::new(AtomicBool::new(false));
    let state = Arc::new(Mutex::new(CaptureDocument {
        version: Version,
        kind: CaptureKind::Cpu,
        operation: None,
        launch_process: Some(locator.clone()),
        related_operations: Vec::new(),
        status: CaptureStatus::Running,
        requested_duration: request.duration,
        observed_duration: Duration::ZERO,
        targets: vec![CaptureTarget {
            process: format!(
                "launch:{}:{}",
                locator.pid,
                locator.start_ticks.unwrap_or(0)
            ),
            locator: locator.clone(),
            clock_aligned: false,
            clock_uncertainty_nanoseconds: None,
            thread_ids: Vec::new(),
            concurrent_operations: false,
            threads_partial: true,
        }],
        outputs: Vec::new(),
        loss: CaptureLoss {
            unknown_clock: true,
            source_partial: true,
            sampling_loss_unknown: true,
            ..CaptureLoss::default()
        },
        symbols: CaptureSymbols::default(),
        open_intervals: 0,
        errors: Vec::new(),
    }));
    let (ready, receiver) = mpsc::sync_channel(1);
    let worker_stop = stop.clone();
    let worker_state = state.clone();
    let timeout = request.startup_timeout;
    let worker = std::thread::Builder::new()
        .name("pse-capture-launch".into())
        .spawn(move || {
            if let Err(error) = collector(&request, &locator, &worker_stop, &worker_state, &ready) {
                failure(&worker_state, error);
            }
            let mut state = document(&worker_state);
            if state.status == CaptureStatus::Running {
                state.status = if !state.errors.is_empty() {
                    CaptureStatus::Failed
                } else if worker_stop.load(Ordering::Acquire) {
                    CaptureStatus::Stopped
                } else {
                    CaptureStatus::Partial
                };
            }
        });
    let worker = match worker {
        Ok(worker) => Some(worker),
        Err(error) => {
            failure(&state, error);
            document(&state).status = CaptureStatus::Failed;
            None
        }
    };
    if worker.is_some() && !matches!(receiver.recv_timeout(timeout), Ok(Ok(()))) {
        failure(
            &state,
            "sampling unavailable before launch; command released without CPU coverage",
        );
        document(&state).status = CaptureStatus::Failed;
        stop.store(true, Ordering::Release);
    }
    // A sampler failure is never permission to suppress, cancel or stall scientific execution.
    let mut gate = gate;
    if let Err(error) = gate.write_all(b"exec\n") {
        failure(&state, error);
    }
    drop(gate);
    Ok((
        command,
        CaptureSession {
            stop,
            state,
            layer: None,
            worker,
        },
    ))
}

#[cfg(test)]
mod tests;
