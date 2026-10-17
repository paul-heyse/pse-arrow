// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Mechanical public workflow projection; all mathematical policy remains native.
use crate::documents;
mod modeling;
use documents::DocumentContent;
mod routes;
mod settings;
mod source_settlement;
pub(crate) use modeling::{
    ModelingDiagnosticSettings, ModelingLimits, NativeLocalStudy, NativeModelingConformance,
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

/// Observe interpreter configuration outside the bounded Rust reconstruction scope.
/// No Python callback or import runs while the native loader lock is held.
#[cfg(Py_GIL_DISABLED)]
fn local_python_configuration(
    _anchor: usize,
) -> std::io::Result<pse_runtime::math::portable::RuntimeConfigurationObservation> {
    Err(std::io::Error::other(
        "local replay requires a genuine GIL environment owner; free-threaded Python is unsupported",
    ))
}
#[cfg(not(Py_GIL_DISABLED))]
fn local_python_configuration(
    _anchor: usize,
) -> std::io::Result<pse_runtime::math::portable::RuntimeConfigurationObservation> {
    // The abi3 boundary exposes no protected interpreter inventory or process-wide
    // environment owner. A main GIL cannot exclude another interpreter's writers.
    // Refuse local evidence before borrowing libc; startup retains the safe fresh
    // path and independently qualified deployment receipts remain available.
    Err(std::io::Error::other(
        "local replay requires a process-wide environment owner unavailable through the supported Python ABI",
    ))
}

/// Original-intent settlement only; never grants active content access.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeArtifactSettlement {
    inner: native::artifacts::ArtifactSettlement,
    owner: Arc<runtime::Runtime>,
}
#[pymethods]
impl NativeArtifactSettlement {
    #[getter]
    fn key(&self) -> String {
        self.inner.key().into()
    }
    fn release(&self, py: Python<'_>, kind: &str, owner: &str) -> PyResult<()> {
        blocking(py, &self.owner, self.inner.release(kind, owner), || {})
    }
    fn collect(&self, py: Python<'_>) -> PyResult<bool> {
        blocking(py, &self.owner, self.inner.collect(), || {})
    }
}

/// Independent retained scientific content; producer availability is not consulted.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeScientificArtifact {
    inner: native::artifacts::ScientificArtifact,
    owner: Arc<runtime::Runtime>,
}
#[pymethods]
impl NativeScientificArtifact {
    #[getter]
    fn key(&self) -> String {
        self.inner.header().key.clone()
    }
    #[getter]
    fn kind(&self) -> String {
        self.inner.header().kind.clone()
    }
    /// Original immutable recovery document, never a content-acquisition capability.
    #[getter]
    fn original_intent(&self, py: Python<'_>) -> PyResult<Vec<u8>> {
        documents::encode(py, self.inner.header())
    }
    fn metadata(&self, py: Python<'_>) -> PyResult<inspection::TableStream> {
        inspection::TableStream::from_batch(blocking(
            py,
            &self.owner,
            self.inner.metadata(),
            || {},
        )?)
        .map_err(|error| errors::diagnostic(py, &error))
    }
    fn block(&self, py: Python<'_>, ordinal: u64) -> PyResult<inspection::TableStream> {
        let cancel = pse_columnar::CancellationToken::new();
        inspection::TableStream::from_batch(blocking(
            py,
            &self.owner,
            self.inner.block(ordinal, cancel.clone()),
            || cancel.cancel(),
        )?)
        .map_err(|error| errors::diagnostic(py, &error))
    }
    fn retain(&self, py: Python<'_>, kind: &str, owner: &str) -> PyResult<()> {
        blocking(py, &self.owner, self.inner.retain(kind, owner), || {})
    }
    fn release(&self, py: Python<'_>, kind: &str, owner: &str) -> PyResult<()> {
        blocking(py, &self.owner, self.inner.release(kind, owner), || {})
    }
    fn collect(&self, py: Python<'_>) -> PyResult<bool> {
        blocking(py, &self.owner, self.inner.collect(), || {})
    }
}

/// Compact exact persisted graph; every page uses the runtime's retained pool.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeAnalysis {
    inner: native::AnalysisHandle,
    owner: Arc<runtime::Runtime>,
}
#[pymethods]
impl NativeAnalysis {
    #[getter]
    fn key(&self) -> String {
        self.inner.key().into()
    }
    fn header(&self, py: Python<'_>) -> PyResult<inspection::TableStream> {
        inspection::TableStream::from_batch(blocking(py, &self.owner, self.inner.header(), || {})?)
            .map_err(|error| errors::diagnostic(py, &error))
    }
    #[pyo3(signature=(*,after=None))]
    fn nodes(&self, py: Python<'_>, after: Option<&str>) -> PyResult<inspection::TableStream> {
        inspection::TableStream::from_batch(blocking(
            py,
            &self.owner,
            self.inner.nodes(after),
            || {},
        )?)
        .map_err(|error| errors::diagnostic(py, &error))
    }
    #[pyo3(signature=(*,after=None))]
    fn edges(&self, py: Python<'_>, after: Option<&str>) -> PyResult<inspection::TableStream> {
        inspection::TableStream::from_batch(blocking(
            py,
            &self.owner,
            self.inner.edges(after),
            || {},
        )?)
        .map_err(|error| errors::diagnostic(py, &error))
    }
}

