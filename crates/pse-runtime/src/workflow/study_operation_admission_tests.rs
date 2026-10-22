// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Study descriptors preserve admission through the existing general operation owners.
#![allow(
    clippy::unwrap_used,
    clippy::panic,
    clippy::unreachable,
    reason = "native admission controls fail on invalid setup or unexpected operation variants"
)]
use super::super::{
    BindingAssignment, BindingQuantity, BindingTarget, FitPreparationDocument, PointOverlay,
    PreparedStudyOperation, StudyPoint, StudyPointPolicy,
};
use super::*;
use pse_model::scalars::{FiniteBound, PositiveCount, Tolerance};
use pse_model::study::{OccurrenceKey, StartPolicy};

fn point(operation: OperationRequest) -> StudyPoint {
    StudyPoint {
        operation,
        preparation: PreparationSettings {
            compiler: super::super::tests::compiler_profile(),
            limits: Limits::default(),
        },
        overlay: PointOverlay::default(),
        policy: StudyPointPolicy {
            key: OccurrenceKey(7),
            dependencies: vec![],
            start: StartPolicy::Fresh,
            attempt_limit: 1,
        },
    }
}
async fn plant() -> (ModelingPackage, DeclarationId) {
    plant_on(super::super::tests::runtime()).await
}
async fn plant_on(runtime: Runtime) -> (ModelingPackage, DeclarationId) {
    let text = "package p {
      def Root {
        domain t:Time from 0{s} to 1{s};
        discretize grid on t using integrated(elements=1,order=1);
        param u:Temperature=300{K}; param base:Temperature=300{K};
        param span:DeltaTemperature=1{K}; var x[i in t]:Time;
        eq rate[i in t]:d(x[i])/di==(u-base)/span;
        eq initial:x[0{s}]==1{s};
      }
      test plant fixture { dof 0; route integrated; procedure integrate;
        integrate samples(0{s},1{s}) relative(1e-8) normalized_absolute(1e-10) step(1e-4{s});
      } { child root:Root=Root(); }
      test Control fixture { dof 0; value prior=300{K}; } { param prior:Temperature=300{K}; param alternate:Temperature=301{K};
        param span:DeltaTemperature=1{K}; var x:Temperature;
        eq match:x==prior; let other:Temperature=alternate;
        let cost:Scalar=((x-prior)/span)*((x-prior)/span);
        annotation objective cost(minimize); annotation start x(300{K}); }
    }";
    let rows = pse_authoring::language::parse(
        text,
        SemanticId::NIL,
        pse_authoring::language::IdentityPolicy::Named,
        Default::default(),
    )
    .unwrap();
    let root = rows
        .iter()
        .find(|r| r.name == "plant")
        .unwrap()
        .declaration_id;
    let mut physical = super::super::tests::physical();
    physical.preconditions = Arc::new(
        pse_quantity::PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions())
            .unwrap(),
    );
    physical.key =
        pse_compiler::workspace::physical_identity(&physical.quantities, &physical.preconditions);
    (
        runtime.modeling_package(rows, physical).await.unwrap(),
        root,
    )
}
fn quantity(package: &ModelingPackage, magnitude: f64, name: &str, unit: &str) -> BindingQuantity {
    BindingQuantity {
        magnitude: FiniteBound::try_new(magnitude).unwrap(),
        quantity: package
            .quantities
            .quantity_types()
            .find(|q| q.name.as_deref() == Some(name))
            .unwrap()
            .id
            .as_id(),
        unit: package
            .quantities
            .units()
            .find(|u| u.symbol == unit)
            .unwrap()
            .id
            .as_id(),
    }
}
async fn control_operation() -> (ModelingPackage, StudyOperation) {
    let (package, _) = plant().await;
    let case = package
        .declarations()
        .await
        .unwrap()
        .iter()
        .find(|declaration| declaration.name == "Control")
        .unwrap()
        .declaration_id;
    let operation = OperationRequest::DeclaredCase(CaseOperation {
        case,
        route: ModelingAnalysisRoute::Steady,
        settings: SolveSettings::default(),
    });
    let preparation = point(operation.clone()).preparation;
    let operation = StudyOperation {
        output: crate::workflow::StudyOutputDemand::Full {},
        version: Version,
        source: OperationSource::of(&package),
        preparation,
        operation,
        admitted_horizon: None,
    };
    (package, operation)
}
fn prior_overlay(package: &ModelingPackage, magnitude: f64, unit: &str) -> PointOverlay {
    PointOverlay {
        assignments: vec![BindingAssignment {
            target: BindingTarget::Path("prior".into()),
            value: quantity(package, magnitude, "Temperature", unit),
        }],
    }
}

