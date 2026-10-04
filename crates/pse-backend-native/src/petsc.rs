// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(
    unsafe_code,
    reason = "serialized PETSc ownership and panic-contained callbacks over the checked project ABI"
)]
//! Serial PETSc root profiles, with process lifetime independent of an attempt.
pub use crate::settings::petsc::{Linear, Method, Preconditioner, Settings};
use crate::{
    NativeFailureKind, NativeStatus, NleOracle, ProblemError,
    callback::CallbackState,
    quality::{Quality, Tolerances, Violation},
    solve::*,
};
use pse_petsc_sys as ffi;
use std::{
    cell::Cell,
    ffi::{CStr, c_void},
    panic::{AssertUnwindSafe, catch_unwind},
    ptr,
    sync::{Mutex, MutexGuard, TryLockError},
    time::Duration,
};
mod blocks;
mod derived;
#[cfg(test)]
mod derived_tests;
mod flow;
pub use blocks::{BlockComposition, BlockOracle, BlockSpec, GhostCoordinates, solve_blocks};
pub use derived::{DomainOracle, DomainWitness};
pub use flow::{ArtificialFlow, FlowLimits, FrozenMassRefresh, solve_flow};

/// Consumer-resolved controls and ownership of one finite PETSc operation.
#[derive(Debug)]
pub struct SolveRequest<'a> {
    /// Actual declared native controls.
    pub controls: &'a Controls,
    /// Consumer-issued resolved original accuracy.
    pub accuracy: &'a ResolvedAccuracy,
    /// Original task clock/cancellation and admitted allocation owner.
    pub execution: Execution,
    /// Independent original assessment tolerances.
    pub tolerances: &'a Tolerances,
    /// Declared admitted start, when present.
    pub warm: Option<&'a WarmStart>,
    /// Exact original/profile/native-layout identity.
    pub compatibility: &'a Compatibility,
}

/// Identity of the actual receipt-qualified native source and project bridge.
pub fn build() -> pse_ids::ContentHash {
    let mut hasher = pse_ids::FramedHasher::new(pse_ids::Frame::NativePetscBuildV1);
    hasher.str(ffi::BUILD_ID).str(ffi::VERSION);
    hasher.finish_hash()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Fresh,
    Ready,
    Shutdown,
    Failed,
}
static PROCESS: Mutex<Phase> = Mutex::new(Phase::Fresh);
thread_local! { static ACTIVE: Cell<bool> = const { Cell::new(false) }; }
struct Admission {
    phase: MutexGuard<'static, Phase>,
    failed: Cell<bool>,
}
impl Admission {
    fn enter(execution: &Execution) -> Result<Self, ProblemError> {
        if ACTIVE.get() {
            return Err(ProblemError::Unsupported(
                "nested PETSc process admission on its owning thread".into(),
            ));
        }
        let mut phase = loop {
            execution.check()?;
            match PROCESS.try_lock() {
                Ok(guard) => break guard,
                Err(TryLockError::WouldBlock) => {
                    std::thread::park_timeout(Duration::from_millis(1))
                }
                Err(TryLockError::Poisoned(_)) => {
                    return Err(ProblemError::internal("PETSc process owner poisoned"));
                }
            }
        };
        execution.check()?;
        if *phase == Phase::Fresh {
            // Once initialization begins, an error must never permit a second initialization.
            *phase = Phase::Failed;
            let (mut initialized, mut finalized) = (0, 0);
            {
                check(
                    // SAFETY: The process admission serializes PETSc access; scalar outputs are writable locals and the bridge accepts this value query.
                    unsafe { ffi::pse_petsc_initialized(&mut initialized, &mut finalized) },
                    "process state",
                )?;
            }
            if initialized != 0 || finalized != 0 {
                return Err(ProblemError::Unsupported(
                    "PETSc process lifetime was initialized or finalized outside its owner".into(),
                ));
            }
            {
                check(
                    // SAFETY: The process admission serializes PETSc access; scalar outputs are writable locals and the bridge accepts this value query.
                    unsafe { ffi::pse_petsc_initialize() },
                    "process initialization",
                )?;
            }
            *phase = Phase::Ready;
        }
        if *phase != Phase::Ready {
            return Err(ProblemError::Unsupported(
                "PETSc process owner is terminally shut down or failed".into(),
            ));
        }
        let mut empty = 0;
        {
            check(
                // SAFETY: The process admission serializes PETSc access; scalar outputs are writable locals and the bridge accepts this value query.
                unsafe { ffi::pse_petsc_default_options_empty(&mut empty) },
                "ambient options query",
            )?;
        }
        if empty == 0 {
            return Err(ProblemError::Unsupported(
                "PETSc default options database must be empty; profiles use private options".into(),
            ));
        }
        ACTIVE.set(true);
        Ok(Self {
            phase,
            failed: Cell::new(false),
        })
    }
}
impl Drop for Admission {
    fn drop(&mut self) {
        if self.failed.get() {
            *self.phase = Phase::Failed;
        }
        ACTIVE.set(false);
    }
}

