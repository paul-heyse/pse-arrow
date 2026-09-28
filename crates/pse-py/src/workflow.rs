// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Mechanical public workflow projection; all mathematical policy remains native.
mod modeling;
mod routes;
mod settings;
pub(crate) use routes::{NativeEligibility, NativeIneligible, NativeRoute};
pub(crate) use modeling::{NativeModelingNativeAnalysis, NativeModelingNonlinearExplanation, NativeModelingElasticAttempt, ModelingLimits, ModelingFixturePolicy, ModelingEventSettings, ModelingModeSettings, ModelingDiagnosticSettings, NativeModelingDiagnosticSamples, NativeModelingDiagnostics, NativeModelingTrajectory, NativeModelingConformance, NativeModelingInitialization, NativeModelingInitializationAttempt, NativeModelingStudy, NativeModelingPackage, NativeModelingResult};
mod strategies;
pub(crate) use strategies::{
    NativeAttempt, NativePreparedFlow, NativePreparedStrategy, NativeStrategyAttempt,
    NativeStrategyResult,
};
use crate::inspection::{self, errors, runtime};
use pse_runtime::{CancelSource, workflow as native};
use pyo3::prelude::*;

use std::{
    future::Future,
    sync::{Arc, Mutex},
    time::Duration,
};

fn invalid(py: Python<'_>, message: impl Into<String>) -> PyErr {
    errors::diagnostic(py, &native::WorkflowError::Contract(message.into()))
}
fn id(py: Python<'_>, value: &str) -> PyResult<pse_ids::SemanticId> {
    pse_ids::SemanticId::parse_hex(value).map_err(|e| invalid(py, e.to_string()))
}
/// Detach while waiting, periodically handle Python signals, and retain native ownership
/// through cancellation and join before propagating a signal exception.
fn blocking<T: Send, F: Future<Output = Result<T, native::WorkflowError>> + Send>(
    py: Python<'_>,
    runtime: &runtime::Runtime,
    future: F,
    cancel: impl Fn(),
) -> PyResult<T> {
    blocking_on(py, runtime.executor, future, cancel)
}
fn blocking_on<T: Send, F: Future<Output = Result<T, native::WorkflowError>> + Send>(
    py: Python<'_>, executor: &tokio::runtime::Runtime, future: F, cancel: impl Fn(),
) -> PyResult<T> {
    if tokio::runtime::Handle::try_current().is_ok() {
        return Err(invalid(
            py,
            "blocking workflow cannot re-enter its async executor",
        ));
    }
    let mut future = Box::pin(future);
    let mut signal = None;
    loop {
        let ready = py.detach(|| {
            executor.block_on(async {
                tokio::time::timeout(Duration::from_millis(100), &mut future).await
            })
        });
        if let Ok(value) = ready {
            return match signal {
                Some(e) => Err(e),
                None => value.map_err(|e| errors::diagnostic(py, &e)),
            };
        }
        if signal.is_none()
            && let Err(e) = py.check_signals()
        {
            cancel();
            signal = Some(e);
        }
    }
}
/// The PostgreSQL operational store a durable runtime registers its runs in (ADR-0114).
/// `url` defaults to `PSE_DATABASE_URL`, else the development default (the local socket
/// with peer authentication); `worker` names the lease owner and defaults to this process.
/// Nothing connects until a runtime is created with it.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct OperationalStore {
    url: String,
    worker: String,
}
#[pymethods]
impl OperationalStore {
    #[new]
    #[pyo3(signature = (url=None, *, worker=None))]
    fn new(url: Option<String>, worker: Option<String>) -> Self {
        Self {
            url: url.unwrap_or_else(native::database_url_from_env),
            worker: worker.unwrap_or_else(|| native::Operations::process_worker("python")),
        }
    }
    /// Connection URL.
    #[getter]
    fn url(&self) -> &str {
        &self.url
    }
    /// The worker identity that owns this process's leases.
    #[getter]
    fn worker(&self) -> &str {
        &self.worker
    }
    fn __repr__(&self) -> String {
        format!("OperationalStore(worker={:?})", self.worker)
    }
}
/// Borrow the same budget, services and executor as exact publication inspection.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeRuntime {
    owner: Arc<runtime::Runtime>,
    inner: native::Runtime,
}
#[pymethods]
impl NativeRuntime {
    fn modeling_from_documents(
        &self,
        py: Python<'_>,
        documents: Vec<std::collections::BTreeMap<String, String>>,
        physical: &NativePhysicalContext,
    ) -> PyResult<NativeModelingPackage> {
        modeling::from_documents(self, py, documents, physical)
    }
    fn prepare_conic(
        &self,
        py: Python<'_>,
        request: &[u8],
        physical: &NativePhysicalContext,
        settings: &[u8],
    ) -> PyResult<NativePreparedStrategy> {
        let settings = settings::solve_profile(py, settings)?;
        if request.len() > self.owner.shared.budget().math.workspace_bytes / 4 {
            return Err(invalid(py, "cone request exceeds workspace allowance"));
        }
        let request =
            serde_json::from_slice::<strategies::AnalysisDocument<native::ConicRequest>>(request)
                .map_err(|e| invalid(py, e.to_string()))?
                .payload;
        let inner = blocking(
            py,
            &self.owner,
            self.inner
                .prepare_conic(request, &physical.inner, settings.clone()),
            || {},
        )?;
        Ok(NativePreparedStrategy {
            owner: self.owner.clone(),
            inner: strategies::Strategy::Cone(inner),
        })
    }
    /// A runtime over the shared deployment. With `store`, every run is a durable attempt
    /// registered in that operational store and may be published; without it runs are
    /// ephemeral and cannot publish (ADR-0114 Outcome 16).
    #[new]
    #[pyo3(signature = (settings, *, store=None))]
    fn new(
        py: Python<'_>,
        settings: &inspection::EngineSettings,
        store: Option<&OperationalStore>,
    ) -> PyResult<Self> {
        let owner = py
            .detach(|| runtime::acquire(settings))
            .map_err(|e| errors::diagnostic(py, &e))?;
        let inner = native::Runtime::from_shared(
            owner.shared.clone(),
            owner.registry.clone(),
            owner.sessions.clone(),
        );
        let inner = match store {
            None => inner,
            Some(store) => {
                let operations = blocking(
                    py,
                    &owner,
                    native::Operations::connect(
                        &store.url,
                        store.worker.clone(),
                        native::LeasePolicy::default(),
                    ),
                    || {},
                )?;
                inner.with_durability(native::Durability::Durable(operations))
            }
        };
        Ok(Self { owner, inner })
    }
    /// Whether runs are durable attempts in an operational store.
    #[getter]
    fn durable(&self) -> bool {
        matches!(self.inner.durability(), native::Durability::Durable(_))
    }
    /// Durable attempts of the store, newest first, as `runtime.operational_attempts`:
    /// optionally those of one run and in the given registry `AttemptState` names.
    #[pyo3(signature = (*, run_id=None, states=Vec::new(), limit=100))]
    fn runs(
        &self,
        py: Python<'_>,
        run_id: Option<&str>,
        states: Vec<String>,
        limit: i64,
    ) -> PyResult<inspection::TableStream> {
        let filter = native::AttemptFilter {
            run: run_id.map(|r| id(py, r).map(Into::into)).transpose()?,
            states: states
                .iter()
                .map(|s| settings::named(py, "attempt state", s))
                .collect::<PyResult<_>>()?,
            limit,
        };
        let batch = blocking(py, &self.owner, self.inner.runs(&filter), || {})?;
        Ok(inspection::TableStream::from_batch(batch))
    }
    fn physical_from_documents(
        &self,
        py: Python<'_>,
        documents: std::collections::BTreeMap<String, String>,
    ) -> PyResult<NativePhysicalContext> {
        let cancel = CancelSource::new();
        let inner = blocking(
            py,
            &self.owner,
            async {
                let pool = self.owner.shared.pool();
                let token = cancel.token();
                let bundle = pse_runtime::authoring_driver::document::load_package_texts_owned(
                    &documents,
                    &self.owner.registry,
                    pse_authoring::ParseBudget::default(),
                    &pool,
                    &token,
                )?;
                let owned =
                    pse_runtime::authoring_driver::document::OwnedDocumentSet::try_from_bundles(
                        vec![bundle],
                        &pool,
                        &token,
                    )?;
                self.inner.physical_from_documents(&owned, &token).await
            },
            || cancel.cancel(),
        )?;
        Ok(NativePhysicalContext { inner })
    }
    fn capabilities(&self, py: Python<'_>) -> PyResult<Vec<u8>> {
        serde_json::to_vec(&self.inner.capabilities()).map_err(|e| invalid(py, e.to_string()))
    }
    fn clear_program_cache(&self) {
        self.inner.clear_program_cache();
    }
    fn settle_publication(&self, py: Python<'_>, ticket: &[u8]) -> PyResult<Vec<u8>> {
        if ticket.len() > self.owner.shared.budget().math.workspace_bytes / 4 {
            return Err(invalid(py, "publication ticket exceeds input allowance"));
        }
        let ticket: native::PublicationTicket =
            serde_json::from_slice(ticket).map_err(|e| invalid(py, e.to_string()))?;
        let cancel = pse_columnar::CancellationToken::new();
        let result = blocking(
            py,
            &self.owner,
            async {
                Ok::<_, native::WorkflowError>(
                    self.inner.settle_publication(&ticket, &cancel).await,
                )
            },
            || cancel.cancel(),
        )?;
        serde_json::to_vec(&result).map_err(|e| invalid(py, e.to_string()))
    }
    #[pyo3(signature=(steps, *, continue_independent=false))]
    fn start(
        &self, py: Python<'_>, steps: Vec<PyRef<'_,NativePreparedOperation>>, continue_independent: bool,
    ) -> PyResult<NativeRunHandle> {
        let steps=steps.into_iter().map(|p|match &p.inner {
            PreparedOperation::Modeling(step)=>Ok(step.as_ref().clone()),
            _=>Err(invalid(py,"finite solve sequences require authored algebraic cases")),
        }).collect::<PyResult<Vec<_>>>()?;
        let cancel=CancelSource::new();
        let inner=blocking(py,&self.owner,self.inner.start_modeling(steps,continue_independent,&cancel),||cancel.cancel())?;
        Ok(NativeRunHandle{owner:self.owner.clone(),inner})
    }

}
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeStart {
    inner: pse_backend_native::solve::WarmStart,
}
#[pymethods]
impl NativeStart {
    fn snapshot_json(&self) -> String {
        self.inner.snapshot().to_string()
    }
}
/// Public handle remains usable after any individual asyncio waiter is cancelled.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeRunHandle {
    owner: Arc<runtime::Runtime>,
    inner: native::RunHandle,
}
struct CancelWait(Option<native::RunHandle>);
impl Drop for CancelWait {
    fn drop(&mut self) {
        if let Some(h) = &self.0 {
            h.cancel();
        }
    }
}
#[pymethods]
impl NativeRunHandle {
    fn cancel(&self) {
        self.inner.cancel();
    }
    fn result(&self) -> Option<NativeRunResult> {
        self.inner.result().map(|inner| NativeRunResult {
            owner: self.owner.clone(),
            inner,
        })
    }
    fn wait(&self, py: Python<'_>) -> PyResult<NativeRunResult> {
        let inner = blocking(py, &self.owner, self.inner.wait(), || self.inner.cancel())?;
        Ok(NativeRunResult {
            owner: self.owner.clone(),
            inner,
        })
    }
    #[pyo3(signature=() -> "typing.Awaitable[NativeRunResult]")]
    fn wait_async<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let locals = pyo3_async_runtimes::tokio::get_current_locals(py)?;
        let handle = self.inner.clone();
        let owner = self.owner.clone();
        let guard = CancelWait(Some(handle.clone()));
        pyo3_async_runtimes::tokio::future_into_py_with_locals(py, locals, async move {
            let mut guard = guard;
            let value = handle.wait().await;
            guard.0.take();
            Python::attach(|py| {
                let inner = value.map_err(|e| errors::diagnostic(py, &e))?;
                Py::new(py, NativeRunResult { owner, inner })
            })
        })
    }
    /// The durable attempt of this run, minted before any effect; `None` when ephemeral.
    #[getter]
    fn attempt_id(&self) -> Option<String> {
        self.inner.attempt_id().map(|id| id.to_string())
    }
    fn progress(&self) -> (Vec<ProgressEvent>, u64) {
        let (events, dropped) = self.inner.progress();
        (events.into_iter().map(ProgressEvent).collect(), dropped)
    }
    #[getter]
    fn progress_count(&self) -> (usize, u64) {
        let (events, dropped) = self.inner.progress();
        (events.len(), dropped)
    }
}
/// Joined immutable original-space reports with retained Arrow export.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeRunResult {
    owner: Arc<runtime::Runtime>,
    inner: Arc<native::RunResult>,
}
#[pymethods]
impl NativeRunResult {
    #[pyo3(signature=(step=0))]
    fn available_start(&self, step: usize) -> Option<NativeStart> {
        match self.inner.report().ok()? {
            native::RunReport::Modeling(r) => match &r.get(step)?.outcome {
                pse_runtime::math::solves::Outcome::Native(r)=>r.warm_start.clone().map(|inner|NativeStart{inner}),
                _=>None,
            },
            native::RunReport::Fit(r) if step == 0 => r
                .solve
                .as_ref()?
                .warm_start
                .clone()
                .map(|inner| NativeStart { inner }),
            _ => None,
        }
    }
    #[getter]
    fn usable(&self) -> bool {
        self.inner.usable()
    }
    #[getter]
    fn run_id(&self) -> String {
        self.inner.run_id.to_hex()
    }
    /// The durable attempt that recorded this run; `None` when the run is ephemeral.
    #[getter]
    fn attempt_id(&self) -> Option<String> {
        match self.inner.durability() {
            native::RunDurability::Ephemeral => None,
            native::RunDurability::Durable(record) => Some(record.attempt_id.to_string()),
        }
    }
    fn completion(&self, py: Python<'_>) -> PyResult<Vec<u8>> {
        let completed = self
            .inner
            .completion()
            .map_err(|e| errors::diagnostic(py, e))?;
        serde_json::to_vec(completed).map_err(|e| invalid(py, e.to_string()))
    }
    fn diagnostics(&self) -> Vec<inspection::DiagnosticReport> {
        match self.inner.completion() {
            Ok(completed) => completed
                .diagnostics
                .iter()
                .map(|d| inspection::DiagnosticReport::observe(d))
                .collect(),
            Err(e) => vec![inspection::DiagnosticReport::observe(e)],
        }
    }
    fn tables(&self, py: Python<'_>) -> PyResult<Vec<String>> {
        py.detach(|| {
            self.inner.tables().map(|tables| {
                tables
                    .keys()
                    .filter_map(|id| {
                        self.owner
                            .registry
                            .relation_by_id(*id)
                            .map(|s| format!("{}.{}", s.key.namespace.as_str(), s.key.name))
                    })
                    .collect()
            })
        })
        .map_err(|e| errors::diagnostic(py, e.as_ref()))
    }
    fn table(&self, py: Python<'_>, name: &str) -> PyResult<inspection::TableStream> {
        py.detach(|| self.inner.table(name))
            .map(inspection::TableStream::from_batch)
            .map_err(|e| errors::diagnostic(py, e.as_ref()))
    }
    #[pyo3(signature=(base, workspace_id, *, parent=None, publication_id=None, attempt_id=None))]
    fn prepare_publication(
        &self,
        py: Python<'_>,
        base: &str,
        workspace_id: &str,
        parent: Option<&str>,
        publication_id: Option<&str>,
        attempt_id: Option<&str>,
    ) -> PyResult<NativePublicationAttempt> {
        let base = url::Url::parse(base).map_err(|e| invalid(py, e.to_string()))?;
        let workspace_id = id(py, workspace_id)?;
        let parent = parent.map(|p| id(py, p)).transpose()?;
        let request = match (publication_id, attempt_id) {
            (None, None) => native::PublicationRequest::new(base, workspace_id, parent),
            (Some(publication), Some(attempt)) => native::PublicationRequest {
                base,
                workspace_id,
                parent,
                publication_id: id(py, publication)?,
                attempt_id: id(py, attempt)?,
            },
            _ => {
                return Err(invalid(
                    py,
                    "publication_id and attempt_id must be supplied together",
                ));
            }
        };
        let attempt = py
            .detach(|| {
                self.inner
                    .prepare_publication_request(request, &pse_columnar::CancellationToken::new())
            })
            .map_err(|e| errors::diagnostic(py, &e))?;
        Ok(NativePublicationAttempt {
            owner: self.owner.clone(),
            attempt_id: attempt.attempt_id.to_hex(),
            publication_id: attempt.publication_id.to_hex(),
            ticket: serde_json::to_vec(&attempt.ticket).map_err(|e| invalid(py, e.to_string()))?,
            inner: Mutex::new(Some(attempt)),
        })
    }
}
/// Reviewable single-consumption native publication command; commit never reruns a solve.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Debug)]
pub(crate) struct NativePublicationAttempt {
    owner: Arc<runtime::Runtime>,
    inner: Mutex<Option<native::PublicationAttempt>>,
    #[pyo3(get)]
    attempt_id: String,
    #[pyo3(get)]
    publication_id: String,
    ticket: Vec<u8>,
}
#[pymethods]
impl NativePublicationAttempt {
    fn ticket(&self) -> Vec<u8> {
        self.ticket.clone()
    }
    fn commit(&self, py: Python<'_>) -> PyResult<(String, i64)> {
        let attempt = self
            .inner
            .lock()
            .map_err(|_| invalid(py, "publication attempt lock poisoned"))?
            .take()
            .ok_or_else(|| {
                invalid(
                    py,
                    "publication attempt already consumed; inspect native settlement before retry",
                )
            })?;
        let cancel = pse_columnar::CancellationToken::new();
        let root = blocking(py, &self.owner, attempt.commit(&cancel), || cancel.cancel())?;
        Ok((root.location.to_string(), root.version))
    }
}

