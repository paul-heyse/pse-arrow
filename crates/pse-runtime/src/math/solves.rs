// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One completion-owned solve lifecycle; finite batches never create persistent native sessions.
use super::{
    ExecutableCase, ExecutionWorker, MathRuntimeError, MathService, Preparation, WorkerBudget,
};
use pse_backend_native::{
    self as native, ProblemError,
    execution::{self, BackendSettings, Retained},
    quality::{self, Quality, Tolerances, Violation},
    routing::{self, Route},
    solve::*,
};
use pse_columnar::flight::FlightCancellation;
use pse_ids::FramedHasher;
pub use pse_math::convexity::ConvexityPolicy;
use pse_math::{
    binding::{CaseValues, ObjectiveSense},
    convexity::QuadraticEvidence,
    normalization::Normalization,
};
use pse_model::numerics::{NumericalPolicy, ResolvedNumericalPolicy};

/// Selected authored inputs to numerical resolution; analysis overrides stay in the profile.
#[derive(Clone, Debug, Default)]
pub struct NumericalInputs {
    /// Model/case/property declarations selected by the workflow.
    pub declarations: Vec<pse_math::numerics::SourcedRequirement>,
    /// Observable and physical closure targets beyond the algebraic coordinates.
    pub targets: Vec<pse_math::numerics::TargetSpec>,
    /// Original residual definitions of the case's implicit blocks, keyed by the provider
    /// their callers invoke, with unknown intervals under the case's bounds. A factorable
    /// route exports them in place of the blocks' realizations (ADR-0105 §1).
    pub implicit: BTreeMap<pse_kernels::ProviderKey, pse_math::factorable::ImplicitDefinition>,
}
use std::{collections::BTreeMap, future::Future, sync::Arc};

