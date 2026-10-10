// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Pure admission and transition operations, with an injected effect boundary.
//! Routing supplies capability facts; failed numerical trajectories live only in history.
use super::StepRetention;
use pse_backend_native::{LimitKind, ProblemError};
use pse_ids::ContentHash;
use pse_model::diagnostic::DiagnosticProjection;
use pse_model::{
    generated::enums::{
        NumericalAttemptObservation as Observation, NumericalEventKind as EventKind,
    },
    strategy::{
        Mechanism, NumericalStrategy, Phase, Position, StartOrigin, Transition, WorkCharge,
        WorkLimits, WorkObservation,
    },
};
use std::{collections::BTreeSet, sync::Arc};

pub(crate) mod accuracy_refinement;
pub(crate) mod admission;
pub(crate) mod path_control;
pub(crate) mod target;

/// Cheap owner-issued applicability facts beside one immutable operation binding.
#[derive(Clone, Debug)]
pub(crate) struct AutoCandidate {
    /// Exact immutable original/profile/reconstruction binding supplied by its owner.
    pub(crate) identity: ContentHash,
    pub(crate) kind: pse_model::strategy::MechanismKind,
    pub(crate) start: StartOrigin,
    pub(crate) replacement: bool,
    pub(crate) support: BTreeSet<ContentHash>,
    pub(crate) reservation: Option<WorkObservation>,
    pub(crate) prepared: bool,
}

/// One pure next decision. Candidate bindings remain with the effect owner until selected.
#[derive(Clone, Debug)]
pub(crate) enum AutoDecision {
    Prepare { candidate: usize },
    Dispatch { candidate: usize },
    Assess,
    Finish,
    Exhausted { cause: Option<Arc<ProblemError>> },
    Stop { cause: Option<Arc<ProblemError>> },
}
pub(crate) struct AutoObservation {
    pub(crate) awaiting_assessment: bool,
    pub(crate) native: Observation,
    pub(crate) original: Option<OriginalConclusion>,
    pub(crate) permission: Option<pse_model::generated::enums::CandidateUse>,
}
/// Project the actual last dispatched operation, preserving its cause for continuation.
pub(crate) fn automatic_observation(events: &[Event]) -> Option<AutoObservation> {
    events
        .iter()
        .rev()
        .filter(|event| {
            !(event.kind == EventKind::Refused && event.transition == Some(Transition::Continue))
        })
        .find_map(|event| {
            event.observation.map(|native| AutoObservation {
                awaiting_assessment: false,
                native,
                original: event.original.clone().or_else(|| {
                    event
                        .cause
                        .clone()
                        .map(|cause| OriginalConclusion::Unavailable { cause })
                }),
                permission: event.permission,
            })
        })
}
pub(crate) fn next_automatic(
    request: &pse_model::strategy::CompositionRequest,
    start: &pse_model::strategy::StartRules,
    candidates: &[AutoCandidate],
    attempted: &BTreeSet<usize>,
    last: Option<&AutoObservation>,
    work: WorkObservation,
    inherited: bool,
) -> AutoDecision {
    if let Err(error) = request.validate() {
        return AutoDecision::Stop {
            cause: Some(Arc::new(ProblemError::Contract(error.to_string()))),
        };
    }
    if let Some(last) = last {
        if last.awaiting_assessment {
            return AutoDecision::Assess;
        }
        if last
            .original
            .as_ref()
            .is_some_and(OriginalConclusion::satisfied)
        {
            // Output-goal permission is composed independently. An unresolved
            // output cannot trigger an unrelated original-solver tournament.
            return AutoDecision::Finish;
        }
        if !permits_numerical_continuation(last.native)
            || last
                .original
                .as_ref()
                .and_then(OriginalConclusion::cause)
                .as_deref()
                .is_some_and(|cause| !permits_numerical_continuation(failure(cause)))
        {
            return AutoDecision::Stop {
                cause: last.original.as_ref().and_then(OriginalConclusion::cause),
            };
        }
    }
    let limits = request.limits.unwrap_or(WorkLimits {
        attempts: candidates
            .iter()
            .try_fold(0u64, |sum, candidate| {
                sum.checked_add(
                    if candidate.kind == pse_model::strategy::MechanismKind::Direct {
                        1
                    } else {
                        2
                    },
                )
            })
            .unwrap_or(u64::MAX)
            .max(1),
        evaluations: None,
        iterations: None,
        factorizations: None,
        proof_steps: None,
    });
    for (index, candidate) in candidates.iter().enumerate() {
        if attempted.contains(&index) {
            continue;
        }
        if candidate.replacement {
            if !request.recovery.contains(&candidate.start)
                || !start.permits_recovery(candidate.start, inherited)
            {
                continue;
            }
        } else if !start.permits_entry(candidate.start, inherited) {
            continue;
        }
        if work.attempts >= limits.attempts {
            return AutoDecision::Stop {
                cause: Some(Arc::new(limit("automatic task execution allowance"))),
            };
        }
        return if candidate.prepared {
            AutoDecision::Dispatch { candidate: index }
        } else {
            AutoDecision::Prepare { candidate: index }
        };
    }
    if let Some(last) = last {
        AutoDecision::Exhausted {
            cause: last.original.as_ref().and_then(OriginalConclusion::cause),
        }
    } else {
        AutoDecision::Stop { cause: None }
    }
}

pub(crate) fn automatic_decision_key(
    request: &pse_model::strategy::CompositionRequest,
    candidates: &[AutoCandidate],
    decision: &AutoDecision,
    last: Option<&AutoObservation>,
    work: WorkObservation,
) -> Result<ContentHash, ProblemError> {
    let mut h = pse_ids::FramedHasher::new(pse_ids::Frame::NumericalDecisionV2);
    h.part(
        &serde_json::to_vec(request).map_err(|error| ProblemError::Contract(error.to_string()))?,
    );
    for candidate in candidates {
        h.hash(&candidate.identity)
            .str(candidate.kind.as_str())
            .str(candidate.start.as_str())
            .bool(candidate.replacement);
        h.bool(candidate.prepared);
        for key in &candidate.support {
            h.hash(key);
        }
        h.part(
            &serde_json::to_vec(&candidate.reservation)
                .map_err(|error| ProblemError::Contract(error.to_string()))?,
        );
    }
    match decision {
        AutoDecision::Prepare { candidate } => {
            h.str("prepare").u64(*candidate as u64);
        }
        AutoDecision::Dispatch { candidate } => {
            h.str("dispatch").u64(*candidate as u64);
        }
        AutoDecision::Finish => {
            h.str("finish");
        }
        AutoDecision::Assess => {
            h.str("assess-original");
        }
        AutoDecision::Exhausted { cause } => {
            h.str("exhausted-owned-catalog");
            if let Some(cause) = cause {
                h.str(&cause.to_string());
            }
        }
        AutoDecision::Stop { cause } => {
            h.str("stop");
            if let Some(cause) = cause {
                h.str(&cause.to_string());
            }
        }
    }
    if let Some(last) = last {
        h.bool(last.awaiting_assessment);
        h.str(last.native.as_str());
        if let Some(original) = &last.original {
            h.str(match original {
                OriginalConclusion::Satisfied => "satisfied",
                OriginalConclusion::Refused { .. } => "refused",
                OriginalConclusion::Unavailable { .. } => "unavailable",
            });
        }
        if let Some(permission) = last.permission {
            h.str(permission.as_str());
        }
    }
    h.part(&serde_json::to_vec(&work).map_err(|error| ProblemError::Contract(error.to_string()))?);
    Ok(h.finish_hash())
}

