// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use arrow_array::{RecordBatch, RecordBatchReader};
use arrow_schema::{ArrowError, SchemaRef};
use pse_catalog::inspection::TableReader;
use pyo3::{prelude::*, types::PyCapsule};
use pyo3_arrow::{PyRecordBatchReader, PySchema};

use super::errors;

/// A one-consumption Arrow stream retaining admitted source ownership.
#[pyclass(frozen, module = "pse._native")]
#[derive(Debug)]
pub(crate) struct TableStream {
    reader: Arc<Mutex<ReaderState>>,
    schema: SchemaRef,
    consumed: AtomicBool,
    cancel: pse_ids::CancellationToken,
}

impl TableStream {
    pub(super) fn new(reader: TableReader, runtime: Arc<super::runtime::Runtime>) -> Self {
        Self {
            schema: reader.schema(),
            cancel: reader.cancellation_token(),
            reader: Arc::new(Mutex::new(ReaderState {
                reader: Some(reader),
                cancelled: false,
                runtime,
            })),
            consumed: AtomicBool::new(false),
        }
    }
}

#[pymethods]
impl TableStream {
    #[pyo3(signature = () -> "object")]
    fn __arrow_c_schema__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        PySchema::new(Arc::clone(&self.schema))
            .into_pyobject(py)?
            .call_method0("__arrow_c_schema__")
    }

    #[pyo3(signature = (requested_schema: "object | None"=None) -> "object")]
    fn __arrow_c_stream__<'py>(
        &self,
        py: Python<'py>,
        requested_schema: Option<Bound<'py, PyCapsule>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if requested_schema.is_some() {
            return Err(errors::diagnostic(
                py,
                &errors::invalid(
                    "requested export schema casts are unsupported; inspect the exact admitted schema",
                ),
            ));
        }
        self.reader
            .lock()
            .map_err(|_| errors::diagnostic(py, &errors::invalid("stream lock poisoned")))?
            .exportable()
            .map_err(|error| errors::diagnostic(py, &error))?;
        if self.consumed.swap(true, Ordering::AcqRel) {
            return Err(errors::diagnostic(py, &errors::closed()));
        }
        let reader = BridgeReader {
            reader: Arc::clone(&self.reader),
            schema: Arc::clone(&self.schema),
            terminal: false,
        };
        PyRecordBatchReader::new(Box::new(reader))
            .into_pyobject(py)?
            .call_method0("__arrow_c_stream__")
    }

    fn close(&self, py: Python<'_>) -> PyResult<()> {
        self.cancel.cancel();
        self.reader
            .lock()
            .map_err(|_| errors::diagnostic(py, &errors::invalid("stream lock poisoned")))?
            .close();
        Ok(())
    }
    fn cancel(&self, py: Python<'_>) -> PyResult<()> {
        self.cancel.cancel();
        let mut state = self
            .reader
            .lock()
            .map_err(|_| errors::diagnostic(py, &errors::invalid("stream lock poisoned")))?;
        state.cancelled = true;
        state.close();
        Ok(())
    }
}

#[derive(Debug)]
struct ReaderState {
    reader: Option<TableReader>,
    cancelled: bool,
    runtime: Arc<super::runtime::Runtime>,
}
impl ReaderState {
    fn exportable(&self) -> Result<(), pse_catalog::CatalogError> {
        if self.cancelled {
            return Err(pse_catalog::CatalogError::Cancelled);
        }
        if self.reader.is_none() {
            return Err(errors::closed());
        }
        Ok(())
    }
    fn next_batch(&mut self) -> Result<Option<RecordBatch>, pse_catalog::CatalogError> {
        if self.cancelled {
            return Err(pse_catalog::CatalogError::Cancelled);
        }
        let Some(reader) = self.reader.as_mut() else {
            return Ok(None);
        };
        if tokio::runtime::Handle::try_current().is_ok() {
            return Err(errors::invalid(
                "synchronous Arrow stream cannot re-enter its async executor",
            ));
        }
        match self.runtime.executor.block_on(reader.next_batch()) {
            Ok(Some(batch)) => Ok(Some(batch)),
            result => {
                self.reader = None;
                result
            }
        }
    }
    fn close(&mut self) {
        self.reader = None;
    }
}

#[derive(Debug)]
struct StreamFailure(pse_catalog::CatalogError);
impl std::fmt::Display for StreamFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&errors::message(&self.0))
    }
}
impl std::error::Error for StreamFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

struct BridgeReader {
    reader: Arc<Mutex<ReaderState>>,
    schema: SchemaRef,
    terminal: bool,
}
impl Iterator for BridgeReader {
    type Item = Result<RecordBatch, ArrowError>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.terminal {
            return None;
        }
        let result = self
            .reader
            .lock()
            .map_err(|_| errors::invalid("stream lock poisoned"))
            .and_then(|mut reader| reader.next_batch());
        match result {
            Ok(Some(batch)) => Some(Ok(batch)),
            Ok(None) => {
                self.terminal = true;
                None
            }
            Err(error) => {
                self.terminal = true;
                Some(Err(ArrowError::ExternalError(Box::new(StreamFailure(
                    error,
                )))))
            }
        }
    }
}
impl RecordBatchReader for BridgeReader {
    fn schema(&self) -> SchemaRef {
        Arc::clone(&self.schema)
    }
}
impl Drop for BridgeReader {
    fn drop(&mut self) {
        if let Ok(mut reader) = self.reader.lock() {
            reader.close();
        }
    }
}
