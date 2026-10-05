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
    execute: impl FnOnce(Arc<TaskAdmission>) -> Result<T, MathRuntimeError>,
    classify: impl Fn(&T) -> Observation,
    assess: impl Fn(&T, Observation) -> strategy::Assessment,
) -> Result<(T, Arc<strategy::Trace>), MathRuntimeError> {
    let declaration = declaration(&source);
    // The producer has frozen its callable, source contract and profile before entry.
    // No derivative, different profile, or fresh scientific start is manufactured here.
    let mut binding = pse_ids::FramedHasher::new(pse_ids::Frame::NumericalDecisionV1);
    binding
        .hash(&source.original)
        .hash(&source.preparation)
        .hash(&source.profile)
        .str(source.start.as_str());
    if let Some(point) = source.start_identity {
        binding.hash(&point);
    }
    let candidate = strategy::AutoCandidate {
        identity: binding.finish_hash(),
        kind: pse_model::strategy::MechanismKind::Direct,
        start: source.start,
        replacement: false,
        support: BTreeSet::from([source.original, source.preparation]),
        reservation: None,
        prepared: true,
    };
    let candidates = [candidate];
    let admission = TaskAdmission::new(
        declaration.limits,
        scope.clone(),
        Some(service.pool.clone()),
        false,
    );
    let decision = strategy::next_automatic(
        source.request,
        &declaration.start,
        &candidates,
        &BTreeSet::new(),
        None,
        admission.observation()?,
        false,
    );
    let decision_key = strategy::automatic_decision_key(
        source.request,
        &candidates,
        &decision,
        None,
        admission.observation()?,
    )?;
    let refusal = if source.request.policy == pse_model::strategy::CompositionPolicy::Declared {
        Some(Arc::new(ProblemError::Unsupported(
            "scientific callable has no producer-bound explicit composition declaration".into(),
        )))
    } else {
        match decision {
            strategy::AutoDecision::Dispatch { candidate: 0 } => None,
            strategy::AutoDecision::Stop { cause } => Some(cause.unwrap_or_else(|| {
                Arc::new(ProblemError::Contract(
                    "scientific callable entry origin is not admitted".into(),
                ))
            })),
            _ => Some(Arc::new(ProblemError::Internal(
                "frozen callable was not ready for dispatch".into(),
            ))),
        }
    };
    let mut charging_owner = pse_ids::FramedHasher::new(pse_ids::Frame::NumericalWorkV1);
    charging_owner
        .hash(&source.original)
        .hash(&source.preparation)
        .hash(&source.profile);
    let charging_owner = charging_owner.finish_hash();
    let mut execute = Some(execute);
    let mut operation_failure = None;
    let mut retention_failure = None;
    let mut result = strategy::run_admitted(
        &declaration,
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
            refusal: refusal.clone(),
        },
        |_, _| {
            let work = execute.take().ok_or_else(|| {
                Arc::new(ProblemError::Internal(
                    "callable direct operation dispatched twice".into(),
                ))
            })?;
            let value = work(admission.clone()).map_err(|error| {
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
    }
    if let Some(assessment) = &result.assessment {
        let observed = strategy::AutoObservation {
            awaiting_assessment: false,
            native: assessment.observation,
            original: Some(assessment.original.clone()),
            permission: Some(assessment.retention.candidate.usability),
        };
        let next = strategy::next_automatic(
            source.request,
            &declaration.start,
            &candidates,
            &BTreeSet::from([0]),
            Some(&observed),
            admission.observation()?,
            false,
        );
        let key = strategy::automatic_decision_key(
            source.request,
            &candidates,
            &next,
            Some(&observed),
            admission.observation()?,
        )?;
        if let Some(event) = result.events.last_mut() {
            event.decision = Some(key);
            event.transition = Some(match next {
                strategy::AutoDecision::Finish => pse_model::strategy::Transition::Finish,
                _ => pse_model::strategy::Transition::Stop,
            });
        }
    }
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
        declaration,
        original: source.original,
        backend: source.backend,
        profile: source.profile,
        start: source.start,
        starts: vec![source.start],
        products: vec![strategy::RungProducts {
            start: source.start_identity,
            ..Default::default()
        }],
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
