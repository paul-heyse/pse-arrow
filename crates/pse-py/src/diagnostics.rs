// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Diagnostic readers retain copied observation authority, never scientific owners.

use crate::{
    documents::{DocumentInput, DocumentValue},
    inspection::errors,
};
use pse_diagnostics::observation::{OperationId, OperationReader, OperationSnapshot};
use pse_runtime::{
    capture::{self, CaptureDocument, CaptureSession},
    inspection::{
        self, DiagnosticListRequest, DiagnosticProcessPage, InspectionDocument, InspectionRequest,
    },
    observation::Diagnostics,
};
use pyo3::prelude::*;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

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
    let directory = directory.unwrap_or_else(inspection::default_directory);
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
        let mut signal = None;
        loop {
            let result = py.detach(|| {
                let mut state = self.state();
                if state.worker_failed {
                    return Some(Err(capture::CaptureError::Worker));
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
            });
            if let Some(result) = result {
                return match signal {
                    Some(error) => Err(error),
                    None => result
                        .map(DocumentValue)
                        .map_err(|error| errors::diagnostic(py, &error)),
                };
            }
            py.detach(|| std::thread::sleep(Duration::from_millis(20)));
            if signal.is_none()
                && let Err(error) = py.check_signals()
            {
                self.request_stop();
                signal = Some(error);
            }
        }
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