/// One task-owned state of actual producer observations. Consumers and trace use the
/// same entries; structural support is never inserted here as evidence.
#[derive(Default)]
pub(crate) struct ProductState {
    actual: Vec<pse_model::strategy::ProductEvidence>,
    _owner: Option<Arc<dyn pse_math::AllocationOwner>>,
    capacity: Option<usize>,
}
impl ProductState {
    pub(crate) fn extent(capacity: usize) -> Result<usize, ProblemError> {
        size_of::<Self>()
            .checked_add(
                capacity
                    .checked_mul(size_of::<pse_model::strategy::ProductEvidence>())
                    .ok_or_else(|| limit("product state extent"))?,
            )
            .ok_or_else(|| limit("product state extent"))
    }
    pub(crate) fn owned(capacity: usize, owner: Arc<dyn pse_math::AllocationOwner>) -> Self {
        Self {
            actual: Vec::with_capacity(capacity),
            _owner: Some(owner),
            capacity: Some(capacity),
        }
    }
    pub(crate) fn publish(
        &mut self,
        evidence: pse_model::strategy::ProductEvidence,
    ) -> Result<(), ProblemError> {
        if evidence.source.point.is_none()
            || evidence.source.normalization != Some(evidence.accuracy.normalization)
            || evidence.branch.validate().is_err()
        {
            return Err(ProblemError::Contract(
                "produced accuracy lacks exact point/normalization/branch ownership".into(),
            ));
        }
        let demand = pse_model::strategy::AccuracyDemand {
            product: evidence.accuracy.product,
            normalization: evidence.accuracy.normalization,
            allowance: evidence.accuracy.error.unwrap_or(f64::NAN),
            class: evidence.accuracy.class,
        };
        if !evidence.accuracy.satisfies(&demand) {
            return Err(ProblemError::Contract(
                "unestablished produced accuracy cannot enter product state".into(),
            ));
        }
        if let Some(prior) = self.actual.iter_mut().find(|prior| {
            prior.source == evidence.source
                && prior.derivative_order == evidence.derivative_order
                && prior.branch == evidence.branch
        }) {
            let prior_demand = pse_model::strategy::AccuracyDemand {
                product: prior.accuracy.product,
                normalization: prior.accuracy.normalization,
                allowance: prior.accuracy.error.unwrap_or(f64::NAN),
                class: prior.accuracy.class,
            };
            if evidence.accuracy.satisfies(&prior_demand) {
                *prior = evidence;
            }
            return Ok(());
        }
        if self
            .capacity
            .is_some_and(|capacity| self.actual.len() >= capacity)
        {
            return Err(limit("owned producer state capacity"));
        }
        self.actual.push(evidence);
        Ok(())
    }
    pub(crate) fn consume(
        &self,
        contract: &pse_model::strategy::OperationContract,
    ) -> Result<Vec<pse_model::strategy::AccuracyEvidence>, ProblemError> {
        contract.inputs.iter().map(|demand| self.actual.iter().find(|actual|actual.satisfies(demand)).map(|actual|actual.accuracy)
            .ok_or_else(||ProblemError::Unsupported("operation input accuracy is not established at the required source/point/order/branch".into()))).collect()
    }
}

pub(crate) type SharedProducts = Arc<std::sync::Mutex<ProductState>>;
pub(crate) fn products(
    state: &std::sync::Mutex<ProductState>,
) -> Result<std::sync::MutexGuard<'_, ProductState>, ProblemError> {
    state
        .lock()
        .map_err(|_| ProblemError::Internal("numerical product state owner panicked".into()))
}
/// Facts supplied by the prepared mathematical/native owners, never inferred from history.
pub(crate) struct Facts {
    pub(crate) support: BTreeSet<ContentHash>,
    pub(crate) accuracy: Vec<pse_model::strategy::AccuracyEvidence>,
    pub(crate) consumption: Vec<pse_model::strategy::AccuracyDemand>,
    /// Complete inclusive bound, supplied by the operation owner before dispatch.
    pub(crate) reservation: Option<WorkObservation>,
    pub(crate) start: StartOrigin,
    /// Producer-established complete hooks for every requested hard counter.
    pub(crate) work_admitted: bool,
    pub(crate) inherited: bool,
    pub(crate) connected: bool,
    /// Contextual incompatibility of this operation/profile, not a numerical failure.
    pub(crate) refusal: Option<Arc<ProblemError>>,
}
#[derive(Clone, Debug)]
pub(crate) enum Admission {
    Ready,
    OptionalRefusal(Arc<ProblemError>),
    RequiredRefusal(Arc<ProblemError>),
}
/// Resolve a declared operation against producer facts without constructing native state.
pub(crate) fn admit(
    strategy: &NumericalStrategy,
    index: usize,
    entry: bool,
    facts: &Facts,
) -> Admission {
    let mechanism = &strategy.mechanisms[index];
    let refusal = facts.refusal.clone().or_else(|| {
        let start_allowed = if entry {
            strategy.start.permits_entry(facts.start, facts.inherited)
        } else {
            strategy
                .start
                .permits_recovery(facts.start, facts.inherited)
        };
        if mechanism.limits.attempts == 0 {
            Some(Arc::new(limit("mechanism execution allowance")))
        } else if mechanism
            .support
            .iter()
            .any(|key| !facts.support.contains(key))
        {
            Some(Arc::new(ProblemError::Unsupported(
                "required mathematical operation is unavailable".into(),
            )))
        } else if facts.consumption.iter().any(|demand| {
            !facts
                .accuracy
                .iter()
                .any(|evidence| evidence.satisfies(demand))
        }) {
            Some(Arc::new(ProblemError::Unsupported(
                "consumed numerical accuracy is not established for this operation".into(),
            )))
        } else if !start_allowed
            || mechanism.position != Position::Preparation
                && !mechanism.starts.contains(&facts.start)
        {
            Some(Arc::new(ProblemError::Contract(
                "strategy start origin is not permitted".into(),
            )))
        } else if strategy.branch.connected.is_some() && !facts.connected {
            Some(Arc::new(ProblemError::Contract(
                "connected path transport is not established".into(),
            )))
        } else {
            None
        }
    });
    match refusal {
        None => Admission::Ready,
        Some(cause) if mechanism.required => Admission::RequiredRefusal(cause),
        Some(cause) => Admission::OptionalRefusal(cause),
    }
}

/// Scientific conclusion supplied by the original owner, independent of native termination.
#[derive(Clone, Debug)]
pub(crate) enum OriginalConclusion {
    Satisfied,
    Refused { cause: Arc<ProblemError> },
    Unavailable { cause: Arc<ProblemError> },
}
impl OriginalConclusion {
    pub(crate) fn cause(&self) -> Option<Arc<ProblemError>> {
        match self {
            Self::Satisfied => None,
            Self::Refused { cause } | Self::Unavailable { cause } => Some(cause.clone()),
        }
    }
    pub(crate) const fn satisfied(&self) -> bool {
        matches!(self, Self::Satisfied)
    }
}
/// Common scientific transport. Work contains only disjoint independently owned assessment
/// operations; native inclusive work must never be repeated here.
pub(crate) struct Assessed<T> {
    pub(crate) product: T,
    pub(crate) original: OriginalConclusion,
    pub(crate) retention: StepRetention,
    pub(crate) work: Vec<WorkCharge>,
    /// Retained output evidence; the session may grant supported accuracy work only
    /// after independent original checks have passed.
    pub(crate) accuracy: Vec<pse_math::engineering_accuracy::GoalResult>,
    /// Pure retained-product update at a finite stopping boundary. This never
    /// evaluates a model or executes a numerical producer.
    pub(crate) accuracy_stop: Option<AccuracyStop<T>>,
}
pub(crate) type AccuracyStop<T> = fn(
    &mut T,
    &super::solves::Outcome,
    &pse_model::numerics::NumericalPolicy,
    pse_model::generated::enums::AccuracyUnavailableReason,
);
impl<T> Assessed<T> {
    /// Native-only consumer: original validation belongs to the native report owner.
    pub(crate) fn native(
        product: T,
        retention: StepRetention,
        outcome: &super::solves::Outcome,
    ) -> Self {
        let original = match cause(outcome) {
            Some(cause) => OriginalConclusion::Unavailable { cause },
            None if retention.candidate.permits_use() => OriginalConclusion::Satisfied,
            None => OriginalConclusion::Refused {
                cause: Arc::new(ProblemError::numerical(retention.candidate.reason())),
            },
        };
        Self {
            product,
            original,
            retention,
            work: Vec::new(),
            accuracy: Vec::new(),
            accuracy_stop: None,
        }
    }
}
/// Independent assessment; a successful auxiliary solve never grants original permission.
pub(crate) struct Assessment {
    pub(crate) auxiliary: bool,
    pub(crate) retention: StepRetention,
    pub(crate) original: OriginalConclusion,
    pub(crate) work: Vec<WorkCharge>,
    pub(crate) observation: Observation,
    pub(crate) cause: Option<Arc<ProblemError>>,
}
/// Terminal semantics are independent of the declaration's numerical recovery permissions.
pub(crate) fn transition(mechanism: &Mechanism, assessment: &Assessment) -> Transition {
    use Observation as O;
    if matches!(
        assessment.observation,
        O::ContractFailure
            | O::ResourceExhausted
            | O::Cancelled
            | O::Panic
            | O::OperationalFailure
            | O::Infeasible
    ) {
        return Transition::Stop;
    }
    if assessment.retention.candidate.permits_seed()
        && assessment.original.satisfied()
        && !assessment.auxiliary
    {
        return Transition::Finish;
    }
    if !assessment.auxiliary && !assessment.original.satisfied() {
        let original = assessment.original.cause();
        if original
            .as_deref()
            .is_some_and(|cause| !permits_numerical_continuation(failure(cause)))
        {
            return Transition::Stop;
        }
    }
    let preferred = match assessment.observation {
        O::Auxiliary if assessment.auxiliary => Transition::Continue,
        O::NumericalFailure | O::Stalled => Transition::Recover,
        O::Limited => Transition::Continue,
        _ => Transition::Stop,
    };
    if mechanism.transitions.contains(&preferred) {
        preferred
    } else if preferred == Transition::Recover
        && mechanism.transitions.contains(&Transition::Subdivide)
    {
        Transition::Subdivide
    } else {
        Transition::Stop
    }
}

