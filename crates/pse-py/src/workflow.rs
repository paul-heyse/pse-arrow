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

/// Observe interpreter configuration outside the bounded Rust reconstruction scope.
/// No Python callback or import runs while the native loader lock is held.
fn local_python_configuration(anchor: usize) -> std::io::Result<Vec<u8>> {
    Python::attach(|py| -> PyResult<Vec<u8>> {
        let sys = py.import("sys")?;
        let module = sys.getattr("modules")?.get_item("pse._native")?;
        let imported: std::path::PathBuf = module.getattr("__file__")?.extract()?;
        let actual = pse_buildinfo::loaded_module_path(anchor)
            .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(error.to_string()))?;
        let imported = imported
            .canonicalize()
            .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(error.to_string()))?;
        let actual = actual
            .canonicalize()
            .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(error.to_string()))?;
        if imported != actual {
            return Err(pyo3::exceptions::PyRuntimeError::new_err(
                "imported native module changed",
            ));
        }
        let mut configuration = std::collections::BTreeMap::<String, String>::new();
        for name in [
            "version",
            "executable",
            "prefix",
            "base_prefix",
            "exec_prefix",
            "base_exec_prefix",
            "flags",
            "_xoptions",
        ] {
            configuration.insert(name.into(), sys.getattr(name)?.repr()?.to_str()?.into());
        }
        let paths: Vec<String> = sys.getattr("path")?.extract()?;
        serde_json::to_vec(&(configuration, paths, imported))
            .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(error.to_string()))
    })
    .map_err(|error| std::io::Error::other(error.to_string()))
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
/// Borrow the shared scientific deployment budget, services and executor.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeRuntime {
    owner: Arc<runtime::Runtime>,
    inner: native::Runtime,
}
#[pymethods]
impl NativeRuntime {
    /// Close this context's local result readers and physical RPC drivers.
    /// Independently supervised native workers retain their separate drain owner.
    fn close(&self, py: Python<'_>) -> PyResult<()> {
        blocking(
            py,
            &self.owner,
            async {
                self.inner
                    .canonical_store()
                    .disconnect()
                    .await
                    .map_err(|error| native::WorkflowError::Input(error.to_string()))
            },
            || {},
        )
    }