#[tokio::test]
async fn host_and_study_requests_share_case_preparation_and_keep_study_route_assertion() {
    let (package, operation) = control_operation().await;
    let OperationRequest::DeclaredCase(case) = &operation.operation else {
        panic!("declared case");
    };
    let cancel = CancelSource::new();
    let (prepared, admissions) = counted_declared_admissions(async {
        let host = package
            .prepare_operation(
                &operation.source,
                &operation.preparation,
                crate::workflow::RuntimeOperationRequest::DeclaredCase {
                    case: case.case,
                    settings: Box::new(case.settings.clone()),
                },
                None,
                &cancel,
            )
            .await
            .unwrap()
            .into_study()
            .unwrap();
        let study = operation.prepare(&package, &cancel).await.unwrap();
        (host, study)
    })
    .await;
    assert_eq!(
        admissions, 2,
        "both callers use the shared declared admission owner"
    );
    let (PreparedStudyOperation::DeclaredCase(host), PreparedStudyOperation::DeclaredCase(study)) =
        prepared
    else {
        panic!("paired case preparations");
    };
    assert_eq!(
        host.solve.request_identity().unwrap(),
        study.solve.request_identity().unwrap()
    );
    assert_eq!(host.solve.numerics().key, study.solve.numerics().key);
    assert_eq!(
        crate::math::solves::profile_key(&host.profile).unwrap(),
        crate::math::solves::profile_key(&study.profile).unwrap()
    );
    let mut wrong_route = operation;
    let OperationRequest::DeclaredCase(case) = &mut wrong_route.operation else {
        panic!("declared case");
    };
    case.route = ModelingAnalysisRoute::Integrated;
    let error = wrong_route.prepare(&package, &cancel).await.unwrap_err();
    assert_eq!(
        error.boundary_diagnostic().class,
        BoundaryClass::Unsupported
    );
}

#[tokio::test]
async fn presentation_only_successive_admission_reuses_basis_with_current_receiving() {
    let (package, operation) = control_operation().await;
    let cancel = CancelSource::new();
    let overlay = prior_overlay(&package, 300.0, "K");
    let mut admission = None;
    let (binding, _) = package
        .admit_operation_overlay(&operation, &overlay, &mut admission, &cancel)
        .await
        .unwrap();
    let model = admission
        .as_ref()
        .unwrap()
        .execution
        .model
        .compiled()
        .model
        .clone();
    let mut presented = operation.clone();
    presented.output = crate::workflow::StudyOutputDemand::Compact {
        detail: Default::default(),
        failed_detail: Default::default(),
    };
    let (again, _) = package
        .admit_operation_overlay(&presented, &overlay, &mut admission, &cancel)
        .await
        .unwrap();
    assert_eq!(binding.identity(), again.identity());
    assert!(pse_math::SharedAllocation::ptr_eq(
        &model,
        &admission.as_ref().unwrap().execution.model.compiled().model
    ));
    assert_eq!(
        DeclaredStudyAdmission::binding_key(&operation, &binding).unwrap(),
        DeclaredStudyAdmission::binding_key(&presented, &again).unwrap()
    );
    let foreign = package.distinct_provider_owner_for_test();
    assert!(
        admission
            .as_ref()
            .unwrap()
            .execution
            .prepared_model(&foreign)
            .is_err()
    );
    assert!(
        admission
            .as_ref()
            .unwrap()
            .execution
            .prepared_model(&package)
            .is_ok()
    );
    let mut changed = presented;
    let OperationRequest::DeclaredCase(case) = &mut changed.operation else {
        panic!("declared case expected");
    };
    case.settings.numerics.supplier_action_accuracy *= 0.5;
    assert_ne!(
        DeclaredStudyAdmission::binding_key(&changed, &binding).unwrap(),
        DeclaredStudyAdmission::binding_key(&operation, &binding).unwrap()
    );
}

#[cfg(feature = "solver-kinsol")]
#[tokio::test]
#[allow(
    clippy::print_stderr,
    reason = "bounded I28 probe reports actual immutable allocations and independent lane preparation counts"
)]
async fn independent_study_lane_immutable_basis_and_changed_premises_probe() {
    let (package, operation) = control_operation().await;
    let cancel = CancelSource::new();
    let mut admission = None;
    let (binding, _) = package
        .admit_operation_overlay(
            &operation,
            &prior_overlay(&package, 300.0, "K"),
            &mut admission,
            &cancel,
        )
        .await
        .unwrap();
    let mut left = StudyPreparationLane::new();
    let mut right = StudyPreparationLane::new();
    let ((first, second), counts) = crate::math::counted(async {
        tokio::join!(
            left.prepare(&package, &operation, &binding, &cancel),
            right.prepare(&package, &operation, &binding, &cancel)
        )
    })
    .await;
    let first = first.unwrap();
    let second = second.unwrap();
    let left_scope = left.scope().unwrap();
    let right_scope = right.scope().unwrap();
    assert!(
        !Arc::ptr_eq(&left_scope, &right_scope),
        "independent lanes never share admission authority"
    );
    let basis = |lane: &StudyPreparationLane| {
        lane.current
            .as_ref()
            .unwrap()
            .admission
            .as_ref()
            .unwrap()
            .execution
            .model
            .compiled()
            .model
            .allocation_identity()
    };
    let (left_basis, right_basis) = (basis(&left), basis(&right));
    eprintln!(
        "I28 equal-basis independent lanes: left={left_basis} right={right_basis} shared={} preparation_counts={counts:?}",
        left_basis == right_basis
    );
    // Sharing is the inquiry's observation, not an admission precondition or
    // a conclusion imposed on the two independent receiving owners.
    drop(first);
    drop(second);
    let mut changed = operation.clone();
    changed
        .preparation
        .compiler
        .evaluation
        .derivative_components += 1;
    let prepared = left
        .prepare(&package, &changed, &binding, &cancel)
        .await
        .unwrap();
    drop(prepared);
    assert!(!left_scope.is_open());
    assert!(right_scope.is_open());
    let compiler_scope = left.scope().unwrap();
    let OperationRequest::DeclaredCase(case) = &mut changed.operation else {
        panic!("declared case expected");
    };
    case.settings.numerics.supplier_action_accuracy *= 0.5;
    let prepared = left
        .prepare(&package, &changed, &binding, &cancel)
        .await
        .unwrap();
    drop(prepared);
    assert!(!compiler_scope.is_open());
    let policy_scope = left.scope().unwrap();
    let foreign = package.distinct_provider_owner_for_test();
    let prepared = left
        .prepare(&foreign, &changed, &binding, &cancel)
        .await
        .unwrap();
    drop(prepared);
    assert!(!policy_scope.is_open());
    assert!(right_scope.is_open());
    let mut wrong_source = changed;
    wrong_source.source.physical_context = ContentHash::from_bytes([91; 32]);
    assert!(
        left.prepare(&foreign, &wrong_source, &binding, &cancel)
            .await
            .is_err()
    );
    assert!(left.current.is_none());
    assert!(right.scope().unwrap().is_open());
    right.close().await.unwrap();
}

