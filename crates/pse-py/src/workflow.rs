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
        Ok(NativePhysicalContext {
            inner,
            documents: Arc::new(documents),
        })
    }
    /// The store's durable studies, newest first, as `runtime.operational_studies`:
    /// optionally those in the given registry `StudyState` names.
    #[pyo3(signature = (*, states=Vec::new(), limit=100))]
    fn studies(
        &self,
        py: Python<'_>,
        states: Vec<String>,
        limit: i64,
    ) -> PyResult<inspection::TableStream> {
        let filter = native::StudyFilter {
            states: states
                .iter()
                .map(|s| settings::named(py, "study state", s))
                .collect::<PyResult<_>>()?,
            limit,
        };
        let batch = blocking(py, &self.owner, self.inner.studies(&filter), || {})?;
        Ok(inspection::TableStream::from_batch(batch))
    }
    /// The store's durable jobs, newest first, as `runtime.operational_jobs`: optionally
    /// those in the given registry `JobState` names.
    #[pyo3(signature = (*, states=Vec::new(), limit=100))]
    fn jobs(
        &self,
        py: Python<'_>,
        states: Vec<String>,
        limit: i64,
    ) -> PyResult<inspection::TableStream> {
        let filter = native::JobFilter {
            states: states
                .iter()
                .map(|s| settings::named(py, "job state", s))
                .collect::<PyResult<_>>()?,
            limit,
        };
        let batch = blocking(py, &self.owner, self.inner.jobs(&filter), || {})?;
        Ok(inspection::TableStream::from_batch(batch))
    }
    /// Run one SQL query over this runtime's query session: the operational relations
    /// under `pse_ops` when durable, a run's retained results under `workspace`, and an
    /// open publication's members. The answer streams in batches of the deployment's
    /// batch size.
    #[pyo3(signature = (sql, *, result=None, publication=None))]
    fn query(
        &self,
        py: Python<'_>,
        sql: &str,
        result: Option<&NativeRunResult>,
        publication: Option<&inspection::Publication>,
    ) -> PyResult<inspection::TableStream> {
        let (base, cancel) = match publication {
            Some(publication) => {
                let (session, cancel) = publication
                    .query_base()
                    .map_err(|e| errors::diagnostic(py, &e))?;
                (Some(session), cancel)
            }
            None => (None, pse_columnar::CancellationToken::new()),
        };
        let batch_size = std::num::NonZeroUsize::new(self.owner.shared.budget().execution.batch_size)
            .ok_or_else(|| invalid(py, "batch size must be positive"))?;
        let reader = blocking(
            py,
            &self.owner,
            async {
                let session = self.inner.query_session(
                    base.as_ref(),
                    result.map(|result| result.inner.as_ref()),
                    &cancel,
                )?;
                Ok::<_, native::WorkflowError>(
                    pse_catalog::inspection::TableReader::query(
                        &session,
                        sql,
                        batch_size,
                        cancel.clone(),
                    )
                    .await?,
                )
            },
            || cancel.cancel(),
        )?;
        Ok(inspection::TableStream::new(reader, self.owner.clone()))
    }
    /// A durable attempt's stored progress events and incumbents in observation order,
    /// `page` of each stream at a time; with `follow`, until the attempt ends.
    #[pyo3(signature = (attempt_id, *, follow=true, page=256))]
    fn progress(
        &self,
        py: Python<'_>,
        attempt_id: &str,
        follow: bool,
        page: usize,
    ) -> PyResult<NativeProgressStream> {
        let attempt = id(py, attempt_id)?;
        let cancel = pse_columnar::CancellationToken::new();
        let stream = blocking(
            py,
            &self.owner,
            self.inner
                .progress(attempt.into(), follow, page, cancel.clone()),
            || cancel.cancel(),
        )?;
        Ok(NativeProgressStream {
            owner: self.owner.clone(),
            attempt_id: attempt.to_hex(),
            cancel,
            slot: Mutex::new(ProgressSlot::Idle(Box::new(ProgressState {
                stream,
                buffered: std::collections::VecDeque::new(),
            }))),
        })
    }
    /// Serve the durable job queue in this process until no job is available (or `jobs`
    /// jobs ran), as `pse-worker --until-idle` does; the number of jobs processed.
    #[pyo3(signature = (*, jobs=None))]
    fn work(&self, py: Python<'_>, jobs: Option<usize>) -> PyResult<usize> {
        let stop = CancelSource::new();
        let settings = native::WorkerSettings {
            jobs,
            until_idle: true,
            ..native::WorkerSettings::default()
        };
        let processed = blocking(py, &self.owner, self.inner.serve(settings, &stop), || {
            stop.cancel();
        })?;
        Ok(processed.len())
    }
    /// A handle on a durable study of this runtime's store.
    fn study(&self, py: Python<'_>, study_id: &str) -> PyResult<NativeStudyHandle> {
        Ok(NativeStudyHandle {
            owner: self.owner.clone(),
            inner: self.inner.study(id(py, study_id)?.into()),
        })
    }
    fn capabilities(&self, py: Python<'_>) -> PyResult<Vec<u8>> {
        serde_json::to_vec(&self.inner.capabilities()).map_err(|e| invalid(py, e.to_string()))
    }
    fn clear_program_cache(&self) {
        self.inner.clear_program_cache();
    }
    /// Settle a ticket whose commit outcome is unknown, by querying the catalog.
    fn settle_publication(&self, py: Python<'_>, ticket: &[u8]) -> PyResult<Vec<u8>> {
        if ticket.len() > self.owner.shared.budget().math.workspace_bytes / 4 {
            return Err(invalid(py, "publication ticket exceeds input allowance"));
        }
        let ticket: native::PublicationTicket =
            serde_json::from_slice(ticket).map_err(|e| invalid(py, e.to_string()))?;
        let result = blocking(
            py,
            &self.owner,
            async {
                Ok::<_, native::WorkflowError>(self.inner.settle_publication(&ticket).await)
            },
            || {},
        )?;
        serde_json::to_vec(&result).map_err(|e| invalid(py, e.to_string()))
    }
    /// Register a publication workspace, or return the one of that name with the same
    /// root; the workspace as JSON.
    fn register_workspace(&self, py: Python<'_>, name: &str, root: &str) -> PyResult<Vec<u8>> {
        let root = url::Url::parse(root).map_err(|e| invalid(py, e.to_string()))?;
        let workspace = blocking(
            py,
            &self.owner,
            self.inner.register_workspace(name, root),
            || {},
        )?;
        serde_json::to_vec(&workspace).map_err(|e| invalid(py, e.to_string()))
    }
    /// A registered workspace by name, as JSON.
    fn workspace(&self, py: Python<'_>, name: &str) -> PyResult<Vec<u8>> {
        let workspace = blocking(py, &self.owner, self.inner.workspace(name), || {})?;
        serde_json::to_vec(&workspace).map_err(|e| invalid(py, e.to_string()))
    }
    /// The head of a workspace; `None` before its first publication.
    fn head(&self, py: Python<'_>, workspace_id: &str) -> PyResult<Option<String>> {
        let workspace = id(py, workspace_id)?.into();
        let head = blocking(py, &self.owner, self.inner.head(workspace), || {})?;
        Ok(head.map(|head| pse_ids::SemanticId::from(head).to_hex()))
    }
    /// Open an exact publication under a catalog reader lease.
    fn open(&self, py: Python<'_>, publication_id: &str) -> PyResult<inspection::Publication> {
        let publication = id(py, publication_id)?.into();
        let cancel = pse_columnar::CancellationToken::new();
        let leased = blocking(
            py,
            &self.owner,
            self.inner.open(publication, &cancel),
            || cancel.cancel(),
        )?;
        Ok(inspection::Publication::leased(
            leased.publication().clone(),
            leased.guard().clone(),
            self.owner.clone(),
        ))
    }
    /// Open the head of a workspace under a catalog reader lease.
    fn open_head(&self, py: Python<'_>, workspace_id: &str) -> PyResult<inspection::Publication> {
        let workspace = id(py, workspace_id)?.into();
        let cancel = pse_columnar::CancellationToken::new();
        let leased = blocking(
            py,
            &self.owner,
            self.inner.open_head(workspace, &cancel),
            || cancel.cancel(),
        )?;
        Ok(inspection::Publication::leased(
            leased.publication().clone(),
            leased.guard().clone(),
            self.owner.clone(),
        ))
    }
    /// Export a publication for offline readers at `destination`, protected for
    /// `valid_for_seconds`; the export receipt as JSON.
    fn export_publication(
        &self,
        py: Python<'_>,
        publication_id: &str,
        destination: &str,
        valid_for_seconds: f64,
    ) -> PyResult<Vec<u8>> {
        let publication = id(py, publication_id)?.into();
        let destination = url::Url::parse(destination).map_err(|e| invalid(py, e.to_string()))?;
        let valid_for = Duration::try_from_secs_f64(valid_for_seconds)
            .map_err(|e| invalid(py, e.to_string()))?;
        let cancel = pse_columnar::CancellationToken::new();
        let receipt = blocking(
            py,
            &self.owner,
            self.inner
                .export_publication(publication, destination, valid_for, &cancel),
            || cancel.cancel(),
        )?;
        serde_json::to_vec(&receipt).map_err(|e| invalid(py, e.to_string()))
    }
    /// Release an export's lease; whether it was still held.
    fn release_export(&self, py: Python<'_>, receipt: &[u8]) -> PyResult<bool> {
        let receipt: native::ExportReceipt =
            serde_json::from_slice(receipt).map_err(|e| invalid(py, e.to_string()))?;
        blocking(py, &self.owner, self.inner.release_export(&receipt), || {})
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
        (events.into_iter().map(ProgressEvent::native).collect(), dropped)
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
        self.inner.run_id.as_id().to_hex()
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
    /// Prepare the publication of this durable attempt's results in a registered
    /// workspace (JSON) against the exact expected parent; performs no write.
    #[pyo3(signature=(workspace, *, parent=None, publication_id=None))]
    fn prepare_publication(
        &self,
        py: Python<'_>,
        workspace: &[u8],
        parent: Option<&str>,
        publication_id: Option<&str>,
    ) -> PyResult<NativePublicationAttempt> {
        let workspace: native::Workspace =
            serde_json::from_slice(workspace).map_err(|e| invalid(py, e.to_string()))?;
        let parent = parent.map(|p| id(py, p).map(Into::into)).transpose()?;
        let publication_id = publication_id
            .map(|p| id(py, p).map(Into::into))
            .transpose()?;
        let attempt = py
            .detach(|| {
                self.inner.prepare_publication(
                    &workspace,
                    parent,
                    publication_id,
                    &pse_columnar::CancellationToken::new(),
                )
            })
            .map_err(|e| errors::diagnostic(py, &e))?;
        Ok(NativePublicationAttempt {
            owner: self.owner.clone(),
            attempt_id: pse_ids::SemanticId::from(attempt.attempt_id).to_hex(),
            publication_id: pse_ids::SemanticId::from(attempt.publication_id).to_hex(),
            ticket: serde_json::to_vec(&attempt.ticket).map_err(|e| invalid(py, e.to_string()))?,
            inner: Mutex::new(Some(attempt)),
        })
    }
}
/// Reviewable single-consumption publication; commit never reruns a solve.
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
    /// Register the intent, write and admit the candidate, commit it once; the
    /// publication as JSON. Never retried implicitly.
    fn commit(&self, py: Python<'_>) -> PyResult<Vec<u8>> {
        let attempt = self
            .inner
            .lock()
            .map_err(|_| invalid(py, "publication attempt lock poisoned"))?
            .take()
            .ok_or_else(|| {
                invalid(
                    py,
                    "publication attempt already consumed; settle its ticket before a retry",
                )
            })?;
        let cancel = pse_columnar::CancellationToken::new();
        let published = blocking(py, &self.owner, attempt.commit(&cancel), || cancel.cancel())?;
        serde_json::to_vec(&published).map_err(|e| invalid(py, e.to_string()))
    }
}

