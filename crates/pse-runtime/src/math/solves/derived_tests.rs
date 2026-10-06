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
use pse_math::implicit::RegimeFactory;

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
        work_admitted: false,
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
                            evidence: Vec::new(),
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
    let factory = Arc::new(ReconstructionFactory::regimes(
        factory.as_ref().clone(),
        SelectedResidualRealization::ZeroResiduals {
            authored_offsets: vec![(row, plan.structure().rows()[eliminated].lower)],
        },
    ));
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
    let output_producer = auxiliary.clone();
    let output_declaration = declaration.clone();
    prepared.solve = original
        .clone()
        .with_strategy(declaration, vec![auxiliary.into(), original.clone().into()])
        .unwrap();
    let result = package
        .solve_case(prepared, compiler, &cancel)
        .await
        .unwrap();
    let anchor = result.prediction_anchor(0.0);
    assert!(
        anchor.is_ok(),
        "{anchor:?}; diagnostic={:?}",
        result.diagnostic()
    );
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
            .filter(|e| {
                e.kind == pse_model::generated::enums::NumericalEventKind::Finished
                    && e.transition.is_some()
            })
            .count(),
        2
    );
    assert!(
        !scope
            .cancellation()
            .load(std::sync::atomic::Ordering::Acquire)
    );
    assert_eq!(trace.declaration.start.policy, StartPolicy::NoPriorStart);
    assert_eq!(trace.starts[1], StartOrigin::Auxiliary);
    assert!(
        matches!(trace.products[1].provider,Some(strategy::ProviderEvidence::Native(profile)) if profile.key==original.strategy_profile().unwrap()),
        "actual derived correction preserves the original native profile"
    );
    let point_receipt = trace.products[0]
        .evidence
        .iter()
        .find(|receipt| receipt.derivative_order == 0)
        .expect("actual reconstructed point receipt");
    assert!(
        point_receipt
            .accuracy
            .error
            .is_some_and(|error| error > 0.0),
        "actual nonzero certified reconstruction error required by tighter-output control"
    );
    for wrong_point in [false, true] {
        let mut demanded = output_declaration.clone();
        let mut output = PreparedRung::Derived(output_producer.clone())
            .operation_contract()
            .unwrap()
            .outputs
            .into_iter()
            .find(|output| output.derivative_order == 0)
            .unwrap();
        if wrong_point {
            output.source.point = Some(pse_ids::ContentHash::from_bytes([221; 32]));
        } else {
            output.allowance = 0.0;
        }
        demanded.mechanisms[0].operation.outputs.push(output);
        let mut rejected = plain.clone();
        rejected.solve = original
            .clone()
            .with_strategy(
                demanded,
                vec![output_producer.clone().into(), original.clone().into()],
            )
            .unwrap();
        let rejected = package.solve_case(rejected, compiler, &cancel).await;
        let trace = match &rejected {
            Ok(result) => {
                assert!(
                    !result.accepted,
                    "unmet caller output cannot grant original permission"
                );
                result
                    .strategy
                    .as_ref()
                    .expect("failed dispatch retains trace")
            }
            Err(error) => error
                .strategy_trace()
                .expect("typed failure retains actual numerical trace"),
        };
        assert!(trace.events.iter().any(|event| {
            event.mechanism == 0
                && event.work.is_some_and(|work| work.observed.attempts == 1)
                && event
                    .cause
                    .as_deref()
                    .is_some_and(|cause| matches!(cause, ProblemError::Unsupported(_)))
        }));
        assert!(!trace.events.iter().any(|event| event.mechanism == 1
            && event.kind == pse_model::generated::enums::NumericalEventKind::Started));
        assert!(
            trace.products[0]
                .evidence
                .iter()
                .any(|receipt| receipt.derivative_order == 0),
            "valid actual receipt remains available in failure trace"
        );
    }
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
                work_admitted: false,
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
            work_admitted: false,
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
    full_reconstruction_kind(false, false, false).await;
}
#[cfg(all(
    feature = "solver-kinsol",
    feature = "solver-ipopt",
    feature = "solver-root-isolation"
))]
#[tokio::test]
async fn actual_single_root_full_reconstruction_preserves_native_factory_and_original_assessment() {
    full_reconstruction_kind(true, true, false).await;
}
#[cfg(all(
    feature = "solver-kinsol",
    feature = "solver-ipopt",
    feature = "solver-root-isolation"
))]
#[tokio::test]
async fn actual_single_root_certifies_authored_observable_over_complete_coordinate_box() {
    full_reconstruction_kind(true, true, true).await;
}
#[cfg(all(
    feature = "solver-kinsol",
    feature = "solver-ipopt",
    feature = "solver-root-isolation"
))]
async fn full_reconstruction_kind(single_root: bool, with_goal: bool, observable: bool) {
    use pse_math::implicit::reconstruction::SelectedResidualRealization;
    use pse_math::{
        implicit::{Configuration, Factory, Options, RegimeFactoryBranch, Selection, Unknown},
        typed::{Binary, BodyBuilder, BodyLimits},
    };
    use pse_model::strategy::AccuracyClass;
    // Regimes retain their existing covering contract. Genuine Root reconstruction
    // admits only its requested 64-cell proof workspace, within the ordinary small pool.
    let runtime = crate::workflow::tests::runtime_on(
        if single_root {
            192usize << 20
        } else {
            24usize << 30
        },
        crate::math::MathPolicy {
            worker_bytes: if single_root {
                64usize << 20
            } else {
                8usize << 30
            },
            workspace_bytes: if single_root {
                64usize << 20
            } else {
                8usize << 30
            },
            foreign_bytes: 16usize << 20,
            ..Default::default()
        },
    );
    let text = if observable {
        "package p { def Root { var x:Scalar; eq balance:x==3; annotation bounds x(1,4); annotation start x(2.5); let q:Scalar=x*x/3; annotation report q(\"computed q\"); } }"
    } else {
        "package p { def Root { var x:Scalar; eq balance:x==3; annotation bounds x(1,4); annotation start x(2.5); } }"
    };
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
    let physical = crate::workflow::tests::physical();
    let quantities = physical.quantities.clone();
    let package = runtime.modeling_package(rows, physical).unwrap();
    let solver = SolverProfile {
        intent: SolveIntent::Initialize,
        selection: SolverSelection::Explicit(Backend::Ipopt),
        controls: Controls {
            hessian: HessianMode::LimitedMemory,
            ..Default::default()
        },
        ..Default::default()
    };
    let cancel = crate::CancelSource::new();
    let initial = package
        .prepare_solve(
            root,
            pse_modeling::specialize::root_instance(root),
            pse_modeling::Bindings::default(),
            pse_modeling::Limits::default(),
            pse_compiler::workspace::ModelingCaseBindings::default(),
            DerivativeOrder::First,
            crate::workflow::tests::compiler_profile(),
            solver.clone(),
            NumericalInputs::default(),
            &cancel,
        )
        .await
        .unwrap();
    let mut original = initial.solve.clone();
    if with_goal {
        use pse_model::generated::enums::*;
        let variable = original
            .numerics
            .targets
            .iter()
            .find(|target| target.kind == NumericalTarget::Variable)
            .unwrap();
        let (target_id, target_kind, quantity_id, unit_id) = if observable {
            let source = initial.model.model.compiled();
            let target = source
                .model
                .symbols
                .iter()
                .find(|(_, symbol)| symbol.lineage.path.ends_with(".q"))
                .map(|(id, _)| *id)
                .unwrap();
            let row = source
                .admitted
                .case()
                .rows()
                .iter()
                .find(|row| {
                    row.id == pse_compiler::workspace::ModelingOutput::Member(target).row_id()
                })
                .unwrap();
            (
                target,
                NumericalTarget::Observable,
                row.quantity.as_id(),
                quantities
                    .quantity_type(row.quantity)
                    .unwrap()
                    .canonical_unit
                    .as_id(),
            )
        } else {
            (variable.id, variable.kind, variable.quantity, variable.unit)
        };
        let goal = pse_model::engineering_accuracy::AccuracyGoal {
            goal_id: SemanticId::from_bytes([85; 16]).into(),
            model_id: None,
            case_id: None,
            instance_id: None,
            fit_id: None,
            target_id,
            target_kind,
            quantity_id,
            unit_id,
            subject: AccuracyGoalSubject::SelectedOutput,
            observation: AccuracyObservation::Steady,
            time: None,
            resolution: Some(0.1),
            criterion_lower: Some(2.0),
            criterion_upper: Some(4.0),
            required_class: AccuracyClass::Certified,
            use_policy: AccuracyGoalUse::Assess,
            refine: false,
            source: NumericalSource::Analysis,
            priority: 0,
            provenance: "actual complete selected-root certificate".into(),
        };
        original.profile.numerics.goals.push(goal.clone());
        Arc::make_mut(&mut original.numerics)
            .policy
            .goals
            .push(goal);
        if observable {
            original = package
                .prepare_solve(
                    root,
                    pse_modeling::specialize::root_instance(root),
                    pse_modeling::Bindings::default(),
                    pse_modeling::Limits::default(),
                    pse_compiler::workspace::ModelingCaseBindings::default(),
                    DerivativeOrder::First,
                    crate::workflow::tests::compiler_profile(),
                    original.profile.clone(),
                    NumericalInputs::default(),
                    &cancel,
                )
                .await
                .unwrap()
                .solve;
            assert!(original.selected_output_program().is_some());
        }
    }
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
    let realization = SelectedResidualRealization::ZeroResiduals {
        authored_offsets: vec![(row, plan.structure().rows()[0].lower)],
    };
    let factory = if single_root {
        let root = factory.alternatives[0].residual.clone();
        let proof =
            pse_math::factorable::single_root_isolation_program(row, &residual, &flag, 10_000)
                .unwrap()
                .unwrap();
        let proof = Arc::new(proof);
        let alternatives = [pse_math::implicit::SelectionAlternative {
            id: root.spec.id,
            program: &proof,
            residual_identity: root.spec.revision,
            unknowns: &root.unknowns,
        }];
        let request = pse_math::implicit::SelectionProofRequest {
            selection: root.spec.id,
            alternatives: &alternatives,
            winner: 0,
            parameters: &[],
            candidate: &[3.0],
            order: DerivativeOrder::First,
            time_limit: std::time::Duration::from_secs(10),
            cancel: &flag,
        };
        let evidence = pse_math::implicit::SelectionVerifier::certify_bounded(
            &native::root_isolation::Ibex,
            &request,
            64,
        )
        .unwrap();
        assert!(
            matches!(evidence, pse_math::implicit::SelectionEvidence::Unique(_)),
            "actual single-root initial evidence: {evidence:?}; source: {proof:?}"
        );
        let source = ReconstructionFactory::new(
            pse_math::implicit::ImplicitFactory::Root(root),
            realization,
            Some(proof),
            Some(Arc::new(native::root_isolation::Ibex)),
        );
        assert!(matches!(
            source.factory(),
            pse_math::implicit::ImplicitFactory::Root(_)
        ));
        Arc::new(source)
    } else {
        Arc::new(ReconstructionFactory::regimes(
            factory.as_ref().clone(),
            realization,
        ))
    };
    let accuracy = ReconstructionAccuracy {
        point: 1e-10,
        action: 1e-10,
        class: AccuracyClass::Certified,
        refinement: math::RefinementLimits {
            rounds: 4,
            proof_cells: 64,
        },
    };
    let descriptor = pse_kernels::AdmittedProvider::new(factory.spec().clone(), registry).unwrap();
    let mut discovered_original = original.clone();
    let Representation::Algebraic(discovery) = &mut discovered_original.representation else {
        panic!("original algebraic source")
    };
    let raw =
        pse_kernels::Registration::bind(descriptor.clone(), Arc::new(factory.factory().clone()))
            .unwrap();
    discovery.providers.insert(factory.spec().key(), raw);
    assert!(
        service
            .automatic_reduced_request(&discovered_original, accuracy)
            .unwrap()
            .is_none(),
        "concrete source alone is not producer metadata"
    );
    let Representation::Algebraic(discovery) = &mut discovered_original.representation else {
        panic!("original algebraic source")
    };
    let issued = pse_kernels::Registration::bind(descriptor, factory.clone()).unwrap();
    discovery.providers.insert(factory.spec().key(), issued);
    let discovered = service
        .automatic_reduced_request(&discovered_original, accuracy)
        .unwrap()
        .unwrap();
    let DerivedRequest::Reduced {
        suppliers,
        retained,
        ..
    } = discovered
    else {
        panic!("automatic reduced request")
    };
    assert!(retained.is_empty());
    assert_eq!(suppliers.len(), 1);
    assert_eq!(
        matches!(
            suppliers[0].factory.factory(),
            pse_math::implicit::ImplicitFactory::Root(_)
        ),
        single_root
    );
    drop(suppliers);
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
    profile.selection = SolverSelection::Explicit(Backend::Kinsol);
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
            if single_root {
                let DerivedRequest::Reduced { suppliers, .. } = &prepared.request else {
                    panic!("actual reduced source required");
                };
                let local = &prepared.reduced.as_ref().unwrap().suppliers[0];
                let _charge = budget.charge(
                    suppliers[0].factory.retained_bytes()?
                        + suppliers[0].factory.reconstruction_workspace_bytes(1)?,
                )?;
                let mut limited = SelectedImplicitReconstruction::<ProblemError>::new_with_binding(
                    suppliers[0].factory.as_ref(),
                    local.contract.clone(),
                    local.normalization.clone(),
                    execution.scope()?,
                    &local.binding,
                )?
                .with_proof_cell_limit(1)?;
                let refusal = math::ReconstructionOracle::admit(&mut limited, &[]).unwrap_err();
                assert!(
                    matches!(
                        refusal,
                        ProblemError::Math(pse_math::MathError::Refinement {
                            reason: math::RefinementRefusal::Unavailable(
                                pse_math::implicit::SelectionProofRefusal::Resource
                            ),
                            ..
                        })
                    ),
                    "actual initial proof budget refusal: {refusal:?}"
                );
            }
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
    if with_goal {
        let receipt = report.certified_reconstruction.as_ref().unwrap();
        let mut values = prepared.source.values.clone();
        values.scalars.extend(report.coordinates.iter().copied());
        let scope = ExecutionScope::new(Arc::default(), None);
        let goals = prepared
            .original
            .coordinate_accuracy(service, &result.outcome, &values, &scope, &budget)
            .unwrap();
        assert_eq!(goals.len(), 1);
        assert_eq!(
            goals[0].classification.status,
            pse_model::generated::enums::AccuracyGoalStatus::Satisfied
        );
        let evidence = goals[0].evidence.as_ref().unwrap();
        assert_eq!(evidence.accuracy.class, AccuracyClass::Certified);
        assert_eq!(
            evidence.method,
            pse_model::generated::enums::AccuracyEvidenceMethod::CertifiedEnclosure
        );
        assert!(evidence.source.branch.is_some());
        assert!(evidence.interval.unwrap().0 <= 3.0 && evidence.interval.unwrap().1 >= 3.0);
        assert_eq!(
            evidence.target_kind,
            if observable {
                pse_model::generated::enums::NumericalTarget::Observable
            } else {
                pse_model::generated::enums::NumericalTarget::Variable
            }
        );
        assert_eq!(evidence.value, Some(3.0));
        assert!(evidence.accuracy.error.unwrap() <= 0.1);
        assert_eq!(
            receipt
                .coordinate_box(&prepared.original, &values)
                .unwrap()
                .unwrap()
                .len(),
            1
        );
        values.scalars.insert(coordinate, 3.0_f64.next_up());
        assert!(
            receipt
                .assess_coordinates(&prepared.original, &values)
                .unwrap()
                .is_none()
        );
        let mut changed = prepared.original.clone();
        changed.normalization.variables[0] *= 2.0;
        values.scalars.insert(coordinate, 3.0);
        assert!(receipt.coordinate_box(&changed, &values).unwrap().is_none());
    } else {
        assert!(report.certified_reconstruction.is_none());
    }
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

#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn reference_flash_completed_blocks_discharge_reporting_allowance() {
    use crate::authoring_driver::document::{OwnedDocumentSet, load_package_documents_owned};
    use crate::workflow::tests as fixture;
    use std::{collections::BTreeMap, path::Path};
    fn documents(root: &Path, at: &Path, values: &mut BTreeMap<String, Vec<u8>>) {
        for entry in std::fs::read_dir(at).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                documents(root, &path, values);
            } else if matches!(
                path.extension().and_then(|value| value.to_str()),
                Some("toml" | "yaml" | "yml" | "pse" | "parquet")
            ) {
                values.insert(
                    path.strip_prefix(root)
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_owned(),
                    std::fs::read(&path).unwrap(),
                );
            }
        }
    }
    let runtime = fixture::runtime_on(64usize << 30, Default::default());
    let pool = runtime.shared.pool();
    let reference = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/reference");
    let token = pse_columnar::CancellationToken::new();
    let load = |name: &str| {
        let root = reference.join(name);
        let mut values = BTreeMap::new();
        documents(&root, &root, &mut values);
        load_package_documents_owned(
            &values,
            &runtime.registry,
            Default::default(),
            &pool,
            &token,
            &runtime.validation_context().unwrap(),
        )
        .unwrap()
    };
    let physical_documents =
        OwnedDocumentSet::try_from_bundles(vec![load("physical")], &pool, &token).unwrap();
    let physical = runtime
        .physical_from_documents(&physical_documents, &token)
        .await
        .unwrap();
    let packages = [
        "data/oracles/teqp-0.23.1",
        "data/oracles/feos-0.10.1",
        "data/gross-sadowski-2001",
        "data/references",
        "data/nist",
        "data/perry7",
        "data/poling2000",
        "data/oracles/idaes-2.13",
        "data/species",
        "data/ciaaw",
        "seed-data",
        "campaign",
        "process",
        "thermodynamics",
        "methods",
        "domain",
        "physical",
    ];
    let sources =
        OwnedDocumentSet::try_from_bundles(packages.into_iter().map(load).collect(), &pool, &token)
            .unwrap();
    let package = runtime.modeling_from_documents(&sources, physical).unwrap();
    let root = package
        .declarations()
        .iter()
        .find(|row| row.name == "measurement_value_sweep")
        .unwrap()
        .declaration_id;
    let cancel = crate::CancelSource::new();
    let mut declared = package
        .declared_execution(
            root,
            Default::default(),
            SolverProfile {
                intent: SolveIntent::FeasiblePoint,
                selection: SolverSelection::Explicit(Backend::Ipopt),
                ..Default::default()
            },
            Default::default(),
            Default::default(),
            &cancel,
        )
        .await
        .unwrap();
    declared
        .analysis
        .case
        .values
        .insert("root.inlet.T".into(), 360.);
    let prepared = package.prepare_declared(&declared, &cancel).await.unwrap();
    let service = runtime.native();
    let scope = ExecutionScope::new(
        Arc::new(std::sync::atomic::AtomicBool::new(false)),
        Some(Instant::now() + prepared.solve.controls().time_limit),
    );
    let blocks = service
        .prepare_blocks(prepared.solve.clone(), scope, &cancel)
        .await
        .unwrap();
    let admitted = blocks.result_bytes().unwrap();
    let completed = package
        .solve_case(prepared, declared.analysis.compiler, &cancel)
        .await
        .unwrap();
    assert!(completed.accepted);
    let Outcome::Constant(report) = &completed.outcome else {
        panic!("reference flash must complete its block schedule");
    };
    let retained = report.owner.as_ref().unwrap().size();
    let known = report
        .component_reports()
        .filter(|report| report.completed_report_allowance().unwrap().is_some())
        .count();
    println!(
        "reference flash: blocks={}, known={}, admitted={}, retained={}",
        blocks.block_count(),
        known,
        admitted,
        retained
    );
    assert_eq!(report.component_reports().len(), blocks.block_count());
    assert!(known > 0);
    assert_eq!(retained, blocks.completed_result_bytes(report).unwrap());
    assert!(retained < admitted / 2);
    let tables = completed.tables().unwrap();
    let retained_sources = pool.reserved();
    drop(completed);
    drop(blocks);
    drop(declared);
    drop(package);
    drop(sources);
    drop(physical_documents);
    drop(runtime);
    assert!(pool.reserved() > 0);
    assert!(pool.reserved() < retained_sources);
    assert!(!tables.is_empty());
    drop(tables);
    assert_eq!(pool.reserved(), 0);
}

