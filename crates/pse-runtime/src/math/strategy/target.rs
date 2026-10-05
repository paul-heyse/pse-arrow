// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Frozen callable targets use the common resolver, task admission and original assessment.
use super::super::{MathRuntimeError, MathService, SessionDisposition, StepRetention};
use super::{self as strategy, admission::TaskAdmission};
use pse_backend_native::{
    ProblemError,
    solve::{Backend, Controls},
};
use pse_ids::ContentHash;
use pse_model::{
    generated::enums::NumericalAttemptObservation as Observation,
    strategy::{Scope, StartOrigin, WorkCharge, WorkObservation},
};
use std::{collections::BTreeSet, sync::Arc};

/// Frozen identities and actual entry origin from the scientific producer.
pub(crate) struct Source<'a> {
    pub(crate) original: ContentHash,
    pub(crate) preparation: ContentHash,
    pub(crate) profile: ContentHash,
    pub(crate) backend: Option<Backend>,
    pub(crate) solver: Option<&'a super::super::solves::SolverProfile>,
    pub(crate) controls: &'a Controls,
    /// The immutable scientific request; optional recovery permission creates no supplier.
    pub(crate) request: &'a pse_model::strategy::CompositionRequest,
    pub(crate) start: StartOrigin,
    pub(crate) start_identity: Option<ContentHash>,
}

/// Mechanical projection of the same frozen callable contract used at execution.
pub(crate) fn declaration(source: &Source<'_>) -> pse_model::strategy::NumericalStrategy {
    let mut declaration = strategy::direct(source.controls);
    declaration.branch = source.request.branch;
    declaration.start.recovery = source.request.recovery.clone();
    if let Some(limits) = source.request.limits {
        declaration.limits = limits;
        declaration.mechanisms[0].limits = limits;
    }
    declaration.mechanisms[0].support = BTreeSet::from([source.original, source.preparation])
        .into_iter()
        .collect();
    declaration.mechanisms[0].profile =
        source
            .backend
            .map(|backend| pse_model::strategy::ProfileRef {
                backend,
                key: source.profile,
            });
    declaration
}

