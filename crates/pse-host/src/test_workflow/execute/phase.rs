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
        status,
        child_exit,
        capture_complete,
        stdout,
        stderr,
        capture,
        failures,
    })
}
