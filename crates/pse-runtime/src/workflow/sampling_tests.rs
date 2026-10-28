// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::super::tests as fixture;
use super::*;
use pse_ids::SemanticId;
fn finite(value: f64) -> FiniteBound {
    FiniteBound::try_new(value).unwrap()
}
async fn fixture(source: &str) -> (super::super::Runtime, ModelingPackage, DesignRequest) {
    let runtime = fixture::runtime_with(256 << 20, 1 << 20, 1 << 30);
    let rows = pse_authoring::language::parse(
        source,
        SemanticId::NIL,
        pse_authoring::language::IdentityPolicy::Named,
        Default::default(),
    )
    .unwrap();
    let root = rows
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    let package = runtime
        .modeling_package(rows, fixture::physical())
        .await
        .unwrap();
    let quantity = package
        .quantities
        .quantity_types()
        .find(|q| q.name.as_deref() == Some("Scalar"))
        .unwrap();
    let request = DesignRequest {
        root,
        bindings: Default::default(),
        coordinates: vec![DesignCoordinate {
            target: BindingTarget::Path("x".into()),
            quantity: quantity.id,
            unit: quantity.canonical_unit,
            range: Some([finite(0.0), finite(1.0)]),
            levels: vec![],
        }],
        kind: DesignKind::Classic,
        count: 8,
        seed: 91,
        grid_levels: vec![],
        maximum_work: 1_000_000,
    };
    (runtime, package, request)
}
const SOURCE: &str = "package p { def Root { var x:Scalar; annotation bounds x(0,1); annotation start x(0.5); eq root:x==0.5; } }";
async fn generate(
    runtime: &super::super::Runtime,
    package: &ModelingPackage,
    request: &DesignRequest,
) -> GeneratedDesign {
    package
        .generate_design(
            &runtime.preparation_services(),
            request,
            Default::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap()
}
#[tokio::test]
async fn seeded_normalized_kinds_preserve_whole_design_strata_and_charge_aliases() {
    let (runtime, package, mut request) = fixture(SOURCE).await;
    for kind in [
        DesignKind::Classic,
        DesignKind::Centered,
        DesignKind::Random,
        DesignKind::Maximin,
        DesignKind::CenteredMaximin,
    ] {
        request.kind = kind;
        let design = generate(&runtime, &package, &request).await;
        let again = generate(&runtime, &package, &request).await;
        assert_eq!(design.points(), again.points());
        assert_eq!(design.points().len(), 8);
        assert!(design.points().iter().all(|p| p.assignments.len() == 1));
        if kind != DesignKind::Random {
            let mut strata = design
                .points()
                .iter()
                .map(|p| (p.assignments[0].value.magnitude.into_inner() * 8.) as usize)
                .collect::<Vec<_>>();
            strata.sort_unstable();
            assert_eq!(strata, (0..8).collect::<Vec<_>>());
        }
        let owner = Arc::downgrade(&design._owner);
        let alias = design.clone();
        drop(design);
        assert!(owner.upgrade().is_some());
        drop(alias);
        assert!(owner.upgrade().is_none());
    }
}
#[tokio::test]
async fn explicit_grid_preserves_endpoints_order_and_duplicate_discrete_occurrences() {
    let (runtime, package, mut request) = fixture(SOURCE).await;
    request.kind = DesignKind::Grid;
    request.count = 3;
    request.grid_levels = vec![3];
    let design = generate(&runtime, &package, &request).await;
    assert_eq!(
        design
            .points()
            .iter()
            .map(|p| p.assignments[0].value.magnitude.into_inner())
            .collect::<Vec<_>>(),
        vec![0.0, 0.5, 1.0]
    );
    request.kind = DesignKind::Classic;
    request.count = 20;
    request.grid_levels.clear();
    request.coordinates[0].levels = vec![DesignLevel::Integer(0), DesignLevel::Integer(1)];
    let design = generate(&runtime, &package, &request).await;
    assert_eq!(design.points().len(), 20);
    assert_eq!(
        design
            .points()
            .iter()
            .filter(|p| p.assignments[0].value.magnitude.into_inner() == 0.0)
            .count(),
        10
    );
    let export = design.export(&runtime.preparation_services()).unwrap();
    assert_eq!(export.points, design.points());
    let source = Arc::downgrade(&design.inner);
    drop(design);
    assert!(source.upgrade().is_some());
    drop(export);
    assert!(source.upgrade().is_none());
}
#[tokio::test]
async fn malformed_ranges_targets_units_levels_and_work_refuse_before_generation() {
    let (runtime, package, request) = fixture(SOURCE).await;
    let refuse = async |next: DesignRequest| {
        assert!(
            package
                .generate_design(
                    &runtime.preparation_services(),
                    &next,
                    Default::default(),
                    &crate::CancelSource::new()
                )
                .await
                .is_err()
        );
    };
    let mut next = request.clone();
    next.coordinates[0].range = Some([finite(2.), finite(1.)]);
    refuse(next).await;
    let mut next = request.clone();
    next.coordinates[0].range = Some([finite(-1.), finite(1.)]);
    refuse(next).await;
    let mut next = request.clone();
    next.coordinates[0].range = Some([finite(-1.), finite(1.)]);
    next.coordinates[0].levels = vec![DesignLevel::Integer(0), DesignLevel::Integer(1)];
    refuse(next).await;
    let mut next = request.clone();
    next.coordinates.push(next.coordinates[0].clone());
    refuse(next).await;
    let mut next = request.clone();
    next.coordinates[0].target = BindingTarget::Path("missing".into());
    refuse(next).await;
    let mut next = request.clone();
    next.coordinates[0].unit = UnitId::from_id(SemanticId::NIL);
    refuse(next).await;
    let mut next = request.clone();
    next.coordinates[0].levels = vec![DesignLevel::Integer(9_007_199_254_740_993)];
    refuse(next).await;
    let mut next = request.clone();
    next.kind = DesignKind::Maximin;
    next.count = 1;
    refuse(next).await;
    let mut next = request.clone();
    next.kind = DesignKind::Grid;
    next.grid_levels = vec![usize::MAX];
    refuse(next).await;
    let mut next = request;
    next.maximum_work = 1;
    refuse(next).await;
}
#[tokio::test]
async fn singleton_and_default_authored_bounds_keep_actual_original_values() {
    let (runtime, package, mut request) = fixture(SOURCE).await;
    request.coordinates[0].range = None;
    let default = generate(&runtime, &package, &request).await;
    assert!(
        default
            .points()
            .iter()
            .all(|p| (0.0..=1.0).contains(&p.assignments[0].value.magnitude.into_inner()))
    );
    request.coordinates[0].range = Some([finite(-0.0), finite(0.0)]);
    let zero = generate(&runtime, &package, &request).await;
    assert!(zero.points().iter().all(|p|p.assignments[0].value.magnitude.into_inner().to_bits()==(-0.0f64).to_bits()));
    let source = crate::CancelSource::new();
    source.cancel();
    assert!(
        package
            .generate_design(
                &runtime.preparation_services(),
                &request,
                Default::default(),
                &source
            )
            .await
            .is_err()
    );
}
#[test]
fn exact_integer_levels_distinguish_representable_large_values() {
    assert_eq!(
        DesignLevel::Integer(9_007_199_254_740_992).value(),
        Some(9_007_199_254_740_992.0)
    );
    assert_eq!(DesignLevel::Integer(9_007_199_254_740_993).value(), None);
    assert_eq!(
        DesignLevel::Integer(9_007_199_254_740_994).value(),
        Some(9_007_199_254_740_994.0)
    );
    assert_eq!(DesignLevel::Integer(i64::MAX).value(), None);
}

#[tokio::test]
async fn declared_discrete_and_semi_domains_require_actual_admissible_levels() {
    for (domain, lower, upper, levels) in [
        (
            "integer",
            0.,
            3.,
            vec![DesignLevel::Integer(0), DesignLevel::Integer(3)],
        ),
        (
            "binary",
            0.,
            1.,
            vec![DesignLevel::Integer(0), DesignLevel::Integer(1)],
        ),
        (
            "semicontinuous",
            2.,
            5.,
            vec![
                DesignLevel::Integer(0),
                DesignLevel::Real(finite(2.5)),
                DesignLevel::Integer(5),
            ],
        ),
        (
            "semiinteger",
            2.,
            5.,
            vec![
                DesignLevel::Integer(0),
                DesignLevel::Integer(2),
                DesignLevel::Integer(5),
            ],
        ),
    ] {
        // Integral domains retain the quantity owner's Count/Indicator eligibility.
        let ty = if domain == "semicontinuous" {
            "Scalar"
        } else {
            "Count"
        };
        let source = format!(
            "package p {{ def Root {{ var x:{ty} in {domain}; annotation bounds x({lower}{{1}},{upper}{{1}}); eq root:x==x; }} }}"
        );
        let (runtime, package, mut request) = fixture(&source).await;
        let quantity = package
            .quantities
            .quantity_types()
            .find(|q| q.name.as_deref() == Some(ty))
            .unwrap();
        request.coordinates[0].quantity = quantity.id;
        request.coordinates[0].unit = quantity.canonical_unit;
        request.coordinates[0].range = Some([finite(0.), finite(upper)]);
        assert!(
            package
                .generate_design(
                    &runtime.preparation_services(),
                    &request,
                    Default::default(),
                    &crate::CancelSource::new()
                )
                .await
                .is_err()
        );
        request.kind = DesignKind::Grid;
        request.count = levels.len();
        request.grid_levels = vec![levels.len()];
        request.coordinates[0].levels = levels.clone();
        let generated = generate(&runtime, &package, &request).await;
        assert_eq!(
            generated
                .points()
                .iter()
                .map(|point| point.assignments[0].value.magnitude.into_inner())
                .collect::<Vec<_>>(),
            levels
                .iter()
                .map(|level| level.value().unwrap())
                .collect::<Vec<_>>()
        );
        let invalid = if domain == "binary" {
            2.
        } else if domain == "semicontinuous" {
            1.
        } else {
            0.5
        };
        request.coordinates[0].levels = vec![DesignLevel::Real(finite(invalid))];
        request.grid_levels = vec![1];
        request.count = 1;
        assert!(
            package
                .generate_design(
                    &runtime.preparation_services(),
                    &request,
                    Default::default(),
                    &crate::CancelSource::new()
                )
                .await
                .is_err()
        );
    }
}

#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn complete_design_feeds_original_multistart_rungs_without_changing_target() {
    use crate::math::solves::{NumericalInputs, PreparedRung, SolverProfile};
    use pse_backend_native::solve::{Backend, SolveIntent, SolverSelection};
    use pse_model::strategy::*;
    let (runtime,package,mut request)=fixture("package p { def Root { var x:Scalar; var y:Scalar; annotation start x(0.5); annotation start y(2); eq first:x==0.5; eq second:y==2; } }").await;
    request.kind = DesignKind::Grid;
    request.count = 3;
    request.grid_levels = vec![3];
    let design = generate(&runtime, &package, &request).await;
    let services = runtime.preparation_services();
    // Human-readable paths are registered only by explicit source demands.
    let bindings = pse_modeling::Bindings {
        demand: vec!["x".into(), "y".into()],
        ..request.bindings.clone()
    };
    let prepared = package
        .prepare_solve(
            &services,
            request.root,
            pse_modeling::specialize::root_instance(request.root),
            bindings,
            Default::default(),
            Default::default(),
            pse_kernels::DerivativeOrder::First,
            fixture::compiler_profile(),
            SolverProfile {
                intent: SolveIntent::Root,
                selection: SolverSelection::Explicit(Backend::Kinsol),
                ..Default::default()
            },
            NumericalInputs::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    // Use the public original case projection and its resolved specification values.
    // A partial design is not itself a complete multistart seed.
    let coordinates = prepared.model.case.compiled().plan.columns().to_vec();
    let baseline = coordinates
        .iter()
        .map(|id| (*id, prepared.model.values.scalars[id]))
        .collect::<std::collections::BTreeMap<_, _>>();
    let model = prepared.model.model;
    let original = prepared.solve;
    assert_eq!(original.original_coordinates().unwrap(), coordinates);
    let y = model.compiled().model.paths["y"];
    assert_eq!(baseline.len(), 2);
    assert_eq!(baseline[&y], 2.0);
    let identity = original.original_identity().unwrap();
    let scope = pse_kernels::ExecutionScope::new(
        pse_diagnostics::Cancellation::default(),
        Some(std::time::Instant::now() + std::time::Duration::from_secs(30)),
    );
    let mut strategy = NumericalStrategy::direct(
        original.replay_profile().controls.start,
        WorkLimits {
            attempts: 4,
            evaluations: None,
            iterations: None,
            factorizations: None,
            proof_steps: None,
        },
    );
    strategy.start.recovery = vec![StartOrigin::ModifiedSpecification];
    strategy.mechanisms[0].profile = Some(ProfileRef {
        method: NumericalMethod::Backend(Backend::Kinsol),
        key: original.strategy_profile().unwrap(),
    });
    let mut rungs = vec![PreparedRung::from(original.clone())];
    for overlay in design.points() {
        let binding = package.admit_overlay(model.compiled(), overlay).unwrap();
        assert_eq!(binding.entries.len(), 1);
        let mut seed = baseline.clone();
        seed.extend(binding.values());
        assert_eq!(seed.len(), 2);
        assert_eq!(seed[&y], 2.0);
        let expected = coordinates.iter().map(|id| seed[id]).collect::<Vec<_>>();
        let rung = runtime
            .native()
            .prepare_multistart(
                original.clone(),
                seed,
                BranchPolicy::any_qualified(),
                scope.clone(),
            )
            .unwrap();
        assert_eq!(rung.point(), expected);
        assert_eq!(rung.original_identity().unwrap(), identity);
        let mut mechanism = strategy.mechanisms[0].clone();
        mechanism.kind = MechanismKind::Multistart;
        mechanism.position = Position::Execution;
        mechanism.profile = Some(rung.profile_ref().unwrap());
        mechanism.support = rung.support().unwrap().into_iter().collect();
        mechanism.starts = vec![StartOrigin::ModifiedSpecification];
        strategy.mechanisms.push(mechanism);
        rungs.push(PreparedRung::Multistart(rung));
    }
    let composed = original
        .within_task(scope)
        .unwrap()
        .with_strategy(strategy, rungs)
        .unwrap();
    assert_eq!(composed.original_identity().unwrap(), identity);
}

#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn generated_occurrences_use_normal_study_admission_and_keep_repeated_identities() {
    use super::super::{
        CaseOperation, OperationRequest, PreparationSettings, StudyInputs, StudyOutputDemand,
        StudyPoint, StudyPointPolicy,
    };
    use pse_model::study::{OccurrenceKey, StartPolicy};
    let (runtime, package, mut request) = fixture(SOURCE).await;
    request.count = 6;
    request.coordinates[0].levels = vec![DesignLevel::Integer(0), DesignLevel::Integer(1)];
    let design = generate(&runtime, &package, &request).await;
    let points = design
        .points()
        .iter()
        .enumerate()
        .map(|(index, overlay)| StudyPoint {
            operation: OperationRequest::DeclaredCase(CaseOperation {
                case: request.root,
                route: pse_model::generated::enums::ModelingAnalysisRoute::Steady,
                settings: crate::math::settings::SolveSettings {
                    backend: Some(pse_backend_native::solve::Backend::Kinsol),
                    intent: pse_backend_native::solve::SolveIntent::Root,
                    ..Default::default()
                },
            }),
            preparation: PreparationSettings {
                compiler: fixture::compiler_profile(),
                ..Default::default()
            },
            overlay: overlay.clone(),
            policy: StudyPointPolicy {
                key: OccurrenceKey(index as u32),
                dependencies: vec![],
                start: StartPolicy::Fresh,
                attempt_limit: 1,
            },
        })
        .collect();
    let admitted = package
        .admit_study(
            &runtime.execution_services(),
            &StudyInputs::Explicit(points),
            &StudyOutputDemand::Full {},
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    assert_eq!(admitted.points.len(), 6);
    for (index, point) in admitted.points.iter().enumerate() {
        assert_eq!(point.policy.key, OccurrenceKey(index as u32));
        let original = design.points()[index].assignments[0]
            .value
            .magnitude
            .into_inner();
        let canonical = point
            .binding
            .entries
            .values()
            .next()
            .unwrap()
            .canonical
            .into_inner();
        assert_eq!(original.to_bits(), canonical.to_bits());
        let matching = admitted
            .points
            .iter()
            .filter(|candidate| candidate.binding_hash == point.binding_hash)
            .count();
        assert_eq!(matching, 3);
    }
}

#[tokio::test]
async fn complete_design_initializes_actual_surrogate_in_original_physical_order() {
    use crate::math::surrogate::*;
    use pse_backend_native::ProblemError;
    use pse_ids::ContentHash;
    #[derive(Debug)]
    struct Evaluate {
        correspondence: FidelityCorrespondence,
        seen: Arc<std::sync::Mutex<Vec<Vec<f64>>>>,
    }
    impl FidelityEvaluator for Evaluate {
        type Error = ProblemError;
        fn correspondence(&self) -> &FidelityCorrespondence {
            &self.correspondence
        }
        fn evaluate(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            self.seen.lock().unwrap().push(x.to_vec());
            out[0] = (x[0] - 0.37).powi(2);
            Ok(())
        }
    }
    let (runtime, package, mut request) = fixture(SOURCE).await;
    request.kind = DesignKind::Grid;
    request.count = 3;
    request.grid_levels = vec![3];
    let design = generate(&runtime, &package, &request).await;
    let initial = design
        .points()
        .iter()
        .map(|p| {
            p.assignments
                .iter()
                .map(|a| a.value.magnitude.into_inner())
                .collect()
        })
        .collect::<Vec<Vec<f64>>>();
    let expected = vec![vec![0.], vec![0.5], vec![1.]];
    assert_eq!(initial, expected);
    let hash = ContentHash::from_bytes([7; 32]);
    let seen = Arc::new(std::sync::Mutex::new(Vec::new()));
    let evaluator = Evaluate {
        correspondence: FidelityCorrespondence {
            original_target: hash,
            model_target: hash,
            source: hash,
            fidelity: hash,
            coordinates: vec![SemanticId::from_bytes([8; 16])],
            outputs: vec![SemanticId::from_bytes([9; 16])],
        },
        seen: seen.clone(),
    };
    let service = runtime.native();
    let options = SurrogateOptions {
        bounds: vec![(0., 1.)],
        initial,
        iterations: 2,
        evaluations: 20,
        infill_starts: 1,
        regression: RegressionSpec::CONSTANT,
        correlation: CorrelationSpec::MATERN52,
        theta: ThetaTuning::Fixed(vec![0.1].into()),
        gp_starts: 0,
        gp_evaluations: 5,
        seed: 74,
        phase_steps: (1, 1),
        radius: 0.2,
        contraction: 0.5,
        threads: 1,
        stack_bytes: service.stack_bytes(),
        foreign_bytes: service.policy().foreign_bytes,
        time: std::time::Duration::from_secs(20),
    };
    let scope = pse_kernels::ExecutionScope::new(
        pse_diagnostics::Cancellation::default(),
        Some(std::time::Instant::now() + std::time::Duration::from_secs(30)),
    );
    let cancel = crate::CancelSource::new();
    let handle = service
        .prepare_surrogate(evaluator, options, scope, &cancel)
        .await
        .unwrap();
    service.advance_surrogate(&handle, &cancel).await.unwrap();
    let observed = seen.lock().unwrap();
    assert!(observed.len() >= 3);
    assert_eq!(&observed[..3], &expected);
    drop(observed);
    drop(handle);
}

#[tokio::test]
async fn grid_cartesian_order_is_last_coordinate_fast_and_product_overflow_refuses() {
    let (runtime,package,mut request)=fixture("package p { def Root { var x:Scalar; var y:Scalar; annotation bounds x(0,1); annotation bounds y(0,1); eq balance:x+y==1; } }").await;
    let mut y = request.coordinates[0].clone();
    y.target = BindingTarget::Path("y".into());
    request.coordinates.push(y);
    request.kind = DesignKind::Grid;
    request.grid_levels = vec![2, 3];
    request.count = 6;
    let design = generate(&runtime, &package, &request).await;
    let values = design
        .points()
        .iter()
        .map(|p| {
            p.assignments
                .iter()
                .map(|a| a.value.magnitude.into_inner())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        values,
        vec![
            vec![0., 0.],
            vec![0., 0.5],
            vec![0., 1.],
            vec![1., 0.],
            vec![1., 0.5],
            vec![1., 1.]
        ]
    );
    request.grid_levels = vec![usize::MAX, usize::MAX];
    let error = package
        .generate_design(
            &runtime.preparation_services(),
            &request,
            Default::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap_err();
    assert!(error.to_string().contains("grid product overflow"));
}

#[tokio::test]
async fn affine_default_bounds_construct_inside_the_actual_canonical_conversion_image() {
    let (runtime,package,mut request)=fixture("package p { def Root { var x:Temperature; annotation bounds x(273.15{K},373.15{K}); eq balance:x==300{K}; } }").await;
    let quantity = package
        .quantities
        .quantity_types()
        .find(|q| q.name.as_deref() == Some("Temperature"))
        .unwrap();
    let unit = package
        .quantities
        .units()
        .find(|u| u.symbol == "degC")
        .unwrap();
    request.coordinates[0].quantity = quantity.id;
    request.coordinates[0].unit = unit.id;
    request.coordinates[0].range = None;
    request.kind = DesignKind::Grid;
    request.count = 11;
    request.grid_levels = vec![11];
    let design = generate(&runtime, &package, &request).await;
    let mut bindings = request.bindings.clone();
    // Prepare the same demanded path: specialization registers human-readable
    // input names only when they are explicitly requested.
    bindings.demand.push("x".into());
    let model = package
        .prepare(
            &runtime.preparation_services(),
            request.root,
            pse_modeling::specialize::root_instance(request.root),
            bindings,
            Default::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    for point in design.points() {
        let binding = package.admit_overlay(model.compiled(), point).unwrap();
        let actual = binding
            .entries
            .values()
            .next()
            .unwrap()
            .canonical
            .into_inner();
        assert!((273.15..=373.15).contains(&actual));
    }
    // Independent application of the quantity owner also covers default ranges for which
    // direct inverse rounding can produce an out-of-domain endpoint.
    let plan =
        CanonicalConversionPlan::registered(&package.quantities, quantity.id, unit.id).unwrap();
    for (a, b) in [
        (273.15, 373.15),
        (273.15_f64.next_up(), 273.15_f64.next_up() + 1.),
        (-1.0e100, 1.0e100),
    ] {
        let [lo, hi] = default_range(&plan, a, b).unwrap();
        for i in 0..=32 {
            let input = interpolate(lo, hi, i as f64 / 32.);
            let output = plan.apply(input).unwrap().value();
            assert!(output >= a && output <= b);
        }
    }
}
