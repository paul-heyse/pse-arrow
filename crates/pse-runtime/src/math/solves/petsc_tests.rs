// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
fn native_report(outcome: &Outcome) -> &SolveReport {
    match outcome {
        Outcome::Native(report) => report,
        _ => panic!("actual PETSc report required"),
    }
}

use pse_ids::SemanticId;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

async fn original(text: &str) -> (crate::workflow::Runtime, PreparedSolve) {
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
        selection: SolverSelection::Explicit(Backend::Petsc),
        backend: BackendSettings::Petsc(native_petsc::Settings::default()),
        ..Default::default()
    };
    profile.numerics.native_scaling = false;
    profile.controls.foreign_bytes = Some(1 << 20);
    let prepared = package
        .prepare_solve(
            root,
            pse_modeling::specialize::root_instance(root),
            pse_modeling::Bindings::default(),
            pse_modeling::Limits::default(),
            pse_compiler::workspace::ModelingCaseBindings::default(),
            DerivativeOrder::First,
            fixture::compiler_profile(),
            profile,
            NumericalInputs::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    (runtime, prepared.solve)
}
fn scope() -> ExecutionScope {
    ExecutionScope::new(
        Arc::new(AtomicBool::new(false)),
        Some(Instant::now() + Duration::from_secs(30)),
    )
}
fn mass(source: &PreparedPetscSource) -> Arc<dyn PetscMassFactory> {
    let mut matrix = pse_math::sparse::AssemblyMatrix::new(
        1,
        1,
        &[Entry::new(GlobalRow::new(0), GlobalCol::new(0))],
        10,
    )
    .unwrap();
    matrix.add(pse_math::index::Addend::new(0), 2.0).unwrap();
    Arc::new(
        FrozenPetscMass::new(source, Arc::new(matrix), ContentHash::from_bytes([80; 32])).unwrap(),
    )
}
fn flow_profile(source: &PreparedPetscSource) -> SolverProfile {
    let mut profile = source.original.profile.clone();
    profile.backend = BackendSettings::Petsc(native_petsc::Settings {
        method: native_petsc::Method::PseudoTransient,
        pseudo: Some(Default::default()),
        ..Default::default()
    });
    profile
}
fn execute(service: &Arc<MathService>, prepared: &PreparedPetsc, point: &[f64]) -> DerivedAttempt {
    let mut execution = Execution::within(
        prepared.scope().cancellation().clone(),
        prepared.controls(),
        prepared.scope().clone(),
    )
    .unwrap();
    execution.memory = Some(service.policy.foreign_allowance(prepared.controls()));
    let budget = WorkerBudget::drawing(service.policy.worker_bytes, &service.pool);
    let result = execution::scoped(
        &[execution::adapter(Backend::Petsc)],
        1,
        service.policy.stack_bytes,
        || service.execute_petsc(prepared, execution, &budget, point),
    )
    .unwrap();
    assert_eq!(
        budget.used(),
        0,
        "all source, guard and native callback workers must end before return"
    );
    result
}