/// Actual admitted physical source rows and compiler context.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativePhysicalContext {
    inner: native::PhysicalContext,
}
#[pymethods]
impl NativePhysicalContext {
    #[getter]
    fn identity(&self) -> String {
        self.inner.identity().to_prefixed()
    }
}

/// One bounded, owned native progress event. No native thread enters Python.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct ProgressEvent(pse_backend_native::solve::Event);
#[pymethods]
impl ProgressEvent {
    #[getter]
    fn phase(&self) -> &str {
        &self.0.phase
    }
    #[getter]
    fn elapsed_seconds(&self) -> f64 {
        self.0.elapsed.as_secs_f64()
    }
    #[pyo3(signature=() -> "dict[str, bool | int | float | str | dict[str, str]]")]
    fn values<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        use pse_backend_native::solve::Metric;
        let dict = pyo3::types::PyDict::new(py);
        for (k, v) in &self.0.values {
            match v {
                Metric::Real(v) => dict.set_item(k, v)?,
                Metric::Integer(v) => dict.set_item(k, v)?,
                Metric::Bool(v) => dict.set_item(k, v)?,
                Metric::Text(v) => dict.set_item(k, v)?,
                Metric::Unavailable(reason) => {
                    let unavailable = pyo3::types::PyDict::new(py);
                    unavailable.set_item("kind", v.kind().as_str())?;
                    unavailable.set_item("reason", reason.as_str())?;
                    dict.set_item(k, unavailable)?;
                }
            }
        }
        Ok(dict)
    }
}

