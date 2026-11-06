// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual aqueous cell stocks and independently accumulated boundary closure.
use super::*;
use crate::{math::solves::NumericalInputs, workflow::tests as fixture};
#[cfg(feature = "solver-diffsol")]
use pse_backend_native::dynamics::Method;
use pse_backend_native::solve::{Backend, SolverSelection};
#[cfg(feature = "solver-diffsol")]
use pse_modeling::{
    SpecializedModel,
    specialize::{
        Value,
        finite::{FamilyKey, MemberKey},
    },
};
use std::collections::BTreeSet;

async fn conform(name: &str) -> ModelingConformanceReport {
    let (runtime, package, _spill, preparation) =
        pr_jacobian_tests::ordinary_delivery_package(false)
            .await
            .unwrap();
    let services = runtime.execution_services();
    let selected = package
        .declarations(services.preparation())
        .await
        .unwrap()
        .into_iter()
        .find(|row| row.name == name)
        .unwrap()
        .declaration_id;
    let mut solver = fixture::profile();
    solver.selection = SolverSelection::Explicit(Backend::Ipopt);
    let policy = ModelingConformancePolicy {
        compiler: preparation.compiler,
        solver,
        numerical: NumericalInputs::default(),
        limits: preparation.limits,
        derivatives: pse_backend_native::derivative_diagnostics::Policy {
            perturbation: 1e-6,
            relative_tolerance: 1e-4,
            maximum_cells: 20_000_000,
        },
        maximum_fixtures: 1,
        maximum_checks: 100_000,
        fixtures: ModelingFixtureSelection::Selected(BTreeSet::from([selected])),
        diagnostics: None,
    };
    package
        .conform(&services, policy, &crate::CancelSource::new())
        .await
        .unwrap()
}

fn assert_complete(report: &ModelingConformanceReport) {
    assert!(
        report.complete && report.passed(),
        "{:?}; {:?}",
        report.checks,
        report.failures
    );
    assert_eq!(report.results.len(), 1);
    let result = report.results.values().next().unwrap();
    let closure = result
        .reports
        .iter()
        .filter(|row| row.label == "aqueous cascade global material closure")
        .collect::<Vec<_>>();
    assert_eq!(
        closure.len(),
        2,
        "both actual water and ethanol inventories require closure"
    );
    for row in closure {
        assert!(row.value.abs() <= 0.001, "{row:?}");
    }
    let inventories = result
        .reports
        .iter()
        .filter(|row| row.label == "aqueous cascade component inventory")
        .collect::<Vec<_>>();
    assert!(
        inventories.len() > 2,
        "retain physical inventory across the original timeline"
    );
    assert!(inventories.iter().all(|row| row.value >= -0.001));
    assert!(
        result
            .reports
            .iter()
            .any(|row| row.label == "aqueous cascade imposed temperature" && row.value == 300.)
    );
}

#[tokio::test]
async fn aqueous_inventory_single_cell_matches_exact_mixed_cell_response() {
    assert_complete(&conform("aqueous_single_cell_step").await);
}

#[tokio::test]
async fn aqueous_inventory_four_cells_preserve_nonzero_clock_and_global_stock_balance() {
    assert_complete(&conform("aqueous_four_cell_nonzero_clock").await);
}

#[tokio::test]
async fn aqueous_inventory_zero_flow_retains_original_component_stock() {
    assert_complete(&conform("aqueous_four_cell_zero_flow").await);
}

#[tokio::test]
async fn aqueous_geometry_requires_readmission_instead_of_an_independent_numeric_override() {
    let (runtime, package, _spill, preparation) =
        pr_jacobian_tests::ordinary_delivery_package(false)
            .await
            .unwrap();
    let services = runtime.execution_services();
    let cancel = crate::CancelSource::new();
    let selected = package
        .declarations(services.preparation())
        .await
        .unwrap()
        .into_iter()
        .find(|row| row.name == "aqueous_single_cell_integrated")
        .unwrap()
        .declaration_id;
    let mut solver = fixture::profile();
    solver.selection = SolverSelection::Explicit(Backend::Ipopt);
    let declared = package
        .declared_execution(
            services.preparation(),
            selected,
            preparation.compiler,
            solver,
            NumericalInputs::default(),
            preparation.limits,
            &cancel,
        )
        .await
        .unwrap();
    // Constructor volume belongs to original admission; the derived width is
    // neither an independently mutable coordinate nor a numeric permission.
    for (path, expected) in [
        ("root.volume", "unknown instantiated member volume"),
        (
            "root.cell_fraction",
            "case value root.cell_fraction requires a finite independent coordinate",
        ),
    ] {
        let mut changed = declared.analysis.clone();
        changed.case.values.insert(path.into(), 0.002);
        let error = package
            .prepare_analysis(services.preparation(), &changed, &cancel)
            .await
            .unwrap_err();
        assert!(error.to_string().contains(expected), "{path}: {error}");
    }
}