/// Compact exact persisted selection; reopening does not retain a worker report.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeStoredResult {
    pub(crate) inner: native::Runtime,
    pub(crate) owner: Arc<runtime::Runtime>,
    pub(crate) run_id: pse_model::generated::identities::RunId,
    pub(crate) run: String,
    pub(crate) attempt: String,
}
#[pymethods]
impl NativeStoredResult {
    #[getter]
    fn run_key(&self) -> String {
        self.run.clone()
    }
    #[getter]
    fn attempt_key(&self) -> String {
        self.attempt.clone()
    }
    #[getter]
    fn run_id(&self) -> String {
        self.run_id.as_id().to_hex()
    }
    #[getter]
    fn usable(&self, py: Python<'_>) -> PyResult<bool> {
        let stored = blocking(
            py,
            &self.owner,
            self.inner.stored_completion(&self.run, &self.attempt),
            || {},
        )?;
        Ok(matches!(
            stored.termination.cause,
            native::TerminationCause::Assessment { usable: true, .. }
        ) && stored.completion.as_ref().is_some_and(|completion| {
            !completion.assessments.is_empty()
                && completion
                    .assessments
                    .iter()
                    .all(|assessment| assessment.permits_result)
        }))
    }
    fn completion(&self, py: Python<'_>) -> PyResult<Vec<u8>> {
        let stored = blocking(
            py,
            &self.owner,
            self.inner.stored_completion(&self.run, &self.attempt),
            || {},
        )?;
        documents::encode(
            py,
            stored
                .completion
                .as_ref()
                .ok_or_else(|| invalid(py, "stored attempt has no scientific completion"))?,
        )
    }
    fn diagnostics(&self, py: Python<'_>) -> PyResult<Vec<errors::DiagnosticReport>> {
        let stored = blocking(
            py,
            &self.owner,
            self.inner.stored_completion(&self.run, &self.attempt),
            || {},
        )?;
        Ok(match &stored.termination.cause {
            native::TerminationCause::Error { diagnostic }
            | native::TerminationCause::Infrastructure { diagnostic } => {
                vec![inspection::DiagnosticReport::observe(diagnostic)]
            }
            native::TerminationCause::Assessment { diagnostics, .. } => diagnostics
                .iter()
                .map(inspection::DiagnosticReport::observe)
                .collect(),
        })
    }
    #[pyo3(signature=(relation,*,start=0,end=u64::MAX))]
    fn table(
        &self,
        py: Python<'_>,
        relation: &str,
        start: u64,
        end: u64,
    ) -> PyResult<inspection::TableStream> {
        let cancel = pse_columnar::CancellationToken::new();
        let reader = blocking(
            py,
            &self.owner,
            self.inner.results(
                &self.run,
                &self.attempt,
                relation,
                start,
                end,
                cancel.clone(),
            ),
            || cancel.cancel(),
        )?;
        Ok(inspection::TableStream::from_canonical(
            reader,
            self.owner.clone(),
        ))
    }
    fn attempt_record(&self, py: Python<'_>) -> PyResult<inspection::TableStream> {
        inspection::TableStream::from_batch(blocking(
            py,
            &self.owner,
            self.inner.attempt_record(&self.attempt),
            || {},
        )?)
        .map_err(|error| errors::diagnostic(py, &error))
    }
    fn progress(&self, py: Python<'_>) -> PyResult<NativeProgressStream> {
        let cancel = pse_columnar::CancellationToken::new();
        let stream = blocking(
            py,
            &self.owner,
            self.inner
                .progress(&self.run, &self.attempt, cancel.clone()),
            || cancel.cancel(),
        )?;
        Ok(NativeProgressStream {
            owner: self.owner.clone(),
            run_key: self.run.clone(),
            attempt_key: self.attempt.clone(),
            coverage: stream.coverage().clone(),
            cancel,
            slot: Mutex::new(ProgressSlot::Idle(Box::new(ProgressState {
                stream,
                buffered: std::collections::VecDeque::new(),
            }))),
        })
    }
    #[pyo3(signature=(relation,destination,*,start=0,end=u64::MAX))]
    fn export(
        &self,
        py: Python<'_>,
        relation: &str,
        destination: &str,
        start: u64,
        end: u64,
    ) -> PyResult<()> {
        let cancel = pse_columnar::CancellationToken::new();
        blocking(
            py,
            &self.owner,
            async {
                let mut reader = self
                    .inner
                    .results(
                        &self.run,
                        &self.attempt,
                        relation,
                        start,
                        end,
                        cancel.clone(),
                    )
                    .await?;
                reader.export_ipc(std::path::Path::new(destination)).await
            },
            || cancel.cancel(),
        )
    }
}

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
    blocking_on_with_recovery(py, executor, future, cancel, |_| None)
}
/// Own a synchronous native pull through signal observation, cancellation and join.
/// The callback never attaches to Python; exports happen only after its drain.
pub(crate) fn blocking_native_call<T: Send>(
    py: Python<'_>,
    work: impl FnOnce() -> Result<T, pse_engine::EngineError> + Send,
    cancel: impl Fn(),
) -> PyResult<T> {
    py.check_signals()?;
    std::thread::scope(|scope| {
        let (sender, mut receiver) = std::sync::mpsc::sync_channel(1);
        let worker = std::thread::Builder::new()
            .name("pse-python-pull".into())
            .spawn_scoped(scope, move || {
                let _ = sender.send(work());
            })
            .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(error.to_string()))?;
        let mut signal = None;
        let value = loop {
            let receiving = &mut receiver;
            let ready = py.detach(move || receiving.recv_timeout(Duration::from_millis(100)));
            match ready {
                Ok(value) => break Some(value),
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break None,
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            }
            if signal.is_none()
                && let Err(error) = py.check_signals()
            {
                cancel();
                signal = Some(error);
            }
        };
        let joined = py.detach(|| worker.join());
        if signal.is_none()
            && let Err(error) = py.check_signals()
        {
            cancel();
            signal = Some(error);
        }
        if let Some(error) = signal {
            if error.cause(py).is_none()
                && let Some(Err(cause)) = &value
            {
                error.set_cause(py, Some(errors::diagnostic(py, cause)));
            }
            return Err(error);
        }
        if joined.is_err() {
            return Err(pyo3::exceptions::PyRuntimeError::new_err(
                "native Arrow pull panicked after drain",
            ));
        }
        value
            .ok_or_else(|| {
                pyo3::exceptions::PyRuntimeError::new_err("native Arrow pull returned no outcome")
            })?
            .map_err(|error| errors::diagnostic(py, &error))
    })
}
fn blocking_analysis<
    F: Future<Output = Result<native::AnalysisHandle, native::WorkflowError>> + Send,
>(
    py: Python<'_>,
    runtime: &runtime::Runtime,
    future: F,
    cancel: impl Fn(),
) -> PyResult<native::AnalysisHandle> {
    blocking_on_with_recovery(py, runtime.executor, future, cancel, |value| {
        Some(("analysis_key", value.key().to_owned(), None))
    })
}
fn blocking_on_with_recovery<
    T: Send,
    F: Future<Output = Result<T, native::WorkflowError>> + Send,
>(
    py: Python<'_>,
    executor: &tokio::runtime::Runtime,
    future: F,
    cancel: impl Fn(),
    recovery_key: impl Fn(
        &T,
    ) -> Option<(
        &'static str,
        String,
        Option<pse_operations::canonical_artifacts::Artifact>,
    )>,
) -> PyResult<T> {
    blocking_on_with_interrupted_value(py, executor, future, cancel, recovery_key, |_| {})
}
fn blocking_on_with_interrupted_value<
    T: Send,
    F: Future<Output = Result<T, native::WorkflowError>> + Send,