/// Finalize the process owner once, after all admitted native work and objects end.
/// Subsequent attempts are refused. Concurrent work is refused, never interrupted.
///
/// # Errors
/// The owner is busy, poisoned, failed, or native finalization fails.
pub fn shutdown() -> Result<(), ProblemError> {
    let mut phase = match PROCESS.try_lock() {
        Ok(guard) => guard,
        Err(TryLockError::WouldBlock) => {
            return Err(ProblemError::Unsupported(
                "PETSc shutdown requires no admitted work or live handles".into(),
            ));
        }
        Err(TryLockError::Poisoned(_)) => {
            return Err(ProblemError::internal("PETSc process owner poisoned"));
        }
    };
    match *phase {
        Phase::Ready => {
            *phase = Phase::Failed;
            // SAFETY: admission owns the same mutex through all object destruction.
            {
                check(
                    // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
                    unsafe { ffi::pse_petsc_finalize() },
                    "process finalization",
                )?;
            }
            *phase = Phase::Shutdown;
            Ok(())
        }
        Phase::Fresh | Phase::Shutdown => {
            *phase = Phase::Shutdown;
            Ok(())
        }
        Phase::Failed => Err(ProblemError::Unsupported(
            "failed PETSc process owner cannot finalize safely".into(),
        )),
    }
}

