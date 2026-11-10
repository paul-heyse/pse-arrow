// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(
    clippy::unwrap_used,
    reason = "finite expectation controls require admitted original fixtures"
)]
use super::super::{
    BindingQuantity, BindingTarget, OperationSource, PreparationSettings, Runtime,
    SelectedObservationOperation, SelectedObservationOutput,
};
use super::*;
use pse_ids::SemanticId;
use pse_model::{generated::identities::StudyId, scalars::FiniteBound};

fn id(value: u8) -> SemanticId {
    SemanticId::from_bytes([value; 16])
}
fn study(value: u8) -> StudyId {
    StudyId::from_id(id(value))
}
fn quantity(
    registry: &pse_quantity::QuantityRegistry,
    name: &str,
    value: f64,
    unit: Option<&str>,
) -> BindingQuantity {
    let quantity = registry
        .quantity_types()
        .find(|q| q.name.as_deref() == Some(name))
        .unwrap();
    let unit = unit.map_or(quantity.canonical_unit, |symbol| {
        registry.units().find(|u| u.symbol == symbol).unwrap().id
    });
    BindingQuantity {
        quantity: quantity.id,
        unit,
        magnitude: FiniteBound::try_new(value).unwrap(),
    }
}

async fn fixture(
    adaptive: bool,
    samples: usize,
) -> (Runtime, super::super::ModelingPackage, ExpectationRequest) {
    let runtime = super::super::tests::runtime();
    let rows = pse_authoring::language::parse("package expectation_fixture {def Root {var x:Scalar;var category:Scalar;param origin:Temperature=300{K};param span:DeltaTemperature=20{K};let value:Scalar=x*category;let delta:DeltaTemperature=span*x;let temperature:Temperature=origin+delta;}def Joint {var u:Scalar;let x:Scalar=u;let category:Scalar=1+u;}}", id(123), pse_authoring::language::IdentityPolicy::Named, Default::default()).unwrap();
    let root = rows
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let package = runtime
        .modeling_package(rows, super::super::tests::physical())
        .await
        .unwrap();
    let services = runtime.preparation_services();
    let preparation = PreparationSettings {
        compiler: super::super::tests::compiler_profile(),
        ..Default::default()
    };
    let bindings = pse_modeling::Bindings {
        demand: vec![
            "x".into(),
            "category".into(),
            "value".into(),
            "temperature".into(),
        ],
        ..Default::default()
    };
    let model = package
        .prepare(
            &services,
            root,
            pse_modeling::specialize::root_instance(root),
            bindings.clone(),
            preparation.limits,
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let member = |name: &str| {
        model
            .compiled()
            .model
            .symbols
            .values()
            .find(|s| s.lineage.path == format!("Root.{name}"))
            .unwrap()
            .id
    };
    let scalar = quantity(&package.quantities, "Scalar", 1., None);
    let output = member("value");
    let request = ExpectationRequest {
        source: OperationSource::of(&package),
        preparation,
        member: ExpectationMember::Observation(SelectedObservationOperation {
            root,
            instance: pse_modeling::specialize::root_instance(root),
            bindings,
            outputs: vec![SelectedObservationOutput {
                member: BindingTarget::Member(output),
                quantity: scalar.quantity,
                unit: scalar.unit,
            }],
        }),
        coordinates: vec![
            ExpectationCoordinate::Uniform {
                target: BindingTarget::Member(member("x")),
                lower: quantity(&package.quantities, "Scalar", 0., None),
                upper: scalar.clone(),
            },
            ExpectationCoordinate::Categorical {
                target: BindingTarget::Member(member("category")),
                categories: vec![
                    ExpectationCategory {
                        category: id(1),
                        value: scalar.clone(),
                        probability: 0.25,
                    },
                    ExpectationCategory {
                        category: id(2),
                        value: quantity(&package.quantities, "Scalar", 2., None),
                        probability: 0.75,
                    },
                    ExpectationCategory {
                        category: id(3),
                        value: scalar.clone(),
                        probability: 0.,
                    },
                ],
            },
        ],
        transform: None,
        outputs: vec![ExpectationOutput {
            output,
            quantity: scalar.quantity,
            unit: scalar.unit,
            reference: None,
            scale: scalar,
            absolute_error: None,
            relative_error: None,
            train: true,
        }],
        estimator: if adaptive {
            ExpectationEstimator::Adaptive(ExpectationAdaptiveSettings {
                bins: 8,
                minimum_training_samples: 2,
                bin_evolution: Some(vec![8, 12]),
                uniform_mixture: 0.1,
                maximum_probability_ratio: 4.,
                discrete_learning_rate: 0.5,
                continuous_learning_rate: 0.5,
            })
        } else {
            ExpectationEstimator::FixedMonteCarlo
        },
        controls: ExpectationControls {
            seed: 123,
            lanes: 3,
            samples_per_iteration: samples,
            maximum_iterations: 3,
            maximum_samples: samples * 3,
            normalization_tolerance: 1e-12,
        },
        failure_probability: true,
        replay: ExpectationReplay::PopulationAndBoundary,
    };
    (runtime, package, request)
}

fn observations(
    iteration: &ExpectationIteration,
    values: impl Fn(usize, &ExpectationSample) -> Vec<Option<f64>>,
) -> Vec<MemberObservation> {
    iteration
        .samples
        .iter()
        .enumerate()
        .map(|(index, sample)| MemberObservation {
            child: iteration.child,
            ordinal: sample.ordinal,
            values: values(index, sample),
            scientific_failure: Some(false),
            operational_incomplete: false,
        })
        .collect()
}

#[tokio::test]
async fn fixed_expectation_matches_independent_finite_contributions_and_original_replay() {
    let (runtime, package, request) = fixture(false, 4).await;
    let admitted = package
        .admit_expectation(
            &runtime.preparation_services(),
            request.clone(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let mut parent = ExpectationParent::new(
        admitted.clone(),
        study(8),
        "numerica-3.0.1-maintained".into(),
    )
    .unwrap();
    let iteration = parent
        .prepare_iteration(&runtime.preparation_services(), study(9))
        .unwrap();
    let original = serde_json::to_vec(&iteration.samples).unwrap();
    assert_eq!(
        serde_json::to_vec(
            &parent
                .prepare_iteration(&runtime.preparation_services(), study(9))
                .unwrap()
                .samples
        )
        .unwrap(),
        original
    );
    assert!(parent.checkpoint().is_err());
    let observed = observations(&iteration, |index, _| vec![Some((index + 1) as f64)]);
    let mut duplicate = observed.clone();
    duplicate[1].ordinal = duplicate[0].ordinal;
    assert!(parent.commit_iteration(study(9), &duplicate).is_err());
    assert_eq!(
        parent.report().issued,
        0,
        "a refused population cannot partially mutate its parent"
    );
    let report = parent.commit_iteration(study(9), &observed).unwrap();
    assert_eq!(report.outputs[0].estimate, Some(2.5));
    assert!((report.outputs[0].standard_error.unwrap() - (5_f64 / 12.).sqrt()).abs() < 1e-12);
    assert!(report.outputs[0].complete);
    assert!(parent.commit_iteration(study(9), &observed).is_err());
    let checkpoint = parent.checkpoint().unwrap();
    let original_boundary = checkpoint.clone();
    let bytes = serde_json::to_vec(&checkpoint).unwrap();
    let restored: ExpectationCheckpoint = serde_json::from_slice(&bytes).unwrap();
    let mut replay =
        ExpectationParent::restore(admitted, study(8), "numerica-3.0.1-maintained", restored)
            .unwrap();
    let next = parent
        .prepare_iteration(&runtime.preparation_services(), study(10))
        .unwrap();
    let replayed = replay
        .prepare_iteration(&runtime.preparation_services(), study(10))
        .unwrap();
    assert_eq!(
        serde_json::to_vec(&next.samples).unwrap(),
        serde_json::to_vec(&replayed.samples).unwrap()
    );
    assert!(
        ExpectationParent::restore(
            parent.admitted().clone(),
            study(7),
            "numerica-3.0.1-maintained",
            checkpoint.clone()
        )
        .is_err()
    );
    assert!(
        ExpectationParent::restore(
            parent.admitted().clone(),
            study(8),
            "other-source",
            checkpoint
        )
        .is_err()
    );
    // An interrupted original parent keeps its prior complete iteration only as
    // restart state, without advertising that convenient subset as completed science.
    let report = parent.interrupt();
    assert!(report.interrupted);
    assert!(!report.outputs[0].complete);
    assert!(!report.failure_probability_complete);
    assert_eq!(original_boundary.next_iteration(), 1);
}

#[tokio::test]
async fn adaptive_expectation_unequal_proposal_uses_target_once_and_stops_at_limit() {
    let (runtime, package, mut request) = fixture(true, 2048).await;
    request.controls.maximum_iterations = 2;
    request.outputs[0].absolute_error = Some(quantity(&package.quantities, "Scalar", 1e-14, None));
    let admitted = package
        .admit_expectation(
            &runtime.preparation_services(),
            request,
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let mut parent =
        ExpectationParent::new(admitted, study(8), "qualified-numerica".into()).unwrap();
    let mut independent = Vec::new();
    for index in 0..2 {
        let iteration = parent
            .prepare_iteration(&runtime.preparation_services(), study(10 + index))
            .unwrap();
        for sample in iteration.samples.iter() {
            // Independently enumerated target masses and original proposal probabilities.
            let mass = if sample.categories[0] == id(1) {
                0.25
            } else {
                0.75
            };
            independent.push(mass * sample.proposal_weight);
            assert_ne!(sample.categories[0], id(3));
        }
        let report = parent
            .commit_iteration(
                iteration.child,
                &observations(&iteration, |_, _| vec![Some(1.)]),
            )
            .unwrap();
        let expected = independent.iter().sum::<f64>() / independent.len() as f64;
        assert!((report.outputs[0].estimate.unwrap() - expected).abs() < 1e-12);
        assert!((report.outputs[0].estimate.unwrap() - 1.).abs() < 0.05);
        if index == 1 {
            assert!(report.limit_reached);
            assert!(!report.outputs[0].precision_met);
        }
    }
    assert!(!parent.has_next());
}

#[tokio::test]
async fn fixed_expectation_signed_and_constant_outputs_keep_nominal_statistics() {
    let (runtime, package, request) = fixture(false, 6).await;
    let admitted = package
        .admit_expectation(
            &runtime.preparation_services(),
            request,
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    for (parent_id, values, mean, error) in [
        (8, [-3., -1., 0., 1., 2., 4.], 0.5, (59_f64 / 60.).sqrt()),
        (18, [-2.; 6], -2., 0.),
    ] {
        let mut parent =
            ExpectationParent::new(admitted.clone(), study(parent_id), "qualified".into()).unwrap();
        let iteration = parent
            .prepare_iteration(&runtime.preparation_services(), study(parent_id + 1))
            .unwrap();
        let report = parent
            .commit_iteration(
                iteration.child,
                &observations(&iteration, |index, _| vec![Some(values[index])]),
            )
            .unwrap();
        assert_eq!(report.outputs[0].estimate, Some(mean));
        assert!((report.outputs[0].standard_error.unwrap() - error).abs() < 1e-12);
    }
}

#[tokio::test]
async fn expectation_missing_output_is_not_renormalized_and_operational_loss_is_not_failure() {
    let (runtime, package, request) = fixture(false, 4).await;
    let admitted = package
        .admit_expectation(
            &runtime.preparation_services(),
            request,
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let mut parent =
        ExpectationParent::new(admitted.clone(), study(8), "qualified".into()).unwrap();
    let iteration = parent
        .prepare_iteration(&runtime.preparation_services(), study(9))
        .unwrap();
    let mut observed = observations(&iteration, |index, _| vec![(index != 1).then_some(2.)]);
    observed[1].scientific_failure = Some(true);
    let report = parent.commit_iteration(study(9), &observed).unwrap();
    assert!(!report.outputs[0].complete);
    assert_eq!(report.outputs[0].available, 3);
    assert_eq!(report.outputs[0].issued, 4);
    assert!(report.outputs[0].estimate.is_none());
    assert!(report.outputs[0].standard_error.is_none());
    assert_eq!(report.failure_probability.unwrap().estimate, 0.25);
    let mut stopped = ExpectationParent::new(admitted, study(18), "qualified".into()).unwrap();
    let iteration = stopped
        .prepare_iteration(&runtime.preparation_services(), study(19))
        .unwrap();
    let mut observed = observations(&iteration, |_, _| vec![Some(2.)]);
    observed[0].operational_incomplete = true;
    let report = stopped.commit_iteration(study(19), &observed).unwrap();
    assert!(report.interrupted);
    assert!(!report.failure_probability_complete);
    assert!(report.failure_probability.is_none());
    assert!(stopped.checkpoint().is_err());
}

#[tokio::test]
async fn expectation_admission_refuses_mass_conflicts_and_missing_physical_outputs() {
    let (runtime, package, request) = fixture(true, 8).await;
    let mut bad = request.clone();
    if let ExpectationCoordinate::Categorical { categories, .. } = &mut bad.coordinates[1] {
        categories[0].probability = -0.25;
    }
    assert!(
        package
            .admit_expectation(
                &runtime.preparation_services(),
                bad,
                &crate::CancelSource::new()
            )
            .await
            .is_err()
    );
    let mut bad = request.clone();
    bad.coordinates.push(request.coordinates[0].clone());
    assert!(
        package
            .admit_expectation(
                &runtime.preparation_services(),
                bad,
                &crate::CancelSource::new()
            )
            .await
            .is_err()
    );
    let mut bad = request.clone();
    bad.outputs[0].output = id(91);
    assert!(
        package
            .admit_expectation(
                &runtime.preparation_services(),
                bad,
                &crate::CancelSource::new()
            )
            .await
            .is_err()
    );
    let mut bad = request.clone();
    if let ExpectationEstimator::Adaptive(settings) = &mut bad.estimator {
        settings.uniform_mixture = 0.;
    }
    assert!(
        package
            .admit_expectation(
                &runtime.preparation_services(),
                bad,
                &crate::CancelSource::new()
            )
            .await
            .is_err()
    );
    for mass in [0., 0.5, f64::NAN, f64::INFINITY] {
        let mut bad = request.clone();
        if let ExpectationCoordinate::Categorical { categories, .. } = &mut bad.coordinates[1] {
            for category in categories {
                category.probability = mass;
            }
        }
        assert!(
            package
                .admit_expectation(
                    &runtime.preparation_services(),
                    bad,
                    &crate::CancelSource::new()
                )
                .await
                .is_err()
        );
    }
    let mut bad = request.clone();
    if let ExpectationCoordinate::Uniform { lower, upper, .. } = &mut bad.coordinates[0] {
        *upper = lower.clone();
    }
    assert!(
        package
            .admit_expectation(
                &runtime.preparation_services(),
                bad,
                &crate::CancelSource::new()
            )
            .await
            .is_err()
    );
    let mut bad = request.clone();
    bad.outputs[0].scale = quantity(&package.quantities, "Temperature", 1., Some("K"));
    assert!(
        package
            .admit_expectation(
                &runtime.preparation_services(),
                bad,
                &crate::CancelSource::new()
            )
            .await
            .is_err()
    );
    let mut bad = request;
    bad.source.physical_context = pse_ids::ContentHash::from_bytes([91; 32]);
    assert!(
        package
            .admit_expectation(
                &runtime.preparation_services(),
                bad,
                &crate::CancelSource::new()
            )
            .await
            .is_err()
    );
}

#[tokio::test]
async fn expectation_local_selected_population_preserves_original_science_and_reader() {
    let (runtime, package, request) = fixture(true, 24).await;
    let services = runtime.execution_services();
    let admitted = package
        .admit_expectation(services.preparation(), request, &crate::CancelSource::new())
        .await
        .unwrap();
    let mut parent =
        ExpectationParent::new(admitted, study(8), "qualified-numerica".into()).unwrap();
    let (iteration, child) = parent
        .prepare_child(&services, study(9), &crate::CancelSource::new())
        .await
        .unwrap();
    let report = child
        .execute(&services, 24, &crate::CancelSource::new())
        .await
        .unwrap();
    let observations = iteration
        .read_observations(parent.admitted(), &report, &crate::CancelSource::new())
        .await
        .unwrap();
    let ExpectationMember::Observation(operation) = &parent.admitted().request().member else {
        unreachable!()
    };
    for (sample, observation) in iteration.samples.iter().zip(&observations) {
        let category = if sample.categories[0] == id(1) {
            1.
        } else {
            2.
        };
        assert!((observation.values[0].unwrap() - sample.base[0] * category).abs() < 1e-13);
        let binding = package
            .admit_overlay(parent.admitted().member_model().compiled(), &sample.overlay)
            .unwrap();
        let scalar = package
            .observe_selected(
                services.preparation(),
                operation,
                Some(&binding),
                &parent.admitted().request().preparation,
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        assert_eq!(
            scalar.members,
            vec![parent.admitted().outputs()[0].request.output]
        );
        assert!(
            (scalar.canonical[0] - observation.values[0].unwrap()).abs() < 1e-13,
            "the actual scalar and batched physical observation agree for this original occurrence"
        );
    }
    let expected = iteration
        .samples
        .iter()
        .map(|sample| {
            sample.base[0]
                * if sample.categories[0] == id(1) {
                    0.5
                } else {
                    3.
                }
        })
        .sum::<f64>()
        / 24.;
    let estimate = parent.commit_iteration(study(9), &observations).unwrap();
    assert!(estimate.outputs[0].complete);
    assert!((estimate.outputs[0].estimate.unwrap() - expected).abs() < 1e-13);
    assert!(report.results.iter().all(Option::is_some));
}

#[tokio::test]
async fn adaptive_expectation_affine_point_reference_keeps_finite_population_covariant() {
    let (runtime, package, request) = fixture(true, 25).await;
    let services = runtime.preparation_services();
    let operation = match &request.member {
        ExpectationMember::Observation(operation) => operation.clone(),
        _ => unreachable!(),
    };
    let mut bindings = operation.bindings.clone();
    bindings.demand.push("temperature".into());
    let model = package
        .prepare(
            &services,
            operation.root,
            operation.instance,
            bindings,
            request.preparation.limits,
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let temperature = model
        .compiled()
        .model
        .symbols
        .values()
        .find(|s| s.lineage.path == "Root.temperature")
        .unwrap()
        .id;
    let mut parents = Vec::new();
    for unit in ["K", "degC"] {
        let mut request = request.clone();
        let point = quantity(&package.quantities, "Temperature", 300., Some("K"));
        let selected = quantity(&package.quantities, "Temperature", 0., Some(unit));
        let scale = quantity(&package.quantities, "DeltaTemperature", 10., Some(unit));
        if let ExpectationMember::Observation(operation) = &mut request.member {
            operation.outputs.push(SelectedObservationOutput {
                member: BindingTarget::Member(temperature),
                quantity: selected.quantity,
                unit: selected.unit,
            });
        }
        request.outputs.push(ExpectationOutput {
            output: temperature,
            quantity: selected.quantity,
            unit: selected.unit,
            reference: Some(point),
            scale,
            absolute_error: None,
            relative_error: None,
            train: true,
        });
        let admitted = package
            .admit_expectation(&services, request, &crate::CancelSource::new())
            .await
            .unwrap();
        parents.push(ExpectationParent::new(admitted, study(8), "qualified".into()).unwrap());
    }
    let mut reports = Vec::new();
    let mut next_populations = Vec::new();
    let mut original = None;
    for parent in &mut parents {
        let iteration = parent.prepare_iteration(&services, study(9)).unwrap();
        let coordinates = serde_json::to_vec(&iteration.samples).unwrap();
        if let Some(original) = &original {
            assert_eq!(original, &coordinates);
        } else {
            original = Some(coordinates);
        }
        assert!(
            (iteration
                .samples
                .iter()
                .map(|sample| sample.target_mass * sample.proposal_weight)
                .sum::<f64>()
                - 25.)
                .abs()
                > 0.1
        );
        let observed = observations(&iteration, |_, sample| {
            vec![Some(1.), Some(300. + 20. * sample.base[0])]
        });
        reports.push(parent.commit_iteration(study(9), &observed).unwrap());
        next_populations.push(parent.prepare_iteration(&services, study(10)).unwrap());
    }
    let kelvin = &reports[0].outputs[1];
    let celsius = &reports[1].outputs[1];
    assert!((kelvin.estimate.unwrap() - celsius.estimate.unwrap() - 273.15).abs() < 1e-12);
    assert!((kelvin.standard_error.unwrap() - celsius.standard_error.unwrap()).abs() < 1e-12);
    assert!(
        (reports[0].outputs[0].estimate.unwrap() - reports[1].outputs[0].estimate.unwrap()).abs()
            < 1e-14
    );
    for (kelvin, celsius) in next_populations[0]
        .samples
        .iter()
        .zip(next_populations[1].samples.iter())
    {
        assert_eq!(kelvin.categories, celsius.categories);
        for (a, b) in kelvin.base.iter().zip(&celsius.base) {
            assert!(
                (a - b).abs() < 1e-12,
                "converted interval scales/reference preserve the adapted physical proposal"
            );
        }
        assert!(
            (kelvin.proposal_weight - celsius.proposal_weight).abs()
                < 1e-11 * kelvin.proposal_weight.abs().max(1.)
        );
    }
}

#[tokio::test]
async fn expectation_joint_authored_pushforward_keeps_input_dependence() {
    let (runtime, package, mut request) = fixture(false, 256).await;
    let services = runtime.execution_services();
    let declarations = package.declarations(services.preparation()).await.unwrap();
    let joint = declarations
        .iter()
        .find(|row| row.name == "Joint")
        .unwrap()
        .declaration_id;
    let model = package
        .prepare(
            services.preparation(),
            joint,
            pse_modeling::specialize::root_instance(joint),
            pse_modeling::Bindings {
                demand: vec!["x".into(), "category".into()],
                ..Default::default()
            },
            request.preparation.limits,
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let member = |name: &str| {
        model
            .compiled()
            .model
            .symbols
            .values()
            .find(|s| s.lineage.path == format!("Joint.{name}"))
            .unwrap()
            .id
    };
    let scalar = quantity(&package.quantities, "Scalar", 1., None);
    let original_targets = request
        .coordinates
        .iter()
        .map(|coordinate| match coordinate {
            ExpectationCoordinate::Uniform { target, .. }
            | ExpectationCoordinate::Categorical { target, .. } => target.clone(),
        })
        .collect::<Vec<_>>();
    request.coordinates = vec![ExpectationCoordinate::Uniform {
        target: BindingTarget::Member(member("u")),
        lower: quantity(&package.quantities, "Scalar", 0., None),
        upper: scalar.clone(),
    }];
    request.transform = Some(ExpectationTransform {
        operation: SelectedObservationOperation {
            root: joint,
            instance: pse_modeling::specialize::root_instance(joint),
            bindings: pse_modeling::Bindings {
                demand: vec!["u".into(), "x".into(), "category".into()],
                ..Default::default()
            },
            outputs: vec![
                SelectedObservationOutput {
                    member: BindingTarget::Member(member("x")),
                    quantity: scalar.quantity,
                    unit: scalar.unit,
                },
                SelectedObservationOutput {
                    member: BindingTarget::Member(member("category")),
                    quantity: scalar.quantity,
                    unit: scalar.unit,
                },
            ],
        },
        outputs: vec![
            ExpectationTransformOutput {
                output: member("x"),
                target: original_targets[0].clone(),
            },
            ExpectationTransformOutput {
                output: member("category"),
                target: original_targets[1].clone(),
            },
        ],
    });
    let admitted = package
        .admit_expectation(services.preparation(), request, &crate::CancelSource::new())
        .await
        .unwrap();
    let mut parent = ExpectationParent::new(admitted, study(8), "qualified".into()).unwrap();
    let (iteration, child) = parent
        .prepare_child(&services, study(9), &crate::CancelSource::new())
        .await
        .unwrap();
    let original = child
        .execute(&services, 256, &crate::CancelSource::new())
        .await
        .unwrap();
    let observed = iteration
        .read_observations(parent.admitted(), &original, &crate::CancelSource::new())
        .await
        .unwrap();
    let actual = parent
        .commit_iteration(study(9), &observed)
        .unwrap()
        .outputs[0]
        .estimate
        .unwrap();
    let mean = iteration
        .samples
        .iter()
        .map(|sample| sample.base[0])
        .sum::<f64>()
        / 256.;
    let independent = iteration
        .samples
        .iter()
        .map(|sample| sample.base[0] * (1. + sample.base[0]))
        .sum::<f64>()
        / 256.;
    assert!((actual - independent).abs() < 1e-12);
    assert!(
        actual - mean * (1. + mean) > 0.05,
        "the retained joint population has positive covariance"
    );
    assert!(
        (actual - 5. / 6.).abs() < 0.12,
        "independent analytic integral of u+u^2"
    );
}
