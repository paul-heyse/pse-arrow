// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared Unix command, capture and original operation facts for the two native lifetimes.
use super::streams::{self, ChunkObserver, TeeReport};
use crate::{
    HostError, LaunchIntent, admission::Selection, environment::Environment,
    invocation::InvocationDirectory,
};
use command_fds::{CommandFdExt, FdMapping};
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsString,
    os::unix::{fs::OpenOptionsExt, process::ExitStatusExt},
    path::PathBuf,
    process::{ExitStatus, Stdio},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio_util::sync::CancellationToken;
/// Original Unix command. Debug deliberately excludes payload arguments, paths and environment.
pub struct PayloadCommand {
    /// Literal program, resolved by the command owner.
    pub program: PathBuf,
    /// Literal operating-system arguments.
    pub arguments: Vec<OsString>,
    /// Original working directory.
    pub cwd: PathBuf,
    /// Complete selected environment; execution clears inherited service values.
    pub environment: Environment,
}
impl std::fmt::Debug for PayloadCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PayloadCommand")
            .field("argument_count", &self.arguments.len())
            .field("environment_count", &self.environment.len())
            .finish_non_exhaustive()
    }
}

/// Explicit operation observation/settlement policy, independent of host resource capacity.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct PayloadPolicy {
    /// Absolute startup budget is minted once at service initialization.
    pub startup: Duration,
    /// One common deadline for independent resource/capture settlement attempts.
    pub drain: Duration,
    /// Required per-stream chunk write observation policy.
    pub write: Duration,
}

/// Original child/capture facts. Only native performing owners construct these values.
pub struct PayloadReport<O: ChunkObserver> {
    pub(super) native_pending: bool,
    pub(super) native_settled: bool,
    pub(super) child: Result<ExitStatus, HostError>,
    pub(super) capture: Result<TeeReport<tokio::fs::File, tokio::fs::File, O>, HostError>,
    pub(super) operation_error: Option<HostError>,
    pub(super) kill: Result<(), HostError>,
    pub(super) borrowed: bool,
    pub(super) cancelled: bool,
}
impl<O: ChunkObserver> std::fmt::Debug for PayloadReport<O> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PayloadReport")
            .field("child", &self.child)
            .field("capture", &self.capture)
            .field("operation_error", &self.operation_error)
            .field("native_pending", &self.native_pending)
            .finish_non_exhaustive()
    }
}
impl<O: ChunkObserver> PayloadReport<O> {
    pub(crate) fn from_foreground(report: crate::foreground::CapturedForeground<O>) -> Self {
        Self {
            native_pending: false,
            native_settled: report.operation.is_ok(),
            child: report.child.ok_or_else(|| {
                HostError::Ownership("foreground original child status unavailable".into())
            }),
            capture: report.capture,
            operation_error: report.operation.err(),
            kill: Ok(()),
            borrowed: report.borrowed,
            cancelled: report.cancelled,
        }
    }
    /// Original child status after actual wait/reap, never inferred from native service exit.
    pub fn child_exit(&self) -> Option<i32> {
        self.child.as_ref().ok().map(|status| {
            status
                .code()
                .unwrap_or_else(|| 128 + status.signal().unwrap_or(0))
        })
    }
    pub(crate) fn resources_settled(&self) -> bool {
        !self.native_pending
            && self.native_settled
            && self.child.is_ok()
            && self.kill.is_ok()
            && self.operation_error.is_none()
    }
}
pub(super) async fn captured_child<O: ChunkObserver>(
    directory: &InvocationDirectory,
    command: &PayloadCommand,
    trampoline: &std::path::Path,
    observer: O,
    write: Duration,
    startup_deadline: Instant,
) -> Result<
    (
        tokio::process::Child,
        std::io::PipeWriter,
        streams::TeeHandle<tokio::fs::File, tokio::fs::File, O>,
    ),
    HostError,
> {
    let file = |name| {
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
            .open(directory.path().join(name))
            .map(tokio::fs::File::from_std)
    };
    let sinks = (file("stdout.raw")?, file("stderr.raw")?);
    let (gate_read, gate_write) = std::io::pipe()?;
    let mut process = tokio::process::Command::new(trampoline);
    process
        .arg("__foreground-exec")
        .arg(&command.program)
        .args(&command.arguments)
        .env_clear()
        .envs(&command.environment)
        .current_dir(&command.cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    process
        .as_std_mut()
        .fd_mappings(vec![FdMapping {
            parent_fd: gate_read.into(),
            child_fd: 3,
        }])
        .map_err(|error| HostError::Ownership(error.to_string()))?;
    let mut child = process.spawn()?;
    drop(process);
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let (Some(stdout), Some(stderr)) = (stdout, stderr) else {
        let operation = HostError::Ownership("managed child lacks required piped output".into());
        let _ = child.start_kill();
        let cleanup = tokio::time::timeout_at(startup_deadline.into(), child.wait()).await;
        return Err(match cleanup {
            Ok(Ok(_)) => operation,
            other => HostError::Settlement {
                operation: Box::new(operation),
                cleanup: Box::new(HostError::Ownership(format!(
                    "unavailable-pipe child reap: {other:?}"
                ))),
            },
        });
    };
    let capture = streams::start(stdout, stderr, sinks, observer, None, write);
    Ok((child, gate_write, capture))
}

/// Inputs to the same-service detached child owner.
pub(crate) struct PayloadRequest<O: ChunkObserver> {
    pub(crate) directory: Arc<InvocationDirectory>,
    pub(crate) controller: LaunchIntent,
    pub(crate) root: PathBuf,
    pub(crate) command: PayloadCommand,
    pub(crate) selection: Selection,
    pub(crate) trampoline: PathBuf,
    pub(crate) policy: PayloadPolicy,
    pub(crate) cancellation: CancellationToken,
    pub(crate) observer: O,
}
