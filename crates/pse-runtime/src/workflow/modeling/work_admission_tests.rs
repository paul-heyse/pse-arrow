// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual original POUNCE targets consume the task's pre-operation evaluation cap.
use super::*;
use crate::{math::solves::Outcome, workflow::tests as fixture};
use pse_backend_native::{
    execution::BackendSettings,
    presolve::Policy,
    solve::{Backend, HessianMode, ReusePolicy, SolveIntent, SolverSelection, StartPolicy},
};
use pse_model::strategy::{Phase, WorkLimits};

async fn solve(
    mode: HessianMode,
    cap: Option<u64>,
) -> Result<ModelingResult, Box<dyn std::error::Error>> {
    let rows = pse_authoring::language::parse(
        "package p { def Root { var x:Scalar; eq root:x*x==4; annotation start x(1); } }",
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
    let package = runtime.modeling_package(rows, fixture::physical())?;
    let mut solver = fixture::profile();
    solver.selection = SolverSelection::Explicit(Backend::Pounce);
    solver.intent = SolveIntent::Root;
    solver.presolve = Policy::Off;
    solver.controls.start = StartPolicy::NoPriorStart;
    solver.controls.reuse = ReusePolicy::Fresh;
    solver.controls.hessian = mode;
    solver.backend = BackendSettings::Pounce(Default::default());
    solver.composition.limits = Some(WorkLimits {
        attempts: 1,
        evaluations: cap,
        iterations: None,
        factorizations: None,
        proof_steps: None,
    });
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
async fn actual_pounce_evaluation_cap_refuses_before_callbacks_and_preserves_work() {
    let refused = solve(HessianMode::LimitedMemory, Some(0)).await.unwrap();
    assert!(!refused.completion.decision.permits_use());
    let trace = refused.strategy.as_ref().unwrap();
    let charges = trace
        .events
        .iter()
        .filter_map(|event| event.work)
        .collect::<Vec<_>>();
    assert!(
        charges
            .iter()
            .all(|charge| charge.observed.evaluations == Some(0)),
        "{charges:?}"
    );
    assert!(trace.events.iter().any(|event| event.cause.is_some()));
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
    let trace = accepted.strategy.as_ref().unwrap();
    let native = trace
        .events
        .iter()
        .filter_map(|event| event.work)
        .find(|charge| charge.phase == Phase::Native)
        .unwrap();
    assert!(native.observed.evaluations.is_some_and(|n| n > 0));
    assert!(native.observed.evaluations.unwrap() >= report.evidence.work.evaluations.unwrap());
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
