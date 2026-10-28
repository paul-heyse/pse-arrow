// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Diagnostic readers retain copied observation authority, never scientific owners.

use crate::{
    documents::{DocumentInput, DocumentValue},
    inspection::errors,
};
use pse_diagnostics::observation::{OperationId, OperationReader, OperationSnapshot};
use pse_runtime::observation::{CheckpointIdentity, Diagnostics, ProcessCheckpoint};
use pse_runtime::tooling::{
    capture::{self, CaptureDocument, CaptureSession},
    inspection::{
        self, DiagnosticListRequest, DiagnosticProcessPage, InspectionDocument, InspectionRequest,
    },
};
use pyo3::prelude::*;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

#[pyfunction]
pub(crate) fn diagnostic_record_launch(
    py: Python<'_>,
    request: DocumentInput<pse_runtime::tooling::review_launch::SourceRecordingLaunchRequest>,
) -> PyResult<DocumentValue<pse_runtime::tooling::review_launch::SourceRecordingLaunchDocument>> {
    use std::sync::atomic::{AtomicBool, Ordering};
    let cancelled = AtomicBool::new(false);
    let result = crate::workflow::blocking_native_call(
        py,
        || {
            Ok(pse_runtime::tooling::review_launch::record_launch(
                request.0, &cancelled,
            ))
        },
        || cancelled.store(true, Ordering::Release),
    )?;
    result.map(DocumentValue).map_err(|error| match error {
        pse_runtime::observation_detail::CaptureError::Io(error) => PyErr::from(error),
        error => errors::diagnostic(py, &error),
    })
}

/// Offline extraction over explicitly supplied recorded codecs. No Runtime is constructed.
#[pyfunction]
pub(crate) fn diagnostic_extract_reproducer(
    py: Python<'_>,
    request: crate::documents::RecordedInput<pse_runtime::workflow::ReproducerExtractionRequest>,
) -> PyResult<DocumentValue<pse_runtime::workflow::ReproducerCommandRequest>> {
    py.detach(|| request.0.extract())
        .map(DocumentValue)
        .map_err(|error| errors::diagnostic(py, &error))
}

fn window(seconds: f64) -> PyResult<Duration> {
    if !seconds.is_finite() || seconds < 0.0 {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "window must be finite and nonnegative",
        ));
    }
    Duration::try_from_secs_f64(seconds)
        .map_err(|error| pyo3::exceptions::PyValueError::new_err(error.to_string()))
}

/// Passive collection facade. It owns no runtime registration or cancellation lease.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
pub(crate) struct NativeDiagnostics {
    inner: Arc<Diagnostics>,
}
impl NativeDiagnostics {
    pub(crate) fn new(inner: Arc<Diagnostics>) -> Self {
        Self { inner }
    }
}
#[pymethods]
impl NativeDiagnostics {
    #[getter]
    fn checkpoint(&self) -> PathBuf {
        self.inner.checkpoint_path()
    }
    #[pyo3(signature = (offset=0, limit=16))]
    fn list(
        &self,
        py: Python<'_>,
        offset: usize,
        limit: usize,
    ) -> DocumentValue<pse_diagnostics::observation::OperationPage> {
        DocumentValue(py.detach(|| self.inner.hub().list(offset, limit.min(32))))
    }
    fn inspect(
        &self,
        py: Python<'_>,
        request: DocumentInput<InspectionRequest>,
    ) -> DocumentValue<InspectionDocument> {
        DocumentValue(py.detach(|| self.inner.inspect(request.0)))
    }
    fn reader(&self, operation: DocumentInput<OperationId>) -> Option<NativeOperationObservations> {
        self.inner
            .reader(operation.0)
            .map(NativeOperationObservations::new)
    }
    /// Explicit cooperating construction boundary; an absent environment request does nothing.
    fn record_from_environment(
        &self,
        py: Python<'_>,
        supported_families: Vec<String>,
    ) -> PyResult<Option<NativeDetailSession>> {
        if std::env::var_os(capture::SOURCE_RECORDING_REQUEST_ENV).is_none() {
            return Ok(None);
        }
        let operation = self.inner.hub().start("selected.source.recording");
        let families = supported_families
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let session = capture::source_recording_from_environment(operation.context(), &families)
            .map_err(|error| errors::diagnostic(py, &error))?;
        Ok(session.map(|session| NativeDetailSession {
            source_acknowledgement: Some(session.acknowledgement().clone()),
            session: Some(SelectedDetailSession::Source(Box::new(session))),
            completed: None,
            entered: None,
            operation: Some(operation),
        }))
    }
    /// Select finite source detail before constructing work under its entered context.
    fn collect(
        &self,
        py: Python<'_>,
        request: DocumentInput<pse_runtime::observation_detail::DetailRequest>,
    ) -> PyResult<NativeDetailSession> {
        let operation = pse_diagnostics::observation::current().map_or_else(
            || self.inner.hub().start("selected.detail"),
            |parent| parent.child("selected.detail", pse_diagnostics::Relationship::Nested),
        );
        let session = pse_runtime::observation_detail::start_detailed_collection(
            operation.context(),
            request.0,
        )
        .map_err(|error| errors::diagnostic(py, &error))?;
        Ok(NativeDetailSession {
            session: Some(SelectedDetailSession::Local(session)),
            source_acknowledgement: None,
            completed: None,
            entered: None,
            operation: Some(operation),
        })
    }
}

