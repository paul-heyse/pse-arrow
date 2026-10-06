// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual POUNCE targets obey task admission and execute automatic profile recovery.
use super::*;
use crate::{math::solves::Outcome, workflow::tests as fixture};
use pse_backend_native::{
    execution::BackendSettings,
    presolve::Policy,
    solve::{Backend, HessianMode, ReusePolicy, SolveIntent, SolverSelection, StartPolicy},
};
use pse_model::{
    generated::enums::{CandidateUse, NumericalAttemptObservation, NumericalEventKind},
    strategy::{Phase, Transition, WorkLimits},
};

async fn solve(
    mode: HessianMode,
    cap: Option<u64>,
) -> Result<ModelingResult, Box<dyn std::error::Error>> {
    solve_source(
        "package p { def Root { var x:Scalar; eq root:x*x==4; annotation start x(1); } }",
        mode,
        Some(WorkLimits {
            attempts: 1,
            evaluations: cap,
            iterations: None,
            factorizations: None,
            proof_steps: None,
        }),
    )
    .await
}

async fn solve_source(
    source: &str,
    mode: HessianMode,
    limits: Option<WorkLimits>,
) -> Result<ModelingResult, Box<dyn std::error::Error>> {
    let rows = pse_authoring::language::parse(
        source,
        SemanticId::NIL,
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )?;
    let root = rows
        .iter()
        .find(|row| row.name == "Root")
        .ok_or("fixture Root declaration absent")?
        .declaration_id;
    let runtime = fixture::runtime_with(64 << 20, 16 << 20, 1 << 30);
    let package = runtime.modeling_package(rows, fixture::physical()).await?;
    let mut solver = fixture::profile();
    solver.selection = SolverSelection::Explicit(Backend::Pounce);
    solver.intent = SolveIntent::Root;
    solver.presolve = Policy::Off;
    solver.controls.start = StartPolicy::NoPriorStart;
    solver.controls.reuse = ReusePolicy::Fresh;
    solver.controls.hessian = mode;
    solver.backend = BackendSettings::Pounce(Default::default());
    solver.composition.limits = limits;
    assert_eq!(
        solver.composition.policy,
        pse_model::strategy::CompositionPolicy::Auto
    );
    let analysis = ModelingAnalysis {
        root,
        instance: pse_modeling::specialize::root_instance(root),
        bindings: Bindings::default(),
        limits: Limits::default(),
        case: Default::default(),
        order: pse_kernels::DerivativeOrder::First,
        compiler: fixture::compiler_profile(),
        solver,
        numerical: Default::default(),
    };
    let cancel = crate::CancelSource::new();
    let prepared = package.prepare_analysis(&analysis, &cancel).await?;
    assert_eq!(
        prepared.solve.required_order(),
        pse_kernels::DerivativeOrder::First
    );
    Ok(package
        .solve_case(prepared, analysis.compiler, &cancel)
        .await?)
}

