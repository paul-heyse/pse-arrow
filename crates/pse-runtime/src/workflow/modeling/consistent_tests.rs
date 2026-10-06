// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use pse_kernels::ProviderError;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

#[test]
fn physical_original_initial_rows_use_admitted_scale_offset_and_task_scope() {
    let scope = ExecutionScope::new(
        Arc::new(AtomicBool::new(false)),
        Some(Instant::now() + Duration::from_secs(1)),
    );
    let coordinate = CoordinateBinding {
        id: SemanticId::from_bytes([1; 16]),
        scale: 2.0,
        offset: 10.0,
    };
    let obligation = OriginalInitialCondition {
        coordinate: 0,
        row: SemanticId::from_bytes([2; 16]),
        source: DeclarationId::from_bytes([3; 16]),
        expected: 16.0,
        tolerance: 0.25,
    };
    let run = RunId::from_bytes([4; 16]);
    let checks = original_initial_checks(
        run,
        7.0,
        &[3.0],
        std::slice::from_ref(&coordinate),
        std::slice::from_ref(&obligation),
        &scope,
    )
    .unwrap();
    assert_eq!(checks.len(), 1);
    assert!(checks[0].satisfied);
    assert_eq!(checks[0].source_id, obligation.source);
    let checks = original_initial_checks(
        run,
        7.0,
        &[3.2],
        std::slice::from_ref(&coordinate),
        std::slice::from_ref(&obligation),
        &scope,
    )
    .unwrap();
    assert!(!checks[0].satisfied);
    scope.cancellation().store(true, Ordering::Release);
    assert!(matches!(
        original_initial_checks(run, 7.0, &[3.0], &[coordinate], &[obligation], &scope),
        Err(ProblemError::Provider(ProviderError::Cancelled))
    ));
}
#[test]
fn ic_generic_controls_refuse_inactive_choices_and_keep_actual_resource_controls() {
    admit_controls(&Controls::default()).unwrap();
    let controls = Controls {
        time_limit: Duration::from_secs(1),
        history: 2,
        foreign_bytes: Some(1 << 20),
        ..Controls::default()
    };
    admit_controls(&controls).unwrap();
    for change in 0..3 {
        let mut controls = Controls::default();
        match change {
            0 => controls.iterations = 1,
            1 => controls.threads = 2,
            _ => {
                controls.options.insert(
                    "max_num_steps".into(),
                    pse_backend_native::solve::OptionValue::Integer(1),
                );
            }
        }
        assert!(matches!(
            admit_controls(&controls),
            Err(ProblemError::Unsupported(_))
        ));
    }
}