enum SelectedDetailSession {
    Local(pse_runtime::observation_detail::DetailSession),
    Source(Box<capture::SourceRecordingSession>),
}
impl SelectedDetailSession {
    fn context(&self) -> pse_diagnostics::observation::ObservationContext {
        match self {
            Self::Local(s) => s.context(),
            Self::Source(s) => s.context(),
        }
    }
    fn snapshot(&self) -> pse_runtime::observation_detail::DetailDocument {
        match self {
            Self::Local(s) => s.snapshot(),
            Self::Source(s) => s.snapshot(),
        }
    }
    fn request_stop(&self) {
        match self {
            Self::Local(s) => s.request_stop(),
            Self::Source(s) => s.request_stop(),
        }
    }
    fn finish(
        self,
    ) -> Result<
        pse_runtime::observation_detail::DetailDocument,
        pse_runtime::observation_detail::CaptureError,
    > {
        match self {
            Self::Local(s) => s.finish(),
            Self::Source(s) => s.finish(),
        }
    }
    fn stop(
        self,
    ) -> Result<
        pse_runtime::observation_detail::DetailDocument,
        pse_runtime::observation_detail::CaptureError,
    > {
        match self {
            Self::Local(s) => s.stop(),
            Self::Source(s) => s.stop(),
        }
    }
}

/// Caller-scoped selected collection; its thread-local guard never crosses an await.
#[pyclass(unsendable, skip_from_py_object, module = "pse._native")]
pub(crate) struct NativeDetailSession {
    session: Option<SelectedDetailSession>,
    source_acknowledgement: Option<capture::SourceRecordingAcknowledgement>,
    completed: Option<pse_runtime::observation_detail::DetailDocument>,
    entered: Option<pse_diagnostics::observation::EnterGuard>,
    operation: Option<pse_diagnostics::observation::OperationGuard>,
}
#[pymethods]
impl NativeDetailSession {
    #[getter]
    fn acknowledgement(&self) -> Option<DocumentValue<capture::SourceRecordingAcknowledgement>> {
        self.source_acknowledgement.clone().map(DocumentValue)
    }
    fn retain_result(
        &mut self,
        py: Python<'_>,
        result: &crate::workflow::NativeRunResult,
    ) -> PyResult<Option<DocumentValue<capture::ReviewHandoffReceipt>>> {
        let was_entered = self.entered.take().is_some();
        let Some(session) = self.session.take() else {
            return Err(crate::workflow::invalid(py, "selected recording has ended"));
        };
        let joined = result.retained_result().clone();
        let (session, outcome) = py.detach(move || {
            let mut session = session;
            let outcome = match &mut session {
                SelectedDetailSession::Source(source) => source.retain_result(&joined),
                SelectedDetailSession::Local(_) => Ok(None),
            };
            (session, outcome)
        });
        if was_entered {
            self.entered = Some(session.context().enter());
        }
        self.session = Some(session);
        outcome
            .map(|receipt| receipt.map(DocumentValue))
            .map_err(|error| errors::diagnostic(py, &error))
    }
    fn retain_context(
        &mut self,
        py: Python<'_>,
        context: crate::documents::RecordedInput<pse_runtime::workflow::ReviewContextDocument>,
    ) -> PyResult<Option<DocumentValue<capture::ReviewHandoffReceipt>>> {
        let was_entered = self.entered.take().is_some();
        let Some(session) = self.session.take() else {
            return Err(crate::workflow::invalid(py, "selected recording has ended"));
        };
        let (session, outcome) = py.detach(move || {
            let mut session = session;
            let outcome = match &mut session {
                SelectedDetailSession::Source(source) => source.retain_context(&context.0),
                SelectedDetailSession::Local(_) => Ok(None),
            };
            (session, outcome)
        });
        if was_entered {
            self.entered = Some(session.context().enter());
        }
        self.session = Some(session);
        outcome
            .map(|receipt| receipt.map(DocumentValue))
            .map_err(|error| errors::diagnostic(py, &error))
    }
    fn enter(&mut self) -> PyResult<()> {
        if self.entered.is_some() {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "selected collection is already entered",
            ));
        }
        let session = self.session.as_ref().ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err("selected collection has ended")
        })?;
        self.entered = Some(session.context().enter());
        Ok(())
    }
    fn snapshot(&self) -> PyResult<DocumentValue<pse_runtime::observation_detail::DetailDocument>> {
        self.session
            .as_ref()
            .map(SelectedDetailSession::snapshot)
            .or_else(|| self.completed.clone())
            .map(DocumentValue)
            .ok_or_else(|| {
                pyo3::exceptions::PyRuntimeError::new_err("selected collection unavailable")
            })
    }
    /// Restore caller-local scope while already constructed work retains its selection.
    fn leave(&mut self) {
        self.entered.take();
    }
    fn request_stop(&self) {
        if let Some(session) = &self.session {
            session.request_stop();
        }
    }
    fn finish(
        &mut self,
        py: Python<'_>,
    ) -> PyResult<DocumentValue<pse_runtime::observation_detail::DetailDocument>> {
        self.join(py, true)
    }
    fn stop(
        &mut self,
        py: Python<'_>,
    ) -> PyResult<DocumentValue<pse_runtime::observation_detail::DetailDocument>> {
        self.join(py, false)
    }
}
impl NativeDetailSession {
    fn join(
        &mut self,
        py: Python<'_>,
        finished: bool,
    ) -> PyResult<DocumentValue<pse_runtime::observation_detail::DetailDocument>> {
        self.entered.take();
        if let Some(session) = self.session.take() {
            let result = py
                .detach(move || {
                    if finished {
                        session.finish()
                    } else {
                        session.stop()
                    }
                })
                .map_err(|error| errors::diagnostic(py, &error));
            if let Some(operation) = self.operation.take() {
                operation.finish(pse_diagnostics::OperationOutcome::Completed);
            }
            self.completed = Some(result?);
        }
        self.snapshot()
    }
}
impl Drop for NativeDetailSession {
    fn drop(&mut self) {
        self.entered.take();
        if let Some(session) = self.session.take() {
            let mut pending = Some(session);
            let _ = Python::try_attach(|py| py.detach(|| drop(pending.take())));
            drop(pending);
        }
    }
}

