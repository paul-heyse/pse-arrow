// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Full retained egobox/TREGO iteration. Every result is a proposal for original correction.
use crate::MathError;
use argmin::core::{CostFunction, Problem, Solver, State};
use egobox_ego::{
    Constraints, Cstr, EgorConfig, EgorSolver, EgorState, InfillOptimizer, RuntimeFlags, to_xtypes,
};
/// Exact library hyperparameter tuning contract.
pub use egobox_gp::ThetaTuning;
use egobox_moe::GpMixtureParams;
/// Exact library GP model specification flags.
pub use egobox_moe::{CorrelationSpec, RegressionSpec};
use ndarray::Array2;
use pse_ids::{ContentHash, FramedHasher, SemanticId};
use pse_kernels::ExecutionScope;
use std::{
    collections::BTreeSet,
    fmt::Debug,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Arc, Mutex},
    time::Duration,
};

/// Immutable observable correspondence. Fidelity may change approximation quality, never the target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FidelityCorrespondence {
    /// Original scientific observable and interpretation.
    pub original_target: ContentHash,
    /// Approximation's declared scientific observable and interpretation; must match the original.
    pub model_target: ContentHash,
    /// Actual evaluator/configuration source.
    pub source: ContentHash,
    /// Authored fidelity level and realization.
    pub fidelity: ContentHash,
    /// Ordered original coordinate identities in physical canonical units.
    pub coordinates: Vec<SemanticId>,
    /// Ordered observable identities: objective then constraint violations.
    pub outputs: Vec<SemanticId>,
}
impl FidelityCorrespondence {
    /// Refuse target substitution, missing coordinates, duplicate identities and absent objective.
    pub fn validate(&self) -> Result<(), MathError> {
        if self.original_target != self.model_target
            || self.coordinates.is_empty()
            || self.outputs.is_empty()
            || self.coordinates.iter().collect::<BTreeSet<_>>().len() != self.coordinates.len()
            || self.outputs.iter().collect::<BTreeSet<_>>().len() != self.outputs.len()
        {
            return Err(MathError::Contract(
                "surrogate fidelity changes scientific target or observable inventory".into(),
            ));
        }
        Ok(())
    }
}
/// Every callback invocation evaluates this declared fidelity and is counted, including failed evaluations.
pub trait FidelityEvaluator: Debug + Send + 'static {
    /// Original typed mathematical/native failure.
    type Error: From<MathError> + std::error::Error + Send + 'static;
    /// Actual correspondence supplied by the evaluator owner.
    fn correspondence(&self) -> &FidelityCorrespondence;
    /// Fill objective followed by explicit signed constraint violations (all <=0 is the declared model criterion).
    fn evaluate(&mut self, coordinates: &[f64], outputs: &mut [f64]) -> Result<(), Self::Error>;
}
/// Finite library/task controls; no inferred scientific acceptance tolerance.
#[derive(Clone, Debug)]
pub struct SurrogateOptions {
    /// Finite physical coordinate intervals in correspondence order.
    pub bounds: Vec<(f64, f64)>,
    /// Explicit initial input points; evaluations cannot be silently imported from another fidelity.
    pub initial: Vec<Vec<f64>>,
    /// Maximum retained library iterations.
    pub iterations: u64,
    /// Inclusive callback point budget (initial samples and failed/repeated trials included).
    pub evaluations: u64,
    /// Finite multistart count for library infill optimization.
    pub infill_starts: usize,
    /// One explicit GP mean-model specification, owned by the library.
    pub regression: RegressionSpec,
    /// One explicit GP correlation-model specification, owned by the library.
    pub correlation: CorrelationSpec,
    /// Actual GP hyperparameter tuning values/bounds; no hidden optimization fallback.
    pub theta: ThetaTuning<f64>,
    /// Finite library GP optimizer starts.
    pub gp_starts: usize,
    /// Finite GP optimization function-call cap.
    pub gp_evaluations: usize,
    /// Explicit reproducibility seed.
    pub seed: u64,
    /// TREGO global/local phase lengths.
    pub phase_steps: (usize, usize),
    /// TREGO initial normalized trust radius.
    pub radius: f64,
    /// TREGO radius contraction factor.
    pub contraction: f64,
    /// Admitted rayon worker count.
    pub threads: usize,
    /// Accounted retained worker stack bytes, supplied by the task policy.
    pub stack_bytes: usize,
    /// Accounted opaque library allowance retained for the whole solver/state lifetime.
    pub foreign_bytes: usize,
    /// Finite local time, capped by the original execution scope.
    pub time: Duration,
}
impl SurrogateOptions {
    fn validate(&self, c: &FidelityCorrespondence) -> Result<(), MathError> {
        c.validate()?;
        if self.bounds.len() != c.coordinates.len()
            || self.initial.len() < 2
            || self.iterations == 0
            || self.evaluations < self.initial.len() as u64
            || self.infill_starts == 0
            || self.gp_evaluations == 0
            || self.threads == 0
            || self.stack_bytes == 0
            || self.foreign_bytes == 0
            || self.time.is_zero()
            || !self.radius.is_finite()
            || self.radius <= 0.0
            || !self.contraction.is_finite()
            || !(0.0..1.0).contains(&self.contraction)
            || self.phase_steps.0 == 0
            || self.phase_steps.1 == 0
            || self
                .bounds
                .iter()
                .any(|&(a, b)| !a.is_finite() || !b.is_finite() || a >= b)
            || self.initial.iter().any(|row| {
                row.len() != self.bounds.len()
                    || row
                        .iter()
                        .zip(&self.bounds)
                        .any(|(&x, &(a, b))| !x.is_finite() || x < a || x > b)
            })
        {
            return Err(MathError::Contract(
                "surrogate finite extent, bounds, DOE or TREGO controls".into(),
            ));
        }
        if self.regression.bits().count_ones() != 1
            || self.correlation.bits().count_ones() != 1
            || !self.regression.difference(RegressionSpec::ALL).is_empty()
            || !self.correlation.difference(CorrelationSpec::ALL).is_empty()
            || self.theta.init().is_empty()
            || ![1, self.bounds.len()].contains(&self.theta.init().len())
            || self
                .theta
                .init()
                .iter()
                .any(|v| !v.is_finite() || *v <= 0.0)
            || (self.gp_starts == 0 && !matches!(self.theta, ThetaTuning::Fixed(_)))
        {
            return Err(MathError::Contract(
                "surrogate explicit GP model or hyperparameter tuning".into(),
            ));
        }
        if let Some(bounds) = self.theta.bounds()
            && (bounds.len() != self.theta.init().len()
                || bounds.iter().zip(self.theta.init()).any(|(&(a, b), &v)| {
                    !a.is_finite() || !b.is_finite() || a <= 0.0 || a >= b || v < a || v > b
                }))
        {
            return Err(MathError::Contract(
                "surrogate GP hyperparameter bounds".into(),
            ));
        }
        if let ThetaTuning::Partial { active, .. } = &self.theta
            && (active.is_empty()
                || active.iter().copied().collect::<BTreeSet<_>>().len() != active.len()
                || active.iter().any(|&i| i >= self.theta.init().len()))
        {
            return Err(MathError::Contract(
                "surrogate GP active hyperparameters".into(),
            ));
        }
        usize::try_from(self.evaluations)
            .map_err(|_| MathError::Limit("surrogate sample extent"))?;
        usize::try_from(self.iterations)
            .map_err(|_| MathError::Limit("surrogate iteration extent"))?;
        Ok(())
    }
    /// Conservative dense-kernel reservation plus the supplied opaque foreign allowance and retained thread stacks.
    /// This is accounted admission, not interception of the library allocator.
    pub fn workspace_bytes(&self, c: &FidelityCorrespondence) -> Result<usize, MathError> {
        self.validate(c)?;
        let n = usize::try_from(self.evaluations)
            .map_err(|_| MathError::Limit("surrogate sample extent"))?;
        let d = c.coordinates.len();
        let m = c.outputs.len();
        // Simultaneous covariance/factor/inverse/gradient work, model outputs and multistart clone work.
        let matrix = n
            .checked_mul(n)
            .and_then(|v| v.checked_mul(d.checked_add(8)?))
            .and_then(|v| v.checked_mul(m));
        let samples = n
            .checked_mul(
                d.checked_add(m)
                    .ok_or(MathError::Limit("surrogate workspace"))?,
            )
            .and_then(|v| v.checked_mul(8));
        matrix
            .and_then(|v| v.checked_add(samples?))
            .and_then(|v| v.checked_mul(size_of::<f64>()))
            .and_then(|v| v.checked_mul(self.threads.checked_add(1)?))
            .and_then(|v| v.checked_add(self.foreign_bytes))
            .and_then(|v| {
                v.checked_add(self.threads.checked_add(1)?.checked_mul(self.stack_bytes)?)
            })
            .ok_or(MathError::Limit("surrogate workspace"))
    }
    /// Full identity of the actual task, including scientific correspondence and finite options.
    pub fn key(&self, c: &FidelityCorrespondence) -> Result<ContentHash, MathError> {
        self.validate(c)?;
        let mut h = FramedHasher::new(pse_ids::Frame::MathSurrogateTaskV1);
        for k in [c.original_target, c.model_target, c.source, c.fidelity] {
            h.part(k.as_bytes());
        }
        for ids in [&c.coordinates, &c.outputs] {
            h.u64(ids.len() as u64);
            for id in ids {
                h.part(id.as_bytes());
            }
        }
        for &(a, b) in &self.bounds {
            h.u64(a.to_bits());
            h.u64(b.to_bits());
        }
        h.u64(u64::from(self.regression.bits()))
            .u64(u64::from(self.correlation.bits()));
        h.u64(match &self.theta {
            ThetaTuning::Fixed(_) => 0,
            ThetaTuning::Full { .. } => 1,
            ThetaTuning::Partial { .. } => 2,
        });
        h.u64(self.theta.init().len() as u64);
        for &v in self.theta.init() {
            h.f64(v);
        }
        if let Some(bounds) = self.theta.bounds() {
            for &(a, b) in bounds {
                h.f64(a).f64(b);
            }
        }
        if let ThetaTuning::Partial { active, .. } = &self.theta {
            h.u64(active.len() as u64);
            for &i in active {
                h.u64(i as u64);
            }
        }
        h.u64(self.initial.len() as u64);
        for row in &self.initial {
            for v in row {
                h.u64(v.to_bits());
            }
        }
        for v in [
            self.iterations,
            self.evaluations,
            self.infill_starts as u64,
            self.gp_starts as u64,
            self.gp_evaluations as u64,
            self.seed,
            self.phase_steps.0 as u64,
            self.phase_steps.1 as u64,
            self.radius.to_bits(),
            self.contraction.to_bits(),
            self.threads as u64,
            self.stack_bytes as u64,
            self.foreign_bytes as u64,
            self.time.as_secs(),
            u64::from(self.time.subsec_nanos()),
        ] {
            h.u64(v);
        }
        Ok(h.finish_hash())
    }
}
/// First terminal source cause survives egobox's internal NaN/rejection handling.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[diagnostic(code(math::library))]
pub enum SurrogateFailure<E: std::error::Error + 'static> {
    /// Exact evaluator failure; never recast as a bad GP fit.
    #[error("surrogate evaluator: {0}")]
    Callback(#[source] E),
    /// Typed mathematical admission/cancellation/resource failure.
    #[error(transparent)]
    Math(#[from] MathError),
    /// Caught callback or library panic; cannot become a recoverable failed sample.
    #[error("surrogate library panic: {0}")]
    Panic(String),
}
/// Actual work counts include partial failed/interrupted attempts; opaque factor counts stay unknown.
#[derive(Clone, Copy, Debug, Default)]
pub struct SurrogateWork {
    /// Callback points whose evaluator was entered, including failure.
    pub evaluations: u64,
    /// Library iteration calls actually entered, including failure.
    pub iterations: u64,
    /// Library initialization calls actually entered.
    pub initializations: u64,
}
impl SurrogateWork {
    /// Actual inclusive observations. GP/infill factorization work is opaque and cannot satisfy a strict factor budget.
    pub fn observation(self) -> pse_model::strategy::WorkObservation {
        pse_model::strategy::WorkObservation {
            attempts: self.initializations,
            evaluations: Some(self.evaluations),
            iterations: Some(self.iterations),
            factorizations: None,
            proof_steps: Some(0),
        }
    }
}
/// Library statistical proposal; it conveys no certified accuracy or original feasibility permission.
#[derive(Clone, Debug)]
pub struct SurrogateProposal {
    owner: Option<Arc<dyn crate::AllocationOwner>>,
    /// Original target that must independently screen and correct this point.
    pub original_target: ContentHash,
    /// Actual approximation fidelity that generated/evaluated it.
    pub fidelity: ContentHash,
    /// Complete solver task identity.
    pub task: ContentHash,
    /// Shared physical original-coordinate point; sample clones retain the same allocation.
    pub coordinates: Arc<Vec<f64>>,
    /// Model objective/constraint values, never original evidence unless separately assessed.
    pub model_values: Arc<Vec<f64>>,
    /// Actual library infill statistic; statistical selection is not a numerical certificate.
    pub infill_statistic: Option<f64>,
    /// Retained library TREGO trust radius.
    pub radius: f64,
    /// Retained TREGO global phase iterations.
    pub global_iterations: usize,
    /// Retained TREGO local phase iterations.
    pub local_iterations: usize,
}
impl SurrogateProposal {
    /// Escaping point/statistic buffer extent, separate from retained optimizer storage.
    /// # Errors
    /// Allocation extent overflow.
    pub fn retained_bytes(&self) -> Result<usize, MathError> {
        self.coordinates
            .capacity()
            .checked_add(self.model_values.capacity())
            .and_then(|n| n.checked_mul(size_of::<f64>()))
            .and_then(|n| n.checked_add(size_of::<Self>()))
            .ok_or(MathError::Limit("surrogate proposal extent"))
    }
    /// Keep the runtime's escaping-product reservation with every shared owner.
    pub fn with_owner(mut self, owner: Arc<dyn crate::AllocationOwner>) -> Self {
        self.owner = Some(crate::retain_allocation_owner(self.owner.take(), owner));
        self
    }
}
struct Callback<E: FidelityEvaluator> {
    evaluator: E,
    scope: ExecutionScope,
    limit: u64,
    work: SurrogateWork,
    terminal: Option<SurrogateFailure<E::Error>>,
    disabled: bool,
    abort: Option<Arc<std::sync::atomic::AtomicBool>>,
}
struct Evaluated<E: FidelityEvaluator>(Arc<Mutex<Callback<E>>>);
impl<E: FidelityEvaluator> CostFunction for Evaluated<E> {
    type Param = Array2<f64>;
    type Output = Array2<f64>;
    #[allow(
        clippy::disallowed_types,
        reason = "argmin 0.11 CostFunction requires this Error alias; it only stops the library while the original typed failure remains in Callback::terminal"
    )]
    fn cost(&self, x: &Self::Param) -> Result<Self::Output, argmin::core::Error> {
        let mut cb = self
            .0
            .lock()
            .map_err(|_| argmin::core::Error::msg("surrogate callback lock poisoned"))?;
        if cb.disabled || cb.terminal.is_some() {
            return Err(argmin::core::Error::msg("surrogate terminal latch"));
        }
        let m = cb.evaluator.correspondence().outputs.len();
        let d = cb.evaluator.correspondence().coordinates.len();
        let mut result = Array2::zeros((x.nrows(), m));
        for (i, row) in x.rows().into_iter().enumerate() {
            let failure = if cb
                .abort
                .as_ref()
                .is_some_and(|flag| flag.load(std::sync::atomic::Ordering::Acquire))
            {
                Some(SurrogateFailure::Math(MathError::Cancelled))
            } else if let Err(e) = cb.scope.check() {
                Some(SurrogateFailure::Math(scope_math(e)))
            } else if cb.work.evaluations >= cb.limit {
                Some(SurrogateFailure::Math(MathError::Limit(
                    "surrogate callback evaluations",
                )))
            } else if row.len() != d || row.iter().any(|v| !v.is_finite()) {
                Some(SurrogateFailure::Math(MathError::Contract(
                    "surrogate callback coordinates".into(),
                )))
            } else {
                None
            };
            if let Some(e) = failure {
                cb.terminal = Some(e);
                cb.disabled = true;
                return Err(argmin::core::Error::msg("surrogate terminal latch"));
            }
            let coordinates = row.to_vec();
            let mut values = vec![0.0; m];
            cb.work.evaluations += 1;
            match catch_unwind(AssertUnwindSafe(|| {
                cb.evaluator.evaluate(&coordinates, &mut values)
            })) {
                Ok(Ok(())) if values.iter().all(|v| v.is_finite()) => {}
                Ok(Ok(())) => {
                    cb.terminal = Some(SurrogateFailure::Math(MathError::Contract(
                        "nonfinite surrogate evaluator output".into(),
                    )));
                }
                Ok(Err(e)) => cb.terminal = Some(SurrogateFailure::Callback(e)),
                Err(_) => {
                    cb.terminal = Some(SurrogateFailure::Panic("evaluator callback unwound".into()))
                }
            }
            if cb.terminal.is_some() {
                cb.disabled = true;
                return Err(argmin::core::Error::msg("surrogate terminal latch"));
            }
            for (j, value) in values.into_iter().enumerate() {
                result[[i, j]] = value;
            }
        }
        Ok(result)
    }
}
impl<E: FidelityEvaluator> Constraints<Cstr> for Evaluated<E> {
    fn constraints(&self) -> &[impl egobox_ego::CstrFn] {
        {
            let empty: &[Cstr] = &[];
            empty
        }
    }
}
/// One worker owns the full library solver and state, including TREGO, DOE, RNG and learned tuning.
struct Worker<E: FidelityEvaluator> {
    solver: EgorSolver<GpMixtureParams<f64>>,
    state: Option<EgorState<f64>>,
    problem: Problem<Evaluated<E>>,
    callback: Arc<Mutex<Callback<E>>>,
    correspondence: FidelityCorrespondence,
    options: SurrogateOptions,
    key: ContentHash,
    workspace: usize,
    disabled: bool,
}
impl<E: FidelityEvaluator> Debug for Worker<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RetainedSurrogate")
            .field("key", &self.key)
            .field("initialized", &self.state.is_some())
            .field("disabled", &self.disabled)
            .finish()
    }
}
impl<E: FidelityEvaluator> Worker<E> {
    /// Construct only inside a task with the requested CPU and workspace admission; no evaluation yet.
    fn new(
        evaluator: E,
        options: SurrogateOptions,
        scope: ExecutionScope,
    ) -> Result<Self, SurrogateFailure<E::Error>> {
        let correspondence = evaluator.correspondence().clone();
        options.validate(&correspondence)?;
        let remaining = scope
            .remaining(options.time)
            .map_err(|e| SurrogateFailure::Math(scope_math(e)))?;
        let deadline = std::time::Instant::now()
            .checked_add(remaining)
            .ok_or(MathError::Limit("surrogate deadline extent"))?;
        let scope = ExecutionScope::new(scope.cancellation().clone(), Some(deadline));
        let workspace = options.workspace_bytes(&correspondence)?;
        let key = options.key(&correspondence)?;
        let bounds = Array2::from_shape_vec(
            (options.bounds.len(), 2),
            options.bounds.iter().flat_map(|&(a, b)| [a, b]).collect(),
        )
        .map_err(|e| MathError::Library(e.to_string()))?;
        let initial = Array2::from_shape_vec(
            (options.initial.len(), options.bounds.len()),
            options.initial.iter().flatten().copied().collect(),
        )
        .map_err(|e| MathError::Library(e.to_string()))?;
        let config = EgorConfig::default()
            .xtypes(&to_xtypes(&bounds))
            .doe(&initial)
            .n_cstr(correspondence.outputs.len() - 1)
            .cstr_tol(ndarray::Array1::zeros(correspondence.outputs.len() - 1))
            .seed(options.seed)
            .max_iters(options.iterations as usize)
            .n_start(options.infill_starts)
            .infill_optimizer(InfillOptimizer::Cobyla)
            .configure_gp(|g| {
                g.regression_spec(options.regression)
                    .correlation_spec(options.correlation)
                    .theta_tuning(options.theta.clone())
                    .n_start(options.gp_starts)
                    .max_eval(options.gp_evaluations)
            })
            .configure_trego(|t| {
                t.n_gl_steps(options.phase_steps)
                    .sigma0(options.radius)
                    .beta(options.contraction)
            })
            .no_outdir()
            .runtime_flags(RuntimeFlags::none())
            .timeout(remaining.as_secs_f64())
            .check()
            .map_err(|e| MathError::Library(e.to_string()))?;
        let callback = Arc::new(Mutex::new(Callback {
            evaluator,
            scope,
            limit: options.evaluations,
            work: SurrogateWork::default(),
            terminal: None,
            disabled: false,
            abort: None,
        }));
        let problem = Problem::new(Evaluated(callback.clone()));
        Ok(Self {
            solver: EgorSolver::new(config),
            state: None,
            problem,
            callback,
            correspondence,
            options,
            key,
            workspace,
            disabled: false,
        })
    }
    /// Full task key; mismatched source/target/bounds/options require a new owner.
    fn key(&self) -> ContentHash {
        self.key
    }
    /// Retained conservative reservation, including opaque foreign allocation allowance and worker stacks.
    fn retained_bytes(&self) -> usize {
        self.workspace
    }
    /// Whether the library or finite iteration cap has ended proposal generation.
    fn is_finished(&self) -> bool {
        self.disabled
            || self.state.as_ref().is_some_and(|state| {
                state.iter >= self.options.iterations
                    || !matches!(
                        state.termination_status,
                        argmin::core::TerminationStatus::NotTerminated
                    )
            })
    }
    /// Initialize once or advance one retained full-library iteration. The caller admits this operation's CPU team.
    fn advance(&mut self) -> Result<Option<SurrogateProposal>, SurrogateFailure<E::Error>> {
        if self.disabled {
            return Err(MathError::Contract("surrogate task is terminal".into()).into());
        }
        let initialize = self.state.is_none();
        {
            let mut cb = self
                .callback
                .lock()
                .map_err(|_| MathError::Library("surrogate callback lock poisoned".into()))?;
            cb.scope.check().map_err(scope_math)?;
            if cb
                .abort
                .as_ref()
                .is_some_and(|flag| flag.load(std::sync::atomic::Ordering::Acquire))
            {
                return Err(MathError::Cancelled.into());
            }
            if !initialize && (cb.work.iterations >= self.options.iterations || self.is_finished())
            {
                return Ok(self.proposal());
            }
            if initialize {
                cb.work.initializations += 1;
            } else {
                cb.work.iterations += 1;
            }
        }
        let state = self
            .state
            .take()
            .unwrap_or_else(|| EgorState::new().max_iters(self.options.iterations));
        let result = catch_unwind(AssertUnwindSafe(|| {
            if initialize {
                self.solver.init(&mut self.problem, state)
            } else {
                self.solver.next_iter(&mut self.problem, state)
            }
        }));
        let terminal = self
            .callback
            .lock()
            .map_err(|_| MathError::Library("surrogate callback lock poisoned".into()))?
            .terminal
            .take();
        if let Some(e) = terminal {
            self.disabled = true;
            return Err(e);
        }
        let (mut state, _) = match result {
            Ok(Ok(v)) => v,
            Ok(Err(e)) => {
                self.disabled = true;
                return Err(MathError::Library(e.to_string()).into());
            }
            Err(_) => {
                self.disabled = true;
                return Err(SurrogateFailure::Panic("egobox iteration unwound".into()));
            }
        };
        if !initialize {
            state.increment_iter();
        }
        state.update();
        self.state = Some(state);
        if let Err(e) = self
            .callback
            .lock()
            .map_err(|_| MathError::Library("surrogate callback lock poisoned".into()))?
            .scope
            .check()
        {
            self.disabled = true;
            return Err(scope_math(e).into());
        }
        Ok(self.proposal())
    }
    /// A model-selected point always requires independent original screening/correction.
    fn proposal(&self) -> Option<SurrogateProposal> {
        if self.disabled {
            return None;
        }
        let state = self.state.as_ref()?;
        let data = state.surrogate.data.as_ref()?;
        let index = state.surrogate.best_index?;
        Some(SurrogateProposal {
            owner: None,
            original_target: self.correspondence.original_target,
            fidelity: self.correspondence.fidelity,
            task: self.key,
            coordinates: Arc::new(data.0.row(index).to_vec()),
            model_values: Arc::new(data.1.row(index).to_vec()),
            infill_statistic: state
                .surrogate
                .infill_value
                .is_finite()
                .then_some(state.surrogate.infill_value),
            radius: state.trego.sigma,
            global_iterations: state.trego.global_trego_iter,
            local_iterations: state.trego.local_trego_iter,
        })
    }
}

