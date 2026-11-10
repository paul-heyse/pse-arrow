// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original explicit-measure heater through the actual native child and retained reader.
use super::*;
use pse_ids::SemanticId;
use pse_model::{generated::identities::StudyId, scalars::FiniteBound};
use pse_relations::columnar::RelationRow;
use pse_runtime::workflow::expectation::*;
use pse_runtime::workflow::{
    BindingQuantity, BindingTarget, PreparedOperation, PreparedStudyOperation,
    SelectedObservationOperation, SelectedObservationOutput,
};
#[cfg(feature = "engineering-service")]
mod wire;

const SOURCE: &str = r#"package expectation_acceptance {
use correlations @"1.0.0";
test Heater fixture {dof 0; route steady; procedure solve;} {
    param inlet:Temperature=300{K};
    var outlet:Temperature; var flow:Flow; var dh:DeltaH; var duty:EnergyTransferRate;
    eq caloric:dh==correlations.dippr100_dh_value(inlet,outlet,30{J/(mol*K)},0{J/(mol*K^2)},0{J/(mol*K^3)},0{J/(mol*K^4)},0{J/(mol*K^5)});
    eq energy:duty==flow*dh;
    annotation fixed outlet(true); annotation fixed flow(true);
    annotation bounds outlet(310{K},330{K}); annotation bounds flow(1{mol/s},2{mol/s});
    annotation start outlet(320{K}); annotation start flow(1{mol/s});
    annotation start dh(600{J/mol}); annotation start duty(1050{W});
}
def Pure {
    param inlet:Temperature=300{K}; var outlet:Temperature; var flow:Flow;
    let dh:DeltaH=correlations.dippr100_dh_value(inlet,outlet,30{J/(mol*K)},0{J/(mol*K^2)},0{J/(mol*K^3)},0{J/(mol*K^4)},0{J/(mol*K^5)});
    let duty:EnergyTransferRate=flow*dh;
    annotation bounds outlet(310{K},330{K});
}
def Squared {
    var u:Scalar; var flow:Flow;
    param origin:Temperature=310{K};param span:DeltaTemperature=20{K};
    let delta:DeltaTemperature=span*u*u;
    let outlet:Temperature=origin+delta; let flow_out:Flow=flow;
}
def Refusal {var sample:Scalar;let value:Scalar=log1p(sample);}
def InvalidPushforward {
    var u:Scalar;var flow:Flow;
    param origin:Temperature=310{K};param span:DeltaTemperature=20{K};
    let delta:DeltaTemperature=span*log1p(u);
    let outlet:Temperature=origin+delta;let flow_out:Flow=flow;
}
}"#;

fn physical_value(
    package: &ModelingPackage,
    name: &str,
    unit: &str,
    magnitude: f64,
) -> BindingQuantity {
    let physical = package.physical_context();
    BindingQuantity {
        quantity: physical.resolve_quantity(name).unwrap(),
        unit: physical.resolve_unit(unit).unwrap(),
        magnitude: FiniteBound::try_new(magnitude).unwrap(),
    }
}

async fn package(science: &Science) -> ModelingPackage {
    let runtime = &science.runtime;
    let capture = |logical: &str| {
        pse_admission::authoring_driver::document::capture_package(
            &root().join(logical),
            runtime.registry(),
            Default::default(),
        )
        .unwrap()
        .files
    };
    let physical = capture("packages/reference/physical");
    let domain = capture("packages/reference/domain");
    let methods = capture("packages/reference/methods");
    let mut manifest: toml::Value = toml::from_str(
        &std::fs::read_to_string(
            root().join("tests/fixtures/packages/minimal_explicit/package.toml"),
        )
        .unwrap(),
    )
    .unwrap();
    manifest["package"]["name"] = "native.expectation.acceptance".into();
    manifest["package"]["id_policy"] = "named".into();
    manifest["package"]["dependencies"] = toml::Value::Array(
        [&physical, &methods]
            .into_iter()
            .map(|package| {
                let header: toml::Value =
                    toml::from_str(std::str::from_utf8(&package["package.toml"]).unwrap()).unwrap();
                toml::Table::from_iter([
                    ("package_id".into(), header["package"]["package_id"].clone()),
                    (
                        "version_req".into(),
                        toml::Table::from_iter([
                            ("operator".into(), "exact".into()),
                            ("major".into(), 1.into()),
                            ("minor".into(), 0.into()),
                            ("patch".into(), 0.into()),
                        ])
                        .into(),
                    ),
                ])
                .into()
            })
            .collect(),
    );
    let authored = BTreeMap::from([
        (
            "package.toml".into(),
            toml::to_string(&manifest).unwrap().into_bytes(),
        ),
        ("models/expectation.pse".into(), SOURCE.as_bytes().to_vec()),
    ]);
    let pool = science.shared.pool();
    let cancel = CancelSource::new();
    let token = cancel.token();
    let validation = runtime.validation_context();
    let load = |packages: Vec<&BTreeMap<String, Vec<u8>>>| {
        OwnedDocumentSet::try_from_bundles(
            packages
                .into_iter()
                .map(|package| {
                    load_package_documents_owned(
                        package,
                        runtime.registry(),
                        Default::default(),
                        &pool,
                        &token,
                        &validation,
                    )
                    .unwrap()
                })
                .collect(),
            &pool,
            &token,
        )
        .unwrap()
    };
    let context = runtime
        .physical_from_documents(&load(vec![&physical]), &token)
        .await
        .unwrap();
    runtime
        .modeling_from_documents(
            &load(vec![&physical, &domain, &methods, &authored]),
            context,
            &cancel,
        )
        .await
        .unwrap()
}

