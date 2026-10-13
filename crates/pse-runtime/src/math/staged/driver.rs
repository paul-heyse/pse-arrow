// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared full numerical strategy; dispatch determines the owning native lane.
use super::super as math;
use super::*;
use std::{future::Future, pin::Pin};

type Pending<'a, T> = Pin<Box<dyn Future<Output = Result<T, MathRuntimeError>> + Send + 'a>>;
pub(super) trait Dispatch {
    fn reserve(
        &self,
        service: &MathService,
        name: &str,
        bytes: usize,
    ) -> Result<Arc<pse_columnar::AllocationLease>, MathRuntimeError> {
        service.reserve(name, bytes)
    }
    fn run_task<'a, T: Send + 'static>(
        &'a self,
        cores: usize,
        backends: Vec<Backend>,
        deadline: Option<std::time::Instant>,
        scope: Option<pse_kernels::ExecutionScope>,
        cancel: &'a crate::CancelSource,
        work: impl FnOnce(
            &mut Retained,
            &Arc<AtomicBool>,
            &Arc<WorkerBudget>,
        ) -> Result<T, MathRuntimeError>
        + Send
        + 'static,
    ) -> Pending<'a, T>;
    fn prepare<'a>(
        &'a self,
        operation: math::solves::AutomaticOperation,
        scope: pse_kernels::ExecutionScope,
        cancel: &'a crate::CancelSource,
    ) -> Pending<'a, math::solves::PreparedRung>;
}
impl Dispatch for NativeSession {
    fn run_task<'a, T: Send + 'static>(
        &'a self,
        cores: usize,
        backends: Vec<Backend>,
        deadline: Option<std::time::Instant>,
        scope: Option<pse_kernels::ExecutionScope>,
        cancel: &'a crate::CancelSource,
        work: impl FnOnce(
            &mut Retained,
            &Arc<AtomicBool>,
            &Arc<WorkerBudget>,
        ) -> Result<T, MathRuntimeError>
        + Send
        + 'static,
    ) -> Pending<'a, T> {
        Box::pin(NativeSession::run_task(
            self, cores, backends, deadline, scope, cancel, work,
        ))
    }
    fn prepare<'a>(
        &'a self,
        operation: math::solves::AutomaticOperation,
        scope: pse_kernels::ExecutionScope,
        cancel: &'a crate::CancelSource,
    ) -> Pending<'a, math::solves::PreparedRung> {
        Box::pin(math::jobs::population_scope(
            self.population.clone(),
            self.service
                .prepare_automatic_operation(operation, scope, cancel),
        ))
    }
}
pub(super) struct StepDriver<'a, D> {
    pub(super) service: Arc<MathService>,
    pub(super) dispatcher: &'a D,
}
impl<D: Dispatch> StepDriver<'_, D> {
    /// Resolve the full strategy on its dispatch owner. Preparation uses that owner's
    /// quiescent seam; no retry obtains a new task clock or work allowance.
    #[expect(
        clippy::too_many_arguments,
        reason = "the common step binds its admitted scientific task"
    )]
    pub(crate) async fn step<T: Send + 'static>(
        &self,
        step: math::solves::PreparedSolve,
        previous: Option<math::solves::Predecessor>,
        attempt: usize,
        progress: Arc<Progress>,
        owner: Arc<pse_columnar::AllocationLease>,
        cancel: &crate::CancelSource,
        assess: impl FnMut(
            &math::solves::Outcome,
            Option<&pse_kernels::ExecutionScope>,
            &Arc<WorkerBudget>,
        ) -> math::strategy::Assessed<T>
        + Send
        + 'static,
    ) -> Result<(math::solves::Outcome, T, math::solves::StrategyTrace), MathRuntimeError> {
        use math::strategy::accuracy_refinement::{Decision, Progress as AccuracyProgress};
        use pse_model::generated::enums::AccuracyUnavailableReason as U;
        // Existing journeys without requested output evidence take exactly the
        // same path, with no additional assessment, factor or admission owner.
        if step.numerics().policy.goals.is_empty() {
            return self
                .step_initial(step, previous, attempt, progress, owner, cancel, assess)
                .await;
        }
        let request = step.composition_request();
        request
            .validate()
            .map_err(|e| ProblemError::Contract(e.to_string()))?;
        let deadline = std::time::Instant::now()
            .checked_add(step.time_limit())
            .ok_or_else(|| ProblemError::Contract("accuracy task deadline extent".into()))?;
        let scope = step
            .task_scope()
            .unwrap_or_else(|| pse_kernels::ExecutionScope::new(Arc::default(), Some(deadline)));
        let mut limits = request
            .limits
            .unwrap_or_else(|| step.numerical_strategy().limits);
        if request.limits.is_none() && !step.composition_is_declared() {
            if let Some(admission) = step.task_admission() {
                limits = admission.limits()?;
            } else {
                limits.attempts = step.automatic_attempt_capacity(scope.cancellation())?;
            }
            if step.task_admission().is_none()
                && step.numerics().policy.goals.iter().any(|goal| goal.refine)
            {
                // A requested goal admits one default original correction before
                // the first dispatch. The same finite owner applies throughout;
                // explicit counters and an enclosing admission are never enlarged.
                limits.attempts = limits
                    .attempts
                    .checked_add(1)
                    .ok_or_else(|| ProblemError::memory("accuracy default attempt capacity"))?;
            }
        }
        let admission = step.task_admission().unwrap_or_else(|| {
            math::strategy::admission::TaskAdmission::new_for(
                limits,
                scope.clone(),
                &self.service,
                step.declared_foreign_bytes() > 0,
            )
        });
        let step = step.within_admitted_task(scope.clone(), admission.clone())?;
        let assess = Arc::new(std::sync::Mutex::new(assess));
        let signal = Arc::new(std::sync::Mutex::new(AccuracySignal::<T> {
            original_satisfied: false,
            goals: Vec::new(),
            stop: None,
        }));
        let callback = accuracy_assessor(assess.clone(), signal.clone());
        let mut result = self
            .step_initial(
                step.clone(),
                previous,
                attempt,
                progress.clone(),
                owner.clone(),
                cancel,
                callback,
            )
            .await?;
        let mut accuracy_progress = AccuracyProgress::default();
        loop {
            let (goals, stop, decision) = {
                let observed = signal.lock().map_err(|_| {
                    ProblemError::Internal("accuracy assessment owner panicked".into())
                })?;
                let goals = observed.goals.clone();
                let decision = accuracy_progress.next(observed.original_satisfied, &goals);
                (goals, observed.stop, decision)
            };
            let stopped = match decision {
                Decision::Finish => break,
                Decision::Stop(reason) => Some(reason),
                Decision::Refine => {
                    scope.check().map_err(ProblemError::Provider)?;
                    if admission.observation()?.attempts >= limits.attempts {
                        Some(U::Budget)
                    } else {
                        None
                    }
                }
            };
            if let Some(reason) = stopped {
                if let Some(stop) = stop {
                    stop(&mut result.1, &result.0, &step.numerics().policy, reason);
                }
                result.2 = self.accuracy_stopped_trace(&result.2, reason)?;
                break;
            }
            let point = match &result.0 {
                math::solves::Outcome::Native(report) => {
                    report.candidate.as_ref().map(|c| c.primal.as_slice())
                }
                _ => None,
            };
            let Some(point) = point else {
                if let Some(stop) = stop {
                    stop(
                        &mut result.1,
                        &result.0,
                        &step.numerics().policy,
                        U::MissingObservation,
                    );
                }
                result.2 = self.accuracy_stopped_trace(&result.2, U::MissingObservation)?;
                break;
            };
            let refined = match step.refine_coordinate_accuracy(&goals, point) {
                Ok(Some(refined)) => {
                    refined.within_admitted_task(scope.clone(), admission.clone())?
                }
                preparation => {
                    let reason = match preparation {
                        Ok(None) => U::Unsupported,
                        Err(error) => return Err(error.into()),
                        Ok(Some(_)) => {
                            return Err(ProblemError::Internal(
                                "accuracy work preparation disappeared".into(),
                            )
                            .into());
                        }
                    };
                    if let Some(stop) = stop {
                        stop(&mut result.1, &result.0, &step.numerics().policy, reason);
                    }
                    result.2 = self.accuracy_stopped_trace(&result.2, reason)?;
                    break;
                }
            };
            if refined.backend() != step.backend()
                || refined.numerics().key != step.numerics().key
                || refined.original_identity()? != step.original_identity()?
            {
                return Err(ProblemError::Contract(
                    "accuracy work changed frozen original acceptance".into(),
                )
                .into());
            }
            let declaration = refined.numerical_strategy();
            let mut mechanism = declaration.mechanisms.first().cloned().ok_or_else(|| {
                ProblemError::Contract("accuracy correction has no native execution owner".into())
            })?;
            mechanism.required = true;
            mechanism.limits = limits;
            mechanism.transitions = vec![
                pse_model::strategy::Transition::Finish,
                pse_model::strategy::Transition::Stop,
            ];
            let ordinal = attempt
                .checked_add(
                    usize::try_from(admission.observation()?.attempts)
                        .map_err(|_| ProblemError::memory("accuracy attempt ordinal"))?,
                )
                .ok_or_else(|| ProblemError::memory("accuracy attempt ordinal"))?;
            let next = self
                .step_bound(
                    refined.clone(),
                    None,
                    ordinal,
                    progress.clone(),
                    owner.clone(),
                    cancel,
                    accuracy_assessor(assess.clone(), signal.clone()),
                    Some(math::solves::PreparedRung::Original(Box::new(refined))),
                    Some(mechanism),
                    None,
                    false,
                    Some(admission.clone()),
                )
                .await?;
            let trace = self.accuracy_merged_trace(&result.2, &next.2)?;
            result = (next.0, next.1, trace);
        }
        Ok(result)
    }

    fn accuracy_stopped_trace(
        &self,
        trace: &math::solves::StrategyTrace,
        reason: pse_model::generated::enums::AccuracyUnavailableReason,
    ) -> Result<math::solves::StrategyTrace, MathRuntimeError> {
        let cause = Arc::new(ProblemError::numerical(format!(
            "output accuracy refinement stopped: {}",
            reason.as_str()
        )));
        let bytes = trace
            .retained_bytes()?
            .checked_add(size_of::<math::strategy::Event>())
            .and_then(|n| n.checked_add(cause.retained_bytes()))
            .ok_or_else(|| ProblemError::memory("accuracy stopping trace extent"))?;
        let owner =
            self.dispatcher
                .reserve(&self.service, "math:accuracy-stopping-trace", bytes)?;
        let mut events = Vec::with_capacity(
            trace
                .events
                .len()
                .checked_add(1)
                .ok_or_else(|| ProblemError::memory("accuracy stopping event extent"))?,
        );
        events.extend(trace.events.iter().cloned());
        if let Some(mechanism) = trace.declaration.mechanisms.len().checked_sub(1) {
            events.push(math::strategy::Event {
                mechanism,
                kind: pse_model::generated::enums::NumericalEventKind::Finished,
                phase: pse_model::strategy::Phase::Assessment,
                original: Some(math::strategy::OriginalConclusion::Satisfied),
                decision: None,
                observation: None,
                transition: Some(pse_model::strategy::Transition::Stop),
                permission: trace.events.iter().rev().find_map(|e| e.permission),
                work: None,
                cause: Some(cause),
            });
        }
        Ok(Arc::new(
            math::strategy::Trace {
                owner: None,
                publication_request: trace.publication_request,
                declaration: trace.declaration.clone(),
                original: trace.original,
                backend: trace.backend,
                profile: trace.profile,
                start: trace.start,
                starts: trace.starts.clone(),
                products: trace.products.clone(),
                events,
            }
            .with_owner(owner),
        ))
    }

    fn accuracy_merged_trace(
        &self,
        prefix: &math::solves::StrategyTrace,
        next: &math::solves::StrategyTrace,
    ) -> Result<math::solves::StrategyTrace, MathRuntimeError> {
        let bytes = prefix
            .retained_bytes()?
            .checked_add(next.retained_bytes()?)
            .ok_or_else(|| ProblemError::memory("accuracy composed trace extent"))?;
        let owner =
            self.dispatcher
                .reserve(&self.service, "math:accuracy-composed-trace", bytes)?;
        let offset = prefix.declaration.mechanisms.len();
        let mut declaration = prefix.declaration.clone();
        let capacity = offset
            .checked_add(next.declaration.mechanisms.len())
            .ok_or_else(|| ProblemError::memory("accuracy mechanism extent"))?;
        declaration.mechanisms = Vec::with_capacity(capacity);
        declaration
            .mechanisms
            .extend(prefix.declaration.mechanisms.iter().cloned());
        declaration
            .mechanisms
            .extend(next.declaration.mechanisms.iter().cloned());
        let mut starts = Vec::with_capacity(capacity);
        starts.extend(prefix.starts.iter().copied());
        starts.extend(next.starts.iter().copied());
        let mut products = Vec::with_capacity(capacity);
        products.extend(prefix.products.iter().cloned());
        products.extend(next.products.iter().cloned());
        let mut events = Vec::with_capacity(
            prefix
                .events
                .len()
                .checked_add(next.events.len())
                .ok_or_else(|| ProblemError::memory("accuracy composed event extent"))?,
        );
        events.extend(prefix.events.iter().cloned());
        for mut event in next.events.iter().cloned() {
            event.mechanism = event
                .mechanism
                .checked_add(offset)
                .ok_or_else(|| ProblemError::memory("accuracy event mechanism"))?;
            events.push(event);
        }
        Ok(Arc::new(
            math::strategy::Trace {
                owner: None,
                publication_request: prefix.publication_request,
                declaration,
                original: prefix.original,
                backend: prefix.backend,
                profile: prefix.profile,
                start: prefix.start,
                starts,
                products,
                events,
            }
            .with_owner(owner),
        ))
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "initial dispatch forwards the same admitted operation contract as step and step_bound"
    )]
    async fn step_initial<T: Send + 'static>(
        &self,
        step: math::solves::PreparedSolve,
        previous: Option<math::solves::Predecessor>,
        attempt: usize,
        progress: Arc<Progress>,
        owner: Arc<pse_columnar::AllocationLease>,
        cancel: &crate::CancelSource,
        assess: impl FnMut(
            &math::solves::Outcome,
            Option<&pse_kernels::ExecutionScope>,
            &Arc<WorkerBudget>,
        ) -> math::strategy::Assessed<T>
        + Send
        + 'static,
    ) -> Result<(math::solves::Outcome, T, math::solves::StrategyTrace), MathRuntimeError> {
        use pse_model::strategy::{CompositionPolicy, Mechanism, Position, Transition};
        let request = step.composition_request().clone();
        request
            .validate()
            .map_err(|error| ProblemError::Contract(error.to_string()))?;
        if step.composition_is_declared() {
            return self
                .step_bound(
                    step, previous, attempt, progress, owner, cancel, assess, None, None, None,
                    false, None,
                )
                .await;
        }
        if request.policy == CompositionPolicy::Declared {
            return Err(ProblemError::Contract(
                "declared composition requires an actually bound declaration".into(),
            )
            .into());
        }
        let deadline = std::time::Instant::now()
            .checked_add(step.time_limit())
            .ok_or_else(|| ProblemError::Contract("automatic task deadline extent".into()))?;
        let scope = step
            .task_scope()
            .unwrap_or_else(|| pse_kernels::ExecutionScope::new(Arc::default(), Some(deadline)));
        let step = step.within_task(scope.clone())?;
        let mut declaration = step.numerical_strategy();
        declaration.branch = request.branch;
        declaration.start.recovery = request.recovery.clone();
        if let Some(limits) = request.limits {
            declaration.limits = limits;
        }
        if request.limits.is_none() {
            if let Some(admission) = step.task_admission() {
                declaration.limits = admission.limits()?;
            } else {
                declaration.limits.attempts =
                    step.automatic_attempt_capacity(scope.cancellation())?;
            }
        }
        let ledger = step.task_admission().unwrap_or_else(|| {
            math::strategy::admission::TaskAdmission::new_for(
                declaration.limits,
                scope.clone(),
                &self.service,
                step.declared_foreign_bytes() > 0,
            )
        });
        let step = step.within_admitted_task(scope.clone(), ledger.clone())?;
        let capacity = usize::try_from(declaration.limits.attempts)
            .map_err(|_| ProblemError::memory("automatic product state capacity"))?
            .checked_mul(3)
            .ok_or_else(|| ProblemError::memory("automatic product state capacity"))?;
        let product_owner = self.dispatcher.reserve(
            &self.service,
            "math:automatic-product-state",
            math::strategy::ProductState::extent(capacity)?,
        )?;
        let shared_products = Arc::new(std::sync::Mutex::new(math::strategy::ProductState::owned(
            capacity,
            product_owner,
        )));
        let assess = Arc::new(std::sync::Mutex::new(assess));
        let initial = step.entry_origin(previous.is_some());
        let inherited = previous.is_some() && initial == pse_model::strategy::StartOrigin::Accepted;
        let mut original_operations = step.automatic_entry_operations(initial, inherited, scope.cancellation())?;
        for operation in &mut original_operations {
            if !operation.candidate.replacement {
                operation.candidate.start = initial;
            }
        }
        let advice_cohort = step.advice_cohort(initial, inherited)?;
        let mut operations = original_operations;
        let mut bound_operations = std::collections::BTreeMap::new();
        let mut attempted = std::collections::BTreeSet::new();
        let mut attempted_bindings = std::collections::BTreeSet::new();
        let mut last_observation = None;
        let mut last_result = None;
        let mut current_assessment = false;
        let mut terminal = None;
        let mut collected = math::strategy::Trace {
            owner: None,
            publication_request: Some(step.request_identity()?),
            declaration: declaration.clone(),
            original: step.original_identity()?,
            backend: step.backend(),
            profile: step.strategy_profile()?,
            start: initial,
            starts: Vec::new(),
            products: Vec::new(),
            events: Vec::new(),
        };
        collected.declaration.mechanisms.clear();
        loop {
            if let Err(error) = scope.check() {
                // No new preparation, dispatch or assessment may follow task expiry.
                // Finalization still owns the actual preceding result and its trace.
                terminal = Some(Arc::new(ProblemError::Provider(error)));
                break;
            }
            let candidates = operations
                .iter()
                .map(|operation| operation.candidate.clone())
                .collect::<Vec<_>>();
            let mut constrained = request.clone();
            constrained.limits = Some(declaration.limits);
            let decision = math::strategy::next_automatic(
                &constrained,
                &declaration.start,
                &candidates,
                &attempted,
                last_observation.as_ref(),
                ledger.observation()?,
                inherited,
                advice_cohort.as_ref(),
            );
            let resolved = math::strategy::automatic_decision_key(
                &request,
                &candidates,
                &decision,
                last_observation.as_ref(),
                ledger.observation()?,
            )?;
            let mut decision_identity =
                pse_ids::FramedHasher::new(pse_ids::Frame::NumericalDecisionV2);
            decision_identity
                .hash(&collected.original)
                .hash(&step.preparation_identity()?)
                .hash(&resolved);
            let decision_key = decision_identity.finish_hash();
            match decision {
                math::strategy::AutoDecision::Finish => break,
                math::strategy::AutoDecision::Exhausted { cause } => {
                    if current_assessment {
                        if let Some(event) = collected.events.last_mut() {
                            event.transition = Some(Transition::Stop);
                            if event.cause.is_none() {
                                event.cause = cause;
                            }
                        }
                    } else {
                        terminal = Some(
                            collected
                                .events
                                .iter()
                                .rev()
                                .find_map(|event| event.cause.clone())
                                .unwrap_or_else(|| {
                                    Arc::new(ProblemError::Unsupported(
                                        "catalog exhausted after an unassessed operation".into(),
                                    ))
                                }),
                        );
                    }
                    break;
                }
                math::strategy::AutoDecision::Assess => {
                    return Err(ProblemError::Internal(
                        "original assessment must execute on its admitted worker".into(),
                    )
                    .into());
                }
                math::strategy::AutoDecision::Stop { cause } => {
                    terminal = cause.clone();
                    if let Some(cause) = cause
                        && let Some(index) = collected.declaration.mechanisms.len().checked_sub(1)
                    {
                        collected.events.push(math::strategy::Event {
                            mechanism: index,
                            kind: pse_model::generated::enums::NumericalEventKind::Abandoned,
                            phase: pse_model::strategy::Phase::Assessment,
                            original: None,
                            decision: Some(decision_key),
                            observation: Some(math::strategy::failure(&cause)),
                            transition: Some(Transition::Stop),
                            permission: None,
                            work: None,
                            cause: Some(cause),
                        });
                    }
                    break;
                }
                math::strategy::AutoDecision::Prepare { candidate }
                | math::strategy::AutoDecision::Dispatch { candidate } => {
                    current_assessment = false;
                    let preparing =
                        matches!(decision, math::strategy::AutoDecision::Prepare { .. });
                    if !preparing {
                        attempted.insert(candidate);
                    }
                    let operation = operations[candidate].clone();
                    if !preparing && !attempted_bindings.insert(operation.candidate.identity) {
                        continue;
                    }
                    let description = operation.candidate.clone();
                    let mechanism = Mechanism {
                        kind: description.kind,
                        position: if preparing {
                            Position::Preparation
                        } else {
                            Position::Execution
                        },
                        required: description.kind == pse_model::strategy::MechanismKind::Direct,
                        operation: Default::default(),
                        profile: None,
                        support: description.support.iter().copied().collect(),
                        limits: declaration.limits,
                        starts: vec![description.start],
                        transitions: vec![
                            Transition::Finish,
                            Transition::Continue,
                            Transition::Recover,
                            Transition::Stop,
                        ],
                    };
                    let index = collected.declaration.mechanisms.len();
                    collected.declaration.mechanisms.push(mechanism.clone());
                    // Binding preparations reserve before effects; execution admission
                    // belongs to run_admitted and must not reserve the same work twice.
                    let admitted = if preparing {
                        ledger.reserve_attempt().and_then(|()| {
                            ledger.reserve(mechanism.limits, description.reservation, false)
                        })
                    } else {
                        Ok(())
                    };
                    let preparation_dispatched =
                        preparing && admitted.is_ok() && !bound_operations.contains_key(&candidate);
                    let prepared = match admitted {
                        Ok(()) if bound_operations.contains_key(&candidate) => {
                            Ok(bound_operations.remove(&candidate).ok_or_else(|| {
                                ProblemError::Internal("selected preparation disappeared".into())
                            })?)
                        }
                        Ok(()) => {
                            self.dispatcher
                                .prepare(operation, scope.clone(), cancel)
                                .await
                        }
                        Err(error) => Err(error.into()),
                    };
                    let prepared = match prepared {
                        Ok(prepared) => prepared,
                        Err(error) => {
                            let mut identity =
                                pse_ids::FramedHasher::new(pse_ids::Frame::NumericalWorkV1);
                            identity.hash(&decision_key).str("automatic-preparation");
                            let failed_work = if preparation_dispatched {
                                Some(ledger.complete_charge(pse_model::strategy::WorkCharge {
                                    phase: pse_model::strategy::Phase::Preparation,
                                    scope: pse_model::strategy::Scope::Task,
                                    charging_owner: identity.finish_hash(),
                                    observed: pse_model::strategy::WorkObservation {
                                        attempts: 1,
                                        evaluations: None,
                                        iterations: None,
                                        factorizations: None,
                                        proof_steps: None,
                                    },
                                })?)
                            } else {
                                None
                            };
                            let observation = math::strategy::runtime_failure(&error);
                            let recoverable = !mechanism.required && matches!(observation, pse_model::generated::enums::NumericalAttemptObservation::CapabilityRefusal);
                            let cause = Arc::new(error.into_problem());
                            collected.starts.push(description.start);
                            collected.products.push(Default::default());
                            collected.events.push(math::strategy::Event {
                                mechanism: index,
                                kind: pse_model::generated::enums::NumericalEventKind::Refused,
                                phase: pse_model::strategy::Phase::Preparation,
                                original: None,
                                observation: Some(observation),
                                transition: Some(if recoverable {
                                    Transition::Continue
                                } else {
                                    Transition::Stop
                                }),
                                permission: None,
                                work: failed_work,
                                cause: Some(cause.clone()),
                                decision: Some(decision_key),
                            });
                            if recoverable {
                                attempted.insert(candidate);
                                attempted_bindings.insert(description.identity);
                                continue;
                            } else {
                                let bytes = collected.retained_bytes()?;
                                let trace =
                                    Arc::new(collected.with_owner(self.dispatcher.reserve(
                                        &self.service,
                                        "math:automatic-refusal-trace",
                                        bytes,
                                    )?));
                                return Err(MathRuntimeError::Strategy {
                                    cause: Arc::new(
                                        ProblemError::Math(pse_math::MathError::Typed {
                                            retained: cause.retained_bytes(),
                                            cause:
                                                pse_model::diagnostic::DiagnosticCause::from_shared(
                                                    cause,
                                                ),
                                        })
                                        .into(),
                                    ),
                                    trace,
                                });
                            }
                        }
                    };
                    if preparing {
                        let mut work_identity =
                            pse_ids::FramedHasher::new(pse_ids::Frame::NumericalWorkV1);
                        work_identity
                            .hash(&decision_key)
                            .str("automatic-preparation");
                        let work = pse_model::strategy::WorkCharge {
                            phase: pse_model::strategy::Phase::Preparation,
                            scope: pse_model::strategy::Scope::Task,
                            charging_owner: work_identity.finish_hash(),
                            observed: pse_model::strategy::WorkObservation {
                                attempts: 1,
                                evaluations: None,
                                iterations: None,
                                factorizations: None,
                                proof_steps: None,
                            },
                        };
                        let work = ledger.complete_charge(work)?;
                        operations[candidate].candidate.prepared = true;
                        bound_operations.insert(candidate, prepared);
                        collected.starts.push(description.start);
                        collected.products.push(math::strategy::RungProducts {
                            provider: Some(math::strategy::ProviderEvidence::NonNative),
                            ..Default::default()
                        });
                        collected.events.push(math::strategy::Event {
                            mechanism: index,
                            kind: pse_model::generated::enums::NumericalEventKind::Finished,
                            phase: pse_model::strategy::Phase::Preparation,
                            original: None,
                            observation: None,
                            transition: Some(Transition::Continue),
                            permission: None,
                            work: Some(work),
                            cause: None,
                            decision: Some(decision_key),
                        });
                        continue;
                    }
                    let assessor = assess.clone();
                    let one = self
                        .step_bound(
                            step.clone(),
                            previous.clone(),
                            attempt
                                .checked_add(index)
                                .ok_or_else(|| ProblemError::memory("automatic attempt ordinal"))?,
                            progress.clone(),
                            owner.clone(),
                            cancel,
                            move |outcome, scope, budget| {
                                match assessor.lock() {
                                    Ok(mut assess) => assess(outcome, scope, budget),
                                    Err(poisoned) => {
                                        // A poisoned scientific owner cannot be retried. The original
                                        // task propagates panic before further numerical work.
                                        let mut assess = poisoned.into_inner();
                                        let rejected = math::solves::Outcome::Rejected(Arc::new(
                                            MathRuntimeError::Panic(
                                                "scientific assessment owner panicked".into(),
                                            ),
                                        ));
                                        assess(&rejected, scope, budget)
                                    }
                                }
                            },
                            Some(prepared),
                            Some(mechanism),
                            Some(shared_products.clone()),
                            true,
                            Some(ledger.clone()),
                        )
                        .await?;
                    if let Some(actual) = one.2.declaration.mechanisms.first() {
                        collected.declaration.mechanisms[index] = actual.clone();
                    }
                    collected
                        .declaration
                        .mechanisms
                        .extend(one.2.declaration.mechanisms.iter().skip(1).cloned());
                    last_observation = math::strategy::automatic_observation(&one.2.events);
                    let report = match &one.0 {
                        math::solves::Outcome::Native(report) => Some(report.as_ref()),
                        _ => None,
                    };
                    if scope.check().is_ok()
                        && report.is_some()
                        && last_observation.as_ref().is_some_and(|last| {
                            !last
                                .original
                                .as_ref()
                                .is_some_and(math::strategy::OriginalConclusion::satisfied)
                        })
                    {
                        let more = step.automatic_operations(report, scope.cancellation())?;
                        let mut known = operations
                            .iter()
                            .map(|operation| operation.candidate.identity)
                            .collect::<std::collections::BTreeSet<_>>();
                        operations.extend(more.into_iter().filter(|operation| {
                            !attempted_bindings.contains(&operation.candidate.identity)
                                && known.insert(operation.candidate.identity)
                        }));
                    }
                    if one.2.starts.len() != one.2.declaration.mechanisms.len()
                        || one.2.products.len() != one.2.declaration.mechanisms.len()
                        || collected.starts.len() < index
                        || collected.products.len() < index
                    {
                        return Err(ProblemError::Internal(
                            "automatic trace operation slots are misaligned".into(),
                        )
                        .into());
                    }
                    // Bind actual operation slots by their declaration index. A
                    // prior binding-preparation slot remains separate from execution.
                    collected.starts.truncate(index);
                    collected.products.truncate(index);
                    collected.starts.extend(one.2.starts.iter().copied());
                    collected.products.extend(one.2.products.iter().cloned());
                    collected
                        .events
                        .extend(one.2.events.iter().cloned().map(|mut event| {
                            event.mechanism += index;
                            if event.decision.is_none() {
                                event.decision = Some(decision_key);
                            }
                            event
                        }));
                    last_result = Some(one);
                    current_assessment = true;
                }
            }
        }
        let bytes = collected.retained_bytes()?;
        let trace = Arc::new(collected.with_owner(self.dispatcher.reserve(
            &self.service,
            "math:automatic-strategy-trace",
            bytes,
        )?));
        if let Some(cause) = terminal {
            return finish_automatic_stop(
                cause,
                current_assessment,
                last_observation.as_ref(),
                last_result,
                trace,
            );
        }
        match last_result {
            Some((outcome, product, _)) => Ok((outcome, product, trace)),
            None => Err(MathRuntimeError::Strategy {
                cause: Arc::new(
                    ProblemError::Unsupported(
                        "no automatic operation satisfies the admitted task constraints".into(),
                    )
                    .into(),
                ),
                trace,
            }),
        }
    }

    /// Execute one bound solve step on the retained native state and `assess` its outcome
    /// on the same worker, charged to the session's worker share (A6). A foreign allowance
    /// the step declares is reserved on admission and retained with escaping native state;
    /// otherwise its work releases it on the session thread. The session also holds the deployment's. Native events go to
    /// `progress`: one stream per step, or one shared by an authored sequence and its run
    /// handle. `assess` returns its product, composed candidate permission and session
    /// admission. A step without both original permission and artifact admission
    /// drops the retained native state, so it cannot poison a later step; the sequence
    /// itself continues.
    ///
    /// # Errors
    /// Admission, cancellation before admission or a lost session.
    #[expect(
        clippy::too_many_arguments,
        reason = "one staged step binds its solve, predecessor, attempt, progress, lease, cancellation and assessment"
    )]
    async fn step_bound<T: Send + 'static>(
        &self,
        mut step: math::solves::PreparedSolve,
        previous: Option<math::solves::Predecessor>,
        attempt: usize,
        progress: Arc<Progress>,
        owner: Arc<pse_columnar::AllocationLease>,
        cancel: &crate::CancelSource,
        mut assess: impl FnMut(
            &math::solves::Outcome,
            Option<&pse_kernels::ExecutionScope>,
            &Arc<WorkerBudget>,
        ) -> math::strategy::Assessed<T>
        + Send
        + 'static,
        operation: Option<math::solves::PreparedRung>,
        mechanism: Option<pse_model::strategy::Mechanism>,
        shared_products: Option<math::strategy::SharedProducts>,
        original_completion: bool,
        shared_admission: Option<Arc<math::strategy::admission::TaskAdmission>>,
    ) -> Result<(math::solves::Outcome, T, math::solves::StrategyTrace), MathRuntimeError> {
        let service = self.service.clone();
        let (threads, backend) = (step.threads(), step.backend());
        let deadline = std::time::Instant::now()
            .checked_add(step.time_limit())
            .ok_or_else(|| ProblemError::Contract("task deadline extent".into()))?;
        let mut declaration = step.numerical_strategy();
        if let Some(mechanism) = mechanism {
            declaration.limits = mechanism.limits;
            declaration.mechanisms = vec![mechanism];
            declaration.branch = step.composition_request().branch;
            declaration.start.recovery = step.composition_request().recovery.clone();
            if let Some(limits) = step.composition_request().limits {
                declaration.limits = limits;
            }
        }
        let original = step.original_identity()?;
        let profile = step.strategy_profile()?;
        let start = match declaration.start.policy {
            pse_model::strategy::StartPolicy::Explicit => {
                pse_model::strategy::StartOrigin::Explicit
            }
            pse_model::strategy::StartPolicy::PreviousAccepted if previous.is_some() => {
                pse_model::strategy::StartOrigin::Accepted
            }
            _ => pse_model::strategy::StartOrigin::Specification,
        };

        let mut declared_foreign = step.declared_foreign_bytes();
        let composition = step.take_composition();
        use math::solves::PreparedRung;
        let paired_correction = original_completion
            && operation.as_ref().is_some_and(|rung| match rung {
                PreparedRung::Path { .. } | PreparedRung::Surrogate(_) => true,
                #[cfg(feature = "solver-petsc")]
                PreparedRung::Petsc(_) => true,
                _ => false,
            });
        let mut rungs = operation.map_or_else(
            || composition.map_or_else(|| vec![step.clone().into()], |c| c.rungs.clone()),
            |operation| vec![operation],
        );
        if paired_correction {
            rungs.push(PreparedRung::Original(Box::new(step.clone())));
            let mut correction = step.numerical_strategy().mechanisms[0].clone();
            correction.required = true;
            correction.limits = declaration.limits;
            correction.starts = declaration.start.recovery.clone();
            declaration.mechanisms.push(correction);
        }
        for (rung, mechanism) in rungs.iter().zip(&mut declaration.mechanisms) {
            declared_foreign = declared_foreign.max(rung.declared_foreign_bytes());
            if mechanism.profile.is_none() {
                mechanism.profile = rung
                    .backend()
                    .map(|backend| {
                        rung.strategy_profile()
                            .map(|key| pse_model::strategy::ProfileRef { backend, key })
                    })
                    .transpose()?;
            }
            let supplied = rung.operation_contract()?;
            for input in supplied.inputs {
                if !mechanism.operation.inputs.contains(&input) {
                    mechanism.operation.inputs.push(input);
                }
            }
            for output in supplied.outputs {
                if !mechanism.operation.outputs.contains(&output) {
                    mechanism.operation.outputs.push(output);
                }
            }
        }
        let start = rungs
            .first()
            .map_or(start, |rung| rung.entry_origin(previous.is_some()));
        let inherited = start == pse_model::strategy::StartOrigin::Accepted;
        let backends = rungs.iter().flat_map(PreparedRung::backends).collect();
        let threads = rungs
            .iter()
            .map(PreparedRung::threads)
            .max()
            .unwrap_or(threads);
        let task_scope = step
            .task_scope()
            .or_else(|| rungs.iter().find_map(PreparedRung::task_scope));
        let deadline = task_scope
            .as_ref()
            .and_then(pse_kernels::ExecutionScope::deadline)
            .map_or(deadline, |outer| outer.min(deadline));
        let products = rungs
            .iter()
            .map(|rung| math::strategy::RungProducts {
                provider: match rung {
                    PreparedRung::Surrogate(p) => {
                        Some(math::strategy::ProviderEvidence::Library(p.key()))
                    }
                    PreparedRung::Derived(_) | PreparedRung::Original(_)
                        if rung.backend().is_none() =>
                    {
                        Some(math::strategy::ProviderEvidence::NonNative)
                    }
                    _ => None,
                },
                derived: match rung {
                    PreparedRung::Multistart(p) => Some(p.key()),
                    PreparedRung::Surrogate(p) => Some(p.key()),
                    PreparedRung::Derived(p) => Some(p.family().key()),
                    #[cfg(feature = "solver-kinsol")]
                    PreparedRung::Blocks(p) => Some(p.key()),
                    #[cfg(feature = "solver-kinsol")]
                    PreparedRung::Causal(p) => Some(p.key()),
                    PreparedRung::Path { prepared, .. } => Some(prepared.family().key()),
                    #[cfg(feature = "solver-petsc")]
                    PreparedRung::Petsc(p) => Some(p.key()),
                    PreparedRung::Original(_) => None,
                },
                ..Default::default()
            })
            .collect::<Vec<_>>();
        let supports = rungs
            .iter()
            .map(PreparedRung::support)
            .collect::<Result<Vec<_>, _>>()?;

        let starts: Vec<_> = rungs
            .iter()
            .enumerate()
            .map(|(index, rung)| {
                if index == 0 {
                    start
                } else {
                    rung.entry_origin(false)
                }
            })
            .collect();
        let initial = if rungs
            .iter()
            .any(|rung| !matches!(rung, PreparedRung::Original(_) | PreparedRung::Path { .. }))
        {
            Some(step.source_start(previous.as_ref())?)
        } else {
            None
        };
        let product_owner = self.dispatcher.reserve(
            &service,
            "math:actual-product-state",
            math::strategy::ProductState::extent(
                rungs
                    .len()
                    .checked_mul(3)
                    .ok_or_else(|| ProblemError::memory("rung product state extent"))?,
            )?,
        )?;
        let allowance = self
            .dispatcher
            .reserve(&service, "math:step-foreign", declared_foreign)?;
        let product_capacity = rungs
            .len()
            .checked_mul(3)
            .ok_or_else(|| ProblemError::memory("rung product state extent"))?;
        let shared_admission = shared_admission.or_else(|| step.task_admission());
        self.dispatcher.run_task(threads, backends, Some(deadline), task_scope.clone(), cancel, move |retained, flag, budget| {
            enum RungOutcome { Original(Box<math::solves::ScopedOutcome>), Derived(Box<math::solves::DerivedAttempt>), Path(Box<math::solves::paths::PathOutcome>), Surrogate(Box<math::surrogate::SurrogatePhaseOutcome>) }
            let _allowance = allowance;
            // run_task has acquired this exact active team before entering the
            // closure. Nested provider workers reset the capability to one thread.
            let admitted_threads=std::num::NonZeroUsize::new(threads)
                .ok_or_else(||ProblemError::Contract("zero admitted native team".into()))?;
            let enclosing = pse_kernels::ExecutionScope::new(flag.clone(), Some(deadline))
                .with_admitted_threads(admitted_threads);
            let admission=shared_admission.unwrap_or_else(||math::strategy::admission::TaskAdmission::new_for(declaration.limits,enclosing.clone(),&service,declared_foreign>0));
            if !admission.matches_scope(&enclosing) { return Err(ProblemError::Contract("bound operation differs from its task admission scope".into()).into()); }
            admission.retain_storage_allowance(_allowance.clone())?;
            let scoped_budget=budget.with_admission(admission.clone());
            let budget=&scoped_budget;
            let mut assessed = None;
            let product_state=shared_products.unwrap_or_else(||Arc::new(std::sync::Mutex::new(math::strategy::ProductState::owned(product_capacity,product_owner))));
            let prepared=std::cell::RefCell::new(rungs.into_iter().map(Some).collect::<Vec<_>>());
            let mut previous = previous;
            let point=std::cell::RefCell::new(initial);
            let fresh_start=std::cell::Cell::new(None::<pse_model::strategy::StartOrigin>);
            let screened_start=std::cell::RefCell::new(None::<math::prediction::Screened>);
            let pending_screened=std::cell::RefCell::new(None::<math::prediction::Screened>);
            let connected=std::cell::Cell::new(false);
            let completion_witness=std::cell::RefCell::new(None::<math::solves::paths::PathCompletion>);
            let actual_starts=std::cell::RefCell::new(starts);
            let actual_products=std::cell::RefCell::new(products);
            // Execute and assess run serially; repeated same-case assessments
            // retain the actual dispatched mechanism as part of their occurrence.
            let dispatched_mechanism=std::cell::Cell::new(0usize);
            let result = math::strategy::run_admitted(
                &declaration, &enclosing, admission.clone(),
                |index| {
                    let declared=actual_starts.borrow()[index];
                    let fresh=fresh_start.get().filter(|origin|index>0 && !matches!(prepared.borrow()[index].as_ref(),Some(PreparedRung::Multistart(_))) && declared!=pse_model::strategy::StartOrigin::Explicit
                        && declaration.start.permits_recovery(*origin,false)
                        && declaration.mechanisms[index].starts.contains(origin));
                    let actual=fresh.unwrap_or(declared);
                    actual_starts.borrow_mut()[index]=actual;
                    let origin=prepared.borrow()[index].as_ref().and_then(|rung|match rung {PreparedRung::Path{prepared,start}=>Some(prepared.origin_connected(start)),_=>None});
                    let (bound,refusal)=match origin {Some(Ok(bound))=>(bound,None),Some(Err(error))=>(false,Some(Arc::new(error))),None=>(connected.get(),None)};
                    let contract = &declaration.mechanisms[index].operation;
                    let consumed = math::strategy::products(&product_state).and_then(|state| state.consume(contract));
                    let (accuracy,accuracy_refusal)=match consumed {Ok(accuracy)=>(accuracy,None),Err(error)=>(Vec::new(),Some(Arc::new(error)))};
                    let local_refusal=prepared.borrow()[index].as_ref().and_then(|rung|match rung {PreparedRung::Derived(p)=>p.local_scope_refusal(&enclosing),_=>None});
                    let work_admitted=prepared.borrow()[index].as_ref().is_some_and(|rung|match rung {PreparedRung::Original(target)=>target.work_admitted(declaration.limits,&enclosing,admission.clone()) && target.work_admitted(declaration.mechanisms[index].limits,&enclosing,admission.clone()),_=>false});
                    math::strategy::Facts { support:supports[index].clone(), accuracy, consumption:contract.inputs.iter().map(|input|input.accuracy).collect(), reservation:None, work_admitted, start:actual, inherited:index==0 && inherited, connected:bound, refusal:refusal.or(local_refusal).or(accuracy_refusal) }
                },
                |index,mechanism| {
                    dispatched_mechanism.set(index);
                    *pending_screened.borrow_mut()=None;
                    let rung = prepared.borrow_mut()[index].take().ok_or_else(|| Arc::new(ProblemError::Internal("strategy mechanism executed twice".into())))?;
                    let input=match &rung {
                        PreparedRung::Original(p)=>if matches!(actual_starts.borrow()[index],pse_model::strategy::StartOrigin::Auxiliary|pse_model::strategy::StartOrigin::Surrogate) {point.borrow().clone().or_else(||p.source_start(previous.as_ref()).ok())} else {p.source_start(previous.as_ref()).ok()},
                        PreparedRung::Multistart(p)=>Some(p.point()),
                        PreparedRung::Surrogate(_)=>point.borrow().clone(),
                        PreparedRung::Derived(p)=>if Some(actual_starts.borrow()[index])==fresh_start.get() {point.borrow().clone()} else {Some(p.original().source_start(previous.as_ref()).map_err(Arc::new)?)},
                        #[cfg(feature="solver-kinsol")]
                        PreparedRung::Blocks(p)=>Some(p.original().source_start(previous.as_ref()).map_err(Arc::new)?),
                        #[cfg(feature="solver-kinsol")]
                        PreparedRung::Causal(p)=>Some(p.original().source_start(previous.as_ref()).map_err(Arc::new)?),
                        PreparedRung::Path{start,..}=>Some(start.point().to_vec()),
                        #[cfg(feature="solver-petsc")] PreparedRung::Petsc(p)=>if Some(actual_starts.borrow()[index])==fresh_start.get() {point.borrow().clone()} else {Some(p.source().original().source_start(previous.as_ref()).map_err(Arc::new)?)},
                    };
                    let actual_input=input.clone();
                    if let Some(input)=input {
                        let mut identity=pse_ids::FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
                        identity.str("actual-original-coordinate-start").hash(&original);
                        for x in input {identity.f64(x);}
                        actual_products.borrow_mut()[index].start=Some(identity.finish_hash());
                    }
                    let (value,observation,observed)=match rung {
                        PreparedRung::Original(mut prepared_step)=>{
                            prepared_step.inherit_point_accuracy(&step);
                            let step = prepared_step;
                            let mut step=*step;
                            if matches!(actual_starts.borrow()[index],pse_model::strategy::StartOrigin::Auxiliary|pse_model::strategy::StartOrigin::Surrogate) {
                                if let Some(screened)=screened_start.borrow().as_ref() {
                                    let branch=completion_witness.borrow().as_ref().map_or(Ok(declaration.branch),|witness|witness.recovery_branch(declaration.branch,screened.proposal())).map_err(Arc::new)?;
                                    step=step.within_admitted_task(enclosing.clone(),admission.clone()).and_then(|step|step.with_composed_recovery_start(screened,&declaration.start,branch)).map_err(Arc::new)?;
                                } else if !step.has_screened_start() {
                                    return Err(Arc::new(ProblemError::Contract("original correction requires the actual screened producer receipt".into())).into());
                                }
                            }
                            if let Some(backend)=step.backend() {
                                actual_products.borrow_mut()[index].provider=Some(math::strategy::ProviderEvidence::Native(pse_model::strategy::ProfileRef {backend,key:step.strategy_profile().map_err(Arc::new)?}));
                            }
                            let target=step.clone();
                            let mut value = service.execute(step, previous.take(), attempt, retained, flag, &progress, budget, &owner, &enclosing)
                                .unwrap_or_else(|error| (math::solves::Outcome::Rejected(Arc::new(error)), Some(enclosing.clone())));
                            if let Some(witness)=completion_witness.borrow().as_ref()
                                && let math::solves::Outcome::Native(report)=&mut value.0
                                && let Some(candidate)=&report.candidate {
                                let execution=pse_backend_native::solve::Execution::within(flag.clone(),&pse_backend_native::solve::Controls::default(),enclosing.clone()).map_err(Arc::new)?;
                                if let Err(cause)=witness.validate_original_candidate(&target,&candidate.primal,&execution) {
                                    report.record_validation_failure(cause);
                                    let failure_owner=service.reserve("math:path-completion-failure",report.failure_bytes()).map_err(|error|Arc::new(error.into_problem()))?;
                                    report.retain_failure_owner(failure_owner);
                                }
                            }
                            let observation=math::strategy::observe(&value.0);
                            let observed=target.inclusive_work(&value.0);
                            (RungOutcome::Original(Box::new(value)),observation,observed)
                        },
                        #[cfg(feature="solver-kinsol")]
                        PreparedRung::Blocks(prepared)=>{
                            let mut execution=pse_backend_native::solve::Execution::within(flag.clone(),prepared.original().controls(),enclosing.clone()).map_err(Arc::new)?;
                            execution.work_admission=budget.admission().map(|owner|->Arc<dyn pse_backend_native::solve::WorkAdmission>{owner});
                            execution.progress=progress.clone();execution.memory=Some(service.policy.foreign_allowance(prepared.original().controls()));
                            let outcome=service.blocks_step(&prepared,execution,retained,budget,actual_input.as_deref().ok_or_else(||Arc::new(ProblemError::Internal("complete original block start absent".into())))?)?;
                            let observed=math::strategy::work(&outcome);let observation=math::strategy::observe(&outcome);
                            (RungOutcome::Original(Box::new((outcome,Some(enclosing.clone())))),observation,observed)
                        },
                        #[cfg(feature="solver-kinsol")]
                        PreparedRung::Causal(prepared)=>{
                            let mut execution=pse_backend_native::solve::Execution::within(flag.clone(),prepared.original().controls(),enclosing.clone()).map_err(Arc::new)?;
                            execution.work_admission=budget.admission().map(|owner|->Arc<dyn pse_backend_native::solve::WorkAdmission>{owner});
                            execution.progress=progress.clone();execution.memory=Some(service.policy.foreign_allowance(prepared.original().controls()));
                            let outcome=service.causal_step(&prepared,execution,retained,budget,actual_input.as_deref().ok_or_else(||Arc::new(ProblemError::Internal("complete original causal start absent".into())))?)?;
                            let observed=math::strategy::work(&outcome);let observation=math::strategy::observe(&outcome);
                            (RungOutcome::Original(Box::new((outcome,Some(enclosing.clone())))),observation,observed)
                        },
                        PreparedRung::Derived(prepared)=>{
                            let mut execution=pse_backend_native::solve::Execution::within(flag.clone(),prepared.controls(),enclosing.clone()).map_err(Arc::new)?;
                            execution.work_admission=budget.admission().map(|owner|->Arc<dyn pse_backend_native::solve::WorkAdmission>{owner});execution.progress=progress.clone();
                            execution.memory=Some(service.policy.foreign_allowance(prepared.controls()));
                            let mut value=match service.derived_step(&prepared,execution.clone(),retained,budget,actual_input.as_deref().ok_or_else(||Arc::new(ProblemError::Internal("original source point absent".into())))?) {
                                Ok(value)=>value,
                                Err(error)=>{
                                    retained.clear();
                                    return Err(error.effect(&prepared,&enclosing));
                                }
                            };
                            if let Some(failed)=prepared.local_attempt_failure(&value,&enclosing) {retained.clear();return Err(failed);}
                            if !original_completion && let Some(proposal)=&value.proposal {
                                    let mut evaluations=0;
                                    match service.screen_auxiliary_start_worker_observed(prepared.original(),proposal,declaration.branch,enclosing.clone(),budget,&mut evaluations) {
                                        Ok(screened)=>*pending_screened.borrow_mut()=Some(screened),
                                        Err(error)=>{value.screening_failure=Some(Arc::new(error.into_problem()));},
                                    }
                                    if let math::solves::Outcome::Native(report)=&mut value.outcome {report.evidence.work.evaluations=report.evidence.work.evaluations.and_then(|n|n.checked_add(evaluations));}
                            }
                            let evidence=prepared.produced_evidence(&value,declaration.branch).map_err(|cause|math::strategy::EffectFailure::component(Arc::new(cause),value.work(),math::strategy::observe(&value.outcome)))?;
                            for product in &evidence {
                                if product.derivative_order == 0 {actual_products.borrow_mut()[index].accuracy=Some(product.accuracy);}
                                actual_products.borrow_mut()[index].evidence.push(product.clone());
                            }
                            math::strategy::validate_outputs(&mechanism.operation,&evidence).map_err(|cause|math::strategy::EffectFailure::component(value.screening_failure.clone().or_else(||math::strategy::cause(&value.outcome)).unwrap_or_else(||Arc::new(cause)),value.work(),math::strategy::observe(&value.outcome)))?;
                            for product in evidence {
                                math::strategy::products(&product_state).and_then(|mut state|state.publish(product)).map_err(Arc::new)?;
                            }
                            if original_completion && value.proposal.is_some() {
                                let outcome=service.derived_original_outcome(&prepared,&value,&execution,budget).map_err(|error|Arc::new(error.into_problem()))?;
                                let observed=math::strategy::work(&outcome);
                                let observation=math::strategy::observe(&outcome);
                                (RungOutcome::Original(Box::new((outcome,Some(enclosing.clone())))),observation,observed)
                            } else {
                                let observed=value.work();
                                let observation=math::strategy::observe(&value.outcome);
                                (RungOutcome::Derived(Box::new(value)),observation,observed)
                            }
                        },
                        PreparedRung::Multistart(prepared)=>{
                            let mut screening_work=0;
                            let math::solves::multistart::BoundMultistart {step,_bindings,screening_evaluations}=service.screen_composed_multistart_worker_observed(&prepared,enclosing.clone(),budget,&mut screening_work,admission.clone(),&declaration).map_err(|error|math::strategy::EffectFailure::observed(Arc::new(error.into_problem()),pse_model::strategy::WorkObservation {attempts:1,evaluations:Some(screening_work),iterations:None,factorizations:None,proof_steps:None}))?;
                            let value=service.execute(step,None,attempt,retained,flag,&progress,budget,&owner,&enclosing)
                                .unwrap_or_else(|error|(math::solves::Outcome::Rejected(Arc::new(error)),Some(enclosing.clone())));
                            let observation=math::strategy::observe(&value.0);
                            let mut observed=math::strategy::work(&value.0);
                            observed.evaluations=observed.evaluations.and_then(|n|n.checked_add(screening_evaluations));
                            (RungOutcome::Original(Box::new(value)),observation,observed)
                        },
                        PreparedRung::Surrogate(prepared)=>{
                            let mut execution=pse_backend_native::solve::Execution::within(flag.clone(),&pse_backend_native::solve::Controls::default(),enclosing.clone()).map_err(Arc::new)?;
                            let phase_budget=WorkerBudget::drawing_for(prepared.worker_bytes().max(service.policy.worker_bytes),&service);
                            execution.work_admission=budget.admission().map(|owner|->Arc<dyn pse_backend_native::solve::WorkAdmission>{owner});
                            let phase_budget=phase_budget.with_admission(admission.clone());
                            let value=prepared.advance(&execution,&phase_budget,owner.clone());
                            actual_products.borrow_mut()[index].statistical=value.statistical.clone();
                            if let Some(screened)=&value.screened {*pending_screened.borrow_mut()=Some(screened.clone());}
                            let observed=value.work;
                            let observation=value.terminal.as_deref().map_or(pse_model::generated::enums::NumericalAttemptObservation::Auxiliary,math::strategy::failure);
                            (RungOutcome::Surrogate(Box::new(value)),observation,observed)
                        },
                        PreparedRung::Path{prepared,start}=>{
                            let mut execution=pse_backend_native::solve::Execution::within(flag.clone(),&prepared.profile().controls,enclosing.clone()).map_err(Arc::new)?;
                            execution.work_admission=budget.admission().map(|owner|->Arc<dyn pse_backend_native::solve::WorkAdmission>{owner});execution.progress=progress.clone();execution.memory=Some(service.policy.foreign_allowance(&prepared.profile().controls));
                            let path_budget=WorkerBudget::drawing_for(prepared.worker_bytes().max(service.policy.worker_bytes),&service);
                            let path_budget=path_budget.with_admission(admission.clone());
                            let mut value=math::solves::paths::run_path(&service,&prepared,*start,execution,&path_budget,owner.clone());
                            let mut observed=value.work();
                            if let Some(proposal)=value.proposal.take() {
                                let mut screening_evaluations=0;
                                match service.screen_start_worker_observed(prepared.original_target(),proposal.clone(),proposal.branch(),enclosing.clone(),budget,&mut screening_evaluations) {
                                    Ok(screened)=>{value.proposal=Some(screened.proposal().clone());*pending_screened.borrow_mut()=Some(screened);},
                                    Err(error)=>value.terminal=Some(Arc::new(error.into_problem())),
                                }
                                observed.evaluations=observed.evaluations.and_then(|n|n.checked_add(screening_evaluations));
                            }
                            actual_products.borrow_mut()[index].path_events=Some(value.events.clone());
                            actual_products.borrow_mut()[index].transport=value.connected.map(|proof|proof.transport);
                            let observation=value.terminal.as_deref().map_or(pse_model::generated::enums::NumericalAttemptObservation::Auxiliary,math::strategy::failure);
                            (RungOutcome::Path(Box::new(value)),observation,observed)
                        },
                        #[cfg(feature="solver-petsc")]
                        PreparedRung::Petsc(prepared)=>{
                            let mut execution=pse_backend_native::solve::Execution::within(flag.clone(),prepared.controls(),enclosing.clone()).map_err(Arc::new)?;
                            execution.work_admission=budget.admission().map(|owner|->Arc<dyn pse_backend_native::solve::WorkAdmission>{owner});execution.progress=progress.clone();execution.memory=Some(service.policy.foreign_allowance(prepared.controls()));
                            let mut value=service.execute_petsc(&prepared,execution,budget,actual_input.as_deref().ok_or_else(||Arc::new(ProblemError::internal("original PETSc start missing")))?).map_err(|error|Arc::new(error.into_problem()))?;
                            let mut observed=value.work();
                            if let Some(proposal)=&value.proposal {
                                let mut evaluations=0;
                                match service.screen_auxiliary_start_worker_observed(prepared.source().original(),proposal,declaration.branch,enclosing.clone(),budget,&mut evaluations) {
                                    Ok(screened)=>*pending_screened.borrow_mut()=Some(screened),
                                    Err(error)=>{value.screening_failure=Some(Arc::new(error.into_problem()));},
                                }
                                observed.evaluations=observed.evaluations.and_then(|n|n.checked_add(evaluations));
                            }
                            let observation=math::strategy::observe(&value.outcome);
                            (RungOutcome::Derived(Box::new(value)),observation,observed)
                        },
                    };
                    let evidence=actual_products.borrow()[index].evidence.clone();
                    math::strategy::validate_outputs(&mechanism.operation,&evidence).map_err(|cause| {
                        let actual_cause=match &value {
                            RungOutcome::Original(value)=>math::strategy::cause(&value.0),
                            RungOutcome::Derived(value)=>value.screening_failure.clone().or_else(||math::strategy::cause(&value.outcome)),
                            RungOutcome::Surrogate(value)=>value.terminal.clone(),
                            RungOutcome::Path(value)=>value.terminal.clone(),
                        };
                        math::strategy::EffectFailure::component(actual_cause.unwrap_or_else(||Arc::new(cause)),observed,observation)
                    })?;
                    let mut charge=pse_ids::FramedHasher::new(pse_ids::Frame::NumericalWorkV1);
                    charge.hash(&original).hash(&profile).u64(attempt as u64).u64(index as u64);
                    let phase=if mechanism.position==pse_model::strategy::Position::Preparation {pse_model::strategy::Phase::Preparation} else {pse_model::strategy::Phase::Native};
                    Ok(math::strategy::Attempt { value, observation, evidence, work:pse_model::strategy::WorkCharge {phase, scope:pse_model::strategy::Scope::Task, charging_owner:charge.finish_hash(), observed } })
                },
                |value,observation| match value {
                    RungOutcome::Original(value)=>{
                        let (outcome,scope)=value.as_ref();
                        let mut assessment = assess(outcome,scope.as_ref(),budget);
                        for charge in &mut assessment.work {
                            let mut occurrence=pse_ids::FramedHasher::new(pse_ids::Frame::NumericalWorkV1);
                            occurrence.hash(&charge.charging_owner).hash(&profile).u64(attempt as u64).u64(dispatched_mechanism.get() as u64);
                            charge.charging_owner=occurrence.finish_hash();
                        }
                        assessed = Some(assessment.product);
                        math::strategy::Assessment { auxiliary:false, retention:assessment.retention, original:assessment.original, work:assessment.work, observation, cause:math::strategy::cause(outcome) }
                    },
                    RungOutcome::Derived(value)=>{
                        let cause=value.screening_failure.clone().or_else(||math::strategy::cause(&value.outcome));
                        let screened=value.screening_failure.is_none() && pending_screened.borrow().is_some();
                        if screened && let Some(proposal)=&value.proposal {
                            *screened_start.borrow_mut()=pending_screened.borrow_mut().take();
                            *point.borrow_mut()=Some(proposal.coordinates.clone());
                            fresh_start.set(Some(pse_model::strategy::StartOrigin::Auxiliary));
                        }
                        let decision=crate::workflow::numerics::auxiliary_start(screened && value.proposal.is_some());
                        let observation=cause.as_deref().map_or_else(||if screened && value.proposal.is_some() {pse_model::generated::enums::NumericalAttemptObservation::Auxiliary} else {observation},math::strategy::failure);
                        math::strategy::Assessment {auxiliary:true,original:math::strategy::OriginalConclusion::Unavailable {cause:Arc::new(ProblemError::Unsupported("auxiliary result requires original correction".into()))},work:Vec::new(),retention:StepRetention {candidate:decision,session:SessionDisposition::Discard},observation,cause}
                    },
                    RungOutcome::Surrogate(value)=>{
                        if let Some(statistical)=&value.statistical {
                            let product_index=actual_products.borrow().iter().position(|p|matches!(p.provider,Some(math::strategy::ProviderEvidence::Library(_))) && p.statistical.is_none());
                            if let Some(index)=product_index {actual_products.borrow_mut()[index].statistical=Some(statistical.clone());}
                        }
                        if let Some(proposal)=&value.proposal {
                            *screened_start.borrow_mut()=pending_screened.borrow_mut().take();
                            *point.borrow_mut()=Some(proposal.values().map(|(_,v)|v).collect());
                            fresh_start.set(Some(pse_model::strategy::StartOrigin::Surrogate));
                        }
                        let observation=if value.callback_terminal && math::strategy::permits_numerical_continuation(observation) {pse_model::generated::enums::NumericalAttemptObservation::ContractFailure} else {observation};
                        math::strategy::Assessment {auxiliary:true,original:math::strategy::OriginalConclusion::Unavailable {cause:Arc::new(ProblemError::Unsupported("auxiliary result requires original correction".into()))},work:Vec::new(),retention:StepRetention {candidate:crate::workflow::numerics::auxiliary_start(value.proposal.is_some() && !value.callback_terminal),session:SessionDisposition::Discard},observation,cause:value.terminal.clone()}
                    },
                    RungOutcome::Path(value)=>{
                        if let Some(event)=value.events.first() {
                            let product_index=actual_products.borrow().iter().position(|p|p.derived==Some(event.family));
                            if let Some(index)=product_index {actual_products.borrow_mut()[index].path_events=Some(value.events.clone());}
                        }
                        if let Some(proposal)=&value.proposal {
                            *screened_start.borrow_mut()=pending_screened.borrow_mut().take();
                            *completion_witness.borrow_mut()=value.completion_witness();
                            connected.set(completion_witness.borrow().is_some());
                            *point.borrow_mut()=Some(proposal.values().map(|(_,v)|v).collect());fresh_start.set(Some(pse_model::strategy::StartOrigin::Auxiliary));
                            let product_index=actual_products.borrow().iter().position(|p|p.derived==proposal.source().derivation);
                            if let Some(index)=product_index {actual_products.borrow_mut()[index].transport=value.connected.map(|p|p.transport);}
                        }
                        math::strategy::Assessment {auxiliary:true,original:math::strategy::OriginalConclusion::Unavailable {cause:Arc::new(ProblemError::Unsupported("auxiliary result requires original correction".into()))},work:Vec::new(),retention:StepRetention {candidate:crate::workflow::numerics::auxiliary_start(value.proposal.is_some()),session:SessionDisposition::Discard},observation,cause:value.terminal.clone()}
                    },
                },
            );
            let trace = math::strategy::Trace { owner:None, publication_request:None, declaration, original, backend, profile, start, starts:actual_starts.into_inner(), products:actual_products.into_inner(), events:result.events };
            let bytes=trace.retained_bytes()?;
            let trace=Arc::new(trace.with_owner(service.reserve("math:strategy-trace",bytes)?));
            match (result.value, assessed, result.terminal) {
                (Some(RungOutcome::Original(value)), Some(product), None) => {
                    let (outcome,_)=*value;
                    if let Some(assessment) = result.assessment { assessment.retention.apply(retained); }
                    retained.charge_session(Box::new(_allowance.clone()));
                    Ok((outcome,product,trace))
                }
                (_, _, terminal) => {
                    retained.clear();
                    let error = terminal.map_or_else(|| ProblemError::Internal("strategy ended without an assessed original".into()), |error| ProblemError::Math(pse_math::MathError::Typed {retained:error.retained_bytes(),cause:pse_model::diagnostic::DiagnosticCause::from_shared(error)}));
                    let outcome = math::solves::Outcome::Rejected(Arc::new(error.into()));
                    let product = assess(&outcome,Some(&enclosing),budget).product;
                    Ok((outcome,product,trace))
                }
            }
        })
        .await
    }
}

