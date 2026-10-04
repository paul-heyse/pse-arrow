// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::{math::SessionDisposition, workflow::numerics};
use pse_model::{
    generated::enums::{CandidateRefusal, CandidateUse},
    strategy::{MechanismKind, Scope},
};
use std::sync::atomic::AtomicBool;

fn hash(n: u8) -> ContentHash {
    ContentHash::from_bytes([n; 32])
}
fn scope() -> pse_kernels::ExecutionScope {
    pse_kernels::ExecutionScope::new(
        Arc::new(AtomicBool::new(false)),
        std::time::Instant::now().checked_add(std::time::Duration::from_secs(10)),
    )
}
fn limits() -> WorkLimits {
    WorkLimits {
        attempts: 5,
        evaluations: Some(20),
        iterations: None,
        factorizations: None,
        proof_steps: None,
    }
}
fn strategy() -> NumericalStrategy {
    let mut s = NumericalStrategy::direct(
        pse_backend_native::solve::StartPolicy::NoPriorStart,
        limits(),
    );
    s.start.recovery.push(StartOrigin::Partial);
    s.mechanisms[0].transitions.push(Transition::Recover);
    let mut recovery = s.mechanisms[0].clone();
    recovery.position = Position::Recovery;
    recovery.starts = vec![StartOrigin::Partial];
    s.mechanisms.push(recovery);
    s
}
fn facts(index: usize) -> Facts {
    Facts {
        support: BTreeSet::new(),
        accuracy: Vec::new(),
        consumption: Vec::new(),
        reservation: Some(charge(index).observed),
        start: if index == 0 {
            StartOrigin::Specification
        } else {
            StartOrigin::Partial
        },
        inherited: false,
        connected: false,
        refusal: None,
    }
}
fn charge(index: usize) -> WorkCharge {
    WorkCharge {
        phase: Phase::Native,
        scope: Scope::Task,
        charging_owner: hash(u8::try_from(index + 1).unwrap()),
        observed: WorkObservation {
            attempts: 1,
            evaluations: Some(3),
            iterations: None,
            factorizations: None,
            proof_steps: None,
        },
    }
}
fn assessment(use_result: bool, auxiliary: bool, observation: Observation) -> Assessment {
    let candidate = if use_result {
        numerics::CandidateDecision {
            usability: CandidateUse::Usable,
            qualifiers: Vec::new(),
            refusals: Vec::new(),
            bound: None,
        }
    } else {
        numerics::refused(CandidateRefusal::NoCandidate)
    };
    Assessment {
        auxiliary,
        original: if use_result {
            OriginalConclusion::Satisfied
        } else {
            OriginalConclusion::Refused {
                cause: Arc::new(ProblemError::numerical("original refusal")),
            }
        },
        work: Vec::new(),
        retention: StepRetention {
            candidate,
            session: SessionDisposition::RetainCompatible,
        },
        observation,
        cause: None,
    }
}
#[test]
fn failed_trajectory_can_run_declared_same_backend_profile_and_only_original_permission_finishes() {
    let mut s = strategy();
    let profile = pse_model::strategy::ProfileRef {
        backend: pse_backend_native::solve::Backend::Kinsol,
        key: hash(4),
    };
    s.mechanisms[0].profile = Some(profile);
    s.mechanisms[1].profile = Some(pse_model::strategy::ProfileRef {
        key: hash(5),
        ..profile
    });
    let mut visited = Vec::new();
    let result = run(
        &s,
        &scope(),
        facts,
        |index, _| {
            visited.push(index);
            Ok(Attempt {
                value: index,
                observation: if index == 0 {
                    Observation::NumericalFailure
                } else {
                    Observation::Converged
                },
                work: charge(index),
            })
        },
        |index, observation| assessment(*index == 1, false, observation),
    );
    assert_eq!(visited, vec![0, 1]);
    assert_eq!(result.value, Some(1));
    assert_eq!(result.work.attempts, 2);
    assert_eq!(result.work.evaluations, Some(6));
    assert!(result.terminal.is_none());
    assert_eq!(
        result.events.last().unwrap().transition,
        Some(Transition::Finish)
    );
}
#[test]
fn failed_dispatched_effect_is_charged_once_and_unknown_work_stays_unknown() {
    let mut declaration = strategy();
    declaration.limits.evaluations = None;
    let result = run::<()>(
        &declaration,
        &scope(),
        facts,
        |_, _| Err(Arc::new(ProblemError::internal("lost dispatched worker")).into()),
        |_, _| panic!("failed effect has no candidate to assess"),
    );
    assert_eq!(result.work.attempts, 1);
    assert_eq!(result.work.evaluations, None);
    assert_eq!(result.work.factorizations, None);
    let event = result.events.last().unwrap();
    assert_eq!(event.work.unwrap().observed.attempts, 1);
    assert_eq!(event.transition, Some(Transition::Stop));
    assert!(matches!(
        result.terminal.as_deref(),
        Some(ProblemError::Internal(_))
    ));
}