fn check(code: i32, operation: &str) -> Result<(), ProblemError> {
    if code == 0 {
        return Ok(());
    }
    let mut message = ptr::null();
    // SAFETY: native static diagnostic storage is copied before returning.
    let description = if {
        // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
        unsafe { ffi::pse_petsc_error_message(code, &mut message) }
    } == 0
        && !message.is_null()
    {
        // SAFETY: The pointer was checked non-null; PETSc supplies a live null-terminated library string for this query.
        unsafe { CStr::from_ptr(message) }
            .to_string_lossy()
            .into_owned()
    } else {
        "unavailable native error description".into()
    };
    // SAFETY: this pure bridge query classifies an integer using pinned header constants and touches no caller pointers.
    let kind = match unsafe { ffi::pse_petsc_error_class(code) } {
        ffi::ErrorClass::Resource => NativeFailureKind::Resource,
        ffi::ErrorClass::Contract => NativeFailureKind::Contract,
        ffi::ErrorClass::Numerical => NativeFailureKind::Numerical,
        ffi::ErrorClass::Infrastructure | ffi::ErrorClass::Success => {
            NativeFailureKind::Infrastructure
        }
    };
    Err(ProblemError::Native {
        status: NativeStatus {
            backend: Backend::Petsc,
            code: code.into(),
            name: error_name(code),
        },
        kind,
        detail: format!("{operation}: {description}"),
    })
}
fn error_name(code: i32) -> String {
    let mut name = ptr::null();
    // SAFETY: the bridge returns tagged enum spellings in static storage.
    if {
        // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
        unsafe { ffi::pse_petsc_error_name(code, &mut name) }
    } == 0
        && !name.is_null()
    {
        // SAFETY: The pointer was checked non-null; PETSc supplies a live null-terminated library string for this query.
        unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned()
    } else {
        "PETSC_UNKNOWN_ERROR".into()
    }
}
// Return whether scientific evaluation must stop. Native counters/readback/cleanup remain lawful.
fn finish_native_error(
    report: &mut SolveReport,
    state: &mut CallbackState,
    code: i32,
    operation: &str,
) -> bool {
    state.finish(report);
    if code == 0 {
        return state.terminal.is_some();
    }
    report.termination.code = code.into();
    report.termination.name = error_name(code);
    // PETSC_ERR_USER is a wrapper when the actual callback owner retained a witness.
    if state.terminal.is_some() {
        return true;
    }
    let Err(cause) = check(code, operation) else {
        return false;
    };
    let kind = match &cause {
        ProblemError::Native { kind, .. } => *kind,
        _ => NativeFailureKind::Infrastructure,
    };
    report.termination.category = match kind {
        NativeFailureKind::Resource => Termination::ResourceExhausted,
        NativeFailureKind::Contract | NativeFailureKind::Infrastructure => Termination::Invalid,
        NativeFailureKind::Numerical => Termination::Numerical,
    };
    report.termination.message = Some(cause.to_string());
    report.record_validation_failure(cause);
    kind != NativeFailureKind::Numerical
}
fn retain_native_candidate(report: &mut SolveReport, point: Vec<f64>) {
    report.candidate = Some(Candidate {
        kind: CandidateKind::FinalIterate,
        primal: point,
        objective: None,
        row_dual: None,
        bound_dual: None,
        reduced_costs: None,
        slacks: None,
        commitment: None,
    });
}
type Destroy = unsafe extern "C" fn(*mut ffi::Handle) -> i32;
struct Objects<'a> {
    entries: Vec<(ffi::Handle, Destroy)>,
    failed: &'a Cell<bool>,
}
impl<'a> Objects<'a> {
    fn new(admission: &'a Admission) -> Self {
        Self {
            entries: vec![],
            failed: &admission.failed,
        }
    }
    fn create(
        &mut self,
        create: unsafe extern "C" fn(*mut ffi::Handle) -> i32,
        destroy: Destroy,
    ) -> Result<ffi::Handle, ProblemError> {
        let mut handle = ptr::null_mut();
        // SAFETY: this admission owns the process mutex and the newly allocated object.
        let code = unsafe { create(&mut handle) };
        if !handle.is_null() {
            self.entries.push((handle, destroy));
        }
        check(code, "object creation")?;
        if handle.is_null() {
            return Err(ProblemError::internal(
                "PETSc successful creation returned null",
            ));
        }
        Ok(handle)
    }
    fn vector(&mut self, values: &[f64]) -> Result<ffi::Handle, ProblemError> {
        let n = index(values.len())?;
        let mut handle = ptr::null_mut();
        let code = {
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_vec_create(n, &mut handle) }
        };
        if !handle.is_null() {
            self.entries.push((handle, ffi::pse_petsc_vec_destroy));
        }
        check(code, "vector creation")?;
        if handle.is_null() {
            return Err(ProblemError::internal(
                "PETSc successful vector creation returned null",
            ));
        }
        {
            check(
                // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
                unsafe { ffi::pse_petsc_vec_write(handle, n, values.as_ptr()) },
                "vector initialization",
            )?;
        }
        Ok(handle)
    }
    fn close(&mut self) -> Result<(), ProblemError> {
        let mut first = 0;
        while let Some((mut handle, destroy)) = self.entries.pop() {
            // SAFETY: reverse registration destroys SNES before its Vec/Mat/options owners.
            let code = unsafe { destroy(&mut handle) };
            if code != 0 {
                self.failed.set(true);
                if first == 0 {
                    first = code;
                }
            }
        }
        check(first, "object destruction")
    }
}
impl Drop for Objects<'_> {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
fn index(n: usize) -> Result<i32, ProblemError> {
    i32::try_from(n).map_err(|_| ProblemError::Contract("PETSc PetscInt32 dimension".into()))
}
fn read(x: ffi::Handle, n: usize) -> Result<Vec<f64>, ProblemError> {
    let mut values = vec![0.0; n];
    {
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_vec_read(x, index(n)?, values.as_mut_ptr()) },
            "vector read",
        )?;
    }
    if values.iter().any(|v| !v.is_finite()) {
        return Err(ProblemError::numerical("nonfinite PETSc vector"));
    }
    Ok(values)
}
struct Context<'a> {
    oracle: &'a mut dyn NleOracle,
    state: CallbackState,
    n: usize,
    rows: Vec<i32>,
    columns: Vec<i32>,
    snes: ffi::Handle,
}
impl Context<'_> {
    fn refused(&self) -> i32 {
        if self.state.terminal.is_some() {
            ffi::EVAL_TERMINAL
        } else {
            ffi::EVAL_DOMAIN
        }
    }
    fn publish(&mut self, operation: impl FnOnce() -> Result<(), ProblemError>) -> i32 {
        // Publication is part of the same callback, not a second counted evaluation.
        match catch_unwind(AssertUnwindSafe(operation)) {
            Ok(Ok(())) => ffi::EVAL_OK,
            Ok(Err(error)) => {
                self.state.terminal = Some((Termination::Evaluation, error.to_string()));
                self.state.last_failure = Some(error);
                ffi::EVAL_TERMINAL
            }
            Err(_) => {
                self.state.terminal =
                    Some((Termination::Panic, "panic publishing PETSc callback".into()));
                ffi::EVAL_TERMINAL
            }
        }
    }
}
unsafe extern "C" fn residual(ctx: *mut c_void, x: ffi::Handle, f: ffi::Handle) -> i32 {
    // SAFETY: the pinned context and serial mutable access remain owned through SNES destruction.
    let context = unsafe { &mut *ctx.cast::<Context<'_>>() };
    let n = context.n;
    let output = context.state.evaluate("residual", || {
        let x = read(x, n)?;
        let mut output = vec![0.0; n];
        context.oracle.residual(&x, &mut output)?;
        if output.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical("nonfinite PETSc residual"));
        }
        Ok(output)
    });
    let Some(output) = output else {
        return context.refused();
    };
    // Publication is contained too; any ABI failure becomes a terminal typed cause.
    context.publish(|| {
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_vec_write(f, index(n)?, output.as_ptr()) },
            "residual publication",
        )
    })
}
unsafe extern "C" fn jacobian(
    ctx: *mut c_void,
    x: ffi::Handle,
    a: ffi::Handle,
    p: ffi::Handle,
) -> i32 {
    // SAFETY: the owning SNES invokes this context under process admission.
    let context = unsafe { &mut *ctx.cast::<Context<'_>>() };
    let output = context.state.evaluate("jacobian", || {
        let x = read(x, context.n)?;
        let mut output = vec![0.0; context.rows.len()];
        context.oracle.jacobian(&x, &mut output)?;
        if output.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical("nonfinite PETSc Jacobian"));
        }
        Ok(output)
    });
    let Some(output) = output else {
        return context.refused();
    };
    let rows = context.rows.as_ptr();
    let columns = context.columns.as_ptr();
    context.publish(|| {
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe {
                ffi::pse_petsc_mat_write(a, index(output.len())?, rows, columns, output.as_ptr())
            },
            "Jacobian publication",
        )?;
        if a != p {
            check(
                // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
                unsafe {
                    ffi::pse_petsc_mat_write(
                        p,
                        index(output.len())?,
                        rows,
                        columns,
                        output.as_ptr(),
                    )
                },
                "preconditioner publication",
            )?;
        }
        Ok(())
    })
}
unsafe extern "C" fn convergence(
    ctx: *mut c_void,
    iteration: i32,
    xnorm: f64,
    ynorm: f64,
    fnorm: f64,
    reason: *mut i32,
) -> i32 {
    // SAFETY: valid context and writable scalar supplied by the C trampoline.
    let context = unsafe { &mut *ctx.cast::<Context<'_>>() };
    let mut observed = 0;
    let result = context.state.evaluate("convergence", || {
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe {
                ffi::pse_petsc_snes_default_convergence(
                    context.snes,
                    iteration,
                    xnorm,
                    ynorm,
                    fnorm,
                    &mut observed,
                )
            },
            "native convergence",
        )
    });
    // SAFETY: The C trampoline supplies a non-null aligned writable scalar for this callback result.
    unsafe {
        *reason = if result.is_some() {
            observed
        } else {
            ffi::SNES_USER_DIVERGED
        };
    }
    ffi::EVAL_OK
}

