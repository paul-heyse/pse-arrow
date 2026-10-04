// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(
    unsafe_code,
    reason = "safe worker-local ownership over the exception-shielded Uno bridge"
)]
//! Explicit, first-order Uno filter trust-region profiles over the original NLP oracle.
pub use crate::settings::uno::{Method, Settings};
use crate::{
    NlpOracle, ProblemError,
    foreign_nlp::{self as cb, Context, Slot},
    nlp_pattern::Pattern,
    quality::{self, Tolerances},
    solve::*,
};
use pse_math::binding::ObjectiveSense;
use pse_uno_sys as ffi;
use std::{
    ffi::{CStr, c_char},
    ptr::{self, NonNull},
    sync::Arc,
};

/// Actual qualified native source and project bridge build identity.
pub fn build() -> pse_ids::ContentHash {
    let mut hasher = pse_ids::FramedHasher::new(pse_ids::Frame::NativeUnoBuildV1);
    hasher.str(ffi::BUILD_ID).str(ffi::VERSION);
    hasher.finish_hash()
}

/// Refuse controls not consumed by the scoped native profiles.
///
/// # Errors
/// Invalid settings or a required shared control unsupported by this binding.
pub fn admit(
    settings: &Settings,
    controls: &Controls,
    accuracy: &ResolvedAccuracy,
) -> Result<(), ProblemError> {
    settings.validate()?;
    controls.validate()?;
    accuracy.validate()?;
    if controls.threads != 1 {
        return Err(ProblemError::Unsupported(
            "scoped Uno HiGHS profiles require one thread".into(),
        ));
    }
    if controls.hessian != HessianMode::LimitedMemory {
        return Err(ProblemError::Unsupported(
            "Uno SQP/SLP profiles require the declared first-order limited-memory policy".into(),
        ));
    }
    if !controls.options.is_empty() {
        return Err(ProblemError::Contract(
            "Uno raw options are unavailable; use typed profile settings".into(),
        ));
    }
    if accuracy.native_scaling || accuracy.acceptable.is_some() {
        return Err(ProblemError::Unsupported(
            "scoped Uno profile does not admit native scaling or relaxed KKT acceptance".into(),
        ));
    }
    // SAFETY: shielded immutable library constant; no native owner or initialization.
    let minimum_tolerance = unsafe { ffi::pse_uno_minimum_tolerance() };
    if accuracy.feasibility < minimum_tolerance
        || accuracy.stationarity.min(accuracy.complementarity) < minimum_tolerance
    {
        return Err(ProblemError::Unsupported(
            "required accuracy below HiGHS 1.15.0 supported 1e-10 tolerance floor".into(),
        ));
    }
    if controls.reuse == ReusePolicy::RequireReuse {
        return Err(ProblemError::Unsupported(
            "scoped Uno bridge uses a fresh visible native attempt".into(),
        ));
    }
    Ok(())
}
fn index(value: usize) -> Result<i32, ProblemError> {
    i32::try_from(value).map_err(|_| ProblemError::Unsupported("Uno 32-bit index extent".into()))
}
fn error_text(buffer: &[c_char]) -> String {
    // SAFETY: every bridge call NUL-terminates this initialized caller-owned buffer.
    unsafe { CStr::from_ptr(buffer.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}
fn boundary_error(status: i32, message: String) -> ProblemError {
    match status {
        ffi::INVALID => ProblemError::Contract(message),
        ffi::MEMORY => ProblemError::memory(message),
        ffi::CANCELLED => ProblemError::Cancelled,
        ffi::DEADLINE => ProblemError::stopped(Termination::TimeLimit, message),
        ffi::ABANDONED => ProblemError::Limit {
            kind: crate::LimitKind::Work,
            detail: message,
        },
        _ => ProblemError::internal(format!("Uno bridge status {status}: {message}")),
    }
}
struct Handle(NonNull<ffi::Handle>);
impl Drop for Handle {
    fn drop(&mut self) {
        let mut pointer = self.0.as_ptr();
        // SAFETY: this owner destroys its handle once while callback slot and native
        // scheduler/thread scopes remain live. The C++ function shields destruction.
        unsafe {
            ffi::pse_uno_destroy(&mut pointer, ptr::null_mut(), 0);
        }
    }
}
/// Preserve both the optimization stop and native iterate assessment without claiming
/// a global certificate. Native stationarity never replaces original-space qualification.
pub fn termination(optimization: i32, iterate: i32) -> NativeTermination {
    let category = match optimization {
        0 => match iterate {
            1 => Termination::Success,
            2 => Termination::FeasibleOnly,
            3 | 5 => Termination::Infeasible,
            4 => Termination::FeasibleOnly,
            6 => Termination::Numerical,
            7 => Termination::Unbounded,
            _ => Termination::Inconclusive,
        },
        1 => Termination::IterationLimit,
        2 => Termination::TimeLimit,
        3 => Termination::Evaluation,
        4 => Termination::Numerical,
        5 => Termination::Cancelled,
        _ => Termination::Inconclusive,
    };
    NativeTermination {
        code: i64::from(optimization),
        name: format!("Uno_optimization_{optimization}_iterate_{iterate}"),
        message: None,
        category,
        assurance: Assurance::None,
    }
}
fn bridge_termination(status: i32, message: String) -> NativeTermination {
    NativeTermination {
        code: i64::from(status),
        name: "Uno_bridge_stop".into(),
        message: Some(message),
        category: match status {
            ffi::CANCELLED => Termination::Cancelled,
            ffi::DEADLINE => Termination::TimeLimit,
            ffi::ABANDONED => Termination::Limit,
            ffi::MEMORY => Termination::ResourceExhausted,
            _ => Termination::Evaluation,
        },
        assurance: Assurance::None,
    }
}
fn string_option(handle: &Handle, key: &CStr) -> Result<String, ProblemError> {
    let mut value: [c_char; 256] = [0; 256];
    let mut error: [c_char; 512] = [0; 512];
    // SAFETY: live owned handle and bounded caller buffers, with NUL-terminated key.
    let status = unsafe {
        ffi::pse_uno_get_string_option(
            handle.0.as_ptr(),
            key.as_ptr(),
            value.as_mut_ptr(),
            value.len(),
            error.as_mut_ptr(),
            error.len(),
        )
    };
    if status != ffi::OK {
        return Err(boundary_error(status, error_text(&error)));
    }
    Ok(error_text(&value))
}
fn integer_option(handle: &Handle, key: &CStr) -> Result<i32, ProblemError> {
    let mut value = 0;
    let mut error: [c_char; 512] = [0; 512];
    // SAFETY: live owned handle, NUL-terminated key and bounded caller outputs.
    let status = unsafe {
        ffi::pse_uno_get_integer_option(
            handle.0.as_ptr(),
            key.as_ptr(),
            &mut value,
            error.as_mut_ptr(),
            error.len(),
        )
    };
    if status != ffi::OK {
        return Err(boundary_error(status, error_text(&error)));
    }
    Ok(value)
}
/// Execute one admitted visible attempt; native objects never escape their scopes.
///
/// # Errors
/// A refused request or attributable foreign failure; callback causes remain in reports.
#[expect(
    clippy::too_many_arguments,
    reason = "one native attempt receives its already admitted independent owners"
)]
pub fn solve(
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
    admit(settings, controls, accuracy)?;
    execution.check()?;
    if compatibility.backend != Backend::Uno {
        return Err(ProblemError::Contract("Uno compatibility backend".into()));
    }
    if oracle.normalization().is_some() {
        return Err(ProblemError::internal(
            "NLP normalization must be transported before Uno execution",
        ));
    }
    crate::validate_nlp(oracle, pse_kernels::DerivativeOrder::First)?;
    let (n, m) = (
        oracle.contract().variables.len(),
        oracle.contract().rows.len(),
    );
    let native_n = index(n)?;
    tolerances.validate(n, m)?;
    if initial.len() != n {
        return Err(ProblemError::Contract("Uno initial dimensions".into()));
    }
    cb::finite(initial)?;
    let mut primal = initial.to_vec();
    if let Some(warm) = warm {
        if controls.start == StartPolicy::NoPriorStart {
            return Err(ProblemError::Contract(
                "NoPriorStart forbids a Uno inherited seed".into(),
            ));
        }
        warm.validate(&compatibility)?;
        let WarmPayload::Nlp {
            primal: seed,
            bounds: None,
            rows: None,
            barrier: None,
            working: None,
        } = &warm.payload
        else {
            return Err(ProblemError::Unsupported(
                "scoped Uno native start is primal only".into(),
            ));
        };
        if seed.len() != n {
            return Err(ProblemError::Contract("Uno start dimensions".into()));
        }
        cb::finite(seed)?;
        primal.clone_from(seed);
    }
    let jac = Pattern::new(oracle.jacobian_pattern(), false)?;
    let lower: Vec<_> = oracle
        .contract()
        .variables
        .iter()
        .map(|v| v.lower)
        .collect();
    let upper: Vec<_> = oracle
        .contract()
        .variables
        .iter()
        .map(|v| v.upper)
        .collect();
    let row_lower: Vec<_> = oracle.constraint_bounds().iter().map(|v| v.0).collect();
    let row_upper: Vec<_> = oracle.constraint_bounds().iter().map(|v| v.1).collect();
    if primal
        .iter()
        .zip(&lower)
        .zip(&upper)
        .any(|((x, l), u)| x < l || x > u)
    {
        return Err(ProblemError::Contract(
            "Uno explicit start outside original bounds".into(),
        ));
    }
    // Retained HiGHS owners must have been cleared by the execution adapter before
    // this exclusive process-global scheduler scope is acquired.
    let _scheduler = crate::highs_lifecycle::exclusive(&execution)?;
    let _threads = crate::mkl::Threads::enter(1)?;
    let slot = Box::<Slot>::default();
    let callbacks = ffi::Callbacks {
        objective: Some(cb::objective),
        constraints: Some(cb::constraints),
        gradient: Some(cb::gradient),
        jacobian: Some(cb::jacobian),
        poll: Some(cb::poll),
        user: slot.data(),
    };
    let config = ffi::Config {
        profile: match settings.method {
            Method::Sqp => ffi::SQP_LBFGS,
            Method::Slp => ffi::SLP,
        },
        max_iterations: index(controls.iterations as usize)?,
        lbfgs_memory: index(settings.limited_memory)?,
        remaining_seconds: cb::remaining(&execution)?,
        primal_tolerance: accuracy.feasibility,
        dual_tolerance: accuracy.stationarity.min(accuracy.complementarity),
        trust_radius: settings.trust_radius,
        materialization_limit_bytes: if settings.method == Method::Sqp {
            u64::try_from(execution.memory.ok_or_else(|| {
                ProblemError::Contract(
                    "Uno SQP requires an admitted finite foreign memory allowance".into(),
                )
            })?)
            .map_err(|_| ProblemError::memory("Uno foreign allowance extent"))?
        } else {
            0
        },
    };
    let mut error: [c_char; 512] = [0; 512];
    let mut pointer = ptr::null_mut();
    // SAFETY: exact admitted extents, canonical COO, stable owned callback slot and
    // C++-copied arrays; create is exception-shielded and writes only caller buffers.
    let status = unsafe {
        ffi::pse_uno_create(
            index(n)?,
            index(m)?,
            index(jac.rows.len())?,
            lower.as_ptr(),
            upper.as_ptr(),
            row_lower.as_ptr(),
            row_upper.as_ptr(),
            jac.rows.as_ptr(),
            jac.columns.as_ptr(),
            &config,
            &callbacks,
            &mut pointer,
            error.as_mut_ptr(),
            error.len(),
        )
    };
    if status != ffi::OK {
        return Err(boundary_error(status, error_text(&error)));
    }
    let handle = Handle(
        NonNull::new(pointer).ok_or_else(|| ProblemError::internal("Uno created a null handle"))?,
    );
    let mut context = Context::new(
        oracle,
        execution,
        jac,
        Pattern {
            rows: vec![],
            columns: vec![],
        },
    );
    let status = slot.with_active(&mut context, || {
        // SAFETY: stable activation lends the oracle only to sequential synchronous
        // callbacks; initial is copied and the handle belongs to this worker.
        unsafe {
            ffi::pse_uno_solve(
                handle.0.as_ptr(),
                primal.as_ptr(),
                native_n,
                error.as_mut_ptr(),
                error.len(),
            )
        }
    });
    if !matches!(
        status,
        ffi::OK
            | ffi::CALLBACK_TERMINAL
            | ffi::CANCELLED
            | ffi::DEADLINE
            | ffi::ABANDONED
            | ffi::MEMORY
    ) {
        return Err(boundary_error(status, error_text(&error)));
    }
    let mut result = ffi::Result::default();
    // A terminal bridge latch can coexist with a native return Uno caught. Scalar
    // evidence remains readable; vector/candidate access stays vetoed separately.
    // SAFETY: the owned handle remains live; the bridge writes only to the initialized
    // result scalar record and the bounded caller-owned, NUL-terminated error buffer.
    let result_status = unsafe {
        ffi::pse_uno_get_result(
            handle.0.as_ptr(),
            &mut result,
            error.as_mut_ptr(),
            error.len(),
        )
    };
    let native_known = match result_status {
        ffi::OK => true,
        ffi::NO_RESULT if status != ffi::OK => false,
        _ => return Err(boundary_error(result_status, error_text(&error))),
    };
    let native = if native_known {
        termination(result.optimization_status, result.iterate_status)
    } else {
        bridge_termination(
            status,
            "foreign boundary stopped before native return".into(),
        )
    };
    let mut report = SolveReport::new(
        Backend::Uno,
        context.oracle.contract(),
        native,
        &context.state.execution,
    );
    report.evidence.start_submitted = warm.is_some();
    if native_known {
        report.metrics.insert(
            "uno.iterate_status".into(),
            Metric::Integer(i64::from(result.iterate_status)),
        );
        report.evidence.work.iterations = u64::try_from(result.iterations).ok();
        for (name, value) in [
            ("objective.calls", result.objective_evaluations),
            ("constraints.calls", result.constraint_evaluations),
            ("gradient.calls", result.gradient_evaluations),
            ("jacobian.calls", result.jacobian_evaluations),
            ("subproblems", result.subproblems),
        ] {
            report
                .metrics
                .insert(format!("uno.{name}"), Metric::Integer(i64::from(value)));
        }
        report
            .metrics
            .insert("uno.cpu_seconds".into(), Metric::Real(result.cpu_seconds));
        for (name, value) in [
            ("materialization.count", result.materialization_count),
            ("materialization.columns", result.materialization_columns),
            (
                "materialization.numeric_bytes",
                result.materialization_numeric_bytes,
            ),
        ] {
            report.metrics.insert(
                format!("uno.{name}"),
                Metric::Integer(
                    i64::try_from(value)
                        .map_err(|_| ProblemError::memory("Uno observation extent"))?,
                ),
            );
        }
        report.metrics.insert(
            "uno.highs.feasibility.calls".into(),
            Metric::Integer(
                i64::try_from(result.feasibility_calls)
                    .map_err(|_| ProblemError::memory("Uno feasibility call extent"))?,
            ),
        );
        if result.feasibility_iterations_available == 1 {
            report.metrics.insert(
                "uno.highs.feasibility.simplex_iterations".into(),
                Metric::Integer(
                    i64::try_from(result.feasibility_iterations)
                        .map_err(|_| ProblemError::memory("Uno feasibility iteration extent"))?,
                ),
            );
        }
        if result.feasibility_status_available == 1 {
            report.metrics.insert(
                "uno.highs.feasibility.status".into(),
                Metric::Integer(i64::from(result.feasibility_status)),
            );
        }
        if result.qp_hot_start >= 0 {
            report.options.insert(
                "highs.qp_allow_hot_start".into(),
                OptionValue::Bool(result.qp_hot_start == 1),
            );
        }
        if result.feasibility_primal_tolerance.is_finite() {
            report.options.insert(
                "highs.feasibility.primal_feasibility_tolerance".into(),
                OptionValue::Real(result.feasibility_primal_tolerance),
            );
        }
        report.options.insert(
            "hessian_representation".into(),
            OptionValue::Text(
                if settings.method == Method::Sqp {
                    "materialized_library_positive_operator"
                } else {
                    "zero"
                }
                .into(),
            ),
        );
        if settings.method == Method::Sqp {
            report.options.insert(
                "quasi_newton_memory_size".into(),
                OptionValue::Integer(integer_option(&handle, c"quasi_newton_memory_size")?),
            );
        }
        for key in [
            c"hessian_model",
            c"inertia_correction_strategy",
            c"globalization_mechanism",
            c"globalization_strategy",
            c"QP_solver",
            c"LP_solver",
        ] {
            report.options.insert(
                key.to_string_lossy().into_owned(),
                OptionValue::Text(string_option(&handle, key)?),
            );
        }
        report
            .options
            .insert("TR_radius".into(), OptionValue::Real(settings.trust_radius));
        report.options.insert(
            "max_iterations".into(),
            OptionValue::Integer(config.max_iterations),
        );
        report.options.insert(
            "primal_tolerance".into(),
            OptionValue::Real(config.primal_tolerance),
        );
        report.options.insert(
            "dual_tolerance".into(),
            OptionValue::Real(config.dual_tolerance),
        );
        for (key, value) in [
            (
                "highs.primal_feasibility_tolerance",
                result.highs_primal_tolerance,
            ),
            (
                "highs.dual_feasibility_tolerance",
                result.highs_dual_tolerance,
            ),
            (
                "highs.optimality_tolerance",
                result.highs_optimality_tolerance,
            ),
            (
                "highs.qp_regularization_value",
                result.highs_qp_regularization,
            ),
        ] {
            if value.is_finite() {
                report.options.insert(key.into(), OptionValue::Real(value));
            }
        }
    }
    report.provenance.insert(
        "duals".into(),
        "minimization L=f+lambda*g-zL*x+zU*x; Uno signed lower multiplier negated once".into(),
    );
    report
        .provenance
        .insert("native".into(), ffi::BUILD_ID.into());
    if status != ffi::OK && context.state.terminal.is_none() {
        // A native subproblem deadline can precede the next Rust clock poll.
        // Preserve its typed scope stop without replacing a callback's own cause.
        let stop = bridge_termination(status, "foreign Uno scope stopped".into());
        context.state.terminal = Some((stop.category, "foreign Uno scope stopped".into()));
        context.state.last_failure =
            Some(boundary_error(status, "foreign Uno scope stopped".into()));
    }
    context.state.finish(&mut report);
    if status == ffi::OK && !report.evidence.callback.terminal_failure {
        let mut lower_dual = vec![f64::NAN; n];
        let mut upper_dual = vec![f64::NAN; n];
        let mut rows = vec![f64::NAN; m];
        let mut row_dual = vec![f64::NAN; m];
        // SAFETY: every output vector has the exact admitted extent; all native
        // storage is copied before handle destruction.
        let code = unsafe {
            ffi::pse_uno_get_vectors(
                handle.0.as_ptr(),
                primal.as_mut_ptr(),
                lower_dual.as_mut_ptr(),
                upper_dual.as_mut_ptr(),
                index(n)?,
                rows.as_mut_ptr(),
                row_dual.as_mut_ptr(),
                index(m)?,
                error.as_mut_ptr(),
                error.len(),
            )
        };
        if code != ffi::OK {
            return Err(boundary_error(code, error_text(&error)));
        }
        for value in &mut lower_dual {
            *value = -*value;
        }
        if primal.iter().all(|v| v.is_finite()) && result.objective.is_finite() {
            let duals = lower_dual
                .iter()
                .chain(&upper_dual)
                .chain(&row_dual)
                .all(|v| v.is_finite())
                && lower_dual.iter().chain(&upper_dual).all(|v| *v >= 0.);
            report.candidate = Some(Candidate {
                kind: CandidateKind::FinalIterate,
                primal: primal.clone(),
                objective: Some(result.objective * sense.sign()),
                row_dual: duals.then_some(row_dual),
                bound_dual: duals.then_some((lower_dual, upper_dual)),
                reduced_costs: None,
                slacks: None,
                commitment: None,
            });
            report.warm_start = Some(WarmStart {
                origin: None,
                compatibility,
                payload: WarmPayload::Nlp {
                    primal,
                    bounds: None,
                    rows: None,
                    barrier: None,
                    working: None,
                },
            });
        }
    }
    // Preserve an available failed trial witness for an unsuccessful native stop;
    // successful recovery clears it in CallbackState::evaluate.
    if report.callback_failure.is_none()
        && !matches!(
            report.termination.category,
            Termination::Success | Termination::Acceptable
        )
        && let Some(cause) = context.state.last_failure.take()
    {
        report.callback_failure = Some(Arc::new(cause));
    }
    drop(handle); // Native subproblems die before the shared scheduler/thread permit.
    drop(_threads);
    drop(_scheduler);
    // Original-space evaluation may itself use the shared provider. All Uno/HiGHS
    // objects have already died, so this validation does not inherit their gate.
    if context.state.execution.stopped().is_some() {
        context.state.finish(&mut report);
        report.candidate = None;
        report.warm_start = None;
    }
    if let Some(candidate) = report.candidate.as_ref() {
        match quality::contained(|| quality::nlp(context.oracle, &candidate.primal, tolerances)) {
            Ok(q) => report.quality = Some(q),
            Err(cause) => report.record_validation_failure(cause),
        }
    }
    if context.state.execution.stopped().is_some() {
        context.state.finish(&mut report);
        report.candidate = None;
        report.warm_start = None;
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn accuracy() -> ResolvedAccuracy {
        let mut value = ResolvedAccuracy::nominal();
        value.native_scaling = false;
        value.acceptable = None;
        value
    }
    fn execution(controls: &Controls) -> Execution {
        let mut value = Execution::new(Arc::default(), controls);
        value.memory = Some(8 << 20);
        value
    }
    #[derive(Debug)]
    struct Bounded {
        contract: crate::OracleContract,
        jacobian: faer::sparse::SparseColMat<usize, f64>,
        fail: bool,
        cancel_validation: Option<Arc<std::sync::atomic::AtomicBool>>,
    }
    impl Bounded {
        fn new() -> Self {
            let mut contract = crate::solver_tests::contract();
            contract.rows.clear();
            contract.variables[0].lower = 1.;
            contract.variables[0].upper = 2.;
            contract.derivatives = pse_kernels::DerivativeOrder::First;
            Self {
                contract,
                jacobian: faer::sparse::SparseColMat::try_new_from_triplets(0, 1, &[]).unwrap(),
                fail: false,
                cancel_validation: None,
            }
        }
        fn coupled() -> Self {
            let mut value = Self::new();
            value.contract.variables.push(crate::Variable {
                id: crate::solver_tests::id(3),
                lower: -2.,
                upper: 2.,
            });
            value.jacobian = faer::sparse::SparseColMat::try_new_from_triplets(0, 2, &[]).unwrap();
            value
        }
    }
    impl NlpOracle for Bounded {
        fn contract(&self) -> &crate::OracleContract {
            &self.contract
        }
        fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
            self.jacobian.symbolic()
        }
        fn hessian_pattern(&self) -> Option<faer::sparse::SymbolicSparseColMatRef<'_, usize>> {
            None
        }
        fn constraint_bounds(&self) -> &[(f64, f64)] {
            &[]
        }
        fn objective(&mut self, x: &[f64]) -> Result<f64, ProblemError> {
            Ok(if x.len() == 2 {
                3. * x[0] * x[0] + 2. * x[0] * x[1] + 2. * x[1] * x[1]
            } else {
                x[0] * x[0]
            })
        }
        fn gradient(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            if x.len() == 2 {
                out[0] = 6. * x[0] + 2. * x[1];
                out[1] = 2. * x[0] + 4. * x[1];
            } else {
                out[0] = 2. * x[0];
            }
            if self.fail {
                Err(ProblemError::Contract(
                    "scripted terminal Uno gradient".into(),
                ))
            } else {
                Ok(())
            }
        }
        fn constraints(&mut self, _: &[f64], _: &mut [f64]) -> Result<(), ProblemError> {
            if let Some(cancel) = &self.cancel_validation {
                cancel.store(true, std::sync::atomic::Ordering::Release);
            }
            Ok(())
        }
        fn jacobian(&mut self, _: &[f64], _: &mut [f64]) -> Result<(), ProblemError> {
            Ok(())
        }
        fn hessian(
            &mut self,
            _: &[f64],
            _: f64,
            _: &[f64],
            _: &mut [f64],
        ) -> Result<(), ProblemError> {
            Err(ProblemError::Unsupported("first-order test Hessian".into()))
        }
    }
    fn bounded_profile(settings: Settings) {
        let controls = Controls {
            hessian: HessianMode::LimitedMemory,
            time_limit: std::time::Duration::from_secs(5),
            iterations: 50,
            ..Controls::default()
        };
        let tolerance = Tolerances {
            variables: vec![1e-7],
            rows: vec![],
            integrality: 1e-7,
        };
        let mut oracle = Bounded::new();
        let mut report = solve(
            &mut oracle,
            &[1.5],
            ObjectiveSense::Minimize,
            &controls,
            &accuracy(),
            &settings,
            execution(&controls),
            &tolerance,
            None,
            crate::solver_tests::stamp(Backend::Uno),
        )
        .unwrap();
        assert_eq!(
            report.termination.category,
            Termination::Success,
            "{report:?}"
        );
        let candidate = report.candidate.as_ref().unwrap();
        assert!((candidate.primal[0] - 1.).abs() < 1e-6);
        let (lower, upper) = candidate.bound_dual.as_ref().unwrap();
        assert!((lower[0] - 2.).abs() < 1e-5);
        assert!(upper[0].abs() < 1e-5);
        quality::attach_nlp(
            &mut report,
            &mut oracle,
            &tolerance,
            ObjectiveSense::Minimize,
        );
        let stationarity = report
            .observation
            .as_ref()
            .unwrap()
            .stationarity
            .as_ref()
            .unwrap();
        assert!(stationarity[0].abs() < 1e-5);
        assert!(report.evidence.work.iterations.is_some());
        assert!(!report.evidence.reused_native_state);
    }
    #[test]
    fn bounded_sqp_transports_signed_native_duals_into_original_kkt_convention() {
        bounded_profile(Settings::default());
    }
    #[test]
    fn bounded_slp_transports_signed_native_duals_into_original_kkt_convention() {
        bounded_profile(Settings {
            method: Method::Slp,
            limited_memory: 0,
            ..Settings::default()
        });
    }
    #[test]
    fn scoped_profile_refuses_required_unconsumed_controls() {
        let settings = Settings::default();
        let mut controls = Controls::default();
        let mut accuracy = accuracy();
        assert!(admit(&settings, &controls, &accuracy).is_err());
        controls.hessian = HessianMode::LimitedMemory;
        assert!(admit(&settings, &controls, &accuracy).is_ok());
        accuracy.native_scaling = true;
        assert!(admit(&settings, &controls, &accuracy).is_err());
        accuracy.native_scaling = false;
        accuracy.stationarity = 1e-11;
        assert!(matches!(
            admit(&settings, &controls, &accuracy),
            Err(ProblemError::Unsupported(_))
        ));
        accuracy.stationarity = 1e-8;
        controls
            .options
            .insert("TR_radius".into(), OptionValue::Real(1.));
        assert!(admit(&settings, &controls, &accuracy).is_err());
        controls.options.clear();
        controls.reuse = ReusePolicy::RequireReuse;
        assert!(admit(&settings, &controls, &accuracy).is_err());
    }
    #[test]
    fn terminal_typed_callback_vetoes_candidate_and_never_reenters_oracle() {
        let controls = Controls {
            hessian: HessianMode::LimitedMemory,
            time_limit: std::time::Duration::from_secs(5),
            iterations: 50,
            ..Controls::default()
        };
        let mut oracle = Bounded::new();
        oracle.fail = true;
        let report = solve(
            &mut oracle,
            &[1.5],
            ObjectiveSense::Minimize,
            &controls,
            &accuracy(),
            &Settings::default(),
            execution(&controls),
            &Tolerances {
                variables: vec![1e-7],
                rows: vec![],
                integrality: 1e-7,
            },
            None,
            crate::solver_tests::stamp(Backend::Uno),
        )
        .unwrap();
        assert_eq!(report.termination.category, Termination::Evaluation);
        assert!(report.evidence.callback.terminal_failure, "{report:?}");
        assert!(matches!(
            report.callback_failure.as_deref(),
            Some(ProblemError::Contract(_))
        ));
        assert!(report.candidate.is_none());
        assert!(report.warm_start.is_none());
        assert!(matches!(
            report.metrics.get("callback.gradient.calls"),
            Some(Metric::Integer(1))
        ));
    }
    #[test]
    fn cancellation_during_original_validation_vetoes_late_native_success() {
        let controls = Controls {
            hessian: HessianMode::LimitedMemory,
            time_limit: std::time::Duration::from_secs(5),
            iterations: 50,
            ..Controls::default()
        };
        let execution = execution(&controls);
        let mut oracle = Bounded::new();
        oracle.cancel_validation = Some(Arc::clone(&execution.cancel));
        // Zero-row Uno never calls the constraint oracle; this cancellation is
        // raised by the separate original-space validation after native teardown.
        let report = solve(
            &mut oracle,
            &[1.5],
            ObjectiveSense::Minimize,
            &controls,
            &accuracy(),
            &Settings::default(),
            execution,
            &Tolerances {
                variables: vec![1e-7],
                rows: vec![],
                integrality: 1e-7,
            },
            None,
            crate::solver_tests::stamp(Backend::Uno),
        )
        .unwrap();
        assert_eq!(report.termination.code, 0);
        assert_eq!(report.termination.category, Termination::Cancelled);
        assert!(matches!(
            report.callback_failure.as_deref(),
            Some(ProblemError::Cancelled)
        ));
        assert!(report.evidence.callback.terminal_failure);
        assert!(report.candidate.is_none());
        assert!(report.warm_start.is_none());
    }
    #[test]
    fn actual_operator_storage_refusal_preserves_typed_resource_failure() {
        let controls = Controls {
            hessian: HessianMode::LimitedMemory,
            time_limit: std::time::Duration::from_secs(5),
            iterations: 50,
            ..Controls::default()
        };
        let mut oracle = Bounded::new();
        let mut execution = execution(&controls);
        execution.memory = Some(1700);
        let settings = Settings {
            limited_memory: 4,
            ..Settings::default()
        };
        let report = solve(
            &mut oracle,
            &[1.5],
            ObjectiveSense::Minimize,
            &controls,
            &accuracy(),
            &settings,
            execution,
            &Tolerances {
                variables: vec![1e-7],
                rows: vec![],
                integrality: 1e-7,
            },
            None,
            crate::solver_tests::stamp(Backend::Uno),
        )
        .unwrap();
        assert_eq!(report.termination.category, Termination::ResourceExhausted);
        assert!(matches!(
            report.callback_failure.as_deref(),
            Some(ProblemError::Limit {
                kind: crate::LimitKind::Memory,
                ..
            })
        ));
        assert!(report.evidence.callback.terminal_failure);
        assert!(report.candidate.is_none());
        assert!(report.warm_start.is_none());
    }
    #[test]
    fn coupled_bound_problem_updates_and_materializes_library_curvature() {
        let controls = Controls {
            hessian: HessianMode::LimitedMemory,
            time_limit: std::time::Duration::from_secs(5),
            iterations: 50,
            ..Controls::default()
        };
        let mut oracle = Bounded::coupled();
        let tolerance = Tolerances {
            variables: vec![1e-7; 2],
            rows: vec![],
            integrality: 1e-7,
        };
        let mut report = solve(
            &mut oracle,
            &[1.5, 1.],
            ObjectiveSense::Minimize,
            &controls,
            &accuracy(),
            &Settings::default(),
            execution(&controls),
            &tolerance,
            None,
            crate::solver_tests::stamp(Backend::Uno),
        )
        .unwrap();
        assert_eq!(
            report.termination.category,
            Termination::Success,
            "{report:?}"
        );
        let candidate = report.candidate.as_ref().unwrap();
        assert!((candidate.primal[0] - 1.).abs() < 1e-5);
        assert!((candidate.primal[1] + 0.5).abs() < 1e-5);
        assert!(
            matches!(report.metrics.get("uno.materialization.count"),Some(Metric::Integer(n)) if *n>1)
        );
        assert!(
            matches!(report.metrics.get("uno.materialization.columns"),Some(Metric::Integer(n)) if *n>2)
        );
        assert_eq!(
            report.metrics.get("uno.materialization.numeric_bytes"),
            Some(&Metric::Integer(24))
        );
        assert_eq!(
            report.options.get("highs.dual_feasibility_tolerance"),
            Some(&OptionValue::Real(1e-8))
        );
        assert_eq!(
            report.options.get("highs.qp_regularization_value"),
            Some(&OptionValue::Real(0.))
        );
        assert_eq!(
            report.options.get("globalization_strategy"),
            Some(&OptionValue::Text("merit_function".into()))
        );
        quality::attach_nlp(
            &mut report,
            &mut oracle,
            &tolerance,
            ObjectiveSense::Minimize,
        );
        assert!(
            report
                .observation
                .as_ref()
                .unwrap()
                .stationarity
                .as_ref()
                .unwrap()
                .iter()
                .all(|v| v.abs() < 1e-5)
        );
    }
    #[test]
    fn constrained_profile_preserves_original_rows_and_canonical_duals() {
        let controls = Controls {
            hessian: HessianMode::LimitedMemory,
            time_limit: std::time::Duration::from_secs(5),
            iterations: 50,
            ..Controls::default()
        };
        let mut oracle = crate::solver_tests::Polynomial::new();
        let tolerance = Tolerances {
            variables: vec![1e-7],
            rows: vec![1e-7],
            integrality: 1e-7,
        };
        let mut report = solve(
            &mut oracle,
            &[1.5],
            ObjectiveSense::Minimize,
            &controls,
            &accuracy(),
            &Settings::default(),
            execution(&controls),
            &tolerance,
            None,
            crate::solver_tests::stamp(Backend::Uno),
        )
        .unwrap();
        assert_eq!(
            report.termination.category,
            Termination::Success,
            "{report:?}"
        );
        assert!(
            matches!(report.metrics.get("uno.highs.feasibility.calls"),Some(Metric::Integer(n)) if *n>1)
        );
        assert!(
            matches!(report.metrics.get("uno.highs.feasibility.simplex_iterations"),Some(Metric::Integer(n)) if *n>=0)
        );
        // HiGHS 1.15.0 kOptimal is 7; this is the LP status, not the QP/native success code.
        assert_eq!(
            report.metrics.get("uno.highs.feasibility.status"),
            Some(&Metric::Integer(7))
        );
        assert_eq!(
            report.options.get("highs.qp_allow_hot_start"),
            Some(&OptionValue::Bool(true))
        );
        assert_eq!(
            report
                .options
                .get("highs.feasibility.primal_feasibility_tolerance"),
            Some(&OptionValue::Real(1e-8))
        );
        assert_eq!(
            report.options.get("globalization_strategy"),
            Some(&OptionValue::Text("fletcher_filter_method".into()))
        );
        assert!((report.candidate.as_ref().unwrap().primal[0] - 1.).abs() < 1e-5);
        assert!(
            (report
                .candidate
                .as_ref()
                .unwrap()
                .row_dual
                .as_ref()
                .unwrap()[0]
                + 2. / 3.)
                .abs()
                < 1e-5
        );
        quality::attach_nlp(
            &mut report,
            &mut oracle,
            &tolerance,
            ObjectiveSense::Minimize,
        );
        assert!(report.quality.as_ref().unwrap().feasible());
        assert!(
            report
                .observation
                .as_ref()
                .unwrap()
                .stationarity
                .as_ref()
                .unwrap()[0]
                .abs()
                < 1e-5
        );
    }
    #[test]
    fn fritz_john_and_small_step_are_feasible_only_without_kkt_or_global_claims() {
        for iterate in [2, 4] {
            assert_eq!(termination(0, iterate).category, Termination::FeasibleOnly);
            assert_eq!(termination(0, iterate).assurance, Assurance::None);
        }
    }
}
