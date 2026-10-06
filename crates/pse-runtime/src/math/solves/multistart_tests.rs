// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
#[cfg(feature = "solver-kinsol")]
use std::sync::atomic::Ordering;
use std::{
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};

#[cfg(feature = "solver-kinsol")]
/// The analytical coordinates and residuals below require physical precision,
/// independently of the ordinary engineering defaults and KKT termination.
fn verification_numerics() -> NumericalPolicy {
    let physical = crate::workflow::tests::physical();
    let scalar = physical
        .quantities
        .quantity_types()
        .find(|quantity| quantity.name.as_deref() == Some("Scalar"))
        .unwrap();
    NumericalPolicy {
        engineering_rules: vec![pse_model::numerics::EngineeringRule {
            rule_id: pse_ids::named_id(scalar.id.as_id(), "multistart-verification-precision")
                .into(),
            quantity_id: scalar.id.as_id(),
            unit_id: scalar.canonical_unit.as_id(),
            physical_allowance: Some(1e-10),
            relative_fraction: Some(0.0),
            provenance: "original multistart analytical coordinate and residual verification"
                .into(),
        }],
        kkt: pse_model::numerics::KktTolerances {
            stationarity: 1e-10,
            complementarity: 1e-10,
        },
        ..Default::default()
    }
}

