// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One completion-owned solve lifecycle; finite batches never create persistent native sessions.
use super::{ExecutableCase, ExecutionWorker, MathRuntimeError, MathService, Preparation};
use pse_backend_native::{
    self as native, ProblemError,
    execution::{self, BackendExecution, BackendSettings, Retained},
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
}
use std::{collections::BTreeMap, sync::Arc};

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
/// A compiled case with its selected values, providers and convexity evidence.
#[derive(Clone, Debug)]
struct AlgebraicCase {
    prepared: Preparation,
    case: Option<Arc<ExecutableCase>>,
    values: CaseValues,
    providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
    certificate: Option<Arc<dyn QuadraticEvidence>>,
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
    route: Route,
    compatibility: Option<Compatibility>,
    explicit_start: Option<WarmStart>,
    eligibility: Vec<routing::Eligibility>,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl PreparedSolve {
    /// Immutable compilation and normalization selected before attaching a seed.
    pub fn preparation_identity(&self) -> Result<pse_ids::ContentHash, ProblemError> {
        let mut h = FramedHasher::new("pse.solve.preparation.v1");
        h.hash(&profile_key(&self.profile)?)
            .hash(&self.numerics.key);
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
        if let Some(c) = &self.compatibility {
            h.hash(&c.layout).hash(&c.data).str(c.backend.as_str());
        }
        Ok(h.finish_hash())
    }
    /// Complete selected request, including explicit seed payload and compatibility data.
    pub fn request_identity(&self) -> Result<pse_ids::ContentHash, ProblemError> {
        let mut h = FramedHasher::new("pse.solve.request.v1");
        h.hash(&self.preparation_identity()?)
            .hash(&self.numerics.key);
        if let Some(compatibility) = &self.compatibility {
            h.bool(true)
                .hash(&compatibility.layout)
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
    Constant(Box<ConstantReport>),
    /// A typed admission/execution failure before a native report became available.
    Rejected(Arc<MathRuntimeError>),
}
impl Outcome {
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
impl SequenceReport {
    /// Transfer the native result allocation to the workflow's retained assessment.
    pub(crate) fn into_parts(self) -> (Vec<Outcome>, usize, Arc<pse_columnar::AllocationLease>) {
        (self.outcomes, self.unattempted, self._owner)
    }
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
/// A prepared, bounded original-contract check executed between native attempts.
/// Implementations consume the current worker's admission and cannot schedule nested work.
pub(crate) trait SequenceAssessment: Send + std::fmt::Debug {
    fn accepted(
        &mut self,
        attempt: usize,
        outcome: &Outcome,
        cancel: &Arc<std::sync::atomic::AtomicBool>,
    ) -> bool;
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
fn hash_controls(h: &mut FramedHasher, p: &SolverProfile) -> Result<(), ProblemError> {
    h.hash(&p.presolve.key()).hash(&p.numerics.key());
    match p.convexity {
        ConvexityPolicy::Exact => {
            h.u64(0);
        }
        ConvexityPolicy::Numerical { absolute, relative } => {
            h.u64(1).u64(absolute.to_bits()).u64(relative.to_bits());
        }
    }
    h.hash(&p.controls.accuracy.key());
    h.str(p.intent.as_str())
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
    // Backend settings identity derives from serde, never from a hand-written list.
    h.hash(&p.backend.identity()?);
    Ok(())
}
fn compatibility(
    plan: &pse_math::assembly::CasePlan,
    values: &CaseValues,
    p: &SolverProfile,
    backend: Backend,
    providers: &BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
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
    for provider in providers.values() {
        data.hash(&provider.configuration_key());
    }
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
        mut profile: SolverProfile,
        mut certificate: Option<Arc<dyn QuadraticEvidence>>,
        numerical: NumericalInputs,
    ) -> Result<PreparedSolve, MathRuntimeError> {
        profile.controls.validate()?;
        // Per-solve overlays are separate from the shared compiler product.
        let structure = prepared.prepared.plan.structure();
        let entries = structure
            .variables()
            .len()
            .checked_add(structure.rows().len())
            .and_then(|n| n.checked_add(numerical.targets.len()))
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
            return Err(ProblemError::Contract("fixed/parameter values differ from compiler assumptions; prepare the selected revision again".into()).into());
        }
        let f = &prepared.prepared.facts;
        if profile.controls.accuracy != Accuracy::default() {
            return Err(ProblemError::Contract(
                "analysis accuracy is owned by the ID-keyed numerical policy".into(),
            )
            .into());
        }
        let plan = &prepared.prepared.plan;
        let mut targets = plan.numerical_targets(&prepared.prepared.quantities)?;
        targets.extend(numerical.targets);
        let numerics = Arc::new(pse_math::numerics::resolve(
            &prepared.prepared.quantities,
            &targets,
            &numerical.declarations,
            &profile.numerics,
        )?);
        let rows: Vec<_> = plan.structure().rows().iter().map(|r| r.id).collect();
        let normalization = Normalization::from_policy(&numerics, plan.columns(), &rows)?;
        let tolerances = Tolerances::from_policy(&numerics, plan.columns(), &rows)?;
        profile.controls.accuracy =
            Accuracy::resolve(&numerics.policy, &tolerances, &normalization)?;

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
        if let Some(adapter) = adapter {
            adapter.admit_contract(
                &native::assembled::contract(plan),
                &prepared.prepared.presolve.signs,
                &profile.backend,
                execution::Budgets {
                    tolerances: &tolerances,
                    normalization: &normalization,
                    feasibility: profile.controls.accuracy.feasibility,
                },
            )?;
        }
        let mut stamp = match route {
            Route::Constant => None,
            Route::Native(backend) => Some(compatibility(
                &prepared.prepared.plan,
                &values,
                &profile,
                backend,
                &providers,
            )?),
        };
        if let Some(stamp) = &mut stamp {
            let mut h = FramedHasher::new("pse.solver.resolved-layout.v1");
            h.hash(&stamp.layout).hash(&numerics.key);
            stamp.layout = h.finish_hash();
        }
        let case = Some(self.assemble(prepared.clone()).await?);
        Ok(PreparedSolve {
            representation: Representation::Algebraic(AlgebraicCase {
                prepared,
                case,
                values,
                providers,
                certificate,
            }),
            profile,
            numerics,
            normalization,
            tolerances,
            route,
            compatibility: stamp,
            explicit_start: None,
            eligibility,
            _owner: owner,
        })
    }
    /// Explicit conic representation enters the same bounded worker/report lifecycle.
    pub async fn prepare_conic(
        self: &Arc<Self>,
        problem: Arc<native::ConicProblem>,
        certificate: Arc<dyn QuadraticEvidence>,
        mut profile: SolverProfile,
        numerics: Arc<ResolvedNumericalPolicy>,
    ) -> Result<PreparedSolve, MathRuntimeError> {
        profile.controls.validate()?;
        if profile.controls.accuracy != Accuracy::default()
            || profile.numerics.key() != numerics.policy.key()
        {
            return Err(ProblemError::Contract(
                "conic numerical policy differs from its resolved authority".into(),
            )
            .into());
        }
        let sparse_bytes = [&problem.quadratic, &problem.constraints]
            .iter()
            .try_fold(0usize, |n, a| {
                n.checked_add(
                    a.colptr
                        .capacity()
                        .checked_add(a.rowval.capacity())?
                        .checked_mul(size_of::<usize>())?,
                )?
                .checked_add(a.nzval.capacity().checked_mul(size_of::<f64>())?)
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
                        field: "coordinate_scale",
                        selected: true,
                        value: *scale,
                        description: "common positive scale preserving the declared cone geometry"
                            .into(),
                    });
            }
        }
        let mut identity = FramedHasher::new("pse.numerical.cone.v1");
        identity.hash(&resolved.key).hash(&normalization.key());
        resolved.key = identity.finish_hash();
        let numerics = Arc::new(resolved);
        let tolerances = Tolerances::from_policy(&numerics, &ids, &problem.contract.rows)?;
        profile.controls.accuracy =
            Accuracy::resolve(&numerics.policy, &tolerances, &normalization)?;
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
        let mut h = FramedHasher::new("pse.solver.conic-layout.v2");
        h.hash(&numerics.key).hash(&normalization.key());
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
    /// Start one finite batch. All CPU permits are acquired once; nested native work
    /// inherits this admission. Only immutable artifact compilation is shared.
    pub fn solve(
        self: &Arc<Self>,
        sequence: SolveSequence,
    ) -> Result<SolveHandle, MathRuntimeError> {
        self.solve_assessed(sequence, None)
    }
    /// Keep allocation and seed decisions in the sequence owner, after an optional original-contract check.
    pub(crate) fn solve_assessed(
        self: &Arc<Self>,
        sequence: SolveSequence,
        assessment: Option<Box<dyn SequenceAssessment>>,
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
            if s.profile.controls.start == StartPolicy::Explicit && s.explicit_start.is_none() {
                return Err(ProblemError::Contract(
                    "explicit start policy requires a seed before submission".into(),
                )
                .into());
            }
            let (n, m) = match &s.representation {
                Representation::Algebraic(a) => (
                    a.prepared.prepared.facts.variables,
                    a.prepared.prepared.facts.rows,
                ),
                Representation::Conic { problem, .. } => (
                    problem.contract.variables.len(),
                    problem.contract.rows.len(),
                ),
            };
            let sources = match &s.representation {
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
        let bytes = self.policy.worker_bytes;
        tokio::spawn(async move {
            let runner = service.clone();
            let result = service
                .job(cores, bytes, control, move |flag| {
                    runner.run_sequence(sequence, flag, events, result_owner, assessment)
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
        assessment: Option<Box<dyn SequenceAssessment>>,
    ) -> Result<SequenceReport, MathRuntimeError> {
        // Every adapter the sequence executes provides its scope (for example an admitted
        // local pool) around the whole sequence, so retained sessions live inside it.
        let adapters: Vec<&dyn BackendExecution> = sequence
            .steps
            .iter()
            .filter_map(|s| match s.route {
                Route::Native(backend) => Some(execution::adapter(backend)),
                Route::Constant => None,
            })
            .collect();
        let threads = sequence
            .steps
            .iter()
            .map(|s| s.profile.controls.threads)
            .max()
            .unwrap_or(1);
        execution::scoped(&adapters, threads, self.policy.stack_bytes, move || {
            self.run_sequence_inner(sequence, flag, progress, owner, assessment)
        })
    }
    fn run_sequence_inner(
        self: &Arc<Self>,
        sequence: SolveSequence,
        flag: Arc<std::sync::atomic::AtomicBool>,
        progress: Arc<Progress>,
        owner: Arc<pse_columnar::AllocationLease>,
        mut assessment: Option<Box<dyn SequenceAssessment>>,
    ) -> Result<SequenceReport, MathRuntimeError> {
        let total = sequence.steps.len();
        let mut outcomes = Vec::new();
        let mut warm: Option<WarmStart> = None;
        let mut retained = Retained::default();
        for (attempt, step) in sequence.steps.into_iter().enumerate() {
            if flag.load(std::sync::atomic::Ordering::Acquire) {
                break;
            }
            let controls = &step.profile.controls;
            let mut execution = Execution::new(flag.clone(), controls);
            execution.progress = progress.clone();
            if controls.reuse == ReusePolicy::Fresh {
                retained.clear();
            }
            let chosen = match controls.start {
                StartPolicy::NoPriorStart => None,
                StartPolicy::Explicit => Some(step.explicit_start.clone().ok_or_else(|| {
                    ProblemError::Contract("explicit start policy requires a seed".into())
                })?),
                StartPolicy::PreviousAccepted => warm.clone(),
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
                    outcomes.push(Outcome::Rejected(Arc::new(error.into())));
                    warm = None;
                    retained.clear();
                    if sequence.continue_independent {
                        continue;
                    }
                    break;
                }
            }
            let receipt = StartReceipt {
                previous_attempt: (controls.start == StartPolicy::PreviousAccepted
                    && chosen.is_some())
                .then(|| attempt.saturating_sub(1)),
                seed: chosen.clone(),
                sparse_seed: step.profile.backend.partial_start().cloned(),
                transformations: vec![
                    "original -> model normalization -> admitted native presolve".into(),
                ],
                submitted: chosen.is_some(),
            };
            let outcome = self
                .run_step(step, execution, chosen.as_ref(), &mut retained)
                .unwrap_or_else(|e| Outcome::Rejected(Arc::new(e)));
            let outcome = match outcome {
                Outcome::Native(mut r) => {
                    if r.failure_bytes() > 0 {
                        let failure_owner =
                            self.reserve("math:solve-failure", r.failure_bytes())?;
                        r = Box::new((*r).with_failure_owner(failure_owner));
                    }
                    if let Some(seed) = &mut r.warm_start {
                        seed.origin = Some(SeedOrigin { run: None, attempt });
                    }
                    let mut receipt = receipt;
                    receipt.submitted = r.evidence.start_submitted;
                    r.start_receipt = Some(receipt);
                    Outcome::Native(Box::new((*r).with_owner(owner.clone())))
                }
                Outcome::Constant(mut r) => {
                    r.owner = Some(owner.clone());
                    Outcome::Constant(r)
                }
                other => other,
            };
            let original_accepted = assessment
                .as_mut()
                .is_none_or(|a| a.accepted(attempt, &outcome, &flag));
            // `PreviousAccepted` seeds only from a result; a seed-only candidate is
            // offered to explicit consumers such as a study's dependent point.
            let successful = outcome.candidate_use().permits_use() && original_accepted;
            warm = match &outcome {
                Outcome::Native(r) if successful => r.warm_start.clone(),
                Outcome::Native(_) | Outcome::Constant(_) | Outcome::Rejected(_) => None,
            };
            // Terminal native failures cannot poison the next independent step.
            if !successful {
                retained.clear();
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
    /// One prepared step: constant evaluation, or the selected adapter's representation
    /// runner. No backend is named; the adapter declares its representation.
    fn run_step(
        &self,
        step: PreparedSolve,
        execution: Execution,
        warm: Option<&WarmStart>,
        retained: &mut Retained,
    ) -> Result<Outcome, MathRuntimeError> {
        let PreparedSolve {
            representation,
            profile,
            normalization,
            tolerances,
            route,
            compatibility,
            ..
        } = step;
        let Route::Native(backend) = route else {
            return self.constant(representation, &tolerances, &execution.cancel);
        };
        let adapter = execution::adapter(backend);
        let stamp = compatibility
            .ok_or_else(|| ProblemError::Internal("missing native compatibility stamp".into()))?;
        let run = execution::Step {
            adapter,
            settings: &profile.backend,
            controls: &profile.controls,
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
                self.coefficient_step(run, retained, case)?
            }
            (
                Representation::Algebraic(case),
                kind @ (execution::Representation::Nlp | execution::Representation::Roots),
            ) => self.callback_step(run, retained, case, kind, &profile)?,
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
    ) -> Result<SolveReport, MathRuntimeError> {
        let AlgebraicCase {
            prepared,
            case,
            values,
            providers,
            certificate,
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
    /// Callback oracle over the compiled case for an NLP or root-system adapter.
    fn callback_step(
        &self,
        run: execution::Step<'_>,
        retained: &mut Retained,
        case: AlgebraicCase,
        kind: execution::Representation,
        profile: &SolverProfile,
    ) -> Result<SolveReport, MathRuntimeError> {
        let AlgebraicCase {
            prepared,
            case,
            values,
            providers,
            ..
        } = case;
        let ExecutionWorker { worker, _case } =
            self.case_worker(case, providers, &run.execution.cancel)?;
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
            // A retained root session keeps the evaluators' owner alive with it.
            return Ok(execution::roots(
                run,
                retained,
                execution::Roots {
                    oracle: Box::new(oracle),
                    initial: &initial,
                    owner: Some(Box::new(_case)),
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
        Ok(report)
    }
    /// Worker-scoped providers and one attempt-local evaluator on this thread.
    fn case_worker(
        &self,
        case: Option<Arc<ExecutableCase>>,
        providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
        cancel: &Arc<std::sync::atomic::AtomicBool>,
    ) -> Result<ExecutionWorker, MathRuntimeError> {
        let case =
            case.ok_or_else(|| ProblemError::Internal("missing executable representation".into()))?;
        let providers = providers
            .into_iter()
            .map(|(key, f)| {
                f.worker_scoped(cancel.clone())
                    .map(|v| (key, v))
                    .map_err(ProblemError::Provider)
            })
            .collect::<Result<_, _>>()?;
        self.worker(case, providers, cancel.clone())
    }
    /// All-fixed original evaluation without a native attempt.
    fn constant(
        &self,
        representation: Representation,
        tolerances: &Tolerances,
        cancel: &Arc<std::sync::atomic::AtomicBool>,
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
        let ExecutionWorker { mut worker, _case } = self.case_worker(case, providers, cancel)?;
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
}
impl execution::OriginalModel for OriginalCase<'_> {
    fn evaluate(&mut self, primal: &[f64]) -> Result<execution::Evaluation, ProblemError> {
        (|| -> Result<_, MathRuntimeError> {
            let case = self.case.clone().ok_or_else(|| {
                ProblemError::Internal("missing original coefficient evaluator".into())
            })?;
            let providers = self
                .providers
                .iter()
                .map(|(key, f)| {
                    f.worker_scoped(self.cancel.clone())
                        .map(|w| (*key, w))
                        .map_err(ProblemError::Provider)
                })
                .collect::<Result<_, _>>()?;
            let mut original = self.service.worker(case, providers, self.cancel.clone())?;
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

/// Complete effective request identity, distinct from native session compatibility.
pub(crate) fn profile_key(p: &SolverProfile) -> Result<pse_ids::ContentHash, ProblemError> {
    let mut h = FramedHasher::new("pse.solver.profile.v1");
    hash_controls(&mut h, p)?;
    h.u64(p.controls.time_limit.as_secs())
        .u64(u64::from(p.controls.time_limit.subsec_nanos()))
        .u64(u64::from(p.controls.iterations))
        .u64(p.controls.accuracy.feasibility.to_bits())
        .u64(p.controls.history as u64)
        .u64(p.controls.reuse as u64)
        .u64(p.controls.start as u64);
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
