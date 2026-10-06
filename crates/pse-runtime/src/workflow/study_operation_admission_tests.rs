// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Study descriptors preserve admission through the existing general operation owners.
#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "native admission controls fail on invalid setup or unexpected operation variants"
)]
use super::super::{
    BindingQuantity, PointOverlay, PreparedStudyOperation, StudyPoint, StudyPointPolicy,
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
    physical.preconditions = std::sync::Arc::new(
        pse_quantity::PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions())
            .unwrap(),
    );
    physical.key =
        pse_compiler::workspace::physical_identity(&physical.quantities, &physical.preconditions);
    (
        super::super::tests::runtime()
            .modeling_package(rows, physical)
            .await
            .unwrap(),
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
            profile: None,
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
        .admit_study_points(
            super::super::PhysicalSource {
                revision: "explicit-ephemeral-fixture".into(),
                identity: ContentHash::from_bytes([0; 32]),
            },
            &[point(OperationRequest::Simulation(SimulationOperation {
                case,
                profile: None,
            }))],
            &CancelSource::new(),
        )
        .await
        .unwrap();
    let decoded: super::super::StudyDefinition =
        serde_json::from_slice(&serde_json::to_vec(&definition).unwrap()).unwrap();
    let p = &decoded.points[0];
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
        settings: FitOperationSettings {
            solver: SolveSettings {
                intent: SolveIntent::Optimize,
                backend: Some(Backend::Ipopt),
                ..Default::default()
            },
            simulations: BTreeMap::new(),
            rank_tolerance: 1e-8,
            max_cells: 100_000,
            derivatives: FitDerivatives::Responses,
            uncertainty: None,
        },
    });
    let definition = package
        .admit_study_points(
            super::super::PhysicalSource {
                revision: "explicit-ephemeral-fixture".into(),
                identity: ContentHash::from_bytes([0; 32]),
            },
            &[point(request)],
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
        .admit_study_points(
            super::super::PhysicalSource {
                revision: "explicit-ephemeral-fixture".into(),
                identity: ContentHash::from_bytes([0; 32]),
            },
            &[point(OperationRequest::Horizon(Box::new(request.clone())))],
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
        .admit_study_points(
            super::super::PhysicalSource {
                revision: "explicit-ephemeral-fixture".into(),
                identity: ContentHash::from_bytes([0; 32]),
            },
            &[point(OperationRequest::Horizon(Box::new(equivalent)))],
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
        .admit_study_points(
            super::super::PhysicalSource {
                revision: "explicit-ephemeral-fixture".into(),
                identity: ContentHash::from_bytes([0; 32]),
            },
            &[point(OperationRequest::Horizon(Box::new(wrong)))],
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
        operation
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
