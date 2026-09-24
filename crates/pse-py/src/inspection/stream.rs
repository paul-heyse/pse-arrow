// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::errors;
use pse_catalog::inspection::{TableReader, stream::BatchStream};
use pyo3::{prelude::*, types::PyCapsule};
use pyo3_arrow::{PyRecordBatchReader, PySchema};
use std::sync::Arc;

/// A one-consumption Arrow stream retaining admitted source ownership.
#[pyclass(frozen, module = "pse._native")]
#[derive(Debug)]
pub(crate) struct TableStream(BatchStream);
impl TableStream {
    pub(crate) fn from_batch(batch: pse_relations::columnar::FieldCheckedBatch) -> Self {
        let schema = batch.batch().schema();
        let mut next = Some(batch.batch().clone());
        Self(BatchStream::new(
            schema,
            pse_columnar::CancellationToken::new(),
            Box::new(move || Ok(next.take())),
        ))
    }
    pub(crate) fn new(mut reader: TableReader, runtime: Arc<super::runtime::Runtime>) -> Self {
        Self(BatchStream::new(
            reader.schema(),
            reader.cancellation_token(),
            Box::new(move || {
                if tokio::runtime::Handle::try_current().is_ok() {
                    return Err(errors::invalid(
                        "synchronous Arrow stream cannot re-enter its async executor",
                    ));
                }
                runtime.executor.block_on(reader.next_batch())
            }),
        ))
    }
}
#[pymethods]
impl TableStream {
    #[pyo3(signature = () -> "object")]
    fn __arrow_c_schema__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        PySchema::new(self.0.schema())
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
            .detach(|| self.0.reader())
            .map_err(|error| errors::diagnostic(py, &error))?;
        PyRecordBatchReader::new(reader)
            .into_pyobject(py)?
            .call_method0("__arrow_c_stream__")
    }
    #[getter]
    fn failure(&self, py: Python<'_>) -> PyResult<Option<super::DiagnosticReport>> {
        py.detach(|| self.0.failure())
            .map(|error| error.map(|error| super::DiagnosticReport::observe(error.as_ref())))
            .map_err(|error| errors::diagnostic(py, &error))
    }
    fn close(&self, py: Python<'_>) -> PyResult<()> {
        py.detach(|| self.0.close())
            .map_err(|error| errors::diagnostic(py, &error))
    }
    fn cancel(&self, py: Python<'_>) -> PyResult<()> {
        py.detach(|| self.0.cancel())
            .map_err(|error| errors::diagnostic(py, &error))
    }
}