#[test]
fn failed_screening_keeps_actual_partial_callback_count() {
    let observed = WorkObservation {
        attempts: 1,
        evaluations: Some(2),
        iterations: None,
        factorizations: None,
        proof_steps: None,
    };
    let result = run::<()>(
        &strategy(),
        &scope(),
        facts,
        |_, _| {
            Err(EffectFailure::observed(
                Arc::new(ProblemError::internal("failed screened supplier")),
                observed,
            ))
        },
        |_, _| panic!("failed screen cannot be assessed"),
    );
    assert_eq!(result.work, observed);
    assert_eq!(
        result.events.last().unwrap().work.unwrap().observed,
        observed
    );
}
#[test]
fn declared_accuracy_is_consumed_and_an_estimate_cannot_establish_certification() {
    use pse_model::strategy::{AccuracyClass, AccuracyDemand, AccuracyEvidence};
    let mut declaration = strategy();
    declaration.accuracy.push(AccuracyDemand {
        product: hash(7),
        normalization: hash(8),
        allowance: 1e-6,
        class: AccuracyClass::Certified,
    });
    let mut supplied = facts(0);
    supplied.consumption = declaration.accuracy.clone();
    assert!(matches!(
        admit(&declaration, 0, true, &supplied),
        Admission::RequiredRefusal(_)
    ));
    supplied.accuracy.push(AccuracyEvidence {
        product: hash(7),
        normalization: hash(8),
        error: Some(1e-7),
        class: AccuracyClass::Estimated,
    });
    assert!(matches!(
        admit(&declaration, 0, true, &supplied),
        Admission::RequiredRefusal(_)
    ));
    supplied.accuracy[0].class = AccuracyClass::Certified;
    assert!(matches!(
        admit(&declaration, 0, true, &supplied),
        Admission::Ready
    ));
    supplied.accuracy[0].normalization = hash(9);
    assert!(matches!(
        admit(&declaration, 0, true, &supplied),
        Admission::RequiredRefusal(_)
    ));
}
#[test]
fn optional_capability_refusal_preserves_base_required_contract_failure_does_not_retry() {
    let mut s = strategy();
    s.mechanisms[0].position = Position::Preparation;
    s.mechanisms[0].kind = MechanismKind::KktPredictor;
    s.mechanisms[0].required = false;
    s.mechanisms[0].support.push(hash(9));
    s.mechanisms[1].position = Position::Execution;
    s.mechanisms[1].starts = vec![StartOrigin::Specification];
    let result = run(
        &s,
        &scope(),
        |_| facts(0),
        |index, _| {
            Ok(Attempt {
                value: index,
                observation: Observation::Converged,
                work: charge(index),
            })
        },
        |_, o| assessment(true, false, o),
    );
    assert_eq!(result.value, Some(1));
    assert_eq!(result.work.attempts, 1);
    assert_eq!(result.events[1].kind, EventKind::Refused);
    s.mechanisms[0].required = true;
    let result = run(
        &s,
        &scope(),
        |_| facts(0),
        |_, _| panic!("required refusal must not execute"),
        |_: &usize, _| panic!("no assessment"),
    );
    assert!(result.terminal.is_some());
    assert_eq!(result.work.attempts, 0);
}
#[test]
fn limited_without_stagnation_does_not_invent_failure_or_recovery() {
    let s = strategy();
    let result = run(
        &s,
        &scope(),
        facts,
        |index, _| {
            Ok(Attempt {
                value: index,
                observation: Observation::Limited,
                work: charge(index),
            })
        },
        |_, o| assessment(false, false, o),
    );
    assert_eq!(result.work.attempts, 1);
    assert_eq!(
        result.events.last().unwrap().transition,
        Some(Transition::Stop)
    );
}
#[test]
fn native_success_and_auxiliary_success_cannot_bypass_original_permission() {
    let s = strategy();
    for auxiliary in [false, true] {
        let result = run(
            &s,
            &scope(),
            facts,
            |index, _| {
                Ok(Attempt {
                    value: index,
                    observation: Observation::Converged,
                    work: charge(index),
                })
            },
            |_, o| assessment(false, auxiliary, o),
        );
        assert!(!result.assessment.unwrap().retention.permits_session());
        assert_eq!(
            result.events.last().unwrap().transition,
            Some(Transition::Stop)
        );
    }
}
#[test]
fn actual_work_is_charged_once_unknown_is_not_zero_and_task_cap_is_terminal() {
    let mut ledger = Ledger::new(limits());
    ledger.charge(charge(0)).unwrap();
    assert!(matches!(
        ledger.charge(charge(0)),
        Err(ProblemError::Internal(_))
    ));
    assert_eq!(ledger.observation().evaluations, Some(3));
    let mut unobserved = charge(1);
    unobserved.observed.evaluations = None;
    assert!(matches!(
        ledger.charge(unobserved),
        Err(ProblemError::Unsupported(_))
    ));
    assert_eq!(ledger.observation().evaluations, None);
    let mut s = strategy();
    s.limits.evaluations = Some(2);
    let result = run::<()>(
        &s,
        &scope(),
        facts,
        |_, _| panic!("the complete reservation exceeds the task allowance before dispatch"),
        |_, _| panic!("exhausted work cannot be assessed or retained"),
    );
    assert_eq!(result.events.last().unwrap().kind, EventKind::Refused);
    assert_eq!(result.work.attempts, 0);
    assert_eq!(result.work.evaluations, Some(0));
    assert!(result.assessment.is_none());
    assert!(result.terminal.is_some());
}