/// One original operation, including all scientific evaluation and completion checks.
/// The generic driver decides the transition; this adapter owns no recovery policy.
pub(crate) fn callable<T>(
    service: &MathService,
    source: Source<'_>,
    scope: &pse_kernels::ExecutionScope,
    mut execute: impl FnMut(
        Option<&super::super::solves::SolverProfile>,
        Arc<TaskAdmission>,
    ) -> Result<T, MathRuntimeError>,
    native_report: impl Fn(&T) -> Option<&pse_backend_native::solve::SolveReport>,
    classify: impl Fn(&T) -> Observation,
    assess: impl Fn(&T, Observation) -> strategy::Assessment,
) -> Result<(T, Arc<strategy::Trace>), MathRuntimeError> {
    let mut declaration = declaration(&source);
    #[cfg(feature = "solver-pounce")]
    if source.request.limits.is_none()
        && source.request.policy == pse_model::strategy::CompositionPolicy::Auto
        && source.backend == Some(Backend::Pounce)
        && let Some(solver) = source.solver
    {
        let settings = match &solver.backend {
            pse_backend_native::execution::BackendSettings::Default => {
                pse_backend_native::settings::pounce::Settings::default()
            }
            pse_backend_native::execution::BackendSettings::Pounce(settings) => settings.clone(),
            _ => {
                return Err(
                    ProblemError::Contract("callable POUNCE settings differ".into()).into(),
                );
            }
        };
        declaration.limits.attempts = u64::try_from(
            1 + pse_backend_native::pounce::second_opinion_capacity(
                &solver.controls,
                &settings,
                false,
            ),
        )
        .map_err(|_| ProblemError::memory("callable profile catalog extent"))?;
    }
    let admission = TaskAdmission::new(
        declaration.limits,
        scope.clone(),
        Some(service.pool.clone()),
        false,
    );
    #[cfg(feature = "solver-pounce")]
    let mut profiles = vec![source.solver.cloned()];
    #[cfg(not(feature = "solver-pounce"))]
    let profiles = [source.solver.cloned()];
    #[cfg(feature = "solver-pounce")]
    let mut known_profiles = BTreeSet::from([source.profile]);
    #[cfg(not(feature = "solver-pounce"))]
    let _ = &native_report;
    let mut attempted = BTreeSet::new();
    let mut last = None;
    let mut events: Vec<strategy::Event> = Vec::new();
    let mut mechanism_count = 0;
    let mut latest_result = None;
    let mut outer_terminal = None;
    let mut operation_failure = None;
    let mut retention_failure = None;
    loop {
        scope.check().map_err(ProblemError::Provider)?;
        let candidates = profiles
            .iter()
            .enumerate()
            .map(|(index, profile)| {
                let key = match profile {
                    Some(profile) if index > 0 => {
                        super::super::solves::profile_key(profile).map(|key| key.as_id())
                    }
                    _ => Ok(source.profile),
                }?;
                let mut binding = pse_ids::FramedHasher::new(pse_ids::Frame::NumericalDecisionV2);
                binding
                    .hash(&source.original)
                    .hash(&source.preparation)
                    .hash(&key)
                    .str(source.start.as_str());
                if let Some(point) = source.start_identity {
                    binding.hash(&point);
                }
                Ok(strategy::AutoCandidate {
                    identity: binding.finish_hash(),
                    kind: if index == 0 {
                        pse_model::strategy::MechanismKind::Direct
                    } else {
                        pse_model::strategy::MechanismKind::NativeGlobalization
                    },
                    start: source.start,
                    replacement: false,
                    support: BTreeSet::from([source.original, source.preparation]),
                    reservation: None,
                    prepared: true,
                })
            })
            .collect::<Result<Vec<_>, ProblemError>>()?;
        let mut request = source.request.clone();
        request.limits = Some(declaration.limits);
        let decision = strategy::next_automatic(
            &request,
            &declaration.start,
            &candidates,
            &attempted,
            last.as_ref(),
            admission.observation()?,
            false,
        );
        let index = match decision {
            strategy::AutoDecision::Dispatch { candidate } => candidate,
            _ => {
                if let strategy::AutoDecision::Stop { cause } = &decision {
                    outer_terminal = cause.clone();
                }
                if latest_result.is_none() {
                    let cause = outer_terminal.clone().unwrap_or_else(|| {
                        Arc::new(if source.start == StartOrigin::Explicit {
                            ProblemError::Contract(
                                "callable explicit point conflicts with its declared entry policy"
                                    .into(),
                            )
                        } else {
                            ProblemError::Unsupported(
                                "no callable operation admits the requested entry origin".into(),
                            )
                        })
                    });
                    let decision_key = strategy::automatic_decision_key(
                        &request,
                        &candidates,
                        &decision,
                        last.as_ref(),
                        admission.observation()?,
                    )?;
                    mechanism_count = 1;
                    declaration
                        .mechanisms
                        .push(declaration.mechanisms[0].clone());
                    latest_result = Some(strategy::DriverResult {
                        value: None,
                        assessment: None,
                        events: Vec::new(),
                        terminal: Some(cause.clone()),
                        work: admission.observation()?,
                    });
                    events.push(strategy::Event {
                        mechanism: 0,
                        kind: pse_model::generated::enums::NumericalEventKind::Refused,
                        phase: pse_model::strategy::Phase::Native,
                        original: None,
                        decision: Some(decision_key),
                        observation: Some(strategy::failure(&cause)),
                        transition: Some(pse_model::strategy::Transition::Stop),
                        permission: None,
                        work: None,
                        cause: Some(cause),
                    });
                }
                if let Some(event) = events.last_mut() {
                    event.transition =
                        Some(if matches!(decision, strategy::AutoDecision::Finish) {
                            pse_model::strategy::Transition::Finish
                        } else {
                            pse_model::strategy::Transition::Stop
                        });
                }
                break;
            }
        };
        attempted.insert(index);
        let decision_key = strategy::automatic_decision_key(
            &request,
            &candidates,
            &decision,
            last.as_ref(),
            admission.observation()?,
        )?;
        let mut operation = declaration.clone();
        operation.mechanisms.truncate(1);
        operation.mechanisms[0].limits = declaration.limits;
        operation.mechanisms[0].kind = candidates[index].kind;
        operation.mechanisms[0].starts = vec![source.start];
        operation.mechanisms[0].transitions = vec![
            pse_model::strategy::Transition::Finish,
            pse_model::strategy::Transition::Recover,
            pse_model::strategy::Transition::Continue,
            pse_model::strategy::Transition::Stop,
        ];
        let actual_profile = if index == 0 {
            source.profile
        } else {
            profiles[index]
                .as_ref()
                .map(super::super::solves::profile_key)
                .transpose()?
                .map_or(source.profile, |key| key.as_id())
        };
        operation.mechanisms[0].profile =
            source
                .backend
                .map(|backend| pse_model::strategy::ProfileRef {
                    backend,
                    key: actual_profile,
                });
        let mut charging_owner = pse_ids::FramedHasher::new(pse_ids::Frame::NumericalWorkV1);
        charging_owner
            .hash(&source.original)
            .hash(&source.preparation)
            .hash(&actual_profile);
        let charging_owner = charging_owner.finish_hash();
        // A later binding or cutoff must not inherit an earlier operation's cause.
        operation_failure = None;
        let mut result = strategy::run_admitted(
            &operation,
            scope,
            admission.clone(),
            |_| strategy::Facts {
                support: BTreeSet::from([source.original, source.preparation]),
                accuracy: Vec::new(),
                consumption: Vec::new(),
                reservation: None,
                work_admitted: false,
                start: source.start,
                inherited: false,
                connected: false,
                refusal: None,
            },
            |_, _| {
                let value = execute(profiles[index].as_ref(), admission.clone()).map_err(|error| {
                    let error = Arc::new(error);
                    operation_failure = Some(error.clone());
                    error
                });
                let observation = match &value {
                    Ok(value) => classify(value),
                    Err(cause) => strategy::runtime_failure(cause),
                };
                // Original checks, integrator actions and fitting statistics are composed
                // work. Their full counters are not supplied by the native subreport.
                Ok(strategy::Attempt {
                    evidence: Vec::new(),
                    value,
                    observation,
                    work: WorkCharge {
                        phase: pse_model::strategy::Phase::Native,
                        scope: Scope::Task,
                        charging_owner,
                        observed: WorkObservation {
                            attempts: 1,
                            evaluations: None,
                            iterations: None,
                            factorizations: None,
                            proof_steps: None,
                        },
                    },
                })
            },
            |value, observed| match value {
                Ok(value) => assess(value, observed),
                Err(cause) => {
                    let retained = match cause.retained_bytes() {
                        Ok(bytes) => bytes,
                        Err(error) => {
                            retention_failure = Some(error);
                            0
                        }
                    };
                    let original = Arc::new(ProblemError::Math(pse_math::MathError::Typed {
                        retained,
                        cause: pse_model::diagnostic::DiagnosticCause::from_shared(cause.clone()),
                    }));
                    strategy::Assessment {
                        auxiliary: false,
                        original: strategy::OriginalConclusion::Unavailable {
                            cause: original.clone(),
                        },
                        work: Vec::new(),
                        retention: StepRetention {
                            candidate: crate::workflow::numerics::refused(
                                pse_model::generated::enums::CandidateRefusal::NativeOutcome,
                            ),
                            session: SessionDisposition::Discard,
                        },
                        observation: observed,
                        cause: Some(original),
                    }
                }
            },
        );
        for event in &mut result.events {
            event.decision = Some(decision_key);
            event.mechanism = mechanism_count;
        }
        declaration.mechanisms.push(operation.mechanisms[0].clone());
        mechanism_count += 1;
        last = result
            .assessment
            .as_ref()
            .map(|assessment| strategy::AutoObservation {
                awaiting_assessment: false,
                native: assessment.observation,
                original: Some(assessment.original.clone()),
                permission: Some(assessment.retention.candidate.usability),
            });
        #[cfg(feature = "solver-pounce")]
        if source.request.policy == pse_model::strategy::CompositionPolicy::Auto
            && source.backend == Some(Backend::Pounce)
            && let Some(baseline) = source.solver
            && let Some(Ok(value)) = result.value.as_ref()
            && let Some(report) = native_report(value)
        {
            let settings = match &baseline.backend {
                pse_backend_native::execution::BackendSettings::Default => {
                    pse_backend_native::settings::pounce::Settings::default()
                }
                pse_backend_native::execution::BackendSettings::Pounce(settings) => {
                    settings.clone()
                }
                _ => {
                    return Err(
                        ProblemError::Contract("callable POUNCE settings differ".into()).into(),
                    );
                }
            };
            for offered in pse_backend_native::pounce::second_opinion_profiles(
                &baseline.controls,
                &settings,
                report,
                false,
            )? {
                let mut profile = baseline.clone();
                profile.controls = offered.controls;
                profile.backend =
                    pse_backend_native::execution::BackendSettings::Pounce(offered.settings);
                let key = super::super::solves::profile_key(&profile)?.as_id();
                if known_profiles.insert(key) {
                    profiles.push(Some(profile));
                }
            }
        }
        events.extend(result.events.iter().cloned());
        let terminal = result.terminal.is_some();
        latest_result = Some(result);
        if terminal || source.request.policy == pse_model::strategy::CompositionPolicy::Declared {
            break;
        }
    }
    let mut result = latest_result.ok_or_else(|| {
        ProblemError::Unsupported("no callable operation satisfies task constraints".into())
    })?;
    declaration.mechanisms.remove(0);
    result.events = events;
    if let Some(refusal) = retention_failure {
        if let Some(cause) = operation_failure {
            return Err(MathRuntimeError::StrategyTraceUnavailable {
                cause,
                refusal: Arc::new(refusal),
            });
        }
        return Err(ProblemError::Internal(
            "callable failure retention refused without source".into(),
        )
        .into());
    }
    let failure_bytes = if let Some(cause) = &operation_failure {
        match cause.retained_bytes() {
            Ok(bytes) => bytes,
            Err(refusal) => {
                return Err(MathRuntimeError::StrategyTraceUnavailable {
                    cause: cause.clone(),
                    refusal: Arc::new(refusal),
                });
            }
        }
    } else {
        0
    };
    let trace = strategy::Trace {
        owner: None,
        publication_request: None,
        declaration,
        original: source.original,
        backend: source.backend,
        profile: source.profile,
        start: source.start,
        starts: vec![source.start; mechanism_count],
        products: vec![
            strategy::RungProducts {
                start: source.start_identity,
                ..Default::default()
            };
            mechanism_count
        ],
        events: result.events,
    };
    let bytes = trace
        .retained_bytes()?
        .checked_add(failure_bytes)
        .and_then(|bytes| bytes.checked_add(size_of::<MathRuntimeError>() + 4 * size_of::<usize>()))
        .ok_or(MathRuntimeError::Limit("callable strategy trace extent"))?;
    let owner = match service.reserve("math:callable-strategy-trace", bytes) {
        Ok(owner) => owner,
        Err(error) => {
            let error = Arc::new(error);
            let retained = error.retained_bytes()?;
            let refusal = Arc::new(ProblemError::Math(pse_math::MathError::Typed {
                retained,
                cause: pse_model::diagnostic::DiagnosticCause::from_shared(error.clone()),
            }));
            return Err(MathRuntimeError::StrategyTraceUnavailable {
                cause: operation_failure.unwrap_or(error),
                refusal,
            });
        }
    };
    let trace = Arc::new(trace.with_owner(owner));
    if let Some(cause) = outer_terminal {
        let source_failure = operation_failure.filter(|_| {
            last.as_ref()
                .and_then(|last| last.original.as_ref())
                .and_then(strategy::OriginalConclusion::cause)
                .is_some_and(|original| Arc::ptr_eq(&original, &cause))
        });
        return Err(MathRuntimeError::Strategy {
            // Preserve the actual dispatched error when this Stop projects its
            // original assessment; an independent budget cutoff remains its own cause.
            cause: source_failure.unwrap_or_else(|| {
                Arc::new(
                    ProblemError::Math(pse_math::MathError::Typed {
                        retained: cause.retained_bytes(),
                        cause: pse_model::diagnostic::DiagnosticCause::from_shared(cause),
                    })
                    .into(),
                )
            }),
            trace,
        });
    }
    match (result.value, result.terminal) {
        (Some(Ok(value)), None) => Ok((value, trace)),
        (_, terminal) => {
            let cause = operation_failure.unwrap_or_else(|| {
                Arc::new(
                    terminal
                        .map_or_else(
                            || {
                                ProblemError::Internal(
                                    "callable strategy ended without original assessment".into(),
                                )
                            },
                            |cause| {
                                ProblemError::Math(pse_math::MathError::Typed {
                                    retained: cause.retained_bytes(),
                                    cause: pse_model::diagnostic::DiagnosticCause::from_shared(
                                        cause,
                                    ),
                                })
                            },
                        )
                        .into(),
                )
            });
            Err(MathRuntimeError::Strategy { cause, trace })
        }
    }
}

