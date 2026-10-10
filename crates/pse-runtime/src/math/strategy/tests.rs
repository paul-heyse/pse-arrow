// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::{math::SessionDisposition, workflow::numerics};
use pse_model::{
    generated::enums::{CandidateRefusal, CandidateUse},
    strategy::{MechanismKind, Scope},
};
use std::sync::atomic::AtomicBool;

fn export_service(pool: Arc<dyn pse_columnar::MemoryPool>) -> Arc<crate::math::MathService> {
    let native = pse_engine::cache_service::NativeCacheService::new(
        pse_engine::cache_service::CacheBudget::disabled(1),
        &pool,
    )
    .unwrap();
    crate::math::MathService::new(
        pool,
        Arc::new(tokio::sync::Semaphore::new(2)),
        2,
        Default::default(),
        &native,
    )
}

fn hash(n: u8) -> ContentHash {
    ContentHash::from_bytes([n; 32])
}

#[test]
fn event_export_admits_current_copy_before_projection() {
    use datafusion::execution::memory_pool::GreedyMemoryPool;
    use pse_columnar::MemoryPool;
    let mut trace = Trace {
        owner: None,
        publication_request: None,
        declaration: strategy(),
        original: hash(1),
        backend: None,
        profile: hash(2),
        start: StartOrigin::Specification,
        starts: Vec::new(),
        products: vec![RungProducts::default()],
        events: vec![Event {
            mechanism: 0,
            kind: EventKind::Refused,
            phase: Phase::Preparation,
            original: None,
            decision: None,
            observation: Some(Observation::CapabilityRefusal),
            transition: Some(Transition::Stop),
            permission: None,
            work: None,
            cause: Some(Arc::new(ProblemError::Unsupported(
                "retained cause".repeat(1024),
            ))),
        }],
    };
    let run = pse_model::generated::identities::RunId::from_bytes([8; 16]);
    let tiny: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(64));
    let tiny_service = export_service(tiny.clone());
    assert!(
        trace
            .rows(run, 0, &tiny_service, 0..usize::MAX)
            .unwrap()
            .next()
            .unwrap()
            .is_err()
    );
    assert_eq!(tiny.reserved(), 0);
    let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(1 << 20));
    let service = export_service(pool.clone());
    let mut rows = trace.rows(run, 0, &service, 0..usize::MAX).unwrap();
    assert_eq!(
        pool.reserved(),
        0,
        "constructing a lazy export performs no row work"
    );
    let row = rows.next().unwrap().unwrap();
    assert_eq!(row.event, 0);
    assert!(
        pool.reserved() > 0,
        "copy admission remains until publication and next demand"
    );
    drop(row);
    assert!(rows.next().is_none());
    drop(rows);
    assert_eq!(pool.reserved(), 0);
    trace.events.clear();
    assert!(
        trace
            .rows(run, 0, &tiny_service, 0..usize::MAX)
            .unwrap()
            .next()
            .is_none()
    );
}

#[test]
fn strategy_export_windows_skip_malformed_events_products_and_empty_ranges() {
    use pse_model::strategy::{
        AccuracyClass, AccuracyEvidence, BranchPolicy, ProductEvidence, SemanticProductKey,
    };
    let event = Event {
        mechanism: 0,
        kind: EventKind::Refused,
        phase: Phase::Preparation,
        original: None,
        decision: None,
        observation: None,
        transition: None,
        permission: None,
        work: None,
        cause: None,
    };
    let good = ProductEvidence {
        source: SemanticProductKey {
            structure: hash(1),
            binding: hash(2),
            numerical_policy: Some(hash(3)),
            normalization: Some(hash(4)),
            point: Some(hash(5)),
            parameters: None,
            derivation: None,
            branch: None,
            accuracy: Some(hash(8)),
        },
        derivative_order: 1,
        branch: BranchPolicy::any_qualified(),
        accuracy: AccuracyEvidence {
            product: hash(8),
            normalization: hash(4),
            class: AccuracyClass::Certified,
            error: Some(1e-8),
        },
    };
    let mut bad = good.clone();
    bad.source.point = None;
    let trace = Trace {
        owner: None,
        publication_request: None,
        declaration: strategy(),
        original: hash(1),
        backend: None,
        profile: hash(2),
        start: StartOrigin::Specification,
        starts: Vec::new(),
        products: vec![
            RungProducts {
                evidence: vec![bad],
                ..Default::default()
            },
            RungProducts {
                evidence: vec![good],
                ..Default::default()
            },
        ],
        events: vec![
            Event {
                mechanism: usize::MAX,
                ..event.clone()
            },
            event,
        ],
    };
    let run = pse_model::generated::identities::RunId::from_bytes([8; 16]);
    let pool: Arc<dyn pse_columnar::MemoryPool> = Arc::new(
        datafusion::execution::memory_pool::GreedyMemoryPool::new(1 << 20),
    );
    let service = export_service(pool.clone());
    assert_eq!(trace.event_count(), 2);
    assert_eq!(trace.product_count().unwrap(), 2);
    assert!(
        trace
            .rows(run, 0, &service, 0..1)
            .unwrap()
            .next()
            .unwrap()
            .is_err()
    );
    let rows = trace
        .rows(run, 0, &service, 1..2)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].event, 1);
    assert!(
        trace
            .product_rows(run, 0, 0..1)
            .unwrap()
            .next()
            .unwrap()
            .is_err()
    );
    let rows = trace
        .product_rows(run, 0, 1..2)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!((rows[0].mechanism, rows[0].product), (1, 0));
    for range in [0..0, 1..1, 2..2, 50..100] {
        assert!(
            trace
                .rows(run, 0, &service, range.clone())
                .unwrap()
                .next()
                .is_none()
        );
        assert!(trace.product_rows(run, 0, range).unwrap().next().is_none());
    }
    assert_eq!(pool.reserved(), 0);
}

