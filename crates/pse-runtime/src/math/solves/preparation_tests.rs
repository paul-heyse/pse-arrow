// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Contextual readiness through the workflow/compiler/runtime boundary, before any attempt.
use super::*;
use crate::workflow::{ModelingPackage, ModelingSolvePreparation, tests as fixture};
use pse_compiler::workspace::ModelingCaseBindings;
use pse_ids::SemanticId;
use pse_kernels::DerivativeOrder;
use pse_modeling::{Bindings, DeclarationId, Limits};

async fn package(text: &str) -> (ModelingPackage, DeclarationId) {
    let (package, root, _) = package_and_runtime(text).await;
    (package, root)
}
async fn package_and_runtime(
    text: &str,
) -> (ModelingPackage, DeclarationId, crate::workflow::Runtime) {
    package_with_runtime(text, fixture::runtime()).await
}
async fn package_with_runtime(
    text: &str,
    runtime: crate::workflow::Runtime,
) -> (ModelingPackage, DeclarationId, crate::workflow::Runtime) {
    let rows = pse_authoring::language::parse(
        text,
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
    let package = runtime
        .modeling_package(rows, fixture::physical())
        .await
        .unwrap();
    (package, root, runtime)
}

async fn prepare(
    text: &str,
    intent: SolveIntent,
    selection: SolverSelection,
) -> ModelingSolvePreparation {
    let (package, root) = package(text).await;
    package
        .prepare_solve(
            root,
            pse_modeling::specialize::root_instance(root),
            Bindings::default(),
            Limits::default(),
            ModelingCaseBindings::default(),
            DerivativeOrder::First,
            fixture::compiler_profile(),
            SolverProfile {
                intent,
                selection,
                ..Default::default()
            },
            NumericalInputs::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap()
}

fn ready(solve: &PreparedSolve) -> &AlgebraicCase {
    let decision = solve.route_decision().unwrap();
    assert_eq!(decision.state, routing::AssessmentState::Ready);
    assert!(decision.evidence.is_empty(), "{decision:?}");
    assert!(decision.artifacts.is_empty(), "{decision:?}");
    let Representation::Algebraic(case) = &solve.representation else {
        panic!("expected an authored algebraic representation");
    };
    assert!(
        case.case.is_some(),
        "the selected numeric program is retained"
    );
    case
}

#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn capped_original_kinsol_root_uses_actual_evaluation_hooks_and_refuses_opaque_counters() {
    let (package, root, runtime) = package_and_runtime(
        "package p { def Root { var x:Scalar; eq root:x*x==4; annotation start x(1); } }",
    )
    .await;
    let limits = pse_model::strategy::WorkLimits {
        attempts: 4,
        evaluations: Some(500),
        iterations: None,
        factorizations: None,
        proof_steps: None,
    };
    let solver = SolverProfile {
        intent: SolveIntent::Root,
        selection: SolverSelection::Explicit(Backend::Kinsol),
        presolve: native::presolve::Policy::Off,
        composition: pse_model::strategy::CompositionRequest {
            limits: Some(limits),
            ..Default::default()
        },
        ..Default::default()
    };
    let cancel = crate::CancelSource::new();
    let mut prepared = package
        .prepare_solve(
            root,
            pse_modeling::specialize::root_instance(root),
            Bindings::default(),
            Limits::default(),
            ModelingCaseBindings::default(),
            DerivativeOrder::First,
            fixture::compiler_profile(),
            solver,
            NumericalInputs::default(),
            &cancel,
        )
        .await
        .unwrap();
    let scope = pse_kernels::ExecutionScope::new(
        Arc::default(),
        Some(std::time::Instant::now() + std::time::Duration::from_secs(20)),
    );
    let admission = super::super::strategy::admission::TaskAdmission::new(
        limits,
        scope.clone(),
        Some(runtime.native().pool.clone()),
        false,
    );
    assert!(
        prepared
            .solve
            .work_admitted(limits, &scope, admission.clone())
    );
    for constrained in [
        pse_model::strategy::WorkLimits {
            iterations: Some(1),
            ..limits
        },
        pse_model::strategy::WorkLimits {
            factorizations: Some(1),
            ..limits
        },
        pse_model::strategy::WorkLimits {
            proof_steps: Some(1),
            ..limits
        },
    ] {
        assert!(
            !prepared
                .solve
                .work_admitted(constrained, &scope, admission.clone())
        );
    }
    let mut preprocessing = prepared.solve.clone();
    preprocessing.profile.presolve = native::presolve::Policy::Auto;
    assert!(!preprocessing.work_admitted(limits, &scope, admission.clone()));
    let mut sensitivity = prepared.solve.clone();
    sensitivity.profile.sensitivity = Some(super::super::settings::SensitivityRequest {
        parameters: Vec::new(),
        reduced_hessian: false,
        propagation: None,
    });
    assert!(!sensitivity.work_admitted(limits, &scope, admission.clone()));
    prepared.solve = prepared
        .solve
        .within_admitted_task(scope, admission.clone())
        .unwrap();
    let result = package
        .solve_case(prepared, fixture::compiler_profile(), &cancel)
        .await
        .unwrap();
    assert!(result.accepted, "diagnostic={:?}", result.diagnostic());
    assert!(matches!(&result.outcome,Outcome::Native(report) if report.backend==Backend::Kinsol));
    let trace = result.strategy.as_ref().unwrap();
    assert!(
        trace
            .events
            .iter()
            .any(|event| event.kind == pse_model::generated::enums::NumericalEventKind::Started)
    );
    let evaluations = admission.observation().unwrap().evaluations.unwrap();
    assert!(evaluations > 0 && evaluations <= 500);
}

#[cfg(all(feature = "solver-scip", feature = "solver-ipopt"))]
#[tokio::test]
async fn compiled_factorable_pricing_retains_separate_demanded_callbacks() {
    let (package, root, runtime) = package_with_runtime(
        "package p { def Root { param size:Power=1{W}; var x:Scalar; var y:Scalar; var on:Indicator in binary; annotation bounds x(0,2); annotation bounds y(0,1); annotation start x(1); annotation start y(1); annotation start on(1{1}); eq link:size*y==size*on; eq floor:x>=y; let cost:Scalar=x*x-3*y; annotation objective cost(minimize); } }",
        fixture::runtime_on(
            1 << 30,
            crate::math::MathPolicy {
                worker_bytes: 8 << 20,
                workspace_bytes: 16 << 20,
                foreign_bytes: 256 << 20,
                ..Default::default()
            },
        ),
    ).await;
    for (hessian, order) in [
        (HessianMode::Exact, DerivativeOrder::Second),
        (HessianMode::LimitedMemory, DerivativeOrder::First),
    ] {
        let driver = crate::CancelSource::new();
        let prepared = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                DerivativeOrder::Value,
                fixture::compiler_profile(),
                SolverProfile {
                    intent: SolveIntent::Optimize,
                    selection: SolverSelection::Explicit(Backend::Scip),
                    controls: Controls {
                        hessian,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                NumericalInputs::default(),
                &driver,
            )
            .await
            .unwrap();
        let case = ready(&prepared.solve);
        assert_eq!(
            case.prepared.compiled().plan.order(),
            DerivativeOrder::Value
        );
        assert_eq!(
            case.case.as_ref().unwrap().assembly.order(),
            DerivativeOrder::Value
        );
        assert_eq!(
            case.pricing_case
                .as_ref()
                .unwrap()
                .executable
                .assembly
                .order(),
            order
        );
        assert_eq!(prepared.solve.required_order(), order);
        assert_eq!(prepared.solve.profile.controls.hessian, hessian);
        assert_eq!(
            prepared.solve.profile.controls.start,
            StartPolicy::NoPriorStart
        );
        let result = runtime
            .native()
            .solve(prepared.solve)
            .unwrap()
            .finish()
            .await
            .unwrap();
        let Outcome::Native(report) = &result.outcome else {
            panic!("expected an actual factorable native solve");
        };
        assert!(
            !report.metrics.contains_key("resolve.refused"),
            "{:?}",
            report.metrics
        );
        let commitment = report
            .candidate
            .as_ref()
            .and_then(|candidate| candidate.commitment.as_ref())
            .unwrap_or_else(|| {
                let resolve = report.metrics.iter()
                    .filter(|(name, _)| name.starts_with("resolve."))
                    .collect::<Vec<_>>();
                panic!("required fixed-assignment commitment missing: hessian={hessian:?}, termination={:?}, primal_source={:?}, quality={:?}, resolve={resolve:?}", report.termination, report.evidence.global.as_ref().map(|evidence| evidence.primal), report.quality);
            });
        assert_eq!(commitment.columns.len(), 1);
        assert_eq!(commitment.columns[0].1, (1.0, 1.0));
        assert!(report.quality.as_ref().unwrap().feasible());
        assert!(report.evidence.kkt.is_some());
    }
}

#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn demanded_krylov_route_prepares_actions_from_value_under_a_narrow_jet_budget() {
    let (package, root, runtime) = package_and_runtime(
        "package p { def Root { var x:Scalar; var y:Scalar; annotation start x(2); annotation start y(3); eq first:x*x==1; eq second:y==x+1; } }",
    ).await;
    let mut compiler = fixture::compiler_profile();
    compiler.evaluation.derivative_components = 2;
    let prepared = package
        .prepare_solve(
            root,
            pse_modeling::specialize::root_instance(root),
            Bindings::default(),
            Limits::default(),
            ModelingCaseBindings::default(),
            DerivativeOrder::Value,
            compiler,
            SolverProfile {
                intent: SolveIntent::Root,
                selection: SolverSelection::Explicit(Backend::Kinsol),
                backend: BackendSettings::Kinsol(native::settings::kinsol::Method {
                    linear: native::settings::kinsol::Linear::Spgmr {
                        dimension: pse_model::scalars::PositiveCount::try_new(3).unwrap(),
                    },
                    ..Default::default()
                }),
                ..Default::default()
            },
            NumericalInputs::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let case = ready(&prepared.solve);
    assert_eq!(case.prepared.prepared.plan.order(), DerivativeOrder::Value);
    assert!(case.prepared.prepared.plan.has_directional_actions());
    assert_eq!(prepared.solve.required_order(), DerivativeOrder::First);
    assert_eq!(
        case.prepared.prepared.facts.prepared_derivatives,
        DerivativeOrder::Value
    );
    // Exercise the full native JVP consumer; automatic block reconstruction has its own
    // controls and can legitimately finish without a single original native report.
    let original = prepared.solve;
    let mut declaration = original.numerical_strategy();
    declaration.mechanisms[0].profile = Some(pse_model::strategy::ProfileRef {
        backend: Backend::Kinsol,
        key: original.strategy_profile().unwrap(),
    });
    let solve = original
        .clone()
        .with_strategy(declaration, vec![original.into()])
        .unwrap();
    let result = runtime
        .native()
        .solve(solve)
        .unwrap()
        .finish()
        .await
        .unwrap();
    let Outcome::Native(report) = &result.outcome else {
        panic!("expected the declared original native JVP operation")
    };
    assert!(report.quality.as_ref().unwrap().feasible(), "{report:?}");
    assert_eq!(
        report.termination.category,
        Termination::Success,
        "{report:?}"
    );
    assert!(report.metrics.contains_key("KINGetNumJtimesEvals"));
}

#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn compiled_preconditioned_krylov_prepares_first_and_actions_for_main_and_conditional() {
    use native::settings::kinsol::{Linear, Method};
    let (package, root, runtime) = package_and_runtime(
        "package p { def Root { var x:Scalar; var y:Scalar; annotation start x(1.5); annotation start y(1.2); eq first:x*x+y==5; eq second:y*y+x==3; } }",
    ).await;
    let service = runtime.native();
    let driver = crate::CancelSource::new();
    let scope = pse_kernels::ExecutionScope::new(
        Arc::new(std::sync::atomic::AtomicBool::new(false)),
        Some(std::time::Instant::now() + std::time::Duration::from_secs(30)),
    );
    for preconditioner in [Preconditioner::Jacobi, Preconditioner::BlockFactor] {
        let profile = SolverProfile {
            intent: SolveIntent::Root,
            selection: SolverSelection::Explicit(Backend::Kinsol),
            backend: BackendSettings::Kinsol(Method {
                linear: Linear::Spgmr {
                    dimension: pse_model::scalars::PositiveCount::try_new(3).unwrap(),
                },
                preconditioner,
                ..Default::default()
            }),
            ..Default::default()
        };
        let original = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                DerivativeOrder::Value,
                fixture::compiler_profile(),
                profile.clone(),
                NumericalInputs::default(),
                &driver,
            )
            .await
            .unwrap();
        let base = original.model.case.clone();
        assert_eq!(base.compiled().plan.order(), DerivativeOrder::Value);
        assert!(
            !base.compiled().plan.has_directional_actions(),
            "the unchanged original compiler product has no demanded actions"
        );
        let value_programs =
            MathService::within_task(&scope, &driver, service.assemble(base.clone()))
                .await
                .unwrap();
        let conditional = service
            .prepare_conditional(
                base.clone(),
                value_programs,
                original.model.values.clone(),
                BTreeMap::new(),
                profile,
                original.solve.numerics.clone(),
                Route::Native(Backend::Kinsol),
                original.solve.snapshot.clone(),
                &scope,
                &driver,
            )
            .await
            .unwrap()
            .within_task(scope.clone())
            .unwrap();
        let actual_scope = conditional.task_scope().unwrap();
        assert!(Arc::ptr_eq(
            actual_scope.cancellation(),
            scope.cancellation()
        ));
        assert_eq!(actual_scope.deadline(), scope.deadline());
        for solve in [&original.solve, &conditional] {
            let case = ready(solve);
            let plan = &case.prepared.compiled().plan;
            let executable = case.case.as_ref().unwrap();
            assert_eq!(plan.order(), DerivativeOrder::First);
            assert_eq!(
                case.prepared.compiled().facts.prepared_derivatives,
                DerivativeOrder::First
            );
            assert!(
                plan.has_directional_actions() && executable.assembly.has_directional_actions()
            );
            assert!(
                executable
                    ._artifacts
                    .iter()
                    .any(|artifact| artifact.program.is_directional())
            );
            assert!(
                executable
                    ._artifacts
                    .iter()
                    .any(|artifact| !artifact.program.is_directional()
                        && artifact.program.compiled_order() == DerivativeOrder::First)
            );
            assert!(
                executable
                    ._artifacts
                    .iter()
                    .all(|artifact| artifact.program.compiled_order() <= DerivativeOrder::First),
                "no second-order product was fabricated"
            );
            let budget = WorkerBudget::drawing(service.policy.worker_bytes, &service.pool);
            {
                let ExecutionWorker {
                    mut worker,
                    _case,
                    _charge,
                } = service
                    .case_worker(case.case.clone(), case.providers.clone(), &scope, &budget)
                    .unwrap();
                let direction = [0.75, -0.5];
                let mut action = [f64::NAN; 2];
                worker
                    .jacobian_product(&case.values, &direction, &mut action)
                    .unwrap();
                let jacobian = worker.jacobian(&case.values).unwrap();
                let mut reference = [0.0; 2];
                let mut coupling = 0;
                for (col, component) in direction.iter().enumerate() {
                    for entry in
                        jacobian.symbolic().col_ptr()[col]..jacobian.symbolic().col_ptr()[col + 1]
                    {
                        let row = jacobian.row_idx()[entry];
                        reference[row] += jacobian.val()[entry] * component;
                        if row != col && jacobian.val()[entry] != 0.0 {
                            coupling += 1;
                        }
                    }
                }
                assert!(
                    coupling > 0,
                    "the action includes actual off-diagonal coupling"
                );
                for (actual, expected) in action.into_iter().zip(reference) {
                    assert!((actual - expected).abs() < 1e-12);
                }
            }
            assert_eq!(budget.used(), 0);
        }
        for solve in [
            original.solve.within_task(scope.clone()).unwrap(),
            conditional,
        ] {
            let result = service.solve(solve).unwrap().finish().await.unwrap();
            let Outcome::Native(report) = &result.outcome else {
                panic!("expected real compiled KINSOL execution")
            };
            assert_eq!(
                report.termination.category,
                Termination::Success,
                "{report:?}"
            );
            assert!(report.quality.as_ref().unwrap().feasible());
            for name in [
                "callback.jvp.calls",
                "callback.preconditioner.calls",
                "KINGetNumJtimesEvals",
                "KINGetNumPrecEvals",
            ] {
                assert!(
                    matches!(report.metrics.get(name),Some(Metric::Integer(count)) if *count>0),
                    "{name}: {report:?}"
                );
            }
        }
        assert_eq!(base.compiled().plan.order(), DerivativeOrder::Value);
        assert!(!base.compiled().plan.has_directional_actions());
    }
}