/// Actual admitted physical source rows and compiler context.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativePhysicalContext {
    inner: native::PhysicalContext,
    /// The authored documents it was admitted from: what a durable study stores for its
    /// workers.
    documents: Arc<std::collections::BTreeMap<String, String>>,
}

/// A durable study (Plan 22 O7): its status, cancellation and publication.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeStudyHandle {
    owner: Arc<runtime::Runtime>,
    inner: native::StudyHandle,
}
#[pymethods]
impl NativeStudyHandle {
    /// The study identity.
    #[getter]
    fn study_id(&self) -> String {
        pse_ids::SemanticId::from(self.inner.study_id()).to_hex()
    }
    /// The study's status as JSON.
    fn status(&self, py: Python<'_>) -> PyResult<Vec<u8>> {
        let status = blocking(py, &self.owner, self.inner.status(), || {})?;
        serde_json::to_vec(&status).map_err(|e| invalid(py, e.to_string()))
    }
    /// Cancel the study; what the cancellation did, as JSON.
    fn cancel(&self, py: Python<'_>) -> PyResult<Vec<u8>> {
        let cancelled = blocking(py, &self.owner, self.inner.cancel(), || {})?;
        serde_json::to_vec(&serde_json::json!({
            "cancelled": cancelled.cancelled,
            "stopping": cancelled.stopping,
            "concluded": cancelled.concluded,
            "already_concluded": cancelled.already_concluded,
        }))
        .map_err(|e| invalid(py, e.to_string()))
    }
    /// The study's publication as JSON once committed.
    fn result(&self, py: Python<'_>) -> PyResult<Option<Vec<u8>>> {
        blocking(py, &self.owner, self.inner.result(), || {})?
            .map(|published| serde_json::to_vec(&published).map_err(|e| invalid(py, e.to_string())))
            .transpose()
    }
    /// Wait until the study is published, polling every `poll_seconds`, at most
    /// `timeout_seconds` when given; the publication as JSON.
    #[pyo3(signature = (*, poll_seconds=0.5, timeout_seconds=None))]
    fn wait(
        &self,
        py: Python<'_>,
        poll_seconds: f64,
        timeout_seconds: Option<f64>,
    ) -> PyResult<Vec<u8>> {
        let poll = Duration::try_from_secs_f64(poll_seconds).map_err(|e| invalid(py, e.to_string()))?;
        let timeout = timeout_seconds
            .map(Duration::try_from_secs_f64)
            .transpose()
            .map_err(|e| invalid(py, e.to_string()))?;
        let study = self.inner.study_id();
        let published = blocking(
            py,
            &self.owner,
            async {
                match timeout {
                    None => self.inner.wait(poll).await,
                    Some(limit) => tokio::time::timeout(limit, self.inner.wait(poll))
                        .await
                        .map_err(|_| {
                            native::WorkflowError::Contract(format!(
                                "study {study} was not published within {limit:?}"
                            ))
                        })?,
                }
            },
            || {},
        )?;
        serde_json::to_vec(&published).map_err(|e| invalid(py, e.to_string()))
    }
}
#[pymethods]
impl NativePhysicalContext {
    #[getter]
    fn identity(&self) -> String {
        self.inner.identity().to_prefixed()
    }
}

