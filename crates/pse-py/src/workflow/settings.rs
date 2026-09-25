// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Thin option projection; native adapters own validation and effective-option reporting.
use super::invalid;
use pse_backend_native::{
    presolve::Policy,
    solve::{
        Backend, Controls, HessianMode, OptionValue, Options, ReusePolicy, SolveIntent,
        SolverSelection,
    },
};
use pse_runtime::math::solves::{BackendSettings, SolverProfile};
use pyo3::{
    prelude::*,
    types::{PyBool, PyDict, PyFloat, PyInt, PyString},
};
use std::{collections::BTreeMap, time::Duration};
/// ID-keyed numerical requirements and native algorithm controls.
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
    #[pyo3(signature=(*,numerics: "dict[str, object] | None"=None,intent="optimize",backend="auto",presolve="auto",presolve_options: "dict[str, bool | int | float | str] | None"=None,required_passes=None,time_limit: "float"=300.0,iterations: "int"=3000,threads: "int"=1,history: "int"=256,hessian="exact",reuse="fresh",start="no_prior_start",options: "dict[str, bool | int | float | str] | None"=None,convexity_absolute=None,convexity_relative=None))]
    fn new(
        py: Python<'_>,
        numerics: Option<&Bound<'_, PyDict>>,
        intent: &str,
        backend: &str,
        presolve: &str,
        presolve_options: Option<&Bound<'_, PyDict>>,
        required_passes: Option<Vec<String>>,
        #[pyo3(from_py_with = crate::inspection::inputs::extract)] time_limit: f64,
        #[pyo3(from_py_with = crate::inspection::inputs::extract)] iterations: u32,
        #[pyo3(from_py_with = crate::inspection::inputs::extract)] threads: usize,
        #[pyo3(from_py_with = crate::inspection::inputs::extract)] history: usize,
        hessian: &str,
        reuse: &str,
        start: &str,
        options: Option<&Bound<'_, PyDict>>,
        convexity_absolute: Option<f64>,
        convexity_relative: Option<f64>,
    ) -> PyResult<Self> {
        let intent = match intent {
            "optimize" => SolveIntent::Optimize,
            "root" => SolveIntent::Root,
            "feasible_point" => SolveIntent::FeasiblePoint,
            "initialize" => SolveIntent::Initialize,
            _ => return Err(invalid(py, "unknown native solve intent")),
        };
        let selection = if backend == "auto" {
            SolverSelection::Auto
        } else {
            let selected: Backend = backend
                .parse()
                .map_err(|_| invalid(py, "unknown native backend"))?;
            SolverSelection::Explicit(selected)
        };
        let required = required_passes
            .unwrap_or_default()
            .iter()
            .map(|s| {
                use pse_backend_native::presolve::Pass;
                match s.as_str() {
                    "linear_bounds" => Ok(Pass::LinearBounds),
                    "redundant_rows" => Ok(Pass::RedundantRows),
                    "affine_elimination" => Ok(Pass::AffineElimination),
                    "fbbt" => Ok(Pass::Fbbt),
                    "rank_diagnostics" => Ok(Pass::RankDiagnostics),
                    "auxiliary" => Ok(Pass::Auxiliary),
                    _ => Err(invalid(py, "unknown required presolve pass")),
                }
            })
            .collect::<PyResult<std::collections::BTreeSet<_>>>()?;
        if presolve != "explicit" && (presolve_options.is_some() || !required.is_empty()) {
            return Err(invalid(
                py,
                "native presolve options/required passes need explicit policy",
            ));
        }
        let presolve = match presolve {
            "auto" => Policy::Auto,
            "off" => Policy::Off,
            "explicit" => {
                Policy::from_native_options(&options_from_py(py, presolve_options)?, required)
                    .map_err(|e| errors(py, &e))?
            }
            _ => return Err(invalid(py, "presolve must be auto, off or explicit")),
        };
        let hessian = match hessian {
            "exact" => HessianMode::Exact,
            "limited_memory" => HessianMode::LimitedMemory,
            _ => return Err(invalid(py, "unknown Hessian policy")),
        };
        let reuse = match reuse {
            "fresh" => ReusePolicy::Fresh,
            "allow_rebuild" => ReusePolicy::AllowRebuild,
            "require_reuse" => ReusePolicy::RequireReuse,
            _ => return Err(invalid(py, "unknown native reuse policy")),
        };
        let time_limit =
            Duration::try_from_secs_f64(time_limit).map_err(|e| invalid(py, e.to_string()))?;
        let native_options = options_from_py(py, options)?;
        let controls = Controls {
            time_limit,
            iterations,
            accuracy: Default::default(),
            threads,
            history,
            hessian,
            reuse,
            start: start
                .parse()
                .map_err(|_| invalid(py, "unknown start policy"))?,
            options: native_options,
        };
        controls.validate().map_err(|e| errors(py, &e))?;
        let numerics = numerical_policy(py, numerics)?;
        let convexity = match (convexity_absolute, convexity_relative) {
            (None, None) => pse_runtime::math::solves::ConvexityPolicy::Exact,
            (Some(absolute), Some(relative))
                if absolute.is_finite()
                    && relative.is_finite()
                    && absolute >= 0.0
                    && relative >= 0.0 =>
            {
                pse_runtime::math::solves::ConvexityPolicy::Numerical { absolute, relative }
            }
            _ => {
                return Err(invalid(
                    py,
                    "numerical convexity requires both finite nonnegative tolerances",
                ));
            }
        };
        Ok(Self {
            profile: SolverProfile {
                presolve,
                numerics,
                convexity,
                intent,
                selection,
                controls,
                backend: BackendSettings::Default,
            },
        })
    }
}
fn errors(py: Python<'_>, e: &pse_backend_native::ProblemError) -> PyErr {
    crate::inspection::errors::diagnostic(py, e)
}

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
    let policy = match value {
        None => pse_model::numerics::NumericalPolicy::default(),
        Some(value) => {
            let json: String = py
                .import("json")?
                .call_method1("dumps", (value,))?
                .extract()?;
            if json.len() > 1 << 20 {
                return Err(invalid(py, "numerical policy extent"));
            }
            serde_json::from_str(&json).map_err(|e| invalid(py, e.to_string()))?
        }
    };
    policy.validate().map_err(|e| invalid(py, e.to_string()))?;
    Ok(policy)
}