#[test]
fn optional_slice_refusal_keeps_base_admitted_and_charges_only_actual_work() {
    let mut s = strategy();
    s.start.recovery.push(StartOrigin::Specification);
    s.mechanisms[0].position = Position::Preparation;
    s.mechanisms[0].required = false;
    s.mechanisms[0].limits.evaluations = Some(1);
    s.mechanisms[1].position = Position::Execution;
    s.mechanisms[1].starts = vec![StartOrigin::Specification];
    let result = run(
        &s,
        &scope(),
        |_| facts(0),
        |index, _| {
            assert_eq!(
                index, 1,
                "the optional complete reservation cannot fit its slice"
            );
            Ok(Attempt {
                value: index,
                observation: Observation::Converged,
                work: charge(index),
            })
        },
        |index, o| {
            assert_eq!(*index, 1);
            assessment(true, false, o)
        },
    );
    assert_eq!(result.value, Some(1));
    assert_eq!(result.work.attempts, 1);
    assert_eq!(result.work.evaluations, Some(3));
    assert!(result.terminal.is_none());
    assert!(
        result
            .events
            .iter()
            .any(|event| event.kind == EventKind::Refused
                && event.transition == Some(Transition::Continue))
    );
}
#[test]
fn late_indivisible_result_does_not_poison_user_cancel_or_grant_permission() {
    let cancel = Arc::new(AtomicBool::new(false));
    let enclosing =
        pse_kernels::ExecutionScope::new(cancel.clone(), Some(std::time::Instant::now()));
    let result = run(
        &strategy(),
        &enclosing,
        facts,
        |_, _| panic!("expired before dispatch"),
        |_: &usize, _| panic!("expired"),
    );
    assert!(result.terminal.is_some());
    assert!(!cancel.load(std::sync::atomic::Ordering::Acquire));
}
#[test]
fn late_native_batch_member_keeps_actual_work_and_cannot_grant_original_permission() {
    let enclosing = pse_kernels::ExecutionScope::new(
        Arc::new(AtomicBool::new(false)),
        Some(std::time::Instant::now()),
    );
    let declaration = NumericalStrategy::direct(
        pse_backend_native::solve::StartPolicy::NoPriorStart,
        limits(),
    );
    let result = run_observed(
        &declaration,
        &enclosing,
        |_| facts(0),
        |_, _| {
            Ok(Attempt {
                value: 7,
                observation: Observation::Converged,
                work: charge(0),
            })
        },
        |_, _| panic!("late observed native work cannot be assessed"),
    );
    assert_eq!(result.work.attempts, 1);
    assert_eq!(result.work.evaluations, Some(3));
    assert!(result.value.is_none());
    assert_eq!(result.events.last().unwrap().kind, EventKind::Abandoned);
    assert_eq!(result.events.last().unwrap().work, Some(charge(0)));
}

