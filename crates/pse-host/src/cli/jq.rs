// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded, status-preserving jq formatting for single-response CLI reads.

use super::{Cli, Command, HostError, OutputFormat, OutputOptions};
use clap::Parser as _;
use pse_diagnostics::{Cancellation, DrainFailurePolicy, SupervisedChild, SupervisionError};
use std::{
    ffi::OsString,
    io::{self, Read as _, Write as _},
    os::unix::process::ExitStatusExt as _,
    path::Path,
    process::{Command as Process, ExitStatus, Stdio},
    time::{Duration, Instant},
};

#[derive(Debug, clap::Args)]
pub(super) struct Args {
    /// One jq filter expression.
    #[arg(long, allow_hyphen_values = true)]
    filter: String,
    /// Supported pse read and its options, after `--`.
    #[arg(last = true, required = true, num_args = 1..)]
    read: Vec<OsString>,
}

const OUTPUT_BYTES: usize = 16 * 1024 * 1024;
const FORMAT_TIMEOUT: Duration = Duration::from_secs(30);
const DRAIN_TIMEOUT: Duration = Duration::from_secs(2);

pub(super) fn dispatch(
    args: Args,
    selected: Option<&Path>,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    if selected.is_some()
        || output.presentation.fields.is_some()
        || output.presentation.budget.is_some()
    {
        return Err(usage("place checkout and projection options after `--`"));
    }
    let arguments = validate_read(args.read)?;
    let mut producer = Process::new(std::env::current_exe()?);
    producer.args(arguments);
    let mut formatter = Process::new("jq");
    formatter.args(["-c", "--", &args.filter]);

    // Only these two subprocesses belong to this adapter. The read retains its own
    // repository and provider ownership, and the formatter receives bytes only.
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async {
        use tokio::signal::unix::{SignalKind, signal};
        let mut interrupt = signal(SignalKind::interrupt())?;
        let mut terminate = signal(SignalKind::terminate())?;
        let control = Cancellation::default();
        let worker_control = control.clone();
        let mut worker = tokio::task::spawn_blocking(move || {
            format_read(producer, formatter, &worker_control, OUTPUT_BYTES, FORMAT_TIMEOUT)
        });
        let stopped = tokio::select! {
            result = &mut worker => {
                let delivery = result??;
                if delivery.content.is_empty() {
                    return Ok(delivery.status);
                }
                let stdout = crate::observer_io::Writer::new(io::stdout())
                    .map_err(|error| transport(&delivery.stage, error))?;
                let stopped = tokio::select! {
                    result = stdout.write(&delivery.content) => {
                        result.map_err(|error| transport(&delivery.stage, error))?;
                        return Ok(delivery.status);
                    }
                    _ = interrupt.recv() => 2,
                    _ = terminate.recv() => 15,
                };
                report(format_args!("{} interrupted by signal {stopped}; output may be incomplete", delivery.stage));
                return Ok(128 + stopped);
            }
            _ = interrupt.recv() => 2,
            _ = terminate.recv() => 15,
        };
        control.cancel();
        // A transport or drain failure still takes precedence over an interruption.
        let settled = worker.await??;
        report(format_args!("interrupted by signal {stopped}; {}", settled.stage));
        Ok(128 + stopped)
    })
}

fn usage(message: impl Into<String>) -> HostError {
    io::Error::new(io::ErrorKind::InvalidInput, message.into()).into()
}

fn validate_read(read: Vec<OsString>) -> Result<Vec<OsString>, HostError> {
    let arguments = [OsString::from("--format=json")]
        .into_iter()
        .chain(read)
        .collect::<Vec<_>>();
    let parsed = Cli::try_parse_from(
        std::iter::once(OsString::from("pse")).chain(arguments.iter().cloned()),
    )
    .map_err(|error| usage(error.to_string()))?;
    let supported = match parsed.command {
        Some(Command::Inspect(options)) => options.supports_jq(),
        Some(Command::History { .. }) => true,
        _ => false,
    };
    if !supported || parsed.version {
        return Err(usage("pse jq requires a supported inspect or history read"));
    }
    if parsed.format != OutputFormat::Json {
        return Err(usage("pse jq requires a single JSON producer response"));
    }
    Ok(arguments)
}