/// Execute the declared serial unbounded root profile against its original oracle.
///
/// # Errors
/// Admission, native construction or teardown fails. Callback failures are retained in the report.
pub fn solve(
    oracle: &mut dyn NleOracle,
    initial: &[f64],
    settings: &Settings,
    request: SolveRequest<'_>,
) -> Result<SolveReport, ProblemError> {
    let SolveRequest {
        controls,
        accuracy,
        execution,
        tolerances,
        warm,
        compatibility,
    } = request;
    settings.validate()?;
    controls.validate()?;
    accuracy.validate()?;
    oracle.operations().admit(oracle.contract(), false)?;
    let n = oracle.contract().variables.len();
    tolerances.validate(n, n)?;
    if initial.len() != n || initial.iter().any(|v| !v.is_finite()) {
        return Err(ProblemError::Contract(
            "PETSc finite initial coordinate inventory".into(),
        ));
    }
    if controls.threads != 1
        || !controls.options.is_empty()
        || accuracy.native_scaling
        || accuracy.acceptable.is_some()
    {
        return Err(ProblemError::Unsupported(
            "PETSc requires serial typed controls, original scaling and strict acceptance".into(),
        ));
    }
    if controls.reuse == ReusePolicy::RequireReuse {
        return Err(ProblemError::Unsupported("PETSc RequireReuse requires retained SNES ownership; this profile owns fresh per-attempt objects".into()));
    }
    if oracle
        .contract()
        .variables
        .iter()
        .any(|v| v.lower.is_finite() || v.upper.is_finite())
    {
        return Err(ProblemError::Unsupported(
            "PETSc root profiles do not enforce arbitrary bounds".into(),
        ));
    }
    if settings.method != Method::NewtonTrustRegion {
        return Err(ProblemError::Unsupported(
            "PETSc derived TS/NASM profile requires its owned composition interface".into(),
        ));
    }
    if compatibility.backend != Backend::Petsc {
        return Err(ProblemError::Contract("PETSc compatibility backend".into()));
    }
    let iteration_cap = i32::try_from(controls.iterations)
        .map_err(|_| ProblemError::Contract("PETSc iteration cap".into()))?;
    let start = if let Some(warm) = warm {
        warm.validate(compatibility)?;
        let WarmPayload::Root(values) = &warm.payload else {
            return Err(ProblemError::Contract(
                "PETSc root primal warm payload".into(),
            ));
        };
        if values.len() != n || values.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::Contract(
                "PETSc warm coordinate inventory".into(),
            ));
        }
        values.as_slice()
    } else {
        initial
    };
    let pattern = oracle.jacobian_pattern();
    crate::structural::check(
        oracle.contract(),
        pattern,
        &vec![(0.0, 0.0); n],
        crate::structural::Mode::Roots,
        oracle.structural_analysis(),
    )?;
    if pattern.nrows() != n || pattern.ncols() != n {
        return Err(ProblemError::Contract("PETSc Jacobian dimensions".into()));
    }
    let crate::nlp_pattern::Pattern { rows, columns } =
        crate::nlp_pattern::Pattern::new(pattern, false)?;
    let mut capacity = vec![0_i32; n];
    for &row in &rows {
        let row = row as usize;
        capacity[row] = capacity[row]
            .checked_add(1)
            .ok_or_else(|| ProblemError::Contract("PETSc sparse row capacity".into()))?;
    }
    let mut context = Box::new(Context {
        oracle,
        state: CallbackState::new(execution.clone()),
        n,
        rows,
        columns,
        snes: ptr::null_mut(),
    });
    let admission = Admission::enter(&execution)?;
    let _threads = crate::mkl::Threads::enter(1)?;
    // Validate the initial domain before submitting a native trajectory. Even a typed trial
    // refusal at this point is terminal; no clipping or invented replacement value is used.
    let admitted = context.state.evaluate("initial", || {
        let mut output = vec![0.0; n];
        context.oracle.residual(start, &mut output)?;
        if output.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical("nonfinite PETSc initial residual"));
        }
        Ok(())
    });
    if admitted.is_none() {
        if context.state.terminal.is_none() {
            context.state.terminal = Some((
                Termination::Evaluation,
                "invalid PETSc initial domain".into(),
            ));
            context.state.trial_rejections = 0;
        }
        let initial_stop = NativeTermination {
            code: 0,
            name: "PETSC_INITIAL_DOMAIN_REFUSED".into(),
            message: None,
            category: Termination::Evaluation,
            assurance: Assurance::None,
        };
        let mut report = SolveReport::new(
            Backend::Petsc,
            context.oracle.contract(),
            initial_stop,
            &execution,
        );
        report
            .provenance
            .insert("native".into(), format!("PETSc {}", ffi::VERSION));
        report.provenance.insert("build".into(), build().to_hex());
        context.state.finish(&mut report);
        return Ok(report);
    }
    let mut objects = Objects::new(&admission);
    let options = objects.create(
        ffi::pse_petsc_options_create,
        ffi::pse_petsc_options_destroy,
    )?;
    let x = objects.vector(start)?;
    let f = objects.vector(&vec![0.0; n])?;
    let mut matrix = ptr::null_mut();
    let matrix_code = {
        // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
        unsafe { ffi::pse_petsc_mat_create(index(n)?, index(n)?, capacity.as_ptr(), &mut matrix) }
    };
    if !matrix.is_null() {
        objects.entries.push((matrix, ffi::pse_petsc_mat_destroy));
    }
    check(matrix_code, "Jacobian creation")?;
    if matrix.is_null() {
        return Err(ProblemError::internal(
            "PETSc successful matrix creation returned null",
        ));
    }
    let snes = objects.create(ffi::pse_petsc_snes_create, ffi::pse_petsc_snes_destroy)?;
    context.snes = snes;
    let ctx = ptr::from_mut(context.as_mut()).cast();
    let ksp = match settings.linear {
        Linear::Gmres => c"gmres",
        Linear::Fgmres => c"fgmres",
        Linear::Bicgstab => c"bcgs",
        Linear::Preonly => c"preonly",
    };
    let pc = match settings.preconditioner {
        Preconditioner::None => c"none",
        Preconditioner::Jacobi => c"jacobi",
        Preconditioner::Ilu => c"ilu",
        Preconditioner::Lu => c"lu",
    };
    // SAFETY: all handles are registered under admission and callback owners remain pinned.
    {
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_snes_type(snes, c"newtontr".as_ptr()) },
            "method",
        )?;
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_snes_options(snes, options) },
            "private options",
        )?;
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_snes_layout(snes, x) },
            "layout",
        )?;
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_snes_function(snes, f, residual, ctx) },
            "residual callback",
        )?;
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_snes_jacobian(snes, matrix, matrix, jacobian, ctx) },
            "Jacobian callback",
        )?;
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_snes_convergence(snes, convergence, ctx) },
            "convergence callback",
        )?;
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe {
                ffi::pse_petsc_snes_tolerances(
                    snes,
                    accuracy.feasibility,
                    0.0,
                    0.0,
                    iteration_cap,
                    i32::MAX,
                )
            },
            "strict residual convergence",
        )?;
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe {
                ffi::pse_petsc_snes_tr_tolerances(
                    snes,
                    settings.trust.minimum,
                    settings.trust.maximum,
                    settings.trust.initial,
                )
            },
            "trust radius controls",
        )?;
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_snes_linear(snes, ksp.as_ptr(), pc.as_ptr()) },
            "linear solver controls",
        )?;
    }
    let code = {
        // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
        unsafe { ffi::pse_petsc_snes_solve(snes, x) }
    };
    let (mut reason, mut iterations, mut evaluations, mut linear_iterations, mut norm) =
        (0, 0, 0, 0, 0.0);
    {
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe {
                ffi::pse_petsc_snes_statistics(
                    snes,
                    &mut reason,
                    &mut iterations,
                    &mut evaluations,
                    &mut linear_iterations,
                    &mut norm,
                )
            },
            "native statistics",
        )?;
    }
    let mut name = ptr::null();
    {
        check(
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_snes_reason_name(snes, &mut name) },
            "native reason",
        )?;
    }
    let name = if name.is_null() {
        "SNES_UNKNOWN".into()
    } else {
        // SAFETY: The pointer was checked non-null; PETSc supplies a live null-terminated library string for this query.
        unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned()
    };
    let mut report = SolveReport::new(
        Backend::Petsc,
        context.oracle.contract(),
        native_termination(reason, name),
        &execution,
    );
    let native_terminal =
        finish_native_error(&mut report, &mut context.state, code, "nonlinear solve");
    report.evidence.work.iterations = u64::try_from(iterations).ok();
    report.evidence.start_submitted = warm.is_some();
    report.metrics.insert(
        "SNESGetNumberFunctionEvals".into(),
        Metric::Integer(evaluations.into()),
    );
    report.metrics.insert(
        "SNESGetLinearSolveIterations".into(),
        Metric::Integer(linear_iterations.into()),
    );
    report
        .metrics
        .insert("SNESGetFunctionNorm".into(), Metric::Real(norm));
    report
        .provenance
        .insert("native".into(), format!("PETSc {}", ffi::VERSION));
    report.provenance.insert("build".into(), build().to_hex());
    report.provenance.insert(
        "settings".into(),
        serde_json::to_string(settings).map_err(|e| ProblemError::internal(e.to_string()))?,
    );
    report.options = std::collections::BTreeMap::from([
        ("snes_type".into(), OptionValue::Text("newtontr".into())),
        ("snes_atol".into(), OptionValue::Real(accuracy.feasibility)),
        ("snes_rtol".into(), OptionValue::Real(0.0)),
        ("snes_stol".into(), OptionValue::Real(0.0)),
        ("snes_max_it".into(), OptionValue::Integer(iteration_cap)),
        ("snes_max_funcs".into(), OptionValue::Integer(i32::MAX)),
        (
            "snes_tr_deltamin".into(),
            OptionValue::Real(settings.trust.minimum),
        ),
        (
            "snes_tr_deltamax".into(),
            OptionValue::Real(settings.trust.maximum),
        ),
        (
            "snes_tr_delta0".into(),
            OptionValue::Real(settings.trust.initial),
        ),
        (
            "ksp_type".into(),
            OptionValue::Text(ksp.to_string_lossy().into_owned()),
        ),
        (
            "pc_type".into(),
            OptionValue::Text(pc.to_string_lossy().into_owned()),
        ),
    ]);
    context.state.finish(&mut report);
    match read(x, n) {
        Ok(point) => {
            report.candidate = Some(Candidate {
                kind: CandidateKind::FinalIterate,
                primal: point.clone(),
                objective: None,
                row_dual: None,
                bound_dual: None,
                reduced_costs: None,
                slacks: None,
                commitment: None,
            });
            if !native_terminal {
                let validated = crate::quality::contained(|| {
                    execution.check()?;
                    let mut values = vec![0.0; n];
                    context.oracle.residual(&point, &mut values)?;
                    let rows = context
                        .oracle
                        .contract()
                        .rows
                        .iter()
                        .zip(&values)
                        .zip(&tolerances.rows)
                        .map(|((id, value), tolerance)| Violation {
                            id: *id,
                            physical: value.abs(),
                            tolerance: *tolerance,
                        })
                        .collect();
                    let bounds = context
                        .oracle
                        .contract()
                        .variables
                        .iter()
                        .zip(&tolerances.variables)
                        .map(|(variable, tolerance)| Violation {
                            id: variable.id,
                            physical: 0.0,
                            tolerance: *tolerance,
                        })
                        .collect();
                    let quality = Quality::new(rows, bounds, vec![])?;
                    let observation = context.oracle.observe(values)?;
                    execution.check()?;
                    Ok((quality, observation))
                });
                if let Ok((quality, observation)) = validated {
                    if quality.feasible() && report.termination.category == Termination::Success {
                        report.termination.assurance = Assurance::Feasible;
                    }
                    report.quality = Some(quality);
                    report.observation = Some(observation);
                    report.warm_start = Some(WarmStart {
                        origin: None,
                        compatibility: compatibility.clone(),
                        payload: WarmPayload::Root(point),
                    });
                } else if let Err(error) = validated {
                    report.record_validation_failure(error);
                }
                context.state.finish(&mut report);
            }
        }
        Err(error) if !native_terminal => report.record_validation_failure(error),
        Err(_) => {} // Preserve the earlier terminal owner; readback is ancillary.
    }
    objects.close()?;
    Ok(report)
}

