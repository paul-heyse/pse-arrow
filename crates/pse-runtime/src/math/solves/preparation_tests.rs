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
                worker_bytes: 16usize << 30,
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
    let (package, root, runtime) = package_with_runtime(
        LINEAR,
        fixture::runtime_on(
            512 << 20,
            crate::math::MathPolicy {
                worker_bytes: 16usize << 30,
                workspace_bytes: 16 << 20,
                foreign_bytes: 1 << 20,
                ..Default::default()
            },
        ),
    )
    .await;
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
                intent: SolveIntent::Optimize,
                selection: SolverSelection::Explicit(Backend::Clarabel),
                ..Default::default()
            },
            NumericalInputs::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
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
    let coefficients = case
        .prepared
        .prepared
        .coefficients
        .as_ref()
        .unwrap()
        .clone();
    let allowance = runtime.shared.math().policy.worker_bytes;
    let demand = coefficients
        .numerical_convexity_allocation_bound(allowance)
        .unwrap();
    assert!(demand < 1 << 20);
    let scales = vec![1.0; coefficients.hessian.ncols()];
    let evidence = runtime
        .shared
        .math()
        .job(1, demand, FlightCancellation::default(), move |flag| {
            coefficients
                .numerical_convexity(1.0, &scales, 1.0, 1e-12, 1e-12, allowance, &flag)
                .map_err(Into::into)
        })
        .await
        .unwrap();
    assert!(evidence.accepted());
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

#[cfg(feature = "solver-highs")]
#[tokio::test]
async fn actual_convex_advice_inputs_are_nonrecursive_and_refuse_unestablished_branch_equivalence() {
    use pse_model::strategy::advice::{AdviceMetric, AdviceObservation, AdviceOutcome, FrozenAdvice};
    let mut prepared=prepare(LINEAR,SolveIntent::Optimize,SolverSelection::Auto).await;
    let mut limits=prepared.solve.numerical_strategy().limits;
    limits.attempts=4;
    prepared.solve.profile.composition.limits=Some(limits);
    let cancel=Arc::new(std::sync::atomic::AtomicBool::new(false));
    let baseline=prepared.solve.advice_inputs(&cancel).unwrap().unwrap();
    assert!(!baseline.candidates.is_empty());
    let before=prepared.solve.request_identity().unwrap();
    let observations = baseline.candidates.iter().enumerate().map(|(index, candidate)| AdviceObservation {
        cohort: baseline.cohort.clone(), matched: pse_ids::ContentHash::from_bytes([40; 32]),
        candidate: *candidate, attempt: *candidate, outcome: AdviceOutcome::Complete,
        value: Some((baseline.candidates.len() - index) as f64),
    }).collect();
    let advice=FrozenAdvice::freeze(baseline.cohort.clone(),AdviceMetric::NativeExecutionNanoseconds,
        baseline.candidates.clone(),observations,baseline.limits).unwrap();
    prepared.solve.profile.composition.advice=Some(advice);
    assert_eq!(prepared.solve.advice_inputs(&cancel).unwrap().unwrap(),baseline);
    assert_ne!(prepared.solve.request_identity().unwrap(),before);
    let entry = prepared.solve.automatic_entry_operations(pse_model::strategy::StartOrigin::Specification, false, &cancel).unwrap();
    assert_eq!(entry.iter().map(|operation| operation.candidate.identity).collect::<Vec<_>>(), baseline.candidates);
    let candidates = entry.iter().map(|operation| operation.candidate.clone()).collect::<Vec<_>>();
    let decision = super::super::strategy::next_automatic(&prepared.solve.profile.composition,
        &prepared.solve.numerical_strategy().start, &candidates, &Default::default(), None,
        pse_model::strategy::WorkObservation { attempts: 0, evaluations: None, iterations: None, factorizations: None, proof_steps: None },
        false, Some(&baseline.cohort));
    assert!(matches!(decision, super::super::strategy::AutoDecision::Prepare { candidate } | super::super::strategy::AutoDecision::Dispatch { candidate }
        if candidate == candidates.len() - 1), "actual entry inventory must permit advice before baseline success");
    let inherited = prepared.solve.automatic_entry_operations(pse_model::strategy::StartOrigin::Accepted, true, &cancel).unwrap();
    assert_eq!(inherited.len(), prepared.solve.automatic_operations(None, &cancel).unwrap().len());
    assert!(prepared.solve.advice_cohort(pse_model::strategy::StartOrigin::Accepted,true).unwrap().is_none());
    prepared.solve.profile.selection=SolverSelection::Explicit(Backend::Highs);
    assert!(prepared.solve.advice_inputs(&cancel).unwrap().is_none());
    prepared.solve.profile.selection=SolverSelection::Auto;
    prepared.solve.profile.intent=SolveIntent::Root;
    assert!(prepared.solve.advice_inputs(&cancel).unwrap().is_none());
}

