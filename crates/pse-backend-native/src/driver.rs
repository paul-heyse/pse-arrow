// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Synchronous, private FFI kernel owned exclusively by the native physical operator.
mod callbacks;
mod workspace;

use crate::{HessianPolicy, NativeError, SolveOptions, Variable, error::invalid};
use datafusion::arrow::array::RecordBatch;
use pse_ids::{CancellationToken, MemoryReserver, ReservationLease};
use pse_ipopt_sys as ffi;
use pse_numerics::EvaluationProgram;
use std::{ffi::CStr, ptr::NonNull, sync::Arc};

pub(crate) struct Outcome {
    pub program: pse_relations::generated::runtime::numerical_programs::Row,
    pub scenario_id: pse_ids::SemanticId,
    pub status: i32,
    pub values: Vec<f64>,
    pub constraints: Vec<f64>,
    pub objective: f64,
    pub constraint_duals: Vec<f64>,
    pub lower_duals: Vec<f64>,
    pub upper_duals: Vec<f64>,
    pub iterations: Vec<Iteration>,
    pub failure: Option<NativeError>,
    pub cancelled: bool,
    pub allocation: Arc<ReservationLease>,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Iteration {
    pub index: i32,
    pub restoration: bool,
    pub objective: f64,
    pub primal_infeasibility: f64,
    pub dual_infeasibility: f64,
    pub barrier: f64,
    pub step: f64,
}

struct Problem(NonNull<ffi::IpoptProblemInfo>);
impl Drop for Problem {
    fn drop(&mut self) {
        // SAFETY: the unique problem owner is dropped only after IpoptSolve and all
        // synchronous callbacks return. No Rust owner retains the C handle.
        unsafe { ffi::FreeIpoptProblem(self.0.as_ptr()) };
    }
}
impl Problem {
    fn string(&self, key: &CStr, value: &CStr) -> Result<(), NativeError> {
        // SAFETY: Ipopt copies the NUL-terminated strings; its historical mutable
        // pointer signature does not mutate these option arguments.
        let ok = unsafe {
            ffi::AddIpoptStrOption(
                self.0.as_ptr(),
                key.as_ptr().cast_mut(),
                value.as_ptr().cast_mut(),
            )
        };
        option(ok, key)
    }
    fn number(&self, key: &CStr, value: f64) -> Result<(), NativeError> {
        // SAFETY: the problem is live and the key is NUL-terminated and copied.
        option(
            unsafe { ffi::AddIpoptNumOption(self.0.as_ptr(), key.as_ptr().cast_mut(), value) },
            key,
        )
    }
    fn integer(&self, key: &CStr, value: i32) -> Result<(), NativeError> {
        // SAFETY: the problem is live and the key is NUL-terminated and copied.
        option(
            unsafe { ffi::AddIpoptIntOption(self.0.as_ptr(), key.as_ptr().cast_mut(), value) },
            key,
        )
    }
}
fn option(ok: bool, key: &CStr) -> Result<(), NativeError> {
    if ok {
        Ok(())
    } else {
        Err(NativeError::Ipopt(key.to_string_lossy().into_owned()))
    }
}

/// The generated C declarations and loaded library must describe the same ABI.
pub(crate) fn qualify_version() -> Result<(), NativeError> {
    let (mut major, mut minor, mut release) = (0, 0, 0);
    // SAFETY: three live, distinct C integer destinations; this stable version API
    // is checked before any callback or problem crosses the version-specific ABI.
    unsafe { ffi::GetIpoptVersion(&raw mut major, &raw mut minor, &raw mut release) };
    let actual = (i64::from(major), i64::from(minor), i64::from(release));
    let expected = (
        i64::from(ffi::IPOPT_VERSION_MAJOR),
        i64::from(ffi::IPOPT_VERSION_MINOR),
        i64::from(ffi::IPOPT_VERSION_RELEASE),
    );
    if actual != expected {
        return Err(NativeError::Capability(format!(
            "loaded Ipopt {actual:?} differs from generated C interface {expected:?}"
        )));
    }
    Ok(())
}

pub(crate) struct Request {
    pub program: Arc<EvaluationProgram>,
    pub input: RecordBatch,
    pub variables: Vec<Variable>,
    pub constraints: Vec<(Option<f64>, Option<f64>, f64)>,
    pub options: SolveOptions,
    pub cancel: CancellationToken,
    pub reserver: Arc<dyn MemoryReserver>,
    /// Explicit reserved allowance for Ipopt/MUMPS allocations outside Rust tracking.
    /// This is admission accounting, not a measured hard limit on C malloc.
    pub foreign_bytes: usize,
}

pub(crate) fn solve(request: &Request) -> Result<Outcome, NativeError> {
    qualify_version()?;
    request.options.validate()?;
    request.cancel.checkpoint()?;
    let mut data = workspace::Workspace::new(request)?;
    let problem = construct(request, &data)?;
    let mut values = data.initial.clone();
    let mut constraints = vec![f64::NAN; request.constraints.len()];
    let mut objective = f64::NAN;
    let mut constraint_duals = vec![f64::NAN; request.constraints.len()];
    let mut lower_duals = vec![f64::NAN; values.len()];
    let mut upper_duals = vec![f64::NAN; values.len()];
    // SAFETY: all mutable output arrays are distinct and have the C problem's
    // extents. `data` stays at this address until synchronous Solve/callbacks return.
    // The solver does not retain user_data after this call.
    let status = unsafe {
        ffi::IpoptSolve(
            problem.0.as_ptr(),
            values.as_mut_ptr(),
            constraints.as_mut_ptr(),
            &raw mut objective,
            constraint_duals.as_mut_ptr(),
            lower_duals.as_mut_ptr(),
            upper_duals.as_mut_ptr(),
            (&raw mut data).cast(),
        )
    };
    drop(problem);
    let scenarios = request
        .input
        .column_by_name("scenario_id")
        .and_then(|values| {
            values
                .as_any()
                .downcast_ref::<datafusion::arrow::array::FixedSizeBinaryArray>()
        })
        .ok_or_else(|| invalid("solver scenario identity is absent"))?;
    let scenario_id = pse_ids::SemanticId::try_from_slice(scenarios.value(0))
        .map_err(|error| invalid(error.to_string()))?;
    Ok(Outcome {
        program: request.program.contract().clone(),
        scenario_id,
        status,
        values,
        constraints,
        objective,
        constraint_duals,
        lower_duals,
        upper_duals,
        iterations: data.iterations,
        failure: data.failure,
        cancelled: request.cancel.is_cancelled(),
        allocation: data.allocation,
    })
}

fn construct(request: &Request, data: &workspace::Workspace) -> Result<Problem, NativeError> {
    let n = i32::try_from(data.positions.len()).map_err(|_| invalid("too many variables"))?;
    let m =
        i32::try_from(request.constraints.len()).map_err(|_| invalid("too many constraints"))?;
    let nnz = i32::try_from(data.jacobian.len())
        .map_err(|_| invalid("Jacobian exceeds C index range"))?;
    let mut lower = Vec::new();
    let mut upper = Vec::new();
    let mut variable_scale = Vec::new();
    for variable in &request.variables {
        let (lo, hi) = bounds(variable.lower, variable.upper)?;
        scale(variable.scale)?;
        lower.push(lo);
        upper.push(hi);
        variable_scale.push(variable.scale);
    }
    let mut g_lower = Vec::new();
    let mut g_upper = Vec::new();
    let mut g_scale = Vec::new();
    for &(lo, hi, scaling) in &request.constraints {
        let (lo, hi) = bounds(lo, hi)?;
        scale(scaling)?;
        g_lower.push(lo);
        g_upper.push(hi);
        g_scale.push(scaling);
    }
    // SAFETY: every bound array has exactly the supplied extent; Ipopt copies
    // bounds at creation. The callback signatures come from these exact headers.
    let pointer = unsafe {
        ffi::CreateIpoptProblem(
            n,
            lower.as_mut_ptr(),
            upper.as_mut_ptr(),
            m,
            g_lower.as_mut_ptr(),
            g_upper.as_mut_ptr(),
            nnz,
            0,
            0,
            Some(callbacks::objective),
            Some(callbacks::constraints),
            Some(callbacks::gradient),
            Some(callbacks::jacobian),
            Some(callbacks::hessian),
        )
    };
    let problem = Problem(
        NonNull::new(pointer).ok_or_else(|| NativeError::Ipopt("problem construction".into()))?,
    );
    match request.options.hessian {
        HessianPolicy::LimitedMemory => {
            problem.string(c"hessian_approximation", c"limited-memory")?;
        }
    }
    problem.string(c"option_file_name", c"")?;
    problem.string(c"sb", c"yes")?;
    problem.string(c"linear_solver", c"mumps")?;
    problem.string(c"nlp_scaling_method", c"user-scaling")?;
    problem.integer(c"print_level", 0)?;
    problem.integer(
        c"max_iter",
        i32::try_from(request.options.max_iterations).map_err(|_| invalid("iteration range"))?,
    )?;
    problem.number(c"tol", request.options.tolerance)?;
    problem.number(c"acceptable_tol", request.options.tolerance)?;
    problem.number(c"max_wall_time", request.options.max_wall_seconds)?;
    // Explicit infinity thresholds ensure finite supplied bounds are never silently
    // reinterpreted as absent by Ipopt's conventional 1e19 threshold.
    problem.number(c"nlp_lower_bound_inf", -f64::MAX)?;
    problem.number(c"nlp_upper_bound_inf", f64::MAX)?;
    // SAFETY: both scale arrays have the exact problem extents and Ipopt copies them.
    let scaled = unsafe {
        ffi::SetIpoptProblemScaling(
            problem.0.as_ptr(),
            1.0,
            variable_scale.as_mut_ptr(),
            g_scale.as_mut_ptr(),
        )
    };
    option(scaled, c"problem scaling")?;
    // SAFETY: this callback has the generated ABI and never lets an unwind escape.
    let set =
        unsafe { ffi::SetIntermediateCallback(problem.0.as_ptr(), Some(callbacks::intermediate)) };
    option(set, c"intermediate callback")?;
    Ok(problem)
}
fn bounds(lower: Option<f64>, upper: Option<f64>) -> Result<(f64, f64), NativeError> {
    for value in lower.into_iter().chain(upper) {
        if !value.is_finite() || value.abs() >= f64::MAX {
            return Err(invalid(
                "finite bounds must lie strictly within Float64 extrema",
            ));
        }
    }
    let (lo, hi) = (lower.unwrap_or(-f64::MAX), upper.unwrap_or(f64::MAX));
    if lo > hi {
        return Err(invalid("lower bound exceeds upper bound"));
    }
    Ok((lo, hi))
}
fn scale(value: f64) -> Result<(), NativeError> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(invalid("scales must be finite and positive"))
    }
}