#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn repeated_completed_blocks_fit_short_pool_and_extracted_components_hold_grant() {
    use crate::workflow::tests as fixture;
    let runtime = fixture::runtime_with(16 << 20, 1 << 20, 128 << 20);
    let (runtime, original) = original_order_on(runtime,
        "package p { def Root { var x:Scalar; var y:Scalar; var z:Scalar; annotation start x(0); annotation start y(0); annotation start z(0); eq first:x==1; eq second:y==x+1; eq third:z==2*y; } }",
        DerivativeOrder::First, fixture::profile()).await;
    let pool = runtime.shared.pool();
    let service = runtime.native();
    let flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let scope = ExecutionScope::new(
        flag.clone(),
        Some(Instant::now() + std::time::Duration::from_secs(120)),
    );
    let prepared = service
        .prepare_blocks(original.clone(), scope.clone(), &crate::CancelSource::new())
        .await
        .unwrap();
    let admitted = prepared.result_bytes().unwrap();
    let budget = WorkerBudget::drawing(service.policy.worker_bytes, &pool);
    let execute = || {
        execution::scoped(
            &[execution::adapter(Backend::Kinsol)],
            1,
            service.policy.stack_bytes,
            || {
                let execution =
                    Execution::within(flag.clone(), original.controls(), scope.clone()).unwrap();
                let Outcome::Constant(report) = service
                    .blocks_step(
                        &prepared,
                        execution,
                        &mut Retained::default(),
                        &budget,
                        &[0., 0., 0.],
                    )
                    .unwrap_or_else(|failure| panic!("{}", failure.cause))
                else {
                    panic!("complete original evaluation required");
                };
                Ok::<_, ProblemError>(report)
            },
        )
        .unwrap()
    };
    drop(execute());
    let baseline = pool.reserved();
    let mut completed = Vec::new();
    for _ in 0..20 {
        completed.push(execute());
    }
    assert!(pool.reserved() < baseline + admitted * 2);
    let mut report = completed.pop().unwrap();
    let weak = Arc::downgrade(report.owner.as_ref().unwrap());
    let cloned = report.clone();
    let extracted = report.components.pop().unwrap();
    drop(report);
    drop(completed);
    assert!(weak.upgrade().is_some());
    drop(cloned);
    assert!(
        weak.upgrade().is_some(),
        "extracted native report keeps aggregate capacity"
    );
    std::thread::spawn(move || drop(extracted)).join().unwrap();
    assert!(weak.upgrade().is_none());
    assert_eq!(pool.reserved(), baseline);
    drop(prepared);
    drop(original);
    drop(budget);
    drop(runtime);
    assert_eq!(pool.reserved(), 0);
}

