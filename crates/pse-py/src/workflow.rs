// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Mechanical public workflow projection; all mathematical policy remains native.
use crate::documents;
mod modeling;
use documents::DocumentContent;
mod routes;
mod settings;
pub(crate) use modeling::{
    ModelingDiagnosticSettings, ModelingLimits, NativeModelingConformance,
    NativeModelingDiagnosticSamples, NativeModelingDiagnostics, NativeModelingElasticAttempt,
    NativeModelingInitialization, NativeModelingInitializationAttempt, NativeModelingKnowledge,
    NativeModelingNativeAnalysis, NativeModelingNonlinearExplanation, NativeModelingPackage,
    NativeModelingResult, NativeModelingTrajectory, NativeStudyReport,
};
use routes::{NativeEligibility, NativeRoute};
mod strategies;
use crate::inspection::{self, errors, runtime};
use pse_runtime::{CancelSource, workflow as native};
use pyo3::prelude::*;
pub(crate) use strategies::{
    NativeAttempt, NativePreparedFlow, NativePreparedStrategy, NativeStrategyAttempt,
    NativeStrategyResult,
};

use std::{
    future::Future,
    sync::{Arc, Mutex},
    time::Duration,
};

pub(crate) fn invalid(py: Python<'_>, message: impl Into<String>) -> PyErr {
    errors::diagnostic(py, &native::WorkflowError::Input(message.into()))
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
    py: Python<'_>,
    executor: &tokio::runtime::Runtime,
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
/// Test-owned isolated database. This private boundary reuses the operational store's
/// schema creation and teardown rather than sending test SQL from Python.
#[pyclass(
    name = "_TestOperationalStore",
    skip_from_py_object,
    module = "pse._native"
)]
#[derive(Debug)]
pub(crate) struct TestOperationalStore {
    database: Option<pse_operations::testing::TestDatabase>,
    executor: Option<tokio::runtime::Runtime>,
}

#[pymethods]
impl TestOperationalStore {
    #[new]
    fn new(py: Python<'_>) -> PyResult<Self> {
        if tokio::runtime::Handle::try_current().is_ok() {
            return Err(invalid(
                py,
                "test database setup cannot re-enter an async executor",
            ));
        }
        let executor = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .map_err(|source| {
                errors::diagnostic(
                    py,
                    &pse_engine::EngineError::Infrastructure {
                        op: "build isolated test database executor".to_owned(),
                        source: Box::new(source),
                    },
                )
            })?;
        let database = py
            .detach(|| executor.block_on(pse_operations::testing::TestDatabase::create()))
            .map_err(|error| errors::diagnostic(py, &native::WorkflowError::Operations(error)))?;
        Ok(Self {
            database: Some(database),
            executor: Some(executor),
        })
    }

    /// A production store descriptor for this database, while its owner is live.
    fn store(&self, py: Python<'_>) -> PyResult<OperationalStore> {
        let database = self
            .database
            .as_ref()
            .ok_or_else(|| invalid(py, "isolated test database has been removed"))?;
        Ok(OperationalStore::new(Some(database.url().to_owned()), None))
    }

    /// Remove this database and close its setup executor. Cleanup errors are surfaced.
    fn remove(&mut self, py: Python<'_>) -> PyResult<()> {
        if tokio::runtime::Handle::try_current().is_ok() {
            return Err(invalid(
                py,
                "test database teardown cannot re-enter an async executor",
            ));
        }
        let Some(database) = self.database.take() else {
            return Ok(());
        };
        let executor = self
            .executor
            .take()
            .ok_or_else(|| invalid(py, "isolated test database has no executor"))?;
        py.detach(|| {
            let result = executor.block_on(database.remove());
            drop(executor);
            result
        })
        .map_err(|error| errors::diagnostic(py, &native::WorkflowError::Operations(error)))
    }
}