#[test]
fn indivisible_assessment_cannot_authorize_a_result_after_the_original_deadline() {
    let cancel = Arc::new(AtomicBool::new(false));
    let enclosing = pse_kernels::ExecutionScope::new(
        cancel.clone(),
        std::time::Instant::now().checked_add(std::time::Duration::from_millis(100)),
    );
    let result = run(
        &strategy(),
        &enclosing,
        facts,
        |index, _| {
            Ok(Attempt {
                value: index,
                observation: Observation::Converged,
                work: charge(index),
            })
        },
        |_, o| {
            std::thread::sleep(std::time::Duration::from_millis(110));
            assessment(true, false, o)
        },
    );
    assert!(result.assessment.is_none());
    assert!(result.value.is_none());
    assert!(result.terminal.is_some());
    assert!(!cancel.load(std::sync::atomic::Ordering::Acquire));
    assert!(result.events.iter().all(|event| event.permission.is_none()));
}

#[test]
fn preparation_first_uses_entry_then_auxiliary_correction_requires_declared_recovery() {
    let mut declaration = strategy();
    declaration.start.recovery = vec![StartOrigin::Auxiliary];
    declaration.mechanisms[0].position = Position::Preparation;
    declaration.mechanisms[0].kind = MechanismKind::BoundedFeasibility;
    declaration.mechanisms[0].transitions = vec![Transition::Continue, Transition::Stop];
    declaration.mechanisms[1].position = Position::Execution;
    declaration.mechanisms[1].starts = vec![StartOrigin::Auxiliary];
    let supplied = |index| {
        let mut f = facts(0);
        if index == 1 {
            f.start = StartOrigin::Auxiliary;
        }
        f
    };
    let mut visited = Vec::new();
    let result = run(
        &declaration,
        &scope(),
        supplied,
        |index, _| {
            visited.push(index);
            Ok(Attempt {
                value: index,
                observation: if index == 0 {
                    Observation::Auxiliary
                } else {
                    Observation::Converged
                },
                work: charge(index),
            })
        },
        |index, o| assessment(*index == 1, *index == 0, o),
    );
    assert_eq!(visited, [0, 1]);
    assert!(result.terminal.is_none());
    assert_eq!(
        result.events.last().unwrap().transition,
        Some(Transition::Finish)
    );
    assert_eq!(result.work.attempts, 2);
    // A preparation does not gain an exemption from the task's entry policy.
    let mut auxiliary = facts(0);
    auxiliary.start = StartOrigin::Auxiliary;
    assert!(matches!(
        admit(&declaration, 0, true, &auxiliary),
        Admission::RequiredRefusal(_)
    ));
    declaration.start.recovery.clear();
    let result = run(
        &declaration,
        &scope(),
        supplied,
        |index, _| {
            assert_eq!(index, 0);
            Ok(Attempt {
                value: index,
                observation: Observation::Auxiliary,
                work: charge(index),
            })
        },
        |_, o| assessment(false, true, o),
    );
    assert!(result.terminal.is_some());
    assert_eq!(result.work.attempts, 1);
    assert_eq!(result.events.last().unwrap().kind, EventKind::Refused);
}
#[test]
fn optional_preparation_refusal_preserves_specification_entry_for_first_dispatch() {
    let mut declaration = strategy();
    declaration.start.recovery.clear();
    declaration.mechanisms[0].position = Position::Preparation;
    declaration.mechanisms[0].kind = MechanismKind::BoundedFeasibility;
    declaration.mechanisms[0].required = false;
    declaration.mechanisms[0].support = vec![hash(19)];
    declaration.mechanisms[1].position = Position::Execution;
    declaration.mechanisms[1].starts = vec![StartOrigin::Specification];
    let result = run(
        &declaration,
        &scope(),
        |_| facts(0),
        |index, _| {
            assert_eq!(index, 1);
            Ok(Attempt {
                value: index,
                observation: Observation::Converged,
                work: charge(index),
            })
        },
        |_, o| assessment(true, false, o),
    );
    assert!(result.terminal.is_none());
    assert_eq!(result.value, Some(1));
    assert_eq!(result.work.attempts, 1);
    assert_eq!(result.events[1].kind, EventKind::Refused);
    assert_eq!(
        result.events.last().unwrap().transition,
        Some(Transition::Finish)
    );
}

