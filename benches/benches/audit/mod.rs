// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Opt-in audit control around the existing once-only scientific workloads.
//! No diagnostic reader owns or cancels scientific work. Only stop.request does so.
#![allow(
    dead_code,
    reason = "the two benchmark targets share audit helpers with different drivers"
)]
use futures_util::FutureExt;
use pse_diagnostics::observation::{ObservationContext, OperationGuard, Outcome, PhaseGuard};
use pse_runtime::tooling::inspection::InspectionRequest;
use pse_runtime::{CancelSource, observation::process_diagnostics};
use serde_json::{Value, json};
use std::{
    future::Future,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

const MAX_INSPECTIONS: u64 = 128;
// The entrypoint's compiled capabilities, not a claim that a particular case used them.
// Actual performing constructors separately declare source activation in the collector.
fn supported_record_families() -> Vec<&'static str> {
    let mut families = vec![
        "evaluator_demand_batch.v1",
        "evaluator_demand.v1",
        "evaluator_census.v1",
        "workflow.integration.v1",
        "workflow.fit_demand.v1",
    ];
    if cfg!(feature = "native-process") {
        families.extend([
            "native_owner_return.v1",
            "root_evaluation.v1",
            "root_raw_map.v1",
            "callback_boundary.v1",
            "callback_failure.v1",
            "nonlinear_iteration.v1",
            "dynamic_native_return.v1",
            "dynamic_native_step.v1",
            "dynamic_segment_summary.v1",
            "dynamic_local_error.v1",
            "native_linear_work.v1",
            "dynamic_replay_effort.v1",
            "dynamic_output.v1",
            "dynamic_adjoint.v1",
            "branch_and_bound.v1",
            "linear_search.v1",
            "conic_iteration.v1",
            "convex_return.v1",
            "native_linear_call.v1",
            "native_factor.v2",
            "native_refinement.v2",
            "petsc_event.v1",
        ]);
    }
    families
}
static STATE: OnceLock<Option<Arc<State>>> = OnceLock::new();
type RecordingOwner = Arc<Mutex<Option<pse_runtime::tooling::capture::SourceRecordingSession>>>;
static RECORDING: OnceLock<RecordingOwner> = OnceLock::new();
static SOURCE_CONTEXT: OnceLock<ObservationContext> = OnceLock::new();
#[derive(Debug)]
struct State {
    directory: PathBuf,
    context: ObservationContext,
    cancel: CancelSource,
    owners: Mutex<Vec<(u64, CancelSource)>>,
    math_controls: Mutex<Vec<(u64, pse_diagnostics::Cancellation)>>,
    next_owner: AtomicU64,
    stop_requested: AtomicBool,
    publisher_stop: AtomicBool,
    inspections: AtomicU64,
    omitted_inspections: AtomicU64,
    cleanups: AtomicU64,
    cleanup_failures: AtomicU64,
    joins: AtomicU64,
    receiver_drains: AtomicU64,
    receiver_drain_failures: AtomicU64,
    result_export_remaining: AtomicU64,
    result_exports: AtomicU64,
    errors: Mutex<Vec<String>>,
}
fn state() -> Option<&'static Arc<State>> {
    STATE.get().and_then(Option::as_ref)
}
fn write(directory: &Path, name: &str, document: &Value) -> Result<(), String> {
    let temporary = directory.join(format!("{name}.pending"));
    std::fs::write(
        &temporary,
        serde_json::to_vec_pretty(document).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    std::fs::rename(temporary, directory.join(name)).map_err(|e| e.to_string())
}
impl State {
    fn error(&self, error: impl ToString) {
        let mut errors = self
            .errors
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if errors.len() < 16 {
            errors.push(error.to_string());
        }
    }
    fn snapshot(&self, stage: &str) {
        if let Err(error) = process_diagnostics().publish() {
            self.error(error);
        }
        let sequence = self.inspections.fetch_add(1, Ordering::Relaxed);
        if sequence >= MAX_INSPECTIONS {
            self.omitted_inspections.fetch_add(1, Ordering::Relaxed);
            return;
        }
        let document = process_diagnostics().inspect(InspectionRequest::new(self.context.id()));
        let result = write(
            &self.directory,
            &format!("inspection-{sequence:04}.json"),
            &json!({"stage":stage,"inspection":document}),
        );
        if let Err(error) = result {
            self.error(error);
        }
    }
}
fn install_recording(
    session: Option<pse_runtime::tooling::capture::SourceRecordingSession>,
) -> Option<RecordingOwner> {
    session.map(|session| {
        let owner = Arc::new(Mutex::new(Some(session)));
        assert!(
            RECORDING.set(owner.clone()).is_ok(),
            "one recording owner per process"
        );
        owner
    })
}
fn take_recording(
    owner: RecordingOwner,
) -> Option<pse_runtime::tooling::capture::SourceRecordingSession> {
    owner
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take()
}
/// Optional failures never alter the original scientific result or cancellation owner.
pub(crate) fn review_context_error(error: impl ToString) {
    let message = format!("review handoff unavailable: {}", error.to_string());
    if let Some(state) = state() {
        state.error(&message);
    } else {
        let _ =
            std::io::Write::write_all(&mut std::io::stderr(), format!("{message}\n").as_bytes());
    }
}
pub(crate) fn review_context_request() -> Option<pse_runtime::workflow::ReviewContextRequest> {
    let recording = RECORDING
        .get()?
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    recording
        .as_ref()?
        .acknowledgement()
        .review
        .as_ref()
        .filter(|review| review.accepted)
        .map(|review| review.request.context.clone())
}
pub(crate) fn retain_context(context: &pse_runtime::workflow::ReviewContextDocument) {
    if let Some(owner) = RECORDING.get() {
        let mut owner = owner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(recording) = owner.as_mut()
            && let Err(error) = recording.retain_context(context)
        {
            review_context_error(error);
        }
    }
}
pub(crate) fn retain_result(result: &Arc<pse_runtime::workflow::RunResult>) {
    if result.canonical_run_key().is_none()
        && std::env::var("PSE_CASE_REVIEW_RESULTS").as_deref() == Ok("1")
        && let Some(state) = state()
    {
        let directory = state
            .directory
            .parent()
            .unwrap_or(&state.directory)
            .join("review-results");
        let outcome = std::fs::create_dir_all(&directory).map_err(|error| error.to_string()).and_then(|()| {
            write(&directory, &format!("{}-export.json", result.run_id), &json!({
                "version":1,"run":result.run_id,"status":"unavailable",
                "canonical_run":null,"canonical_attempt":null,"pack":null,
                "error":"original result is ephemeral; protected canonical export is unavailable"
            }))
        });
        if let Err(error) = outcome {
            review_context_error(error);
        }
    }
    if let Some(owner) = RECORDING.get() {
        let mut owner = owner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(recording) = owner.as_mut()
            && let Err(error) = recording.retain_result(result)
        {
            review_context_error(error);
        }
    }
}

/// Preserve actual direct-math facts without inventing a workflow invocation.
pub(crate) fn retain_math_output(name: &str, document: &Value) {
    if review_context_request().is_none() {
        return;
    }
    let Some(state) = state() else {
        return;
    };
    let directory = state.directory.join("math-originals");
    let outcome = (|| -> Result<(), String> {
        std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
        let bytes = serde_json::to_vec_pretty(document).map_err(|error| error.to_string())?;
        if bytes.len() > 16 << 20 {
            return Err("direct math original exceeds 16 MiB handoff allowance".into());
        }
        write(&directory, &format!("{name}.json"), document)
    })();
    if let Err(error) = outcome {
        review_context_error(error);
    }
    review_context_error(format!(
        "{name}: original math output retained separately; workflow context adapter unavailable"
    ));
}

/// Publish exact original protected selections while their fixture still exists.
/// This diagnostic export has its own finite allowance and cancellation token.
pub(crate) async fn export_result(
    runtime: &pse_runtime::workflow::Runtime,
    result: &Arc<pse_runtime::workflow::RunResult>,
) {
    if std::env::var("PSE_CASE_REVIEW_RESULTS").as_deref() != Ok("1") {
        return;
    }
    let Some(state) = state() else {
        return;
    };
    let base = state
        .directory
        .parent()
        .unwrap_or(&state.directory)
        .join("review-results");
    let run = result.run_id.to_string();
    let destination = base.join(&run);
    if destination.exists() {
        return;
    }
    let sequence = state.result_exports.fetch_add(1, Ordering::Relaxed);
    let maximum_members = RECORDING
        .get()
        .and_then(|recording| {
            recording
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .as_ref()
                .and_then(|recording| recording.acknowledgement().review.as_ref())
                .filter(|review| review.accepted)
                .map(|review| review.request.maximum_members)
        })
        .unwrap_or(256);
    // A whole remaining grant prevents concurrent exports from overcommitting the cap.
    let allowance = if sequence < maximum_members as u64 {
        state.result_export_remaining.swap(0, Ordering::AcqRel)
    } else {
        0
    };
    let outcome: Result<(), String> = async {
        if allowance == 0 { return Err("selected result export allowance exhausted".into()); }
        let (Some(canonical_run), Some(attempt)) = (result.canonical_run_key(), result.canonical_attempt_key()) else {
            return Err("original result is ephemeral; protected canonical export is unavailable".into());
        };
        std::fs::create_dir_all(&base).map_err(|error| error.to_string())?;
        let evidence = pse_runtime::tooling::frozen_inspection::FrozenEvidenceSet::default();
        let pool: Arc<dyn pse_columnar::MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(16 << 20));
        let cancel = pse_columnar::CancellationToken::new();
        let analysis = pse_runtime::tooling::review_analysis::analyze(
            &evidence, &pse_runtime::tooling::review_analysis::AnalysisRequest::new(state.context.id()),
            &pool, &cancel,
        ).map_err(|error| error.to_string())?;
        let mut request = pse_runtime::tooling::review_pack::PackExportRequest::default();
        request.selection.maximum_output_bytes = allowance;
        request.selection.missing_context.push("Original descriptive context is retained independently by the selected source-recording handoff; this pack exports only actual protected result selections.".into());
        request.scientific_results = result.table_names().into_iter().enumerate().map(|(index, relation)| {
            pse_runtime::workflow::ReviewScientificSelection {
                member: format!("result-{index:03}"), run: canonical_run.into(), attempt: attempt.into(),
                relation, row_start: 0, row_end: u64::MAX, output: None,
            }
        }).collect();
        pse_runtime::tooling::review_export::export_review_pack(&runtime.retained_data_services(), &destination, &evidence, &analysis, request, cancel)
            .await.map_err(|error| error.to_string())?;
        Ok(())
    }.await;
    // Include incomplete retained prefixes in the finite campaign allowance.
    fn retained_bytes(path: &Path) -> u64 {
        std::fs::read_dir(path)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .map(|entry| {
                let path = entry.path();
                if path.is_dir() {
                    retained_bytes(&path)
                } else {
                    entry.metadata().map(|meta| meta.len()).unwrap_or(0)
                }
            })
            .fold(0, u64::saturating_add)
    }
    let bytes = retained_bytes(&destination);
    state
        .result_export_remaining
        .fetch_add(allowance.saturating_sub(bytes), Ordering::AcqRel);
    if let Err(error) = &outcome {
        review_context_error(format!("result {}: {error}", result.run_id));
    }
    if let Err(error) = std::fs::create_dir_all(&base).map_err(|error| error.to_string()).and_then(|()| {
        write(&base, &format!("{run}-export.json"), &json!({
            "version":1,"run":run,"canonical_run":result.canonical_run_key(),
            "canonical_attempt":result.canonical_attempt_key(),"pack":destination,
            "status":if outcome.is_ok() { "published" } else { "unavailable" },
            "error":outcome.err(),"retained_bytes":bytes,
            "scope":"actual protected original result selections exported before fixture teardown; no reconstructed science"
        }))
    }) { review_context_error(error); }
}

