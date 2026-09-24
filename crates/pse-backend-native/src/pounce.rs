// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native POUNCE TNLP adapter sharing the exact NLP oracle and callback failure policy.
use crate::tnlp::{Adapter, finite};
use crate::{
    NlpOracle, ProblemError,
    callback::CallbackState,
    nlp_pattern::Pattern,
    quality::{self, Tolerances},
    solve::*,
};
use pounce_rs::{ApplicationReturnStatus, IpoptApplication, TNLP};
use pse_math::binding::ObjectiveSense;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};
/// Complete native FERAL configuration, with thread/FMA policy applied at execution.
pub type LinearSettings = pounce_feral::FeralConfig;
/// Complete bit-preserving native linear profile identity, independent of Debug text.
pub fn linear_key(c: &LinearSettings) -> pse_ids::ContentHash {
    let mut h = pse_ids::FramedHasher::new("pse.pounce.feral-profile.v1");
    for v in [c.cascade_break, c.parallel, c.static_pivoting] {
        h.u64(v.map_or(0, |v| if v { 2 } else { 1 }));
    }
    h.bool(c.fma)
        .bool(c.refine)
        .bool(c.increase_quality)
        .u64(c.refine_max_steps as u64);
    for v in [c.refine_target, c.singular_pivot_floor, c.pivtol] {
        h.u64(v.to_bits());
    }
    h.bool(c.inertia_pivot_floor.is_some())
        .u64(c.inertia_pivot_floor.map_or(0, f64::to_bits));
    h.bool(c.min_par_flops.is_some())
        .u64(c.min_par_flops.unwrap_or(0));
    use pounce_feral::{OrderingMethod as O, ScalingStrategy as S};
    match &c.ordering {
        O::Amd => {
            h.u64(0);
        }
        O::Amf => {
            h.u64(1);
        }
        O::MetisND => {
            h.u64(2);
        }
        O::ScotchND => {
            h.u64(3);
        }
        O::KahipND => {
            h.u64(4);
        }
        O::Auto => {
            h.u64(5);
        }
        O::AutoRace => {
            h.u64(6);
        }
        O::External(v) => {
            h.u64(7).u64(v.len() as u64);
            for x in v {
                h.u64(*x as u64);
            }
        }
    }
    match &c.scaling {
        S::InfNorm => {
            h.u64(0);
        }
        S::Mc64Symmetric => {
            h.u64(1);
        }
        S::Identity => {
            h.u64(2);
        }
        S::Auto => {
            h.u64(3);
        }
        S::External(v) => {
            h.u64(4).u64(v.len() as u64);
            for x in v {
                h.u64(x.to_bits());
            }
        }
    }
    h.finish_hash()
}
/// Algorithm is explicit; POUNCE never silently changes the selected problem class.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    /// Native barrier/filter NLP method.
    InteriorPoint,
    /// Native active-set sequential quadratic programming.
    ActiveSetSqp,
}
thread_local! {static ADMITTED:std::cell::Cell<usize>=const{std::cell::Cell::new(0)};}
struct Admission(usize);
impl Drop for Admission {
    fn drop(&mut self) {
        ADMITTED.with(|a| a.set(self.0));
    }
}
/// Enter a local admitted pool for the complete finite sequence, including native teardown.
pub fn with_threads<T: Send, E: From<ProblemError> + Send>(
    threads: usize,
    stack: usize,
    work: impl FnOnce() -> Result<T, E> + Send,
) -> Result<T, E> {
    if threads == 0 || stack == 0 {
        return Err(ProblemError::Contract("POUNCE pool admission".into()).into());
    }
    let run = move || {
        let _guard = Admission(ADMITTED.with(|a| a.replace(threads)));
        work()
    };
    if threads == 1 {
        return run();
    }
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .stack_size(stack)
        .build_scoped(|thread| thread.run(), |pool| pool.install(run))
        .map_err(|e| E::from(ProblemError::Contract(format!("POUNCE local pool: {e}"))))?
}
/// Worker-local native application reuse; each call still constructs native iteration
/// state. This makes no claim of retaining numeric factors across different solves.
#[derive(Default)]
pub struct Session {
    app: Option<IpoptApplication>,
    layout: Option<pse_ids::ContentHash>,
}
impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PounceSession")
            .field("layout", &self.layout)
            .finish_non_exhaustive()
    }
}
impl Session {
    /// Construct after the owning worker has received admission.
    pub fn new() -> Self {
        Self::default()
    }
    /// Execute native POUNCE over the already-preprocessed oracle. The oracle
    /// must be created on this worker; a parallel profile requires `with_threads`.
    pub fn solve(
        &mut self,
        oracle: Box<dyn NlpOracle>,
        initial: &[f64],
        sense: ObjectiveSense,
        controls: &Controls,
        method: Method,
        mut feral: pounce_feral::FeralConfig,
        execution: Execution,
        tolerances: &Tolerances,
        warm: Option<&WarmStart>,
        compatibility: Compatibility,
    ) -> Result<SolveReport, ProblemError> {
        controls.validate()?;
        let n = oracle.contract().variables.len();
        let m = oracle.contract().rows.len();
        tolerances.validate(n, m)?;
        let exact = controls.hessian == HessianMode::Exact;
        crate::validate_nlp(
            oracle.as_ref(),
            if exact {
                pse_kernels::DerivativeOrder::Second
            } else {
                pse_kernels::DerivativeOrder::First
            },
        )?;
        if initial.len() != n || oracle.constraint_bounds().len() != m {
            return Err(ProblemError::Contract("POUNCE dimensions".into()));
        }
        finite(initial)?;
        if controls.threads > 1 && ADMITTED.with(|a| a.get()) < controls.threads {
            return Err(ProblemError::Contract(
                "POUNCE must execute inside its admitted local pool".into(),
            ));
        }
        for v in oracle
            .contract()
            .variables
            .iter()
            .flat_map(|v| [v.lower, v.upper])
            .chain(oracle.constraint_bounds().iter().flat_map(|v| [v.0, v.1]))
        {
            if v.is_nan() || v.is_finite() && v.abs() >= 1e19 {
                return Err(ProblemError::Contract(
                    "POUNCE finite bound reaches native infinity threshold".into(),
                ));
            }
        }
        let jac = Pattern::new(oracle.jacobian_pattern(), false)?;
        let hess = if exact {
            Pattern::new(
                oracle.hessian_pattern().ok_or_else(|| {
                    ProblemError::Contract("POUNCE exact Hessian unavailable".into())
                })?,
                true,
            )?
        } else {
            Pattern {
                rows: vec![],
                columns: vec![],
            }
        };
        let mut initial = initial.to_vec();
        let mut duals = None;
        let mut sqp_seed = None;
        if let Some(w) = warm {
            w.validate(&compatibility)?;
            match &w.payload {
                WarmPayload::Nlp {
                    primal,
                    bounds,
                    rows,
                } => {
                    if primal.len() != n {
                        return Err(ProblemError::Contract("POUNCE seed shape".into()));
                    }
                    finite(primal)?;
                    initial.clone_from(primal);
                    if let (Some((l, u)), Some(r)) = (bounds, rows) {
                        if l.len() != n
                            || u.len() != n
                            || r.len() != m
                            || l.iter().chain(u).any(|v| *v < 0.0)
                        {
                            return Err(ProblemError::Contract(
                                "POUNCE dual seed shape/sign".into(),
                            ));
                        }
                        finite(l)?;
                        finite(u)?;
                        finite(r)?;
                        duals = Some((l.clone(), u.clone(), r.clone()));
                    } else if bounds.is_some() || rows.is_some() {
                        return Err(ProblemError::Contract("partial POUNCE dual seed".into()));
                    }
                }
                WarmPayload::PounceSqp(s) if method == Method::ActiveSetSqp => {
                    if s.n() != n || s.m() != m {
                        return Err(ProblemError::Contract("POUNCE SQP seed shape".into()));
                    }
                    finite(&s.x)?;
                    finite(&s.lambda_g)?;
                    finite(&s.lambda_x)?;
                    initial.clone_from(&s.x);
                    sqp_seed = Some(s.clone());
                }
                _ => return Err(ProblemError::Contract("POUNCE seed class".into())),
            }
        }
        reject_reserved(
            &controls.options,
            &[
                "algorithm",
                "hessian_approximation",
                "gradient_approximation",
                "jacobian_approximation",
                "grad_f_constant",
                "jac_c_constant",
                "jac_d_constant",
                "hessian_constant",
                "presolve",
                "max_iter",
                "max_wall_time",
                "max_cpu_time",
                "tol",
                "warm_start_init_point",
                "nlp_scaling_method",
                "linear_solver",
                "option_file_name",
                "nlp_lower_bound_inf",
                "nlp_upper_bound_inf",
            ],
        )?;
        if controls.options.keys().any(|k| {
            k.rsplit('.')
                .next()
                .is_some_and(|k| k.starts_with("feral_") || k.starts_with("ma57_"))
        }) {
            return Err(ProblemError::Contract(
                "linear solver settings use the explicit FERAL profile".into(),
            ));
        }
        let mut options = controls.options.clone();
        options.extend([
            (
                "algorithm".into(),
                OptionValue::Text(
                    match method {
                        Method::InteriorPoint => "interior-point",
                        Method::ActiveSetSqp => "active-set-sqp",
                    }
                    .into(),
                ),
            ),
            (
                "hessian_approximation".into(),
                OptionValue::Text(if exact { "exact" } else { "limited-memory" }.into()),
            ),
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
                "nlp_scaling_method".into(),
                OptionValue::Text(
                    if oracle.scaling().is_some() {
                        "user-scaling"
                    } else {
                        "none"
                    }
                    .into(),
                ),
            ),
            ("linear_solver".into(), OptionValue::Text("feral".into())),
            (
                "warm_start_init_point".into(),
                OptionValue::Bool(duals.is_some()),
            ),
            ("print_level".into(), OptionValue::Integer(0)),
        ]);
        let reused = self.app.is_some()
            && self.layout == Some(compatibility.layout)
            && controls.reuse != ReusePolicy::Fresh;
        if self.app.is_some() && !reused && controls.reuse == ReusePolicy::RequireReuse {
            return Err(ProblemError::Contract(
                "POUNCE application reuse changes layout/profile".into(),
            ));
        }
        let mut app = if reused {
            self.app
                .take()
                .ok_or_else(|| ProblemError::Contract("missing POUNCE application".into()))?
        } else {
            self.app = None;
            IpoptApplication::new()
        };
        app.set_convex_routing_available(false);
        for (k, v) in &options {
            let o = app.options_mut();
            let set = match v {
                OptionValue::Text(v) => o.set_string_value(k, v, true, false),
                OptionValue::Real(v) => o.set_numeric_value(k, *v, true, false),
                OptionValue::Integer(v) => o.set_integer_value(k, *v, true, false),
                OptionValue::Bool(v) => o.set_bool_value(k, *v, true, false),
            };
            if !set.map_err(|e| ProblemError::Contract(format!("POUNCE option {k}: {e}")))? {
                return Err(ProblemError::Contract(format!("POUNCE ignored option {k}")));
            }
        }
        feral.parallel = Some(controls.threads > 1);
        feral.fma = false;
        let sink = Arc::new(Mutex::new(Default::default()));
        app.set_linear_backend_factory(
            pounce_rs::pounce_algorithm::application::default_backend_factory_with_sink(
                feral.clone(),
                Default::default(),
                sink.clone(),
            ),
        );
        app.initialize()
            .map_err(|e| ProblemError::Contract(format!("POUNCE initialization: {e}")))?;
        let inner = app.algorithm_builder_from_options();
        let config = feral.clone();
        let restore_sink = sink.clone();
        app.set_restoration_factory_provider(pounce_rs::pounce_restoration::resto_inner_solver::make_default_restoration_factory_provider(
        Default::default(),inner,move || {let config=config.clone();let sink=restore_sink.clone();Box::new(move ||pounce_rs::pounce_algorithm::application::default_backend_factory_with_sink(config.clone(),Default::default(),sink.clone()))}));
        if let Some(seed) = sqp_seed {
            app.set_sqp_warm_start(seed);
        }
        let adapter = Rc::new(RefCell::new(Adapter {
            normalize_affine: false,
            oracle,
            state: CallbackState::new(execution.clone()),
            jac,
            hess,
            initial,
            duals,
            solution: None,
        }));
        let native: Rc<RefCell<dyn TNLP>> = adapter.clone();
        let status = app.optimize_tnlp_without_presolve(native);
        let mut a = adapter.borrow_mut();
        let mut report = SolveReport::new(
            Backend::Pounce,
            a.oracle.contract(),
            termination(status),
            &execution,
        );
        report.options = options;
        report
            .metrics
            .insert("reuse.native_application".into(), Metric::Bool(reused));
        let mut statistics = app.statistics();
        if statistics.iterations.len() > controls.history {
            report.dropped_events += (statistics.iterations.len() - controls.history) as u64;
            statistics.iterations.truncate(controls.history);
        }
        // The pinned library's serde projection includes all statistics, including
        // new native fields. JSON null explicitly denotes unavailable/nonfinite data.
        match serde_json::to_value(&statistics) {
            Ok(value) => insert_native_metrics(&mut report.metrics, "statistics", value),
            Err(error) => {
                report.metrics.insert(
                    "statistics.encoding_error".into(),
                    Metric::Text(error.to_string()),
                );
            }
        }
        report.pounce_statistics = Some(Box::new(statistics));
        let timing = app.timing_stats();
        macro_rules! times{($($field:ident),*)=>{$(report.metrics.insert(concat!("timing.",stringify!($field)).into(),Metric::Real(timing.$field.total_wallclock_time()));)*}}
        times!(
            overall_alg,
            print_problem_statistics,
            initialize_iterates,
            update_hessian,
            output_iteration,
            update_barrier_parameter,
            compute_search_direction,
            compute_acceptable_trial_point,
            accept_trial_point,
            check_convergence,
            fire_intermediate,
            linear_system_symbolic_factorization,
            linear_system_factorization,
            linear_system_back_solve,
            quality_function_search,
            total_callback_time,
            total_function_evaluation_time,
            eval_obj,
            eval_grad_obj,
            eval_constr,
            eval_constr_jac,
            eval_lag_hess
        );
        if let Ok(s) = sink.lock() {
            report
                .metrics
                .insert("linear.factors".into(), Metric::Integer(s.n_factors as i64));
            report.metrics.insert(
                "linear.pattern_reuse".into(),
                Metric::Integer(s.n_pattern_reuse as i64),
            );
            report.metrics.insert(
                "linear.pattern_changes".into(),
                Metric::Integer(s.n_pattern_changes as i64),
            );
            for (k, v) in [
                ("linear.fill_ratio", s.max_fill_ratio),
                ("linear.min_pivot", s.min_abs_pivot),
                ("linear.max_pivot", s.max_abs_pivot),
            ] {
                if let Some(v) = v {
                    report.metrics.insert(k.into(), Metric::Real(v));
                }
            }
            if let Some((p, n, z)) = s.last_inertia {
                for (k, v) in [
                    ("linear.inertia.positive", p),
                    ("linear.inertia.negative", n),
                    ("linear.inertia.zero", z),
                ] {
                    report.metrics.insert(k.into(), Metric::Integer(v as i64));
                }
            }
            for (k, v) in [("linear.nnzA", s.last_nnz_a), ("linear.nnzL", s.last_nnz_l)] {
                if let Some(v) = v {
                    report.metrics.insert(k.into(), Metric::Integer(v as i64));
                }
            }
        }
        if let Some(d) = app.warm_start_diagnostics() {
            report
                .provenance
                .insert("warm.diagnostics".into(), format!("{d:?}"));
        }
        if let Some(c) = app.crossover_report() {
            report
                .provenance
                .insert("crossover".into(), format!("{c:?}"));
        }
        report.provenance.insert(
            "native".into(),
            "POUNCE 0.12.0; FERAL; shared preprocessing is applied before this adapter".into(),
        );
        report
            .provenance
            .insert("feral.effective".into(), format!("{feral:?}"));
        a.state.finish(&mut report);
        if let Some(mut candidate) = a.solution.take() {
            if candidate.primal.iter().all(|v| v.is_finite())
                && candidate.objective.is_some_and(f64::is_finite)
            {
                candidate.objective = candidate.objective.map(|v| v * sense.sign());
                match quality::contained(|| {
                    quality::nlp(a.oracle.as_mut(), &candidate.primal, tolerances)
                }) {
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
                let payload = if method == Method::ActiveSetSqp {
                    let mut s = pounce_rs::pounce_algorithm::sqp::SqpIterates::cold(n, m);
                    s.x = candidate.primal.clone();
                    if let Some(r) = &candidate.row_dual {
                        s.lambda_g.clone_from(r);
                    }
                    if let Some((l, u)) = &candidate.bound_dual {
                        s.lambda_x = l.iter().zip(u).map(|(l, u)| l - u).collect();
                    }
                    s.working = app.last_sqp_working_set().cloned();
                    WarmPayload::PounceSqp(s)
                } else {
                    WarmPayload::Nlp {
                        primal: candidate.primal.clone(),
                        bounds: candidate.bound_dual.clone(),
                        rows: candidate.row_dual.clone(),
                    }
                };
                report.warm_start = Some(WarmStart {
                    compatibility: compatibility.clone(),
                    payload,
                });
                report.candidate = Some(candidate);
            }
        }
        if report.candidate.is_none() {
            report.termination.assurance = Assurance::None
        }
        drop(a);
        if !matches!(
            report.termination.category,
            Termination::Panic
                | Termination::Evaluation
                | Termination::Invalid
                | Termination::Numerical
        ) {
            self.app = Some(app);
            self.layout = Some(compatibility.layout);
        }
        Ok(report)
    }
}
/// POUNCE statuses share numeric ABI values with Ipopt, but retain their native identity.
pub fn termination(status: ApplicationReturnStatus) -> NativeTermination {
    use ApplicationReturnStatus::*;
    let (category, assurance) = match status {
        SolveSucceeded => (Termination::Success, Assurance::LocalStationary),
        SolvedToAcceptableLevel => (Termination::Acceptable, Assurance::LocalStationary),
        FeasiblePointFound => (Termination::FeasibleOnly, Assurance::Feasible),
        InfeasibleProblemDetected => (Termination::Infeasible, Assurance::None),
        MaximumIterationsExceeded | InsufficientMemory => (Termination::Limit, Assurance::None),
        MaximumCpuTimeExceeded | MaximumWallTimeExceeded => {
            (Termination::TimeLimit, Assurance::None)
        }
        UserRequestedStop => (Termination::Cancelled, Assurance::None),
        InvalidNumberDetected => (Termination::Evaluation, Assurance::None),
        InvalidOption | InvalidProblemDefinition | NotEnoughDegreesOfFreedom | InternalError => {
            (Termination::Invalid, Assurance::None)
        }
        _ => (Termination::Numerical, Assurance::None),
    };
    NativeTermination {
        code: i64::from(status.as_int()),
        name: format!("{status:?}"),
        message: None,
        category,
        assurance,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pounce_nlp::tnlp::SparsityRequest;
    fn adapter() -> Adapter {
        let oracle = Box::new(crate::solver_tests::Polynomial::new());
        Adapter {
            normalize_affine: false,
            jac: Pattern::new(oracle.jacobian_pattern(), false).unwrap(),
            hess: Pattern::new(oracle.hessian_pattern().unwrap(), true).unwrap(),
            oracle,
            state: CallbackState::new(crate::solver_tests::execution()),
            initial: vec![2.0],
            duals: None,
            solution: None,
        }
    }
    #[test]
    fn native_tnlp_sparse_and_weighted_hessian_contract() {
        let mut a = adapter();
        let mut rows = [-1];
        let mut cols = [-1];
        assert!(a.eval_jac_g(
            None,
            false,
            SparsityRequest::Structure {
                irow: &mut rows,
                jcol: &mut cols
            }
        ));
        assert_eq!((rows, cols), ([0], [0]));
        let mut out = [0.0];
        assert!(a.eval_h(
            Some(&[2.0]),
            true,
            4.0,
            Some(&[3.0]),
            true,
            SparsityRequest::Values { values: &mut out }
        ));
        assert_eq!(out, [44.0]);
    }
    #[test]
    fn linear_profile_identity_preserves_external_order_and_float_bits() {
        let a = LinearSettings::default();
        let mut b = a.clone();
        b.ordering = pounce_feral::OrderingMethod::External(vec![1, 0]);
        assert_ne!(linear_key(&a), linear_key(&b));
        let k = linear_key(&b);
        b.ordering = pounce_feral::OrderingMethod::External(vec![0, 1]);
        assert_ne!(k, linear_key(&b));
    }
    #[test]
    fn local_pool_admission_does_not_accept_the_global_pool() {
        assert_eq!(ADMITTED.with(|a| a.get()), 0);
        let count: Result<usize, ProblemError> =
            with_threads(2, 2 << 20, || Ok(ADMITTED.with(|a| a.get())));
        assert_eq!(count.unwrap(), 2);
        assert_eq!(ADMITTED.with(|a| a.get()), 0);
    }
}