#[cfg(any(feature = "solver-ipopt", feature = "solver-pounce"))]
#[tokio::test]
async fn auto_boxed_square_upgrades_first_to_ready_nlp_second() {
    let prepared = prepare(
        "package p { def Root { var x: Scalar; annotation bounds x(0.25, 2); annotation start x(1.5); eq square: x*x == 1; } }",
        SolveIntent::Root, SolverSelection::Auto,
    ).await;
    let original = &prepared.model.case.compiled().plan;
    assert_eq!(original.order(), DerivativeOrder::First);
    assert_eq!(original.available_order(), DerivativeOrder::Second);
    assert!(
        original
            .supports()
            .iter()
            .all(|support| support.order() == DerivativeOrder::First)
    );
    let case = ready(&prepared.solve);
    let Route::Native(backend) = prepared.solve.route() else {
        panic!("expected a native route");
    };
    assert_eq!(
        execution::adapter(backend).representation(),
        execution::Representation::Nlp
    );
    assert_eq!(prepared.solve.required_order(), DerivativeOrder::Second);
    let upgraded = &case.prepared.prepared.plan;
    assert_eq!(upgraded.columns(), original.columns());
    let bounds = |plan: &pse_math::assembly::CasePlan| {
        plan.structure()
            .variables()
            .iter()
            .map(|variable| (variable.port.id, variable.lower, variable.upper))
            .collect::<Vec<_>>()
    };
    assert_eq!(bounds(upgraded), bounds(original));
    assert_eq!(bounds(upgraded)[0].1, Some(0.25));
    assert_eq!(bounds(upgraded)[0].2, Some(2.0));
    assert!(
        upgraded
            .supports()
            .iter()
            .all(|support| support.order() == DerivativeOrder::Second)
    );
    assert_eq!(
        original.order(),
        DerivativeOrder::First,
        "the weaker product remains immutable"
    );
    assert!(
        matches!(
            case.prepared.prepared.presolve.class_status,
            pse_math::presolve::ClassStatus::Unassessed
        ),
        "Root readiness does not request coefficient objective proof"
    );
}

