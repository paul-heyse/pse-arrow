// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(
    unsafe_code,
    reason = "direct pinned Ipopt C boundary with checked buffers and contained callbacks"
)]
//! Direct Ipopt 3.14 C adapter; native NLP state stays on its owning worker.
mod runtime;
mod settings;
pub use crate::settings::ipopt::{
    Linear, MuStrategy, MumpsOrdering, PardisoMatching, PardisoOrdering, Settings, SpralOrdering,
    SpralPivot, SpralScaling,
};
use crate::{
    NlpOracle, ProblemError, ReuseRefusal,
    callback::CallbackState,
    quality::{self, Tolerances},
    solve::*,
};
use pse_ipopt_sys as ffi;
use pse_math::binding::ObjectiveSense;
pub use runtime::{Build, Runtime, build};
pub use settings::admit;
use std::{
    ffi::{CString, c_void},
    ptr::NonNull,
};

const INFINITY: f64 = 1e19;
use crate::nlp_pattern::Pattern;
fn index(n: usize) -> Result<i32, ProblemError> {
    i32::try_from(n).map_err(|_| ProblemError::Unsupported("Ipopt index overflow".into()))
}
fn cstring(s: &str) -> Result<CString, ProblemError> {
    CString::new(s).map_err(|_| ProblemError::Contract("NUL in native option".into()))
}
fn native_bound(x: f64) -> Result<f64, ProblemError> {
    if x.is_nan() || x.is_finite() && x.abs() >= INFINITY {
        return Err(ProblemError::Unsupported(
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
    /// Barrier parameter of the most recent iteration.
    barrier: Option<f64>,
}
// The native API calls sequentially with this worker-local context. Native dimensions
// are checked before reading any pointer, and zero-length nullable inputs are allowed.
unsafe fn input<'a>(p: *const f64, n: usize) -> Result<&'a [f64], ProblemError> {
    if n == 0 {
        return Ok(&[]);
    }
    if p.is_null() {
        return Err(ProblemError::Internal("null callback input".into()));
    }
    Ok(unsafe { std::slice::from_raw_parts(p, n) })
}
unsafe fn publish<T: Copy>(p: *mut T, values: &[T]) -> Result<(), ProblemError> {
    if values.is_empty() {
        return Ok(());
    }
    if p.is_null() {
        return Err(ProblemError::Internal("null callback output".into()));
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
        Err(ProblemError::Internal(
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
                return Err(ProblemError::numerical(
                    "nonfinite objective returned by oracle",
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
                return Err(ProblemError::Internal("Jacobian nnz".into()));
            }
            if out.is_null() {
                if nnz > 0 && (rows.is_null() || cols.is_null()) {
                    return Err(ProblemError::Internal(
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
                return Err(ProblemError::Internal("Hessian nnz".into()));
            }
            if out.is_null() {
                if nnz > 0 && (rows.is_null() || cols.is_null()) {
                    return Err(ProblemError::Internal(
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
        Err(ProblemError::numerical("nonfinite callback output"))
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
    if barrier.is_finite() && barrier > 0.0 {
        c.barrier = Some(barrier);
    }
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
                    "stationarity.normalized".into(),
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
                    "iterate.normalized.infinity_norm".into(),
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
                incumbent: None,
            })
        })
        .is_some()
}

/// Map every pinned Ipopt return status without claiming global NLP certificates.
pub fn termination(code: i32) -> NativeTermination {
    let (name, category, assurance) = match code {
        ffi::ApplicationReturnStatus_Solve_Succeeded => {
            ("Solve_Succeeded", Termination::Success, Assurance::None)
        }
        ffi::ApplicationReturnStatus_Solved_To_Acceptable_Level => (
            "Solved_To_Acceptable_Level",
            Termination::Acceptable,
            Assurance::None,
        ),
        ffi::ApplicationReturnStatus_Infeasible_Problem_Detected => (
            "Infeasible_Problem_Detected",
            Termination::Infeasible,
            Assurance::None,
        ),
        ffi::ApplicationReturnStatus_Search_Direction_Becomes_Too_Small => (
            "Search_Direction_Becomes_Too_Small",
            Termination::Numerical,
            Assurance::None,
        ),
        ffi::ApplicationReturnStatus_Diverging_Iterates => (
            "Diverging_Iterates",
            Termination::Numerical,
            Assurance::None,
        ),
        ffi::ApplicationReturnStatus_User_Requested_Stop => (
            "User_Requested_Stop",
            Termination::Cancelled,
            Assurance::None,
        ),
        ffi::ApplicationReturnStatus_Feasible_Point_Found => (
            "Feasible_Point_Found",
            Termination::FeasibleOnly,
            Assurance::None,
        ),
        ffi::ApplicationReturnStatus_Maximum_Iterations_Exceeded => (
            "Maximum_Iterations_Exceeded",
            Termination::IterationLimit,
            Assurance::None,
        ),
        ffi::ApplicationReturnStatus_Restoration_Failed => (
            "Restoration_Failed",
            Termination::Numerical,
            Assurance::None,
        ),
        ffi::ApplicationReturnStatus_Error_In_Step_Computation => (
            "Error_In_Step_Computation",
            Termination::Numerical,
            Assurance::None,
        ),
        ffi::ApplicationReturnStatus_Maximum_CpuTime_Exceeded => (
            "Maximum_CpuTime_Exceeded",
            Termination::TimeLimit,
            Assurance::None,
        ),
        ffi::ApplicationReturnStatus_Maximum_WallTime_Exceeded => (
            "Maximum_WallTime_Exceeded",
            Termination::TimeLimit,
            Assurance::None,
        ),
        ffi::ApplicationReturnStatus_Not_Enough_Degrees_Of_Freedom => (
            "Not_Enough_Degrees_Of_Freedom",
            Termination::Invalid,
            Assurance::None,
        ),
        ffi::ApplicationReturnStatus_Invalid_Problem_Definition => (
            "Invalid_Problem_Definition",
            Termination::Invalid,
            Assurance::None,
        ),
        ffi::ApplicationReturnStatus_Invalid_Option => {
            ("Invalid_Option", Termination::Invalid, Assurance::None)
        }
        ffi::ApplicationReturnStatus_Invalid_Number_Detected => (
            "Invalid_Number_Detected",
            Termination::Evaluation,
            Assurance::None,
        ),
        ffi::ApplicationReturnStatus_Unrecoverable_Exception => (
            "Unrecoverable_Exception",
            Termination::Numerical,
            Assurance::None,
        ),
        ffi::ApplicationReturnStatus_NonIpopt_Exception_Thrown => (
            "NonIpopt_Exception_Thrown",
            Termination::Numerical,
            Assurance::None,
        ),
        ffi::ApplicationReturnStatus_Insufficient_Memory => (
            "Insufficient_Memory",
            Termination::ResourceExhausted,
            Assurance::None,
        ),
        ffi::ApplicationReturnStatus_Internal_Error => {
            ("Internal_Error", Termination::Invalid, Assurance::None)
        }
        _ => (
            "Unknown_Ipopt_Status",
            Termination::Inconclusive,
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
    signature: Option<Signature>,
}
/// Everything a retained C problem keeps: its coordinates and profile, sparsity and bounds,
/// and the set of option keys ever set on it. The C interface cannot unset an option, but
/// every solve re-applies all of its own option values, so a step may reuse the problem when
/// its keys cover every retained key: no option of an earlier step survives into it (F02).
/// A step that adds keys, such as a primal-dual restart after a cold start, reuses it.
type Signature = (
    (
        (pse_ids::ContentHash, pse_ids::ContentHash),
        Pattern,
        Pattern,
        Vec<u64>,
    ),
    Vec<String>,
);
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
        accuracy: &ResolvedAccuracy,
        settings: &Settings,
        execution: Execution,
        tolerances: &Tolerances,
        warm: Option<&WarmStart>,
        compatibility: Compatibility,
    ) -> Result<SolveReport, ProblemError> {
        controls.validate()?;
        // Admission on the owning worker: the linked solver, its thread count and the
        // process environment it needs (ADR-0108 items 11–14).
        admit(settings, controls.threads, &Runtime::observe())?;
        if oracle.normalization().is_some() {
            return Err(ProblemError::Internal("model normalization must be transported through the shared NLP pipeline before native execution".into()));
        }
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
            return Err(ProblemError::Internal("initial point dimensions".into()));
        }
        finite(initial)?;
        if oracle.constraint_bounds().len() != m {
            return Err(ProblemError::Internal(
                "constraint bounds dimensions".into(),
            ));
        }
        let jac = Pattern::new(oracle.jacobian_pattern(), false)?;
        let hess = if exact {
            Pattern::new(
                oracle
                    .hessian_pattern()
                    .ok_or_else(|| ProblemError::Unsupported("exact Hessian unavailable".into()))?,
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
                "constr_viol_tol",
                "dual_inf_tol",
                "compl_inf_tol",
                "acceptable_tol",
                "acceptable_iter",
                "acceptable_constr_viol_tol",
                "acceptable_dual_inf_tol",
                "acceptable_compl_inf_tol",
                "bound_relax_factor",
                "honor_original_bounds",
                "warm_start_init_point",
                "nlp_scaling_method",
                "obj_scaling_factor",
            ],
        )?;
        reject_reserved(&controls.options, &settings::RESERVED)?;
        reject_reserved(&controls.options, &RESTART_OPTIONS)?;
        let mut options = controls.options.clone();
        options.extend(accuracy.nlp_options());
        options.extend(settings::options(settings));
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
            (
                "hessian_approximation".into(),
                OptionValue::Text(if exact { "exact" } else { "limited-memory" }.into()),
            ),
            ("nlp_lower_bound_inf".into(), OptionValue::Real(-INFINITY)),
            ("nlp_upper_bound_inf".into(), OptionValue::Real(INFINITY)),
            (
                "nlp_scaling_method".into(),
                OptionValue::Text(
                    if accuracy.native_scaling {
                        "gradient-based"
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
        // The seed is read before the session signature: a primal-dual restart sets its own
        // options, which must be part of the retained problem's key set (F02).
        let mut x = initial.to_vec();
        let mut lower = vec![0.0; n];
        let mut upper = vec![0.0; n];
        let mut rows = vec![0.0; m];
        let mut restart = None;
        if let Some(warm) = warm {
            warm.validate(&compatibility)?;
            let WarmPayload::Nlp {
                primal,
                bounds,
                rows: row_seed,
                barrier,
                working,
            } = &warm.payload
            else {
                return Err(ProblemError::Contract("Ipopt warm payload class".into()));
            };
            if working.is_some() {
                return Err(ProblemError::Contract(
                    "an active-set working set is not an Ipopt seed".into(),
                ));
            }
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
                // A primal-dual seed restarts under the typed profile (L-N3).
                let (restart_options, applied) = settings
                    .restart
                    .apply(*barrier, settings.mu_strategy == MuStrategy::Monotone);
                options.extend(restart_options);
                restart = Some(applied);
            } else if bounds.is_some() || row_seed.is_some() {
                return Err(ProblemError::Unsupported("partial NLP dual seed".into()));
            }
        }
        let signature: Signature = (
            (
                (compatibility.layout, compatibility.profile),
                jac.clone(),
                hess.clone(),
                xl.iter()
                    .chain(&xu)
                    .chain(&gl)
                    .chain(&gu)
                    .map(|v| v.to_bits())
                    .collect(),
            ),
            options.keys().cloned().collect(),
        );
        // The retained problem serves this step when its structure is unchanged and the step
        // sets every key ever set on it; the loop below then re-applies all of its values.
        let refusal = match &self.signature {
            Some((retained, keys)) if *retained == signature.0 => {
                let dropped: Vec<String> = keys
                    .iter()
                    .filter(|key| !options.contains_key(*key))
                    .cloned()
                    .collect();
                (!dropped.is_empty()).then_some(ReuseRefusal::DroppedOptions(dropped))
            }
            _ => Some(ReuseRefusal::Structure),
        };
        let reused =
            refusal.is_none() && self.handle.is_some() && controls.reuse != ReusePolicy::Fresh;
        if reused {
            // The retained keys are a subset of this step's, which it now sets.
            self.signature = Some(signature);
        } else {
            if controls.reuse == ReusePolicy::RequireReuse && self.handle.is_some() {
                return Err(ProblemError::Reuse {
                    backend: Backend::Ipopt,
                    refusal: refusal.unwrap_or(ReuseRefusal::Structure),
                });
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
                .ok_or_else(|| ProblemError::Internal("Ipopt refused problem creation".into()))?,
            );
            self.handle = Some(created);
            self.signature = Some(signature);
        }
        let handle = self
            .handle
            .as_ref()
            .ok_or_else(|| ProblemError::Internal("lost Ipopt native owner".into()))?;
        for (key, value) in &options {
            handle.option(key, value)?
        }
        if !unsafe { ffi::SetIntermediateCallback(handle.0.as_ptr(), Some(intermediate)) } {
            return Err(ProblemError::Internal(
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
            barrier: None,
        };
        let mut g = vec![f64::NAN; m];
        let mut objective = f64::NAN;
        let threads = runtime::Threads::enter(controls.threads)?;
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
        drop(threads);
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
        report.evidence.reused_native_state = reused;
        report.evidence.start_submitted = warm.is_some();
        report.evidence.restart = restart;
        report
            .metrics
            .insert("start.submitted".into(), Metric::Bool(warm.is_some()));
        let build = build();
        report
            .provenance
            .insert("native".into(), build.ipopt.clone());
        report.provenance.insert(
            "linear".into(),
            serde_json::to_string(&settings.linear)
                .map_err(|e| ProblemError::Internal(format!("linear settings record: {e}")))?,
        );
        report.provenance.insert("blas".into(), build.mkl.clone());
        report.provenance.insert(
            "mkl_cbwr".into(),
            runtime::cbwr_name(Runtime::observe().cbwr),
        );
        report.metrics.insert(
            "linear.threads".into(),
            Metric::Integer(i64::try_from(controls.threads).unwrap_or(i64::MAX)),
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
                kind: CandidateKind::FinalIterate,
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
                    report.record_validation_failure(e);
                    report.termination.assurance = Assurance::None
                }
            }
            report.warm_start = Some(WarmStart {
                origin: None,
                compatibility,
                payload: WarmPayload::Nlp {
                    primal: x,
                    bounds: duals.then_some((lower, upper)),
                    rows: duals.then_some(rows),
                    // The barrier value of the last iteration, as the intermediate callback
                    // observed it; it seeds the next restart's `mu_init`.
                    barrier: duals.then_some(context.barrier).flatten(),
                    working: None,
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
        assert_eq!(termination(6).assurance, Assurance::None);
        assert_eq!(termination(6).category, Termination::FeasibleOnly);
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
            barrier: None,
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
    fn solve_with(
        session: &mut Session,
        options: Options,
        settings: &Settings,
        threads: usize,
    ) -> Result<SolveReport, ProblemError> {
        let controls = Controls {
            options,
            threads,
            reuse: ReusePolicy::AllowRebuild,
            ..Controls::default()
        };
        session.solve(
            &mut crate::solver_tests::Polynomial::new(),
            &[2.0],
            ObjectiveSense::Minimize,
            &controls,
            &ResolvedAccuracy::nominal(),
            settings,
            crate::solver_tests::execution(),
            &Tolerances {
                variables: vec![1e-8],
                rows: vec![1e-8],
                integrality: 1e-8,
            },
            None,
            crate::solver_tests::stamp(Backend::Ipopt),
        )
    }
    fn run(session: &mut Session, options: Options) -> SolveReport {
        solve_with(session, options, &Settings::default(), 1).unwrap()
    }
    #[test]
    fn reused_session_does_not_inherit_options() {
        let mut session = Session::new();
        let adaptive =
            Options::from([("mu_linear_decrease_factor".into(), OptionValue::Real(0.3))]);
        let first = run(&mut session, adaptive.clone());
        assert_eq!(first.metrics["reuse.native_model"], Metric::Bool(false));
        assert_eq!(
            first.options["mu_linear_decrease_factor"],
            OptionValue::Real(0.3)
        );
        // The C problem keeps every option set on it, so a step without the option
        // cannot run on the retained problem: its option key set differs.
        let second = run(&mut session, Options::new());
        assert_eq!(second.metrics["reuse.native_model"], Metric::Bool(false));
        assert!(!second.options.contains_key("mu_linear_decrease_factor"));
        let fresh = run(&mut Session::new(), Options::new());
        assert_eq!(second.options, fresh.options);
        assert_eq!(second.termination.category, fresh.termination.category);
        assert_eq!(
            second.candidate.as_ref().map(|c| c.primal.clone()),
            fresh.candidate.as_ref().map(|c| c.primal.clone())
        );
        // The same option keys re-apply every value, so the problem is reused.
        let third = run(&mut session, Options::new());
        assert_eq!(third.metrics["reuse.native_model"], Metric::Bool(true));
        // Added keys cover every retained one (a restart after a cold start), so the
        // problem is reused; dropping them again needs a fresh problem.
        let fourth = run(&mut session, adaptive);
        assert_eq!(fourth.metrics["reuse.native_model"], Metric::Bool(true));
        assert_eq!(
            fourth.options["mu_linear_decrease_factor"],
            OptionValue::Real(0.3)
        );
        let fifth = run(&mut session, Options::new());
        assert_eq!(fifth.metrics["reuse.native_model"], Metric::Bool(false));
    }
    fn solve_under(
        session: &mut Session,
        oracle: &mut crate::solver_tests::Polynomial,
        iterations: u32,
        reuse: ReusePolicy,
        options: Options,
    ) -> Result<SolveReport, ProblemError> {
        let controls = Controls {
            options,
            iterations,
            reuse,
            ..Controls::default()
        };
        session.solve(
            oracle,
            &[2.0],
            ObjectiveSense::Minimize,
            &controls,
            &ResolvedAccuracy::nominal(),
            &Settings::default(),
            crate::solver_tests::execution(),
            &Tolerances {
                variables: vec![1e-8],
                rows: vec![1e-8],
                integrality: 1e-8,
            },
            None,
            crate::solver_tests::stamp(Backend::Ipopt),
        )
    }
    #[test]
    fn reused_problem_reapplies_changed_values() {
        let mut session = Session::new();
        let mut oracle = crate::solver_tests::Polynomial::new();
        // One iteration cannot reach x³ = 1 from x = 2.
        let first = solve_under(
            &mut session,
            &mut oracle,
            1,
            ReusePolicy::AllowRebuild,
            Options::new(),
        )
        .unwrap();
        assert_eq!(first.termination.category, Termination::IterationLimit);
        assert_eq!(first.options["max_iter"], OptionValue::Integer(1));
        // The retained problem still holds max_iter = 1; the reused step sets its own value.
        let second = solve_under(
            &mut session,
            &mut oracle,
            100,
            ReusePolicy::RequireReuse,
            Options::new(),
        )
        .unwrap();
        assert!(second.evidence.reused_native_state);
        assert_eq!(second.metrics["reuse.native_model"], Metric::Bool(true));
        assert_eq!(second.options["max_iter"], OptionValue::Integer(100));
        assert!(solved(&second), "{:?}", second.termination);
    }
    #[test]
    fn required_reuse_names_its_refusal() {
        let mut session = Session::new();
        let mut oracle = crate::solver_tests::Polynomial::new();
        let adaptive = Options::from([
            ("mu_linear_decrease_factor".into(), OptionValue::Real(0.3)),
            ("mu_superlinear_decrease_power".into(), OptionValue::Real(1.4)),
        ]);
        solve_under(
            &mut session,
            &mut oracle,
            100,
            ReusePolicy::AllowRebuild,
            adaptive.clone(),
        )
        .unwrap();
        // The step sets neither retained option key, which the C problem cannot unset.
        let error = solve_under(
            &mut session,
            &mut oracle,
            100,
            ReusePolicy::RequireReuse,
            Options::new(),
        )
        .unwrap_err();
        assert!(
            matches!(
                &error,
                ProblemError::Reuse {
                    backend: Backend::Ipopt,
                    refusal: ReuseRefusal::DroppedOptions(keys),
                } if keys == &["mu_linear_decrease_factor", "mu_superlinear_decrease_power"]
            ),
            "{error:?}"
        );
        // Changed bounds are a structural refusal, whatever the options.
        oracle.bounds = vec![(0.5, 0.5)];
        let error = solve_under(
            &mut session,
            &mut oracle,
            100,
            ReusePolicy::RequireReuse,
            adaptive,
        )
        .unwrap_err();
        assert!(
            matches!(
                error,
                ProblemError::Reuse {
                    backend: Backend::Ipopt,
                    refusal: ReuseRefusal::Structure,
                }
            ),
            "{error:?}"
        );
    }
    fn solved(report: &SolveReport) -> bool {
        report.termination.category == Termination::Success
            && report
                .candidate
                .as_ref()
                .is_some_and(|c| (c.primal[0] - 1.0).abs() < 1e-6)
    }
    fn spral() -> Settings {
        Settings {
            linear: Linear::Spral {
                ordering: SpralOrdering::Metis,
                scaling: SpralScaling::Matching,
                pivot: SpralPivot::Block,
            },
            ..Settings::default()
        }
    }
    fn pardiso() -> Settings {
        Settings {
            linear: Linear::PardisoMkl {
                ordering: PardisoOrdering::Metis,
                matching: PardisoMatching::CompletePlus2x2,
            },
            ..Settings::default()
        }
    }
    fn text(value: &str) -> OptionValue {
        OptionValue::Text(value.into())
    }
    #[test]
    fn ipopt_mumps_metis_ordering_selectable() {
        // METIS is stated, never inherited from MUMPS's automatic choice (T06).
        assert_eq!(
            Settings::default().linear,
            Linear::Mumps {
                ordering: MumpsOrdering::Metis
            }
        );
        for ordering in [
            MumpsOrdering::Metis,
            MumpsOrdering::Amd,
            MumpsOrdering::Qamd,
        ] {
            let settings = Settings {
                linear: Linear::Mumps { ordering },
                ..Settings::default()
            };
            let report = solve_with(&mut Session::new(), Options::new(), &settings, 1).unwrap();
            assert!(solved(&report), "{:?}", report.termination);
            assert_eq!(report.options["linear_solver"], text("mumps"));
            assert_eq!(
                report.options["mumps_pivot_order"],
                OptionValue::Integer(settings::mumps_pivot_order(ordering))
            );
        }
        assert_eq!(settings::mumps_pivot_order(MumpsOrdering::Metis), 5);
    }
    #[test]
    fn ipopt_spral_selectable_and_recorded() {
        let settings = spral();
        for threads in [1, 2] {
            let report =
                solve_with(&mut Session::new(), Options::new(), &settings, threads).unwrap();
            assert!(solved(&report), "{:?}", report.termination);
            assert_eq!(report.options["linear_solver"], text("spral"));
            assert_eq!(report.options["spral_order"], text("metis"));
            assert_eq!(report.options["spral_pivot_method"], text("block"));
            assert_eq!(
                report.metrics["linear.threads"],
                Metric::Integer(i64::try_from(threads).unwrap())
            );
            // The record is the typed settings' serde encoding, never Rust `Debug` (F30).
            let linear: serde_json::Value =
                serde_json::from_str(&report.provenance["linear"]).unwrap();
            assert_eq!(
                linear,
                serde_json::json!({"kind": "spral", "ordering": "metis", "scaling": "matching", "pivot": "block"})
            );
        }
    }
    #[test]
    fn ipopt_pardisomkl_selectable_under_cbwr() {
        // The image pins MKL_CBWR and admission reads the branch back (T04).
        let observed = Runtime::observe();
        assert_eq!(observed.cbwr, runtime::pinned_cbwr());
        assert!(!observed.mkl_dynamic);
        let settings = pardiso();
        for threads in [1, 2] {
            let report =
                solve_with(&mut Session::new(), Options::new(), &settings, threads).unwrap();
            assert!(solved(&report), "{:?}", report.termination);
            assert_eq!(report.options["linear_solver"], text("pardisomkl"));
            assert_eq!(report.options["pardisomkl_order"], text("metis"));
            assert_eq!(report.provenance["mkl_cbwr"], "COMPATIBLE");
        }
        // Outside the pinned branch, or with dynamic MKL threads, Pardiso is refused.
        for runtime in [
            Runtime {
                cbwr: 2,
                ..observed
            },
            Runtime {
                mkl_dynamic: true,
                ..observed
            },
        ] {
            assert!(
                matches!(admit(&settings, 1, &runtime), Err(ProblemError::Unsupported(m)) if m.contains("MKL")),
                "{runtime:?}"
            );
        }
    }
    #[test]
    fn ipopt_unavailable_linear_solver_refused() {
        let observed = Runtime::observe();
        // The image links exactly MUMPS, SPRAL and oneMKL Pardiso: no HSL, no loaded Pardiso.
        for solver in IpoptLinearSolver::ALL {
            assert_ne!(observed.linked & settings::mask(solver), 0, "{solver:?}");
        }
        assert_eq!(observed.linked & ffi::IPOPTLINEARSOLVER_ALLHSL, 0);
        assert_eq!(observed.linked & ffi::IPOPTLINEARSOLVER_PARDISO, 0);
        for solver in IpoptLinearSolver::ALL {
            let settings = match solver {
                IpoptLinearSolver::Mumps => Settings::default(),
                IpoptLinearSolver::Spral => spral(),
                IpoptLinearSolver::Pardisomkl => pardiso(),
            };
            let without = Runtime {
                linked: observed.linked & !settings::mask(solver),
                ..observed
            };
            assert!(
                matches!(admit(&settings, 1, &without), Err(ProblemError::Unsupported(m)) if m.contains("not linked")),
                "{solver:?}"
            );
        }
        // A raw option cannot select any solver, load an excluded library or change the
        // typed ordering: the typed settings are the only way in.
        for (key, value) in [
            ("linear_solver", "ma57"),
            ("linear_solver", "mumps"),
            ("hsllib", "libhsl.so"),
            ("pardisolib", "libpardiso.so"),
            ("mumps_pivot_order", "7"),
        ] {
            let error = solve_with(
                &mut Session::new(),
                Options::from([(key.into(), text(value))]),
                &Settings::default(),
                1,
            )
            .unwrap_err();
            assert!(
                matches!(error, ProblemError::Contract(_)),
                "{key}: {error:?}"
            );
        }
        // MUMPS is sequential: threads are refused, never silently ignored.
        assert!(matches!(
            solve_with(&mut Session::new(), Options::new(), &Settings::default(), 2),
            Err(ProblemError::Unsupported(_))
        ));
    }
    const SPRAL_CHILD: &str = "PSE_TEST_SPRAL_WITHOUT_CANCELLATION";
    #[test]
    fn spral_refused_without_omp_cancellation() {
        if std::env::var_os(SPRAL_CHILD).is_some() {
            // A process whose OpenMP runtime started without OMP_CANCELLATION.
            assert!(!Runtime::observe().cancellation);
            let error = solve_with(&mut Session::new(), Options::new(), &spral(), 1).unwrap_err();
            assert!(
                matches!(&error, ProblemError::Unsupported(m) if m.contains("OMP_CANCELLATION")),
                "{error:?}"
            );
            // MUMPS needs neither setting.
            assert!(solved(&run(&mut Session::new(), Options::new())));
            return;
        }
        let observed = Runtime::observe();
        assert!(
            observed.cancellation,
            "the solver image sets OMP_CANCELLATION"
        );
        assert_ne!(observed.proc_bind, 0, "the solver image sets OMP_PROC_BIND");
        for (runtime, variable) in [
            (
                Runtime {
                    cancellation: false,
                    ..observed
                },
                "OMP_CANCELLATION",
            ),
            (
                Runtime {
                    proc_bind: 0,
                    ..observed
                },
                "OMP_PROC_BIND",
            ),
        ] {
            assert!(
                matches!(admit(&spral(), 1, &runtime), Err(ProblemError::Unsupported(m)) if m.contains(variable))
            );
        }
        // The same refusal in a fresh process without the variable, before any solve.
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "ipopt::tests::spral_refused_without_omp_cancellation",
                "--test-threads",
                "1",
            ])
            .env(SPRAL_CHILD, "1")
            .env_remove("OMP_CANCELLATION")
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            output.status.success() && stdout.contains("1 passed"),
            "{stdout}\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    #[test]
    fn single_blas_provider_in_process() {
        for settings in [Settings::default(), spral(), pardiso()] {
            assert!(solved(
                &solve_with(&mut Session::new(), Options::new(), &settings, 1).unwrap()
            ));
        }
        runtime::tests::assert_single_provider();
    }
    #[test]
    fn ipopt_linear_solver_in_profile_key() {
        use crate::execution::{BackendExecution, BackendSettings, LINKED};
        let key = |settings: Settings| BackendSettings::Ipopt(settings).identity().unwrap();
        let mumps = key(Settings::default());
        for other in [
            Settings {
                linear: Linear::Mumps {
                    ordering: MumpsOrdering::Amd,
                },
                ..Settings::default()
            },
            spral(),
            pardiso(),
        ] {
            assert_ne!(mumps, key(other));
        }
        // Library versions, the image manifest and the CBWR branch enter every profile key
        // through the linked build identity.
        let adapter: &dyn BackendExecution = LINKED.get(Backend::Ipopt).unwrap();
        assert_eq!(adapter.build(), Some(build().identity));
        assert_eq!(LINKED.build_identity(), LINKED.build_identity());
    }
    #[test]
    fn metric_names_state_coordinates() {
        // Ipopt sees the normalized model: its `unscaled` readbacks undo only Ipopt's own
        // scaling, so they are normalized-coordinate values, never physical ones (F10).
        let report = run(&mut Session::new(), Options::new());
        let events = report
            .events
            .iter()
            .filter(|e| e.phase == "ipopt.iteration")
            .collect::<Vec<_>>();
        assert!(!events.is_empty());
        for event in events {
            for key in event.values.keys() {
                assert!(!key.contains("unscaled"), "{key}");
            }
            for key in [
                "objective.normalized",
                "stationarity.normalized",
                "iterate.normalized.infinity_norm",
                "primal.native",
                "dual.native",
            ] {
                assert!(event.values.contains_key(key), "{key}");
            }
        }
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
