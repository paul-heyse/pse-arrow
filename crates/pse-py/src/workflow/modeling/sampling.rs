// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Owning design handle; exported documents are independent, bounded Python copies.
use super::*;
/// Retain the native point buffer and its prepaid capacity through every handle alias.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct NativeGeneratedDesign {
    pub(super) inner: pse_runtime::workflow::sampling::GeneratedDesign,
    pub(super) services: native::PreparationServices,
}
#[pymethods]
impl NativeGeneratedDesign {
    #[getter]
    fn count(&self) -> usize {
        self.inner.points().len()
    }
    /// Materialize a bounded independent document while retaining source and conversion owners.
    fn export(
        &self,
        py: Python<'_>,
    ) -> PyResult<documents::DocumentValue<pse_runtime::workflow::sampling::DesignExport>> {
        self.inner
            .export(&self.services)
            .map(documents::DocumentValue)
            .map_err(|error| errors::diagnostic(py, &error))
    }
}
