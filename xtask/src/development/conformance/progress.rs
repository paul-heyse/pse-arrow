// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One bounded owner projection, replaced atomically rather than recording every event.
use pse_model::generated::identities::{DeclarationId, RunId};
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
    runtime_run: Option<RunId>,
    phase: Option<&'static str>,
    cases: BTreeMap<DeclarationId, Case>,
    total_fixtures: Option<usize>,
    passed: Option<bool>,
    coverage_complete: Option<bool>,
    execution_error: Option<String>,
    settlement: Option<&'static str>,
    settlement_error: Option<String>,
    export: Option<&'static str>,
    export_error: Option<String>,
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
            || (!force
                && self
                    .last_publication
                    .is_some_and(|last| last.elapsed() < REFRESH))
        {
            return;
        }
        self.snapshot.updated_at_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_millis());
        let operation = (|| -> io::Result<()> {
            let bytes = serde_json::to_vec(&self.snapshot).map_err(io::Error::other)?;
            if bytes.len() > FILE_BYTES {
                return Err(io::Error::other(
                    "conformance progress exceeds its reserved extent",
                ));
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
        Some(Self::new(path, campaign, historical_image))
    }
    fn new(path: PathBuf, campaign: &super::Campaign, historical_image: bool) -> Self {
        let requested = campaign.fixtures.as_deref().unwrap_or_default();
        let mut state = State {
            path,
            snapshot: Snapshot {
                version: 1,
                historical_image,
                updated_at_ms: 0,
                requested_fixtures: requested
                    .iter()
                    .take(MAX_CASES)
                    .map(ToString::to_string)
                    .collect(),
                selected_runs: campaign.runs.len(),
                omitted_requested_fixtures: requested.len().saturating_sub(MAX_CASES),
                runs: campaign
                    .runs()
                    .take(MAX_RUNS)
                    .map(|run| {
                        // A fixture request is constrained to one manifest run. The runtime
                        // replaces these requested rows with its admitted selection.
                        let cases = requested
                            .iter()
                            .take(MAX_CASES)
                            .map(|id| {
                                (
                                    *id,
                                    Case {
                                        phase: Some("requested"),
                                        ..Case::default()
                                    },
                                )
                            })
                            .collect();
                        (
                            run.name.clone(),
                            Run {
                                cases,
                                ..Run::default()
                            },
                        )
                    })
                    .collect(),
            },
            last_publication: None,
            unavailable: false,
            dirty: true,
        };
        state.publish(true);
        Self(Arc::new(Mutex::new(state)))
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
    /// Publish all runs left unattempted after the driver closes admission.
    pub fn unattempted<'a>(&self, runs: impl IntoIterator<Item = &'a str>) {
        let Ok(mut state) = self.0.lock() else { return };
        if state.unavailable {
            return;
        }
        let mut changed = false;
        for run in runs {
            let Some(entry) = state.snapshot.runs.get_mut(run) else {
                continue;
            };
            changed = true;
            entry.phase = Some("unattempted");
            for case in entry.cases.values_mut() {
                case.phase = Some("unattempted");
                case.status = Some("Unattempted".into());
            }
        }
        if changed {
            state.dirty = true;
            state.publish(true);
        }
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
        let Ok(mut state) = self.0.lock() else { return };
        if state.unavailable {
            return;
        }
        let case_limit = MAX_CASES / state.snapshot.runs.len().max(1);
        let Some(entry) = state.snapshot.runs.get_mut(name) else {
            return;
        };
        let run_id = event.run_id();
        match event {
            ConformanceObservation::Selected { run, fixtures } => {
                if entry.runtime_run.is_some() {
                    return;
                }
                entry.runtime_run = Some(run);
                entry.total_fixtures = Some(fixtures.len());
                entry.cases = fixtures
                    .into_iter()
                    .take(case_limit)
                    .map(|id| {
                        (
                            id,
                            Case {
                                phase: Some("selected"),
                                ..Case::default()
                            },
                        )
                    })
                    .collect();
            }
            _ if entry.runtime_run != Some(run_id) => return,
            ConformanceObservation::Phase { fixture, phase, .. } => {
                let Some(case) = entry.cases.get_mut(&fixture) else {
                    return;
                };
                if matches!(case.phase, Some("returned" | "unattempted")) {
                    return;
                }
                case.phase = Some(phase);
            }
            ConformanceObservation::Returned {
                fixture,
                status,
                finding,
                ..
            } => {
                let Some(case) = entry.cases.get_mut(&fixture) else {
                    return;
                };
                if matches!(case.phase, Some("returned" | "unattempted")) {
                    return;
                }
                case.phase = Some("returned");
                case.status = status.map(|status| format!("{status:?}"));
                case.finding = finding.as_deref().map(bounded_finding);
            }
            ConformanceObservation::Unattempted { fixture, .. } => {
                let Some(case) = entry.cases.get_mut(&fixture) else {
                    return;
                };
                if case.phase != Some("selected") {
                    return;
                }
                case.phase = Some("unattempted");
                case.status = Some("Unattempted".into());
            }
        }
        state.dirty = true;
        state.publish(force);
    }
    /// Retain scientific completion independently of later cleanup and export.
    pub fn report(&self, report: &super::RunReport) {
        self.update(&report.name, true, |entry| {
            if entry.runtime_run != Some(report.report.run_id) {
                return;
            }
            entry.passed = Some(report.report.passed());
            entry.coverage_complete = Some(report.report.complete);
            for (id, status) in &report.report.fixture_statuses {
                if let Some(case) = entry.cases.get_mut(id) {
                    case.status = Some(format!("{status:?}"));
                }
            }
        });
    }
    /// Keep an execution error even if scientific completion is unavailable.
    pub fn execution_error(&self, run: &str, error: &anyhow::Error) {
        self.update(run, true, |entry| {
            entry.execution_error = Some(bounded_finding(&error.to_string()))
        });
    }
    /// Record the native owner result independently of original process exit.
    pub fn settlement(&self, run: &str, state: &'static str, error: Option<&str>) {
        self.update(run, true, |entry| {
            entry.settlement = Some(state);
            entry.settlement_error = error.map(bounded_finding);
        });
    }
    /// Record export disposition independently of scientific evidence.
    pub fn export(&self, run: &str, state: &'static str, error: Option<&str>) {
        self.update(run, true, |entry| {
            entry.export = Some(state);
            entry.export_error = error.map(bounded_finding);
        });
    }
    /// Record one actual file publication, including partial exports.
    pub fn published_output(&self, run: &str, output: &std::path::Path) {
        if let Some(name) = output.file_name() {
            self.update(run, true, |entry| {
                entry
                    .published_outputs
                    .push(name.to_string_lossy().into_owned())
            });
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

#[cfg(test)]
mod tests {
    use super::*;
    use pse_model::generated::enums::ModelingConformanceStatus as Status;
    use std::{
        io::{BufRead, BufReader},
        process::{Child, Command, Stdio},
        sync::mpsc,
    };

    // The same test binary is the original publisher in the controlled subprocess.
    // The parent may kill it only after the child's pipe acknowledges the boundary.
    #[test]
    fn abrupt_publisher_exit_preserves_only_observed_checkpoint() -> anyhow::Result<()> {
        const MANIFEST: &str = "PSE_CONFORMANCE_CONTROL_MANIFEST";
        const CHECKPOINT: &str = "PSE_CONFORMANCE_CONTROL_CHECKPOINT";
        const PUBLISH: &str = "PSE_CONFORMANCE_CONTROL_PUBLISH";
        const READY: &str = "checkpoint control reached";
        if let Some(manifest) = std::env::var_os(MANIFEST) {
            let campaign =
                super::super::Campaign::load(std::path::Path::new(&manifest), Some("first"), None)?;
            let checkpoint = PathBuf::from(std::env::var_os(CHECKPOINT).expect("child checkpoint"));
            if std::env::var_os(PUBLISH).as_deref() == Some(std::ffi::OsStr::new("yes")) {
                let progress = CampaignProgress::new(checkpoint, &campaign, false);
                let run = RunId::from_bytes([42; 16]);
                let fixture = DeclarationId::from_bytes([7; 16]);
                progress.observation(
                    "first",
                    ConformanceObservation::Selected {
                        run,
                        fixtures: vec![fixture],
                    },
                );
            }
            let mut out = io::stdout().lock();
            writeln!(out, "{READY}")?;
            out.flush()?;
            io::copy(&mut io::stdin().lock(), &mut io::sink())?;
            anyhow::bail!("parent should kill the original process before EOF");
        }
        struct Owner(Child);
        impl Drop for Owner {
            fn drop(&mut self) {
                let _ = self.0.kill();
                let _ = self.0.wait();
            }
        }
        let (dir, manifest) = super::super::tests::fixture()?;
        let module = module_path!().split_once("::").expect("qualified module").1;
        let test = format!("{module}::abrupt_publisher_exit_preserves_only_observed_checkpoint");
        for publish in ["no", "yes"] {
            let checkpoint = dir.path().join(format!("{publish}.json"));
            let mut owner = Owner(
                Command::new(std::env::current_exe()?)
                    .args(["--exact", &test, "--nocapture"])
                    .env(MANIFEST, &manifest)
                    .env(CHECKPOINT, &checkpoint)
                    .env(PUBLISH, publish)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()?,
            );
            let stdout = owner.0.stdout.take().expect("child stdout");
            let (send, recv) = mpsc::sync_channel(1);
            let reader = std::thread::spawn(move || {
                let seen = BufReader::new(stdout)
                    .lines()
                    .map_while(Result::ok)
                    .any(|line| line.contains(READY));
                let _ = send.send(seen);
            });
            let seen = recv.recv_timeout(Duration::from_secs(10));
            owner.0.kill()?;
            assert!(!owner.0.wait()?.success());
            reader.join().expect("child output reader");
            assert!(
                seen?,
                "original process did not reach the selected boundary"
            );
            if publish == "no" {
                assert!(!checkpoint.exists());
            } else {
                let observation: serde_json::Value =
                    serde_json::from_slice(&std::fs::read(checkpoint)?)?;
                let run = &observation["runs"]["first"];
                assert_eq!(run["total_fixtures"], 1);
                assert!(
                    run["passed"].is_null()
                        && run["settlement"].is_null()
                        && run["export"].is_null()
                );
            }
        }
        Ok(())
    }

    #[test]
    fn checkpoint_binds_original_run_and_bounds_cases_without_inventing_completion()
    -> anyhow::Result<()> {
        let (dir, manifest) = super::super::tests::fixture()?;
        let campaign = super::super::Campaign::load(&manifest, Some("first"), None)?;
        let path = dir.path().join("conformance-progress.json");
        let progress = CampaignProgress::new(path.clone(), &campaign, false);
        let read = || -> anyhow::Result<serde_json::Value> {
            Ok(serde_json::from_slice(&std::fs::read(&path)?)?)
        };
        assert_eq!(
            read()?["runs"]["first"]["runtime_run"],
            serde_json::Value::Null
        );
        let run = RunId::from_bytes([1; 16]);
        let foreign = RunId::from_bytes([2; 16]);
        let fixture_id = |index: usize| DeclarationId::from_bytes((index as u128).to_be_bytes());
        let fixture = fixture_id(0);
        let unattempted = fixture_id(1);
        progress.observation(
            "first",
            ConformanceObservation::Selected {
                run,
                fixtures: (0..=MAX_CASES).map(fixture_id).collect(),
            },
        );
        let admitted = read()?;
        let original = &admitted["runs"]["first"];
        assert_eq!(original["total_fixtures"], MAX_CASES + 1);
        assert_eq!(
            original["cases"].as_object().map(serde_json::Map::len),
            Some(MAX_CASES)
        );
        assert_eq!(original["cases"][fixture.to_string()]["phase"], "selected");
        assert!(original["passed"].is_null());
        progress.observation(
            "first",
            ConformanceObservation::Returned {
                run: foreign,
                fixture,
                status: Some(Status::Passed),
                finding: None,
            },
        );
        progress.observation(
            "first",
            ConformanceObservation::Returned {
                run,
                fixture,
                status: Some(Status::Failed),
                finding: Some("first failure".into()),
            },
        );
        progress.observation(
            "first",
            ConformanceObservation::Returned {
                run,
                fixture,
                status: Some(Status::Passed),
                finding: None,
            },
        );
        progress.observation(
            "first",
            ConformanceObservation::Unattempted {
                run,
                fixture: unattempted,
            },
        );
        progress.settlement("first", "unresolved", Some("cleanup failure"));
        let final_observation = read()?;
        let original = &final_observation["runs"]["first"];
        let case = &original["cases"][fixture.to_string()];
        assert_eq!(case["status"], "Failed");
        assert_eq!(case["finding"], "first failure");
        assert_eq!(
            original["cases"][unattempted.to_string()]["status"],
            "Unattempted"
        );
        assert_eq!(original["settlement_error"], "cleanup failure");
        assert!(original["passed"].is_null());
        Ok(())
    }
}
