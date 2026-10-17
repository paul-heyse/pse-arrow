// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit command launch with sampling enabled before its first scientific instruction.

use super::{
    CaptureBudget, CaptureDocument, CaptureError, CaptureKind, CaptureLoss, CaptureSession,
    CaptureStatus, CaptureSymbols, CaptureTarget, cpu, document, failure,
};
use crate::observation::ProcessLocator;
use pse_model::document::Version;
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    path::PathBuf,
    process::{Child, Command, ExitStatus, Stdio},
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
    child: Child,
    locator: ProcessLocator,
    finished: Option<ExitStatus>,
    process_fd: std::os::fd::OwnedFd,
}
impl LaunchedCommand {
    /// Exact launched process identity, retained independently of optional diagnostics.
    pub fn locator(&self) -> &ProcessLocator {
        &self.locator
    }
    /// Reap an already completed command after draining its exact owned process group.
    pub fn try_wait(&mut self) -> Result<Option<i32>, CaptureError> {
        if let Some(status) = self.finished {
            return Ok(Some(code(status)));
        }
        let pid = owned_pid(self.child.id())?;
        let result = rustix::process::waitid(
            rustix::process::WaitId::Pid(pid),
            rustix::process::WaitIdOptions::EXITED
                | rustix::process::WaitIdOptions::NOWAIT
                | rustix::process::WaitIdOptions::NOHANG,
        )
        .map_err(std::io::Error::from)?;
        if result.is_none() {
            return Ok(None);
        }
        self.drain_and_reap().map(Some)
    }
    /// Wait on the kernel process descriptor, without a retry sleep or reaping the leader early.
    pub fn wait_ready(&mut self, interval: Duration) -> Result<Option<i32>, CaptureError> {
        if let Some(status) = self.try_wait()? {
            return Ok(Some(status));
        }
        let timeout = rustix::event::Timespec {
            tv_sec: interval.as_secs() as i64,
            tv_nsec: interval.subsec_nanos() as i64,
        };
        let mut descriptors = [rustix::event::PollFd::new(
            &self.process_fd,
            rustix::event::PollFlags::IN,
        )];
        match rustix::event::poll(&mut descriptors, Some(&timeout)) {
            Ok(_) => self.try_wait(),
            Err(rustix::io::Errno::INTR) => Ok(None),
            Err(error) => Err(std::io::Error::from(error).into()),
        }
    }
    /// Wait for command completion, preserving the leader until owned descendants drain.
    pub fn wait(&mut self) -> Result<i32, CaptureError> {
        if let Some(status) = self.finished {
            return Ok(code(status));
        }
        rustix::process::waitid(
            rustix::process::WaitId::Pid(owned_pid(self.child.id())?),
            rustix::process::WaitIdOptions::EXITED | rustix::process::WaitIdOptions::NOWAIT,
        )
        .map_err(std::io::Error::from)?;
        self.drain_and_reap()
    }
    /// Explicitly cancel only this independently owned command and its process group.
    pub fn cancel(&mut self) -> Result<i32, CaptureError> {
        if let Some(status) = self.finished {
            return Ok(code(status));
        }
        kill_group(self.child.id())?;
        self.wait()
    }
    /// End command ownership; a live command is cancelled and reaped.
    pub fn close(&mut self) -> Result<i32, CaptureError> {
        self.cancel()
    }
    fn drain_and_reap(&mut self) -> Result<i32, CaptureError> {
        // WNOWAIT keeps the leader PID/PGID reserved while signalling and observing the group.
        kill_group(self.child.id())?;
        let started = Instant::now();
        while group_has_live_descendants(self.child.id())? {
            if started.elapsed() >= Duration::from_secs(3) {
                return Err(CaptureError::Invalid(
                    "owned launch process group did not drain; leader remains unreaped".into(),
                ));
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let status = self.child.wait()?;
        self.finished = Some(status);
        Ok(code(status))
    }
}
impl Drop for LaunchedCommand {
    fn drop(&mut self) {
        if self.finished.is_none() {
            let _ = self.cancel();
        }
    }
}
fn owned_pid(pid: u32) -> Result<rustix::process::Pid, CaptureError> {
    i32::try_from(pid)
        .ok()
        .and_then(rustix::process::Pid::from_raw)
        .ok_or_else(|| CaptureError::Invalid("owned launch PID cannot be represented".into()))
}
fn kill_group(pid: u32) -> Result<(), CaptureError> {
    match rustix::process::kill_process_group(owned_pid(pid)?, rustix::process::Signal::KILL) {
        Ok(()) | Err(rustix::io::Errno::SRCH) => Ok(()),
        Err(error) => Err(std::io::Error::from(error).into()),
    }
}
fn group_has_live_descendants(leader: u32) -> Result<bool, CaptureError> {
    for entry in std::fs::read_dir("/proc")? {
        let entry = entry?;
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|text| text.parse::<u32>().ok())
        else {
            continue;
        };
        if pid == leader {
            continue;
        }
        let stat = match std::fs::read_to_string(entry.path().join("stat")) {
            Ok(stat) => stat,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error.into()),
        };
        let Some((_, fields)) = stat.rsplit_once(')') else {
            return Err(CaptureError::Invalid(
                "cannot determine launch process-group drain".into(),
            ));
        };
        let fields: Vec<_> = fields.split_whitespace().take(3).collect();
        if fields.get(2).and_then(|value| value.parse::<u32>().ok()) == Some(leader)
            && fields
                .first()
                .is_none_or(|state| *state != "Z" && *state != "X")
        {
            return Ok(true);
        }
    }
    Ok(false)
}
fn code(status: ExitStatus) -> i32 {
    use std::os::unix::process::ExitStatusExt;
    status
        .code()
        .unwrap_or_else(|| 128 + status.signal().unwrap_or(0))
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
    // This standard spawn attribute is applied by Rust, not a caller-supplied pre_exec hook.
    use std::os::unix::process::CommandExt;
    command.process_group(0);
    if let Some(cwd) = &request.cwd {
        command.current_dir(cwd);
    }
    let mut child = command.spawn()?;
    let gate = child
        .stdin
        .take()
        .ok_or_else(|| CaptureError::Invalid("launch exec gate unavailable".into()))?;
    let locator = ProcessLocator::for_pid(child.id());
    let process_fd = match rustix::process::pidfd_open(
        owned_pid(child.id())?,
        rustix::process::PidfdFlags::empty(),
    ) {
        Ok(fd) => fd,
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(std::io::Error::from(error).into());
        }
    };
    let command = LaunchedCommand {
        child,
        locator: locator.clone(),
        finished: None,
        process_fd,
    };
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