#[test]
fn tagged_native_report_failures_are_terminal_under_declared_recovery() {
    use pse_backend_native::{
        NativeFailureKind, NativeStatus,
        solve::{
            Assurance, Backend, Controls, Execution, NativeTermination, SolveReport, Termination,
        },
    };
    let contract = pse_backend_native::OracleContract {
        identity: hash(29),
        derivatives: pse_kernels::DerivativeOrder::Value,
        smoothness: pse_kernels::DerivativeOrder::Value,
        variables: vec![],
        rows: vec![],
    };
    for (kind, expected) in [
        (NativeFailureKind::Resource, Observation::ResourceExhausted),
        (NativeFailureKind::Contract, Observation::ContractFailure),
        (
            NativeFailureKind::Infrastructure,
            Observation::OperationalFailure,
        ),
        (NativeFailureKind::Numerical, Observation::NumericalFailure),
    ] {
        let execution = Execution::new(Arc::new(AtomicBool::new(false)), &Controls::default());
        let mut report = SolveReport::new(
            Backend::Petsc,
            &contract,
            NativeTermination {
                code: 55,
                name: "native tagged status".into(),
                message: None,
                category: Termination::Numerical,
                assurance: Assurance::None,
            },
            &execution,
        );
        report.record_validation_failure(ProblemError::Native {
            status: NativeStatus {
                backend: Backend::Petsc,
                code: 55,
                name: "original tagged status".into(),
            },
            kind,
            detail: "native operation".into(),
        });
        assert_eq!(observe_native(&report), expected);
        assert_eq!(
            permits_numerical_continuation(expected),
            kind == NativeFailureKind::Numerical
        );
        let declaration = strategy();
        let result = run(
            &declaration,
            &scope(),
            facts,
            |index, _| {
                Ok(Attempt {
                    value: (),
                    observation: expected,
                    work: charge(index),
                })
            },
            |_, observation| {
                let mut assessment = assessment(false, false, observation);
                assessment.cause = cause_native(&report);
                assessment
            },
        );
        if kind != NativeFailureKind::Numerical {
            assert_eq!(
                result.events.last().unwrap().transition,
                Some(Transition::Stop)
            );
            assert_eq!(result.work.attempts, 1);
        }
        assert!(
            matches!(cause_native(&report).as_deref(),Some(ProblemError::Native {kind:actual,..}) if *actual==kind)
        );
    }
}

