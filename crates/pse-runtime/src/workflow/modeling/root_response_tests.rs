// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public Root request, physical response publication and independent withholding.
use super::*;
use crate::math::{settings::SensitivityRequest, solves::NumericalInputs};
use crate::workflow::tests as fixture;
use pse_backend_native::solve::{Backend, SolveIntent, SolverSelection};
use pse_kernels::DerivativeOrder;
use pse_relations::{
    columnar::RelationRow,
    generated::{
        enums::NumericalTarget,
        runtime::{local_validity, parametric_sensitivities, solve_runs},
    },
};
type TestResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::test]
async fn retained_root_action_screens_target_then_original_corrector_qualifies() {
    use crate::math::solves::Outcome;
    use pse_backend_native::solve::{Controls, Execution};
    use pse_model::strategy::BranchPolicy;
    let rows=pse_authoring::language::parse(
        "package p { def Root { param p:Scalar=4; var x:Scalar; eq root:x*x==p; annotation start x(2); } }",
        SemanticId::NIL,pse_authoring::language::IdentityPolicy::Named,pse_authoring::ParseBudget::default(),
    ).unwrap();
    let root = rows
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    let runtime = fixture::runtime_with(256 << 20, 16 << 20, 1 << 30);
    let package = runtime.modeling_package(rows, fixture::physical()).unwrap();
    let mut solver = fixture::profile();
    solver.intent = SolveIntent::Root;
    solver.selection = SolverSelection::Explicit(Backend::Kinsol);
    let mut analysis = ModelingAnalysis {
        root,
        instance: pse_modeling::specialize::root_instance(root),
        bindings: Bindings::default(),
        limits: Limits::default(),
        case: Default::default(),
        order: DerivativeOrder::First,
        compiler: fixture::compiler_profile(),
        solver,
        numerical: NumericalInputs::default(),
    };
    analysis.bindings.demand.push("p".into());
    analysis.case.variables.insert(
        "x".into(),
        pse_compiler::workspace::ModelingVariableState {
            lower: Some(Some(0.)),
            upper: None,
            fixed: None,
        },
    );
    let cancel = crate::CancelSource::new();
    let plain = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    let parameter = plain.model.model.compiled().model.paths["p"];
    analysis.solver.sensitivity = Some(SensitivityRequest {
        parameters: vec![parameter],
        reduced_hessian: false,
        propagation: None,
    });
    let mut prepared = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    prepared.solve = prepared.solve.retaining_factor().unwrap();
    let base = package
        .solve_case(prepared, analysis.compiler, &cancel)
        .await
        .unwrap();
    assert!(base.completion.decision.permits_use());
    let predictor = base.root_predictor().unwrap();
    analysis.case.values.insert("p".into(), 4.04);
    let mut target = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    let scope = pse_kernels::ExecutionScope::new(
        Arc::default(),
        Some(std::time::Instant::now() + std::time::Duration::from_secs(10)),
    );
    let execution = Execution::within(
        scope.cancellation().clone(),
        &Controls::default(),
        scope.clone(),
    )
    .unwrap();
    let (proposal, work) = base
        .root_prediction(&target.solve, BranchPolicy::any_qualified(), &execution)
        .unwrap();
    assert!((proposal.values().next().unwrap().1 - 2.01).abs() < 1e-12);
    assert_eq!(work.backsolves, 1);
    let screened = runtime
        .native()
        .screen_start(
            target.solve.clone(),
            proposal,
            BranchPolicy::any_qualified(),
            scope,
            &cancel,
        )
        .await
        .unwrap();
    target.solve = target.solve.with_screened_start(&screened).unwrap();
    let corrected = package
        .solve_case(target, analysis.compiler, &cancel)
        .await
        .unwrap();
    assert!(corrected.completion.decision.permits_use());
    let Outcome::Native(report) = &corrected.outcome else {
        panic!("native original corrector expected");
    };
    assert!((report.candidate.as_ref().unwrap().primal[0] - 4.04_f64.sqrt()).abs() < 1e-8);
    // Retained factor and source point survive independent native-session teardown.
    assert_eq!(predictor.factor().point(), [2.]);
    let mut changed = analysis.clone();
    changed.case.variables.get_mut("x").unwrap().lower = Some(Some(1.));
    let different = package.prepare_analysis(&changed, &cancel).await.unwrap();
    assert!(
        base.root_prediction(&different.solve, BranchPolicy::any_qualified(), &execution)
            .is_err()
    );
    // A declared approximate fidelity follows the same original screening/correction path.
    use pse_math::surrogate::{FidelityCorrespondence, FidelityEvaluator, SurrogateOptions};
    #[derive(Debug)]
    struct ApproximateRootMerit(FidelityCorrespondence);
    impl FidelityEvaluator for ApproximateRootMerit {
        type Error = pse_backend_native::ProblemError;
        fn correspondence(&self) -> &FidelityCorrespondence {
            &self.0
        }
        fn evaluate(&mut self, x: &[f64], values: &mut [f64]) -> Result<(), Self::Error> {
            values[0] = (x[0] - 2.02).powi(2);
            Ok(())
        }
    }
    let mut target = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    let identity = target.solve.original_identity().unwrap();
    let correspondence = FidelityCorrespondence {
        original_target: identity,
        model_target: identity,
        source: pse_ids::ContentHash::from_bytes([17; 32]),
        fidelity: pse_ids::ContentHash::from_bytes([18; 32]),
        coordinates: vec![target.model.model.compiled().model.paths["x"]],
        outputs: vec![SemanticId::NIL],
    };
    let options = SurrogateOptions {
        bounds: vec![(1., 3.)],
        initial: vec![vec![1.], vec![2.02], vec![3.]],
        iterations: 2,
        evaluations: 20,
        infill_starts: 1,
        regression: pse_math::surrogate::RegressionSpec::CONSTANT,
        correlation: pse_math::surrogate::CorrelationSpec::MATERN52,
        theta: pse_math::surrogate::ThetaTuning::Fixed(vec![0.1].into()),
        gp_starts: 0,
        gp_evaluations: 5,
        seed: 74,
        phase_steps: (1, 1),
        radius: 0.2,
        contraction: 0.5,
        threads: 1,
        stack_bytes: runtime.shared.budget().math.stack_bytes,
        foreign_bytes: 1 << 20,
        time: std::time::Duration::from_secs(10),
    };
    let scope = pse_kernels::ExecutionScope::new(
        Arc::default(),
        Some(std::time::Instant::now() + std::time::Duration::from_secs(10)),
    );
    let handle = runtime
        .native()
        .prepare_surrogate(
            ApproximateRootMerit(correspondence.clone()),
            options,
            scope.clone(),
            &cancel,
        )
        .await
        .unwrap();
    let statistical = runtime
        .native()
        .advance_surrogate(&handle, &cancel)
        .await
        .unwrap()
        .unwrap();
    let proposal = target
        .solve
        .surrogate_start(
            &statistical,
            &correspondence,
            handle.key(),
            BranchPolicy::any_qualified(),
        )
        .unwrap();
    assert!(proposal.source().accuracy.is_none());
    let screened = runtime
        .native()
        .screen_start(
            target.solve.clone(),
            proposal,
            BranchPolicy::any_qualified(),
            scope,
            &cancel,
        )
        .await
        .unwrap();
    target.solve = target.solve.with_screened_start(&screened).unwrap();
    drop(handle);
    let corrected = package
        .solve_case(target, analysis.compiler, &cancel)
        .await
        .unwrap();
    assert!(corrected.completion.decision.permits_use());
    let Outcome::Native(report) = &corrected.outcome else {
        panic!("native original corrector expected");
    };
    assert!((report.candidate.as_ref().unwrap().primal[0] - 4.04_f64.sqrt()).abs() < 1e-8);
}