#[cfg(feature = "solver-highs")]
#[tokio::test]
async fn actual_convex_advice_selects_equivalent_original_profile_before_first_success() {
    use pse_model::strategy::advice::{AdviceMetric, AdviceObservation, AdviceOutcome, FrozenAdvice};
    use native::settings::highs::{Method, Settings};
    let (package, root, _) = package_and_runtime(LINEAR).await;
    async fn bound(package: &ModelingPackage, root: DeclarationId, profile: SolverProfile) -> ModelingSolvePreparation {
        package.prepare_solve(root, pse_modeling::specialize::root_instance(root), Bindings::default(),
            Limits::default(), ModelingCaseBindings::default(), DerivativeOrder::First,
            fixture::compiler_profile(), profile, NumericalInputs::default(), &crate::CancelSource::new()).await.unwrap()
    }
    let mut profile = SolverProfile { intent: SolveIntent::Optimize, ..Default::default() };
    profile.composition.recovery = vec![pse_model::strategy::StartOrigin::Specification];
    profile.composition.limits = Some(pse_model::strategy::WorkLimits {
        attempts:4,evaluations:None,iterations:None,factorizations:None,proof_steps:None,
    });
    let mut prepared = bound(&package, root, profile.clone()).await;
    let mut leaves = Vec::new();
    for method in [Method::Simplex,Method::Ipm] {
        let mut leaf = profile.clone();
        leaf.selection=SolverSelection::Explicit(Backend::Highs);
        leaf.backend=BackendSettings::Highs(Settings {method,..Default::default()});
        leaves.push(bound(&package,root,leaf).await.solve);
    }
    prepared.solve=prepared.solve.with_automatic_products(leaves.into_iter().map(PreparedRung::from).collect()).unwrap();
    let flag=Arc::new(std::sync::atomic::AtomicBool::new(false));
    let inputs=prepared.solve.advice_inputs(&flag).unwrap().unwrap();
    let inventory=prepared.solve.advice_comparison_operations(&flag).unwrap();
    let advised=inventory.iter().find(|operation| matches!(&operation.binding,
        AutomaticBinding::Prepared(rung) if matches!(rung.as_ref(),PreparedRung::Original(leaf)
            if matches!(&leaf.profile.backend,BackendSettings::Highs(settings) if settings.method==Method::Ipm))))
        .unwrap().candidate.identity;
    assert_ne!(advised,inputs.candidates[0]);
    let observations=inputs.candidates.iter().map(|candidate|AdviceObservation {
        cohort:inputs.cohort.clone(),matched:pse_ids::ContentHash::from_bytes([41;32]),
        candidate:*candidate,attempt:*candidate,outcome:AdviceOutcome::Complete,
        value:Some(if *candidate==advised {1.0} else {10.0}),
    }).collect();
    prepared.solve.profile.composition.advice=Some(FrozenAdvice::freeze(inputs.cohort.clone(),
        AdviceMetric::NativeExecutionNanoseconds,inputs.candidates.clone(),observations,inputs.limits).unwrap());
    assert_eq!(prepared.solve.advice_inputs(&flag).unwrap().unwrap(),inputs,
        "comparison identity excludes advice while preserving the actual ordered profile inventory");
    let scope=pse_kernels::ExecutionScope::new(Arc::default(),
        Some(std::time::Instant::now()+std::time::Duration::from_secs(30)));
    let admitted=prepared.solve.clone().within_task(scope).unwrap();
    assert_eq!(admitted.advice_inputs(&flag).unwrap().unwrap(),inputs,
        "binding the original task scope cannot disable otherwise applicable advice");
    let effective_request=prepared.solve.request_identity().unwrap();
    let joined=prepared.start().unwrap().wait().await.unwrap();
    assert!(joined.usable(),"the advised original profile must pass independent original scientific acceptance");
    let crate::workflow::RunReport::Modeling(results)=joined.report().unwrap() else {panic!("modeling report");};
    let Outcome::Native(native)=&results[0].outcome else {panic!("native report");};
    assert_eq!(native.backend,Backend::Highs);
    assert_eq!(native.options["solver"],OptionValue::Text("ipm".into()));
    let trace=results[0].strategy.as_ref().unwrap();
    assert_eq!(trace.publication_request,Some(effective_request));
    assert_eq!(trace.events.iter().filter(|event|event.kind==pse_model::generated::enums::NumericalEventKind::Started
        && event.phase==pse_model::strategy::Phase::Native).count(),1,
        "the advised equivalent profile executes before the successful baseline profile");

    let mut tied=prepared.solve.clone();
    tied.profile.composition.advice=Some(FrozenAdvice::freeze(inputs.cohort.clone(),
        AdviceMetric::NativeExecutionNanoseconds,inputs.candidates.clone(),vec![],inputs.limits).unwrap());
    assert_eq!(tied.automatic_entry_operations(pse_model::strategy::StartOrigin::Specification,false,&flag).unwrap().len(),
        tied.automatic_operations(None,&flag).unwrap().len());
    let mut changed=tied.clone();
    let PreparedRung::Original(leaf)=&mut changed.automatic_products[0] else {panic!("Original profile");};
    leaf.tolerances.rows[0]*=2.0;
    assert!(changed.advice_inputs(&flag).unwrap().is_none(),"changed physical acceptance refuses equivalence");
    let mut pending=tied.clone();
    let PreparedRung::Original(leaf)=&mut pending.automatic_products[0] else {panic!("Original profile");};
    leaf.route_decision.as_mut().unwrap().state=routing::AssessmentState::PendingEvidence;
    assert!(pending.advice_inputs(&flag).unwrap().is_none(),"pending scientific evidence preserves the baseline barrier");
    let mut reordered=tied.clone();reordered.automatic_products.swap(0,1);
    let reordered_inputs=reordered.advice_inputs(&flag).unwrap().unwrap();
    assert_ne!(reordered_inputs.cohort.profiles,inputs.cohort.profiles,
        "the cohort binds actual profile order, settings and local allowances");
    assert!(!tied.profile.composition.advice.as_ref().unwrap().applies(&reordered_inputs.cohort,
        &reordered_inputs.candidates,reordered_inputs.limits));
}

