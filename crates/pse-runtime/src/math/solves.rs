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
}
#[derive(Clone, Debug)]
enum Representation {
    Algebraic(AlgebraicCase),
    Conic {
        problem: Arc<native::ConicProblem>,
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
        certificate: Option<Arc<dyn QuadraticEvidence>>,
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
            certificate,
            numerics,
            numerical.implicit,
        )
        .await
    }
    /// Admission of a bound case under an already resolved numerical policy. A conditional
    /// initialization block resolves its policy once for the whole case and passes it here.
    /// `implicit` holds the residual definitions a factorable route exports.
    #[expect(
        clippy::too_many_arguments,
        reason = "a bound case binds its view, values, providers, profile, evidence, policy and implicit definitions"
    )]
    pub(crate) async fn prepare_resolved(
        self: &Arc<Self>,
        prepared: Preparation,
        values: CaseValues,
        providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
        profile: SolverProfile,
        mut certificate: Option<Arc<dyn QuadraticEvidence>>,
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
            let supplied = certificate.take();
            let coordinates = normalization.clone();
            let policy = profile.convexity;
            let bytes = self.policy.worker_bytes;
            let evidence = self
                .job(1, bytes, FlightCancellation::default(), move |flag| {
                    let sign = plan.structure().objective().map_or(1.0, |o| o.sense.sign());
                    if let Some(proof) = &supplied {
                        proof.validate(&coefficients.hessian, sign)?;
                        if proof
                            .assumptions()
                            .is_some_and(|key| key != coefficients.assumptions)
                        {
                            return Err(ProblemError::Contract(
                                "quadratic evidence assumptions differ from the selected snapshot"
                                    .into(),
                            )
                            .into());
                        }
                    }
                    // A numerical witness is qualified again against this request's policy and coordinates.
                    let proof: Arc<dyn QuadraticEvidence> = match supplied {
                        Some(proof)
                            if !matches!(
                                proof.assessment(),
                                Some(pse_math::convexity::ConvexityAssessment::NumericalPsd { .. })
                            ) =>
                        {
                            proof
                        }
                        _ => Arc::new(coefficients.convexity(
                            sign,
                            &coordinates.variables,
                            coordinates.objective,
                            policy,
                            pse_math::convexity::ConvexityLimits {
                                bytes,
                                exact_operations: bytes / 16,
                            },
                            &flag,
                        )?),
                    };
                    let eligible = proof.validate(&coefficients.hessian, sign).is_ok();
                    Ok((proof, eligible))
                })
                .await?;
            convex = evidence.1;
            certificate = Some(evidence.0);
        }
        let requirements = routing::Requirements {
            table: &execution::LINKED,
            facts: f,
            intent: profile.intent,
            convex,
            controls: &profile.controls,
        };
        let route = requirements.select(profile.selection)?;
        let eligibility = requirements.eligibility();
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
                    if let Some(envelope) = registration.envelope().map_err(ProblemError::from)? {
                        envelopes.insert(*key, envelope);
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
        let bytes = (problem.contract.variables.len() + problem.contract.rows.len())
            .checked_mul(size_of::<pse_math::numerics::TargetSpec>() + 8 * size_of::<f64>())
            .and_then(|n| n.checked_add(sparse_bytes))
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
        // A cone request routes to a cone adapter: the explicit one, or the preferred linked one.
        let adapter = match profile.selection {
            SolverSelection::Explicit(backend) => Some(execution::adapter(backend)),
            SolverSelection::Auto => execution::LINKED
                .adapters()
                .filter(|a| a.representation() == execution::Representation::Cone && a.linked())
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
        let controls = step.profile.controls.clone();
        if controls.reuse == ReusePolicy::Fresh {
            retained.clear();
        }
        let (chosen, previous_attempt) = match controls.start {
            StartPolicy::NoPriorStart => (None, None),
            StartPolicy::Explicit => match step.explicit_start.clone() {
                Some(seed) => (Some(seed), None),
                None => {
                    return Ok(Outcome::Rejected(Arc::new(
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
                return Ok(Outcome::Rejected(Arc::new(error.into())));
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
        let outcome = self
            .run_step(step, execution, chosen.as_ref(), retained, budget)
            .unwrap_or_else(|e| Outcome::Rejected(Arc::new(e)));
        Ok(match outcome {
            Outcome::Native(mut r) => {
                if r.failure_bytes() > 0 {
                    let failure_owner = self.reserve("math:solve-failure", r.failure_bytes())?;
                    r = Box::new((*r).with_failure_owner(failure_owner));
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
        let report = match (representation, adapter.representation()) {
            (
                Representation::Conic {
                    problem,
                    certificate,
                },
                execution::Representation::Cone,
            ) => execution::cone(run, retained, &problem, certificate.as_ref())?,
            (Representation::Algebraic(case), execution::Representation::Coefficients) => {
                self.coefficient_step(run, retained, case, budget)?
            }
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
        let mut fixed = |assignment: &BTreeMap<usize, f64>| {
            (|| -> Result<Box<dyn native::NlpOracle>, MathRuntimeError> {
                let ExecutionWorker {
                    worker,
                    _case,
                    _charge,
                } = self.case_worker(case.clone(), providers.clone(), &cancel, budget)?;
                owners.push((_case, _charge));
                let assignment = assignment
                    .iter()
                    .map(|(i, v)| (plan.columns()[*i], *v))
                    .collect();
                let oracle = native::assembled::AlgebraicOracle::with_fixed_assignment(
                    worker,
                    values.clone(),
                    &assignment,
                )?
                .with_normalization(normalization.clone())?;
                Ok(Box::new(oracle))
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
                    oracle: &mut fixed,
                    presolve: &profile.presolve,
                    limit: self.policy.worker_bytes / 256,
                }),
            },
        )?;
        drop(owners);
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
            ..
        } = case;
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
        let report = execution::nlp(
            run,
            retained,
            execution::Nlp {
                oracle: Box::new(oracle),
                initial: &initial,
                presolve: &profile.presolve,
                intent: profile.intent,
                sense,
                limit: self.policy.worker_bytes / 256,
            },
        )?;
        // The case owner and enclosing job reservation outlive every native callback.
        drop(_case);
        drop(_charge);
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
    Ok(h.finish_hash())
}