/// Unified solver request policy; physical tolerances are never inferred from trial magnitudes.
#[derive(Clone, Debug)]
pub struct SolverProfile {
    /// Qualified library-owned NLP preprocessing policy.
    pub presolve: native::presolve::Policy,
    /// ID-keyed physical magnitudes and independent numerical requirements.
    pub numerics: NumericalPolicy,
    /// Exact default or explicitly requested numerical convexity qualification.
    pub convexity: ConvexityPolicy,
    /// Mathematical purpose.
    pub intent: SolveIntent,
    /// Deterministic auto or an explicit eligible backend.
    pub selection: SolverSelection,
    /// Finite shared resource/method controls.
    pub controls: Controls,
    /// Complete backend-specific typed settings.
    pub backend: BackendSettings,
    /// Parametric sensitivities to compute at the candidate (Plan 22 S1); only a modeling
    /// solve prepares the parametric program they need.
    pub sensitivity: Option<super::settings::SensitivityRequest>,
}
/// The one owner of request defaults: every boundary takes an omitted field from here
/// rather than restating it (ADR-0113).
impl Default for SolverProfile {
    fn default() -> Self {
        Self {
            presolve: native::presolve::Policy::default(),
            numerics: NumericalPolicy::default(),
            convexity: ConvexityPolicy::default(),
            intent: SolveIntent::Optimize,
            selection: SolverSelection::Auto,
            controls: Controls::default(),
            backend: BackendSettings::Default,
            sensitivity: None,
        }
    }
}
/// A compiled case with its selected values, providers and convexity evidence.
#[derive(Clone, Debug)]
struct AlgebraicCase {
    prepared: Preparation,
    case: Option<Arc<ExecutableCase>>,
    values: CaseValues,
    providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
    certificate: Option<Arc<dyn QuadraticEvidence>>,
    /// Factorable projection under `values`, built only for a factorable route, with the
    /// reservation that admits its retained size.
    factorable: Option<(
        Arc<pse_math::factorable::FactorableProgram>,
        Arc<pse_columnar::AllocationLease>,
    )>,
    /// The parametric program of a sensitivity request (Plan 22 S1), attached by
    /// [`PreparedSolve::with_sensitivity`].
    sensitivity: Option<SensitivityProgram>,
    /// A recognized convex program's cone form (ADR-0121), built only for a cone route
    /// over a program that is not a coefficient program, with its reservation.
    recognized: Option<(
        Arc<native::conic::Recognized>,
        Arc<pse_columnar::AllocationLease>,
    )>,
}
/// The parametric program a sensitivity request differentiates, with the normalization of
/// its columns and each parameter's value.
#[derive(Clone, Debug)]
struct SensitivityProgram {
    program: Arc<ExecutableCase>,
    normalization: Normalization,
    parameters: Vec<(pse_ids::SemanticId, f64)>,
    reduced_hessian: bool,
    /// Keep the pinned factor in the worker's retained state for an advanced step (Plan 22
    /// Y5c2), charged to the job's allowance; set by [`PreparedSolve::retaining_factor`].
    retain: bool,
}
impl SensitivityProgram {
    /// The request for the backend: callbacks over `worker`, or the reason none could be
    /// built.
    fn request(
        &self,
        worker: pse_math::assembly::CaseWorker,
        values: &CaseValues,
    ) -> Result<native::kkt::Sensitivity, ProblemError> {
        let oracle = native::assembled::AlgebraicOracle::new(worker, values.clone())?
            .with_normalization(self.normalization.clone())?;
        Ok(native::kkt::Sensitivity {
            oracle: Box::new(oracle),
            parameters: self.parameters.clone(),
            reduced_hessian: self.reduced_hessian,
            retain: self.retain,
        })
    }
    /// Every quantity withheld because the parametric callbacks could not be built.
    fn withheld(&self, cause: ProblemError) -> native::kkt::Parametric {
        native::kkt::Parametric::withheld(
            self.parameters.iter().map(|(id, _)| *id).collect(),
            self.reduced_hessian,
            native::kkt::Withheld::Analysis(cause.into()),
        )
    }
}
#[derive(Clone, Debug)]
enum Representation {
    Algebraic(AlgebraicCase),
    Conic {
        /// Normalized at preparation.
        problem: Arc<native::ConicProblem>,
        /// The submitted data a certificate is verified against.
        original: Arc<native::ConicProblem>,
        certificate: Arc<dyn QuadraticEvidence>,
    },
}
/// Immutable routing decision, semantic maps and representation. Native state is constructed later.
#[derive(Clone, Debug)]
pub struct PreparedSolve {
    representation: Representation,
    profile: SolverProfile,
    numerics: Arc<ResolvedNumericalPolicy>,
    normalization: Normalization,
    tolerances: Tolerances,
    /// Stopping budgets resolved from `numerics`; never taken from user controls (F20).
    accuracy: ResolvedAccuracy,
    route: Route,
    compatibility: Option<Compatibility>,
    explicit_start: Option<WarmStart>,
    eligibility: Vec<routing::Eligibility>,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl PreparedSolve {
    /// Immutable compilation and normalization selected before attaching a seed.
    pub fn preparation_identity(&self) -> Result<pse_ids::ContentHash, ProblemError> {
        let mut h = FramedHasher::new(pse_ids::Frame::SolvePreparationV1);
        h.hash(&profile_key(&self.profile)?)
            .hash(&self.numerics.key);
        match &self.representation {
            Representation::Algebraic(AlgebraicCase {
                prepared,
                providers,
                factorable,
                ..
            }) => {
                for provider in providers.values() {
                    h.hash(&provider.configuration_key());
                }
                // The projection's key covers its implicit definitions and envelopes.
                if let Some((program, _)) = factorable {
                    h.hash(&program.key);
                }
                h.str("algebraic")
                    .hash(&prepared.compiled().plan.structure().key());
                for artifact in prepared.compiled().artifacts.iter() {
                    h.hash(&artifact.key());
                }
            }
            Representation::Conic { .. } => {
                h.str("conic");
            }
        }
        if let Some(c) = &self.compatibility {
            h.hash(&c.layout)
                .hash(&c.profile)
                .hash(&c.data)
                .str(c.backend.as_str());
        }
        Ok(h.finish_hash())
    }
    /// The preparation a stored seed is keyed by (ADR-0112 Outcome 17): the compiled
    /// structure, its artifacts and providers, and the seed coordinates and backend. Numeric
    /// data, the start policy, native options and attempt limits are excluded, because a
    /// seed in source coordinates stays valid across them (F24); it is `None` for a constant
    /// evaluation, which consumes no seed.
    pub fn seed_preparation_identity(&self) -> Option<pse_ids::ContentHash> {
        let compatibility = self.compatibility.as_ref()?;
        let mut h = FramedHasher::new(pse_ids::Frame::SolveSeedPreparationV1);
        match &self.representation {
            Representation::Algebraic(AlgebraicCase {
                prepared,
                providers,
                ..
            }) => {
                for provider in providers.values() {
                    h.hash(&provider.configuration_key());
                }
                h.str("algebraic")
                    .hash(&prepared.compiled().plan.structure().key());
                for artifact in prepared.compiled().artifacts.iter() {
                    h.hash(&artifact.key());
                }
            }
            Representation::Conic { .. } => {
                h.str("conic");
            }
        }
        h.hash(&compatibility.layout)
            .str(compatibility.backend.as_str());
        Some(h.finish_hash())
    }
    /// Complete selected request, including explicit seed payload and compatibility data.
    pub fn request_identity(&self) -> Result<pse_ids::ContentHash, ProblemError> {
        let mut h = FramedHasher::new(pse_ids::Frame::SolveRequestV1);
        h.hash(&self.preparation_identity()?)
            .hash(&self.numerics.key);
        if let Some(compatibility) = &self.compatibility {
            h.bool(true)
                .hash(&compatibility.layout)
                .hash(&compatibility.profile)
                .hash(&compatibility.data)
                .str(compatibility.backend.as_str());
        } else {
            h.bool(false);
        }
        if let Some(start) = &self.explicit_start {
            h.bool(true).str(&start.snapshot().to_string());
        } else {
            h.bool(false);
        }
        Ok(h.finish_hash())
    }
    /// Construct a primal-only explicit seed in semantic source coordinates.
    pub fn with_primal_start(
        self,
        values: BTreeMap<pse_ids::SemanticId, f64>,
    ) -> Result<Self, ProblemError> {
        let compatibility = self.compatibility.clone().ok_or_else(|| {
            ProblemError::Contract("constant evaluation has no numerical start".into())
        })?;
        let ids: Vec<_> = match &self.representation {
            Representation::Algebraic(a) => a.prepared.prepared.plan.columns().to_vec(),
            Representation::Conic { problem, .. } => {
                problem.contract.variables.iter().map(|v| v.id).collect()
            }
        };
        if values.len() != ids.len()
            || ids
                .iter()
                .any(|id| values.get(id).is_none_or(|v| !v.is_finite()))
        {
            return Err(ProblemError::Contract(
                "explicit start must cover every original free coordinate exactly once".into(),
            ));
        }
        let primal = ids.iter().map(|id| values[id]).collect();
        let payload = execution::adapter(compatibility.backend).primal_start(primal)?;
        self.with_start(WarmStart {
            origin: None,
            compatibility,
            payload,
        })
    }
    /// All contextual alternatives, distinct from the linked adapter inventory.
    pub fn eligibility(&self) -> &[routing::Eligibility] {
        &self.eligibility
    }
    /// Attach a compatible explicitly selected seed without changing allocation policy.
    pub fn with_start(mut self, seed: WarmStart) -> Result<Self, ProblemError> {
        let target = self.compatibility.as_ref().ok_or_else(|| {
            ProblemError::Contract("constant evaluation cannot consume a seed".into())
        })?;
        seed.validate(target)?;
        let (n, m) = match &self.representation {
            Representation::Algebraic(a) => (
                a.prepared.prepared.facts.variables,
                a.prepared.prepared.facts.rows,
            ),
            Representation::Conic { problem, .. } => (
                problem.contract.variables.len(),
                problem.contract.rows.len(),
            ),
        };
        seed.validate_shape(n, m)?;
        self.profile.controls.start = StartPolicy::Explicit;
        self.explicit_start = Some(seed);
        Ok(self)
    }
    /// Attach the parametric program of the profile's sensitivity request (Plan 22 S1):
    /// the case's plan with the requested parameters appended as coordinates
    /// ([`pse_math::assembly::CasePlan::parametric`]). Its parameter columns are normalized
    /// by the step's resolved numerical policy, which must resolve each parameter's
    /// coordinate scale.
    ///
    /// # Errors
    /// No request in the profile, a conic representation, a program whose columns are not
    /// the case's followed by the requested parameters, or an unresolved coordinate.
    pub fn with_sensitivity(
        mut self,
        program: Arc<ExecutableCase>,
    ) -> Result<Self, MathRuntimeError> {
        let request = self.profile.sensitivity.clone().ok_or_else(|| {
            ProblemError::Contract("no sensitivity request to attach a program to".into())
        })?;
        request.admit(self.profile.intent)?;
        let Representation::Algebraic(case) = &mut self.representation else {
            return Err(ProblemError::Contract(
                "parametric sensitivities need an algebraic case".into(),
            )
            .into());
        };
        let plan = &case.prepared.prepared.plan;
        let columns = program.assembly.columns();
        if columns.len() != plan.columns().len() + request.parameters.len()
            || columns[..plan.columns().len()] != *plan.columns()
            || columns[plan.columns().len()..] != *request.parameters
            || program.assembly.structure().key() != plan.structure().key()
        {
            return Err(ProblemError::Contract(
                "the parametric program is not the case's columns followed by the requested parameters".into(),
            )
            .into());
        }
        let rows: Vec<_> = plan.structure().rows().iter().map(|r| r.id).collect();
        let normalization = Normalization::from_policy(&self.numerics, columns, &rows)?;
        let parameters = request
            .parameters
            .iter()
            .map(|id| {
                case.values
                    .scalars
                    .get(id)
                    .map(|v| (*id, *v))
                    .ok_or_else(|| ProblemError::Contract(format!("no value for parameter {id}")))
            })
            .collect::<Result<_, _>>()?;
        case.sensitivity = Some(SensitivityProgram {
            program,
            normalization,
            parameters,
            reduced_hessian: request.reduced_hessian,
            retain: false,
        });
        Ok(self)
    }
    /// Keep the attached sensitivity request's pinned factor after the step, for an
    /// advanced-step prediction (Plan 22 Y5c2): the worker's retained state holds it,
    /// charged to the job's allowance, until a later step replaces or releases it. Only the
    /// callback route keeps one.
    ///
    /// # Errors
    /// No sensitivity program is attached.
    pub fn retaining_factor(mut self) -> Result<Self, MathRuntimeError> {
        match &mut self.representation {
            Representation::Algebraic(AlgebraicCase {
                sensitivity: Some(program),
                ..
            }) => {
                program.retain = true;
                Ok(self)
            }
            _ => Err(ProblemError::Contract(
                "an advanced step keeps the factor of an attached sensitivity request".into(),
            )
            .into()),
        }
    }
    /// Current quadratic evidence, including explicit inconclusive or numerical assessments.
    pub fn quadratic_evidence(&self) -> Option<&dyn QuadraticEvidence> {
        match &self.representation {
            Representation::Algebraic(a) => a.certificate.as_deref(),
            Representation::Conic { certificate, .. } => Some(certificate.as_ref()),
        }
    }

    /// Immutable physical requirements and their provenance.
    pub fn numerics(&self) -> &ResolvedNumericalPolicy {
        &self.numerics
    }
    /// Frozen original-coordinate acceptance budgets.
    pub fn tolerances(&self) -> &Tolerances {
        &self.tolerances
    }
    /// Native stopping budgets resolved from the numerical policy at preparation.
    pub fn accuracy(&self) -> &ResolvedAccuracy {
        &self.accuracy
    }
    /// The absolute accuracy of the objective value in original units: the continuous
    /// absolute gap budget at the objective's coordinate scale.
    pub fn objective_accuracy(&self) -> f64 {
        self.accuracy.gap_absolute * self.normalization.objective
    }
    /// Deterministic selected route, available for inspection before admission.
    pub fn route(&self) -> Route {
        self.route
    }
    /// Semantic/numerical reuse identity, absent for all-fixed validation.
    pub fn compatibility(&self) -> Option<&Compatibility> {
        self.compatibility.as_ref()
    }
    /// Retained result allowance of one attempt of this step.
    pub(crate) fn result_bytes(&self) -> Result<usize, MathRuntimeError> {
        let (n, m) = match &self.representation {
            Representation::Algebraic(a) => (
                a.prepared.prepared.facts.variables,
                a.prepared.prepared.facts.rows,
            ),
            Representation::Conic { problem, .. } => (
                problem.contract.variables.len(),
                problem.contract.rows.len(),
            ),
        };
        let sources = match &self.representation {
            Representation::Algebraic(a) => a
                .prepared
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
            .and_then(|v| v.checked_add(self.profile.controls.report_allowance().ok()?))
            .ok_or(MathRuntimeError::Limit("solve result allowance"))
    }
    /// The step's own starting point as a primal seed for its coordinates: what a later
    /// stage of a conditional block receives from the block's committed predecessor.
    #[cfg(feature = "solver-kinsol")]
    pub(crate) fn primal_seed(&self) -> Result<WarmStart, ProblemError> {
        let compatibility = self.compatibility.clone().ok_or_else(|| {
            ProblemError::Contract("constant evaluation has no numerical start".into())
        })?;
        let Representation::Algebraic(a) = &self.representation else {
            return Err(ProblemError::Contract(
                "a cone step has no primal seed".into(),
            ));
        };
        let primal = a
            .prepared
            .prepared
            .plan
            .columns()
            .iter()
            .map(|id| {
                a.values
                    .scalars
                    .get(id)
                    .copied()
                    .ok_or_else(|| ProblemError::Contract("missing start coordinate".into()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(WarmStart {
            origin: None,
            payload: execution::adapter(compatibility.backend).primal_start(primal)?,
            compatibility,
        })
    }
    /// Native threads this step is admitted with.
    pub(crate) fn threads(&self) -> usize {
        self.profile.controls.threads
    }
    /// The adapter whose scope this step's native state lives in.
    pub(crate) fn backend(&self) -> Option<Backend> {
        match self.route {
            Route::Native(backend) => Some(backend),
            Route::Constant => None,
        }
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
    Constant(Box<ConstantReport>),
    /// A typed admission/execution failure before a native report became available.
    Rejected(Arc<MathRuntimeError>),
}
impl Outcome {
    /// Registry run state of this outcome: the one name of each kind of step result.
    pub const fn state(&self) -> pse_model::generated::enums::NativeRunState {
        use pse_model::generated::enums::NativeRunState;
        match self {
            Self::Native(_) => NativeRunState::Native,
            Self::Constant(_) => NativeRunState::ConstantEvaluation,
            Self::Rejected(_) => NativeRunState::Rejected,
        }
    }
    /// The native candidate-use decision of the workflow completion owner (§16.6,
    /// ADR-0106). Seeding, commits, homotopy, studies and publication consume it.
    pub(crate) fn candidate_use(&self) -> crate::workflow::numerics::CandidateDecision {
        use crate::workflow::numerics;
        match self {
            Self::Rejected(_) => numerics::refused(numerics::CandidateReason::NoCandidate),
            Self::Constant(r) => numerics::constant_use(&r.quality),
            Self::Native(r) => numerics::native_use(r),
        }
    }
}
/// Owned result of one prepared step, retaining its result allowance through the last reader.
#[derive(Debug)]
pub struct StepReport {
    /// The step's native attempt, constant evaluation or typed refusal.
    pub outcome: Outcome,
    _owner: Arc<pse_columnar::AllocationLease>,
}
/// The output seed of an earlier accepted step, offered to a step whose start policy is
/// `PreviousAccepted` (§17.6).
#[derive(Clone, Debug)]
pub(crate) struct Predecessor {
    /// The accepted step, recorded in the receipt.
    pub attempt: usize,
    /// Its output seed.
    pub seed: WarmStart,
}
/// Dropping the handle requests cancellation; awaiting it witnesses native destruction and join.
pub struct SolveHandle<T = StepReport> {
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
impl<T: Send + 'static> SolveHandle<T> {
    /// Supervise asynchronous native work under this handle's lifecycle: cancelling or
    /// dropping the handle cancels `work`'s source, and finishing waits until `work` has
    /// completed, including every native join it awaits.
    pub(crate) fn supervise<F>(
        progress: Arc<Progress>,
        work: impl FnOnce(crate::CancelSource) -> F,
    ) -> Self
    where
        F: Future<Output = Result<T, MathRuntimeError>> + Send + 'static,
    {
        let cancel = FlightCancellation::default();
        let control = cancel.clone();
        let source = crate::CancelSource::new();
        let operation = work(source.clone());
        let (sender, receiver) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            tokio::pin!(operation);
            let result = tokio::select! {
                result = &mut operation => result,
                () = control.cancelled() => {
                    source.cancel();
                    operation.await
                }
            };
            let _ = sender.send(result);
        });
        Self {
            cancel,
            receiver: Some(receiver),
            progress,
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
/// Admit a selected route's typed settings and controls through its adapter. Required
/// library preprocessing needs an NLP representation.
pub(crate) fn admit_profile(profile: &SolverProfile, route: Route) -> Result<(), ProblemError> {
    let adapter = match route {
        Route::Native(backend) => Some(execution::adapter(backend)),
        Route::Constant => None,
    };
    if adapter.is_none_or(|a| a.representation() != execution::Representation::Nlp)
        && matches!(&profile.presolve, native::presolve::Policy::Explicit { required, .. } if !required.is_empty())
    {
        return Err(ProblemError::Unsupported("common NLP scales/required preprocessing need an NLP route; use the selected class's native controls".into()));
    }
    let Some(adapter) = adapter else {
        return Ok(());
    };
    if !adapter.representation().algebraic() || !adapter.linked() {
        return Err(ProblemError::Unavailable {
            backend: adapter.backend(),
            alternatives: vec![],
        });
    }
    adapter.admit_settings(&profile.backend, &profile.controls)
}
/// The native session profile: every control and setting a retained native session
/// depends on. Controls and backend settings are identified through serde (F09); only the
/// per-attempt budgets and the sequencing policies, which every attempt re-applies, are
/// left out, so a new control field enters the identity without an edit here.
fn hash_session(h: &mut FramedHasher, p: &SolverProfile) -> Result<(), ProblemError> {
    h.hash(&p.presolve.key()).hash(&p.numerics.key());
    match p.convexity {
        ConvexityPolicy::Exact => {
            h.u64(0);
        }
        ConvexityPolicy::Numerical { absolute, relative } => {
            h.u64(1).u64(absolute.to_bits()).u64(relative.to_bits());
        }
    }
    let session = Controls {
        time_limit: std::time::Duration::ZERO,
        iterations: 0,
        history: 0,
        reuse: ReusePolicy::Fresh,
        start: StartPolicy::NoPriorStart,
        ..p.controls.clone()
    };
    h.str(p.intent.as_str())
        .hash(&session.identity()?)
        .hash(&p.backend.identity()?);
    Ok(())
}
fn compatibility(
    plan: &pse_math::assembly::CasePlan,
    values: &CaseValues,
    p: &SolverProfile,
    numerics: &ResolvedNumericalPolicy,
    backend: Backend,
    providers: &BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
) -> Result<Compatibility, ProblemError> {
    // Seed coordinates only: backend, objective sense, free variables, rows and
    // structure. Controls, settings and the numerical policy form the profile (F24).
    let mut layout = FramedHasher::new(pse_ids::Frame::SolverCoordinatesV1);
    layout
        .str(backend.as_str())
        .u64(plan.structure().objective().map_or(0, |o| {
            if o.sense == ObjectiveSense::Minimize {
                1
            } else {
                2
            }
        }));
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
    let mut profile = FramedHasher::new(pse_ids::Frame::SolverSessionV1);
    hash_session(&mut profile, p)?;
    profile.hash(&numerics.key);
    let mut data = FramedHasher::new(pse_ids::Frame::SolverDataV1);
    data.hash(&plan.structure().key());
    for provider in providers.values() {
        data.hash(&provider.configuration_key());
    }
    for (id, v) in &values.scalars {
        data.id(id).u64(v.to_bits());
    }
    Ok(Compatibility {
        layout: layout.finish_hash(),
        profile: profile.finish_hash(),
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
        numerical: NumericalInputs,
    ) -> Result<PreparedSolve, MathRuntimeError> {
        profile.controls.validate()?;
        let mut targets = prepared
            .prepared
            .plan
            .numerical_targets(&prepared.prepared.quantities)?;
        targets.extend(numerical.targets);
        let numerics = Arc::new(pse_math::numerics::resolve(
            &prepared.prepared.quantities,
            &targets,
            &numerical.declarations,
            &profile.numerics,
        )?);
        self.prepare_resolved(
            prepared,
            values,
            providers,
            profile,
            numerics,
            numerical.implicit,
        )
        .await
    }
    /// Admission of a bound case under an already resolved numerical policy. A conditional
    /// initialization block resolves its policy once for the whole case and passes it here.
    /// `implicit` holds the residual definitions a factorable route exports.
    ///
    /// Convexity is the preparation's fact (ADR-0121): routing reads it, and a convex
    /// quadratic coefficient objective carries its exact certificate. An explicit
    /// [`ConvexityPolicy::Numerical`] may qualify this request's coefficient objective as
    /// positive semidefinite when the fact does not; that evidence serves this request only
    /// and never becomes a fact.
    #[expect(
        clippy::too_many_lines,
        reason = "one admission binds routing, the representation each route needs and its identity"
    )]
    pub(crate) async fn prepare_resolved(
        self: &Arc<Self>,
        prepared: Preparation,
        values: CaseValues,
        providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
        profile: SolverProfile,
        numerics: Arc<ResolvedNumericalPolicy>,
        implicit: BTreeMap<pse_kernels::ProviderKey, pse_math::factorable::ImplicitDefinition>,
    ) -> Result<PreparedSolve, MathRuntimeError> {
        profile.controls.validate()?;
        // Per-solve overlays are separate from the shared compiler product.
        let structure = prepared.prepared.plan.structure();
        let entries = structure
            .variables()
            .len()
            .checked_add(structure.rows().len())
            .and_then(|n| n.checked_add(numerics.targets.len()))
            .ok_or(MathRuntimeError::Limit("solve metadata extent"))?;
        let bytes = entries
            .checked_mul(size_of::<pse_math::numerics::TargetSpec>() + 8 * size_of::<f64>())
            .and_then(|n| {
                n.checked_add(values.scalars.len() * size_of::<(pse_ids::SemanticId, f64)>())
            })
            .and_then(|n| n.checked_add(size_of::<PreparedSolve>()))
            .and_then(|n| n.checked_add(self.policy.foreign_bytes))
            .ok_or(MathRuntimeError::Limit("solve metadata extent"))?;
        let owner = self.reserve("math:prepared-solve", bytes)?;
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
            return Err(ProblemError::Contract("fixed/parameter values differ from compiler assumptions; rebind the prepared structure to these values".into()).into());
        }
        let f = &prepared.prepared.facts;
        let plan = &prepared.prepared.plan;
        let rows: Vec<_> = plan.structure().rows().iter().map(|r| r.id).collect();
        let normalization = Normalization::from_policy(&numerics, plan.columns(), &rows)?;
        let tolerances = Tolerances::from_policy(&numerics, plan.columns(), &rows)?;
        let accuracy = ResolvedAccuracy::resolve(&numerics.policy, &tolerances, &normalization)?;

        let mut certificate: Option<Arc<dyn QuadraticEvidence>> = f
            .convexity
            .convex_quadratic()
            .map(|c| -> Arc<dyn QuadraticEvidence> { c.clone() });
        let mut numerical_psd = false;
        if let Some(c) = &prepared.prepared.coefficients {
            if prepared
                .prepared
                .coefficient_values
                .iter()
                .any(|(id, bits)| values.scalars.get(id).map(|v| v.to_bits()) != Some(*bits))
            {
                return Err(ProblemError::Contract("case values differ from the compiler's coefficient assumptions; prepare a new snapshot".into()).into());
            }
            if let (None, true, ConvexityPolicy::Numerical { absolute, relative }) =
                (&certificate, f.quadratic, profile.convexity)
            {
                let coefficients = c.clone();
                let coordinates = normalization.clone();
                let sign = plan.structure().objective().map_or(1.0, |o| o.sense.sign());
                let bytes = self.policy.worker_bytes;
                let evidence = self
                    .job(1, bytes, FlightCancellation::default(), move |flag| {
                        Ok(coefficients.numerical_convexity(
                            sign,
                            &coordinates.variables,
                            coordinates.objective,
                            absolute,
                            relative,
                            bytes,
                            &flag,
                        )?)
                    })
                    .await?;
                numerical_psd = evidence.accepted();
                certificate = Some(Arc::new(evidence));
            }
        }
        let requirements = routing::Requirements {
            table: &execution::LINKED,
            facts: f,
            intent: profile.intent,
            numerical_psd,
            least_squares: false,
            controls: &profile.controls,
            settings: &profile.backend,
            sensitivity: profile.sensitivity.is_some(),
        };
        let route = requirements.select(profile.selection)?;
        let eligibility = requirements.eligibility();
        // An authored realization's structural requirement selects the method it needs on
        // the route it admitted (ADR-0104 §5): the author's selection, recorded with the
        // result, never an automatic choice.
        let profile = match route {
            Route::Native(backend) => SolverProfile {
                backend: profile.backend.for_requirements(backend, &f.requirements),
                ..profile
            },
            Route::Constant => profile,
        };
        let adapter = match route {
            Route::Native(backend) => Some(execution::adapter(backend)),
            Route::Constant => None,
        };
        match adapter.map(|a| a.representation()) {
            Some(execution::Representation::Roots) => {
                native::structural::admit(prepared.structure(), native::structural::Mode::Roots)?
            }
            Some(execution::Representation::Nlp) => {
                native::structural::admit(prepared.structure(), native::structural::Mode::Nlp)?
            }
            _ => {}
        }
        admit_profile(&profile, route)?;
        // The factorable projection exists only for a factorable route. It is built and
        // admitted before any worker, refusing with every typed reason (ADR-0105 §2).
        let factorable = match adapter.map(|a| a.representation()) {
            Some(execution::Representation::Factorable) => {
                let plan = prepared.prepared.plan.clone();
                let values = values.clone();
                let limit = self.policy.worker_bytes / 256;
                let intent = profile.intent;
                // Implicit blocks export their residuals; providers that declare an
                // enforced envelope export as auxiliaries inside it (ADR-0105 §1).
                let mut envelopes = BTreeMap::new();
                for (key, registration) in &providers {
                    if let Some(envelope) = registration.envelope() {
                        envelopes.insert(*key, envelope.to_vec());
                    }
                }
                let request = pse_math::factorable::FactorableRequest {
                    implicit,
                    envelopes,
                    ..Default::default()
                };
                let program = self
                    .job(
                        1,
                        self.policy.worker_bytes,
                        FlightCancellation::default(),
                        move |flag| {
                            let program = plan
                                .factorable_program(&values, &request, limit, &flag)
                                .map_err(|e| match e {
                                    pse_math::factorable::FactorableError::Math(e) => {
                                        ProblemError::Math(e)
                                    }
                                    other => ProblemError::Unsupported(other.to_string()),
                                })?;
                            let refusals = execution::admit_program(&program, intent);
                            if !refusals.is_empty() {
                                let reasons: Vec<String> =
                                    refusals.iter().map(ToString::to_string).collect();
                                return Err(ProblemError::Unsupported(format!(
                                    "factorable export refused: {}",
                                    reasons.join("; ")
                                ))
                                .into());
                            }
                            Ok(program)
                        },
                    )
                    .await?;
                let owner = self.reserve("math:factorable-program", program.bytes())?;
                Some((Arc::new(program), owner))
            }
            _ => None,
        };
        // A recognized convex program on a cone route is rebuilt, recognized again and
        // lowered to cone form before any worker (ADR-0121 Outcome 6). A coefficient program
        // on a cone route is lowered by the coefficient runner instead.
        let recognized = match adapter.map(|a| a.representation()) {
            Some(execution::Representation::Cone)
                if prepared.prepared.coefficients.is_none() && f.convexity.cone() =>
            {
                let plan = prepared.prepared.plan.clone();
                let values = values.clone();
                let limit = self.policy.worker_bytes / 256;
                let intent = profile.intent;
                let fact = f.convexity.clone();
                let lowered = self
                    .job(
                        1,
                        self.policy.worker_bytes,
                        FlightCancellation::default(),
                        move |flag| {
                            let program = plan
                                .factorable_program(
                                    &values,
                                    &pse_math::factorable::FactorableRequest::default(),
                                    limit,
                                    &flag,
                                )
                                .map_err(|e| match e {
                                    pse_math::factorable::FactorableError::Math(e) => {
                                        ProblemError::Math(e)
                                    }
                                    other => ProblemError::Unsupported(other.to_string()),
                                })?;
                            Ok(native::conic::lower(&program, &fact, intent, &flag)?)
                        },
                    )
                    .await?;
                let owner = self.reserve("math:recognized-cone", lowered.bytes())?;
                Some((Arc::new(lowered), owner))
            }
            _ => None,
        };
        if let Some(adapter) = adapter {
            adapter.admit_contract(
                &native::assembled::contract(plan),
                &prepared.prepared.presolve.signs,
                &profile.backend,
                execution::Budgets {
                    tolerances: &tolerances,
                    normalization: &normalization,
                    feasibility: accuracy.feasibility,
                },
            )?;
        }
        let stamp = match route {
            Route::Constant => None,
            Route::Native(backend) => Some(compatibility(
                &prepared.prepared.plan,
                &values,
                &profile,
                &numerics,
                backend,
                &providers,
            )?),
        };
        let case = Some(self.assemble(prepared.clone()).await?);
        Ok(PreparedSolve {
            representation: Representation::Algebraic(AlgebraicCase {
                prepared,
                case,
                values,
                providers,
                certificate,
                factorable,
                sensitivity: None,
                recognized,
            }),
            profile,
            numerics,
            normalization,
            tolerances,
            accuracy,
            route,
            compatibility: stamp,
            explicit_start: None,
            eligibility,
            _owner: owner,
        })
    }
    /// A conditional initialization block as a solve step (A6): the block's bound view, its
    /// assembled programs, the case-level numerical policy and the route resolved for the
    /// block before execution ([`super::initialization::PreparedInitialization::strategies`]).
    /// The block's acceptance budgets and transport come from that policy over its own
    /// columns and rows, as for any solve.
    #[cfg(feature = "solver-kinsol")]
    #[expect(
        clippy::too_many_arguments,
        reason = "a block step binds its view, programs, values, providers, profile, policy and route"
    )]
    pub(crate) fn prepare_conditional(
        &self,
        prepared: Preparation,
        executable: Arc<ExecutableCase>,
        values: CaseValues,
        providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
        profile: SolverProfile,
        numerics: Arc<ResolvedNumericalPolicy>,
        route: Route,
    ) -> Result<PreparedSolve, MathRuntimeError> {
        profile.controls.validate()?;
        let plan = &prepared.prepared.plan;
        plan.structure().validate_values(&values)?;
        if !prepared.prepared.presolve.matches(plan, &values) {
            return Err(ProblemError::Internal(
                "conditional block values differ from its bound view".into(),
            )
            .into());
        }
        let Route::Native(backend) = route else {
            return Err(ProblemError::Unsupported(
                "unsupported conditional initialization route".into(),
            )
            .into());
        };
        if !matches!(
            execution::adapter(backend).representation(),
            execution::Representation::Roots | execution::Representation::Nlp
        ) {
            return Err(ProblemError::Unsupported(
                "unsupported conditional initialization route".into(),
            )
            .into());
        }
        admit_profile(&profile, route)?;
        let rows: Vec<_> = plan.structure().rows().iter().map(|r| r.id).collect();
        let normalization = Normalization::from_policy(&numerics, plan.columns(), &rows)?;
        let tolerances = Tolerances::from_policy(&numerics, plan.columns(), &rows)?;
        let accuracy = ResolvedAccuracy::resolve(&numerics.policy, &tolerances, &normalization)?;
        let stamp = compatibility(plan, &values, &profile, &numerics, backend, &providers)?;
        let bytes = (plan.structure().variables().len() + rows.len())
            .checked_mul(size_of::<pse_math::numerics::TargetSpec>() + 8 * size_of::<f64>())
            .and_then(|n| n.checked_add(size_of::<PreparedSolve>()))
            .ok_or(MathRuntimeError::Limit("solve metadata extent"))?;
        Ok(PreparedSolve {
            representation: Representation::Algebraic(AlgebraicCase {
                prepared,
                case: Some(executable),
                values,
                providers,
                certificate: None,
                // A block runs only on a root or NLP route, refused above otherwise.
                factorable: None,
                sensitivity: None,
                recognized: None,
            }),
            profile,
            numerics,
            normalization,
            tolerances,
            accuracy,
            route,
            compatibility: Some(stamp),
            explicit_start: None,
            eligibility: vec![],
            _owner: self.reserve("math:prepared-block", bytes)?,
        })
    }
    /// Explicit conic representation enters the same bounded worker/report lifecycle.
    pub async fn prepare_conic(
        self: &Arc<Self>,
        problem: Arc<native::ConicProblem>,
        certificate: Arc<dyn QuadraticEvidence>,
        profile: SolverProfile,
        numerics: Arc<ResolvedNumericalPolicy>,
    ) -> Result<PreparedSolve, MathRuntimeError> {
        profile.controls.validate()?;
        if profile.numerics.key() != numerics.policy.key() {
            return Err(ProblemError::Contract(
                "conic numerical policy differs from its resolved authority".into(),
            )
            .into());
        }
        let sparse_bytes = [&problem.quadratic, &problem.constraints]
            .iter()
            .try_fold(0usize, |n, a| {
                n.checked_add(
                    a.column_starts
                        .capacity()
                        .checked_add(a.row_indices.capacity())?
                        .checked_mul(size_of::<usize>())?,
                )?
                .checked_add(a.values.capacity().checked_mul(size_of::<f64>())?)
            })
            .ok_or(MathRuntimeError::Limit("conic product extent"))?;
        // The normalized copy beside the retained original.
        let bytes = (problem.contract.variables.len() + problem.contract.rows.len())
            .checked_mul(size_of::<pse_math::numerics::TargetSpec>() + 8 * size_of::<f64>())
            .and_then(|n| n.checked_add(sparse_bytes.checked_mul(2)?))
            .and_then(|n| n.checked_add(self.policy.foreign_bytes))
            .ok_or(MathRuntimeError::Limit("conic product extent"))?;
        let owner = self.reserve("math:prepared-conic", bytes)?;
        let admitted = problem.clone();
        let proof = certificate.clone();
        self.job(
            1,
            self.policy.worker_bytes,
            FlightCancellation::default(),
            move |_| admitted.validate(proof.as_ref()).map_err(Into::into),
        )
        .await?;
        let ids: Vec<_> = problem.contract.variables.iter().map(|v| v.id).collect();
        let normalization = native::transport::cone_normalization(&problem, &numerics)?;
        certificate.validate_policy(
            profile.convexity,
            &normalization.variables,
            normalization.objective,
        )?;
        let mut resolved = numerics.as_ref().clone();
        for (id, scale) in problem.contract.rows.iter().zip(&normalization.rows) {
            let target = resolved
                .targets
                .iter_mut()
                .find(|t| {
                    t.id == *id && t.kind == pse_model::generated::enums::NumericalTarget::Row
                })
                .ok_or_else(|| ProblemError::Contract("missing cone numerical row".into()))?;
            if target.coordinate_scale != *scale {
                target.coordinate_scale = *scale;
                target
                    .provenance
                    .push(pse_model::numerics::NumericalProvenance {
                        declaration: None,
                        source: pse_model::generated::enums::NumericalSource::CanonicalFallback,
                        field:
                            pse_model::generated::enums::NumericalProvenanceField::CoordinateScale,
                        selected: true,
                        value: *scale,
                        description: "common positive scale preserving the declared cone geometry"
                            .into(),
                    });
            }
        }
        let mut identity = FramedHasher::new(pse_ids::Frame::NumericalConeV1);
        identity.hash(&resolved.key).hash(&normalization.key());
        resolved.key = identity.finish_hash();
        let numerics = Arc::new(resolved);
        let tolerances = Tolerances::from_policy(&numerics, &ids, &problem.contract.rows)?;
        let accuracy = ResolvedAccuracy::resolve(&numerics.policy, &tolerances, &normalization)?;
        // A cone request routes to a cone adapter: the explicit one, or the preferred linked
        // automatic owner of explicit cones (ADR-0121).
        let adapter = match profile.selection {
            SolverSelection::Explicit(backend) => Some(execution::adapter(backend)),
            SolverSelection::Auto => execution::LINKED
                .adapters()
                .filter(|a| a.representation() == execution::Representation::Cone && a.linked())
                .filter(|a| {
                    a.capability()
                        .automatic_classes
                        .contains(&ProblemClass::ContinuousCone)
                })
                .filter_map(|a| a.automatic().map(|rank| (rank, a)))
                .min_by_key(|(rank, _)| *rank)
                .map(|(_, a)| a),
        }
        .filter(|a| a.representation() == execution::Representation::Cone);
        let Some(adapter) = adapter.filter(|_| profile.intent == SolveIntent::Optimize) else {
            return Err(ProblemError::Unsupported(
                "explicit continuous cone representation requires cone optimization".into(),
            )
            .into());
        };
        let route = Route::Native(adapter.backend());
        admit_profile(&profile, route)?;
        let (normalized, transported) =
            native::transport::conic(&problem, &normalization, certificate.as_ref())?;
        let original = problem;
        let problem = Arc::new(normalized);
        let certificate: Arc<dyn QuadraticEvidence> = Arc::new(transported);
        // Cone coordinates are normalized at preparation, so the numerical policy belongs to
        // them; controls and settings form the profile (F24).
        let mut h = FramedHasher::new(pse_ids::Frame::SolverConicLayoutV3);
        h.hash(&numerics.key).hash(&normalization.key());
        h.hash(&problem.contract.identity);
        let mut session = FramedHasher::new(pse_ids::Frame::SolverConicSessionV1);
        hash_session(&mut session, &profile)?;
        h.hash(&native::conic::cone_key(&problem.cones)?);
        for v in &problem.contract.variables {
            h.id(&v.id);
        }
        for id in &problem.contract.rows {
            h.id(id);
        }
        for a in [&problem.quadratic, &problem.constraints] {
            h.u64(a.rows as u64)
                .u64(a.columns as u64)
                .u64(a.column_starts.len() as u64)
                .u64(a.row_indices.len() as u64);
            for v in a.column_starts.iter().chain(&a.row_indices) {
                h.u64(*v as u64);
            }
        }
        let mut d = FramedHasher::new(pse_ids::Frame::SolverConicDataV1);
        for x in problem
            .quadratic
            .values
            .iter()
            .chain(&problem.constraints.values)
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
            profile: session.finish_hash(),
            data: d.finish_hash(),
            backend: adapter.backend(),
        };
        Ok(PreparedSolve {
            representation: Representation::Conic {
                problem,
                original,
                certificate,
            },
            profile,
            numerics,
            normalization,
            tolerances,
            accuracy,
            route,
            compatibility: Some(stamp),
            explicit_start: None,
            eligibility: vec![routing::Eligibility {
                backend: adapter.backend(),
                reasons: vec![],
            }],
            _owner: owner,
        })
    }
    /// Start one prepared step on its own native session: the job lifecycle every single
    /// solve shares with staged sequences (A6). Dropping the handle requests cancellation;
    /// finishing witnesses native teardown and join.
    ///
    /// # Errors
    /// An explicit start policy without a seed, or no admission capacity.
    pub fn solve(
        self: &Arc<Self>,
        step: PreparedSolve,
    ) -> Result<SolveHandle<StepReport>, MathRuntimeError> {
        if step.profile.controls.start == StartPolicy::Explicit && step.explicit_start.is_none() {
            return Err(ProblemError::Contract(
                "explicit start policy requires a seed before submission".into(),
            )
            .into());
        }
        let owner = self.reserve("math:solve-results", step.result_bytes()?)?;
        let progress = Arc::new(Progress::new(step.profile.controls.history));
        let session = self.open_session()?;
        let events = progress.clone();
        Ok(SolveHandle::supervise(progress, move |cancel| async move {
            let result = session
                .step(step, None, 0, events, owner.clone(), &cancel, |_, _, _| {
                    ((), true)
                })
                .await;
            session.close().await;
            result.map(|(outcome, ())| StepReport {
                outcome,
                _owner: owner,
            })
        }))
    }
    /// One bound step on a session's retained native state (A6): the seed its start policy
    /// selects, checked against its coordinates, the selected adapter's representation
    /// runner and the submitted-start receipt. A `Fresh` reuse policy drops retained state
    /// first; a refused seed drops it as well.
    #[expect(
        clippy::too_many_arguments,
        reason = "the session supplies its retained state, stop flag, progress, worker share and result owner"
    )]
    pub(crate) fn execute(
        &self,
        step: PreparedSolve,
        previous: Option<Predecessor>,
        attempt: usize,
        retained: &mut Retained,
        flag: &Arc<std::sync::atomic::AtomicBool>,
        progress: &Arc<Progress>,
        budget: &Arc<WorkerBudget>,
        owner: &Arc<pse_columnar::AllocationLease>,
    ) -> Result<Outcome, MathRuntimeError> {
        let admitted = match self.admit_step(step, previous, retained, flag, progress) {
            Ok(admitted) => admitted,
            Err(refused) => return Ok(refused),
        };
        let Admitted {
            step,
            chosen,
            execution,
            receipt,
            normalization,
        } = admitted;
        let outcome = self
            .run_step(step, execution, chosen.as_ref(), retained, budget)
            .unwrap_or_else(|e| Outcome::Rejected(Arc::new(e)));
        self.conclude(outcome, receipt, normalization, attempt, owner)
    }
    /// The independent steps of one batch on the session's retained native state (Plan 22
    /// N5): each is admitted as [`Self::execute`] admits it, those that share one batching
    /// adapter over coefficient programs are solved together
    /// ([`execution::coefficients_batch`]), and each outcome is concluded on its own. Steps
    /// that cannot join the batch run in turn. One outcome per step, in order.
    pub(crate) fn execute_batch(
        &self,
        members: Vec<BatchMember>,
        retained: &mut Retained,
        flag: &Arc<std::sync::atomic::AtomicBool>,
        progress: &Arc<Progress>,
        budget: &Arc<WorkerBudget>,
    ) -> Vec<Result<Outcome, MathRuntimeError>> {
        let mut outcomes: Vec<Option<Result<Outcome, MathRuntimeError>>> =
            members.iter().map(|_| None).collect();
        let mut admitted = Vec::new();
        for (i, member) in members.into_iter().enumerate() {
            match self.admit_step(member.step, None, retained, flag, progress) {
                Ok(step) => admitted.push((i, member.attempt, member.owner, step)),
                Err(refused) => outcomes[i] = Some(Ok(refused)),
            }
        }
        let batching = |a: &Admitted| {
            let Route::Native(backend) = a.step.route else {
                return None;
            };
            let adapter = execution::adapter(backend);
            let coefficients = matches!(
                &a.step.representation,
                Representation::Algebraic(case)
                    if case.prepared.prepared.coefficients.is_some()
                        && case.recognized.is_none()
                        && case.sensitivity.is_none()
            );
            (adapter.capability().batch
                && coefficients
                && matches!(
                    adapter.representation(),
                    execution::Representation::Coefficients | execution::Representation::Cone
                ))
            .then_some(backend)
        };
        let shared = admitted
            .first()
            .and_then(|(_, _, _, a)| batching(a))
            .filter(|backend| {
                admitted
                    .iter()
                    .all(|(_, _, _, a)| batching(a) == Some(*backend))
            });
        if shared.is_some() && admitted.len() > 1 {
            let concluded = self.coefficient_batch(admitted, retained, budget);
            for (i, outcome) in concluded {
                outcomes[i] = Some(outcome);
            }
        } else {
            for (i, attempt, owner, a) in admitted {
                let Admitted {
                    step,
                    chosen,
                    execution,
                    receipt,
                    normalization,
                } = a;
                let outcome = self
                    .run_step(step, execution, chosen.as_ref(), retained, budget)
                    .unwrap_or_else(|e| Outcome::Rejected(Arc::new(e)));
                outcomes[i] = Some(self.conclude(outcome, receipt, normalization, attempt, &owner));
            }
        }
        outcomes
            .into_iter()
            .map(|o| {
                o.unwrap_or_else(|| {
                    Err(MathRuntimeError::Infrastructure(
                        "batch member without an outcome".into(),
                    ))
                })
            })
            .collect()
    }
    /// Admit one step: its reuse policy, its seed by start policy, checked against its
    /// coordinates, its execution controls and its start receipt; or the outcome that
    /// refuses it.
    fn admit_step(
        &self,
        step: PreparedSolve,
        previous: Option<Predecessor>,
        retained: &mut Retained,
        flag: &Arc<std::sync::atomic::AtomicBool>,
        progress: &Arc<Progress>,
    ) -> Result<Admitted, Outcome> {
        let controls = step.profile.controls.clone();
        if controls.reuse == ReusePolicy::Fresh {
            retained.clear();
        }
        let (chosen, previous_attempt) = match controls.start {
            StartPolicy::NoPriorStart => (None, None),
            StartPolicy::Explicit => match step.explicit_start.clone() {
                Some(seed) => (Some(seed), None),
                None => {
                    return Err(Outcome::Rejected(Arc::new(
                        ProblemError::Contract("explicit start policy requires a seed".into())
                            .into(),
                    )));
                }
            },
            StartPolicy::PreviousAccepted => {
                previous.map_or((None, None), |p| (Some(p.seed), Some(p.attempt)))
            }
        };
        if let Some(seed) = &chosen {
            let validation = step
                .compatibility
                .as_ref()
                .ok_or_else(|| {
                    ProblemError::Contract("constant evaluation cannot consume a seed".into())
                })
                .and_then(|target| seed.validate(target));
            if let Err(error) = validation {
                retained.clear();
                return Err(Outcome::Rejected(Arc::new(error.into())));
            }
        }
        let mut execution = Execution::new(flag.clone(), &controls);
        execution.progress = progress.clone();
        // Libraries that enforce their own memory limit read the job's foreign allowance.
        execution.memory = Some(self.policy.foreign_bytes);
        let receipt = StartReceipt {
            previous_attempt,
            seed: chosen.clone(),
            sparse_seed: step.profile.backend.partial_start().cloned(),
            transformations: vec![],
            submitted: chosen.is_some(),
        };
        let normalization = step.normalization.key();
        Ok(Admitted {
            step,
            chosen,
            execution,
            receipt,
            normalization,
        })
    }
    /// A step's outcome with its failure owner, its seed's origin, its start receipt and its
    /// result owner.
    fn conclude(
        &self,
        outcome: Outcome,
        receipt: StartReceipt,
        normalization: pse_ids::ContentHash,
        attempt: usize,
        owner: &Arc<pse_columnar::AllocationLease>,
    ) -> Result<Outcome, MathRuntimeError> {
        Ok(match outcome {
            Outcome::Native(mut r) => {
                if r.failure_bytes() > 0 {
                    let failure_owner = self.reserve("math:solve-failure", r.failure_bytes())?;
                    *r = (*r).with_failure_owner(failure_owner);
                }
                if let Some(seed) = &mut r.warm_start {
                    seed.origin = Some(SeedOrigin { run: None, attempt });
                }
                // The recorded path is what this step's transport, library presolve and
                // interior-point restart actually applied, never a constant label (F25).
                let mut receipt = receipt;
                receipt.record(&r, normalization);
                r.start_receipt = Some(receipt);
                Outcome::Native(Box::new((*r).with_owner(owner.clone())))
            }
            Outcome::Constant(mut r) => {
                r.owner = Some(owner.clone());
                Outcome::Constant(r)
            }
            other => other,
        })
    }
    /// The sum-of-squares bound on a prepared polynomial program (Plan 22 N5; I5): its
    /// factorable projection, expanded into monomials and bounded by the moment relaxation
    /// on one admitted worker within the step's time limit. The bound is labelled
    /// `sos_bound_nonrigorous` (never a certified global bound).
    ///
    /// # Errors
    /// A step that is not an algebraic case, a program that is not polynomial or exceeds
    /// the relaxation's bounds, admission, or a build without POUNCE-convex.
    pub(crate) async fn sos_bound(
        self: &Arc<Self>,
        step: &PreparedSolve,
        order: Option<usize>,
    ) -> Result<execution::sos::SosBound, MathRuntimeError> {
        let Representation::Algebraic(case) = &step.representation else {
            return Err(
                ProblemError::Contract("an SOS bound needs an algebraic case".into()).into(),
            );
        };
        let plan = case.prepared.prepared.plan.clone();
        let values = case.values.clone();
        let limit = self.policy.worker_bytes / 256;
        let tolerance = step.accuracy.stationarity.max(1e-9);
        let time_limit = step.profile.controls.time_limit;
        self.job(
            1,
            self.policy.worker_bytes,
            FlightCancellation::default(),
            move |flag| {
                let program = plan
                    .factorable_program(
                        &values,
                        &pse_math::factorable::FactorableRequest::default(),
                        limit,
                        &flag,
                    )
                    .map_err(|e| match e {
                        pse_math::factorable::FactorableError::Math(e) => ProblemError::Math(e),
                        other => ProblemError::Unsupported(other.to_string()),
                    })?;
                let problem = execution::sos::polynomial(&program)?;
                Ok(execution::sos::bound(
                    &problem, order, tolerance, time_limit,
                )?)
            },
        )
        .await
    }
    /// The coefficient programs of one batch on their shared batching adapter: each member's
    /// case, projection and original model, solved together by the runner and concluded one
    /// by one.
    fn coefficient_batch(
        &self,
        admitted: Vec<(usize, usize, Arc<pse_columnar::AllocationLease>, Admitted)>,
        retained: &mut Retained,
        budget: &Arc<WorkerBudget>,
    ) -> Vec<(usize, Result<Outcome, MathRuntimeError>)> {
        /// One member's owned parts, which the runner's views borrow.
        struct Part {
            index: usize,
            attempt: usize,
            owner: Arc<pse_columnar::AllocationLease>,
            receipt: StartReceipt,
            key: pse_ids::ContentHash,
            chosen: Option<WarmStart>,
            execution: Execution,
            profile: SolverProfile,
            normalization: Normalization,
            tolerances: Tolerances,
            accuracy: ResolvedAccuracy,
            compatibility: Compatibility,
            backend: Backend,
            case: AlgebraicCase,
            problem: native::CoefficientProblem,
        }
        let mut concluded = Vec::new();
        let mut parts = Vec::new();
        for (index, attempt, owner, a) in admitted {
            let Admitted {
                step,
                chosen,
                execution,
                receipt,
                normalization: key,
            } = a;
            let PreparedSolve {
                representation,
                profile,
                normalization,
                tolerances,
                accuracy,
                route,
                compatibility,
                ..
            } = step;
            let part = (|| -> Result<Part, MathRuntimeError> {
                let (Representation::Algebraic(case), Route::Native(backend)) =
                    (representation, route)
                else {
                    return Err(ProblemError::Internal(
                        "a batch member is a native coefficient step".into(),
                    )
                    .into());
                };
                let coefficients = case
                    .prepared
                    .prepared
                    .coefficients
                    .as_ref()
                    .ok_or_else(|| ProblemError::Internal("missing coefficient product".into()))?;
                let problem = native::CoefficientProblem::from_plan(
                    &case.prepared.prepared.plan,
                    coefficients.as_ref().clone(),
                )?;
                let compatibility = compatibility.ok_or_else(|| {
                    ProblemError::Internal("missing native compatibility stamp".into())
                })?;
                Ok(Part {
                    index,
                    attempt,
                    owner: owner.clone(),
                    receipt: receipt.clone(),
                    key,
                    chosen: chosen.clone(),
                    execution: execution.clone(),
                    profile,
                    normalization,
                    tolerances,
                    accuracy,
                    compatibility,
                    backend,
                    case,
                    problem,
                })
            })();
            match part {
                Ok(part) => parts.push(part),
                Err(error) => {
                    let outcome = Outcome::Rejected(Arc::new(error));
                    concluded.push((index, self.conclude(outcome, receipt, key, attempt, &owner)));
                }
            }
        }
        let mut originals: Vec<OriginalCase<'_>> = parts
            .iter()
            .map(|p| OriginalCase {
                service: self,
                case: p.case.case.clone(),
                providers: &p.case.providers,
                values: &p.case.values,
                plan: &p.case.prepared.prepared.plan,
                cancel: p.execution.cancel.clone(),
                budget,
            })
            .collect();
        let mut steps = Vec::with_capacity(parts.len());
        for (p, original) in parts.iter().zip(originals.iter_mut()) {
            let Some(coefficients) = p.case.prepared.prepared.coefficients.as_ref() else {
                continue;
            };
            let plan = &p.case.prepared.prepared.plan;
            steps.push((
                execution::Step {
                    adapter: execution::adapter(p.backend),
                    settings: &p.profile.backend,
                    controls: &p.profile.controls,
                    accuracy: &p.accuracy,
                    execution: p.execution.clone(),
                    tolerances: &p.tolerances,
                    normalization: &p.normalization,
                    compatibility: p.compatibility.clone(),
                    warm: p.chosen.as_ref(),
                },
                execution::Coefficients {
                    problem: &p.problem,
                    certificate: p.case.certificate.as_deref(),
                    row_constants: &coefficients.row_constants,
                    row_bounds: plan
                        .structure()
                        .rows()
                        .iter()
                        .map(|r| (r.lower, r.upper))
                        .collect(),
                    original,
                },
            ));
        }
        let reports = execution::coefficients_batch(retained, steps);
        drop(originals);
        for (p, report) in parts.into_iter().zip(reports) {
            let outcome = match report {
                Ok(report) => Outcome::Native(Box::new(report)),
                Err(error) => Outcome::Rejected(Arc::new(error.into())),
            };
            concluded.push((
                p.index,
                self.conclude(outcome, p.receipt, p.key, p.attempt, &p.owner),
            ));
        }
        concluded
    }
    /// One prepared step: constant evaluation, or the selected adapter's representation
    /// runner. No backend is named; the adapter declares its representation.
    fn run_step(
        &self,
        step: PreparedSolve,
        execution: Execution,
        warm: Option<&WarmStart>,
        retained: &mut Retained,
        budget: &Arc<WorkerBudget>,
    ) -> Result<Outcome, MathRuntimeError> {
        let PreparedSolve {
            representation,
            profile,
            normalization,
            tolerances,
            accuracy,
            route,
            compatibility,
            ..
        } = step;
        // Only a modeling solve prepares the parametric program a sensitivity request
        // differentiates; any other caller is refused rather than silently ignored.
        if profile.sensitivity.is_some()
            && !matches!(&representation, Representation::Algebraic(a) if a.sensitivity.is_some())
        {
            return Err(ProblemError::Contract(
                "a sensitivity request needs the parametric program a modeling solve prepares"
                    .into(),
            )
            .into());
        }
        let Route::Native(backend) = route else {
            return self.constant(representation, &tolerances, &execution.cancel, budget);
        };
        let adapter = execution::adapter(backend);
        let stamp = compatibility
            .ok_or_else(|| ProblemError::Internal("missing native compatibility stamp".into()))?;
        let run = execution::Step {
            adapter,
            settings: &profile.backend,
            controls: &profile.controls,
            accuracy: &accuracy,
            execution,
            tolerances: &tolerances,
            normalization: &normalization,
            compatibility: stamp,
            warm,
        };
        let authored = match &representation {
            Representation::Algebraic(a) => a.prepared.prepared.facts.requirements.clone(),
            Representation::Conic { .. } => Vec::new(),
        };
        let mut report = match (representation, adapter.representation()) {
            (
                Representation::Conic {
                    problem,
                    original,
                    certificate,
                },
                execution::Representation::Cone,
            ) => execution::cone(run, retained, &problem, &original, certificate.as_ref())?,
            // A cone adapter serves a recognized convex program in its cone form, and a
            // coefficient model through the coefficient runner's lowering.
            (Representation::Algebraic(case), execution::Representation::Cone)
                if case.recognized.is_some() =>
            {
                self.recognized_step(run, retained, case, budget)?
            }
            (
                Representation::Algebraic(case),
                execution::Representation::Coefficients | execution::Representation::Cone,
            ) => self.coefficient_step(run, retained, case, budget)?,
            (Representation::Algebraic(case), execution::Representation::Factorable) => {
                self.factorable_step(run, retained, case, &profile, budget)?
            }
            (
                Representation::Algebraic(case),
                kind @ (execution::Representation::Nlp | execution::Representation::Roots),
            ) => self.callback_step(run, retained, case, kind, &profile, budget)?,
            _ => {
                return Err(ProblemError::Internal(
                    "prepared representation differs from the selected adapter".into(),
                )
                .into());
            }
        };
        if authored
            .contains(&pse_model::generated::enums::ModelingStructuralRequirement::L1ExactPenalty)
        {
            report.provenance.insert(
                "method.selection".into(),
                "the authored penalty(l1) realization selects POUNCE's l1 exact penalty-barrier (ADR-0104 §5): the author's selection, not an automatic one".into(),
            );
            report.provenance.insert(
                "method.penalty_scope".into(),
                "the l1 exact penalty relaxes every constraint row of the solve, not only the rows of the penalty(l1) realization".into(),
            );
        }
        Ok(Outcome::Native(Box::new(report)))
    }
    /// Coefficient projection of the compiled case, re-checked against the original case.
    fn coefficient_step(
        &self,
        run: execution::Step<'_>,
        retained: &mut Retained,
        case: AlgebraicCase,
        budget: &Arc<WorkerBudget>,
    ) -> Result<SolveReport, MathRuntimeError> {
        let AlgebraicCase {
            prepared,
            case,
            values,
            providers,
            certificate,
            ..
        } = case;
        let coefficients = prepared
            .prepared
            .coefficients
            .as_ref()
            .ok_or_else(|| ProblemError::Internal("missing coefficient product".into()))?;
        let plan = &prepared.prepared.plan;
        let problem = native::CoefficientProblem::from_plan(plan, coefficients.as_ref().clone())?;
        let mut original = OriginalCase {
            service: self,
            case,
            providers: &providers,
            values: &values,
            plan,
            cancel: run.execution.cancel.clone(),
            budget,
        };
        Ok(execution::coefficients(
            run,
            retained,
            execution::Coefficients {
                problem: &problem,
                certificate: certificate.as_deref(),
                row_constants: &coefficients.row_constants,
                row_bounds: plan
                    .structure()
                    .rows()
                    .iter()
                    .map(|r| (r.lower, r.upper))
                    .collect(),
                original: &mut original,
            },
        )?)
    }
    /// A recognized convex program's cone form on a cone adapter, raised to the program
    /// and re-checked against the original case (ADR-0121 Outcome 6).
    fn recognized_step(
        &self,
        run: execution::Step<'_>,
        retained: &mut Retained,
        case: AlgebraicCase,
        budget: &Arc<WorkerBudget>,
    ) -> Result<SolveReport, MathRuntimeError> {
        let AlgebraicCase {
            prepared,
            case,
            values,
            providers,
            recognized,
            ..
        } = case;
        let (lowered, _owner) = recognized
            .ok_or_else(|| ProblemError::Internal("missing recognized cone form".into()))?;
        let plan = &prepared.prepared.plan;
        let mut original = OriginalCase {
            service: self,
            case,
            providers: &providers,
            values: &values,
            plan,
            cancel: run.execution.cancel.clone(),
            budget,
        };
        Ok(execution::recognized(
            run,
            retained,
            execution::Recognized {
                lowered: &lowered,
                original: &mut original,
            },
        )?)
    }
    /// Factorable export of the compiled case for a global adapter. Candidates are
    /// re-checked against the original case, and a discrete assignment is re-solved as a
    /// continuous problem through the one NLP runner (ADR-0105 §2).
    fn factorable_step(
        &self,
        run: execution::Step<'_>,
        retained: &mut Retained,
        case: AlgebraicCase,
        profile: &SolverProfile,
        budget: &Arc<WorkerBudget>,
    ) -> Result<SolveReport, MathRuntimeError> {
        let AlgebraicCase {
            prepared,
            case,
            values,
            providers,
            factorable,
            sensitivity,
            ..
        } = case;
        let (program, _owner) = factorable
            .ok_or_else(|| ProblemError::Internal("missing factorable program".into()))?;
        let plan = &prepared.prepared.plan;
        if !program.matches(plan, &values) {
            return Err(ProblemError::Contract(
                "factorable program assumptions differ from the case values".into(),
            )
            .into());
        }
        let initial: Vec<_> = plan.columns().iter().map(|id| values.scalars[id]).collect();
        let mut original = OriginalCase {
            service: self,
            case: case.clone(),
            providers: &providers,
            values: &values,
            plan,
            cancel: run.execution.cancel.clone(),
            budget,
        };
        let normalization = run.normalization.clone();
        let cancel = run.execution.cancel.clone();
        // Executable owners and their budget charges outlive every re-solve oracle.
        let mut owners = Vec::new();
        let mut relaxed = || {
            (|| -> Result<native::transform::Relaxed, MathRuntimeError> {
                let ExecutionWorker {
                    worker,
                    _case,
                    _charge,
                } = self.case_worker(case.clone(), providers.clone(), &cancel, budget)?;
                owners.push((_case, _charge));
                Ok(native::assembled::AlgebraicOracle::relaxation(
                    worker,
                    values.clone(),
                    normalization.clone(),
                )?)
            })()
            .map_err(MathRuntimeError::into_problem)
        };
        // The re-solve's parametric callbacks under the same assignment (Plan 22 S1).
        let mut parametric_owners = Vec::new();
        let mut parametric = || {
            (|| -> Result<native::transform::Relaxed, MathRuntimeError> {
                let request = sensitivity.as_ref().ok_or_else(|| {
                    ProblemError::Internal("no sensitivity program to differentiate".into())
                })?;
                let ExecutionWorker {
                    worker,
                    _case,
                    _charge,
                } = self.worker(request.program.clone(), &providers, cancel.clone(), budget)?;
                parametric_owners.push((_case, _charge));
                Ok(native::assembled::AlgebraicOracle::relaxation(
                    worker,
                    values.clone(),
                    request.normalization.clone(),
                )?)
            })()
            .map_err(MathRuntimeError::into_problem)
        };
        let report = execution::factorable(
            run,
            retained,
            execution::Factorable {
                program: &program,
                initial: &initial,
                intent: profile.intent,
                original: &mut original,
                resolve: Some(execution::Resolve {
                    oracle: &mut relaxed,
                    presolve: &profile.presolve,
                    limit: self.policy.worker_bytes / 256,
                    sensitivity: sensitivity.as_ref().map(|request| {
                        execution::ResolveSensitivity {
                            oracle: &mut parametric,
                            parameters: request.parameters.clone(),
                            reduced_hessian: request.reduced_hessian,
                        }
                    }),
                }),
            },
        )?;
        drop(owners);
        drop(parametric_owners);
        Ok(report)
    }
    /// Callback oracle over the compiled case for an NLP or root-system adapter.
    fn callback_step(
        &self,
        run: execution::Step<'_>,
        retained: &mut Retained,
        case: AlgebraicCase,
        kind: execution::Representation,
        profile: &SolverProfile,
        budget: &Arc<WorkerBudget>,
    ) -> Result<SolveReport, MathRuntimeError> {
        let AlgebraicCase {
            prepared,
            case,
            values,
            providers,
            sensitivity,
            ..
        } = case;
        // The parametric callbacks of a sensitivity request (Plan 22 S1), with their
        // evaluator's owner and charge; callbacks that cannot be built withhold the
        // sensitivities and never refuse the solve.
        let parametric = sensitivity.as_ref().map(|program| {
            self.worker(
                program.program.clone(),
                &providers,
                run.execution.cancel.clone(),
                budget,
            )
            .map_err(MathRuntimeError::into_problem)
            .and_then(
                |ExecutionWorker {
                     worker,
                     _case,
                     _charge,
                 }| { Ok((program.request(worker, &values)?, (_case, _charge))) },
            )
        });
        let ExecutionWorker {
            worker,
            _case,
            _charge,
        } = self.case_worker(case, providers, &run.execution.cancel, budget)?;
        let plan = &prepared.prepared.plan;
        let initial: Vec<_> = plan.columns().iter().map(|id| values.scalars[id]).collect();
        let mut oracle = native::assembled::AlgebraicOracle::new(worker, values)?
            .with_structural_analysis(prepared.prepared.structure.clone())
            .with_presolve_facts(prepared.prepared.presolve.clone())?
            .with_normalization(run.normalization.clone())?;
        if let Some(c) = &prepared.prepared.coefficients {
            oracle = oracle.with_coefficient_facts(c)?;
        }
        if kind == execution::Representation::Roots {
            oracle.admit_nle()?;
            // A retained root session keeps the evaluators' owner, and the evaluator's
            // share of the job reservation, alive with it.
            return Ok(execution::roots(
                run,
                retained,
                execution::Roots {
                    oracle: Box::new(oracle),
                    initial: &initial,
                    owner: Some(Box::new((_case, _charge))),
                },
            )?);
        }
        let sense = plan
            .structure()
            .objective()
            .map_or(ObjectiveSense::Minimize, |o| o.sense);
        let (request, unbuilt, parametric_owner) = match (parametric, &sensitivity) {
            (Some(Ok((request, owner))), _) => (Some(request), None, Some(owner)),
            (Some(Err(cause)), Some(program)) => (None, Some(program.withheld(cause)), None),
            _ => (None, None, None),
        };
        let mut report = execution::nlp(
            run,
            retained,
            execution::Nlp {
                oracle: Box::new(oracle),
                initial: &initial,
                presolve: &profile.presolve,
                intent: profile.intent,
                sense,
                limit: self.policy.worker_bytes / 256,
                analysis: execution::Analysis {
                    sensitivity: request,
                    ..execution::Analysis::for_intent(profile.intent)
                },
            },
        )?;
        if unbuilt.is_some() {
            report.evidence.sensitivity = unbuilt;
        }
        // A kept advanced-step factor is charged to the job's allowance before the next
        // step runs; one the allowance cannot hold is released and recorded as not kept.
        if let Some(bytes) = retained.uncharged() {
            match budget.charge(bytes) {
                Ok(charge) => retained.charge(Box::new(charge)),
                Err(_) => {
                    retained.release();
                    if let Some(parametric) = &mut report.evidence.sensitivity {
                        parametric.retained = None;
                    }
                }
            }
        }
        // The case owner and enclosing job reservation outlive every native callback.
        drop(_case);
        drop(_charge);
        drop(parametric_owner);
        Ok(report)
    }
    /// Worker-scoped providers and one attempt-local evaluator on this thread.
    fn case_worker(
        &self,
        case: Option<Arc<ExecutableCase>>,
        providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
        cancel: &Arc<std::sync::atomic::AtomicBool>,
        budget: &Arc<WorkerBudget>,
    ) -> Result<ExecutionWorker, MathRuntimeError> {
        let case =
            case.ok_or_else(|| ProblemError::Internal("missing executable representation".into()))?;
        self.worker(case, &providers, cancel.clone(), budget)
    }
    /// All-fixed original evaluation without a native attempt.
    fn constant(
        &self,
        representation: Representation,
        tolerances: &Tolerances,
        cancel: &Arc<std::sync::atomic::AtomicBool>,
        budget: &Arc<WorkerBudget>,
    ) -> Result<Outcome, MathRuntimeError> {
        let Representation::Algebraic(AlgebraicCase {
            prepared,
            case,
            values,
            providers,
            ..
        }) = representation
        else {
            return Err(
                ProblemError::Internal("constant route needs an algebraic case".into()).into(),
            );
        };
        let ExecutionWorker {
            mut worker,
            _case,
            _charge,
        } = self.case_worker(case, providers, cancel, budget)?;
        let structure = prepared.prepared.plan.structure();
        let objective = structure
            .objective()
            .map(|o| worker.objective(&values).map(|v| v * o.sense.sign()))
            .transpose()?;
        let constraints = worker.constraints(&values)?;
        let rows = structure
            .rows()
            .iter()
            .zip(&constraints)
            .zip(&tolerances.rows)
            .map(|((r, v), t)| Violation {
                id: r.id,
                physical: quality::interval(*v, r.lower, r.upper),
                tolerance: *t,
            })
            .collect();
        let sources = worker.constraint_sources()?;
        let mut observation = quality::Observation::from_values(
            objective,
            constraints,
            structure
                .rows()
                .iter()
                .map(|r| (r.lower, r.upper))
                .collect(),
        )?;
        observation.sources = sources;
        Ok(Outcome::Constant(Box::new(ConstantReport {
            owner: None,
            objective,
            observation,
            quality: Quality::new(rows, vec![], vec![])?,
        })))
    }
}
/// A step admitted for native work: its seed, execution controls and start receipt.
struct Admitted {
    step: PreparedSolve,
    chosen: Option<WarmStart>,
    execution: Execution,
    receipt: StartReceipt,
    normalization: pse_ids::ContentHash,
}
/// One independent step of a batch (Plan 22 N5), with its attempt and result owner.
#[derive(Debug)]
pub(crate) struct BatchMember {
    /// The bound step.
    pub step: PreparedSolve,
    /// Its attempt in the run.
    pub attempt: usize,
    /// The owner of its result.
    pub owner: Arc<pse_columnar::AllocationLease>,
}
/// The original compiled case, evaluated fresh at a coefficient candidate.
struct OriginalCase<'a> {
    service: &'a MathService,
    case: Option<Arc<ExecutableCase>>,
    providers: &'a BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
    values: &'a CaseValues,
    plan: &'a pse_math::assembly::CasePlan,
    cancel: Arc<std::sync::atomic::AtomicBool>,
    budget: &'a Arc<WorkerBudget>,
}
impl execution::OriginalModel for OriginalCase<'_> {
    fn evaluate(&mut self, primal: &[f64]) -> Result<execution::Evaluation, ProblemError> {
        (|| -> Result<_, MathRuntimeError> {
            let case = self.case.clone().ok_or_else(|| {
                ProblemError::Internal("missing original coefficient evaluator".into())
            })?;
            let mut original =
                self.service
                    .worker(case, self.providers, self.cancel.clone(), self.budget)?;
            let mut trial = self.values.clone();
            for (id, value) in self.plan.columns().iter().zip(primal) {
                trial.scalars.insert(*id, *value);
            }
            let constraints = original.worker.constraints(&trial)?;
            let objective = self
                .plan
                .structure()
                .objective()
                .map(|o| {
                    original
                        .worker
                        .objective(&trial)
                        .map(|v| v * o.sense.sign())
                })
                .transpose()?;
            let sources = original.worker.constraint_sources()?;
            Ok(execution::Evaluation {
                constraints,
                objective,
                sources,
            })
        })()
        .map_err(MathRuntimeError::into_problem)
    }
}

