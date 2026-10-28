// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::errors;
use datafusion::arrow::array::RecordBatchReader;
use pse_engine::arrow_stream::BatchStream;
use pyo3::{prelude::*, types::PyCapsule};
use pyo3_arrow::{PyRecordBatch, PyRecordBatchReader, PySchema};
use std::sync::{Arc, Mutex};

/// A one-consumption Arrow stream retaining admitted source ownership.
#[pyclass(frozen, module = "pse._native")]
pub(crate) struct TableStream {
    stream: BatchStream,
    controlled: Mutex<Option<Box<dyn RecordBatchReader + Send>>>,
}
impl std::fmt::Debug for TableStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TableStream")
            .field("stream", &self.stream)
            .finish_non_exhaustive()
    }
}
impl TableStream {
    fn new(stream: BatchStream) -> Self {
        Self {
            stream,
            controlled: Mutex::new(None),
        }
    }
    pub(crate) fn from_batch(
        batch: pse_relations::columnar::FieldCheckedBatch,
    ) -> Result<Self, pse_engine::EngineError> {
        let cancel = pse_columnar::CancellationToken::new();
        let exported = batch
            .checked_export(&cancel)
            .map_err(|error| pse_engine::EngineError::Semantic(Arc::new(error)))?;
        let schema = exported.schema();
        let mut next = Some(exported);
        Ok(Self::new(BatchStream::new(
            schema,
            cancel,
            Box::new(move || Ok(next.take())),
        )))
    }
    pub(crate) fn from_result(mut cursor: pse_runtime::workflow::ResultCursor<'static>) -> Self {
        let cancel = cursor.cancellation_token();
        let export_cancel = cancel.clone();
        Self::new(BatchStream::new(
            cursor.schema(),
            cancel,
            Box::new(move || {
                cursor
                    .next_chunk()
                    .map_err(|error| pse_engine::EngineError::Semantic(error))?
                    .map(|batch| {
                        batch
                            .checked_export(&export_cancel)
                            .map_err(|error| pse_engine::EngineError::Semantic(Arc::new(error)))
                    })
                    .transpose()
            }),
        ))
    }
    pub(crate) fn from_canonical(
        mut reader: pse_runtime::workflow::CanonicalResultReader,
        runtime: Arc<super::runtime::Runtime>,
    ) -> Self {
        Self::new(BatchStream::new(
            reader.schema(),
            reader.cancellation_token(),
            Box::new(move || {
                if tokio::runtime::Handle::try_current().is_ok() {
                    return Err(errors::invalid(
                        "synchronous Arrow stream cannot re-enter its async executor",
                    ));
                }
                runtime
                    .executor
                    .block_on(reader.next_batch())
                    .map_err(|error| pse_engine::EngineError::Semantic(Arc::new(error)))
            }),
        ))
    }
    pub(crate) fn from_selected(
        mut reader: pse_runtime::workflow::SelectedResultReader,
        runtime: Arc<super::runtime::Runtime>,
    ) -> Self {
        Self::new(BatchStream::new(
            reader.schema(),
            reader.cancellation_token(),
            Box::new(move || {
                if tokio::runtime::Handle::try_current().is_ok() {
                    return Err(errors::invalid(
                        "synchronous Arrow stream cannot re-enter its async executor",
                    ));
                }
                runtime
                    .executor
                    .block_on(reader.next_batch())
                    .map_err(|error| pse_engine::EngineError::Semantic(Arc::new(error)))
            }),
        ))
    }
}
#[pymethods]
impl TableStream {
    #[pyo3(signature = () -> "object")]
    fn __arrow_c_schema__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        PySchema::new(self.stream.schema())
            .into_pyobject(py)?
            .call_method0("__arrow_c_schema__")
    }
    #[pyo3(signature = (requested_schema: "object | None"=None) -> "object")]
    fn __arrow_c_stream__<'py>(
        &self,
        py: Python<'py>,
        requested_schema: Option<&Bound<'py, PyCapsule>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if requested_schema.is_some() {
            return Err(errors::diagnostic(
                py,
                &errors::invalid(
                    "requested export schema casts are unsupported; inspect the exact admitted schema",
                ),
            ));
        }
        let reader = py
            .detach(|| self.stream.reader())
            .map_err(|error| errors::diagnostic(py, &error))?;
        PyRecordBatchReader::new(reader)
            .into_pyobject(py)?
            .call_method0("__arrow_c_stream__")
    }
    /// Pull through the interpreter's signal owner rather than a C-stream callback.
    /// This route and capsule export consume the same stream exactly once.
    #[pyo3(signature = () -> "object | None")]
    fn next_batch<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        crate::wait::precheck(py)?;
        if tokio::runtime::Handle::try_current().is_ok() {
            return Err(errors::diagnostic(
                py,
                &errors::invalid("synchronous Arrow stream cannot re-enter its async executor"),
            ));
        }
        // Reject concurrent consumption before waiting on any producer-owned lock.
        let mut reader = self.controlled.try_lock().map_err(|_| {
            errors::diagnostic(
                py,
                &errors::invalid("another controlled Arrow pull is active"),
            )
        })?;
        if reader.is_none() {
            *reader = Some(
                self.stream
                    .reader()
                    .map_err(|error| errors::diagnostic(py, &error))?,
            );
        }
        // Iterator terminal state cannot turn an original stream failure into
        // successful EOF on a later controlled pull. Consumption exclusion was
        // established first, so this does not wait behind another active pull.
        if let Some(error) = py
            .detach(|| self.stream.failure())
            .map_err(|error| errors::diagnostic(py, &error))?
        {
            return Err(errors::diagnostic(py, error.as_ref()));
        }
        let mut owned = reader.take().ok_or_else(|| {
            errors::diagnostic(py, &errors::invalid("controlled Arrow reader is absent"))
        })?;
        let cancel = self.stream.cancellation_token();
        let batch = crate::workflow::blocking_native_call(
            py,
            || {
                owned
                    .next()
                    .transpose()
                    .map_err(|source| pse_engine::EngineError::Infrastructure {
                        op: "pull admitted Arrow stream".into(),
                        source: Box::new(source),
                    })
            },
            || cancel.cancel(),
        );
        *reader = Some(owned);
        // The owned worker has joined here. Terminalize interruption without
        // acquiring the stream lock from inside the pending producer.
        match batch {
            Ok(batch) => batch
                .map(|batch| PyRecordBatch::new(batch).into_pyarrow(py))
                .transpose(),
            Err(error) => {
                if cancel.is_cancelled()
                    && let Err(cause) = self.stream.cancel()
                    && error.cause(py).is_none()
                {
                    error.set_cause(py, Some(errors::diagnostic(py, &cause)));
                }
                Err(error)
            }
        }
    }
    #[getter]
    fn failure(&self, py: Python<'_>) -> PyResult<Option<super::DiagnosticReport>> {
        py.detach(|| self.stream.failure())
            .map(|error| error.map(|error| super::DiagnosticReport::observe(error.as_ref())))
            .map_err(|error| errors::diagnostic(py, &error))
    }
    fn close(&self, py: Python<'_>) -> PyResult<()> {
        py.detach(|| self.stream.close())
            .map_err(|error| errors::diagnostic(py, &error))
    }
    fn cancel(&self, py: Python<'_>) -> PyResult<()> {
        py.detach(|| self.stream.cancel())
            .map_err(|error| errors::diagnostic(py, &error))
    }
}