#[tokio::test]
async fn declared_admission_reuses_one_basis_for_thousand_physical_points() {
    let (package, operation) = control_operation().await;
    let cancel = CancelSource::new();
    let ((), preparations) = crate::math::counted(async {
        let mut admission = None;
        let mut identities = std::collections::BTreeSet::new();
        let mut base_allocations = std::collections::BTreeSet::new();
        let mut demand_allocations = std::collections::BTreeSet::new();
        for index in 0..1000 {
            let (magnitude, unit, expected) = if index % 2 == 0 {
                (300.0 + f64::from(index), "K", 300.0 + f64::from(index))
            } else {
                let magnitude = 20.0 + f64::from(index);
                (magnitude, "degC", magnitude + 273.15)
            };
            let overlay = prior_overlay(&package, magnitude, unit);
            let (binding, seed_need) = package
                .admit_operation_overlay(&operation, &overlay, &mut admission, &cancel)
                .await
                .unwrap();
            assert_eq!(seed_need, SeedNeed::Required);
            let entry = binding.entries.values().next().unwrap();
            assert!(entry.parameter);
            assert_eq!(entry.canonical.into_inner(), expected);
            assert_eq!(entry.supplied_unit, overlay.assignments[0].value.unit);
            assert!(identities.insert(binding.identity()));
            let basis = admission.as_ref().unwrap();
            // Inspect the actual owner-held allocations, rather than a nominal
            // compiler cache key. All numeric points retain these same products.
            base_allocations.insert(basis.execution.model.compiled().model.allocation_identity());
            demand_allocations.insert(basis.overlay_model().model.allocation_identity());
            assert!(basis.matches(&DeclaredStudyAdmission::key(&operation, &overlay).unwrap()));
        }
        assert_eq!(identities.len(), 1000);
        assert_eq!(base_allocations.len(), 1);
        assert_eq!(demand_allocations.len(), 1);
    })
    .await;
    // These production counters cover solver-view/observation construction and
    // rebinds, not selected-source RPCs or server allocator consumption.
    assert_eq!(preparations, crate::math::PreparationCounts::default());
}

