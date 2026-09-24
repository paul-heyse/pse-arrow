// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(
    unsafe_code,
    reason = "direct pinned Ipopt C boundary with checked buffers and contained callbacks"
)]
//! Direct Ipopt 3.14 C adapter; native NLP state stays on its owning worker.
use crate::{
    NlpOracle, ProblemError,
    callback::CallbackState,
    quality::{self, Tolerances},
    solve::*,
};
use pse_ipopt_sys as ffi;
use pse_math::binding::ObjectiveSense;
use std::{
    ffi::{CString, c_void},
    ptr::NonNull,
};

const INFINITY: f64 = 1e19;
use crate::nlp_pattern::Pattern;
pub use crate::presolve::Scaling;
fn index(n: usize) -> Result<i32, ProblemError> {
    i32::try_from(n).map_err(|_| ProblemError::Contract("Ipopt index overflow".into()))
}
fn cstring(s: &str) -> Result<CString, ProblemError> {
    CString::new(s).map_err(|_| ProblemError::Contract("NUL in native option".into()))
}
fn native_bound(x: f64) -> Result<f64, ProblemError> {
    if x.is_nan() || x.is_finite() && x.abs() >= INFINITY {
        return Err(ProblemError::Contract(
            "finite bound reaches Ipopt infinity threshold".into(),
        ));
    }
    Ok(if x == f64::INFINITY {
        INFINITY
    } else if x == f64::NEG_INFINITY {
        -INFINITY
    } else {
        x
    })
}
struct Handle(NonNull<ffi::IpoptProblemInfo>);
impl Drop for Handle {
    fn drop(&mut self) {
        unsafe { ffi::FreeIpoptProblem(self.0.as_ptr()) }
    }
}
impl Handle {
    fn option(&self, key: &str, value: &OptionValue) -> Result<(), ProblemError> {
        let key = cstring(key)?;
        let ok = unsafe {
            match value {
                OptionValue::Real(v) => {
                    ffi::AddIpoptNumOption(self.0.as_ptr(), key.as_ptr().cast_mut(), *v)
                }
                OptionValue::Integer(v) => {
                    ffi::AddIpoptIntOption(self.0.as_ptr(), key.as_ptr().cast_mut(), *v)
                }
                OptionValue::Text(v) => {
                    let value = cstring(v)?;
                    ffi::AddIpoptStrOption(
                        self.0.as_ptr(),
                        key.as_ptr().cast_mut(),
                        value.as_ptr().cast_mut(),
                    )
                }
                OptionValue::Bool(v) => {
                    let value = cstring(if *v { "yes" } else { "no" })?;
                    ffi::AddIpoptStrOption(
                        self.0.as_ptr(),
                        key.as_ptr().cast_mut(),
                        value.as_ptr().cast_mut(),
                    )
                }
            }
        };
        if !ok {
            return Err(ProblemError::Contract(format!(
                "Ipopt rejected option {}",
                key.to_string_lossy()
            )));
        }
        Ok(())
    }
}
struct Context<'a> {
    oracle: &'a mut dyn NlpOracle,
    state: CallbackState,
    n: usize,
    m: usize,
    jac: Pattern,
    hess: Pattern,
    handle: ffi::IpoptProblem,
}
// The native API calls sequentially with this worker-local context. Native dimensions
// are checked before reading any pointer, and zero-length nullable inputs are allowed.
unsafe fn input<'a>(p: *const f64, n: usize) -> Result<&'a [f64], ProblemError> {
    if n == 0 {
        return Ok(&[]);
    }
    if p.is_null() {
        return Err(ProblemError::Contract("null callback input".into()));
    }
    Ok(unsafe { std::slice::from_raw_parts(p, n) })
}
unsafe fn publish<T: Copy>(p: *mut T, values: &[T]) -> Result<(), ProblemError> {
    if values.is_empty() {
        return Ok(());
    }
    if p.is_null() {
        return Err(ProblemError::Contract("null callback output".into()));
    }
    unsafe { std::ptr::copy_nonoverlapping(values.as_ptr(), p, values.len()) };
    Ok(())
}
unsafe fn context<'a>(p: *mut c_void) -> Option<&'a mut Context<'a>> {
    unsafe { p.cast::<Context<'a>>().as_mut() }
}
fn dimensions(c: &Context<'_>, n: i32, m: Option<i32>) -> Result<(), ProblemError> {
    if usize::try_from(n).ok() != Some(c.n)
        || m.is_some_and(|m| usize::try_from(m).ok() != Some(c.m))
    {
        Err(ProblemError::Contract(
            "callback dimensions differ from admitted NLP".into(),
        ))
    } else {
        Ok(())
    }
}
unsafe extern "C" fn objective(
    n: i32,
    x: *mut f64,
    _new: bool,
    out: *mut f64,
    data: *mut c_void,
) -> bool {
    let Some(c) = (unsafe { context(data) }) else {
        return false;
    };
    let valid = dimensions(c, n, None);
    c.state
        .evaluate("objective", || {
            valid?;
            let value = c.oracle.objective(unsafe { input(x, c.n) }?)?;
            if !value.is_finite() {
                return Err(ProblemError::Contract(
                    "nonfinite objective returned by oracle".into(),
                ));
            }
            unsafe { publish(out, &[value]) }
        })
        .is_some()
}
unsafe extern "C" fn gradient(
    n: i32,
    x: *mut f64,
    _new: bool,
    out: *mut f64,
    data: *mut c_void,
) -> bool {
    let Some(c) = (unsafe { context(data) }) else {
        return false;
    };
    let valid = dimensions(c, n, None);
    c.state
        .evaluate("gradient", || {
            valid?;
            let mut values = vec![0.0; c.n];
            c.oracle.gradient(unsafe { input(x, c.n) }?, &mut values)?;
            finite(&values)?;
            unsafe { publish(out, &values) }
        })
        .is_some()
}
unsafe extern "C" fn constraints(
    n: i32,
    x: *mut f64,
    _new: bool,
    m: i32,
    out: *mut f64,
    data: *mut c_void,
) -> bool {
    let Some(c) = (unsafe { context(data) }) else {
        return false;
    };
    let valid = dimensions(c, n, Some(m));
    c.state
        .evaluate("constraints", || {
            valid?;
            let mut values = vec![0.0; c.m];
            c.oracle
                .constraints(unsafe { input(x, c.n) }?, &mut values)?;
            finite(&values)?;
            unsafe { publish(out, &values) }
        })
        .is_some()
}
unsafe extern "C" fn jacobian(
    n: i32,
    x: *mut f64,
    _new: bool,
    m: i32,
    nnz: i32,
    rows: *mut i32,
    cols: *mut i32,
    out: *mut f64,
    data: *mut c_void,
) -> bool {
    let Some(c) = (unsafe { context(data) }) else {
        return false;
    };
    let valid = dimensions(c, n, Some(m));
    c.state
        .evaluate("jacobian", || {
            valid?;
            if usize::try_from(nnz).ok() != Some(c.jac.rows.len()) {
                return Err(ProblemError::Contract("Jacobian nnz".into()));
            }
            if out.is_null() {
                if nnz > 0 && (rows.is_null() || cols.is_null()) {
                    return Err(ProblemError::Contract(
                        "null sparse structure output".into(),
                    ));
                }
                unsafe {
                    publish(rows, &c.jac.rows)?;
                    publish(cols, &c.jac.columns)
                }
            } else {
                let mut values = vec![0.0; c.jac.rows.len()];
                c.oracle.jacobian(unsafe { input(x, c.n) }?, &mut values)?;
                finite(&values)?;
                unsafe { publish(out, &values) }
            }
        })
        .is_some()
}
unsafe extern "C" fn hessian(
    n: i32,
    x: *mut f64,
    _new: bool,
    weight: f64,
    m: i32,
    lambda: *mut f64,
    _new_lambda: bool,
    nnz: i32,
    rows: *mut i32,
    cols: *mut i32,
    out: *mut f64,
    data: *mut c_void,
) -> bool {
    let Some(c) = (unsafe { context(data) }) else {
        return false;
    };
    let valid = dimensions(c, n, Some(m));
    c.state
        .evaluate("hessian", || {
            valid?;
            if usize::try_from(nnz).ok() != Some(c.hess.rows.len()) {
                return Err(ProblemError::Contract("Hessian nnz".into()));
            }
            if out.is_null() {
                if nnz > 0 && (rows.is_null() || cols.is_null()) {
                    return Err(ProblemError::Contract(
                        "null sparse structure output".into(),
                    ));
                }
                unsafe {
                    publish(rows, &c.hess.rows)?;
                    publish(cols, &c.hess.columns)
                }
            } else {
                let mut values = vec![0.0; c.hess.rows.len()];
                c.oracle.hessian(
                    unsafe { input(x, c.n) }?,
                    weight,
                    unsafe { input(lambda, c.m) }?,
                    &mut values,
                )?;
                finite(&values)?;
                unsafe { publish(out, &values) }
            }
        })
        .is_some()
}
pub(crate) fn finite(values: &[f64]) -> Result<(), ProblemError> {
    if values.iter().any(|v| !v.is_finite()) {
        Err(ProblemError::Contract("nonfinite callback output".into()))
    } else {
        Ok(())
    }
}
unsafe extern "C" fn intermediate(
    mode: i32,
    iteration: i32,
    objective: f64,
    primal: f64,
    dual: f64,
    barrier: f64,
    step: f64,
    regularization: f64,
    alpha_dual: f64,
    alpha_primal: f64,
    trials: i32,
    data: *mut c_void,
) -> bool {
    let Some(c) = (unsafe { context(data) }) else {
        return false;
    };
    c.state
        .evaluate("intermediate", || {
            let mut values = std::collections::BTreeMap::from([
                ("iteration".into(), Metric::Integer(i64::from(iteration))),
                ("restoration".into(), Metric::Bool(mode == 1)),
                ("objective.normalized".into(), Metric::Real(objective)),
                ("primal.native".into(), Metric::Real(primal)),
                ("dual.native".into(), Metric::Real(dual)),
                ("barrier".into(), Metric::Real(barrier)),
                ("step.norm".into(), Metric::Real(step)),
                ("regularization".into(), Metric::Real(regularization)),
                ("alpha.dual".into(), Metric::Real(alpha_dual)),
                ("alpha.primal".into(), Metric::Real(alpha_primal)),
                (
                    "line_search.trials".into(),
                    Metric::Integer(i64::from(trials)),
                ),
            ]);
            let mut lagrangian = vec![0.0; c.n];
            if !c.handle.is_null()
                && unsafe {
                    ffi::GetIpoptCurrentViolations(
                        c.handle,
                        false,
                        index(c.n)?,
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        lagrangian.as_mut_ptr(),
                        index(c.m)?,
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                    )
                }
            {
                values.insert(
                    "stationarity.unscaled".into(),
                    Metric::Real(lagrangian.iter().map(|v| v.abs()).fold(0.0, f64::max)),
                );
            }
            let mut x = vec![0.0; c.n];
            if !c.handle.is_null()
                && unsafe {
                    ffi::GetIpoptCurrentIterate(
                        c.handle,
                        false,
                        index(c.n)?,
                        x.as_mut_ptr(),
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        index(c.m)?,
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                    )
                }
            {
                values.insert(
                    "iterate.unscaled.infinity_norm".into(),
                    Metric::Real(x.iter().map(|v| v.abs()).fold(0.0, f64::max)),
                );
            }
            // Push uses an independent Arc to avoid lending callback-native storage.
            Ok(values)
        })
        .map(|values| {
            c.state.execution.progress.push(Event {
                phase: "ipopt.iteration".into(),
                elapsed: c.state.execution.started.elapsed(),
                values,
            })
        })
        .is_some()
}