const LINEAR: &str = "package p { def Root { var x: Scalar; annotation bounds x(0, 2); annotation start x(1); let f: Scalar = x; annotation objective f(minimize); eq floor: x >= 0.5; } }";

#[cfg(feature = "solver-ipopt")]
#[tokio::test]
async fn explicit_ipopt_auto_demands_independent_affine_rows_and_off_preserves_base() {
    let (package, root, runtime) = package_and_runtime(
        "package p { def Root { var x:Scalar; var y:Scalar; eq linear:x+y==2; eq nonlinear:log(x)+y==1; annotation bounds x(-2,2); annotation valid x(0.25,2); annotation start x(1); annotation start y(1); } }",
    ).await;
    let service = runtime.native();
    let flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    for policy in [
        native::presolve::Policy::Off,
        native::presolve::Policy::Auto,
    ] {
        let requested = !matches!(policy, native::presolve::Policy::Off);
        let prepared = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                DerivativeOrder::First,
                fixture::compiler_profile(),
                SolverProfile {
                    intent: SolveIntent::Root,
                    selection: SolverSelection::Explicit(Backend::Ipopt),
                    presolve: policy.clone(),
                    ..Default::default()
                },
                NumericalInputs::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        assert_eq!(prepared.solve.route(), Route::Native(Backend::Ipopt));
        let base = prepared.model.case.compiled();
        assert!(base.presolve.affine.iter().all(Option::is_none));
        let case = ready(&prepared.solve);
        let facts = &case.prepared.compiled().presolve;
        assert_eq!(
            facts.class_status,
            pse_math::presolve::ClassStatus::Unassessed
        );
        assert!(case.prepared.compiled().coefficients.is_none());
        assert_eq!(facts.obligations, base.presolve.obligations);
        assert_eq!(
            facts.affine.iter().filter(|row| row.is_some()).count(),
            usize::from(requested)
        );
        assert_eq!(
            facts.proof_remaining < base.presolve.proof_remaining,
            requested
        );
        assert_eq!(facts.key != base.presolve.key, requested);
        let scope = pse_kernels::ExecutionScope::new(flag.clone(), None);
        let budget = WorkerBudget::drawing(service.policy.worker_bytes, &service.pool);
        let ExecutionWorker {
            worker,
            _case,
            _charge,
        } = service
            .case_worker(case.case.clone(), case.providers.clone(), &scope, &budget)
            .unwrap();
        let oracle = native::assembled::AlgebraicOracle::new(worker, case.values.clone())
            .unwrap()
            .with_presolve_facts(facts.clone())
            .unwrap()
            .with_normalization(prepared.solve.normalization.clone())
            .unwrap();
        let initial = case
            .prepared
            .compiled()
            .plan
            .columns()
            .iter()
            .map(|id| case.values.scalars[id])
            .collect::<Vec<_>>();
        let execution = Execution::within(flag.clone(), prepared.solve.controls(), scope).unwrap();
        let pipeline = native::presolve::Pipeline::new(
            Box::new(oracle),
            &initial,
            &policy,
            prepared.solve.tolerances(),
            prepared.solve.accuracy(),
            execution,
            None,
            prepared.solve.compatibility().unwrap().clone(),
            case.prepared.compiled().plan.limits().native_index,
        )
        .unwrap();
        assert_eq!(
            pipeline.report().passes[&native::presolve::Pass::AffineElimination].applied,
            requested
        );
        assert!(facts.has_guards);
        assert!(base.presolve.affine.iter().all(Option::is_none));
    }
}