fn limit(detail: &str) -> ProblemError {
    ProblemError::Limit {
        kind: LimitKind::Work,
        detail: detail.into(),
    }
}
/// Actual work is charged once by an operation identity. Reservations are not observations.
pub(crate) struct Ledger {
    limits: WorkLimits,
    charged: BTreeSet<ContentHash>,
    total: WorkObservation,
    reserved: WorkObservation,
    has_reservation: bool,
}
impl Ledger {
    pub(crate) fn new(limits: WorkLimits) -> Self {
        Self {
            limits,
            charged: BTreeSet::new(),
            has_reservation: false,
            reserved: WorkObservation {
                attempts: 0,
                evaluations: Some(0),
                iterations: Some(0),
                factorizations: Some(0),
                proof_steps: Some(0),
            },
            total: WorkObservation {
                attempts: 0,
                evaluations: Some(0),
                iterations: Some(0),
                factorizations: Some(0),
                proof_steps: Some(0),
            },
        }
    }
    pub(crate) fn observation(&self) -> WorkObservation {
        self.total
    }
    pub(crate) fn effective(&self) -> WorkObservation {
        WorkObservation {
            attempts: self.total.attempts,
            evaluations: self.total.evaluations.or(self.reserved.evaluations),
            iterations: self.total.iterations.or(self.reserved.iterations),
            factorizations: self.total.factorizations.or(self.reserved.factorizations),
            proof_steps: self.total.proof_steps.or(self.reserved.proof_steps),
        }
    }
    pub(crate) fn reserve_attempt(&self) -> Result<(), ProblemError> {
        if self.total.attempts >= self.limits.attempts {
            Err(limit("task execution allowance"))
        } else {
            Ok(())
        }
    }
    /// Reserve the complete inclusive bound before any effect. Unknown actual work retains
    /// this full reservation; it never turns an allowance into a measured observation.
    pub(crate) fn reserve(
        &mut self,
        local: WorkLimits,
        bound: Option<WorkObservation>,
    ) -> Result<(), ProblemError> {
        let strict = [
            self.limits.evaluations,
            self.limits.iterations,
            self.limits.factorizations,
            self.limits.proof_steps,
            local.evaluations,
            local.iterations,
            local.factorizations,
            local.proof_steps,
        ]
        .iter()
        .any(Option::is_some);
        if !strict {
            return Ok(());
        }
        let bound = bound.ok_or_else(|| ProblemError::Unsupported("strict work allowance requires a complete inclusive operation bound before dispatch".into()))?;
        check_limits(local, bound)?;
        // Known completed work replaces its conservative reservation. Unknown
        // completed work retains the full bound, including prior known work.
        let prior = self.effective();
        let sum = add_work(prior, bound)?;
        check_limits(self.limits, sum)?;
        self.reserved = sum;
        self.has_reservation = true;
        Ok(())
    }
    pub(crate) fn charge(&mut self, charge: WorkCharge) -> Result<(), ProblemError> {
        if !self.charged.insert(charge.charging_owner) {
            return Err(ProblemError::Internal(
                "inclusive numerical work was charged twice".into(),
            ));
        }
        self.total.attempts = self
            .total
            .attempts
            .checked_add(charge.observed.attempts)
            .ok_or_else(|| limit("work counter overflow"))?;
        fn add(a: Option<u64>, b: Option<u64>) -> Result<Option<u64>, ProblemError> {
            match (a, b) {
                (Some(a), Some(b)) => a
                    .checked_add(b)
                    .map(Some)
                    .ok_or_else(|| limit("work counter overflow")),
                _ => Ok(None),
            }
        }
        self.total.evaluations = add(self.total.evaluations, charge.observed.evaluations)?;
        self.total.iterations = add(self.total.iterations, charge.observed.iterations)?;
        self.total.factorizations = add(self.total.factorizations, charge.observed.factorizations)?;
        self.total.proof_steps = add(self.total.proof_steps, charge.observed.proof_steps)?;
        // A source-backed pre-operation reservation enforces a strict unknown counter;
        // post-operation measurements remain unknown and do not release its allowance.
        if !self.has_reservation {
            return check_limits(self.limits, self.total);
        }
        let effective = WorkObservation {
            attempts: self.total.attempts,
            evaluations: self.total.evaluations.or(self.reserved.evaluations),
            iterations: self.total.iterations.or(self.reserved.iterations),
            factorizations: self.total.factorizations.or(self.reserved.factorizations),
            proof_steps: self.total.proof_steps.or(self.reserved.proof_steps),
        };
        check_limits(self.limits, effective)
    }
}
fn add_work(a: WorkObservation, b: WorkObservation) -> Result<WorkObservation, ProblemError> {
    fn add(a: Option<u64>, b: Option<u64>) -> Result<Option<u64>, ProblemError> {
        match (a, b) {
            (Some(a), Some(b)) => a
                .checked_add(b)
                .map(Some)
                .ok_or_else(|| limit("work counter overflow")),
            _ => Ok(None),
        }
    }
    Ok(WorkObservation {
        attempts: a
            .attempts
            .checked_add(b.attempts)
            .ok_or_else(|| limit("work counter overflow"))?,
        evaluations: add(a.evaluations, b.evaluations)?,
        iterations: add(a.iterations, b.iterations)?,
        factorizations: add(a.factorizations, b.factorizations)?,
        proof_steps: add(a.proof_steps, b.proof_steps)?,
    })
}
fn check_limits(limits: WorkLimits, actual: WorkObservation) -> Result<(), ProblemError> {
    if actual.attempts > limits.attempts {
        return Err(limit("execution allowance"));
    }
    for (cap, count, name) in [
        (
            limits.evaluations,
            actual.evaluations,
            "evaluation allowance",
        ),
        (limits.iterations, actual.iterations, "iteration allowance"),
        (
            limits.factorizations,
            actual.factorizations,
            "factorization allowance",
        ),
        (limits.proof_steps, actual.proof_steps, "proof allowance"),
    ] {
        if let Some(cap) = cap {
            match count {
                Some(count) if count <= cap => {}
                Some(_) => return Err(limit(name)),
                None => {
                    return Err(ProblemError::Unsupported(format!(
                        "{name} requires an observable charging owner"
                    )));
                }
            }
        }
    }
    Ok(())
}