#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn completed_blocks_refuse_excess_capacity_and_keep_unknown_extensions_bounded() {
    let (runtime, original) = original("package p { def Root { var x:Scalar; var y:Scalar; annotation start x(0); annotation start y(0); eq first:x==1; eq second:y==x+1; } }").await;
    let service = runtime.native();
    let flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let scope = ExecutionScope::new(
        flag.clone(),
        Some(Instant::now() + std::time::Duration::from_secs(30)),
    );
    let prepared = service
        .prepare_blocks(original.clone(), scope.clone(), &crate::CancelSource::new())
        .await
        .unwrap();
    let budget = WorkerBudget::drawing(service.policy.worker_bytes, &service.pool);
    let mut report = execution::scoped(
        &[execution::adapter(Backend::Kinsol)],
        1,
        service.policy.stack_bytes,
        || {
            let execution = Execution::within(flag, original.controls(), scope).unwrap();
            let Outcome::Constant(report) = service
                .blocks_step(
                    &prepared,
                    execution,
                    &mut Retained::default(),
                    &budget,
                    &[0., 0.],
                )
                .unwrap_or_else(|failure| panic!("{}", failure.cause))
            else {
                panic!("complete original evaluation required");
            };
            Ok::<_, ProblemError>(report)
        },
    )
    .unwrap();
    let visible = prepared.completed_result_bytes(&report).unwrap();
    // A bulky unmeasured extension preserves its full pre-admitted envelope. It is
    // unavailable, never interpreted as zero bytes or a discharged analysis owner.
    report.components[0].evidence.root_response =
        Some(Err(native::square_response::Withheld::Memory));
    assert!(prepared.completed_result_bytes(&report).unwrap() > visible);
    assert!(prepared.completed_result_bytes(&report).unwrap() <= prepared.result_bytes().unwrap());
    report.components[0].evidence.root_response = None;
    // Pure sizing refusal before publication: even an empty string may retain a large
    // allocation. Length-based counting would incorrectly accept this envelope.
    let excess = String::with_capacity(prepared.result_bytes().unwrap());
    report.components[0]
        .metrics
        .insert("oversized".into(), Metric::Text(excess));
    let error = prepared
        .completed_result_bytes(&report)
        .unwrap_err()
        .into_problem();
    assert!(matches!(
        error,
        ProblemError::Limit {
            kind: native::LimitKind::Memory,
            ..
        }
    ));
}