async fn request(
    runtime: &Runtime,
    package: &ModelingPackage,
    ordinary: bool,
    squared: bool,
) -> ExpectationRequest {
    let declarations = package
        .declarations(&runtime.preparation_services())
        .await
        .unwrap();
    let root = declarations
        .iter()
        .find(|row| row.name == if ordinary { "Heater" } else { "Pure" })
        .unwrap()
        .declaration_id;
    let settings = PreparationSettings::default();
    let bindings = pse_modeling::Bindings {
        demand: vec!["outlet".into(), "flow".into(), "dh".into(), "duty".into()],
        ..Default::default()
    };
    let model = package
        .prepare(
            &runtime.preparation_services(),
            root,
            pse_modeling::specialize::root_instance(root),
            bindings.clone(),
            settings.limits,
            &CancelSource::new(),
        )
        .await
        .unwrap();
    let member = |name: &str| {
        model
            .compiled()
            .model
            .symbols
            .values()
            .find(|symbol| symbol.lineage.path.ends_with(&format!(".{name}")))
            .unwrap()
            .id
    };
    let mut solve = SolveSettings {
        intent: SolveIntent::Root,
        backend: Some(Backend::Ipopt),
        presolve: PolicyKind::Off,
        ..Default::default()
    };
    if ordinary {
        let prepared = package
            .prepare_operation(
                &runtime.preparation_services(),
                &OperationSource::of(package),
                &settings,
                RuntimeOperationRequest::DeclaredCase {
                    case: root,
                    settings: Box::new(solve.clone()),
                },
                None,
                &CancelSource::new(),
            )
            .await
            .unwrap();
        let PreparedOperation::Study(PreparedStudyOperation::DeclaredCase(prepared)) = prepared
        else {
            panic!("authored heater case required")
        };
        let solved = prepared.model.model.solved();
        solve.numerics.requirements = prepared
            .solve
            .numerics()
            .targets
            .iter()
            .filter(|target| {
                matches!(
                    target.kind,
                    pse_model::generated::enums::NumericalTarget::Variable
                        | pse_model::generated::enums::NumericalTarget::Row
                )
            })
            .map(|target| pse_model::numerics::NumericalRequirement {
                requirement_id: pse_ids::named_id(
                    root.as_id(),
                    &format!("expectation-accuracy:{:?}:{}", target.kind, target.id),
                ),
                model_id: Some(solved.model()),
                case_id: solved.case(),
                instance_id: Some(solved.instance()),
                fit_id: None,
                target_id: target.id,
                target_kind: target.kind,
                nominal: None,
                scaling_factor: None,
                absolute_tolerance: Some(1e-7),
                relative_tolerance: Some(0.),
                shared_engineering_allowance: None,
                engineering_rule_id: None,
                unit_id: Some(target.unit),
                coordinates: pse_model::generated::enums::NumericalCoordinates::Physical,
                priority: 0,
                required: true,
                provenance: "independent constant-cp heater scalar oracle".into(),
            })
            .collect();
    }
    let outputs = [
        ("duty", "EnergyTransferRate", "W", 1000.),
        ("dh", "DeltaH", "J/mol", 600.),
    ]
    .into_iter()
    .map(|(name, quantity, unit, scale)| {
        let scale = physical_value(package, quantity, unit, scale);
        ExpectationOutput {
            output: member(name),
            quantity: scale.quantity,
            unit: scale.unit,
            reference: None,
            scale,
            absolute_error: None,
            relative_error: None,
            train: true,
        }
    })
    .collect::<Vec<_>>();
    let operation = SelectedObservationOperation {
        root,
        instance: pse_modeling::specialize::root_instance(root),
        bindings,
        outputs: outputs
            .iter()
            .map(|output| SelectedObservationOutput {
                member: BindingTarget::Member(output.output),
                quantity: output.quantity,
                unit: output.unit,
            })
            .collect(),
    };
    let categories = vec![
        ExpectationCategory {
            category: SemanticId::from_bytes([1; 16]),
            value: physical_value(package, "Flow", "mol/s", 1.),
            probability: 0.25,
        },
        ExpectationCategory {
            category: SemanticId::from_bytes([2; 16]),
            value: physical_value(package, "Flow", "mol/s", 2.),
            probability: 0.75,
        },
    ];
    let mut coordinates = vec![
        ExpectationCoordinate::Uniform {
            target: BindingTarget::Member(member("outlet")),
            lower: physical_value(package, "Temperature", "K", 310.),
            upper: physical_value(package, "Temperature", "K", 330.),
        },
        ExpectationCoordinate::Categorical {
            target: BindingTarget::Member(member("flow")),
            categories: categories.clone(),
        },
    ];
    let transform = if squared {
        let transform = declarations
            .iter()
            .find(|row| row.name == "Squared")
            .unwrap()
            .declaration_id;
        let bindings = pse_modeling::Bindings {
            demand: vec![
                "u".into(),
                "flow".into(),
                "outlet".into(),
                "flow_out".into(),
            ],
            ..Default::default()
        };
        let model = package
            .prepare(
                &runtime.preparation_services(),
                transform,
                pse_modeling::specialize::root_instance(transform),
                bindings.clone(),
                settings.limits,
                &CancelSource::new(),
            )
            .await
            .unwrap();
        let tmember = |name: &str| {
            model
                .compiled()
                .model
                .symbols
                .values()
                .find(|symbol| symbol.lineage.path.ends_with(&format!(".{name}")))
                .unwrap()
                .id
        };
        let temperature = physical_value(package, "Temperature", "K", 310.);
        let flow = physical_value(package, "Flow", "mol/s", 1.);
        coordinates = vec![
            ExpectationCoordinate::Uniform {
                target: BindingTarget::Member(tmember("u")),
                lower: physical_value(package, "Scalar", "1", 0.),
                upper: physical_value(package, "Scalar", "1", 1.),
            },
            ExpectationCoordinate::Categorical {
                target: BindingTarget::Member(tmember("flow")),
                categories,
            },
        ];
        Some(ExpectationTransform {
            operation: SelectedObservationOperation {
                root: transform,
                instance: pse_modeling::specialize::root_instance(transform),
                bindings,
                outputs: vec![
                    SelectedObservationOutput {
                        member: BindingTarget::Member(tmember("outlet")),
                        quantity: temperature.quantity,
                        unit: temperature.unit,
                    },
                    SelectedObservationOutput {
                        member: BindingTarget::Member(tmember("flow_out")),
                        quantity: flow.quantity,
                        unit: flow.unit,
                    },
                ],
            },
            outputs: vec![
                ExpectationTransformOutput {
                    output: tmember("outlet"),
                    target: BindingTarget::Member(member("outlet")),
                },
                ExpectationTransformOutput {
                    output: tmember("flow_out"),
                    target: BindingTarget::Member(member("flow")),
                },
            ],
        })
    } else {
        None
    };
    ExpectationRequest {
        source: OperationSource::of(package),
        preparation: settings,
        member: if ordinary {
            ExpectationMember::Case(CaseOperation {
                case: root,
                route: pse_model::generated::enums::ModelingAnalysisRoute::Steady,
                settings: solve,
            })
        } else {
            ExpectationMember::Observation(operation)
        },
        coordinates,
        transform,
        outputs,
        estimator: if ordinary {
            ExpectationEstimator::FixedMonteCarlo
        } else {
            ExpectationEstimator::Adaptive(ExpectationAdaptiveSettings {
                bins: 8,
                minimum_training_samples: 2,
                bin_evolution: None,
                uniform_mixture: 0.1,
                maximum_probability_ratio: 4.,
                discrete_learning_rate: 0.5,
                continuous_learning_rate: 0.5,
            })
        },
        controls: ExpectationControls {
            seed: 456,
            lanes: 4,
            samples_per_iteration: 32,
            maximum_iterations: 2,
            maximum_samples: 64,
            normalization_tolerance: 1e-12,
        },
        failure_probability: true,
        replay: ExpectationReplay::PopulationAndBoundary,
    }
}

