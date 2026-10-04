// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
#[cfg(feature = "solver-kinsol")]
fn native_report(outcome: &Outcome) -> &SolveReport {
    match outcome {
        Outcome::Native(report) => report,
        _ => panic!("actual native attempt required"),
    }
}

use pse_ids::SemanticId;

#[cfg(all(feature = "solver-kinsol", feature = "solver-root-isolation"))]
#[tokio::test]
async fn compiled_ill_conditioned_nested_relation_chain_refines_against_tighter_native_reference() {
    use crate::workflow::tests as fixture;
    use pse_compiler::workspace::{ImplicitCapabilities, SelectionNeighborhood};
    use pse_kernels::{EvaluationContext, ProviderFactory, ProviderRequest};
    use pse_math::{
        derived::ReconstructionOracle,
        implicit::{Configuration, ImplicitFactory, Options, Unknown},
        normalization::Normalization,
    };
    use pse_model::strategy::{AccuracyClass, AccuracyDemand};
    use std::{collections::BTreeMap, time::Duration};

    // A composed nested relation, not opaque nested provider callbacks. Both equations
    // are compiler-admitted together, so the verifier sees the complete actual chain.
    let runtime = fixture::runtime_on(
        24usize << 30,
        crate::math::MathPolicy {
            worker_bytes: 8usize << 30,
            workspace_bytes: 8usize << 30,
            foreign_bytes: 16usize << 20,
            ..Default::default()
        },
    );
    let physical = fixture::physical();
    let rows = pse_authoring::language::parse(
        "package p { def Root { var p:Scalar; implicit chain select minimum(0,0) { var x:Scalar; var z:Scalar; annotation bounds x(0.0000999,0.0001001); annotation bounds z(0.00999,0.01001); regime positive eligible(x>0 and z>0) { eq first:x*x==p; eq second:z*z==x; } } realize r on chain using nested; annotation report chain.z(\"nested-root\"); } }",
        SemanticId::NIL,
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    ).unwrap();
    let declaration = |name| rows.iter().find(|r| r.name == name).unwrap().declaration_id;
    let root = declaration("Root");
    let x_decl = declaration("x");
    let z_decl = declaration("z");
    let package = runtime
        .modeling_package(rows.clone(), physical.clone())
        .unwrap();
    let cancel = crate::CancelSource::new();
    let prepared = package
        .prepare(
            root,
            pse_modeling::specialize::root_instance(root),
            pse_modeling::Bindings::default(),
            pse_modeling::Limits::default(),
            &cancel,
        )
        .await
        .unwrap();
    let product = prepared.compiled();
    let inner = product.admitted.implicit_systems().next().unwrap();
    assert_eq!(inner.unknowns.len(), 2);
    assert_eq!(inner.residuals.len(), 1);
    assert_eq!(
        inner.selection.neighborhood_evidence,
        SelectionNeighborhood::RuntimeIsolation
    );
    let residual = &inner.residuals[0];
    assert_eq!(residual.rows.len(), 2);
    assert_eq!(residual.body.math().input_count(), 3);
    let x = inner
        .unknowns
        .iter()
        .position(|id| product.model.symbols[id].lineage.declaration == x_decl)
        .unwrap();
    let z = inner
        .unknowns
        .iter()
        .position(|id| product.model.symbols[id].lineage.declaration == z_decl)
        .unwrap();
    assert_ne!(x, z);
    let unknowns = inner
        .unknowns
        .iter()
        .enumerate()
        .map(|(i, &id)| {
            let (lower, upper) = if i == x {
                (0.0000999, 0.0001001)
            } else {
                (0.00999, 0.01001)
            };
            Unknown { id, lower, upper }
        })
        .collect::<Vec<_>>();
    let options = |tight| Options {
        start: (0..2)
            .map(|i| {
                let center = if i == x { 1e-4 } else { 1e-2 };
                center * if tight { 1.00001 } else { 1.000001 }
            })
            .collect(),
        variable_nominals: vec![1.0; 2],
        variable_tolerance: vec![if tight { 1e-14 } else { 1e-8 }; 2],
        residual_tolerance: vec![if tight { 1e-19 } else { 1e-8 }; 2],
        derivative_tolerance: if tight { 1e-13 } else { 1e-8 },
        iterations: 100,
        time_limit: Duration::from_secs(10),
    };
    let flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let scope = ExecutionScope::new(flag.clone(), Some(Instant::now() + Duration::from_secs(60)));
    let accelerators = pse_math::implicit::accelerators::Accelerators::standard();
    let make_factory = |tight| {
        let factory = inner
            .factory(
                BTreeMap::from([(
                    residual.id,
                    Configuration::Fixed(unknowns.clone(), options(tight)),
                )]),
                ImplicitCapabilities {
                    solver: Arc::new(native::implicit::Kinsol),
                    verifier: Some(Arc::new(native::root_isolation::Ibex)),
                    accelerators: &accelerators,
                },
                DerivativeOrder::First,
                flag.clone(),
                pse_math::jets::EvaluationLimits::default(),
            )
            .unwrap();
        let ImplicitFactory::Regimes(factory) = factory else {
            panic!("compiler-issued isolated selector")
        };
        factory
    };
    let factory = make_factory(false);
    let mut reference_selector = make_factory(true);
    let reference = reference_selector.alternatives.remove(0).residual;
    let numerical = &factory.alternatives[0].residual;
    assert!(factory.verifier.is_some());
    assert_eq!(factory.alternatives.len(), 1);
    assert_eq!(
        factory.alternatives[0]
            .isolation
            .as_ref()
            .unwrap()
            .residuals
            .len(),
        2
    );
    assert_eq!(numerical.rows, residual.rows);
    assert_eq!(
        numerical
            .spec
            .outputs
            .iter()
            .map(|p| p.id)
            .collect::<Vec<_>>(),
        inner.unknowns
    );
    assert_eq!(numerical.spec.inputs.len(), 1);
    assert_eq!(numerical.body.coordinates(), [0, 1, 2]);
    assert_eq!(numerical.body.prepared_support().outputs(), [0, 1]);
    assert_eq!(numerical.body.compiled_order(), DerivativeOrder::First);
    assert_eq!(numerical.spec, reference.spec);
    assert_ne!(numerical.configuration_key(), reference.configuration_key());

    let spec = factory.spec.clone();
    let structural = numerical.body.incidence(&flag).unwrap();
    let incidence = structural
        .outputs()
        .iter()
        .enumerate()
        .flat_map(|(i, &output)| {
            structural
                .first_for_output(output)
                .unwrap()
                .iter()
                .map(move |&j| Entry::new(GlobalRow::new(i), GlobalCol::new(j)))
        })
        .collect::<Vec<_>>();
    let service = runtime.native();
    let _owner = service
        .reserve(
            "test:compiled-nested-chain",
            factory.retained_bytes().unwrap(),
        )
        .unwrap();
    let source = SelectedImplicitReconstruction::<pse_math::MathError>::source(&factory);
    let normalization = Normalization::identity(3, 2);
    let original = Arc::new(
        OriginalContract::new(
            spec.identity(),
            normalization.key(),
            unknowns
                .iter()
                .map(|u| math::Coordinate {
                    id: u.id,
                    lower: u.lower,
                    upper: u.upper,
                })
                .chain(spec.inputs.iter().map(|p| math::Coordinate {
                    id: p.id,
                    lower: 0.999e-8,
                    upper: 1.001e-8,
                }))
                .collect(),
            residual
                .rows
                .iter()
                .map(|&id| math::Constraint {
                    id,
                    lower: 0.0,
                    upper: 0.0,
                })
                .collect(),
            incidence,
            DerivativeSupport {
                order: DerivativeOrder::First,
                jacobian_product: true,
                source,
            },
            OriginalObligations {
                guards: source,
                selection: source,
                objective: None,
            },
        )
        .unwrap(),
    );
    let contract = SelectedImplicitReconstruction::<pse_math::MathError>::prepare_contract(
        &factory,
        original,
        vec![GlobalCol::new(2)],
        vec![GlobalRow::new(0), GlobalRow::new(1)],
        source,
        &flag,
    )
    .unwrap();
    let mut reconstruction = SelectedImplicitReconstruction::<pse_math::MathError>::new(
        &factory,
        contract,
        normalization.clone(),
        scope.clone(),
    )
    .unwrap();
    let input = [1e-8];
    ReconstructionOracle::admit(&mut reconstruction, &input).unwrap();
    let chart = reconstruction.selection().selected_chart().unwrap().clone();
    let product_key = |action| {
        let mut key = FramedHasher::new(pse_ids::Frame::AccuracyProductV1);
        key.str(if action {
            "compiled-chain-first-action"
        } else {
            "compiled-chain-point"
        })
        .hash(&source)
        .hash(&normalization.key())
        .f64(input[0]);
        if action {
            key.f64(1.0);
        }
        key.finish_hash()
    };
    let limits = math::RefinementLimits {
        rounds: 8,
        proof_cells: 64,
    };
    let weak = AccuracyDemand {
        product: product_key(false),
        normalization: normalization.key(),
        allowance: 1e-6,
        class: AccuracyClass::Certified,
    };
    let weak_point =
        ReconstructionOracle::point(&mut reconstruction, &input, &weak, limits).unwrap();
    assert!(weak_point.accuracy.satisfies(&weak));
    // The loose numerical proposal really is insufficient for the stronger consumer.
    let strong = AccuracyDemand {
        allowance: 1e-12_f64.min(weak_point.accuracy.error.unwrap() / 8.0),
        ..weak
    };
    assert!(strong.allowance > 0.0);
    assert!(!weak_point.accuracy.satisfies(&strong));
    let before = reconstruction.selection().observed_point_cells();
    let strong_point =
        ReconstructionOracle::point(&mut reconstruction, &input, &strong, limits).unwrap();
    assert!(strong_point.accuracy.satisfies(&strong));
    assert_eq!(strong_point.accuracy.class, AccuracyClass::Certified);
    assert!(reconstruction.selection().observed_point_cells() > before);
    let action = AccuracyDemand {
        product: product_key(true),
        normalization: normalization.key(),
        allowance: 1e-4,
        class: AccuracyClass::Certified,
    };
    let response = ReconstructionOracle::jacobian_product(
        &mut reconstruction,
        &input,
        &[1.0],
        &action,
        limits,
    )
    .unwrap();
    assert!(response.accuracy.satisfies(&action));
    assert_eq!(response.accuracy.class, AccuracyClass::Certified);
    assert!(reconstruction.selection().observed_action_cells() >= 2);

    // A separate native worker solves the same original compiled residual with stricter
    // controls, verifies its residual/bounds, then computes the actual library IFT.
    // This is a numerical reference, not certified accuracy evidence.
    let mut reference_worker = reference.create_scoped(scope.clone()).unwrap();
    let reference_values = reference_worker
        .evaluate(
            &input,
            &ProviderRequest::all(&reference.spec, DerivativeOrder::First),
            &EvaluationContext {
                cancelled: &flag,
                max_result_bytes: 1 << 20,
            },
        )
        .unwrap();
    for i in 0..2 {
        assert!((strong_point.values[i] - reference_values.values[i]).abs() <= strong.allowance);
        assert!((response.values[i] - reference_values.jacobian[i]).abs() <= action.allowance);
    }
    assert_eq!(strong_point.values[2], input[0]);
    assert_eq!(response.values[2], 1.0);
    assert!((reference_values.values[x] - input[0].sqrt()).abs() < 1e-14);
    assert!((reference_values.values[z] - input[0].sqrt().sqrt()).abs() < 1e-14);
    assert!((reference_values.jacobian[x] - 5000.0).abs() < 1e-7);
    assert!((reference_values.jacobian[z] - 250000.0).abs() < 1e-5);
    let tighter_action = AccuracyDemand {
        allowance: 1e-6_f64.min(response.accuracy.error.unwrap() / 8.0),
        ..action
    };
    assert!(tighter_action.allowance > 0.0);
    assert!(!response.accuracy.satisfies(&tighter_action));
    match ReconstructionOracle::jacobian_product(
        &mut reconstruction,
        &input,
        &[1.0],
        &tighter_action,
        limits,
    ) {
        Ok(tighter) => {
            assert!(tighter.accuracy.satisfies(&tighter_action));
            for i in 0..2 {
                assert!(
                    (tighter.values[i] - reference_values.jacobian[i]).abs()
                        <= tighter_action.allowance
                );
            }
        }
        Err(pse_math::MathError::Refinement {
            product,
            source_key,
            validity,
            reason,
        }) => {
            assert_eq!(product, tighter_action.product);
            assert_eq!(source_key, source);
            assert_eq!(validity, source);
            assert!(matches!(
                reason,
                math::RefinementRefusal::Precision
                    | math::RefinementRefusal::Rounds
                    | math::RefinementRefusal::ProofCells
            ));
        }
        Err(error) => panic!(
            "stronger action must be certified or refuse its own finite refinement: {error:?}"
        ),
    }
    let retained = reconstruction.selection().selected_chart().unwrap();
    assert_eq!(retained.parameters, chart.parameters);
    assert_eq!(retained.existence, chart.existence);
    assert_eq!(retained.uniqueness, chart.uniqueness);
    assert_eq!(retained.verifier_identity, chart.verifier_identity);
    assert_eq!(retained.order, chart.order);
    scope.check().unwrap();
}
#[test]
fn terminal_typed_observations_cannot_escape_as_proposals() {
    use pse_model::generated::enums::NumericalAttemptObservation as O;
    for observation in [
        O::ContractFailure,
        O::Panic,
        O::OperationalFailure,
        O::Cancelled,
        O::ResourceExhausted,
        O::CapabilityRefusal,
    ] {
        assert!(!proposal_observation_allowed(observation));
    }
    for observation in [O::Converged, O::Limited, O::NumericalFailure] {
        assert!(proposal_observation_allowed(observation));
    }
}
#[test]
fn proposal_bounds_screen_is_original_and_transactional() {
    let id = SemanticId::from_bytes([1; 16]);
    let hash = pse_ids::ContentHash::from_bytes([1; 32]);
    let original = OriginalContract::new(
        hash,
        hash,
        vec![math::Coordinate {
            id,
            lower: 1.0,
            upper: 4.0,
        }],
        vec![math::Constraint {
            id: SemanticId::from_bytes([2; 16]),
            lower: 3.0,
            upper: 3.0,
        }],
        vec![Entry::new(GlobalRow::new(0), GlobalCol::new(0))],
        DerivativeSupport {
            order: DerivativeOrder::First,
            jacobian_product: true,
            source: hash,
        },
        OriginalObligations {
            guards: hash,
            selection: hash,
            objective: None,
        },
    )
    .unwrap();
    assert!(screen_bounds(&original, &[2.0]).is_ok());
    assert!(screen_bounds(&original, &[0.0]).is_err());
    assert!(screen_bounds(&original, &[f64::NAN]).is_err());
    assert!(screen_bounds(&original, &[]).is_err());
    let numerical = zero_contract(&Arc::new(original.clone())).unwrap();
    assert_eq!(original.constraints()[0].lower, 3.0);
    assert_eq!(numerical.constraints()[0].lower, 0.0);
    assert_ne!(numerical.identity(), original.identity());
}
#[cfg(feature = "solver-kinsol")]
async fn original(text: &str) -> (crate::workflow::Runtime, PreparedSolve) {
    original_order(text, DerivativeOrder::First).await
}
#[cfg(feature = "solver-kinsol")]
async fn original_order(
    text: &str,
    order: DerivativeOrder,
) -> (crate::workflow::Runtime, PreparedSolve) {
    use crate::workflow::tests as fixture;
    let runtime = fixture::runtime_with(128 << 20, 1 << 20, 512 << 20);
    original_order_on(
        runtime,
        text,
        order,
        SolverProfile {
            intent: SolveIntent::Root,
            selection: SolverSelection::Explicit(Backend::Kinsol),
            ..Default::default()
        },
    )
    .await
}
#[cfg(feature = "solver-kinsol")]
async fn original_order_on(
    runtime: crate::workflow::Runtime,
    text: &str,
    order: DerivativeOrder,
    profile: SolverProfile,
) -> (crate::workflow::Runtime, PreparedSolve) {
    use crate::workflow::tests as fixture;
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
    let package = runtime.modeling_package(rows, fixture::physical()).unwrap();
    let prepared = package
        .prepare_solve(
            root,
            pse_modeling::specialize::root_instance(root),
            pse_modeling::Bindings::default(),
            pse_modeling::Limits::default(),
            pse_compiler::workspace::ModelingCaseBindings::default(),
            order,
            fixture::compiler_profile(),
            profile,
            NumericalInputs::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    (runtime, prepared.solve)
}
#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn actual_compiled_nonzero_equality_homotopy_is_screened_proposal_not_original_acceptance() {
    let (runtime, original) = original(
        "package p { def Root { var x:Scalar; annotation start x(0); eq balance:x==3; } }",
    )
    .await;
    let service = runtime.native();
    let flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let scope = ExecutionScope::new(
        flag.clone(),
        Some(Instant::now() + std::time::Duration::from_secs(30)),
    );
    let mut profile = original.profile.clone();
    profile.controls.time_limit = std::time::Duration::from_secs(10);
    let scoped_original = original.clone().within_task(scope.clone()).unwrap();
    let wrong_scope = ExecutionScope::new(
        Arc::new(std::sync::atomic::AtomicBool::new(false)),
        scope.deadline(),
    );
    assert!(matches!(
        service
            .prepare_derived(
                scoped_original,
                DerivedRequest::AnchoredHomotopy {
                    anchor: vec![0.0],
                    parameter: 0.5
                },
                profile.clone(),
                wrong_scope,
                &crate::CancelSource::new()
            )
            .await,
        Err(MathRuntimeError::Solve(ProblemError::Contract(_)))
    ));
    let prepared = service
        .prepare_derived(
            original.clone(),
            DerivedRequest::AnchoredHomotopy {
                anchor: vec![0.0],
                parameter: 0.5,
            },
            profile,
            scope.clone(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    assert_eq!(
        prepared.original_identity(),
        original.original_identity().unwrap()
    );
    assert_eq!(
        prepared.original().profile.controls.start,
        StartPolicy::NoPriorStart
    );
    assert!(Arc::ptr_eq(
        prepared.scope().cancellation(),
        scope.cancellation()
    ));
    assert_eq!(prepared.scope().deadline(), scope.deadline());
    assert_eq!(prepared.physical.constraints()[0].lower, 0.0);
    assert_eq!(prepared.family.original().constraints()[0].lower, 0.0);
    let execution = Execution::within(flag, &prepared.profile.controls, scope).unwrap();
    let budget = WorkerBudget::drawing(service.policy.worker_bytes, &service.pool);
    let result = execution::scoped(
        &[execution::adapter(Backend::Kinsol)],
        1,
        service.policy.stack_bytes,
        || {
            let mut retained = Retained::default();
            let result =
                service.derived_step(&prepared, execution.clone(), &mut retained, &budget, &[0.0]);
            if let Ok(attempt) = &result {
                let complete = service
                    .derived_original_outcome(&prepared, attempt, &execution, &budget)
                    .unwrap();
                let original_report = native_report(&complete);
                assert_eq!(original_report.termination.category, Termination::Success);
                assert!(!original_report.quality.as_ref().unwrap().feasible());
                assert_eq!(original_report.qualification, Qualification::Unqualified);
                assert!(
                    original_report
                        .candidate
                        .as_ref()
                        .unwrap()
                        .row_dual
                        .is_none()
                );
                assert_eq!(
                    original_report.variables,
                    prepared
                        .physical
                        .coordinates()
                        .iter()
                        .map(|c| c.id)
                        .collect::<Vec<_>>()
                );
            }
            drop(retained);
            assert_eq!(budget.used(), 0);
            result
        },
    )
    .unwrap();
    let proposal = result.proposal.unwrap();
    assert!((proposal.coordinates[0] - 1.5).abs() < 1e-7);
    assert!((proposal.coordinates[0] - 3.0).abs() > 1.0);
    assert_eq!(proposal.original, original.original_identity().unwrap());
    assert!(proposal.reconstruction_accuracy.is_none());
    assert!(result.screening_failure.is_none());
    assert_eq!(
        native_report(&result.outcome).termination.category,
        Termination::Success
    );
    assert_eq!(budget.used(), 0);
}
#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn actual_shifted_frozen_mass_retains_offset_and_submission_scope() {
    let (runtime, original) = original(
        "package p { def Root { var x:Scalar; annotation start x(0); eq balance:x==3; } }",
    )
    .await;
    let service = runtime.native();
    let flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let scope = ExecutionScope::new(
        flag.clone(),
        Some(Instant::now() + std::time::Duration::from_secs(30)),
    );
    let mut mass = AssemblyMatrix::new(
        1,
        1,
        &[Entry::new(GlobalRow::new(0), GlobalCol::new(0))],
        10,
    )
    .unwrap();
    mass.add(pse_math::index::Addend::new(0), 2.0).unwrap();
    let prepared = service
        .prepare_derived(
            original.clone(),
            DerivedRequest::ShiftedPseudoTime {
                anchor: vec![0.0],
                step: 1.0,
                sign: 1.0,
                pairing: vec![GlobalCol::new(0)],
                mass: Arc::new(mass),
            },
            original.profile.clone(),
            scope.clone(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let execution = Execution::within(flag, &prepared.profile.controls, scope).unwrap();
    let budget = WorkerBudget::drawing(service.policy.worker_bytes, &service.pool);
    let result = execution::scoped(
        &[execution::adapter(Backend::Kinsol)],
        1,
        service.policy.stack_bytes,
        || {
            let mut retained = Retained::default();
            let result = service.derived_step(&prepared, execution, &mut retained, &budget, &[0.0]);
            drop(retained);
            assert_eq!(budget.used(), 0);
            result
        },
    )
    .unwrap();
    assert!((result.proposal.unwrap().coordinates[0] - 1.0).abs() < 1e-7);
    assert_eq!(prepared.physical.constraints()[0].upper, 0.0);
    assert_eq!(budget.used(), 0);
    let expired = ExecutionScope::new(
        Arc::new(std::sync::atomic::AtomicBool::new(false)),
        Some(Instant::now()),
    );
    assert!(
        service
            .prepare_derived(
                original.clone(),
                DerivedRequest::AnchoredHomotopy {
                    anchor: vec![0.0],
                    parameter: 1.0
                },
                original.profile.clone(),
                expired,
                &crate::CancelSource::new()
            )
            .await
            .is_err()
    );
}

#[cfg(all(feature = "solver-kinsol", feature = "solver-ipopt"))]
#[tokio::test]
async fn least_deviation_preserves_original_rows_metric_roles_and_actual_derivative_order() {
    let (runtime, original) = original(
        "package p { def Root { var x:Scalar; annotation start x(0); eq balance:x==3; } }",
    )
    .await;
    let service = runtime.native();
    let Representation::Algebraic(source) = &original.representation else {
        panic!("expected actual algebraic fixture")
    };
    let id = source.prepared.prepared.plan.columns()[0];
    let request = DerivedRequest::LeastDeviation {
        center: vec![0.0],
        scales: vec![2.0],
        weights: vec![8.0],
        free: vec![id],
        held: vec![],
        metric_source: pse_ids::ContentHash::from_bytes([9; 32]),
    };
    let flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let scope = ExecutionScope::new(
        flag.clone(),
        Some(Instant::now() + std::time::Duration::from_secs(30)),
    );
    let mut profile = original.profile.clone();
    profile.intent = SolveIntent::Initialize;
    profile.selection = SolverSelection::Explicit(Backend::Ipopt);
    profile.controls.hessian = HessianMode::LimitedMemory;
    let prepared = service
        .prepare_derived(
            original.clone(),
            request.clone(),
            profile.clone(),
            scope.clone(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    assert_eq!(prepared.mechanism(), MechanismKind::LeastDeviation);
    let PreparedFamily::LeastDeviation(family) = prepared.family() else {
        panic!("must retain actual distance product")
    };
    assert_eq!(family.scales(), [2.0]);
    assert_eq!(family.weights(), [8.0]);
    assert_eq!(family.curvature(), [2.0]);
    assert_eq!(family.original().constraints()[0].lower, 0.0);
    assert_eq!(family.original().support().order, DerivativeOrder::First);
    assert_eq!(family.objective_support().order, DerivativeOrder::Second);
    assert_eq!(prepared.physical.constraints()[0].upper, 0.0);
    let execution = Execution::within(flag, &prepared.profile.controls, scope.clone()).unwrap();
    let budget = WorkerBudget::drawing(service.policy.worker_bytes, &service.pool);
    let result = execution::scoped(
        &[execution::adapter(Backend::Ipopt)],
        1,
        service.policy.stack_bytes,
        || {
            let mut retained = Retained::default();
            let result =
                service.derived_step(&prepared, execution.clone(), &mut retained, &budget, &[0.0]);
            if let Ok(attempt) = &result {
                let complete = service
                    .derived_original_outcome(&prepared, attempt, &execution, &budget)
                    .unwrap();
                let original_report = native_report(&complete);
                assert!(original_report.quality.as_ref().unwrap().feasible());
                assert_eq!(original_report.qualification, Qualification::Feasible);
                assert!(
                    original_report
                        .candidate
                        .as_ref()
                        .unwrap()
                        .row_dual
                        .is_none()
                );
                assert!(original_report.certificate.is_none());
                assert!(
                    original_report
                        .observation
                        .as_ref()
                        .unwrap()
                        .stationarity
                        .is_none()
                );
            }
            drop(retained);
            assert_eq!(budget.used(), 0);
            result
        },
    )
    .unwrap();
    assert!((result.proposal.unwrap().coordinates[0] - 3.0).abs() < 1e-6);
    assert!(result.screening_failure.is_none());
    assert_eq!(
        native_report(&result.outcome).termination.category,
        Termination::Success
    );
    profile.controls.hessian = HessianMode::Exact;
    assert!(
        service
            .prepare_derived(
                original.clone(),
                request.clone(),
                profile.clone(),
                scope.clone(),
                &crate::CancelSource::new()
            )
            .await
            .is_err()
    );
    profile.controls.hessian = HessianMode::LimitedMemory;
    let wrong = DerivedRequest::LeastDeviation {
        center: vec![0.0],
        scales: vec![2.0],
        weights: vec![8.0],
        free: vec![SemanticId::NIL],
        held: vec![],
        metric_source: pse_ids::ContentHash::from_bytes([9; 32]),
    };
    assert!(
        service
            .prepare_derived(original, wrong, profile, scope, &crate::CancelSource::new())
            .await
            .is_err()
    );
}

#[cfg(all(feature = "solver-kinsol", feature = "solver-ipopt"))]
#[tokio::test]
async fn least_deviation_exact_second_and_named_held_roles_use_actual_source() {
    let (runtime, original) = original_order(
        "package p { def Root { var x:Scalar; annotation start x(2); eq balance:x*x==9; } }",
        DerivativeOrder::Second,
    )
    .await;
    let service = runtime.native();
    let Representation::Algebraic(source) = &original.representation else {
        panic!("compiled source")
    };
    let id = source.prepared.prepared.plan.columns()[0];
    let scope = ExecutionScope::new(
        Arc::new(std::sync::atomic::AtomicBool::new(false)),
        Some(Instant::now() + std::time::Duration::from_secs(30)),
    );
    let mut profile = original.profile.clone();
    profile.intent = SolveIntent::Initialize;
    profile.selection = SolverSelection::Explicit(Backend::Ipopt);
    profile.controls.hessian = HessianMode::Exact;
    let request = DerivedRequest::LeastDeviation {
        center: vec![0.0],
        scales: vec![2.0],
        weights: vec![8.0],
        free: vec![id],
        held: vec![],
        metric_source: pse_ids::ContentHash::from_bytes([10; 32]),
    };
    let prepared = service
        .prepare_derived(
            original.clone(),
            request,
            profile.clone(),
            scope.clone(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    assert_eq!(prepared.family().support().order, DerivativeOrder::Second);
    let execution = Execution::within(
        scope.cancellation().clone(),
        &prepared.profile.controls,
        scope.clone(),
    )
    .unwrap();
    let budget = WorkerBudget::drawing(service.policy.worker_bytes, &service.pool);
    let result = execution::scoped(
        &[execution::adapter(Backend::Ipopt)],
        1,
        service.policy.stack_bytes,
        || {
            let mut retained = Retained::default();
            let result = service.derived_step(&prepared, execution, &mut retained, &budget, &[2.0]);
            drop(retained);
            assert_eq!(budget.used(), 0);
            result
        },
    )
    .unwrap();
    assert_eq!(
        native_report(&result.outcome).termination.category,
        Termination::Success
    );
    assert!((result.proposal.unwrap().coordinates[0] - 3.0).abs() < 1e-6);
    let held = DerivedRequest::LeastDeviation {
        center: vec![3.0],
        scales: vec![2.0],
        weights: vec![8.0],
        free: vec![],
        held: vec![id],
        metric_source: pse_ids::ContentHash::from_bytes([10; 32]),
    };
    let held = service
        .prepare_derived(original, held, profile, scope, &crate::CancelSource::new())
        .await
        .unwrap();
    let PreparedFamily::LeastDeviation(family) = held.family() else {
        panic!("distance product")
    };
    assert_eq!(family.coordinate_bounds(), [(3.0, 3.0)]);
    assert_ne!(prepared.family().key(), held.family().key());
    assert!(family.validate_point(&[2.0]).is_err());
}

#[cfg(feature = "solver-kinsol")]
fn expiry_declaration(
    prepared: &PreparedDerived,
    original: &PreparedSolve,
) -> pse_model::strategy::NumericalStrategy {
    use pse_model::strategy::{NumericalStrategy, Position, ProfileRef, Transition, WorkLimits};
    let mut declaration = NumericalStrategy::direct(
        StartPolicy::NoPriorStart,
        WorkLimits {
            attempts: 2,
            evaluations: None,
            iterations: None,
            factorizations: None,
            proof_steps: None,
        },
    );
    declaration.mechanisms[0].profile = Some(ProfileRef {
        backend: Backend::Kinsol,
        key: original.strategy_profile().unwrap(),
    });
    let mut preparation = declaration.mechanisms[0].clone();
    preparation.kind = MechanismKind::Homotopy;
    preparation.position = Position::Preparation;
    preparation.required = false;
    preparation.profile = Some(prepared.profile_ref().unwrap());
    preparation.support = vec![prepared.family().key()];
    preparation.transitions = vec![Transition::Continue, Transition::Stop];
    declaration.mechanisms.insert(0, preparation);
    declaration
}
#[cfg(feature = "solver-kinsol")]
fn wait_for_actual_deadline(scope: &ExecutionScope) {
    let deadline = scope.deadline().unwrap();
    if let Some(remaining) = deadline.checked_duration_since(Instant::now()) {
        std::thread::sleep(remaining + std::time::Duration::from_millis(1));
    }
    assert!(matches!(
        scope.check(),
        Err(pse_kernels::ProviderError::Deadline)
    ));
}
#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn expired_optional_derived_preparation_refuses_before_dispatch_and_original_still_qualifies()
{
    use crate::math::{SessionDisposition, StepRetention};
    use crate::workflow::numerics;
    use pse_model::generated::enums::NumericalEventKind;
    use pse_model::strategy::{StartOrigin, Transition};
    let (runtime, original) = original(
        "package p { def Root { var x:Scalar; annotation start x(0); eq balance:x==3; } }",
    )
    .await;
    let service = runtime.native();
    let enclosing = ExecutionScope::new(
        Arc::default(),
        Some(Instant::now() + std::time::Duration::from_secs(30)),
    );
    let mut profile = original.profile.clone();
    profile.controls.time_limit = std::time::Duration::from_secs(1);
    let prepared = service
        .prepare_derived(
            original.clone(),
            DerivedRequest::AnchoredHomotopy {
                anchor: vec![0.],
                parameter: 0.5,
            },
            profile,
            enclosing.clone(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let declaration = expiry_declaration(&prepared, &original);
    assert!(declaration.start.recovery.is_empty());
    wait_for_actual_deadline(prepared.operation_scope());
    assert!(prepared.local_scope_refusal(&enclosing).is_some());
    let composed = original
        .clone()
        .with_strategy(declaration, vec![prepared.into(), original.into()])
        .unwrap();
    let owner = service
        .reserve(
            "test:derived-expiry-result",
            composed.result_bytes().unwrap(),
        )
        .unwrap();
    let session = service.open_session().unwrap();
    let (outcome, accepted, trace) = session
        .step(
            composed,
            None,
            0,
            Arc::new(Progress::new(100)),
            owner,
            &crate::CancelSource::new(),
            |outcome, _, _| {
                let candidate = match outcome {
                    Outcome::Native(report) => {
                        numerics::native_use(report, &NumericalPolicy::default())
                    }
                    Outcome::Constant(report) => numerics::constant_use(&report.quality),
                    Outcome::Rejected(_) => numerics::refused(
                        pse_model::generated::enums::CandidateRefusal::NoCandidate,
                    ),
                };
                crate::math::strategy::Assessed::native(
                    candidate.permits_use(),
                    StepRetention {
                        candidate,
                        session: SessionDisposition::Discard,
                    },
                    outcome,
                )
            },
        )
        .await
        .unwrap();
    session.close().await;
    assert!(accepted);
    let Outcome::Native(report) = outcome else {
        panic!("original corrector required")
    };
    assert!((report.candidate.as_ref().unwrap().primal[0] - 3.).abs() < 1e-8);
    assert!(
        trace
            .events
            .iter()
            .any(|e| e.mechanism == 0 && e.kind == NumericalEventKind::Refused && e.work.is_none())
    );
    assert_eq!(
        trace
            .events
            .iter()
            .filter(|e| e.kind == NumericalEventKind::Started)
            .count(),
        1
    );
    assert_eq!(trace.starts[1], StartOrigin::Specification);
    assert_eq!(
        trace.events.last().unwrap().transition,
        Some(Transition::Finish)
    );
    assert!(enclosing.check().is_ok());
    assert!(
        !enclosing
            .cancellation()
            .load(std::sync::atomic::Ordering::Acquire)
    );
}

#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn actual_derived_callback_local_expiry_continues_with_charged_work_but_other_failures_stop()
{
    use crate::math::{SessionDisposition, StepRetention, strategy};
    use pse_model::{
        generated::enums::{NumericalAttemptObservation as O, NumericalEventKind},
        strategy::{Phase, Scope, StartOrigin, Transition, WorkCharge},
    };
    let (runtime, original) = original(
        "package p { def Root { var x:Scalar; annotation start x(0); eq balance:x==3; } }",
    )
    .await;
    let service = runtime.native();
    let enclosing = ExecutionScope::new(
        Arc::default(),
        Some(Instant::now() + std::time::Duration::from_secs(4)),
    );
    let mut profile = original.profile.clone();
    profile.controls.time_limit = std::time::Duration::from_secs(1);
    let prepared = service
        .prepare_derived(
            original.clone(),
            DerivedRequest::AnchoredHomotopy {
                anchor: vec![0.],
                parameter: 0.5,
            },
            profile,
            enclosing.clone(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let mut declaration = expiry_declaration(&prepared, &original);
    declaration.start.recovery.push(StartOrigin::Specification);
    let budget = WorkerBudget::drawing(service.policy.worker_bytes, &service.pool);
    let facts = |_: usize| strategy::Facts {
        support: prepared.support(),
        accuracy: Vec::new(),
        consumption: Vec::new(),
        reservation: None,
        start: StartOrigin::Specification,
        inherited: false,
        connected: false,
        refusal: None,
    };
    let observed = WorkObservation {
        attempts: 1,
        evaluations: Some(1),
        iterations: None,
        factorizations: None,
        proof_steps: None,
    };
    let result = execution::scoped(
        &[execution::adapter(Backend::Kinsol)],
        1,
        service.policy.stack_bytes,
        || {
            Ok::<_, ProblemError>(strategy::run(
                &declaration,
                &enclosing,
                facts,
                |index, _| {
                    if index == 0 {
                        // A real compiled callback runs inside the producer's actual operation scope.
                        let (mut oracle, _owner) = service
                            .derived_original_oracle(&prepared, prepared.operation_scope(), &budget)
                            .unwrap();
                        let mut rows = [0.];
                        NlpOracle::constraints(&mut oracle, &[0.], &mut rows).unwrap();
                        assert_eq!(rows, [-3.]);
                        drop(oracle);
                        drop(_owner);
                        wait_for_actual_deadline(prepared.operation_scope());
                        let cause = Arc::new(ProblemError::Provider(
                            prepared.operation_scope().check().unwrap_err(),
                        ));
                        Err(prepared.effect_failure(cause, observed, &enclosing))
                    } else {
                        let (mut oracle, _owner) = service
                            .derived_original_oracle(&prepared, &enclosing, &budget)
                            .unwrap();
                        let mut rows = [f64::NAN];
                        NlpOracle::constraints(&mut oracle, &[3.], &mut rows).unwrap();
                        assert_eq!(rows, [0.]);
                        Ok(strategy::Attempt {
                            value: rows[0],
                            observation: O::Converged,
                            work: WorkCharge {
                                phase: Phase::Native,
                                scope: Scope::Task,
                                charging_owner: pse_ids::ContentHash::from_bytes([241; 32]),
                                observed,
                            },
                        })
                    }
                },
                |residual, observation| {
                    assert_eq!(*residual, 0.);
                    strategy::Assessment {
                        original: strategy::OriginalConclusion::Satisfied,
                        work: vec![],
                        auxiliary: false,
                        retention: StepRetention {
                            candidate: crate::workflow::numerics::CandidateDecision {
                                usability: pse_model::generated::enums::CandidateUse::Usable,
                                qualifiers: Vec::new(),
                                refusals: Vec::new(),
                                bound: None,
                            },
                            session: SessionDisposition::Discard,
                        },
                        observation,
                        cause: None,
                    }
                },
            ))
        },
    )
    .unwrap();
    assert!(result.terminal.is_none());
    assert_eq!(result.value, Some(0.));
    assert_eq!(result.work.attempts, 2);
    assert_eq!(result.work.evaluations, Some(2));
    assert_eq!(budget.used(), 0);
    let abandoned = result
        .events
        .iter()
        .find(|e| e.kind == NumericalEventKind::Abandoned)
        .unwrap();
    assert_eq!(abandoned.transition, Some(Transition::Continue));
    assert_eq!(abandoned.work.unwrap().observed, observed);
    assert!(matches!(
        abandoned.cause.as_deref(),
        Some(ProblemError::Provider(pse_kernels::ProviderError::Deadline))
    ));
    // Expiration does not turn a genuine finite-pool or contract failure into time.
    let pool_cause = Arc::new(
        service
            .reserve("test:actual-pool-refusal", usize::MAX)
            .unwrap_err()
            .into_problem(),
    );
    for cause in [
        pool_cause,
        Arc::new(ProblemError::Contract(
            "actual supplier contract refusal".into(),
        )),
        Arc::new(ProblemError::Provider(
            pse_kernels::ProviderError::Cancelled,
        )),
    ] {
        assert!(!genuine_time_cause(&cause));
        let result = strategy::run::<()>(
            &declaration,
            &enclosing,
            facts,
            |index, _| {
                assert_eq!(index, 0);
                Err(prepared.effect_failure(cause.clone(), observed, &enclosing))
            },
            |_, _| panic!("terminal failure cannot assess"),
        );
        assert!(result.terminal.is_some());
        assert_eq!(result.work, observed);
        assert_eq!(
            result.events.last().unwrap().transition,
            Some(Transition::Stop)
        );
    }
    let time = Arc::new(ProblemError::Provider(
        prepared.operation_scope().check().unwrap_err(),
    ));
    declaration.mechanisms[0].required = true;
    let result = strategy::run::<()>(
        &declaration,
        &enclosing,
        facts,
        |index, _| {
            assert_eq!(index, 0);
            Err(prepared.effect_failure(time.clone(), observed, &enclosing))
        },
        |_, _| panic!("required local expiry cannot assess"),
    );
    assert!(result.terminal.is_some());
    assert_eq!(result.work, observed);
    assert_eq!(
        result.events.last().unwrap().transition,
        Some(Transition::Stop)
    );
    declaration.mechanisms[0].required = false;
    declaration.mechanisms[0].transitions = vec![Transition::Stop];
    let result = strategy::run::<()>(
        &declaration,
        &enclosing,
        facts,
        |index, _| {
            assert_eq!(index, 0);
            Err(prepared.effect_failure(time.clone(), observed, &enclosing))
        },
        |_, _| panic!("undeclared local continuation cannot assess"),
    );
    assert!(result.terminal.is_some());
    assert_eq!(result.work, observed);
    assert_eq!(
        result.events.last().unwrap().transition,
        Some(Transition::Stop)
    );
    wait_for_actual_deadline(&enclosing);
    assert!(prepared.local_scope_refusal(&enclosing).is_none());
    let result = strategy::run::<()>(
        &declaration,
        &enclosing,
        facts,
        |_, _| panic!("expired original task cannot dispatch"),
        |_, _| panic!("parent terminal"),
    );
    assert!(result.terminal.is_some());
    assert_eq!(result.work.attempts, 0);
    assert!(
        !enclosing
            .cancellation()
            .load(std::sync::atomic::Ordering::Acquire)
    );
}

#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn derived_expiry_marker_preserves_actual_native_report_work_and_foreign_failure_veto() {
    let (runtime, original) = original(
        "package p { def Root { var x:Scalar; annotation start x(0); eq balance:x==3; } }",
    )
    .await;
    let service = runtime.native();
    let enclosing = ExecutionScope::new(
        Arc::default(),
        Some(Instant::now() + std::time::Duration::from_secs(30)),
    );
    let mut profile = original.profile.clone();
    profile.controls.time_limit = std::time::Duration::from_secs(1);
    let prepared = service
        .prepare_derived(
            original,
            DerivedRequest::AnchoredHomotopy {
                anchor: vec![0.],
                parameter: 0.5,
            },
            profile,
            enclosing.clone(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let execution = Execution::within(
        enclosing.cancellation().clone(),
        prepared.controls(),
        enclosing.clone(),
    )
    .unwrap();
    let budget = WorkerBudget::drawing(service.policy.worker_bytes, &service.pool);
    let mut result = execution::scoped(
        &[execution::adapter(Backend::Kinsol)],
        1,
        service.policy.stack_bytes,
        || {
            let mut retained = Retained::default();
            let value = service.derived_step(&prepared, execution, &mut retained, &budget, &[0.]);
            drop(retained);
            value
        },
    )
    .unwrap();
    assert_eq!(
        native_report(&result.outcome).termination.category,
        Termination::Success
    );
    assert!(result.work().evaluations.is_some_and(|n| n > 0));
    assert!(
        prepared
            .local_attempt_failure(&result, &enclosing)
            .is_none()
    );
    wait_for_actual_deadline(prepared.operation_scope());
    let cause = Arc::new(ProblemError::Provider(
        prepared.operation_scope().check().unwrap_err(),
    ));
    // Supply the actual scope failure at the post-native screening boundary; the
    // completed library report and counters are retained independently of it.
    result.proposal = None;
    result.screening_failure = Some(cause.clone());
    let failed = prepared.local_attempt_failure(&result, &enclosing).unwrap();
    assert!(Arc::ptr_eq(&failed.cause, &cause));
    assert_eq!(failed.observed, result.work());
    let actual_pool_failure = service
        .reserve("test:derived-report-pool-refusal", usize::MAX)
        .unwrap_err()
        .into_problem();
    if let Outcome::Native(report) = &mut result.outcome {
        report.record_validation_failure(actual_pool_failure);
    }
    assert!(
        prepared
            .local_attempt_failure(&result, &enclosing)
            .is_none()
    );
    assert_eq!(budget.used(), 0);
}

#[cfg(all(
    feature = "solver-kinsol",
    feature = "solver-ipopt",
    feature = "solver-root-isolation"
))]
#[tokio::test]
async fn reduced_compiler_source_refines_consumed_accuracy_then_original_corrector_qualifies() {
    use crate::math::strategy;
    use crate::workflow::tests as fixture;
    use pse_math::{
        implicit::{Configuration, Factory, Options, RegimeFactoryBranch, Selection, Unknown},
        library::Optimization,
        typed::{BodyBuilder, BodyLimits},
    };
    use pse_model::strategy::{
        AccuracyClass, NumericalStrategy, Position, ProfileRef, StartOrigin, Transition, WorkLimits,
    };
    use std::{collections::BTreeMap, time::Duration};
    let runtime = fixture::runtime_on(
        24usize << 30,
        crate::math::MathPolicy {
            worker_bytes: 8usize << 30,
            workspace_bytes: 8usize << 30,
            foreign_bytes: 16usize << 20,
            ..Default::default()
        },
    );
    let physical = fixture::physical();
    let rows=pse_authoring::language::parse("package p { def Root { var x:Scalar; var p:Scalar; eq root:x*x==p; eq cap:p<=8; let cost:Scalar=(p-4)*(p-4); annotation objective cost(minimize); annotation bounds x(0.01,3.5); annotation bounds p(0.1,9); annotation start x(2); annotation start p(4); } }",SemanticId::NIL,pse_authoring::language::IdentityPolicy::Named,pse_authoring::ParseBudget::default()).unwrap();
    let root = rows
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    let package = runtime.modeling_package(rows, physical.clone()).unwrap();
    let mut profile = fixture::profile();
    profile.intent = SolveIntent::Optimize;
    profile.selection = SolverSelection::Explicit(Backend::Ipopt);
    profile.controls.hessian = HessianMode::LimitedMemory;
    profile.controls.time_limit = Duration::from_secs(20);
    let compiler = fixture::compiler_profile();
    let cancel = crate::CancelSource::new();
    let mut prepared = package
        .prepare_solve(
            root,
            pse_modeling::specialize::root_instance(root),
            pse_modeling::Bindings::default(),
            pse_modeling::Limits::default(),
            pse_compiler::workspace::ModelingCaseBindings::default(),
            DerivativeOrder::First,
            compiler,
            profile.clone(),
            NumericalInputs::default(),
            &cancel,
        )
        .await
        .unwrap();
    let original = prepared.solve.clone();
    let Representation::Algebraic(source) = &original.representation else {
        panic!("compiled source")
    };
    let plan = &source.prepared.prepared.plan;
    let binding = plan
        .structure()
        .instances()
        .iter()
        .find(|i| {
            i.slots.len() == 2
                && i.contributions
                    .iter()
                    .any(|c| matches!(c.target, pse_math::binding::Target::Row(_)))
        })
        .unwrap();
    assert_eq!(binding.slots.len(), 2);
    assert!(
        binding
            .slots
            .iter()
            .all(|s| s.scale() == 1.0 && s.offset() == 0.0)
    );
    let unknown_id = binding.slots[0].source();
    let unknown_is_x = plan
        .structure()
        .variables()
        .iter()
        .find(|v| v.port.id == unknown_id)
        .unwrap()
        .lower
        == Some(0.01);
    let parameter_id = binding.slots[1].source();
    let unknown_column = plan
        .columns()
        .iter()
        .position(|id| *id == unknown_id)
        .unwrap();
    let retained_column = plan
        .columns()
        .iter()
        .position(|id| *id == parameter_id)
        .unwrap();
    let eliminated = plan
        .structure()
        .rows()
        .iter()
        .position(|r| r.lower == r.upper)
        .unwrap();
    let row = plan.structure().rows()[eliminated].id;
    let output = binding
        .contributions
        .iter()
        .find(|c| c.target == pse_math::binding::Target::Row(row))
        .unwrap();
    assert_eq!(output.scale, 1.0);
    let residual = plan.bodies().get(&binding.body).unwrap();
    assert_eq!(residual.input_count(), 2);
    let flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let scope = ExecutionScope::new(flag.clone(), Some(Instant::now() + Duration::from_secs(30)));
    let support = residual
        .prepare_support(&[output.output], &[0, 1], DerivativeOrder::First, &flag)
        .unwrap();
    assert_eq!(support.outputs(), [output.output]);
    assert_eq!(support.coordinates(), [0, 1]);
    let body = Arc::new(
        support
            .compile(
                Optimization::default(),
                pse_math::jets::EvaluationLimits::default(),
                &flag,
            )
            .unwrap(),
    );
    let q = binding.slots[0].quantity();
    assert_eq!(binding.slots[1].quantity(), q);
    let unit_id = physical.quantities.quantity_type(q).unwrap().canonical_unit;
    let unit = physical.quantities.unit(unit_id).unwrap();
    let make = |values: &[f64]| {
        let mut b = BodyBuilder::new(
            pse_math::initialize().unwrap(),
            &physical.quantities,
            &pse_quantity::standard::StandardInvariantChecker,
            2,
            BodyLimits::default(),
        )
        .unwrap();
        let out = values
            .iter()
            .map(|v| {
                b.literal(
                    *v,
                    unit,
                    pse_quantity::literal::LiteralContext::Explicit { quantity_type: q },
                    row,
                )
                .unwrap()
            })
            .collect::<Vec<_>>();
        b.prepare(&out).unwrap()
    };
    let eligibility = make(&[1.0]);
    let criterion = make(&[0.0, 0.0]);
    let projection = pse_math::factorable::root_isolation_program_for_outputs(
        binding.instance,
        residual,
        &[output.output],
        &eligibility,
        &criterion,
        &flag,
        10_000,
    )
    .unwrap()
    .unwrap();
    assert_eq!(projection.residuals.len(), 1);
    let eligibility = Arc::new(
        eligibility
            .compile(
                &[0],
                &[0, 1],
                DerivativeOrder::First,
                Optimization::default(),
                pse_math::jets::EvaluationLimits::default(),
                &flag,
            )
            .unwrap(),
    );
    let criterion = Arc::new(
        criterion
            .compile(
                &[0, 1],
                &[0, 1],
                DerivativeOrder::First,
                Optimization::default(),
                pse_math::jets::EvaluationLimits::default(),
                &flag,
            )
            .unwrap(),
    );
    let port = |id| pse_kernels::Port {
        id,
        quantity: q,
        unit: unit_id,
    };
    let source_key = pse_ids::ContentHash::from_bytes([72; 32]);
    let spec = pse_kernels::ProviderSpec {
        shapes: Default::default(),
        derivative_source: pse_kernels::DerivativeSource::Implicit,
        id: row,
        revision: source_key,
        data: source_key,
        inputs: vec![port(parameter_id)],
        outputs: vec![port(unknown_id)],
        derivatives: DerivativeOrder::First,
        smoothness: DerivativeOrder::First,
    };
    let unknown = plan
        .structure()
        .variables()
        .iter()
        .find(|v| v.port.id == unknown_id)
        .unwrap();
    let unknowns = vec![Unknown {
        id: unknown_id,
        lower: unknown.lower.unwrap(),
        upper: unknown.upper.unwrap(),
    }];
    let factory = Arc::new(RegimeFactory {
        spec: spec.clone(),
        alternatives: vec![RegimeFactoryBranch {
            residual: Factory {
                selection: Selection::default(),
                requirements: pse_kernels::DerivativeRequirements::new(
                    DerivativeOrder::First,
                    DerivativeOrder::First,
                    DerivativeOrder::First,
                    DerivativeOrder::First,
                    DerivativeOrder::First,
                )
                .unwrap(),
                spec,
                body,
                unknowns: unknowns.clone(),
                rows: vec![row],
                configuration: Configuration::Fixed(
                    unknowns,
                    Options {
                        start: vec![if unknown_is_x {
                            2.000000001
                        } else {
                            4.000000001
                        }],
                        variable_nominals: vec![1.0],
                        variable_tolerance: vec![1e-8],
                        residual_tolerance: vec![1e-8],
                        iterations: 100,
                        time_limit: Duration::from_secs(10),
                        derivative_tolerance: 1e-8,
                    },
                ),
                hints: None,
                terms: None,
                solver: Arc::new(native::implicit::Kinsol),
                cancel: flag.clone(),
                max_entries: 100,
                providers: BTreeMap::new(),
            },
            eligibility,
            criterion,
            isolation: Some(Arc::new(projection)),
        }],
        maximum_regimes: 1,
        time_limit: Duration::from_secs(10),
        cancel: flag.clone(),
        verifier: Some(Arc::new(native::root_isolation::Ibex)),
    });
    let service = runtime.native();
    let owner = service
        .reserve(
            "test:actual-selected-source",
            factory.retained_bytes().unwrap(),
        )
        .unwrap();
    let request = DerivedRequest::Reduced {
        suppliers: vec![ReducedSupplier {
            factory,
            factory_owner: owner,
            validity: source_key,
            realization: SelectedResidualRealization::ZeroResiduals {
                authored_offsets: vec![(row, plan.structure().rows()[eliminated].lower)],
            },
        }],
        retained: vec![GlobalCol::new(retained_column)],
        accuracy: ReconstructionAccuracy {
            point: 1e-11,
            action: 1e-11,
            class: AccuracyClass::Certified,
            refinement: math::RefinementLimits {
                rounds: 8,
                proof_cells: 64,
            },
        },
    };
    let mut limited_request = request.clone();
    if let DerivedRequest::Reduced { accuracy, .. } = &mut limited_request {
        accuracy.refinement.proof_cells = 1;
    }
    let plain = prepared.clone();
    let auxiliary = service
        .prepare_derived(
            original.clone(),
            request,
            profile.clone(),
            scope.clone(),
            &cancel,
        )
        .await
        .unwrap();
    let mut declaration = NumericalStrategy::direct(
        StartPolicy::NoPriorStart,
        WorkLimits {
            attempts: 2,
            evaluations: None,
            iterations: None,
            factorizations: None,
            proof_steps: None,
        },
    );
    declaration.start.recovery.push(StartOrigin::Auxiliary);
    declaration.mechanisms[0]
        .starts
        .push(StartOrigin::Auxiliary);
    declaration.mechanisms[0].profile = Some(ProfileRef {
        backend: Backend::Ipopt,
        key: original.strategy_profile().unwrap(),
    });
    let mut prep = declaration.mechanisms[0].clone();
    prep.kind = MechanismKind::ReducedSpace;
    prep.position = Position::Preparation;
    prep.profile = Some(auxiliary.profile_ref().unwrap());
    prep.support = vec![auxiliary.family().key()];
    prep.transitions.push(Transition::Continue);
    declaration.mechanisms.insert(0, prep);
    prepared.solve = original
        .clone()
        .with_strategy(declaration, vec![auxiliary.into(), original.clone().into()])
        .unwrap();
    let result = package
        .solve_case(prepared, compiler, &cancel)
        .await
        .unwrap();
    let anchor = result.prediction_anchor(0.0);
    assert!(anchor.is_ok(), "{anchor:?}; diagnostic={:?}", result.diagnostic());
    let Outcome::Native(report) = &result.outcome else {
        panic!("actual original corrector")
    };
    let point = &report.candidate.as_ref().unwrap().primal;
    let (x, p) = if unknown_is_x {
        (point[unknown_column], point[retained_column])
    } else {
        (point[retained_column], point[unknown_column])
    };
    assert!((x * x - p).abs() < 1e-7);
    assert!((p - 4.0).abs() < 1e-5);
    let trace = result.strategy.as_ref().unwrap();
    assert_eq!(
        trace
            .events
            .iter()
            .filter(|e| e.kind == pse_model::generated::enums::NumericalEventKind::Finished)
            .count(),
        2
    );
    assert!(
        !scope
            .cancellation()
            .load(std::sync::atomic::Ordering::Acquire)
    );
    let auxiliary = service
        .prepare_derived(
            original.clone(),
            limited_request,
            profile,
            scope.clone(),
            &cancel,
        )
        .await
        .unwrap();
    let execution = Execution::within(flag.clone(), auxiliary.controls(), scope.clone()).unwrap();
    let budget = WorkerBudget::drawing(service.policy.worker_bytes, &service.pool);
    let mut receipt = execution::scoped(
        &[execution::adapter(Backend::Ipopt)],
        1,
        service.policy.stack_bytes,
        || {
            let mut retained = Retained::default();
            let result = service.derived_step(
                &auxiliary,
                execution,
                &mut retained,
                &budget,
                &original.source_start(None).unwrap(),
            );
            drop(retained);
            result
        },
    )
    .unwrap_err();
    let cause = receipt.cause.clone();
    let (product, source_key, validity) = refinement_cause(&cause).unwrap();
    assert_eq!(receipt.refinement_product, Some(product));
    assert!(matches!(
        cause.as_ref(),
        ProblemError::Math(pse_math::MathError::Refinement {
            reason: math::RefinementRefusal::ProofCells,
            ..
        })
    ));
    let admitted = receipt.clone().effect(&auxiliary, &scope);
    receipt.refinement_product = Some(pse_ids::ContentHash::from_bytes([88; 32]));
    let wrong_product = receipt.clone().effect(&auxiliary, &scope);
    receipt.refinement_product = Some(product);
    let mut denied = vec![wrong_product];
    for (source_key, validity) in [
        (pse_ids::ContentHash::from_bytes([88; 32]), validity),
        (source_key, pse_ids::ContentHash::from_bytes([88; 32])),
    ] {
        receipt.cause = Arc::new(ProblemError::Math(pse_math::MathError::Refinement {
            product,
            source_key,
            validity,
            reason: math::RefinementRefusal::ProofCells,
        }));
        denied.push(receipt.clone().effect(&auxiliary, &scope));
    }
    receipt.cause = Arc::new(
        service
            .reserve("test:actual-unrelated-refinement-pool-refusal", usize::MAX)
            .unwrap_err()
            .into_problem(),
    );
    denied.push(receipt.clone().effect(&auxiliary, &scope));
    receipt.cause = cause;
    let mut declaration = NumericalStrategy::direct(
        StartPolicy::NoPriorStart,
        WorkLimits {
            attempts: 2,
            evaluations: None,
            iterations: None,
            factorizations: None,
            proof_steps: None,
        },
    );
    declaration.start.recovery.push(StartOrigin::Specification);
    declaration.mechanisms[0].profile = Some(ProfileRef {
        backend: Backend::Ipopt,
        key: original.strategy_profile().unwrap(),
    });
    let mut prep = declaration.mechanisms[0].clone();
    prep.kind = MechanismKind::ReducedSpace;
    prep.position = Position::Preparation;
    prep.required = false;
    prep.profile = Some(auxiliary.profile_ref().unwrap());
    prep.support = vec![auxiliary.family().key()];
    prep.transitions.push(Transition::Continue);
    declaration.mechanisms.insert(0, prep);
    for failure in denied {
        let mut failure = Some(failure);
        let result = strategy::run::<()>(
            &declaration,
            &scope,
            |_| strategy::Facts {
                start: StartOrigin::Specification,
                inherited: false,
                support: auxiliary.support(),
                accuracy: vec![],
                consumption: vec![],
                reservation: None,
                connected: false,
                refusal: None,
            },
            |index, _| {
                assert_eq!(index, 0);
                Err(failure.take().unwrap())
            },
            |_, _| panic!("unowned failure cannot assess"),
        );
        assert!(result.terminal.is_some());
        assert_eq!(
            result.events.last().unwrap().transition,
            Some(Transition::Stop)
        );
    }
    drop(admitted);
    let required = NumericalStrategy {
        mechanisms: declaration
            .mechanisms
            .iter()
            .cloned()
            .map(|mut m| {
                m.required = true;
                m
            })
            .collect(),
        ..declaration.clone()
    };
    let required_result = strategy::run::<()>(
        &required,
        &scope,
        |_| strategy::Facts {
            start: StartOrigin::Specification,
            inherited: false,
            support: auxiliary.support(),
            accuracy: vec![],
            consumption: vec![],
            reservation: None,
            connected: false,
            refusal: None,
        },
        |index, _| {
            assert_eq!(index, 0);
            Err(receipt.clone().effect(&auxiliary, &scope))
        },
        |_, _| panic!("required refusal cannot assess"),
    );
    assert!(required_result.terminal.is_some());
    assert_eq!(
        required_result.events.last().unwrap().transition,
        Some(Transition::Stop)
    );
    let mut retry = plain;
    retry.solve = original
        .clone()
        .with_strategy(declaration, vec![auxiliary.into(), original.into()])
        .unwrap();
    let recovered = package.solve_case(retry, compiler, &cancel).await.unwrap();
    assert!(
        recovered.prediction_anchor(0.0).is_ok(),
        "{:?}",
        recovered.diagnostic()
    );
    let rows = recovered
        .strategy
        .as_ref()
        .unwrap()
        .rows(recovered.run_id, 0)
        .unwrap();
    assert!(
        rows.iter()
            .any(|e| e.mechanism == MechanismKind::ReducedSpace
                && e.kind == pse_model::generated::enums::NumericalEventKind::Abandoned
                && e.transition == Some(Transition::Continue))
    );
    assert!(
        !scope
            .cancellation()
            .load(std::sync::atomic::Ordering::Acquire)
    );
}

#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn composition_retains_boxed_rung_bodies_and_refuses_a_short_pool() {
    use datafusion::execution::memory_pool::GreedyMemoryPool;
    use pse_model::{
        HeapUsage,
        strategy::{NumericalStrategy, ProfileRef, WorkLimits},
    };
    let (_runtime, original) =
        original("package p { def Root { var x:Scalar; annotation start x(0); eq root:x==1; } }")
            .await;
    let mut declaration = NumericalStrategy::direct(
        StartPolicy::NoPriorStart,
        WorkLimits {
            attempts: 1,
            evaluations: None,
            iterations: None,
            factorizations: None,
            proof_steps: None,
        },
    );
    declaration.mechanisms[0].profile = Some(ProfileRef {
        backend: original.backend().unwrap(),
        key: original.strategy_profile().unwrap(),
    });
    let rungs = vec![original.clone().into()];
    let expected = size_of::<PreparedComposition>()
        + declaration.heap_bytes()
        + rungs.capacity() * size_of::<PreparedRung>()
        + size_of::<PreparedSolve>();
    let before = original.pool.reserved();
    let composed = original
        .clone()
        .with_strategy(declaration.clone(), rungs)
        .unwrap();
    assert_eq!(original.pool.reserved() - before, expected);
    let shared = composed.composition.as_ref().unwrap().clone();
    drop(composed);
    assert_eq!(original.pool.reserved() - before, expected);
    drop(shared);
    assert_eq!(original.pool.reserved(), before);

    // This pool admits the old inline-only estimate but cannot admit the actual box body.
    let pool = Arc::new(GreedyMemoryPool::new(expected - 1));
    let mut limited = original;
    limited.pool = pool.clone();
    let rungs = vec![limited.clone().into()];
    let failure = limited.with_strategy(declaration, rungs).unwrap_err();
    assert!(matches!(
        failure,
        ProblemError::Limit {
            kind: native::LimitKind::Memory,
            ..
        }
    ));
    use datafusion::execution::memory_pool::MemoryPool;
    assert_eq!(pool.reserved(), 0);
}

#[cfg(all(
    feature = "solver-kinsol",
    feature = "solver-ipopt",
    feature = "solver-root-isolation"
))]
#[tokio::test]
async fn actual_full_reconstruction_uses_direct_original_outcome_without_native_attempt() {
    use pse_math::implicit::reconstruction::SelectedResidualRealization;
    use pse_math::{
        implicit::{Configuration, Factory, Options, RegimeFactoryBranch, Selection, Unknown},
        typed::{Binary, BodyBuilder, BodyLimits},
    };
    use pse_model::strategy::AccuracyClass;
    // IBEX's existing declared covering workspace is admitted even for this small
    // chart; retain the same bounded pool used by other actual isolation tests.
    let runtime = crate::workflow::tests::runtime_on(
        24usize << 30,
        crate::math::MathPolicy {
            worker_bytes: 8usize << 30,
            workspace_bytes: 8usize << 30,
            foreign_bytes: 16usize << 20,
            ..Default::default()
        },
    );
    let (runtime, original) = original_order_on(runtime,
        "package p { def Root { var x:Scalar; eq balance:x==3; annotation bounds x(1,4); annotation start x(2.5); } }",
        DerivativeOrder::First, SolverProfile {
            intent: SolveIntent::Initialize,
            selection: SolverSelection::Explicit(Backend::Ipopt),
            controls: Controls { hessian: HessianMode::LimitedMemory, ..Default::default() },
            ..Default::default()
        }).await;
    let service = runtime.native();
    let Representation::Algebraic(source) = &original.representation else {
        panic!("authored source required")
    };
    let plan = &source.prepared.prepared.plan;
    let coordinate = plan.columns()[0];
    let row = plan.structure().rows()[0].id;
    let registry = &source.prepared.prepared.quantities;
    let port = plan
        .structure()
        .variables()
        .iter()
        .find(|v| v.port.id == coordinate)
        .unwrap()
        .port
        .clone();
    let unit = registry.unit(port.unit).unwrap();
    let flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let scope = ExecutionScope::new(
        flag.clone(),
        Some(Instant::now() + std::time::Duration::from_secs(30)),
    );
    let builder = || {
        BodyBuilder::new(
            pse_math::initialize().unwrap(),
            registry,
            &pse_quantity::standard::StandardInvariantChecker,
            1,
            BodyLimits::default(),
        )
        .unwrap()
    };
    let literal = |b: &mut BodyBuilder<'_>, value| {
        b.literal(
            value,
            unit,
            pse_quantity::literal::LiteralContext::Explicit {
                quantity_type: port.quantity,
            },
            row,
        )
        .unwrap()
    };
    let mut b = builder();
    let x = b
        .input(0, port.quantity, pse_quantity::IndexSet::new(), coordinate)
        .unwrap();
    let rhs = literal(&mut b, 3.0);
    let residual = b.binary(Binary::Sub, x, rhs, None, row).unwrap();
    let residual = b.prepare(&[residual]).unwrap();
    let make = |values: &[f64]| {
        let mut b = builder();
        let out = values
            .iter()
            .map(|v| literal(&mut b, *v))
            .collect::<Vec<_>>();
        b.prepare(&out).unwrap()
    };
    let eligibility = make(&[1.0]);
    let criterion = make(&[0.0, 0.0]);
    let compile = |body: &pse_math::guarded::PreparedBody, outputs: &[usize]| {
        Arc::new(
            body.compile(
                outputs,
                &[0],
                DerivativeOrder::First,
                pse_math::library::Optimization::default(),
                pse_math::jets::EvaluationLimits::default(),
                &flag,
            )
            .unwrap(),
        )
    };
    let isolation = pse_math::factorable::root_isolation_program_for_outputs(
        row,
        &residual,
        &[0],
        &eligibility,
        &criterion,
        &flag,
        10_000,
    )
    .unwrap()
    .unwrap();
    let key = pse_ids::ContentHash::from_bytes([84; 32]);
    let spec = pse_kernels::ProviderSpec {
        id: row,
        revision: key,
        data: key,
        derivative_source: pse_kernels::DerivativeSource::Implicit,
        shapes: Default::default(),
        inputs: vec![],
        outputs: vec![port],
        derivatives: DerivativeOrder::First,
        smoothness: DerivativeOrder::First,
    };
    let unknowns = vec![Unknown {
        id: coordinate,
        lower: 1.0,
        upper: 4.0,
    }];
    let factory = Arc::new(RegimeFactory {
        spec: spec.clone(),
        alternatives: vec![RegimeFactoryBranch {
            residual: Factory {
                selection: Selection::default(),
                requirements: pse_kernels::DerivativeRequirements::new(
                    DerivativeOrder::First,
                    DerivativeOrder::First,
                    DerivativeOrder::First,
                    DerivativeOrder::First,
                    DerivativeOrder::First,
                )
                .unwrap(),
                spec,
                body: compile(&residual, &[0]),
                unknowns: unknowns.clone(),
                rows: vec![row],
                configuration: Configuration::Fixed(
                    unknowns,
                    Options {
                        start: vec![2.5],
                        variable_nominals: vec![1.0],
                        variable_tolerance: vec![1e-8],
                        residual_tolerance: vec![1e-8],
                        iterations: 100,
                        time_limit: std::time::Duration::from_secs(10),
                        derivative_tolerance: 1e-8,
                    },
                ),
                hints: None,
                terms: None,
                solver: Arc::new(native::implicit::Kinsol),
                cancel: flag.clone(),
                max_entries: 1000,
                providers: Default::default(),
            },
            eligibility: compile(&eligibility, &[0]),
            criterion: compile(&criterion, &[0, 1]),
            isolation: Some(Arc::new(isolation)),
        }],
        maximum_regimes: 1,
        time_limit: std::time::Duration::from_secs(10),
        cancel: flag.clone(),
        verifier: Some(Arc::new(native::root_isolation::Ibex)),
    });
    let owner = service
        .reserve(
            "test:complete-admitted-factory",
            factory.retained_bytes().unwrap(),
        )
        .unwrap();
    let request = DerivedRequest::Reduced {
        suppliers: vec![ReducedSupplier {
            factory,
            factory_owner: owner,
            validity: key,
            realization: SelectedResidualRealization::ZeroResiduals {
                authored_offsets: vec![(row, plan.structure().rows()[0].lower)],
            },
        }],
        retained: vec![],
        accuracy: ReconstructionAccuracy {
            point: 1e-10,
            action: 1e-10,
            class: AccuracyClass::Certified,
            refinement: math::RefinementLimits {
                rounds: 4,
                proof_cells: 64,
            },
        },
    };
    let mut profile = original.profile.clone();
    profile.selection = SolverSelection::Explicit(Backend::Ipopt);
    profile.controls.hessian = HessianMode::LimitedMemory;
    let prepared = service
        .prepare_derived(
            original,
            request,
            profile,
            scope.clone(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let execution = Execution::within(flag, prepared.controls(), scope).unwrap();
    let budget = WorkerBudget::drawing(service.policy.worker_bytes, &service.pool);
    let result = execution::scoped(
        &[execution::adapter(Backend::Kinsol)],
        1,
        service.policy.stack_bytes,
        || {
            let mut retained = Retained::default();
            service.derived_step(&prepared, execution, &mut retained, &budget, &[2.5])
        },
    )
    .unwrap();
    let Outcome::Constant(report) = &result.outcome else {
        panic!("no native outer attempt exists")
    };
    assert!(report.quality.feasible());
    assert_eq!(report.coordinates, vec![(coordinate, 3.0)]);
    assert!(report.work.evaluations.is_none());
    assert!(
        result
            .proposal
            .as_ref()
            .unwrap()
            .reconstruction_accuracy
            .as_ref()
            .unwrap()
            .class
            == AccuracyClass::Certified
    );
    assert_eq!(budget.used(), 0);
}
