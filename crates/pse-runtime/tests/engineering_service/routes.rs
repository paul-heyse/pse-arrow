// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Selected native operation families through the independent generated client.
//! The existing qualification coordinator retains the original keeper and worker.

use super::study::{read_result, status, submit};
use super::*;
use pse_ids::SemanticId;
use pse_model::{
    generated::{
        authored::fit_cases as fit_source,
        enums::{
            EngineeringScaleKind, ModelingAnalysisRoute, ModelingDeclarationKind, NumericalSource,
            NumericalTarget,
        },
        extension_values::QuantityValue,
        identities::{DeclarationId, FitId, RunId},
        structures::ScientificCoordinateValue,
    },
    scalars::{FiniteBound, PositiveCount, Tolerance},
    study::{OccurrenceKey, StartPolicy},
};
use pse_relations::columnar::ArrowValue as _;
use pse_runtime::{
    engineering_projection as codec, host_operations::engineering_contract::*, workflow::*,
};

#[path = "../../../../tests/fixtures/authored-package.rs"]
mod authored_package;

// Native dynamic-workflow ACCUMULATION fixture, with an explicit input-owned Time
// characteristic added for the client output-error assertion before execution.
const ACCUMULATION: &str = r#"package accumulation {
  def Experiment {
    domain t:Time from 0{s} to 1{s};
    discretize grid on t using integrated(elements=1,order=1);
    param rate:Scalar=3; var total[i in t]:Time;
    eq balance[i in t]:d(total[i])/di==rate;
    eq initial:total[0{s}]==2{s};
    let observed[i in t]:Time=total[i];
    annotation report observed("measurement");
    annotation check total(total[i]>=2{s});
    annotation engineering_scale total(kind=magnitude,value=2{s});
  }
  test experiment_integrated fixture {
    dof 0; route integrated; procedure integrate;
    integrate samples(0{s},0.5{s},1{s}) relative(global)
      normalized_absolute(global) step(1e-4{s});
  } { child root:Experiment=Experiment(); }
}"#;

// Actual Measurement fixture from native_scientific_modes, retaining the original
// two-metre observation and its provenance. The fit's separate output allowance
// below comes from an explicit Length engineering scale, never this sigma.
const MEASUREMENT: &str = r#"package fitting {
  @id("9c9c9c9c9c9c9c9c9c9c9c9c9c9c9c9c") def Measurement {
    param length:Length=2{m}; annotation check length(length>0{m});
  }
  entity kind origin provenance {attribute title:Text;}
  entity origin experiment {title="known length"}
  enum role {measured facets(measured)}
  entity kind reading {attribute value:Length?;attribute sigma:Length?;}
  @id("9d9d9d9d9d9d9d9d9d9d9d9d9d9d9d9d")
  entity reading measured provenance(experiment,role.measured) {value=2{m},sigma=0.1{m}}
}"#;

fn identity(byte: u8) -> SemanticId {
    SemanticId::from_bytes([byte; 16])
}

