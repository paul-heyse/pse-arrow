// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(
    unsafe_code,
    reason = "checked extensions to the pinned HiGHS owning Rust model and C callbacks"
)]
//! HiGHS coefficient adapter with completion-owned native scheduler teardown.
use crate::{
    CoefficientProblem, GramCertificate, ProblemError,
    quality::{Quality, Tolerances, Violation, interval},
    solve::*,
};
use highs_sys as ffi;
use pse_math::binding::{ObjectiveSense, VariableDomain};
use std::{
    collections::BTreeMap,
    ffi::{CString, c_char, c_void},
    marker::PhantomData,
    rc::Rc,
    sync::{
        RwLock, RwLockReadGuard,
        atomic::{AtomicBool, Ordering},
    },
};
pub mod diagnostics;
static LIFECYCLE: RwLock<()> = RwLock::new(());
thread_local! {static ACTIVE:std::cell::Cell<bool>=const {std::cell::Cell::new(false)};}

/// Explicit native method; automatic remains a native class-specific decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    /// Native method selection.
    Choose,
    /// Simplex for LP.
    Simplex,
    /// Interior point for LP.
    Ipm,
    /// First-order primal-dual LP method.
    Pdlp,
}
/// Complete HiGHS-specific request on the unified lifecycle.
#[derive(Clone, Debug)]
pub struct Settings {
    /// Eligible LP algorithm; mixed models retain native class routing.
    pub method: Method,
    /// Opt-in native work, separate from the original candidate.
    pub diagnostics: diagnostics::Request,
    /// Partial source-attributed MIP start, with unspecified coordinates absent.
    pub sparse_start: Option<BTreeMap<pse_ids::SemanticId, f64>>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            method: Method::Choose,
            diagnostics: Default::default(),
            sparse_start: None,
        }
    }
}
/// Native model reuse is confined to one admitted worker and finite caller sequence.
pub struct Session {
    model: Option<highs::Model>,
    gate: Option<RwLockReadGuard<'static, ()>>,
    compatibility: Compatibility,
    structure: (Vec<usize>, Vec<usize>, Vec<VariableDomain>),
    _local: PhantomData<Rc<()>>,
}
impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HighsSession")
            .field("compatibility", &self.compatibility)
            .finish_non_exhaustive()
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        drop(self.model.take());
        drop(self.gate.take());
        // Header contract: reset is unsafe concurrently with ANY use of HiGHS.
        // CPU admission is held by the owning runtime until this blocking join ends.
        let _exclusive = LIFECYCLE
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        unsafe { ffi::Highs_resetGlobalScheduler(1) };
        ACTIVE.with(|a| a.set(false));
    }
}
fn check(code: i32, operation: &str) -> Result<(), ProblemError> {
    if code != ffi::STATUS_OK {
        Err(ProblemError::Contract(format!(
            "HiGHS {operation} returned {code}; upload/options warnings are not silently accepted"
        )))
    } else {
        Ok(())
    }
}
fn index(n: usize) -> Result<i32, ProblemError> {
    i32::try_from(n).map_err(|_| ProblemError::Contract("HiGHS index overflow".into()))
}
fn text(value: &str) -> Result<CString, ProblemError> {
    CString::new(value).map_err(|_| ProblemError::Contract("NUL in HiGHS option".into()))
}
fn bounds(value: f64) -> Result<f64, ProblemError> {
    if value.is_finite() && value.abs() >= 1e20 {
        Err(ProblemError::Contract(
            "finite HiGHS bound reaches infinity threshold".into(),
        ))
    } else {
        Ok(value)
    }
}
fn admit(
    p: &CoefficientProblem,
    certificate: Option<&GramCertificate>,
) -> Result<(), ProblemError> {
    p.validate_convex(certificate)?;
    let quadratic = p
        .hessian
        .as_ref()
        .is_some_and(|q| q.val().iter().any(|v| *v != 0.0));
    if quadratic && p.domains.iter().any(|d| *d != VariableDomain::Continuous) {
        return Err(ProblemError::Contract("HiGHS MIQP is unsupported".into()));
    }
    index(p.contract.variables.len())?;
    index(p.contract.rows.len())?;
    index(p.constraints.val().len())?;
    for (v, d) in p.contract.variables.iter().zip(&p.domains) {
        bounds(v.lower)?;
        bounds(v.upper)?;
        if *d == VariableDomain::Binary && v.lower.max(0.0).ceil() > v.upper.min(1.0).floor() {
            return Err(ProblemError::Contract("empty binary domain".into()));
        }
        if d.is_semi()
            && (!v.lower.is_finite()
                || !v.upper.is_finite()
                || v.lower <= 0.0
                || v.upper > 1e5
                || d.is_integer() && v.lower.ceil() > v.upper.floor())
        {
            return Err(ProblemError::Contract("pinned HiGHS semi domain needs a nonempty positive interval with upper bound <=1e5".into()));
        }
    }
    for &(l, u) in &p.bounds {
        bounds(l)?;
        bounds(u)?;
    }
    Ok(())
}
fn domain_bounds(p: &CoefficientProblem, c: usize) -> (f64, f64) {
    let v = &p.contract.variables[c];
    if p.domains[c] == VariableDomain::Binary {
        (v.lower.max(0.0), v.upper.min(1.0))
    } else {
        (v.lower, v.upper)
    }
}
fn upload(p: &CoefficientProblem) -> Result<highs::Model, ProblemError> {
    // The high-level uploader accepts native warnings. Upload through the checked C
    // API so coefficient pruning and other model changes cannot be silently accepted.
    let mut model = highs::Model::try_new(highs::ColProblem::new())
        .map_err(|e| ProblemError::Contract(format!("HiGHS allocation: {e:?}")))?;
    let start: Vec<_> = p
        .constraints
        .col_ptr()
        .iter()
        .map(|v| index(*v))
        .collect::<Result<_, _>>()?;
    let rows: Vec<_> = p
        .constraints
        .row_idx()
        .iter()
        .map(|v| index(*v))
        .collect::<Result<_, _>>()?;
    let lo: Vec<_> = (0..p.domains.len())
        .map(|c| domain_bounds(p, c).0)
        .collect();
    let hi: Vec<_> = (0..p.domains.len())
        .map(|c| domain_bounds(p, c).1)
        .collect();
    let rl: Vec<_> = p.bounds.iter().map(|v| v.0).collect();
    let ru: Vec<_> = p.bounds.iter().map(|v| v.1).collect();
    let domains: Vec<i32> = p
        .domains
        .iter()
        .map(|v| match v {
            VariableDomain::Continuous => 0,
            VariableDomain::Integer | VariableDomain::Binary => 1,
            VariableDomain::SemiContinuous => 2,
            VariableDomain::SemiInteger => 3,
        })
        .collect();
    check(
        unsafe {
            ffi::Highs_passMip(
                model.as_mut_ptr(),
                index(lo.len())?,
                index(rl.len())?,
                index(rows.len())?,
                1,
                if p.sense == ObjectiveSense::Minimize {
                    1
                } else {
                    -1
                },
                p.objective_constant,
                p.objective.as_ptr(),
                lo.as_ptr(),
                hi.as_ptr(),
                rl.as_ptr(),
                ru.as_ptr(),
                start.as_ptr(),
                rows.as_ptr(),
                p.constraints.val().as_ptr(),
                domains.as_ptr(),
            )
        },
        "exact model upload",
    )?;
    hessian(&mut model, p)?;
    Ok(model)
}
fn hessian(model: &mut highs::Model, p: &CoefficientProblem) -> Result<(), ProblemError> {
    let mut start = vec![0];
    let mut rows = vec![];
    let mut values = vec![];
    if let Some(q) = &p.hessian {
        for c in 0..q.ncols() {
            for (r, &v) in q.row_idx_of_col(c).zip(q.val_of_col(c)) {
                if r >= c {
                    rows.push(index(r)?);
                    values.push(v);
                }
            }
            start.push(index(rows.len())?);
        }
    }
    if p.hessian.is_none() {
        start.resize(p.contract.variables.len() + 1, 0);
    }
    check(
        unsafe {
            ffi::Highs_passHessian(
                model.as_mut_ptr(),
                index(p.contract.variables.len())?,
                index(rows.len())?,
                1,
                start.as_ptr(),
                rows.as_ptr(),
                values.as_ptr(),
            )
        },
        "exact Hessian upload",
    )
}
impl Session {
    /// Construct only after runtime CPU/memory admission; one session per owner thread.
    pub fn new(
        p: &CoefficientProblem,
        certificate: Option<&GramCertificate>,
        compatibility: Compatibility,
    ) -> Result<Self, ProblemError> {
        admit(p, certificate)?;
        if ACTIVE.with(|a| a.replace(true)) {
            return Err(ProblemError::Contract(
                "nested HiGHS session would deadlock scheduler teardown".into(),
            ));
        }
        let gate = LIFECYCLE
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut session = Self {
            model: None,
            gate: Some(gate),
            compatibility,
            structure: (
                p.constraints.col_ptr().to_vec(),
                p.constraints.row_idx().to_vec(),
                p.domains.clone(),
            ),
            _local: PhantomData,
        };
        session.model = Some(upload(p)?);
        Ok(session)
    }
    fn model(&mut self) -> Result<&mut highs::Model, ProblemError> {
        self.model
            .as_mut()
            .ok_or_else(|| ProblemError::Contract("destroyed HiGHS model".into()))
    }
    /// Update compatible costs, bounds, matrix values and Hessian. Numeric factors are
    /// refreshed by HiGHS; retaining allocation/basis does not promise factor reuse.
    pub fn update(
        &mut self,
        p: &CoefficientProblem,
        certificate: Option<&GramCertificate>,
        compatibility: Compatibility,
    ) -> Result<(), ProblemError> {
        admit(p, certificate)?;
        if compatibility.layout != self.compatibility.layout
            || compatibility.backend != Backend::Highs
            || self.structure
                != (
                    p.constraints.col_ptr().to_vec(),
                    p.constraints.row_idx().to_vec(),
                    p.domains.clone(),
                )
        {
            return Err(ProblemError::Contract("HiGHS update changes layout".into()));
        }
        let ptr = self.model()?.as_mut_ptr();
        check(
            unsafe {
                ffi::Highs_changeObjectiveSense(
                    ptr,
                    if p.sense == ObjectiveSense::Minimize {
                        1
                    } else {
                        -1
                    },
                )
            },
            "update objective sense",
        )?;
        for c in 0..p.contract.variables.len() {
            check(
                unsafe { ffi::Highs_changeColCost(ptr, index(c)?, p.objective[c]) },
                "update cost",
            )?;
            check(
                unsafe {
                    ffi::Highs_changeColBounds(
                        ptr,
                        index(c)?,
                        domain_bounds(p, c).0,
                        domain_bounds(p, c).1,
                    )
                },
                "update bound",
            )?;
            for (r, &value) in p
                .constraints
                .row_idx_of_col(c)
                .zip(p.constraints.val_of_col(c))
            {
                check(
                    unsafe { ffi::Highs_changeCoeff(ptr, index(r)?, index(c)?, value) },
                    "update coefficient",
                )?;
            }
        }
        for (r, &(l, u)) in p.bounds.iter().enumerate() {
            check(
                unsafe { ffi::Highs_changeRowBounds(ptr, index(r)?, l, u) },
                "update row bounds",
            )?;
        }
        check(
            unsafe { ffi::Highs_changeObjectiveOffset(ptr, p.objective_constant) },
            "update offset",
        )?;
        hessian(self.model()?, p)?;
        self.compatibility = compatibility;
        Ok(())
    }
    /// Run the native model and recover original coefficients, sense and source maps.
    pub fn solve(
        &mut self,
        p: &CoefficientProblem,
        controls: &Controls,
        method: Method,
        execution: Execution,
        tolerances: &Tolerances,
        warm: Option<&WarmStart>,
    ) -> Result<SolveReport, ProblemError> {
        controls.validate()?;
        let n = p.contract.variables.len();
        let m = p.contract.rows.len();
        tolerances.validate(n, m)?;
        reject_reserved(
            &controls.options,
            &[
                "threads",
                "parallel",
                "time_limit",
                "solver",
                "infinite_bound",
                "infinite_cost",
                "small_matrix_value",
                "large_matrix_value",
                "user_bound_scale",
                "user_cost_scale",
                "solve_relaxation",
                "mip_max_nodes",
                "simplex_iteration_limit",
                "ipm_iteration_limit",
                "pdlp_iteration_limit",
                "qp_iteration_limit",
                "primal_feasibility_tolerance",
                "dual_feasibility_tolerance",
                "mip_feasibility_tolerance",
                "log_file",
            ],
        )?;
        if controls
            .options
            .values()
            .any(|v| matches!(v,OptionValue::Text(t)if t.len()>=512))
        {
            return Err(ProblemError::Contract(
                "HiGHS text option exceeds native readback capacity".into(),
            ));
        }
        let mut options = controls.options.clone();
        options.extend([
            (
                "threads".into(),
                OptionValue::Integer(controls.threads as i32),
            ),
            (
                "parallel".into(),
                OptionValue::Text(if controls.threads == 1 { "off" } else { "on" }.into()),
            ),
            (
                "time_limit".into(),
                OptionValue::Real(controls.time_limit.as_secs_f64()),
            ),
            (
                "solver".into(),
                OptionValue::Text(
                    match method {
                        Method::Choose => "choose",
                        Method::Simplex => "simplex",
                        Method::Ipm => "ipm",
                        Method::Pdlp => "pdlp",
                    }
                    .into(),
                ),
            ),
            ("log_file".into(), OptionValue::Text(String::new())),
            ("output_flag".into(), OptionValue::Bool(false)),
        ]);
        let discrete = p.domains.iter().any(|d| *d != VariableDomain::Continuous);
        if discrete && method != Method::Choose {
            return Err(ProblemError::Contract(
                "explicit LP method cannot relax a mixed-integer model".into(),
            ));
        }
        for key in [
            "simplex_iteration_limit",
            "ipm_iteration_limit",
            "pdlp_iteration_limit",
            "qp_iteration_limit",
            "mip_max_nodes",
        ] {
            options.insert(key.into(), OptionValue::Integer(controls.iterations as i32));
        }
        for key in [
            "primal_feasibility_tolerance",
            "dual_feasibility_tolerance",
            "mip_feasibility_tolerance",
        ] {
            options.insert(key.into(), OptionValue::Real(controls.tolerance));
        }
        if let Some(stop) = execution.stopped() {
            let mut report =
                SolveReport::new(Backend::Highs, &p.contract, termination(0), &execution);
            report.termination.category = stop;
            return Ok(report);
        }
        let compatible = self.compatibility.clone();
        let model = self.model()?;
        for (key, value) in &options {
            let result = match value {
                OptionValue::Text(v) => model.try_set_option(key.as_str(), v.as_str()),
                OptionValue::Real(v) => model.try_set_option(key.as_str(), *v),
                OptionValue::Integer(v) => model.try_set_option(key.as_str(), *v),
                OptionValue::Bool(v) => model.try_set_option(key.as_str(), *v),
            };
            result.map_err(|_| ProblemError::Contract(format!("HiGHS rejected option {key}")))?;
        }
        if let Some(w) = warm {
            w.validate(&compatible)?;
            let WarmPayload::Highs {
                primal,
                dual,
                basis,
            } = &w.payload
            else {
                return Err(ProblemError::Contract("HiGHS seed class".into()));
            };
            if primal
                .as_ref()
                .is_some_and(|v| v.len() != n || v.iter().any(|v| !v.is_finite()))
                || dual.as_ref().is_some_and(|(c, r)| {
                    c.len() != n || r.len() != m || c.iter().chain(r).any(|v| !v.is_finite())
                })
            {
                return Err(ProblemError::Contract(
                    "HiGHS seed dimensions/values".into(),
                ));
            }
            model
                .try_set_solution(
                    primal.as_deref(),
                    None,
                    dual.as_ref().map(|(c, _)| c.as_slice()),
                    dual.as_ref().map(|(_, r)| r.as_slice()),
                )
                .map_err(|e| ProblemError::Contract(format!("HiGHS start: {e:?}")))?;
            if let Some(basis) = basis {
                if basis.columns.len() != n
                    || basis.rows.len() != m
                    || basis
                        .columns
                        .iter()
                        .chain(&basis.rows)
                        .any(|v| !(0..=4).contains(v))
                {
                    return Err(ProblemError::Contract(
                        "HiGHS basis dimensions/status".into(),
                    ));
                }
                check(
                    unsafe {
                        ffi::Highs_setBasis(
                            model.as_mut_ptr(),
                            basis.columns.as_ptr(),
                            basis.rows.as_ptr(),
                        )
                    },
                    "basis seed",
                )?;
            }
        }
        let ptr = model.as_mut_ptr();
        let callback_binding = CallbackBinding::new(ptr, execution.clone())?;
        let run = if execution.stopped().is_some() {
            0
        } else {
            unsafe { ffi::Highs_run(ptr) }
        };
        let panicked = callback_binding.context.panicked.load(Ordering::Acquire);
        drop(callback_binding);
        let code = unsafe { ffi::Highs_getModelStatus(ptr) };
        let mut report =
            SolveReport::new(Backend::Highs, &p.contract, termination(code), &execution);
        let (effective, defaults) = option_snapshot(ptr)?;
        report.options = effective;
        report.native_defaults = defaults;
        if let Some(stop) = execution.stopped() {
            report.termination.category = stop;
            report.termination.assurance = Assurance::None
        }
        if panicked {
            report.termination.category = Termination::Panic;
            report.termination.assurance = Assurance::None
        }
        report
            .metrics
            .insert("run.status".into(), Metric::Integer(i64::from(run)));
        for key in [
            "simplex_iteration_count",
            "ipm_iteration_count",
            "pdlp_iteration_count",
            "qp_iteration_count",
            "crossover_iteration_count",
            "primal_solution_status",
            "dual_solution_status",
            "basis_validity",
            "mip_node_count",
            "objective_function_value",
            "mip_dual_bound",
            "mip_gap",
            "max_integrality_violation",
            "num_primal_infeasibilities",
            "num_dual_infeasibilities",
            "max_primal_infeasibility",
            "max_dual_infeasibility",
            "sum_primal_infeasibilities",
            "sum_dual_infeasibilities",
            "max_primal_residual_error",
            "max_dual_residual_error",
            "num_primal_residual_errors",
            "num_dual_residual_errors",
            "primal_dual_objective_error",
            "num_complementarity_violations",
            "max_complementarity_violation",
        ] {
            if let Some(value) = info(ptr, key)? {
                report.metrics.insert(key.into(), value);
            }
        }
        report.provenance.insert(
            "native".into(),
            "HiGHS 1.14.0; highs 2.4.0; highs-sys 1.14.3".into(),
        );
        report.provenance.insert(
            "interrupt".into(),
            "simplex/IPM/MIP callbacks; QP native time limit only".into(),
        );
        report.provenance.insert("duals".into(),"native authored-sense row multipliers and reduced costs; no split bound-dual fabrication".into());
        let primal = matches!(
            report.metrics.get("primal_solution_status"),
            Some(Metric::Integer(1 | 2))
        );
        let dual = matches!(
            report.metrics.get("dual_solution_status"),
            Some(Metric::Integer(1 | 2))
        );
        if primal {
            let mut x = vec![0.0; n];
            let mut cd = vec![0.0; n];
            let mut rv = vec![0.0; m];
            let mut rd = vec![0.0; m];
            check(
                unsafe {
                    ffi::Highs_getSolution(
                        ptr,
                        x.as_mut_ptr(),
                        cd.as_mut_ptr(),
                        rv.as_mut_ptr(),
                        rd.as_mut_ptr(),
                    )
                },
                "solution",
            )?;
            if x.iter().all(|v| v.is_finite()) {
                let objective = coefficient_objective(p, &x);
                report.candidate = Some(Candidate {
                    primal: x.clone(),
                    objective: Some(objective),
                    row_dual: dual.then(|| rd.clone()),
                    bound_dual: None,
                    reduced_costs: dual.then(|| cd.clone()),
                    slacks: None,
                });
                let quality = coefficient_quality(p, &x, tolerances)?;
                if !quality.feasible() {
                    report.termination.assurance = Assurance::None
                }
                report.quality = Some(quality);
                let basis = if matches!(
                    report.metrics.get("basis_validity"),
                    Some(Metric::Integer(1))
                ) {
                    let mut b = Basis {
                        columns: vec![0; n],
                        rows: vec![0; m],
                    };
                    check(
                        unsafe {
                            ffi::Highs_getBasis(ptr, b.columns.as_mut_ptr(), b.rows.as_mut_ptr())
                        },
                        "basis",
                    )?;
                    Some(b)
                } else {
                    None
                };
                report.warm_start = Some(WarmStart {
                    compatibility: compatible,
                    payload: WarmPayload::Highs {
                        primal: Some(x),
                        dual: dual.then_some((cd, rd)),
                        basis,
                    },
                });
            }
        }
        if report.candidate.is_none() {
            report.termination.assurance = Assurance::None
        }
        Ok(report)
    }
}
// HiGHS_getOptionName uses malloc in the pinned C wrapper. Pair it with the
// platform C allocator, never CString::from_raw (which owns Rust allocations).
unsafe extern "C" {
    fn free(ptr: *mut c_void);
}
struct NativeName(*mut c_char);
impl Drop for NativeName {
    fn drop(&mut self) {
        unsafe { free(self.0.cast()) }
    }
}
fn option_snapshot(ptr: *const c_void) -> Result<(Options, Options), ProblemError> {
    let count = unsafe { ffi::Highs_getNumOptions(ptr) };
    if !(0..=4096).contains(&count) {
        return Err(ProblemError::Contract(
            "native option inventory bound".into(),
        ));
    }
    let mut current = Options::new();
    let mut defaults = Options::new();
    for i in 0..count {
        let mut name = NativeName(std::ptr::null_mut());
        check(
            unsafe { ffi::Highs_getOptionName(ptr, i, &mut name.0) },
            "option name",
        )?;
        if name.0.is_null() {
            return Err(ProblemError::Contract(
                "native option name allocation".into(),
            ));
        }
        let key = unsafe { std::ffi::CStr::from_ptr(name.0) }
            .to_str()
            .map_err(|_| ProblemError::Contract("native option name UTF8".into()))?
            .to_owned();
        let mut kind = 0;
        check(
            unsafe { ffi::Highs_getOptionType(ptr, name.0, &mut kind) },
            "option type",
        )?;
        let (c, d) = match kind {
            0 => {
                let (mut c, mut d) = (0, 0);
                check(
                    unsafe { ffi::Highs_getBoolOptionValues(ptr, name.0, &mut c, &mut d) },
                    "bool option readback",
                )?;
                (OptionValue::Bool(c != 0), OptionValue::Bool(d != 0))
            }
            1 => {
                let (mut c, mut d) = (0, 0);
                check(
                    unsafe {
                        ffi::Highs_getIntOptionValues(
                            ptr,
                            name.0,
                            &mut c,
                            std::ptr::null_mut(),
                            std::ptr::null_mut(),
                            &mut d,
                        )
                    },
                    "int option readback",
                )?;
                (OptionValue::Integer(c), OptionValue::Integer(d))
            }
            2 => {
                let (mut c, mut d) = (0.0, 0.0);
                check(
                    unsafe {
                        ffi::Highs_getDoubleOptionValues(
                            ptr,
                            name.0,
                            &mut c,
                            std::ptr::null_mut(),
                            std::ptr::null_mut(),
                            &mut d,
                        )
                    },
                    "double option readback",
                )?;
                (OptionValue::Real(c), OptionValue::Real(d))
            }
            3 => {
                let (mut c, mut d) = ([0i8; 512], [0i8; 512]);
                check(
                    unsafe {
                        ffi::Highs_getStringOptionValues(
                            ptr,
                            name.0,
                            c.as_mut_ptr(),
                            d.as_mut_ptr(),
                        )
                    },
                    "string option readback",
                )?;
                let read = |v: &[i8; 512]| {
                    unsafe { std::ffi::CStr::from_ptr(v.as_ptr()) }
                        .to_string_lossy()
                        .into_owned()
                };
                (OptionValue::Text(read(&c)), OptionValue::Text(read(&d)))
            }
            _ => return Err(ProblemError::Contract("unknown native option type".into())),
        };
        current.insert(key.clone(), c);
        defaults.insert(key, d);
    }
    Ok((current, defaults))
}
fn info(ptr: *const c_void, name: &str) -> Result<Option<Metric>, ProblemError> {
    let name = text(name)?;
    let mut kind = 0;
    if unsafe { ffi::Highs_getInfoType(ptr, name.as_ptr(), &mut kind) } != 0 {
        return Ok(None);
    }
    let value = match kind {
        1 => {
            let mut v = 0;
            check(
                unsafe { ffi::Highs_getIntInfoValue(ptr, name.as_ptr(), &mut v) },
                "integer info",
            )?;
            Metric::Integer(i64::from(v))
        }
        -1 => {
            let mut v = 0;
            check(
                unsafe { ffi::Highs_getInt64InfoValue(ptr, name.as_ptr(), &mut v) },
                "int64 info",
            )?;
            Metric::Integer(v)
        }
        2 => {
            let mut v = 0.0;
            check(
                unsafe { ffi::Highs_getDoubleInfoValue(ptr, name.as_ptr(), &mut v) },
                "real info",
            )?;
            Metric::Real(v)
        }
        _ => return Ok(None),
    };
    Ok(Some(value))
}
struct CallbackBinding {
    ptr: *mut c_void,
    context: Box<Callback>,
}
impl CallbackBinding {
    fn new(ptr: *mut c_void, execution: Execution) -> Result<Self, ProblemError> {
        let mut binding = Self {
            ptr,
            context: Box::new(Callback {
                execution,
                panicked: AtomicBool::new(false),
            }),
        };
        check(
            unsafe {
                ffi::Highs_setCallback(ptr, Some(callback), (&raw mut *binding.context).cast())
            },
            "callback",
        )?;
        for kind in [1, 2, 3, 4, 5, 6] {
            check(
                unsafe { ffi::Highs_startCallback(ptr, kind) },
                "callback kind",
            )?;
        }
        Ok(binding)
    }
}
impl Drop for CallbackBinding {
    fn drop(&mut self) {
        unsafe {
            ffi::Highs_setCallback(self.ptr, None, std::ptr::null_mut());
        }
    }
}
struct Callback {
    execution: Execution,
    panicked: AtomicBool,
}
unsafe extern "C" fn callback(
    kind: i32,
    _message: *const c_char,
    out: *const ffi::HighsCallbackDataOut,
    input: *mut ffi::HighsCallbackDataIn,
    data: *mut c_void,
) {
    let Some(c) = (unsafe { data.cast::<Callback>().as_ref() }) else {
        return;
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if let Some(out) = unsafe { out.as_ref() } {
            let mut values = BTreeMap::from([
                ("kind".into(), Metric::Integer(i64::from(kind))),
                ("seconds".into(), Metric::Real(out.running_time)),
            ]);
            if matches!(kind, 3..=6) {
                for (k, v) in [
                    ("primal_bound", out.mip_primal_bound),
                    ("dual_bound", out.mip_dual_bound),
                    ("gap", out.mip_gap),
                ] {
                    values.insert(k.into(), Metric::Real(v));
                }
                values.insert("nodes".into(), Metric::Integer(out.mip_node_count));
            }
            if kind == 1 {
                values.insert(
                    "simplex.iterations".into(),
                    Metric::Integer(i64::from(out.simplex_iteration_count)),
                );
            }
            if kind == 2 {
                values.insert(
                    "ipm.iterations".into(),
                    Metric::Integer(i64::from(out.ipm_iteration_count)),
                );
            }
            c.execution.progress.push(Event {
                phase: "highs.callback".into(),
                elapsed: c.execution.started.elapsed(),
                values,
            });
        }
    }));
    if result.is_err() {
        c.panicked.store(true, Ordering::Release)
    }
    if let Some(input) = unsafe { input.as_mut() } {
        input.user_interrupt =
            i32::from(c.execution.stopped().is_some() || c.panicked.load(Ordering::Acquire));
    }
}
/// Preserve every pinned native model status.
pub fn termination(code: i32) -> NativeTermination {
    let name = highs::HighsModelStatus::try_from(code)
        .map_or_else(|_| format!("Unknown({code})"), |s| format!("{s:?}"));
    let (category, assurance) = match code {
        7 => (Termination::Success, Assurance::NativeOptimal),
        8 => (Termination::Infeasible, Assurance::None),
        9 => (Termination::InfeasibleOrUnbounded, Assurance::None),
        10 => (Termination::Unbounded, Assurance::None),
        11 | 12 | 14 | 16 | 18 => (Termination::Limit, Assurance::None),
        13 => (Termination::TimeLimit, Assurance::None),
        17 => (Termination::Cancelled, Assurance::None),
        3..=5 => (Termination::Numerical, Assurance::None),
        _ => (Termination::Invalid, Assurance::None),
    };
    NativeTermination {
        code: i64::from(code),
        name,
        message: None,
        category,
        assurance,
    }
}
/// Exact declared objective convention `constant + c*x + 1/2 x'Qx`.
pub fn coefficient_objective(p: &CoefficientProblem, x: &[f64]) -> f64 {
    let mut value =
        p.objective_constant + p.objective.iter().zip(x).map(|(c, x)| c * x).sum::<f64>();
    if let Some(q) = &p.hessian {
        for c in 0..q.ncols() {
            for (r, &v) in q.row_idx_of_col(c).zip(q.val_of_col(c)) {
                value += 0.5 * x[r] * v * x[c];
            }
        }
    }
    value
}
/// Recompute all affine rows and disjunctive domain violations independently.
pub fn coefficient_quality(
    p: &CoefficientProblem,
    x: &[f64],
    t: &Tolerances,
) -> Result<Quality, ProblemError> {
    t.validate(p.contract.variables.len(), p.contract.rows.len())?;
    if x.len() != p.contract.variables.len() || x.iter().any(|v| !v.is_finite()) {
        return Err(ProblemError::Contract(
            "invalid coefficient candidate".into(),
        ));
    }
    let mut activity = vec![0.0; p.bounds.len()];
    for (c, &x) in x.iter().enumerate() {
        for (r, &v) in p
            .constraints
            .row_idx_of_col(c)
            .zip(p.constraints.val_of_col(c))
        {
            activity[r] += v * x;
        }
    }
    let rows = p
        .contract
        .rows
        .iter()
        .zip(activity)
        .zip(&p.bounds)
        .zip(&t.rows)
        .map(|(((id, x), (l, u)), t)| Violation {
            id: *id,
            physical: interval(x, *l, *u),
            tolerance: *t,
        })
        .collect();
    let mut bounds = Vec::new();
    let mut integrality = Vec::new();
    for (((v, d), &x), tolerance) in p
        .contract
        .variables
        .iter()
        .zip(&p.domains)
        .zip(x)
        .zip(&t.variables)
    {
        let physical = if d.is_semi() {
            x.abs().min(interval(x, v.lower, v.upper))
        } else {
            interval(
                x,
                if *d == VariableDomain::Binary {
                    v.lower.max(0.0)
                } else {
                    v.lower
                },
                if *d == VariableDomain::Binary {
                    v.upper.min(1.0)
                } else {
                    v.upper
                },
            )
        };
        bounds.push(Violation {
            id: v.id,
            physical,
            tolerance: *tolerance,
        });
        if d.is_integer() {
            integrality.push(Violation {
                id: v.id,
                physical: (x - x.round()).abs(),
                tolerance: t.integrality,
            });
        }
    }
    Quality::new(rows, bounds, integrality)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn full_status_mapping_preserves_limits_and_ambiguity() {
        for c in 0..=18 {
            assert!(!termination(c).name.starts_with("Unknown("));
        }
        assert_eq!(termination(9).category, Termination::InfeasibleOrUnbounded);
        assert_eq!(termination(14).assurance, Assurance::None);
        assert!(bounds(1e20).is_err());
    }
    #[test]
    fn abi_and_scheduler_entrypoints_are_pinned() {
        assert_eq!(size_of::<ffi::HighsInt>(), 4);
        std::hint::black_box(ffi::Highs_resetGlobalScheduler);
        std::hint::black_box(ffi::Highs_setCallback);
        let _ = std::ffi::CStr::from_bytes_with_nul(b"threads\0").unwrap();
    }
    fn problem() -> CoefficientProblem {
        let o = crate::solver_tests::Polynomial::new();
        CoefficientProblem {
            contract: o.c,
            objective: vec![2.0],
            objective_constant: 7.0,
            sense: ObjectiveSense::Maximize,
            domains: vec![VariableDomain::Binary],
            assumptions: crate::solver_tests::stamp(Backend::Highs).data,
            constraints: o.matrix,
            hessian: None,
            bounds: vec![(0.0, 3.0)],
        }
    }
    #[test]
    fn checked_upload_clips_binary_bounds_and_preserves_authored_sense_offset() {
        let p = problem();
        let mut s = Session::new(&p, None, crate::solver_tests::stamp(Backend::Highs)).unwrap();
        let ptr = s.model().unwrap().as_mut_ptr();
        let (mut n, mut nnz, mut cost, mut lower, mut upper) = (0, 0, 0.0, 0.0, 0.0);
        check(
            unsafe {
                ffi::Highs_getColsByRange(
                    ptr,
                    0,
                    0,
                    &mut n,
                    &mut cost,
                    &mut lower,
                    &mut upper,
                    &mut nnz,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
            },
            "test bounds",
        )
        .unwrap();
        assert_eq!((n, cost, lower, upper), (1, 2.0, 0.0, 1.0));
        let mut sense = 0;
        let mut offset = 0.0;
        check(
            unsafe { ffi::Highs_getObjectiveSense(ptr, &mut sense) },
            "test sense",
        )
        .unwrap();
        check(
            unsafe { ffi::Highs_getObjectiveOffset(ptr, &mut offset) },
            "test offset",
        )
        .unwrap();
        assert_eq!((sense, offset), (-1, 7.0));
        let (effective, defaults) = option_snapshot(ptr).unwrap();
        assert_eq!(effective.len(), defaults.len());
        assert!(effective.len() > 50);
        assert!(defaults.contains_key("threads"));
        assert!(
            s.sparse_start(&p, &BTreeMap::from([(crate::solver_tests::id(99), 1.0)]))
                .is_err()
        );
    }
    #[test]
    fn native_upload_warnings_are_refused_and_semi_quality_keeps_zero_branch() {
        let mut p = problem();
        p.constraints.val_mut()[0] = 1e-12;
        assert!(Session::new(&p, None, crate::solver_tests::stamp(Backend::Highs)).is_err());
        p.constraints.val_mut()[0] = 1.0;
        p.domains[0] = VariableDomain::SemiContinuous;
        p.contract.variables[0].lower = 2.0;
        p.contract.variables[0].upper = 3.0;
        let t = Tolerances {
            variables: vec![1e-8],
            rows: vec![1e-8],
            integrality: 1e-8,
        };
        assert!(coefficient_quality(&p, &[0.0], &t).unwrap().feasible());
        assert!(!coefficient_quality(&p, &[1.0], &t).unwrap().feasible());
    }
}

/// Independently reconstruct original affine row values and authored objective.
/// The native model has shifted row bounds; source constants are applied exactly once.
pub fn coefficient_observation(
    p: &CoefficientProblem,
    x: &[f64],
    constants: &[f64],
    bounds: Vec<(f64, f64)>,
) -> Result<crate::quality::Observation, ProblemError> {
    if x.len() != p.contract.variables.len() || constants.len() != p.bounds.len() {
        return Err(ProblemError::Contract(
            "coefficient observation dimensions".into(),
        ));
    }
    let column = faer::ColRef::from_slice(x);
    let activity = p.constraints.as_ref() * column;
    let values = activity.iter().zip(constants).map(|(v, c)| v + c).collect();
    let objective = faer::ColRef::from_slice(&p.objective).transpose() * column;
    let quadratic = p.hessian.as_ref().map_or(0.0, |q| {
        let product = q * column;
        0.5 * (column.transpose() * product.as_ref())
    });
    crate::quality::Observation::from_values(
        Some(p.objective_constant + objective + quadratic),
        values,
        bounds,
    )
}