/// A new incumbent of a branch-and-bound search: its objective in original units under
/// the post-solve convention, the search's bounds then, and, for a stored incumbent whose
/// solution was kept for resumption, that solution's identity.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Incumbent {
    objective: f64,
    dual_bound: Option<f64>,
    gap: Option<f64>,
    nodes: Option<i64>,
    seconds: Option<f64>,
    solution_id: Option<String>,
}
impl Incumbent {
    fn native(incumbent: &pse_backend_native::solve::IncumbentEvent) -> Self {
        Self {
            objective: incumbent.objective,
            dual_bound: incumbent.dual_bound,
            gap: incumbent.gap,
            nodes: Some(incumbent.nodes),
            seconds: Some(incumbent.seconds),
            solution_id: None,
        }
    }
}
#[pymethods]
impl Incumbent {
    /// The incumbent's objective value.
    #[getter]
    fn objective(&self) -> f64 {
        self.objective
    }
    /// The global dual bound in the objective's units; `None` while none is finite.
    #[getter]
    fn dual_bound(&self) -> Option<f64> {
        self.dual_bound
    }
    /// The relative gap; `None` while it is not finite.
    #[getter]
    fn gap(&self) -> Option<f64> {
        self.gap
    }
    /// Branch-and-bound nodes explored by then.
    #[getter]
    fn nodes(&self) -> Option<i64> {
        self.nodes
    }
    /// The search's native running time then, in seconds.
    #[getter]
    fn seconds(&self) -> Option<f64> {
        self.seconds
    }
    /// The stored solution it captured (32 hexadecimal digits), if one was kept.
    #[getter]
    fn solution_id(&self) -> Option<String> {
        self.solution_id.clone()
    }
    fn __repr__(&self) -> String {
        format!(
            "Incumbent(objective={}, dual_bound={:?}, gap={:?}, nodes={:?})",
            self.objective, self.dual_bound, self.gap, self.nodes
        )
    }
}