#[pyfunction]
pub(crate) fn diagnostic_read_detail(
    py: Python<'_>,
    path: PathBuf,
    maximum_bytes: u64,
) -> PyResult<DocumentValue<pse_runtime::observation_detail::DetailDocument>> {
    py.detach(|| pse_runtime::observation_detail::read_detailed_collection(&path, maximum_bytes))
        .map(DocumentValue)
        .map_err(|error| errors::diagnostic(py, &error))
}
#[pyfunction]
pub(crate) fn diagnostic_detail_records(
    py: Python<'_>,
    path: PathBuf,
    offset: u64,
    limit: usize,
    maximum_bytes: u64,
) -> PyResult<DocumentValue<pse_runtime::observation_detail::DetailRecordPage>> {
    py.detach(|| {
        pse_runtime::observation_detail::read_detail_records(&path, offset, limit, maximum_bytes)
    })
    .map(DocumentValue)
    .map_err(|error| errors::diagnostic(py, &error))
}

/// Finite Arrow projection of retained source records; buffers retain their own admission.
#[pyfunction]
#[pyo3(signature = (path, offset, limit, maximum_bytes, memory_bytes) -> "object")]
pub(crate) fn diagnostic_detail_arrow(
    py: Python<'_>,
    path: PathBuf,
    offset: u64,
    limit: usize,
    maximum_bytes: u64,
    memory_bytes: usize,
) -> PyResult<pyo3_arrow::PyRecordBatch> {
    py.detach(|| {
        let page = pse_runtime::observation_detail::read_detail_records(
            &path,
            offset,
            limit,
            maximum_bytes,
        )?;
        let pool: Arc<dyn pse_columnar::MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(memory_bytes));
        capture::evidence::detail_arrow(&page, &pool, &pse_columnar::CancellationToken::new())
    })
    .map(pyo3_arrow::PyRecordBatch::new)
    .map_err(|error| errors::diagnostic(py, &error))
}
/// The original capture document remains independent from this optional view.
#[pyfunction]
#[pyo3(signature = (document, memory_bytes) -> "object")]
pub(crate) fn diagnostic_capture_arrow(
    py: Python<'_>,
    document: DocumentInput<CaptureDocument>,
    memory_bytes: usize,
) -> PyResult<pyo3_arrow::PyRecordBatch> {
    py.detach(|| {
        let pool: Arc<dyn pse_columnar::MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(memory_bytes));
        capture::evidence::capture_arrow(
            &document.0,
            &pool,
            &pse_columnar::CancellationToken::new(),
        )
    })
    .map(pyo3_arrow::PyRecordBatch::new)
    .map_err(|error| errors::diagnostic(py, &error))
}