async fn journey(science: &Science) {
    let runtime = &science.runtime;
    let package = package(science).await;
    for (ordinal, ordinary, squared) in [(0, true, false), (1, false, false), (2, false, true)] {
        let request = request(runtime, &package, ordinary, squared).await;
        let parent = StudyId::from_id(SemanticId::from_bytes([40 + ordinal as u8; 16]));
        let report = runtime
            .execute_expectation(&package, parent, request.clone(), &CancelSource::new())
            .await
            .unwrap();
        assert_eq!(report.issued, 64);
        assert_eq!(report.completed_iterations, 2);
        assert!(report.outputs.iter().all(|output| output.complete));
        let expected = if squared { [875., 500.] } else { [1050., 600.] };
        for (output, mean) in report.outputs.iter().zip(expected) {
            assert!(
                (output.estimate.unwrap() - mean).abs() < 200.,
                "original output {output:?}, analytic mean {mean}"
            );
        }
        assert_eq!(report.failure_probability.as_ref().unwrap().estimate, 0.);
        let store = runtime.canonical_store().unwrap();
        let original = store
            .expectation_proposal(&parent.to_string(), 0)
            .await
            .unwrap()
            .unwrap();
        let page = runtime
            .expectation_samples(parent, 0, 0, 32, &CancelSource::new())
            .await
            .unwrap();
        let rows = pse_model::generated::runtime::expectation_samples::Row::rows(&page).unwrap();
        assert_eq!(rows.len(), 32);
        let publication = store
            .expectation_publication(&parent.to_string())
            .await
            .unwrap()
            .unwrap();
        let protected = store
            .read_results(&publication.run, &publication.attempt)
            .await
            .unwrap();
        assert_eq!(protected.manifest().key, publication.manifest);
        for (relation, count) in [
            (
                pse_relations::generated::runtime::expectation_samples::RELATION_ID.to_string(),
                64,
            ),
            (
                pse_relations::generated::runtime::expectation_estimates::RELATION_ID.to_string(),
                2,
            ),
            (
                pse_relations::generated::runtime::expectation_summaries::RELATION_ID.to_string(),
                1,
            ),
        ] {
            assert_eq!(
                protected
                    .sets()
                    .iter()
                    .find(|set| set.name == relation)
                    .unwrap()
                    .row_count,
                count,
                "ordinary canonical manifest owns each original product"
            );
        }
        let temperature = physical_value(&package, "Temperature", "K", 0.).quantity;
        let flow = physical_value(&package, "Flow", "mol/s", 1.).quantity;
        let next_page = runtime
            .expectation_samples(parent, 1, 0, 32, &CancelSource::new())
            .await
            .unwrap();
        let next_rows =
            pse_model::generated::runtime::expectation_samples::Row::rows(&next_page).unwrap();
        assert_eq!(next_rows.len(), 32);
        let mut finite_sums = [0_f64; 2];
        for (index, row) in rows.iter().chain(&next_rows).enumerate() {
            assert_eq!(row.parent_id, parent);
            assert_eq!(row.ordinal, index as i64);
            assert_eq!(row.iteration, (index / 32) as i64);
            if index < 32 {
                assert_eq!(row.child_id.to_string(), original.child);
            } else {
                assert_ne!(row.child_id, rows[0].child_id);
            }
            let outlet = row
                .inputs
                .iter()
                .find(|input| input.quantity == temperature)
                .unwrap()
                .canonical;
            let flow = row
                .inputs
                .iter()
                .find(|input| input.quantity == flow)
                .unwrap()
                .canonical;
            let u = row.base_coordinates[0];
            let expected_outlet = 310. + 20. * if squared { u * u } else { u };
            assert!(
                (outlet - expected_outlet).abs() < 1e-12,
                "the retained physical point follows its declared base-law pushforward"
            );
            let enthalpy = 30. * (outlet - 300.);
            let (mass, expected_flow) = match row.categories.as_slice() {
                [category] if *category == SemanticId::from_bytes([1; 16]) => (0.25, 1.),
                [category] if *category == SemanticId::from_bytes([2; 16]) => (0.75, 2.),
                _ => panic!("original authored flow category required"),
            };
            assert_eq!(flow, expected_flow);
            assert_eq!(row.target_mass, mass);
            let correction = if ordinary {
                1.
            } else {
                mass * row.proposal_weight
            };
            assert_eq!(row.outputs.len(), 2);
            for (output_index, (output, expected)) in row
                .outputs
                .iter()
                .zip([flow * enthalpy, enthalpy])
                .enumerate()
            {
                assert!((output.canonical.unwrap() - expected).abs() < 1e-5);
                finite_sums[output_index] += correction * expected;
            }
        }
        for (output, sum) in report.outputs.iter().zip(finite_sums) {
            assert!(
                (output.estimate.unwrap() - sum / 64.).abs() < 1e-5,
                "public expectation equals the independent corrected sum of all original physical members"
            );
        }
        let estimates = runtime
            .expectation_estimates(parent, &CancelSource::new())
            .await
            .unwrap();
        let estimates =
            pse_model::generated::runtime::expectation_estimates::Row::rows(&estimates).unwrap();
        assert_eq!(estimates.len(), 2);
        assert!(
            estimates
                .iter()
                .all(|estimate| estimate.run_id == rows[0].run_id
                    && estimate.parent_id == parent
                    && estimate.complete)
        );
        let missing = runtime
            .expectation_samples(parent, 0, 32, 32, &CancelSource::new())
            .await
            .unwrap();
        assert_eq!(
            missing.batch().num_rows(),
            0,
            "missing retained range cannot trigger another population"
        );
        assert_eq!(
            runtime
                .expectation_samples(parent, 0, 33, 32, &CancelSource::new())
                .await
                .unwrap()
                .batch()
                .num_rows(),
            0,
            "an offset past this child cannot leak the next child"
        );
        let tail = runtime
            .expectation_samples(parent, 0, 31, 32, &CancelSource::new())
            .await
            .unwrap();
        let tail = pse_model::generated::runtime::expectation_samples::Row::rows(&tail).unwrap();
        assert_eq!(tail.len(), 1);
        assert_eq!(tail[0].ordinal, 31);
        assert_eq!(tail[0].child_id, rows[0].child_id);
        let read_cancel = CancelSource::new();
        read_cancel.cancel();
        assert!(
            runtime
                .expectation_samples(parent, 0, 0, 32, &read_cancel)
                .await
                .is_err()
        );
        assert_eq!(
            runtime.expectation_report(parent).await.unwrap().issued,
            64,
            "one cancelled read cannot change original science"
        );
        let before = original.samples.clone();
        let replay = runtime
            .execute_expectation(&package, parent, request, &CancelSource::new())
            .await
            .unwrap();
        assert_eq!(
            serde_json::to_vec(&report.report).unwrap(),
            serde_json::to_vec(&replay.report).unwrap()
        );
        assert_eq!(
            store
                .expectation_proposal(&parent.to_string(), 0)
                .await
                .unwrap()
                .unwrap()
                .samples,
            before
        );
        let mut changed = report.request.clone();
        changed.controls.seed += 1;
        assert!(
            runtime
                .execute_expectation(&package, parent, changed, &CancelSource::new())
                .await
                .is_err()
        );
        // A completed earlier boundary reproduces the actual next retained population.
        let second = store
            .expectation_proposal(&parent.to_string(), 1)
            .await
            .unwrap()
            .unwrap();
        let original_samples: Vec<ExpectationSample> =
            serde_json::from_slice(second.samples.as_slice()).unwrap();
        assert_eq!(original_samples.len(), 32);
        assert_eq!(original_samples[0].ordinal, 32);
        let boundary = store
            .expectation_boundary_at(&parent.to_string(), 0)
            .await
            .unwrap()
            .unwrap();
        assert!(
            store
                .canonical_study(&boundary.child)
                .await
                .unwrap()
                .unwrap()
                .terminal
        );
        let checkpoint: ExpectationCheckpoint =
            serde_json::from_slice(boundary.checkpoint.as_ref().unwrap().as_slice()).unwrap();
        let admitted = package
            .admit_expectation(
                &runtime.preparation_services(),
                report.request.clone(),
                &CancelSource::new(),
            )
            .await
            .unwrap();
        let original_parent = store
            .expectation(&parent.to_string())
            .await
            .unwrap()
            .unwrap();
        let mut restored =
            ExpectationParent::restore(admitted, parent, &original_parent.supplier, checkpoint)
                .unwrap();
        let child = StudyId::from_id(SemanticId::parse_hex(&second.child).unwrap());
        let regenerated = restored
            .prepare_iteration(&runtime.preparation_services(), child)
            .unwrap();
        for (a, b) in regenerated.samples.iter().zip(&original_samples) {
            assert_eq!(a.base, b.base);
            assert_eq!(a.categories, b.categories);
            assert_eq!(a.proposal_weight.to_bits(), b.proposal_weight.to_bits());
        }
        let reader = Science::new(store.independent_caller().unwrap(), true);
        let retained = reader.runtime.expectation_report(parent).await.unwrap();
        assert_eq!(
            serde_json::to_vec(&retained.report).unwrap(),
            serde_json::to_vec(&report.report).unwrap()
        );
        reader.close().await.unwrap();
    }
    {
        let mut invalid = request(runtime, &package, false, true).await;
        let declarations = package
            .declarations(&runtime.preparation_services())
            .await
            .unwrap();
        let root = declarations
            .iter()
            .find(|row| row.name == "InvalidPushforward")
            .unwrap()
            .declaration_id;
        let bindings = pse_modeling::Bindings {
            demand: vec![
                "u".into(),
                "flow".into(),
                "outlet".into(),
                "flow_out".into(),
            ],
            ..Default::default()
        };
        let model = package
            .prepare(
                &runtime.preparation_services(),
                root,
                pse_modeling::specialize::root_instance(root),
                bindings.clone(),
                invalid.preparation.limits,
                &CancelSource::new(),
            )
            .await
            .unwrap();
        let member = |name: &str| {
            model
                .compiled()
                .model
                .symbols
                .values()
                .find(|symbol| symbol.lineage.path.ends_with(&format!(".{name}")))
                .unwrap()
                .id
        };
        if let ExpectationCoordinate::Uniform {
            target,
            lower,
            upper,
        } = &mut invalid.coordinates[0]
        {
            *target = BindingTarget::Member(member("u"));
            *lower = physical_value(&package, "Scalar", "1", -2.);
            *upper = physical_value(&package, "Scalar", "1", -1.5);
        }
        if let ExpectationCoordinate::Categorical { target, .. } = &mut invalid.coordinates[1] {
            *target = BindingTarget::Member(member("flow"));
        }
        let transform = invalid.transform.as_mut().unwrap();
        transform.operation.root = root;
        transform.operation.instance = pse_modeling::specialize::root_instance(root);
        transform.operation.bindings = bindings;
        for (output, name) in transform
            .operation
            .outputs
            .iter_mut()
            .zip(["outlet", "flow_out"])
        {
            output.member = BindingTarget::Member(member(name));
        }
        for (output, name) in transform.outputs.iter_mut().zip(["outlet", "flow_out"]) {
            output.output = member(name);
        }
        let parent = StudyId::from_id(SemanticId::from_bytes([47; 16]));
        let error = runtime
            .execute_expectation(&package, parent, invalid, &CancelSource::new())
            .await
            .unwrap_err();
        let diagnostic = error.boundary_diagnostic();
        assert_eq!(
            diagnostic.class,
            pse_model::diagnostic::BoundaryClass::InvalidModel
        );
        assert_eq!(
            diagnostic.rule,
            pse_diagnostics::DiagnosticRule::StudyPolicyAdmission
        );
        assert!(diagnostic.observations.contains_key("sample_ordinal"));
        assert!(diagnostic.observations.contains_key("base_coordinate_0"));
        assert!(diagnostic.observations.contains_key("category_0"));
        assert!(diagnostic.observations.contains_key("proposal_weight"));
        assert!(!diagnostic.causes.is_empty());
        assert!(
            runtime
                .canonical_store()
                .unwrap()
                .expectation_proposal(&parent.to_string(), 0)
                .await
                .unwrap()
                .is_none(),
            "invalid law cannot activate a downstream child"
        );
    }
    {
        let declarations = package
            .declarations(&runtime.preparation_services())
            .await
            .unwrap();
        let selected = declarations
            .iter()
            .find(|row| row.name == "Refusal")
            .unwrap()
            .declaration_id;
        let bindings = pse_modeling::Bindings {
            demand: vec!["sample".into(), "value".into()],
            ..Default::default()
        };
        let settings = PreparationSettings::default();
        let model = package
            .prepare(
                &runtime.preparation_services(),
                selected,
                pse_modeling::specialize::root_instance(selected),
                bindings.clone(),
                settings.limits,
                &CancelSource::new(),
            )
            .await
            .unwrap();
        let member = |name: &str| {
            model
                .compiled()
                .model
                .symbols
                .values()
                .find(|symbol| symbol.lineage.path.ends_with(&format!(".{name}")))
                .unwrap()
                .id
        };
        let scalar = physical_value(&package, "Scalar", "1", 1.);
        let parent = StudyId::from_id(SemanticId::from_bytes([48; 16]));
        let operation = SelectedObservationOperation {
            root: selected,
            instance: pse_modeling::specialize::root_instance(selected),
            bindings,
            outputs: vec![SelectedObservationOutput {
                member: BindingTarget::Member(member("value")),
                quantity: scalar.quantity,
                unit: scalar.unit,
            }],
        };
        let request = ExpectationRequest {
            source: OperationSource::of(&package),
            preparation: settings,
            member: ExpectationMember::Observation(operation),
            coordinates: vec![ExpectationCoordinate::Categorical {
                target: BindingTarget::Member(member("sample")),
                categories: vec![
                    ExpectationCategory {
                        category: SemanticId::from_bytes([1; 16]),
                        value: physical_value(&package, "Scalar", "1", -2.),
                        probability: 0.25,
                    },
                    ExpectationCategory {
                        category: SemanticId::from_bytes([2; 16]),
                        value: physical_value(&package, "Scalar", "1", 0.),
                        probability: 0.75,
                    },
                ],
            }],
            transform: None,
            outputs: vec![ExpectationOutput {
                output: member("value"),
                quantity: scalar.quantity,
                unit: scalar.unit,
                reference: None,
                scale: scalar,
                absolute_error: None,
                relative_error: None,
                train: false,
            }],
            estimator: ExpectationEstimator::FixedMonteCarlo,
            controls: ExpectationControls {
                seed: 123,
                lanes: 4,
                samples_per_iteration: 32,
                maximum_iterations: 1,
                maximum_samples: 32,
                normalization_tolerance: 1e-12,
            },
            failure_probability: true,
            replay: ExpectationReplay::PopulationAndBoundary,
        };
        let report = runtime
            .execute_expectation(&package, parent, request, &CancelSource::new())
            .await
            .unwrap();
        assert!(!report.outputs[0].complete);
        assert!(report.outputs[0].estimate.is_none());
        assert!(report.outputs[0].standard_error.is_none());
        assert!(!report.outputs[0].unavailable.is_empty());
        assert_eq!(report.outputs[0].issued, 32);
        let proposal = runtime
            .canonical_store()
            .unwrap()
            .expectation_proposal(&parent.to_string(), 0)
            .await
            .unwrap()
            .unwrap();
        let samples: Vec<ExpectationSample> =
            serde_json::from_slice(proposal.samples.as_slice()).unwrap();
        let failed = samples
            .iter()
            .filter(|sample| sample.categories[0] == SemanticId::from_bytes([1; 16]))
            .count();
        assert_eq!(report.outputs[0].unavailable.len(), failed);
        assert!(report.failure_probability_complete);
        assert_eq!(
            report.failure_probability.as_ref().unwrap().estimate,
            failed as f64 / 32.
        );
    }
    for (parent_byte, selected_iteration) in [(46, 0), (49, 1)] {
        let mut stopped_request = request(runtime, &package, false, false).await;
        stopped_request.controls.samples_per_iteration = 64;
        stopped_request.controls.maximum_samples = 128;
        let stopped_parent = StudyId::from_id(SemanticId::from_bytes([parent_byte; 16]));
        let stop = CancelSource::new();
        let work = runtime.execute_expectation(&package, stopped_parent, stopped_request, &stop);
        tokio::pin!(work);
        let observed = async {
            loop {
                if let Some(proposal) = runtime
                    .canonical_store()
                    .unwrap()
                    .expectation_proposal(&stopped_parent.to_string(), selected_iteration)
                    .await
                    .unwrap()
                {
                    let key = pse_operations::canonical_studies::point_key(
                        &proposal.child,
                        pse_model::study::OccurrenceKey(0),
                    );
                    if runtime
                        .canonical_store()
                        .unwrap()
                        .canonical_study_point(&key)
                        .await
                        .unwrap()
                        .is_some_and(|point| point.attempt.is_some())
                    {
                        stop.cancel();
                        break;
                    }
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        };
        tokio::select! {
            result=&mut work=>panic!("original native population ended before the selected issuance boundary: {result:?}"),
            result=tokio::time::timeout(Duration::from_secs(120),observed)=>result.unwrap(),
        }
        let incomplete = work.await.unwrap();
        assert!(incomplete.interrupted);
        assert!(incomplete.outputs.iter().all(|output| !output.complete));
        assert!(!incomplete.failure_probability_complete);
        assert!(incomplete.failure_probability.is_none());
        let retained = runtime.expectation_report(stopped_parent).await.unwrap();
        assert_eq!(
            serde_json::to_vec(&incomplete.report).unwrap(),
            serde_json::to_vec(&retained.report).unwrap()
        );
        let boundary = runtime
            .canonical_store()
            .unwrap()
            .expectation_boundary_at(&stopped_parent.to_string(), 0)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            boundary.checkpoint.is_some(),
            selected_iteration == 1,
            "only a prior complete iteration remains restart authority"
        );
    }
    drop(package);
    runtime.clear_program_cache();
    let readers = (0..3)
        .map(|_| {
            Science::new(
                runtime
                    .canonical_store()
                    .unwrap()
                    .independent_caller()
                    .unwrap(),
                true,
            )
        })
        .collect::<Vec<_>>();
    #[cfg(feature = "engineering-service")]
    wire::journey(science).await;
    runtime.close();
    // Independent readers remain valid after all producer mathematical buffers close.
    for (ordinal, reader) in readers.iter().enumerate() {
        let parent = StudyId::from_id(SemanticId::from_bytes([40 + ordinal as u8; 16]));
        assert_eq!(
            reader
                .runtime
                .expectation_report(parent)
                .await
                .unwrap()
                .issued,
            64
        );
        assert_eq!(
            reader
                .runtime
                .expectation_samples(parent, 1, 0, 32, &CancelSource::new())
                .await
                .unwrap()
                .batch()
                .num_rows(),
            32
        );
        assert_eq!(
            reader
                .runtime
                .expectation_estimates(parent, &CancelSource::new())
                .await
                .unwrap()
                .batch()
                .num_rows(),
            2
        );
        reader.close().await.unwrap();
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn public_native_expectation_heater_preserves_law_children_and_independent_reading() {
    let mut service = service();
    let outcome=AssertUnwindSafe(service.run(Instant::now()+Duration::from_secs(600),|service|Box::pin(async move{
        service.launch(Instant::now()+Duration::from_secs(90)).await?;
        let mut science=Science::new(service.ready()?.store().clone(),true);
        let allocation=NativeAllocation::materialize(8<<30,512<<20,1,6<<30,Some(NativeExecutionAllocation{pool_memory_bytes:4<<30,worker_bytes:1<<30,cpu_threads:16,case_lanes:16,math_jobs:32,compiler_cores:1,admission_wait_ms:30_000,process_headroom_bytes:2<<30,observer_memory_bytes:512<<20})).unwrap();
        let mut worker=NativeStudyWorker::new(NativeStudyWorkerSpec{ready:service.ready()?,executable:PathBuf::from(std::env::var_os("CARGO_BIN_EXE_pse-worker").expect("Cargo provides the original native worker")),environment:native_worker_environment::selected(),allocation,settings:WorkerSettings{maximum_in_flight:Some(2),..Default::default()}}).unwrap();
        worker.launch(Instant::now()+Duration::from_secs(90)).await.unwrap();
        science.runtime=science.runtime.clone().with_managed_worker(worker.observer().unwrap()).unwrap();
        let passive=Science::new(service.ready()?.store().independent_caller().unwrap(),true);
        let body=AssertUnwindSafe(journey(&science)).catch_unwind().await;
        let worker_drain=worker.shutdown(Instant::now()+Duration::from_secs(45)).await;
        let producer_drain=science.close().await;
        let passive_read=AssertUnwindSafe(async {
            if body.is_ok()&&worker_drain.is_ok()&&producer_drain.is_ok() {
                for ordinal in 0..3 {
                    let parent=StudyId::from_id(SemanticId::from_bytes([40+ordinal;16]));
                    assert_eq!(passive.runtime.expectation_report(parent).await.unwrap().issued,64);
                    assert_eq!(passive.runtime.expectation_samples(parent,1,0,32,&CancelSource::new()).await.unwrap().batch().num_rows(),32);
                    assert_eq!(passive.runtime.expectation_estimates(parent,&CancelSource::new()).await.unwrap().batch().num_rows(),2);
                }
            }
        }).catch_unwind().await;
        let passive_drain=passive.close().await;
        if body.is_ok()&&worker_drain.is_ok(){worker.dispose().unwrap();}
        if let Err(panic)=body{eprintln!("expectation original worker={worker_drain:?};producer={producer_drain:?};reader={passive_drain:?};state={}",service.state().display());std::panic::resume_unwind(panic);}
        if let Err(panic)=passive_read{eprintln!("expectation passive cleanup={passive_drain:?};state={}",service.state().display());std::panic::resume_unwind(panic);}
        worker_drain.unwrap();producer_drain.unwrap();passive_drain.unwrap();Ok(())
    }))).catch_unwind().await;
    match outcome {
        Err(panic) => std::panic::resume_unwind(panic),
        Ok(result) => {
            result.unwrap();
            service.dispose().unwrap();
        }
    }
}