type Snapshot = (Option<EgorState<f64>>, Option<SurrogateProposal>);
enum Command<E: std::error::Error + 'static> {
    Advance(std::sync::mpsc::SyncSender<Result<Snapshot, SurrogateFailure<E>>>),
    Stop(std::sync::mpsc::SyncSender<()>),
}
/// Channel owner of the full solver/state, permanently retained inside one actual local-rayon worker.
/// Library non-Send strategies never cross threads. Read-only snapshots do not replace worker state.
pub struct RetainedSurrogate<E: FidelityEvaluator> {
    commands: std::sync::mpsc::Sender<Command<E::Error>>,
    pool: Option<rayon::ThreadPool>,
    callback: Arc<Mutex<Callback<E>>>,
    state: Option<EgorState<f64>>,
    proposal: Option<SurrogateProposal>,
    key: ContentHash,
    workspace: usize,
    disabled: bool,
    iterations: u64,
}
impl<E: FidelityEvaluator> Debug for RetainedSurrogate<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RetainedSurrogate")
            .field("key", &self.key)
            .field("initialized", &self.state.is_some())
            .field("disabled", &self.disabled)
            .finish()
    }
}
impl<E: FidelityEvaluator> Drop for RetainedSurrogate<E> {
    fn drop(&mut self) {
        let (reply, finished) = std::sync::mpsc::sync_channel(1);
        if self.commands.send(Command::Stop(reply)).is_ok() {
            let _ = finished.recv();
        }
        // Stop's receipt is sent after actual solver/state destruction on the owning worker.
        drop(self.pool.take());
    }
}
impl<E: FidelityEvaluator> RetainedSurrogate<E> {
    /// Construct once inside an admitted task. The complete library object remains inside its local pool.
    pub fn new(
        evaluator: E,
        options: SurrogateOptions,
        scope: ExecutionScope,
    ) -> Result<Self, SurrogateFailure<E::Error>> {
        options.validate(evaluator.correspondence())?;
        let allowance = scope.remaining(options.time).map_err(scope_math)?;
        let deadline = std::time::Instant::now()
            .checked_add(allowance)
            .ok_or(MathError::Limit("surrogate deadline extent"))?;
        let scope = ExecutionScope::new(scope.cancellation().clone(), Some(deadline));
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(options.threads)
            .stack_size(options.stack_bytes)
            .build()
            .map_err(|e| MathError::Library(e.to_string()))?;
        let iterations = options.iterations;
        let (commands, receiver) = std::sync::mpsc::channel();
        let (ready, initialized) = std::sync::mpsc::sync_channel(1);
        pool.spawn(move || {
            let built = catch_unwind(AssertUnwindSafe(|| Worker::new(evaluator, options, scope)));
            let mut worker = match built {
                Ok(Ok(worker)) => worker,
                Ok(Err(error)) => {
                    let _ = ready.send(Err(error));
                    return;
                }
                Err(_) => {
                    let _ = ready.send(Err(SurrogateFailure::Panic(
                        "egobox construction unwound".into(),
                    )));
                    return;
                }
            };
            if ready
                .send(Ok((
                    worker.callback.clone(),
                    worker.key(),
                    worker.retained_bytes(),
                )))
                .is_err()
            {
                return;
            }
            while let Ok(command) = receiver.recv() {
                match command {
                    Command::Advance(reply) => {
                        let result = catch_unwind(AssertUnwindSafe(|| {
                            worker
                                .advance()
                                .map(|proposal| (worker.state.clone(), proposal))
                        }));
                        let result = match result {
                            Ok(result) => result,
                            Err(_) => {
                                worker.disabled = true;
                                Err(SurrogateFailure::Panic(
                                    "egobox state update unwound".into(),
                                ))
                            }
                        };
                        let _ = reply.send(result);
                    }
                    Command::Stop(reply) => {
                        drop(worker);
                        let _ = reply.send(());
                        return;
                    }
                }
            }
        });
        let (callback, key, workspace) = initialized.recv().map_err(|_| {
            SurrogateFailure::Panic("surrogate worker initialization lost".into())
        })??;
        Ok(Self {
            commands,
            pool: Some(pool),
            callback,
            state: None,
            proposal: None,
            key,
            workspace,
            disabled: false,
            iterations,
        })
    }
    /// Original scope remains unchanged; supervisor cancellation is a separate additional observer.
    pub fn bind_abort(
        &mut self,
        flag: Arc<std::sync::atomic::AtomicBool>,
    ) -> Result<(), MathError> {
        let mut cb = self
            .callback
            .lock()
            .map_err(|_| MathError::Library("surrogate callback lock poisoned".into()))?;
        if cb
            .abort
            .as_ref()
            .is_some_and(|old| !Arc::ptr_eq(old, &flag))
        {
            return Err(MathError::Contract(
                "surrogate abort owner already bound".into(),
            ));
        }
        cb.abort = Some(flag);
        Ok(())
    }
    /// Complete consumed source, correspondence, physical domain and option identity.
    pub fn key(&self) -> ContentHash {
        self.key
    }
    /// Conservative retained admission extent; not foreign allocator interception.
    pub fn retained_bytes(&self) -> usize {
        self.workspace
    }
    /// Inclusive actual work, including failure, readable without replaying any proposal.
    pub fn work(&self) -> Result<SurrogateWork, MathError> {
        self.callback
            .lock()
            .map(|cb| cb.work)
            .map_err(|_| MathError::Library("surrogate callback lock poisoned".into()))
    }
    /// Read-only actual library-state snapshot; the worker's full state remains retained independently.
    pub fn state(&self) -> Option<&EgorState<f64>> {
        self.state.as_ref()
    }
    /// Library termination or the explicit iteration cap prevents further iteration calls.
    pub fn is_finished(&self) -> bool {
        self.disabled
            || self.state.as_ref().is_some_and(|state| {
                state.iter >= self.iterations
                    || !matches!(
                        state.termination_status,
                        argmin::core::TerminationStatus::NotTerminated
                    )
            })
    }
    /// Enter one call on the same actual library owner. The caller supplies active CPU admission.
    pub fn advance(&mut self) -> Result<Option<SurrogateProposal>, SurrogateFailure<E::Error>> {
        if self.disabled {
            return Err(MathError::Contract("surrogate task is terminal".into()).into());
        }
        let (reply, response) = std::sync::mpsc::sync_channel(1);
        if self.commands.send(Command::Advance(reply)).is_err() {
            self.disabled = true;
            return Err(SurrogateFailure::Panic("surrogate worker exited".into()));
        }
        let result = response
            .recv()
            .map_err(|_| SurrogateFailure::Panic("surrogate worker response lost".into()))?;
        match result {
            Ok((state, proposal)) => {
                self.state = state;
                self.proposal = proposal;
                Ok(self.proposal())
            }
            Err(error) => {
                self.disabled = true;
                self.proposal = None;
                Err(error)
            }
        }
    }
    /// Narrow one active library operation under the existing cancellation owner.
    /// The retained task clock is never renewed; callback/provider checkpoints consume
    /// the minimum of the original task and caller phase deadlines during iteration.
    /// # Errors
    /// Changed cancellation owner, expired scope or the original typed library/callback stop.
    pub fn advance_within(
        &mut self,
        scope: ExecutionScope,
    ) -> Result<Option<SurrogateProposal>, SurrogateFailure<E::Error>> {
        let prior = {
            let mut callback = self
                .callback
                .lock()
                .map_err(|_| MathError::Library("surrogate callback lock poisoned".into()))?;
            if !Arc::ptr_eq(callback.scope.cancellation(), scope.cancellation()) {
                return Err(MathError::Contract(
                    "surrogate phase cancellation owner mismatch".into(),
                )
                .into());
            }
            scope.check().map_err(scope_math)?;
            callback.scope.check().map_err(scope_math)?;
            let deadline = callback
                .scope
                .deadline()
                .into_iter()
                .chain(scope.deadline())
                .min();
            let prior = callback.scope.clone();
            callback.scope = ExecutionScope::new(prior.cancellation().clone(), deadline);
            prior
        };
        let result = self.advance();
        self.callback
            .lock()
            .map_err(|_| MathError::Library("surrogate callback lock poisoned".into()))?
            .scope = prior;
        result
    }
    /// A statistical proposal has no original scientific permission or certified numerical accuracy.
    pub fn proposal(&self) -> Option<SurrogateProposal> {
        if self.disabled {
            None
        } else {
            self.proposal.clone()
        }
    }
}