/// Complete effective request identity, distinct from native session compatibility: the
/// session profile, every control through serde (F09), the selection, whose backend is
/// named by its registry spelling, and the linked native build (library versions, image
/// manifest and numerical contract, ADR-0108 item 14).
pub(crate) fn profile_key(p: &SolverProfile) -> Result<pse_ids::ContentHash, ProblemError> {
    let mut h = FramedHasher::new(pse_ids::Frame::SolverProfileV3);
    hash_session(&mut h, p)?;
    h.hash(&p.controls.identity()?)
        .hash(&execution::LINKED.build_identity());
    match p.selection {
        SolverSelection::Auto => {
            h.str("auto");
        }
        SolverSelection::Explicit(b) => {
            h.str(b.as_str());
        }
    }
    // A sensitivity request changes what the step computes and publishes, not how it
    // solves; a profile without one keeps its identity.
    if let Some(request) = &p.sensitivity {
        h.str("sensitivity")
            .bool(request.reduced_hessian)
            .u64(request.parameters.len() as u64);
        for parameter in &request.parameters {
            h.id(parameter);
        }
        // A propagation is identified by its complete serde encoding (F09); a request
        // without one keeps its identity.
        if let Some(propagation) = &request.propagation {
            let encoded = serde_json::to_string(propagation)
                .map_err(|e| ProblemError::Internal(format!("propagation encoding: {e}")))?;
            h.str("propagation").str(&encoded);
        }
    }
    Ok(h.finish_hash())
}