#[tokio::test]
async fn reused_declared_admission_rechecks_each_binding_and_structural_change() {
    let (package, operation) = control_operation().await;
    let cancel = CancelSource::new();
    let overlay = prior_overlay(&package, 80.0, "degC");
    let mut admission = None;
    let (first, _) = package
        .admit_operation_overlay(&operation, &overlay, &mut admission, &cancel)
        .await
        .unwrap();
    let original_key = DeclaredStudyAdmission::key(&operation, &overlay).unwrap();
    let base = admission
        .as_ref()
        .unwrap()
        .execution
        .model
        .compiled()
        .model
        .clone();
    let demand = admission.as_ref().unwrap().overlay_model().model.clone();
    let canonical = prior_overlay(&package, 353.15, "K");
    let (equivalent, _) = package
        .admit_operation_overlay(&operation, &canonical, &mut admission, &cancel)
        .await
        .unwrap();
    assert_eq!(first.identity(), equivalent.identity());
    assert!(pse_math::SharedAllocation::ptr_eq(
        &base,
        &admission.as_ref().unwrap().execution.model.compiled().model
    ));
    assert!(pse_math::SharedAllocation::ptr_eq(
        &demand,
        &admission.as_ref().unwrap().overlay_model().model
    ));

    let mut duplicate = overlay.clone();
    duplicate
        .assignments
        .push(prior_overlay(&package, 81.0, "degC").assignments.remove(0));
    let error = package
        .admit_operation_overlay(&operation, &duplicate, &mut admission, &cancel)
        .await
        .unwrap_err();
    assert_eq!(
        error.boundary_diagnostic().rule,
        DiagnosticRule::StudyBindingDuplicate
    );
    let mut wrong = overlay.clone();
    wrong.assignments[0].value = quantity(&package, 1.0, "Time", "s");
    let error = package
        .admit_operation_overlay(&operation, &wrong, &mut admission, &cancel)
        .await
        .unwrap_err();
    assert_eq!(
        error.boundary_diagnostic().rule,
        DiagnosticRule::StudyBindingPhysical
    );
    assert!(admission.as_ref().unwrap().matches(&original_key));
    let mut invalid_member = overlay.clone();
    invalid_member.assignments.push(BindingAssignment {
        target: BindingTarget::Member(SemanticId::NIL),
        value: overlay.assignments[0].value.clone(),
    });
    let error = package
        .admit_operation_overlay(&operation, &invalid_member, &mut admission, &cancel)
        .await
        .unwrap_err();
    assert_eq!(
        error.boundary_diagnostic().rule,
        DiagnosticRule::StudyBindingTarget
    );
    assert!(admission.as_ref().unwrap().matches(&original_key));
    let mut forged = first.clone();
    forged.entries.values_mut().next().unwrap().parameter = false;
    assert!(
        package
            .validate_binding(
                admission.as_ref().unwrap().execution.model.compiled(),
                &forged
            )
            .is_err()
    );

    let mut changed_path = overlay.clone();
    changed_path.assignments[0].target = BindingTarget::Path("cost".into());
    let error = package
        .admit_operation_overlay(&operation, &changed_path, &mut admission, &cancel)
        .await
        .unwrap_err();
    assert_eq!(
        error.boundary_diagnostic().rule,
        DiagnosticRule::StudyBindingTarget
    );
    assert!(!admission.as_ref().unwrap().matches(&original_key));

    let mut changed_settings = operation.clone();
    let OperationRequest::DeclaredCase(case) = &mut changed_settings.operation else {
        unreachable!()
    };
    case.settings.controls.threads = 0;
    assert!(
        package
            .admit_operation_overlay(&changed_settings, &overlay, &mut admission, &cancel)
            .await
            .is_err()
    );
    assert!(
        admission.is_none(),
        "failed replacement releases the previous basis"
    );
    package
        .admit_operation_overlay(&operation, &overlay, &mut admission, &cancel)
        .await
        .unwrap();
    let mut changed_limits = operation.clone();
    changed_limits.preparation.limits.depth = 0;
    assert!(
        package
            .admit_operation_overlay(&changed_limits, &overlay, &mut admission, &cancel)
            .await
            .is_err()
    );
    assert!(admission.is_none());
    package
        .admit_operation_overlay(&operation, &overlay, &mut admission, &cancel)
        .await
        .unwrap();
    let mut changed_compiler = operation.clone();
    changed_compiler.preparation.compiler.class_proof_work += 1;
    assert_ne!(
        DeclaredStudyAdmission::key(&changed_compiler, &overlay).unwrap(),
        original_key
    );
    let mut changed_route = operation.clone();
    let OperationRequest::DeclaredCase(case) = &mut changed_route.operation else {
        unreachable!()
    };
    case.route = ModelingAnalysisRoute::Integrated;
    let error = package
        .admit_operation_overlay(&changed_route, &wrong, &mut admission, &cancel)
        .await
        .unwrap_err();
    assert_eq!(
        error.boundary_diagnostic().rule,
        DiagnosticRule::StudyBindingPhysical,
        "overlay physical admission precedes route/procedure validation"
    );
    let error = package
        .admit_operation_overlay(&changed_route, &overlay, &mut admission, &cancel)
        .await
        .unwrap_err();
    assert_eq!(
        error.boundary_diagnostic().rule,
        DiagnosticRule::StudyOperationUnsupported
    );
    assert!(!admission.as_ref().unwrap().matches(&original_key));
    package
        .admit_operation_overlay(&operation, &overlay, &mut admission, &cancel)
        .await
        .unwrap();
    let cancelled = CancelSource::new();
    cancelled.cancel();
    let error = package
        .admit_operation_overlay(&operation, &overlay, &mut admission, &cancelled)
        .await
        .unwrap_err();
    assert_eq!(
        error.boundary_diagnostic().code,
        pse_diagnostics::DiagnosticCode::RuntimeCancelled,
        "{error:?}"
    );
    assert!(
        admission.as_ref().unwrap().matches(&original_key),
        "cancelled admission does not replace or prepare the structural basis"
    );
}

#[tokio::test]
async fn declared_admission_rechecks_nonfinite_optional_settings() {
    let (package, operation) = control_operation().await;
    let cancel = CancelSource::new();
    let overlay = prior_overlay(&package, 300.0, "K");
    let valid_key = DeclaredStudyAdmission::key(&operation, &overlay).unwrap();
    let mut admission = None;
    for absolute in [true, false] {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            package
                .admit_operation_overlay(&operation, &overlay, &mut admission, &cancel)
                .await
                .unwrap();
            let mut invalid = operation.clone();
            let OperationRequest::DeclaredCase(case) = &mut invalid.operation else {
                unreachable!()
            };
            if absolute {
                case.settings.convexity_absolute = Some(value);
            } else {
                case.settings.convexity_relative = Some(value);
            }
            assert_ne!(
                DeclaredStudyAdmission::key(&invalid, &overlay).unwrap(),
                valid_key,
                "canonical framing distinguishes None from Some({value})"
            );
            assert!(
                package
                    .admit_operation_overlay(&invalid, &overlay, &mut admission, &cancel)
                    .await
                    .is_err()
            );
            assert!(
                admission.is_none(),
                "invalid settings release the previous basis"
            );
        }
    }
}

#[tokio::test]
async fn scoped_study_admission_preserves_individual_point_bindings() {
    let (package, operation) = control_operation().await;
    let mut points = Vec::new();
    for (key, magnitude, unit) in [(0, 80.0, "degC"), (1, 354.15, "K")] {
        let mut submitted = point(operation.operation.clone());
        submitted.preparation = operation.preparation.clone();
        submitted.policy.key = OccurrenceKey(key);
        submitted.overlay = prior_overlay(&package, magnitude, unit);
        points.push(submitted);
    }
    let definition = package
        .admit_study(
            &crate::workflow::StudyInputs::Explicit(points.to_vec()),
            &crate::workflow::StudyOutputDemand::Full {},
            &CancelSource::new(),
        )
        .await
        .unwrap();
    assert_eq!(definition.points.len(), 2);
    for (point, expected) in definition.points.iter().zip([353.15, 354.15]) {
        assert_eq!(
            point
                .binding
                .entries
                .values()
                .next()
                .unwrap()
                .canonical
                .into_inner(),
            expected
        );
        assert_eq!(point.binding_hash, point.binding.identity());
        assert_eq!(point.policy.seed_need, SeedNeed::Required);
    }
    assert_ne!(
        definition.points[0].binding_hash,
        definition.points[1].binding_hash
    );
}