fn scope_math(error: pse_kernels::ProviderError) -> MathError {
    crate::error::scope_error(error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    #[derive(Debug)]
    struct Evaluator {
        correspondence: FidelityCorrespondence,
        calls: Arc<AtomicUsize>,
        fail: bool,
        cancel: Option<Arc<AtomicBool>>,
        threads: usize,
    }
    impl FidelityEvaluator for Evaluator {
        type Error = MathError;
        fn correspondence(&self) -> &FidelityCorrespondence {
            &self.correspondence
        }
        fn evaluate(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), MathError> {
            assert_eq!(rayon::current_num_threads(), self.threads);
            self.calls.fetch_add(1, Ordering::SeqCst);
            if self.fail {
                return Err(MathError::Domain {
                    source_id: SemanticId::from_bytes([33; 16]),
                    requirement: "actual sentinel fidelity refusal",
                });
            }
            out[0] = (x[0] - 0.37).powi(2);
            if let Some(cancel) = &self.cancel {
                cancel.store(true, Ordering::Release);
            }
            Ok(())
        }
    }
    fn correspondence() -> FidelityCorrespondence {
        FidelityCorrespondence {
            original_target: ContentHash::from_bytes([1; 32]),
            model_target: ContentHash::from_bytes([1; 32]),
            source: ContentHash::from_bytes([2; 32]),
            fidelity: ContentHash::from_bytes([3; 32]),
            coordinates: vec![SemanticId::from_bytes([4; 16])],
            outputs: vec![SemanticId::from_bytes([5; 16])],
        }
    }
    fn options() -> SurrogateOptions {
        SurrogateOptions {
            bounds: vec![(0.0, 1.0)],
            initial: vec![vec![0.0], vec![0.5], vec![1.0]],
            iterations: 2,
            evaluations: 20,
            infill_starts: 1,
            regression: RegressionSpec::CONSTANT,
            correlation: CorrelationSpec::MATERN52,
            theta: ThetaTuning::Fixed(ndarray::array![0.1]),
            gp_starts: 0,
            gp_evaluations: 5,
            seed: 74,
            phase_steps: (1, 1),
            radius: 0.2,
            contraction: 0.5,
            threads: 1,
            stack_bytes: 4 * 1024 * 1024,
            foreign_bytes: 1024 * 1024,
            time: Duration::from_secs(20),
        }
    }
    fn evaluator(calls: Arc<AtomicUsize>) -> Evaluator {
        Evaluator {
            correspondence: correspondence(),
            calls,
            fail: false,
            cancel: None,
            threads: 1,
        }
    }
    fn scope() -> ExecutionScope {
        ExecutionScope::new(Arc::new(AtomicBool::new(false)), None)
    }
    #[test]
    fn actual_full_trego_solver_state_retains_rng_training_phase_and_radius() {
        let calls = Arc::new(AtomicUsize::new(0));
        let mut task =
            RetainedSurrogate::new(evaluator(calls.clone()), options(), scope()).unwrap();
        assert!(task.state().is_none());
        let initial = task.advance().unwrap().unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 3);
        assert_eq!(
            task.state()
                .unwrap()
                .surrogate
                .data
                .as_ref()
                .unwrap()
                .0
                .nrows(),
            3
        );
        assert!(task.state().unwrap().rng.is_some());
        let mut reference =
            RetainedSurrogate::new(evaluator(Arc::new(AtomicUsize::new(0))), options(), scope())
                .unwrap();
        reference.advance().unwrap();
        for iteration in 1..=2 {
            let proposal = task.advance().unwrap().unwrap();
            let same = reference.advance().unwrap().unwrap();
            assert_eq!(proposal.coordinates, same.coordinates);
            assert_eq!(proposal.radius, same.radius);
            assert_eq!(task.state().unwrap().iter, iteration);
            assert!(task.state().unwrap().rng.is_some());
            assert!(task.state().unwrap().surrogate.theta_inits.is_some());
            assert!(
                task.state()
                    .unwrap()
                    .surrogate
                    .data
                    .as_ref()
                    .unwrap()
                    .0
                    .nrows()
                    > 3
            );
            assert!(proposal.infill_statistic.is_some());
            assert_eq!(proposal.original_target, correspondence().original_target);
            assert_eq!(proposal.fidelity, correspondence().fidelity);
            assert_eq!(proposal.task, initial.task);
        }
        assert_eq!(task.work().unwrap().initializations, 1);
        assert_eq!(task.work().unwrap().iterations, 2);
        assert_eq!(
            task.work().unwrap().evaluations,
            calls.load(Ordering::SeqCst) as u64
        );
        let work = task.work().unwrap();
        task.advance().unwrap();
        assert_eq!(task.work().unwrap().iterations, work.iterations);
        assert!(task.retained_bytes() > options().foreign_bytes);
    }
    #[test]
    fn owned_statistical_sample_clones_share_buffers_and_retain_every_allocation_owner() {
        #[derive(Debug)]
        struct Lease;
        let mut task =
            RetainedSurrogate::new(evaluator(Arc::new(AtomicUsize::new(0))), options(), scope())
                .unwrap();
        let first = Arc::new(Lease);
        let second = Arc::new(Lease);
        let first_weak = Arc::downgrade(&first);
        let second_weak = Arc::downgrade(&second);
        let sample = task
            .advance()
            .unwrap()
            .unwrap()
            .with_owner(first.clone())
            .with_owner(second.clone());
        let cloned = sample.clone();
        assert!(Arc::ptr_eq(&sample.coordinates, &cloned.coordinates));
        assert!(Arc::ptr_eq(&sample.model_values, &cloned.model_values));
        assert_eq!(
            sample.retained_bytes().unwrap(),
            cloned.retained_bytes().unwrap()
        );
        assert_eq!(sample.task, cloned.task);
        drop(first);
        drop(second);
        drop(sample);
        assert!(first_weak.upgrade().is_some());
        assert!(second_weak.upgrade().is_some());
        drop(cloned);
        assert!(first_weak.upgrade().is_none());
        assert!(second_weak.upgrade().is_none());
    }
    #[test]
    fn actual_fidelity_callback_terminal_cause_survives_library_nan_rejection() {
        let calls = Arc::new(AtomicUsize::new(0));
        let mut e = evaluator(calls.clone());
        e.fail = true;
        let mut task = RetainedSurrogate::new(e, options(), scope()).unwrap();
        assert!(matches!(
            task.advance(),
            Err(SurrogateFailure::Callback(MathError::Domain {
                requirement: "actual sentinel fidelity refusal",
                ..
            }))
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(task.work().unwrap().evaluations, 1);
        assert_eq!(task.work().unwrap().initializations, 1);
        assert!(task.advance().is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(task.proposal().is_none());
    }
    #[test]
    fn cancellation_and_point_limit_latch_before_additional_callback_work() {
        let cancel = Arc::new(AtomicBool::new(false));
        let calls = Arc::new(AtomicUsize::new(0));
        let mut e = evaluator(calls.clone());
        e.cancel = Some(cancel.clone());
        let mut task =
            RetainedSurrogate::new(e, options(), ExecutionScope::new(cancel, None)).unwrap();
        assert!(matches!(
            task.advance(),
            Err(SurrogateFailure::Math(MathError::Cancelled))
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        let calls = Arc::new(AtomicUsize::new(0));
        let mut o = options();
        o.evaluations = 3;
        let mut task = RetainedSurrogate::new(evaluator(calls.clone()), o, scope()).unwrap();
        task.advance().unwrap();
        assert!(matches!(
            task.advance(),
            Err(SurrogateFailure::Math(MathError::Limit(
                "surrogate callback evaluations"
            )))
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 3);
        assert_eq!(task.work().unwrap().iterations, 1);
    }
    #[test]
    fn fidelity_source_domain_criterion_and_options_are_consumed_before_allocation() {
        let c = correspondence();
        let o = options();
        let key = o.key(&c).unwrap();
        let mut changed = c.clone();
        changed.model_target = ContentHash::from_bytes([8; 32]);
        assert!(o.key(&changed).is_err());
        let calls = Arc::new(AtomicUsize::new(0));
        let mut e = evaluator(calls.clone());
        e.correspondence = changed;
        assert!(RetainedSurrogate::new(e, o.clone(), scope()).is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        let mut changed = c.clone();
        changed.source = ContentHash::from_bytes([9; 32]);
        assert_ne!(key, o.key(&changed).unwrap());
        let mut changed = c.clone();
        changed.outputs[0] = SemanticId::from_bytes([9; 16]);
        assert_ne!(key, o.key(&changed).unwrap());
        let mut changed = o.clone();
        changed.bounds[0].1 = 2.0;
        assert_ne!(key, changed.key(&c).unwrap());
        let mut changed = o.clone();
        changed.evaluations += 1;
        assert_ne!(key, changed.key(&c).unwrap());
        let mut changed = o.clone();
        changed.seed += 1;
        assert_ne!(key, changed.key(&c).unwrap());
        let mut changed = o;
        changed.iterations = u64::MAX;
        changed.evaluations = u64::MAX;
        assert!(changed.workspace_bytes(&c).is_err());
    }
    #[test]
    fn actual_library_lhs_initial_design_is_evaluated_in_declared_fidelity() {
        use egobox_doe::{Lhs, SamplingMethod};
        let design = Lhs::new(&ndarray::array![[0.0, 1.0]]).sample(4);
        let mut o = options();
        o.initial = design.rows().into_iter().map(|row| row.to_vec()).collect();
        let calls = Arc::new(AtomicUsize::new(0));
        let mut task = RetainedSurrogate::new(evaluator(calls.clone()), o, scope()).unwrap();
        task.advance().unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 4);
        assert_eq!(
            task.state().unwrap().surrogate.data.as_ref().unwrap().0,
            design
        );
    }
    #[test]
    fn actual_callbacks_run_in_requested_local_pool() {
        let calls = Arc::new(AtomicUsize::new(0));
        let mut e = evaluator(calls);
        e.threads = 2;
        let mut o = options();
        o.threads = 2;
        let mut task = RetainedSurrogate::new(e, o, scope()).unwrap();
        task.advance().unwrap();
        assert_eq!(task.work().unwrap().evaluations, 3);
    }
    #[test]
    fn scoped_advance_uses_same_callback_owner_and_phase_deadline_without_cancelling_task() {
        #[derive(Debug)]
        struct Slow(Evaluator);
        impl FidelityEvaluator for Slow {
            type Error = MathError;
            fn correspondence(&self) -> &FidelityCorrespondence {
                self.0.correspondence()
            }
            fn evaluate(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), MathError> {
                std::thread::sleep(Duration::from_millis(20));
                self.0.evaluate(x, out)
            }
        }
        let cancel = Arc::new(AtomicBool::new(false));
        let calls = Arc::new(AtomicUsize::new(0));
        let outer = ExecutionScope::new(
            cancel.clone(),
            Some(std::time::Instant::now() + Duration::from_secs(2)),
        );
        let mut task =
            RetainedSurrogate::new(Slow(evaluator(calls.clone())), options(), outer).unwrap();
        let wrong = ExecutionScope::new(
            Arc::new(AtomicBool::new(false)),
            Some(std::time::Instant::now() + Duration::from_secs(1)),
        );
        assert!(matches!(
            task.advance_within(wrong),
            Err(SurrogateFailure::Math(MathError::Contract(_)))
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        let phase = ExecutionScope::new(
            cancel.clone(),
            Some(std::time::Instant::now() + Duration::from_millis(10)),
        );
        assert!(matches!(
            task.advance_within(phase),
            Err(SurrogateFailure::Math(MathError::Scope(
                pse_kernels::ProviderError::Deadline
            )))
        ));
        assert!(!cancel.load(Ordering::SeqCst));
        assert_eq!(task.work().unwrap().evaluations, 1);
        assert_eq!(task.work().unwrap().initializations, 1);
        assert!(task.is_finished());
        assert!(task.proposal().is_none());
    }
}