/// Ordered actual event; presentation does not decide the transition.
#[derive(Clone, Debug)]
pub(crate) struct Event {
    pub(crate) mechanism: usize,
    pub(crate) kind: EventKind,
    pub(crate) phase: Phase,
    pub(crate) original: Option<OriginalConclusion>,
    pub(crate) decision: Option<ContentHash>,
    pub(crate) observation: Option<Observation>,
    pub(crate) transition: Option<Transition>,
    pub(crate) permission: Option<pse_model::generated::enums::CandidateUse>,
    pub(crate) work: Option<WorkCharge>,
    pub(crate) cause: Option<Arc<ProblemError>>,
}
pub(crate) fn validate_outputs(
    contract: &pse_model::strategy::OperationContract,
    evidence: &[pse_model::strategy::ProductEvidence],
) -> Result<(), ProblemError> {
    if contract.outputs.iter().any(|demand| {
        !evidence.iter().any(|actual| {
            actual.source.point.is_some()
                && actual.source.normalization == Some(actual.accuracy.normalization)
                && demand.admits(actual)
        })
    }) {
        return Err(ProblemError::Unsupported("operation did not establish every frozen output source/point/order/branch/accuracy obligation".into()));
    }
    Ok(())
}
pub(crate) struct Attempt<T> {
    pub(crate) value: T,
    pub(crate) observation: Observation,
    pub(crate) work: WorkCharge,
    pub(crate) evidence: Vec<pse_model::strategy::ProductEvidence>,
}
/// A dispatched effect that failed still owns its actual work. A local clock is
/// supplied only by the producer that narrowed it; ordinary resource errors never
/// acquire local-abandonment semantics from their diagnostic class.
pub(crate) struct EffectFailure {
    pub(crate) cause: Arc<ProblemError>,
    pub(crate) observed: WorkObservation,
    local_expiry: Option<std::time::Instant>,
    local_refinement: bool,
    /// Actual unsuccessful component observation, supplied only by its native owner.
    observation: Option<Observation>,
}
impl From<Arc<ProblemError>> for EffectFailure {
    fn from(cause: Arc<ProblemError>) -> Self {
        Self {
            cause,
            observed: WorkObservation {
                attempts: 1,
                evaluations: None,
                iterations: None,
                factorizations: None,
                proof_steps: None,
            },
            local_expiry: None,
            local_refinement: false,
            observation: None,
        }
    }
}
impl EffectFailure {
    pub(crate) fn component(
        cause: Arc<ProblemError>,
        observed: WorkObservation,
        observation: Observation,
    ) -> Self {
        Self {
            cause,
            observed,
            local_expiry: None,
            local_refinement: false,
            observation: Some(observation),
        }
    }
    pub(crate) fn observed(cause: Arc<ProblemError>, observed: WorkObservation) -> Self {
        Self {
            cause,
            observed,
            local_expiry: None,
            local_refinement: false,
            observation: None,
        }
    }
    /// Bind actual producer-issued time exhaustion to its local scope. The producer
    /// separately verifies that the cause arose from time, rather than a coincident
    /// pool or callback failure. Cancellation always disqualifies this evidence.
    pub(crate) fn expired_scope(
        cause: Arc<ProblemError>,
        observed: WorkObservation,
        local: &pse_kernels::ExecutionScope,
        enclosing: &pse_kernels::ExecutionScope,
    ) -> Self {
        let local_expiry = local.deadline().filter(|deadline| {
            Arc::ptr_eq(local.cancellation(), enclosing.cancellation())
                && enclosing.deadline().is_some_and(|outer| *deadline < outer)
                && std::time::Instant::now() >= *deadline
                && enclosing.check().is_ok()
        });
        Self {
            cause,
            observed,
            local_expiry,
            local_refinement: false,
            observation: None,
        }
    }
    /// Bind a producer-validated refusal of this operation's exact consumed product.
    /// The producer checks source/validity/product ownership; this boundary refuses
    /// renewed, cancelled or expired scopes and never classifies generic resource errors.
    pub(crate) fn refinement(
        cause: Arc<ProblemError>,
        observed: WorkObservation,
        local: &pse_kernels::ExecutionScope,
        enclosing: &pse_kernels::ExecutionScope,
    ) -> Self {
        let local_refinement = Arc::ptr_eq(local.cancellation(), enclosing.cancellation())
            && local.deadline().is_some_and(|deadline| {
                enclosing.deadline().is_some_and(|outer| deadline <= outer)
            })
            && local.check().is_ok()
            && enclosing.check().is_ok();
        Self {
            cause,
            observed,
            local_expiry: None,
            local_refinement,
            observation: None,
        }
    }
}
pub(crate) struct DriverResult<T> {
    pub(crate) value: Option<T>,
    pub(crate) assessment: Option<Assessment>,
    pub(crate) events: Vec<Event>,
    pub(crate) terminal: Option<Arc<ProblemError>>,
    pub(crate) work: WorkObservation,
}