#[tokio::test]
async fn scoped_study_admission_transitions_from_declared_case_to_horizon() {
    let (package, operation) = control_operation().await;
    let plant = package
        .declarations()
        .await
        .unwrap()
        .iter()
        .find(|declaration| declaration.name == "plant")
        .unwrap()
        .declaration_id;
    let mut declared = point(operation.operation);
    declared.policy.key = OccurrenceKey(0);
    declared.overlay = prior_overlay(&package, 80.0, "degC");
    let mut horizon = point(OperationRequest::Horizon(Box::new(
        horizon(&package, plant).await,
    )));
    horizon.policy.key = OccurrenceKey(1);
    let definition = package
        .admit_study(
            &crate::workflow::StudyInputs::Explicit([declared, horizon].to_vec()),
            &crate::workflow::StudyOutputDemand::Full {},
            &CancelSource::new(),
        )
        .await
        .unwrap();
    assert_eq!(definition.points.len(), 2);
    assert_eq!(definition.points[0].policy.seed_need, SeedNeed::Required);
    assert_eq!(
        definition.points[0]
            .binding
            .entries
            .values()
            .next()
            .unwrap()
            .canonical
            .into_inner(),
        353.15
    );
    let horizon = &definition.points[1];
    assert_eq!(horizon.policy.seed_need, SeedNeed::NotNeeded);
    assert!(horizon.binding.entries.is_empty());
    assert_eq!(
        horizon.operation.admitted_horizon.as_ref().unwrap().inputs[0]
            .canonical
            .into_inner(),
        353.15
    );
}
async fn horizon(package: &ModelingPackage, case: DeclarationId) -> HorizonOperation {
    let simulation = package
        .declared_simulation(
            case,
            super::super::tests::compiler_profile(),
            None,
            Limits::default(),
            &CancelSource::new(),
        )
        .await
        .unwrap();
    let parameter = simulation
        .model()
        .compiled()
        .model
        .symbols
        .values()
        .find(|symbol| symbol.lineage.path == "plant.root.u")
        .unwrap()
        .id;
    assert!(simulation.contract().parameters.contains(&parameter));
    let control = package
        .declarations()
        .await
        .unwrap()
        .iter()
        .find(|r| r.name == "Control")
        .unwrap()
        .declaration_id;
    HorizonOperation {
        plant: SimulationOperation {
            case,
            settings: None,
        },
        period: Tolerance::try_new(0.5).unwrap(),
        steps: PositiveCount::try_new(2).unwrap(),
        inputs: vec![HorizonInputDocument {
            parameter,
            initial: quantity(package, 80., "Temperature", "degC"),
        }],
        estimator: None,
        controller: Some(ControllerOperation {
            case: CaseOperation {
                case: control,
                route: ModelingAnalysisRoute::Steady,
                settings: SolveSettings::default(),
            },
            bindings: vec![HorizonBinding {
                target: "prior".into(),
                signal: HorizonSignalDocument::Trajectory(vec![
                    quantity(
                        package,
                        80.,
                        "Temperature",
                        "degC"
                    );
                    2
                ]),
            }],
            moves: vec![("x".into(), 0)],
            predictions: None,
        }),
    }
}

#[tokio::test]
async fn admitted_simulation_descriptor_reconstructs_the_existing_owner() {
    let (package, case) = plant().await;
    let definition = package
        .admit_study(
            &crate::workflow::StudyInputs::Explicit(
                [point(OperationRequest::Simulation(SimulationOperation {
                    case,
                    settings: None,
                }))]
                .to_vec(),
            ),
            &crate::workflow::StudyOutputDemand::Full {},
            &CancelSource::new(),
        )
        .await
        .unwrap();
    let p: super::super::StudyPointDefinition =
        serde_json::from_slice(&serde_json::to_vec(&definition.points[0]).unwrap()).unwrap();
    assert_eq!(p.policy.seed_need, SeedNeed::NotNeeded);
    assert!(matches!(
        package
            .prepare_bound_operation(&p.operation, &p.binding, &CancelSource::new())
            .await
            .unwrap(),
        PreparedStudyOperation::Simulation(_)
    ));
}

#[tokio::test]
async fn admitted_fit_descriptor_reconstructs_the_existing_owner() {
    use super::super::fitting::regression;
    use pse_backend_native::solve::{Backend, SolveIntent};
    let package = regression::package(regression::declared(), [1.; 4]).await;
    let request = OperationRequest::Fit(FitOperation {
        fit: super::super::tests::id(32).into(),
        settings: crate::workflow::PhysicalFitRequest::new(FitPreparationDocument {
            optimizer: crate::workflow::FitOptimizer::NativeNlp,
            solver: SolveSettings {
                intent: SolveIntent::Optimize,
                backend: Some(Backend::Ipopt),
                ..Default::default()
            },
            simulations: BTreeMap::new(),
            rank_tolerance: 1e-8,
            max_cells: 100_000,
            derivatives: FitDerivatives::Responses,
            experiment_workers: pse_model::scalar!(PositiveCount(1)),
            uncertainty: None,
        }),
    });
    let definition = package
        .admit_study(
            &crate::workflow::StudyInputs::Explicit([point(request)].to_vec()),
            &crate::workflow::StudyOutputDemand::Full {},
            &CancelSource::new(),
        )
        .await
        .unwrap();
    let p = &definition.points[0];
    assert_eq!(p.policy.seed_need, SeedNeed::NotNeeded);
    assert!(matches!(
        package
            .prepare_bound_operation(&p.operation, &p.binding, &CancelSource::new())
            .await
            .unwrap(),
        PreparedStudyOperation::Fit(_)
    ));
}