#[cfg(feature = "solver-diffsol")]
#[derive(Clone)]
struct ComponentRead {
    component: DeclarationId,
    original_stock: f64,
    stocks: Vec<(SemanticId, usize)>,
    incoming: usize,
    outgoing: usize,
}

#[cfg(feature = "solver-diffsol")]
fn finite_family(
    model: &SpecializedModel,
    instance: InstanceId,
    declaration: DeclarationId,
) -> &[Vec<Value>] {
    model
        .finite_facts
        .family(&FamilyKey {
            instance,
            declaration,
            role: "binding.indices.domain".into(),
            parents: Vec::new(),
        })
        .unwrap()
}

#[cfg(feature = "solver-diffsol")]
fn component_reads(trajectory: &ModelingTrajectory, initial_tracer: f64) -> Vec<ComponentRead> {
    let declaration = |text| SemanticId::parse_hex(text).unwrap().into();
    let model = &trajectory.prepared().model().compiled().model;
    let contract = trajectory.prepared().contract();
    let cascade = model
        .instances
        .values()
        .find(|instance| instance.definition == declaration("66e03000000000000000000000000120"))
        .unwrap()
        .id;
    let supplied = declaration("66e03000000000000000000000000130");
    let discharged = declaration("66e03000000000000000000000000131");
    let phase_amount = declaration("64e02000000000000000000000000207");
    let cell = declaration("66e03000000000000000000000000110");
    let components = finite_family(model, cascade, supplied);
    assert_eq!(components.len(), 2);
    // Existing species catalogue identities, not quantity or row-order guesses.
    [
        (declaration("01a0e1461e93764398a669ec398dd8a0"), 55.),
        (
            declaration("01a0e1461e93764398a669f485d31290"),
            initial_tracer,
        ),
    ]
    .into_iter()
    .map(|(component, original_stock)| {
        let tuple = components
            .iter()
            .find(
                |tuple| matches!(tuple.as_slice(), [Value::Entity { id, .. }] if *id == component),
            )
            .unwrap();
        let quadrature = |declaration| {
            let member = model
                .finite_facts
                .require_member(&MemberKey {
                    instance: cascade,
                    declaration,
                    tuple: tuple.clone(),
                })
                .unwrap();
            let integral = model.integral_of(member).unwrap();
            contract
                .quadratures
                .iter()
                .position(|id| *id == integral)
                .unwrap()
        };
        let stocks = model
            .instances
            .values()
            .filter(|instance| instance.definition == cell)
            .map(|instance| {
                let candidates = finite_family(model, instance.id, phase_amount)
                    .iter()
                    .filter(|phase_tuple| phase_tuple.last() == tuple.first())
                    .collect::<Vec<_>>();
                assert_eq!(
                    candidates.len(),
                    1,
                    "one actual liquid-phase stock per component/cell"
                );
                let member = model
                    .finite_facts
                    .require_member(&MemberKey {
                        instance: instance.id,
                        declaration: phase_amount,
                        tuple: candidates[0].clone(),
                    })
                    .unwrap();
                let index = contract.states.iter().position(|id| *id == member).unwrap();
                assert!(contract.differential[index]);
                (member, index)
            })
            .collect();
        ComponentRead {
            component,
            original_stock,
            stocks,
            incoming: quadrature(supplied),
            outgoing: quadrature(discharged),
        }
    })
    .collect()
}

