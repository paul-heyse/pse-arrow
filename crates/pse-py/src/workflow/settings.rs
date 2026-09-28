// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Typed settings projection (ADR-0113). Every setting is a pse-owned Rust type whose serde
//! encoding is its one boundary form: names are registry `as_str` spellings or the owning
//! type's serde spellings, an omitted argument takes the Rust default, and admission and
//! numerical validation stay native. There is no string-to-enum table here.
use super::invalid;
use pse_backend_native::{
    dynamics,
    execution::{self, SETTINGS_VERSION, SettingsDocument},
    presolve::{Pass, Policy, PolicyKind},
    solve::{OptionValue, Options, SolverSelection},
};
use pse_runtime::math::solves::{ConvexityPolicy, SolverProfile};
use pyo3::{
    prelude::*,
    types::{PyBool, PyDict, PyFloat, PyInt, PyString},
};
use serde::{Serialize, de::DeserializeOwned};
use std::{collections::BTreeMap, time::Duration};

/// Bounded JSON projection of a Python value through the `json` module; string enums
/// cross as their values.
fn json_value(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<serde_json::Value> {
    let text: String = py
        .import("json")?
        .call_method1("dumps", (value,))?
        .extract()?;
    if text.len() > 1 << 20 {
        return Err(invalid(py, "settings extent"));
    }
    serde_json::from_str(&text).map_err(|e| invalid(py, e.to_string()))
}
fn python_value<'py>(py: Python<'py>, value: &serde_json::Value) -> PyResult<Bound<'py, PyAny>> {
    let text = serde_json::to_string(value).map_err(|e| invalid(py, e.to_string()))?;
    py.import("json")?.call_method1("loads", (text,))
}
/// Keyword fields as one serde object; none means every field takes its Rust default.
fn keyword_fields(
    py: Python<'_>,
    fields: Option<&Bound<'_, PyDict>>,
) -> PyResult<serde_json::Value> {
    fields.map_or_else(
        || Ok(serde_json::Value::Object(serde_json::Map::new())),
        |fields| json_value(py, fields.as_any()),
    )
}
/// A boundary name, parsed by the owning type's serde spelling (a registry enum's is its
/// `as_str`); a refusal lists every accepted name.
pub(super) fn named<T: DeserializeOwned>(
    py: Python<'_>,
    what: &str,
    name: &str,
) -> PyResult<T> {
    serde_json::from_value(serde_json::Value::String(name.into()))
        .map_err(|e| invalid(py, format!("{what}: {e}")))
}
/// The boundary name of a value: its serde spelling.
fn name_of<T: Serialize>(py: Python<'_>, value: &T) -> PyResult<String> {
    match serde_json::to_value(value) {
        Ok(serde_json::Value::String(name)) => Ok(name),
        _ => Err(invalid(py, "setting has no boundary name")),
    }
}
fn errors(py: Python<'_>, e: &(dyn miette::Diagnostic + 'static)) -> PyErr {
    crate::inspection::errors::diagnostic(py, e)
}