#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn failed_blocks_release_construction_allowance_and_preserve_original_cause() {
    let (runtime, original) = original("package p { def Root { var x:Scalar; var y:Scalar; annotation start x(0); annotation start y(0); eq first:x==1; eq second:y==log(x-2); } }").await;
    let service = runtime.native();
    let pool = runtime.shared.pool();
    let flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let scope = ExecutionScope::new(
        flag.clone(),
        Some(Instant::now() + std::time::Duration::from_secs(30)),
    );
    let prepared = service
        .prepare_blocks(original.clone(), scope.clone(), &crate::CancelSource::new())
        .await
        .unwrap();
    let budget = WorkerBudget::drawing(service.policy.worker_bytes, &pool);
    let execute = || {
        execution::scoped(
            &[execution::adapter(Backend::Kinsol)],
            1,
            service.policy.stack_bytes,
            || {
                let execution =
                    Execution::within(flag.clone(), original.controls(), scope.clone()).unwrap();
                let failure = service
                    .blocks_step(
                        &prepared,
                        execution,
                        &mut Retained::default(),
                        &budget,
                        &[0., 0.],
                    )
                    .unwrap_err();
                Ok::<_, ProblemError>(failure)
            },
        )
        .unwrap()
    };
    let first = execute();
    let cause = first.cause.clone();
    assert!(
        matches!(&*cause, ProblemError::Math(_)),
        "original mathematical evaluation cause required: {cause:?}"
    );
    assert!(Arc::ptr_eq(&first.cause, &cause));
    let baseline = pool.reserved();
    for _ in 0..3 {
        let failed = execute();
        assert_eq!(pool.reserved(), baseline);
        drop(failed);
    }
    drop(first);
    drop(cause);
    drop(prepared);
    drop(original);
    drop(budget);
    drop(runtime);
    assert_eq!(pool.reserved(), 0);
}