#[tokio::test]
async fn actual_auto_pounce_stationary_failure_runs_generated_second_opinion() {
    use crate::math::strategy::OriginalConclusion;
    use pse_model::strategy::{MechanismKind, Position};
    // A finite, smooth stationary nonroot. No callback failure, invented native stop,
    // shortened native iteration cap or replacement-start permission opens the ladder.
    let outcome = solve_source(
        "package p { def Root { var x:Scalar; eq root:x*x==4; annotation start x(0); } }",
        HessianMode::LimitedMemory,
        None,
    )
    .await;
    let trace = match &outcome {
        Ok(result) => result.strategy.as_ref().unwrap(),
        Err(error) => error
            .downcast_ref::<WorkflowError>()
            .unwrap()
            .strategy_trace()
            .unwrap(),
    };
    assert_eq!(trace.backend, Some(Backend::Pounce));
    assert_eq!(trace.declaration.mechanisms[0].kind, MechanismKind::Direct);
    let generated = trace
        .declaration
        .mechanisms
        .iter()
        .enumerate()
        .filter(|(_, mechanism)| {
            mechanism.kind == MechanismKind::NativeGlobalization
                && mechanism.position == Position::Execution
        })
        .collect::<Vec<_>>();
    assert!(
        !generated.is_empty(),
        "actual numerical trajectory must generate a second opinion: {:?}",
        trace
            .events
            .iter()
            .map(|event| (
                event.mechanism,
                event.kind,
                event.phase,
                event.observation,
                event.transition,
                event.permission,
                event.work
            ))
            .collect::<Vec<_>>()
    );
    let baseline = trace
        .events
        .iter()
        .find(|event| {
            event.mechanism == 0
                && event.phase == Phase::Assessment
                && event.kind == NumericalEventKind::Finished
                && event.original.is_some()
        })
        .unwrap();
    assert!(matches!(
        baseline.observation,
        Some(
            NumericalAttemptObservation::Limited
                | NumericalAttemptObservation::NumericalFailure
                | NumericalAttemptObservation::Stalled
        )
    ));
    assert!(matches!(
        baseline.original.as_ref(),
        Some(OriginalConclusion::Refused { .. })
    ));
    assert_eq!(baseline.permission, Some(CandidateUse::DiagnosticOnly));
    assert!(
        trace.events.iter().all(|event| !matches!(
            event.observation,
            Some(
                NumericalAttemptObservation::ResourceExhausted
                    | NumericalAttemptObservation::ContractFailure
                    | NumericalAttemptObservation::CapabilityRefusal
                    | NumericalAttemptObservation::Panic
                    | NumericalAttemptObservation::Cancelled
            )
        )),
        "finite scientific fixture must reach actual numerical observations"
    );
    let mut profiles = std::collections::BTreeSet::new();
    for (index, mechanism) in
        std::iter::once((0, &trace.declaration.mechanisms[0])).chain(generated.iter().copied())
    {
        let profile = mechanism.profile.unwrap();
        assert_eq!(profile.backend, Backend::Pounce);
        assert!(
            profiles.insert(profile.key),
            "each actually attempted profile has a distinct identity"
        );
        assert!(trace.events.iter().any(|event| event.mechanism == index
            && event.kind == NumericalEventKind::Started
            && event.phase == Phase::Native));
        let work = trace
            .events
            .iter()
            .filter(|event| event.mechanism == index)
            .filter_map(|event| event.work)
            .find(|charge| charge.phase == Phase::Native)
            .unwrap();
        assert_eq!(work.observed.attempts, 1);
        assert!(
            work.observed.evaluations.is_some_and(|count| count > 0),
            "generated operation must execute actual callbacks"
        );
        assert!(trace.events.iter().any(|event| event.mechanism == index
            && event.kind == NumericalEventKind::Finished
            && event.phase == Phase::Assessment
            && event.original.is_some()
            && event.permission.is_some()));
    }
    let assessed = trace
        .events
        .iter()
        .rev()
        .find(|event| {
            event.kind == NumericalEventKind::Finished && event.phase == Phase::Assessment
        })
        .unwrap();
    match &outcome {
        Ok(result) => {
            assert_eq!(
                trace.original,
                result.prepared.solve.original_identity().unwrap()
            );
            assert_eq!(
                assessed.original.as_ref().unwrap().satisfied(),
                result.completion.decision.permits_use()
            );
            assert_eq!(
                assessed.permission,
                Some(result.completion.decision.usability)
            );
            let Outcome::Native(report) = &result.outcome else {
                panic!("generated native profile must retain its actual report");
            };
            assert_eq!(report.backend, Backend::Pounce);
            assert!(
                report
                    .evidence
                    .work
                    .evaluations
                    .is_some_and(|count| count > 0)
            );
            // Noncumulative library profiles vary only MC64 scaling or adaptive mu.
            let effective: serde_json::Value =
                serde_json::from_str(&report.provenance["feral.effective"]).unwrap();
            let mc64 = effective["scaling"] == "mc64_symmetric";
            let adaptive = report.options.get("mu_strategy")
                == Some(&pse_backend_native::solve::OptionValue::Text(
                    "adaptive".into(),
                ));
            assert!(
                mc64 ^ adaptive,
                "the acting profile must change only one baseline setting"
            );
        }
        Err(_) => {
            assert!(matches!(
                assessed.original.as_ref(),
                Some(OriginalConclusion::Unavailable { .. })
            ));
            assert_eq!(assessed.permission, Some(CandidateUse::Unusable));
        }
    }
    println!(
        "actual Auto POUNCE: baseline {:?}/{:?}, generated {}, assessments {:?}",
        baseline.observation,
        baseline.permission,
        generated.len(),
        trace
            .events
            .iter()
            .filter(|event| event.original.is_some())
            .map(|event| (
                event.mechanism,
                event.observation,
                event.permission,
                event.work.map(|charge| charge.observed.evaluations)
            ))
            .collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn actual_pounce_evaluation_cap_refuses_before_callbacks_and_preserves_work() {
    let refused = solve(HessianMode::LimitedMemory, Some(0))
        .await
        .unwrap_err();
    let error = refused.downcast_ref::<WorkflowError>().unwrap();
    assert_eq!(
        crate::math::strategy::workflow_failure(error),
        NumericalAttemptObservation::ResourceExhausted
    );
    let trace = error.strategy_trace().unwrap();
    assert_eq!(trace.backend, Some(Backend::Pounce));
    assert_eq!(trace.declaration.mechanisms.len(), 1);
    assert_eq!(
        trace
            .events
            .iter()
            .filter(|event| event.kind == NumericalEventKind::Started)
            .count(),
        1,
        "a work refusal cannot retry the native operation"
    );
    let charges = trace
        .events
        .iter()
        .filter_map(|event| event.work)
        .collect::<Vec<_>>();
    assert_eq!(
        charges.len(),
        1,
        "no original checks run without a candidate"
    );
    assert_eq!(charges[0].phase, Phase::Native);
    assert_eq!(charges[0].observed.attempts, 1);
    assert_eq!(charges[0].observed.evaluations, Some(0));
    let assessed = trace
        .events
        .iter()
        .find(|event| {
            event.phase == Phase::Assessment && event.kind == NumericalEventKind::Finished
        })
        .unwrap();
    assert_eq!(
        assessed.observation,
        Some(NumericalAttemptObservation::ResourceExhausted)
    );
    assert_eq!(assessed.transition, Some(Transition::Stop));
    assert_eq!(assessed.permission, Some(CandidateUse::Unusable));
    let crate::math::strategy::OriginalConclusion::Unavailable { cause } =
        assessed.original.as_ref().unwrap()
    else {
        panic!("pre-operation work refusal cannot assess an original candidate")
    };
    let pse_backend_native::ProblemError::Math(pse_math::MathError::Typed {
        cause: source, ..
    }) = cause.as_ref()
    else {
        panic!("original refusal must retain its runtime source: {cause:?}")
    };
    assert!(
        matches!(
            source
                .as_error()
                .downcast_ref::<crate::math::MathRuntimeError>(),
            Some(crate::math::MathRuntimeError::Solve(
                pse_backend_native::ProblemError::Limit {
                    kind: pse_backend_native::LimitKind::Work,
                    ..
                }
            ))
        ),
        "{source:?}"
    );
    assert!(Arc::ptr_eq(cause, assessed.cause.as_ref().unwrap()));
    assert!(
        trace
            .events
            .iter()
            .filter_map(|event| event.cause.as_ref())
            .all(|event_cause| Arc::ptr_eq(cause, event_cause))
    );
    let accepted = solve(HessianMode::LimitedMemory, Some(10000))
        .await
        .unwrap();
    assert!(
        accepted.completion.decision.permits_use(),
        "{:?}",
        accepted.completion
    );
    let Outcome::Native(report) = &accepted.outcome else {
        panic!("actual POUNCE attempt absent")
    };
    assert_eq!(report.backend, Backend::Pounce);
    assert!(report.evidence.work.evaluations.is_some_and(|n| n > 0));
    let trace = accepted.strategy.as_ref().unwrap();
    let native = trace
        .events
        .iter()
        .filter_map(|event| event.work)
        .find(|charge| charge.phase == Phase::Native)
        .unwrap();
    assert!(native.observed.evaluations.is_some_and(|n| n > 0));
    assert!(native.observed.evaluations.unwrap() >= report.evidence.work.evaluations.unwrap());
    let assessment = trace
        .events
        .iter()
        .filter_map(|event| event.work)
        .find(|charge| charge.phase == Phase::Assessment)
        .unwrap();
    assert_eq!(assessment.observed.attempts, 0);
    // This fixture has no report/expectation observation program; the native
    // equation checks are already included in the native work charge.
    assert_eq!(assessment.observed.evaluations, Some(0));
    assert_ne!(native.charging_owner, assessment.charging_owner);
    assert!(
        native.observed.evaluations.unwrap() + assessment.observed.evaluations.unwrap() <= 10000
    );
    assert!(
        trace
            .events
            .iter()
            .any(|event| event.phase == Phase::Assessment && event.decision.is_some())
    );
}

#[tokio::test]
async fn compiler_first_order_partitioned_and_fd_profiles_act_and_assess_original() {
    let mut profiles = std::collections::BTreeSet::new();
    for mode in [HessianMode::Partitioned, HessianMode::FiniteDifference] {
        let result = solve(mode, None).await.unwrap();
        assert!(
            result.completion.decision.permits_use(),
            "{mode:?}: {:?}",
            result.completion
        );
        let Outcome::Native(report) = &result.outcome else {
            panic!("actual native target absent")
        };
        assert_eq!(report.backend, Backend::Pounce);
        let stats = report.pounce_statistics.as_ref().unwrap();
        match mode {
            HessianMode::Partitioned => assert!(stats.partitioned_elements > 0),
            HessianMode::FiniteDifference => assert_eq!(stats.fd_hessian_n, 1),
            _ => panic!("unexpected test curvature mode: {mode:?}"),
        }
        let trace = result.strategy.as_ref().unwrap();
        profiles.insert(trace.profile);
        assert!(
            trace
                .events
                .iter()
                .any(|event| event.phase == Phase::Assessment
                    && event.original.as_ref().is_some_and(|original| matches!(
                        original,
                        crate::math::strategy::OriginalConclusion::Satisfied
                    )))
        );
    }
    assert_eq!(
        profiles.len(),
        2,
        "effective curvature profiles retain distinct identities"
    );
}