/// ID-keyed numerical requirements and native algorithm controls. Every argument is
/// optional; an omitted one takes the native default of the solve profile.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct SolveSettings {
    pub(super) profile: SolverProfile,
}
#[pymethods]
impl SolveSettings {
    #[new]
    #[expect(
        clippy::too_many_arguments,
        reason = "mechanical keyword-only native controls"
    )]
    #[pyo3(signature=(*, numerics: "dict[str, object] | None"=None, intent=None, backend=None, settings=None, presolve=None, presolve_options: "dict[str, bool | int | float | str] | None"=None, required_passes=None, time_limit=None, iterations: "int | None"=None, threads: "int | None"=None, history: "int | None"=None, hessian=None, reuse=None, start=None, options: "dict[str, bool | int | float | str] | None"=None, convexity_absolute=None, convexity_relative=None))]
    fn new(
        py: Python<'_>,
        numerics: Option<&Bound<'_, PyDict>>,
        intent: Option<&str>,
        backend: Option<&str>,
        settings: Option<&BackendSettings>,
        presolve: Option<&str>,
        presolve_options: Option<&Bound<'_, PyDict>>,
        required_passes: Option<Vec<String>>,
        time_limit: Option<f64>,
        #[pyo3(from_py_with = crate::inspection::inputs::extract)] iterations: Option<u32>,
        #[pyo3(from_py_with = crate::inspection::inputs::extract)] threads: Option<usize>,
        #[pyo3(from_py_with = crate::inspection::inputs::extract)] history: Option<usize>,
        hessian: Option<&str>,
        reuse: Option<&str>,
        start: Option<&str>,
        options: Option<&Bound<'_, PyDict>>,
        convexity_absolute: Option<f64>,
        convexity_relative: Option<f64>,
    ) -> PyResult<Self> {
        let mut profile = SolverProfile::default();
        // Registry spellings; `certify` parses and routing decides whether it is served.
        if let Some(intent) = intent {
            profile.intent = named(py, "native solve intent", intent)?;
        }
        if let Some(backend) = backend {
            profile.selection = SolverSelection::Explicit(named(py, "native backend", backend)?);
        }
        if let Some(settings) = settings {
            profile.backend = settings.inner.clone();
        }
        let required = required_passes
            .unwrap_or_default()
            .iter()
            .map(|pass| named::<Pass>(py, "presolve pass", pass))
            .collect::<PyResult<_>>()?;
        let kind = match presolve {
            Some(kind) => named::<PolicyKind>(py, "presolve policy", kind)?,
            None => profile.presolve.kind(),
        };
        profile.presolve = Policy::new(kind, &options_from_py(py, presolve_options)?, required)
            .map_err(|e| errors(py, &e))?;
        let controls = &mut profile.controls;
        if let Some(time_limit) = time_limit {
            controls.time_limit =
                Duration::try_from_secs_f64(time_limit).map_err(|e| invalid(py, e.to_string()))?;
        }
        if let Some(iterations) = iterations {
            controls.iterations = iterations;
        }
        if let Some(threads) = threads {
            controls.threads = threads;
        }
        if let Some(history) = history {
            controls.history = history;
        }
        if let Some(hessian) = hessian {
            controls.hessian = named(py, "Hessian mode", hessian)?;
        }
        if let Some(reuse) = reuse {
            controls.reuse = named(py, "native reuse policy", reuse)?;
        }
        if let Some(start) = start {
            controls.start = named(py, "start policy", start)?;
        }
        controls.options = options_from_py(py, options)?;
        controls.validate().map_err(|e| errors(py, &e))?;
        profile.numerics = numerical_policy(py, numerics)?;
        profile.convexity =
            ConvexityPolicy::from_tolerances(convexity_absolute, convexity_relative)
                .map_err(|e| errors(py, &e))?;
        Ok(Self { profile })
    }
    /// Registry name of the mathematical purpose.
    #[getter]
    fn intent(&self) -> &'static str {
        self.profile.intent.as_str()
    }
    /// Registry name of the explicitly selected backend; `None` is automatic routing.
    #[getter]
    fn backend(&self) -> Option<&'static str> {
        match self.profile.selection {
            SolverSelection::Explicit(backend) => Some(backend.as_str()),
            SolverSelection::Auto => None,
        }
    }
    /// Typed backend settings; `None` is the routed backend's native defaults.
    #[getter]
    fn settings(&self) -> Option<BackendSettings> {
        self.profile.backend.backend().map(|_| BackendSettings {
            inner: self.profile.backend.clone(),
        })
    }
    /// Presolve policy kind.
    #[getter]
    fn presolve(&self, py: Python<'_>) -> PyResult<String> {
        name_of(py, &self.profile.presolve.kind())
    }
    /// Wall-clock allowance in seconds.
    #[getter]
    fn time_limit(&self) -> f64 {
        self.profile.controls.time_limit.as_secs_f64()
    }
    /// Native iteration limit.
    #[getter]
    fn iterations(&self) -> u32 {
        self.profile.controls.iterations
    }
    /// Admitted native thread count.
    #[getter]
    fn threads(&self) -> usize {
        self.profile.controls.threads
    }
    /// Retained progress history.
    #[getter]
    fn history(&self) -> usize {
        self.profile.controls.history
    }
    /// Hessian representation.
    #[getter]
    fn hessian(&self, py: Python<'_>) -> PyResult<String> {
        name_of(py, &self.profile.controls.hessian)
    }
    /// Native reuse policy.
    #[getter]
    fn reuse(&self, py: Python<'_>) -> PyResult<String> {
        name_of(py, &self.profile.controls.reuse)
    }
    /// Registry name of the numerical start policy.
    #[getter]
    fn start(&self) -> &'static str {
        self.profile.controls.start.as_str()
    }
}

