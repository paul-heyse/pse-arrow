// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One completion-owned solve lifecycle; finite batches never create persistent native sessions.
use super::{ExecutableCase, ExecutionWorker, MathRuntimeError, MathService, Preparation};
use pse_backend_native::{
    self as native, GramCertificate, ProblemError,
    quality::{self, Quality, Tolerances, Violation},
    routing::{self, Route},
    solve::*,
};
use pse_engine::cache_service::flight::FlightCancellation;
use pse_ids::FramedHasher;
use pse_math::binding::{CaseValues, ObjectiveSense};
use std::{collections::BTreeMap, sync::Arc};

/// The selected backend's complete typed controls remain visible through the common pipeline.
#[derive(Clone, Debug)]
pub enum BackendSettings {
    /// Use native defaults and the common semantic controls on the routed backend.
    Default,
    /// POUNCE algorithm and complete native FERAL configuration.
    #[cfg(feature = "solver-pounce")]
    Pounce {
        /// NLP method.
        method: native::pounce::Method,
        /// Native linear settings.
        linear: native::pounce::LinearSettings,
    },
    /// KINSOL equation/map and linear/scaling controls.
    #[cfg(feature = "solver-kinsol")]
    Kinsol(native::kinsol::Settings),
    /// HiGHS LP method (MIP/QP use native class routing).
    #[cfg(feature = "solver-highs")]
    Highs(native::highs::Settings),
    /// Clarabel's complete settings and preprocessing/data-update mode.
    Clarabel {
        /// Native settings.
        native: native::conic::Settings,
        /// Reuse/preprocessing mode.
        mode: native::conic::Mode,
    },
}
/// Unified solver request policy; physical tolerances are never inferred from trial magnitudes.
#[derive(Clone, Debug)]
pub struct SolverProfile {
    /// Qualified library-owned NLP preprocessing policy.
    pub presolve: native::presolve::Policy,
    /// Explicit numerical scales in original physical coordinates.
    pub scaling: Option<native::presolve::Scaling>,
    /// Mathematical purpose.
    pub intent: SolveIntent,
    /// Deterministic auto or an explicit eligible backend.
    pub selection: SolverSelection,
    /// Finite shared resource/method controls.
    pub controls: Controls,
    /// Complete backend-specific typed settings.
    pub backend: BackendSettings,
    /// Source-space acceptance scales.
    pub tolerances: Tolerances,
}
#[derive(Clone, Debug)]
enum Representation {
    Algebraic {
        prepared: Preparation,
        case: Option<Arc<ExecutableCase>>,
        values: CaseValues,
        providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
        certificate: Option<Arc<GramCertificate>>,
    },
    Conic {
        problem: Arc<native::ConicProblem>,
        certificate: Arc<GramCertificate>,
    },
}
/// Immutable routing decision, semantic maps and representation. Native state is constructed later.
#[derive(Clone, Debug)]
pub struct PreparedSolve {
    representation: Representation,
    profile: SolverProfile,
    route: Route,
    compatibility: Option<Compatibility>,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl PreparedSolve {
    /// Deterministic selected route, available for inspection before admission.
    pub fn route(&self) -> Route {
        self.route
    }
    /// Semantic/numerical reuse identity, absent for all-fixed validation.
    pub fn compatibility(&self) -> Option<&Compatibility> {
        self.compatibility.as_ref()
    }
}
/// Direct original-model validation when there are no free variables.
#[derive(Clone, Debug)]
pub struct ConstantReport {
    owner: Option<Arc<pse_columnar::AllocationLease>>,
    /// Authored objective if declared.
    pub objective: Option<f64>,
    /// Fresh constraint values in the complete original row order.
    pub observation: quality::Observation,
    /// Source-space quality without a fake native attempt.
    pub quality: Quality,
}
/// A step has either an actual native attempt or direct constant evaluation.
#[derive(Clone, Debug)]
pub enum Outcome {
    /// Native attempt, including limited/failed exits.
    Native(Box<SolveReport>),
    /// All-fixed original evaluation.
    Constant(ConstantReport),
    /// A typed admission/execution failure before a native report became available.
    Rejected(Arc<MathRuntimeError>),
}
impl Outcome {
    fn successful(&self) -> bool {
        match self {
            Self::Rejected(_) => false,
            Self::Constant(r) => r.quality.feasible(),
            Self::Native(r) => {
                matches!(
                    r.termination.category,
                    Termination::Success | Termination::Acceptable | Termination::FeasibleOnly
                ) && r.quality.as_ref().is_some_and(Quality::feasible)
            }
        }
    }
}
/// Owned finite batch result; unattempted steps are counted rather than fabricated.
#[derive(Debug)]
pub struct SequenceReport {
    /// Completed step outcomes in request order.
    pub outcomes: Vec<Outcome>,
    /// Steps not started after failure/cancellation.
    pub unattempted: usize,
    // Returned owned vectors retain their allocation allowance through the last reader.
    _owner: Arc<pse_columnar::AllocationLease>,
}
/// A finite batch and its explicit failure-continuation policy.
#[derive(Debug)]
pub struct SolveSequence {
    /// Finite fully prepared steps.
    pub steps: Vec<PreparedSolve>,
    /// Continue after failure only when steps are independent.
    pub continue_independent: bool,
    /// Explicit ceiling for retained result entries.
    pub result_limit: usize,
}
/// Dropping the handle requests cancellation; awaiting it witnesses native destruction and join.
pub struct SolveHandle<T = SequenceReport> {
    pub(super) cancel: FlightCancellation,
    pub(super) receiver: Option<tokio::sync::oneshot::Receiver<Result<T, MathRuntimeError>>>,
    pub(super) progress: Arc<Progress>,
}
impl<T> std::fmt::Debug for SolveHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SolveHandle").finish_non_exhaustive()
    }
}
impl<T> Drop for SolveHandle<T> {
    fn drop(&mut self) {
        if self.receiver.is_some() {
            self.cancel.cancel();
        }
    }
}
impl<T> SolveHandle<T> {
    /// Share cancellation with a public supervisor without moving native ownership.
    pub fn cancellation(&self) -> FlightCancellation {
        self.cancel.clone()
    }
    /// Shared bounded progress retained independently of a waiter.
    pub fn progress_source(&self) -> Arc<Progress> {
        self.progress.clone()
    }
    /// Request cancellation and retain the ability to await the terminal report.
    pub fn cancel(&self) {
        self.cancel.cancel();
    }
    /// Bounded owned progress; no native buffer crosses the thread boundary.
    pub fn progress(&self) -> (Vec<Event>, u64) {
        self.progress.snapshot()
    }
    /// Completion occurs after native teardown, thread-local destruction and join.
    pub async fn finish(mut self) -> Result<T, MathRuntimeError> {
        let receiver = self
            .receiver
            .as_mut()
            .ok_or_else(|| MathRuntimeError::Infrastructure("consumed solve handle".into()))?;
        let result = receiver
            .await
            .map_err(|_| MathRuntimeError::Infrastructure("lost solve supervisor".into()))?;
        self.receiver.take();
        result
    }
}
pub(crate) fn admit_profile(profile: &SolverProfile, route: Route) -> Result<(), ProblemError> {
    if !matches!(route, Route::Native(Backend::Ipopt | Backend::Pounce)) {
        if profile.scaling.is_some()
            || matches!(&profile.presolve,native::presolve::Policy::Explicit{required,..}if !required.is_empty())
        {
            return Err(ProblemError::Contract("common NLP scales/required preprocessing need an NLP route; use the selected class's native controls".into()));
        }
    }
    let Route::Native(backend) = route else {
        return Ok(());
    };
    let runtime_available = match backend {
        Backend::Ipopt => cfg!(feature = "solver-ipopt"),
        Backend::Pounce => cfg!(feature = "solver-pounce"),
        Backend::Kinsol => cfg!(feature = "solver-kinsol"),
        Backend::Highs => cfg!(feature = "solver-highs"),
        Backend::Clarabel => true,
    };
    if !runtime_available {
        return Err(ProblemError::Unavailable {
            backend,
            alternatives: vec![],
        });
    }
    let selected = match &profile.backend {
        BackendSettings::Default => backend,
        #[cfg(feature = "solver-pounce")]
        BackendSettings::Pounce { .. } => Backend::Pounce,
        #[cfg(feature = "solver-kinsol")]
        BackendSettings::Kinsol(_) => Backend::Kinsol,
        #[cfg(feature = "solver-highs")]
        BackendSettings::Highs(_) => Backend::Highs,
        BackendSettings::Clarabel { .. } => Backend::Clarabel,
    };
    if selected != backend {
        return Err(ProblemError::Contract(
            "backend settings do not match selected route".into(),
        ));
    }
    if matches!(
        backend,
        Backend::Ipopt | Backend::Kinsol | Backend::Clarabel
    ) && profile.controls.threads != 1
    {
        return Err(ProblemError::Contract(
            "selected linked native profile is serial".into(),
        ));
    }
    Ok(())
}
fn hash_controls(h: &mut FramedHasher, p: &SolverProfile) -> Result<(), ProblemError> {
    h.hash(&p.presolve.key()).bool(p.scaling.is_some());
    if let Some(s) = &p.scaling {
        h.hash(&s.key());
    }
    h.u64(p.intent as u64)
        .u64(p.controls.hessian as u64)
        .u64(p.controls.threads as u64);
    h.u64(p.controls.options.len() as u64);
    for (k, v) in &p.controls.options {
        h.str(k);
        match v {
            OptionValue::Text(v) => {
                h.u64(0).str(v);
            }
            OptionValue::Integer(v) => {
                h.u64(1).u64(*v as u64);
            }
            OptionValue::Real(v) => {
                h.u64(2).u64(v.to_bits());
            }
            OptionValue::Bool(v) => {
                h.u64(3).bool(*v);
            }
        }
    }
    match &p.backend {
        BackendSettings::Default => {
            h.u64(0);
        }
        #[cfg(feature = "solver-pounce")]
        BackendSettings::Pounce { method, linear } => {
            h.u64(2)
                .u64(*method as u64)
                .hash(&native::pounce::linear_key(linear));
        }
        #[cfg(feature = "solver-kinsol")]
        BackendSettings::Kinsol(s) => {
            h.u64(3).u64(s.strategy as u64);
            match s.linear {
                native::kinsol::Linear::Klu => {
                    h.u64(0);
                }
                native::kinsol::Linear::Dense { limit } => {
                    h.u64(1).u64(limit as u64);
                }
                native::kinsol::Linear::Spgmr { dimension } => {
                    h.u64(2).u64(dimension as u64);
                }
            }
            for v in s.variable_scales.iter().chain(&s.residual_scales) {
                h.u64(v.to_bits());
            }
            h.u64(s.anderson as u64)
                .u64(s.damping.to_bits())
                .u64(s.setup_interval as u64)
                .u64(s.step_tolerance.to_bits());
        }
        #[cfg(feature = "solver-highs")]
        BackendSettings::Highs(s) => {
            h.u64(4).u64(s.method as u64);
        }
        BackendSettings::Clarabel { native, mode } => {
            h.u64(5).u64(*mode as u64).str(
                &serde_json::to_string(native).map_err(|e| {
                    ProblemError::Contract(format!("Clarabel settings identity: {e}"))
                })?,
            );
        }
    }
    h.u64(p.tolerances.integrality.to_bits());
    for v in p.tolerances.variables.iter().chain(&p.tolerances.rows) {
        h.u64(v.to_bits());
    }
    Ok(())
}
fn compatibility(
    plan: &pse_math::assembly::CasePlan,
    values: &CaseValues,
    p: &SolverProfile,
    backend: Backend,
) -> Result<Compatibility, ProblemError> {
    let mut layout = FramedHasher::new("pse.solver.layout.v1");
    layout
        .u64(backend as u64)
        .u64(plan.structure().objective().map_or(0, |o| {
            if o.sense == ObjectiveSense::Minimize {
                1
            } else {
                2
            }
        }));
    hash_controls(&mut layout, p)?;
    for v in plan.structure().variables().iter().filter(|v| !v.fixed) {
        layout
            .id(&v.port.id)
            .id(&v.port.quantity.as_id())
            .id(&v.port.unit.as_id())
            .u64(v.domain as u64);
    }
    for r in plan.structure().rows() {
        layout
            .id(&r.id)
            .id(&r.quantity.as_id())
            .bool(r.lower == r.upper);
    }
    layout.u64(plan.bodies().len() as u64);
    for key in plan.bodies().keys() {
        layout.hash(key);
    }
    for pattern in [plan.jacobian_pattern(), plan.hessian_pattern()] {
        layout.hash(&pse_math::sparse::pattern_key(pattern));
    }
    let mut data = FramedHasher::new("pse.solver.data.v1");
    data.hash(&plan.structure().key());
    for (id, v) in &values.scalars {
        data.id(id).u64(v.to_bits());
    }
    Ok(Compatibility {
        layout: layout.finish_hash(),
        data: data.finish_hash(),
        backend,
    })
}
impl MathService {
    /// Complete pure routing and required immutable artifact compilation before native admission.
    pub async fn prepare_solve(
        self: &Arc<Self>,
        prepared: Preparation,
        values: CaseValues,
        providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
        profile: SolverProfile,
        certificate: Option<Arc<GramCertificate>>,
    ) -> Result<PreparedSolve, MathRuntimeError> {
        profile.controls.validate()?;
        let owner = self.reserve("math:prepared-solve", self.policy.workspace_bytes)?;
        prepared
            .prepared
            .plan
            .structure()
            .validate_values(&values)?;
        if !prepared
            .prepared
            .presolve
            .matches(&prepared.prepared.plan, &values)
        {
            return Err(ProblemError::Contract("fixed/parameter values differ from compiler assumptions; prepare the selected revision again".into()).into());
        }
        let f = &prepared.prepared.facts;
        profile.tolerances.validate(f.variables, f.rows)?;
        let mut convex = false;
        if let Some(c) = &prepared.prepared.coefficients {
            if prepared
                .prepared
                .coefficient_values
                .iter()
                .any(|(id, bits)| values.scalars.get(id).map(|v| v.to_bits()) != Some(*bits))
            {
                return Err(ProblemError::Contract("case values differ from the compiler's coefficient assumptions; prepare a new snapshot".into()).into());
            }
            let plan = prepared.prepared.plan.clone();
            let coefficients = c.clone();
            let proof = certificate.clone();
            convex = self
                .job(
                    1,
                    self.policy.worker_bytes,
                    FlightCancellation::default(),
                    move |_| {
                        let p = native::CoefficientProblem::from_plan(
                            &plan,
                            coefficients.as_ref().clone(),
                        )?;
                        Ok(p.validate_convex(proof.as_deref()).is_ok())
                    },
                )
                .await?;
        }
        let route = routing::select(f, profile.intent, profile.selection, convex)?;
        match route {
            Route::Native(Backend::Kinsol) => {
                native::structural::admit(prepared.structure(), native::structural::Mode::Roots)?
            }
            Route::Native(Backend::Ipopt | Backend::Pounce) => {
                native::structural::admit(prepared.structure(), native::structural::Mode::Nlp)?
            }
            _ => {}
        }
        admit_profile(&profile, route)?;
        if matches!(route, Route::Native(Backend::Ipopt | Backend::Pounce))
            && profile.controls.hessian == HessianMode::Exact
            && f.derivatives < pse_kernels::DerivativeOrder::Second
        {
            return Err(ProblemError::Contract(
                "exact NLP profile requires second-order compiler preparation".into(),
            )
            .into());
        }
        let stamp = match route {
            Route::Constant => None,
            Route::Native(backend) => Some(compatibility(
                &prepared.prepared.plan,
                &values,
                &profile,
                backend,
            )?),
        };
        let case = Some(self.assemble(prepared.clone()).await?);
        Ok(PreparedSolve {
            representation: Representation::Algebraic {
                prepared,
                case,
                values,
                providers,
                certificate,
            },
            profile,
            route,
            compatibility: stamp,
            _owner: owner,
        })
    }
    /// Explicit conic representation enters the same bounded worker/report lifecycle.
    pub async fn prepare_conic(
        self: &Arc<Self>,
        problem: Arc<native::ConicProblem>,
        certificate: Arc<GramCertificate>,
        profile: SolverProfile,
    ) -> Result<PreparedSolve, MathRuntimeError> {
        profile.controls.validate()?;
        let owner = self.reserve("math:prepared-conic", self.policy.workspace_bytes)?;
        let admitted = problem.clone();
        let proof = certificate.clone();
        self.job(
            1,
            self.policy.worker_bytes,
            FlightCancellation::default(),
            move |_| admitted.validate(&proof).map_err(Into::into),
        )
        .await?;
        profile.tolerances.validate(
            problem.contract.variables.len(),
            problem.contract.rows.len(),
        )?;
        if profile.intent != SolveIntent::Optimize
            || matches!(profile.selection,SolverSelection::Explicit(b)if b!=Backend::Clarabel)
        {
            return Err(ProblemError::Contract(
                "explicit continuous cone representation requires Clarabel optimization".into(),
            )
            .into());
        }
        admit_profile(&profile, Route::Native(Backend::Clarabel))?;
        let mut h = FramedHasher::new("pse.solver.conic-layout.v1");
        h.hash(&problem.contract.identity);
        hash_controls(&mut h, &profile)?;
        h.hash(&native::conic::cone_key(&problem.cones));
        for v in &problem.contract.variables {
            h.id(&v.id);
        }
        for id in &problem.contract.rows {
            h.id(id);
        }
        for a in [&problem.quadratic, &problem.constraints] {
            h.u64(a.m as u64)
                .u64(a.n as u64)
                .u64(a.colptr.len() as u64)
                .u64(a.rowval.len() as u64);
            for v in a.colptr.iter().chain(&a.rowval) {
                h.u64(*v as u64);
            }
        }
        let mut d = FramedHasher::new("pse.solver.conic-data.v1");
        for x in problem
            .quadratic
            .nzval
            .iter()
            .chain(&problem.constraints.nzval)
            .chain(&problem.objective)
            .chain(&problem.rhs)
        {
            d.u64(x.to_bits());
        }
        d.u64(problem.objective_constant.to_bits());
        for v in &problem.contract.variables {
            d.u64(v.lower.to_bits()).u64(v.upper.to_bits());
            h.bool(v.lower.is_finite()).bool(v.upper.is_finite());
        }
        let stamp = Compatibility {
            layout: h.finish_hash(),
            data: d.finish_hash(),
            backend: Backend::Clarabel,
        };
        Ok(PreparedSolve {
            representation: Representation::Conic {
                problem,
                certificate,
            },
            profile,
            route: Route::Native(Backend::Clarabel),
            compatibility: Some(stamp),
            _owner: owner,
        })
    }
    /// Start one finite batch. All CPU permits are acquired once; nested native work
    /// inherits this admission. Only immutable artifact compilation is shared.
    pub fn solve(
        self: &Arc<Self>,
        sequence: SolveSequence,
    ) -> Result<SolveHandle, MathRuntimeError> {
        if sequence.steps.is_empty()
            || sequence.steps.len() > sequence.result_limit
            || sequence.result_limit > 4096
        {
            return Err(MathRuntimeError::Limit(
                "finite solve sequence/result bound",
            ));
        }
        let result_bytes = sequence.steps.iter().try_fold(0usize, |total, s| {
            let (n, m) = match &s.representation {
                Representation::Algebraic { prepared, .. } => (
                    prepared.prepared.facts.variables,
                    prepared.prepared.facts.rows,
                ),
                Representation::Conic { problem, .. } => (
                    problem.contract.variables.len(),
                    problem.contract.rows.len(),
                ),
            };
            let sources = match &s.representation {
                Representation::Algebraic { prepared, .. } => prepared
                    .prepared
                    .plan
                    .structure()
                    .instances()
                    .iter()
                    .try_fold(0usize, |n, i| n.checked_add(i.contributions.len()))
                    .ok_or(MathRuntimeError::Limit("source observation extent"))?,
                Representation::Conic { .. } => 0,
            };
            n.checked_add(m)
                .and_then(|v| v.checked_add(sources))
                .and_then(|v| v.checked_mul(512))
                .and_then(|v| v.checked_add(s.profile.controls.report_allowance().ok()?))
                .and_then(|v| total.checked_add(v))
                .ok_or(MathRuntimeError::Limit("solve result allowance"))
        })?;
        let result_owner = self.reserve("math:solve-results", result_bytes)?;
        let cores = sequence
            .steps
            .iter()
            .map(|s| s.profile.controls.threads)
            .max()
            .unwrap_or(1);
        let history = sequence
            .steps
            .iter()
            .map(|s| s.profile.controls.history)
            .max()
            .unwrap_or(0);
        let progress = Arc::new(Progress::new(history));
        let events = progress.clone();
        let cancel = FlightCancellation::default();
        let control = cancel.clone();
        let service = self.clone();
        let (receive_tx, receiver) = tokio::sync::oneshot::channel();
        let stacks = if cores > 1
            && sequence
                .steps
                .iter()
                .any(|s| s.route == Route::Native(Backend::Pounce))
        {
            cores
        } else {
            cores.saturating_sub(1)
        };
        let extra_stacks = stacks
            .checked_mul(self.policy.stack_bytes)
            .ok_or(MathRuntimeError::Limit("solver stack allowance"))?;
        let bytes = self
            .policy
            .worker_bytes
            .checked_add(extra_stacks)
            .ok_or(MathRuntimeError::Limit("solver allowance"))?;
        tokio::spawn(async move {
            let runner = service.clone();
            let result = service
                .job(cores, bytes, control, move |flag| {
                    runner.run_sequence(sequence, flag, events, result_owner)
                })
                .await;
            let _ = receive_tx.send(result);
        });
        Ok(SolveHandle {
            cancel,
            receiver: Some(receiver),
            progress,
        })
    }
    fn run_sequence(
        self: &Arc<Self>,
        sequence: SolveSequence,
        flag: Arc<std::sync::atomic::AtomicBool>,
        progress: Arc<Progress>,
        owner: Arc<pse_columnar::AllocationLease>,
    ) -> Result<SequenceReport, MathRuntimeError> {
        #[cfg(feature = "solver-pounce")]
        if sequence
            .steps
            .iter()
            .any(|s| s.route == Route::Native(Backend::Pounce))
        {
            let threads = sequence
                .steps
                .iter()
                .map(|s| s.profile.controls.threads)
                .max()
                .unwrap_or(1);
            return native::pounce::with_threads(threads, self.policy.stack_bytes, move || {
                self.run_sequence_inner(sequence, flag, progress, owner)
            });
        }
        self.run_sequence_inner(sequence, flag, progress, owner)
    }
    fn run_sequence_inner(
        self: &Arc<Self>,
        sequence: SolveSequence,
        flag: Arc<std::sync::atomic::AtomicBool>,
        progress: Arc<Progress>,
        owner: Arc<pse_columnar::AllocationLease>,
    ) -> Result<SequenceReport, MathRuntimeError> {
        let total = sequence.steps.len();
        let mut outcomes = Vec::new();
        let mut warm: Option<WarmStart> = None;
        let mut retained = Retained::None;
        for step in sequence.steps {
            if flag.load(std::sync::atomic::Ordering::Acquire) {
                break;
            }
            let controls = &step.profile.controls;
            let mut execution = Execution::new(flag.clone(), controls);
            execution.progress = progress.clone();
            let compatible = warm.as_ref().is_some_and(|w| {
                step.compatibility
                    .as_ref()
                    .is_some_and(|s| w.validate(s).is_ok())
            });
            if !compatible {
                warm = None;
            }
            if controls.reuse == ReusePolicy::Fresh {
                retained = Retained::None;
                warm = None;
            }
            let outcome = self
                .run_step(step, execution, warm.as_ref(), &mut retained)
                .unwrap_or_else(|e| Outcome::Rejected(Arc::new(e)));
            let outcome = match outcome {
                Outcome::Native(r) => Outcome::Native(Box::new((*r).with_owner(owner.clone()))),
                Outcome::Constant(mut r) => {
                    r.owner = Some(owner.clone());
                    Outcome::Constant(r)
                }
                other => other,
            };
            let successful = outcome.successful();
            warm = match &outcome {
                Outcome::Native(r) => r.warm_start.clone(),
                Outcome::Constant(_) | Outcome::Rejected(_) => None,
            };
            // Terminal native failures cannot poison the next independent step.
            if !successful {
                retained = Retained::None;
                warm = None;
            }
            outcomes.push(outcome);
            if !successful && !sequence.continue_independent {
                break;
            }
        }
        drop(retained);
        let unattempted = total - outcomes.len();
        Ok(SequenceReport {
            outcomes,
            unattempted,
            _owner: owner,
        })
    }
    fn run_step(
        &self,
        step: PreparedSolve,
        execution: Execution,
        warm: Option<&WarmStart>,
        retained: &mut Retained,
    ) -> Result<Outcome, MathRuntimeError> {
        #[cfg(not(any(
            feature = "solver-ipopt",
            feature = "solver-pounce",
            feature = "solver-kinsol",
            feature = "solver-highs"
        )))]
        let _ = warm;
        let controls = &step.profile.controls;
        let tolerance = &step.profile.tolerances;
        let rebuild = controls.reuse != ReusePolicy::RequireReuse;
        match step.representation {
            Representation::Conic {
                problem,
                certificate,
            } => {
                let stamp = step
                    .compatibility
                    .ok_or_else(|| ProblemError::Contract("missing conic stamp".into()))?;
                let (native_settings, mode) = match step.profile.backend {
                    BackendSettings::Default => (
                        native::conic::Settings::default(),
                        native::conic::Mode::SingleSolve,
                    ),
                    BackendSettings::Clarabel { native, mode } => (native, mode),
                    #[cfg(any(
                        feature = "solver-ipopt",
                        feature = "solver-pounce",
                        feature = "solver-kinsol",
                        feature = "solver-highs"
                    ))]
                    _ => {
                        return Err(ProblemError::Contract(
                            "wrong backend settings for Clarabel".into(),
                        )
                        .into());
                    }
                };
                let reusable = if let Retained::Clarabel(s) = retained {
                    s.update(&problem, &certificate, stamp.clone()).is_ok()
                } else {
                    false
                };
                if !reusable {
                    if !rebuild && !matches!(retained, Retained::None) {
                        return Err(ProblemError::Contract(
                            "required Clarabel reuse unavailable".into(),
                        )
                        .into());
                    }
                    *retained = Retained::None;
                    *retained = Retained::Clarabel(native::conic::Session::new(
                        &problem,
                        &certificate,
                        controls,
                        native_settings.clone(),
                        mode,
                        stamp,
                    )?);
                }
                let Retained::Clarabel(s) = retained else {
                    return Err(ProblemError::Contract("lost Clarabel owner".into()).into());
                };
                let mut report =
                    s.solve(&problem, controls, native_settings, execution, tolerance)?;
                report
                    .metrics
                    .insert("reuse.native_model".into(), Metric::Bool(reusable));
                Ok(Outcome::Native(Box::new(report)))
            }
            Representation::Algebraic {
                prepared,
                case,
                values,
                providers,
                certificate,
            } => {
                #[cfg(not(feature = "solver-highs"))]
                let _ = certificate;
                let backend = match step.route {
                    Route::Constant => None,
                    Route::Native(b) => Some(b),
                };
                #[cfg(feature = "solver-highs")]
                if backend == Some(Backend::Highs) {
                    let c = prepared.prepared.coefficients.as_ref().ok_or_else(|| {
                        ProblemError::Contract("missing coefficient product".into())
                    })?;
                    let p = native::CoefficientProblem::from_plan(
                        &prepared.prepared.plan,
                        c.as_ref().clone(),
                    )?;
                    let stamp = step.compatibility.ok_or_else(|| {
                        ProblemError::Contract("missing coefficient stamp".into())
                    })?;
                    let settings = match step.profile.backend {
                        BackendSettings::Default => native::highs::Settings::default(),
                        BackendSettings::Highs(m) => m,
                        _ => {
                            return Err(ProblemError::Contract(
                                "wrong backend settings for HiGHS".into(),
                            )
                            .into());
                        }
                    };
                    let reusable = if let Retained::Highs(s) = retained {
                        s.update(&p, certificate.as_deref(), stamp.clone()).is_ok()
                    } else {
                        false
                    };
                    if !reusable {
                        if !rebuild && !matches!(retained, Retained::None) {
                            return Err(ProblemError::Contract(
                                "required HiGHS reuse unavailable".into(),
                            )
                            .into());
                        }
                        *retained = Retained::None;
                        *retained = Retained::Highs(native::highs::Session::new(
                            &p,
                            certificate.as_deref(),
                            stamp,
                        )?);
                    }
                    let Retained::Highs(s) = retained else {
                        return Err(ProblemError::Contract("lost HiGHS owner".into()).into());
                    };
                    if let Some(start) = &settings.sparse_start {
                        s.sparse_start(&p, start)?;
                    }
                    let mut report = s.solve(
                        &p,
                        controls,
                        settings.method,
                        execution.clone(),
                        tolerance,
                        warm,
                    )?;
                    if settings.diagnostics.rays
                        || settings.diagnostics.iis
                        || settings.diagnostics.ranging
                        || settings.diagnostics.relaxation.is_some()
                    {
                        let diagnostics = s
                            .diagnose(&p, &settings.diagnostics, &execution)
                            .unwrap_or_else(|e| native::highs::diagnostics::Report {
                                unavailable: BTreeMap::from([("operation".into(), e.to_string())]),
                                ..Default::default()
                            });
                        report.highs_diagnostics = Some(Box::new(diagnostics));
                    }
                    report
                        .metrics
                        .insert("reuse.native_model".into(), Metric::Bool(reusable));
                    if let Some(candidate) = &report.candidate {
                        let bounds = prepared
                            .prepared
                            .plan
                            .structure()
                            .rows()
                            .iter()
                            .map(|r| (r.lower, r.upper))
                            .collect();
                        match native::highs::coefficient_observation(
                            &p,
                            &candidate.primal,
                            &c.row_constants,
                            bounds,
                        ) {
                            Ok(o) => report.observation = Some(o),
                            Err(e) => {
                                report.validation_error = Some(e.to_string());
                                report.termination.assurance = Assurance::None;
                            }
                        }
                    }
                    if let (Some(candidate), Some(observation)) =
                        (&report.candidate, &mut report.observation)
                    {
                        let validation = (|| -> Result<_, MathRuntimeError> {
                            let executable = case.clone().ok_or_else(|| {
                                ProblemError::Contract(
                                    "missing original coefficient evaluator".into(),
                                )
                            })?;
                            let providers = providers
                                .iter()
                                .map(|(key, f)| {
                                    f.worker()
                                        .map(|w| (*key, w))
                                        .map_err(|e| ProblemError::Contract(e.to_string()))
                                })
                                .collect::<Result<_, _>>()?;
                            let mut original =
                                self.worker(executable, providers, execution.cancel.clone())?;
                            let mut trial = values.clone();
                            for (id, value) in prepared
                                .prepared
                                .plan
                                .columns()
                                .iter()
                                .zip(&candidate.primal)
                            {
                                trial.scalars.insert(*id, *value);
                            }
                            original.worker.constraints(&trial)?;
                            Ok(original.worker.constraint_sources()?)
                        })();
                        match validation {
                            Ok(sources) => observation.sources = sources,
                            Err(e) => {
                                report.validation_error = Some(e.to_string());
                                report.termination.assurance = Assurance::None;
                            }
                        }
                    }
                    return Ok(Outcome::Native(Box::new(report)));
                }
                let case = case.ok_or_else(|| {
                    ProblemError::Contract("missing executable representation".into())
                })?;
                let providers = providers
                    .into_iter()
                    .map(|(key, f)| {
                        f.worker()
                            .map(|v| (key, v))
                            .map_err(|e| ProblemError::Contract(e.to_string()))
                    })
                    .collect::<Result<_, _>>()?;
                let ExecutionWorker {
                    mut worker,
                    _case,
                    _lease,
                } = self.worker(case, providers, execution.cancel.clone())?;
                #[cfg(feature = "solver-kinsol")]
                let mut owners = Some((_case, _lease));
                #[cfg(not(feature = "solver-kinsol"))]
                let owners = Some((_case, _lease));
                if backend.is_none() {
                    let objective = prepared
                        .prepared
                        .plan
                        .structure()
                        .objective()
                        .map(|o| worker.objective(&values).map(|v| v * o.sense.sign()))
                        .transpose()?;
                    let values = worker.constraints(&values)?;
                    let rows = prepared
                        .prepared
                        .plan
                        .structure()
                        .rows()
                        .iter()
                        .zip(&values)
                        .zip(&tolerance.rows)
                        .map(|((r, v), t)| Violation {
                            id: r.id,
                            physical: quality::interval(*v, r.lower, r.upper),
                            tolerance: *t,
                        })
                        .collect();
                    let sources = worker.constraint_sources()?;
                    return Ok(Outcome::Constant(ConstantReport {
                        owner: None,
                        objective,
                        observation: {
                            let mut observation = quality::Observation::from_values(
                                objective,
                                values,
                                prepared
                                    .prepared
                                    .plan
                                    .structure()
                                    .rows()
                                    .iter()
                                    .map(|r| (r.lower, r.upper))
                                    .collect(),
                            )?;
                            observation.sources = sources;
                            observation
                        },
                        quality: Quality::new(rows, vec![], vec![])?,
                    }));
                }
                #[cfg(not(any(
                    feature = "solver-ipopt",
                    feature = "solver-pounce",
                    feature = "solver-kinsol"
                )))]
                {
                    drop(owners);
                    Err(ProblemError::Contract("selected native backend unavailable".into()).into())
                }
                #[cfg(any(
                    feature = "solver-ipopt",
                    feature = "solver-pounce",
                    feature = "solver-kinsol"
                ))]
                {
                    let initial: Vec<_> = prepared
                        .prepared
                        .plan
                        .columns()
                        .iter()
                        .map(|id| values.scalars[id])
                        .collect();
                    let sense = prepared
                        .prepared
                        .plan
                        .structure()
                        .objective()
                        .map_or(ObjectiveSense::Minimize, |o| o.sense);
                    let stamp = step
                        .compatibility
                        .ok_or_else(|| ProblemError::Contract("missing nonlinear stamp".into()))?;
                    let mut oracle = native::assembled::AlgebraicOracle::new(worker, values)?
                        .with_presolve_facts(prepared.prepared.presolve.clone())?;
                    if let Some(c) = &prepared.prepared.coefficients {
                        oracle = oracle.with_coefficient_facts(c)?;
                    }
                    let report = match backend {
                        #[cfg(feature = "solver-ipopt")]
                        Some(Backend::Ipopt) => {
                            if !matches!(retained, Retained::Ipopt(_)) {
                                if !rebuild && !matches!(retained, Retained::None) {
                                    return Err(ProblemError::Contract(
                                        "required Ipopt session unavailable".into(),
                                    )
                                    .into());
                                }
                                *retained = Retained::None;
                                *retained = Retained::Ipopt(native::ipopt::Session::new());
                            }
                            if !matches!(step.profile.backend, BackendSettings::Default) {
                                return Err(
                                    ProblemError::Contract("wrong Ipopt settings".into()).into()
                                );
                            }
                            let mut oracle: Box<dyn native::NlpOracle> = Box::new(oracle);
                            if step.profile.intent == SolveIntent::FeasiblePoint {
                                oracle = Box::new(native::assembled::FeasibilityOracle(oracle));
                            }
                            let Retained::Ipopt(session) = retained else {
                                return Err(
                                    ProblemError::Contract("lost Ipopt session".into()).into()
                                );
                            };
                            let mut pipeline = native::presolve::Pipeline::new(
                                oracle,
                                &initial,
                                &step.profile.presolve,
                                tolerance,
                                step.profile.scaling.as_ref(),
                                execution.clone(),
                                warm,
                                stamp,
                                self.policy.worker_bytes / 256,
                            )?;
                            let mut transport = pipeline.take_oracle()?;
                            let scaling = native::NlpOracle::scaling(&transport).cloned();
                            let report = if let Some(report) = pipeline.terminal_report(sense)? {
                                report
                            } else {
                                session.solve(
                                    &mut transport,
                                    pipeline.initial(),
                                    sense,
                                    controls,
                                    execution,
                                    &pipeline.tolerances(tolerance),
                                    scaling.as_ref(),
                                    pipeline.warm(),
                                    pipeline.compatibility().clone(),
                                )?
                            };
                            pipeline.finish(report, tolerance, sense)
                        }
                        #[cfg(feature = "solver-pounce")]
                        Some(Backend::Pounce) => {
                            if !matches!(retained, Retained::Pounce(_)) {
                                if !rebuild && !matches!(retained, Retained::None) {
                                    return Err(ProblemError::Contract(
                                        "required POUNCE application unavailable".into(),
                                    )
                                    .into());
                                }
                                *retained = Retained::None;
                                *retained = Retained::Pounce(native::pounce::Session::new());
                            }
                            let (method, linear) = match step.profile.backend {
                                BackendSettings::Default => (
                                    native::pounce::Method::InteriorPoint,
                                    native::pounce::LinearSettings::default(),
                                ),
                                BackendSettings::Pounce { method, linear } => (method, linear),
                                _ => {
                                    return Err(ProblemError::Contract(
                                        "wrong POUNCE settings".into(),
                                    )
                                    .into());
                                }
                            };
                            let mut oracle: Box<dyn native::NlpOracle> = Box::new(oracle);
                            if step.profile.intent == SolveIntent::FeasiblePoint {
                                oracle = Box::new(native::assembled::FeasibilityOracle(oracle));
                            }
                            let Retained::Pounce(session) = retained else {
                                return Err(
                                    ProblemError::Contract("lost POUNCE session".into()).into()
                                );
                            };
                            let mut pipeline = native::presolve::Pipeline::new(
                                oracle,
                                &initial,
                                &step.profile.presolve,
                                tolerance,
                                step.profile.scaling.as_ref(),
                                execution.clone(),
                                warm,
                                stamp,
                                self.policy.worker_bytes / 256,
                            )?;
                            let transport = pipeline.take_oracle()?;
                            let report = if let Some(report) = pipeline.terminal_report(sense)? {
                                report
                            } else {
                                session.solve(
                                    Box::new(transport),
                                    pipeline.initial(),
                                    sense,
                                    controls,
                                    method,
                                    linear,
                                    execution,
                                    &pipeline.tolerances(tolerance),
                                    pipeline.warm(),
                                    pipeline.compatibility().clone(),
                                )?
                            };
                            pipeline.finish(report, tolerance, sense)
                        }
                        #[cfg(feature = "solver-kinsol")]
                        Some(Backend::Kinsol) => {
                            oracle.admit_nle()?;
                            let settings = match step.profile.backend {
                                BackendSettings::Kinsol(s) => s,
                                BackendSettings::Default => native::kinsol::Settings {
                                    strategy: native::kinsol::Strategy::LineSearch,
                                    linear: native::kinsol::Linear::Klu,
                                    variable_scales: vec![1.0; initial.len()],
                                    residual_scales: tolerance
                                        .rows
                                        .iter()
                                        .map(|t| controls.tolerance / t)
                                        .collect(),
                                    anderson: 0,
                                    damping: 1.0,
                                    setup_interval: 10,
                                    step_tolerance: controls.tolerance,
                                },
                                _ => {
                                    return Err(ProblemError::Contract(
                                        "wrong KINSOL settings".into(),
                                    )
                                    .into());
                                }
                            };
                            let function = native::kinsol::Function::Equations(Box::new(oracle));
                            let reused = matches!(retained,Retained::Kinsol{session,..}if session.matches_layout(&stamp));
                            if reused {
                                if let Retained::Kinsol { session, _owners } = retained {
                                    session.replace(function, stamp)?;
                                    *_owners = owners.take();
                                }
                            } else {
                                if !rebuild && !matches!(retained, Retained::None) {
                                    return Err(ProblemError::Contract(
                                        "required KINSOL reuse unavailable".into(),
                                    )
                                    .into());
                                }
                                *retained = Retained::None;
                                *retained = Retained::Kinsol {
                                    session: native::kinsol::Session::new(
                                        function,
                                        settings,
                                        execution.clone(),
                                        stamp,
                                    )?,
                                    _owners: owners.take(),
                                };
                            }
                            let Retained::Kinsol { session, .. } = retained else {
                                return Err(
                                    ProblemError::Contract("lost KINSOL session".into()).into()
                                );
                            };
                            let mut report =
                                session.solve(&initial, controls, execution, tolerance, warm)?;
                            report
                                .metrics
                                .insert("reuse.native_model".into(), Metric::Bool(reused));
                            report
                        }
                        _ => {
                            return Err(ProblemError::Contract(
                                "selected native backend unavailable".into(),
                            )
                            .into());
                        }
                    };
                    // _case and _lease outlive every callback/native handle above.
                    drop(owners);
                    Ok(Outcome::Native(Box::new(report)))
                }
            }
        }
    }
}
enum Retained {
    None,
    #[cfg(feature = "solver-pounce")]
    Pounce(native::pounce::Session),
    #[cfg(feature = "solver-kinsol")]
    Kinsol {
        session: native::kinsol::Session,
        _owners: Option<(Arc<ExecutableCase>, Arc<pse_columnar::AllocationLease>)>,
    },
    #[cfg(feature = "solver-ipopt")]
    Ipopt(native::ipopt::Session),
    Clarabel(native::conic::Session),
    #[cfg(feature = "solver-highs")]
    Highs(native::highs::Session),
}

/// Complete effective request identity, distinct from native session compatibility.
pub(crate) fn profile_key(p: &SolverProfile) -> Result<pse_ids::ContentHash, ProblemError> {
    let mut h = FramedHasher::new("pse.solver.profile.v1");
    hash_controls(&mut h, p)?;
    h.u64(p.controls.time_limit.as_secs())
        .u64(u64::from(p.controls.time_limit.subsec_nanos()))
        .u64(u64::from(p.controls.iterations))
        .u64(p.controls.tolerance.to_bits())
        .u64(p.controls.history as u64)
        .u64(p.controls.reuse as u64);
    match p.selection {
        SolverSelection::Auto => {
            h.u64(0);
        }
        SolverSelection::Explicit(b) => {
            h.u64(1).u64(b as u64);
        }
    }
    Ok(h.finish_hash())
}