#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn automatic_blocks_execute_complete_nonport_coupled_original_and_keep_actual_components() {
    let (runtime, original) = original("package p { def Root { var x:Scalar; var y:Scalar; var z:Scalar; annotation start x(0); annotation start y(0); annotation start z(0); eq first:x==1; eq second:y==x+1; eq third:z==2*y; } }").await;
    let service = runtime.native();
    let flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    assert_eq!(original.automatic_block_count(&flag).unwrap(), 3);
    let offers = original.automatic_operations(None, &flag).unwrap();
    assert!(
        offers
            .iter()
            .any(|offer| offer.candidate.kind == MechanismKind::Block)
    );
    let scope = ExecutionScope::new(
        flag.clone(),
        Some(Instant::now() + std::time::Duration::from_secs(30)),
    );
    let prepared = service
        .prepare_blocks(original.clone(), scope.clone(), &crate::CancelSource::new())
        .await
        .unwrap();
    assert_eq!(prepared.block_count(), 3);
    assert_eq!(
        prepared.original().original_identity().unwrap(),
        original.original_identity().unwrap()
    );
    let execution = Execution::within(flag, &original.profile.controls, scope).unwrap();
    let budget = WorkerBudget::drawing(service.policy.worker_bytes, &service.pool);
    execution::scoped(
        &[execution::adapter(Backend::Kinsol)],
        1,
        service.policy.stack_bytes,
        || {
            let outcome = service
                .blocks_step(
                    &prepared,
                    execution.clone(),
                    &mut Retained::default(),
                    &budget,
                    &[0., 0., 0.],
                )
                .unwrap_or_else(|failure| panic!("{}", failure.cause));
            let Outcome::Constant(report) = outcome else {
                panic!("fresh complete original report required");
            };
            assert!(report.quality.feasible());
            assert_eq!(report.coordinates.len(), 3);
            let mut coordinates: Vec<_> =
                report.coordinates.iter().map(|(_, value)| *value).collect();
            coordinates.sort_by(f64::total_cmp);
            assert_eq!(coordinates, [1., 2., 4.]);
            assert_eq!(report.component_reports().len(), 3);
            let expected = report.component_reports().fold(1u64, |sum, component| {
                sum + component.evidence.work.evaluations.unwrap()
            });
            assert_eq!(report.work.evaluations, Some(expected));
            for component in report.component_reports() {
                assert_eq!(component.variables.len(), 1);
                assert!(
                    crate::workflow::numerics::native_use(component, &original.numerics.policy)
                        .permits_use()
                );
            }
            assert_eq!(report.observation.values.len(), 3);
            assert!(
                report
                    .observation
                    .equality_residuals
                    .iter()
                    .all(|residual| residual.is_some_and(|residual| residual.abs() < 1e-8))
            );
            Ok::<_, ProblemError>(())
        },
    )
    .unwrap();
    let completed = service
        .solve(original.clone())
        .unwrap()
        .finish()
        .await
        .unwrap();
    assert!(
        completed
            .outcome
            .candidate_use(&original.numerics.policy)
            .permits_use()
    );
    let Outcome::Constant(report) = &completed.outcome else {
        panic!("automatic block result must retain fresh original evaluation");
    };
    assert_eq!(report.component_reports().len(), 3);
    assert!(
        completed
            .strategy
            .declaration
            .mechanisms
            .iter()
            .any(|mechanism| mechanism.kind == MechanismKind::Block)
    );
    let mut explicit = original.clone();
    explicit.profile.controls.start = StartPolicy::Explicit;
    assert_eq!(
        explicit
            .automatic_block_count(&Arc::new(std::sync::atomic::AtomicBool::new(false)))
            .unwrap(),
        0
    );
}

#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn automatic_blocks_merge_domain_control_cycle_and_refuse_single_block() {
    // Cancellation of the arithmetic value leaves the log-domain dependency owned by
    // original evaluation. It closes a cycle with the second equation.
    let (_runtime, original) = original("package p { def Root { var x:Scalar; var y:Scalar; annotation start x(1); annotation start y(2); eq first:x+log(y)-log(y)==1; eq second:y==x+1; } }").await;
    let flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    assert_eq!(original.automatic_block_count(&flag).unwrap(), 0);
    assert!(
        !original
            .automatic_operations(None, &flag)
            .unwrap()
            .iter()
            .any(|offer| offer.candidate.kind == MechanismKind::Block)
    );
}

#[cfg(all(feature = "solver-kinsol", feature = "solver-ipopt"))]
#[tokio::test]
async fn automatic_blocks_refuse_original_objective_coupling() {
    let runtime = crate::workflow::tests::runtime_with(128 << 20, 1 << 20, 512 << 20);
    let (_runtime, original) = original_order_on(runtime, "package p { def Root { var x:Scalar; var y:Scalar; annotation start x(1); annotation start y(2); eq first:x==1; eq second:y==x+1; let cost:Scalar=x*y; annotation objective cost(minimize); } }", DerivativeOrder::Second, SolverProfile { intent:SolveIntent::Optimize, selection:SolverSelection::Explicit(Backend::Ipopt), ..Default::default() }).await;
    assert_eq!(
        original
            .automatic_block_count(&Arc::new(std::sync::atomic::AtomicBool::new(false)))
            .unwrap(),
        0
    );
}