/// An exact operation read lease, independent from RunHandle's cancellation owner.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
pub(crate) struct NativeOperationObservations {
    inner: OperationReader,
}
impl NativeOperationObservations {
    pub(crate) fn new(inner: OperationReader) -> Self {
        Self { inner }
    }
}
#[pymethods]
impl NativeOperationObservations {
    #[getter]
    fn id(&self) -> DocumentValue<OperationId> {
        DocumentValue(self.inner.id())
    }
    #[pyo3(signature = (window_seconds=60.0))]
    fn snapshot(
        &self,
        py: Python<'_>,
        window_seconds: f64,
    ) -> PyResult<DocumentValue<OperationSnapshot>> {
        let interval = window(window_seconds)?;
        Ok(DocumentValue(py.detach(|| self.inner.snapshot(interval))))
    }
    #[pyo3(signature = (window_seconds=60.0))]
    fn inspect(
        &self,
        py: Python<'_>,
        window_seconds: f64,
    ) -> PyResult<DocumentValue<InspectionDocument>> {
        let mut request = InspectionRequest::new(self.inner.id());
        request.window = window(window_seconds)?;
        request.include_related = false;
        Ok(DocumentValue(py.detach(|| {
            inspection::inspect_snapshot(self.inner.snapshot(request.window), request)
        })))
    }
}

#[pyfunction]
pub(crate) fn diagnostic_operation_id(
    py: Python<'_>,
    text: &str,
) -> PyResult<DocumentValue<OperationId>> {
    text.parse::<OperationId>()
        .map(DocumentValue)
        .map_err(|error| crate::workflow::invalid(py, error.to_string()))
}
#[pyfunction]
pub(crate) fn diagnostic_operation_text(operation: DocumentInput<OperationId>) -> String {
    operation.0.to_string()
}
#[pyfunction]
#[pyo3(signature = (request, directory=None))]
pub(crate) fn diagnostic_list(
    py: Python<'_>,
    request: DocumentInput<DiagnosticListRequest>,
    directory: Option<PathBuf>,
) -> PyResult<DocumentValue<DiagnosticProcessPage>> {
    let directory = directory.unwrap_or_else(pse_runtime::observation::default_directory);
    py.detach(|| inspection::list_processes(&directory, request.0))
        .map(DocumentValue)
        .map_err(|error| errors::diagnostic(py, &error))
}
#[pyfunction]
pub(crate) fn diagnostic_inspect(
    py: Python<'_>,
    checkpoint: PathBuf,
    request: DocumentInput<InspectionRequest>,
) -> PyResult<DocumentValue<InspectionDocument>> {
    py.detach(|| inspection::inspect_checkpoint(&checkpoint, request.0))
        .map(DocumentValue)
        .map_err(|error| errors::diagnostic(py, &error))
}

#[pyfunction]
#[pyo3(signature = (checkpoint, expected=None))]
pub(crate) fn diagnostic_read_checkpoint(
    py: Python<'_>,
    checkpoint: PathBuf,
    expected: Option<DocumentInput<CheckpointIdentity>>,
) -> PyResult<DocumentValue<ProcessCheckpoint>> {
    py.detach(|| {
        let image = pse_runtime::observation::read_checkpoint(&checkpoint)?;
        if let Some(expected) = expected {
            expected.0.validate(&image)?;
        }
        Ok::<_, pse_runtime::observation::ObservationError>(image)
    })
    .map(DocumentValue)
    .map_err(|error| errors::diagnostic(py, &error))
}

#[pyfunction]
pub(crate) fn diagnostic_inspect_image(
    py: Python<'_>,
    checkpoint: PathBuf,
    image: DocumentInput<ProcessCheckpoint>,
    request: DocumentInput<InspectionRequest>,
) -> DocumentValue<InspectionDocument> {
    DocumentValue(py.detach(|| inspection::inspect_image(&checkpoint, request.0, image.0)))
}