#[tokio::test]
async fn engineering_refinement_original_success_stops_catalog_without_promoting_goal_permission() {
    let declaration = strategy();
    let last = AutoObservation {
        awaiting_assessment: false,
        native: Observation::Converged,
        original: Some(OriginalConclusion::Satisfied),
        permission: Some(CandidateUse::Unusable),
    };
    let request = pse_model::strategy::CompositionRequest::default();
    assert!(matches!(
        next_automatic(
            &request,
            &declaration.start,
            &[],
            &BTreeSet::new(),
            Some(&last),
            charge(0).observed,
            false,
        ),
        AutoDecision::Finish
    ));
    assert_eq!(last.permission, Some(CandidateUse::Unusable));
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
#[tokio::test]
async fn automatic_refusal_only_trace_publishes_without_admitting_empty_execution() {
    use crate::workflow::tests as fixture;
    let runtime = fixture::runtime();
    let declarations = pse_authoring::language::parse(
        "package p {def Constant {param p:Scalar=1;eq check:p==1;}}",
        pse_ids::SemanticId::NIL,
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap();
    let root = declarations
        .iter()
        .find(|row| row.name == "Constant")
        .unwrap()
        .declaration_id;
    let package = runtime
        .modeling_package(declarations, fixture::physical())
        .await
        .unwrap();
    let prepared = package
        .prepare_solve(
            root,
            pse_modeling::specialize::root_instance(root),
            Default::default(),
            Default::default(),
            Default::default(),
            pse_kernels::DerivativeOrder::Value,
            fixture::compiler_profile(),
            fixture::profile(),
            Default::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let request = prepared.solve.request_identity().unwrap();
    let mut observed = prepared.solve.numerical_strategy();
    observed.mechanisms.clear();
    assert!(observed.key().is_err());
    assert!(
        prepared
            .solve
            .clone()
            .with_strategy(observed.clone(), Vec::new())
            .is_err()
    );
    let mut trace = Trace {
        owner: None,
        publication_request: Some(request),
        declaration: observed,
        original: prepared.solve.original_identity().unwrap(),
        backend: prepared.solve.backend(),
        profile: prepared.solve.strategy_profile().unwrap(),
        start: StartOrigin::Specification,
        starts: Vec::new(),
        products: Vec::new(),
        events: Vec::new(),
    };
    let run_id = pse_operations::mint_id();
    assert!(
        trace
            .rows(
                run_id,
                0,
                &export_service(Arc::new(
                    datafusion::execution::memory_pool::UnboundedMemoryPool::default()
                )),
                0..usize::MAX
            )
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
            .is_empty()
    );
    assert!(
        trace
            .product_rows(run_id, 0, 0..usize::MAX)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
            .is_empty()
    );
    let mut preparation = prepared.solve.numerical_strategy().mechanisms.remove(0);
    preparation.position = Position::Preparation;
    preparation.kind = MechanismKind::BoundedFeasibility;
    preparation.required = false;
    trace.declaration.mechanisms.push(preparation);
    trace.starts.push(StartOrigin::Specification);
    trace.products.push(RungProducts::default());
    let cause = Arc::new(ProblemError::Unsupported(
        "required proof count unavailable before preparation".into(),
    ));
    trace.events.push(Event {
        mechanism: 0,
        kind: EventKind::Refused,
        phase: Phase::Preparation,
        original: Some(OriginalConclusion::Unavailable {
            cause: cause.clone(),
        }),
        decision: None,
        observation: Some(Observation::CapabilityRefusal),
        transition: Some(Transition::Stop),
        permission: Some(CandidateUse::Unusable),
        work: None,
        cause: Some(cause),
    });
    let rows = trace
        .rows(
            run_id,
            0,
            &export_service(Arc::new(
                datafusion::execution::memory_pool::UnboundedMemoryPool::default(),
            )),
            0..usize::MAX,
        )
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].strategy_identity, request.as_id());
    assert_eq!(rows[0].kind, EventKind::Refused);
    assert!(rows[0].backend.is_none());
    assert!(rows[0].profile_identity.is_none());
    assert!(
        rows[0].attempts.is_none(),
        "no execution or proof work was dispatched"
    );
    let pool = runtime.shared.pool();
    let cancelled = pse_columnar::CancellationToken::new();
    let validation = runtime.validation_context().unwrap();
    let mut published =
        pse_relations::columnar::Collection::new(&runtime.registry, &pool, &cancelled, &validation);
    published
        .ensure::<pse_model::generated::runtime::solve_strategy_events::Row>()
        .unwrap();
    published.push(rows[0].clone()).unwrap();
    let batches = published.finish().unwrap();
    assert_eq!(
        batches
            .values()
            .map(|batch| batch.batch().num_rows())
            .sum::<usize>(),
        1
    );
    assert!(trace.declaration.key().is_err());
    assert!(
        prepared
            .solve
            .clone()
            .with_strategy(
                trace.declaration.clone(),
                vec![prepared.solve.clone().into()]
            )
            .is_err()
    );
    trace.publication_request = None;
    assert!(
        trace
            .rows(
                run_id,
                0,
                &export_service(Arc::new(
                    datafusion::execution::memory_pool::UnboundedMemoryPool::default()
                )),
                0..usize::MAX
            )
            .is_err(),
        "explicit declaration publication retains admission validation"
    );
}
fn facts(index: usize) -> Facts {
    Facts {
        support: BTreeSet::new(),
        accuracy: Vec::new(),
        consumption: Vec::new(),
        reservation: Some(charge(index).observed),
        work_admitted: false,
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
#[tokio::test]
async fn failed_trajectory_can_run_declared_same_backend_profile_and_only_original_permission_finishes()
 {
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
                evidence: Vec::new(),
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
#[tokio::test]
async fn failed_dispatched_effect_is_charged_once_and_unknown_work_stays_unknown() {
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

#[tokio::test]
async fn failed_screening_keeps_actual_partial_callback_count() {
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
#[tokio::test]
async fn declared_accuracy_is_consumed_and_an_estimate_cannot_establish_certification() {
    use pse_model::strategy::{AccuracyClass, AccuracyDemand, AccuracyEvidence};
    let mut declaration = strategy();
    let demand = AccuracyDemand {
        product: hash(7),
        normalization: hash(8),
        allowance: 1e-6,
        class: AccuracyClass::Certified,
    };
    declaration.mechanisms[0]
        .operation
        .inputs
        .push(pse_model::strategy::ProductDemand {
            source: pse_model::strategy::SemanticProductKey {
                structure: hash(1),
                binding: hash(2),
                numerical_policy: None,
                normalization: Some(hash(8)),
                point: Some(hash(3)),
                parameters: None,
                derivation: None,
                branch: None,
                accuracy: Some(hash(7)),
            },
            derivative_order: 0,
            branch: pse_model::strategy::BranchPolicy::any_qualified(),
            accuracy: demand,
        });
    let mut supplied = facts(0);
    supplied.consumption = vec![demand];
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
#[tokio::test]
async fn optional_capability_refusal_preserves_base_required_contract_failure_does_not_retry() {
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
                evidence: Vec::new(),
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
#[tokio::test]
async fn limited_without_stagnation_does_not_invent_failure_or_recovery() {
    let s = strategy();
    let result = run(
        &s,
        &scope(),
        facts,
        |index, _| {
            Ok(Attempt {
                evidence: Vec::new(),
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
#[tokio::test]
async fn native_success_and_auxiliary_success_cannot_bypass_original_permission() {
    let s = strategy();
    for auxiliary in [false, true] {
        let result = run(
            &s,
            &scope(),
            facts,
            |index, _| {
                Ok(Attempt {
                    evidence: Vec::new(),
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
#[tokio::test]
async fn actual_work_is_charged_once_unknown_is_not_zero_and_task_cap_is_terminal() {
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

#[tokio::test]
async fn contained_native_panic_is_terminal_while_original_numerical_cause_is_retained() {
    use pse_backend_native::solve::{
        Assurance, Backend, Controls, Execution, NativeTermination, SolveReport, Termination,
    };
    let contract = pse_backend_native::OracleContract {
        identity: hash(29),
        derivatives: pse_kernels::DerivativeOrder::Value,
        smoothness: pse_kernels::DerivativeOrder::Value,
        variables: vec![],
        rows: vec![],
    };
    let execution = Execution::new(Arc::new(AtomicBool::new(false)), &Controls::default());
    let mut report = SolveReport::new(
        Backend::Pounce,
        &contract,
        NativeTermination {
            code: i64::MIN,
            name: "rust.unwind".into(),
            message: None,
            category: Termination::Panic,
            assurance: Assurance::None,
        },
        &execution,
    );
    // An original numerical refusal cannot turn a contained unwind into recovery.
    report.record_validation_failure(ProblemError::numerical("failed derivative probe"));
    assert_eq!(observe_native(&report), Observation::Panic);
    assert!(cause_native(&report).is_some());
}

#[tokio::test]
async fn optional_slice_refusal_keeps_base_admitted_and_charges_only_actual_work() {
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
                evidence: Vec::new(),
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
#[tokio::test]
async fn late_indivisible_result_does_not_poison_user_cancel_or_grant_permission() {
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
#[tokio::test]
async fn late_native_batch_member_keeps_actual_work_and_cannot_grant_original_permission() {
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
                evidence: Vec::new(),
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

#[tokio::test]
async fn indivisible_assessment_cannot_authorize_a_result_after_the_original_deadline() {
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
                evidence: Vec::new(),
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

#[tokio::test]
async fn preparation_first_uses_entry_then_auxiliary_correction_requires_declared_recovery() {
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
                evidence: Vec::new(),
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
    // Fresh declared recovery is lawful; incoming Explicit keeps hard precedence.
    let mut auxiliary = facts(0);
    auxiliary.start = StartOrigin::Auxiliary;
    declaration.mechanisms[0]
        .starts
        .push(StartOrigin::Auxiliary);
    assert!(matches!(
        admit(&declaration, 0, true, &auxiliary),
        Admission::Ready
    ));
    declaration.start.policy = pse_model::strategy::StartPolicy::Explicit;
    assert!(matches!(
        admit(&declaration, 0, true, &auxiliary),
        Admission::RequiredRefusal(_)
    ));
    declaration.start.policy = pse_model::strategy::StartPolicy::NoPriorStart;
    declaration.start.recovery.clear();
    let result = run(
        &declaration,
        &scope(),
        supplied,
        |index, _| {
            assert_eq!(index, 0);
            Ok(Attempt {
                evidence: Vec::new(),
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
#[tokio::test]
async fn optional_preparation_refusal_preserves_specification_entry_for_first_dispatch() {
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
                evidence: Vec::new(),
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

#[tokio::test]
async fn compiler_derivative_support_limit_retains_resource_observation_and_stops() {
    let source = pse_ids::SemanticId::from_bytes([17; 16]);
    let error = crate::math::MathRuntimeError::Compile(
        pse_compiler::workspace::CompileError::from(pse_math::MathError::WorkLimit {
            source_id: source,
            resource: "derivative support construction",
            required: 1,
            available: 0,
            components: 0,
        }),
    );
    let diagnostic = error.boundary_diagnostic(pse_diagnostics::DiagnosticStage::Native);
    assert_eq!(
        diagnostic.class,
        pse_model::diagnostic::BoundaryClass::ResourceLimit
    );
    assert_eq!(diagnostic.sources, vec![source]);
    let observation = runtime_failure(&error);
    assert_eq!(observation, Observation::ResourceExhausted);
    let cause = Arc::new(error.into_problem());
    assert_eq!(failure(&cause), observation);
    let preserved = cause.boundary_diagnostic(pse_diagnostics::DiagnosticStage::Native);
    assert_eq!(preserved.class, diagnostic.class);
    assert_eq!(preserved.sources, diagnostic.sources);
    let mut assessed = assessment(false, false, observation);
    assessed.cause = Some(cause.clone());
    assessed.original = OriginalConclusion::Unavailable { cause };
    let mut preparation = strategy().mechanisms.remove(0);
    preparation.position = Position::Preparation;
    preparation.required = false;
    assert_eq!(transition(&preparation, &assessed), Transition::Stop);
    assert!(!permits_numerical_continuation(observation));
    let invalid = crate::math::MathRuntimeError::Compile(
        pse_compiler::workspace::CompileError::Missing("authored compiler input".into()),
    );
    assert_eq!(runtime_failure(&invalid), Observation::ContractFailure);
    assert_eq!(
        failure(&invalid.into_problem()),
        Observation::ContractFailure
    );
}

#[tokio::test]
async fn tagged_native_report_failures_are_terminal_under_declared_recovery() {
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
                    evidence: Vec::new(),
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

#[tokio::test]
async fn native_convergence_retains_original_refusal_and_terminal_assessor_cause() {
    let cause = Arc::new(ProblemError::Contract(
        "authored safety check refused the original point".into(),
    ));
    let result = run(
        &strategy(),
        &scope(),
        facts,
        |index, _| {
            Ok(Attempt {
                evidence: Vec::new(),
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
#[tokio::test]
async fn strict_unknown_inclusive_work_refuses_before_dispatch_and_keeps_complete_reservation_unknown()
 {
    let result = run::<()>(
        &strategy(),
        &scope(),
        |_| Facts {
            support: BTreeSet::new(),
            accuracy: Vec::new(),
            consumption: Vec::new(),
            reservation: None,
            work_admitted: false,
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
#[tokio::test]
async fn disjoint_scientific_assessment_work_is_charged_after_failed_original_check() {
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
                evidence: Vec::new(),
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
#[tokio::test]
async fn actual_producer_state_consumes_exact_point_source_order_normalization_and_class() {
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
    let contract = OperationContract {
        inputs: vec![ProductDemand {
            source,
            derivative_order: 0,
            branch: actual.branch,
            accuracy: demand,
        }],
        outputs: Vec::new(),
    };
    let mut state = ProductState::default();
    assert!(state.consume(&contract).is_err());
    state.publish(actual.clone()).unwrap();
    assert_eq!(state.consume(&contract).unwrap(), vec![accuracy]);
    let reject = |dependency: ProductDemand| {
        assert!(
            state
                .consume(&OperationContract {
                    inputs: vec![dependency],
                    outputs: Vec::new()
                })
                .is_err()
        );
    };
    let original = contract.inputs[0].clone();
    let mut wrong = original.clone();
    wrong.source.point = Some(hash(9));
    reject(wrong);
    let mut wrong = original.clone();
    wrong.source.binding = hash(9);
    reject(wrong);
    let mut wrong = original.clone();
    wrong.derivative_order = 1;
    reject(wrong);
    let mut wrong = original.clone();
    wrong.accuracy.normalization = hash(9);
    reject(wrong);
    let mut estimated = actual;
    estimated.accuracy.class = AccuracyClass::Estimated;
    let mut estimated_state = ProductState::default();
    estimated_state.publish(estimated).unwrap();
    assert!(estimated_state.consume(&contract).is_err());
}

#[tokio::test]
async fn automatic_next_preserves_empty_start_grants_and_terminal_scientific_refusal() {
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
            prepared: false,
        },
        AutoCandidate {
            identity: hash(2),
            kind: MechanismKind::Direct,
            start: StartOrigin::Specification,
            replacement: false,
            support: BTreeSet::new(),
            reservation: None,
            prepared: true,
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
        awaiting_assessment: false,
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
#[tokio::test]
async fn preparation_refusal_does_not_consume_first_actual_execution_allowance() {
    let mut only = limits();
    only.attempts = 1;
    let ledger = Ledger::new(only);
    ledger.reserve_attempt().unwrap();
    ledger.reserve_attempt().unwrap();
    assert_eq!(ledger.observation().attempts, 0);
}
#[tokio::test]
async fn automatic_preparation_is_named_and_its_execution_requires_remaining_allowance() {
    let request = pse_model::strategy::CompositionRequest::default();
    let start = pse_model::strategy::StartRules {
        policy: pse_model::strategy::StartPolicy::NoPriorStart,
        recovery: Vec::new(),
    };
    let mut candidates = [AutoCandidate {
        identity: hash(1),
        kind: MechanismKind::ReducedSpace,
        start: StartOrigin::Specification,
        replacement: false,
        support: BTreeSet::new(),
        reservation: None,
        prepared: false,
    }];
    let mut work = WorkObservation {
        attempts: 0,
        evaluations: Some(0),
        iterations: Some(0),
        factorizations: Some(0),
        proof_steps: Some(0),
    };
    let prepare = next_automatic(
        &request,
        &start,
        &candidates,
        &BTreeSet::new(),
        None,
        work,
        false,
    );
    assert!(matches!(prepare, AutoDecision::Prepare { candidate: 0 }));
    let before = automatic_decision_key(&request, &candidates, &prepare, None, work).unwrap();
    candidates[0].prepared = true;
    work.attempts = 1;
    // The admitted catalog includes both the preparation and its execution.
    let mut constrained = request.clone();
    constrained.limits = Some(WorkLimits {
        attempts: 2,
        evaluations: None,
        iterations: None,
        factorizations: None,
        proof_steps: None,
    });
    let dispatch = next_automatic(
        &constrained,
        &start,
        &candidates,
        &BTreeSet::new(),
        None,
        work,
        false,
    );
    assert!(matches!(dispatch, AutoDecision::Dispatch { candidate: 0 }));
    assert_ne!(
        before,
        automatic_decision_key(&constrained, &candidates, &dispatch, None, work).unwrap()
    );
    work.attempts = 2;
    assert!(matches!(
        next_automatic(
            &constrained,
            &start,
            &candidates,
            &BTreeSet::new(),
            None,
            work,
            false
        ),
        AutoDecision::Stop { .. }
    ));
}

#[tokio::test]
async fn optional_component_numerical_failure_retains_observation_and_charges_before_direct() {
    for observed in [
        Observation::NumericalFailure,
        Observation::Stalled,
        Observation::Limited,
        Observation::ContractFailure,
        Observation::Cancelled,
        Observation::ResourceExhausted,
    ] {
        let mut declaration = strategy();
        declaration.start.recovery.push(StartOrigin::Specification);
        declaration.mechanisms[0].required = false;
        declaration.mechanisms[0]
            .transitions
            .push(Transition::Continue);
        declaration.mechanisms[1].starts = vec![StartOrigin::Specification];
        let mut visited = Vec::new();
        let result = run(
            &declaration,
            &scope(),
            |_| facts(0),
            |index, _| {
                visited.push(index);
                if index == 0 {
                    return Err(EffectFailure::component(
                        Arc::new(ProblemError::numerical(
                            "failed optional numerical component",
                        )),
                        charge(index).observed,
                        observed,
                    ));
                }
                Ok(Attempt {
                    evidence: Vec::new(),
                    value: index,
                    observation: Observation::Converged,
                    work: charge(index),
                })
            },
            |_, observation| assessment(true, false, observation),
        );
        assert_eq!(
            result
                .events
                .iter()
                .find(|event| event.work.is_some())
                .unwrap()
                .observation,
            Some(observed)
        );
        if permits_numerical_continuation(observed) {
            assert_eq!(visited, vec![0, 1]);
            assert_eq!(result.value, Some(1));
            assert_eq!(result.work.attempts, 2);
            assert_eq!(result.work.evaluations, Some(6));
        } else {
            assert_eq!(visited, vec![0]);
            assert!(result.terminal.is_some());
        }
    }
}

#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn abandoned_block_native_refusal_reaches_direct_through_production_projection() {
    use pse_backend_native::{self as native, solve::*};
    use pse_ids::SemanticId;
    use pse_model::strategy::{CompositionRequest, StartPolicy};

    let column = SemanticId::from_bytes([1; 16]);
    let row = SemanticId::from_bytes([2; 16]);
    let boundary = pse_structural::initialization::Block {
        id: pse_structural::incidence::BlockId(hash(1)),
        members: pse_structural::incidence::Part {
            rows: vec![row],
            columns: vec![column],
        },
        inputs: vec![],
    };
    let contract = native::OracleContract {
        identity: hash(2),
        variables: vec![native::Variable {
            id: column,
            lower: f64::NEG_INFINITY,
            upper: f64::INFINITY,
        }],
        rows: vec![row],
        derivatives: pse_kernels::DerivativeOrder::First,
        smoothness: pse_kernels::DerivativeOrder::First,
    };
    let mut report = SolveReport::new(
        Backend::Ipopt,
        &contract,
        NativeTermination {
            code: 2,
            name: "Infeasible_Problem_Detected".into(),
            message: None,
            category: Termination::Infeasible,
            assurance: Assurance::None,
        },
        &Execution::new(Arc::default(), &Controls::default()),
    );
    report.candidate = Some(Candidate {
        kind: CandidateKind::FinalIterate,
        primal: vec![2.0],
        objective: None,
        row_dual: None,
        bound_dual: None,
        reduced_costs: None,
        slacks: None,
        commitment: None,
    });
    report.quality = Some(
        native::quality::Quality::new(
            vec![native::quality::Violation {
                id: row,
                physical: 2.0,
                tolerance: 1.0,
            }],
            vec![],
            vec![],
        )
        .unwrap(),
    );
    let policy = pse_model::numerics::NumericalPolicy::default();
    native::quality::qualify(
        &mut report,
        &ResolvedAccuracy::from_policy(&policy, pse_model::numerics::DEFAULT_ENGINEERING_ACCURACY)
            .unwrap(),
    );

    let mut declaration = NumericalStrategy::direct(StartPolicy::NoPriorStart, limits());
    declaration.mechanisms[0].kind = MechanismKind::Block;
    declaration.mechanisms[0].required = false;
    declaration.mechanisms[0]
        .transitions
        .push(Transition::Continue);
    let candidates = [MechanismKind::Block, MechanismKind::Direct].map(|kind| AutoCandidate {
        identity: if kind == MechanismKind::Block {
            hash(3)
        } else {
            hash(4)
        },
        kind,
        start: StartOrigin::Specification,
        replacement: false,
        support: BTreeSet::new(),
        reservation: None,
        prepared: true,
    });
    for missing_candidate in [false, true] {
        let mut native = report.clone();
        if missing_candidate {
            native.candidate = None;
            native.quality = None;
        }
        let mut values = pse_math::binding::CaseValues {
            scalars: std::collections::BTreeMap::from([(column, 1.0)]),
        };
        let cause = crate::math::initialization::commit_block(
            &mut values,
            &boundary,
            Some(&native),
            &policy,
        )
        .unwrap_err();
        assert!(matches!(cause.as_ref(), ProblemError::Numerical { .. }));
        assert_eq!(values.scalars[&column], 1.0);
        assert_eq!(observe_native(&native), Observation::Limited);
        let failed = run::<()>(
            &declaration,
            &scope(),
            |_| facts(0),
            |_, _| {
                Err(EffectFailure::component(
                    cause.clone(),
                    charge(0).observed,
                    observe_native(&native),
                ))
            },
            |_, _| panic!("failed block cannot be accepted or assessed as an original result"),
        );
        assert!(failed.value.is_none());
        assert_eq!(failed.work.attempts, 1);
        let event = failed.events.last().unwrap();
        assert_eq!(event.kind, EventKind::Abandoned);
        assert_eq!(event.transition, Some(Transition::Continue));
        let last = automatic_observation(&failed.events).unwrap();
        assert!(Arc::ptr_eq(
            &last.original.as_ref().unwrap().cause().unwrap(),
            &cause
        ));
        assert!(matches!(
            next_automatic(
                &CompositionRequest::default(),
                &declaration.start,
                &candidates,
                &BTreeSet::from([0]),
                Some(&last),
                failed.work,
                false,
            ),
            AutoDecision::Dispatch { candidate: 1 },
        ));
    }

    // Actual terminal component witnesses still stop before any Direct operation.
    for cause in [
        ProblemError::Contract("coordinate contract".into()),
        ProblemError::memory("component allocation"),
        ProblemError::Cancelled,
    ] {
        let cause = Arc::new(cause);
        let failed = run::<()>(
            &declaration,
            &scope(),
            |_| facts(0),
            |_, _| {
                Err(EffectFailure::component(
                    cause.clone(),
                    charge(0).observed,
                    failure(&cause),
                ))
            },
            |_, _| panic!("terminal component cannot be accepted"),
        );
        assert!(Arc::ptr_eq(failed.terminal.as_ref().unwrap(), &cause));
        let last = automatic_observation(&failed.events).unwrap();
        assert!(matches!(
            next_automatic(
                &CompositionRequest::default(),
                &declaration.start,
                &candidates,
                &BTreeSet::from([0]),
                Some(&last),
                failed.work,
                false,
            ),
            AutoDecision::Stop { .. },
        ));
    }
}

#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn malformed_component_metadata_stops_explicit_direct_despite_native_observation() {
    use pse_backend_native::{self as native, solve::*};
    use pse_ids::SemanticId;

    let column = SemanticId::from_bytes([1; 16]);
    let row = SemanticId::from_bytes([2; 16]);
    let boundary = pse_structural::initialization::Block {
        id: pse_structural::incidence::BlockId(hash(1)),
        members: pse_structural::incidence::Part {
            rows: vec![row],
            columns: vec![column],
        },
        inputs: vec![],
    };
    let contract = native::OracleContract {
        identity: hash(2),
        variables: vec![native::Variable {
            id: column,
            lower: f64::NEG_INFINITY,
            upper: f64::INFINITY,
        }],
        rows: vec![row],
        derivatives: pse_kernels::DerivativeOrder::First,
        smoothness: pse_kernels::DerivativeOrder::First,
    };
    let mut declaration = strategy();
    declaration.mechanisms[0].kind = MechanismKind::Block;
    declaration.mechanisms[0].required = false;
    declaration.mechanisms[0]
        .transitions
        .push(Transition::Continue);
    declaration.mechanisms[1].starts = vec![StartOrigin::Specification];
    declaration.start.recovery.push(StartOrigin::Specification);
    let policy = pse_model::numerics::NumericalPolicy::default();
    let stop = |cause: Arc<ProblemError>, observed: Observation| {
        let mut visited = Vec::new();
        let failed = run::<()>(
            &declaration,
            &scope(),
            |_| facts(0),
            |index, _| {
                visited.push(index);
                Err(EffectFailure::component(
                    cause.clone(),
                    charge(0).observed,
                    observed,
                ))
            },
            |_, _| panic!("malformed component must not grant original permission"),
        );
        assert_eq!(
            visited,
            vec![0],
            "Direct must not dispatch after a terminal actual cause"
        );
        assert!(Arc::ptr_eq(failed.terminal.as_ref().unwrap(), &cause));
        assert!(failed.value.is_none());
        assert_eq!(failed.work.attempts, 1);
        assert_eq!(failed.work.evaluations, Some(3));
        let event = failed.events.last().unwrap();
        assert_eq!(event.transition, Some(Transition::Stop));
        assert_eq!(event.observation, Some(observed));
        assert!(Arc::ptr_eq(event.cause.as_ref().unwrap(), &cause));
    };
    for category in [Termination::Success, Termination::Limit] {
        let mut report = SolveReport::new(
            Backend::Ipopt,
            &contract,
            NativeTermination {
                code: 0,
                name: "fixture native outcome".into(),
                message: None,
                category,
                assurance: Assurance::None,
            },
            &Execution::new(Arc::default(), &Controls::default()),
        );
        report.candidate = Some(Candidate {
            kind: CandidateKind::FinalIterate,
            primal: vec![],
            objective: None,
            row_dual: None,
            bound_dual: None,
            reduced_costs: None,
            slacks: None,
            commitment: None,
        });
        report.quality = Some(native::quality::Quality::new(vec![], vec![], vec![]).unwrap());
        native::quality::qualify(
            &mut report,
            &ResolvedAccuracy::from_policy(
                &policy,
                pse_model::numerics::DEFAULT_ENGINEERING_ACCURACY,
            )
            .unwrap(),
        );
        let mut values = pse_math::binding::CaseValues {
            scalars: std::collections::BTreeMap::from([(column, 1.0)]),
        };
        let cause = crate::math::initialization::commit_block(
            &mut values,
            &boundary,
            Some(&report),
            &policy,
        )
        .unwrap_err();
        assert!(matches!(cause.as_ref(), ProblemError::Contract(_)));
        assert_eq!(values.scalars[&column], 1.0);
        let observed = observe_native(&report);
        assert_eq!(
            observed,
            if category == Termination::Success {
                Observation::Converged
            } else {
                Observation::Limited
            }
        );
        stop(cause, observed);
    }
    for cause in [
        ProblemError::memory("actual component allocation"),
        ProblemError::Cancelled,
    ] {
        stop(Arc::new(cause), Observation::Limited);
    }
}

#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn effective_original_failure_controls_commit_auto_and_explicit_terminal_paths() {
    use pse_backend_native::{self as native, callback::CallbackState, solve::*};
    use pse_ids::SemanticId;
    use pse_model::strategy::CompositionRequest;

    let column = SemanticId::from_bytes([1; 16]);
    let row = SemanticId::from_bytes([2; 16]);
    let boundary = pse_structural::initialization::Block {
        id: pse_structural::incidence::BlockId(hash(1)),
        members: pse_structural::incidence::Part {
            rows: vec![row],
            columns: vec![column],
        },
        inputs: vec![],
    };
    let contract = native::OracleContract {
        identity: hash(2),
        variables: vec![native::Variable {
            id: column,
            lower: f64::NEG_INFINITY,
            upper: f64::INFINITY,
        }],
        rows: vec![row],
        derivatives: pse_kernels::DerivativeOrder::First,
        smoothness: pse_kernels::DerivativeOrder::First,
    };
    let with_callback = |cause: ProblemError| {
        let execution = Execution::new(Arc::default(), &Controls::default());
        let mut report = SolveReport::new(
            Backend::Kinsol,
            &contract,
            NativeTermination {
                code: -13,
                name: "callback exit fixture".into(),
                message: None,
                category: Termination::Evaluation,
                assurance: Assurance::None,
            },
            &execution,
        );
        let mut callbacks = CallbackState::new(execution);
        assert!(
            callbacks
                .evaluate::<()>("residual", || Err(cause))
                .is_none()
        );
        callbacks.finish(&mut report);
        report
    };
    let trial = || {
        ProblemError::Math(pse_math::MathError::Domain {
            source_id: row,
            requirement: "trial domain",
        })
    };
    let mut declaration = strategy();
    declaration.mechanisms[0].kind = MechanismKind::Block;
    declaration.mechanisms[0].required = false;
    declaration.mechanisms[0]
        .transitions
        .push(Transition::Continue);
    declaration.mechanisms[1].starts = vec![StartOrigin::Specification];
    declaration.start.recovery.push(StartOrigin::Specification);
    let candidates = [MechanismKind::Block, MechanismKind::Direct].map(|kind| AutoCandidate {
        identity: if kind == MechanismKind::Block {
            hash(3)
        } else {
            hash(4)
        },
        kind,
        start: StartOrigin::Specification,
        replacement: false,
        support: BTreeSet::new(),
        reservation: None,
        prepared: true,
    });
    let terminal = |report: &SolveReport, expected: &Arc<ProblemError>, observed: Observation| {
        assert_eq!(observe_native(report), observed);
        assert!(Arc::ptr_eq(&cause_native(report).unwrap(), expected));
        let mut values = pse_math::binding::CaseValues {
            scalars: std::collections::BTreeMap::from([(column, 1.0)]),
        };
        let cause = crate::math::initialization::commit_block(
            &mut values,
            &boundary,
            Some(report),
            &Default::default(),
        )
        .unwrap_err();
        assert!(Arc::ptr_eq(&cause, expected));
        assert_eq!(values.scalars[&column], 1.0);
        let mut visited = Vec::new();
        let result = run::<()>(
            &declaration,
            &scope(),
            |_| facts(0),
            |index, _| {
                visited.push(index);
                Err(EffectFailure::component(
                    cause.clone(),
                    charge(0).observed,
                    observed,
                ))
            },
            |_, _| panic!("terminal original source cannot grant permission"),
        );
        assert_eq!(visited, vec![0]);
        assert!(Arc::ptr_eq(result.terminal.as_ref().unwrap(), expected));
        assert_eq!(result.work.attempts, 1);
        assert_eq!(
            result.events.last().unwrap().transition,
            Some(Transition::Stop)
        );
        let last = automatic_observation(&result.events).unwrap();
        assert!(matches!(
            next_automatic(
                &CompositionRequest::default(),
                &declaration.start,
                &candidates,
                &BTreeSet::from([0]),
                Some(&last),
                result.work,
                false,
            ),
            AutoDecision::Stop { .. }
        ));
    };
    for late_terminal_flag in [false, true] {
        for (validation, observed) in [
            (
                ProblemError::Contract("original contract".into()),
                Observation::ContractFailure,
            ),
            (
                ProblemError::memory("original validation allocation"),
                Observation::ResourceExhausted,
            ),
            (ProblemError::Cancelled, Observation::Cancelled),
        ] {
            let mut report = with_callback(trial());
            let callback = report.shared_callback_failure().unwrap();
            assert!(!report.evidence.callback.terminal_failure);
            report.record_validation_failure(validation);
            report.evidence.callback.terminal_failure = late_terminal_flag;
            let validation = report.shared_validation_failure().unwrap();
            terminal(&report, &validation, observed);
            assert!(Arc::ptr_eq(
                &report.shared_callback_failure().unwrap(),
                &callback
            ));
        }
    }
    let mut report = with_callback(ProblemError::Contract("terminal callback contract".into()));
    let callback = report.shared_callback_failure().unwrap();
    assert!(report.evidence.callback.terminal_failure);
    report.record_validation_failure(ProblemError::memory("later original validation"));
    terminal(&report, &callback, Observation::ContractFailure);

    // Merely retaining a trial witness does not create a terminal native observation.
    let mut report = with_callback(trial());
    report.termination.category = Termination::Success;
    assert!(!report.evidence.callback.terminal_failure);
    assert!(report.validation_failure().is_none());
    assert!(report.shared_effective_failure().is_some());
    assert_eq!(observe_native(&report), Observation::Converged);
    assert!(cause_native(&report).is_none());
}

#[tokio::test]
async fn product_refinement_reuses_capacity_and_exact_dependencies_keep_other_consumers_valid() {
    use pse_model::strategy::{
        AccuracyClass, AccuracyDemand, AccuracyEvidence, BranchPolicy, OperationContract,
        ProductDemand, ProductEvidence, SemanticProductKey,
    };
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
    let mut evidence = ProductEvidence {
        source,
        derivative_order: 1,
        branch: BranchPolicy::any_qualified(),
        accuracy: AccuracyEvidence {
            product: hash(8),
            normalization: hash(4),
            class: AccuracyClass::Certified,
            error: Some(1e-4),
        },
    };
    let mut state = ProductState {
        capacity: Some(1),
        ..Default::default()
    };
    state.publish(evidence.clone()).unwrap();
    evidence.accuracy.error = Some(1e-8);
    state.publish(evidence.clone()).unwrap();
    let demand = ProductDemand {
        source,
        derivative_order: 1,
        branch: BranchPolicy::any_qualified(),
        accuracy: AccuracyDemand {
            product: hash(8),
            normalization: hash(4),
            allowance: 1e-7,
            class: AccuracyClass::Certified,
        },
    };
    let mut contract = OperationContract {
        inputs: vec![demand],
        outputs: vec![],
    };
    assert_eq!(state.consume(&contract).unwrap(), vec![evidence.accuracy]);
    contract.inputs[0].source.parameters = Some(hash(9));
    assert!(state.consume(&contract).is_err());
    contract.inputs[0].source.parameters = source.parameters;
    assert!(
        state.consume(&contract).is_ok(),
        "a changed consumer binding does not invalidate independent immutable products"
    );
    contract.inputs[0].derivative_order = 0;
    assert!(state.consume(&contract).is_err());
}

#[tokio::test]
async fn effective_output_obligations_precede_permission_and_preserve_dispatched_work() {
    use pse_model::strategy::{
        AccuracyClass, AccuracyEvidence, BranchPolicy, ProductEvidence, ProductionDemand,
        SemanticProductKey,
    };
    let actual = ProductEvidence {
        source: SemanticProductKey {
            structure: hash(1),
            binding: hash(2),
            numerical_policy: Some(hash(3)),
            normalization: Some(hash(4)),
            point: Some(hash(5)),
            parameters: Some(hash(6)),
            derivation: Some(hash(7)),
            branch: None,
            accuracy: Some(hash(8)),
        },
        derivative_order: 1,
        branch: BranchPolicy::any_qualified(),
        accuracy: AccuracyEvidence {
            product: hash(8),
            normalization: hash(4),
            class: AccuracyClass::Certified,
            error: Some(1e-8),
        },
    };
    let demand = ProductionDemand {
        source: actual.source,
        derivative_order: 1,
        branch: actual.branch,
        allowance: 1e-7,
        class: AccuracyClass::Certified,
    };
    for case in 0..5 {
        let mut declaration = strategy();
        declaration.mechanisms.truncate(1);
        let mut output = demand.clone();
        let evidence = match case {
            0 => vec![actual.clone()],
            1 => Vec::new(),
            2 => {
                output.source.point = Some(hash(9));
                vec![actual.clone()]
            }
            3 => {
                output.allowance = 1e-9;
                vec![actual.clone()]
            }
            _ => {
                output.derivative_order = 0;
                vec![actual.clone()]
            }
        };
        declaration.mechanisms[0].operation.outputs.push(output);
        let mut assessed = 0;
        let result = run(
            &declaration,
            &scope(),
            facts,
            |_, _| {
                Ok(Attempt {
                    value: 42,
                    evidence: evidence.clone(),
                    observation: Observation::Converged,
                    work: charge(0),
                })
            },
            |_, observation| {
                assessed += 1;
                assessment(true, false, observation)
            },
        );
        assert_eq!(result.work.attempts, 1);
        assert_eq!(result.work.evaluations, Some(3));
        if case == 0 {
            assert_eq!(result.value, Some(42));
            assert_eq!(assessed, 1);
        } else {
            assert!(matches!(
                result.terminal.as_deref(),
                Some(ProblemError::Unsupported(_))
            ));
            assert!(result.value.is_none());
            assert_eq!(assessed, 0);
            assert!(result.events.iter().any(|event| event.work.is_some()
                && event.observation == Some(Observation::Converged)
                && event.permission.is_none()));
        }
    }
}

#[tokio::test]
async fn exhausted_owned_catalog_preserves_actual_conclusion_while_available_binding_hits_cap() {
    use pse_model::strategy::{CompositionRequest, StartPolicy, StartRules};
    let request = CompositionRequest {
        limits: Some(WorkLimits {
            attempts: 1,
            evaluations: None,
            iterations: None,
            factorizations: None,
            proof_steps: None,
        }),
        ..Default::default()
    };
    let start = StartRules {
        policy: StartPolicy::NoPriorStart,
        recovery: Vec::new(),
    };
    let candidate = AutoCandidate {
        identity: hash(1),
        kind: MechanismKind::Direct,
        start: StartOrigin::Specification,
        replacement: false,
        support: Default::default(),
        reservation: None,
        prepared: true,
    };
    let cause = Arc::new(ProblemError::numerical("actual limited original candidate"));
    let last = AutoObservation {
        awaiting_assessment: false,
        native: Observation::Limited,
        original: Some(OriginalConclusion::Refused {
            cause: cause.clone(),
        }),
        permission: Some(CandidateUse::Unusable),
    };
    let mut work = charge(0).observed;
    work.attempts = 1;
    assert!(
        matches!(next_automatic(&request,&start,std::slice::from_ref(&candidate),&BTreeSet::from([0]),Some(&last),work,false),AutoDecision::Exhausted {cause:Some(actual)} if Arc::ptr_eq(&actual,&cause))
    );
    assert!(
        matches!(next_automatic(&request,&start,&[candidate],&BTreeSet::new(),Some(&last),work,false),AutoDecision::Stop {cause:Some(actual)} if matches!(actual.as_ref(),ProblemError::Limit {..}))
    );
    let qualified = AutoObservation {
        awaiting_assessment: false,
        native: Observation::Converged,
        original: Some(OriginalConclusion::Satisfied),
        permission: Some(CandidateUse::SeedOnly),
    };
    assert!(matches!(
        next_automatic(
            &request,
            &start,
            &[],
            &BTreeSet::new(),
            Some(&qualified),
            work,
            false
        ),
        AutoDecision::Finish
    ));
    let seed = numerics::auxiliary_start(true);
    let qualified = Assessment {
        auxiliary: false,
        original: OriginalConclusion::Satisfied,
        retention: StepRetention {
            candidate: seed.clone(),
            session: SessionDisposition::Discard,
        },
        work: Vec::new(),
        observation: Observation::Converged,
        cause: None,
    };
    assert_eq!(
        transition(&strategy().mechanisms[0], &qualified),
        Transition::Finish
    );
    assert_eq!(
        qualified.retention.candidate.usability,
        CandidateUse::SeedOnly
    );
    let auxiliary = Assessment {
        auxiliary: true,
        ..qualified
    };
    assert_eq!(
        transition(&strategy().mechanisms[0], &auxiliary),
        Transition::Stop,
        "an auxiliary seed cannot finish original correction"
    );
}