#[cfg(all(feature = "solver-kinsol", feature = "solver-ipopt"))]
#[tokio::test]
async fn actual_zero_row_feasibility_retains_objective_and_objective_free_original_bounds() {
    for with_objective in [true, false] {
        let runtime = crate::workflow::tests::runtime_with(128 << 20, 1 << 20, 512 << 20);
        let text = if with_objective {
            "package p { def Root { var x:Scalar; annotation bounds x(1,5); annotation start x(4); let cost:Scalar=(x-2)*(x-2); annotation objective cost(minimize); } }"
        } else {
            "package p { def Root { var x:Scalar; annotation bounds x(1,5); annotation start x(4); } }"
        };
        let (runtime, original) = original_order_on(
            runtime,
            text,
            DerivativeOrder::Second,
            SolverProfile {
                intent: if with_objective {
                    SolveIntent::Optimize
                } else {
                    SolveIntent::Initialize
                },
                selection: SolverSelection::Explicit(Backend::Ipopt),
                controls: Controls {
                    hessian: HessianMode::LimitedMemory,
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .await;
        let service = runtime.native();
        let flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let scope = ExecutionScope::new(
            flag.clone(),
            Some(Instant::now() + std::time::Duration::from_secs(30)),
        );
        let mut profile = original.profile.clone();
        profile.intent = SolveIntent::Initialize;
        let prepared = service
            .prepare_derived(
                original.clone(),
                DerivedRequest::Feasibility,
                profile,
                scope.clone(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        assert!(prepared.physical.constraints().is_empty());
        assert_eq!(
            prepared.physical.obligations().objective.is_some(),
            with_objective
        );
        assert_eq!(prepared.physical.coordinates()[0].lower, 1.);
        assert_eq!(prepared.physical.coordinates()[0].upper, 5.);
        let execution = Execution::within(flag, prepared.controls(), scope).unwrap();
        let budget = WorkerBudget::drawing(service.policy.worker_bytes, &service.pool);
        execution::scoped(
            &[execution::adapter(Backend::Ipopt)],
            1,
            service.policy.stack_bytes,
            || {
                let attempt = service
                    .derived_step(
                        &prepared,
                        execution.clone(),
                        &mut Retained::default(),
                        &budget,
                        &[4.],
                    )
                    .map_err(|failure| failure.cause)?;
                let Outcome::Native(auxiliary) = &attempt.outcome else {
                    panic!("actual zero-row native NLP required");
                };
                assert_eq!(auxiliary.backend, Backend::Ipopt);
                assert!(auxiliary.quality.as_ref().unwrap().feasible());
                let checked = service
                    .derived_original_outcome(&prepared, &attempt, &execution, &budget)
                    .unwrap();
                let Outcome::Native(checked) = checked else {
                    panic!("actual original assessment required");
                };
                assert!(checked.quality.as_ref().unwrap().feasible());
                assert!(checked.observation.as_ref().unwrap().values.is_empty());
                assert_eq!(
                    checked.observation.as_ref().unwrap().objective.is_some(),
                    with_objective
                );
                assert!((1. ..=5.).contains(&checked.candidate.as_ref().unwrap().primal[0]));
                if with_objective {
                    // Feasibility supplies an original-screened point; its constant auxiliary
                    // objective does not establish original minimization stationarity.
                    let x = checked.candidate.as_ref().unwrap().primal[0];
                    assert!(
                        (checked.observation.as_ref().unwrap().objective.unwrap()
                            - (x - 2.).powi(2))
                        .abs()
                            < 1e-8
                    );
                    assert!(checked.evidence.local.is_none());
                    assert!(checked.evidence.kkt.is_none());
                }
                assert_eq!(checked.qualification, Qualification::Feasible);
                assert_eq!(checked.termination.assurance, Assurance::Feasible);
                assert!(
                    crate::workflow::numerics::native_use(&checked, &original.numerics.policy)
                        .permits_use()
                );
                Ok::<_, Arc<ProblemError>>(())
            },
        )
        .unwrap();
    }
}

#[cfg(all(
    feature = "solver-kinsol",
    feature = "solver-ipopt",
    feature = "solver-root-isolation"
))]
#[tokio::test]
async fn actual_multiple_root_suppliers_consume_nonzero_authored_offsets_and_chain_actions() {
    use pse_math::{
        composite_reconstruction::CompositeReconstruction,
        derived::ReconstructionOracle,
        implicit::reconstruction::{SelectedResidualBinding, SelectedResidualRealization},
        implicit::{Configuration, Factory, ImplicitFactory, Options, Selection, Unknown},
        normalization::Normalization,
        typed::{Binary, BodyBuilder, BodyLimits},
    };
    use pse_model::strategy::{AccuracyClass, AccuracyDemand};
    let runtime = crate::workflow::tests::runtime_on(
        160usize << 20,
        crate::math::MathPolicy {
            worker_bytes: 64usize << 20,
            workspace_bytes: 64usize << 20,
            foreign_bytes: 16usize << 20,
            ..Default::default()
        },
    );
    let (runtime, original) = original_order_on(runtime, "package p { def Root { var y:Scalar; var z:Scalar; var p:Scalar; annotation bounds y(4,6); annotation bounds z(6,8); annotation bounds p(1,2); annotation start y(5); annotation start z(7); annotation start p(2); eq first:y-p==3; eq second:z-y==2; } }", DerivativeOrder::First, SolverProfile { intent:SolveIntent::Initialize, selection:SolverSelection::Explicit(Backend::Ipopt), controls:Controls { hessian:HessianMode::LimitedMemory, ..Default::default() }, ..Default::default() }).await;
    let Representation::Algebraic(source) = &original.representation else {
        panic!("compiled original source required");
    };
    let plan = &source.prepared.prepared.plan;
    let registry = &source.prepared.prepared.quantities;
    let port = |lower| {
        plan.structure()
            .variables()
            .iter()
            .find(|v| v.lower == Some(lower))
            .unwrap()
            .port
            .clone()
    };
    let y = port(4.);
    let z = port(6.);
    let p = port(1.);
    let coordinates = vec![
        math::Coordinate {
            id: y.id,
            lower: 4.,
            upper: 6.,
        },
        math::Coordinate {
            id: z.id,
            lower: 6.,
            upper: 8.,
        },
        math::Coordinate {
            id: p.id,
            lower: 1.,
            upper: 2.,
        },
    ];
    let flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let scope = ExecutionScope::new(
        flag.clone(),
        Some(Instant::now() + std::time::Duration::from_secs(30)),
    );
    let key = pse_ids::ContentHash::from_bytes([92; 32]);
    let scales = Normalization::identity(3, 2);
    let first_rows = plan.incidence(&flag).unwrap();
    let row_for = |parameter| {
        plan.structure()
            .instances()
            .iter()
            .zip(&first_rows)
            .find_map(|(instance, support)| {
                instance.contributions.iter().find_map(|contribution| {
                    let pse_math::binding::Target::Row(row) = contribution.target else {
                        return None;
                    };
                    support
                        .first_for_output(contribution.output)
                        .unwrap()
                        .iter()
                        .any(|column| instance.slots[*column].source() == parameter)
                        .then_some(row)
                })
            })
            .unwrap()
    };
    let rows = [row_for(p.id), row_for(z.id)];
    assert_eq!(rows.len(), 2);
    assert_ne!(rows[0], rows[1]);
    // These exact typed source outputs are authored values. Their physical row
    // bounds remain 3 and 2; the producer binds each offset to native and proof DAGs.
    let global = Arc::new(
        OriginalContract::new(
            key,
            scales.key(),
            coordinates.clone(),
            vec![
                math::Constraint {
                    id: rows[0],
                    lower: 3.,
                    upper: 3.,
                },
                math::Constraint {
                    id: rows[1],
                    lower: 2.,
                    upper: 2.,
                },
            ],
            vec![
                Entry::new(GlobalRow::new(0), GlobalCol::new(0)),
                Entry::new(GlobalRow::new(0), GlobalCol::new(2)),
                Entry::new(GlobalRow::new(1), GlobalCol::new(1)),
                Entry::new(GlobalRow::new(1), GlobalCol::new(0)),
            ],
            DerivativeSupport {
                order: DerivativeOrder::First,
                jacobian_product: true,
                source: key,
            },
            OriginalObligations {
                guards: key,
                selection: key,
                objective: None,
            },
        )
        .unwrap(),
    );
    let mut factories = Vec::new();
    let mut contracts = Vec::new();
    let mut bindings = Vec::new();
    let mut local_scales = Vec::new();
    for (i, output, input, output_col, input_col, offset) in [
        (0, y.clone(), p.clone(), 0, 2, 3.),
        (1, z.clone(), y.clone(), 1, 0, 2.),
    ] {
        let row = rows[i];
        let mut builder = BodyBuilder::new(
            pse_math::initialize().unwrap(),
            registry,
            &pse_quantity::standard::StandardInvariantChecker,
            2,
            BodyLimits::default(),
        )
        .unwrap();
        let unknown = builder
            .input(0, output.quantity, pse_quantity::IndexSet::new(), output.id)
            .unwrap();
        let parameter = builder
            .input(1, input.quantity, pse_quantity::IndexSet::new(), input.id)
            .unwrap();
        let expression = builder
            .binary(Binary::Sub, unknown, parameter, None, row)
            .unwrap();
        let body = builder.prepare(&[expression]).unwrap();
        let compiled = Arc::new(
            body.compile(
                &[0],
                &[0, 1],
                DerivativeOrder::First,
                pse_math::library::Optimization::default(),
                pse_math::jets::EvaluationLimits::default(),
                &flag,
            )
            .unwrap(),
        );
        let proof = Arc::new(
            pse_math::factorable::single_root_isolation_program(row, &body, &flag, 10_000)
                .unwrap()
                .unwrap(),
        );
        let coordinate = &coordinates[output_col];
        let unknowns = vec![Unknown {
            id: output.id,
            lower: coordinate.lower,
            upper: coordinate.upper,
        }];
        let factory = ReconstructionFactory::new(
            ImplicitFactory::Root(Factory {
                spec: pse_kernels::ProviderSpec {
                    id: row,
                    revision: key,
                    data: key,
                    derivative_source: pse_kernels::DerivativeSource::Implicit,
                    shapes: Default::default(),
                    inputs: vec![input],
                    outputs: vec![output],
                    derivatives: DerivativeOrder::First,
                    smoothness: DerivativeOrder::First,
                },
                body: compiled,
                unknowns: unknowns.clone(),
                rows: vec![row],
                configuration: Configuration::Fixed(
                    unknowns,
                    Options {
                        start: vec![coordinate.lower + 0.5],
                        variable_nominals: vec![1.],
                        variable_tolerance: vec![1e-8],
                        residual_tolerance: vec![1e-8],
                        iterations: 100,
                        time_limit: std::time::Duration::from_secs(10),
                        derivative_tolerance: 1e-8,
                    },
                ),
                selection: Selection::default(),
                requirements: pse_kernels::DerivativeRequirements::new(
                    DerivativeOrder::First,
                    DerivativeOrder::First,
                    DerivativeOrder::First,
                    DerivativeOrder::First,
                    DerivativeOrder::First,
                )
                .unwrap(),
                hints: None,
                terms: None,
                solver: Arc::new(native::implicit::Kinsol),
                cancel: flag.clone(),
                max_entries: 1000,
                providers: Default::default(),
            }),
            SelectedResidualRealization::AuthoredValues,
            Some(proof),
            Some(Arc::new(native::root_isolation::Ibex)),
        );
        let normalization = Normalization::identity(2, 1);
        let local = Arc::new(
            OriginalContract::new(
                key,
                normalization.key(),
                vec![
                    coordinates[output_col].clone(),
                    coordinates[input_col].clone(),
                ],
                vec![math::Constraint {
                    id: row,
                    lower: offset,
                    upper: offset,
                }],
                vec![
                    Entry::new(GlobalRow::new(0), GlobalCol::new(0)),
                    Entry::new(GlobalRow::new(0), GlobalCol::new(1)),
                ],
                DerivativeSupport {
                    order: DerivativeOrder::First,
                    jacobian_product: true,
                    source: key,
                },
                OriginalObligations {
                    guards: key,
                    selection: key,
                    objective: None,
                },
            )
            .unwrap(),
        );
        let binding =
            SelectedResidualBinding::for_original(&factory, &local, &[GlobalRow::new(0)]).unwrap();
        assert_eq!(
            binding.realization(),
            &SelectedResidualRealization::AuthoredValues
        );
        let contract =
            SelectedImplicitReconstruction::<pse_math::MathError>::prepare_contract_with_binding(
                &factory,
                local,
                vec![GlobalCol::new(1)],
                vec![GlobalRow::new(0)],
                key,
                &flag,
                &binding,
            )
            .unwrap();
        factories.push(factory);
        contracts.push(contract);
        bindings.push(binding);
        local_scales.push(normalization);
    }
    let combined = CompositeReconstruction::<SelectedImplicitReconstruction<pse_math::MathError>>::prepare_contract_normalized(global.clone(),vec![GlobalCol::new(2)],&contracts,&scales,&local_scales).unwrap();
    assert_eq!(combined.eliminated().len(), 2);
    let service = runtime.native();
    let budget = WorkerBudget::drawing(service.policy.worker_bytes, &service.pool);
    execution::scoped(
        &[execution::adapter(Backend::Kinsol)],
        1,
        service.policy.stack_bytes,
        || {
            let _charge = budget
                .charge(
                    factories
                        .iter()
                        .map(|f| {
                            Ok::<_, pse_math::MathError>(
                                f.retained_bytes()? + f.reconstruction_workspace_bytes(256)?,
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()?
                        .iter()
                        .sum(),
                )
                .map_err(MathRuntimeError::into_problem)?;
            let workers = factories
                .iter()
                .zip(&contracts)
                .zip(&bindings)
                .zip(&local_scales)
                .map(|(((factory, contract), binding), normalization)| {
                    SelectedImplicitReconstruction::<pse_math::MathError>::new_with_binding(
                        factory,
                        contract.clone(),
                        normalization.clone(),
                        scope.clone(),
                        binding,
                    )?
                    .with_proof_cell_limit(256)
                })
                .collect::<Result<Vec<_>, _>>()?;
            let mut reconstruction =
                CompositeReconstruction::new_normalized(combined, workers, &scales, &local_scales)?;
            let admitted = reconstruction.admit(&[1.5])?;
            assert_eq!(admitted.values, [4.5, 6.5, 1.5]);
            let demand = AccuracyDemand {
                product: key,
                normalization: scales.key(),
                allowance: 1e-7,
                class: AccuracyClass::Certified,
            };
            let limits = math::RefinementLimits {
                rounds: 16,
                proof_cells: 1024,
            };
            let point = reconstruction.point(&[1.5], &demand, limits)?;
            assert!(point.accuracy.satisfies(&demand));
            assert_eq!(point.accuracy.class, AccuracyClass::Certified);
            assert!(
                (point.values[0] - point.values[2] - global.constraints()[0].lower).abs() < 1e-8
            );
            assert!(
                (point.values[1] - point.values[0] - global.constraints()[1].lower).abs() < 1e-8
            );
            for (value, coordinate) in point.values.iter().zip(global.coordinates()) {
                assert!((coordinate.lower..=coordinate.upper).contains(value));
            }
            let action = reconstruction.jacobian_product(&[1.5], &[2.], &demand, limits)?;
            assert!(action.accuracy.satisfies(&demand));
            assert_eq!(action.accuracy.class, AccuracyClass::Certified);
            for value in action.values {
                assert!((value - 2.).abs() < 1e-8);
            }
            Ok::<_, ProblemError>(())
        },
    )
    .unwrap();
    assert_eq!(budget.used(), 0);
}

#[cfg(all(
    feature = "solver-kinsol",
    feature = "solver-ipopt",
    feature = "solver-root-isolation"
))]
#[tokio::test]
async fn automatic_authored_supplier_actions_match_complete_original_equations() {
    use pse_math::derived::ReconstructionOracle;
    use pse_model::strategy::{AccuracyClass, AccuracyDemand};
    let runtime = crate::workflow::tests::runtime_on(
        512 << 20,
        crate::math::MathPolicy {
            worker_bytes: 128 << 20,
            workspace_bytes: 128 << 20,
            foreign_bytes: 32 << 20,
            ..Default::default()
        },
    );
    let (runtime, original) = original_order_on(runtime,
        "package p {def Root {param p:Scalar=1;var x:Scalar;annotation start x(1.5);annotation bounds x(0.5,3);implicit a {var y:Scalar;eq ey:y==2*x+p;annotation start y(4);annotation bounds y(1,8);}realize ra on a using nested;implicit b {var z:Scalar;eq ez:z==3*x-p;annotation start z(3.5);annotation bounds z(0.1,9);}realize rb on b using nested;eq floor:a.y+b.z>=3;let cost:Scalar=(x-2)*(x-2);annotation objective cost(minimize);}}",
        DerivativeOrder::First, SolverProfile { presolve: native::presolve::Policy::Off,
            intent: SolveIntent::Optimize, selection: SolverSelection::Explicit(Backend::Ipopt),
            controls: Controls { hessian: HessianMode::LimitedMemory, ..Default::default() },
            ..Default::default() }).await;
    let service = runtime.shared.math();
    let accuracy = ReconstructionAccuracy {
        point: original
            .tolerances
            .variables
            .iter()
            .zip(&original.normalization.variables)
            .map(|(budget, scale)| budget / scale)
            .fold(f64::INFINITY, f64::min),
        action: original.numerics.policy.kkt.stationarity,
        class: AccuracyClass::Certified,
        refinement: math::RefinementLimits {
            rounds: 16,
            proof_cells: 256,
        },
    };
    let request = service
        .automatic_reduced_request(&original, accuracy)
        .unwrap()
        .unwrap();
    let DerivedRequest::Reduced {
        suppliers,
        retained,
        ..
    } = &request
    else {
        panic!("actual reduced request");
    };
    assert_eq!(suppliers.len(), 2);
    assert_eq!(retained.len(), 1);
    let flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let scope = ExecutionScope::new(
        flag.clone(),
        Some(Instant::now() + std::time::Duration::from_secs(30)),
    );
    let profile = original.profile.clone();
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
    let budget = WorkerBudget::drawing(service.policy.worker_bytes, &service.pool);
    execution::scoped(&[execution::adapter(Backend::Kinsol)], 1, service.policy.stack_bytes, || {
        let p = &prepared;
        let local = p.reduced.as_ref().unwrap();
        let DerivedRequest::Reduced { suppliers, .. } = &p.request else { panic!("selected request"); };
        let workers = suppliers.iter().zip(&local.suppliers).map(|(source, local)| {
            SelectedImplicitReconstruction::<ProblemError>::new_with_binding(source.factory.as_ref(),
                local.contract.clone(), local.normalization.clone(), scope.clone(), &local.binding)?
                .with_proof_cell_limit(accuracy.refinement.proof_cells)
        }).collect::<Result<Vec<_>, pse_math::MathError>>()?;
        let mut reconstruction = CompositeReconstruction::new_normalized(local.contract.clone(), workers,
            &p.original.normalization, &local.suppliers.iter().map(|p|p.normalization.clone()).collect::<Vec<_>>())?;
        reconstruction.admit(&[1.5])?;
        let demand = AccuracyDemand { product: p.family.key(), normalization: p.physical.normalization(),
            allowance: accuracy.point, class: accuracy.class };
        let point = reconstruction.point(&[1.5], &demand, accuracy.refinement)?;
        let action = reconstruction.jacobian_product(&[1.5], &[2.0],
            &AccuracyDemand { allowance: accuracy.action, ..demand }, accuracy.refinement)?;
        assert!(point.accuracy.satisfies(&demand));
        let (mut oracle, _owner) = service.derived_original_oracle(p, &scope, &budget)?;
        let mut rows = vec![0.0; p.physical.constraints().len()];
        oracle.constraints(&point.values, &mut rows)?;
        for (value, row) in rows.iter().zip(p.physical.constraints()) {
            if row.lower == row.upper { assert!(value.abs() < 1e-8); }
            else {
                // The compiler retains the authored lhs-minus-rhs residual:
                // (2*x+p)+(3*x-p)-3 >= 0, rather than the left side alone.
                assert_eq!((row.lower, row.upper), (0.0, f64::INFINITY));
                let original_floor = (2.0 * 1.5 + 1.0) + (3.0 * 1.5 - 1.0) - 3.0;
                assert!((*value - original_floor).abs() < 1e-8);
                assert!(*value >= row.lower);
            }
        }
        assert!((oracle.objective(&point.values)? - 0.25).abs() < 1e-10);
        let mut gradient = vec![0.0; point.values.len()];
        oracle.gradient(&point.values, &mut gradient)?;
        assert!((gradient.iter().zip(&action.values).map(|(g,v)|g*v).sum::<f64>() + 2.0).abs() < 1e-8);
        let pattern = oracle.jacobian_pattern();
        let mut values = vec![0.0; pattern.compute_nnz()];
        oracle.jacobian(&point.values, &mut values)?;
        let pattern = oracle.jacobian_pattern();
        let mut applied = vec![0.0; rows.len()];
        for (column, value) in action.values.iter().enumerate() {
            for index in pattern.col_ptr()[column]..pattern.col_ptr()[column+1] {
                applied[pattern.row_idx()[index]] += values[index] * value;
            }
        }
        for (value, row) in applied.iter().zip(p.physical.constraints()) {
            if row.lower == row.upper { assert!(value.abs() < 1e-8); }
            else { assert!((*value - 10.0).abs() < 1e-8); }
        }
        // Native action receipts name the actual full original point consumed by
        // the action, while their accuracy identity still binds its exact direction.
        let bound = p.family.general()?.bind_reduced(
            NlpBridge::new(Box::new(oracle), p.physical.clone())?, reconstruction,
            p.original_identity)?;
        let other_direction = bound.action_product(&[1.5], &[2.0])?.key()?;
        let mut native = ReducedOracle::new(bound, &[1.5], accuracy,
            p.source.prepared.prepared.plan.available_order(), isize::MAX as usize)?;
        let mut actual_gradient = [0.0];
        native.gradient(&[1.5], &mut actual_gradient)?;
        let receipt = native.applied_action().copied().unwrap();
        let original_point = p.original.semantic_point_key(&point.values)?.point.unwrap();
        assert_eq!(receipt.0, original_point);
        assert_ne!(receipt.1.product, other_direction);
        let source = pse_model::strategy::SemanticProductKey {
            point: Some(receipt.0), accuracy: Some(receipt.1.product), ..p.product_source()?
        };
        let evidence = pse_model::strategy::ProductEvidence {
            source, derivative_order: 1, branch: p.original.profile.composition.branch,
            accuracy: receipt.1,
        };
        assert!(p.operation_contract()?.outputs.iter().any(|output| output.admits(&evidence)));
        let mut state = crate::math::strategy::ProductState::owned(1,
            service.reserve("test:actual-reconstruction-action", crate::math::strategy::ProductState::extent(1)?)?);
        state.publish(evidence)?;
        let demand = pse_model::strategy::ProductDemand {
            source: pse_model::strategy::SemanticProductKey { point: Some(original_point), ..source },
            derivative_order: 1, branch: p.original.profile.composition.branch,
            accuracy: AccuracyDemand { product: receipt.1.product,
                normalization: receipt.1.normalization, allowance: accuracy.action, class: accuracy.class },
        };
        let operation = |demand| pse_model::strategy::OperationContract { inputs: vec![demand], outputs: Vec::new() };
        assert_eq!(state.consume(&operation(demand.clone()))?, vec![receipt.1]);
        let mut wrong_point = point.values.clone();
        wrong_point[0] += 0.1;
        let mut mismatched = demand.clone();
        mismatched.source.point = p.original.semantic_point_key(&wrong_point)?.point;
        assert!(state.consume(&operation(mismatched)).is_err());
        let mut mismatched = demand;
        mismatched.source.accuracy = Some(other_direction);
        mismatched.accuracy.product = other_direction;
        assert!(state.consume(&operation(mismatched)).is_err(),
            "one actual directional action cannot certify another direction or the whole Jacobian");
        let later = native.original_proposal(&[2.0])?;
        assert_ne!(p.original.semantic_point_key(&later.values)?.point, Some(receipt.0));
        assert_eq!(native.applied_action().copied(), Some(receipt),
            "a later value-only proposal cannot relabel the last actual action point");
        let mut actual_jacobian = vec![0.0; native.jacobian_pattern().compute_nnz()];
        native.jacobian(&[1.5], &mut actual_jacobian)?;
        assert_eq!(native.applied_action().unwrap().0, original_point);
        Ok::<_, MathRuntimeError>(())
    }).unwrap();
}