struct CaptureState {
    session: Option<CaptureSession>,
    completed: Option<CaptureDocument>,
    worker_failed: bool,
}
/// Owns only a bounded optional capture and its join, never target cancellation.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
pub(crate) struct NativeCaptureSession {
    state: Mutex<CaptureState>,
}
impl Drop for NativeCaptureSession {
    fn drop(&mut self) {
        let session = self
            .state
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .session
            .take();
        if let Some(session) = session {
            // Deallocation normally runs attached. Detach for the exporter join too.
            let mut pending = Some(session);
            let _ = Python::try_attach(|py| py.detach(|| drop(pending.take())));
            // During interpreter shutdown attachment can be unavailable; the Rust
            // capture still owns and drains its worker without calling Python.
            drop(pending);
        }
    }
}
impl NativeCaptureSession {
    fn state(&self) -> std::sync::MutexGuard<'_, CaptureState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
    fn document(&self) -> Option<CaptureDocument> {
        let state = self.state();
        state
            .session
            .as_ref()
            .map(CaptureSession::snapshot)
            .or_else(|| state.completed.clone())
    }
    fn join(&self, py: Python<'_>) -> PyResult<DocumentValue<CaptureDocument>> {
        let waited = crate::wait::wait(
            py,
            Duration::from_millis(20),
            |_| false,
            crate::wait::SignalAction::Drain,
            |interval| {
                let result = (|| {
                    let mut state = self.state();
                    if state.worker_failed {
                        return Some(Err(pse_runtime::observation_detail::CaptureError::Worker));
                    }
                    if let Some(document) = &state.completed {
                        return Some(Ok(document.clone()));
                    }
                    let document = state.session.as_ref()?.snapshot();
                    if !state.session.as_ref()?.is_finished() {
                        return None;
                    }
                    let session = state.session.take()?;
                    let result = session.wait();
                    let mut completed = result.as_ref().cloned().unwrap_or(document);
                    if let Err(error) = &result {
                        completed.status = capture::CaptureStatus::Failed;
                        completed.errors.push(error.to_string());
                        state.worker_failed = true;
                    }
                    state.completed = Some(completed);
                    Some(result)
                })();
                match result {
                    Some(value) => std::task::Poll::Ready(value),
                    None => {
                        std::thread::sleep(interval);
                        std::task::Poll::Pending
                    }
                }
            },
            || self.request_stop(),
        );
        if let Some(signal) = waited.signal {
            return Err(signal);
        }
        waited
            .value
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("capture wait lost outcome"))?
            .map(DocumentValue)
            .map_err(|error| errors::diagnostic(py, &error))
    }
}
#[pymethods]
impl NativeCaptureSession {
    fn snapshot(&self) -> PyResult<DocumentValue<CaptureDocument>> {
        self.document().map(DocumentValue).ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("capture join state unavailable")
        })
    }
    fn request_stop(&self) {
        if let Some(session) = &self.state().session {
            session.request_stop();
        }
    }
    fn wait(&self, py: Python<'_>) -> PyResult<DocumentValue<CaptureDocument>> {
        self.join(py)
    }
    fn stop(&self, py: Python<'_>) -> PyResult<DocumentValue<CaptureDocument>> {
        self.request_stop();
        self.join(py)
    }
}
#[pyfunction]
pub(crate) fn diagnostic_capture(
    py: Python<'_>,
    request: DocumentInput<capture::CaptureRequest>,
) -> PyResult<NativeCaptureSession> {
    let session = py
        .detach(|| capture::start_capture(request.0))
        .map_err(|error| errors::diagnostic(py, &error))?;
    Ok(NativeCaptureSession {
        state: Mutex::new(CaptureState {
            session: Some(session),
            completed: None,
            worker_failed: false,
        }),
    })
}

/// The independently owned launched command. No capture session retains this object.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
pub(crate) struct NativeLaunchedCommand {
    command: Mutex<capture::LaunchedCommand>,
}
impl NativeLaunchedCommand {
    fn command(&self) -> std::sync::MutexGuard<'_, capture::LaunchedCommand> {
        self.command
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}
impl Drop for NativeLaunchedCommand {
    fn drop(&mut self) {
        let command = self
            .command
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let _ = Python::try_attach(|py| py.detach(|| command.close()));
        // The Rust owner also drains during interpreter shutdown if attachment is unavailable.
    }
}
#[pymethods]
impl NativeLaunchedCommand {
    #[getter]
    fn locator(&self) -> DocumentValue<pse_runtime::observation::ProcessLocator> {
        DocumentValue(self.command().locator().clone())
    }
    fn try_wait(&self, py: Python<'_>) -> PyResult<Option<i32>> {
        py.detach(|| self.command().try_wait())
            .map_err(|error| errors::diagnostic(py, &error))
    }
    fn wait(&self, py: Python<'_>) -> PyResult<i32> {
        let waited = crate::wait::wait(
            py,
            Duration::from_millis(20),
            |_| false,
            crate::wait::SignalAction::Return,
            |interval| match self.command().wait_ready(interval) {
                Ok(Some(status)) => std::task::Poll::Ready(Ok(status)),
                Ok(None) => std::task::Poll::Pending,
                Err(error) => std::task::Poll::Ready(Err(error)),
            },
            || {
                let _ = py.detach(|| self.command().cancel());
            },
        );
        if let Some(signal) = waited.signal {
            return Err(signal);
        }
        waited
            .value
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("command wait lost outcome"))?
            .map_err(|error| errors::diagnostic(py, &error))
    }

    fn cancel(&self, py: Python<'_>) -> PyResult<i32> {
        py.detach(|| self.command().cancel())
            .map_err(|error| errors::diagnostic(py, &error))
    }
    fn close(&self, py: Python<'_>) -> PyResult<i32> {
        self.cancel(py)
    }
}
#[pyfunction]
pub(crate) fn diagnostic_launch_capture(
    py: Python<'_>,
    request: DocumentInput<capture::LaunchCaptureRequest>,
) -> PyResult<(NativeLaunchedCommand, NativeCaptureSession)> {
    let (command, session) = py
        .detach(|| capture::launch_capture(request.0))
        .map_err(|error| errors::diagnostic(py, &error))?;
    Ok((
        NativeLaunchedCommand {
            command: Mutex::new(command),
        },
        NativeCaptureSession {
            state: Mutex::new(CaptureState {
                session: Some(session),
                completed: None,
                worker_failed: false,
            }),
        },
    ))
}