fn format_read(
    producer: Process,
    formatter: Process,
    control: &Cancellation,
    limit: usize,
    timeout: Duration,
) -> Result<Delivery, HostError> {
    let original = capture(producer, None, control, limit, None)
        .map_err(|error| transport("producer", error))?;
    let producer_status = status_code(original.status);
    let bytes = original.stdout.map_err(|error| {
        transport(
            &format!("producer {} output", describe(original.status)),
            error,
        )
    })?;
    let producer_stage = format!("producer {}; jq skipped", describe(original.status));
    if control.is_cancelled() {
        return Ok(Delivery {
            status: producer_status,
            stage: producer_stage,
            content: Vec::new(),
        });
    }
    let complete = !bytes.overflow
        && serde_json::from_slice::<&serde_json::value::RawValue>(&bytes.content).is_ok();
    if !original.status.success() {
        let availability = if bytes.overflow {
            "; response exceeded capture limit"
        } else if !complete && !bytes.content.is_empty() {
            "; incomplete or invalid JSON response"
        } else {
            ""
        };
        report(format_args!("{producer_stage}{availability}"));
        return Ok(Delivery {
            status: producer_status,
            stage: format!("{producer_stage}; response delivery"),
            content: if complete { bytes.content } else { Vec::new() },
        });
    }
    if !complete {
        let reason = if bytes.overflow {
            "response exceeded capture limit"
        } else {
            "expected one complete JSON response"
        };
        return Err(transport("producer exit 0; jq skipped", reason));
    }

    let formatted = capture(
        formatter,
        Some(bytes.content),
        control,
        limit,
        Some(timeout),
    )
    .map_err(|error| transport("producer exit 0; jq", error))?;
    let status = describe(formatted.status);
    let stage = format!("producer exit 0; jq {status}");
    let bytes = formatted
        .stdout
        .map_err(|error| transport(&format!("{stage} output"), error))?;
    if bytes.overflow || formatted.timed_out {
        let reason = if bytes.overflow {
            "output exceeded capture limit"
        } else {
            "formatter deadline elapsed"
        };
        return Err(transport(&stage, reason));
    }
    if !formatted.status.success() {
        report(format_args!("{stage}"));
    }
    Ok(Delivery {
        status: status_code(formatted.status),
        stage,
        content: if formatted.status.success() {
            bytes.content
        } else {
            Vec::new()
        },
    })
}

struct Delivery {
    status: i32,
    stage: String,
    content: Vec<u8>,
}

fn report(message: std::fmt::Arguments<'_>) {
    let _ = writeln!(io::stderr().lock(), "pse jq: {message}");
}

fn transport(stage: &str, error: impl std::fmt::Display) -> HostError {
    io::Error::other(format!("pse jq: {stage}: {error}")).into()
}

fn status_code(status: ExitStatus) -> i32 {
    status
        .code()
        .or_else(|| status.signal().map(|signal| 128 + signal))
        .unwrap_or(125)
}

fn describe(status: ExitStatus) -> String {
    match status.signal() {
        Some(signal) => format!("signal {signal} (status {})", status_code(status)),
        None => format!("exit {}", status_code(status)),
    }
}

struct Capture {
    status: ExitStatus,
    stdout: io::Result<Bytes>,
    timed_out: bool,
}

struct Bytes {
    content: Vec<u8>,
    overflow: bool,
}

