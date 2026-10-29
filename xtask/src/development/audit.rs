// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original dependency-tool output, with findings separate from operational failure.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    io::Write,
    path::Path,
    process::Command,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

/// Retained read-only dependency assessments. None of these commands rewrites manifests.
#[derive(Clone, Copy, Debug, Eq, PartialEq, clap::ValueEnum)]
pub enum Gate {
    /// Cargo-deny dependency graph diagnostics.
    #[value(name = "audit-dependencies")]
    Dependencies,
    /// RustSec vulnerability and warning diagnostics.
    #[value(name = "audit-advisories")]
    Advisories,
    /// Cargo-shear unused dependency and declaration diagnostics.
    #[value(name = "audit-shear")]
    Shear,
}

impl Gate {
    /// Stable report name retained at the existing evidence boundary.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Dependencies => "audit-dependencies",
            Self::Advisories => "audit-advisories",
            Self::Shear => "audit-shear",
        }
    }

    /// Original tool invocation, without a policy wrapper or alternate dependency resolver.
    pub fn command(self) -> Vec<String> {
        let arguments: &[&str] = match self {
            Self::Dependencies => &[
                "cargo",
                "deny",
                "--format",
                "json",
                "--all-features",
                "--locked",
                "check",
            ],
            Self::Advisories => &["cargo", "audit", "--json", "--deny", "warnings"],
            Self::Shear => &[
                "cargo",
                "shear",
                "--locked",
                "--check-test-targets",
                "--deny-warnings",
                "--format",
                "json",
            ],
        };
        arguments
            .iter()
            .map(|argument| (*argument).to_owned())
            .collect()
    }
}

/// A strict tool result; the policy consumer decides whether findings block adoption.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// The original tool completed without findings.
    Passed,
    /// A valid report accounts for the tool's strict finding exit.
    Findings,
    /// Launch, execution or report interpretation failed.
    Failed,
}

impl Status {
    /// Preserve the existing assessment exit distinction (0 clean, 1 findings, 2 failure).
    pub const fn exit_code(self) -> i32 {
        match self {
            Self::Passed => 0,
            Self::Findings => 1,
            Self::Failed => 2,
        }
    }
}

#[derive(Deserialize)]
struct ShearReport {
    findings: Vec<ShearFinding>,
    summary: ShearSummary,
}
#[derive(Deserialize)]
struct ShearFinding {
    code: String,
    severity: String,
}
#[derive(Deserialize)]
struct ShearSummary {
    errors: u64,
    warnings: u64,
}

/// Interpret only documented finding exits, never arbitrary nonzero tool failures.
pub fn finding_status(gate: Gate, code: Option<i32>, stdout: &str, stderr: &str) -> Status {
    if gate == Gate::Shear {
        let Ok(report) = serde_json::from_str::<ShearReport>(stdout) else {
            return Status::Failed;
        };
        let mut errors = 0;
        let mut warnings = 0;
        for finding in &report.findings {
            if !finding.code.starts_with("shear/") {
                return Status::Failed;
            }
            match finding.severity.as_str() {
                "error" => errors += 1,
                "warning" => warnings += 1,
                _ => return Status::Failed,
            }
        }
        if report.summary.errors != errors || report.summary.warnings != warnings {
            return Status::Failed;
        }
        return match (code, report.findings.is_empty()) {
            (Some(0), true) => Status::Passed,
            (Some(1), false) => Status::Findings,
            _ => Status::Failed,
        };
    }
    if code == Some(0) {
        return Status::Passed;
    }
    if gate == Gate::Dependencies {
        return deny_status(code, stderr).unwrap_or(Status::Failed);
    }
    if code == Some(1)
        && let Ok(report) = serde_json::from_str::<Value>(stdout)
        && (report
            .pointer("/vulnerabilities/found")
            .and_then(Value::as_bool)
            == Some(true)
            || report
                .get("warnings")
                .and_then(Value::as_object)
                .is_some_and(|warnings| !warnings.is_empty()))
    {
        return Status::Findings;
    }
    Status::Failed
}