#[pyfunction]
pub(crate) fn diagnostic_inspect_frozen(
    py: Python<'_>,
    evidence: crate::documents::RecordedInput<
        pse_runtime::tooling::frozen_inspection::FrozenEvidenceSet,
    >,
    request: crate::documents::RecordedInput<
        pse_runtime::tooling::frozen_inspection::FrozenInspectionRequest,
    >,
) -> PyResult<DocumentValue<pse_runtime::tooling::frozen_inspection::FrozenInspectionDocument>> {
    py.detach(|| pse_runtime::tooling::frozen_inspection::inspect_frozen(&evidence.0, request.0))
        .map(DocumentValue)
        .map_err(|error| errors::diagnostic(py, &error))
}
#[pyfunction]
pub(crate) fn diagnostic_read_evidence(
    py: Python<'_>,
    path: PathBuf,
) -> PyResult<DocumentValue<pse_runtime::tooling::frozen_inspection::FrozenEvidenceSet>> {
    py.detach(|| {
        let set: pse_runtime::tooling::frozen_inspection::FrozenEvidenceSet =
            pse_runtime::tooling::frozen_inspection::read_document::<_, 3>(&path)?;
        set.validate()?;
        Ok::<_, pse_runtime::observation::ObservationError>(set)
    })
    .map(DocumentValue)
    .map_err(|error| errors::diagnostic(py, &error))
}
#[pyfunction]
pub(crate) fn diagnostic_read_invocation(
    py: Python<'_>,
    path: PathBuf,
) -> PyResult<DocumentValue<pse_runtime::invocation::InvocationEvidenceDocument>> {
    py.detach(|| pse_runtime::tooling::frozen_inspection::read_invocation(&path))
        .map(DocumentValue)
        .map_err(|error| errors::diagnostic(py, &error))
}
#[pyfunction]
pub(crate) fn diagnostic_read_context(
    py: Python<'_>,
    path: PathBuf,
) -> PyResult<DocumentValue<pse_runtime::workflow::ReviewContextDocument>> {
    py.detach(|| pse_runtime::tooling::frozen_inspection::read_context(&path))
        .map(DocumentValue)
        .map_err(|error| errors::diagnostic(py, &error))
}
#[pyfunction]
#[pyo3(signature=(source,policy=None))]
pub(crate) fn diagnostic_explain_invocation(
    py: Python<'_>,
    source: crate::documents::RecordedInput<pse_runtime::invocation::InvocationEvidenceDocument>,
    policy: Option<DocumentInput<pse_runtime::tooling::inspection_comparison::ComparisonPolicy>>,
) -> DocumentValue<pse_runtime::tooling::inspection_comparison::InvocationComparisonDocument> {
    DocumentValue(py.detach(|| {
        pse_runtime::tooling::inspection_comparison::explain_invocation(
            &source.0,
            policy.map_or_else(Default::default, |p| p.0),
        )
    }))
}
#[pyfunction]
#[pyo3(signature=(left,right,policy=None))]
pub(crate) fn diagnostic_compare_invocations(
    py: Python<'_>,
    left: crate::documents::RecordedInput<pse_runtime::invocation::InvocationEvidenceDocument>,
    right: crate::documents::RecordedInput<pse_runtime::invocation::InvocationEvidenceDocument>,
    policy: Option<DocumentInput<pse_runtime::tooling::inspection_comparison::ComparisonPolicy>>,
) -> DocumentValue<pse_runtime::tooling::inspection_comparison::InvocationComparisonDocument> {
    DocumentValue(py.detach(|| {
        pse_runtime::tooling::inspection_comparison::compare_invocations(
            &left.0,
            &right.0,
            policy.map_or_else(Default::default, |p| p.0),
        )
    }))
}