>(
    py: Python<'_>,
    executor: &tokio::runtime::Runtime,
    future: F,
    cancel: impl Fn(),
    recovery_key: impl Fn(
        &T,
    ) -> Option<(
        &'static str,
        String,
        Option<pse_operations::canonical_artifacts::Artifact>,
    )>,
    interrupted_value: impl Fn(&T),
) -> PyResult<T> {
    if tokio::runtime::Handle::try_current().is_ok() {
        return Err(invalid(
            py,
            "blocking workflow cannot re-enter its async executor",
        ));
    }
    let mut future = Box::pin(future);
    let mut signal: Option<PyErr> = None;
    loop {
        let ready = py.detach(|| {
            executor.block_on(async {
                tokio::time::timeout(Duration::from_millis(100), &mut future).await
            })
        });
        if let Ok(value) = ready {
            // Observe a signal that arrived during the final (possibly first)
            // poll before Python can discard the acknowledged result on return.
            if signal.is_none()
                && let Err(error) = py.check_signals()
            {
                cancel();
                signal = Some(error);
            }
            return match signal {
                Some(e) => {
                    use errors::ReportSource;
                    match &value {
                        Ok(value) => {
                            interrupted_value(value);
                            if let Some((field, key, intent)) = recovery_key(value) {
                                e.value(py).setattr(field, key)?;
                                if let Some(intent) = intent {
                                    e.value(py).setattr(
                                        "artifact_intent",
                                        documents::encode(py, &intent)?,
                                    )?;
                                }
                            }
                        }
                        Err(error) => {
                            for (field, key) in [
                                ("analysis_key", error.report_analysis_key()),
                                ("artifact_key", error.report_artifact_key()),
                                ("study_key", error.report_study_key()),
                                ("source_problem", error.report_source_problem()),
                                ("source_operation", error.report_source_operation()),
                            ] {
                                if let Some(key) = key {
                                    e.value(py).setattr(field, key)?;
                                }
                            }
                            if let Some(intent) = error.report_artifact_intent() {
                                e.value(py)
                                    .setattr("artifact_intent", documents::encode(py, &intent)?)?;
                            }
                            if let Some((kind, owner, read)) = error.report_artifact_cleanup() {
                                e.value(py).setattr("artifact_owner_kind", kind)?;
                                e.value(py).setattr("artifact_owner", owner)?;
                                e.value(py).setattr("artifact_read_key", read)?;
                            }
                        }
                    }
                    if e.cause(py).is_none()
                        && let Err(error) = &value
                    {
                        e.set_cause(py, Some(errors::diagnostic(py, error)));
                    }
                    Err(e)
                }
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
/// Borrow the shared scientific deployment budget, services and executor.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeRuntime {
    owner: Arc<runtime::Runtime>,
    inner: native::Runtime,
    recording: Option<Arc<DeferredCanonical>>,
}

#[derive(Debug)]
struct DeferredCanonical {
    substrate: std::path::PathBuf,
    database: Option<String>,
    producer: Option<std::path::PathBuf>,
    artifact: std::path::PathBuf,
    connected: tokio::sync::OnceCell<native::Runtime>,
}

impl NativeRuntime {
    async fn configured_runtime(
        &self,
        stop: &CancelSource,
    ) -> Result<native::Runtime, native::WorkflowError> {
        if self.inner.canonical().is_ok() {
            return Ok(self.inner.clone());
        }
        let configured = self
            .recording
            .as_ref()
            .ok_or(native::WorkflowError::RecordingUnavailable)?;
        configured
            .connected
            .get_or_try_init(|| self.open_recording(configured, stop))
            .await
            .cloned()
    }
    async fn recording_operation<T, F, Fut>(
        &self,
        stop: &CancelSource,
        operation: F,
    ) -> Result<T, native::WorkflowError>
    where
        F: FnOnce(native::Runtime) -> Fut,
        Fut: Future<Output = Result<T, native::WorkflowError>>,
    {
        self.inner
            .scope_activity(async {
                let recording = self.configured_runtime(stop).await?;
                operation(recording).await
            })
            .await
    }
    fn recording_runtime(&self, py: Python<'_>) -> PyResult<native::Runtime> {
        let stop = CancelSource::new();
        blocking(
            py,
            &self.owner,
            self.recording_operation(&stop, |recording| async { Ok(recording) }),
            || stop.cancel(),
        )
    }
    async fn open_recording(
        &self,
        configured: &DeferredCanonical,
        stop: &CancelSource,
    ) -> Result<native::Runtime, native::WorkflowError> {
        use pse_runtime::math::portable::{PortableError, ReplayAdmission};
        let typed = |error: PortableError| {
            native::WorkflowError::Typed(pse_model::diagnostic::DiagnosticCause::new(
                pse_model::diagnostic::project_typed(
                    &error,
                    pse_diagnostics::DiagnosticStage::ModelingAdmission,
                ),
            ))
        };
        let deadline = std::time::Instant::now()
            .checked_add(ReplayAdmission::STARTUP_OBSERVATION_LIMIT)
            .ok_or_else(|| {
                native::WorkflowError::Input("deployment startup deadline extent".into())
            })?;
        // The loaded image, not a path saved at construction, identifies this integration.
        // Revalidate immediately before and after receiving observation: a rebuilt or
        // deleted mapped artifact cannot acquire the replacement file's qualification.
        let verify_loaded = || -> Result<(), native::WorkflowError> {
            let loaded = pse_buildinfo::loaded_module_path(Self::new as *const () as usize)
                .map_err(|error| {
                    native::WorkflowError::Input(format!(
                        "actual loaded module association: {error}"
                    ))
                })?;
            if loaded != configured.artifact {
                return Err(native::WorkflowError::Input(
                    "actual loaded module differs from configured recording integration".into(),
                ));
            }
            Ok(())
        };
        verify_loaded()?;
        let math = self.owner.shared.math();
        #[allow(
            unsafe_code,
            reason = "actual imported composition root qualifies its requested recording/import integration"
        )]
        // SAFETY: artifact is verified against this actual imported module at construction;
        // the operator-selected receipt retains the existing deployment trust contract.
        let startup = unsafe {
            math.observe_deployment_runtime(
                configured.artifact.clone(),
                configured.producer.clone(),
                pse_runtime::math::portable::ExpectedProducerTarget::PYTHON,
                deadline,
                stop,
            )
        }
        .await
        .map_err(typed)?;
        let producer = if configured.producer.is_some() {
            startup.producer.clone().map(Into::into)
        } else {
            let anchor = Self::new as *const () as usize;
            #[allow(
                unsafe_code,
                reason = "actual imported composition root retains controlled receiving qualification at import integration"
            )]
            // SAFETY: callback observes this composition root under the existing controlled
            // profile, which refuses unavailable process-wide Python ownership.
            let observed = unsafe {
                math.observe_local_runtime(
                    pse_runtime::math::portable::ExpectedProducerTarget::PYTHON,
                    anchor,
                    Arc::new(move || local_python_configuration(anchor)),
                    deadline,
                    stop,
                )
            }
            .await;
            match observed {
                Ok(producer) => Some(producer),
                Err(PortableError::Qualification(_)) => None,
                Err(error) => return Err(typed(error)),
            }
        };
        verify_loaded()?;
        let checkpoint = || {
            stop.checkpoint().map_err(|_| {
                native::WorkflowError::Math(pse_runtime::math::MathRuntimeError::Cancelled)
            })
        };
        checkpoint()?;
        let options = pse_operations::canonical::CanonicalOptions::from_state_for_database(
            &configured.substrate,
            configured.database.as_deref(),
        )?;
        let store = pse_operations::canonical::CanonicalStore::connect(&options).await?;
        checkpoint()?;
        store.open().await?;
        checkpoint()?;
        // `startup` retains only the escaped observation, including through a
        // slow or failed store open; native receipt/namespace scratch has drained.
        self.inner.with_canonical(native::CanonicalDeployment::new(
            store,
            native::OuterAttestation {
                source: startup.outer.source,
                build: startup.outer.build,
            },
            producer,
        ))
    }
}
#[pymethods]
impl NativeRuntime {
    /// Release this caller and its aliases while admitted work drains independently.
    fn close(&self) {
        self.inner.close();
    }

    /// Return passive diagnostic authority without retaining this runtime caller.
    fn observations(&self) -> crate::diagnostics::NativeDiagnostics {
        crate::diagnostics::NativeDiagnostics::new(self.inner.observations().clone())
    }

