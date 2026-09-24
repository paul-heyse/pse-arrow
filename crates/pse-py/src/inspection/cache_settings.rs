// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Mechanical projection of the native cache policy input declaration.
use super::errors;
use pyo3::prelude::*;

/// Native cache capacities; zero disables a cache, and working capacity stays positive.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct CacheSettings {
    pub(super) budget: pse_runtime::DeltaCacheBudget,
}
macro_rules! projection {
    ($budget:ident; $($name:ident: $ty:ty, $hint:literal $(, $default:expr)? => $out:ty, $read:expr;)*) => {
        #[pymethods]
        impl CacheSettings {
            #[new]
            #[expect(clippy::too_many_arguments, reason = "generated keyword projection of the native declaration")]
            #[pyo3(signature = (*, $($name: $hint $(= $default)?),*))]
            fn new(py: Python<'_>, $(#[pyo3(from_py_with = super::inputs::extract)] $name: $ty,)*) -> PyResult<Self> {
                pse_catalog::cache_service::settings::CacheSettingsInput { $($name,)* }.resolve()
                    .map(|budget| Self { budget }).map_err(|error| errors::diagnostic(py, &error))
            }
            $(#[getter] fn $name(&self, py: Python<'_>) -> PyResult<$out> {
                let $budget = &self.budget;
                let read = || -> Result<$out, pse_engine::EngineError> { Ok($read) };
                read().map_err(|error| errors::diagnostic(py, &error))
            })*
        }
    };
}
pse_catalog::cache_settings_fields!(projection, budget);