/// Typed settings of one algebraic backend: its adapter's pse-owned settings type
/// (ADR-0113). The backend is a registry name, fields are that type's serde fields, an
/// omitted field takes its Rust default and an unknown field is refused. A backend this
/// build does not link is refused.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct BackendSettings {
    pub(super) inner: execution::BackendSettings,
}
impl BackendSettings {
    fn document(&self, py: Python<'_>) -> PyResult<SettingsDocument> {
        self.inner.document().map_err(|e| errors(py, &e))
    }
}
#[pymethods]
impl BackendSettings {
    #[new]
    #[pyo3(signature = (backend, /, **fields: "object"))]
    fn new(py: Python<'_>, backend: &str, fields: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
        let backend = named(py, "native backend", backend)?;
        execution::BackendSettings::from_fields(backend, keyword_fields(py, fields)?)
            .map(|inner| Self { inner })
            .map_err(|e| errors(py, &e))
    }
    /// Registry name of the backend whose adapter owns these settings.
    #[getter]
    fn backend(&self, py: Python<'_>) -> PyResult<&'static str> {
        Ok(self.document(py)?.backend.as_str())
    }
    /// Every field, with Rust defaults for those not supplied.
    #[pyo3(signature = () -> "dict[str, object]")]
    fn fields<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        python_value(py, &self.document(py)?.settings)
    }
    /// Complete settings identity, derived from serde (request identity input).
    #[getter]
    fn identity(&self, py: Python<'_>) -> PyResult<String> {
        self.inner
            .identity()
            .map(|id| id.to_prefixed())
            .map_err(|e| errors(py, &e))
    }
    /// The versioned settings document.
    fn to_json(&self, py: Python<'_>) -> PyResult<String> {
        serde_json::to_string(&self.document(py)?).map_err(|e| invalid(py, e.to_string()))
    }
    /// Admit a versioned settings document; unknown versions and fields are refused.
    #[staticmethod]
    fn from_json(py: Python<'_>, source: &str) -> PyResult<Self> {
        if source.len() > 1 << 20 {
            return Err(invalid(py, "backend settings extent"));
        }
        let document: SettingsDocument =
            serde_json::from_str(source).map_err(|e| invalid(py, e.to_string()))?;
        execution::BackendSettings::from_document(document)
            .map(|inner| Self { inner })
            .map_err(|e| errors(py, &e))
    }
    fn __eq__(&self, py: Python<'_>, other: &Self) -> PyResult<bool> {
        Ok(self.document(py)? == other.document(py)?)
    }
    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        Ok(format!(
            "BackendSettings.from_json({:?})",
            self.to_json(py)?
        ))
    }
}