/// One thread-confined strategy dispatcher for a complete admitted serial operation.
/// Native state belongs to this scope until the sequence finishes or fails.
#[cfg(any(test, feature = "solver-kinsol"))]
pub(crate) struct LocalOperation<'a> {
    service: Arc<MathService>,
    retained: std::cell::RefCell<&'a mut Retained>,
    context: std::cell::RefCell<&'a mut OperationContext>,
    flag: Arc<AtomicBool>,
    budget: Arc<WorkerBudget>,
    execution: pse_backend_native::solve::Execution,
    scopes: std::cell::RefCell<execution::SerialScopes>,
}
#[cfg(any(test, feature = "solver-kinsol"))]
impl<'a> LocalOperation<'a> {
    pub(super) fn new(
        service: Arc<MathService>,
        retained: &'a mut Retained,
        context: &'a mut OperationContext,
        flag: Arc<AtomicBool>,
        budget: Arc<WorkerBudget>,
        scope: pse_kernels::ExecutionScope,
        controls: pse_backend_native::solve::Controls,
    ) -> Result<Self, MathRuntimeError> {
        let execution =
            pse_backend_native::solve::Execution::within(flag.clone(), &controls, scope)?;
        Ok(Self {
            service,
            retained: std::cell::RefCell::new(retained),
            context: std::cell::RefCell::new(context),
            flag,
            budget,
            execution,
            scopes: Default::default(),
        })
    }
    #[cfg(any(feature = "solver-kinsol", all(test, feature = "solver-ipopt")))]
    pub(crate) fn run_ready<T>(
        &self,
        work: impl Future<Output = Result<T, MathRuntimeError>>,
    ) -> Result<T, MathRuntimeError> {
        // Both local effect seams complete and drain synchronously before returning
        // Ready. Only their genuine quiescent child boundary enters the async runtime.
        futures_util::FutureExt::now_or_never(work).ok_or(MathRuntimeError::Infrastructure(
            "local strategy attempted to escape its synchronous owner".into(),
        ))?
    }
    #[cfg(any(feature = "solver-kinsol", all(test, feature = "solver-ipopt")))]
    pub(crate) fn execution(&self) -> &pse_backend_native::solve::Execution {
        &self.execution
    }
    #[cfg(feature = "solver-kinsol")]
    pub(crate) fn construct<T>(
        &self,
        bytes: usize,
        work: impl FnOnce(Arc<pse_columnar::AllocationLease>) -> Result<T, MathRuntimeError>,
    ) -> Result<T, MathRuntimeError> {
        self.construct_estimated(math::jobs::ConstructionEstimate::Known(bytes), work)
    }
    #[cfg(feature = "solver-kinsol")]
    pub(crate) fn construct_estimated<T>(
        &self,
        construction: math::jobs::ConstructionEstimate,
        work: impl FnOnce(Arc<pse_columnar::AllocationLease>) -> Result<T, MathRuntimeError>,
    ) -> Result<T, MathRuntimeError> {
        let bytes = construction.bytes();
        if bytes > self.service.policy.worker_bytes {
            return Err(MathRuntimeError::Limit("local constructor worker capacity"));
        }
        let runtime = self.context.borrow().runtime.clone();
        let _entered = runtime.enter();
        self.service.quiescent_phase_owned_construction(
            1,
            construction,
            &self.execution,
            |reservation| work(pse_columnar::AllocationLease::new(reservation.split(bytes))),
        )?
    }
    fn reserved<T>(
        &self,
        bytes: usize,
        work: impl FnOnce(Arc<pse_columnar::AllocationLease>) -> Result<T, MathRuntimeError>,
    ) -> Result<T, MathRuntimeError> {
        let runtime = self.context.borrow().runtime.clone();
        let _entered = runtime.enter();
        self.service
            .quiescent_phase_owned(1, bytes, &self.execution, |reservation| {
                work(pse_columnar::AllocationLease::new(reservation.split(bytes)))
            })?
    }
    /// Directly awaited preparation borrows this exact lane after releasing active
    /// resources. Its actual children drain before the exclusive operation use returns.
    pub(crate) fn preparation<T>(
        &self,
        work: impl Future<Output = Result<T, MathRuntimeError>>,
    ) -> Result<T, MathRuntimeError> {
        self.execution.check()?;
        execution::pause_compute();
        let (lane, runtime) = {
            let mut context = self.context.borrow_mut();
            let lane = context.lane.clone();
            let runtime = context.runtime.clone();
            drop(context._usage.take());
            (lane, runtime)
        };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            runtime.block_on(math::jobs::population_scope(lane.clone(), work))
        }));
        runtime.block_on(lane.drain());
        self.context.borrow_mut()._usage = Some(lane.claim()?);
        let resumed = execution::resume_compute(&self.execution);
        match result {
            Ok(Err(error)) => Err(error),
            Ok(Ok(value)) => {
                resumed?;
                Ok(value)
            }
            Err(_) => Err(MathRuntimeError::Panic(
                "local strategy preparation panic".into(),
            )),
        }
    }
    #[cfg(feature = "solver-kinsol")]
    #[allow(
        clippy::too_many_arguments,
        reason = "the local step uses the same explicit predecessor, progress, allocation, cancellation and original assessment protocol as the staged dispatcher"
    )]
    pub(crate) async fn step<T: Send + 'static>(
        &self,
        step: math::solves::PreparedSolve,
        previous: Option<math::solves::Predecessor>,
        attempt: usize,
        progress: Arc<Progress>,
        owner: Arc<pse_columnar::AllocationLease>,
        cancel: &crate::CancelSource,
        assess: impl FnMut(
            &math::solves::Outcome,
            Option<&pse_kernels::ExecutionScope>,
            &Arc<WorkerBudget>,
        ) -> math::strategy::Assessed<T>
        + Send
        + 'static,
    ) -> Result<(math::solves::Outcome, T, math::solves::StrategyTrace), MathRuntimeError> {
        StepDriver {
            service: self.service.clone(),
            dispatcher: self,
        }
        .step(step, previous, attempt, progress, owner, cancel, assess)
        .await
    }
}
#[cfg(any(test, feature = "solver-kinsol"))]
impl Drop for LocalOperation<'_> {
    fn drop(&mut self) {
        let retained = self.retained.get_mut();
        retained.clear();
        retained.release();
        // `scopes` drop after native callbacks and their storage owners.
    }
}
#[cfg(any(test, feature = "solver-kinsol"))]
impl Dispatch for LocalOperation<'_> {
    fn reserve(
        &self,
        _service: &MathService,
        _name: &str,
        bytes: usize,
    ) -> Result<Arc<pse_columnar::AllocationLease>, MathRuntimeError> {
        self.reserved(bytes, Ok)
    }
    fn run_task<'a, T: Send + 'static>(
        &'a self,
        cores: usize,
        backends: Vec<Backend>,
        deadline: Option<std::time::Instant>,
        scope: Option<pse_kernels::ExecutionScope>,
        cancel: &'a crate::CancelSource,
        work: impl FnOnce(
            &mut Retained,
            &Arc<AtomicBool>,
            &Arc<WorkerBudget>,
        ) -> Result<T, MathRuntimeError>
        + Send
        + 'static,
    ) -> Pending<'a, T> {
        let result = (|| {
            self.execution.check()?;
            cancel
                .checkpoint()
                .map_err(|_| MathRuntimeError::Cancelled)?;
            let cutoff = self.execution.scope()?.deadline();
            let extends_clock = |deadline: Option<std::time::Instant>| {
                cutoff.is_some_and(|outer| deadline.is_none_or(|deadline| deadline > outer))
            };
            if cores != 1
                || extends_clock(deadline)
                || scope.as_ref().is_some_and(|scope| {
                    !Arc::ptr_eq(scope.cancellation(), &self.flag)
                        || extends_clock(scope.deadline())
                })
            {
                return Err(ProblemError::Contract(
                    "local strategy differs from its serial task owner".into(),
                )
                .into());
            }
            let adapters = backends
                .iter()
                .map(|backend| execution::adapter(*backend))
                .collect::<Vec<_>>();
            if execution::scope_stack_bytes(&adapters, 1, self.service.policy.stack_bytes)? != 0 {
                return Err(ProblemError::Unsupported(
                    "local strategy needs an unqualified scope team".into(),
                )
                .into());
            }
            for adapter in adapters {
                self.scopes
                    .borrow_mut()
                    .enter(adapter, self.service.policy.stack_bytes)?;
            }
            let mut retained = self.retained.try_borrow_mut().map_err(|_| {
                ProblemError::Internal("overlapping local strategy callback".into())
            })?;
            work(&mut retained, &self.flag, &self.budget)
        })();
        Box::pin(std::future::ready(result))
    }
    fn prepare<'a>(
        &'a self,
        operation: math::solves::AutomaticOperation,
        scope: pse_kernels::ExecutionScope,
        cancel: &'a crate::CancelSource,
    ) -> Pending<'a, math::solves::PreparedRung> {
        Box::pin(std::future::ready(
            self.preparation(
                self.service
                    .prepare_automatic_operation(operation, scope, cancel),
            ),
        ))
    }
}