#[test]
fn native_convergence_retains_original_refusal_and_terminal_assessor_cause() {
    let cause = Arc::new(ProblemError::Contract(
        "authored safety check refused the original point".into(),
    ));
    let result = run(
        &strategy(),
        &scope(),
        facts,
        |index, _| {
            Ok(Attempt {
                value: index,
                observation: Observation::Converged,
                work: charge(index),
            })
        },
        |_, observation| {
            let mut assessed = assessment(false, false, observation);
            assessed.original = OriginalConclusion::Refused {
                cause: cause.clone(),
            };
            assessed
        },
    );
    let event = result.events.last().unwrap();
    assert_eq!(event.observation, Some(Observation::Converged));
    assert_eq!(event.transition, Some(Transition::Stop));
    assert!(Arc::ptr_eq(event.cause.as_ref().unwrap(), &cause));
    assert_eq!(result.work.attempts, 1);
}
#[test]
fn strict_unknown_inclusive_work_refuses_before_dispatch_and_keeps_complete_reservation_unknown() {
    let result = run::<()>(
        &strategy(),
        &scope(),
        |_| Facts {
            support: BTreeSet::new(),
            accuracy: Vec::new(),
            consumption: Vec::new(),
            reservation: None,
            start: StartOrigin::Specification,
            inherited: false,
            connected: false,
            refusal: None,
        },
        |_, _| panic!("unknown inclusive work under strict caps cannot dispatch"),
        |_, _| panic!("no assessment before admission"),
    );
    assert!(matches!(
        result.terminal.as_deref(),
        Some(ProblemError::Unsupported(_))
    ));
    assert_eq!(result.work.attempts, 0);
    let mut ledger = Ledger::new(limits());
    ledger.reserve(limits(), Some(charge(0).observed)).unwrap();
    let mut actual = charge(0);
    actual.observed.evaluations = None;
    ledger.charge(actual).unwrap();
    assert_eq!(ledger.observation().evaluations, None);
    let mut excessive = charge(1).observed;
    excessive.evaluations = Some(18);
    assert!(ledger.reserve(limits(), Some(excessive)).is_err());
}
#[test]
fn disjoint_scientific_assessment_work_is_charged_after_failed_original_check() {
    let result = run(
        &strategy(),
        &scope(),
        |index| {
            let mut f = facts(index);
            f.reservation.as_mut().unwrap().evaluations = Some(5);
            f
        },
        |index, _| {
            Ok(Attempt {
                value: index,
                observation: Observation::Converged,
                work: charge(index),
            })
        },
        |_, observation| {
            let mut assessed = assessment(false, false, observation);
            assessed.original = OriginalConclusion::Unavailable {
                cause: Arc::new(ProblemError::Internal(
                    "assessor failed after its callback".into(),
                )),
            };
            assessed.work.push(WorkCharge {
                phase: Phase::Assessment,
                scope: Scope::Task,
                charging_owner: hash(100),
                observed: WorkObservation {
                    attempts: 0,
                    evaluations: Some(2),
                    iterations: Some(0),
                    factorizations: Some(0),
                    proof_steps: Some(0),
                },
            });
            assessed
        },
    );
    assert_eq!(result.work.evaluations, Some(5));
    assert_eq!(result.work.attempts, 1);
    assert_eq!(
        result.events.last().unwrap().transition,
        Some(Transition::Stop)
    );
}
#[test]
fn actual_producer_state_consumes_exact_point_source_order_normalization_and_class() {
    use pse_model::strategy::*;
    let source = SemanticProductKey {
        structure: hash(1),
        binding: hash(2),
        numerical_policy: Some(hash(3)),
        normalization: Some(hash(4)),
        point: Some(hash(5)),
        parameters: Some(hash(6)),
        derivation: Some(hash(7)),
        branch: None,
        accuracy: Some(hash(8)),
    };
    let accuracy = AccuracyEvidence {
        product: hash(8),
        normalization: hash(4),
        class: AccuracyClass::Certified,
        error: Some(1e-8),
    };
    let actual = ProductEvidence {
        source,
        derivative_order: 0,
        branch: BranchPolicy::any_qualified(),
        accuracy,
    };
    let demand = AccuracyDemand {
        product: hash(8),
        normalization: hash(4),
        class: AccuracyClass::Certified,
        allowance: 1e-6,
    };
    let mut state = ProductState::default();
    assert!(
        state
            .bind_inputs(
                &[demand],
                hash(2),
                hash(5),
                BranchPolicy::any_qualified(),
                0
            )
            .is_err()
    );
    state.publish(actual.clone()).unwrap();
    let contract = state
        .bind_inputs(
            &[demand],
            hash(2),
            hash(5),
            BranchPolicy::any_qualified(),
            0,
        )
        .unwrap();
    assert_eq!(state.consume(&contract).unwrap(), vec![accuracy]);
    assert!(
        state
            .bind_inputs(
                &[demand],
                hash(2),
                hash(9),
                BranchPolicy::any_qualified(),
                0
            )
            .is_err()
    );
    assert!(
        state
            .bind_inputs(
                &[demand],
                hash(9),
                hash(5),
                BranchPolicy::any_qualified(),
                0
            )
            .is_err()
    );
    assert!(
        state
            .bind_inputs(
                &[demand],
                hash(2),
                hash(5),
                BranchPolicy::any_qualified(),
                1
            )
            .is_err()
    );
    let mut wrong = demand;
    wrong.normalization = hash(9);
    assert!(
        state
            .bind_inputs(&[wrong], hash(2), hash(5), BranchPolicy::any_qualified(), 0)
            .is_err()
    );
    let mut estimate = actual;
    estimate.accuracy.class = AccuracyClass::Estimated;
    let mut estimated_state = ProductState::default();
    estimated_state.publish(estimate).unwrap();
    assert!(
        estimated_state
            .bind_inputs(
                &[demand],
                hash(2),
                hash(5),
                BranchPolicy::any_qualified(),
                0
            )
            .is_err()
    );
}