/// Ordinary scientific owners retain original completion and permission separately from
/// a native subreport's observation/cause; no numerical convergence grants permission.
pub(crate) fn original_assessment(
    decision: crate::workflow::numerics::CandidateDecision,
    original_failure: Option<Arc<ProblemError>>,
    native_cause: Option<Arc<ProblemError>>,
    observation: Observation,
) -> strategy::Assessment {
    let original = match &original_failure {
        Some(cause) => strategy::OriginalConclusion::Unavailable {
            cause: cause.clone(),
        },
        None if decision.permits_use() => strategy::OriginalConclusion::Satisfied,
        None => strategy::OriginalConclusion::Refused {
            cause: Arc::new(ProblemError::numerical(decision.reason())),
        },
    };
    strategy::Assessment {
        auxiliary: false,
        original,
        work: Vec::new(),
        retention: StepRetention {
            candidate: decision,
            session: SessionDisposition::Discard,
        },
        observation,
        cause: original_failure.or(native_cause),
    }
}

/// Actual declared coordinates use the same point identity as other driver producers.
pub(crate) fn point_identity(original: ContentHash, point: &[f64]) -> ContentHash {
    let mut identity = pse_ids::FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
    identity
        .str("actual-original-coordinate-start")
        .hash(&original);
    for value in point {
        identity.f64(*value);
    }
    identity.finish_hash()
}

