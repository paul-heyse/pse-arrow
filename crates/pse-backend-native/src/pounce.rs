// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native POUNCE TNLP adapter sharing the exact NLP oracle and callback failure policy.
mod equalities;
mod observed;
mod profiles;
mod records;
pub use profiles::{SecondOpinionProfile, second_opinion_profiles};
mod retained;
pub use crate::settings::pounce::{LinearSettings, Method, Settings};
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
/// Worker count of FERAL's own factorization pool, by the rule feral 0.18 applies when it
/// builds that pool (`Solver::pool_num_threads`): `RAYON_NUM_THREADS` when it parses as a
/// positive count, else the available parallelism, else one. The pool cannot be injected,
/// so it is sized independently of the admitted local pool.
fn feral_pool_threads() -> usize {
    std::env::var("RAYON_NUM_THREADS")
        .ok()
        .and_then(|s| s.trim().parse::<usize>().ok())
        .filter(|n| *n > 0)
        .or_else(|| std::thread::available_parallelism().ok().map(|n| n.get()))
        .unwrap_or(1)
        .max(1)
}
/// FERAL's effective factorization thread count for an admitted count (F01). FERAL
/// factorizes on its own pool, outside the admitted scoped pool, so it runs parallel only
/// when admission covers every core that pool would use; otherwise it runs serial.
fn feral_threads(admitted: usize, pool: usize) -> usize {
    if admitted > 1 && admitted >= pool {
        pool
    } else {
        1
    }
}
/// Hidden second solves are pinned off, and the ℓ1 options are reserved: only the typed
/// [`Method::L1ExactPenalty`] sets the exact-penalty switch, and the automatic ℓ1 retry after
/// restoration failure stays off. A result never comes from an undeclared attempt beyond
/// the admitted iteration budget (F03).
const PINNED_OFF: [&str; 2] = ["mu_strategy_fallback", "dual_divergence_retry"];
const RESERVED_METHODS: [&str; 2] = [
    "l1_fallback_on_restoration_failure",
    "l1_exact_penalty_barrier",
];
/// The complete effective option table after a solve, with the registered defaults, read
/// back from the library as the HiGHS adapter does: every registered option at its current
/// value, plus any explicitly set prefixed option such as `resto.tol`.
fn option_snapshot(app: &IpoptApplication) -> (Options, Options) {
    use pounce_common::{DefaultValue, OptionType};
    let list = app.options();
    let mut effective = Options::new();
    let mut defaults = Options::new();
    for option in app.registered_options().registered_options_in_order() {
        let name = option.name.as_str();
        let boolean = option.option_type == OptionType::OT_String
            && option.valid_strings.len() == 2
            && option
                .valid_strings
                .iter()
                .all(|s| matches!(s.value.as_str(), "yes" | "no"));
        let current = match option.option_type {
            OptionType::OT_Number => list
                .get_numeric_value(name, "")
                .ok()
                .map(|(v, _)| OptionValue::Real(v)),
            OptionType::OT_Integer => list
                .get_integer_value(name, "")
                .ok()
                .map(|(v, _)| OptionValue::Integer(v)),
            OptionType::OT_String if boolean => list
                .get_bool_value(name, "")
                .ok()
                .map(|(v, _)| OptionValue::Bool(v)),
            OptionType::OT_String => list
                .get_string_value(name, "")
                .ok()
                .map(|(v, _)| OptionValue::Text(v)),
            OptionType::OT_Unknown => None,
        };
        let default = match &option.default {
            DefaultValue::Number(v) => Some(OptionValue::Real(*v)),
            DefaultValue::Integer(v) => Some(OptionValue::Integer(*v)),
            DefaultValue::String(v) if boolean => Some(OptionValue::Bool(v == "yes")),
            DefaultValue::String(v) => Some(OptionValue::Text(v.clone())),
            DefaultValue::None => None,
        };
        if let Some(value) = current {
            effective.insert(option.name.clone(), value);
        }
        if let Some(value) = default {
            defaults.insert(option.name.clone(), value);
        }
    }
    for name in list.names() {
        if !effective.contains_key(name)
            && let Ok((value, true)) = list.get_string_value(name, "")
        {
            effective.insert(name.into(), OptionValue::Text(value));
        }
    }
    (effective, defaults)
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
        .map_err(|e| E::from(ProblemError::Internal(format!("POUNCE local pool: {e}"))))?
}
/// Worker-local application and actual FERAL backend retention. Each call constructs
/// iteration state and refactors its matrix; compatible library symbolic state survives
/// through exclusive role-specific leases rather than a fresh backend factory.
#[derive(Default)]
pub struct Session {
    app: Option<IpoptApplication>,
    stamp: Option<Compatibility>,
    factors: Option<Rc<RefCell<retained::Pool>>>,
    foreign_allowance: Option<usize>,
    // Dropped after the app and factor pool; prior attempt reservations stay
    // retained while any compatible source-owned factor can remain live.
    storage_admissions: Vec<Arc<dyn WorkAdmission>>,
}
impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PounceSession")
            .field("stamp", &self.stamp)
            .finish_non_exhaustive()
    }
}
impl Session {
    /// Construct after the owning worker has received admission.
    pub fn new() -> Self {
        Self::default()
    }
    /// Observable pattern/owner buffers. This is not the extent of opaque FERAL
    /// allocations; the execution owner must retain its admitted foreign allowance.
    pub fn retained_layout_bytes(&self) -> usize {
        size_of::<Self>().saturating_add(
            self.factors
                .as_ref()
                .map_or(0, |pool| pool.borrow().layout_bytes()),
        )
    }
    /// Full finite foreign allowance to keep charged while opaque backends are retained.
    /// `None` means no full allocation accounting was supplied by this caller.
    pub fn retained_foreign_allowance(&self) -> Option<usize> {
        self.factors.as_ref().and(self.foreign_allowance)
    }
    /// Execute native POUNCE over the already-preprocessed oracle. The oracle
    /// must be created on this worker; a parallel profile requires `with_threads`.
    #[expect(
        clippy::too_many_arguments,
        reason = "the native entry takes the oracle, start, sense, controls, accuracy, settings, execution, tolerances, warm start and compatibility as independent inputs of one attempt"
    )]
    pub fn solve(
        &mut self,
        oracle: Box<dyn NlpOracle>,
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
        settings.restart.validate()?;
        let method = settings.method;
        let mut feral = settings.linear.clone();
        if oracle.normalization().is_some() {
            return Err(ProblemError::Internal("model normalization must be transported through the shared NLP pipeline before native execution".into()));
        }
        let separator = oracle.solve_separator().cloned();
        let n = oracle.contract().variables.len();
        let m = oracle.contract().rows.len();
        tolerances.validate(n, m)?;
        // Every mode but the library's quasi-Newton approximation supplies the Hessian:
        // the exact Lagrangian or the oracle's Gauss–Newton Gram.
        let curvature_options = settings.curvature_options(controls.hessian)?;
        let supplied = matches!(
            controls.hessian,
            HessianMode::Exact | HessianMode::GaussNewton
        );
        crate::validate_nlp(
            oracle.as_ref(),
            if supplied {
                pse_kernels::DerivativeOrder::Second
            } else {
                pse_kernels::DerivativeOrder::First
            },
        )?;
        if initial.len() != n || oracle.constraint_bounds().len() != m {
            return Err(ProblemError::Internal("POUNCE dimensions".into()));
        }
        finite(initial)?;
        if controls.threads > 1 && ADMITTED.with(|a| a.get()) < controls.threads {
            return Err(ProblemError::Internal(
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
            crate::settings::pounce::admit_bound(v)?;
        }
        let jac = Pattern::new(oracle.jacobian_pattern(), false)?;
        let hess = if supplied
            || controls.hessian == HessianMode::FiniteDifference
                && oracle.hessian_pattern().is_some()
        {
            Pattern::new(
                oracle.hessian_pattern().ok_or_else(|| {
                    ProblemError::Unsupported("POUNCE exact Hessian unavailable".into())
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
        let mut barrier = None;
        let mut sqp_seed = None;
        if let Some(w) = warm {
            w.validate(&compatibility)?;
            let WarmPayload::Nlp {
                primal,
                bounds,
                rows,
                barrier: seed_barrier,
                working,
            } = &w.payload
            else {
                return Err(ProblemError::Contract("POUNCE seed class".into()));
            };
            if primal.len() != n {
                return Err(ProblemError::Contract("POUNCE seed shape".into()));
            }
            finite(primal)?;
            initial.clone_from(primal);
            let complete = match (bounds, rows) {
                (Some((l, u)), Some(r)) => {
                    if l.len() != n
                        || u.len() != n
                        || r.len() != m
                        || l.iter().chain(u).any(|v| *v < 0.0)
                    {
                        return Err(ProblemError::Contract("POUNCE dual seed shape/sign".into()));
                    }
                    finite(l)?;
                    finite(u)?;
                    finite(r)?;
                    Some((l.clone(), u.clone(), r.clone()))
                }
                (None, None) => None,
                _ => return Err(ProblemError::Unsupported("partial POUNCE dual seed".into())),
            };
            if method == Method::L1ExactPenalty {
                // The exact-penalty problem has its own slack multipliers; only the primal
                // seed is portable into it.
            } else if method == Method::ActiveSetSqp {
                // The active-set iterate: primal, row and packed bound multipliers, and the
                // working set when the native transformation retained it (F07).
                let mut s = pounce_rs::sqp::SqpIterates::cold(n, m);
                s.x = primal.clone();
                if let Some((l, u, r)) = &complete {
                    s.lambda_g.clone_from(r);
                    s.lambda_x = l.iter().zip(u).map(|(l, u)| l - u).collect();
                }
                if let Some(ws) = working {
                    if ws.active.n() != n || ws.active.m() != m {
                        return Err(ProblemError::Contract("POUNCE working set shape".into()));
                    }
                    s.working = Some(ws.active.clone());
                }
                sqp_seed = Some(s);
            } else {
                // Interior-point methods consume the primal-dual iterate and its barrier; a
                // working set has no interior-point meaning and is recorded as not submitted.
                duals = complete;
                barrier = *seed_barrier;
            }
        }
        reject_reserved(&controls.options, &PINNED_OFF)?;
        reject_reserved(&controls.options, &RESERVED_METHODS)?;
        reject_reserved(&controls.options, &RESTART_OPTIONS)?;
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
        if controls.options.keys().any(|key| {
            key.rsplit('.').next().is_some_and(|key| {
                key.starts_with("partitioned_") || key.starts_with("fd_hessian_")
            })
        }) {
            return Err(ProblemError::Contract(
                "curvature settings use the typed POUNCE profile".into(),
            ));
        }
        let mut options = controls.options.clone();
        options.extend(curvature_options);
        options.extend(accuracy.pounce_options());
        options.extend([
            (
                "algorithm".into(),
                OptionValue::Text(
                    match method {
                        Method::InteriorPoint | Method::L1ExactPenalty => "interior-point",
                        Method::ActiveSetSqp => "active-set-sqp",
                    }
                    .into(),
                ),
            ),
            (
                "hessian_approximation".into(),
                OptionValue::Text(
                    match controls.hessian {
                        HessianMode::Exact | HessianMode::GaussNewton => "exact",
                        HessianMode::LimitedMemory => "limited-memory",
                        HessianMode::Partitioned => "partitioned",
                        HessianMode::FiniteDifference => "finite-difference",
                        HessianMode::Auto => {
                            return Err(ProblemError::Contract(
                                "unresolved automatic curvature".into(),
                            ));
                        }
                    }
                    .into(),
                ),
            ),
            (
                "max_iter".into(),
                OptionValue::Integer(controls.iterations as i32),
            ),
            (
                "max_wall_time".into(),
                OptionValue::Real(controls.time_limit.as_secs_f64()),
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
            ("linear_solver".into(), OptionValue::Text("feral".into())),
            (
                "warm_start_init_point".into(),
                OptionValue::Bool(duals.is_some()),
            ),
            ("print_level".into(), OptionValue::Integer(0)),
        ]);
        options.extend(PINNED_OFF.map(|k| (k.to_owned(), OptionValue::Bool(false))));
        options.extend([
            (
                "l1_exact_penalty_barrier".to_owned(),
                OptionValue::Bool(method == Method::L1ExactPenalty),
            ),
            (
                "l1_fallback_on_restoration_failure".to_owned(),
                OptionValue::Bool(false),
            ),
        ]);
        // A primal-dual seed restarts under the typed profile (L-N3); `mu_init` is read only
        // by the monotone barrier update.
        let restart = duals.is_some().then(|| {
            let monotone = !matches!(
                controls.options.get("mu_strategy"),
                Some(OptionValue::Text(s)) if s == "adaptive"
            );
            let (restart_options, applied) = settings.restart.apply(barrier, monotone);
            options.extend(restart_options);
            applied
        });
        let linear_threads = feral_threads(controls.threads, feral_pool_threads());
        feral.parallel = Some(linear_threads > 1);
        feral.fma = false;
        let factor_identity = serde_json::json!({"feral":crate::settings::pounce::record(&feral).map_err(|e|ProblemError::Internal(format!("FERAL identity: {e}")))?,"hessian_supplied":supplied,"method":serde_json::to_value(method).map_err(|e|ProblemError::Internal(format!("POUNCE method identity: {e}")))?});
        let reused = self.app.is_some()
            && self
                .stamp
                .as_ref()
                .is_some_and(|s| s.same_session(&compatibility))
            && controls.reuse != ReusePolicy::Fresh;
        if self.app.is_some() && !reused && controls.reuse == ReusePolicy::RequireReuse {
            return Err(ProblemError::Reuse {
                backend: Backend::Pounce,
                refusal: crate::ReuseRefusal::Structure,
            });
        }
        let factor_reused = reused
            && self
                .factors
                .as_ref()
                .is_some_and(|pool| pool.borrow().matches(&factor_identity));
        if self.factors.is_some() && !factor_reused && controls.reuse == ReusePolicy::RequireReuse {
            return Err(ProblemError::Reuse {
                backend: Backend::Pounce,
                refusal: crate::ReuseRefusal::Structure,
            });
        }
        let mut app = if reused {
            let mut app = self
                .app
                .take()
                .ok_or_else(|| ProblemError::Internal("missing POUNCE application".into()))?;
            // A reused application starts from an empty option table, so no option of an
            // earlier step survives into this one (F02).
            app.options_mut().clear();
            app
        } else {
            self.app = None;
            IpoptApplication::new()
        };
        app.set_convex_routing_available(false);
        app.set_effective_feral_config(feral.clone());
        app.clear_kkt_schur_block();
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
        let pool = if factor_reused {
            self.factors
                .as_ref()
                .ok_or_else(|| ProblemError::Internal("missing retained FERAL pool".into()))?
                .clone()
        } else {
            retained::Pool::new(feral.clone(), factor_identity)
        };
        self.factors = Some(pool.clone());
        self.foreign_allowance = controls.foreign_bytes.or(execution.memory);
        let before_factors = pool.borrow().counts();
        let sink = Arc::new(Mutex::new(Default::default()));
        app.set_linear_backend_factory(retained::factory(
            pool.clone(),
            sink.clone(),
            retained::Phase::Main,
        ));
        app.initialize()
            .map_err(|e| ProblemError::Internal(format!("POUNCE initialization: {e}")))?;
        let inner = app.algorithm_builder_from_options();
        let restore_sink = sink.clone();
        let restore_pool = pool.clone();
        app.set_restoration_factory_provider(pounce_rs::pounce_restoration::resto_inner_solver::make_default_restoration_factory_provider(
        Default::default(),inner,move || {let pool=restore_pool.clone();let sink=restore_sink.clone();Box::new(move ||retained::factory(pool.clone(),sink.clone(),retained::Phase::Restoration))}));
        let working_set_submitted = sqp_seed.as_ref().is_some_and(|s| s.working.is_some());
        if let Some(seed) = sqp_seed {
            app.set_sqp_warm_start(seed);
        }
        let adapter = Rc::new(RefCell::new(Adapter {
            normalization: pse_math::normalization::Normalization::identity(
                oracle.contract().variables.len(),
                oracle.contract().rows.len(),
            ),
            certification_budget: None,
            normalize_affine: false,
            oracle,
            state: CallbackState::new(execution.clone()),
            jac,
            hess,
            initial,
            duals,
            solution: None,
        }));
        let native: Rc<RefCell<dyn TNLP>> = if method == Method::L1ExactPenalty {
            let inner: Rc<RefCell<dyn TNLP>> = adapter.clone();
            Rc::new(RefCell::new(
                equalities::Equalities::new(inner).ok_or_else(|| {
                    ProblemError::Internal("POUNCE equality form of the NLP".into())
                })?,
            ))
        } else {
            adapter.clone()
        };
        if let Some(admission) = &execution.work_admission {
            self.storage_admissions.push(admission.clone());
        }
        let observation = Rc::new(observed::Observation::new(
            execution.clone(),
            separator,
            controls.foreign_bytes.or(execution.memory),
        ));
        let observation_scope = pounce_common::observed::Scope::enter(observation.clone());
        let status = app.optimize_tnlp_without_presolve(native);
        let mut a = adapter.borrow_mut();
        let mut report = SolveReport::new(
            Backend::Pounce,
            a.oracle.contract(),
            termination(status),
            &execution,
        );
        // The option table read back from the application is what ran; `options` is only
        // what this adapter set on it.
        let (effective, defaults) = option_snapshot(&app);
        report.options = effective;
        report.native_defaults = defaults;
        report
            .metrics
            .insert("reuse.native_application".into(), Metric::Bool(reused));
        report.evidence.reused_native_state = reused;
        let after_factors = pool.borrow().counts();
        for (name, count) in [
            (
                "linear.backend_objects.created",
                after_factors.created.saturating_sub(before_factors.created),
            ),
            (
                "linear.backend_objects.reused",
                after_factors.reused.saturating_sub(before_factors.reused),
            ),
            (
                "linear.structure_kept",
                after_factors
                    .pattern_kept
                    .saturating_sub(before_factors.pattern_kept),
            ),
            (
                "linear.structure_refreshes",
                after_factors
                    .structure_refreshes
                    .saturating_sub(before_factors.structure_refreshes),
            ),
        ] {
            report.metrics.insert(
                name.into(),
                Metric::Integer(i64::try_from(count).unwrap_or(i64::MAX)),
            );
        }
        report.metrics.insert(
            "linear.retained_pool_compatible".into(),
            Metric::Bool(factor_reused),
        );
        report.metrics.insert(
            "linear.retained_layout_bytes".into(),
            Metric::Integer(i64::try_from(self.retained_layout_bytes()).unwrap_or(i64::MAX)),
        );
        report.metrics.insert(
            "linear.lifetime_extrema".into(),
            Metric::Text("unavailable-for-this-attempt-from-pinned-cumulative-summary".into()),
        );
        report
            .metrics
            .insert("linear.fresh_response_factor".into(), Metric::Bool(false));
        report.metrics.insert(
            "linear.threads".into(),
            Metric::Integer(i64::try_from(linear_threads).unwrap_or(i64::MAX)),
        );
        let mut statistics = app.statistics();
        let final_barrier = Some(statistics.final_mu)
            .filter(|v| method == Method::InteriorPoint && v.is_finite() && *v > 0.0);
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
            report.evidence.work.factorizations = Some(s.n_factors);
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
                .insert("warm.diagnostics".into(), records::warm(&d));
        }
        if let Some(c) = app.crossover_report() {
            report
                .provenance
                .insert("crossover".into(), records::crossover(c));
        }
        report.evidence.start_submitted = warm.is_some();
        report.evidence.restart = restart;
        report.evidence.working_set_submitted = working_set_submitted;
        report
            .metrics
            .insert("start.submitted".into(), Metric::Bool(warm.is_some()));
        report.provenance.insert(
            "native".into(),
            "POUNCE 0.12.0; FERAL; shared preprocessing is applied before this adapter".into(),
        );
        // The effective FERAL configuration through its identity encoding (F30).
        let effective = crate::settings::pounce::record(&feral)
            .map_err(|e| ProblemError::Internal(format!("FERAL settings record: {e}")))?;
        report
            .provenance
            .insert("feral.effective".into(), effective.to_string());
        observation.record(&mut report);
        if let Some(mut candidate) = a.solution.take()
            && candidate.primal.iter().all(|v| v.is_finite())
            && candidate.objective.is_some_and(f64::is_finite)
        {
            candidate.objective = candidate.objective.map(|v| v * sense.sign());
            let Adapter { state, oracle, .. } = &mut *a;
            match state.evaluate("native.validation.constraints", || {
                quality::nlp(oracle.as_mut(), &candidate.primal, tolerances)
            }) {
                Some(q) => {
                    if !q.feasible() {
                        report.termination.assurance = Assurance::None;
                    }
                    report.quality = Some(q);
                }
                None => {
                    let error = state
                        .terminal_error()
                        .or_else(|| state.last_failure.take())
                        .unwrap_or_else(|| {
                            ProblemError::Internal("native validation callback failed".into())
                        });
                    report.record_validation_failure(error);
                    report.termination.assurance = Assurance::None;
                }
            }
            // The active-set working set is keyed by this attempt's native coordinates;
            // an interior-point seed carries its final barrier value instead.
            let working = (method == Method::ActiveSetSqp)
                .then(|| app.last_sqp_working_set().cloned())
                .flatten()
                .map(|active| WorkingSet {
                    transformation: compatibility.layout,
                    active,
                });
            let complete = candidate.bound_dual.is_some() && candidate.row_dual.is_some();
            let payload = WarmPayload::Nlp {
                primal: candidate.primal.clone(),
                bounds: candidate.bound_dual.clone(),
                rows: candidate.row_dual.clone(),
                barrier: final_barrier.filter(|_| complete),
                working,
            };
            report.warm_start = Some(WarmStart {
                origin: None,
                compatibility: compatibility.clone(),
                payload,
            });
            report.candidate = Some(candidate);
        }
        if report.candidate.is_none() {
            report.termination.assurance = Assurance::None
        }
        a.state.finish(&mut report);
        let source_abort = observation_scope.abort();
        let source_error = observation.take_error().or_else(|| {
            source_abort.as_ref().map(|abort| match abort {
                pounce_common::observed::Abort::Cancelled => ProblemError::Cancelled,
                pounce_common::observed::Abort::Resource(detail) => {
                    ProblemError::memory(detail.clone())
                }
                pounce_common::observed::Abort::Contract(detail) => {
                    ProblemError::Contract(detail.clone())
                }
                pounce_common::observed::Abort::Panic => {
                    ProblemError::Internal("panic in native operation observer".into())
                }
            })
        });
        if let Some(error) = source_error {
            report.termination.category =
                if matches!(source_abort, Some(pounce_common::observed::Abort::Panic)) {
                    Termination::Panic
                } else {
                    match crate::callback::classify(&error) {
                        crate::callback::Failure::Stopped(stop) => stop,
                        _ => Termination::Invalid,
                    }
                };
            report.record_validation_failure(error);
            report.termination.assurance = Assurance::None;
        }
        drop(observation_scope);
        drop(a);
        if !matches!(
            report.termination.category,
            Termination::Panic
                | Termination::Evaluation
                | Termination::Invalid
                | Termination::Numerical
        ) {
            self.app = Some(app);
            self.stamp = Some(compatibility);
        } else {
            self.factors = None;
            self.foreign_allowance = None;
        }
        Ok(report)
    }
}
/// POUNCE statuses share numeric ABI values with Ipopt, but retain their native identity
/// under the upstream C spelling. The match is exhaustive: a status added by an
/// upgrade fails to compile instead of being silently categorized.
pub fn termination(status: ApplicationReturnStatus) -> NativeTermination {
    use ApplicationReturnStatus::*;
    let category = match status {
        SolveSucceeded => Termination::Success,
        SolvedToAcceptableLevel => Termination::Acceptable,
        FeasiblePointFound => Termination::FeasibleOnly,
        InfeasibleProblemDetected => Termination::Infeasible,
        MaximumIterationsExceeded => Termination::IterationLimit,
        InsufficientMemory => Termination::ResourceExhausted,
        MaximumCpuTimeExceeded | MaximumWallTimeExceeded => Termination::TimeLimit,
        UserRequestedStop => Termination::Cancelled,
        InvalidNumberDetected => Termination::Evaluation,
        InvalidOption | InvalidProblemDefinition | NotEnoughDegreesOfFreedom | InternalError => {
            Termination::Invalid
        }
        SearchDirectionBecomesTooSmall
        | DivergingIterates
        | RestorationFailed
        | ErrorInStepComputation
        | UnrecoverableException
        | NonIpoptExceptionThrown => Termination::Numerical,
    };
    NativeTermination {
        code: i64::from(status.as_int()),
        name: status.upstream_name().into(),
        message: None,
        category,
        assurance: Assurance::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pounce_nlp::tnlp::SparsityRequest;
    fn adapter() -> Adapter {
        let oracle = Box::new(crate::solver_tests::Polynomial::new());
        Adapter {
            normalization: pse_math::normalization::Normalization::identity(
                oracle.contract().variables.len(),
                oracle.contract().rows.len(),
            ),
            certification_budget: None,
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
    fn objective_support_remains_conservative_at_stationary_start() {
        let mut a = adapter();
        let mut gradient = [1.0];
        assert!(a.eval_grad_f(&[0.0], true, &mut gradient));
        assert_eq!(gradient, [0.0]);
        let mut linearity = [pounce_nlp::tnlp::Linearity::Linear];
        assert!(a.get_objective_variables_linearity(&mut linearity));
        assert_eq!(linearity, [pounce_nlp::tnlp::Linearity::NonLinear]);
    }
    #[test]
    fn typed_curvature_settings_preserve_default_setness_and_refuse_unresolved_auto() {
        let mut settings = Settings::default();
        let options = settings
            .curvature_options(HessianMode::Partitioned)
            .unwrap();
        assert!(!options.contains_key("partitioned_update_type"));
        settings.partitioned.update_type =
            Some(pse_model::generated::enums::PouncePartitionedUpdate::Sr1);
        assert_eq!(
            settings
                .curvature_options(HessianMode::Partitioned)
                .unwrap()["partitioned_update_type"],
            OptionValue::Text("sr1".into())
        );
        assert!(settings.curvature_options(HessianMode::Auto).is_err());
        settings.finite_difference.reuse_tolerance = f64::NAN;
        assert!(
            settings
                .curvature_options(HessianMode::FiniteDifference)
                .is_err()
        );
    }
    #[derive(Debug)]
    struct FirstOnly {
        contract: crate::OracleContract,
        jac: faer::sparse::SparseColMat<usize, f64>,
        gradients: Arc<std::sync::atomic::AtomicUsize>,
        separator: Option<crate::SolveSeparator>,
    }
    impl FirstOnly {
        fn new(gradients: Arc<std::sync::atomic::AtomicUsize>, separated: bool) -> Self {
            Self {
                contract: crate::OracleContract {
                    identity: pse_ids::ContentHash::from_bytes([19; 32]),
                    variables: (0..4)
                        .map(|i| crate::Variable {
                            id: crate::solver_tests::id(i + 1),
                            lower: -1.0,
                            upper: 1.0,
                        })
                        .collect(),
                    rows: vec![crate::solver_tests::id(5)],
                    derivatives: pse_kernels::DerivativeOrder::First,
                    smoothness: pse_kernels::DerivativeOrder::Second,
                },
                jac: faer::sparse::SparseColMat::try_new_from_triplets(
                    1,
                    4,
                    &(0..4)
                        .map(|j| faer::sparse::Triplet::new(0, j, 1.0))
                        .collect::<Vec<_>>(),
                )
                .unwrap(),
                gradients,
                separator: separated.then(|| crate::SolveSeparator {
                    variables: vec![],
                    rows: vec![0],
                }),
            }
        }
    }
    impl NlpOracle for FirstOnly {
        fn contract(&self) -> &crate::OracleContract {
            &self.contract
        }
        fn solve_separator(&self) -> Option<&crate::SolveSeparator> {
            self.separator.as_ref()
        }
        fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
            self.jac.symbolic()
        }
        fn hessian_pattern(&self) -> Option<faer::sparse::SymbolicSparseColMatRef<'_, usize>> {
            None
        }
        fn constraint_bounds(&self) -> &[(f64, f64)] {
            &[(1.0, 1.0)]
        }
        fn objective(&mut self, x: &[f64]) -> Result<f64, ProblemError> {
            Ok(x.iter().map(|v| v * v * 0.5).sum())
        }
        fn gradient(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            self.gradients
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            out.copy_from_slice(x);
            Ok(())
        }
        fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            out[0] = x.iter().sum();
            Ok(())
        }
        fn jacobian(&mut self, _: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            out.fill(1.0);
            Ok(())
        }
        fn hessian(
            &mut self,
            _: &[f64],
            _: f64,
            _: &[f64],
            _: &mut [f64],
        ) -> Result<(), ProblemError> {
            panic!("First oracle Hessian must never be called")
        }
    }
    fn first_run(
        mode: HessianMode,
        separated: bool,
        settings: &Settings,
        execution: Execution,
    ) -> SolveReport {
        let gradients = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let report = Session::new()
            .solve(
                Box::new(FirstOnly::new(gradients.clone(), separated)),
                &[0.0; 4],
                ObjectiveSense::Minimize,
                &Controls {
                    hessian: mode,
                    foreign_bytes: Some(64 << 20),
                    ..Default::default()
                },
                &ResolvedAccuracy::nominal(),
                settings,
                execution,
                &Tolerances {
                    variables: vec![1e-8; 4],
                    rows: vec![1e-8],
                    integrality: 1e-8,
                },
                None,
                crate::solver_tests::stamp(Backend::Pounce),
            )
            .unwrap();
        assert_eq!(
            report.metrics["callback.gradient.calls"],
            Metric::Integer(gradients.load(std::sync::atomic::Ordering::Relaxed) as i64)
        );
        report
    }
    #[test]
    fn curvature_modes_act_on_first_only_oracle_and_count_fd_probes() {
        for mode in [
            HessianMode::LimitedMemory,
            HessianMode::Partitioned,
            HessianMode::FiniteDifference,
        ] {
            let report = first_run(
                mode,
                false,
                &Settings::default(),
                crate::solver_tests::execution(),
            );
            assert_eq!(
                report.termination.category,
                Termination::Success,
                "{mode:?}"
            );
            let stats = report.pounce_statistics.as_ref().unwrap();
            match mode {
                HessianMode::Partitioned => {
                    assert!(stats.partitioned_elements > 0);
                    assert!(stats.partitioned_stored_reals > 0);
                }
                HessianMode::FiniteDifference => {
                    assert_eq!(stats.fd_hessian_n, 4);
                    assert_eq!(stats.fd_hessian_groups, 4);
                    assert!(
                        counted(&report, "callback.gradient.calls")
                            >= i64::from(stats.num_obj_grad_evals)
                    );
                }
                _ => {}
            }
        }
    }
    #[test]
    fn bounded_schur_consumer_reports_complete_linear_reservation_and_unknown_heap() {
        let mut settings = Settings::default();
        settings.linear.bounded_dense_max_dimension = Some(7);
        settings.linear.ordering = feral::symbolic::OrderingMethod::Amd;
        let report = first_run(
            HessianMode::FiniteDifference,
            true,
            &settings,
            crate::solver_tests::execution(),
        );
        assert_eq!(report.termination.category, Termination::Success);
        assert_eq!(
            report.metrics["linear.schur.actual_use"],
            Metric::Bool(true)
        );
        assert_eq!(
            report.metrics["linear.storage.complete_extent"],
            Metric::Bool(true)
        );
        assert_eq!(
            report.metrics["linear.storage.actual_total"],
            Metric::Text("unknown".into())
        );
        assert!(counted(&report, "linear.storage.reservation_bound") > 0);
        assert_eq!(
            report.metrics["native.application.storage.opaque"],
            Metric::Bool(true)
        );
    }
    #[test]
    fn settings_identity_frames_float_bits_and_refuses_external_ordering() {
        let key =
            |s: &Settings| pse_ids::document::of(pse_ids::Frame::BackendSettingsV5, s).unwrap();
        let a = Settings::default();
        let mut b = a.clone();
        b.linear.ordering = feral::symbolic::OrderingMethod::MetisND;
        assert_ne!(key(&a), key(&b));
        // A caller-supplied permutation is problem data: it has no settings encoding.
        b.linear.ordering = feral::symbolic::OrderingMethod::External(vec![1, 0]);
        assert!(pse_ids::document::of(pse_ids::Frame::BackendSettingsV5, &b).is_err());
        let mut c = a.clone();
        c.linear.pivtol = -0.0;
        let mut d = a.clone();
        d.linear.pivtol = 0.0;
        assert_ne!(key(&c), key(&d));
        c = a.clone();
        c.method = Method::ActiveSetSqp;
        assert_ne!(key(&a), key(&c));
    }
    fn run(
        session: &mut Session,
        options: Options,
        threads: usize,
    ) -> Result<SolveReport, ProblemError> {
        let controls = Controls {
            options,
            threads,
            reuse: ReusePolicy::AllowRebuild,
            ..Controls::default()
        };
        session.solve(
            Box::new(crate::solver_tests::Polynomial::new()),
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
            crate::solver_tests::stamp(Backend::Pounce),
        )
    }
    #[test]
    fn feral_threads_bounded_by_admission() {
        // FERAL's own pool runs only when admission covers every core it would use.
        assert_eq!(feral_threads(1, 8), 1);
        assert_eq!(feral_threads(2, 8), 1);
        assert_eq!(feral_threads(8, 8), 8);
        assert_eq!(feral_threads(16, 8), 8);
        assert_eq!(feral_threads(1, 1), 1);
        let pool = feral_pool_threads();
        let report =
            with_threads(2, 8 << 20, || run(&mut Session::new(), Options::new(), 2)).unwrap();
        assert!(matches!(report.termination.category, Termination::Success));
        let expected = i64::try_from(feral_threads(2, pool)).unwrap();
        assert_eq!(report.metrics["linear.threads"], Metric::Integer(expected));
        let effective: serde_json::Value =
            serde_json::from_str(&report.provenance["feral.effective"]).unwrap();
        assert_eq!(effective["parallel"], serde_json::json!(expected > 1));
    }
    #[test]
    fn reused_session_does_not_inherit_options() {
        let mut session = Session::new();
        let adaptive =
            Options::from([("mu_strategy".into(), OptionValue::Text("adaptive".into()))]);
        let first = run(&mut session, adaptive, 1).unwrap();
        assert_eq!(
            first.options["mu_strategy"],
            OptionValue::Text("adaptive".into())
        );
        let second = run(&mut session, Options::new(), 1).unwrap();
        assert_eq!(
            second.metrics["reuse.native_application"],
            Metric::Bool(true)
        );
        // The reused application ran at the registered default, not the first step's value.
        assert_eq!(
            second.options["mu_strategy"],
            second.native_defaults["mu_strategy"]
        );
        let fresh = run(&mut Session::new(), Options::new(), 1).unwrap();
        assert_eq!(second.options, fresh.options);
    }
    fn reuse_run(
        session: &mut Session,
        settings: &Settings,
        reuse: ReusePolicy,
        stamp: Compatibility,
    ) -> Result<SolveReport, ProblemError> {
        session.solve(
            Box::new(crate::solver_tests::Polynomial::new()),
            &[2.0],
            ObjectiveSense::Minimize,
            &Controls {
                reuse,
                foreign_bytes: Some(64 << 20),
                ..Controls::default()
            },
            &ResolvedAccuracy::nominal(),
            settings,
            crate::solver_tests::execution(),
            &Tolerances {
                variables: vec![1e-8],
                rows: vec![1e-8],
                integrality: 1e-8,
            },
            None,
            stamp,
        )
    }
    fn counted(report: &SolveReport, name: &str) -> i64 {
        match report.metrics.get(name) {
            Some(Metric::Integer(n)) => *n,
            other => panic!("{name}: {other:?}"),
        }
    }
    #[test]
    fn compatible_applications_retain_actual_feral_backend_and_observe_attempt_factor_deltas() {
        let mut session = Session::new();
        let stamp = crate::solver_tests::stamp(Backend::Pounce);
        let cold = reuse_run(
            &mut session,
            &Settings::default(),
            ReusePolicy::AllowRebuild,
            stamp.clone(),
        )
        .unwrap();
        assert_eq!(cold.termination.category, Termination::Success);
        assert!(counted(&cold, "linear.backend_objects.created") > 0);
        assert!(counted(&cold, "linear.factors") > 0);
        let pool = session.factors.as_ref().unwrap().clone();
        let mut changed = stamp.clone();
        changed.data = pse_ids::ContentHash::from_bytes([61; 32]);
        let warm = reuse_run(
            &mut session,
            &Settings::default(),
            ReusePolicy::AllowRebuild,
            changed,
        )
        .unwrap();
        assert_eq!(warm.termination.category, Termination::Success);
        assert!(Rc::ptr_eq(&pool, session.factors.as_ref().unwrap()));
        assert_eq!(counted(&warm, "linear.backend_objects.created"), 0);
        assert!(counted(&warm, "linear.backend_objects.reused") > 0);
        assert!(counted(&warm, "linear.structure_kept") > 0);
        assert!(counted(&warm, "linear.pattern_reuse") > 0);
        assert_eq!(
            warm.evidence.work.factorizations,
            Some(counted(&warm, "linear.factors") as u64)
        );
        assert!(!warm.metrics.contains_key("linear.min_pivot"));
        assert_eq!(
            warm.metrics["linear.fresh_response_factor"],
            Metric::Bool(false)
        );
        assert_eq!(session.retained_foreign_allowance(), Some(64 << 20));
        assert!(session.retained_layout_bytes() > size_of::<Session>());
        let fresh = reuse_run(
            &mut session,
            &Settings::default(),
            ReusePolicy::Fresh,
            stamp,
        )
        .unwrap();
        assert_eq!(fresh.termination.category, Termination::Success);
        assert!(!Rc::ptr_eq(&pool, session.factors.as_ref().unwrap()));
        assert_eq!(counted(&fresh, "linear.backend_objects.reused"), 0);
        assert!(counted(&fresh, "linear.backend_objects.created") > 0);
    }
    #[test]
    fn native_layout_and_effective_factor_profile_invalidate_retained_backends_and_require_reuse_refuses()
     {
        let mut session = Session::new();
        let stamp = crate::solver_tests::stamp(Backend::Pounce);
        reuse_run(
            &mut session,
            &Settings::default(),
            ReusePolicy::AllowRebuild,
            stamp.clone(),
        )
        .unwrap();
        let pool = session.factors.as_ref().unwrap().clone();
        let mut changed = Settings::default();
        changed.linear.refine = !changed.linear.refine;
        assert!(matches!(
            reuse_run(
                &mut session,
                &changed,
                ReusePolicy::RequireReuse,
                stamp.clone()
            ),
            Err(ProblemError::Reuse { .. })
        ));
        assert!(Rc::ptr_eq(&pool, session.factors.as_ref().unwrap()));
        assert!(session.app.is_some());
        let updated = reuse_run(
            &mut session,
            &changed,
            ReusePolicy::AllowRebuild,
            stamp.clone(),
        )
        .unwrap();
        assert_eq!(updated.termination.category, Termination::Success);
        assert!(!Rc::ptr_eq(&pool, session.factors.as_ref().unwrap()));
        assert_eq!(counted(&updated, "linear.backend_objects.reused"), 0);
        let current = session.factors.as_ref().unwrap().clone();
        let mut layout = stamp;
        layout.layout = pse_ids::ContentHash::from_bytes([62; 32]);
        assert!(matches!(
            reuse_run(
                &mut session,
                &changed,
                ReusePolicy::RequireReuse,
                layout.clone()
            ),
            Err(ProblemError::Reuse { .. })
        ));
        assert!(Rc::ptr_eq(&current, session.factors.as_ref().unwrap()));
        let rebuilt = reuse_run(&mut session, &changed, ReusePolicy::AllowRebuild, layout).unwrap();
        assert_eq!(rebuilt.termination.category, Termination::Success);
        assert!(!Rc::ptr_eq(&current, session.factors.as_ref().unwrap()));
        assert_eq!(counted(&rebuilt, "linear.backend_objects.reused"), 0);
    }
    #[test]
    fn pounce_retry_options_reserved_and_snapshotted() {
        for key in PINNED_OFF.iter().chain(&RESERVED_METHODS) {
            let options = Options::from([(key.to_string(), OptionValue::Bool(true))]);
            assert!(
                run(&mut Session::new(), options, 1).is_err(),
                "{key} must be reserved"
            );
        }
        let report = run(&mut Session::new(), Options::new(), 1).unwrap();
        for key in PINNED_OFF.iter().chain(&RESERVED_METHODS) {
            assert_eq!(report.options[*key], OptionValue::Bool(false), "{key}");
        }
        // The retry is default-on upstream; the snapshot records that it was pinned off.
        assert_eq!(
            report.native_defaults["mu_strategy_fallback"],
            OptionValue::Bool(true)
        );
        assert_eq!(
            report.options["linear_solver"],
            OptionValue::Text("feral".into())
        );
        assert_eq!(report.options["max_iter"], OptionValue::Integer(3000));
        assert!(report.options.len() > 100);
        assert!(report.options.len() >= report.native_defaults.len());
    }
    #[test]
    fn library_rungs_act_on_typed_factories_without_accumulating_options() {
        let mut report = run(&mut Session::new(), Options::new(), 1).unwrap();
        report.termination.code =
            i64::from(ApplicationReturnStatus::InfeasibleProblemDetected.as_int());
        let baseline = Controls::default();
        let settings = Settings::default();
        let rungs = second_opinion_profiles(&baseline, &settings, &report, true).unwrap();
        let mc64 = rungs
            .iter()
            .find(|rung| rung.label == "feral_scaling=mc64")
            .unwrap();
        assert!(matches!(
            mc64.settings.linear.scaling,
            feral::scaling::ScalingStrategy::Mc64Symmetric
        ));
        assert!(!mc64.controls.options.contains_key("mu_strategy"));
        let adaptive = rungs
            .iter()
            .find(|rung| rung.label == "mu_strategy=adaptive")
            .unwrap();
        assert!(matches!(
            adaptive.settings.linear.scaling,
            feral::scaling::ScalingStrategy::Auto
        ));
        let perturbed = rungs.iter().find(|rung| rung.replaces_start).unwrap();
        assert!(!perturbed.controls.options.contains_key("mu_strategy"));
        assert!(
            second_opinion_profiles(&baseline, &settings, &report, false)
                .unwrap()
                .iter()
                .all(|rung| !rung.replaces_start)
        );
        let acting = reuse_run(
            &mut Session::new(),
            &mc64.settings,
            ReusePolicy::Fresh,
            crate::solver_tests::stamp(Backend::Pounce),
        )
        .unwrap();
        let effective: serde_json::Value =
            serde_json::from_str(&acting.provenance["feral.effective"]).unwrap();
        assert_eq!(effective["scaling"], serde_json::json!("mc64_symmetric"));
        report.evidence.callback.terminal_failure = true;
        assert!(
            second_opinion_profiles(&baseline, &settings, &report, true)
                .unwrap()
                .is_empty()
        );
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