#[pyfunction]
pub(crate) fn diagnostic_inventory_page(
    py: Python<'_>,
    selection: crate::documents::RecordedInput<pse_runtime::workflow::ReviewInventorySelection>,
    offset: usize,
    limit: usize,
) -> PyResult<DocumentValue<pse_runtime::workflow::ReviewInventoryPage>> {
    py.detach(|| selection.0.page(offset, limit))
        .map(DocumentValue)
        .map_err(|error| errors::diagnostic(py, &error))
}

#[pyfunction]
#[pyo3(signature=(evidence,request,scope="supplied.operations"))]
pub(crate) fn diagnostic_inventory_from_evidence(
    py: Python<'_>,
    evidence: crate::documents::RecordedInput<
        pse_runtime::tooling::frozen_inspection::FrozenEvidenceSet,
    >,
    request: DocumentInput<pse_runtime::workflow::ReviewInventoryRequest>,
    scope: &str,
) -> PyResult<DocumentValue<pse_runtime::workflow::ReviewInventorySelection>> {
    py.detach(|| evidence.0.operation_inventory(scope.into(), &request.0))
        .map(DocumentValue)
        .map_err(|error| errors::diagnostic(py, &error))
}

#[pyfunction]
#[pyo3(signature=(evidence,request,*,memory_bytes=64<<20))]
pub(crate) fn diagnostic_analyze(
    py: Python<'_>,
    evidence: crate::documents::RecordedInput<
        pse_runtime::tooling::frozen_inspection::FrozenEvidenceSet,
    >,
    request: crate::documents::RecordedInput<
        pse_runtime::tooling::review_analysis::AnalysisRequest,
    >,
    memory_bytes: usize,
) -> PyResult<DocumentValue<pse_runtime::tooling::review_analysis::Analysis>> {
    if memory_bytes == 0 {
        return Err(crate::workflow::invalid(
            py,
            "analysis requires a positive finite memory allowance",
        ));
    }
    let pool: Arc<dyn pse_columnar::MemoryPool> =
        Arc::new(pse_columnar::GreedyMemoryPool::new(memory_bytes));
    let cancel = pse_columnar::CancellationToken::new();
    crate::workflow::blocking_native_call(
        py,
        || {
            pse_runtime::tooling::review_analysis::analyze(&evidence.0, &request.0, &pool, &cancel)
                .map_err(|error| pse_engine::EngineError::Infrastructure {
                    op: "analyze supplied review facts".into(),
                    source: Box::new(error),
                })
        },
        || cancel.cancel(),
    )
    .map(DocumentValue)
}
#[pyfunction]
pub(crate) fn diagnostic_query_specification(
    py: Python<'_>,
    request: crate::documents::RecordedInput<
        pse_runtime::tooling::review_query::ReviewQueryRequest,
    >,
) -> PyResult<DocumentValue<pse_runtime::tooling::review_query::ReviewQuerySpecification>> {
    pse_runtime::tooling::review_query::specification(&request.0)
        .map(DocumentValue)
        .map_err(|error| errors::diagnostic(py, &error))
}
#[pyfunction]
pub(crate) fn diagnostic_export_pack(
    py: Python<'_>,
    destination: PathBuf,
    evidence: crate::documents::RecordedInput<
        pse_runtime::tooling::frozen_inspection::FrozenEvidenceSet,
    >,
    analysis: crate::documents::RecordedInput<
        pse_runtime::tooling::review_analysis::AnalysisDocument,
    >,
    selection: crate::documents::RecordedInput<
        pse_runtime::tooling::review_pack::PackExportRequest,
    >,
) -> PyResult<DocumentValue<pse_runtime::tooling::review_pack::PackIndex>> {
    let cancel = pse_columnar::CancellationToken::new();
    crate::workflow::blocking_native_call(
        py,
        || {
            pse_runtime::tooling::review_export::export_supplied_review_pack(
                &destination,
                &evidence.0,
                &analysis.0,
                selection.0,
                cancel.clone(),
            )
            .map_err(|error| pse_engine::EngineError::Infrastructure {
                op: "export selected review pack".into(),
                source: Box::new(error),
            })
        },
        || cancel.cancel(),
    )
    .map(DocumentValue)
}
#[pyfunction]
pub(crate) fn diagnostic_review_pack(
    py: Python<'_>,
    directory: PathBuf,
) -> PyResult<DocumentValue<pse_runtime::tooling::review_pack::PackIndex>> {
    py.detach(|| {
        pse_runtime::tooling::review_pack::PackReader::open(
            &directory,
            pse_runtime::tooling::review_pack::MAXIMUM_PACK_DOCUMENT_BYTES,
        )
        .map(|reader| reader.index().clone())
    })
    .map(DocumentValue)
    .map_err(|error| errors::diagnostic(py, &error))
}