impl Drop for TestOperationalStore {
    fn drop(&mut self) {
        // Normal fixture teardown reports errors directly. Also clean up an abandoned
        // owner; destructor failures use Python's unraisable-error channel.
        if let Some(executor) = self.executor.take() {
            if let Some(database) = self.database.take() {
                // A pyclass may be dropped from an async host. Join cleanup on a
                // separate thread so Runtime::block_on never nests in that host.
                let result = std::thread::scope(|scope| {
                    scope.spawn(|| executor.block_on(database.remove())).join()
                })
                .unwrap_or_else(|_| {
                    Err(pse_operations::OperationsError::Configuration {
                        reason: "isolated test database cleanup task panicked".to_owned(),
                    })
                });
                if let Err(error) = result {
                    Python::try_attach(|py| {
                        errors::diagnostic(py, &native::WorkflowError::Operations(error))
                            .write_unraisable(py, None);
                    });
                }
            }
            executor.shutdown_background();
        }
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
        documents: Vec<std::collections::BTreeMap<String, DocumentContent>>,
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
        let request = documents::decode::<native::ConicRequest>(
            py,
            "cone request",
            request,
            self.owner.shared.budget().math.workspace_bytes / 4,
        )?;
        let inner = blocking(
            py,
            &self.owner,
            self.inner
                .prepare_conic(request, &physical.inner, settings.clone()),
            || {},
        )?;
        Ok(NativePreparedStrategy {
            owner: self.owner.clone(),
            inner: strategies::Strategy::Cone(Box::new(inner)),
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
    /// Observe the existing deployment pool and process memory report.
    fn resource_usage(
        &self,
        py: Python<'_>,
    ) -> PyResult<documents::DocumentValue<pse_runtime::ResourceReport>> {
        self.owner
            .shared
            .report()
            .map(documents::DocumentValue)
            .map_err(|error| errors::diagnostic(py, &error))
    }
    /// Durable attempts of the store, newest first, as `runtime.operational_attempts`:
    /// optionally those of one run and in the given registry `AttemptState` names.
    #[pyo3(signature = (*, run_id=None, states=Vec::new(), controls=None))]
    fn runs(
        &self,
        py: Python<'_>,
        run_id: Option<&str>,
        states: Vec<String>,
        controls: Option<&[u8]>,
    ) -> PyResult<inspection::TableStream> {
        let native::InventoryControls { limit } = documents::controls(
            py,
            "runs",
            controls,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        let filter = native::AttemptFilter {
            run: run_id.map(|r| id(py, r).map(Into::into)).transpose()?,
            states: states
                .iter()
                .map(|s| settings::named(py, "attempt state", s))
                .collect::<PyResult<_>>()?,
            limit,
        };
        let batch = blocking(py, &self.owner, self.inner.runs(&filter), || {})?;
        inspection::TableStream::from_batch(batch).map_err(|e| errors::diagnostic(py, &e))
    }
    fn physical_from_documents(
        &self,
        py: Python<'_>,
        documents: std::collections::BTreeMap<String, DocumentContent>,
    ) -> PyResult<NativePhysicalContext> {
        let documents = document_bytes(documents);
        let cancel = CancelSource::new();
        let inner = blocking(
            py,
            &self.owner,
            async {
                let pool = self.owner.shared.pool();
                let token = cancel.token();
                let validation = self
                    .owner
                    .sessions
                    .validation_context(&self.owner.registry)?;
                let bundle = pse_runtime::authoring_driver::document::load_package_documents_owned(
                    &documents,
                    &self.owner.registry,
                    pse_authoring::ParseBudget::default(),
                    &pool,
                    &token,
                    &validation,
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
    #[pyo3(signature = (*, states=Vec::new(), controls=None))]
    fn studies(
        &self,
        py: Python<'_>,
        states: Vec<String>,
        controls: Option<&[u8]>,
    ) -> PyResult<inspection::TableStream> {
        let native::InventoryControls { limit } = documents::controls(
            py,
            "studies",
            controls,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        let filter = native::StudyFilter {
            states: states
                .iter()
                .map(|s| settings::named(py, "study state", s))
                .collect::<PyResult<_>>()?,
            limit,
        };
        let batch = blocking(py, &self.owner, self.inner.studies(&filter), || {})?;
        inspection::TableStream::from_batch(batch).map_err(|e| errors::diagnostic(py, &e))
    }
    /// The store's durable jobs, newest first, as `runtime.operational_jobs`: optionally
    /// those in the given registry `JobState` names.
    #[pyo3(signature = (*, states=Vec::new(), controls=None))]
    fn jobs(
        &self,
        py: Python<'_>,
        states: Vec<String>,
        controls: Option<&[u8]>,
    ) -> PyResult<inspection::TableStream> {
        let native::InventoryControls { limit } = documents::controls(
            py,
            "jobs",
            controls,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        let filter = native::JobFilter {
            states: states
                .iter()
                .map(|s| settings::named(py, "job state", s))
                .collect::<PyResult<_>>()?,
            limit,
        };
        let batch = blocking(py, &self.owner, self.inner.jobs(&filter), || {})?;
        inspection::TableStream::from_batch(batch).map_err(|e| errors::diagnostic(py, &e))
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
        let batch_size =
            std::num::NonZeroUsize::new(self.owner.shared.budget().execution.batch_size)
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
    #[pyo3(signature = (attempt_id, *, controls=None))]
    fn progress(
        &self,
        py: Python<'_>,
        attempt_id: &str,
        controls: Option<&[u8]>,
    ) -> PyResult<NativeProgressStream> {
        let native::ProgressControls { follow, page } = documents::controls(
            py,
            "progress",
            controls,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
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
        documents::encode(py, &self.inner.capabilities())
    }
    fn clear_program_cache(&self) {
        self.inner.clear_program_cache();
    }
    /// Settle a ticket whose commit outcome is unknown, by querying the catalog.
    fn settle_publication(&self, py: Python<'_>, ticket: &[u8]) -> PyResult<Vec<u8>> {
        if ticket.len() > self.owner.shared.budget().math.workspace_bytes / 4 {
            return Err(invalid(py, "publication ticket exceeds input allowance"));
        }
        let ticket: native::PublicationTicket = documents::decode(
            py,
            "operation",
            ticket,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        let result = blocking(
            py,
            &self.owner,
            async { Ok::<_, native::WorkflowError>(self.inner.settle_publication(&ticket).await) },
            || {},
        )?;
        documents::encode(py, &result)
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
        documents::encode(py, &workspace)
    }
    /// A registered workspace by name, as JSON.
    fn workspace(&self, py: Python<'_>, name: &str) -> PyResult<Vec<u8>> {
        let workspace = blocking(py, &self.owner, self.inner.workspace(name), || {})?;
        documents::encode(py, &workspace)
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
        documents::encode(py, &receipt)
    }
    /// Release an export's lease; whether it was still held.
    fn release_export(&self, py: Python<'_>, receipt: &[u8]) -> PyResult<bool> {
        let receipt: native::ExportReceipt = documents::decode(
            py,
            "operation",
            receipt,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        blocking(py, &self.owner, self.inner.release_export(&receipt), || {})
    }
    #[pyo3(signature=(steps, *, controls=None))]
    fn start(
        &self,
        py: Python<'_>,
        steps: Vec<PyRef<'_, NativePreparedOperation>>,
        controls: Option<&[u8]>,
    ) -> PyResult<NativeRunHandle> {
        let native::RunControls {
            continue_independent,
        } = documents::controls(
            py,
            "run",
            controls,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        let steps = steps
            .into_iter()
            .map(|p| match &p.inner {
                PreparedOperation::Modeling(step) => Ok(step.as_ref().clone()),
                _ => Err(invalid(
                    py,
                    "finite solve sequences require authored algebraic cases",
                )),
            })
            .collect::<PyResult<Vec<_>>>()?;
        let cancel = CancelSource::new();
        let inner = blocking(
            py,
            &self.owner,
            self.inner
                .start_modeling(steps, continue_independent, &cancel),
            || cancel.cancel(),
        )?;
        Ok(NativeRunHandle {
            owner: self.owner.clone(),
            inner,
        })
    }
}
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeStart {
    inner: pse_backend_native::solve::WarmStart,
}
#[pymethods]
impl NativeStart {
    fn snapshot(&self, py: Python<'_>) -> PyResult<Vec<u8>> {
        documents::encode(py, &self.inner.snapshot())
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
        (
            events
                .into_iter()
                .map(|event| documents::DocumentValue(event.into()))
                .collect(),
            dropped,
        )
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
    #[pyo3(signature=(step=None))]
    fn available_start(&self, step: Option<usize>) -> Option<NativeStart> {
        self.inner
            .available_start(step)
            .map(|inner| NativeStart { inner })
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
        documents::encode(py, completed)
    }
    fn fit_profiles(&self, py: Python<'_>) -> PyResult<Option<Vec<u8>>> {
        let report = self
            .inner
            .report()
            .map_err(|error| errors::diagnostic(py, error))?;
        match report {
            native::RunReport::Fit(report) => {
                documents::encode(py, &report.profile_document()).map(Some)
            }
            _ => Ok(None),
        }
    }
    fn diagnostics(&self) -> Vec<inspection::DiagnosticReport> {
        match self.inner.completion() {
            Ok(completed) => completed
                .diagnostics
                .iter()
                .map(inspection::DiagnosticReport::observe)
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
            .map_err(|e| errors::diagnostic(py, e.as_ref()))?
            .map_err(|e| errors::diagnostic(py, &e))
    }
    fn export_fit_parameters(&self, py: Python<'_>) -> PyResult<inspection::TableStream> {
        py.detach(|| self.inner.export_fit_parameters())
            .map(inspection::TableStream::from_batch)
            .map_err(|error| errors::diagnostic(py, &error))?
            .map_err(|e| errors::diagnostic(py, &e))
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
        let workspace: native::Workspace = documents::decode(
            py,
            "operation",
            workspace,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
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
            ticket: documents::encode(py, &attempt.ticket)?,
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
        documents::encode(py, &published)
    }
}

/// Actual admitted physical source rows and compiler context.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativePhysicalContext {
    inner: native::PhysicalContext,
    /// The authored documents it was admitted from: what a durable study stores for its
    /// workers.
    documents: Arc<std::collections::BTreeMap<String, Vec<u8>>>,
}
/// Normalize text and binary documents once, preserving the exact durable byte payload.
pub(crate) fn document_bytes(
    documents: std::collections::BTreeMap<String, DocumentContent>,
) -> std::collections::BTreeMap<String, Vec<u8>> {
    documents
        .into_iter()
        .map(|(path, content)| (path, content.0))
        .collect()
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
        documents::encode(py, &status)
    }
    /// Cancel the study; what the cancellation did, as JSON.
    fn cancel(&self, py: Python<'_>) -> PyResult<Vec<u8>> {
        let cancelled = blocking(py, &self.owner, self.inner.cancel(), || {})?;
        documents::encode(py, &cancelled)
    }
    /// The study's publication as JSON once committed.
    fn result(&self, py: Python<'_>) -> PyResult<Option<Vec<u8>>> {
        blocking(py, &self.owner, self.inner.result(), || {})?
            .map(|published| documents::encode(py, &published))
            .transpose()
    }
    /// Wait until the study is published, polling every `poll_seconds`, at most
    /// `timeout_seconds` when given; the publication as JSON.
    #[pyo3(signature = (*, controls=None))]
    fn wait(&self, py: Python<'_>, controls: Option<&[u8]>) -> PyResult<Vec<u8>> {
        let native::StudyWaitControls {
            poll_seconds,
            timeout_seconds,
        } = documents::controls(
            py,
            "study wait",
            controls,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        let poll =
            Duration::try_from_secs_f64(poll_seconds).map_err(|e| invalid(py, e.to_string()))?;
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
                            native::WorkflowError::Input(format!(
                                "study {study} was not published within {limit:?}"
                            ))
                        })?,
                }
            },
            || {},
        )?;
        documents::encode(py, &published)
    }
}
#[pymethods]
impl NativePhysicalContext {
    #[getter]
    fn identity(&self) -> String {
        self.inner.identity().to_prefixed()
    }
}

type ProgressEvent = documents::DocumentValue<native::ProgressEventDocument>;

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
                    return Err(invalid(
                        py,
                        "another thread is reading this progress stream",
                    ));
                }
                ProgressSlot::Closed => {
                    *slot = ProgressSlot::Closed;
                    return Ok(None);
                }
            }
        };
        let cancel = self.cancel.clone();
        let page = blocking(py, &self.owner, state.stream.next_page(), || {
            cancel.cancel()
        });
        let mut slot = self.slot(py)?;
        match page {
            Ok(Some(records)) if !matches!(*slot, ProgressSlot::Closed) => {
                state.buffered.extend(
                    records
                        .iter()
                        .map(|record| documents::DocumentValue(record.into())),
                );
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
    #[pyo3(signature=(*, start, end, samples, atol, parameter_scales, sensitivity=None, rtol=None, out_rtol=None, out_atol=None, initial_step=None, max_steps: "int | None"=None, max_events: "int | None"=None, time_limit=None, max_cells: "int | None"=None, method=None, trial_failures=None, numerics=None, diffsol=None, idas=None, adjoint=None))]
    fn new(
        py: Python<'_>,
        start: f64,
        end: f64,
        samples: Vec<f64>,
        atol: Vec<f64>,
        parameter_scales: Vec<f64>,
        sensitivity: Option<&str>,
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
        numerics: Option<documents::DocumentInput<pse_model::numerics::NumericalPolicy>>,
        diffsol: Option<&[u8]>,
        idas: Option<&[u8]>,
        adjoint: Option<&[u8]>,
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
        if let Some(sensitivity) = sensitivity {
            profile.sensitivity = settings::named(py, "dynamic sensitivity", sensitivity)?;
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
        if let Some(adjoint) = adjoint {
            profile.adjoint = settings::adjoint(py, adjoint)?;
        }
        Ok(Self { profile })
    }
    /// The encoded Diffsol settings document in effect.
    #[getter]
    fn diffsol(&self, py: Python<'_>) -> PyResult<Vec<u8>> {
        documents::encode(py, &self.profile.diffsol)
    }
    /// The encoded IDAS settings document in effect.
    #[getter]
    fn idas(&self, py: Python<'_>) -> PyResult<Vec<u8>> {
        documents::encode(py, &self.profile.idas)
    }
    /// The encoded adjoint checkpoint settings document in effect.
    #[getter]
    fn adjoint(&self, py: Python<'_>) -> PyResult<Vec<u8>> {
        documents::encode(py, &self.profile.adjoint)
    }
    /// Round-trip the complete pinned native BDF and initialization options.
    fn to_json(&self, py: Python<'_>) -> PyResult<String> {
        String::from_utf8(documents::encode(py, &self.profile)?)
            .map_err(|error| invalid(py, error.to_string()))
    }
    #[staticmethod]
    fn from_json(py: Python<'_>, source: &str) -> PyResult<Self> {
        documents::decode_versioned::<native::SimulationProfile, 1>(
            py,
            "simulation settings",
            source.as_bytes(),
            1 << 20,
        )
        .map(|profile| Self { profile })
    }
}
#[derive(Clone, Debug)]
enum PreparedOperation {
    Modeling(Box<native::ModelingSolvePreparation>),
    Simulation(Box<native::ModelingSimulation>),
    Fit(Box<native::PreparedFit>),
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
        let PreparedOperation::Modeling(p) = &self.inner else {
            return Err(invalid(
                py,
                "algebraic route inspection requires an algebraic solve",
            ));
        };
        Ok(p.solve.route().into())
    }
    /// Typed eligibility row of every assessed adapter.
    #[getter]
    fn eligibility(&self, py: Python<'_>) -> PyResult<Vec<NativeEligibility>> {
        let PreparedOperation::Modeling(p) = &self.inner else {
            return Err(invalid(
                py,
                "algebraic eligibility requires an algebraic solve",
            ));
        };
        Ok(p.solve.eligibility().iter().map(Into::into).collect())
    }
    /// Exact effective profile identity used by declared numerical mechanisms.
    #[getter]
    fn strategy_profile(&self, py: Python<'_>) -> PyResult<String> {
        match &self.inner {
            PreparedOperation::Modeling(p) => p
                .solve
                .strategy_profile()
                .map(|key| key.to_prefixed())
                .map_err(|error| errors::diagnostic(py, &error)),
            PreparedOperation::Fit(p) => Ok(p.strategy_profile().to_prefixed()),
            PreparedOperation::Simulation(_) => Err(invalid(
                py,
                "numerical profile is inspected on prepared algebraic or fitting targets",
            )),
        }
    }
    /// Requested automatic/declared composition and preserved constraints.
    #[getter]
    fn composition_request(
        &self,
        py: Python<'_>,
    ) -> PyResult<documents::DocumentValue<pse_model::strategy::CompositionRequest>> {
        let request = match &self.inner {
            PreparedOperation::Modeling(p) => p.solve.composition_request(),
            PreparedOperation::Fit(p) => p.composition_request(),
            PreparedOperation::Simulation(_) => {
                return Err(invalid(
                    py,
                    "integration owns its scientific controller; numerical composition is inspected on its algebraic targets",
                ));
            }
        };
        Ok(documents::DocumentValue(request.clone()))
    }
    /// Mechanical document projection of the actual declared execution policy.
    #[getter]
    fn numerical_strategy(&self, py: Python<'_>) -> PyResult<Vec<u8>> {
        let strategy = match &self.inner {
            PreparedOperation::Modeling(p) => p.solve.numerical_strategy(),
            PreparedOperation::Fit(p) => p.numerical_strategy(),
            PreparedOperation::Simulation(_) => {
                return Err(invalid(
                    py,
                    "numerical strategy is inspected on prepared algebraic or fitting targets",
                ));
            }
        };
        let document = pse_model::strategy::NumericalStrategyDocument::new(strategy)
            .map_err(|error| invalid(py, error.to_string()))?;
        documents::encode(py, &document)
    }
    /// Bind prepared original-system profiles to the validated numerical declaration.
    fn with_numerical_strategy(
        &self,
        py: Python<'_>,
        declaration: &[u8],
        rungs: Vec<PyRef<'_, NativePreparedOperation>>,
    ) -> PyResult<Self> {
        let PreparedOperation::Modeling(p) = &self.inner else {
            return Err(invalid(
                py,
                "numerical composition requires an algebraic solve",
            ));
        };
        let declaration =
            documents::decode_versioned::<pse_model::strategy::NumericalStrategyDocument, 3>(
                py,
                "numerical strategy",
                declaration,
                self.owner.shared.budget().math.workspace_bytes,
            )?;
        let rungs = rungs
            .iter()
            .map(|rung| {
                if !Arc::ptr_eq(&self.owner, &rung.owner) {
                    return Err(invalid(py, "numerical rungs belong to another runtime"));
                }
                let PreparedOperation::Modeling(prepared) = &rung.inner else {
                    return Err(invalid(py, "numerical rung requires an algebraic solve"));
                };
                Ok(prepared.solve.clone().into())
            })
            .collect::<PyResult<Vec<_>>>()?;
        let mut prepared = p.as_ref().clone();
        prepared.solve = prepared
            .solve
            .with_strategy(declaration.strategy, rungs)
            .map_err(|error| errors::diagnostic(py, &error))?;
        Ok(Self {
            owner: self.owner.clone(),
            inner: PreparedOperation::Modeling(Box::new(prepared)),
        })
    }
    fn with_start(&self, py: Python<'_>, seed: &NativeStart) -> PyResult<Self> {
        let PreparedOperation::Modeling(p) = &self.inner else {
            return Err(invalid(py, "native warm starts require an algebraic solve"));
        };
        let inner = p
            .as_ref()
            .clone()
            .with_start(seed.inner.clone())
            .map_err(|e| errors::diagnostic(py, &e))?;
        Ok(Self {
            owner: self.owner.clone(),
            inner: PreparedOperation::Modeling(Box::new(inner)),
        })
    }
    fn with_primal_start(
        &self,
        py: Python<'_>,
        values: std::collections::BTreeMap<String, f64>,
    ) -> PyResult<Self> {
        let PreparedOperation::Modeling(p) = &self.inner else {
            return Err(invalid(py, "native warm starts require an algebraic solve"));
        };
        let values = values
            .into_iter()
            .map(|(key, value)| id(py, &key).map(|id| (id, value)))
            .collect::<PyResult<_>>()?;
        let inner = p
            .as_ref()
            .clone()
            .with_primal_start(values)
            .map_err(|e| errors::diagnostic(py, &e))?;
        Ok(Self {
            owner: self.owner.clone(),
            inner: PreparedOperation::Modeling(Box::new(inner)),
        })
    }
    #[getter]
    fn identity(&self, py: Python<'_>) -> PyResult<String> {
        Ok(match &self.inner {
            PreparedOperation::Modeling(p) => p
                .solve
                .request_identity()
                .map_err(|e| errors::diagnostic(py, &e))?
                .as_id(),
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