/// The versioned boundary document of a dynamics profile settings type.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileDocument<T> {
    version: u32,
    settings: T,
}
/// A frozen projection of one pse-owned dynamics settings type: keyword fields are its
/// serde fields, omitted fields take its Rust defaults and unknown ones are refused.
macro_rules! profile_settings {
    ($(#[$doc:meta])* $name:ident => $ty:ty) => {
        $(#[$doc])*
        #[pyclass(frozen, skip_from_py_object, module = "pse._native")]
        #[derive(Clone, Debug)]
        pub(crate) struct $name {
            pub(super) inner: $ty,
        }
        #[pymethods]
        impl $name {
            #[new]
            #[pyo3(signature = (**fields: "object"))]
            fn new(py: Python<'_>, fields: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
                serde_json::from_value(keyword_fields(py, fields)?)
                    .map(|inner| Self { inner })
                    .map_err(|e| invalid(py, format!("{}: {e}", stringify!($name))))
            }
            /// Every field, with Rust defaults for those not supplied.
            #[pyo3(signature = () -> "dict[str, object]")]
            fn fields<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
                let value = serde_json::to_value(&self.inner).map_err(|e| invalid(py, e.to_string()))?;
                python_value(py, &value)
            }
            /// The versioned settings document.
            fn to_json(&self, py: Python<'_>) -> PyResult<String> {
                serde_json::to_string(&ProfileDocument { version: SETTINGS_VERSION, settings: &self.inner })
                    .map_err(|e| invalid(py, e.to_string()))
            }
            /// Admit a versioned settings document; unknown versions and fields are refused.
            #[staticmethod]
            fn from_json(py: Python<'_>, source: &str) -> PyResult<Self> {
                if source.len() > 1 << 20 {
                    return Err(invalid(py, "dynamics settings extent"));
                }
                let document: ProfileDocument<serde_json::Value> =
                    serde_json::from_str(source).map_err(|e| invalid(py, e.to_string()))?;
                if document.version != SETTINGS_VERSION {
                    return Err(invalid(py, format!("unknown settings document version {}", document.version)));
                }
                serde_json::from_value(document.settings)
                    .map(|inner| Self { inner })
                    .map_err(|e| invalid(py, format!("{}: {e}", stringify!($name))))
            }
            fn __eq__(&self, other: &Self) -> bool {
                self.inner == other.inner
            }
            fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
                Ok(format!("{}.from_json({:?})", stringify!($name), self.to_json(py)?))
            }
        }
    };
}
profile_settings!(
    /// Typed Diffsol scheme and Newton linear solver of a simulation profile.
    DiffsolSettings => dynamics::DiffsolSettings
);
profile_settings!(
    /// Typed IDAS linear solver, sensitivity corrector, start and state signs of a
    /// simulation profile.
    IdasSettings => dynamics::IdasSettings
);

fn options_from_py(py: Python<'_>, options: Option<&Bound<'_, PyDict>>) -> PyResult<Options> {
    let mut native_options = BTreeMap::new();
    if let Some(options) = options {
        for (key, value) in options {
            let key = key.extract::<String>()?;
            let v = if value.is_instance_of::<PyBool>() {
                OptionValue::Bool(value.extract()?)
            } else if value.is_instance_of::<PyInt>() {
                OptionValue::Integer(value.extract()?)
            } else if value.is_instance_of::<PyFloat>() {
                OptionValue::Real(value.extract()?)
            } else if value.is_instance_of::<PyString>() {
                OptionValue::Text(value.extract()?)
            } else {
                return Err(invalid(py, "native option must be bool, int, float or str"));
            };
            native_options.insert(key, v);
        }
    }
    Ok(native_options)
}

pub(super) fn numerical_policy(
    py: Python<'_>,
    value: Option<&Bound<'_, PyDict>>,
) -> PyResult<pse_model::numerics::NumericalPolicy> {
    let policy: pse_model::numerics::NumericalPolicy = match value {
        None => Default::default(),
        Some(value) => serde_json::from_value(json_value(py, value.as_any())?)
            .map_err(|e| invalid(py, e.to_string()))?,
    };
    policy.validate().map_err(|e| invalid(py, e.to_string()))?;
    Ok(policy)
}