/// Where a stored event sits in its attempt's streams.
#[derive(Clone, Copy, Debug)]
struct Stored {
    step: i32,
    sequence: i64,
    at: i64,
}

/// One bounded, owned progress event: observed in memory, or read back from the
/// operational store with its step, stream sequence and observation time. An incumbent
/// of a branch-and-bound search carries it typed. No native thread enters Python.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct ProgressEvent {
    phase: String,
    elapsed_seconds: f64,
    values: std::collections::BTreeMap<String, pse_backend_native::solve::Metric>,
    incumbent: Option<Incumbent>,
    stored: Option<Stored>,
}
impl ProgressEvent {
    /// An event observed in memory.
    pub(crate) fn native(event: pse_backend_native::solve::Event) -> Self {
        Self {
            incumbent: event.incumbent.as_ref().map(Incumbent::native),
            phase: event.phase,
            elapsed_seconds: event.elapsed.as_secs_f64(),
            values: event.values,
            stored: None,
        }
    }
    /// A record read back from the operational store.
    fn stored(record: &native::StreamRecord) -> Self {
        let incumbent = match record {
            native::StreamRecord::Incumbent(row) => Some(Incumbent {
                objective: row.objective,
                dual_bound: row.dual_bound,
                gap: row.gap,
                nodes: row.nodes,
                seconds: row.seconds,
                solution_id: row
                    .solution_id
                    .map(|id| pse_ids::SemanticId::from(id).to_hex()),
            }),
            native::StreamRecord::Progress(_) => None,
        };
        Self {
            phase: record.phase().to_owned(),
            elapsed_seconds: record.elapsed_seconds(),
            values: record.values(),
            incumbent,
            stored: Some(Stored {
                step: record.step(),
                sequence: record.sequence(),
                at: record.at(),
            }),
        }
    }
}
#[pymethods]
impl ProgressEvent {
    #[getter]
    fn phase(&self) -> &str {
        &self.phase
    }
    #[getter]
    fn elapsed_seconds(&self) -> f64 {
        self.elapsed_seconds
    }
    /// The typed incumbent this event reports, if it reports one.
    #[getter]
    fn incumbent(&self) -> Option<Incumbent> {
        self.incumbent.clone()
    }
    /// The step of the run that produced a stored event; `None` in memory.
    #[getter]
    fn step(&self) -> Option<i32> {
        self.stored.map(|stored| stored.step)
    }
    /// A stored event's sequence number in its stream (progress events and incumbents
    /// are numbered separately); `None` in memory.
    #[getter]
    fn sequence(&self) -> Option<i64> {
        self.stored.map(|stored| stored.sequence)
    }
    /// When a stored event was observed, in microseconds since the Unix epoch; `None`
    /// in memory.
    #[getter]
    fn at(&self) -> Option<i64> {
        self.stored.map(|stored| stored.at)
    }
    #[pyo3(signature=() -> "dict[str, bool | int | float | str | dict[str, str]]")]
    fn values<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        use pse_backend_native::solve::Metric;
        let dict = pyo3::types::PyDict::new(py);
        for (k, v) in &self.values {
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

/// An open stored stream and the events of its last page not yet returned.
#[derive(Debug)]
struct ProgressState {
    stream: native::ProgressStream,
    buffered: std::collections::VecDeque<ProgressEvent>,
}

/// Where a [`NativeProgressStream`] is. The lock is never held while a page is read, so a
/// reader waiting without the GIL never blocks a thread that holds it.
#[derive(Debug)]
enum ProgressSlot {
    Idle(Box<ProgressState>),
    /// A thread is reading the next page.
    Reading,
    Closed,
}

/// A durable attempt's stored progress events and incumbents, one bounded page held at a
/// time. Closing it ends a read that is waiting.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Debug)]
pub(crate) struct NativeProgressStream {
    owner: Arc<runtime::Runtime>,
    attempt_id: String,
    cancel: pse_columnar::CancellationToken,
    slot: Mutex<ProgressSlot>,
}
impl NativeProgressStream {
    fn slot(&self, py: Python<'_>) -> PyResult<std::sync::MutexGuard<'_, ProgressSlot>> {
        self.slot
            .lock()
            .map_err(|_| invalid(py, "progress stream lock poisoned"))
    }
    /// Release a closed stream's store resources on the runtime that owns them.
    fn release(&self, state: Box<ProgressState>) {
        let _enter = self.owner.executor.enter();
        drop(state);
    }
}
#[pymethods]
impl NativeProgressStream {
    /// The attempt whose streams these are (32 hexadecimal digits).
    #[getter]
    fn attempt_id(&self) -> &str {
        &self.attempt_id
    }
    /// The next event in observation order; `None` once the stream ended or was closed.
    /// Reads the next page from the store (waiting for it when following) only when the
    /// last one is exhausted.
    fn next_event(&self, py: Python<'_>) -> PyResult<Option<ProgressEvent>> {
        let mut state = {
            let mut slot = self.slot(py)?;
            match std::mem::replace(&mut *slot, ProgressSlot::Reading) {
                ProgressSlot::Idle(mut state) => {
                    if let Some(event) = state.buffered.pop_front() {
                        *slot = ProgressSlot::Idle(state);
                        return Ok(Some(event));
                    }
                    state
                }
                ProgressSlot::Reading => {
                    *slot = ProgressSlot::Reading;
                    return Err(invalid(py, "another thread is reading this progress stream"));
                }
                ProgressSlot::Closed => {
                    *slot = ProgressSlot::Closed;
                    return Ok(None);
                }
            }
        };
        let cancel = self.cancel.clone();
        let page = blocking(py, &self.owner, state.stream.next_page(), || cancel.cancel());
        let mut slot = self.slot(py)?;
        match page {
            Ok(Some(records)) if !matches!(*slot, ProgressSlot::Closed) => {
                state
                    .buffered
                    .extend(records.iter().map(ProgressEvent::stored));
                let event = state.buffered.pop_front();
                *slot = ProgressSlot::Idle(state);
                Ok(event)
            }
            ended => {
                // Ended, failed or closed while reading: the reader releases what it holds.
                *slot = ProgressSlot::Closed;
                drop(slot);
                self.release(state);
                ended.map(|_| None)
            }
        }
    }
    /// Stop reading: a waiting read ends and later reads return nothing.
    fn close(&self, py: Python<'_>) -> PyResult<()> {
        self.cancel.cancel();
        let previous = std::mem::replace(&mut *self.slot(py)?, ProgressSlot::Closed);
        if let ProgressSlot::Idle(state) = previous {
            self.release(state);
        }
        Ok(())
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