#[cfg(feature = "solver-highs")]
#[tokio::test]
async fn auto_linear_retains_established_coefficient_readiness() {
    let prepared = prepare(LINEAR, SolveIntent::Optimize, SolverSelection::Auto).await;
    let case = ready(&prepared.solve);
    assert_eq!(prepared.solve.route(), Route::Native(Backend::Highs));
    assert_eq!(
        case.prepared.prepared.presolve.class_status,
        pse_math::presolve::ClassStatus::Established
    );
    let coefficients = case.prepared.prepared.coefficients.as_ref().unwrap();
    assert_eq!(
        coefficients.objective.len(),
        case.prepared.prepared.plan.columns().len()
    );
    assert_eq!(prepared.solve.required_order(), DerivativeOrder::First);
}

#[tokio::test]
async fn explicit_coefficient_cone_retains_representation_and_paired_proof() {
    let prepared = prepare(
        LINEAR,
        SolveIntent::Optimize,
        SolverSelection::Explicit(Backend::Clarabel),
    )
    .await;
    let case = ready(&prepared.solve);
    assert_eq!(prepared.solve.route(), Route::Native(Backend::Clarabel));
    assert_eq!(
        case.prepared.prepared.presolve.class_status,
        pse_math::presolve::ClassStatus::Established
    );
    assert!(case.prepared.prepared.coefficients.is_some());
    let (lowered, owner) = case.coefficient_cone.as_ref().unwrap();
    assert!(owner.size() > 0);
    assert_eq!(
        lowered.problem.contract.variables.len(),
        case.prepared.prepared.plan.columns().len()
    );
    assert!(lowered.problem.validate(&lowered.evidence).is_ok());
}

#[tokio::test]
async fn auto_recognized_cone_retains_representation_and_paired_proof() {
    let prepared = prepare(
        "package p { def Root { var x: Scalar; annotation bounds x(-1, 2); annotation start x(0.5); let f: Scalar = exp(x); annotation objective f(minimize); eq floor: x >= 0; } }",
        SolveIntent::Optimize, SolverSelection::Auto,
    ).await;
    let case = ready(&prepared.solve);
    assert_eq!(prepared.solve.route(), Route::Native(Backend::Clarabel));
    assert!(case.prepared.prepared.facts.convexity.cone());
    assert!(case.prepared.prepared.coefficients.is_none());
    assert!(case.coefficient_cone.is_none());
    let (recognized, proof, owner) = case.recognized.as_ref().unwrap();
    assert!(owner.size() > 0);
    assert!(recognized.problem.validate(proof.as_ref()).is_ok());
    assert_eq!(prepared.solve.required_order(), DerivativeOrder::First);
}