fn source_request(operation: &str) -> pb::EngineeringAdmitSource {
    let registry = pse_schema::shared_registry().unwrap();
    let physical = authored_package::package_documents(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
        "packages/reference/physical",
        &registry,
    )
    .unwrap();
    let header: toml::Value =
        toml::from_str(std::str::from_utf8(&physical["package.toml"]).unwrap()).unwrap();
    let version = header["package"]["version"]
        .as_str()
        .unwrap()
        .split('.')
        .map(|part| part.parse::<i64>().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(version.len(), 3);
    let mut input = source(operation.into());
    input.physical = vec![pb::EngineeringPackage {
        files: physical.into_iter().collect(),
        ..Default::default()
    }];
    let mut manifest: toml::Value =
        toml::from_str(std::str::from_utf8(&input.modeling[0].files["package.toml"]).unwrap())
            .unwrap();
    manifest["package"]["dependencies"] = toml::Value::Array(vec![
        toml::Table::from_iter([
            ("package_id".into(), header["package"]["package_id"].clone()),
            (
                "version_req".into(),
                toml::Table::from_iter([
                    ("operator".into(), "exact".into()),
                    ("major".into(), version[0].into()),
                    ("minor".into(), version[1].into()),
                    ("patch".into(), version[2].into()),
                ])
                .into(),
            ),
        ])
        .into(),
    ]);
    input.modeling[0].files.insert(
        "package.toml".into(),
        toml::to_string(&manifest).unwrap().into_bytes(),
    );
    input.modeling[0]
        .files
        .insert("models/root.pse".into(), ACCUMULATION.as_bytes().to_vec());
    input.modeling[0].files.insert(
        "models/measurement.pse".into(),
        MEASUREMENT.as_bytes().to_vec(),
    );
    let fit = fit_source::Row {
        fit_id: identity(158).into(),
        parameters: vec![fit_source::AuthoredFitCasesFieldParametersItem {
            symbol_id: identity(153),
            fixed: false,
            value: 1.0,
            lower: None,
            upper: None,
            scale: 1.0,
        }],
        experiments: vec![fit_source::AuthoredFitCasesFieldExperimentsItem {
            experiment_id: identity(159).into(),
            case_id: identity(156).into(),
            route: ModelingAnalysisRoute::Steady,
            bindings: vec![
                fit_source::AuthoredFitCasesFieldExperimentsItemBindingsItem {
                    parameter_id: identity(153),
                    path: "length".into(),
                    schedule_start: None,
                },
            ],
        }],
        observations: vec![fit_source::AuthoredFitCasesFieldObservationsItem {
            observation_id: identity(157).into(),
            experiment_id: identity(159).into(),
            output_path: "length".into(),
            value_attribute: "value".into(),
            standard_deviation_attribute: Some("sigma".into()),
            time: None,
            time_basis: None,
            time_unit_id: None,
            included: true,
            importance: 1.0,
        }],
    };
    input.modeling[0].files.insert(
        "cases/measurement.yaml".into(),
        serde_json::to_vec(&serde_json::json!({"fit_cases": [fit]})).unwrap(),
    );
    input
}

async fn quantity(
    client: &EngineeringClient,
    operation: &str,
    name: &str,
) -> PhysicalQuantityDescription {
    let document = codec::decode_physical_inspection_document(
        client
            .generated()
            .inspect_physical(pb::EngineeringInspectPhysical {
                source_operation: operation.into(),
                request: codec::encode_physical_inspection_request(PhysicalInspectionRequest {
                    query: PhysicalInspectionQuery::QuantityNamed { name: name.into() },
                    offset: 0,
                    limit: 1,
                })
                .unwrap()
                .into(),
                ..Default::default()
            })
            .await
            .unwrap()
            .into_owned(),
    )
    .unwrap();
    let PhysicalInspectionDocument::Page {
        mut quantities,
        count,
        ..
    } = document
    else {
        panic!("required original physical quantity is unavailable")
    };
    assert_eq!(count, 1);
    assert_eq!(quantities.len(), 1);
    quantities.remove(0)
}

fn value(
    coordinate_id: SemanticId,
    magnitude: f64,
    physical: &PhysicalQuantityDescription,
) -> ScientificCoordinateValue {
    ScientificCoordinateValue {
        coordinate_id,
        value: QuantityValue {
            value: magnitude,
            quantity_type_id: physical.quantity.id.as_id(),
            unit_id: physical.canonical_unit.id.as_id(),
        },
    }
}

fn point(operation: OperationRequest, key: u32) -> StudyPoint {
    StudyPoint {
        operation,
        preparation: PreparationSettings::default(),
        overlay: PointOverlay::default(),
        policy: StudyPointPolicy {
            key: OccurrenceKey(key),
            dependencies: Vec::new(),
            start: StartPolicy::Fresh,
            attempt_limit: 1,
        },
    }
}

async fn members(
    client: &EngineeringClient,
    capture: &str,
) -> Vec<CapturedStudyRow<StudyCohortMember>> {
    let page = codec::decode_study_cohort_member_page(
        client
            .study()
            .read_study_members(
                codec::encode_engineering_study_cohort_page(EngineeringStudyCohortPage {
                    operation: capture.into(),
                    request: StudyCohortPageRequest {
                        maximum: 4,
                        cursor: None,
                    },
                })
                .unwrap(),
            )
            .await
            .unwrap()
            .into_owned(),
    )
    .unwrap();
    assert!(page.next.is_none());
    page.members
}

fn batches(page: &EngineeringStudyResultPage) -> Vec<datafusion::arrow::record_batch::RecordBatch> {
    let batches = datafusion::arrow::ipc::reader::StreamReader::try_new(
        std::io::Cursor::new(page.ipc.as_slice()),
        None,
    )
    .unwrap()
    .map(Result::unwrap)
    .collect::<Vec<_>>();
    assert_eq!(
        batches
            .iter()
            .map(|batch| batch.num_rows() as u64)
            .sum::<u64>(),
        page.rows
    );
    for batch in &batches {
        for row in 0..batch.num_rows() {
            assert_eq!(
                format!(
                    "run:{}",
                    RunId::read(batch.column_by_name("run_id").unwrap().as_ref(), row).unwrap()
                ),
                page.run
            );
        }
    }
    batches
}

fn output_budget(
    numerics: &EngineeringStudyResultPage,
    target: SemanticId,
    physical: &PhysicalQuantityDescription,
    expected_characteristic: f64,
) -> f64 {
    let mut found = None;
    for batch in batches(numerics) {
        for row in 0..batch.num_rows() {
            let field = |name| batch.column_by_name(name).unwrap().as_ref();
            if SemanticId::read(field("target_id"), row).unwrap() != target
                || NumericalTarget::read(field("target_kind"), row).unwrap()
                    != NumericalTarget::Variable
            {
                continue;
            }
            assert!(
                found.is_none(),
                "one exact original variable owns the selected output allowance"
            );
            assert_eq!(
                SemanticId::read(field("quantity_id"), row).unwrap(),
                physical.quantity.id.as_id()
            );
            assert_eq!(
                SemanticId::read(field("unit_id"), row).unwrap(),
                physical.canonical_unit.id.as_id()
            );
            let context =
                Option::<pse_model::numerics::EngineeringContext>::read(field("engineering"), row)
                    .unwrap()
                    .expect("original explicit physical scale must be retained");
            assert!(!context.canonical_fallback);
            assert_eq!(context.scale_kind, Some(EngineeringScaleKind::Magnitude));
            assert!(context.scale_id.is_some());
            assert_eq!(
                context.characteristic.unwrap().to_bits(),
                expected_characteristic.to_bits()
            );
            let fraction =
                pse_model::numerics::NumericalPolicy::default().engineering_relative_fraction;
            assert_eq!(context.relative_fraction.to_bits(), fraction.to_bits());
            let budget = f64::read(field("budget"), row).unwrap();
            assert!(budget.is_finite() && budget > 0.0);
            assert_eq!(
                budget.to_bits(),
                (fraction * expected_characteristic).to_bits()
            );
            assert_eq!(context.budget.to_bits(), budget.to_bits());
            found = Some(budget);
        }
    }
    found.expect("original source/target/quantity/unit allowance must be present")
}

#[tokio::test(flavor = "multi_thread")]
async fn prepared_engineering_study_client_simulation_fit_and_horizon_keep_original_physics() {
    let prefix = format!("study-routes-{}", std::process::id());
    let source_operation = format!("{prefix}-source");
    let mut client = EngineeringClient::connect(endpoint()).await.unwrap();
    client
        .generated()
        .admit_source(source_request(&source_operation))
        .await
        .unwrap();
    let original = selected(&mut client, &source_operation).await;
    let source_headers = client
        .generated()
        .inspect_source(pb::EngineeringInspectSource {
            operation: source_operation.clone(),
            offset: 0,
            limit: 128,
            ..Default::default()
        })
        .await
        .unwrap()
        .into_owned();
    let case = DeclarationId::from_id(SemanticId::from_bytes(
        source_headers
            .declarations
            .iter()
            .find(|row| row.name == "experiment_integrated")
            .unwrap()
            .declaration
            .as_slice()
            .try_into()
            .unwrap(),
    ));
    let inspection = codec::decode_modeling_inspection(
        client
            .generated()
            .inspect_modeling(
                codec::encode_engineering_inspect_modeling(EngineeringInspectModeling {
                    source_operation: source_operation.clone(),
                    request: ModelingInspectionRequest {
                        case,
                        preparation: PreparationSettings::default(),
                        settings: Default::default(),
                        offset: 0,
                        limit: 128,
                    },
                })
                .unwrap(),
            )
            .await
            .unwrap()
            .into_owned(),
    )
    .unwrap();
    assert!(inspection.next_offset.is_none());
    let rate = inspection
        .members
        .iter()
        .find(|member| member.lineage.path.ends_with(".rate"))
        .unwrap()
        .id;
    let state_declaration = DeclarationId::from_id(SemanticId::from_bytes(
        source_headers
            .declarations
            .iter()
            .find(|row| row.name == "total")
            .unwrap()
            .declaration
            .as_slice()
            .try_into()
            .unwrap(),
    ));
    // The integrated derivative has its own generated Variable coordinate.
    // The physical state is the coordinate of the original authored variable.
    let states = inspection
        .members
        .iter()
        .filter(|member| {
            member.role == ModelingDeclarationKind::Variable
                && member.lineage.declaration == state_declaration
        })
        .collect::<Vec<_>>();
    assert_eq!(
        states.len(),
        1,
        "the original integrated fixture owns one differential state"
    );
    let state = states[0].id;
    let time = quantity(&client, &source_operation, "Time").await;
    let scalar = quantity(&client, &source_operation, "Scalar").await;
    let length = quantity(&client, &source_operation, "Length").await;
    assert_eq!(time.canonical_unit.symbol, "s");
    assert_eq!(length.canonical_unit.symbol, "m");
    let fraction = pse_model::numerics::NumericalPolicy::default().engineering_relative_fraction;
    let controls = pse_backend_native::dynamics::Profile {
        method: pse_backend_native::dynamics::Method::Diffsol,
        start: 0.0,
        end: 1.0,
        samples: vec![0.0, 0.5, 1.0],
        rtol: fraction,
        initial_step: 1e-4,
        ..Default::default()
    }
    .with_coordinates(
        vec![value(state, 2.0 * fraction, &time)],
        Vec::new(),
        vec![value(rate, 1.0, &scalar)],
    );
    let simulation = SimulationOperation {
        case,
        settings: Some(PhysicalSimulationRequest::new(controls)),
    };
    let horizon = HorizonOperation {
        plant: simulation.clone(),
        period: Tolerance::try_new(0.5).unwrap(),
        steps: PositiveCount::try_new(2).unwrap(),
        inputs: vec![HorizonInputDocument {
            parameter: rate,
            initial: BindingQuantity {
                magnitude: FiniteBound::try_new(3.0).unwrap(),
                quantity: scalar.quantity.id,
                unit: scalar.canonical_unit.id,
            },
        }],
        estimator: None,
        controller: None,
    };
    let mut fit_solver = pse_runtime::math::settings::SolveSettings {
        intent: pse_backend_native::solve::SolveIntent::Optimize,
        backend: Some(pse_backend_native::solve::Backend::Ipopt),
        ..Default::default()
    };
    fit_solver
        .numerics
        .engineering_scales
        .push(pse_model::numerics::EngineeringScale {
            scale_id: identity(154).into(),
            model_id: None,
            case_id: None,
            instance_id: None,
            fit_id: Some(FitId::from_id(identity(158))),
            target_id: identity(153),
            target_kind: NumericalTarget::Variable,
            quantity_id: length.quantity.id.as_id(),
            unit_id: length.canonical_unit.id.as_id(),
            kind: EngineeringScaleKind::Magnitude,
            value: 2.0,
            source: NumericalSource::Analysis,
            priority: 100,
            provenance: "original native Measurement source is two metres before fitting".into(),
        });
    let fit = FitOperation {
        fit: identity(158).into(),
        settings: PhysicalFitRequest::new(FitPreparationDocument {
            optimizer: FitOptimizer::NativeNlp,
            solver: fit_solver,
            simulations: BTreeMap::new(),
            rank_tolerance: 1e-8,
            max_cells: 100_000,
            derivatives: FitDerivatives::Responses,
            experiment_workers: PositiveCount::try_new(1).unwrap(),
            uncertainty: None,
        }),
    };
    let prepare = EngineeringPrepareExplicitStudy {
        operation: format!("{prefix}-prepare"),
        source_operation: source_operation.clone(),
        expected_source: original.local_revision.clone(),
        output: StudyOutputDemand::Full {},
        points: vec![
            point(OperationRequest::Simulation(simulation), 1),
            point(OperationRequest::Fit(fit), 2),
            point(OperationRequest::Horizon(Box::new(horizon.clone())), 3),
        ],
    };
    let mut incompatible = prepare.clone();
    incompatible.operation.push_str("-physical-refusal");
    let mut incompatible_horizon = horizon;
    incompatible_horizon.inputs[0].initial.quantity = length.quantity.id;
    incompatible_horizon.inputs[0].initial.unit = length.canonical_unit.id;
    incompatible.points = vec![point(
        OperationRequest::Horizon(Box::new(incompatible_horizon)),
        3,
    )];
    let refused = failure(
        client
            .explicit_study()
            .prepare_explicit_study(codec::encode_engineering_prepare_explicit_study(incompatible).unwrap())
            .await
            .unwrap_err(),
    );
    assert_eq!(
        refused.rule.as_deref(),
        Some(pse_diagnostics::DiagnosticRule::StudyBindingPhysical.as_str()),
        "original Horizon admission refusal: {refused:?}"
    );
    assert_eq!(
        refused.code,
        pse_diagnostics::DiagnosticCode::StudyBindingPhysical.as_str()
    );
    assert!(
        refused
            .sources
            .iter()
            .any(|source| source.as_slice() == rate.as_bytes()),
        "physical route refusal must preserve the actual original driven parameter identity"
    );
    let prepared = codec::decode_engineering_prepared_study(
        client
            .explicit_study()
            .prepare_explicit_study(codec::encode_engineering_prepare_explicit_study(prepare).unwrap())
            .await
            .unwrap()
            .into_owned(),
    )
    .unwrap();
    assert_eq!(prepared.occurrences, 3);
    let submitted = EngineeringSubmitStudy {
        operation: format!("{prefix}-issued"),
        preparation_operation: prepared.operation,
        expected_preparation: prepared.preparation,
        study: pse_operations::mint_id(),
    };
    let registered = submit(&client, submitted.clone()).await.canonical.unwrap();
    let mut stream = client
        .study()
        .observe_study(
            codec::encode_engineering_observe_study(EngineeringObserveStudy {
                original: EngineeringStudyKey {
                    operation: submitted.operation.clone(),
                    study: submitted.study,
                },
                request: StudyObservationRequest {
                    study: submitted.study.to_string(),
                    maximum_members: 3,
                    maximum_events: 16,
                },
                cursor: None,
            })
            .unwrap(),
        )
        .await
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(120);
    let mut horizon_events = Vec::new();
    let mut last = None;
    loop {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .expect("native route study deadline");
        let next = tokio::time::timeout(remaining, stream.message::<pb::StudyObservationPage>())
            .await
            .unwrap()
            .unwrap();
        let Some(page) = next else {
            break;
        };
        let page = codec::decode_study_observation_page(page.to_owned_message()).unwrap();
        assert_eq!(page.status.header.run, registered.run);
        for source in &page.sources {
            assert!(source.history_refusal.is_none());
            horizon_events.extend(
                source
                    .events
                    .iter()
                    .filter(|event| event.phase == "horizon.step")
                    .cloned(),
            );
        }
        last = Some(page);
    }
    let last = last.unwrap();
    assert!(last.status.header.terminal);
    assert!(
        last.sources
            .iter()
            .all(|source| source.position.history_complete)
    );
    assert_eq!(
        status(&client, &submitted).await.canonical.unwrap().run,
        registered.run
    );
    let capture = format!("{prefix}-capture");
    // This cohort has three members; use its actual complete finite request rather
    // than the two-member helper selected by the separate lifecycle fixture.
    let receipt = codec::decode_study_cohort_receipt(
        client
            .study()
            .freeze_study_cohort(
                codec::encode_engineering_freeze_study_cohort(EngineeringFreezeStudyCohort {
                    operation: capture.clone(),
                    request: StudyCohortRequest {
                        filter: StudyWorkbenchFilter {
                            study: Some(submitted.study.to_string()),
                            ..Default::default()
                        },
                        maximum_studies: 1,
                        maximum_members: 3,
                    },
                })
                .unwrap(),
            )
            .await
            .unwrap()
            .into_owned(),
    )
    .unwrap();
    assert_eq!(receipt.member_count, 3);
    assert_eq!(receipt.header_count, 1);
    assert!(receipt.complete);
    let members = members(&client, &capture).await;
    assert_eq!(members.len(), 3);
    assert!(
        members
            .iter()
            .all(|member| member.value.settled && member.value.facts.scientific.usable)
    );
    let input = |key, relation: &str| {
        let member = members
            .iter()
            .find(|member| member.value.facts.key == OccurrenceKey(key))
            .unwrap();
        EngineeringReadStudyResult {
            operation: capture.clone(),
            position: usize::try_from(member.position).unwrap(),
            relation: relation.into(),
            start: 0,
            end: 128,
            maximum_bytes: 1 << 20,
        }
    };
    let sim_numerics = read_result(&client, input(1, "runtime.resolved_numerics")).await;
    let state_budget = output_budget(&sim_numerics, state, &time, 2.0);
    let samples = read_result(&client, input(1, "runtime.simulation_samples")).await;
    let mut sample_count = 0;
    let mut sample_members = std::collections::BTreeSet::new();
    let mut sampled_symbols = std::collections::BTreeSet::new();
    for batch in batches(&samples) {
        for row in 0..batch.num_rows() {
            let field = |name| batch.column_by_name(name).unwrap().as_ref();
            let at = f64::read(field("time"), row).unwrap();
            let sample = usize::try_from(i64::read(field("sample"), row).unwrap()).unwrap();
            assert_eq!(at.to_bits(), [0.0_f64, 0.5, 1.0][sample].to_bits());
            let symbol = SemanticId::read(field("symbol_id"), row).unwrap();
            assert!(
                sample_members.insert((sample, symbol)),
                "each original coordinate/time appears once"
            );
            sampled_symbols.insert(symbol);
            assert!([0.0, 0.5, 1.0].contains(&at));
            assert_eq!(
                SemanticId::read(field("quantity_id"), row).unwrap(),
                time.quantity.id.as_id()
            );
            assert_eq!(
                SemanticId::read(field("unit_id"), row).unwrap(),
                time.canonical_unit.id.as_id()
            );
            assert!(
                (f64::read(field("value"), row).unwrap() - (2.0 + 3.0 * at)).abs() <= state_budget
            );
            sample_count += 1;
        }
    }
    assert_eq!(
        sample_count, 6,
        "original state and authored observation at all three requested times"
    );
    assert_eq!(sampled_symbols.len(), 2);
    assert!(sampled_symbols.contains(&state));
    let fit_numerics = read_result(&client, input(2, "runtime.resolved_numerics")).await;
    let fit_budget = output_budget(&fit_numerics, identity(153), &length, 2.0);
    let estimates = read_result(&client, input(2, "runtime.fit_parameters")).await;
    assert_eq!(estimates.rows, 1);
    for batch in batches(&estimates) {
        assert_eq!(
            SemanticId::read(batch.column_by_name("parameter_id").unwrap().as_ref(), 0).unwrap(),
            identity(153)
        );
        assert_eq!(
            SemanticId::read(batch.column_by_name("unit_id").unwrap().as_ref(), 0).unwrap(),
            length.canonical_unit.id.as_id()
        );
        let fitted = Option::<f64>::read(batch.column_by_name("value").unwrap().as_ref(), 0)
            .unwrap()
            .unwrap();
        assert!(fitted.is_finite() && (fitted - 2.0).abs() <= fit_budget);
    }
    assert_eq!(
        horizon_events.len(),
        2,
        "both original open-loop horizon steps retain native observations"
    );
    let real = |metric: &ProgressMetricDocument| match metric {
        ProgressMetricDocument::Real(pse_model::diagnostic::Observation::Real(value)) => *value,
        other => panic!("original finite horizon magnitude required: {other:?}"),
    };
    for (index, event) in horizon_events.iter().enumerate() {
        let at = real(&event.values["time"]);
        assert_eq!(at.to_bits(), (index as f64 * 0.5).to_bits());
        assert!(
            matches!(&event.values["decision"], ProgressMetricDocument::Text(value) if value == "open_loop")
        );
        assert_eq!(real(&event.values["input.0"]).to_bits(), 3.0_f64.to_bits());
        // Source Time coordinate and original two-second characteristic remain
        // fixed before either route. This is an independent linear-response check.
        assert!((real(&event.values["output.0"]) - (2.0 + 3.0 * at)).abs() <= state_budget);
    }
    for member in &members {
        assert!(member.value.attempt.is_some() && member.value.manifest.is_some());
        for product in [&sim_numerics, &samples, &fit_numerics, &estimates] {
            if product.position as u64 == member.position {
                assert_eq!(product.capture, receipt.capture);
                assert_eq!(product.run, member.value.run);
                assert_eq!(Some(&product.attempt), member.value.attempt.as_ref());
                assert_eq!(Some(&product.manifest), member.value.manifest.as_ref());
            }
        }
    }
    client
        .study()
        .release_study_cohort(pb::EngineeringOperationKey {
            operation: capture,
            ..Default::default()
        })
        .await
        .unwrap();
}