#[cfg(feature = "solver-diffsol")]
fn prefix_balances(
    trajectory: &ModelingTrajectory,
    samples: &[&pse_backend_native::dynamics::Sample],
    times: &[f64],
    reads: &[ComponentRead],
) -> Result<(), String> {
    if samples.iter().map(|sample| sample.time).collect::<Vec<_>>() != times {
        return Err("original requested physical point is missing or reordered".into());
    }
    let contract = trajectory.prepared().contract();
    for sample in samples {
        if sample.integrals.len() != contract.quadratures.len() {
            return Err("native cumulative quadrature extent differs".into());
        }
        // This owner only projects produced native coordinates to canonical
        // physical values; it does not reevaluate a model or a definite integral.
        let (_, physical, _) = trajectory
            .sample_context(sample)
            .map_err(|error| error.to_string())?;
        for read in reads {
            let amount = read.stocks.iter().try_fold(0., |total, (member, index)| {
                if contract.states.get(*index) != Some(member) || sample.state.get(*index).is_none()
                {
                    return Err(
                        "actual phase stock/native coordinate correspondence differs".to_owned(),
                    );
                }
                physical
                    .scalars
                    .get(member)
                    .map(|amount| total + amount)
                    .ok_or_else(|| "actual phase stock is unavailable".to_owned())
            })?;
            let error = (amount - read.original_stock)
                - (sample.integrals[read.incoming] - sample.integrals[read.outgoing]);
            if !error.is_finite() || error.abs() > 0.001 {
                return Err(format!(
                    "component {:?} at {}s: independent global stock/flux error {error}mol",
                    read.component, sample.time
                ));
            }
        }
    }
    Ok(())
}

#[cfg(feature = "solver-diffsol")]
async fn assert_integrated(
    name: &str,
    cells: usize,
    times: &[f64],
    response: &str,
    stock: &str,
    initial_tracer: f64,
    counterexamples: bool,
) {
    let report = conform(name).await;
    assert!(
        report.complete && report.passed(),
        "{:?}; {:?}",
        report.checks,
        report.failures
    );
    assert!(
        report.results.is_empty(),
        "an integrated trajectory is its own native result"
    );
    assert_eq!(report.trajectories.len(), 1);
    let trajectory = report.trajectories.values().next().unwrap();
    assert!(trajectory.accepted() && trajectory.checks_complete());
    assert_eq!(trajectory.prepared().profile().method, Method::Diffsol);
    assert_eq!(
        trajectory
            .report()
            .samples
            .iter()
            .map(|sample| sample.time)
            .collect::<Vec<_>>(),
        times
    );
    let cell: DeclarationId = SemanticId::parse_hex("66e03000000000000000000000000110")
        .unwrap()
        .into();
    assert_eq!(
        trajectory
            .prepared()
            .model()
            .compiled()
            .model
            .instances
            .values()
            .filter(|instance| instance.definition == cell)
            .count(),
        cells,
        "requested output times must not create additional physical cells"
    );
    assert_eq!(
        trajectory
            .prepared()
            .contract()
            .differential
            .iter()
            .filter(|state| **state)
            .count(),
        cells * 2,
        "water and ethanol actual amount are the differential states per cell"
    );
    for source in [response, stock] {
        let source: DeclarationId = SemanticId::parse_hex(source).unwrap().into();
        for &time in times {
            let checks = trajectory
                .checks()
                .iter()
                .filter(|check| check.source_id == source && check.time == Some(time))
                .collect::<Vec<_>>();
            assert_eq!(
                checks
                    .iter()
                    .map(|check| check.target_id)
                    .collect::<BTreeSet<_>>()
                    .len(),
                2,
                "original source {source:?} must check both component targets at physical time {time}"
            );
            assert!(checks.iter().all(|check| check.satisfied));
        }
    }
    let reads = component_reads(trajectory, initial_tracer);
    assert!(reads.iter().all(|read| read.stocks.len() == cells));
    let samples = trajectory.report().samples.iter().collect::<Vec<_>>();
    prefix_balances(trajectory, &samples, times, &reads).unwrap();
    if counterexamples {
        let mut wrong_component = reads.clone();
        wrong_component[1].incoming = reads[0].incoming;
        wrong_component[1].outgoing = reads[0].outgoing;
        assert!(
            prefix_balances(trajectory, &samples, times, &wrong_component).is_err(),
            "water boundary quadratures cannot certify ethanol inventory"
        );
        let missing = samples
            .iter()
            .copied()
            .filter(|sample| sample.time != 5.25)
            .collect::<Vec<_>>();
        assert!(
            prefix_balances(trajectory, &missing, times, &reads).is_err(),
            "missing the original5.25s point cannot be treated as full coverage"
        );
    }
    let closure = trajectory
        .reports()
        .iter()
        .filter(|row| row.label == "aqueous cascade global material closure")
        .collect::<Vec<_>>();
    // Sample checks above retain physical time. Reports are terminal whole-domain
    // observations, not a second time-indexed trajectory table.
    assert_eq!(
        closure.len(),
        2,
        "both component closures retain their terminal original report"
    );
    assert!(closure.iter().all(|row| row.value.abs() <= 0.001));
    let terminal_source: DeclarationId = SemanticId::parse_hex("66e03000000000000000000000000157")
        .unwrap()
        .into();
    let terminal = trajectory
        .checks()
        .iter()
        .filter(|check| check.source_id == terminal_source)
        .collect::<Vec<_>>();
    assert_eq!(
        terminal
            .iter()
            .map(|check| check.target_id)
            .collect::<BTreeSet<_>>()
            .len(),
        2
    );
    assert!(
        terminal
            .iter()
            .all(|check| check.time == times.last().copied() && check.satisfied)
    );
}