#[test]
fn automatic_next_preserves_empty_start_grants_and_terminal_scientific_refusal() {
    use pse_model::strategy::{CompositionRequest, MechanismKind, StartPolicy, StartRules};
    let request = CompositionRequest {
        limits: Some(limits()),
        ..CompositionRequest::default()
    };
    let start = StartRules {
        policy: StartPolicy::NoPriorStart,
        recovery: Vec::new(),
    };
    let candidates = vec![
        AutoCandidate {
            identity: hash(1),
            kind: MechanismKind::ReducedSpace,
            start: StartOrigin::Auxiliary,
            replacement: true,
            support: BTreeSet::new(),
            reservation: None,
        },
        AutoCandidate {
            identity: hash(2),
            kind: MechanismKind::Direct,
            start: StartOrigin::Specification,
            replacement: false,
            support: BTreeSet::new(),
            reservation: None,
        },
    ];
    let work = WorkObservation {
        attempts: 0,
        evaluations: Some(0),
        iterations: Some(0),
        factorizations: Some(0),
        proof_steps: Some(0),
    };
    assert!(matches!(
        next_automatic(
            &request,
            &start,
            &candidates,
            &BTreeSet::new(),
            None,
            work,
            false
        ),
        AutoDecision::Dispatch { candidate: 1 }
    ));
    let refused = AutoObservation {
        native: Observation::Converged,
        original: Some(OriginalConclusion::Unavailable {
            cause: Arc::new(ProblemError::Internal(
                "scientific assessor unavailable".into(),
            )),
        }),
        permission: Some(CandidateUse::Unusable),
    };
    assert!(matches!(
        next_automatic(
            &request,
            &start,
            &candidates,
            &BTreeSet::new(),
            Some(&refused),
            work,
            false
        ),
        AutoDecision::Stop { .. }
    ));
    let mut attempted = BTreeSet::new();
    attempted.insert(1);
    assert!(matches!(
        next_automatic(&request, &start, &candidates, &attempted, None, work, false),
        AutoDecision::Stop { .. }
    ));
}
#[test]
fn preparation_refusal_does_not_consume_first_actual_execution_allowance() {
    let mut only = limits();
    only.attempts = 1;
    let ledger = Ledger::new(only);
    ledger.reserve_attempt().unwrap();
    ledger.reserve_attempt().unwrap();
    assert_eq!(ledger.observation().attempts, 0);
}