#[cfg(feature = "solver-pounce")]
#[tokio::test]
async fn actual_pounce_observed_recovery_precedes_unobserved_profiles() {
    let prepared=prepare("package p { def Root { var x:Scalar; annotation bounds x(-1,2); annotation start x(0.5); let f:Scalar=exp(x); annotation objective f(minimize); eq floor:x>=0; } }",
        SolveIntent::Optimize,SolverSelection::Explicit(Backend::Pounce)).await;
    let flag=Arc::new(std::sync::atomic::AtomicBool::new(false));
    let inventory=prepared.solve.automatic_operations(None,&flag).unwrap();
    let Representation::Algebraic(source)=&prepared.solve.representation else {panic!("algebraic original");};
    let execution=Execution::new(flag.clone(),&prepared.solve.profile.controls);
    let contract=native::assembled::contract(&source.prepared.prepared.plan);
    let mut observed=SolveReport::new(Backend::Pounce,&contract,NativeTermination {code:-1,
        name:"MaximumIterationsExceeded".into(),message:None,category:Termination::Limit,assurance:Assurance::None},&execution);
    assert!(prepared.solve.automatic_operations(Some(&observed),&flag).unwrap().iter().all(|operation|operation.candidate.observed.is_none()),
        "missing native quality escalation evidence cannot authorize recovery");
    observed.pounce_statistics=Some(Box::default());
    assert!(prepared.solve.automatic_operations(Some(&observed),&flag).unwrap().iter().all(|operation|operation.candidate.observed.is_none()));
    observed.pounce_statistics.as_mut().unwrap().quality_escalations=1;
    let recovered=prepared.solve.automatic_operations(Some(&observed),&flag).unwrap();
    assert!(recovered.iter().any(|operation|operation.candidate.observed.is_some()));
    let mut candidates=inventory.iter().map(|operation|operation.candidate.clone()).collect::<Vec<_>>();
    let mut known=candidates.iter().map(|candidate|candidate.identity).collect::<std::collections::BTreeSet<_>>();
    candidates.extend(recovered.into_iter().map(|operation|operation.candidate).filter(|candidate|known.insert(candidate.identity)));
    let last=super::super::strategy::AutoObservation {awaiting_assessment:false,
        native:pse_model::generated::enums::NumericalAttemptObservation::Limited,original:None,permission:None};
    let decision=super::super::strategy::next_automatic(&prepared.solve.profile.composition,
        &prepared.solve.numerical_strategy().start,&candidates,&std::collections::BTreeSet::from([0]),Some(&last),
        pse_model::strategy::WorkObservation {attempts:1,evaluations:None,iterations:None,factorizations:None,proof_steps:None},false,None);
    assert!(matches!(decision,super::super::strategy::AutoDecision::Prepare {candidate}
        if candidates[candidate].observed.is_some()),"actual observation-dependent recovery precedes ordinary alternatives");
}

