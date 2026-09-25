// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Mechanical public workflow projection; all mathematical policy remains native.
mod settings;
mod strategies;
pub(crate) use strategies::{
    NativeAttempt, NativePreparedFlow, NativePreparedStrategy, NativeStrategyResult,
};
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ModelEnvelope {
    declaration: native::ModelDeclaration,
    #[serde(default)]
    sources: native::SourceDeclarations,
}
use crate::inspection::{self, errors, runtime};
use pse_runtime::{CancelSource, workflow as native};
use pyo3::{prelude::*, types::PyBytes};
pub(crate) use settings::SolveSettings;
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
            runtime.executor.block_on(async {
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
/// Actual linked adapter capability, distinct from eligibility of a selected case.
#[pyclass(frozen, skip_from_py_object, get_all, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct SolverCapability {
    backend: String,
    classes: Vec<String>,
    derivatives: String,
    warm: String,
    reuse: String,
    cancellation: String,
    diagnostics: String,
    general_bounds: bool,
    sign_bounds: bool,
    parallel: bool,
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
    fn prepare_conic(
        &self,
        py: Python<'_>,
        request: &[u8],
        physical: &NativePhysicalContext,
        settings: &SolveSettings,
    ) -> PyResult<NativePreparedStrategy> {
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
                .prepare_conic(request, &physical.inner, settings.profile.clone()),
            || {},
        )?;
        Ok(NativePreparedStrategy {
            owner: self.owner.clone(),
            inner: strategies::Strategy::Cone(inner),
        })
    }
    #[new]
    fn new(py: Python<'_>, settings: &inspection::EngineSettings) -> PyResult<Self> {
        let owner = py
            .detach(|| runtime::acquire(settings))
            .map_err(|e| errors::diagnostic(py, &e))?;
        let inner = native::Runtime::from_shared(
            owner.shared.clone(),
            owner.registry.clone(),
            owner.sessions.clone(),
        );
        Ok(Self { owner, inner })
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
    fn capabilities(&self) -> Vec<SolverCapability> {
        self.inner
            .capabilities()
            .into_iter()
            .map(|(b, c)| SolverCapability {
                backend: b.as_str().into(),
                classes: c.classes.iter().map(|v| v.as_str().into()).collect(),
                derivatives: c.derivatives.as_str().into(),
                warm: c.warm.as_str().into(),
                reuse: c.reuse.into(),
                cancellation: c.cancellation.into(),
                diagnostics: c.diagnostics.into(),
                general_bounds: c.general_bounds,
                sign_bounds: c.sign_bounds,
                parallel: c.parallel,
            })
            .collect()
    }
    fn model(
        &self,
        py: Python<'_>,
        declaration: &[u8],
        physical: &NativePhysicalContext,
    ) -> PyResult<NativeModelRevision> {
        if declaration.len() > self.owner.shared.budget().math.workspace_bytes / 4 {
            return Err(invalid(py, "model input exceeds workspace allowance"));
        }
        let inner = py
            .detach(|| {
                let wire = serde_json::from_slice::<ModelEnvelope>(declaration)
                    .map_err(|e| native::WorkflowError::Contract(e.to_string()))?;
                let mut draft = native::ModelBuilder::from_declaration(
                    self.inner.clone(),
                    wire.declaration,
                    physical.inner.clone(),
                );
                *draft.sources_mut() = wire.sources;
                draft.freeze()
            })
            .map_err(|e| errors::diagnostic(py, &e))?;
        Ok(NativeModelRevision {
            owner: self.owner.clone(),
            inner,
        })
    }
    fn models_from_documents(
        &self,
        py: Python<'_>,
        documents: std::collections::BTreeMap<String, String>,
        physical: &NativePhysicalContext,
    ) -> PyResult<Vec<NativeModelRevision>> {
        let rows = py
            .detach(|| {
                let cancel = pse_columnar::CancellationToken::new();
                let pool = self.owner.shared.pool();
                let bundle = pse_runtime::authoring_driver::document::load_package_texts_owned(
                    &documents,
                    &self.owner.registry,
                    pse_authoring::ParseBudget::default(),
                    &pool,
                    &cancel,
                )?;
                let documents =
                    pse_runtime::authoring_driver::document::OwnedDocumentSet::try_from_bundles(
                        vec![bundle],
                        &pool,
                        &cancel,
                    )?;
                self.inner
                    .models_from_documents(&documents, physical.inner.clone())?
                    .into_iter()
                    .map(native::ModelBuilder::freeze)
                    .collect::<Result<Vec<_>, native::WorkflowError>>()
            })
            .map_err(|e| errors::diagnostic(py, &e))?;
        Ok(rows
            .into_iter()
            .map(|inner| NativeModelRevision {
                owner: self.owner.clone(),
                inner,
            })
            .collect())
    }
    #[pyo3(signature=(steps, *, continue_independent=false))]
    fn start(
        &self,
        py: Python<'_>,
        steps: Vec<PyRef<'_, NativePreparedCase>>,
        continue_independent: bool,
    ) -> PyResult<NativeRunHandle> {
        let steps = steps.into_iter().map(|p| p.inner.clone()).collect();
        let inner = py
            .detach(|| {
                let _enter = self.owner.executor.enter();
                self.inner.start(steps, continue_independent)
            })
            .map_err(|e| errors::diagnostic(py, &e))?;
        Ok(NativeRunHandle {
            owner: self.owner.clone(),
            inner,
        })
    }
}
/// Immutable admitted typed declarations.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeModelRevision {
    owner: Arc<runtime::Runtime>,
    inner: native::ModelRevision,
}
#[pymethods]
impl NativeModelRevision {
    fn prepare_flow(
        &self,
        py: Python<'_>,
        case_id: &str,
        flow_id: &str,
    ) -> PyResult<NativePreparedFlow> {
        let (case, flow) = (id(py, case_id)?, id(py, flow_id)?);
        let inner = blocking(py, &self.owner, self.inner.prepare_flow(case, flow), || {})?;
        Ok(NativePreparedFlow {
            owner: self.owner.clone(),
            math: self.owner.shared.math().clone(),
            inner,
        })
    }
    fn prepare_recycle(
        &self,
        py: Python<'_>,
        request: &[u8],
        settings: &SolveSettings,
    ) -> PyResult<NativePreparedStrategy> {
        #[cfg(feature = "native-solvers")]
        {
            if request.len() > self.owner.shared.budget().math.workspace_bytes / 4 {
                return Err(invalid(py, "recycle request exceeds workspace allowance"));
            }
            let request = serde_json::from_slice::<
                strategies::AnalysisDocument<native::RecycleRequest>,
            >(request)
            .map_err(|e| invalid(py, e.to_string()))?
            .payload;
            let cancel = CancelSource::new();
            let inner = blocking(
                py,
                &self.owner,
                self.inner.prepare_recycle(
                    request,
                    settings.profile.clone(),
                    Default::default(),
                    &cancel,
                ),
                || cancel.cancel(),
            )?;
            Ok(NativePreparedStrategy {
                owner: self.owner.clone(),
                inner: strategies::Strategy::Recycle(inner),
            })
        }
        #[cfg(not(feature = "native-solvers"))]
        {
            let _ = (request, settings);
            Err(invalid(py, "KINSOL strategy workflow is not linked"))
        }
    }
    fn prepare_initialization(
        &self,
        py: Python<'_>,
        case_id: &str,
        settings: &SolveSettings,
        stages: Vec<std::collections::BTreeMap<String, f64>>,
    ) -> PyResult<NativePreparedStrategy> {
        #[cfg(feature = "native-solvers")]
        {
            let case = id(py, case_id)?;
            if !matches!(
                settings.profile.intent,
                pse_backend_native::solve::SolveIntent::Initialize
                    | pse_backend_native::solve::SolveIntent::Root
            ) || matches!(
                settings.profile.presolve,
                pse_backend_native::presolve::Policy::Explicit { .. }
            ) || !matches!(
                settings.profile.convexity,
                pse_runtime::math::solves::ConvexityPolicy::Exact
            ) || !matches!(
                settings.profile.backend,
                pse_runtime::math::solves::BackendSettings::Default
            ) {
                return Err(invalid(
                    py,
                    "initialization requires root/initialize intent and has no explicit preprocessing or convexity strategy",
                ));
            }
            if stages.len() > 4096 {
                return Err(invalid(py, "continuation stage allowance"));
            }
            let stages = stages
                .into_iter()
                .map(|s| {
                    s.into_iter()
                        .map(|(k, v)| id(py, &k).map(|k| (k, v)))
                        .collect::<PyResult<_>>()
                })
                .collect::<PyResult<_>>()?;
            let profile = pse_runtime::math::initialization::InitializationProfile {
                selection: settings.profile.selection,
                controls: settings.profile.controls.clone(),
                linear: pse_backend_native::kinsol::Linear::Klu,
                numerics: settings.profile.numerics.clone(),
                stages,
            };
            let inner = blocking(
                py,
                &self.owner,
                self.inner
                    .prepare_initialization(case, profile, Default::default()),
                || {},
            )?;
            Ok(NativePreparedStrategy {
                owner: self.owner.clone(),
                inner: strategies::Strategy::Initialization(inner),
            })
        }
        #[cfg(not(feature = "native-solvers"))]
        {
            let _ = (case_id, settings, stages);
            Err(invalid(py, "initialization workflow is not linked"))
        }
    }
    fn prepare_simulation(
        &self,
        py: Python<'_>,
        dynamic_id: &str,
        settings: &SimulationSettings,
    ) -> PyResult<NativePreparedOperation> {
        let id = id(py, dynamic_id)?;
        let cancel = CancelSource::new();
        let inner = blocking(
            py,
            &self.owner,
            self.inner.prepare_simulation(
                id,
                settings.profile.clone(),
                Default::default(),
                &cancel,
            ),
            || cancel.cancel(),
        )?;
        Ok(NativePreparedOperation {
            owner: self.owner.clone(),
            inner: PreparedOperation::Simulation(inner),
        })
    }
    #[pyo3(signature=(fit_id, settings, simulations, *, rank_tolerance=1e-8, max_cells=1000000))]
    fn prepare_fit(
        &self,
        py: Python<'_>,
        fit_id: &str,
        settings: &SolveSettings,
        simulations: Vec<(String, PyRef<'_, SimulationSettings>)>,
        rank_tolerance: f64,
        max_cells: usize,
    ) -> PyResult<NativePreparedOperation> {
        let fit = id(py, fit_id)?;
        let count = simulations.len();
        let simulations = simulations
            .into_iter()
            .map(|(key, v)| id(py, &key).map(|key| (key, v.profile.clone())))
            .collect::<PyResult<std::collections::BTreeMap<_, _>>>()?;
        if simulations.len() != count {
            return Err(invalid(py, "duplicate experiment settings"));
        }
        let cancel = CancelSource::new();
        let profile = native::FitProfile {
            solver: settings.profile.clone(),
            simulations,
            rank_tolerance,
            max_cells,
        };
        let inner = blocking(
            py,
            &self.owner,
            self.inner
                .prepare_fit(fit, profile, Default::default(), &cancel),
            || cancel.cancel(),
        )?;
        Ok(NativePreparedOperation {
            owner: self.owner.clone(),
            inner: PreparedOperation::Fit(inner),
        })
    }

    #[getter]
    fn identity(&self) -> String {
        self.inner.identity().to_prefixed()
    }
    fn declaration<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        let bytes = py
            .detach(|| {
                serde_json::to_vec(&ModelEnvelope {
                    declaration: self.inner.declaration().clone(),
                    sources: self.inner.source_declarations().clone(),
                })
            })
            .map_err(|e| invalid(py, e.to_string()))?;
        Ok(PyBytes::new(py, &bytes))
    }
    fn revise(&self, py: Python<'_>, declaration: &[u8]) -> PyResult<Self> {
        if declaration.len() > self.owner.shared.budget().math.workspace_bytes / 4 {
            return Err(invalid(py, "model input exceeds workspace allowance"));
        }
        let inner = py
            .detach(|| {
                let mut draft = self.inner.edit();
                let wire = serde_json::from_slice::<ModelEnvelope>(declaration)
                    .map_err(|e| native::WorkflowError::Contract(e.to_string()))?;
                *draft.declaration_mut() = wire.declaration;
                *draft.sources_mut() = wire.sources;
                draft.freeze()
            })
            .map_err(|e| errors::diagnostic(py, &e))?;
        Ok(Self {
            owner: self.owner.clone(),
            inner,
        })
    }
    #[pyo3(signature=(case_id, settings))]
    fn prepare(
        &self,
        py: Python<'_>,
        case_id: &str,
        settings: &SolveSettings,
    ) -> PyResult<NativePreparedCase> {
        let case = id(py, case_id)?;
        let cancel = CancelSource::new();
        let inner = blocking(
            py,
            &self.owner,
            self.inner
                .prepare(case, settings.profile.clone(), Default::default(), &cancel),
            || cancel.cancel(),
        )?;
        Ok(NativePreparedCase {
            owner: self.owner.clone(),
            inner,
        })
    }
}
/// Immutable class-specific admitted preparation.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativePreparedCase {
    owner: Arc<runtime::Runtime>,
    inner: native::PreparedCase,
}
#[pymethods]
impl NativePreparedCase {
    fn with_primal_start(
        &self,
        py: Python<'_>,
        values: std::collections::BTreeMap<String, f64>,
    ) -> PyResult<Self> {
        let values = values
            .into_iter()
            .map(|(key, v)| id(py, &key).map(|key| (key, v)))
            .collect::<PyResult<_>>()?;
        let inner = self
            .inner
            .clone()
            .with_primal_start(values)
            .map_err(|e| errors::diagnostic(py, &e))?;
        Ok(Self {
            owner: self.owner.clone(),
            inner,
        })
    }
    #[getter]
    fn eligibility(&self) -> Vec<(String, Vec<String>)> {
        self.inner
            .eligibility()
            .iter()
            .map(|e| (e.backend.as_str().into(), e.reasons.clone()))
            .collect()
    }
    fn with_start(&self, py: Python<'_>, seed: &NativeStart) -> PyResult<Self> {
        let inner = self
            .inner
            .clone()
            .with_start(seed.inner.clone())
            .map_err(|e| errors::diagnostic(py, &e))?;
        Ok(Self {
            owner: self.owner.clone(),
            inner,
        })
    }
    #[getter]
    fn route(&self) -> String {
        format!("{:?}", self.inner.route())
    }
    fn start(&self, py: Python<'_>) -> PyResult<NativeRunHandle> {
        let inner = py
            .detach(|| {
                let _enter = self.owner.executor.enter();
                self.inner.start()
            })
            .map_err(|e| errors::diagnostic(py, &e))?;
        Ok(NativeRunHandle {
            owner: self.owner.clone(),
            inner,
        })
    }
}
/// Immutable portable numerical seed. Native allocation state never crosses this boundary.
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
            native::RunReport::Solves(r) => match r.outcomes.get(step)? {
                pse_runtime::math::solves::Outcome::Native(r) => {
                    r.warm_start.clone().map(|inner| NativeStart { inner })
                }
                _ => None,
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
    fn diagnostics(&self) -> Vec<inspection::DiagnosticReport> {
        match self.inner.report() {
            Err(e) => vec![inspection::DiagnosticReport::observe(e)],
            Ok(native::RunReport::Fit(_)) => vec![],
            Ok(native::RunReport::Simulation(report)) => report
                .error
                .as_ref()
                .map(|e| inspection::DiagnosticReport::observe(e))
                .into_iter()
                .collect(),
            Ok(native::RunReport::Solves(report)) => report
                .outcomes
                .iter()
                .filter_map(|outcome| match outcome {
                    pse_runtime::math::solves::Outcome::Rejected(e) => {
                        Some(inspection::DiagnosticReport::observe(e.as_ref()))
                    }
                    _ => None,
                })
                .collect(),
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
    #[pyo3(signature=(base, workspace_id, *, parent=None))]
    fn prepare_publication(
        &self,
        py: Python<'_>,
        base: &str,
        workspace_id: &str,
        parent: Option<&str>,
    ) -> PyResult<NativePublicationAttempt> {
        let base = url::Url::parse(base).map_err(|e| invalid(py, e.to_string()))?;
        let workspace = id(py, workspace_id)?;
        let parent = parent.map(|p| id(py, p)).transpose()?;
        let attempt = py
            .detach(|| {
                self.inner.prepare_publication(
                    base,
                    workspace,
                    parent,
                    &pse_columnar::CancellationToken::new(),
                )
            })
            .map_err(|e| errors::diagnostic(py, &e))?;
        Ok(NativePublicationAttempt {
            owner: self.owner.clone(),
            attempt_id: attempt.attempt_id.to_hex(),
            publication_id: attempt.publication_id.to_hex(),
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
}
#[pymethods]
impl NativePublicationAttempt {
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
    #[pyo3(signature=(*, start, end, samples, atol, parameter_scales, sensitivities=false, rtol=1e-6, out_rtol=None, out_atol=None, initial_step=1e-4, max_steps=100000, max_events=1000, time_limit=300.0, max_cells=1000000,method="auto",trial_failures="terminal",numerics: "dict[str, object] | None"=None))]
    fn new(
        py: Python<'_>,
        start: f64,
        end: f64,
        samples: Vec<f64>,
        atol: Vec<f64>,
        parameter_scales: Vec<f64>,
        sensitivities: bool,
        rtol: f64,
        out_rtol: Option<f64>,
        out_atol: Option<Vec<f64>>,
        initial_step: f64,
        max_steps: usize,
        max_events: usize,
        time_limit: f64,
        max_cells: usize,
        method: &str,
        trial_failures: &str,
        numerics: Option<&Bound<'_, pyo3::types::PyDict>>,
    ) -> PyResult<Self> {
        let time_limit =
            Duration::try_from_secs_f64(time_limit).map_err(|e| invalid(py, e.to_string()))?;
        Ok(Self {
            profile: native::SimulationProfile {
                method: match method {
                    "auto" => pse_backend_native::dynamics::Method::Auto,
                    "diffsol" => pse_backend_native::dynamics::Method::Diffsol,
                    "idas" => pse_backend_native::dynamics::Method::Idas,
                    _ => return Err(invalid(py, "unknown dynamics method")),
                },
                trial_failures: match trial_failures {
                    "terminal" => pse_backend_native::dynamics::TrialPolicy::Terminal,
                    "recoverable" => pse_backend_native::dynamics::TrialPolicy::Recoverable,
                    _ => return Err(invalid(py, "unknown trial failure policy")),
                },
                numerics: settings::numerical_policy(py, numerics)?,
                start,
                end,
                samples,
                atol,
                parameter_scales,
                sensitivities,
                rtol,
                out_rtol,
                out_atol: out_atol.unwrap_or_default(),
                initial_step,
                max_steps,
                max_events,
                time_limit,
                max_cells,
                ..Default::default()
            },
        })
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
    Simulation(native::PreparedSimulation),
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
    #[getter]
    fn identity(&self) -> String {
        match &self.inner {
            PreparedOperation::Simulation(s) => s.identity(),
            PreparedOperation::Fit(f) => f.identity(),
        }
        .to_prefixed()
    }
    fn start(&self, py: Python<'_>) -> PyResult<NativeRunHandle> {
        let inner = py
            .detach(|| {
                let _enter = self.owner.executor.enter();
                match &self.inner {
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