async fn original(text: &str, backend: Backend) -> (crate::workflow::Runtime, PreparedSolve) {
    let (runtime, _, prepared) = modeling_original(text, backend, Default::default()).await;
    (runtime, prepared.solve)
}
async fn modeling_original(
    text: &str,
    backend: Backend,
    numerics: NumericalPolicy,
) -> (
    crate::workflow::Runtime,
    crate::workflow::ModelingPackage,
    crate::workflow::ModelingSolvePreparation,
) {
    use crate::workflow::tests as fixture;
    let runtime = fixture::runtime_with(256 << 20, 1 << 20, 1 << 30);
    let rows = pse_authoring::language::parse(
        text,
        SemanticId::NIL,
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap();
    let root = rows
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    let package = runtime
        .modeling_package(rows, fixture::physical())
        .await
        .unwrap();
    let mut profile = SolverProfile {
        intent: SolveIntent::Root,
        selection: SolverSelection::Explicit(backend),
        numerics,
        ..Default::default()
    };
    if backend == Backend::Ipopt {
        profile.controls.hessian = HessianMode::LimitedMemory;
    }
    let prepared = package
        .prepare_solve(
            root,
            pse_modeling::specialize::root_instance(root),
            pse_modeling::Bindings::default(),
            pse_modeling::Limits::default(),
            Default::default(),
            pse_kernels::DerivativeOrder::First,
            fixture::compiler_profile(),
            profile,
            NumericalInputs::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    (runtime, package, prepared)
}
fn scope() -> ExecutionScope {
    ExecutionScope::new(
        Arc::new(AtomicBool::new(false)),
        Some(Instant::now() + Duration::from_secs(30)),
    )
}
fn seed(original: &PreparedSolve, value: f64) -> BTreeMap<SemanticId, f64> {
    BTreeMap::from([(original.original_coordinates().unwrap()[0], value)])
}

#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn fresh_seed_uses_original_screening_without_result_or_native_permission() {
    let (runtime, mut original) = original(
        "package p { def Root { var x:Scalar; annotation start x(0); eq root:x*x==1; } }",
        Backend::Kinsol,
    )
    .await;
    original.profile.composition.recovery = vec![StartOrigin::ModifiedSpecification];
    let service = runtime.native();
    let task = scope();
    let target = original.original_identity().unwrap();
    let prepared = service
        .prepare_multistart(
            original.clone(),
            seed(&original, -2.0),
            BranchPolicy::any_qualified(),
            task.clone(),
        )
        .unwrap();
    assert_eq!(prepared.original_identity().unwrap(), target);
    assert_eq!(prepared.origin(), StartOrigin::ModifiedSpecification);
    assert_eq!(
        prepared.original_target().profile.controls.start,
        StartPolicy::NoPriorStart
    );
    assert_eq!(prepared.profile().controls.start, StartPolicy::Explicit);
    assert_ne!(
        prepared.strategy_profile().unwrap(),
        original.strategy_profile().unwrap()
    );
    assert!(prepared.data.proposal.source().accuracy.is_none());
    assert!(prepared.data.proposal.source().branch.is_none());
    let clone = prepared.clone();
    assert!(Arc::ptr_eq(&prepared.data, &clone.data));
    let budget = WorkerBudget::drawing(service.policy.worker_bytes, &service.pool);
    let mut evaluations = 3;
    let bound = service
        .screen_multistart_worker_observed(&prepared, task.clone(), &budget, &mut evaluations)
        .unwrap();
    assert_eq!(
        evaluations, 5,
        "screening extends an observed counter without replacing prior work"
    );
    assert_eq!(bound.step.original_identity().unwrap(), target);
    assert_eq!(bound.step.profile.controls.start, StartPolicy::Explicit);
    assert_eq!(
        bound.step.strategy_profile().unwrap(),
        prepared.strategy_profile().unwrap()
    );
    assert_eq!(
        prepared.original_target().profile.controls.start,
        StartPolicy::NoPriorStart
    );
    assert_eq!(bound.step.source_start(None).unwrap(), [-2.0]);
    assert_eq!(bound.screening_evaluations, 2);
    assert!(budget.used() > 0);
    drop(bound);
    assert_eq!(budget.used(), 0);
    let later = service
        .prepare_multistart(
            original.clone(),
            seed(&original, 2.0),
            BranchPolicy::any_qualified(),
            task,
        )
        .unwrap();
    assert_ne!(prepared.key(), later.key());
}
#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn standalone_fresh_seed_requires_original_recovery_grant() {
    let (runtime, original) = original(
        "package p { def Root { var x:Scalar; annotation start x(0); eq root:x*x==1; } }",
        Backend::Kinsol,
    )
    .await;
    assert!(original.profile.composition.recovery.is_empty());
    let service = runtime.native();
    let task = scope();
    let prepared = service
        .prepare_multistart(
            original.clone(),
            seed(&original, -2.0),
            BranchPolicy::any_qualified(),
            task.clone(),
        )
        .unwrap();
    let budget = WorkerBudget::drawing(service.policy.worker_bytes, &service.pool);
    let mut evaluations = 0;
    let error =
        match service.screen_multistart_worker_observed(&prepared, task, &budget, &mut evaluations)
        {
            Ok(_) => panic!("standalone screening cannot invent a source recovery grant"),
            Err(error) => error.into_problem(),
        };
    assert!(matches!(error, ProblemError::Contract(ref reason)
        if reason == "screened proposal origin or branch is not admitted by the original request"));
    assert_eq!(
        evaluations, 2,
        "actual original screening work remains visible on refusal"
    );
    assert_eq!(budget.used(), 0);
    assert_eq!(original.profile.controls.start, StartPolicy::NoPriorStart);
    assert!(original.profile.composition.recovery.is_empty());
}
#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn original_invalid_start_guard_and_task_limits_preserve_causes_without_dispatch() {
    let (runtime, original) = original(
        "package p { def Root { var x:Scalar; annotation start x(1); eq root:log(x)==0; } }",
        Backend::Kinsol,
    )
    .await;
    let service = runtime.native();
    let task = scope();
    let prepared = service
        .prepare_multistart(
            original.clone(),
            seed(&original, -1.0),
            BranchPolicy::any_qualified(),
            task.clone(),
        )
        .unwrap();
    let budget = WorkerBudget::drawing(service.policy.worker_bytes, &service.pool);
    let mut evaluations = 0;
    let error = match service.screen_multistart_worker_observed(
        &prepared,
        task.clone(),
        &budget,
        &mut evaluations,
    ) {
        Ok(_) => panic!("invalid original domain must refuse before native dispatch"),
        Err(error) => error.into_problem(),
    };
    assert_eq!(
        evaluations, 2,
        "actual objective invocation and failed original constraint invocation"
    );
    assert!(
        matches!(&error,ProblemError::Math(pse_math::MathError::Instance {cause,..}) if matches!(cause.as_ref(),pse_math::MathError::Domain {..})),
        "{error:?}"
    );
    assert_eq!(budget.used(), 0);
    assert!(!task.cancellation().load(Ordering::Acquire));
    let wrong = scope();
    assert!(
        service
            .screen_multistart_worker(&prepared, wrong, &budget)
            .is_err()
    );
    assert_eq!(budget.used(), 0);
    let late = ExecutionScope::new(task.cancellation().clone(), Some(Instant::now()));
    assert!(
        service
            .screen_multistart_worker(&prepared, late, &budget)
            .is_err()
    );
    assert!(!task.cancellation().load(Ordering::Acquire));
    task.cancellation().store(true, Ordering::Release);
    assert!(
        service
            .screen_multistart_worker(&prepared, task, &budget)
            .is_err()
    );
    assert_eq!(budget.used(), 0);
}
#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn connected_incomplete_and_nonfinite_seeds_are_refused_before_native_work() {
    let (runtime, original) = original(
        "package p { def Root { var x:Scalar; annotation start x(0); eq root:x*x==1; } }",
        Backend::Kinsol,
    )
    .await;
    let service = runtime.native();
    let hash = ContentHash::from_bytes([9; 32]);
    let connected = BranchPolicy {
        kind: pse_model::strategy::BranchKind::Connected,
        connected: Some(pse_model::strategy::ConnectedPath {
            path: hash,
            sheet: hash,
            transport: hash,
            orientation: hash,
        }),
    };
    assert!(
        service
            .prepare_multistart(original.clone(), seed(&original, 1.0), connected, scope())
            .is_err()
    );
    for values in [
        BTreeMap::new(),
        seed(&original, f64::NAN),
        BTreeMap::from([(SemanticId::NIL, 1.0)]),
    ] {
        assert!(
            service
                .prepare_multistart(
                    original.clone(),
                    values,
                    BranchPolicy::any_qualified(),
                    scope()
                )
                .is_err()
        );
    }
    let flag = Arc::new(AtomicBool::new(false));
    let expired = ExecutionScope::new(flag.clone(), Some(Instant::now()));
    assert!(
        service
            .prepare_multistart(
                original.clone(),
                seed(&original, 1.0),
                BranchPolicy::any_qualified(),
                expired
            )
            .is_err()
    );
    assert!(!flag.load(Ordering::Acquire));
}
#[cfg(feature = "solver-ipopt")]
#[tokio::test]
async fn authored_bounds_refuse_fresh_seed_without_clamping_or_source_mutation() {
    let (runtime,original)=original("package p { def Root { var x:Scalar; annotation bounds x(-2,2); annotation start x(0); eq root:x*x==1; } }",Backend::Ipopt).await;
    let service = runtime.native();
    let identity = original.original_identity().unwrap();
    assert!(
        service
            .prepare_multistart(
                original.clone(),
                seed(&original, 3.0),
                BranchPolicy::any_qualified(),
                scope()
            )
            .is_err()
    );
    let admitted = service
        .prepare_multistart(
            original.clone(),
            seed(&original, -1.0),
            BranchPolicy::any_qualified(),
            scope(),
        )
        .unwrap();
    assert_eq!(admitted.point(), [-1.0]);
    assert_eq!(admitted.original_identity().unwrap(), identity);
    assert_eq!(original.source_start(None).unwrap(), [0.0]);
}

