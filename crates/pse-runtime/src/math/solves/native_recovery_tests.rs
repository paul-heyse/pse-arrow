// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Real native-profile binding consumes a dispatched entry and retains Root products.
use super::*;
use crate::workflow::tests as fixture;
use pse_model::strategy::{MechanismKind, ProfileRef, StartOrigin, Transition};

#[tokio::test]
async fn explicit_entry_then_screened_native_recovery_dispatches_and_retains_original_root_factor()
{
    let rows=pse_authoring::language::parse(
        "package p { def Root { param p:Scalar=4; var x:Scalar; eq root:x*x==p; annotation start x(1); } }",
        pse_ids::SemanticId::NIL,pse_authoring::language::IdentityPolicy::Named,pse_authoring::ParseBudget::default()).unwrap();
    let root = rows
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let runtime = fixture::runtime_with(256 << 20, 16 << 20, 1 << 30);
    let package = runtime.modeling_package(rows, fixture::physical()).unwrap();
    let cancel = crate::CancelSource::new();
    let mut solver = fixture::profile();
    solver.selection = SolverSelection::Explicit(Backend::Pounce);
    solver.composition.recovery = vec![StartOrigin::Auxiliary];
    let mut analysis = crate::workflow::ModelingAnalysis {
        root,
        instance: pse_modeling::specialize::root_instance(root),
        bindings: Default::default(),
        limits: Default::default(),
        case: Default::default(),
        order: pse_kernels::DerivativeOrder::First,
        compiler: fixture::compiler_profile(),
        solver,
        numerical: Default::default(),
    };
    analysis.bindings.demand.push("p".into());
    let initial = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    let parameter = initial.model.model.compiled().model.paths["p"];
    analysis.solver.sensitivity = Some(super::super::settings::SensitivityRequest {
        parameters: vec![parameter],
        reduced_hessian: false,
        propagation: None,
    });
    let prepared = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    let original = prepared.solve.retaining_factor().unwrap();
    let source = original.source_start(None).unwrap();
    let ids = match &original.representation {
        Representation::Algebraic(case) => case.prepared.compiled().plan.columns().to_vec(),
        _ => panic!("original algebraic fixture"),
    };
    let original = original
        .with_primal_start(ids.into_iter().zip(source).collect())
        .unwrap();
    let scope = pse_kernels::ExecutionScope::new(
        Arc::default(),
        Some(std::time::Instant::now() + std::time::Duration::from_secs(20)),
    );
    let mut limits = original.numerical_strategy().limits;
    limits.attempts = 8;
    let ledger = super::super::strategy::admission::TaskAdmission::new(
        limits,
        scope.clone(),
        Some(runtime.native().pool.clone()),
        false,
    );
    let original = original
        .within_admitted_task(scope.clone(), ledger.clone())
        .unwrap();
    let native = runtime.native();
    let session = native.open_session().unwrap();
    let owner = native
        .reserve(
            "test:original-profile-result",
            original.result_bytes().unwrap(),
        )
        .unwrap();
    let policy = original.numerics.policy.clone();
    let mut initial_declaration = original.numerical_strategy();
    initial_declaration.limits = limits;
    initial_declaration.mechanisms[0].profile = Some(ProfileRef {
        backend: Backend::Pounce,
        key: original.strategy_profile().unwrap(),
    });
    let initial_task = original
        .clone()
        .with_strategy(initial_declaration, vec![original.clone().into()])
        .unwrap();
    let (first, _, first_trace) = session
        .step(
            initial_task,
            None,
            0,
            Arc::new(Progress::new(16)),
            owner.clone(),
            &cancel,
            move |outcome, _, _| {
                super::super::strategy::Assessed::native(
                    (),
                    super::super::StepRetention {
                        candidate: outcome.candidate_use(&policy),
                        session: super::super::SessionDisposition::Discard,
                    },
                    outcome,
                )
            },
        )
        .await
        .unwrap();
    assert!(ledger.entry_dispatched());
    assert_eq!(first_trace.starts[0], StartOrigin::Explicit);
    assert!(matches!(first, Outcome::Native(_)));
    // The actual preparation API preserves the original parametric program while
    // admitting a different acting native profile and a source-owned displacement.
    let mut profile = original.profile.clone();
    profile.controls.iterations += 1;
    let operation = AutomaticOperation {
        candidate: super::super::strategy::AutoCandidate {
            identity: original.request_identity().unwrap().as_id(),
            kind: MechanismKind::NativeGlobalization,
            start: StartOrigin::Auxiliary,
            replacement: true,
            support: Default::default(),
            reservation: None,
            prepared: false,
        },
        binding: AutomaticBinding::NativeProfile {
            original: Box::new(original.clone()),
            profile: Box::new(profile),
            perturbation: Some(native::pounce::StartPerturbation {
                seed: 17,
                scale: 0.01,
            }),
        },
    };
    let rung = native
        .prepare_automatic_operation(operation, scope, &cancel)
        .await
        .unwrap();
    let PreparedRung::Original(rebound) = &rung else {
        panic!("bound original profile")
    };
    assert_eq!(rebound.profile.controls.start, StartPolicy::Explicit);
    assert_eq!(rebound.entry_origin(false), StartOrigin::Auxiliary);
    assert!(rebound.proposal_start.is_some());
    assert!(Arc::ptr_eq(&rebound.task_admission().unwrap(), &ledger));
    let mut declaration = rebound.numerical_strategy();
    declaration.start.recovery = rebound.composition_request().recovery.clone();
    declaration.limits = limits;
    declaration.mechanisms[0].kind = MechanismKind::NativeGlobalization;
    declaration.mechanisms[0].starts = vec![StartOrigin::Auxiliary];
    declaration.mechanisms[0].transitions = vec![Transition::Finish, Transition::Stop];
    declaration.mechanisms[0].profile = Some(ProfileRef {
        backend: Backend::Pounce,
        key: rebound.strategy_profile().unwrap(),
    });
    let policy = rebound.numerics.policy.clone();
    let composed = rebound
        .as_ref()
        .clone()
        .with_strategy(declaration, vec![rung])
        .unwrap();
    let (outcome, _, trace) = session
        .step(
            composed,
            None,
            1,
            Arc::new(Progress::new(16)),
            owner,
            &cancel,
            move |outcome, _, _| {
                super::super::strategy::Assessed::native(
                    (),
                    super::super::StepRetention {
                        candidate: outcome.candidate_use(&policy),
                        session: super::super::SessionDisposition::Discard,
                    },
                    outcome,
                )
            },
        )
        .await
        .unwrap();
    session.close().await;
    assert_eq!(trace.starts, [StartOrigin::Auxiliary]);
    assert!(
        trace
            .events
            .iter()
            .any(|event| event.kind == pse_model::generated::enums::NumericalEventKind::Started)
    );
    let Outcome::Native(report) = outcome else {
        panic!("original native correction expected")
    };
    assert!((report.candidate.as_ref().unwrap().primal[0] - 2.).abs() < 1e-7);
    let predictor = report
        .evidence
        .root_predictor
        .as_ref()
        .unwrap()
        .as_ref()
        .unwrap();
    assert!((predictor.factor().point()[0] - 2.).abs() < 1e-7);
    assert_eq!(predictor.parameters(), [(parameter, 4.)]);
}
