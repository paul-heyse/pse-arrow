// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Opt-in native diagnostics preserve their scope and never replace the original solve.
use super::*;
use pse_ids::SemanticId;

/// Requested native diagnostic work, bounded by the original attempt's deadline.
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct Request {
    /// Native primal/dual rays when available for the continuous model.
    pub rays: bool,
    /// Native irreducible infeasible subsystem; MIP scope is its LP relaxation.
    pub iis: bool,
    /// Basis sensitivity ranges for an optimal continuous LP with a valid basis.
    pub ranging: bool,
    /// Separate feasibility-relaxation solve on a copied LP/MIP. Negative penalties
    /// forbid violation, as in the native API; no penalty is inferred from units.
    pub relaxation: Option<Penalties>,
}
/// Complete physical penalty declarations for native feasibility relaxation.
#[derive(Clone, Debug, serde::Serialize)]
pub struct Penalties {
    /// Global lower-bound, upper-bound and constraint penalties.
    pub global: [f64; 3],
    /// Optional per-variable lower-bound penalties.
    pub lower: Option<Vec<f64>>,
    /// Optional per-variable upper-bound penalties.
    pub upper: Option<Vec<f64>>,
    /// Optional per-row penalties.
    pub rows: Option<Vec<f64>>,
}
impl Penalties {
    /// Validate complete dimensions and finite weights before native mutation.
    pub fn validate(&self, n: usize, m: usize) -> Result<(), ProblemError> {
        if self.global.iter().any(|v| !v.is_finite())
            || [(&self.lower, n), (&self.upper, n), (&self.rows, m)]
                .iter()
                .any(|(v, n)| {
                    v.as_ref()
                        .is_some_and(|v| v.len() != *n || v.iter().any(|v| !v.is_finite()))
                })
        {
            return Err(ProblemError::Contract(
                "relaxation penalty dimensions/values".into(),
            ));
        }
        Ok(())
    }
}
/// IIS bound membership in original source coordinates; integer codes are retained.
#[derive(Clone, Debug)]
pub struct Iis {
    /// Original variable IDs and native IIS bound code.
    pub columns: Vec<(SemanticId, i32)>,
    /// Original row IDs and native IIS bound code.
    pub rows: Vec<(SemanticId, i32)>,
    /// Native status for every original column.
    pub column_status: Vec<i32>,
    /// Native status for every original row.
    pub row_status: Vec<i32>,
    /// Whether the diagnostic concerns a continuous relaxation of a discrete model.
    pub relaxation_only: bool,
}
/// One native ranging family, indexed by original columns or rows as its name states.
#[derive(Clone, Debug)]
pub struct Range {
    /// Source variable or row coordinate.
    pub ids: Vec<SemanticId>,
    /// Limiting coefficient/bound value (infinity remains meaningful).
    pub value: Vec<f64>,
    /// Authored-sense objective at that limit.
    pub objective: Vec<f64>,
    /// Native entering variable index; negatives are native sentinels and indices
    /// >= number of columns refer to row slacks.
    pub entering: Vec<i32>,
    /// Native leaving variable index in the same documented augmented ordering.
    pub leaving: Vec<i32>,
}
impl Range {
    fn new(ids: Vec<SemanticId>) -> Self {
        let n = ids.len();
        Self {
            ids,
            value: vec![0.0; n],
            objective: vec![0.0; n],
            entering: vec![0; n],
            leaving: vec![0; n],
        }
    }
}
/// Separate relaxed attempt, explicitly outside original-model assurance.
#[derive(Clone, Debug)]
pub struct Relaxation {
    /// Raw native operation status.
    pub operation_status: i32,
    /// Native model termination after diagnostic relaxation.
    pub termination: NativeTermination,
    /// Available relaxed coordinates; never the original solve's candidate.
    pub primal: Option<Vec<f64>>,
}
/// Available native diagnostics and explicit reasons for requested unavailable data.
#[derive(Clone, Debug, Default)]
pub struct Report {
    /// Native primal unbounded direction in original variable order.
    pub primal_ray: Option<Vec<f64>>,
    /// Native dual infeasibility ray in original row order.
    pub dual_ray: Option<Vec<f64>>,
    /// Native irreducible subsystem if found.
    pub iis: Option<Iis>,
    /// Six native cost/bound range families when the basis supports them.
    pub ranging: BTreeMap<String, Range>,
    /// Separate explicitly penalized relaxation attempt.
    pub relaxation: Option<Relaxation>,
    /// Unavailable/refused/failed diagnostic reasons; absence never means zero.
    pub unavailable: BTreeMap<String, String>,
}
impl Session {
    /// Ask native diagnostics under the same owner/scheduler gate. A relaxation owns
    /// a separate model and cannot alter this session's coefficients or candidate.
    pub fn diagnose(
        &mut self,
        p: &CoefficientProblem,
        request: &Request,
        execution: &Execution,
    ) -> Result<Report, ProblemError> {
        let n = p.contract.variables.len();
        let m = p.contract.rows.len();
        if let Some(v) = &request.relaxation {
            v.validate(n, m)?;
        }
        let mut report = Report::default();
        if execution.stopped().is_some() {
            report.unavailable.insert(
                "all".into(),
                "attempt cancelled or deadline exhausted".into(),
            );
            return Ok(report);
        }
        let ptr = self.model()?.as_mut_ptr();
        check(
            unsafe {
                ffi::Highs_setDoubleOptionValue(
                    ptr,
                    c"time_limit".as_ptr(),
                    execution
                        .time_limit
                        .saturating_sub(execution.started.elapsed())
                        .as_secs_f64(),
                )
            },
            "remaining diagnostic time",
        )?;
        let _callbacks = CallbackBinding::new(ptr, execution.clone())?;
        let discrete = p.domains.iter().any(|d| *d != VariableDomain::Continuous);
        let quadratic = p
            .hessian
            .as_ref()
            .is_some_and(|v| v.val().iter().any(|v| *v != 0.0));
        if request.rays {
            if discrete || quadratic {
                report.unavailable.insert(
                    "rays".into(),
                    "adapter exposes rays only for continuous LP models".into(),
                );
            } else {
                for (primal, len) in [(true, n), (false, m)] {
                    let mut has = 0;
                    let mut ray = vec![0.0; len];
                    let status = unsafe {
                        if primal {
                            ffi::Highs_getPrimalRay(ptr, &mut has, ray.as_mut_ptr())
                        } else {
                            ffi::Highs_getDualRay(ptr, &mut has, ray.as_mut_ptr())
                        }
                    };
                    let key = if primal { "primal_ray" } else { "dual_ray" };
                    if status == 0 && has != 0 && ray.iter().all(|v| v.is_finite()) {
                        if primal {
                            report.primal_ray = Some(ray)
                        } else {
                            report.dual_ray = Some(ray)
                        }
                    } else {
                        report.unavailable.insert(
                            key.into(),
                            format!("native status={status}, available={has}"),
                        );
                    }
                }
            }
        }
        if request.iis {
            let (mut nc, mut nr) = (0, 0);
            let mut ci = vec![0; n];
            let mut ri = vec![0; m];
            let mut cb = vec![0; n];
            let mut rb = vec![0; m];
            let mut cs = vec![0; n];
            let mut rs = vec![0; m];
            let status = unsafe {
                ffi::Highs_getIis(
                    ptr,
                    &mut nc,
                    &mut nr,
                    ci.as_mut_ptr(),
                    ri.as_mut_ptr(),
                    cb.as_mut_ptr(),
                    rb.as_mut_ptr(),
                    cs.as_mut_ptr(),
                    rs.as_mut_ptr(),
                )
            };
            if status == 0
                && nc >= 0
                && nr >= 0
                && nc as usize <= n
                && nr as usize <= m
                && (nc > 0 || nr > 0)
            {
                let columns = ci[..nc as usize]
                    .iter()
                    .zip(&cb)
                    .map(|(&i, &b)| {
                        p.contract
                            .variables
                            .get(i as usize)
                            .map(|v| (v.id, b))
                            .ok_or_else(|| ProblemError::Contract("native IIS column index".into()))
                    })
                    .collect::<Result<_, _>>()?;
                let rows = ri[..nr as usize]
                    .iter()
                    .zip(&rb)
                    .map(|(&i, &b)| {
                        p.contract
                            .rows
                            .get(i as usize)
                            .map(|v| (*v, b))
                            .ok_or_else(|| ProblemError::Contract("native IIS row index".into()))
                    })
                    .collect::<Result<_, _>>()?;
                report.iis = Some(Iis {
                    columns,
                    rows,
                    column_status: cs,
                    row_status: rs,
                    relaxation_only: discrete,
                });
            } else {
                report.unavailable.insert(
                    "iis".into(),
                    format!("native status={status}, columns={nc}, rows={nr}"),
                );
            }
        }
        if request.ranging {
            if discrete
                || quadratic
                || unsafe { ffi::Highs_getModelStatus(ptr) } != 7
                || !matches!(info(ptr, "basis_validity")?, Some(Metric::Integer(1)))
            {
                report.unavailable.insert(
                    "ranging".into(),
                    "requires an optimal continuous LP and native valid basis".into(),
                );
            } else {
                let ids: Vec<_> = p.contract.variables.iter().map(|v| v.id).collect();
                let (mut cu, mut cd, mut bu, mut bd, mut ru, mut rd) = (
                    Range::new(ids.clone()),
                    Range::new(ids.clone()),
                    Range::new(ids.clone()),
                    Range::new(ids),
                    Range::new(p.contract.rows.clone()),
                    Range::new(p.contract.rows.clone()),
                );
                let status = unsafe {
                    ffi::Highs_getRanging(
                        ptr,
                        cu.value.as_mut_ptr(),
                        cu.objective.as_mut_ptr(),
                        cu.entering.as_mut_ptr(),
                        cu.leaving.as_mut_ptr(),
                        cd.value.as_mut_ptr(),
                        cd.objective.as_mut_ptr(),
                        cd.entering.as_mut_ptr(),
                        cd.leaving.as_mut_ptr(),
                        bu.value.as_mut_ptr(),
                        bu.objective.as_mut_ptr(),
                        bu.entering.as_mut_ptr(),
                        bu.leaving.as_mut_ptr(),
                        bd.value.as_mut_ptr(),
                        bd.objective.as_mut_ptr(),
                        bd.entering.as_mut_ptr(),
                        bd.leaving.as_mut_ptr(),
                        ru.value.as_mut_ptr(),
                        ru.objective.as_mut_ptr(),
                        ru.entering.as_mut_ptr(),
                        ru.leaving.as_mut_ptr(),
                        rd.value.as_mut_ptr(),
                        rd.objective.as_mut_ptr(),
                        rd.entering.as_mut_ptr(),
                        rd.leaving.as_mut_ptr(),
                    )
                };
                if status == 0 {
                    report.ranging = BTreeMap::from([
                        ("column_cost_up".into(), cu),
                        ("column_cost_down".into(), cd),
                        ("column_bound_up".into(), bu),
                        ("column_bound_down".into(), bd),
                        ("row_bound_up".into(), ru),
                        ("row_bound_down".into(), rd),
                    ]);
                } else {
                    report
                        .unavailable
                        .insert("ranging".into(), format!("native status={status}"));
                }
            }
        }
        if let Some(v) = &request.relaxation {
            if quadratic || execution.stopped().is_some() {
                report.unavailable.insert(
                    "relaxation".into(),
                    "requires LP/MIP and remaining attempt time".into(),
                );
            } else {
                let mut model = upload(p)?;
                model
                    .try_set_option(
                        "time_limit",
                        execution
                            .time_limit
                            .saturating_sub(execution.started.elapsed())
                            .as_secs_f64(),
                    )
                    .map_err(|_| ProblemError::Contract("diagnostic time limit".into()))?;
                let ptr = model.as_mut_ptr();
                let binding = CallbackBinding::new(ptr, execution.clone())?;
                let data =
                    |v: &Option<Vec<f64>>| v.as_ref().map_or(std::ptr::null(), |v| v.as_ptr());
                let status = unsafe {
                    ffi::Highs_feasibilityRelaxation(
                        ptr,
                        v.global[0],
                        v.global[1],
                        v.global[2],
                        data(&v.lower),
                        data(&v.upper),
                        data(&v.rows),
                    )
                };
                drop(binding);
                let termination = termination(unsafe { ffi::Highs_getModelStatus(ptr) });
                let mut x = vec![0.0; n];
                let primal = if matches!(
                    info(ptr, "primal_solution_status")?,
                    Some(Metric::Integer(1 | 2))
                ) && unsafe {
                    ffi::Highs_getSolution(
                        ptr,
                        x.as_mut_ptr(),
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                    )
                } == 0
                    && x.iter().all(|v| v.is_finite())
                {
                    Some(x)
                } else {
                    None
                };
                report.relaxation = Some(Relaxation {
                    operation_status: status,
                    termination,
                    primal,
                });
            }
        }
        Ok(report)
    }
    /// Native sparse MIP start. Unspecified coordinates stay unspecified; no zeros
    /// are fabricated. The start is restricted to declared source variable IDs.
    pub fn sparse_start(
        &mut self,
        p: &CoefficientProblem,
        values: &BTreeMap<SemanticId, f64>,
    ) -> Result<(), ProblemError> {
        if !p.domains.iter().any(|d| *d != VariableDomain::Continuous) {
            return Err(ProblemError::Contract(
                "sparse start requires mixed-linear model".into(),
            ));
        }
        let map: BTreeMap<_, _> = p
            .contract
            .variables
            .iter()
            .enumerate()
            .map(|(i, v)| (v.id, i))
            .collect();
        let mut indices = vec![];
        let mut x = vec![];
        for (id, v) in values {
            let i = *map
                .get(id)
                .ok_or_else(|| ProblemError::Contract("sparse start unknown source".into()))?;
            if !v.is_finite() {
                return Err(ProblemError::Contract("nonfinite sparse start".into()));
            }
            indices.push(index(i)?);
            x.push(*v);
        }
        check(
            unsafe {
                ffi::Highs_setSparseSolution(
                    self.model()?.as_mut_ptr(),
                    index(indices.len())?,
                    indices.as_ptr(),
                    x.as_ptr(),
                )
            },
            "sparse start",
        )
    }
}