#[tokio::test]
async fn compiled_flow_keeps_original_offsets_scope_and_auxiliary_role() {
    let (runtime, original) = original(
        "package p { def Root { var x:Scalar; annotation start x(0); eq balance:x==3; } }",
    )
    .await;
    let service = runtime.native();
    let task = scope();
    let guards = original.petsc_guard_inventory().unwrap();
    assert!(!guards.obligations.is_empty());
    let source = service
        .prepare_petsc_source(
            original.clone(),
            guards,
            task.clone(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    // The authored compiler already lowers this equation to x-3 == 0. Preserve
    // that actual source inventory rather than reconstructing a second RHS authority.
    assert_eq!(source.physical().constraints()[0].lower, 0.0);
    assert_eq!(source.zero().constraints()[0].lower, 0.0);
    assert_ne!(source.physical().identity(), source.zero().identity());
    let profile = flow_profile(&source);
    let actions = mass(&source);
    let prepared = service
        .prepare_petsc_flow(
            source,
            profile,
            PetscFlowDefinition {
                domain: None,
                mass: actions,
                pairing: vec![GlobalCol::new(0)],
                sign: 1.0,
                limits: native_petsc::FlowLimits {
                    steps: 100,
                    artificial_time: 1e6,
                },
            },
        )
        .unwrap();
    assert_eq!(prepared.scope().deadline(), task.deadline());
    assert!(Arc::ptr_eq(
        prepared.scope().cancellation(),
        task.cancellation()
    ));
    let result = execute(service, &prepared, &[0.0]);
    assert_eq!(
        native_report(&result.outcome).termination.name,
        "TS_CONVERGED_USER",
        "{:?}",
        native_report(&result.outcome)
    );
    assert_eq!(
        native_report(&result.outcome).termination.category,
        Termination::Success
    );
    let proposal = result.proposal.unwrap();
    assert!((proposal.coordinates[0] - 3.0).abs() < 1e-7);
    assert_eq!(proposal.original, original.original_identity().unwrap());
    assert!(result.screening_failure.is_none());
    assert_eq!(
        native_report(&result.outcome).provenance["mathematical_role"],
        "auxiliary-original-coordinate-proposal"
    );
    assert!(
        native_report(&result.outcome)
            .evidence
            .work
            .iterations
            .is_some()
    );
    assert!(
        native_report(&result.outcome)
            .evidence
            .work
            .evaluations
            .is_none()
    );
}
#[tokio::test]
async fn compiled_original_guard_refuses_invalid_flow_initial_without_proposal() {
    let (runtime, original) = original(
        "package p { def Root { var x:Scalar; annotation start x(1); eq balance:log(x)==0; } }",
    )
    .await;
    let service = runtime.native();
    let source = service
        .prepare_petsc_source(
            original.clone(),
            original.petsc_guard_inventory().unwrap(),
            scope(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let prepared = service
        .prepare_petsc_flow(
            source.clone(),
            flow_profile(&source),
            PetscFlowDefinition {
                domain: None,
                mass: mass(&source),
                pairing: vec![GlobalCol::new(0)],
                sign: 1.0,
                limits: native_petsc::FlowLimits {
                    steps: 2,
                    artificial_time: 10.0,
                },
            },
        )
        .unwrap();
    let result = execute(service, &prepared, &[-1.0]);
    assert_eq!(
        native_report(&result.outcome).termination.name,
        "PETSC_FLOW_INITIAL_DOMAIN_REFUSED"
    );
    assert!(
        native_report(&result.outcome)
            .evidence
            .callback
            .terminal_failure
    );
    assert!(native_report(&result.outcome).callback_failure().is_some());
    assert!(result.proposal.is_none());
    assert!(result.screening_failure.is_none());
    assert!(native_report(&result.outcome).quality.is_none());
}
#[tokio::test]
async fn actual_compiled_guard_inventory_and_deadline_are_mandatory() {
    let (runtime, original) = original(
        "package p { def Root { var x:Scalar; annotation start x(0); eq balance:x==3; } }",
    )
    .await;
    let service = runtime.native();
    let guards = original.petsc_guard_inventory().unwrap();
    let mut partial = guards.clone();
    partial.obligations.clear();
    assert!(
        service
            .prepare_petsc_source(
                original.clone(),
                partial,
                scope(),
                &crate::CancelSource::new()
            )
            .await
            .is_err()
    );
    let mut wrong = guards.clone();
    wrong.preparation = ContentHash::from_bytes([90; 32]);
    assert!(
        service
            .prepare_petsc_source(
                original.clone(),
                wrong,
                scope(),
                &crate::CancelSource::new()
            )
            .await
            .is_err()
    );
    let cancel = Arc::new(AtomicBool::new(false));
    let expired = ExecutionScope::new(cancel.clone(), Some(Instant::now()));
    assert!(
        service
            .prepare_petsc_source(
                original.clone(),
                guards.clone(),
                expired,
                &crate::CancelSource::new()
            )
            .await
            .is_err()
    );
    assert!(!cancel.load(Ordering::Acquire));
    let task = scope();
    task.cancellation().store(true, Ordering::Release);
    assert!(
        service
            .prepare_petsc_source(original, guards, task, &crate::CancelSource::new())
            .await
            .is_err()
    );
}
