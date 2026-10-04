// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Opaque scientific operations use the shared numerical strategy and original assessment.
use super::{MathRuntimeError, MathService, SessionDisposition, StepRetention, strategy};
use pse_backend_native::{
    ProblemError,
    solve::{Backend, Controls, SolveReport},
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
    pub(crate) start: StartOrigin,
    pub(crate) start_identity: Option<ContentHash>,
}

/// One original operation, including all scientific evaluation and completion checks.
/// The generic driver decides the transition; this adapter owns no recovery policy.
pub(crate) fn direct<T>(
    service: &MathService,
    source: Source<'_>,
    scope: &pse_kernels::ExecutionScope,
    execute: impl FnOnce() -> Result<T, MathRuntimeError>,
    native: impl Fn(&T) -> Option<&SolveReport>,
    permission: impl Fn(&T) -> crate::workflow::numerics::CandidateDecision,
    original_failure: impl Fn(&T) -> Option<Arc<ProblemError>>,
) -> Result<(T, Arc<strategy::Trace>), MathRuntimeError> {
    let declaration = strategy::direct(source.controls);
    let mut charging_owner = pse_ids::FramedHasher::new(pse_ids::Frame::NumericalWorkV1);
    charging_owner
        .hash(&source.original)
        .hash(&source.preparation)
        .hash(&source.profile);
    let charging_owner = charging_owner.finish_hash();
    let mut execute = Some(execute);
    let mut operation_failure = None;
    let mut retention_failure = None;
    let result = strategy::run(
        &declaration,
        scope,
        |_| strategy::Facts {
            support: BTreeSet::from([source.original, source.preparation]),
            accuracy: Vec::new(),
            start: source.start,
            inherited: false,
            connected: false,
            refusal: None,
        },
        |_, _| {
            let work = execute.take().ok_or_else(|| {
                Arc::new(ProblemError::Internal(
                    "opaque direct operation dispatched twice".into(),
                ))
            })?;
            let value = work().map_err(|error| {
                let error = Arc::new(error);
                operation_failure = Some(error.clone());
                error
            });
            let observation = match &value {
                Ok(value) => native(value).map_or(Observation::Converged, strategy::observe_native),
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
            Ok(value) => {
                let cause = original_failure(value)
                    .or_else(|| native(value).and_then(strategy::cause_native));
                let observation = if strategy::permits_numerical_continuation(observed) {
                    cause
                        .as_ref()
                        .map_or(observed, |cause| strategy::failure(cause))
                } else {
                    observed
                };
                strategy::Assessment {
                    auxiliary: false,
                    retention: StepRetention {
                        candidate: permission(value),
                        session: SessionDisposition::Discard,
                    },
                    observation,
                    cause,
                }
            }
            Err(cause) => {
                let retained = match cause.retained_bytes() {
                    Ok(bytes) => bytes,
                    Err(error) => {
                        retention_failure = Some(error);
                        0
                    }
                };
                strategy::Assessment {
                    auxiliary: false,
                    retention: StepRetention {
                        candidate: crate::workflow::numerics::refused(
                            pse_model::generated::enums::CandidateRefusal::NativeOutcome,
                        ),
                        session: SessionDisposition::Discard,
                    },
                    observation: observed,
                    cause: Some(Arc::new(ProblemError::Math(pse_math::MathError::Typed {
                        retained,
                        cause: pse_model::diagnostic::DiagnosticCause::from_shared(cause.clone()),
                    }))),
                }
            }
        },
    );
    if let Some(refusal) = retention_failure {
        if let Some(cause) = operation_failure {
            return Err(MathRuntimeError::StrategyTraceUnavailable {
                cause,
                refusal: Arc::new(refusal),
            });
        }
        return Err(ProblemError::Internal(
            "opaque failure retention refused without source".into(),
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
        .ok_or(MathRuntimeError::Limit("opaque strategy trace extent"))?;
    let owner = match service.reserve("math:opaque-strategy-trace", bytes) {
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
                                    "opaque strategy ended without original assessment".into(),
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
    use pse_model::generated::enums::{
        CandidateRefusal, NumericalEventKind as Kind, NumericalTransition as Transition,
    };
    use std::sync::atomic::{AtomicBool, Ordering};
    fn scope() -> pse_kernels::ExecutionScope {
        pse_kernels::ExecutionScope::new(
            Arc::new(AtomicBool::new(false)),
            Some(std::time::Instant::now() + std::time::Duration::from_secs(10)),
        )
    }
    fn source(controls: &Controls, start: StartOrigin) -> Source<'_> {
        Source {
            original: ContentHash::from_bytes([1; 32]),
            preparation: ContentHash::from_bytes([2; 32]),
            profile: ContentHash::from_bytes([3; 32]),
            backend: None,
            controls,
            start,
            start_identity: None,
        }
    }
    #[test]
    fn failure_extent_uses_owned_capacity_and_refuses_unknown_foreign_payloads() {
        let mut text = String::with_capacity(8192);
        text.push_str("failure");
        let capacity = text.capacity();
        let error = MathRuntimeError::Panic(text);
        assert_eq!(
            error.retained_bytes().unwrap(),
            size_of::<MathRuntimeError>() + capacity
        );
        let lowered = error.into_problem();
        assert!(lowered.retained_bytes() >= size_of::<MathRuntimeError>() + capacity);

        let pool = MathRuntimeError::Pool(datafusion::common::DataFusionError::ResourcesExhausted(
            String::with_capacity(4096),
        ));
        assert!(pool.retained_bytes().unwrap() >= 4096);
        let opaque = MathRuntimeError::Pool(datafusion::common::DataFusionError::External(
            Box::new(std::io::Error::other("foreign error")),
        ));
        assert!(matches!(
            opaque.retained_bytes(),
            Err(ProblemError::Unsupported(_))
        ));
        let controls = Controls::default();
        let scope = scope();
        let runtime = crate::workflow::tests::runtime_with(128 << 20, 1 << 20, 512 << 20);
        let result = direct(
            runtime.native(),
            source(&controls, StartOrigin::Specification),
            &scope,
            || Err::<(), _>(opaque),
            |_| None,
            |_| crate::workflow::numerics::refused(CandidateRefusal::NoCandidate),
            |_| None,
        )
        .unwrap_err();
        assert!(result.strategy_trace().is_none());
        assert!(matches!(
            result.strategy_trace_refusal().unwrap().as_ref(),
            ProblemError::Unsupported(_)
        ));
        let MathRuntimeError::StrategyTraceUnavailable { cause, .. } = result else {
            panic!("observable accounting refusal")
        };
        assert!(matches!(
            cause.as_ref(),
            MathRuntimeError::Pool(datafusion::common::DataFusionError::External(_))
        ));
        let refused = MathRuntimeError::StrategyTraceUnavailable {
            cause: cause.clone(),
            refusal: Arc::new(ProblemError::Unsupported(
                "foreign allocation is unavailable".into(),
            )),
        };
        use pse_model::diagnostic::DiagnosticProjection;
        let original_class = cause
            .boundary_diagnostic(pse_diagnostics::DiagnosticStage::Native)
            .class;
        let lowered = refused.into_problem();
        assert_eq!(lowered.retained_bytes(), usize::MAX);
        assert!(
            runtime
                .native()
                .reserve("test:unreservable-foreign-error", lowered.retained_bytes())
                .is_err()
        );
        let diagnostic = lowered.boundary_diagnostic(pse_diagnostics::DiagnosticStage::Native);
        assert_eq!(diagnostic.class, original_class);
        assert!(
            diagnostic
                .causes
                .iter()
                .any(|cause| cause.class == pse_model::diagnostic::BoundaryClass::Unsupported)
        );
        let known = MathRuntimeError::StrategyTraceUnavailable {
            cause: Arc::new(MathRuntimeError::Panic("known panic".into())),
            refusal: Arc::new(ProblemError::Limit {
                kind: pse_backend_native::LimitKind::Memory,
                detail: "trace pool denied".into(),
            }),
        };
        assert!(known.retained_bytes().is_ok());
    }
    #[test]
    fn original_assessment_precedes_finish_and_composed_work_is_not_native_counters() {
        let runtime = crate::workflow::tests::runtime_with(128 << 20, 1 << 20, 512 << 20);
        let scope = scope();
        let controls = Controls::default();
        let (checked, trace) = direct(
            runtime.native(),
            source(&controls, StartOrigin::Specification),
            &scope,
            || Ok(true),
            |_| None,
            |checked| {
                assert!(*checked);
                crate::workflow::numerics::constant_use(&pse_backend_native::quality::Quality {
                    rows: Vec::new(),
                    bounds: Vec::new(),
                    integrality: Vec::new(),
                    normalized_max: 0.0,
                })
            },
            |_| None,
        )
        .unwrap();
        assert!(checked);
        assert!(trace.owner.is_some());
        let final_event = trace.events.last().unwrap();
        assert_eq!(final_event.kind, Kind::Finished);
        assert_eq!(final_event.transition, Some(Transition::Finish));
        let work = final_event.work.unwrap().observed;
        assert_eq!(work.attempts, 1);
        assert_eq!(work.evaluations, None);
        assert_eq!(work.iterations, None);
        assert_eq!(work.factorizations, None);
        assert_eq!(work.proof_steps, None);
    }
    #[test]
    fn original_failure_is_typed_and_unusable_without_retry() {
        let runtime = crate::workflow::tests::runtime_with(128 << 20, 1 << 20, 512 << 20);
        let controls = Controls::default();
        let scope = scope();
        let cause = Arc::new(ProblemError::Internal(
            "scientific assessment failed".into(),
        ));
        let (_, trace) = direct(
            runtime.native(),
            source(&controls, StartOrigin::Specification),
            &scope,
            || Ok(()),
            |_| None,
            |_| crate::workflow::numerics::refused(CandidateRefusal::ValidationFailed),
            |_| Some(cause.clone()),
        )
        .unwrap();
        let event = trace.events.last().unwrap();
        assert_eq!(event.transition, Some(Transition::Stop));
        assert_eq!(event.observation, Some(Observation::OperationalFailure));
        assert!(Arc::ptr_eq(event.cause.as_ref().unwrap(), &cause));
        assert_eq!(
            trace
                .events
                .iter()
                .filter(|e| e.kind == Kind::Started)
                .count(),
            1
        );
    }
    #[test]
    fn original_scope_cancellation_after_scientific_work_retains_abandonment_trace() {
        let runtime = crate::workflow::tests::runtime_with(128 << 20, 1 << 20, 512 << 20);
        let controls = Controls::default();
        let scope = scope();
        let result = direct(
            runtime.native(),
            source(&controls, StartOrigin::Specification),
            &scope,
            || {
                scope.cancellation().store(true, Ordering::Release);
                Ok(())
            },
            |_| None,
            |_| crate::workflow::numerics::refused(CandidateRefusal::NoCandidate),
            |_| None,
        );
        let error = result.unwrap_err();
        let trace = error.strategy_trace().unwrap();
        assert!(trace.owner.is_some());
        assert_eq!(trace.events.last().unwrap().kind, Kind::Abandoned);
        assert_eq!(
            trace.events.last().unwrap().observation,
            Some(Observation::Cancelled)
        );
        assert_eq!(
            trace.events.last().unwrap().work.unwrap().observed.attempts,
            1
        );
    }
    #[test]
    fn explicit_start_requires_its_declared_entry_policy_and_failure_keeps_original_cause() {
        let runtime = crate::workflow::tests::runtime_with(128 << 20, 1 << 20, 512 << 20);
        let mut controls = Controls::default();
        let scope = scope();
        let refused = direct(
            runtime.native(),
            source(&controls, StartOrigin::Explicit),
            &scope,
            || panic!("refused source must not dispatch"),
            |_: &()| None,
            |_| crate::workflow::numerics::refused(CandidateRefusal::NoCandidate),
            |_| None,
        )
        .unwrap_err();
        assert_eq!(
            refused
                .strategy_trace()
                .unwrap()
                .events
                .last()
                .unwrap()
                .kind,
            Kind::Refused
        );
        controls.start = pse_backend_native::solve::StartPolicy::Explicit;
        let error = direct(
            runtime.native(),
            source(&controls, StartOrigin::Explicit),
            &scope,
            || Err::<(), _>(MathRuntimeError::Panic("original operation panic".into())),
            |_| None,
            |_| crate::workflow::numerics::refused(CandidateRefusal::NoCandidate),
            |_| None,
        )
        .unwrap_err();
        let MathRuntimeError::Strategy { cause, trace } = error else {
            panic!("trace carrier required")
        };
        assert!(
            matches!(cause.as_ref(),MathRuntimeError::Panic(message) if message=="original operation panic")
        );
        assert_eq!(
            trace.events.last().unwrap().observation,
            Some(Observation::Panic)
        );
        assert_eq!(
            trace.events.last().unwrap().work.unwrap().observed.attempts,
            1
        );
    }
}
