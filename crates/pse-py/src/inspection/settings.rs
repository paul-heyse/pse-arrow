// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Mechanical projection; runtime owns default resolution and validation.
use super::errors;
use pyo3::prelude::*;

/// Explicit deployment budget, validated by the native runtime.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct EngineSettings {
    pub(super) budget: pse_runtime::ResourceBudget,
}
macro_rules! projection {
    ($budget:ident; $($name:ident: $ty:ty, $hint:literal $(, $default:expr)? => $out:ty, $read:expr;)*) => {
        #[pymethods]
        impl EngineSettings {
            #[new]
            #[expect(clippy::too_many_arguments, reason = "generated keyword projection of the native declaration")]
            #[pyo3(signature = (*, $($name: $hint $(= $default)?,)* cache=None))]
            fn new(py: Python<'_>, $(#[pyo3(from_py_with = super::inputs::extract)] $name: $ty,)* cache: Option<PyRef<'_, super::CacheSettings>>) -> PyResult<Self> {
                let input = pse_runtime::settings::EngineSettingsInput { $($name,)* };
                let cache = cache.map(|value| value.budget.clone());
                py.detach(|| input.resolve(cache)).map(|budget| Self { budget }).map_err(|error| errors::diagnostic(py, &error))
            }
            $(#[getter] fn $name(&self) -> $out { let $budget = &self.budget; $read })*
            #[getter] fn cache(&self) -> super::CacheSettings { super::CacheSettings { budget: self.budget.cache.clone() } }
        }
    };
}
pse_runtime::engine_settings_fields!(projection, budget);
