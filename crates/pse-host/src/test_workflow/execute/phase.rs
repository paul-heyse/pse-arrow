// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Phase capture reuses the managed stream owner; the enclosing helper scope owns children.
use crate::{
    HostError,
    managed::{
        payload::PayloadPolicy,
        streams::{self, ChunkObserver, TeeReport},
    },
    test_workflow::WorkflowError,
};
use std::{
    path::Path,
    process::{Command, Stdio},
    time::Instant,
};
use tokio::fs::File;

pub(crate) struct Captured<O: ChunkObserver> {
    pub elapsed_seconds: f64,
    pub status: Option<std::process::ExitStatus>,
    pub child_exit: Option<i32>,
    pub capture_complete: bool,
    pub stdout: Option<crate::managed::result::StreamSummary>,
    pub stderr: Option<crate::managed::result::StreamSummary>,
    pub capture: Option<TeeReport<File, File, O>>,
    pub failures: Vec<WorkflowError>,
}
/// This starts no process-group, cgroup, supervisor or resource budget. The fixed helper's
/// authenticated managed physical scope remains responsible for cancellation and drain.
pub(crate) async fn run<O: ChunkObserver>(
    command: Command,
    stdout: &Path,
    stderr: &Path,
    observer: O,
    policy: PayloadPolicy,
) -> Result<Captured<O>, WorkflowError> {
    let started = Instant::now();
    let stdout_sink = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o400)
        .open(stdout)
        .await?;
    let stderr_sink = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o400)
        .open(stderr)
        .await?;
    let mut command = tokio::process::Command::from(command);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn()?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| WorkflowError::Invalid("phase stdout pipe missing".into()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| WorkflowError::Invalid("phase stderr pipe missing".into()))?;
    let tee = streams::start(
        stdout,
        stderr,
        (stdout_sink, stderr_sink),
        observer,
        None,
        policy.write,
    );
    let mut failures = Vec::new();
    let status = match child.wait().await {
        Ok(status) => Some(status),
        Err(source) => {
            failures.push(WorkflowError::ChildWait {
                pid: child.id(),
                source,
            });
            None
        }
    };
    let child_exit = status.map(|status| {
        use std::os::unix::process::ExitStatusExt as _;
        status
            .code()
            .unwrap_or_else(|| 128 + status.signal().unwrap_or(1))
    });
    let now = Instant::now();
    let deadline = match now.checked_add(policy.drain) {
        Some(deadline) => deadline,
        None => {
            failures.push(WorkflowError::Invalid(
                "phase capture drain deadline exceeds clock range".into(),
            ));
            now
        }
    };
    let mut capture = match tee.finish(deadline).await {
        Ok(capture) => Some(capture),
        Err(error) => {
            failures.push(error.into());
            None
        }
    };
    let capture_complete = capture
        .as_ref()
        .is_some_and(|capture| capture.log_complete());
    let stdout = capture
        .as_ref()
        .map(|capture| crate::managed::result::StreamSummary::from(&capture.stdout));
    let stderr = capture
        .as_ref()
        .map(|capture| crate::managed::result::StreamSummary::from(&capture.stderr));
    if let Some(capture) = capture.as_mut() {
        for stream in [&mut capture.stdout, &mut capture.stderr] {
            failures.extend(stream.pipe.read_error.take().map(WorkflowError::from));
            failures.extend(
                stream
                    .pipe
                    .task_error
                    .take()
                    .map(|error| WorkflowError::Host(HostError::Task(error))),
            );
            failures.extend(stream.write_error.take().map(WorkflowError::from));
            failures.extend(stream.flush_error.take().map(WorkflowError::from));
            failures.extend(stream.synchronization_error.take().map(WorkflowError::from));
            if stream.pipe.observation_expired || stream.io_unresolved {
                failures.push(WorkflowError::Invalid(
                    "required phase raw I/O observation is incomplete".into(),
                ));
            }
        }
    }
    Ok(Captured {
        elapsed_seconds: started.elapsed().as_secs_f64(),
        status,
        child_exit,
        capture_complete,
        stdout,
        stderr,
        capture,
        failures,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{os::unix::process::ExitStatusExt as _, time::Duration};
    #[tokio::test]
    async fn phase_signal_exit_retains_original_status_and_complete_raw_capture() {
        let root = tempfile::tempdir().unwrap();
        let stdout = root.path().join("original.stdout");
        let stderr = root.path().join("original.stderr");
        let mut command = Command::new("/bin/sh");
        command.args(["-c", "printf 'original-before-signal\\n'; printf 'original-error\\n' >&2; kill -TERM $$; printf 'unreachable\\n'"]);
        let observed = run(
            command,
            &stdout,
            &stderr,
            crate::managed::result::GenericObserver,
            PayloadPolicy {
                startup: Duration::from_secs(30),
                drain: Duration::from_secs(30),
                write: Duration::from_secs(5),
            },
        )
        .await
        .unwrap();
        assert_eq!(observed.status.unwrap().signal(), Some(15));
        assert_eq!(observed.child_exit, Some(128 + 15));
        assert!(
            observed.capture_complete,
            "raw EOF/custody is independent of successful execution"
        );
        assert!(
            observed.failures.is_empty(),
            "the raw owner does not reinterpret a signal as an I/O failure"
        );
        assert!(observed.stdout.as_ref().unwrap().complete);
        assert!(observed.stderr.as_ref().unwrap().complete);
        assert_eq!(std::fs::read(stdout).unwrap(), b"original-before-signal\n");
        assert_eq!(std::fs::read(stderr).unwrap(), b"original-error\n");
        // This raw phase observation grants no workload pass or enclosing scope settlement.
    }
}