/// Map every pinned Ipopt return status without claiming global NLP certificates.
pub fn termination(code: i32) -> NativeTermination {
    let (name, category, assurance) = match code {
        0 => (
            "Solve_Succeeded",
            Termination::Success,
            Assurance::LocalStationary,
        ),
        1 => (
            "Solved_To_Acceptable_Level",
            Termination::Acceptable,
            Assurance::LocalStationary,
        ),
        2 => (
            "Infeasible_Problem_Detected",
            Termination::Infeasible,
            Assurance::None,
        ),
        3 => (
            "Search_Direction_Becomes_Too_Small",
            Termination::Numerical,
            Assurance::None,
        ),
        4 => (
            "Diverging_Iterates",
            Termination::Numerical,
            Assurance::None,
        ),
        5 => (
            "User_Requested_Stop",
            Termination::Cancelled,
            Assurance::None,
        ),
        6 => (
            "Feasible_Point_Found",
            Termination::FeasibleOnly,
            Assurance::Feasible,
        ),
        -1 => (
            "Maximum_Iterations_Exceeded",
            Termination::Limit,
            Assurance::None,
        ),
        -2 => (
            "Restoration_Failed",
            Termination::Numerical,
            Assurance::None,
        ),
        -3 => (
            "Error_In_Step_Computation",
            Termination::Numerical,
            Assurance::None,
        ),
        -4 => (
            "Maximum_CpuTime_Exceeded",
            Termination::TimeLimit,
            Assurance::None,
        ),
        -5 => (
            "Maximum_WallTime_Exceeded",
            Termination::TimeLimit,
            Assurance::None,
        ),
        -10 => (
            "Not_Enough_Degrees_Of_Freedom",
            Termination::Invalid,
            Assurance::None,
        ),
        -11 => (
            "Invalid_Problem_Definition",
            Termination::Invalid,
            Assurance::None,
        ),
        -12 => ("Invalid_Option", Termination::Invalid, Assurance::None),
        -13 => (
            "Invalid_Number_Detected",
            Termination::Evaluation,
            Assurance::None,
        ),
        -100 => (
            "Unrecoverable_Exception",
            Termination::Numerical,
            Assurance::None,
        ),
        -101 => (
            "NonIpopt_Exception_Thrown",
            Termination::Numerical,
            Assurance::None,
        ),
        -102 => ("Insufficient_Memory", Termination::Limit, Assurance::None),
        -199 => ("Internal_Error", Termination::Invalid, Assurance::None),
        _ => (
            "Unknown_Ipopt_Status",
            Termination::Invalid,
            Assurance::None,
        ),
    };
    NativeTermination {
        code: i64::from(code),
        name: name.into(),
        message: None,
        category,
        assurance,
    }
}