#[cfg(feature = "solver-highs")]
#[tokio::test]
async fn fresh_replay_recursively_detaches_exhausted_original_graph_and_preserves_requested_start() {
    use pse_model::strategy::{NumericalStrategy, Phase, Scope, WorkCharge, WorkObservation};
    let prepared = prepare(LINEAR, SolveIntent::Optimize, SolverSelection::Auto).await;
    let mut original = prepared.solve;
    original.profile.controls.start = StartPolicy::Explicit;
    let seed = WarmStart {
        origin: None,
        compatibility: original.compatibility.clone().unwrap(),
        payload: WarmPayload::Highs { primal: Some(vec![1.0]), dual: None, basis: None },
    };
    let seed_identity = seed.content_key();
    original = original.with_start(seed).unwrap();
    let mut limits = original.numerical_strategy().limits;
    limits.attempts = 1;
    let flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let scope = pse_kernels::ExecutionScope::new(flag.clone(),
        Some(std::time::Instant::now() + std::time::Duration::from_secs(30)));
    let admission = super::super::strategy::admission::TaskAdmission::new(limits,
        scope.clone(), Some(original.pool.clone()), false);
    admission.charge(WorkCharge { phase:Phase::Native, scope:Scope::Task,
        charging_owner:original.original_identity().unwrap(),
        observed:WorkObservation {attempts:1,evaluations:Some(0),iterations:Some(0),factorizations:Some(0),proof_steps:Some(0)} }).unwrap();
    assert!(admission.reserve_attempt().is_err());
    original.task_scope = Some(scope.clone());
    original.task_admission = Some(admission.clone());
    let leaf = original.clone();
    original.composition = Some(Arc::new(PreparedComposition {
        declaration:NumericalStrategy::direct(StartPolicy::Explicit,limits),
        rungs:vec![PreparedRung::Original(Box::new(leaf.clone()))],
        _owner:original._owner.clone(),
    }));
    original.automatic_products = vec![PreparedRung::Original(Box::new(leaf))];
    flag.store(true,std::sync::atomic::Ordering::Release);
    assert!(original.task_scope().unwrap().check().is_err());
    let request = original.request_identity().unwrap();
    let replay = original.fresh_replay_copy().unwrap();
    assert_eq!(replay.request_identity().unwrap(),request);
    assert_eq!(replay.requested_start().unwrap().content_key(),seed_identity);
    assert!(Arc::ptr_eq(&replay._owner,&original._owner));
    assert_eq!(replay.replay_declaration(),original.replay_declaration());
    for copied in std::iter::once(&replay)
        .chain(replay.composition.as_ref().unwrap().rungs.iter().map(|rung| {
            let PreparedRung::Original(original)=rung else {panic!("original-only graph");};
            original.as_ref()
        }))
        .chain(replay.automatic_products.iter().map(|rung| {
            let PreparedRung::Original(original)=rung else {panic!("original-only graph");};
            original.as_ref()
        })) {
        assert!(copied.task_scope.is_none());
        assert!(copied.task_admission.is_none());
        assert!(copied.point_accuracy.is_none());
        assert_eq!(copied.requested_start().unwrap().content_key(),seed_identity);
    }
    assert_eq!(admission.observation().unwrap().attempts,1);
    assert!(original.task_scope().unwrap().check().is_err());
    let new_scope=pse_kernels::ExecutionScope::new(Arc::default(),
        Some(std::time::Instant::now()+std::time::Duration::from_secs(30)));
    assert!(replay.within_task(new_scope).is_ok());
}