#[tokio::test]
async fn admitted_horizon_normalizes_inputs_once_and_refuses_wrong_physical_meaning() {
    let (package, case) = plant().await;
    let request = horizon(&package, case).await;
    let definition = package
        .admit_study(
            &crate::workflow::StudyInputs::Explicit(
                [point(OperationRequest::Horizon(Box::new(request.clone())))].to_vec(),
            ),
            &crate::workflow::StudyOutputDemand::Full {},
            &CancelSource::new(),
        )
        .await
        .unwrap();
    let p = &definition.points[0];
    let canonical = &p.operation.admitted_horizon.as_ref().unwrap().inputs[0];
    assert_eq!(canonical.canonical.into_inner(), 353.15);
    let PreparedStudyOperation::Horizon(prepared) = package
        .prepare_bound_operation(&p.operation, &p.binding, &CancelSource::new())
        .await
        .unwrap()
    else {
        panic!("horizon owner")
    };
    assert_eq!(prepared.inputs[0].initial, 353.15);
    let mut equivalent = request.clone();
    equivalent.inputs[0].initial = quantity(&package, 353.15, "Temperature", "K");
    let other = package
        .admit_study(
            &crate::workflow::StudyInputs::Explicit(
                [point(OperationRequest::Horizon(Box::new(equivalent)))].to_vec(),
            ),
            &crate::workflow::StudyOutputDemand::Full {},
            &CancelSource::new(),
        )
        .await
        .unwrap();
    assert_eq!(
        other.points[0]
            .operation
            .admitted_horizon
            .as_ref()
            .unwrap()
            .inputs[0]
            .canonical,
        canonical.canonical
    );
    let mut wrong = request;
    wrong.inputs[0].initial = quantity(&package, 1., "Time", "s");
    let error = package
        .admit_study(
            &crate::workflow::StudyInputs::Explicit(
                [point(OperationRequest::Horizon(Box::new(wrong)))].to_vec(),
            ),
            &crate::workflow::StudyOutputDemand::Full {},
            &CancelSource::new(),
        )
        .await
        .unwrap_err();
    assert_eq!(
        error.boundary_diagnostic().rule,
        DiagnosticRule::StudyBindingPhysical
    );
}

#[tokio::test]
async fn horizon_priors_and_trajectories_retain_canonical_target_correspondence() {
    let (package, case) = plant().await;
    let control = package
        .declarations()
        .await
        .unwrap()
        .iter()
        .find(|r| r.name == "Control")
        .unwrap()
        .declaration_id;
    let selected = CaseOperation {
        case: control,
        route: ModelingAnalysisRoute::Steady,
        settings: SolveSettings::default(),
    };
    let mut request = horizon(&package, case).await;
    request.controller = Some(ControllerOperation {
        case: selected.clone(),
        bindings: vec![HorizonBinding {
            target: "prior".into(),
            signal: HorizonSignalDocument::Trajectory(vec![quantity(
                &package,
                80.,
                "Temperature",
                "degC",
            )]),
        }],
        moves: vec![],
        predictions: None,
    });
    request.estimator = Some(EstimatorOperation {
        case: selected,
        window: 1,
        measurements: vec![],
        inputs: vec![],
        arrival: vec![ArrivalDocument {
            prior: "prior".into(),
            next: "x".into(),
            initial: quantity(&package, 80., "Temperature", "degC"),
        }],
    });
    let mut operation = StudyOperation {
        output: crate::workflow::StudyOutputDemand::Full {},
        version: Version,
        source: OperationSource::of(&package),
        preparation: point(OperationRequest::Horizon(Box::new(request.clone()))).preparation,
        operation: OperationRequest::Horizon(Box::new(request)),
        admitted_horizon: None,
    };
    operation
        .admit_horizon_values(&package, &CancelSource::new())
        .await
        .unwrap();
    let canonical = operation.admitted_horizon.as_ref().unwrap();
    assert_eq!(canonical.arrival[0].canonical.into_inner(), 353.15);
    assert_eq!(canonical.trajectories[&0][0].canonical.into_inner(), 353.15);
    let OperationRequest::Horizon(spec) = &operation.operation else {
        panic!("horizon")
    };
    let model = declared_paths(
        &package,
        &spec.estimator.as_ref().unwrap().case,
        operation.preparation.compiler,
        Limits::default(),
        &CancelSource::new(),
        vec!["prior".into(), "alternate".into()],
    )
    .await
    .unwrap();
    let prior = model.model.compiled().model.paths["prior"];
    let alternate = model.model.compiled().model.paths["alternate"];
    assert_ne!(prior, alternate);
    let mut forged = canonical.arrival[0].clone();
    forged.member = alternate;
    assert!(
        OperationPreparation {
            source: &operation.source,
            preparation: &operation.preparation,
            operation: &operation.operation,
            output: &operation.output,
            admitted_horizon: operation.admitted_horizon.as_ref(),
            binding: None,
        }
        .validate_entry(&package, model.model.compiled(), prior, &forged)
        .is_err()
    );
    let mut incompatible = operation.clone();
    let OperationRequest::Horizon(spec) = &mut incompatible.operation else {
        panic!("horizon")
    };
    spec.estimator.as_mut().unwrap().arrival[0].initial = quantity(&package, 1., "Time", "s");
    let error = incompatible
        .admit_horizon_values(&package, &CancelSource::new())
        .await
        .unwrap_err();
    assert_eq!(
        error.boundary_diagnostic().rule,
        DiagnosticRule::StudyBindingPhysical
    );
}

