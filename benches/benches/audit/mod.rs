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
use pse_runtime::{CancelSource, inspection::InspectionRequest, observation::process_diagnostics};
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
static STATE: OnceLock<Option<Arc<State>>> = OnceLock::new();
#[derive(Debug)]
struct State {
    directory: PathBuf,
    context: ObservationContext,
    cancel: CancelSource,
    owners: Mutex<Vec<(u64, CancelSource)>>,
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
/// One pre-runtime operation. It coexists with the benchmark's tracing subscriber.
#[derive(Debug)]
pub(crate) struct Audit {
    operation: Option<OperationGuard>,
    state: Option<Arc<State>>,
    publisher: Option<JoinHandle<()>>,
    completed: bool,
}
impl Audit {
    pub(crate) fn start(case: &str, workload: &Value) -> Self {
        let Some(directory) = std::env::var_os("PSE_CASE_AUDIT_DIR").map(PathBuf::from) else {
            return Self {
                operation: None,
                state: None,
                publisher: None,
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
        let context = operation.context();
        context.source_ref("audit.case", case.as_bytes());
        context.text("audit.case", case);
        context.text("audit.workload", &workload.to_string());
        let state = Arc::new(State {
            directory,
            context,
            cancel: CancelSource::new(),
            owners: Mutex::new(Vec::new()),
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
            errors: Mutex::new(Vec::new()),
        });
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
            state: Some(state),
            publisher: Some(publisher),
            completed: false,
        }
    }
    pub(crate) fn enter(&self) -> Option<pse_diagnostics::observation::EnterGuard> {
        self.state.as_ref().map(|state| state.context.enter())
    }
    pub(crate) fn complete(&mut self) {
        checkpoint();
        self.completed = true;
    }
}
impl Drop for Audit {
    fn drop(&mut self) {
        let Some(state) = &self.state else {
            return;
        };
        state.publisher_stop.store(true, Ordering::Release);
        if let Some(publisher) = self.publisher.take() {
            if publisher.join().is_err() {
                state.error("audit publisher panicked");
            }
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
        state.snapshot("terminal");
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
    pse_diagnostics::observation::scope(state().map(|state| state.context.clone()), future).await
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
            if let Err(error) = cleanup {
                if let Some(state) = state() {
                    state.error(format!("fixture cleanup after failure: {error}"));
                }
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
            if let Err(error) = cleanup {
                if let Some(state) = state() {
                    state.error(format!("fixture cleanup after failure: {error}"));
                }
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
    if result.is_ok() {
        if let Some(state) = state() {
            state.joins.fetch_add(1, Ordering::Relaxed);
        }
    }
    checkpoint();
    result
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
    if result.is_ok() {
        if let Some(state) = state() {
            state.joins.fetch_add(1, Ordering::Relaxed);
        }
    }
    checkpoint();
    result
}