/// Actual shared-driver trace. Result transports consume the registry rows directly.
#[derive(Debug)]
pub struct Trace {
    pub(crate) owner: Option<Arc<dyn pse_math::AllocationOwner>>,
    /// Automatic observations retain their admitted source request even when the
    /// selected prefix has no execution. This is publication attribution only;
    /// it never grants admission to an empty execution declaration.
    pub(crate) publication_request: Option<pse_ids::roles::LineageRequestHash>,
    pub(crate) declaration: NumericalStrategy,
    pub(crate) original: ContentHash,
    pub(crate) backend: Option<pse_backend_native::solve::Backend>,
    pub(crate) profile: ContentHash,
    pub(crate) start: StartOrigin,
    pub(crate) starts: Vec<StartOrigin>,
    pub(crate) products: Vec<RungProducts>,
    pub(crate) events: Vec<Event>,
}
/// Producer observations for the actual operation, separate from its declaration.
#[derive(Clone, Debug, Default)]
pub(crate) struct RungProducts {
    pub(crate) provider: Option<ProviderEvidence>,
    pub(crate) statistical: Option<Arc<pse_math::surrogate::SurrogateProposal>>,
    pub(crate) derived: Option<ContentHash>,
    pub(crate) start: Option<ContentHash>,
    pub(crate) transport: Option<ContentHash>,
    pub(crate) accuracy: Option<pse_model::strategy::AccuracyEvidence>,
    pub(crate) evidence: Vec<pse_model::strategy::ProductEvidence>,
    pub(crate) path_events:
        Option<pse_math::SharedAllocation<Vec<pse_backend_native::kkt::path::arclength::Event>>>,
}
/// The actual library phase has no native adapter; its retained options/task key is
/// separate from the original corrector's native profile.
#[derive(Clone, Copy, Debug)]
pub(crate) enum ProviderEvidence {
    Native(pse_model::strategy::ProfileRef),
    Library(ContentHash),
    /// An actual original evaluation/reconstruction without a native adapter or profile.
    NonNative,
}
impl Trace {
    pub(crate) fn retained_bytes(&self) -> Result<usize, ProblemError> {
        use pse_model::HeapUsage;
        let initial = size_of::<Self>()
            .checked_add(self.declaration.heap_bytes())
            .and_then(|bytes| {
                bytes.checked_add(self.events.capacity().checked_mul(size_of::<Event>())?)
            })
            .and_then(|bytes| {
                bytes.checked_add(
                    self.starts
                        .capacity()
                        .checked_mul(size_of::<StartOrigin>())?,
                )
            })
            .and_then(|bytes| {
                bytes.checked_add(
                    self.products
                        .capacity()
                        .checked_mul(size_of::<RungProducts>())?,
                )
            })
            .ok_or_else(|| limit("strategy trace extent"))?;
        let initial = self.products.iter().try_fold(initial, |bytes, product| {
            bytes
                .checked_add(
                    product
                        .evidence
                        .capacity()
                        .checked_mul(size_of::<pse_model::strategy::ProductEvidence>())
                        .ok_or_else(|| limit("strategy product evidence extent"))?,
                )
                .ok_or_else(|| limit("strategy product evidence extent"))
        })?;
        self.events.iter().try_fold(initial, |bytes, event| {
            bytes
                .checked_add(
                    event
                        .cause
                        .as_ref()
                        .map_or(0, |cause| cause.retained_bytes()),
                )
                .ok_or_else(|| limit("strategy trace extent"))
        })
    }
    pub(crate) fn with_owner(mut self, owner: Arc<dyn pse_math::AllocationOwner>) -> Self {
        self.owner = Some(owner);
        self
    }
    /// Exact metadata count, without projecting any event.
    pub fn event_count(&self) -> usize {
        self.events.len()
    }
    /// Exact metadata count over original product receipts, with checked global extent.
    pub fn product_count(&self) -> Result<usize, ProblemError> {
        self.products.iter().try_fold(0usize, |total, products| {
            total
                .checked_add(products.evidence.len())
                .ok_or_else(|| limit("strategy product extent"))
        })
    }
    /// Publish the selected actual receipts without validating skipped products.
    pub fn product_rows(
        &self,
        run_id: pse_model::generated::identities::RunId,
        step: usize,
        range: std::ops::Range<usize>,
    ) -> Result<
        impl Iterator<
            Item = Result<
                pse_model::generated::runtime::solve_strategy_products::Row,
                ProblemError,
            >,
        > + '_,
        ProblemError,
    > {
        use pse_model::generated::runtime::solve_strategy_products::Row;
        let ordinal = |value: usize| {
            i64::try_from(value).map_err(|_| limit("strategy product ordinal extent"))
        };
        let count = range
            .end
            .checked_sub(range.start)
            .ok_or_else(|| ProblemError::Contract("reversed strategy product range".into()))?;
        Ok(self
            .products
            .iter()
            .enumerate()
            .flat_map(|(mechanism, products)| {
                products
                    .evidence
                    .iter()
                    .enumerate()
                    .map(move |(product, evidence)| (mechanism, product, evidence))
            })
            .skip(range.start)
            .take(count)
            .map(move |(mechanism, product, evidence)| {
                let source = evidence.source;
                let normalization = source.normalization.ok_or_else(|| {
                    ProblemError::Contract("published product normalization missing".into())
                })?;
                let point = source.point.ok_or_else(|| {
                    ProblemError::Contract("published product exact point missing".into())
                })?;
                evidence
                    .branch
                    .validate()
                    .map_err(|error| ProblemError::Contract(error.to_string()))?;
                let accuracy = evidence.accuracy;
                let demand = pse_model::strategy::AccuracyDemand {
                    product: accuracy.product,
                    normalization,
                    allowance: accuracy.error.unwrap_or(f64::NAN),
                    class: accuracy.class,
                };
                if source.accuracy != Some(accuracy.product) || !accuracy.satisfies(&demand) {
                    return Err(ProblemError::Contract(
                        "published product source/order accuracy is not established".into(),
                    ));
                }
                let connected = evidence.branch.connected;
                Ok(Row {
                    run_id,
                    step: ordinal(step)?,
                    mechanism: ordinal(mechanism)?,
                    product: ordinal(product)?,
                    derivative_order: i64::from(evidence.derivative_order),
                    product_identity: accuracy.product,
                    source_structure: source.structure,
                    source_binding: source.binding,
                    normalization,
                    point,
                    numerical_policy: source.numerical_policy,
                    parameters: source.parameters,
                    derivation: source.derivation,
                    source_branch: source.branch,
                    source_accuracy: source.accuracy,
                    path: connected.map(|path| path.path),
                    sheet: connected.map(|path| path.sheet),
                    transport: connected.map(|path| path.transport),
                    orientation: connected.map(|path| path.orientation),
                    accuracy_class: accuracy.class,
                    error: accuracy.error,
                    branch_policy: evidence.branch.kind,
                })
            }))
    }
    /// Project ordered events without reconstructing numerical decisions from metrics.
    pub fn rows(
        &self,
        run_id: pse_model::generated::identities::RunId,
        step: usize,
        pool: Arc<dyn pse_columnar::MemoryPool>,
        range: std::ops::Range<usize>,
    ) -> Result<
        impl Iterator<
            Item = Result<pse_model::generated::runtime::solve_strategy_events::Row, ProblemError>,
        > + '_,
        ProblemError,
    > {
        use pse_model::generated::runtime::solve_strategy_events::Row;
        let count_rows = range
            .end
            .checked_sub(range.start)
            .ok_or_else(|| ProblemError::Contract("reversed strategy event range".into()))?;
        let strategy_identity = match self.publication_request {
            Some(request) => request.as_id(),
            None => self
                .declaration
                .key()
                .map_err(|e| ProblemError::Contract(e.to_string()))?,
        };
        let ordinal =
            |value: usize| i64::try_from(value).map_err(|_| limit("strategy event ordinal"));
        let count = |value: Option<u64>| {
            value
                .map(|n| i64::try_from(n).map_err(|_| limit("strategy work projection")))
                .transpose()
        };
        // The iterator retains the current DTO's copy admission until the consumer has
        // published it and requests another row (or drops this request).
        let mut working: Option<pse_columnar::MemoryReservation> = None;
        Ok(self.events
            .iter()
            .enumerate()
            .skip(range.start).take(count_rows)
            .map(move |(index, event)| {
                let mechanism = self.declaration.mechanisms.get(event.mechanism)
                    .ok_or_else(|| ProblemError::Contract("published event has no observed mechanism".into()))?;
                let actual = !matches!(event.kind, EventKind::Planned | EventKind::Refused);
                let product = actual.then(|| self.products.get(event.mechanism)).flatten();
                let produced = matches!(event.kind, EventKind::Finished | EventKind::Abandoned)
                    .then_some(product)
                    .flatten();
                let work = event.work;
                drop(working.take());
                let statistical = produced.and_then(|p| p.statistical.as_ref());
                let events = produced.and_then(|p| p.path_events.as_ref());
                let path_scalars = events.map_or(Ok(0usize), |events| events.iter().try_fold(0usize, |total, event| {
                    total.checked_add(event.point.len()).and_then(|n| n.checked_add(event.state_singular_values.len())).and_then(|n| n.checked_add(event.augmented_singular_values.len())).ok_or_else(|| limit("strategy event copy extent"))
                }))?;
                let parts: &[(usize, usize)] = &[
                    (1, 4096), (statistical.map_or(0, |s| s.coordinates.len()), 8), (statistical.map_or(0, |s| s.model_values.len()), 8),
                    (events.map_or(0, |e| e.len()), size_of::<pse_model::generated::runtime::solve_strategy_events::RuntimeSolveStrategyEventsFieldPathEventsItem>()),
                    (path_scalars, 8), (event.cause.as_ref().map_or(0, |cause| cause.retained_bytes()), 4),
                ];
                let bytes = parts.iter().try_fold(0usize, |total, (count, width)| count.checked_mul(*width).and_then(|bytes| total.checked_add(bytes))).ok_or_else(|| limit("strategy event copy extent"))?;
                let copy = pse_columnar::MemoryConsumer::new("result:strategy-event-copy").register(&pool);
                copy.try_grow(bytes).map_err(|e| ProblemError::memory(e.to_string()))?;
                working = Some(copy);
                Ok(Row {
                    run_id,
                    step: ordinal(step)?,
                    event: ordinal(index)?,
                    strategy_identity,
                    mechanism: mechanism.kind,
                    kind: event.kind,
                    phase: event.phase,
                    scope: work.map_or(pse_model::strategy::Scope::Task, |w| w.scope),
                    charging_owner: work.map(|w| w.charging_owner),
                    original_identity: self.original,
                    decision_identity:event.decision,
                    original_conclusion:event.original.as_ref().map(|conclusion|match conclusion {OriginalConclusion::Satisfied=>pse_model::generated::enums::NumericalOriginalConclusion::Satisfied,OriginalConclusion::Refused{..}=>pse_model::generated::enums::NumericalOriginalConclusion::Refused,OriginalConclusion::Unavailable{..}=>pse_model::generated::enums::NumericalOriginalConclusion::Unavailable}),
                    derived_identity: product.and_then(|p| p.derived),
                    profile_identity: actual.then(|| match product.and_then(|p| p.provider) {
                        Some(ProviderEvidence::Library(key)) => Some(key),
                        Some(ProviderEvidence::Native(profile)) => Some(profile.key),
                        Some(ProviderEvidence::NonNative) => None,
                        None => Some(mechanism.profile.map_or(self.profile, |p| p.key)),
                    }).flatten(),
                    backend: if let Some(provider) = product.and_then(|p| p.provider) {
                        match provider {
                            ProviderEvidence::Library(_) | ProviderEvidence::NonNative => None,
                            ProviderEvidence::Native(profile) => Some(profile.backend),
                        }
                    } else if actual {
                        mechanism.profile.map(|p| p.backend).or(self.backend)
                    } else {
                        mechanism.profile.map(|p| p.backend)
                    },
                    start_origin: actual.then_some(
                        self.starts
                            .get(event.mechanism)
                            .copied()
                            .unwrap_or(self.start),
                    ),
                    start_identity: product.and_then(|p| p.start),
                    transport_identity: produced.and_then(|p| p.transport),
                    accuracy_class: produced.and_then(|p| p.accuracy.map(|a| a.class)),
                    accuracy_identity: produced.and_then(|p| p.accuracy.map(|a| a.product)),
                    fidelity_identity: produced
                        .and_then(|p| p.statistical.as_ref().map(|s| s.fidelity)),
                    surrogate_task_identity: produced
                        .and_then(|p| p.statistical.as_ref().map(|s| s.task)),
                    infill_statistic: produced
                        .and_then(|p| p.statistical.as_ref().and_then(|s| s.infill_statistic)),
                    surrogate_radius: produced
                        .and_then(|p| p.statistical.as_ref().map(|s| s.radius)),
                    surrogate_global_iterations: produced
                        .and_then(|p| p.statistical.as_ref().map(|s| s.global_iterations))
                        .map(ordinal)
                        .transpose()?,
                    surrogate_local_iterations: produced
                        .and_then(|p| p.statistical.as_ref().map(|s| s.local_iterations))
                        .map(ordinal)
                        .transpose()?,
                    surrogate_coordinates: produced.and_then(|p| {
                        p.statistical
                            .as_ref()
                            .map(|s| s.coordinates.as_ref().clone())
                    }),
                    surrogate_model_values: produced.and_then(|p| {
                        p.statistical
                            .as_ref()
                            .map(|s| s.model_values.as_ref().clone())
                    }),
                    path_events: produced
                        .and_then(|p| p.path_events.as_ref())
                        .map(|events| events.iter().map(path_event_row).collect())
                        .transpose()?,
                    observation: event.observation,
                    transition: event.transition,
                    permission: event.permission,
                    attempts: count(work.map(|w| w.observed.attempts))?,
                    evaluations: count(work.and_then(|w| w.observed.evaluations))?,
                    iterations: count(work.and_then(|w| w.observed.iterations))?,
                    factorizations: count(work.and_then(|w| w.observed.factorizations))?,
                    proof_steps: count(work.and_then(|w| w.observed.proof_steps))?,
                    failures: event
                        .cause
                        .iter()
                        .map(|cause| {
                            crate::workflow::diagnostic_rows::strategy_failure(
                                &cause
                                    .boundary_diagnostic(pse_diagnostics::DiagnosticStage::Native),
                            )
                        })
                        .collect(),
                    detail: None,
                })
            })
            )
    }
}
/// Copy actual estimated event observations into the registry boundary. Their shared
/// allocation remains owned by the trace until the final trace holder is dropped.
fn path_event_row(event:&pse_backend_native::kkt::path::arclength::Event)->Result<pse_model::generated::runtime::solve_strategy_events::RuntimeSolveStrategyEventsFieldPathEventsItem,ProblemError>{
    use pse_backend_native::kkt::path::arclength::EventKind;
    use pse_model::generated::enums::NumericalPathEventKind as Kind;
    use pse_model::generated::runtime::solve_strategy_events::{
        RuntimeSolveStrategyEventsFieldPathEventsItem as Item,
        RuntimeSolveStrategyEventsFieldPathEventsItemLocalization as Localization,
    };
    let count = |value: u64| i64::try_from(value).map_err(|_| limit("path event work projection"));
    let rank = |value: usize| i64::try_from(value).map_err(|_| limit("path event rank projection"));
    Ok(Item {
        kind: match event.kind {
            EventKind::Regular => Kind::Regular,
            EventKind::SimpleFold => Kind::SimpleFold,
            EventKind::Unresolved => Kind::Unresolved,
        },
        source: event.source,
        family: event.family,
        point: event.point.clone(),
        localization: event.localization.map(|value| Localization {
            left: value.left,
            right: value.right,
            lower: value.interval.0,
            upper: value.interval.1,
            at: value.at,
        }),
        state_rank: rank(event.state_rank)?,
        augmented_rank: rank(event.augmented_rank)?,
        state_singular_values: event.state_singular_values.clone(),
        augmented_singular_values: event.augmented_singular_values.clone(),
        rank_threshold: event.rank_threshold,
        transversality: event.transversality,
        curvature: event.curvature,
        curvature_source: event.curvature_source,
        nondegeneracy_threshold: event.nondegeneracy_threshold,
        values: count(event.work.values)?,
        state_actions: count(event.work.state_actions)?,
        parameter_actions: count(event.work.parameter_actions)?,
        curvature_actions: count(event.work.curvature_actions)?,
        factorizations: count(event.work.factorizations)?,
        backsolves: count(event.work.backsolves)?,
        rank_probes: count(event.work.rank_probes)?,
    })
}
/// Execute a finite declaration. Every native/scientific effect is injected; the same
/// operation is used by production and scripted policy controls.
#[cfg(test)]
pub(crate) fn run<T>(
    strategy: &NumericalStrategy,
    scope: &pse_kernels::ExecutionScope,
    facts: impl FnMut(usize) -> Facts,
    execute: impl FnMut(usize, &Mechanism) -> Result<Attempt<T>, EffectFailure>,
    assess: impl FnMut(&T, Observation) -> Assessment,
) -> DriverResult<T> {
    run_inner(strategy, scope, facts, execute, assess, true, None)
}
pub(crate) fn run_admitted<T>(
    strategy: &NumericalStrategy,
    scope: &pse_kernels::ExecutionScope,
    admission: Arc<admission::TaskAdmission>,
    facts: impl FnMut(usize) -> Facts,
    execute: impl FnMut(usize, &Mechanism) -> Result<Attempt<T>, EffectFailure>,
    assess: impl FnMut(&T, Observation) -> Assessment,
) -> DriverResult<T> {
    run_inner(
        strategy,
        scope,
        facts,
        execute,
        assess,
        true,
        Some(admission),
    )
}
/// Classify a direct member already dispatched by the admitted native batch owner.
/// Post-native work, deadline checks and original permission use the same driver.
pub(crate) fn run_observed<T>(
    strategy: &NumericalStrategy,
    scope: &pse_kernels::ExecutionScope,
    facts: impl FnMut(usize) -> Facts,
    execute: impl FnMut(usize, &Mechanism) -> Result<Attempt<T>, EffectFailure>,
    assess: impl FnMut(&T, Observation) -> Assessment,
) -> DriverResult<T> {
    if strategy.mechanisms.len() != 1 || strategy.mechanisms[0].position != Position::Execution {
        return DriverResult {
            value: None,
            assessment: None,
            events: Vec::new(),
            terminal: Some(Arc::new(ProblemError::Contract(
                "already dispatched batch observations require one execution mechanism".into(),
            ))),
            work: WorkObservation {
                attempts: 0,
                evaluations: None,
                iterations: None,
                factorizations: None,
                proof_steps: None,
            },
        };
    }
    run_inner(strategy, scope, facts, execute, assess, false, None)
}
fn run_inner<T>(
    strategy: &NumericalStrategy,
    scope: &pse_kernels::ExecutionScope,
    mut facts: impl FnMut(usize) -> Facts,
    mut execute: impl FnMut(usize, &Mechanism) -> Result<Attempt<T>, EffectFailure>,
    mut assess: impl FnMut(&T, Observation) -> Assessment,
    check_before_dispatch: bool,
    shared_admission: Option<Arc<admission::TaskAdmission>>,
) -> DriverResult<T> {
    let mut result = DriverResult {
        value: None,
        assessment: None,
        events: Vec::new(),
        terminal: None,
        work: WorkObservation {
            attempts: 0,
            evaluations: None,
            iterations: None,
            factorizations: None,
            proof_steps: None,
        },
    };
    let ledger = shared_admission.unwrap_or_else(|| {
        admission::TaskAdmission::new(strategy.limits, scope.clone(), None, false)
    });
    let strategy_identity = match strategy.key() {
        Ok(key) => key,
        Err(cause) => {
            result.terminal = Some(Arc::new(ProblemError::Contract(cause.to_string())));
            return result;
        }
    };
    // Entry belongs to the first operation actually dispatched, including preparation.
    // Optional admission refusal has performed no operation and consumes no entry.
    let mut dispatched = false;
    for (index, mechanism) in strategy.mechanisms.iter().enumerate() {
        let phase = if mechanism.position == Position::Preparation {
            Phase::Preparation
        } else {
            Phase::Native
        };
        result.events.push(Event {
            mechanism: index,
            kind: EventKind::Planned,
            phase,
            original: None,
            decision: None,
            observation: None,
            transition: None,
            permission: None,
            work: None,
            cause: None,
        });
        let scope_check = if check_before_dispatch {
            scope.check().map_err(ProblemError::Provider)
        } else {
            Ok(())
        };
        if let Err(cause) = scope_check.and_then(|()| ledger.reserve_attempt()) {
            let observation = failure(&cause);
            let cause = Arc::new(cause);
            result.events.push(Event {
                mechanism: index,
                kind: EventKind::Refused,
                phase,
                original: None,
                decision: None,
                observation: Some(observation),
                transition: Some(Transition::Stop),
                permission: None,
                work: None,
                cause: Some(cause.clone()),
            });
            result.terminal = Some(cause);
            break;
        }
        let actual_facts = facts(index);
        let trusted_recovery = ledger.entry_dispatched()
            && strategy.start.recovery.contains(&actual_facts.start)
            && actual_facts.start != StartOrigin::Explicit;
        let admission = match admit(
            strategy,
            index,
            !dispatched && !trusted_recovery,
            &actual_facts,
        ) {
            Admission::Ready => match ledger.reserve(
                mechanism.limits,
                actual_facts.reservation,
                actual_facts.work_admitted,
            ) {
                Ok(()) => Admission::Ready,
                Err(cause) if mechanism.required => Admission::RequiredRefusal(Arc::new(cause)),
                Err(cause) => Admission::OptionalRefusal(Arc::new(cause)),
            },
            refused => refused,
        };
        match admission {
            Admission::Ready => {}
            Admission::OptionalRefusal(cause) | Admission::RequiredRefusal(cause) => {
                result.events.push(Event {
                    mechanism: index,
                    kind: EventKind::Refused,
                    phase,
                    original: None,
                    decision: None,
                    observation: Some(Observation::CapabilityRefusal),
                    transition: Some(if mechanism.required {
                        Transition::Stop
                    } else {
                        Transition::Continue
                    }),
                    permission: None,
                    work: None,
                    cause: Some(cause.clone()),
                });
                if mechanism.required {
                    result.terminal = Some(cause);
                    break;
                }
                continue;
            }
        }
        dispatched = true;
        ledger.mark_entry_dispatched();
        result.events.push(Event {
            mechanism: index,
            kind: EventKind::Started,
            phase,
            original: None,
            decision: None,
            observation: None,
            transition: None,
            permission: None,
            work: None,
            cause: None,
        });
        let mut attempt = match execute(index, mechanism) {
            Ok(attempt) => attempt,
            Err(failed) => {
                let mut identity = pse_ids::FramedHasher::new(pse_ids::Frame::NumericalWorkV1);
                identity
                    .hash(&strategy_identity)
                    .u64(index as u64)
                    .str("failed-dispatched-effect");
                let mut work = WorkCharge {
                    phase,
                    scope: if failed.local_expiry.is_some() || failed.local_refinement {
                        pse_model::strategy::Scope::Mechanism
                    } else {
                        pse_model::strategy::Scope::Task
                    },
                    charging_owner: identity.finish_hash(),
                    observed: failed.observed,
                };
                let accounting = match ledger.complete_charge(work) {
                    Ok(emitted) => {
                        work = emitted;
                        Ok(())
                    }
                    Err(error) => Err(error),
                };
                let enclosing = scope
                    .check()
                    .map_err(ProblemError::Provider)
                    .and(accounting);
                let observation = failed.observation.unwrap_or_else(|| failure(&failed.cause));
                let can_continue = enclosing.is_ok()
                    && (failed.local_expiry.is_some()
                        || failed.local_refinement
                        || (failed
                            .observation
                            .is_some_and(permits_numerical_continuation)
                            && permits_numerical_continuation(failure(&failed.cause))))
                    && !mechanism.required
                    && mechanism.transitions.contains(&Transition::Continue);
                let cause = failed.cause;
                result.events.push(Event {
                    mechanism: index,
                    kind: if can_continue {
                        EventKind::Abandoned
                    } else {
                        EventKind::Finished
                    },
                    phase,
                    original: None,
                    decision: None,
                    observation: Some(observation),
                    transition: Some(if can_continue {
                        Transition::Continue
                    } else {
                        Transition::Stop
                    }),
                    permission: None,
                    work: Some(work),
                    cause: Some(cause.clone()),
                });
                if let Err(refusal) = enclosing {
                    let refusal = Arc::new(refusal);
                    result.events.push(Event {
                        mechanism: index,
                        kind: EventKind::Abandoned,
                        phase,
                        original: None,
                        decision: None,
                        observation: Some(failure(&refusal)),
                        transition: Some(Transition::Stop),
                        permission: None,
                        work: Some(work),
                        cause: Some(refusal.clone()),
                    });
                    result.terminal = Some(refusal);
                    break;
                }
                if can_continue {
                    continue;
                }
                result.terminal = Some(cause);
                break;
            }
        };
        let accounting = match ledger.complete_charge(attempt.work) {
            Ok(work) => {
                attempt.work = work;
                Ok(())
            }
            Err(error) => Err(error),
        };
        // The enclosing clock is checked after indivisible work and before assessment/use.
        if let Err(cause) = scope
            .check()
            .map_err(ProblemError::Provider)
            .and(accounting)
        {
            let cause = Arc::new(cause);
            result.events.push(Event {
                mechanism: index,
                kind: EventKind::Abandoned,
                phase,
                original: None,
                decision: None,
                observation: Some(failure(&cause)),
                transition: Some(Transition::Stop),
                permission: None,
                work: Some(attempt.work),
                cause: Some(cause.clone()),
            });
            result.terminal = Some(cause);
            break;
        }
        let local_actual = actual_facts
            .reservation
            .map_or(attempt.work.observed, |bound| WorkObservation {
                attempts: attempt.work.observed.attempts,
                evaluations: attempt.work.observed.evaluations.or(bound.evaluations),
                iterations: attempt.work.observed.iterations.or(bound.iterations),
                factorizations: attempt
                    .work
                    .observed
                    .factorizations
                    .or(bound.factorizations),
                proof_steps: attempt.work.observed.proof_steps.or(bound.proof_steps),
            });
        if let Err(cause) = check_limits(mechanism.limits, local_actual) {
            let cause = Arc::new(cause);
            result.events.push(Event {
                mechanism: index,
                kind: EventKind::Abandoned,
                phase,
                original: None,
                decision: None,
                observation: Some(Observation::ResourceExhausted),
                transition: Some(if mechanism.required {
                    Transition::Stop
                } else {
                    Transition::Continue
                }),
                permission: None,
                work: Some(attempt.work),
                cause: Some(cause.clone()),
            });
            if mechanism.required {
                result.terminal = Some(cause);
                break;
            }
            continue;
        }
        if let Err(cause) = validate_outputs(&mechanism.operation, &attempt.evidence) {
            let cause = Arc::new(cause);
            let can_continue = !mechanism.required
                && mechanism.transitions.contains(&Transition::Continue)
                && permits_numerical_continuation(attempt.observation);
            result.events.push(Event {
                mechanism: index,
                kind: EventKind::Finished,
                phase,
                original: None,
                decision: None,
                observation: Some(attempt.observation),
                transition: Some(if can_continue {
                    Transition::Continue
                } else {
                    Transition::Stop
                }),
                permission: None,
                work: Some(attempt.work),
                cause: Some(cause.clone()),
            });
            if !can_continue {
                result.terminal = Some(cause);
                break;
            }
            continue;
        }
        let pending = AutoObservation {
            awaiting_assessment: true,
            native: attempt.observation,
            original: None,
            permission: None,
        };
        let request = pse_model::strategy::CompositionRequest {
            policy: pse_model::strategy::CompositionPolicy::Declared,
            branch: strategy.branch,
            limits: Some(strategy.limits),
            recovery: strategy.start.recovery.clone(),
        };
        let work = match ledger.observation() {
            Ok(work) => work,
            Err(cause) => {
                result.terminal = Some(Arc::new(cause));
                break;
            }
        };
        let decision = next_automatic(
            &request,
            &strategy.start,
            &[],
            &BTreeSet::new(),
            Some(&pending),
            work,
            actual_facts.inherited,
        );
        if !matches!(decision, AutoDecision::Assess) {
            result.terminal = Some(Arc::new(ProblemError::Internal(
                "original assessment was not selected".into(),
            )));
            break;
        }
        let assessment_decision =
            match automatic_decision_key(&request, &[], &decision, Some(&pending), work) {
                Ok(key) => {
                    let mut h = pse_ids::FramedHasher::new(pse_ids::Frame::NumericalDecisionV2);
                    h.hash(&attempt.work.charging_owner).hash(&key);
                    Some(h.finish_hash())
                }
                Err(cause) => {
                    result.terminal = Some(Arc::new(cause));
                    break;
                }
            };
        let mut assessment = assess(&attempt.value, attempt.observation);
        let assessment_accounting = assessment.work.iter_mut().try_for_each(|charge| {
            let accounting = match ledger.complete_charge(*charge) {
                Ok(emitted) => {
                    *charge = emitted;
                    Ok(())
                }
                Err(error) => Err(Arc::new(error)),
            };
            result.events.push(Event {
                mechanism: index,
                kind: EventKind::Finished,
                phase: charge.phase,
                original: None,
                decision: assessment_decision,
                observation: None,
                transition: None,
                permission: None,
                work: Some(*charge),
                cause: accounting.as_ref().err().cloned(),
            });
            accounting
        });
        if let Err(cause) = assessment_accounting {
            result.terminal = Some(cause);
            break;
        }
        if let Err(cause) = scope.check() {
            let cause = Arc::new(ProblemError::Provider(cause));
            result.events.push(Event {
                mechanism: index,
                kind: EventKind::Abandoned,
                phase: Phase::Assessment,
                original: None,
                decision: None,
                observation: Some(failure(&cause)),
                transition: Some(Transition::Stop),
                permission: None,
                work: Some(attempt.work),
                cause: Some(cause.clone()),
            });
            result.terminal = Some(cause);
            break;
        }
        let next = transition(mechanism, &assessment);
        result.events.push(Event {
            mechanism: index,
            kind: EventKind::Finished,
            phase: Phase::Assessment,
            original: Some(assessment.original.clone()),
            decision: assessment_decision,
            observation: Some(assessment.observation),
            transition: Some(next),
            permission: Some(assessment.retention.candidate.usability),
            work: Some(attempt.work),
            cause: assessment
                .original
                .cause()
                .or_else(|| assessment.cause.clone()),
        });
        result.value = Some(attempt.value);
        result.assessment = Some(assessment);
        if matches!(next, Transition::Finish | Transition::Stop) {
            break;
        }
    }
    match ledger.observation() {
        Ok(work) => result.work = work,
        Err(error) => result.terminal = Some(Arc::new(error)),
    }
    result
}