/// Acknowledged durable source plus a closed real caller makes accidental recording
/// observable as a refusal. Local science must complete with all recording effects denied.
#[cfg(all(
    feature = "canonical-tests",
    any(feature = "solver-diffsol", feature = "solver-ipopt")
))]
async fn deny_recording_after_durable_source(package: &ModelingPackage) {
    assert!(matches!(
        package.runtime.durability(),
        super::super::Durability::Durable(_)
    ));
    let origin = package.canonical_revision().unwrap();
    let store = package.runtime.canonical_store().unwrap();
    assert_eq!(
        store.revision(&origin.key).await.unwrap().unwrap().key,
        origin.key
    );
    store.close();
    assert!(
        store.admit_activity().is_err(),
        "the actual source/run publication caller is closed"
    );
    assert!(
        store.revision(&origin.key).await.is_err(),
        "the denial control also refuses a real canonical read"
    );
}

#[cfg(all(
    feature = "canonical-tests",
    any(feature = "solver-diffsol", feature = "solver-ipopt")
))]
fn original_local_result(report: &super::super::StudyReport, index: usize) -> &RunResult {
    let outcome = &report.outcomes[index];
    assert!(outcome.scientific.usable, "{outcome:?}");
    assert!(outcome.diagnostic.is_none(), "{outcome:?}");
    let occurrence = report.results[index].as_ref().unwrap();
    assert!(occurrence.stored_keys().is_none());
    let super::super::StudyOccurrenceResult::Ephemeral(result) = occurrence else {
        panic!("full local study must retain its original ephemeral result: {occurrence:?}");
    };
    assert!(matches!(
        result.durability(),
        super::super::RunDurability::Ephemeral
    ));
    assert!(result.canonical_run_key().is_none());
    assert!(result.canonical_attempt_key().is_none());
    assert!(result.usable(), "{:?}", result.assessments());
    result
}

#[cfg(all(feature = "canonical-tests", feature = "solver-diffsol"))]
#[tokio::test]
async fn durable_origin_local_simulation_keeps_original_full_and_compact_results_without_recording()
{
    use super::super::{RunReport, StudyDetailDemand, StudyInputs, StudyOutputDemand};
    let (package, case) = plant_on(super::super::durable_tests::durable_runtime()).await;
    let origin = package.canonical_revision().unwrap().key.clone();
    deny_recording_after_durable_source(&package).await;
    let cancel = CancelSource::new();
    let inputs = StudyInputs::Explicit(vec![point(OperationRequest::Simulation(
        SimulationOperation {
            case,
            settings: None,
        },
    ))]);
    let full = package
        .admit_study(&inputs, &StudyOutputDemand::Full {}, &cancel)
        .await
        .unwrap();
    let report = package.study(&full, 1, &cancel).await.unwrap();
    let original = original_local_result(&report, 0);
    let RunReport::Simulation(trajectory) = original.report().unwrap() else {
        panic!("original simulation");
    };
    assert_eq!(trajectory.report().samples.len(), 2);
    assert_eq!(trajectory.report().samples.last().unwrap().time, 1.0);
    assert!((trajectory.report().samples.last().unwrap().state[0] - 1.0).abs() < 1e-8);

    let compact = package
        .admit_study(
            &inputs,
            &StudyOutputDemand::Compact {
                detail: StudyDetailDemand::default(),
                failed_detail: StudyDetailDemand::default(),
            },
            &cancel,
        )
        .await
        .unwrap();
    let report = compact.execute(1, &cancel).await.unwrap();
    assert!(
        report.outcomes[0].scientific.usable,
        "{:?}",
        report.outcomes[0]
    );
    let occurrence = report.results[0].as_ref().unwrap();
    assert!(matches!(
        occurrence,
        super::super::StudyOccurrenceResult::Compact(_)
    ));
    assert!(occurrence.stored_keys().is_none());
    occurrence.completion().unwrap();
    assert_eq!(
        package.canonical_revision().unwrap().key,
        origin,
        "local execution preserves its original source attribution"
    );
    assert!(
        package
            .runtime
            .canonical_store()
            .unwrap()
            .admit_activity()
            .is_err()
    );
}