fn deny_status(code: Option<i32>, stderr: &str) -> Option<Status> {
    let mut summary = None;
    for line in stderr.lines().filter(|line| !line.trim().is_empty()) {
        let record: Value = serde_json::from_str(line).ok()?;
        match record.get("type")?.as_str()? {
            "diagnostic" => {}
            "summary" if summary.is_none() => summary = record.get("fields").cloned(),
            _ => return None,
        }
    }
    let summary = summary?;
    let mut mask = 0;
    // cargo-deny check uses a bitset, not a Boolean exit (upstream check.md).
    for (name, bit) in [
        ("advisories", 1),
        ("bans", 2),
        ("licenses", 4),
        ("sources", 8),
    ] {
        if summary.get(name)?.get("errors")?.as_u64()? > 0 {
            mask |= bit;
        }
    }
    Some(if mask > 0 && code == Some(mask) {
        Status::Findings
    } else {
        Status::Failed
    })
}

#[derive(Debug, Serialize)]
struct Invocation {
    command: Vec<String>,
    exit_code: Option<i32>,
    signal: Option<i32>,
    stdout: String,
    stderr: String,
    started: f64,
    elapsed_seconds: f64,
    status: Status,
}

#[derive(Debug, Serialize)]
struct Report {
    gate: &'static str,
    started: f64,
    status: Status,
    invocations: Vec<Invocation>,
}

fn timestamp() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0.0, |duration| duration.as_secs_f64())
}

fn invoke(root: &Path, gate: Gate, command: Vec<String>) -> Invocation {
    let started = timestamp();
    let clock = Instant::now();
    let outcome = Command::new(&command[0])
        .args(&command[1..])
        .current_dir(root)
        .output();
    let mut record = Invocation {
        command,
        exit_code: None,
        signal: None,
        stdout: String::new(),
        stderr: String::new(),
        started,
        elapsed_seconds: clock.elapsed().as_secs_f64(),
        status: Status::Failed,
    };
    match outcome {
        Ok(result) => {
            record.exit_code = result.status.code();
            #[cfg(unix)]
            {
                use std::os::unix::process::ExitStatusExt;
                record.signal = result.status.signal();
            }
            record.stdout = String::from_utf8_lossy(&result.stdout).into_owned();
            record.stderr = String::from_utf8_lossy(&result.stderr).into_owned();
            if std::str::from_utf8(&result.stdout).is_ok()
                && std::str::from_utf8(&result.stderr).is_ok()
            {
                record.status =
                    finding_status(gate, record.exit_code, &record.stdout, &record.stderr);
            }
        }
        Err(error) => record.stderr = error.to_string(),
    }
    record
}

/// Run one original tool and publish its raw detail at the existing audit output boundary.
///
/// # Errors
/// Report publication or forwarding original output failed. Missing tools and tool failures
/// are retained in the report and return status 2.
pub fn run(root: &Path, gate: Gate, output: &Path) -> Result<i32> {
    let record = invoke(root, gate, gate.command());
    let status = record.status;
    let report = Report {
        gate: gate.name(),
        started: record.started,
        status,
        invocations: vec![record],
    };
    std::fs::create_dir_all(output).with_context(|| format!("creating {}", output.display()))?;
    let path = output.join(format!("{}-tool.json", gate.name()));
    std::fs::write(&path, serde_json::to_vec_pretty(&report)?)
        .with_context(|| format!("writing {}", path.display()))?;
    for invocation in &report.invocations {
        std::io::stdout()
            .lock()
            .write_all(invocation.stdout.as_bytes())?;
        std::io::stderr()
            .lock()
            .write_all(invocation.stderr.as_bytes())?;
    }
    Ok(status.exit_code())
}

#[cfg(test)]
#[path = "audit_tests.rs"]
mod tests;
