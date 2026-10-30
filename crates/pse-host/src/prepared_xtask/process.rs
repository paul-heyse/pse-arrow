// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded stdout draining under the existing original native child owner.
use super::{PreparedXtaskError, invalid};
use pse_diagnostics::{DrainFailurePolicy, StderrCapture, SupervisedChild};
use std::{
    io::Read,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
pub(super) struct Output {
    pub status: i32,
    pub stdout: Vec<u8>,
    pub stdout_bytes: u64,
    pub stderr: Vec<u8>,
}
pub(super) fn run(
    mut command: Command,
    maximum: usize,
    timeout: Option<Duration>,
) -> Result<Output, PreparedXtaskError> {
    command.stdin(Stdio::null()).stdout(Stdio::piped());
    let deadline = timeout.map(|duration| Instant::now() + duration);
    let mut child = SupervisedChild::spawn(
        &mut command,
        DrainFailurePolicy::Park,
        Some(StderrCapture {
            bytes: 65536,
            stack_bytes: 256 * 1024,
        }),
    )?;
    let mut failures = Vec::new();
    let reader = match child.take_stdout() {
        Some(mut stdout) => match std::thread::Builder::new()
            .name("pse-xtask-output".into())
            .spawn(move || {
                let mut bytes = Vec::new();
                let mut total = 0_u64;
                let mut buffer = [0_u8; 8192];
                loop {
                    let count = match stdout.read(&mut buffer) {
                        Ok(0) => return (bytes, total, None),
                        Ok(count) => count,
                        Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                        Err(error) => return (bytes, total, Some(error)),
                    };
                    total += count as u64;
                    bytes.extend_from_slice(&buffer[..count.min(maximum - bytes.len())]);
                }
            }) {
            Ok(task) => Some(task),
            Err(error) => {
                failures.push(error.into());
                None
            }
        },
        None => {
            failures.push(invalid("child stdout pipe absent"));
            None
        }
    };
    if failures.is_empty()
        && let Err(error) = child.wait_ready(None, deadline)
    {
        failures.push(error.into());
    }
    let status = match child.cancel() {
        Ok(status) => Some(status),
        Err(error) => {
            failures.push(error.into());
            child.retained_status()
        }
    };
    let (mut stdout, mut stdout_bytes) = (Vec::new(), 0);
    if let Some(reader) = reader {
        match reader.join() {
            Ok((bytes, total, error)) => {
                stdout = bytes;
                stdout_bytes = total;
                if let Some(error) = error {
                    failures.push(error.into());
                }
            }
            Err(_) => failures.push(invalid("native stdout worker panicked")),
        }
    }
    let stderr = child.take_stderr_prefix().unwrap_or_default();
    if !failures.is_empty() {
        return Err(PreparedXtaskError::Execution {
            failures,
            stdout,
            stderr,
        });
    }
    let status = status.ok_or_else(|| invalid("native child did not establish original exit"))?;
    use std::os::unix::process::ExitStatusExt as _;
    Ok(Output {
        status: status
            .code()
            .unwrap_or_else(|| 128 + status.signal().unwrap_or(1)),
        stdout,
        stdout_bytes,
        stderr,
    })
}