/// Typed finite integration controls; native options serialize losslessly at the boundary.
/// Every optional argument takes the native profile default when omitted.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct SimulationSettings {
    profile: native::SimulationProfile,
}
#[pymethods]
impl SimulationSettings {
    #[new]
    #[expect(
        clippy::too_many_arguments,
        reason = "mechanical keyword-only projection of native integration controls"
    )]
    #[pyo3(signature=(*, start, end, samples, atol, parameter_scales, sensitivities=None, rtol=None, out_rtol=None, out_atol=None, initial_step=None, max_steps: "int | None"=None, max_events: "int | None"=None, time_limit=None, max_cells: "int | None"=None, method=None, trial_failures=None, numerics: "dict[str, object] | None"=None, diffsol=None, idas=None))]
    fn new(
        py: Python<'_>,
        start: f64,
        end: f64,
        samples: Vec<f64>,
        atol: Vec<f64>,
        parameter_scales: Vec<f64>,
        sensitivities: Option<bool>,
        rtol: Option<f64>,
        out_rtol: Option<f64>,
        out_atol: Option<Vec<f64>>,
        initial_step: Option<f64>,
        #[pyo3(from_py_with = inspection::inputs::extract)] max_steps: Option<usize>,
        #[pyo3(from_py_with = inspection::inputs::extract)] max_events: Option<usize>,
        time_limit: Option<f64>,
        #[pyo3(from_py_with = inspection::inputs::extract)] max_cells: Option<usize>,
        method: Option<&str>,
        trial_failures: Option<&str>,
        numerics: Option<&Bound<'_, pyo3::types::PyDict>>,
        diffsol: Option<&[u8]>,
        idas: Option<&[u8]>,
    ) -> PyResult<Self> {
        let mut profile = native::SimulationProfile {
            start,
            end,
            samples,
            atol,
            parameter_scales,
            out_rtol,
            numerics: settings::numerical_policy(py, numerics)?,
            ..Default::default()
        };
        if let Some(sensitivities) = sensitivities {
            profile.sensitivities = sensitivities;
        }
        if let Some(rtol) = rtol {
            profile.rtol = rtol;
        }
        if let Some(out_atol) = out_atol {
            profile.out_atol = out_atol;
        }
        if let Some(initial_step) = initial_step {
            profile.initial_step = initial_step;
        }
        if let Some(max_steps) = max_steps {
            profile.max_steps = max_steps;
        }
        if let Some(max_events) = max_events {
            profile.max_events = max_events;
        }
        if let Some(time_limit) = time_limit {
            profile.time_limit =
                Duration::try_from_secs_f64(time_limit).map_err(|e| invalid(py, e.to_string()))?;
        }
        if let Some(max_cells) = max_cells {
            profile.max_cells = max_cells;
        }
        if let Some(method) = method {
            profile.method = settings::named(py, "dynamics method", method)?;
        }
        if let Some(trial_failures) = trial_failures {
            profile.trial_failures = settings::named(py, "trial failure policy", trial_failures)?;
        }
        if let Some(diffsol) = diffsol {
            profile.diffsol = settings::diffsol(py, diffsol)?;
        }
        if let Some(idas) = idas {
            profile.idas = settings::idas(py, idas)?;
        }
        Ok(Self { profile })
    }
    /// The encoded Diffsol settings document in effect.
    #[getter]
    fn diffsol(&self, py: Python<'_>) -> PyResult<Vec<u8>> {
        serde_json::to_vec(&self.profile.diffsol).map_err(|e| invalid(py, e.to_string()))
    }
    /// The encoded IDAS settings document in effect.
    #[getter]
    fn idas(&self, py: Python<'_>) -> PyResult<Vec<u8>> {
        serde_json::to_vec(&self.profile.idas).map_err(|e| invalid(py, e.to_string()))
    }
    /// Round-trip the complete pinned native BDF and initialization options.
    fn to_json(&self, py: Python<'_>) -> PyResult<String> {
        serde_json::to_string(&self.profile).map_err(|e| invalid(py, e.to_string()))
    }
    #[staticmethod]
    fn from_json(py: Python<'_>, source: &str) -> PyResult<Self> {
        if source.len() > 1 << 20 {
            return Err(invalid(py, "simulation settings extent"));
        }
        serde_json::from_str(source)
            .map(|profile| Self { profile })
            .map_err(|e| invalid(py, e.to_string()))
    }
}
#[derive(Clone, Debug)]
enum PreparedOperation {
    Modeling(Box<native::ModelingSolvePreparation>),
    Simulation(Box<native::ModelingSimulation>),
    Fit(native::PreparedFit),
}
/// Immutable simulation/fitting view of the same owned job and Arrow result lifecycle.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativePreparedOperation {
    owner: Arc<runtime::Runtime>,
    inner: PreparedOperation,
}
#[pymethods]
impl NativePreparedOperation {
    /// Admitted algebraic route, before native execution.
    #[getter]
    fn route(&self, py: Python<'_>) -> PyResult<NativeRoute> {
        let PreparedOperation::Modeling(p)=&self.inner else {return Err(invalid(py,"algebraic route inspection requires an algebraic solve"));};
        Ok(p.solve.route().into())
    }
    /// Typed eligibility row of every assessed adapter.
    #[getter]
    fn eligibility(&self, py: Python<'_>) -> PyResult<Vec<NativeEligibility>> {
        let PreparedOperation::Modeling(p)=&self.inner else {return Err(invalid(py,"algebraic eligibility requires an algebraic solve"));};
        Ok(p.solve.eligibility().iter().map(Into::into).collect())
    }
    fn with_start(&self, py: Python<'_>, seed: &NativeStart) -> PyResult<Self> {
        let PreparedOperation::Modeling(p)=&self.inner else {return Err(invalid(py,"native warm starts require an algebraic solve"));};
        let inner=p.as_ref().clone().with_start(seed.inner.clone()).map_err(|e|errors::diagnostic(py,&e))?;
        Ok(Self{owner:self.owner.clone(),inner:PreparedOperation::Modeling(Box::new(inner))})
    }
    fn with_primal_start(&self, py: Python<'_>, values: std::collections::BTreeMap<String,f64>) -> PyResult<Self> {
        let PreparedOperation::Modeling(p)=&self.inner else {return Err(invalid(py,"native warm starts require an algebraic solve"));};
        let values=values.into_iter().map(|(key,value)|id(py,&key).map(|id|(id,value))).collect::<PyResult<_>>()?;
        let inner=p.as_ref().clone().with_primal_start(values).map_err(|e|errors::diagnostic(py,&e))?;
        Ok(Self{owner:self.owner.clone(),inner:PreparedOperation::Modeling(Box::new(inner))})
    }
    #[getter]
    fn identity(&self, py: Python<'_>) -> PyResult<String> {
        Ok(match &self.inner {
            PreparedOperation::Modeling(p) => p.solve.request_identity().map_err(|e|errors::diagnostic(py,&e))?,
            PreparedOperation::Simulation(s) => s.identity(),
            PreparedOperation::Fit(f) => f.identity(),
        }
        .to_prefixed())
    }
    fn start(&self, py: Python<'_>) -> PyResult<NativeRunHandle> {
        let inner = py
            .detach(|| {
                let _enter = self.owner.executor.enter();
                match &self.inner {
                    PreparedOperation::Modeling(p) => p.start(),
                    PreparedOperation::Simulation(s) => s.start(),
                    PreparedOperation::Fit(f) => f.start(),
                }
            })
            .map_err(|e| errors::diagnostic(py, &e))?;
        Ok(NativeRunHandle {
            owner: self.owner.clone(),
            inner,
        })
    }
}
