// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One bounded owner projection, replaced atomically rather than recording every event.
use pse_runtime::{CancelSource, workflow::ConformanceObservation};
use serde::Serialize;
use std::{
    collections::BTreeMap,
    fs::OpenOptions,
    io::{self, Write},
    os::unix::fs::OpenOptionsExt,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

// Both fixed files can coexist during replacement, including after an abrupt process exit.
const FILE_BYTES: usize = 2 * 1024 * 1024;
const MAX_CASES: usize = 1024;
const MAX_RUNS: usize = 128;
const REFRESH: Duration = Duration::from_millis(250);

#[derive(Debug, Default, Serialize)]
struct Case {
    phase: Option<&'static str>,
    status: Option<String>,
    finding: Option<String>,
}
#[derive(Debug, Default, Serialize)]
struct Run {
    runtime_run: Option<String>,
    phase: Option<&'static str>,
    cases: BTreeMap<String, Case>,
    total_fixtures: Option<usize>,
    passed: Option<bool>,
    coverage_complete: Option<bool>,
    execution_error: Option<String>,
    settlement: Option<&'static str>,
    export: Option<&'static str>,
    published_outputs: Vec<String>,
}
#[derive(Debug, Serialize)]
struct Snapshot {
    version: u8,
    historical_image: bool,
    updated_at_ms: u128,
    requested_fixtures: Vec<String>,
    selected_runs: usize,
    omitted_requested_fixtures: usize,
    runs: BTreeMap<String, Run>,
}

#[derive(Debug)]
struct State {
    path: PathBuf,
    snapshot: Snapshot,
    last_publication: Option<Instant>,
    unavailable: bool,
    dirty: bool,
}
impl State {
    fn publish(&mut self, force: bool) {
        if self.unavailable
            || !self.dirty
            || (!force && self.last_publication.is_some_and(|last| last.elapsed() < REFRESH))
        {
            return;
        }
        self.snapshot.updated_at_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_millis());
        let operation = (|| -> io::Result<()> {
            let bytes = serde_json::to_vec(&self.snapshot).map_err(io::Error::other)?;
            if bytes.len() > FILE_BYTES {
                return Err(io::Error::other("conformance progress exceeds its reserved extent"));
            }
            let staging = self.path.with_extension("pending");
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&staging)?;
            let published = (|| {
                file.write_all(&bytes)?;
                file.flush()?;
                std::fs::rename(&staging, &self.path)
            })();
            if published.is_err() {
                let _ = std::fs::remove_file(staging);
            }
            published
        })();
        if let Err(error) = operation {
            self.unavailable = true;
            eprintln!("conformance progress unavailable: {error}");
        } else {
            self.last_publication = Some(Instant::now());
            self.dirty = false;
        }
    }
}