/// A direct declaration's consumed work caps come from admitted controls; an absent
/// strategy preserves the existing native counter observability and absolute deadline.
pub(crate) fn direct(controls: &pse_backend_native::solve::Controls) -> NumericalStrategy {
    NumericalStrategy::direct(
        controls.start,
        WorkLimits {
            attempts: 1,
            evaluations: None,
            iterations: None,
            factorizations: None,
            proof_steps: None,
        },
    )
}

/// Native categories stay observations, separate from independently composed permission.
pub(crate) fn observe(outcome: &super::solves::Outcome) -> Observation {
    match outcome {
        super::solves::Outcome::Constant(_) => Observation::Converged,
        super::solves::Outcome::Rejected(error) => runtime_failure(error),
        super::solves::Outcome::Native(report) => observe_native(report),
    }
}
/// Read the native owner's evidence without copying its retained report.
pub(crate) fn observe_native(report: &pse_backend_native::solve::SolveReport) -> Observation {
    use pse_backend_native::solve::Termination as T;
    if let Some(abandoned) = report.evidence.abandoned {
        return abandoned;
    }
    // A contained native unwind is terminal even when the triggering callback
    // carries a numerical domain refusal. Keep that source cause independently.
    if report.termination.category == T::Panic {
        return Observation::Panic;
    }
    if report.evidence.callback.terminal_failure {
        if let Some(cause) = report.shared_effective_failure() {
            return failure(&cause);
        }
        return match report.termination.category {
            T::Cancelled => Observation::Cancelled,
            T::TimeLimit => Observation::ResourceExhausted,
            T::Panic => Observation::Panic,
            _ => Observation::ContractFailure,
        };
    }
    if report.validation_failure().is_some()
        && let Some(cause) = report.shared_effective_failure()
    {
        return failure(&cause);
    }
    match report.termination.category {
        T::Success | T::Acceptable => Observation::Converged,
        T::IterationLimit | T::Limit => Observation::Limited,
        T::Cancelled => Observation::Cancelled,
        T::TimeLimit | T::ResourceExhausted => Observation::ResourceExhausted,
        T::Numerical | T::Evaluation => Observation::NumericalFailure,
        T::Panic => Observation::Panic,
        T::Invalid => Observation::ContractFailure,
        // Infeasibility is terminal only after independent certificate validation.
        T::Infeasible | T::Unbounded => Observation::Limited,
        _ => Observation::Limited,
    }
}
/// Preserve the actual typed failure independently of observation and candidate permission.
pub(crate) fn cause(outcome: &super::solves::Outcome) -> Option<Arc<ProblemError>> {
    match outcome {
        super::solves::Outcome::Constant(_) => None,
        super::solves::Outcome::Native(report) => cause_native(report),
        super::solves::Outcome::Rejected(error) => {
            Some(Arc::new(ProblemError::Math(pse_math::MathError::Typed {
                retained: size_of::<super::MathRuntimeError>(),
                cause: pse_model::diagnostic::DiagnosticCause::from_shared(error.clone()),
            })))
        }
    }
}
/// Actual native-owner work; absent measurements remain absent in every consumer.
pub(crate) fn work(outcome: &super::solves::Outcome) -> WorkObservation {
    match outcome {
        super::solves::Outcome::Native(report) => work_native(report),
        super::solves::Outcome::Constant(report) => WorkObservation {
            attempts: 1,
            evaluations: report.work.evaluations,
            iterations: report.work.iterations,
            factorizations: report.work.factorizations,
            proof_steps: report.work.proof_steps,
        },
        super::solves::Outcome::Rejected(_) => WorkObservation {
            attempts: 1,
            evaluations: None,
            iterations: None,
            factorizations: None,
            proof_steps: None,
        },
    }
}
pub(crate) fn cause_native(
    report: &pse_backend_native::solve::SolveReport,
) -> Option<Arc<ProblemError>> {
    (report.evidence.callback.terminal_failure || report.validation_failure().is_some())
        .then(|| report.shared_effective_failure())
        .flatten()
}
pub(crate) fn work_native(report: &pse_backend_native::solve::SolveReport) -> WorkObservation {
    WorkObservation {
        attempts: 1,
        evaluations: report.evidence.work.evaluations,
        iterations: report.evidence.work.iterations,
        factorizations: report.evidence.work.factorizations,
        proof_steps: report.evidence.work.proof_steps,
    }
}
pub(crate) fn failure(cause: &ProblemError) -> Observation {
    match cause {
        ProblemError::Native { kind, .. } => match kind {
            pse_backend_native::NativeFailureKind::Resource => Observation::ResourceExhausted,
            pse_backend_native::NativeFailureKind::Contract => Observation::ContractFailure,
            pse_backend_native::NativeFailureKind::Numerical => Observation::NumericalFailure,
            pse_backend_native::NativeFailureKind::Infrastructure => {
                Observation::OperationalFailure
            }
        },
        ProblemError::Numerical { .. } => Observation::NumericalFailure,
        ProblemError::Linear { kind, .. } => match kind {
            pse_backend_native::LinearFailureKind::Contract => Observation::ContractFailure,
            pse_backend_native::LinearFailureKind::Numerical => Observation::NumericalFailure,
            pse_backend_native::LinearFailureKind::Memory => Observation::ResourceExhausted,
            pse_backend_native::LinearFailureKind::Internal => Observation::OperationalFailure,
        },
        ProblemError::Unavailable { .. }
        | ProblemError::Unsupported(_)
        | ProblemError::RouteRefused(_)
        | ProblemError::DynamicRouteRefused(_) => Observation::CapabilityRefusal,
        ProblemError::Limit { .. } => Observation::ResourceExhausted,
        ProblemError::Cancelled => Observation::Cancelled,
        ProblemError::Internal(_) => Observation::OperationalFailure,
        _ => match pse_backend_native::callback::classify(cause) {
            pse_backend_native::callback::Failure::Trial => Observation::NumericalFailure,
            pse_backend_native::callback::Failure::Stopped(
                pse_backend_native::solve::Termination::Cancelled,
            ) => Observation::Cancelled,
            pse_backend_native::callback::Failure::Stopped(_) => Observation::ResourceExhausted,
            pse_backend_native::callback::Failure::Fatal => boundary_failure(
                cause
                    .boundary_diagnostic(pse_diagnostics::DiagnosticStage::Native)
                    .class,
            ),
        },
    }
}
pub(crate) fn runtime_failure(error: &super::MathRuntimeError) -> Observation {
    use super::MathRuntimeError as E;
    match error {
        E::Shared(error) => runtime_failure(error),
        E::Strategy { cause, .. } | E::StrategyTraceUnavailable { cause, .. } => {
            runtime_failure(cause)
        }
        E::Solve(cause) => failure(cause),
        E::Cancelled => Observation::Cancelled,
        E::Limit(_) | E::Pool(_) => Observation::ResourceExhausted,
        E::Panic(_) => Observation::Panic,
        E::Infrastructure(_) | E::Retiring => Observation::OperationalFailure,
        E::Math(cause) => boundary_failure(
            cause
                .boundary_diagnostic(pse_diagnostics::DiagnosticStage::Native)
                .class,
        ),
        E::Compile(cause) => boundary_failure(
            cause
                .boundary_diagnostic(pse_diagnostics::DiagnosticStage::Native)
                .class,
        ),
    }
}
/// Scientific consumers classify the same typed failure before applying their own
/// stage, parameter, statistical or control policy.
pub(crate) fn workflow_failure(error: &crate::workflow::WorkflowError) -> Observation {
    if let crate::workflow::WorkflowError::Math(error) = error {
        return runtime_failure(error);
    }
    boundary_failure(error.boundary_diagnostic().class)
}
pub(crate) const fn boundary_failure(class: pse_model::diagnostic::BoundaryClass) -> Observation {
    use pse_model::diagnostic::BoundaryClass as B;
    match class {
        B::TrialRejected | B::Numerical => Observation::NumericalFailure,
        B::Cancelled => Observation::Cancelled,
        B::ResourceLimit => Observation::ResourceExhausted,
        B::Infrastructure => Observation::OperationalFailure,
        B::Unsupported => Observation::CapabilityRefusal,
        B::Internal => Observation::ContractFailure,
        B::InvalidModel | B::Nonfinite | B::Inconclusive | B::Conflict | B::Incompatible => {
            Observation::ContractFailure
        }
    }
}
/// A later scientific target may use a completed/partial numerical trajectory, while
/// terminal failures remain terminal regardless of the outer consumer's retry policy.
pub(crate) const fn permits_numerical_continuation(observation: Observation) -> bool {
    matches!(
        observation,
        Observation::Converged
            | Observation::Auxiliary
            | Observation::Limited
            | Observation::Stalled
            | Observation::NumericalFailure
    )
}

#[cfg(test)]
mod tests;