    /// Observe the original source acknowledgement without issuing another edit.
    fn source_settlement(
        &self,
        py: Python<'_>,
        problem: &str,
        operation: &str,
    ) -> PyResult<documents::DocumentValue<native::SourceSettlement>> {
        self.source_settlement_native(py, problem, operation)
    }
    /// Reopen an exact retained canonical source selection without authored document replay.
    fn modeling_revision(
        &self,
        py: Python<'_>,
        revision: &str,
        physical: &NativePhysicalContext,
    ) -> PyResult<NativeModelingPackage> {
        let connection_stop = CancelSource::new();
        let inner = blocking(
            py,
            &self.owner,
            self.recording_operation(&connection_stop, |recording| async move {
                (async {
                    let revision = recording
                        .canonical_store()?
                        .revision(revision)
                        .await?
                        .ok_or_else(|| {
                            native::WorkflowError::Input(
                                "canonical modeling revision absent".into(),
                            )
                        })?;
                    recording
                        .modeling_revision(revision, physical.inner.clone(), Default::default())
                        .await
                })
                .await
            }),
            || connection_stop.cancel(),
        )?;
        Ok(NativeModelingPackage::from_revision(
            self.owner.clone(),
            inner,
        ))
    }
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
    /// Explicit database/schema installation after the test context is registered.
    #[staticmethod]
    #[pyo3(signature = (settings, *, substrate, database))]
    fn initialize_database(
        py: Python<'_>,
        settings: &inspection::EngineSettings,
        substrate: &str,
        database: &str,
    ) -> PyResult<()> {
        let owner = py
            .detach(|| runtime::acquire(settings))
            .map_err(|error| errors::diagnostic(py, &error))?;
        let options = pse_operations::canonical::CanonicalOptions::from_state_for_database(
            std::path::Path::new(substrate),
            Some(database),
        )
        .map_err(|error| errors::diagnostic(py, &error))?;
        blocking(
            py,
            &owner,
            async {
                pse_operations::canonical::CanonicalStore::initialize_database(&options)
                    .await
                    .map_err(native::WorkflowError::Canonical)
            },
            || {},
        )
    }
    /// Canonical durable deployment; ephemeral execution is an explicit local choice.
    #[new]
    #[pyo3(signature = (settings, *, substrate=None, database=None, producer=None, ephemeral=false))]
    fn new(
        py: Python<'_>,
        settings: &inspection::EngineSettings,
        substrate: Option<&str>,
        database: Option<&str>,
        producer: Option<&str>,
        ephemeral: bool,
    ) -> PyResult<Self> {
        if substrate.is_none() && (!ephemeral || database.is_some() || producer.is_some()) {
            return Err(invalid(
                py,
                "a canonical substrate is required for durable construction, database or producer selection",
            ));
        }
        let owner = py
            .detach(|| runtime::acquire(settings))
            .map_err(|error| errors::diagnostic(py, &error))?;
        let actual = pse_buildinfo::loaded_module_path(Self::new as *const () as usize)
            .map_err(|error| invalid(py, error.to_string()))?;
        let imported: std::path::PathBuf =
            py.import("pse._native")?.getattr("__file__")?.extract()?;
        if imported
            .canonicalize()
            .map_err(|error| invalid(py, error.to_string()))?
            != actual
                .canonicalize()
                .map_err(|error| invalid(py, error.to_string()))?
        {
            return Err(invalid(
                py,
                "Python import does not identify the actual loaded native module",
            ));
        }
        let inner = native::Runtime::from_shared_local(owner.shared.clone(), owner.model.clone())
            .map_err(|error| errors::diagnostic(py, &error))?;
        let recording = substrate.map(|substrate| {
            Arc::new(DeferredCanonical {
                substrate: substrate.into(),
                database: database.map(str::to_owned),
                producer: producer.map(Into::into),
                artifact: actual,
                connected: tokio::sync::OnceCell::new(),
            })
        });
        let mut runtime = Self {
            owner,
            inner,
            recording,
        };
        if !ephemeral {
            runtime.inner = runtime
                .recording_runtime(py)?
                .with_durability(native::DurabilitySelection::Durable {
                    worker: native::Operations::process_worker("runtime"),
                    policy: native::LeasePolicy::default(),
                })
                .map_err(|error| errors::diagnostic(py, &error))?;
        }
        Ok(runtime)
    }
    /// Whether runs durably record canonical terminal observations.
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
            self.inner.scope_activity(async {
                let pool = self.owner.shared.pool();
                let token = cancel.token();
                let validation = self.inner.admission_context();
                let bundle =
                    pse_admission::authoring_driver::document::load_package_documents_owned(
                        &documents,
                        self.owner.model.registry(),
                        pse_authoring::ParseBudget::default(),
                        &pool,
                        &token,
                        &validation,
                    )?;
                let owned =
                    pse_admission::authoring_driver::document::OwnedDocumentSet::try_from_bundles(
                        vec![bundle],
                        &pool,
                        &token,
                    )?;
                self.inner.physical_from_documents(&owned, &token).await
            }),
            || cancel.cancel(),
        )?;
        Ok(NativePhysicalContext { inner })
    }
    /// Exact canonical run metadata through its generated one-row Arrow contract.
    fn run_record(&self, py: Python<'_>, run: &str) -> PyResult<inspection::TableStream> {
        let connection_stop = CancelSource::new();
        let batch = blocking(
            py,
            &self.owner,
            self.recording_operation(&connection_stop, |recording| async move {
                (recording.run_record(run)).await
            }),
            || connection_stop.cancel(),
        )?;
        inspection::TableStream::from_batch(batch).map_err(|error| errors::diagnostic(py, &error))
    }
    /// Exact attempt metadata and recorded terminal class.
    fn attempt_record(&self, py: Python<'_>, attempt: &str) -> PyResult<inspection::TableStream> {
        let connection_stop = CancelSource::new();
        let batch = blocking(
            py,
            &self.owner,
            self.recording_operation(&connection_stop, |recording| async move {
                (recording.attempt_record(attempt)).await
            }),
            || connection_stop.cancel(),
        )?;
        inspection::TableStream::from_batch(batch).map_err(|error| errors::diagnostic(py, &error))
    }
    /// Exact protected admitted manifest, projected by its registry schema.
    fn result_manifest(&self, py: Python<'_>, attempt: &str) -> PyResult<inspection::TableStream> {
        let connection_stop = CancelSource::new();
        let batch = blocking(
            py,
            &self.owner,
            self.recording_operation(&connection_stop, |recording| async move {
                (recording.result_manifest(attempt)).await
            }),
            || connection_stop.cancel(),
        )?;
        inspection::TableStream::from_batch(batch).map_err(|error| errors::diagnostic(py, &error))
    }
    fn recover_artifact_intent(
        &self,
        py: Python<'_>,
        intent: &[u8],
    ) -> PyResult<NativeArtifactSettlement> {
        let connection_stop = CancelSource::new();
        let inner = blocking(
            py,
            &self.owner,
            self.recording_operation(&connection_stop, |recording| async move {
                recording.recover_artifact_intent(intent)
            }),
            || connection_stop.cancel(),
        )?;
        Ok(NativeArtifactSettlement {
            inner,
            owner: self.owner.clone(),
        })
    }
    fn scientific_artifact(&self, py: Python<'_>, key: &str) -> PyResult<NativeScientificArtifact> {
        let connection_stop = CancelSource::new();
        Ok(NativeScientificArtifact {
            inner: blocking(
                py,
                &self.owner,
                self.recording_operation(&connection_stop, |recording| async move {
                    (recording.scientific_artifact(key)).await
                }),
                || connection_stop.cancel(),
            )?,
            owner: self.owner.clone(),
        })
    }
    fn promote_seed_artifact(
        &self,
        py: Python<'_>,
        solution: &str,
        owner_kind: &str,
        owner: &str,
    ) -> PyResult<NativeScientificArtifact> {
        let connection_stop = CancelSource::new();
        let solution = pse_ids::SemanticId::parse_hex(solution)
            .map(pse_model::generated::identities::SolutionId::from_id)
            .map_err(|error| invalid(py, error.to_string()))?;
        let inner = blocking_on_with_recovery(
            py,
            self.owner.executor,
            self.recording_operation(&connection_stop, |recording| async move {
                (recording.promote_seed_artifact(solution, owner_kind, owner)).await
            }),
            || connection_stop.cancel(),
            |value| {
                Some((
                    "artifact_key",
                    value.header().key.clone(),
                    Some(value.header().clone()),
                ))
            },
        )?;
        Ok(NativeScientificArtifact {
            inner,
            owner: self.owner.clone(),
        })
    }
    fn promote_scientific_artifact(
        &self,
        py: Python<'_>,
        run: &str,
        attempt: &str,
        kind: &str,
        owner_kind: &str,
        owner: &str,
    ) -> PyResult<NativeScientificArtifact> {
        let connection_stop = CancelSource::new();
        let kind = match kind {
            "final-state" => native::artifacts::ScientificArtifactKind::FinalState,
            "trajectory" => native::artifacts::ScientificArtifactKind::Trajectory,
            "local-analysis" => native::artifacts::ScientificArtifactKind::LocalAnalysis,
            _ => return Err(invalid(py, "unknown scientific artifact kind")),
        };
        let cancel = pse_columnar::CancellationToken::new();
        let operation_cancel = cancel.clone();
        let inner = blocking_on_with_recovery(
            py,
            self.owner.executor,
            self.recording_operation(&connection_stop, |recording| async move {
                (recording.promote_scientific_artifact(
                    run,
                    attempt,
                    kind,
                    owner_kind,
                    owner,
                    operation_cancel.clone(),
                ))
                .await
            }),
            || {
                connection_stop.cancel();
                cancel.cancel();
            },
            |value| {
                Some((
                    "artifact_key",
                    value.header().key.clone(),
                    Some(value.header().clone()),
                ))
            },
        )?;
        Ok(NativeScientificArtifact {
            inner,
            owner: self.owner.clone(),
        })
    }
    fn analysis(&self, py: Python<'_>, key: &str) -> PyResult<NativeAnalysis> {
        let connection_stop = CancelSource::new();
        Ok(NativeAnalysis {
            inner: blocking(
                py,
                &self.owner,
                self.recording_operation(&connection_stop, |recording| async move {
                    (recording.analysis(key)).await
                }),
                || connection_stop.cancel(),
            )?,
            owner: self.owner.clone(),
        })
    }
    fn result_analysis(
        &self,
        py: Python<'_>,
        run: &str,
        attempt: &str,
        controls: &[u8],
    ) -> PyResult<NativeAnalysis> {
        let connection_stop = CancelSource::new();
        let controls: native::AnalysisControls =
            documents::decode(py, "analysis controls", controls, 128 * 1024)?;
        let cancel = pse_columnar::CancellationToken::new();
        let operation_cancel = cancel.clone();
        Ok(NativeAnalysis {
            inner: blocking_analysis(
                py,
                &self.owner,
                self.recording_operation(&connection_stop, |recording| async move {
                    (recording.result_analysis(run, attempt, &controls, operation_cancel.clone()))
                        .await
                }),
                || {
                    connection_stop.cancel();
                    cancel.cancel();
                },
            )?,
            owner: self.owner.clone(),
        })
    }
    fn retain_history_obligation(&self, py: Python<'_>, obligation: &[u8]) -> PyResult<()> {
        let connection_stop = CancelSource::new();
        let obligation: native::HistoryObligation =
            documents::decode(py, "history obligation", obligation, 128 * 1024)?;
        blocking(
            py,
            &self.owner,
            self.recording_operation(&connection_stop, |recording| async move {
                (recording.retain_history_obligation(&obligation)).await
            }),
            || connection_stop.cancel(),
        )
    }
    fn release_history_obligation(&self, py: Python<'_>, obligation: &[u8]) -> PyResult<()> {
        let connection_stop = CancelSource::new();
        let obligation: native::HistoryObligation =
            documents::decode(py, "history obligation", obligation, 128 * 1024)?;
        blocking(
            py,
            &self.owner,
            self.recording_operation(&connection_stop, |recording| async move {
                (recording.release_history_obligation(&obligation)).await
            }),
            || connection_stop.cancel(),
        )
    }
    fn release_history(&self, py: Python<'_>, owner: &[u8]) -> PyResult<()> {
        let connection_stop = CancelSource::new();
        let owner: native::HistoryOwner = documents::decode(py, "history owner", owner, 32 * 1024)?;
        blocking(
            py,
            &self.owner,
            self.recording_operation(&connection_stop, |recording| async move {
                (recording.release_history(&owner)).await
            }),
            || connection_stop.cancel(),
        )
    }
    fn reclaim_history_page(&self, py: Python<'_>, owner: &[u8]) -> PyResult<Vec<u8>> {
        let connection_stop = CancelSource::new();
        let owner: native::HistoryOwner = documents::decode(py, "history owner", owner, 32 * 1024)?;
        let cancel = pse_columnar::CancellationToken::new();
        let operation_cancel = cancel.clone();
        let page = blocking(
            py,
            &self.owner,
            self.recording_operation(&connection_stop, |recording| async move {
                (recording.reclaim_history_page(&owner, &operation_cancel)).await
            }),
            || {
                connection_stop.cancel();
                cancel.cancel();
            },
        )?;
        documents::encode(py, &page)
    }
    fn forget_study_results(&self, py: Python<'_>, study: &str) -> PyResult<()> {
        let connection_stop = CancelSource::new();
        blocking(
            py,
            &self.owner,
            self.recording_operation(&connection_stop, |recording| async move {
                (recording.forget_study_results(study)).await
            }),
            || connection_stop.cancel(),
        )
    }
    fn forget_analysis_results(&self, py: Python<'_>, analysis: &str) -> PyResult<bool> {
        let connection_stop = CancelSource::new();
        blocking(
            py,
            &self.owner,
            self.recording_operation(&connection_stop, |recording| async move {
                (recording.forget_analysis_results(analysis)).await
            }),
            || connection_stop.cancel(),
        )
    }
    fn reclaim_run_results(&self, py: Python<'_>, run: &str) -> PyResult<Vec<u8>> {
        let connection_stop = CancelSource::new();
        let cancel = pse_columnar::CancellationToken::new();
        let operation_cancel = cancel.clone();
        let page = blocking(
            py,
            &self.owner,
            self.recording_operation(&connection_stop, |recording| async move {
                (recording.reclaim_run_results(run, &operation_cancel)).await
            }),
            || {
                connection_stop.cancel();
                cancel.cancel();
            },
        )?;
        documents::encode(py, &page)
    }
    /// Newest semantic run under explicit recorded result classes.
    #[pyo3(signature=(problem,relation,*,classes=Vec::new(),start=0,end=u64::MAX))]
    fn latest_results(
        &self,
        py: Python<'_>,
        problem: &str,
        relation: &str,
        classes: Vec<String>,
        start: u64,
        end: u64,
    ) -> PyResult<inspection::TableStream> {
        let connection_stop = CancelSource::new();
        let classes = classes
            .into_iter()
            .map(|class| {
                serde_json::from_value::<pse_operations::canonical_execution::TerminalClass>(
                    serde_json::Value::String(class),
                )
                .map_err(|error| invalid(py, error.to_string()))
            })
            .collect::<PyResult<Vec<_>>>()?;
        let cancel = pse_columnar::CancellationToken::new();
        let operation_cancel = cancel.clone();
        let reader = blocking(
            py,
            &self.owner,
            self.recording_operation(&connection_stop, |recording| async move {
                (recording.latest_results(
                    problem,
                    &classes,
                    relation,
                    start,
                    end,
                    operation_cancel.clone(),
                ))
                .await
            }),
            || {
                connection_stop.cancel();
                cancel.cancel();
            },
        )?;
        Ok(inspection::TableStream::from_canonical(
            reader,
            self.owner.clone(),
        ))
    }
    /// Stream one exact canonical run/attempt's scientific relation after restart.
    /// Half-open row coordinates select recorded coverage without substituting
    /// another attempt's values. Late failures remain on the Arrow stream.
    #[pyo3(signature = (run,attempt,relation,*,start=0,end=u64::MAX))]
    fn results(
        &self,
        py: Python<'_>,
        run: &str,
        attempt: &str,
        relation: &str,
        start: u64,
        end: u64,
    ) -> PyResult<inspection::TableStream> {
        let connection_stop = CancelSource::new();
        let cancel = pse_columnar::CancellationToken::new();
        let operation_cancel = cancel.clone();
        let reader = blocking(
            py,
            &self.owner,
            self.recording_operation(&connection_stop, |recording| async move {
                (recording.results(run, attempt, relation, start, end, operation_cancel.clone()))
                    .await
            }),
            || {
                connection_stop.cancel();
                cancel.cancel();
            },
        )?;
        Ok(inspection::TableStream::from_canonical(
            reader,
            self.owner.clone(),
        ))
    }
    #[pyo3(signature=(run,attempt,relation,output,field,*,partition="0",start=0,end=u64::MAX,minimum=None,maximum=None,missing=None))]
    #[allow(
        clippy::too_many_arguments,
        reason = "The Python signature exposes the exact attempt, output, range and predicate selectors directly."
    )]
    fn output_results(
        &self,
        py: Python<'_>,
        run: &str,
        attempt: &str,
        relation: &str,
        output: &str,
        field: &str,
        partition: &str,
        start: u64,
        end: u64,
        minimum: Option<f64>,
        maximum: Option<f64>,
        missing: Option<bool>,
    ) -> PyResult<inspection::TableStream> {
        let connection_stop = CancelSource::new();
        let output = id(py, output)?;
        let cancel = pse_columnar::CancellationToken::new();
        let operation_cancel = cancel.clone();
        let reader = blocking(
            py,
            &self.owner,
            self.recording_operation(&connection_stop, |recording| async move {
                (recording.output_results(
                    run,
                    attempt,
                    relation,
                    output,
                    field,
                    partition,
                    start,
                    end,
                    minimum,
                    maximum,
                    missing,
                    operation_cancel.clone(),
                ))
                .await
            }),
            || {
                connection_stop.cancel();
                cancel.cancel();
            },
        )?;
        Ok(inspection::TableStream::from_canonical(
            reader,
            self.owner.clone(),
        ))
    }
    /// Export the exact selected results, publishing the final IPC path only
    /// after clean stream completion. Interrupted files retain `.incomplete`.
    #[pyo3(signature=(run,attempt,relation,destination,*,start=0,end=u64::MAX))]
    #[allow(
        clippy::too_many_arguments,
        reason = "The Python signature keeps the destination and exact attempt/range selectors explicit."
    )]
    fn export_results(
        &self,
        py: Python<'_>,
        run: &str,
        attempt: &str,
        relation: &str,
        destination: &str,
        start: u64,
        end: u64,
    ) -> PyResult<()> {
        let connection_stop = CancelSource::new();
        let cancel = pse_columnar::CancellationToken::new();
        let operation_cancel = cancel.clone();
        blocking(
            py,
            &self.owner,
            self.recording_operation(&connection_stop, |recording| async move {
                (async {
                    // Keep the two large native phase futures out of this
                    // Python-thread polling frame. Their owned readers and
                    // cancellation/drain contract remain unchanged.
                    let mut reader = Box::pin(recording.results(
                        run,
                        attempt,
                        relation,
                        start,
                        end,
                        operation_cancel.clone(),
                    ))
                    .await?;
                    Box::pin(reader.export_ipc(std::path::Path::new(destination))).await
                })
                .await
            }),
            || {
                connection_stop.cancel();
                cancel.cancel();
            },
        )
    }
    /// Read one admitted terminal attempt's exact recorded progress history.
    #[pyo3(signature = (run, attempt))]
    fn progress(&self, py: Python<'_>, run: &str, attempt: &str) -> PyResult<NativeProgressStream> {
        let connection_stop = CancelSource::new();
        let cancel = pse_columnar::CancellationToken::new();
        let operation_cancel = cancel.clone();
        let stream = blocking(
            py,
            &self.owner,
            self.recording_operation(&connection_stop, |recording| async move {
                (recording.progress(run, attempt, operation_cancel.clone())).await
            }),
            || {
                connection_stop.cancel();
                cancel.cancel();
            },
        )?;
        Ok(NativeProgressStream {
            owner: self.owner.clone(),
            run_key: run.into(),
            attempt_key: attempt.into(),
            coverage: stream.coverage().clone(),
            cancel,
            slot: Mutex::new(ProgressSlot::Idle(Box::new(ProgressState {
                stream,
                buffered: std::collections::VecDeque::new(),
            }))),
        })
    }
    /// Serve native ready points and finalization until idle or the finite action limit.
    #[pyo3(signature = (*, maximum_actions=None, maximum_in_flight=None))]
    fn work(
        &self,
        py: Python<'_>,
        maximum_actions: Option<usize>,
        maximum_in_flight: Option<usize>,
    ) -> PyResult<usize> {
        let connection_stop = CancelSource::new();
        let stop = CancelSource::new();
        let settings = native::WorkerSettings {
            maximum_actions,
            maximum_in_flight,
            until_idle: true,
            ..native::WorkerSettings::default()
        };
        let operation_stop = stop.clone();
        let processed = blocking(
            py,
            &self.owner,
            self.recording_operation(&connection_stop, |recording| async move {
                (recording.serve(settings, &operation_stop)).await
            }),
            || {
                connection_stop.cancel();
                stop.cancel();
            },
        )?;
        Ok(processed)
    }
    /// Publish retained local inputs through this runtime's configured recording sink.
    fn record_study(&self, py: Python<'_>, local: &NativeLocalStudy) -> PyResult<Vec<u8>> {
        let cancel = CancelSource::new();
        let operation_stop = cancel.clone();
        let inner = blocking(
            py,
            &self.owner,
            self.recording_operation(&cancel, |recording| async move {
                recording.record_study(&local.inner, &operation_stop).await
            }),
            || cancel.cancel(),
        )?;
        documents::encode(py, &inner)
    }
    /// Join recorded durable execution and retain the original result report.
    #[pyo3(signature=(definition, *, controls=None))]
    fn execute_study(
        &self,
        py: Python<'_>,
        definition: &[u8],
        controls: Option<&[u8]>,
    ) -> PyResult<NativeStudyReport> {
        let definition = documents::study_definition(
            py,
            definition,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        let native::StudyRunControls { maximum_points } = documents::controls(
            py,
            "study",
            controls,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        let cancel = CancelSource::new();
        let operation_stop = cancel.clone();
        let inner = blocking(
            py,
            &self.owner,
            self.recording_operation(&cancel, |recording| async move {
                Box::pin(recording.execute_study(&definition, maximum_points, &operation_stop))
                    .await
            }),
            || cancel.cancel(),
        )?;
        Ok(NativeStudyReport {
            inner: Arc::new(inner),
            owner: self.owner.clone(),
        })
    }
    /// Start only a genuinely recorded portable study definition.
    fn start_study(&self, py: Python<'_>, definition: &[u8]) -> PyResult<NativeStudyHandle> {
        let definition = documents::study_definition(
            py,
            definition,
            self.owner.shared.budget().math.workspace_bytes,
        )?;
        let cancel = CancelSource::new();
        let operation_stop = cancel.clone();
        let inner = blocking(
            py,
            &self.owner,
            self.recording_operation(&cancel, |recording| async move {
                // Keep durable admission out of the recording/activity polling frames.
                Box::pin(recording.start_recorded_study(definition, &operation_stop)).await
            }),
            || cancel.cancel(),
        )?;
        Ok(NativeStudyHandle {
            inner,
            owner: self.owner.clone(),
        })
    }
    /// A handle on a durable study of this runtime's store.
    fn study(&self, py: Python<'_>, study_id: &str) -> PyResult<NativeStudyHandle> {
        let connection_stop = CancelSource::new();
        let study = id(py, study_id)?.into();
        let inner = blocking(
            py,
            &self.owner,
            self.recording_operation(&connection_stop, |recording| async move {
                Ok(recording.study(study))
            }),
            || connection_stop.cancel(),
        )?;
        Ok(NativeStudyHandle {
            owner: self.owner.clone(),
            inner,
        })
    }
    fn capabilities(&self, py: Python<'_>) -> PyResult<Vec<u8>> {
        documents::encode(py, &self.inner.capabilities())
    }
    fn clear_program_cache(&self) {
        self.inner.clear_program_cache();
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
    /// Read copied diagnostics independently of this handle's cancellation lease.
    fn observations(&self) -> Option<crate::diagnostics::NativeOperationObservations> {
        self.inner
            .observations()
            .map(crate::diagnostics::NativeOperationObservations::new)
    }
    fn cancel(&self) {
        self.inner.cancel();
    }
    fn result(&self) -> Option<NativeRunResult> {
        self.inner.result().map(|inner| NativeRunResult {
            inner,
            owner: self.owner.clone(),
        })
    }
    fn wait(&self, py: Python<'_>) -> PyResult<NativeRunResult> {
        let inner = blocking(py, &self.owner, self.inner.wait(), || self.inner.cancel())?;
        Ok(NativeRunResult {
            inner,
            owner: self.owner.clone(),
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
                Py::new(py, NativeRunResult { inner, owner })
            })
        })
    }
    /// Exact persistent run key; absent for explicit ephemeral execution.
    #[getter]
    fn canonical_run_key(&self) -> Option<String> {
        self.inner.canonical_run_key().map(str::to_owned)
    }
    /// Exact persistent attempt key, with no legacy semantic-ID conversion.
    #[getter]
    fn canonical_attempt_key(&self) -> Option<String> {
        self.inner.canonical_attempt_key().map(str::to_owned)
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
    fn current_progress(&self) -> documents::DocumentValue<native::ProgressCheckpointDocument> {
        documents::DocumentValue(self.inner.current_progress())
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
    inner: Arc<native::RunResult>,
    owner: Arc<runtime::Runtime>,
}
#[pymethods]
impl NativeRunResult {
    fn replay(&self) -> NativeScientificReplay {
        NativeScientificReplay {
            inner: self.inner.replay(),
            owner: self.owner.clone(),
        }
    }
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
    /// Exact persistent run key; absent for explicit ephemeral execution.
    #[getter]
    fn canonical_run_key(&self) -> Option<String> {
        self.inner.canonical_run_key().map(str::to_owned)
    }
    /// Exact persistent attempt key, with no legacy semantic-ID conversion.
    #[getter]
    fn canonical_attempt_key(&self) -> Option<String> {
        self.inner.canonical_attempt_key().map(str::to_owned)
    }
    fn completion(&self, py: Python<'_>) -> PyResult<Vec<u8>> {
        let completed = self
            .inner
            .completion()
            .map_err(|e| errors::diagnostic(py, e))?;
        documents::encode(py, completed)
    }
    /// Bounded synchronous evidence, independent of dropped progress events.
    fn attempt_summary(
        &self,
        py: Python<'_>,
    ) -> PyResult<documents::DocumentValue<Arc<native::AttemptSummary>>> {
        self.inner
            .attempt_summary()
            .map(documents::DocumentValue)
            .map_err(|error| errors::diagnostic(py, &error))
    }
    /// Joined actual invocation endpoints and existing attempt/work evidence.
    fn invocation_evidence(
        &self,
        py: Python<'_>,
    ) -> PyResult<documents::DocumentValue<pse_runtime::invocation::InvocationEvidenceDocument>>
    {
        self.inner
            .invocation_evidence()
            .map(documents::DocumentValue)
            .map_err(|error| errors::diagnostic(py, &error))
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
        Ok(py.detach(|| self.inner.table_names()))
    }
    fn table(&self, py: Python<'_>, name: &str) -> PyResult<inspection::TableStream> {
        py.detach(|| self.inner.clone().into_table_cursor(name, 1024))
            .map(inspection::TableStream::from_result)
            .map_err(|error| errors::diagnostic(py, error.as_ref()))
    }
    fn export_fit_parameters(&self, py: Python<'_>) -> PyResult<inspection::TableStream> {
        py.detach(|| self.inner.clone().into_fit_parameters_cursor(1024))
            .map(inspection::TableStream::from_result)
            .map_err(|error| errors::diagnostic(py, &error))
    }
}
/// Retained immutable inputs, exported evidence and supervised tolerant reruns.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeScientificReplay {
    inner: native::ScientificReplay,
    owner: Arc<runtime::Runtime>,
}
#[pymethods]
impl NativeScientificReplay {
    #[pyo3(signature=(bytes=16777216))]
    fn document(
        &self,
        py: Python<'_>,
        bytes: usize,
    ) -> PyResult<documents::DocumentValue<native::ReplayDocument>> {
        py.detach(|| self.inner.document_with_limit(bytes))
            .map(documents::DocumentValue)
            .map_err(|error| errors::diagnostic(py, &error))
    }
    fn rerun(&self, py: Python<'_>) -> PyResult<NativeRunHandle> {
        let cancel = CancelSource::new();
        let inner = blocking_on_with_interrupted_value(
            py,
            self.owner.executor,
            self.inner.rerun_with_cancel(&cancel),
            || cancel.cancel(),
            |_| None,
            |handle| {
                handle.cancel();
                // A launch-ready signal must join the launched run, not only its
                // preparation future. Keep the ordinary successful launch prompt.
                py.detach(|| {
                    let _ = self.owner.executor.block_on(handle.wait());
                });
            },
        )?;
        Ok(NativeRunHandle {
            owner: self.owner.clone(),
            inner,
        })
    }
    fn native_continuation(&self, py: Python<'_>) -> PyResult<NativeRunHandle> {
        self.inner
            .native_continuation()
            .map(|inner| NativeRunHandle {
                owner: self.owner.clone(),
                inner,
            })
            .map_err(|error| errors::diagnostic(py, &error))
    }
}
/// Actual admitted physical source rows and compiler context.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativePhysicalContext {
    inner: native::PhysicalContext,
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

/// A canonical study: exact occurrence status, cancellation and retained result keys.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeStudyHandle {
    owner: Arc<runtime::Runtime>,
    inner: native::StudyHandle,
}
#[pymethods]
impl NativeStudyHandle {
    /// Explicit bounded inspection of selected managed diagnostics; no scientific effects.
    fn diagnostic_selection(
        &self,
        py: Python<'_>,
    ) -> PyResult<documents::DocumentValue<native::ManagedStudyDiagnosticsDocument>> {
        blocking(py, &self.owner, self.inner.diagnostic_selection(), || {})
            .map(documents::DocumentValue)
    }
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
    /// The study's exact canonical parent and occurrence keys after admission.
    fn result(&self, py: Python<'_>) -> PyResult<Option<Vec<u8>>> {
        blocking(py, &self.owner, self.inner.result(), || {})?
            .map(|published| documents::encode(py, &published))
            .transpose()
    }
    /// Wait for the canonical parent manifest under Rust-owned timing controls.
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
                                "study {study} did not conclude within {limit:?}"
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
    run_key: String,
    attempt_key: String,
    coverage: native::ProgressCoverage,
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
    /// Exact opaque run key.
    #[getter]
    fn run_key(&self) -> &str {
        &self.run_key
    }
    /// Exact opaque terminal attempt key.
    #[getter]
    fn attempt_key(&self) -> &str {
        &self.attempt_key
    }
    /// Original terminal sampling loss and writer coverage, distinct from science.
    #[getter]
    fn coverage(&self) -> documents::DocumentValue<native::ProgressCoverage> {
        documents::DocumentValue(self.coverage.clone())
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
                state
                    .buffered
                    .extend(records.into_iter().map(documents::DocumentValue));
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
    /// Actual comparable cohort and candidates, absent when equivalence is unestablished.
    #[getter]
    fn advice_inputs(
        &self,
        py: Python<'_>,
    ) -> PyResult<Option<documents::DocumentValue<pse_runtime::math::solves::SolverAdviceInputs>>>
    {
        let PreparedOperation::Modeling(prepared) = &self.inner else {
            return Ok(None);
        };
        py.detach(|| {
            prepared
                .solve
                .advice_inputs(&Arc::new(std::sync::atomic::AtomicBool::new(false)))
        })
        .map(|inputs| inputs.map(documents::DocumentValue))
        .map_err(|error| errors::diagnostic(py, &error))
    }
    /// Freeze observed support against this owner's actual baseline, without a history lookup.
    fn freeze_advice(
        &self,
        py: Python<'_>,
        metric: &[u8],
        observations: &[u8],
    ) -> PyResult<documents::DocumentValue<pse_model::strategy::advice::FrozenAdvice>> {
        let PreparedOperation::Modeling(prepared) = &self.inner else {
            return Err(invalid(
                py,
                "this operation has no established comparable advice cohort",
            ));
        };
        let metric = documents::decode(py, "advice metric", metric, 256)?;
        let observations = documents::decode(py, "advice observations", observations, 8 << 20)?;
        let inputs = py
            .detach(|| {
                prepared
                    .solve
                    .advice_inputs(&Arc::new(std::sync::atomic::AtomicBool::new(false)))
            })
            .map_err(|error| errors::diagnostic(py, &error))?
            .ok_or_else(|| invalid(py, "original branch/profile equivalence is not established"))?;
        pse_model::strategy::advice::FrozenAdvice::freeze(
            inputs.cohort,
            metric,
            inputs.candidates,
            observations,
            inputs.limits,
        )
        .map(documents::DocumentValue)
        .map_err(|error| invalid(py, error.to_string()))
    }
    fn dependency_analysis(&self, py: Python<'_>, controls: &[u8]) -> PyResult<NativeAnalysis> {
        let PreparedOperation::Modeling(prepared) = &self.inner else {
            return Err(invalid(
                py,
                "dependency analysis currently requires an admitted algebraic preparation",
            ));
        };
        let controls: native::AnalysisControls =
            documents::decode(py, "analysis controls", controls, 128 * 1024)?;
        let cancel = pse_columnar::CancellationToken::new();
        Ok(NativeAnalysis {
            inner: blocking_analysis(
                py,
                &self.owner,
                prepared.dependency_analysis(&controls, cancel.clone()),
                || cancel.cancel(),
            )?,
            owner: self.owner.clone(),
        })
    }
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
    fn with_artifact_start(
        &self,
        py: Python<'_>,
        artifact: &NativeScientificArtifact,
    ) -> PyResult<Self> {
        let PreparedOperation::Modeling(prepared) = &self.inner else {
            return Err(invalid(
                py,
                "retained artifact starts require an algebraic solve",
            ));
        };
        let inner = blocking(
            py,
            &artifact.owner,
            artifact.inner.apply_start(prepared.as_ref()),
            || {},
        )?;
        Ok(Self {
            owner: self.owner.clone(),
            inner: PreparedOperation::Modeling(Box::new(inner)),
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
    /// Bind requested live physical output before dispatch of a prepared simulation.
    #[pyo3(signature=(*, capacity=16, compact=false))]
    fn start_live(
        &self,
        py: Python<'_>,
        capacity: usize,
        compact: bool,
    ) -> PyResult<(NativeRunHandle, NativeLiveSimulationCursor)> {
        let PreparedOperation::Simulation(simulation) = &self.inner else {
            return Err(invalid(py, "live samples require a prepared simulation"));
        };
        let (inner, cursor) = py
            .detach(|| {
                let _enter = self.owner.executor.enter();
                simulation.start_live(native::LiveSimulationOptions {
                    capacity,
                    retention: if compact {
                        pse_backend_native::dynamics::SampleRetention::Compact
                    } else {
                        pse_backend_native::dynamics::SampleRetention::Full
                    },
                })
            })
            .map_err(|error| errors::diagnostic(py, &error))?;
        Ok((
            NativeRunHandle {
                owner: self.owner.clone(),
                inner,
            },
            NativeLiveSimulationCursor {
                owner: self.owner.clone(),
                inner: Arc::new(cursor),
            },
        ))
    }
}

/// Owned subscription independent of the public run cancellation lease.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeLiveSimulationCursor {
    owner: Arc<runtime::Runtime>,
    inner: Arc<native::LiveSimulationCursor>,
}
#[pymethods]
impl NativeLiveSimulationCursor {
    fn close(&self) {
        self.inner.close();
    }
    fn next_event(&self, py: Python<'_>) -> PyResult<Option<NativeLiveSimulationEvent>> {
        let event = blocking(
            py,
            &self.owner,
            async { Ok(self.inner.next().await) },
            || self.inner.close(),
        )?;
        Ok(event.map(|inner| NativeLiveSimulationEvent {
            owner: self.owner.clone(),
            inner: Arc::new(inner),
        }))
    }
    #[pyo3(signature=() -> "typing.Awaitable[NativeLiveSimulationEvent | None]")]
    fn next_async<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let locals = pyo3_async_runtimes::tokio::get_current_locals(py)?;
        let inner = self.inner.clone();
        let owner = self.owner.clone();
        pyo3_async_runtimes::tokio::future_into_py_with_locals(py, locals, async move {
            let event = inner.next().await;
            Python::attach(|py| {
                event
                    .map(|inner| {
                        Py::new(
                            py,
                            NativeLiveSimulationEvent {
                                owner,
                                inner: Arc::new(inner),
                            },
                        )
                    })
                    .transpose()
            })
        })
    }
}
/// Typed projection keeps provisional rows, ACK and joined completion distinguishable.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeLiveSimulationEvent {
    owner: Arc<runtime::Runtime>,
    inner: Arc<native::LiveSimulationEvent>,
}
#[pymethods]
impl NativeLiveSimulationEvent {
    #[getter]
    fn kind(&self) -> &'static str {
        match self.inner.as_ref() {
            native::LiveSimulationEvent::Sample(_) => "sample",
            native::LiveSimulationEvent::AttemptFinished(_) => "attempt_finished",
            native::LiveSimulationEvent::Selected { .. } => "selected",
            native::LiveSimulationEvent::Recorded { .. } => "recorded",
            native::LiveSimulationEvent::RecordingUnknown(_) => "recording_unknown",
            native::LiveSimulationEvent::RecordingClosed => "recording_closed",
            native::LiveSimulationEvent::Joined(_) => "joined",
        }
    }
    #[getter]
    fn accuracy_attempt(&self) -> Option<u64> {
        match self.inner.as_ref() {
            native::LiveSimulationEvent::Sample(batch) => Some(batch.accuracy_attempt),
            native::LiveSimulationEvent::AttemptFinished(summary) => Some(summary.attempt),
            native::LiveSimulationEvent::Selected { accuracy_attempt } => Some(*accuracy_attempt),
            _ => None,
        }
    }
    #[getter]
    fn sample_ordinal(&self) -> Option<usize> {
        match self.inner.as_ref() {
            native::LiveSimulationEvent::Sample(batch) => Some(batch.ordinal),
            _ => None,
        }
    }
    #[getter]
    fn recorded_rows(&self) -> Option<u64> {
        match self.inner.as_ref() {
            native::LiveSimulationEvent::Recorded { rows } => Some(*rows),
            _ => None,
        }
    }
    #[getter]
    fn canonical_attempt_key(&self) -> Option<String> {
        match self.inner.as_ref() {
            native::LiveSimulationEvent::Sample(batch) => batch.canonical_attempt.clone(),
            _ => None,
        }
    }
    #[getter]
    fn completed_time(&self) -> Option<f64> {
        match self.inner.as_ref() {
            native::LiveSimulationEvent::AttemptFinished(summary) => Some(summary.completed_time),
            _ => None,
        }
    }
    #[getter]
    fn produced_samples(&self) -> Option<usize> {
        match self.inner.as_ref() {
            native::LiveSimulationEvent::AttemptFinished(summary) => {
                Some(summary.coverage.produced())
            }
            _ => None,
        }
    }
    #[getter]
    fn checks_satisfied(&self) -> Option<bool> {
        match self.inner.as_ref() {
            native::LiveSimulationEvent::AttemptFinished(summary) => Some(summary.checks_satisfied),
            _ => None,
        }
    }
    #[getter]
    fn checks_complete(&self) -> Option<bool> {
        match self.inner.as_ref() {
            native::LiveSimulationEvent::AttemptFinished(summary) => Some(summary.checks_complete),
            _ => None,
        }
    }
    #[getter]
    fn termination(&self) -> Option<&'static str> {
        match self.inner.as_ref() {
            native::LiveSimulationEvent::AttemptFinished(summary) => {
                Some(summary.termination.as_str())
            }
            _ => None,
        }
    }
    fn table(&self, py: Python<'_>) -> PyResult<Option<inspection::TableStream>> {
        match self.inner.as_ref() {
            native::LiveSimulationEvent::Sample(batch) => {
                inspection::TableStream::from_batch(batch.table.clone())
                    .map(Some)
                    .map_err(|error| errors::diagnostic(py, &error))
            }
            _ => Ok(None),
        }
    }
    fn result(&self) -> Option<NativeRunResult> {
        match self.inner.as_ref() {
            native::LiveSimulationEvent::Joined(inner) => Some(NativeRunResult {
                owner: self.owner.clone(),
                inner: inner.clone(),
            }),
            _ => None,
        }
    }
    fn diagnostic(&self) -> Option<inspection::DiagnosticReport> {
        match self.inner.as_ref() {
            native::LiveSimulationEvent::RecordingUnknown(error) => {
                Some(inspection::DiagnosticReport::observe(error.as_ref()))
            }
            _ => None,
        }
    }
}
