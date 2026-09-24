// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Thin option projection; native adapters own validation and effective-option reporting.
use super::invalid;
use pse_backend_native::{
    presolve::{Policy, Scaling},
    quality::Tolerances,
    solve::*,
};
use pse_runtime::math::solves::{BackendSettings, SolverProfile};
use pyo3::{
    prelude::*,
    types::{PyBool, PyDict, PyFloat, PyInt, PyString},
};
use std::{collections::BTreeMap, time::Duration};
/// Explicit physical tolerances and native common controls. Array order is the admitted
/// case's canonical semantic-ID order, never source insertion order.
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
    #[pyo3(signature=(*,variable_tolerances,row_tolerances,intent="optimize",backend="auto",presolve="auto",presolve_options: "dict[str, bool | int | float | str] | None"=None,required_passes=None,time_limit: "float"=300.0,iterations: "int"=3000,tolerance: "float"=1e-8,threads: "int"=1,history: "int"=256,hessian="exact",reuse="fresh",integrality_tolerance: "float"=1e-8,options: "dict[str, bool | int | float | str] | None"=None,objective_scale=None,variable_scales=None,row_scales=None))]
    fn new(
        py: Python<'_>,
        variable_tolerances: Vec<f64>,
        row_tolerances: Vec<f64>,
        intent: &str,
        backend: &str,
        presolve: &str,
        presolve_options: Option<&Bound<'_, PyDict>>,
        required_passes: Option<Vec<String>>,
        #[pyo3(from_py_with = crate::inspection::inputs::extract)] time_limit: f64,
        #[pyo3(from_py_with = crate::inspection::inputs::extract)] iterations: u32,
        #[pyo3(from_py_with = crate::inspection::inputs::extract)] tolerance: f64,
        #[pyo3(from_py_with = crate::inspection::inputs::extract)] threads: usize,
        #[pyo3(from_py_with = crate::inspection::inputs::extract)] history: usize,
        hessian: &str,
        reuse: &str,
        #[pyo3(from_py_with = crate::inspection::inputs::extract)] integrality_tolerance: f64,
        options: Option<&Bound<'_, PyDict>>,
        objective_scale: Option<f64>,
        variable_scales: Option<Vec<f64>>,
        row_scales: Option<Vec<f64>>,
    ) -> PyResult<Self> {
        let intent = match intent {
            "optimize" => SolveIntent::Optimize,
            "root" => SolveIntent::Root,
            "feasible_point" => SolveIntent::FeasiblePoint,
            "initialize" => SolveIntent::Initialize,
            _ => return Err(invalid(py, "unknown native solve intent")),
        };
        let selection = match backend {
            "auto" => SolverSelection::Auto,
            "ipopt" => SolverSelection::Explicit(Backend::Ipopt),
            "pounce" => SolverSelection::Explicit(Backend::Pounce),
            "kinsol" => SolverSelection::Explicit(Backend::Kinsol),
            "highs" => SolverSelection::Explicit(Backend::Highs),
            _ => {
                return Err(invalid(
                    py,
                    "unknown algebraic solver; cones require the conic representation",
                ));
            }
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
            tolerance,
            threads,
            history,
            hessian,
            reuse,
            options: native_options,
        };
        controls.validate().map_err(|e| errors(py, &e))?;
        let tolerances = Tolerances {
            variables: variable_tolerances,
            rows: row_tolerances,
            integrality: integrality_tolerance,
        };
        tolerances
            .validate(tolerances.variables.len(), tolerances.rows.len())
            .map_err(|e| errors(py, &e))?;
        let scaling = match (objective_scale, variable_scales, row_scales) {
            (None, None, None) => None,
            (Some(objective), Some(variables), Some(constraints)) => Some(Scaling {
                objective,
                variables,
                constraints,
            }),
            _ => {
                return Err(invalid(
                    py,
                    "numerical scaling requires objective, variable and row scales together",
                ));
            }
        };
        if let Some(s) = &scaling {
            s.validate(tolerances.variables.len(), tolerances.rows.len())
                .map_err(|e| errors(py, &e))?;
        }
        Ok(Self {
            profile: SolverProfile {
                presolve,
                scaling,
                intent,
                selection,
                controls,
                backend: BackendSettings::Default,
                tolerances,
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