    /// Reopen an exact retained canonical source selection without authored document replay.
    fn modeling_revision(
        &self,
        py: Python<'_>,
        revision: &str,
        physical: &NativePhysicalContext,
    ) -> PyResult<NativeModelingPackage> {
        let inner = blocking(
            py,
            &self.owner,
            async {
                let revision = self
                    .inner
                    .canonical_store()
                    .revision(revision)
                    .await?
                    .ok_or_else(|| {
                        native::WorkflowError::Input("canonical modeling revision absent".into())
                    })?;
                self.inner
                    .modeling_revision(revision, physical.inner.clone(), Default::default())
                    .await
            },
            || {},
        )?;
        Ok(NativeModelingPackage::from_revision(
            self.owner.clone(),
            inner,
            physical.documents.clone(),
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
                let store = pse_operations::canonical::CanonicalStore::connect(&options)
                    .await
                    .map_err(|error| native::WorkflowError::Input(error.to_string()))?;
                let initialized = store.create().await;
                let drained = store.disconnect().await;
                initialized.map_err(|error| native::WorkflowError::Input(error.to_string()))?;
                drained.map_err(|error| native::WorkflowError::Input(error.to_string()))
            },
            || {},
        )
    }
    /// Canonical durable deployment; ephemeral execution is an explicit local choice.
    #[new]
    #[pyo3(signature = (settings, *, substrate, database=None, producer=None, ephemeral=false))]
    fn new(
        py: Python<'_>,
        settings: &inspection::EngineSettings,
        substrate: &str,
        database: Option<&str>,
        producer: Option<&str>,
        ephemeral: bool,
    ) -> PyResult<Self> {
        let owner = py
            .detach(|| runtime::acquire(settings))
            .map_err(|e| errors::diagnostic(py, &e))?;
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
        let outer = pse_buildinfo::observe_deployment(&actual)
            .map_err(|error| invalid(py, error.to_string()))?;
        let attestation = native::OuterAttestation {
            source: outer.source,
            build: outer.build,
        };
        let options = pse_operations::canonical::CanonicalOptions::from_state_for_database(
            std::path::Path::new(substrate),
            database,
        )
        .map_err(|e| errors::diagnostic(py, &e))?;
        let canonical = blocking(
            py,
            &owner,
            async {
                let store = pse_operations::canonical::CanonicalStore::connect(&options)
                    .await
                    .map_err(|e| native::WorkflowError::Input(e.to_string()))?;
                store
                    .open()
                    .await
                    .map_err(|e| native::WorkflowError::Input(e.to_string()))?;
                Ok(store)
            },
            || {},
        )?;
        let strict = producer.map(|path| {
            let bytes = std::fs::read(path).map_err(|e| invalid(py, e.to_string()))?;
            let artifact = pse_buildinfo::verify_receipt_artifact(&bytes, &actual).map_err(|error| invalid(py,error.to_string()))?;
            #[allow(unsafe_code, reason = "ADR-0164 controlled deployment qualification; no unsafe memory operation")]
            // SAFETY: reviewed capture is bound to the actual native code mapping,
            // imported module and current consumed inputs before qualification.
            unsafe { pse_runtime::math::portable::QualifiedProducer::from_deployment_receipt(&bytes, &artifact.sha256, pse_runtime::math::portable::ExpectedProducerTarget::PYTHON) }
                .map_err(|e| errors::diagnostic(py, &e))
        }).transpose()?.flatten();
        let producer = if producer.is_some() {
            strict.map(Into::into)
        } else {
            use pse_runtime::math::portable::{PortableError, ReplayAdmission};

            let anchor = Self::new as *const () as usize;
            let deadline = std::time::Instant::now()
                .checked_add(ReplayAdmission::STARTUP_OBSERVATION_LIMIT)
                .ok_or_else(|| invalid(py, "local replay startup deadline extent"))?;
            let cancelled = CancelSource::new();
            let captured_stop = cancelled.clone();
            let math = owner.shared.math().clone();
            // SAFETY: this actual imported Rust composition root observes its own
            // interpreter/module context. Reconstruction is immutable Rust math,
            // with no Python/plugin/provider callbacks or uncontrolled executable
            // mutation; later native use has its separate generation owner.
            #[allow(
                unsafe_code,
                reason = "ADR-0164 controlled Python deployment-local reconstruction admission"
            )]
            // SAFETY: the actual imported composition root and effective interpreter
            // are observed under the controlled reconstruction contract above.
            let observed = blocking(
                py,
                &owner,
                async move {
                    // The signal bridge cancels the completion-owned math job and
                    // waits for its native drain. Python attaches outside the loader scope.
                    // SAFETY: this actual imported composition root observes its own
                    // interpreter and module under the controlled reconstruction profile.
                    let observation = unsafe {
                        math.observe_local_runtime(
                            pse_runtime::math::portable::ExpectedProducerTarget::PYTHON,
                            anchor,
                            Arc::new(move || local_python_configuration(anchor)),
                            deadline,
                            &captured_stop,
                        )
                    }
                    .await;
                    Ok(observation)
                },
                || cancelled.cancel(),
            )?;
            match observed {
                Ok(admission) => Some(admission),
                Err(PortableError::Qualification(_)) => None,
                Err(error) => return Err(errors::diagnostic(py, &error)),
            }
        };
        let inner = native::Runtime::from_shared(
            owner.shared.clone(),
            owner.registry.clone(),
            owner.sessions.clone(),
            native::CanonicalDeployment::new(canonical, attestation, producer),
        );
        let inner = if ephemeral {
            inner.with_durability(native::Durability::Ephemeral)
        } else {
            inner
        };
        Ok(Self { owner, inner })
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
    /// Exact canonical run metadata through its generated one-row Arrow contract.
    fn run_record(&self, py: Python<'_>, run: &str) -> PyResult<inspection::TableStream> {
        let batch = blocking(py, &self.owner, self.inner.run_record(run), || {})?;
        inspection::TableStream::from_batch(batch).map_err(|error| errors::diagnostic(py, &error))
    }
    /// Exact attempt metadata and recorded terminal class.
    fn attempt_record(&self, py: Python<'_>, attempt: &str) -> PyResult<inspection::TableStream> {
        let batch = blocking(py, &self.owner, self.inner.attempt_record(attempt), || {})?;
        inspection::TableStream::from_batch(batch).map_err(|error| errors::diagnostic(py, &error))
    }
    /// Exact protected admitted manifest, projected by its registry schema.
    fn result_manifest(&self, py: Python<'_>, attempt: &str) -> PyResult<inspection::TableStream> {
        let batch = blocking(py, &self.owner, self.inner.result_manifest(attempt), || {})?;
        inspection::TableStream::from_batch(batch).map_err(|error| errors::diagnostic(py, &error))
    }
    fn analysis(&self, py: Python<'_>, key: &str) -> PyResult<NativeAnalysis> {
        Ok(NativeAnalysis {
            inner: blocking(py, &self.owner, self.inner.analysis(key), || {})?,
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
        let controls: native::AnalysisControls =
            documents::decode(py, "analysis controls", controls, 128 * 1024)?;
        let cancel = pse_columnar::CancellationToken::new();
        Ok(NativeAnalysis {
            inner: blocking(
                py,
                &self.owner,
                self.inner
                    .result_analysis(run, attempt, &controls, cancel.clone()),
                || cancel.cancel(),
            )?,
            owner: self.owner.clone(),
        })
    }
    fn forget_study_results(&self, py: Python<'_>, study: &str) -> PyResult<()> {
        blocking(
            py,
            &self.owner,
            self.inner.forget_study_results(study),
            || {},
        )
    }
    fn forget_analysis_results(&self, py: Python<'_>, analysis: &str) -> PyResult<()> {
        blocking(
            py,
            &self.owner,
            self.inner.forget_analysis_results(analysis),
            || {},
        )
    }
    fn reclaim_run_results(&self, py: Python<'_>, run: &str) -> PyResult<Vec<u8>> {
        let cancel = pse_columnar::CancellationToken::new();
        let page = blocking(
            py,
            &self.owner,
            self.inner.reclaim_run_results(run, &cancel),
            || cancel.cancel(),
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
        let reader = blocking(
            py,
            &self.owner,
            self.inner
                .latest_results(problem, &classes, relation, start, end, cancel.clone()),
            || cancel.cancel(),
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
        let cancel = pse_columnar::CancellationToken::new();
        let reader = blocking(
            py,
            &self.owner,
            self.inner
                .results(run, attempt, relation, start, end, cancel.clone()),
            || cancel.cancel(),
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
        let output = id(py, output)?;
        let cancel = pse_columnar::CancellationToken::new();
        let reader = blocking(
            py,
            &self.owner,
            self.inner.output_results(
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
                cancel.clone(),
            ),
            || cancel.cancel(),
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
        let cancel = pse_columnar::CancellationToken::new();
        blocking(
            py,
            &self.owner,
            async {
                let mut reader = self
                    .inner
                    .results(run, attempt, relation, start, end, cancel.clone())
                    .await?;
                reader.export_ipc(std::path::Path::new(destination)).await
            },
            || cancel.cancel(),
        )
    }
    /// Read one admitted terminal attempt's exact recorded progress history.
    #[pyo3(signature = (run, attempt))]
    fn progress(&self, py: Python<'_>, run: &str, attempt: &str) -> PyResult<NativeProgressStream> {
        let cancel = pse_columnar::CancellationToken::new();
        let stream = blocking(
            py,
            &self.owner,
            self.inner.progress(run, attempt, cancel.clone()),
            || cancel.cancel(),
        )?;
        Ok(NativeProgressStream {
            owner: self.owner.clone(),
            run_key: run.into(),
            attempt_key: attempt.into(),
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
        let stop = CancelSource::new();
        let settings = native::WorkerSettings {
            maximum_actions,
            maximum_in_flight,
            until_idle: true,
            ..native::WorkerSettings::default()
        };
        let processed = blocking(py, &self.owner, self.inner.serve(settings, &stop), || {
            stop.cancel();
        })?;
        Ok(processed)
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

/// A canonical study: exact occurrence status, cancellation and retained result keys.
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
            inner: blocking(
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