#[cfg(feature = "solver-diffsol")]
#[tokio::test]
async fn aqueous_integrated_single_cell_preserves_original_response_and_stock() {
    assert_integrated(
        "aqueous_single_cell_integrated",
        1,
        &[0., 1., 5.],
        "66e03000000000000000000000000242",
        "66e03000000000000000000000000243",
        0.,
        false,
    )
    .await;
}

#[cfg(feature = "solver-diffsol")]
#[tokio::test]
async fn aqueous_integrated_four_cells_use_one_nonzero_physical_clock() {
    assert_integrated(
        "aqueous_four_cell_integrated_nonzero_clock",
        4,
        &[5., 5.25, 6., 7., 10.],
        "66e03000000000000000000000000252",
        "66e03000000000000000000000000253",
        0.,
        true,
    )
    .await;
}

#[cfg(feature = "solver-diffsol")]
#[tokio::test]
async fn aqueous_integrated_zero_flow_preserves_original_nonzero_inventory() {
    assert_integrated(
        "aqueous_four_cell_integrated_zero_flow",
        4,
        &[0., 1., 5.],
        "66e03000000000000000000000000262",
        "66e03000000000000000000000000263",
        0.01,
        false,
    )
    .await;
}

#[cfg(all(feature = "canonical-tests", feature = "solver-diffsol"))]
#[tokio::test]
async fn aqueous_integrated_original_native_outcome_survives_protected_export_and_portable_read() {
    use base64::Engine as _;
    use buffa_descriptor::{
        DescriptorPool, DynamicMessage, ReflectMessage as _, Value as WireValue,
        ValueRef as WireValueRef,
    };
    use datafusion::arrow::{
        array::{FixedSizeBinaryArray, Float64Array, Int64Array},
        ipc::reader::StreamReader,
    };
    use pse_compiler::workspace::ModelingOutput;

    let (runtime, package, _spill, preparation) =
        pr_jacobian_tests::ordinary_delivery_package(true)
            .await
            .unwrap();
    let physical = package.physical_context().clone();
    let runtime = super::super::durable_tests::durable_runtime_on(runtime);
    let store = runtime.canonical_store().unwrap();
    store
        .create_project(
            "aqueous-lifecycle-create",
            "aqueous-lifecycle",
            "Original aqueous inventory lifecycle",
        )
        .await
        .unwrap();
    let project = store.project("aqueous-lifecycle").await.unwrap().unwrap();
    let source = Box::pin(runtime.project_fork(
        &project,
        "aqueous-lifecycle-main",
        "aqueous-lifecycle-problem",
        "aqueous-lifecycle-fork",
        &package,
    ))
    .await
    .unwrap();
    let original_revision = source.key.clone();
    let original = Box::pin(runtime.modeling_revision(source, physical, BTreeMap::new()))
        .await
        .unwrap();
    let services = runtime.execution_services();
    let case = original
        .declarations(services.preparation())
        .await
        .unwrap()
        .into_iter()
        .find(|row| row.name == "aqueous_four_cell_integrated_nonzero_clock")
        .unwrap()
        .declaration_id;
    let cancel = crate::CancelSource::new();
    let operation = Box::pin(original.prepare_operation(
        services.preparation(),
        &crate::workflow::OperationSource::of(&original),
        &preparation,
        crate::workflow::RuntimeOperationRequest::Study {
            operation: Box::new(crate::workflow::OperationRequest::Simulation(
                crate::workflow::SimulationOperation {
                    case,
                    settings: None,
                },
            )),
            output: crate::workflow::StudyOutputDemand::Full {},
            admitted_horizon: None,
        },
        None,
        &cancel,
    ))
    .await
    .unwrap();
    let handle = operation.start(&services, &cancel).await.unwrap();
    let run_key = handle.canonical_run_key().unwrap().to_owned();
    let attempt_key = handle.canonical_attempt_key().unwrap().to_owned();
    let run = handle.wait().await.unwrap();
    assert!(run.usable());
    let crate::workflow::RunReport::Simulation(trajectory) = run.report().unwrap() else {
        panic!("the authored integrate procedure must produce its original native simulation");
    };
    assert!(trajectory.accepted() && trajectory.checks_complete());
    assert_eq!(trajectory.prepared().profile().method, Method::Diffsol);
    let times = [5., 5.25, 6., 7., 10.];
    let samples = trajectory.report().samples.iter().collect::<Vec<_>>();
    prefix_balances(
        trajectory,
        &samples,
        &times,
        &component_reads(trajectory, 0.),
    )
    .unwrap();
    // Expectations come directly from the producing native vectors and their
    // admitted member/physical contracts, not the relation projection also used
    // by the canonical writer.
    let prepared = trajectory.prepared();
    let product = prepared.model().compiled();
    let mut expected = BTreeMap::new();
    for (sample, point) in trajectory.report().samples.iter().enumerate() {
        assert_eq!(point.time.to_bits(), times[sample].to_bits());
        assert_eq!(point.outputs.len(), prepared.contract().outputs.len());
        for (column, target) in prepared.contract().outputs.iter().enumerate() {
            let ModelingOutput::Member(member) = product
                .admitted
                .outputs
                .iter()
                .find(|output| output.row_id() == *target)
                .unwrap()
            else {
                panic!("native trajectory output is not an original member");
            };
            let quantity = product
                .admitted
                .case()
                .rows()
                .iter()
                .find(|row| row.id == *target)
                .unwrap()
                .quantity;
            let unit = original
                .physical_context()
                .quantities()
                .quantity_type(quantity)
                .unwrap()
                .canonical_unit;
            assert!(
                expected
                    .insert(
                        (*member, sample as i64),
                        (
                            trajectory.run_id(),
                            quantity.as_id(),
                            unit.as_id(),
                            point.time.to_bits(),
                            point.outputs[column].to_bits()
                        )
                    )
                    .is_none()
            );
        }
    }
    assert_eq!(
        expected.len(),
        times.len() * prepared.contract().outputs.len()
    );
    assert!(!expected.is_empty());
    let row_count = expected.len() as u64;
    drop((run, handle, operation, original, package));
    runtime.clear_program_cache();

    // This reader acquires the original sealed run/attempt after producer handles drop.
    // Neither the selection observation nor its portable copy grants execution.
    let token = pse_columnar::CancellationToken::new();
    let mut reader = runtime
        .results(
            &run_key,
            &attempt_key,
            "runtime.simulation_samples",
            0,
            row_count,
            token.clone(),
        )
        .await
        .unwrap();
    let selection = reader.pack_selection().await.unwrap();
    assert_eq!((selection.row_start, selection.row_end), (0, row_count));
    // The accepted native trajectory is readable even though this selected
    // relation currently retains no scientific-use grant. Copying cannot add it.
    assert_eq!(
        selection.usability,
        crate::workflow::PackScientificState::Unknown
    );
    let context = selection.context.as_ref().unwrap();
    assert_eq!(context.problem, "aqueous-lifecycle-problem");
    assert_eq!(context.revision, original_revision);
    assert_eq!(context.manifest, reader.selection().manifest().digest);
    let directory = tempfile::tempdir().unwrap();
    let destination = directory.path().join("aqueous-original.arrow");
    reader
        .export_ipc_bounded(&destination, 8 << 20, &token)
        .await
        .unwrap();
    drop(reader);
    assert!(
        !directory
            .path()
            .join("aqueous-original.arrow.incomplete")
            .exists()
    );
    let artifact = std::fs::File::open(&destination).unwrap();
    let readable_bytes = artifact.metadata().unwrap().len();
    crate::workflow::portable_payload::validate_scientific_ipc(
        artifact,
        readable_bytes,
        8 << 20,
        &selection,
    )
    .unwrap();

    // Arrow and dynamic Protobuf read the actual protected export independently of
    // the generated native result adapter and generated relation row decoder.
    let mut portable =
        StreamReader::try_new(std::fs::File::open(&destination).unwrap(), None).unwrap();
    let schema = portable.schema();
    assert!(!schema.metadata().contains_key("pse.canonical.attempt"));
    let encoded = &schema.metadata()[crate::workflow::result_selection::SELECTION_METADATA_KEY];
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .unwrap();
    let world = Arc::new(
        DescriptorPool::decode(crate::host_operations::engineering::generated::DESCRIPTORS)
            .unwrap(),
    );
    let message = world
        .message_index("pse.engineering.v1.RecordedResultSelection")
        .unwrap();
    let observed = DynamicMessage::decode(world, message, &bytes).unwrap();
    for (tag, value) in [
        (1, run_key.as_str()),
        (2, attempt_key.as_str()),
        (3, "runtime.simulation_samples"),
    ] {
        assert_eq!(
            observed.field_by_number(tag),
            Some(&WireValue::String(value.into()))
        );
    }
    // Distinguish canonical wire absence from its descriptor-issued proto3 scalar default.
    let row_start = observed.message_descriptor().field(5).unwrap();
    assert!(observed.field_by_number(5).is_none());
    assert!(!observed.has(row_start));
    assert!(matches!(observed.get(row_start), WireValueRef::U64(0)));
    assert_eq!(
        observed.field_by_number(6),
        Some(&WireValue::U64(row_count))
    );
    let Some(WireValue::Message(usability)) = observed.field_by_number(10) else {
        panic!("original passive usability absent");
    };
    assert!(
        matches!(usability.field_by_number(1), Some(WireValue::Message(_))),
        "the original Unknown oneof alternative must survive copying"
    );
    for alternate in 2..=5 {
        assert!(
            usability.field_by_number(alternate).is_none(),
            "portable delivery cannot introduce a scientific-use alternative"
        );
    }
    let Some(WireValue::Message(observed_context)) = observed.field_by_number(11) else {
        panic!("original native context absent");
    };
    for (tag, value) in [
        (1, context.problem.as_str()),
        (2, context.revision.as_str()),
        (3, context.manifest.as_str()),
        (4, context.interpretation.as_str()),
    ] {
        assert_eq!(
            observed_context.field_by_number(tag),
            Some(&WireValue::String(value.into()))
        );
    }
    crate::workflow::portable_payload::validate_scientific_lineage(schema.as_ref(), &selection)
        .unwrap();
    let mut foreign = selection.clone();
    foreign.context.as_mut().unwrap().revision = "foreign-original-revision".into();
    assert!(
        crate::workflow::portable_payload::validate_scientific_lineage(schema.as_ref(), &foreign)
            .is_err()
    );

    let mut previous = None;
    for batch in &mut portable {
        let batch = batch.unwrap();
        assert!(
            batch
                .columns()
                .iter()
                .all(|column| column.null_count() == 0)
        );
        let ids = |name| {
            batch
                .column_by_name(name)
                .unwrap()
                .as_any()
                .downcast_ref::<FixedSizeBinaryArray>()
                .unwrap()
        };
        let sample = batch
            .column_by_name("sample")
            .unwrap()
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap();
        let time = batch
            .column_by_name("time")
            .unwrap()
            .as_any()
            .downcast_ref::<Float64Array>()
            .unwrap();
        let value = batch
            .column_by_name("value")
            .unwrap()
            .as_any()
            .downcast_ref::<Float64Array>()
            .unwrap();
        for index in 0..batch.num_rows() {
            let symbol = SemanticId::from_bytes(ids("symbol_id").value(index).try_into().unwrap());
            let key = (symbol, sample.value(index));
            assert!(
                previous.is_none_or(|before| before < key),
                "canonical output/sample order is original"
            );
            previous = Some(key);
            let original = expected
                .remove(&key)
                .expect("portable member/sample must have an actual native producer");
            assert_eq!(ids("run_id").value(index), original.0.as_bytes());
            assert_eq!(ids("quantity_id").value(index), original.1.as_bytes());
            assert_eq!(ids("unit_id").value(index), original.2.as_bytes());
            assert_eq!(time.value(index).to_bits(), original.3);
            assert_eq!(value.value(index).to_bits(), original.4);
        }
    }
    assert!(
        expected.is_empty(),
        "every original native member/sample must survive portable delivery"
    );
    assert!(
        portable.is_finished(),
        "independent portable read must consume the complete export"
    );
}