/// Worker-local Ipopt problem retained across compatible numeric right-hand sides.
#[derive(Default)]
pub struct Session {
    handle: Option<Handle>,
    signature: Option<(pse_ids::ContentHash, Pattern, Pattern, Vec<u64>)>,
}
impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IpoptSession")
            .field("initialized", &self.handle.is_some())
            .finish_non_exhaustive()
    }
}
impl Session {
    /// Create an empty worker-local session; no allocation happens until admission.
    pub fn new() -> Self {
        Self::default()
    }
    /// Run an admitted minimization oracle. Its objective is already sense-normalized;
    /// `sense` is used only to recover authored output. No normalization is repeated.
    pub fn solve(
        &mut self,
        oracle: &mut dyn NlpOracle,
        initial: &[f64],
        sense: ObjectiveSense,
        controls: &Controls,
        execution: Execution,
        tolerances: &Tolerances,
        scaling: Option<&Scaling>,
        warm: Option<&WarmStart>,
        compatibility: Compatibility,
    ) -> Result<SolveReport, ProblemError> {
        controls.validate()?;
        let exact = controls.hessian == HessianMode::Exact;
        crate::validate_nlp(
            oracle,
            if exact {
                pse_kernels::DerivativeOrder::Second
            } else {
                pse_kernels::DerivativeOrder::First
            },
        )?;
        let n = oracle.contract().variables.len();
        let m = oracle.contract().rows.len();
        tolerances.validate(n, m)?;
        if initial.len() != n {
            return Err(ProblemError::Contract("initial point dimensions".into()));
        }
        finite(initial)?;
        if controls.threads != 1 {
            return Err(ProblemError::Contract(
                "pinned sequential Ipopt/MUMPS profile requires one core".into(),
            ));
        }
        if oracle.constraint_bounds().len() != m {
            return Err(ProblemError::Contract(
                "constraint bounds dimensions".into(),
            ));
        }
        let jac = Pattern::new(oracle.jacobian_pattern(), false)?;
        let hess = if exact {
            Pattern::new(
                oracle
                    .hessian_pattern()
                    .ok_or_else(|| ProblemError::Contract("exact Hessian unavailable".into()))?,
                true,
            )?
        } else {
            Pattern {
                rows: vec![],
                columns: vec![],
            }
        };
        let mut xl: Vec<_> = oracle
            .contract()
            .variables
            .iter()
            .map(|v| native_bound(v.lower))
            .collect::<Result<_, _>>()?;
        let mut xu: Vec<_> = oracle
            .contract()
            .variables
            .iter()
            .map(|v| native_bound(v.upper))
            .collect::<Result<_, _>>()?;
        let mut gl: Vec<_> = oracle
            .constraint_bounds()
            .iter()
            .map(|v| native_bound(v.0))
            .collect::<Result<_, _>>()?;
        let mut gu: Vec<_> = oracle
            .constraint_bounds()
            .iter()
            .map(|v| native_bound(v.1))
            .collect::<Result<_, _>>()?;
        if gl.iter().zip(&gu).any(|(l, u)| l > u) {
            return Err(ProblemError::Contract("invalid row bounds".into()));
        }
        let signature = (
            compatibility.layout,
            jac.clone(),
            hess.clone(),
            xl.iter()
                .chain(&xu)
                .chain(&gl)
                .chain(&gu)
                .map(|v| v.to_bits())
                .collect(),
        );
        let reused = self.signature.as_ref() == Some(&signature)
            && self.handle.is_some()
            && controls.reuse != ReusePolicy::Fresh;
        if !reused {
            if controls.reuse == ReusePolicy::RequireReuse && self.handle.is_some() {
                return Err(ProblemError::Contract(
                    "Ipopt C problem bounds or layout changed and require rebuilding".into(),
                ));
            }
            self.handle.take();
            let created = Handle(
                NonNull::new(unsafe {
                    ffi::CreateIpoptProblem(
                        index(n)?,
                        xl.as_mut_ptr(),
                        xu.as_mut_ptr(),
                        index(m)?,
                        gl.as_mut_ptr(),
                        gu.as_mut_ptr(),
                        index(jac.rows.len())?,
                        index(hess.rows.len())?,
                        0,
                        Some(objective),
                        Some(constraints),
                        Some(gradient),
                        Some(jacobian),
                        Some(hessian),
                    )
                })
                .ok_or_else(|| ProblemError::Contract("Ipopt refused problem creation".into()))?,
            );
            self.handle = Some(created);
            self.signature = Some(signature);
        }
        let handle = self
            .handle
            .as_ref()
            .ok_or_else(|| ProblemError::Contract("lost Ipopt native owner".into()))?;
        reject_reserved(
            &controls.options,
            &[
                "option_file_name",
                "hessian_approximation",
                "gradient_approximation",
                "jacobian_approximation",
                "grad_f_constant",
                "jac_c_constant",
                "jac_d_constant",
                "hessian_constant",
                "nlp_lower_bound_inf",
                "nlp_upper_bound_inf",
                "max_iter",
                "max_wall_time",
                "max_cpu_time",
                "tol",
                "warm_start_init_point",
                "nlp_scaling_method",
                "obj_scaling_factor",
                "linear_solver",
            ],
        )?;
        let mut options = controls.options.clone();
        options.extend([
            ("option_file_name".into(), OptionValue::Text(String::new())),
            (
                "max_iter".into(),
                OptionValue::Integer(controls.iterations as i32),
            ),
            (
                "max_wall_time".into(),
                OptionValue::Real(controls.time_limit.as_secs_f64()),
            ),
            ("tol".into(), OptionValue::Real(controls.tolerance)),
            (
                "hessian_approximation".into(),
                OptionValue::Text(if exact { "exact" } else { "limited-memory" }.into()),
            ),
            ("nlp_lower_bound_inf".into(), OptionValue::Real(-INFINITY)),
            ("nlp_upper_bound_inf".into(), OptionValue::Real(INFINITY)),
            ("linear_solver".into(), OptionValue::Text("mumps".into())),
            (
                "nlp_scaling_method".into(),
                OptionValue::Text(
                    if scaling.is_some() {
                        "user-scaling"
                    } else {
                        "none"
                    }
                    .into(),
                ),
            ),
        ]);
        let facts = oracle.derivative_facts();
        for (k, v) in [
            ("grad_f_constant", facts.gradient_constant),
            ("jac_c_constant", facts.jacobian_constant),
            ("jac_d_constant", facts.jacobian_constant),
            ("hessian_constant", exact && facts.hessian_constant),
        ] {
            options.insert(k.into(), OptionValue::Bool(v));
        }
        options
            .entry("print_level".into())
            .or_insert(OptionValue::Integer(0));
        options.insert("warm_start_init_point".into(), OptionValue::Bool(false));
        let mut x = initial.to_vec();
        let mut lower = vec![0.0; n];
        let mut upper = vec![0.0; n];
        let mut rows = vec![0.0; m];
        if let Some(warm) = warm {
            warm.validate(&compatibility)?;
            let WarmPayload::Nlp {
                primal,
                bounds,
                rows: row_seed,
            } = &warm.payload
            else {
                return Err(ProblemError::Contract("Ipopt warm payload class".into()));
            };
            if primal.len() != n {
                return Err(ProblemError::Contract("warm primal dimensions".into()));
            }
            finite(primal)?;
            x.clone_from(primal);
            if let (Some((l, u)), Some(r)) = (bounds, row_seed) {
                if l.len() != n
                    || u.len() != n
                    || r.len() != m
                    || l.iter().chain(u).any(|v| *v < 0.0)
                {
                    return Err(ProblemError::Contract("warm dual dimensions/sign".into()));
                }
                finite(l)?;
                finite(u)?;
                finite(r)?;
                lower.clone_from(l);
                upper.clone_from(u);
                rows.clone_from(r);
                options.insert("warm_start_init_point".into(), OptionValue::Bool(true));
            } else if bounds.is_some() || row_seed.is_some() {
                return Err(ProblemError::Contract("partial NLP dual seed".into()));
            }
        }
        for (key, value) in &options {
            handle.option(key, value)?
        }
        if let Some(s) = scaling {
            if s.variables.len() != n
                || s.constraints.len() != m
                || std::iter::once(&s.objective)
                    .chain(&s.variables)
                    .chain(&s.constraints)
                    .any(|v| !v.is_finite() || *v <= 0.0)
            {
                return Err(ProblemError::Contract(
                    "positive native scaling dimensions/values".into(),
                ));
            }
            let mut variables = s.variables.clone();
            let mut rows = s.constraints.clone();
            if !unsafe {
                ffi::SetIpoptProblemScaling(
                    handle.0.as_ptr(),
                    s.objective,
                    variables.as_mut_ptr(),
                    rows.as_mut_ptr(),
                )
            } {
                return Err(ProblemError::Contract("Ipopt rejected scaling".into()));
            }
        }
        if !unsafe { ffi::SetIntermediateCallback(handle.0.as_ptr(), Some(intermediate)) } {
            return Err(ProblemError::Contract(
                "Ipopt intermediate callback registration".into(),
            ));
        }
        let mut context = Context {
            oracle,
            state: CallbackState::new(execution),
            n,
            m,
            jac,
            hess,
            handle: handle.0.as_ptr(),
        };
        let mut g = vec![f64::NAN; m];
        let mut objective = f64::NAN;
        let code = unsafe {
            ffi::IpoptSolve(
                handle.0.as_ptr(),
                x.as_mut_ptr(),
                g.as_mut_ptr(),
                &mut objective,
                rows.as_mut_ptr(),
                lower.as_mut_ptr(),
                upper.as_mut_ptr(),
                (&raw mut context).cast(),
            )
        };
        let mut report = SolveReport::new(
            Backend::Ipopt,
            context.oracle.contract(),
            termination(code),
            &context.state.execution,
        );
        report.options = options;
        report
            .metrics
            .insert("reuse.native_model".into(), Metric::Bool(reused));
        report.provenance.insert(
            "native".into(),
            "Ipopt 3.14.20; MUMPS 5.9.1; sequential LP64".into(),
        );
        report.provenance.insert(
            "duals".into(),
            "minimization L=f+lambda*g-zL*x+zU*x; authored objective recovered once".into(),
        );
        report.metrics.insert(
            "execution.seconds".into(),
            Metric::Real(context.state.execution.started.elapsed().as_secs_f64()),
        );
        context.state.finish(&mut report);
        if x.iter().all(|v| v.is_finite()) && objective.is_finite() {
            let duals = rows
                .iter()
                .chain(&lower)
                .chain(&upper)
                .all(|v| v.is_finite());
            report.candidate = Some(Candidate {
                primal: x.clone(),
                objective: Some(objective * sense.sign()),
                row_dual: duals.then(|| rows.clone()),
                bound_dual: duals.then(|| (lower.clone(), upper.clone())),
                reduced_costs: None,
                slacks: None,
            });
            match quality::contained(|| quality::nlp(context.oracle, &x, tolerances)) {
                Ok(q) => {
                    if !q.feasible() {
                        report.termination.assurance = Assurance::None
                    }
                    report.quality = Some(q)
                }
                Err(e) => {
                    report.validation_error = Some(e.to_string());
                    report.termination.assurance = Assurance::None
                }
            }
            report.warm_start = Some(WarmStart {
                compatibility,
                payload: WarmPayload::Nlp {
                    primal: x,
                    bounds: duals.then_some((lower, upper)),
                    rows: duals.then_some(rows),
                },
            });
        } else {
            report.termination.assurance = Assurance::None
        }
        // Ipopt calls user-data only inside IpoptSolve; no borrowed callback storage is retained.
        drop(context);
        if matches!(
            report.termination.category,
            Termination::Panic
                | Termination::Evaluation
                | Termination::Invalid
                | Termination::Numerical
        ) {
            self.handle.take();
            self.signature.take();
        }
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn status_scope_and_finite_infinity_are_not_conflated() {
        assert_eq!(termination(2).assurance, Assurance::None);
        assert_eq!(termination(6).assurance, Assurance::Feasible);
        for code in [
            0, 1, 2, 3, 4, 5, 6, -1, -2, -3, -4, -5, -10, -11, -12, -13, -100, -101, -102, -199,
        ] {
            assert!(!termination(code).name.starts_with("Unknown"))
        }
        assert!(native_bound(INFINITY).is_err());
        assert_eq!(native_bound(f64::INFINITY).unwrap(), INFINITY);
        assert!(index(usize::MAX).is_err());
    }
    #[test]
    fn null_zero_slices_are_legal_but_nonempty_null_is_refused() {
        unsafe {
            assert!(input(std::ptr::null(), 0).unwrap().is_empty());
            assert!(input(std::ptr::null(), 1).is_err());
            assert!(publish::<f64>(std::ptr::null_mut(), &[]).is_ok());
            assert!(publish(std::ptr::null_mut(), &[1.0]).is_err());
            assert!(!objective(
                0,
                std::ptr::null_mut(),
                false,
                std::ptr::null_mut(),
                std::ptr::null_mut()
            ));
        }
    }
    fn context_for(o: &mut dyn NlpOracle) -> Context<'_> {
        Context {
            jac: Pattern::new(o.jacobian_pattern(), false).unwrap(),
            hess: Pattern::new(o.hessian_pattern().unwrap(), true).unwrap(),
            oracle: o,
            state: CallbackState::new(crate::solver_tests::execution()),
            n: 1,
            m: 1,
            handle: std::ptr::null_mut(),
        }
    }
    #[test]
    fn callbacks_publish_transactionally_and_contain_unwinds() {
        let mut o = crate::solver_tests::Polynomial::new();
        o.fail = true;
        let mut c = context_for(&mut o);
        let mut x = [2.0];
        let mut out = [73.0];
        assert!(!unsafe {
            gradient(
                1,
                x.as_mut_ptr(),
                true,
                out.as_mut_ptr(),
                (&raw mut c).cast(),
            )
        });
        assert_eq!(out, [73.0]);
        assert_eq!(
            c.state.terminal.as_ref().unwrap().0,
            Termination::Evaluation
        );
        let mut o = crate::solver_tests::Polynomial::new();
        o.panic = true;
        let mut c = context_for(&mut o);
        assert!(!unsafe {
            objective(
                1,
                x.as_mut_ptr(),
                true,
                out.as_mut_ptr(),
                (&raw mut c).cast(),
            )
        });
        assert_eq!(out, [73.0]);
        assert_eq!(c.state.terminal.as_ref().unwrap().0, Termination::Panic);
    }
    #[test]
    fn sparse_structure_is_null_trial_safe_and_all_or_nothing() {
        let mut o = crate::solver_tests::Polynomial::new();
        let mut c = context_for(&mut o);
        let mut rows = [77];
        let mut cols = [77];
        assert!(unsafe {
            jacobian(
                1,
                std::ptr::null_mut(),
                false,
                1,
                1,
                rows.as_mut_ptr(),
                cols.as_mut_ptr(),
                std::ptr::null_mut(),
                (&raw mut c).cast(),
            )
        });
        assert_eq!((rows, cols), ([0], [0]));
        rows = [77];
        assert!(!unsafe {
            jacobian(
                1,
                std::ptr::null_mut(),
                false,
                1,
                1,
                rows.as_mut_ptr(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                (&raw mut c).cast(),
            )
        });
        assert_eq!(rows, [77]);
    }
    #[test]
    fn exact_hessian_uses_native_objective_and_row_weights_once() {
        let mut o = crate::solver_tests::Polynomial::new();
        let mut c = context_for(&mut o);
        let mut x = [2.0];
        let mut lambda = [3.0];
        let mut out = [0.0];
        assert!(unsafe {
            hessian(
                1,
                x.as_mut_ptr(),
                true,
                4.0,
                1,
                lambda.as_mut_ptr(),
                true,
                1,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                out.as_mut_ptr(),
                (&raw mut c).cast(),
            )
        });
        assert_eq!(out, [44.0]);
    }
}