#[cfg(all(feature = "canonical-tests", feature = "solver-diffsol"))]
#[tokio::test]
async fn durable_origin_local_lazy_simulation_constructs_only_selected_occurrences_without_recording()
 {
    use super::super::{
        RunReport, SelectedStudyVariant, SourceChoice, SourceDelta, StudyInputs, StudyOutputDemand,
        StudyVariants,
    };
    let (package, case) = plant_on(super::super::durable_tests::durable_runtime()).await;
    let requested = |key| {
        let mut selected = point(OperationRequest::Simulation(SimulationOperation {
            case,
            settings: None,
        }));
        selected.policy.key = OccurrenceKey(key);
        selected
    };
    let variants = StudyVariants {
        version: Version,
        source: OperationSource::of(&package),
        choices: vec![SourceChoice {
            alternatives: vec![
                SourceDelta {
                    parent: package.revision_id().into(),
                    ..Default::default()
                },
                SourceDelta {
                    parent: package.revision_id().into(),
                    ..Default::default()
                },
                // Unselected invalid science must neither construct nor publish a source.
                SourceDelta {
                    parent: package.revision_id().into(),
                    removals: vec![DeclarationId::from_bytes([0; 16])],
                    ..Default::default()
                },
            ],
        }],
        selected: vec![
            SelectedStudyVariant {
                ordinal: 0,
                point: requested(23),
            },
            SelectedStudyVariant {
                ordinal: 1,
                point: requested(5),
            },
        ],
    };
    deny_recording_after_durable_source(&package).await;
    let cancel = CancelSource::new();
    let before = package.runtime.shared.math().preparations().views;
    let local = package
        .admit_study(
            &StudyInputs::Lazy(variants),
            &StudyOutputDemand::Full {},
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(
        package.runtime.shared.math().preparations().views,
        before,
        "lazy admission does not construct selected science"
    );
    let report = local.execute(2, &cancel).await.unwrap();
    assert_eq!(
        report
            .outcomes
            .iter()
            .map(|outcome| outcome.key)
            .collect::<Vec<_>>(),
        vec![OccurrenceKey(23), OccurrenceKey(5)]
    );
    for index in 0..2 {
        let original = original_local_result(&report, index);
        let RunReport::Simulation(trajectory) = original.report().unwrap() else {
            panic!("original selected simulation");
        };
        assert_eq!(trajectory.report().samples.last().unwrap().time, 1.0);
        assert!((trajectory.report().samples.last().unwrap().state[0] - 1.0).abs() < 1e-8);
    }
    assert_ne!(
        report.results[0].as_ref().unwrap().run_id(),
        report.results[1].as_ref().unwrap().run_id()
    );
    assert!(
        package
            .runtime
            .canonical_store()
            .unwrap()
            .admit_activity()
            .is_err()
    );
}

#[cfg(all(feature = "canonical-tests", feature = "solver-ipopt"))]
#[tokio::test]
async fn durable_origin_local_fit_keeps_original_regression_without_recording() {
    use super::super::{RunReport, StudyInputs, StudyOutputDemand, fitting::regression};
    use pse_backend_native::solve::{Backend, SolveIntent};
    let package = regression::package_on(
        super::super::durable_tests::durable_runtime(),
        false,
        regression::declared(),
        [1.; 4],
    )
    .await;
    deny_recording_after_durable_source(&package).await;
    let request = OperationRequest::Fit(FitOperation {
        fit: super::super::tests::id(32).into(),
        settings: crate::workflow::PhysicalFitRequest::new(FitPreparationDocument {
            optimizer: crate::workflow::FitOptimizer::NativeNlp,
            solver: SolveSettings {
                intent: SolveIntent::Optimize,
                backend: Some(Backend::Ipopt),
                ..Default::default()
            },
            simulations: BTreeMap::new(),
            rank_tolerance: 1e-8,
            max_cells: 100_000,
            derivatives: FitDerivatives::Responses,
            experiment_workers: pse_model::scalar!(PositiveCount(1)),
            uncertainty: None,
        }),
    });
    let cancel = CancelSource::new();
    let local = package
        .admit_study(
            &StudyInputs::Explicit(vec![point(request)]),
            &StudyOutputDemand::Full {},
            &cancel,
        )
        .await
        .unwrap();
    let report = local.execute(1, &cancel).await.unwrap();
    let original = original_local_result(&report, 0);
    let RunReport::Fit(fit) = original.report().unwrap() else {
        panic!("original regression fit");
    };
    let (_, expected) = regression::analytic();
    let candidate = fit.candidate.as_ref().unwrap();
    assert_eq!(candidate.len(), expected.len());
    for (actual, expected) in candidate.iter().zip(expected) {
        assert!((actual - expected).abs() < 1e-6);
    }
    assert_eq!(fit.predictions.len(), 4);
    assert!(fit.checks_complete);
    assert!(
        package
            .runtime
            .canonical_store()
            .unwrap()
            .admit_activity()
            .is_err()
    );
}

#[cfg(all(feature = "canonical-tests", feature = "solver-diffsol"))]
#[tokio::test]
async fn durable_origin_local_horizon_keeps_original_two_step_plant_without_recording() {
    use super::super::{HorizonDecision, StudyInputs, StudyOutputDemand};
    let (package, case) = plant_on(super::super::durable_tests::durable_runtime()).await;
    let request = horizon(&package, case).await;
    deny_recording_after_durable_source(&package).await;
    let cancel = CancelSource::new();
    let local = package
        .admit_study(
            &StudyInputs::Explicit(vec![point(OperationRequest::Horizon(Box::new(request)))]),
            &StudyOutputDemand::Full {},
            &cancel,
        )
        .await
        .unwrap();
    let report = local.execute(1, &cancel).await.unwrap();
    let original = original_local_result(&report, 0);
    let horizon = original.horizon().unwrap();
    assert_eq!(horizon.completed_steps, 2);
    assert_eq!(horizon.steps.len(), 2);
    for (index, step) in horizon.steps.iter().enumerate() {
        assert_eq!(step.time, index as f64 * 0.5);
        assert_eq!(step.decision, HorizonDecision::Solved);
        assert_eq!(step.applied.len(), 1);
        assert!((step.applied[0] - 353.15).abs() < 1e-6);
        assert_eq!(step.reached.len(), 1);
        assert!((step.reached[0] - (1.0 + 53.15 * (index + 1) as f64 * 0.5)).abs() < 1e-5);
    }
    assert!(
        package
            .runtime
            .canonical_store()
            .unwrap()
            .admit_activity()
            .is_err()
    );
}