fn capture(
    mut command: Process,
    input: Option<Vec<u8>>,
    control: &Cancellation,
    limit: usize,
    timeout: Option<Duration>,
) -> io::Result<Capture> {
    command.stdin(if input.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    });
    command.stdout(Stdio::piped()).stderr(Stdio::inherit());
    let mut child = SupervisedChild::spawn(
        &mut command,
        DrainFailurePolicy::Refuse(DRAIN_TIMEOUT),
        None,
    )
    .map_err(io::Error::other)?;
    let mut stdout = child
        .take_stdout()
        .ok_or_else(|| io::Error::other("stdout pipe unavailable"))?;
    let stdin = child.take_stdin();
    std::thread::scope(|scope| {
        let reader = scope.spawn(move || {
            let mut bytes = Bytes {
                content: Vec::new(),
                overflow: false,
            };
            let mut chunk = [0; 8192];
            loop {
                let count = match stdout.read(&mut chunk) {
                    Ok(0) => return Ok(bytes),
                    Ok(count) => count,
                    Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                    Err(error) => {
                        control.cancel();
                        return Err(error);
                    }
                };
                let keep = count.min(limit - bytes.content.len());
                bytes.content.extend_from_slice(&chunk[..keep]);
                if keep != count {
                    bytes.overflow = true;
                    if timeout.is_some() {
                        control.cancel();
                    }
                }
            }
        });
        let writer = input
            .zip(stdin)
            .map(|(input, mut stdin)| scope.spawn(move || stdin.write_all(&input)));
        let readiness =
            child.wait_ready(Some(control), timeout.map(|value| Instant::now() + value));
        let timed_out = matches!(&readiness, Err(SupervisionError::Deadline));
        let status = if readiness.is_ok() {
            child.wait()
        } else {
            child.cancel()
        }
        .map_err(io::Error::other)?;
        if let Err(error) = readiness
            && !matches!(
                error,
                SupervisionError::Deadline | SupervisionError::Cancelled
            )
        {
            return Err(io::Error::other(format!("{}: {error}", describe(status))));
        }
        let stdout = reader
            .join()
            .map_err(|_| io::Error::other("stdout reader panicked"))?;
        if let Some(writer) = writer
            && let Err(error) = writer
                .join()
                .map_err(|_| io::Error::other("stdin writer panicked"))?
            && error.kind() != io::ErrorKind::BrokenPipe
        {
            return Err(io::Error::other(format!("{}: {error}", describe(status))));
        }
        Ok(Capture {
            status,
            stdout,
            timed_out,
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shell(script: &str) -> Process {
        let mut command = Process::new("/bin/sh");
        command.args(["-c", script]);
        command
    }

    #[test]
    fn jq_preserves_complete_failed_responses_and_rejects_invalid_successes() {
        let run = |script| {
            format_read(
                shell(script),
                shell("exit 91"),
                &Cancellation::default(),
                1024,
                FORMAT_TIMEOUT,
            )
        };
        let original = run("printf '{\"subject_state\":\"failed\"}\\n'; exit 23").unwrap();
        assert_eq!(original.status, 23);
        assert_eq!(original.content, b"{\"subject_state\":\"failed\"}\n");
        for script in ["true", "printf '{'", "printf '{} {}'"] {
            assert!(
                run(script)
                    .err()
                    .unwrap()
                    .to_string()
                    .contains("expected one complete JSON response")
            );
        }
    }

    #[test]
    fn jq_bounds_capture_and_settles_on_formatter_deadline() {
        let run = |producer, formatter, limit, timeout| {
            format_read(
                shell(producer),
                shell(formatter),
                &Cancellation::default(),
                limit,
                timeout,
            )
        };
        let original = run("printf 123456789; exit 23", "exit 91", 8, FORMAT_TIMEOUT).unwrap();
        assert_eq!(original.status, 23);
        assert!(original.content.is_empty());
        let producer = run("printf 123456789", "exit 91", 8, FORMAT_TIMEOUT)
            .err()
            .unwrap();
        assert!(producer.to_string().contains("producer exit 0; jq skipped"));
        let overflow = run(
            "printf '{}'",
            "printf 123456789; exec /bin/sleep 10",
            8,
            FORMAT_TIMEOUT,
        )
        .err()
        .unwrap();
        assert!(
            overflow
                .to_string()
                .contains("output exceeded capture limit")
        );
        let deadline = run(
            "printf '{}'",
            "exec /bin/sleep 10",
            8,
            Duration::from_millis(100),
        )
        .err()
        .unwrap();
        assert!(deadline.to_string().contains("formatter deadline elapsed"));
    }
}
