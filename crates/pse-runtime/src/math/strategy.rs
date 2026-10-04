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

pub(crate) mod path_control;

/// Facts supplied by the prepared mathematical/native owners, never inferred from history.
pub(crate) struct Facts {
    pub(crate) support: BTreeSet<ContentHash>,
    pub(crate) accuracy: Vec<pse_model::strategy::AccuracyEvidence>,
    pub(crate) start: StartOrigin,
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
        } else if strategy.accuracy.iter().any(|demand| {
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

/// Independent assessment; a successful auxiliary solve never grants original permission.
pub(crate) struct Assessment {
    pub(crate) auxiliary: bool,
    pub(crate) retention: StepRetention,
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
    if assessment.retention.candidate.permits_use() && !assessment.auxiliary {
        return Transition::Finish;
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
}
impl Ledger {
    pub(crate) fn new(limits: WorkLimits) -> Self {
        Self {
            limits,
            charged: BTreeSet::new(),
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
    pub(crate) fn reserve_attempt(&self) -> Result<(), ProblemError> {
        if self.total.attempts >= self.limits.attempts {
            Err(limit("task execution allowance"))
        } else {
            Ok(())
        }
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
        check_limits(self.limits, self.total)
    }
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
    pub(crate) observation: Option<Observation>,
    pub(crate) transition: Option<Transition>,
    pub(crate) permission: Option<pse_model::generated::enums::CandidateUse>,
    pub(crate) work: Option<WorkCharge>,
    pub(crate) cause: Option<Arc<ProblemError>>,
}
pub(crate) struct Attempt<T> {
    pub(crate) value: T,
    pub(crate) observation: Observation,
    pub(crate) work: WorkCharge,
}
/// A dispatched effect that failed still owns its actual work. A local clock is
/// supplied only by the producer that narrowed it; ordinary resource errors never
/// acquire local-abandonment semantics from their diagnostic class.
pub(crate) struct EffectFailure {
    pub(crate) cause: Arc<ProblemError>,
    pub(crate) observed: WorkObservation,
    local_expiry: Option<std::time::Instant>,
    local_refinement: bool,
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
        }
    }
}
impl EffectFailure {
    pub(crate) fn observed(cause: Arc<ProblemError>, observed: WorkObservation) -> Self {
        Self {
            cause,
            observed,
            local_expiry: None,
            local_refinement: false,
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
    pub(crate) path_events:
        Option<pse_math::SharedAllocation<Vec<pse_backend_native::kkt::path::arclength::Event>>>,
}
/// The actual library phase has no native adapter; its retained options/task key is
/// separate from the original corrector's native profile.
#[derive(Clone, Copy, Debug)]
pub(crate) enum ProviderEvidence {
    Native(pse_model::strategy::ProfileRef),
    Library(ContentHash),
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
    /// Project ordered events without reconstructing numerical decisions from metrics.
    pub fn rows(
        &self,
        run_id: pse_model::generated::identities::RunId,
        step: usize,
    ) -> Result<Vec<pse_model::generated::runtime::solve_strategy_events::Row>, ProblemError> {
        use pse_model::generated::runtime::solve_strategy_events::Row;
        let strategy_identity = self
            .declaration
            .key()
            .map_err(|e| ProblemError::Contract(e.to_string()))?;
        let ordinal =
            |value: usize| i64::try_from(value).map_err(|_| limit("strategy event ordinal"));
        let count = |value: Option<u64>| {
            value
                .map(|n| i64::try_from(n).map_err(|_| limit("strategy work projection")))
                .transpose()
        };
        self.events
            .iter()
            .enumerate()
            .map(|(index, event)| {
                let mechanism = &self.declaration.mechanisms[event.mechanism];
                let actual = !matches!(event.kind, EventKind::Planned | EventKind::Refused);
                let product = actual.then(|| self.products.get(event.mechanism)).flatten();
                let produced = matches!(event.kind, EventKind::Finished | EventKind::Abandoned)
                    .then_some(product)
                    .flatten();
                let work = event.work;
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
                    derived_identity: product.and_then(|p| p.derived),
                    profile_identity: actual.then_some(match product.and_then(|p| p.provider) {
                        Some(ProviderEvidence::Library(key)) => key,
                        Some(ProviderEvidence::Native(profile)) => profile.key,
                        None => mechanism.profile.map_or(self.profile, |p| p.key),
                    }),
                    backend: if let Some(provider) = product.and_then(|p| p.provider) {
                        match provider {
                            ProviderEvidence::Library(_) => None,
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
            .collect()
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
pub(crate) fn run<T>(
    strategy: &NumericalStrategy,
    scope: &pse_kernels::ExecutionScope,
    facts: impl FnMut(usize) -> Facts,
    execute: impl FnMut(usize, &Mechanism) -> Result<Attempt<T>, EffectFailure>,
    assess: impl FnMut(&T, Observation) -> Assessment,
) -> DriverResult<T> {
    run_inner(strategy, scope, facts, execute, assess, true)
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
    run_inner(strategy, scope, facts, execute, assess, false)
}
fn run_inner<T>(
    strategy: &NumericalStrategy,
    scope: &pse_kernels::ExecutionScope,
    mut facts: impl FnMut(usize) -> Facts,
    mut execute: impl FnMut(usize, &Mechanism) -> Result<Attempt<T>, EffectFailure>,
    mut assess: impl FnMut(&T, Observation) -> Assessment,
    check_before_dispatch: bool,
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
    let mut ledger = Ledger::new(strategy.limits);
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
                observation: Some(observation),
                transition: Some(Transition::Stop),
                permission: None,
                work: None,
                cause: Some(cause.clone()),
            });
            result.terminal = Some(cause);
            break;
        }
        match admit(strategy, index, !dispatched, &facts(index)) {
            Admission::Ready => {}
            Admission::OptionalRefusal(cause) | Admission::RequiredRefusal(cause) => {
                result.events.push(Event {
                    mechanism: index,
                    kind: EventKind::Refused,
                    phase,
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
        result.events.push(Event {
            mechanism: index,
            kind: EventKind::Started,
            phase,
            observation: None,
            transition: None,
            permission: None,
            work: None,
            cause: None,
        });
        let attempt = match execute(index, mechanism) {
            Ok(attempt) => attempt,
            Err(failed) => {
                let mut identity = pse_ids::FramedHasher::new(pse_ids::Frame::NumericalWorkV1);
                identity
                    .hash(&strategy_identity)
                    .u64(index as u64)
                    .str("failed-dispatched-effect");
                let work = WorkCharge {
                    phase,
                    scope: if failed.local_expiry.is_some() || failed.local_refinement {
                        pse_model::strategy::Scope::Mechanism
                    } else {
                        pse_model::strategy::Scope::Task
                    },
                    charging_owner: identity.finish_hash(),
                    observed: failed.observed,
                };
                let accounting = ledger.charge(work);
                let enclosing = scope
                    .check()
                    .map_err(ProblemError::Provider)
                    .and(accounting);
                let can_continue = enclosing.is_ok()
                    && (failed.local_expiry.is_some() || failed.local_refinement)
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
                    observation: Some(failure(&cause)),
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
        let accounting = ledger.charge(attempt.work);
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
                observation: Some(failure(&cause)),
                transition: Some(Transition::Stop),
                permission: None,
                work: Some(attempt.work),
                cause: Some(cause.clone()),
            });
            result.terminal = Some(cause);
            break;
        }
        if let Err(cause) = check_limits(mechanism.limits, attempt.work.observed) {
            let cause = Arc::new(cause);
            result.events.push(Event {
                mechanism: index,
                kind: EventKind::Abandoned,
                phase,
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
        let assessment = assess(&attempt.value, attempt.observation);
        if let Err(cause) = scope.check() {
            let cause = Arc::new(ProblemError::Provider(cause));
            result.events.push(Event {
                mechanism: index,
                kind: EventKind::Abandoned,
                phase: Phase::Assessment,
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
            observation: Some(assessment.observation),
            transition: Some(next),
            permission: Some(assessment.retention.candidate.usability),
            work: Some(attempt.work),
            cause: assessment.cause.clone(),
        });
        result.value = Some(attempt.value);
        result.assessment = Some(assessment);
        if matches!(next, Transition::Finish | Transition::Stop) {
            break;
        }
    }
    result.work = ledger.observation();
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
    if report.evidence.callback.terminal_failure {
        if let Some(cause) = report.callback_failure() {
            return failure(cause);
        }
        return match report.termination.category {
            T::Cancelled => Observation::Cancelled,
            T::TimeLimit => Observation::ResourceExhausted,
            T::Panic => Observation::Panic,
            _ => Observation::ContractFailure,
        };
    }
    if let Some(cause) = report.validation_failure() {
        return failure(cause);
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
        super::solves::Outcome::Constant(_) => WorkObservation {
            attempts: 1,
            evaluations: Some(1),
            iterations: Some(0),
            factorizations: Some(0),
            proof_steps: Some(0),
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
    report
        .evidence
        .callback
        .terminal_failure
        .then(|| report.shared_callback_failure())
        .flatten()
        .or_else(|| report.shared_validation_failure())
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
            pse_backend_native::callback::Failure::Fatal => Observation::ContractFailure,
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
        E::Compile(_) => Observation::ContractFailure,
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