/// Bounded publisher allocated only by the managed conformance route.
#[derive(Clone, Debug)]
pub struct CampaignProgress(Arc<Mutex<State>>);
impl CampaignProgress {
    /// Bind to the managed invocation's exact progress path; direct callers need not publish.
    pub fn from_environment(campaign: &super::Campaign, historical_image: bool) -> Option<Self> {
        let path = PathBuf::from(std::env::var_os("PSE_CONFORMANCE_PROGRESS")?);
        if !path.is_absolute() || !path.parent().is_some_and(std::path::Path::is_dir) {
            eprintln!("conformance progress unavailable: original owner path is invalid");
            return None;
        }
        let requested = campaign.fixtures.as_deref().unwrap_or_default();
        let mut state = State {
            path,
            snapshot: Snapshot {
                version: 1,
                historical_image,
                updated_at_ms: 0,
                requested_fixtures: requested.iter().take(MAX_CASES).map(ToString::to_string).collect(),
                selected_runs: campaign.runs.len(),
                omitted_requested_fixtures: requested.len().saturating_sub(MAX_CASES),
                runs: campaign.runs().take(MAX_RUNS).map(|run| {
                    // A fixture request is constrained to one manifest run. The runtime
                    // replaces these requested rows with its admitted selection.
                    let cases = requested.iter().take(MAX_CASES)
                        .map(|id| (id.to_string(), Case { phase: Some("requested"), ..Case::default() }))
                        .collect();
                    (run.name.clone(), Run { cases, ..Run::default() })
                }).collect(),
            },
            last_publication: None,
            unavailable: false,
            dirty: true,
        };
        state.publish(true);
        Some(Self(Arc::new(Mutex::new(state))))
    }
    fn update(&self, run: &str, force: bool, update: impl FnOnce(&mut Run)) {
        if let Ok(mut state) = self.0.lock() {
            if state.unavailable {
                return;
            }
            if let Some(selected) = state.snapshot.runs.get_mut(run) {
                update(selected);
                state.dirty = true;
                state.publish(force);
            }
        }
    }
    /// Record a phase observed by the campaign driver.
    pub fn phase(&self, run: &str, phase: &'static str) {
        self.update(run, true, |entry| entry.phase = Some(phase));
    }
    /// Flush changed owner facts when a long phase produces no further transitions.
    pub fn flush(&self) {
        if let Ok(mut state) = self.0.lock() {
            state.publish(false);
        }
    }
    /// Project native or pure case facts supplied by the runtime owner.
    pub fn observation(&self, name: &str, event: ConformanceObservation) {
        let force = matches!(event, ConformanceObservation::Selected { .. });
        let case_limit = self.0.lock().ok().map_or(0, |state| MAX_CASES / state.snapshot.runs.len().max(1));
        self.update(name, force, |entry| match event {
            ConformanceObservation::Selected { run, fixtures } => {
                entry.runtime_run = Some(run.to_string());
                entry.total_fixtures = Some(fixtures.len());
                entry.cases = fixtures.iter().take(case_limit).map(|id| (id.to_string(), Case::default())).collect();
            }
            ConformanceObservation::Phase { fixture, phase, .. } => {
                if let Some(case) = entry.cases.get_mut(&fixture.to_string()) {
                    case.phase = Some(phase);
                }
            }
            ConformanceObservation::Returned { fixture, status, finding, .. } => {
                if let Some(case) = entry.cases.get_mut(&fixture.to_string()) {
                    case.phase = Some("returned");
                    case.status = status.map(|status| format!("{status:?}"));
                    case.finding = finding.as_deref().map(bounded_finding);
                }
            }
        });
    }
    /// Retain scientific completion independently of later cleanup and export.
    pub fn report(&self, report: &super::RunReport) {
        self.update(&report.name, true, |entry| {
            entry.passed = Some(report.report.passed());
            entry.coverage_complete = Some(report.report.complete);
            for (id, status) in &report.report.fixture_statuses {
                if let Some(case) = entry.cases.get_mut(&id.to_string()) {
                    case.status = Some(format!("{status:?}"));
                }
            }
        });
    }
    /// Keep an execution error even if scientific completion is unavailable.
    pub fn execution_error(&self, run: &str, error: &anyhow::Error) {
        self.update(run, true, |entry| entry.execution_error = Some(bounded_finding(&error.to_string())));
    }
    /// Record the native owner result independently of original process exit.
    pub fn settlement(&self, run: &str, state: &'static str) {
        self.update(run, true, |entry| entry.settlement = Some(state));
    }
    /// Record export disposition independently of scientific evidence.
    pub fn export(&self, run: &str, state: &'static str) {
        self.update(run, true, |entry| entry.export = Some(state));
    }
    /// Record one actual file publication, including partial exports.
    pub fn published_output(&self, run: &str, output: &std::path::Path) {
        if let Some(name) = output.file_name() {
            self.update(run, true, |entry| entry.published_outputs.push(name.to_string_lossy().into_owned()));
        }
    }
}

fn bounded_finding(text: &str) -> String {
    text.chars().take(128).collect()
}

/// Process-root interruption leaves native draining with the scientific owner.
#[derive(Debug)]
pub struct ProcessInterrupt {
    listener: tokio::task::JoinHandle<()>,
}
impl ProcessInterrupt {
    /// Register both signals synchronously in the entered Tokio runtime before admitting work.
    pub fn new(cancel: CancelSource) -> io::Result<Self> {
        use tokio::signal::unix::{SignalKind, signal};
        let mut interrupt = signal(SignalKind::interrupt())?;
        let mut terminate = signal(SignalKind::terminate())?;
        Ok(Self {
            listener: tokio::spawn(async move {
                tokio::select! {
                    _ = interrupt.recv() => {},
                    _ = terminate.recv() => {},
                }
                cancel.cancel();
            }),
        })
    }
    /// Join the listener only after issued scientific work has drained.
    pub async fn finish(mut self) {
        self.listener.abort();
        let _ = (&mut self.listener).await;
    }
}
impl Drop for ProcessInterrupt {
    fn drop(&mut self) {
        self.listener.abort();
    }
}
