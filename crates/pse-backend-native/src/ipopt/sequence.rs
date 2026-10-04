// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(
    unsafe_code,
    reason = "worker-local ownership and copied buffers over the exception-shielded persistent TNLP C++ bridge"
)]
//! Same-object TNLP/application reuse; borrowed oracles are activated only for a call.
use super::{MuStrategy, Runtime, Settings, cstring, index, native_bound, termination};
use crate::{
    NlpOracle, ProblemError, ReuseRefusal,
    foreign_nlp::{self as cb, Context, Slot},
    nlp_pattern::Pattern,
    quality::{self, Tolerances},
    solve::*,
};
use pse_ipopt_sys::sequence as ffi;
use pse_math::binding::ObjectiveSense;
use std::{
    ffi::{CStr, c_char},
    ptr::{self, NonNull},
    sync::Arc,
};

fn text(error: &[c_char]) -> String {
    // SAFETY: the exception shield NUL-terminates this initialized caller-owned buffer.
    unsafe { CStr::from_ptr(error.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}
fn failure(code: i32, message: String) -> ProblemError {
    match code {
        ffi::INVALID => ProblemError::Contract(message),
        ffi::MEMORY => ProblemError::memory(message),
        ffi::CANCEL => ProblemError::Cancelled,
        ffi::DEADLINE => ProblemError::stopped(Termination::TimeLimit, message),
        ffi::ABANDON => ProblemError::Limit {
            kind: crate::LimitKind::Work,
            detail: message,
        },
        _ => ProblemError::internal(format!("Ipopt sequence bridge status {code}: {message}")),
    }
}
struct Handle {
    pointer: NonNull<ffi::Handle>,
    threads: usize,
}
impl Drop for Handle {
    fn drop(&mut self) {
        let _threads = crate::mkl::Threads::enter(self.threads).ok();
        let mut pointer = self.pointer.as_ptr();
        // SAFETY: this is the sole owner. Native teardown precedes callback-slot
        // destruction; the C++ boundary contains every foreign exception.
        unsafe {
            ffi::pse_ipopt_sequence_destroy(&mut pointer, ptr::null_mut(), 0);
        }
    }
}
impl Handle {
    fn option(&self, key: &str, value: &OptionValue) -> Result<(), ProblemError> {
        let key = cstring(key)?;
        let mut error: [c_char; 512] = [0; 512];
        // Every branch passes bounded caller storage and strings Ipopt copies before
        // return; this handle is exclusively owned and not yet optimized.
        let code = match value {
            // SAFETY: exclusive live handle, copied key and bounded writable error buffer.
            OptionValue::Real(value) => unsafe {
                ffi::pse_ipopt_sequence_set_double(
                    self.pointer.as_ptr(),
                    key.as_ptr(),
                    *value,
                    error.as_mut_ptr(),
                    error.len(),
                )
            },
            // SAFETY: exclusive live handle, copied key and bounded writable error buffer.
            OptionValue::Integer(value) => unsafe {
                ffi::pse_ipopt_sequence_set_integer(
                    self.pointer.as_ptr(),
                    key.as_ptr(),
                    *value,
                    error.as_mut_ptr(),
                    error.len(),
                )
            },
            OptionValue::Text(value) => {
                let value = cstring(value)?;
                // SAFETY: exclusive live handle; key/value strings and error buffer outlive the call.
                unsafe {
                    ffi::pse_ipopt_sequence_set_string(
                        self.pointer.as_ptr(),
                        key.as_ptr(),
                        value.as_ptr(),
                        error.as_mut_ptr(),
                        error.len(),
                    )
                }
            }
            OptionValue::Bool(value) => {
                let value = if *value { c"yes" } else { c"no" };
                // SAFETY: exclusive live handle; NUL-terminated key/value and error buffer are live.
                unsafe {
                    ffi::pse_ipopt_sequence_set_string(
                        self.pointer.as_ptr(),
                        key.as_ptr(),
                        value.as_ptr(),
                        error.as_mut_ptr(),
                        error.len(),
                    )
                }
            }
        };
        if code != ffi::OK {
            Err(failure(code, text(&error)))
        } else {
            Ok(())
        }
    }
}
struct Owned {
    handle: Handle,
    slot: Box<Slot>,
}
#[derive(PartialEq)]
struct Signature {
    layout: pse_ids::ContentHash,
    profile: pse_ids::ContentHash,
    sense: ObjectiveSense,
    jac: Pattern,
    hess: Pattern,
    bounds: Vec<u64>,
    options: Options,
}
/// An optional compatible persistent C++ TNLP and application, confined to its worker.
#[derive(Default)]
pub struct Sequence {
    owned: Option<Owned>,
    signature: Option<Signature>,
}
impl std::fmt::Debug for Sequence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IpoptSequence")
            .field("initialized", &self.owned.is_some())
            .finish_non_exhaustive()
    }
}
impl Sequence {
    /// Empty native owner; no foreign allocation until admitted execution.
    pub fn new() -> Self {
        Self::default()
    }
    fn clear(&mut self) {
        self.owned.take();
        self.signature.take();
    }
    /// Execute an admitted oracle on the same retained TNLP when its complete native
    /// contract matches; rebuilding remains independent of numerical start policy.
    ///
    /// # Errors
    /// Refused profile/start/reuse, scope stop, or attributable foreign failure.
    #[expect(
        clippy::too_many_arguments,
        reason = "native attempt owners match the established execution interface"
    )]
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
        let result = (|| {
            controls.validate()?;
            accuracy.validate()?;
            execution.check()?;
            super::admit(settings, controls.threads, &Runtime::observe())?;
            if compatibility.backend != Backend::Ipopt {
                return Err(ProblemError::Contract(
                    "Ipopt sequence compatibility backend".into(),
                ));
            }
            if oracle.normalization().is_some() {
                return Err(ProblemError::internal(
                    "NLP normalization must be transported before native execution",
                ));
            }
            let supplied = controls.hessian != HessianMode::LimitedMemory;
            crate::validate_nlp(
                oracle,
                if supplied {
                    pse_kernels::DerivativeOrder::Second
                } else {
                    pse_kernels::DerivativeOrder::First
                },
            )?;
            let (n, m) = (
                oracle.contract().variables.len(),
                oracle.contract().rows.len(),
            );
            let (native_n, native_m) = (index(n)?, index(m)?);
            tolerances.validate(n, m)?;
            if initial.len() != n {
                return Err(ProblemError::Contract("Ipopt initial dimensions".into()));
            }
            cb::finite(initial)?;
            let jac = Pattern::new(oracle.jacobian_pattern(), false)?;
            let hess = if supplied {
                Pattern::new(
                    oracle.hessian_pattern().ok_or_else(|| {
                        ProblemError::Unsupported("Ipopt Hessian unavailable".into())
                    })?,
                    true,
                )?
            } else {
                Pattern {
                    rows: vec![],
                    columns: vec![],
                }
            };
            let lower: Vec<_> = oracle
                .contract()
                .variables
                .iter()
                .map(|v| native_bound(v.lower))
                .collect::<Result<_, _>>()?;
            let upper: Vec<_> = oracle
                .contract()
                .variables
                .iter()
                .map(|v| native_bound(v.upper))
                .collect::<Result<_, _>>()?;
            let row_lower: Vec<_> = oracle
                .constraint_bounds()
                .iter()
                .map(|v| native_bound(v.0))
                .collect::<Result<_, _>>()?;
            let row_upper: Vec<_> = oracle
                .constraint_bounds()
                .iter()
                .map(|v| native_bound(v.1))
                .collect::<Result<_, _>>()?;
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
            reject_reserved(&controls.options, &super::settings::RESERVED)?;
            reject_reserved(&controls.options, &RESTART_OPTIONS)?;
            let mut options = controls.options.clone();
            options.extend(accuracy.ipopt_options());
            options.extend(super::settings::options(settings));
            options.extend([
                (
                    "max_iter".into(),
                    OptionValue::Integer(controls.iterations as i32),
                ),
                (
                    "hessian_approximation".into(),
                    OptionValue::Text(if supplied { "exact" } else { "limited-memory" }.into()),
                ),
                (
                    "nlp_lower_bound_inf".into(),
                    OptionValue::Real(-super::INFINITY),
                ),
                (
                    "nlp_upper_bound_inf".into(),
                    OptionValue::Real(super::INFINITY),
                ),
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
            for (key, value) in [
                ("grad_f_constant", facts.gradient_constant),
                ("jac_c_constant", facts.jacobian_constant),
                ("jac_d_constant", facts.jacobian_constant),
                (
                    "hessian_constant",
                    controls.hessian == HessianMode::Exact && facts.hessian_constant,
                ),
            ] {
                options.insert(key.into(), OptionValue::Bool(value));
            }
            options
                .entry("print_level".into())
                .or_insert(OptionValue::Integer(0));
            let structural_options = options.clone();
            let mut restart_options = Options::new();
            let mut primal = initial.to_vec();
            let mut input_duals = None;
            let mut restart = None;
            if let Some(warm) = warm {
                if controls.start == StartPolicy::NoPriorStart {
                    return Err(ProblemError::Contract(
                        "NoPriorStart forbids inherited Ipopt starts".into(),
                    ));
                }
                warm.validate(&compatibility)?;
                let WarmPayload::Nlp {
                    primal: seed,
                    bounds,
                    rows,
                    barrier,
                    working,
                } = &warm.payload
                else {
                    return Err(ProblemError::Contract("Ipopt warm payload class".into()));
                };
                if working.is_some() {
                    return Err(ProblemError::Unsupported(
                        "active working set is not an Ipopt sequence start".into(),
                    ));
                }
                if seed.len() != n {
                    return Err(ProblemError::Contract("Ipopt start dimensions".into()));
                }
                cb::finite(seed)?;
                primal.clone_from(seed);
                match (bounds, rows) {
                    (Some((zl, zu)), Some(lambda)) => {
                        if zl.len() != n
                            || zu.len() != n
                            || lambda.len() != m
                            || zl.iter().chain(zu).any(|v| *v < 0.)
                        {
                            return Err(ProblemError::Contract(
                                "Ipopt initial dual dimensions/sign".into(),
                            ));
                        }
                        cb::finite(zl)?;
                        cb::finite(zu)?;
                        cb::finite(lambda)?;
                        input_duals = Some((zl.as_slice(), zu.as_slice(), lambda.as_slice()));
                        let (applied, receipt) = settings
                            .restart
                            .apply(*barrier, settings.mu_strategy == MuStrategy::Monotone);
                        restart_options = applied;
                        options.extend(restart_options.clone());
                        restart = Some(receipt);
                    }
                    (None, None) => {}
                    _ => return Err(ProblemError::Unsupported("partial Ipopt dual start".into())),
                }
            }
            // A finite supplied start may lie outside presolve-tightened bounds;
            // Ipopt owns its bound push, while original qualification remains intact.
            let signature = Signature {
                layout: compatibility.layout,
                profile: compatibility.profile,
                sense,
                jac: jac.clone(),
                hess: hess.clone(),
                bounds: lower
                    .iter()
                    .chain(&upper)
                    .chain(&row_lower)
                    .chain(&row_upper)
                    .map(|v| v.to_bits())
                    .collect(),
                options: structural_options.clone(),
            };
            let reused = self.owned.is_some()
                && self.signature.as_ref() == Some(&signature)
                && controls.reuse != ReusePolicy::Fresh;
            if !reused {
                if self.owned.is_some() && controls.reuse == ReusePolicy::RequireReuse {
                    return Err(ProblemError::Reuse {
                        backend: Backend::Ipopt,
                        refusal: ReuseRefusal::Structure,
                    });
                }
                self.clear();
                let slot = Box::<Slot>::default();
                let callbacks = ffi::Callbacks {
                    objective: Some(cb::objective),
                    gradient: Some(cb::gradient),
                    constraints: Some(cb::constraints),
                    jacobian: Some(cb::jacobian),
                    hessian: Some(cb::hessian),
                    poll: Some(cb::poll),
                    iteration: Some(cb::iteration),
                    user: slot.data(),
                };
                let mut pointer = ptr::null_mut();
                let mut error: [c_char; 512] = [0; 512];
                let _threads = crate::mkl::Threads::enter(controls.threads)?;
                // SAFETY: canonical fixed structure and bounds are copied; callback
                // slot is stable owned storage, not the borrowed oracle.
                let code = unsafe {
                    ffi::pse_ipopt_sequence_create(
                        native_n,
                        native_m,
                        index(jac.rows.len())?,
                        index(hess.rows.len())?,
                        lower.as_ptr(),
                        upper.as_ptr(),
                        row_lower.as_ptr(),
                        row_upper.as_ptr(),
                        jac.rows.as_ptr(),
                        jac.columns.as_ptr(),
                        hess.rows.as_ptr(),
                        hess.columns.as_ptr(),
                        &callbacks,
                        &mut pointer,
                        error.as_mut_ptr(),
                        error.len(),
                    )
                };
                if code != ffi::OK {
                    return Err(failure(code, text(&error)));
                }
                let handle = Handle {
                    pointer: NonNull::new(pointer).ok_or_else(|| {
                        ProblemError::internal("Ipopt created a null TNLP handle")
                    })?,
                    threads: controls.threads,
                };
                for (key, value) in &structural_options {
                    handle.option(key, value)?;
                }
                self.owned = Some(Owned { handle, slot });
                self.signature = Some(signature);
            }
            let owned = self
                .owned
                .as_mut()
                .ok_or_else(|| ProblemError::internal("lost Ipopt sequence owner"))?;
            let mut restart_error: [c_char; 512] = [0; 512];
            // SAFETY: exclusive live handle and bounded writable error storage;
            // resetting only per-attempt restart options leaves the TNLP immutable.
            let restart_code = unsafe {
                ffi::pse_ipopt_sequence_reset_restart(
                    owned.handle.pointer.as_ptr(),
                    restart_error.as_mut_ptr(),
                    restart_error.len(),
                )
            };
            if restart_code != ffi::OK {
                return Err(failure(restart_code, text(&restart_error)));
            }
            for (key, value) in &restart_options {
                owned.handle.option(key, value)?;
            }
            let mut context = Context::new(oracle, execution, jac, hess);
            let seconds = cb::remaining(&context.state.execution)?;
            let (zl, zu, lambda) = input_duals.map_or(
                (ptr::null(), ptr::null(), ptr::null()),
                |(zl, zu, lambda)| (zl.as_ptr(), zu.as_ptr(), lambda.as_ptr()),
            );
            let mut error: [c_char; 512] = [0; 512];
            let _threads = crate::mkl::Threads::enter(controls.threads)?;
            let code = owned.slot.with_active(&mut context, || {
                // SAFETY: only this call activates the borrowed oracle. All starts
                // have checked dimensions and remain immutable until native return.
                unsafe {
                    ffi::pse_ipopt_sequence_solve(
                        owned.handle.pointer.as_ptr(),
                        i32::from(reused),
                        seconds,
                        native_n,
                        primal.as_ptr(),
                        zl,
                        zu,
                        native_m,
                        lambda,
                        error.as_mut_ptr(),
                        error.len(),
                    )
                }
            });
            if !matches!(
                code,
                ffi::OK | ffi::TERMINAL | ffi::CANCEL | ffi::DEADLINE | ffi::ABANDON | ffi::MEMORY
            ) {
                return Err(failure(code, text(&error)));
            }
            let mut result = ffi::Result::default();
            // Preserve a real Ipopt return even if its internally caught callback
            // exception caused our bridge terminal latch to veto candidate access.
            // SAFETY: exclusively owned live bridge handle and separate bounded result/error outputs.
            let native_status = unsafe {
                ffi::pse_ipopt_sequence_get_result(
                    owned.handle.pointer.as_ptr(),
                    &mut result,
                    error.as_mut_ptr(),
                    error.len(),
                )
            };
            let native_known = match native_status {
                ffi::OK => true,
                ffi::NO_RESULT if code != ffi::OK => false,
                _ => return Err(failure(native_status, text(&error))),
            };
            let native = if native_known {
                termination(result.application_status)
            } else {
                NativeTermination {
                    code: i64::from(code),
                    name: "Ipopt_sequence_bridge_stop".into(),
                    message: Some("foreign boundary stopped before native return".into()),
                    category: match code {
                        ffi::CANCEL => Termination::Cancelled,
                        ffi::DEADLINE => Termination::TimeLimit,
                        ffi::ABANDON => Termination::Limit,
                        ffi::MEMORY => Termination::ResourceExhausted,
                        _ => Termination::Evaluation,
                    },
                    assurance: Assurance::None,
                }
            };
            let mut report = SolveReport::new(
                Backend::Ipopt,
                context.oracle.contract(),
                native,
                &context.state.execution,
            );
            report.options = options;
            report.evidence.reused_native_state = reused;
            report.evidence.start_submitted = warm.is_some();
            report.evidence.restart = restart;
            report
                .metrics
                .insert("reuse.native_model".into(), Metric::Bool(reused));
            report.metrics.insert(
                "reuse.reoptimize_tnlp".into(),
                Metric::Bool(native_known && result.reoptimized == 1),
            );
            if native_known && result.iterations_available == 1 {
                report.evidence.work.iterations = u64::try_from(result.iterations).ok();
            }
            let build = super::build();
            report
                .provenance
                .insert("native".into(), build.ipopt.clone());
            report.provenance.insert("blas".into(), build.mkl.clone());
            report.provenance.insert(
                "linear".into(),
                serde_json::to_string(&settings.linear)
                    .map_err(|error| ProblemError::internal(error.to_string()))?,
            );
            report.provenance.insert("duals".into(),"minimization L=f+lambda*g-zL*x+zU*x; same TNLP/application; authored objective recovered once".into());
            report.options.insert(
                "warm_start_same_structure".into(),
                OptionValue::Bool(reused),
            );
            report.options.insert(
                "warm_start_init_point".into(),
                OptionValue::Bool(input_duals.is_some()),
            );
            report
                .options
                .insert("max_wall_time".into(), OptionValue::Real(seconds));
            report
                .options
                .insert("option_file_name".into(), OptionValue::Text(String::new()));
            if native_known && result.barrier_available == 1 {
                report
                    .metrics
                    .insert("barrier.final".into(), Metric::Real(result.barrier));
            }
            if code != ffi::OK && context.state.terminal.is_none() {
                let category = match code {
                    ffi::CANCEL => Termination::Cancelled,
                    ffi::DEADLINE => Termination::TimeLimit,
                    ffi::ABANDON => Termination::Limit,
                    ffi::MEMORY => Termination::ResourceExhausted,
                    _ => Termination::Evaluation,
                };
                context.state.terminal =
                    Some((category, "foreign Ipopt sequence scope stopped".into()));
                context.state.last_failure =
                    Some(failure(code, "foreign Ipopt sequence scope stopped".into()));
            }
            context.state.finish(&mut report);
            if code == ffi::OK
                && result.candidate_available == 1
                && !report.evidence.callback.terminal_failure
            {
                let mut lower_dual = vec![f64::NAN; n];
                let mut upper_dual = vec![f64::NAN; n];
                let mut rows = vec![f64::NAN; m];
                let mut row_dual = vec![f64::NAN; m];
                // SAFETY: every output is exact admitted capacity and copied; no
                // borrowed foreign storage survives the call.
                let status = unsafe {
                    ffi::pse_ipopt_sequence_get_vectors(
                        owned.handle.pointer.as_ptr(),
                        native_n,
                        primal.as_mut_ptr(),
                        lower_dual.as_mut_ptr(),
                        upper_dual.as_mut_ptr(),
                        native_m,
                        rows.as_mut_ptr(),
                        row_dual.as_mut_ptr(),
                        error.as_mut_ptr(),
                        error.len(),
                    )
                };
                if status != ffi::OK {
                    return Err(failure(status, text(&error)));
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
                        row_dual: duals.then(|| row_dual.clone()),
                        bound_dual: duals.then(|| (lower_dual.clone(), upper_dual.clone())),
                        reduced_costs: None,
                        slacks: None,
                        commitment: None,
                    });
                    match quality::contained(|| quality::nlp(context.oracle, &primal, tolerances)) {
                        Ok(q) => report.quality = Some(q),
                        Err(cause) => report.record_validation_failure(cause),
                    }
                    report.warm_start = Some(WarmStart {
                        origin: None,
                        compatibility,
                        payload: WarmPayload::Nlp {
                            primal,
                            bounds: duals.then_some((lower_dual, upper_dual)),
                            rows: duals.then_some(row_dual),
                            barrier: (duals
                                && result.barrier_available == 1
                                && result.barrier.is_finite()
                                && result.barrier > 0.)
                                .then_some(result.barrier),
                            working: None,
                        },
                    });
                }
            }
            if report.callback_failure.is_none()
                && !matches!(
                    report.termination.category,
                    Termination::Success | Termination::Acceptable
                )
                && let Some(cause) = context.state.last_failure.take()
            {
                report.callback_failure = Some(Arc::new(cause));
            }
            // The foreign sequence admits further ReOptimizeTNLP only after a
            // successful/acceptable actual native solve; all terminal/failed state dies.
            if context.state.execution.stopped().is_some() {
                context.state.finish(&mut report);
                report.candidate = None;
                report.warm_start = None;
            }
            if code != ffi::OK
                || !matches!(result.application_status, 0 | 1)
                || report.evidence.callback.terminal_failure
            {
                self.clear();
            }
            Ok(report)
        })();
        if result.is_err() {
            self.clear();
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Debug)]
    struct Scaled {
        inner: crate::solver_tests::Polynomial,
        factor: f64,
    }
    impl NlpOracle for Scaled {
        fn contract(&self) -> &crate::OracleContract {
            NlpOracle::contract(&self.inner)
        }
        fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
            NlpOracle::jacobian_pattern(&self.inner)
        }
        fn hessian_pattern(&self) -> Option<faer::sparse::SymbolicSparseColMatRef<'_, usize>> {
            NlpOracle::hessian_pattern(&self.inner)
        }
        fn constraint_bounds(&self) -> &[(f64, f64)] {
            NlpOracle::constraint_bounds(&self.inner)
        }
        fn objective(&mut self, x: &[f64]) -> Result<f64, ProblemError> {
            Ok(self.factor * self.inner.objective(x)?)
        }
        fn gradient(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            self.inner.gradient(x, out)?;
            for value in out {
                *value *= self.factor;
            }
            Ok(())
        }
        fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            self.inner.constraints(x, out)
        }
        fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            NlpOracle::jacobian(&mut self.inner, x, out)
        }
        fn hessian(
            &mut self,
            x: &[f64],
            w: f64,
            l: &[f64],
            out: &mut [f64],
        ) -> Result<(), ProblemError> {
            self.inner.hessian(x, w * self.factor, l, out)
        }
    }
    fn tolerance() -> Tolerances {
        Tolerances {
            variables: vec![1e-7],
            rows: vec![1e-7],
            integrality: 1e-7,
        }
    }
    #[test]
    fn persistent_tnlp_reuses_real_application_with_fresh_borrowed_numeric_oracle() {
        let controls = Controls {
            reuse: ReusePolicy::AllowRebuild,
            time_limit: std::time::Duration::from_secs(5),
            iterations: 50,
            ..Controls::default()
        };
        let mut sequence = Sequence::new();
        let mut first = Scaled {
            inner: crate::solver_tests::Polynomial::new(),
            factor: 2.,
        };
        let first_report = sequence
            .solve(
                &mut first,
                &[1.],
                ObjectiveSense::Minimize,
                &controls,
                &ResolvedAccuracy::nominal(),
                &Settings::default(),
                Execution::new(Arc::default(), &controls),
                &tolerance(),
                None,
                crate::solver_tests::stamp(Backend::Ipopt),
            )
            .unwrap();
        assert_eq!(
            first_report.termination.category,
            Termination::Success,
            "{first_report:?}"
        );
        assert!(!first_report.evidence.reused_native_state);
        assert!((first_report.candidate.unwrap().objective.unwrap() - 2.).abs() < 1e-6);
        drop(first);
        let mut second = Scaled {
            inner: crate::solver_tests::Polynomial::new(),
            factor: 4.,
        };
        let mut stamp = crate::solver_tests::stamp(Backend::Ipopt);
        stamp.data = pse_ids::ContentHash::from_bytes([7; 32]);
        let second_report = sequence
            .solve(
                &mut second,
                &[2.],
                ObjectiveSense::Minimize,
                &controls,
                &ResolvedAccuracy::nominal(),
                &Settings::default(),
                Execution::new(Arc::default(), &controls),
                &tolerance(),
                None,
                stamp,
            )
            .unwrap();
        assert_eq!(second_report.termination.category, Termination::Success);
        assert!(second_report.evidence.reused_native_state);
        assert!(matches!(
            second_report.metrics.get("reuse.reoptimize_tnlp"),
            Some(Metric::Bool(true))
        ));
        assert!((second_report.candidate.unwrap().objective.unwrap() - 4.).abs() < 1e-6);
        assert!(second_report.evidence.work.iterations.is_some());
        assert!(
            second_report
                .events
                .iter()
                .any(|event| event.phase == "ipopt.iteration")
        );
        let first_iteration = second_report
            .events
            .iter()
            .find(|event| event.phase == "ipopt.iteration")
            .unwrap();
        assert_eq!(
            first_iteration
                .values
                .get("iterate.normalized.infinity_norm"),
            Some(&Metric::Real(2.))
        );
        super::super::runtime::tests::assert_single_provider();
    }
    #[test]
    fn persistent_tnlp_changes_restart_data_without_rebuilding_or_retaining_stale_options() {
        let controls = Controls {
            reuse: ReusePolicy::RequireReuse,
            start: StartPolicy::PreviousAccepted,
            time_limit: std::time::Duration::from_secs(5),
            iterations: 50,
            ..Controls::default()
        };
        let mut sequence = Sequence::new();
        let mut oracle = crate::solver_tests::Polynomial::new();
        oracle.c.variables[0].lower = 0.5;
        oracle.c.variables[0].upper = 3.;
        let mut seed: Option<WarmStart> = None;
        for (attempt, barrier) in [None, Some(1e-5), Some(1e-7), None].into_iter().enumerate() {
            let warm = barrier.map(|barrier| {
                let mut warm = seed.clone().expect("prior qualified native seed");
                let WarmPayload::Nlp {
                    barrier: value,
                    bounds,
                    rows,
                    ..
                } = &mut warm.payload
                else {
                    panic!("native NLP seed")
                };
                assert!(bounds.is_some() && rows.is_some());
                *value = Some(barrier);
                warm
            });
            // Ipopt can push a finite supplied start into the native box. This
            // mirrors an original start outside bounds tightened by presolve.
            let report = sequence
                .solve(
                    &mut oracle,
                    &[0.],
                    ObjectiveSense::Minimize,
                    &controls,
                    &ResolvedAccuracy::nominal(),
                    &Settings::default(),
                    Execution::new(Arc::default(), &controls),
                    &tolerance(),
                    warm.as_ref(),
                    crate::solver_tests::stamp(Backend::Ipopt),
                )
                .unwrap();
            assert_eq!(
                report.termination.category,
                Termination::Success,
                "{report:?}"
            );
            assert_eq!(report.evidence.reused_native_state, attempt > 0);
            assert_eq!(
                report.options.get("warm_start_init_point"),
                Some(&OptionValue::Bool(barrier.is_some()))
            );
            match barrier {
                Some(value) => {
                    assert_eq!(
                        report.options.get("mu_init"),
                        Some(&OptionValue::Real(value))
                    );
                    assert_eq!(
                        report.evidence.restart.as_ref().unwrap().mu_init,
                        Some(value)
                    );
                }
                None => {
                    assert!(report.evidence.restart.is_none());
                    assert!(
                        RESTART_OPTIONS
                            .iter()
                            .all(|key| !report.options.contains_key(*key))
                    );
                }
            }
            assert!((report.candidate.as_ref().unwrap().primal[0] - 1.).abs() < 1e-6);
            seed = report.warm_start;
        }
    }
    #[test]
    fn persistent_profile_change_refuses_required_reuse_and_discards_native_state() {
        let mut controls = Controls {
            reuse: ReusePolicy::AllowRebuild,
            time_limit: std::time::Duration::from_secs(5),
            iterations: 50,
            ..Controls::default()
        };
        let mut sequence = Sequence::new();
        let mut oracle = crate::solver_tests::Polynomial::new();
        sequence
            .solve(
                &mut oracle,
                &[1.],
                ObjectiveSense::Minimize,
                &controls,
                &ResolvedAccuracy::nominal(),
                &Settings::default(),
                Execution::new(Arc::default(), &controls),
                &tolerance(),
                None,
                crate::solver_tests::stamp(Backend::Ipopt),
            )
            .unwrap();
        controls.reuse = ReusePolicy::RequireReuse;
        controls.iterations = 51;
        let result = sequence.solve(
            &mut oracle,
            &[1.],
            ObjectiveSense::Minimize,
            &controls,
            &ResolvedAccuracy::nominal(),
            &Settings::default(),
            Execution::new(Arc::default(), &controls),
            &tolerance(),
            None,
            crate::solver_tests::stamp(Backend::Ipopt),
        );
        assert!(matches!(
            result,
            Err(ProblemError::Reuse {
                refusal: ReuseRefusal::Structure,
                ..
            })
        ));
        assert!(sequence.owned.is_none());
    }
    #[test]
    fn terminal_callback_preserves_original_cause_and_discards_failed_sequence() {
        let controls = Controls {
            reuse: ReusePolicy::AllowRebuild,
            time_limit: std::time::Duration::from_secs(5),
            iterations: 50,
            ..Controls::default()
        };
        let mut sequence = Sequence::new();
        let mut oracle = crate::solver_tests::Polynomial::new();
        oracle.fail = true;
        let report = sequence
            .solve(
                &mut oracle,
                &[1.],
                ObjectiveSense::Minimize,
                &controls,
                &ResolvedAccuracy::nominal(),
                &Settings::default(),
                Execution::new(Arc::default(), &controls),
                &tolerance(),
                None,
                crate::solver_tests::stamp(Backend::Ipopt),
            )
            .unwrap();
        assert_eq!(
            report.termination.category,
            Termination::Evaluation,
            "{report:?}"
        );
        assert!(report.evidence.callback.terminal_failure);
        assert!(matches!(
            report.callback_failure.as_deref(),
            Some(ProblemError::Contract(_))
        ));
        assert!(report.candidate.is_none());
        assert!(report.warm_start.is_none());
        assert!(sequence.owned.is_none());
        assert!(matches!(
            report.metrics.get("callback.gradient.calls"),
            Some(Metric::Integer(1))
        ));
        oracle.fail = false;
        let report = sequence
            .solve(
                &mut oracle,
                &[1.],
                ObjectiveSense::Minimize,
                &controls,
                &ResolvedAccuracy::nominal(),
                &Settings::default(),
                Execution::new(Arc::default(), &controls),
                &tolerance(),
                None,
                crate::solver_tests::stamp(Backend::Ipopt),
            )
            .unwrap();
        assert_eq!(report.termination.category, Termination::Success);
        assert!(!report.evidence.reused_native_state);
    }
}