/// One pre-runtime operation. It coexists with the benchmark's tracing subscriber.
#[derive(Debug)]
pub(crate) struct Audit {
    operation: Option<OperationGuard>,
    state: Option<Arc<State>>,
    publisher: Option<JoinHandle<()>>,
    detail: Option<RecordingOwner>,
    capture: Option<pse_runtime::tooling::capture::CaptureSession>,
    context: Option<ObservationContext>,
    completed: bool,
}
impl Audit {
    pub(crate) fn capture_layer(&self) -> Option<pse_runtime::tooling::capture::CaptureLayer> {
        self.capture.as_ref().and_then(|session| session.layer())
    }

    pub(crate) fn start(case: &str, workload: &Value) -> Self {
        let Some(directory) = std::env::var_os("PSE_CASE_AUDIT_DIR").map(PathBuf::from) else {
            // A direct CLI launch can cooperate without enabling the audit publisher.
            let operation =
                std::env::var_os(pse_runtime::tooling::capture::SOURCE_RECORDING_REQUEST_ENV)
                    .map(|_| process_diagnostics().hub().start("audit.simulation_case"));
            let detail = operation.as_ref().and_then(|operation| {
                match pse_runtime::tooling::capture::source_recording_from_environment(
                    operation.context(),
                    &supported_record_families(),
                ) {
                    Ok(detail) => detail,
                    Err(error) => {
                        let _ = std::io::Write::write_all(
                            &mut std::io::stderr(),
                            format!("source recording unavailable: {error}\n").as_bytes(),
                        );
                        None
                    }
                }
            });
            let context = detail.as_ref().map(|detail| detail.context());
            if let Some(context) = &context {
                let _ = SOURCE_CONTEXT.set(context.clone());
            }
            return Self {
                operation,
                state: None,
                publisher: None,
                detail: install_recording(detail),
                capture: None,
                context,
                completed: false,
            };
        };
        assert!(
            !std::env::args().any(|arg| arg == "--bench"),
            "diagnostic audit requires once-only test mode"
        );
        std::fs::create_dir_all(&directory).unwrap();
        assert!(
            !directory.join("case-operation.json").exists(),
            "audit case directory already has an owner"
        );
        let diagnostics = process_diagnostics();
        let operation = diagnostics.hub().start("audit.simulation_case");
        let mut context = operation.context();
        // Select source sequences before constructing any scientific invocation.
        // Failure is observational and leaves the ordinary scientific route intact.
        let mut detail_error = None;
        let detail = match pse_runtime::tooling::capture::source_recording_from_environment(
            context.clone(),
            &supported_record_families(),
        ) {
            Ok(detail) => detail,
            Err(error) => {
                detail_error = Some(error.to_string());
                None
            }
        };
        if let Some(detail) = &detail {
            context = detail.context();
        }
        context.source_ref("audit.case", case.as_bytes());
        context.text("audit.case", case);
        context.text("audit.workload", &workload.to_string());
        let state = Arc::new(State {
            directory,
            context,
            cancel: CancelSource::new(),
            owners: Mutex::new(Vec::new()),
            math_controls: Mutex::new(Vec::new()),
            next_owner: AtomicU64::new(0),
            stop_requested: AtomicBool::new(false),
            publisher_stop: AtomicBool::new(false),
            inspections: AtomicU64::new(0),
            omitted_inspections: AtomicU64::new(0),
            cleanups: AtomicU64::new(0),
            cleanup_failures: AtomicU64::new(0),
            joins: AtomicU64::new(0),
            receiver_drains: AtomicU64::new(0),
            receiver_drain_failures: AtomicU64::new(0),
            result_export_remaining: AtomicU64::new(256 << 20),
            result_exports: AtomicU64::new(0),
            errors: Mutex::new(Vec::new()),
        });
        if let Some(error) = detail_error {
            state.error(error);
        }
        assert!(
            STATE.set(Some(state.clone())).is_ok(),
            "one audit case per benchmark process"
        );
        write(&state.directory, "case-operation.json", &json!({
            "version":1,"case":case,"workload":workload,"operation":state.context.id().to_string(),
            "checkpoint":diagnostics.checkpoint_path(),"locator":diagnostics.locator(),
            "stop_request":state.directory.join("stop.request"),
            "scope":"harness setup, original scientific operation, assessment and explicit teardown"
        })).unwrap();
        state.snapshot("startup");
        // The real root is discoverable before attachment and before preparation.
        let capture = if std::env::var("PSE_CASE_CAPTURE_FROM_START").as_deref() == Ok("1") {
            let request = pse_runtime::tooling::capture::CaptureRequest {
                version: pse_model::document::Version,
                checkpoint: diagnostics.checkpoint_path().to_path_buf(),
                operation: state.context.id(),
                kind: pse_runtime::tooling::capture::CaptureKind::Timeline,
                output: state.directory.join("capture-from-start"),
                duration: Duration::from_secs(3600),
                budget: pse_runtime::observation_detail::CaptureBudget {
                    memory_bytes: 64 << 20,
                    output_bytes: 256 << 20,
                },
            };
            if let Err(error) = write(&state.directory, "timeline-request.json", &json!(request)) {
                state.error(error);
            }
            match pse_runtime::tooling::capture::start_capture(request) {
                Ok(session) => Some(session),
                Err(error) => {
                    state.error(format!("timeline capture unavailable: {error}"));
                    None
                }
            }
        } else {
            None
        };
        let worker = state.clone();
        let publisher = std::thread::spawn(move || {
            let mut last_publish = Instant::now();
            let mut last_history = Instant::now();
            while !worker.publisher_stop.load(Ordering::Acquire) {
                if worker.directory.join("stop.request").is_file() {
                    worker.stop_requested.store(true, Ordering::Release);
                    worker.cancel.cancel();
                    for (_, source) in worker
                        .owners
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .iter()
                    {
                        source.cancel();
                    }
                    for (_, control) in worker
                        .math_controls
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .iter()
                    {
                        control.cancel();
                    }
                    worker.context.boolean("audit.stop_requested", true);
                }
                if last_publish.elapsed() >= Duration::from_secs(1) {
                    if let Err(error) = process_diagnostics().publish() {
                        worker.error(error);
                    }
                    last_publish = Instant::now();
                }
                if last_history.elapsed() >= Duration::from_secs(15) {
                    worker.snapshot("live");
                    last_history = Instant::now();
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        });
        Self {
            operation: Some(operation),
            context: Some(state.context.clone()),
            state: Some(state),
            publisher: Some(publisher),
            detail: install_recording(detail),
            capture,
            completed: false,
        }
    }
    pub(crate) fn enter(&self) -> Option<pse_diagnostics::observation::EnterGuard> {
        self.context.as_ref().map(ObservationContext::enter)
    }
    pub(crate) fn complete(&mut self) {
        checkpoint();
        self.completed = true;
    }
}
impl Drop for Audit {
    fn drop(&mut self) {
        let Some(state) = &self.state else {
            if let Some(detail) = self.detail.take().and_then(take_recording) {
                let _ = if self.completed && !std::thread::panicking() {
                    detail.finish()
                } else {
                    detail.stop()
                };
            }
            if let Some(operation) = self.operation.take() {
                operation.finish(if self.completed && !std::thread::panicking() {
                    Outcome::Completed
                } else {
                    Outcome::Failed
                });
            }
            return;
        };
        state.publisher_stop.store(true, Ordering::Release);
        if let Some(publisher) = self.publisher.take()
            && publisher.join().is_err()
        {
            state.error("audit publisher panicked");
        }
        let interrupted = state.stop_requested.load(Ordering::Acquire);
        let status = if state.cleanup_failures.load(Ordering::Relaxed) > 0
            || state.receiver_drain_failures.load(Ordering::Relaxed) > 0
        {
            "failed"
        } else if interrupted {
            "interrupted"
        } else if self.completed && !std::thread::panicking() {
            "completed"
        } else {
            "failed"
        };
        if let Some(operation) = self.operation.take() {
            operation.finish(match status {
                "completed" => Outcome::Completed,
                "interrupted" => Outcome::Interrupted,
                _ => Outcome::Failed,
            });
        }
        if let Some(detail) = self.detail.take().and_then(take_recording) {
            let result = if std::thread::panicking() {
                detail.stop()
            } else {
                detail.finish()
            };
            match result {
                Ok(document) => {
                    if let Err(error) = write(&state.directory, "detail.json", &json!(document)) {
                        state.error(error);
                    }
                }
                Err(error) => state.error(error),
            }
        }
        state.snapshot("terminal");
        if let Some(capture) = self.capture.take() {
            match capture.stop() {
                Ok(document) => {
                    if let Err(error) = write(
                        &state.directory,
                        "capture-from-start.json",
                        &json!(document),
                    ) {
                        state.error(error);
                    }
                }
                Err(error) => state.error(format!("timeline capture join: {error}")),
            }
        }
        // Terminal always has its own file even when bounded intermediate retention filled.
        let document = process_diagnostics().inspect(InspectionRequest::new(state.context.id()));
        if let Err(error) = write(
            &state.directory,
            "terminal-inspection.json",
            &json!(document),
        ) {
            state.error(error);
        }
        let errors = state
            .errors
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let outcome = json!({"version":1,"status":status,"operation":state.context.id().to_string(),
            "stop_requested":interrupted,"fixture_cleanups_completed":state.cleanups.load(Ordering::Relaxed),
            "fixture_cleanup_failures":state.cleanup_failures.load(Ordering::Relaxed),
            "native_joins_completed":state.joins.load(Ordering::Relaxed),
            "managed_receiver_drains_completed":state.receiver_drains.load(Ordering::Relaxed),
            "managed_receiver_drain_failures":state.receiver_drain_failures.load(Ordering::Relaxed),
            "inspections_omitted":state.omitted_inspections.load(Ordering::Relaxed),
            "diagnostic_errors":*errors,
            "cleanup_scope":"counts record explicit successful fixture cleanup and terminal native wait only; process exit is not drain evidence"});
        if let Err(error) = write(&state.directory, "audit-outcome.json", &outcome) {
            let _ = std::io::Write::write_all(
                &mut std::io::stderr(),
                format!("audit outcome write failed: {error}\n").as_bytes(),
            );
        }
    }
}
/// An entered phase, with retained observations at both actual boundaries.
#[derive(Debug)]
pub(crate) struct Stage {
    phase: Option<PhaseGuard>,
    name: String,
    started: Instant,
}
pub(crate) fn stage(name: &str) -> Stage {
    let phase = state().map(|state| {
        state.context.text("audit.stage", name);
        let phase = state.context.phase(format!("audit.{name}"));
        state.snapshot(&format!("{name}:start"));
        phase
    });
    Stage {
        phase,
        name: name.to_owned(),
        started: Instant::now(),
    }
}
impl Drop for Stage {
    fn drop(&mut self) {
        drop(self.phase.take());
        if let Some(state) = state() {
            state
                .context
                .duration(&format!("audit.{}", self.name), self.started.elapsed());
            state.snapshot(&format!("{}:end", self.name));
        }
    }
}
pub(crate) fn enabled() -> bool {
    state().is_some()
}
/// Discover this fixture's real external receiver without assuming caller placement.
pub(crate) fn managed_target(runtime: &pse_runtime::workflow::Runtime) {
    if let Some(state) = state() {
        let store = runtime.canonical().unwrap().store();
        let selected = match pse_runtime::workflow::NativeExecutionSelection::from_store(store) {
            Ok(selected) => selected,
            Err(error) => {
                state.error(format!("managed target unavailable: {error}"));
                return;
            }
        };
        if let Err(error) = write(
            &state.directory,
            "managed-target.json",
            &json!({
                "state": selected.receiver_state(), "database": store.database(),
                "caller": {
                    "checkpoint": runtime.observations().checkpoint_path(),
                    "identity": runtime.observations().checkpoint_identity(),
                },
                "receiver": null,
                "pending": "actual ready handoff is published by its consuming responsibility",
                "scope": "exact fixture receiver association; not caller-process counters"
            }),
        ) {
            state.error(error);
        }
    }
}
pub(crate) fn cancel_source() -> CancelSource {
    state().map_or_else(CancelSource::new, |state| state.cancel.clone())
}
pub(crate) fn stop_requested() -> bool {
    state().is_some_and(|state| state.stop_requested.load(Ordering::Acquire))
}
pub(crate) fn checkpoint() {
    if let Some(state) = state() {
        assert!(
            !state.stop_requested.load(Ordering::Acquire),
            "explicit audit stop requested; native work drained before interruption"
        );
    }
}
pub(crate) async fn scope<T>(future: impl Future<Output = T>) -> T {
    let context = state()
        .map(|state| state.context.clone())
        .or_else(|| SOURCE_CONTEXT.get().cloned());
    pse_diagnostics::observation::scope(context, future).await
}
#[derive(Debug)]
pub(crate) struct OwnerBinding {
    id: u64,
}
pub(crate) fn bind_owner(owner: &crate::WorkflowRuntime) -> Option<OwnerBinding> {
    let state = state()?;
    let id = state.next_owner.fetch_add(1, Ordering::Relaxed);
    if state.stop_requested.load(Ordering::Acquire) {
        owner.cancellation.cancel();
    }
    state
        .owners
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .push((id, owner.cancellation.clone()));
    Some(OwnerBinding { id })
}
impl Drop for OwnerBinding {
    fn drop(&mut self) {
        if let Some(state) = state() {
            state
                .owners
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .retain(|(id, _)| *id != self.id);
        }
    }
}
/// Register a direct synchronous mathematical owner with the same real stop request.
/// Its caller retains the control until the operation returns; no worker is detached.
#[derive(Debug)]
pub(crate) struct MathBinding {
    id: u64,
}
pub(crate) fn bind_math_control(control: &pse_diagnostics::Cancellation) -> Option<MathBinding> {
    let state = state()?;
    let id = state.next_owner.fetch_add(1, Ordering::Relaxed);
    let mut controls = state
        .math_controls
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if state.stop_requested.load(Ordering::Acquire) {
        control.cancel();
    }
    controls.push((id, control.clone()));
    Some(MathBinding { id })
}
impl Drop for MathBinding {
    fn drop(&mut self) {
        if let Some(state) = state() {
            state
                .math_controls
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .retain(|(id, _)| *id != self.id);
        }
    }
}
/// Keep an external owner (such as the managed receiver) outside an unwind boundary.
pub(crate) async fn caught<T>(future: impl Future<Output = T>) -> std::thread::Result<T> {
    if enabled() {
        std::panic::AssertUnwindSafe(scope(future))
            .catch_unwind()
            .await
    } else {
        Ok(future.await)
    }
}
pub(crate) fn receiver_drain_result(completed: bool) {
    if let Some(state) = state() {
        if completed {
            state.receiver_drains.fetch_add(1, Ordering::Relaxed);
        } else {
            state
                .receiver_drain_failures
                .fetch_add(1, Ordering::Relaxed);
        }
    }
}
/// Catch only to run the owning fixture's cleanup; scientific panics are resumed unchanged.
pub(crate) async fn with_owner<T>(
    owner: &crate::WorkflowRuntime,
    future: impl Future<Output = T>,
) -> T {
    if !enabled() {
        return future.await;
    }
    let _binding = bind_owner(owner);
    let result = std::panic::AssertUnwindSafe(scope(future))
        .catch_unwind()
        .await;
    match result {
        Ok(value) => value,
        Err(panic) => {
            let cleanup = owner.cleanup_fixtures().await;
            cleanup_result(cleanup.is_ok());
            if let Err(error) = cleanup
                && let Some(state) = state()
            {
                state.error(format!("fixture cleanup after failure: {error}"));
            }
            std::panic::resume_unwind(panic)
        }
    }
}
pub(crate) fn cleanup_result(completed: bool) {
    if let Some(state) = state() {
        if completed {
            state.cleanups.fetch_add(1, Ordering::Relaxed);
        } else {
            state.cleanup_failures.fetch_add(1, Ordering::Relaxed);
        }
    }
}
/// Synchronous driver boundary; nested block_on calls stay outside a running Tokio future.
pub(crate) fn with_owner_sync<T>(
    owner: &crate::WorkflowRuntime,
    executor: &tokio::runtime::Runtime,
    body: impl FnOnce() -> T,
) -> T {
    if !enabled() {
        return body();
    }
    let _binding = bind_owner(owner);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(body));
    match result {
        Ok(value) => value,
        Err(panic) => {
            let cleanup = executor.block_on(owner.cleanup_fixtures());
            cleanup_result(cleanup.is_ok());
            if let Err(error) = cleanup
                && let Some(state) = state()
            {
                state.error(format!("fixture cleanup after failure: {error}"));
            }
            std::panic::resume_unwind(panic)
        }
    }
}
/// Own this handle only through its actual wait; cancellation is followed by terminal join.
pub(crate) async fn wait(
    handle: pse_runtime::workflow::RunHandle,
) -> Result<Arc<pse_runtime::workflow::RunResult>, pse_runtime::workflow::WorkflowError> {
    let result = wait_ref(&handle).await;
    drop(handle);
    result
}
/// Borrow an already-owned handle without retaining a second scientific lease.
pub(crate) async fn wait_ref(
    handle: &pse_runtime::workflow::RunHandle,
) -> Result<Arc<pse_runtime::workflow::RunResult>, pse_runtime::workflow::WorkflowError> {
    let result = if let Some(state) = state() {
        tokio::select! {
            result = handle.wait() => result,
            () = state.cancel.cancelled() => { handle.cancel(); handle.wait().await }
        }
    } else {
        handle.wait().await
    };
    if let Ok(result) = &result {
        retain_result(result);
        if let Some(state) = state() {
            state.joins.fetch_add(1, Ordering::Relaxed);
        }
    }
    // Return the joined original before any stop checkpoint can unwind its caller.
    // The caller may still need to copy context/results before fixture teardown.
    result
}

/// Account only for an external native scope that has already returned joined.
/// This creates no workflow identity or cancellation authority.
pub(crate) fn direct_joined() {
    if let Some(state) = state() {
        state.joins.fetch_add(1, Ordering::Relaxed);
    }
}

/// Direct mathematical handles retain only their cancellation signal while finish owns join.
pub(crate) async fn finish<T>(
    handle: pse_runtime::math::solves::SolveHandle<T>,
) -> Result<T, pse_runtime::math::MathRuntimeError> {
    let cancellation = handle.cancellation();
    let finish = handle.finish();
    tokio::pin!(finish);
    let result = if let Some(state) = state() {
        tokio::select! {
            result = &mut finish => result,
            () = state.cancel.cancelled() => { cancellation.cancel(); finish.await }
        }
    } else {
        finish.await
    };
    if result.is_ok()
        && let Some(state) = state()
    {
        state.joins.fetch_add(1, Ordering::Relaxed);
    }
    // Standalone callers must first retain the actual joined report/context.
    result
}
