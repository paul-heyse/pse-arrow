// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(
    unsafe_code,
    reason = "checked extensions to the pinned HiGHS owning Rust model and C callbacks"
)]
//! HiGHS coefficient adapter with completion-owned native scheduler teardown.
use crate::{CoefficientProblem, ProblemError, quality::Tolerances, solve::*};
use highs_sys as ffi;
use pse_math::binding::ObjectiveSense;
use pse_model::generated::enums::ModelingVariableDomain;
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
#[cfg(test)]
mod mip_tests;
static LIFECYCLE: RwLock<()> = RwLock::new(());
thread_local! {static ACTIVE:std::cell::Cell<bool>=const {std::cell::Cell::new(false)};}

pub use crate::settings::highs::{Method, Settings};

/// Native model reuse is confined to one admitted worker and finite caller sequence.
pub struct Session {
    model: Option<highs::Model>,
    gate: Option<RwLockReadGuard<'static, ()>>,
    compatibility: Compatibility,
    structure: (Vec<usize>, Vec<usize>, Vec<ModelingVariableDomain>),
    pending_sparse: Option<BTreeMap<pse_ids::SemanticId, f64>>,
    /// The root cut pool of the last solve, when requested.
    cut_pool: Option<diagnostics::CutPool>,
    /// Solves run on this native model; a later one reuses its allocation.
    solves: u64,
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
    i32::try_from(n).map_err(|_| ProblemError::Unsupported("HiGHS index overflow".into()))
}
fn text(value: &str) -> Result<CString, ProblemError> {
    CString::new(value).map_err(|_| ProblemError::Contract("NUL in HiGHS option".into()))
}
fn bounds(value: f64) -> Result<f64, ProblemError> {
    if value.is_finite() && value.abs() >= 1e20 {
        Err(ProblemError::Unsupported(
            "finite HiGHS bound reaches infinity threshold".into(),
        ))
    } else {
        Ok(value)
    }
}
fn admit(
    p: &CoefficientProblem,
    certificate: Option<&dyn pse_math::convexity::QuadraticEvidence>,
) -> Result<(), ProblemError> {
    p.validate_convex(certificate)?;
    let quadratic = p
        .hessian
        .as_ref()
        .is_some_and(|q| q.val().iter().any(|v| *v != 0.0));
    if quadratic
        && p.domains
            .iter()
            .any(|d| *d != ModelingVariableDomain::Continuous)
    {
        return Err(ProblemError::Unsupported(
            "HiGHS MIQP is unsupported".into(),
        ));
    }
    index(p.contract.variables.len())?;
    index(p.contract.rows.len())?;
    index(p.constraints.val().len())?;
    for (v, d) in p.contract.variables.iter().zip(&p.domains) {
        bounds(v.lower)?;
        bounds(v.upper)?;
        if *d == ModelingVariableDomain::Binary
            && v.lower.max(0.0).ceil() > v.upper.min(1.0).floor()
        {
            return Err(ProblemError::Contract("empty binary domain".into()));
        }
        if d.is_semi()
            && (!v.lower.is_finite()
                || !v.upper.is_finite()
                || v.lower <= 0.0
                || v.upper > 1e5
                || d.is_integer() && v.lower.ceil() > v.upper.floor())
        {
            return Err(ProblemError::Unsupported("pinned HiGHS semi domain needs a nonempty positive interval with upper bound <=1e5".into()));
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
    if p.domains[c] == ModelingVariableDomain::Binary {
        (v.lower.max(0.0), v.upper.min(1.0))
    } else {
        (v.lower, v.upper)
    }
}
fn upload(p: &CoefficientProblem) -> Result<highs::Model, ProblemError> {
    // The high-level uploader accepts native warnings. Upload through the checked C
    // API so coefficient pruning and other model changes cannot be silently accepted.
    let mut model = highs::Model::try_new(highs::ColProblem::new())
        .map_err(|e| ProblemError::memory(format!("HiGHS allocation: {e:?}")))?;
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
            ModelingVariableDomain::Continuous => 0,
            ModelingVariableDomain::Integer | ModelingVariableDomain::Binary => 1,
            ModelingVariableDomain::Semicontinuous => 2,
            ModelingVariableDomain::Semiinteger => 3,
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
// Read back the complete original native model before trusting a primal, dual or bound.
// A feasible point alone cannot prove equivalence of the optimization problems.
fn verify_upload(model: &highs::Model, p: &CoefficientProblem) -> Result<(), ProblemError> {
    let ptr = model.as_ptr();
    let (mut nc, mut nr, mut nz, mut qz) = unsafe {
        (
            ffi::Highs_getNumCol(ptr),
            ffi::Highs_getNumRow(ptr),
            ffi::Highs_getNumNz(ptr),
            ffi::Highs_getHessianNumNz(ptr),
        )
    };
    if nc != index(p.objective.len())?
        || nr != index(p.bounds.len())?
        || nz < 0
        || qz < 0
        || nz as usize > p.constraints.val().len()
        || qz as usize > p.hessian.as_ref().map_or(0, |q| q.val().len())
    {
        return Err(ProblemError::Internal(
            "HiGHS model readback dimensions differ from upload".into(),
        ));
    }
    let (n, m) = (nc as usize, nr as usize);
    let (mut cost, mut lo, mut hi) = (vec![0.0; n], vec![0.0; n], vec![0.0; n]);
    let (mut rl, mut ru) = (vec![0.0; m], vec![0.0; m]);
    let (mut ap, mut ai, mut av) = (vec![0; n + 1], vec![0; nz as usize], vec![0.0; nz as usize]);
    let (mut qp, mut qi, mut qv) = (vec![0; n + 1], vec![0; qz as usize], vec![0.0; qz as usize]);
    let mut domains = vec![0; n];
    let mut sense = 0;
    let mut offset = 0.0;
    check(
        unsafe {
            ffi::Highs_getModel(
                ptr,
                1,
                1,
                &raw mut nc,
                &raw mut nr,
                &raw mut nz,
                &raw mut qz,
                &raw mut sense,
                &raw mut offset,
                cost.as_mut_ptr(),
                lo.as_mut_ptr(),
                hi.as_mut_ptr(),
                rl.as_mut_ptr(),
                ru.as_mut_ptr(),
                ap.as_mut_ptr(),
                ai.as_mut_ptr(),
                av.as_mut_ptr(),
                qp.as_mut_ptr(),
                qi.as_mut_ptr(),
                qv.as_mut_ptr(),
                domains.as_mut_ptr(),
            )
        },
        "full model readback",
    )?;
    // The C API promises n starts; the queried nonzero count supplies the sentinel.
    ap[n] = nz;
    qp[n] = qz;
    let same_bound =
        |a: f64, b: f64| a == b || (!b.is_finite() && a.signum() == b.signum() && a.abs() >= 1e20);
    let mut equal = sense
        == if p.sense == ObjectiveSense::Minimize {
            1
        } else {
            -1
        }
        && offset == p.objective_constant
        && cost == p.objective;
    for c in 0..n {
        let (l, u) = domain_bounds(p, c);
        equal &= same_bound(lo[c], l) && same_bound(hi[c], u);
        equal &= domains[c]
            == match p.domains[c] {
                ModelingVariableDomain::Continuous => 0,
                ModelingVariableDomain::Integer | ModelingVariableDomain::Binary => 1,
                ModelingVariableDomain::Semicontinuous => 2,
                ModelingVariableDomain::Semiinteger => 3,
            };
    }
    equal &= p
        .bounds
        .iter()
        .enumerate()
        .all(|(i, (l, u))| same_bound(rl[i], *l) && same_bound(ru[i], *u));
    let entries = |starts: &[i32],
                   rows: &[i32],
                   values: &[f64]|
     -> Result<BTreeMap<(usize, usize), f64>, ProblemError> {
        let mut out = BTreeMap::new();
        for c in 0..n {
            if starts[c] < 0 || starts[c] > starts[c + 1] || starts[c + 1] as usize > values.len() {
                return Err(ProblemError::Internal(
                    "invalid native sparse readback".into(),
                ));
            }
            for k in starts[c] as usize..starts[c + 1] as usize {
                if values[k] != 0.0 {
                    out.insert((rows[k] as usize, c), values[k]);
                }
            }
        }
        Ok(out)
    };
    let expected_a: BTreeMap<_, _> = (0..n)
        .flat_map(|c| {
            p.constraints
                .row_idx_of_col(c)
                .zip(p.constraints.val_of_col(c))
                .filter(|(_, v)| **v != 0.0)
                .map(move |(r, v)| ((r, c), *v))
        })
        .collect();
    let expected_q: BTreeMap<_, _> = p
        .hessian
        .iter()
        .flat_map(|q| {
            (0..n).flat_map(move |c| {
                q.row_idx_of_col(c)
                    .zip(q.val_of_col(c))
                    .filter(move |(r, v)| *r >= c && **v != 0.0)
                    .map(move |(r, v)| ((r, c), *v))
            })
        })
        .collect();
    equal &= entries(&ap, &ai, &av)? == expected_a && entries(&qp, &qi, &qv)? == expected_q;
    if equal {
        Ok(())
    } else {
        Err(ProblemError::Unsupported(
            "HiGHS upload differs from the complete admitted coefficient model".into(),
        ))
    }
}
impl Session {
    /// Construct only after runtime CPU/memory admission; one session per owner thread.
    pub fn new(
        p: &CoefficientProblem,
        certificate: Option<&dyn pse_math::convexity::QuadraticEvidence>,
        compatibility: Compatibility,
    ) -> Result<Self, ProblemError> {
        admit(p, certificate)?;
        if ACTIVE.with(|a| a.replace(true)) {
            return Err(ProblemError::Internal(
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
            pending_sparse: None,
            cut_pool: None,
            solves: 0,
            _local: PhantomData,
        };
        session.model = Some(upload(p)?);
        Ok(session)
    }
    fn model(&mut self) -> Result<&mut highs::Model, ProblemError> {
        self.model
            .as_mut()
            .ok_or_else(|| ProblemError::Internal("destroyed HiGHS model".into()))
    }
    /// Update compatible costs, bounds, matrix values and Hessian. Numeric factors are
    /// refreshed by HiGHS; retaining allocation/basis does not promise factor reuse.
    pub fn update(
        &mut self,
        p: &CoefficientProblem,
        certificate: Option<&dyn pse_math::convexity::QuadraticEvidence>,
        compatibility: Compatibility,
    ) -> Result<(), ProblemError> {
        admit(p, certificate)?;
        if !compatibility.same_session(&self.compatibility)
            || compatibility.backend != Backend::Highs
            || self.structure
                != (
                    p.constraints.col_ptr().to_vec(),
                    p.constraints.row_idx().to_vec(),
                    p.domains.clone(),
                )
        {
            return Err(ProblemError::Unsupported(
                "HiGHS update changes layout".into(),
            ));
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
    /// Run the native model and recover original coefficients, sense and source maps. The
    /// settings' method and node budget apply; its sparse start and diagnostics go through
    /// [`Self::sparse_start`] and [`Self::diagnose`], except that a requested cut pool is
    /// captured during this solve. `normalization` maps `p`'s coordinates to the original
    /// ones, in which the search's incumbents are reported while it runs.
    #[expect(
        clippy::too_many_arguments,
        reason = "one native solve binds its problem, coordinates, controls, accuracy, settings, execution, tolerances and seed"
    )]
    pub fn solve(
        &mut self,
        p: &CoefficientProblem,
        normalization: &pse_math::normalization::Normalization,
        controls: &Controls,
        accuracy: &ResolvedAccuracy,
        settings: &Settings,
        execution: Execution,
        tolerances: &Tolerances,
        warm: Option<&WarmStart>,
    ) -> Result<SolveReport, ProblemError> {
        controls.validate()?;
        let method = settings.method;
        if settings.nodes == Some(0) || settings.nodes.is_some_and(|v| v > i32::MAX as u32) {
            return Err(ProblemError::Contract(
                "a HiGHS node budget must be positive and within the native range".into(),
            ));
        }
        let n = p.contract.variables.len();
        let m = p.contract.rows.len();
        tolerances.validate(n, m)?;
        normalization.validate(n, m)?;
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
                "qp_regularization_value",
                "primal_feasibility_tolerance",
                "dual_feasibility_tolerance",
                "mip_feasibility_tolerance",
                "mip_abs_gap",
                "mip_rel_gap",
                "simplex_scale_strategy",
                "log_file",
            ],
        )?;
        if controls
            .options
            .values()
            .any(|v| matches!(v,OptionValue::Text(t)if t.len()>=512))
        {
            return Err(ProblemError::Unsupported(
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
        // HiGHS 1.15 makes the active-set QP hot start opt-in; a submitted QP start would
        // otherwise be silently ignored (PS-11). An explicit native option still wins, and
        // the option snapshot records what ran.
        options
            .entry("qp_allow_hot_start".into())
            .or_insert(OptionValue::Bool(true));
        let discrete = p
            .domains
            .iter()
            .any(|d| *d != ModelingVariableDomain::Continuous);
        let quadratic = p
            .hessian
            .as_ref()
            .is_some_and(|q| q.val().iter().any(|v| *v != 0.0));
        if !accuracy.native_scaling && (method != Method::Simplex || discrete || quadratic) {
            return Err(ProblemError::Unsupported("disabling all HiGHS algorithmic scaling is qualified only for explicit continuous simplex LP".into()));
        }
        if discrete && method != Method::Choose {
            return Err(ProblemError::Unsupported(
                "explicit LP method cannot relax a mixed-integer model".into(),
            ));
        }
        // HiGHS solves a continuous QP with its active-set QP solver whatever `solver`
        // says, and HiPO (the QP interior point) is not built: an explicit LP method on a
        // quadratic objective would be ignored or refused natively (F04).
        if quadratic && method != Method::Choose {
            return Err(ProblemError::Unsupported(
                "explicit LP method cannot solve a quadratic objective".into(),
            ));
        }
        for key in [
            "simplex_iteration_limit",
            "ipm_iteration_limit",
            "pdlp_iteration_limit",
            "qp_iteration_limit",
        ] {
            options.insert(key.into(), OptionValue::Integer(controls.iterations as i32));
        }
        // Nodes are not iterations (F10): the node budget is its own setting, and without
        // one the native default (`kHighsIInf`, no limit) is restored explicitly because a
        // retained model keeps the options of its previous solve.
        options.insert(
            "mip_max_nodes".into(),
            OptionValue::Integer(settings.nodes.map_or(i32::MAX, |v| v as i32)),
        );
        if quadratic {
            options.insert(
                "qp_regularization_value".into(),
                OptionValue::Real(qp_regularization(p, accuracy)),
            );
        }
        for (key, value) in [
            ("primal_feasibility_tolerance", accuracy.feasibility),
            ("dual_feasibility_tolerance", accuracy.stationarity),
            // HiGHS also uses its MIP feasibility tolerance in subproblems.
            (
                "mip_feasibility_tolerance",
                accuracy.integrality.min(accuracy.feasibility),
            ),
            ("mip_abs_gap", accuracy.mip_absolute_gap),
            ("mip_rel_gap", accuracy.mip_relative_gap),
        ] {
            options.insert(key.into(), OptionValue::Real(value));
        }
        if !accuracy.native_scaling {
            options.insert("simplex_scale_strategy".into(), OptionValue::Integer(0));
        }
        if let Some(stop) = execution.stopped() {
            let mut report =
                SolveReport::new(Backend::Highs, &p.contract, termination(0), &execution);
            report.termination.category = stop;
            return Ok(report);
        }
        let compatible = self.compatibility.clone();
        let sparse = self.pending_sparse.take();
        if sparse.is_some() && warm.is_some() {
            return Err(ProblemError::Contract(
                "select one explicit HiGHS seed".into(),
            ));
        }
        let model = self.model()?;
        verify_upload(model, p)?;
        check(
            unsafe { ffi::Highs_clearSolver(model.as_mut_ptr()) },
            "clear retained solution and basis",
        )?;
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
                .map_err(|e| ProblemError::Internal(format!("HiGHS start: {e:?}")))?;
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
        if let Some(values) = &sparse {
            let indices: Vec<_> = values
                .keys()
                .map(|id| {
                    p.contract
                        .variables
                        .iter()
                        .position(|v| v.id == *id)
                        .ok_or_else(|| ProblemError::Contract("unknown sparse coordinate".into()))
                        .and_then(index)
                })
                .collect::<Result<_, _>>()?;
            let values: Vec<_> = values.values().copied().collect();
            check(
                unsafe {
                    ffi::Highs_setSparseSolution(
                        model.as_mut_ptr(),
                        index(indices.len())?,
                        indices.as_ptr(),
                        values.as_ptr(),
                    )
                },
                "selected sparse seed",
            )?;
        }
        let ptr = model.as_mut_ptr();
        let callback_binding = CallbackBinding::with_capture(
            ptr,
            execution.clone(),
            Capture::incumbents(normalization, settings.diagnostics.cut_pool),
        )?;
        let run = if execution.stopped().is_some() {
            0
        } else {
            unsafe { ffi::Highs_run(ptr) }
        };
        let panicked = callback_binding.context.panicked.load(Ordering::Acquire);
        let cut_pool = callback_binding.finish();
        drop(callback_binding);
        self.cut_pool = cut_pool;
        let reused = self.solves > 0;
        self.solves += 1;
        let code = unsafe { ffi::Highs_getModelStatus(ptr) };
        let mut report =
            SolveReport::new(Backend::Highs, &p.contract, termination(code), &execution);
        // The complete native model was read back and compared before this solve.
        report
            .metrics
            .insert("upload.equivalent".into(), Metric::Bool(true));
        report
            .metrics
            .insert("model.discrete".into(), Metric::Bool(discrete));
        report.evidence.start_submitted = warm.is_some() || sparse.is_some();
        report.evidence.reused_native_state = reused;
        report.metrics.insert(
            "start.submitted".into(),
            Metric::Bool(report.evidence.start_submitted),
        );
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
        let real = |key| -> Result<Option<f64>, ProblemError> {
            Ok(match info(ptr, key)? {
                Some(Metric::Real(v)) => Some(v),
                _ => None,
            })
        };
        let status = |key| -> Result<SolutionStatus, ProblemError> {
            Ok(match info(ptr, key)? {
                Some(Metric::Integer(v)) if v == i64::from(ffi::kHighsSolutionStatusFeasible) => {
                    SolutionStatus::Feasible
                }
                Some(Metric::Integer(v)) if v == i64::from(ffi::kHighsSolutionStatusInfeasible) => {
                    SolutionStatus::Infeasible
                }
                _ => SolutionStatus::Unavailable,
            })
        };
        let evidence = CoefficientEvidence {
            upload_equivalent: true,
            discrete,
            objective: real("objective_function_value")?,
            mip_gap: real("mip_gap")?,
            mip_dual_bound: real("mip_dual_bound")?,
            primal: status("primal_solution_status")?,
            dual: status("dual_solution_status")?,
            max_dual_infeasibility: real("max_dual_infeasibility")?,
            primal_dual_objective_error: real("primal_dual_objective_error")?,
        };
        report.evidence.coefficient = Some(evidence);
        report.provenance.insert(
            "native".into(),
            unsafe { std::ffi::CStr::from_ptr(ffi::Highs_version()) }
                .to_string_lossy()
                .into_owned(),
        );
        report.provenance.insert(
            "interrupt".into(),
            "simplex/IPM/MIP callbacks; QP and PDLP native time limit only".into(),
        );
        report.provenance.insert("duals".into(),"native authored-sense row multipliers and reduced costs; no split bound-dual fabrication".into());
        let primal = evidence.primal != SolutionStatus::Unavailable;
        let dual = evidence.dual != SolutionStatus::Unavailable;
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
                let objective = p.objective_at(&x);
                report.candidate = Some(Candidate {
                    kind: CandidateKind::FinalIterate,
                    primal: x.clone(),
                    objective: Some(objective),
                    row_dual: dual.then(|| rd.clone()),
                    bound_dual: None,
                    reduced_costs: dual.then(|| cd.clone()),
                    slacks: None,
                    commitment: None,
                });
                let quality = p.quality(&x, tolerances)?;
                if !quality.feasible() {
                    report.termination.assurance = Assurance::None
                }
                report.quality = Some(quality);
                // HiGHS reports `basis_validity` only after simplex. Its active-set QP
                // solver writes a valid basis with every solution it returns
                // (`quass2highs`, 1.15), and a QP hot start needs that basis as well as
                // the primal (`computeStartingPointHighs`).
                let basis = if quadratic
                    || matches!(
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
                    origin: None,
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
        return Err(ProblemError::Internal(
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
            return Err(ProblemError::Internal(
                "native option name allocation".into(),
            ));
        }
        let key = unsafe { std::ffi::CStr::from_ptr(name.0) }
            .to_str()
            .map_err(|_| ProblemError::Internal("native option name UTF8".into()))?
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
            _ => return Err(ProblemError::Internal("unknown native option type".into())),
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
/// The regularization HiGHS's active-set QP solver adds to the Hessian diagonal. It changes
/// the objective by `δ/2·‖x‖²` at the solution, so it is bounded by the absolute gap budget
/// over the variables' bounding box (unbounded coordinates at unit scale, the normalized
/// scale) and never exceeds the native default.
fn qp_regularization(p: &CoefficientProblem, accuracy: &ResolvedAccuracy) -> f64 {
    let radius = p
        .contract
        .variables
        .iter()
        .map(|v| {
            let extent = v.lower.abs().max(v.upper.abs());
            if extent.is_finite() {
                extent * extent
            } else {
                1.0
            }
        })
        .sum::<f64>()
        .max(1.0);
    // `kHessianRegularizationValue` in HiGHS 1.15.
    const NATIVE_DEFAULT: f64 = 1e-7;
    (2.0 * accuracy.gap_absolute / radius).min(NATIVE_DEFAULT)
}
/// What the callback captures besides progress events.
struct Capture {
    /// Native column count; incumbents of another length are not reported.
    columns: usize,
    /// The native-to-original scale of each column and of the objective: incumbents are
    /// reported in original coordinates and units (the model normalization).
    scales: Vec<f64>,
    objective: f64,
    /// When an incumbent's solution is captured.
    throttle: CaptureThrottle,
    /// Capture the root cut pool (callback kind 7).
    cut_pool: bool,
}
impl Capture {
    /// No incumbents and no cut pool: the callbacks of a diagnostic solve.
    fn none() -> Self {
        Self {
            columns: 0,
            scales: Vec::new(),
            objective: 1.0,
            throttle: CaptureThrottle::new(),
            cut_pool: false,
        }
    }
    /// The incumbents of a solve over a model normalized by `normalization`.
    fn incumbents(normalization: &pse_math::normalization::Normalization, cut_pool: bool) -> Self {
        Self {
            columns: normalization.variables.len(),
            scales: normalization.variables.clone(),
            objective: normalization.objective,
            throttle: CaptureThrottle::new(),
            cut_pool,
        }
    }
    /// A native objective value in original units; absent when not finite.
    fn original(&self, value: f64) -> Option<f64> {
        let value = value * self.objective;
        value.is_finite().then_some(value)
    }
    /// A native solution in original coordinates; absent when any value is not finite.
    fn primal(&self, native: &[f64]) -> Option<Vec<f64>> {
        let primal: Vec<f64> = native
            .iter()
            .zip(&self.scales)
            .map(|(v, s)| v * s)
            .collect();
        primal.iter().all(|v| v.is_finite()).then_some(primal)
    }
}
struct CallbackBinding {
    ptr: *mut c_void,
    context: Box<Callback>,
}
impl CallbackBinding {
    fn new(ptr: *mut c_void, execution: Execution) -> Result<Self, ProblemError> {
        Self::with_capture(ptr, execution, Capture::none())
    }
    fn with_capture(
        ptr: *mut c_void,
        execution: Execution,
        capture: Capture,
    ) -> Result<Self, ProblemError> {
        let cut_pool = capture.cut_pool;
        let mut binding = Self {
            ptr,
            context: Box::new(Callback {
                execution,
                panicked: AtomicBool::new(false),
                capture,
                deferred: std::sync::Mutex::new(None),
                cut_pool: std::sync::Mutex::new(None),
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
        if cut_pool {
            check(
                unsafe { ffi::Highs_startCallback(ptr, ffi::kHighsCallbackMipGetCutPool) },
                "cut pool callback",
            )?;
        }
        Ok(binding)
    }
    /// Report the incumbent whose solution capture the throttle still defers, then return
    /// the captured cut pool. Called once the run returned.
    fn finish(&self) -> Option<diagnostics::CutPool> {
        let c = &self.context;
        if c.capture.throttle.outstanding()
            && let Some(incumbent) = c.take_deferred()
        {
            c.report(incumbent);
        }
        c.cut_pool
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
    }
}
impl Drop for CallbackBinding {
    fn drop(&mut self) {
        unsafe {
            // A retained model must not keep kind 7 active for a later solve; HiGHS stops a
            // kind only while a user callback is still set.
            if self.context.capture.cut_pool {
                ffi::Highs_stopCallback(self.ptr, ffi::kHighsCallbackMipGetCutPool);
            }
            ffi::Highs_setCallback(self.ptr, None, std::ptr::null_mut());
        }
    }
}
struct Callback {
    execution: Execution,
    panicked: AtomicBool,
    capture: Capture,
    /// The latest incumbent whose solution the throttle deferred, with that solution in
    /// native coordinates.
    deferred: std::sync::Mutex<Option<(IncumbentEvent, Vec<f64>)>>,
    cut_pool: std::sync::Mutex<Option<diagnostics::CutPool>>,
}
impl Callback {
    /// An improving solution (kind 4) is a new incumbent: a typed event in original units
    /// and coordinates, its solution captured when the throttle admits it and deferred
    /// otherwise. A merely feasible solution (kind 3) is no incumbent; HiGHS issues it
    /// before kind 4 for every improving one.
    fn incumbent(&self, out: &ffi::HighsCallbackDataOut) {
        let size = usize::try_from(out.mip_solution_size).unwrap_or(0);
        if out.mip_solution.is_null() || size != self.capture.columns || size == 0 {
            return;
        }
        let Some(objective) = self.capture.original(out.objective_function_value) else {
            return;
        };
        let native = unsafe { std::slice::from_raw_parts(out.mip_solution, size) };
        let mut incumbent = IncumbentEvent {
            objective,
            dual_bound: self.capture.original(out.mip_dual_bound),
            gap: out.mip_gap.is_finite().then_some(out.mip_gap),
            nodes: out.mip_node_count,
            seconds: out.running_time,
            primal: None,
        };
        let mut deferred = self
            .deferred
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if self.capture.throttle.admit() {
            *deferred = None;
            incumbent.primal = self.capture.primal(native);
        } else {
            *deferred = Some((incumbent.clone(), native.to_vec()));
        }
        drop(deferred);
        self.report(incumbent);
    }
    /// A deferred incumbent whose capture is due, observed with the search's current
    /// bounds.
    fn capture_deferred(&self, out: &ffi::HighsCallbackDataOut) {
        if let Some(mut incumbent) = self.take_deferred() {
            incumbent.dual_bound = self.capture.original(out.mip_dual_bound);
            incumbent.gap = out.mip_gap.is_finite().then_some(out.mip_gap);
            incumbent.nodes = out.mip_node_count;
            incumbent.seconds = out.running_time;
            self.report(incumbent);
        }
    }
    /// The deferred incumbent with its solution in original coordinates.
    fn take_deferred(&self) -> Option<IncumbentEvent> {
        let (mut incumbent, native) = self
            .deferred
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()?;
        incumbent.primal = self.capture.primal(&native);
        Some(incumbent)
    }
    fn report(&self, incumbent: IncumbentEvent) {
        self.execution.progress.push(Event {
            phase: "highs.incumbent".into(),
            elapsed: self.execution.started.elapsed(),
            values: BTreeMap::new(),
            incumbent: Some(incumbent),
        });
    }
    /// Record the root cut pool (kind 7).
    fn capture_cut_pool(&self, kind: i32, out: &ffi::HighsCallbackDataOut) {
        if kind == ffi::kHighsCallbackMipGetCutPool && self.capture.cut_pool {
            *self
                .cut_pool
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) =
                diagnostics::CutPool::from_callback(out);
        }
    }
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
            if kind == ffi::kHighsCallbackMipImprovingSolution {
                // The incumbent's objective and bounds travel with it, typed.
                c.incumbent(out);
                return;
            }
            let mut values = BTreeMap::from([
                ("kind".into(), Metric::Integer(i64::from(kind))),
                ("seconds".into(), Metric::Real(out.running_time)),
            ]);
            if matches!(kind, 3 | 5 | 6) {
                let bound = |v: f64| {
                    c.capture.original(v).map_or(
                        Metric::Unavailable(UnavailableReason::NotApplicable),
                        Metric::Real,
                    )
                };
                values.insert("primal_bound".into(), bound(out.mip_primal_bound));
                values.insert("dual_bound".into(), bound(out.mip_dual_bound));
                values.insert(
                    "gap".into(),
                    if out.mip_gap.is_finite() {
                        Metric::Real(out.mip_gap)
                    } else {
                        Metric::Unavailable(UnavailableReason::NotApplicable)
                    },
                );
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
                incumbent: None,
            });
            if matches!(kind, 5 | 6) && c.capture.throttle.deferred() {
                c.capture_deferred(out);
            }
            c.capture_cut_pool(kind, out);
        }
    }));
    if result.is_err() {
        c.panicked.store(true, Ordering::Release)
    }
    // HiGHS acts on `user_interrupt` only for the interrupt kinds and asserts that it
    // stays unset for every other kind, so it is written for those kinds alone.
    if interruptible(kind)
        && let Some(input) = unsafe { input.as_mut() }
    {
        input.user_interrupt =
            i32::from(c.execution.stopped().is_some() || c.panicked.load(Ordering::Acquire));
    }
}
/// Callback kinds whose `user_interrupt` HiGHS acts on: simplex, IPM and MIP interrupts.
fn interruptible(kind: i32) -> bool {
    matches!(
        kind,
        ffi::kHighsCallbackSimplexInterrupt
            | ffi::kHighsCallbackIpmInterrupt
            | ffi::kHighsCallbackMipInterrupt
    )
}
/// Preserve every pinned native model status under its C API name. Every declared
/// status has an explicit arm; only an undeclared integer from a different native
/// build reaches the final arm, and it is never given a confident category.
pub fn termination(code: i32) -> NativeTermination {
    let (name, category) = match code {
        ffi::kHighsModelStatusNotset => ("kHighsModelStatusNotset", Termination::Inconclusive),
        ffi::kHighsModelStatusLoadError => ("kHighsModelStatusLoadError", Termination::Invalid),
        ffi::kHighsModelStatusModelError => ("kHighsModelStatusModelError", Termination::Invalid),
        ffi::kHighsModelStatusPresolveError => {
            ("kHighsModelStatusPresolveError", Termination::Numerical)
        }
        ffi::kHighsModelStatusSolveError => ("kHighsModelStatusSolveError", Termination::Numerical),
        ffi::kHighsModelStatusPostsolveError => {
            ("kHighsModelStatusPostsolveError", Termination::Numerical)
        }
        ffi::kHighsModelStatusModelEmpty => ("kHighsModelStatusModelEmpty", Termination::Invalid),
        ffi::kHighsModelStatusOptimal => ("kHighsModelStatusOptimal", Termination::Success),
        ffi::kHighsModelStatusInfeasible => {
            ("kHighsModelStatusInfeasible", Termination::Infeasible)
        }
        ffi::kHighsModelStatusUnboundedOrInfeasible => (
            "kHighsModelStatusUnboundedOrInfeasible",
            Termination::InfeasibleOrUnbounded,
        ),
        ffi::kHighsModelStatusUnbounded => ("kHighsModelStatusUnbounded", Termination::Unbounded),
        ffi::kHighsModelStatusObjectiveBound => (
            "kHighsModelStatusObjectiveBound",
            Termination::ObjectiveLimit,
        ),
        ffi::kHighsModelStatusObjectiveTarget => (
            "kHighsModelStatusObjectiveTarget",
            Termination::ObjectiveLimit,
        ),
        ffi::kHighsModelStatusTimeLimit => ("kHighsModelStatusTimeLimit", Termination::TimeLimit),
        ffi::kHighsModelStatusIterationLimit => (
            "kHighsModelStatusIterationLimit",
            Termination::IterationLimit,
        ),
        ffi::kHighsModelStatusUnknown => ("kHighsModelStatusUnknown", Termination::Inconclusive),
        ffi::kHighsModelStatusSolutionLimit => {
            ("kHighsModelStatusSolutionLimit", Termination::SolutionLimit)
        }
        ffi::kHighsModelStatusInterrupt => ("kHighsModelStatusInterrupt", Termination::Cancelled),
        ffi::MODEL_STATUS_REACHED_MEMORY_LIMIT => (
            "kHighsModelStatusMemoryLimit",
            Termination::ResourceExhausted,
        ),
        _ => ("kHighsModelStatusUndeclared", Termination::Inconclusive),
    };
    let name = name.to_owned();
    let assurance = Assurance::None;
    NativeTermination {
        code: i64::from(code),
        name,
        message: None,
        category,
        assurance,
    }
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
            domains: vec![ModelingVariableDomain::Binary],
            assumptions: crate::solver_tests::stamp(Backend::Highs).data,
            constraints: o.matrix,
            hessian: None,
            bounds: vec![(0.0, 3.0)],
        }
    }
    /// minimize (x-1)^2 + (y-2)^2 subject to x + y <= 1: optimum (0, 1).
    fn quadratic() -> (CoefficientProblem, crate::GramCertificate) {
        use faer::sparse::{SparseColMat, Triplet};
        let mut contract = crate::solver_tests::contract();
        contract.variables = [1, 2]
            .map(|i| crate::Variable {
                id: crate::solver_tests::id(i),
                lower: -10.,
                upper: 10.,
            })
            .into();
        contract.rows = vec![crate::solver_tests::id(3)];
        let hessian = SparseColMat::try_new_from_triplets(
            2,
            2,
            &[Triplet::new(0, 0, 2.), Triplet::new(1, 1, 2.)],
        )
        .unwrap();
        let certificate =
            crate::GramCertificate::new(&hessian, 1.0, &faer::Mat::identity(2, 2), &[2., 2.], 64)
                .unwrap();
        let problem = CoefficientProblem {
            contract,
            objective: vec![-2., -4.],
            objective_constant: 5.,
            sense: ObjectiveSense::Minimize,
            domains: vec![ModelingVariableDomain::Continuous; 2],
            assumptions: crate::solver_tests::stamp(Backend::Highs).data,
            constraints: SparseColMat::try_new_from_triplets(
                1,
                2,
                &[Triplet::new(0, 0, 1.), Triplet::new(0, 1, 1.)],
            )
            .unwrap(),
            hessian: Some(hessian),
            bounds: vec![(f64::NEG_INFINITY, 1.)],
        };
        (problem, certificate)
    }
    fn solve_quadratic(
        session: &mut Session,
        p: &CoefficientProblem,
        method: Method,
        options: Options,
        warm: Option<&WarmStart>,
    ) -> Result<SolveReport, ProblemError> {
        let controls = Controls {
            options,
            ..Controls::default()
        };
        session.solve(
            p,
            &pse_math::normalization::Normalization::identity(
                p.contract.variables.len(),
                p.contract.rows.len(),
            ),
            &controls,
            &ResolvedAccuracy::nominal(),
            &Settings {
                method,
                ..Settings::default()
            },
            Execution::new(Default::default(), &controls),
            &Tolerances {
                variables: vec![1e-8; 2],
                rows: vec![1e-8],
                integrality: 1e-8,
            },
            warm,
        )
    }
    #[test]
    fn highs_qp_explicit_method_refused() {
        let (p, certificate) = quadratic();
        let stamp = crate::solver_tests::stamp(Backend::Highs);
        let mut session = Session::new(&p, Some(&certificate), stamp).unwrap();
        for method in [Method::Simplex, Method::Ipm, Method::Pdlp] {
            let error =
                solve_quadratic(&mut session, &p, method, Options::new(), None).unwrap_err();
            assert!(matches!(error, ProblemError::Unsupported(_)), "{error:?}");
        }
        let report =
            solve_quadratic(&mut session, &p, Method::Choose, Options::new(), None).unwrap();
        assert_eq!(report.termination.category, Termination::Success);
        assert_eq!(report.options["solver"], OptionValue::Text("choose".into()));
    }
    #[test]
    fn highs_qp_hot_start_consumed() {
        let (p, certificate) = quadratic();
        let stamp = crate::solver_tests::stamp(Backend::Highs);
        let mut session = Session::new(&p, Some(&certificate), stamp).unwrap();
        let iterations = |r: &SolveReport| match r.metrics.get("qp_iteration_count") {
            Some(Metric::Integer(k)) => *k,
            other => panic!("{other:?}"),
        };
        let cold = solve_quadratic(&mut session, &p, Method::Choose, Options::new(), None).unwrap();
        assert_eq!(cold.options["qp_allow_hot_start"], OptionValue::Bool(true));
        let primal = &cold.candidate.as_ref().unwrap().primal;
        assert!((primal[0] - 0.).abs() < 1e-7 && (primal[1] - 1.).abs() < 1e-7);
        let seed = cold.warm_start.clone().unwrap();
        assert!(matches!(
            &seed.payload,
            WarmPayload::Highs { basis: Some(_), .. }
        ));
        let hot = solve_quadratic(
            &mut session,
            &p,
            Method::Choose,
            Options::new(),
            Some(&seed),
        )
        .unwrap();
        assert!(hot.evidence.start_submitted);
        assert_eq!(hot.termination.category, Termination::Success);
        // With hot start disabled the same seed is ignored and the cold path reruns.
        let off = Options::from([("qp_allow_hot_start".into(), OptionValue::Bool(false))]);
        let ignored = solve_quadratic(&mut session, &p, Method::Choose, off, Some(&seed)).unwrap();
        assert_eq!(
            ignored.options["qp_allow_hot_start"],
            OptionValue::Bool(false)
        );
        assert!(iterations(&cold) > 0, "{:?}", cold.metrics);
        assert_eq!(iterations(&ignored), iterations(&cold));
        assert!(
            iterations(&hot) < iterations(&cold),
            "hot {} cold {}",
            iterations(&hot),
            iterations(&cold)
        );
    }
    #[test]
    fn native_iis_discrete_scope_is_an_explicit_continuous_relaxation() {
        use std::sync::{Arc, atomic::AtomicBool};
        let mut p = problem();
        p.contract.variables[0].lower = -10.;
        p.contract.variables[0].upper = 10.;
        p.constraints = faer::sparse::SparseColMat::try_new_from_triplets(
            1,
            1,
            &[faer::sparse::Triplet::new(0, 0, 1.)],
        )
        .unwrap();
        let controls = Controls::default();
        for (domain, bounds, row, conflict) in [
            (
                ModelingVariableDomain::Binary,
                (-10., 10.),
                (2., f64::INFINITY),
                true,
            ),
            (
                ModelingVariableDomain::Integer,
                (-10., 10.),
                (0.5, 0.5),
                false,
            ),
            (
                ModelingVariableDomain::Semicontinuous,
                (1., 10.),
                (0., 0.),
                false,
            ),
        ] {
            p.domains[0] = domain;
            p.contract.variables[0].lower = bounds.0;
            p.contract.variables[0].upper = bounds.1;
            p.bounds[0] = row;
            let mut session =
                Session::new(&p, None, crate::solver_tests::stamp(Backend::Highs)).unwrap();
            let evidence = session
                .diagnose(
                    &p,
                    &diagnostics::Request {
                        iis: true,
                        ..Default::default()
                    },
                    &Execution::new(Arc::new(AtomicBool::new(false)), &controls),
                )
                .unwrap();
            assert_eq!(
                evidence.iis.is_some(),
                conflict,
                "{:?}",
                evidence.unavailable
            );
            if let Some(iis) = evidence.iis {
                assert!(iis.relaxation_only);
                assert_eq!(iis.columns[0].0, p.contract.variables[0].id);
                assert_eq!(iis.rows[0].0, p.contract.rows[0]);
            }
            assert_eq!(p.domains[0], domain);
        }
    }
    #[test]
    fn native_relaxation_preserves_restored_status_and_physical_penalty() {
        use std::sync::{Arc, atomic::AtomicBool};
        let mut p = problem();
        p.domains[0] = ModelingVariableDomain::Continuous;
        p.contract.variables[0].lower = 0.;
        p.contract.variables[0].upper = 1.;
        p.bounds[0] = (2., f64::INFINITY);
        p.constraints = faer::sparse::SparseColMat::try_new_from_triplets(
            1,
            1,
            &[faer::sparse::Triplet::new(0, 0, 1.)],
        )
        .unwrap();
        let n = pse_math::normalization::Normalization {
            variables: vec![4.],
            rows: vec![2.],
            objective: 5.,
        };
        let (normalized, _) = crate::transport::coefficients(&p, &n, None).unwrap();
        let request = crate::transport::diagnostic_request(
            &diagnostics::Request {
                relaxation: Some(diagnostics::Penalties {
                    global: [-1., -1., 1.]
                        .map(|v| pse_model::scalars::FiniteBound::try_new(v).unwrap()),
                    lower: None,
                    upper: None,
                    rows: None,
                }),
                ..Default::default()
            },
            &n,
        )
        .unwrap();
        let mut session = Session::new(
            &normalized,
            None,
            crate::solver_tests::stamp(Backend::Highs),
        )
        .unwrap();
        let controls = Controls::default();
        let mut evidence = session
            .diagnose(
                &normalized,
                &request,
                &Execution::new(Arc::new(AtomicBool::new(false)), &controls),
            )
            .unwrap();
        crate::transport::recover_diagnostics(&mut evidence, &n, &[0.], &normalized.contract)
            .unwrap();
        let relaxed = evidence.relaxation.unwrap();
        assert_eq!(relaxed.operation_status, 0);
        assert_eq!(relaxed.restored_status.name, "kHighsModelStatusNotset");
        assert_eq!(relaxed.restored_status.category, Termination::Inconclusive);
        assert!((relaxed.penalty.unwrap() - 1.).abs() < 1e-8, "{relaxed:?}");
        assert!((relaxed.primal.unwrap()[0] - 1.).abs() < 1e-8);
        assert_eq!(p.contract.variables[0].upper, 1.);
        assert_eq!(p.bounds[0].0, 2.);
    }
    #[test]
    fn highs_callback_interrupt_kinds() {
        let callback_data = Callback {
            execution: Execution::new(
                std::sync::Arc::new(AtomicBool::new(true)),
                &Controls::default(),
            ),
            panicked: AtomicBool::new(false),
            capture: Capture::none(),
            deferred: std::sync::Mutex::new(None),
            cut_pool: std::sync::Mutex::new(None),
        };
        let data = (&raw const callback_data).cast_mut().cast::<c_void>();
        for kind in ffi::kHighsCallbackLogging..=ffi::kHighsCallbackCallbackMipUserSolution {
            // SAFETY: plain C data with no invariants; zero is its initial native state.
            let mut input: ffi::HighsCallbackDataIn = unsafe { std::mem::zeroed() };
            // SAFETY: a null output is accepted; input and data outlive the call.
            unsafe {
                callback(
                    kind,
                    std::ptr::null(),
                    std::ptr::null(),
                    &raw mut input,
                    data,
                )
            };
            let interrupt = matches!(kind, 1 | 2 | 6);
            assert_eq!(interruptible(kind), interrupt, "kind {kind}");
            assert_eq!(input.user_interrupt, i32::from(interrupt), "kind {kind}");
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
    fn defective_native_upload_is_refused_even_if_a_candidate_could_be_feasible() {
        let p = problem();
        let stamp = Compatibility {
            layout: p.contract.identity,
            profile: p.contract.identity,
            data: p.contract.identity,
            backend: Backend::Highs,
        };
        let mut session = Session::new(&p, None, stamp).unwrap();
        check(
            unsafe {
                ffi::Highs_changeColCost(
                    session.model().unwrap().as_mut_ptr(),
                    0,
                    p.objective[0] + 1.0,
                )
            },
            "intentional defective upload",
        )
        .unwrap();
        assert!(
            verify_upload(session.model().unwrap(), &p)
                .unwrap_err()
                .to_string()
                .contains("differs")
        );
    }
    #[test]
    fn native_upload_warnings_are_refused_and_semi_quality_keeps_zero_branch() {
        let mut p = problem();
        p.constraints.val_mut()[0] = 1e-12;
        assert!(Session::new(&p, None, crate::solver_tests::stamp(Backend::Highs)).is_err());
        p.constraints.val_mut()[0] = 1.0;
        p.domains[0] = ModelingVariableDomain::Semicontinuous;
        p.contract.variables[0].lower = 2.0;
        p.contract.variables[0].upper = 3.0;
        let t = Tolerances {
            variables: vec![1e-8],
            rows: vec![1e-8],
            integrality: 1e-8,
        };
        assert!(p.quality(&[0.0], &t).unwrap().feasible());
        assert!(!p.quality(&[1.0], &t).unwrap().feasible());
    }
}