#[tokio::test]
async fn derived_preparation_corrects_original_with_declared_recovery_and_truthful_trace() {
    use crate::math::solves::{DerivedRequest, Outcome};
    use pse_backend_native::solve::StartPolicy;
    use pse_model::generated::enums::{
        CandidateUse, NumericalAttemptObservation, NumericalEventKind,
    };
    use pse_model::strategy::{
        MechanismKind, NumericalStrategy, Position, ProfileRef, StartOrigin, Transition, WorkLimits,
    };
    let rows = pse_authoring::language::parse(
        "package p { def Root { var x:Scalar; annotation start x(0); eq balance:x==3; } }",
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
    let runtime = fixture::runtime_with(128 << 20, 1 << 20, 512 << 20);
    let package = runtime.modeling_package(rows, fixture::physical()).unwrap();
    let mut solver = fixture::profile();
    solver.intent = SolveIntent::Root;
    solver.selection = SolverSelection::Explicit(Backend::Kinsol);
    let analysis = ModelingAnalysis {
        root,
        instance: pse_modeling::specialize::root_instance(root),
        bindings: Bindings::default(),
        limits: Limits::default(),
        case: Default::default(),
        order: DerivativeOrder::First,
        compiler: fixture::compiler_profile(),
        solver: solver.clone(),
        numerical: NumericalInputs::default(),
    };
    let cancel = crate::CancelSource::new();
    let mut prepared = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    let scope = pse_kernels::ExecutionScope::new(
        Arc::default(),
        Some(std::time::Instant::now() + std::time::Duration::from_secs(20)),
    );
    let auxiliary = runtime
        .native()
        .prepare_derived(
            prepared.solve.clone(),
            DerivedRequest::AnchoredHomotopy {
                anchor: vec![0.],
                parameter: 0.5,
            },
            solver,
            scope.clone(),
            &cancel,
        )
        .await
        .unwrap();
    let family = auxiliary.family().key();
    let limits = WorkLimits {
        attempts: 2,
        evaluations: None,
        iterations: None,
        factorizations: None,
        proof_steps: None,
    };
    let mut declaration = NumericalStrategy::direct(StartPolicy::NoPriorStart, limits);
    declaration.start.recovery.push(StartOrigin::Auxiliary);
    declaration.mechanisms[0].profile = Some(ProfileRef {
        backend: Backend::Kinsol,
        key: prepared.solve.strategy_profile().unwrap(),
    });
    declaration.mechanisms[0]
        .starts
        .push(StartOrigin::Auxiliary);
    let mut preparation = declaration.mechanisms[0].clone();
    preparation.kind = MechanismKind::Homotopy;
    preparation.position = Position::Preparation;
    preparation.profile = Some(auxiliary.profile_ref().unwrap());
    preparation.support = vec![family];
    preparation.transitions.push(Transition::Continue);
    declaration.mechanisms.insert(0, preparation);
    prepared.solve = prepared
        .solve
        .clone()
        .with_strategy(
            declaration,
            vec![auxiliary.into(), prepared.solve.clone().into()],
        )
        .unwrap();
    let result = package
        .solve_case(prepared, analysis.compiler, &cancel)
        .await
        .unwrap();
    assert!(result.completion.decision.permits_use());
    let Outcome::Native(report) = &result.outcome else {
        panic!("original corrector missing");
    };
    assert!((report.candidate.as_ref().unwrap().primal[0] - 3.).abs() < 1e-8);
    assert!(
        (match &report
            .start_receipt
            .as_ref()
            .unwrap()
            .seed
            .as_ref()
            .unwrap()
            .payload
        {
            pse_backend_native::solve::WarmPayload::Root(x) => x[0],
            _ => panic!("root start required"),
        } - 1.5)
            .abs()
            < 1e-8
    );
    let rows = result
        .strategy
        .as_ref()
        .unwrap()
        .rows(result.run_id, 0)
        .unwrap();
    let finished = rows
        .iter()
        .filter(|row| row.kind == NumericalEventKind::Finished)
        .collect::<Vec<_>>();
    assert_eq!(finished.len(), 2);
    assert_eq!(
        finished[0].observation,
        Some(NumericalAttemptObservation::Auxiliary)
    );
    assert_eq!(finished[0].permission, Some(CandidateUse::SeedOnly));
    assert_eq!(finished[0].derived_identity, Some(family));
    assert!(finished[0].start_identity.is_some());
    assert_eq!(finished[1].start_origin, Some(StartOrigin::Auxiliary));
    assert_eq!(finished[1].transition, Some(Transition::Finish));
    assert!(
        !scope
            .cancellation()
            .load(std::sync::atomic::Ordering::Acquire)
    );
}

async fn solve_with_backend(
    boundary: bool,
    backend: Backend,
) -> TestResult<Arc<crate::workflow::RunResult>> {
    let text = "package p { def Root { param p: Scalar=4; var x: Scalar; eq root: x*x==p; annotation start x(2); } }";
    let rows = pse_authoring::language::parse(
        text,
        SemanticId::NIL,
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )?;
    let root = rows
        .iter()
        .find(|r| r.name == "Root")
        .ok_or_else(|| std::io::Error::other("Root declaration absent"))?
        .declaration_id;
    let package = fixture::runtime_with(16 << 20, 16 << 20, 1 << 30)
        .modeling_package(rows, fixture::physical())?;
    let mut solver = fixture::profile();
    solver.intent = SolveIntent::Root;
    solver.selection = SolverSelection::Explicit(backend);
    let order = DerivativeOrder::First;
    let mut analysis = ModelingAnalysis {
        root,
        instance: pse_modeling::specialize::root_instance(root),
        bindings: Bindings::default(),
        limits: Limits::default(),
        case: Default::default(),
        order,
        compiler: fixture::compiler_profile(),
        solver,
        numerical: NumericalInputs::default(),
    };
    analysis.bindings.demand.push("p".into());
    analysis.case.variables.insert(
        "x".into(),
        pse_compiler::workspace::ModelingVariableState {
            lower: Some(Some(if boundary { 2. } else { 0. })),
            upper: Some(if backend == Backend::Scip {
                Some(10.)
            } else {
                None
            }),
            fixed: None,
        },
    );
    let cancel = crate::CancelSource::new();
    let plain = package.prepare_analysis(&analysis, &cancel).await?;
    let p = plain.model.model.compiled().model.paths["p"];
    analysis.solver.sensitivity = Some(SensitivityRequest {
        parameters: vec![p],
        reduced_hessian: false,
        propagation: None,
    });
    Ok(package
        .prepare_analysis(&analysis, &cancel)
        .await?
        .start()?
        .wait()
        .await?)
}
#[tokio::test]
async fn root_response_publication_has_physical_primal_and_no_kkt_fields() {
    let result = solve_with_backend(false, Backend::Kinsol).await.unwrap();
    let validity =
        local_validity::Row::rows(&result.table("runtime.local_validity").unwrap()).unwrap();
    assert_eq!(validity.len(), 1);
    let v = &validity[0].validity;
    assert!(v.certified, "{v:?}");
    assert_eq!(v.root_rank, Some(1));
    assert!(v.root_backward_error.is_some());
    assert!(v.root_neighborhood.is_some());
    assert!(v.licq.is_none() && v.strict_complementarity.is_none() && v.second_order.is_none());
    let response = parametric_sensitivities::Row::rows(
        &result.table("runtime.parametric_sensitivities").unwrap(),
    )
    .unwrap();
    assert_eq!(response.len(), 1);
    assert_eq!(response[0].target_kind, NumericalTarget::Variable);
    assert!(response[0].dual.is_none());
    assert!((response[0].primal.unwrap() - 0.25).abs() < 1e-10);
    let h = 1e-5;
    let independently = (4.0_f64 + h).sqrt() - (4.0_f64 - h).sqrt();
    assert!((response[0].primal.unwrap() - independently / (2. * h)).abs() < 1e-9);
}
#[tokio::test]
async fn root_response_withheld_at_active_bound_keeps_base_solution() {
    let result = solve_with_backend(true, Backend::Kinsol).await.unwrap();
    let validity =
        local_validity::Row::rows(&result.table("runtime.local_validity").unwrap()).unwrap();
    assert_eq!(validity.len(), 1);
    assert!(!validity[0].validity.certified);
    assert_eq!(
        validity[0].validity.reason,
        Some(pse_relations::generated::enums::WithheldReason::NeighborhoodUnavailable)
    );
    assert!(
        parametric_sensitivities::Row::rows(
            &result.table("runtime.parametric_sensitivities").unwrap()
        )
        .unwrap()
        .is_empty()
    );
    let runs = solve_runs::Row::rows(&result.table("runtime.solve_runs").unwrap()).unwrap();
    assert!(!runs.is_empty());
    assert!(
        runs.iter()
            .any(|r| r.qualification
                == pse_relations::generated::enums::NativeQualification::Feasible)
    );
}

#[tokio::test]
async fn root_response_rank_loss_withholds_sensitivity_and_keeps_feasible_base() {
    let rows = pse_authoring::language::parse(
        "package p { def Root { param p:Scalar=0; var x:Scalar; eq root:x*x*x==p; annotation start x(0); } }",
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
    let package = fixture::runtime_with(16 << 20, 16 << 20, 1 << 30)
        .modeling_package(rows, fixture::physical())
        .unwrap();
    let mut solver = fixture::profile();
    solver.intent = SolveIntent::Root;
    solver.selection = SolverSelection::Explicit(Backend::Kinsol);
    let mut analysis = ModelingAnalysis {
        root,
        instance: pse_modeling::specialize::root_instance(root),
        bindings: Bindings::default(),
        limits: Limits::default(),
        case: Default::default(),
        order: DerivativeOrder::First,
        compiler: fixture::compiler_profile(),
        solver,
        numerical: NumericalInputs::default(),
    };
    analysis.bindings.demand.push("p".into());
    let cancel = crate::CancelSource::new();
    let prepared = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    let parameter = prepared.model.model.compiled().model.paths["p"];
    analysis.solver.sensitivity = Some(SensitivityRequest {
        parameters: vec![parameter],
        reduced_hessian: false,
        propagation: None,
    });
    let result = package
        .prepare_analysis(&analysis, &cancel)
        .await
        .unwrap()
        .start()
        .unwrap()
        .wait()
        .await
        .unwrap();
    // The native initial point satisfies x³=p exactly; its actual original Jacobian
    // is zero, so a finite square response cannot be established there.
    assert!(result.usable());
    let crate::workflow::RunReport::Modeling(steps) = result.report().unwrap() else {
        panic!("modeling report expected");
    };
    let crate::math::solves::Outcome::Native(native) = &steps[0].outcome else {
        panic!("native root report expected");
    };
    assert_eq!(native.backend, Backend::Kinsol);
    assert_eq!(native.candidate.as_ref().unwrap().primal, [0.]);
    assert!(native.quality.as_ref().unwrap().feasible());
    assert!(matches!(
        native.evidence.root_response.as_ref(),
        Some(Err(pse_backend_native::square_response::Withheld::Rank {
            rank: 0,
            dimension: 1,
            ..
        }))
    ));
    let validity =
        local_validity::Row::rows(&result.table("runtime.local_validity").unwrap()).unwrap();
    let [validity] = validity.as_slice() else {
        panic!("one validity record expected")
    };
    assert!(!validity.validity.certified);
    assert_eq!(validity.validity.root_rank, Some(0));
    assert_eq!(
        validity.validity.reason,
        Some(pse_relations::generated::enums::WithheldReason::RankDeficient)
    );
    assert!(validity.validity.licq.is_none());
    assert!(validity.validity.second_order.is_none());
    assert!(
        parametric_sensitivities::Row::rows(
            &result.table("runtime.parametric_sensitivities").unwrap()
        )
        .unwrap()
        .is_empty()
    );
    let runs = solve_runs::Row::rows(&result.table("runtime.solve_runs").unwrap()).unwrap();
    assert!(runs.iter().any(|r| r.feasible == Some(true)
        && r.qualification == pse_relations::generated::enums::NativeQualification::Feasible
        && r.objective.is_none()));
}

#[cfg(any(
    feature = "solver-ipopt",
    feature = "solver-pounce",
    feature = "solver-scip"
))]
async fn assert_root_response(backend: Backend, boundary: bool) -> TestResult<()> {
    let result = solve_with_backend(boundary, backend).await?;
    let validity = local_validity::Row::rows(&result.table("runtime.local_validity")?)?;
    assert_eq!(validity.len(), 1);
    let v = &validity[0].validity;
    let response =
        parametric_sensitivities::Row::rows(&result.table("runtime.parametric_sensitivities")?)?;
    assert!(v.licq.is_none() && v.strict_complementarity.is_none() && v.second_order.is_none());
    if boundary {
        assert!(!v.certified, "{backend:?}: {v:?}");
        assert_eq!(
            v.reason,
            Some(pse_relations::generated::enums::WithheldReason::NeighborhoodUnavailable),
            "{backend:?}: {v:?}"
        );
        assert!(response.is_empty());
    } else {
        assert!(v.certified, "{backend:?}: {v:?}");
        assert_eq!(v.root_rank, Some(1));
        assert_eq!(response.len(), 1);
        assert!(response[0].dual.is_none());
        assert!(
            (response[0]
                .primal
                .ok_or_else(|| std::io::Error::other("physical primal response absent"))?
                - 0.25)
                .abs()
                < 1e-9,
            "{backend:?}: {:?}",
            response[0]
        );
    }
    let expected = if matches!(backend, Backend::Ipopt | Backend::Pounce) {
        pse_relations::generated::enums::NativeQualification::Stationary
    } else {
        pse_relations::generated::enums::NativeQualification::Feasible
    };
    let crate::workflow::RunReport::Modeling(steps) = result.report().unwrap() else {
        panic!("modeling report expected");
    };
    let crate::math::solves::Outcome::Native(native) = &steps[0].outcome else {
        panic!("native root report expected");
    };
    assert!(
        native
            .quality
            .as_ref()
            .is_some_and(|quality| quality.feasible())
    );
    if matches!(backend, Backend::Ipopt | Backend::Pounce) {
        let kkt = native.evidence.kkt.as_ref().unwrap();
        assert_eq!(kkt.stationarity, Some(true));
        assert_eq!(kkt.complementarity, Some(true));
        assert_eq!(
            native.termination.assurance,
            pse_backend_native::solve::Assurance::LocalStationary
        );
    }
    let runs = solve_runs::Row::rows(&result.table("runtime.solve_runs")?)?;
    assert!(
        runs.iter().any(|r| r.backend == Some(backend)
            && r.feasible == Some(true)
            && r.qualification == expected),
        "{backend:?}: {runs:?}"
    );
    Ok(())
}
#[cfg(feature = "solver-ipopt")]
#[tokio::test]
async fn explicit_ipopt_root_response_uses_shared_square_analysis() {
    assert_root_response(Backend::Ipopt, false).await.unwrap();
}
#[cfg(feature = "solver-ipopt")]
#[tokio::test]
async fn explicit_ipopt_root_active_bound_withholds_response_and_keeps_base() {
    assert_root_response(Backend::Ipopt, true).await.unwrap();
}
#[cfg(feature = "solver-pounce")]
#[tokio::test]
async fn explicit_pounce_root_response_uses_shared_square_analysis() {
    assert_root_response(Backend::Pounce, false).await.unwrap();
}
#[cfg(feature = "solver-pounce")]
#[tokio::test]
async fn explicit_pounce_root_active_bound_withholds_response_and_keeps_base() {
    assert_root_response(Backend::Pounce, true).await.unwrap();
}

#[cfg(feature = "solver-scip")]
#[tokio::test]
async fn explicit_scip_root_response_uses_shared_square_analysis() {
    assert_root_response(Backend::Scip, false).await.unwrap();
}
#[cfg(feature = "solver-scip")]
#[tokio::test]
async fn explicit_scip_root_active_bound_withholds_response_and_keeps_base() {
    assert_root_response(Backend::Scip, true).await.unwrap();
}