#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn shared_original_driver_coordinates_fresh_seeds_and_stops_after_permission() {
    shared_original_driver(true).await;
}
#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn shared_original_driver_refuses_fresh_seed_without_outer_recovery_grant() {
    shared_original_driver(false).await;
}
#[cfg(feature = "solver-kinsol")]
async fn shared_original_driver(permit_recovery: bool) {
    use pse_model::generated::enums::NumericalEventKind;
    use pse_model::strategy::{
        MechanismKind, NumericalStrategy, Phase, Position, Transition, WorkLimits,
    };
    let (runtime, package, mut prepared) = modeling_original(
        "package p { def Root { var x:Scalar; annotation start x(0); eq root:x*x==1; } }",
        Backend::Kinsol,
        verification_numerics(),
    )
    .await;
    let original = prepared.solve.clone();
    assert!(!original.tolerances().rows.is_empty());
    assert!(
        original
            .tolerances()
            .rows
            .iter()
            .all(|budget| *budget <= 1e-10)
    );
    assert!(
        original
            .tolerances()
            .variables
            .iter()
            .all(|budget| *budget <= 1e-10)
    );
    assert_eq!(original.profile.controls.start, StartPolicy::NoPriorStart);
    assert!(original.profile.composition.recovery.is_empty());
    let task = scope();
    let service = runtime.native();
    let mut rungs = vec![original.clone().into()];
    let limits = WorkLimits {
        attempts: 4,
        evaluations: None,
        iterations: None,
        factorizations: None,
        proof_steps: None,
    };
    let mut strategy = NumericalStrategy::direct(original.profile.controls.start, limits);
    if permit_recovery {
        strategy.start.recovery = vec![StartOrigin::ModifiedSpecification];
    }
    strategy.mechanisms[0].profile = Some(ProfileRef {
        backend: Backend::Kinsol,
        key: original.strategy_profile().unwrap(),
    });
    strategy.mechanisms[0].transitions = vec![
        Transition::Finish,
        Transition::Recover,
        Transition::Continue,
        Transition::Stop,
    ];
    for value in [0.0, -2.0, 2.0] {
        let start = service
            .prepare_multistart(
                original.clone(),
                seed(&original, value),
                BranchPolicy::any_qualified(),
                task.clone(),
            )
            .unwrap();
        let mut mechanism = strategy.mechanisms[0].clone();
        mechanism.kind = MechanismKind::Multistart;
        mechanism.position = Position::Execution;
        mechanism.profile = Some(start.profile_ref().unwrap());
        mechanism.support = start.support().unwrap().into_iter().collect();
        mechanism.starts = vec![StartOrigin::ModifiedSpecification];
        strategy.mechanisms.push(mechanism);
        rungs.push(PreparedRung::Multistart(start));
    }
    let mut connected = strategy.clone();
    let hash = ContentHash::from_bytes([8; 32]);
    connected.branch = BranchPolicy {
        kind: pse_model::strategy::BranchKind::Connected,
        connected: Some(pse_model::strategy::ConnectedPath {
            path: hash,
            sheet: hash,
            transport: hash,
            orientation: hash,
        }),
    };
    assert!(
        original
            .clone()
            .within_task(task.clone())
            .unwrap()
            .with_strategy(connected, rungs.clone())
            .is_err(),
        "a connected original task cannot consume unconnected seed mechanisms"
    );
    prepared.solve = original
        .within_task(task)
        .unwrap()
        .with_strategy(strategy, rungs)
        .unwrap();
    let result = package
        .solve_case(
            prepared,
            crate::workflow::tests::compiler_profile(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    assert_eq!(
        result.prepared.solve.profile.controls.start,
        StartPolicy::NoPriorStart
    );
    assert!(
        result
            .prepared
            .solve
            .profile
            .composition
            .recovery
            .is_empty()
    );
    if !permit_recovery {
        assert!(!result.accepted);
        assert!(result.prediction_anchor(0.0).is_err());
        let trace = result.strategy.as_ref().unwrap();
        let refusal = trace
            .events
            .iter()
            .find(|event| event.mechanism == 1 && event.kind == NumericalEventKind::Refused)
            .unwrap();
        assert_eq!(
            refusal.observation,
            Some(pse_model::generated::enums::NumericalAttemptObservation::CapabilityRefusal)
        );
        assert_eq!(refusal.transition, Some(Transition::Stop));
        assert!(
            matches!(refusal.cause.as_deref(), Some(ProblemError::Contract(reason))
            if reason == "strategy start origin is not permitted")
        );
        assert!(
            trace
                .events
                .iter()
                .filter(|event| event.mechanism > 0)
                .all(|event| event.kind != NumericalEventKind::Started && event.work.is_none()),
            "an ungranted seed cannot screen or dispatch native work"
        );
        return;
    }
    let anchor = result.prediction_anchor(0.0);
    assert!(
        anchor.is_ok(),
        "original completion must grant this public anchor operation; accepted={}; native={:?}; anchor={:?}; events={:?}",
        result.accepted,
        match &result.outcome {
            Outcome::Native(report) => Some((
                report.termination.name.as_str(),
                report
                    .candidate
                    .as_ref()
                    .and_then(|candidate| candidate.primal.first().copied())
            )),
            _ => None,
        },
        anchor
            .as_ref()
            .err()
            .map(|cause| cause.to_string().chars().take(300).collect::<String>()),
        result.strategy.as_ref().map(|trace| trace
            .events
            .iter()
            .map(|event| (
                event.mechanism,
                event.kind,
                event.phase,
                event.observation,
                event.permission,
                event.cause.as_ref().map(|cause| cause
                    .to_string()
                    .chars()
                    .take(300)
                    .collect::<String>()),
            ))
            .collect::<Vec<_>>()),
    );
    let Outcome::Native(report) = &result.outcome else {
        panic!("original native report required")
    };
    assert!((report.candidate.as_ref().unwrap().primal[0] + 1.0).abs() < 1e-7);
    let trace = result.strategy.as_ref().unwrap();
    let assessments = trace
        .events
        .iter()
        .filter_map(|event| event.work)
        .filter(|charge| charge.phase == Phase::Assessment)
        .collect::<Vec<_>>();
    assert_eq!(
        assessments.len(),
        3,
        "each actual same-case candidate has its own original assessment"
    );
    assert_eq!(
        assessments
            .iter()
            .map(|charge| charge.charging_owner)
            .collect::<BTreeSet<_>>()
            .len(),
        3,
        "distinct same-case original assessments cannot share a charging owner"
    );
    assert!(
        assessments
            .iter()
            .all(|charge| charge.observed.attempts == 0 && charge.observed.evaluations == Some(0)),
        "identity repair cannot invent assessment work"
    );
    let rows = trace.rows(result.run_id, 0).unwrap();
    let finished = rows
        .iter()
        .filter(|row| {
            row.kind == NumericalEventKind::Finished
                && row.phase == Phase::Assessment
                && row.permission.is_some()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        finished.len(),
        3,
        "direct, failed fresh seed and first qualified fresh seed only"
    );
    assert_eq!(finished[0].transition, Some(Transition::Recover));
    assert_eq!(finished[1].transition, Some(Transition::Recover));
    assert_eq!(finished[2].transition, Some(Transition::Finish));
    assert_eq!(
        finished[1].start_origin,
        Some(StartOrigin::ModifiedSpecification)
    );
    assert_eq!(
        finished[2].start_origin,
        Some(StartOrigin::ModifiedSpecification)
    );
    assert_eq!(
        trace.declaration.mechanisms.len(),
        4,
        "unused fresh seed remains in the declared plan"
    );
    assert!(
        !trace
            .events
            .iter()
            .any(|event| event.mechanism == 3 && event.kind == NumericalEventKind::Started),
        "remaining declared seed must stay unstarted after original permission"
    );
}