fn native_termination(code: i32, name: String) -> NativeTermination {
    let category = match code {
        ffi::SNES_FNORM_ABSOLUTE | ffi::SNES_FNORM_RELATIVE => Termination::Success,
        ffi::SNES_STEP_RELATIVE | ffi::SNES_TR_DELTA => Termination::Limit,
        ffi::SNES_FUNCTION_COUNT => Termination::Limit,
        ffi::SNES_MAX_ITERATIONS => Termination::IterationLimit,
        ffi::SNES_FUNCTION_DOMAIN | ffi::SNES_JACOBIAN_DOMAIN => Termination::Evaluation,
        0 => Termination::Inconclusive,
        _ => Termination::Numerical,
    };
    NativeTermination {
        code: code.into(),
        name,
        message: None,
        category,
        assurance: Assurance::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    };
    #[derive(Debug)]
    struct LogOracle {
        inner: crate::solver_tests::Polynomial,
        calls: usize,
        rejected: usize,
        fail_at: usize,
        panic_at: usize,
    }
    impl LogOracle {
        fn new() -> Self {
            Self {
                inner: crate::solver_tests::Polynomial::new(),
                calls: 0,
                rejected: 0,
                fail_at: usize::MAX,
                panic_at: usize::MAX,
            }
        }
    }
    impl NleOracle for LogOracle {
        fn contract(&self) -> &crate::OracleContract {
            &self.inner.c
        }
        fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
            self.inner.matrix.symbolic()
        }
        fn residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            self.calls += 1;
            assert_ne!(self.calls, self.panic_at, "contained PETSc oracle panic");
            if self.calls == self.fail_at {
                out[0] = 99.0;
                return Err(ProblemError::Contract(
                    "original PETSc callback witness".into(),
                ));
            }
            if x[0] <= 0.0 {
                self.rejected += 1;
                return Err(pse_math::MathError::Domain {
                    source_id: self.inner.c.variables[0].id,
                    requirement: "positive logarithm argument",
                }
                .into());
            }
            out[0] = (x[0] / 0.1).ln();
            Ok(())
        }
        fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            out[0] = 1.0 / x[0];
            Ok(())
        }
        fn jacobian_product(
            &mut self,
            x: &[f64],
            direction: &[f64],
            out: &mut [f64],
        ) -> Result<(), ProblemError> {
            out[0] = direction[0] / x[0];
            Ok(())
        }
    }
    fn execution(controls: &Controls) -> Execution {
        Execution::new(Arc::new(AtomicBool::new(false)), controls)
    }
    fn run(
        oracle: &mut LogOracle,
        start: f64,
        controls: &Controls,
        warm: Option<&WarmStart>,
    ) -> Result<SolveReport, ProblemError> {
        let tolerance = Tolerances {
            variables: vec![1e-8],
            rows: vec![1e-8],
            integrality: 1e-8,
        };
        let accuracy = ResolvedAccuracy {
            native_scaling: false,
            ..ResolvedAccuracy::nominal()
        };
        let settings = Settings {
            trust: crate::settings::petsc::TrustRegion {
                initial: 2.0,
                ..Default::default()
            },
            ..Default::default()
        };
        solve(
            oracle,
            &[start],
            &settings,
            SolveRequest {
                controls,
                accuracy: &accuracy,
                execution: execution(controls),
                tolerances: &tolerance,
                warm,
                compatibility: &crate::solver_tests::stamp(Backend::Petsc),
            },
        )
    }
    #[test]
    fn tagged_native_errors_preserve_categories_status_and_report_owner() {
        use pse_diagnostics::DiagnosticStage;
        use pse_model::diagnostic::{BoundaryClass, DiagnosticProjection, Observation};
        let controls = Controls::default();
        let oracle = LogOracle::new();
        // The constants come from the compiled pinned headers, never copied numeric values.
        // SAFETY: These immutable tagged constants are exported by the checked bridge built against the pinned PETSc headers.
        let cases = unsafe {
            [
                (
                    ffi::PSE_PETSC_ERR_MEM,
                    NativeFailureKind::Resource,
                    BoundaryClass::ResourceLimit,
                    Termination::ResourceExhausted,
                ),
                (
                    ffi::PSE_PETSC_ERR_ARG_SIZ,
                    NativeFailureKind::Contract,
                    BoundaryClass::InvalidModel,
                    Termination::Invalid,
                ),
                (
                    ffi::PSE_PETSC_ERR_ORDER,
                    NativeFailureKind::Contract,
                    BoundaryClass::InvalidModel,
                    Termination::Invalid,
                ),
                (
                    ffi::PSE_PETSC_ERR_SYS,
                    NativeFailureKind::Infrastructure,
                    BoundaryClass::Infrastructure,
                    Termination::Invalid,
                ),
                (
                    ffi::PSE_PETSC_ERR_LIB,
                    NativeFailureKind::Infrastructure,
                    BoundaryClass::Infrastructure,
                    Termination::Invalid,
                ),
                (
                    ffi::PSE_PETSC_ERR_PLIB,
                    NativeFailureKind::Infrastructure,
                    BoundaryClass::Infrastructure,
                    Termination::Invalid,
                ),
                (
                    ffi::PSE_PETSC_ERR_MEMC,
                    NativeFailureKind::Infrastructure,
                    BoundaryClass::Infrastructure,
                    Termination::Invalid,
                ),
                (
                    ffi::PSE_PETSC_ERR_USER,
                    NativeFailureKind::Infrastructure,
                    BoundaryClass::Infrastructure,
                    Termination::Invalid,
                ),
                (
                    ffi::PSE_PETSC_ERR_FP,
                    NativeFailureKind::Numerical,
                    BoundaryClass::Numerical,
                    Termination::Numerical,
                ),
                (
                    ffi::PSE_PETSC_ERR_NOT_CONVERGED,
                    NativeFailureKind::Numerical,
                    BoundaryClass::Numerical,
                    Termination::Numerical,
                ),
                (
                    i32::MAX,
                    NativeFailureKind::Infrastructure,
                    BoundaryClass::Infrastructure,
                    Termination::Invalid,
                ),
            ]
        };
        for (code, kind, class, termination) in cases {
            let mut state = CallbackState::new(execution(&controls));
            let mut report = SolveReport::new(
                Backend::Petsc,
                oracle.contract(),
                native_termination(2, "initial native reason".into()),
                &state.execution,
            );
            let stop = finish_native_error(&mut report, &mut state, code, "tagged native exit");
            assert_eq!(stop, kind != NativeFailureKind::Numerical);
            assert_eq!(report.termination.category, termination);
            assert_eq!(report.termination.code, i64::from(code));
            let cause = report.validation_failure().unwrap();
            let ProblemError::Native {
                status,
                kind: actual,
                ..
            } = cause
            else {
                panic!("native tagged cause erased")
            };
            assert_eq!(*actual, kind);
            assert_eq!(status.code, i64::from(code));
            assert_eq!(status.name, report.termination.name);
            let diagnostic = cause.boundary_diagnostic(DiagnosticStage::Native);
            assert_eq!(diagnostic.class, class);
            assert!(
                matches!(&diagnostic.observations["native_code"],Observation::Integer(actual) if *actual==i64::from(code))
            );
            assert!(report.quality.is_none() && report.warm_start.is_none());
            // This exact gate is consumed by root, TS and NASM before original callbacks.
            if !stop {
                assert!(matches!(
                    cause,
                    ProblemError::Native {
                        kind: NativeFailureKind::Numerical,
                        ..
                    }
                ));
            }
        }
        // Exercise a real project ABI argument failure, without requesting an allocation.
        let mut null = ptr::null_mut();
        let code = {
            // SAFETY: The serialized admission retains these live handles, and the checked-size buffers and pinned callback context outlive this call.
            unsafe { ffi::pse_petsc_vec_create(0, &mut null) }
        };
        // SAFETY: These immutable tagged constants are exported by the checked bridge built against the pinned PETSc headers.
        assert_eq!(code, unsafe { ffi::PSE_PETSC_ERR_ARG_SIZ });
        assert!(matches!(
            check(code, "actual vector creation"),
            Err(ProblemError::Native {
                kind: NativeFailureKind::Contract,
                ..
            })
        ));
    }
    #[test]
    fn native_report_terminal_callback_cause_precedes_user_wrapper() {
        let controls = Controls::default();
        let oracle = LogOracle::new();
        let mut state = CallbackState::new(execution(&controls));
        assert!(
            state
                .evaluate("actual terminal callback", || Err::<(), _>(
                    ProblemError::Contract("actual source witness".into())
                ))
                .is_none()
        );
        let mut report = SolveReport::new(
            Backend::Petsc,
            oracle.contract(),
            native_termination(0, "native wrapper".into()),
            &state.execution,
        );
        assert!(finish_native_error(
            &mut report,
            &mut state,
            // SAFETY: These immutable tagged constants are exported by the checked bridge built against the pinned PETSc headers.
            unsafe { ffi::PSE_PETSC_ERR_USER },
            "callback wrapper"
        ));
        assert_eq!(report.termination.name, "PETSC_ERR_USER");
        assert!(report.validation_failure().is_none());
        assert!(
            matches!(report.callback_failure(),Some(ProblemError::Contract(message)) if message=="actual source witness")
        );
        assert!(report.evidence.callback.terminal_failure);
    }
    #[test]
    fn trust_owner_preserves_domain_phase_causes_limits_and_terminal_lifetime() {
        let controls = Controls::default();
        let mut oracle = LogOracle::new();
        let report = run(&mut oracle, 1.0, &controls, None).unwrap();
        assert_eq!(report.termination.category, Termination::Success);
        assert!(report.quality.as_ref().unwrap().feasible());
        assert!(
            oracle.rejected > 0,
            "actual native trial must cross the logarithm domain"
        );
        assert!((report.candidate.as_ref().unwrap().primal[0] - 0.1).abs() < 1e-8);
        assert!(report.evidence.callback.trial_rejections > 0);
        assert!(!report.evidence.callback.terminal_failure);
        assert!(report.evidence.work.iterations.is_some_and(|n| n > 0));
        assert!(report.evidence.work.evaluations.is_some_and(|n| n > 0));
        assert_eq!(report.evidence.work.factorizations, None);
        assert!(
            !report.metrics.keys().any(|key| key.contains(".publish")),
            "publication is part of one counted callback"
        );
        assert_eq!(report.provenance["build"], build().to_hex());
        let warm = report.warm_start.unwrap();
        let mut seeded = LogOracle::new();
        let report = run(&mut seeded, 3.0, &controls, Some(&warm)).unwrap();
        assert!(report.evidence.start_submitted && report.quality.unwrap().feasible());

        let mut invalid = LogOracle::new();
        let report = run(&mut invalid, -1.0, &controls, None).unwrap();
        assert_eq!(invalid.calls, 1);
        assert_eq!(report.termination.name, "PETSC_INITIAL_DOMAIN_REFUSED");
        assert_eq!(report.termination.code, 0);
        assert_eq!(report.termination.category, Termination::Evaluation);
        assert!(matches!(
            report.callback_failure(),
            Some(ProblemError::Math(pse_math::MathError::Domain { .. }))
        ));
        assert_eq!(report.evidence.callback.trial_rejections, 0);
        assert_eq!(
            report.evidence.work.iterations, None,
            "no native trajectory means no observed iteration counter"
        );
        assert!(
            report.candidate.is_none() && report.quality.is_none() && report.warm_start.is_none()
        );

        let mut fatal = LogOracle::new();
        fatal.fail_at = 2;
        let report = run(&mut fatal, 1.0, &controls, None).unwrap();
        assert_eq!(
            fatal.calls, 2,
            "no original validation callback after terminal latch"
        );
        assert_eq!(report.termination.category, Termination::Evaluation);
        assert!(
            matches!(report.callback_failure(), Some(ProblemError::Contract(message)) if message == "original PETSc callback witness")
        );
        assert!(
            report.evidence.callback.terminal_failure
                && report.quality.is_none()
                && report.warm_start.is_none()
        );
        assert_ne!(
            report.termination.code, 0,
            "actual PETSc callback error remains native evidence"
        );
        assert_eq!(report.termination.name, "PETSC_ERR_USER");
        let mut panicking = LogOracle::new();
        panicking.panic_at = 2;
        let report = run(&mut panicking, 1.0, &controls, None).unwrap();
        assert_eq!(report.termination.category, Termination::Panic);
        assert_eq!(panicking.calls, 2);
        assert!(report.quality.is_none() && report.warm_start.is_none());

        let limited = Controls {
            iterations: 1,
            ..controls.clone()
        };
        let report = run(&mut LogOracle::new(), 1.0, &limited, None).unwrap();
        assert_eq!(report.termination.category, Termination::IterationLimit);
        assert_eq!(report.evidence.work.iterations, Some(1));
        assert_eq!(report.termination.assurance, Assurance::None);
        let required = Controls {
            reuse: ReusePolicy::RequireReuse,
            ..controls.clone()
        };
        assert!(matches!(
            run(&mut LogOracle::new(), 1.0, &required, None),
            Err(ProblemError::Unsupported(_))
        ));

        derived_tests::exercise_declared_flow_and_blocks();

        // Admission is serialized, cancellable while queued, and cannot renew the process.
        let admission = Admission::enter(&execution(&controls)).unwrap();
        assert!(matches!(shutdown(), Err(ProblemError::Unsupported(_))));
        assert!(matches!(
            Admission::enter(&execution(&controls)),
            Err(ProblemError::Unsupported(_))
        ));
        let queued = execution(&controls);
        let cancel = queued.cancel.clone();
        let (ready_tx, ready_rx) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            ready_tx.send(()).unwrap();
            Admission::enter(&queued).map(|_| ())
        });
        ready_rx.recv().unwrap();
        std::thread::sleep(Duration::from_millis(10));
        assert!(
            !worker.is_finished(),
            "other-thread admission remains queued while the owner is busy"
        );
        cancel.store(true, Ordering::Relaxed);
        assert!(matches!(
            worker.join().unwrap(),
            Err(ProblemError::Cancelled)
        ));
        drop(admission);
        shutdown().unwrap();
        shutdown().unwrap();
        let mut after_shutdown = LogOracle::new();
        assert!(matches!(
            run(&mut after_shutdown, 1.0, &controls, None),
            Err(ProblemError::Unsupported(_))
        ));
        assert_eq!(
            after_shutdown.calls, 0,
            "terminal process refusal precedes oracle callbacks"
        );
    }
}