/// Retain the existing original-assessment envelope without reconstructing its cause.
pub(crate) fn assessment_failure(
    error: &pse_model::diagnostic::BoundaryDiagnostic,
) -> Arc<ProblemError> {
    use pse_model::HeapUsage;
    Arc::new(ProblemError::Math(pse_math::MathError::Typed {
        retained: error.owned_bytes(),
        cause: pse_model::diagnostic::DiagnosticCause::new(error.clone()),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn hash(value: u8) -> ContentHash {
        ContentHash::from_bytes([value; 32])
    }
    #[test]
    fn declared_callable_executes_its_actual_bound_direct_operation_once() {
        let service = crate::math::tests::service();
        let controls = Controls::default();
        let request = pse_model::strategy::CompositionRequest {
            policy: pse_model::strategy::CompositionPolicy::Declared,
            ..Default::default()
        };
        let source = Source {
            original: hash(1),
            preparation: hash(2),
            profile: hash(3),
            backend: None,
            solver: None,
            controls: &controls,
            request: &request,
            start: StartOrigin::Specification,
            start_identity: None,
        };
        let scope = pse_kernels::ExecutionScope::new(Arc::default(), None);
        let mut dispatched = 0;
        let (value, trace) = callable(
            &service,
            source,
            &scope,
            |profile, _| {
                assert!(profile.is_none());
                dispatched += 1;
                Ok(17)
            },
            |_| None,
            |_| Observation::Converged,
            |_, observation| {
                original_assessment(
                    crate::workflow::numerics::auxiliary_start(true),
                    None,
                    None,
                    observation,
                )
            },
        )
        .unwrap();
        assert_eq!(value, 17);
        assert_eq!(dispatched, 1);
        assert_eq!(trace.declaration.mechanisms.len(), 1);
        assert!(trace.events.iter().any(|event|event.kind==pse_model::generated::enums::NumericalEventKind::Started));
    }
    #[cfg(feature = "solver-pounce")]
    #[test]
    fn callable_runs_actual_same_backend_profile_after_recoverable_observation() {
        use pse_backend_native::solve::{
            Assurance, Execution, NativeTermination, SolveReport, Termination,
        };
        let service = crate::math::tests::service();
        let solver = super::super::super::solves::SolverProfile::default();
        let scope = pse_kernels::ExecutionScope::new(Arc::default(), None);
        let source = Source {
            original: hash(1),
            preparation: hash(2),
            profile: super::super::super::solves::profile_key(&solver)
                .unwrap()
                .as_id(),
            backend: Some(Backend::Pounce),
            solver: Some(&solver),
            controls: &solver.controls,
            request: &solver.composition,
            start: StartOrigin::Specification,
            start_identity: None,
        };
        let mut profiles = Vec::new();
        let (report, trace) = callable(
            &service,
            source,
            &scope,
            |profile, _| {
                let profile = profile.unwrap();
                profiles.push(super::super::super::solves::profile_key(profile).unwrap());
                let execution = Execution::new(Arc::default(), &profile.controls);
                let contract = pse_backend_native::OracleContract {
                    identity: hash(1),
                    derivatives: pse_kernels::DerivativeOrder::First,
                    smoothness: pse_kernels::DerivativeOrder::First,
                    variables: vec![],
                    rows: vec![],
                };
                Ok(SolveReport::new(
                    Backend::Pounce,
                    &contract,
                    NativeTermination {
                        // Source-owned local-infeasibility recovery offers a scaling
                        // profile without a replacement point or guessed escalation.
                        code: if profiles.len() == 1 { 2 } else { 0 },
                        name: "binder fixture".into(),
                        message: None,
                        category: if profiles.len() == 1 {
                            Termination::Infeasible
                        } else {
                            Termination::Success
                        },
                        assurance: Assurance::None,
                    },
                    &execution,
                ))
            },
            |report| Some(report),
            strategy::observe_native,
            |report, observation| {
                let usable = report.termination.category == Termination::Success;
                let decision = crate::workflow::numerics::CandidateDecision {
                    usability: if usable {
                        pse_model::generated::enums::CandidateUse::Usable
                    } else {
                        pse_model::generated::enums::CandidateUse::Unusable
                    },
                    qualifiers: vec![],
                    refusals: vec![],
                    bound: None,
                };
                original_assessment(decision, None, None, observation)
            },
        )
        .unwrap();
        assert_eq!(report.termination.category, Termination::Success);
        assert_eq!(profiles.len(), 2);
        assert_ne!(profiles[0], profiles[1]);
        assert_eq!(trace.declaration.mechanisms.len(), 2);
        assert_eq!(
            trace.declaration.mechanisms[1].kind,
            pse_model::strategy::MechanismKind::NativeGlobalization
        );
    }
}