#[pyfunction]
pub(crate) fn diagnostic_export_members(
    py: Python<'_>,
    destination: PathBuf,
    selection: crate::documents::RecordedInput<
        pse_runtime::tooling::review_pack::PackExportRequest,
    >,
) -> PyResult<DocumentValue<pse_runtime::tooling::review_pack::PackIndex>> {
    py.detach(|| {
        pse_runtime::tooling::review_export::export_supplied_members(
            &destination,
            selection.0,
            pse_columnar::CancellationToken::new(),
        )
    })
    .map(DocumentValue)
    .map_err(|error| errors::diagnostic(py, &error))
}

#[pyfunction]
pub(crate) fn diagnostic_pack_member(
    py: Python<'_>,
    directory: PathBuf,
    member: String,
    maximum_bytes: usize,
) -> PyResult<Py<pyo3::types::PyBytes>> {
    let bytes = py
        .detach(|| {
            use std::io::Read;
            if maximum_bytes == 0
                || maximum_bytes > pse_runtime::tooling::review_pack::MAXIMUM_PACK_DOCUMENT_BYTES
            {
                return Err(pse_runtime::tooling::review_pack::PackError::Contract(
                    "finite member byte allowance required".into(),
                ));
            }
            let reader = pse_runtime::tooling::review_pack::PackReader::open(
                &directory,
                pse_runtime::tooling::review_pack::MAXIMUM_PACK_DOCUMENT_BYTES,
            )?;
            let mut selected = reader.open_member(&member)?;
            if selected.limit() > maximum_bytes as u64 {
                return Err(pse_runtime::tooling::review_pack::PackError::Contract(
                    "member exceeds selected byte allowance".into(),
                ));
            }
            let mut bytes = Vec::new();
            selected.read_to_end(&mut bytes)?;
            Ok(bytes)
        })
        .map_err(|error| errors::diagnostic(py, &error))?;
    Ok(pyo3::types::PyBytes::new(py, &bytes).unbind())
}
#[pyfunction]
pub(crate) fn diagnostic_pack_evidence(
    py: Python<'_>,
    directory: PathBuf,
) -> PyResult<DocumentValue<pse_runtime::tooling::frozen_inspection::FrozenEvidenceSet>> {
    py.detach(|| {
        pse_runtime::tooling::review_pack::PackReader::open(
            &directory,
            pse_runtime::tooling::review_pack::MAXIMUM_PACK_DOCUMENT_BYTES,
        )?
        .read_frozen()
    })
    .map(DocumentValue)
    .map_err(|error| errors::diagnostic(py, &error))
}

#[pyfunction]
pub(crate) fn diagnostic_pack_analysis(
    py: Python<'_>,
    directory: PathBuf,
) -> PyResult<DocumentValue<pse_runtime::tooling::review_analysis::AnalysisDocument>> {
    py.detach(|| {
        pse_runtime::tooling::review_pack::PackReader::open(
            &directory,
            pse_runtime::tooling::review_pack::MAXIMUM_PACK_DOCUMENT_BYTES,
        )?
        .read_analysis()
    })
    .map(DocumentValue)
    .map_err(|error| errors::diagnostic(py, &error))
}
#[pyfunction]
pub(crate) fn diagnostic_render_analysis(
    py: Python<'_>,
    analysis: crate::documents::RecordedInput<
        pse_runtime::tooling::review_analysis::AnalysisDocument,
    >,
) -> String {
    py.detach(|| pse_runtime::tooling::review_analysis::render_markdown(&analysis.0))
}

/// Select one immutable registry view; no schema inference, store, native evaluation or live lookup.
#[pyfunction]
#[pyo3(signature=(evidence,request) -> "object")]
pub(crate) fn diagnostic_review_arrow(
    py: Python<'_>,
    evidence: crate::documents::RecordedInput<
        pse_runtime::tooling::frozen_inspection::FrozenEvidenceSet,
    >,
    request: crate::documents::RecordedInput<
        pse_runtime::tooling::review_query::ReviewTableRequest,
    >,
) -> PyResult<pyo3_arrow::PyRecordBatch> {
    let cancel = pse_columnar::CancellationToken::new();
    crate::workflow::blocking_native_call(
        py,
        || {
            pse_runtime::tooling::review_query::table_arrow(&evidence.0, &request.0, &cancel)
                .map_err(|error| pse_engine::EngineError::Infrastructure {
                    op: "project selected review table".into(),
                    source: Box::new(error),
                })
        },
        || cancel.cancel(),
    )
    .map(pyo3_arrow::PyRecordBatch::new)
}

#[pyfunction]
pub(crate) fn diagnostic_read_handoff(
    py: Python<'_>,
    path: PathBuf,
) -> PyResult<DocumentValue<pse_runtime::tooling::frozen_inspection::FrozenEvidenceSet>> {
    py.detach(|| capture::read_review_handoff(&path))
        .map(DocumentValue)
        .map_err(|error| errors::diagnostic(py, &error))
}