#[tokio::test]
async fn actual_authored_idas_ic_preserves_initial_roles_without_integrating() {
    let runtime = crate::workflow::tests::runtime();
    let mut physical = crate::workflow::tests::physical();
    physical.preconditions = Arc::new(
        pse_quantity::PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions())
            .unwrap(),
    );
    physical.key =
        pse_compiler::workspace::physical_identity(&physical.quantities, &physical.preconditions);
    let source = "package p { def Root { domain t: Time from 0{s} to 2{s}; discretize mesh on t using integrated(elements=1,order=1); param p: Scalar = 2; param offset: Time = 1{s}; var x[i in t]: Time; var y: Scalar; eq ode[i in t]: d(x[i])/di == y; eq initial: x[0{s}] == offset; eq algebraic: y == 2*p; annotation start x(0{s}); annotation start y(999); } }";
    let rows = pse_authoring::language::parse(
        source,
        SemanticId::NIL,
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap();
    let root = rows
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let package = runtime.modeling_package(rows, physical).await.unwrap();
    let cancel = crate::CancelSource::new();
    let profile = native::Profile {
        end: 1.0,
        samples: vec![0.0, 1.0],
        atol: vec![1e-8; 2],
        parameter_scales: vec![1.0; 2],
        ..native::Profile::default()
    };
    let prepared = package
        .prepare_simulation(
            root,
            pse_modeling::specialize::root_instance(root),
            Bindings::default(),
            Limits::default(),
            ModelingCaseBindings::default(),
            crate::workflow::tests::compiler_profile(),
            profile,
            DerivativeOrder::First,
            &cancel,
        )
        .await
        .unwrap();
    let request = native::ConsistentInitialization {
        time: prepared.profile.start,
        toward: 0.01,
        rtol: prepared.profile.rtol,
        atol: prepared.profile.atol.clone(),
        residual_tolerances: vec![1e-7; 2],
        mode: native::IdasInitialization::AlgebraicAndRates,
        linear: native::IdasLinear::Klu,
        controls: native::IdasInitialConditions::default(),
        trial_failures: native::TrialPolicy::Recoverable,
    };
    let cancellation = FlightCancellation::default();
    let scope = ExecutionScope::new(
        cancellation.flag(),
        Some(Instant::now() + Duration::from_secs(5)),
    );
    let result = prepared
        .initialize_consistent(
            RunId::from_bytes([8; 16]),
            request.clone(),
            Controls::default(),
            scope,
            cancellation,
        )
        .await
        .unwrap();
    assert_eq!(result.report().termination.name, "IDA_SUCCESS");
    let trace = result.strategy().unwrap();
    let event = trace.events.last().unwrap();
    assert!(matches!(
        event.original,
        Some(crate::math::strategy::OriginalConclusion::Satisfied)
    ));
    assert_eq!(
        event.permission,
        Some(pse_model::generated::enums::CandidateUse::SeedOnly)
    );
    assert_eq!(
        event.transition,
        Some(pse_model::strategy::Transition::Finish)
    );
    assert!(event.decision.is_some());
    assert_eq!(event.work.unwrap().observed.evaluations, None);
    assert_eq!(result.report().time, prepared.profile.start);
    assert!(
        result
            .report()
            .assessment
            .as_ref()
            .unwrap()
            .residual_satisfied
    );
    assert!(result.report().assessment.as_ref().unwrap().roles_preserved);
    assert!(result.checks_complete());
    assert!(result.checks().iter().all(|row| row.satisfied));
    let xi = prepared
        .contract
        .differential
        .iter()
        .position(|value| *value)
        .unwrap();
    let yi = 1 - xi;
    let point = result.report().candidate.as_ref().unwrap();
    assert!((point.state[xi] - 1.0).abs() < 1e-7);
    assert!((point.state[yi] - 4.0).abs() < 1e-7);
    let cancellation = FlightCancellation::default();
    let scope = ExecutionScope::new(
        cancellation.flag(),
        Some(Instant::now() + Duration::from_secs(5)),
    );
    let submission = Submission {
        cancel: cancellation.clone(),
        progress: Arc::new(Progress::new(2)),
        queue: false,
        admitted: None,
        deadline: scope.deadline(),
    };
    let (mut observed, owner) = prepared
        .submit_consistent(
            RunId::from_bytes([8; 16]),
            request.clone(),
            Controls::default(),
            scope,
            submission,
        )
        .unwrap()
        .finish()
        .await
        .unwrap();
    let work = observed.report.evidence.work;
    let code = observed.report.termination.code;
    cancellation.cancel();
    observed.check_after_join();
    assert_eq!(observed.report.termination.code, code);
    assert_eq!(observed.report.termination.name, "IDA_SUCCESS");
    assert_eq!(observed.report.evidence.work, work);
    assert_eq!(
        observed.report.termination.category,
        pse_backend_native::solve::Termination::Cancelled
    );
    assert!(observed.report.assessment.is_none());
    assert!(!observed.checks_complete);
    let event = observed.strategy.as_ref().unwrap().events.last().unwrap();
    assert_eq!(
        event.kind,
        pse_model::generated::enums::NumericalEventKind::Abandoned
    );
    assert_eq!(
        event.permission,
        Some(pse_model::generated::enums::CandidateUse::Unusable)
    );
    assert!(event.original.is_none());
    drop(owner);
    let cancellation = FlightCancellation::default();
    let scope = ExecutionScope::new(
        cancellation.flag(),
        Some(Instant::now() + Duration::from_secs(5)),
    );
    let controls = Controls {
        foreign_bytes: Some(1),
        ..Controls::default()
    };
    let error = prepared
        .initialize_consistent(
            RunId::from_bytes([8; 16]),
            request.clone(),
            controls,
            scope,
            cancellation,
        )
        .await
        .unwrap_err();
    let WorkflowError::Math(MathRuntimeError::Strategy { cause, trace }) = error else {
        panic!("IC failure must retain its actual cause and numerical decisions")
    };
    assert!(matches!(
        cause.as_ref(),
        MathRuntimeError::Solve(ProblemError::Limit {
            kind: pse_backend_native::LimitKind::Memory,
            ..
        })
    ));
    let event = trace.events.last().unwrap();
    assert!(event.decision.is_some());
    assert_eq!(
        event.observation,
        Some(pse_model::generated::enums::NumericalAttemptObservation::ResourceExhausted)
    );
    assert!(matches!(
        event.original,
        Some(crate::math::strategy::OriginalConclusion::Unavailable { .. })
    ));
    let cancellation = FlightCancellation::default();
    let scope = ExecutionScope::new(
        Arc::new(AtomicBool::new(false)),
        Some(Instant::now() + Duration::from_secs(5)),
    );
    assert!(
        prepared
            .initialize_consistent(
                RunId::from_bytes([8; 16]),
                request.clone(),
                Controls::default(),
                scope,
                cancellation
            )
            .await
            .is_err()
    );
    let mut wrong_time = request.clone();
    wrong_time.time = 0.5;
    let cancellation = FlightCancellation::default();
    let scope = ExecutionScope::new(
        cancellation.flag(),
        Some(Instant::now() + Duration::from_secs(5)),
    );
    assert!(
        prepared
            .initialize_consistent(
                RunId::from_bytes([8; 16]),
                wrong_time,
                Controls::default(),
                scope,
                cancellation
            )
            .await
            .is_err()
    );
    let cancellation = FlightCancellation::default();
    cancellation.cancel();
    let scope = ExecutionScope::new(
        cancellation.flag(),
        Some(Instant::now() + Duration::from_secs(5)),
    );
    assert!(
        prepared
            .initialize_consistent(
                RunId::from_bytes([8; 16]),
                request,
                Controls::default(),
                scope,
                cancellation
            )
            .await
            .is_err()
    );
    let owner = Arc::downgrade(&result.inner._owner);
    let retained = result.clone();
    drop(result);
    assert!(owner.upgrade().is_some());
    assert_eq!(retained.report().termination.name, "IDA_SUCCESS");
    drop(retained);
    assert!(owner.upgrade().is_none());
}
