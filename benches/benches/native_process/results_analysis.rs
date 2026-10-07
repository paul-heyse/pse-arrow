// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact retained reads and complete source-to-result publication operations.
//! Both variants share the original scientific and exact transport checks.
use super::{k4_support as support, *};
use pse_backend_native::dynamics as native;
use pse_ids::{ContentHash, Frame, FramedHasher, SemanticId};
use pse_kernels::DerivativeOrder;
use pse_model::{SemanticFrame, generated::identities::RunId};
use pse_relations::{
    columnar::RelationRow,
    generated::runtime::{
        candidate_assessments, canonical_analyses, canonical_analysis_edges,
        canonical_analysis_nodes, canonical_result_manifests, canonical_runs, computation_runs,
        modeling_checks, response_sensitivities, simulation_samples,
    },
};
use pse_runtime::workflow::{AnalysisControls, AnalysisDirection, AnalysisHandle, Runtime};
use serde_json::{Value, json};
use std::{io::Write, iter::repeat_n};

const ENGINEERING_ACCURACY: f64 = pse_model::numerics::DEFAULT_ENGINEERING_ACCURACY;

fn hasher(kind: &str) -> FramedHasher {
    let mut hash = FramedHasher::new(Frame::CanonicalPayloadV1);
    hash.str("pse.benchmark.exact-result-analysis.v1").str(kind);
    hash
}

/// Compact original expectations; no trajectory, native report or Arrow batch
/// survives setup into the timed selection loop.
struct Expected {
    run: String,
    attempt: String,
    scientific_run: RunId,
    symbols: BTreeMap<SemanticId, usize>,
    time_quantity: SemanticId,
    time_unit: SemanticId,
    parameter: SemanticId,
    sources: BTreeSet<String>,
    selected: SemanticId,
    samples: usize,
    dense: ContentHash,
    dense_batches: u64,
    output: ContentHash,
    graph: ContentHash,
    graph_key: String,
    header: canonical_analyses::Row,
    reclamation_pages: u64,
}

#[derive(Debug, PartialEq, Eq)]
struct ReadObservation {
    digest: ContentHash,
    rows: u64,
    pages: u64,
}

fn check_sample(row: &simulation_samples::Row, expected: &Expected) {
    assert_eq!(row.run_id, expected.scientific_run);
    assert_eq!(row.quantity_id, expected.time_quantity);
    assert_eq!(row.unit_id, expected.time_unit);
    let state = expected.symbols[&row.symbol_id];
    let sample = usize::try_from(row.sample).unwrap();
    assert!(sample < expected.samples);
    let time = 2.0 * sample as f64 / (expected.samples - 1) as f64;
    assert_eq!(row.time.to_bits(), time.to_bits());
    assert!(row.value.is_finite());
    let reference = state as f64 + 2.0 * time;
    assert!(
        (row.value - reference).abs() <= ENGINEERING_ACCURACY * reference.abs().max(1.0),
        "original x{state}(t) = {state} + 2t is not satisfied at sample {sample}"
    );
}

async fn trajectory(runtime: &Runtime, expected: &Expected, one_output: bool) -> ReadObservation {
    let cancel = CancelSource::new();
    let mut reader = if one_output {
        runtime
            .output_results(
                &expected.run,
                &expected.attempt,
                "runtime.simulation_samples",
                expected.selected,
                "value",
                "0",
                0,
                u64::MAX,
                None,
                None,
                None,
                cancel.token(),
            )
            .await
            .unwrap()
    } else {
        runtime
            .results(
                &expected.run,
                &expected.attempt,
                "runtime.simulation_samples",
                0,
                u64::MAX,
                cancel.token(),
            )
            .await
            .unwrap()
    };
    let mut hash = hasher("trajectory");
    let mut rows = 0_u64;
    let mut pages = 0_u64;
    let mut previous = None;
    while let Some(batch) = reader.next_relation_batch().await.unwrap() {
        pages += 1;
        let view = simulation_samples::View::from_checked(&batch).unwrap();
        for index in 0..view.len() {
            let row = view.row(index).unwrap();
            check_sample(&row, expected);
            let key = (row.symbol_id, row.sample);
            assert!(previous.is_none_or(|previous| previous < key));
            previous = Some(key);
            if one_output {
                assert_eq!(row.symbol_id, expected.selected);
            }
            // Exact finite IEEE bits, all original typed identities and columns.
            row.frame(&mut hash);
            rows += 1;
        }
    }
    ReadObservation {
        digest: hash.finish_hash(),
        rows,
        pages,
    }
}

/// During setup this additionally checks original sensitivity receipt references
/// and endpoint semantics. Timed reads check the exact complete graph digest.
async fn graph(
    analysis: &AnalysisHandle,
    expected: &Expected,
    original_evidence: Option<&BTreeMap<String, (SemanticId, SemanticId)>>,
) -> ReadObservation {
    let headers = canonical_analyses::Row::rows(&analysis.header().await.unwrap()).unwrap();
    assert_eq!(headers.as_slice(), std::slice::from_ref(&expected.header));
    let header = &headers[0];
    assert!(header.active);
    assert_eq!(header.key, expected.graph_key);
    assert_eq!(
        header.method,
        "result-sensitivity-provenance-reachability:v1"
    );
    assert!(header.node_count > 64 && header.node_count <= 4096);
    assert!(header.edge_count > 64 && header.edge_count <= 8192);
    let mut hash = hasher("graph");
    header.frame(&mut hash);
    let mut semantics = BTreeMap::new();
    let mut after = None::<String>;
    let mut node_count = 0_u64;
    let mut pages = 0_u64;
    loop {
        let batch = analysis.nodes(after.as_deref()).await.unwrap();
        let view = canonical_analysis_nodes::View::from_checked(&batch).unwrap();
        assert!(view.len() <= 64);
        if view.is_empty() {
            break;
        }
        pages += 1;
        for index in 0..view.len() {
            let row = view.row(index).unwrap();
            assert_eq!(row.analysis, expected.graph_key);
            assert!(after.as_ref().is_none_or(|previous| previous < &row.key));
            if original_evidence.is_some() {
                assert!(
                    semantics
                        .insert(row.key.clone(), (row.semantic.clone(), row.kind.clone()))
                        .is_none()
                );
            }
            after = Some(row.key.clone());
            row.frame(&mut hash);
            node_count += 1;
        }
    }
    assert_eq!(node_count, header.node_count);
    let mut after = None::<String>;
    let mut edge_count = 0_u64;
    let mut response_count = 0_u64;
    let mut provenance_count = 0_u64;
    let mut seen = BTreeSet::new();
    let mut produced = BTreeSet::new();
    loop {
        let batch = analysis.edges(after.as_deref()).await.unwrap();
        let view = canonical_analysis_edges::View::from_checked(&batch).unwrap();
        assert!(view.len() <= 64);
        if view.is_empty() {
            break;
        }
        pages += 1;
        for index in 0..view.len() {
            let row = view.row(index).unwrap();
            assert_eq!(row.analysis, expected.graph_key);
            assert!(after.as_ref().is_none_or(|previous| previous < &row.key));
            if let Some(original) = original_evidence {
                let (source, source_kind) = &semantics[&row.source];
                let (target, target_kind) = &semantics[&row.target];
                match row.kind.as_str() {
                    "retained_response_sensitivity" => {
                        assert_eq!(source_kind, "parameter");
                        assert_eq!(target_kind, "output");
                        let evidence = row.evidence.as_ref().unwrap();
                        let (output, parameter) = original[evidence];
                        assert_eq!(parameter, expected.parameter);
                        assert_eq!(source, &parameter.to_string());
                        assert_eq!(target, &output.to_string());
                        assert!(seen.insert(evidence.clone()));
                        response_count += 1;
                    }
                    "produced_evidence" => {
                        assert_eq!(source_kind, "attempt");
                        assert_eq!(source, &format!("attempt:{}", expected.attempt));
                        assert_eq!(target_kind, "output");
                        let evidence = format!("{}:value", row.evidence.as_ref().unwrap());
                        let (output, _) = original[&evidence];
                        assert_eq!(target, &output.to_string());
                        assert!(produced.insert(evidence));
                    }
                    "input_provenance" => {
                        assert_eq!(source_kind, "source_revision");
                        assert!(
                            expected
                                .sources
                                .contains(source.strip_prefix("revision:").unwrap())
                        );
                        assert_eq!(target, &format!("attempt:{}", expected.attempt));
                        assert!(row.evidence.is_none());
                        provenance_count += 1;
                    }
                    other => panic!("unexpected original method edge {other}"),
                }
            }
            after = Some(row.key.clone());
            row.frame(&mut hash);
            edge_count += 1;
        }
    }
    assert_eq!(edge_count, header.edge_count);
    if let Some(original) = original_evidence {
        assert_eq!(response_count as usize, original.len());
        assert_eq!(produced.len(), original.len());
        assert_eq!(provenance_count as usize, expected.sources.len());
        assert_eq!(edge_count, 2 * response_count + provenance_count);
    }
    ReadObservation {
        digest: hash.finish_hash(),
        rows: node_count + edge_count,
        pages,
    }
}

async fn prepare(
    runtime: &Runtime,
    owner: &WorkflowRuntime,
    states: usize,
    samples: usize,
    publish_sources: bool,
    physical_padding_bytes: usize,
) -> (Expected, Value) {
    let mut phase_seconds = BTreeMap::new();
    let started = Instant::now();
    assert!(
        states > 73,
        "the declared trajectory must contain selected state x73 and more than 64 graph nodes"
    );
    assert!(
        samples >= 2,
        "the declared output grid must contain both endpoints"
    );
    assert!(states.checked_mul(samples).unwrap() <= 4096);
    assert!(2 * states * samples + 65 <= 8192);
    let mut source = String::from(
        "package k4 { def Root { domain t:Time from 0{s} to 2{s}; discretize grid on t using integrated(elements=1,order=1); param p:Scalar=2;",
    );
    for state in 0..states {
        let allowance = ENGINEERING_ACCURACY * (state as f64 + 4.0).max(1.0);
        source.push_str(&format!("var x{state}[i in t]:Time; eq rate{state}[i in t]:d(x{state}[i])/di==p; eq initial{state}:x{state}[0{{s}}]=={state}{{s}}; annotation start x{state}(0{{s}}); annotation check x{state}(abs(x{state}[i]-{state}{{s}}-p*i)<{allowance}{{s}});"));
    }
    source.push_str("} }");
    let mut sources = support::sources(&source);
    let physical_block_bytes = pse_operations::canonical_execution::RESULT_BATCH_BYTES;
    if physical_padding_bytes > 0 {
        assert!(publish_sources);
        assert!(
            physical_padding_bytes > physical_block_bytes,
            "source-growth padding must cross the actual source block bound"
        );
        // The physical fixture is YAML, including existing # comments. Append
        // one comment after its complete document; all declarations are intact.
        let document = sources.physical.get_mut("materials/physical.yaml").unwrap();
        document.extend_from_slice(b"\n#");
        document.extend(repeat_n(b'p', physical_padding_bytes - 3));
        document.push(b'\n');
    }
    let physical_hash =
        pse_runtime::authoring_driver::document::package_checksum(&sources.physical);
    let physical_bytes = sources.physical.values().map(Vec::len).sum::<usize>();
    let physical_documents = sources.physical.len();
    let physical_blocks = sources
        .physical
        .values()
        .map(|bytes| bytes.len().div_ceil(physical_block_bytes))
        .sum::<usize>();
    let physical_largest_document = sources.physical.values().map(Vec::len).max().unwrap();
    let physical = if publish_sources {
        let pse_runtime::workflow::Durability::Durable(operations) = runtime.durability() else {
            panic!("complete publication requires the canonical execution owner")
        };
        let receipt = operations.put_sources(&sources.physical).await.unwrap();
        mark(&mut phase_seconds, "physical_document_publication", started);
        let reopened = Instant::now();
        let exact_sources = operations.sources(&receipt).await.unwrap();
        mark(&mut phase_seconds, "physical_protected_reopen", reopened);
        let checked = Instant::now();
        assert_eq!(receipt.identity, physical_hash);
        assert_eq!(&**exact_sources, &sources.physical);
        assert_eq!(
            pse_runtime::authoring_driver::document::package_checksum(&**exact_sources),
            physical_hash
        );
        mark(&mut phase_seconds, "original_source_byte_checks", checked);
        let admitted = Instant::now();
        let documents = support::admitted_documents(owner, std::slice::from_ref(&**exact_sources));
        let physical = runtime
            .physical_from_documents(&documents, &owner.cancel)
            .await
            .unwrap();
        mark(&mut phase_seconds, "physical_document_admission", admitted);
        drop(documents);
        drop(exact_sources);
        physical
    } else {
        let physical = support::physical(runtime, owner, &sources).await;
        mark(&mut phase_seconds, "physical_document_admission", started);
        physical
    };
    let started = Instant::now();
    let time = physical
        .quantities()
        .quantity_types()
        .find(|q| q.name.as_deref() == Some("Time"))
        .unwrap();
    let time_quantity = time.id.as_id();
    let time_unit = time.canonical_unit.as_id();
    let policy = pse_model::numerics::NumericalPolicy::default();
    let package = runtime
        .modeling_from_documents(
            &support::admitted_documents(owner, &sources.modeling),
            physical,
        )
        .await
        .unwrap();
    let revision = package.canonical_revision().key.clone();
    let root = package
        .declarations()
        .await
        .unwrap()
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    mark(&mut phase_seconds, "modeling_source_publication", started);
    let started = Instant::now();
    let cancel = CancelSource::new();
    let profile = native::Profile {
        method: native::Method::Diffsol,
        end: 2.0,
        samples: (0..samples)
            .map(|sample| 2.0 * sample as f64 / (samples - 1) as f64)
            .collect(),
        rtol: ENGINEERING_ACCURACY,
        atol: vec![ENGINEERING_ACCURACY; states],
        numerics: policy,
        sensitivity: native::DynamicSensitivity::Forward,
        parameter_scales: vec![1.0],
        ..Default::default()
    };
    let prepared = package
        .prepare_simulation(
            root,
            pse_modeling::specialize::root_instance(root),
            Default::default(),
            Default::default(),
            Default::default(),
            compiler(),
            profile,
            DerivativeOrder::First,
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(prepared.contract().states.len(), states);
    assert_eq!(prepared.contract().outputs.len(), states);
    assert_eq!(prepared.contract().parameters.len(), 1);
    assert_eq!(
        prepared.numerics().policy.engineering_relative_fraction,
        ENGINEERING_ACCURACY
    );
    let model = &prepared.model().compiled().model;
    let mut symbols = BTreeMap::new();
    for state in 0..states {
        let symbol = model
            .symbols
            .values()
            .find(|symbol| symbol.lineage.path.ends_with(&format!(".x{state}")))
            .unwrap();
        assert!(symbols.insert(symbol.id, state).is_none());
    }
    let selected = *symbols.iter().find(|(_, state)| **state == 73).unwrap().0;
    let parameter = prepared.contract().parameters[0];
    let parameter_unit = prepared
        .model()
        .compiled()
        .admitted
        .case()
        .parameters()
        .iter()
        .find(|port| port.id == parameter)
        .unwrap()
        .unit
        .as_id();
    let experiment = prepared.model().solved().instance();
    mark(&mut phase_seconds, "simulation_preparation", started);
    let started = Instant::now();
    let result = prepared.start().unwrap().wait().await.unwrap();
    mark(&mut phase_seconds, "simulate_and_publish", started);
    let started = Instant::now();
    assert!(
        result.usable(),
        "original full trajectory completion must be usable: {:?}",
        result.report()
    );
    let RunReport::Simulation(report) = result.report().unwrap() else {
        panic!("wrong long-trajectory report");
    };
    assert_eq!(report.report().termination, native::Termination::Completed);
    assert!(report.report().error.is_none());
    assert_eq!(report.report().samples.len(), samples);
    assert!(report.checks_complete());
    assert!(report.validation_error().is_none());
    assert!(report.accepted());
    let endpoint = report.report().assess_endpoint(prepared.profile());
    assert!(endpoint.satisfied && endpoint.prefix_complete);
    let mut authored_grid = BTreeSet::new();
    for check in report.checks() {
        assert_eq!(check.run_id, result.run_id);
        assert!(check.satisfied);
        if check.kind == pse_model::generated::enums::ModelingCheckKind::Check {
            assert!(symbols.contains_key(&check.target_id));
            let sample = usize::try_from(check.sample_index).unwrap();
            assert!(sample < samples);
            let time = 2.0 * sample as f64 / (samples - 1) as f64;
            assert_eq!(check.time.unwrap().to_bits(), time.to_bits());
            assert!(authored_grid.insert((check.target_id, sample)));
        }
    }
    assert_eq!(authored_grid.len(), states * samples);
    for output in symbols.keys() {
        for sample in 0..samples {
            assert!(authored_grid.contains(&(*output, sample)));
        }
    }
    let retained_checks =
        modeling_checks::Row::rows(&result.table("runtime.modeling_checks").unwrap()).unwrap();
    assert_eq!(retained_checks, report.checks());
    let mut numerical = observations::Observations::default();
    numerical.trajectory(report.report());
    let run = result.canonical_run_key().unwrap().to_owned();
    let attempt = result.canonical_attempt_key().unwrap().to_owned();
    let input_sources = runtime
        .canonical_store()
        .analysis_run_sources(&run)
        .await
        .unwrap()
        .into_iter()
        .map(|source| source.key)
        .collect::<BTreeSet<_>>();
    assert!(input_sources.contains(&revision));
    assert!(input_sources.len() <= 65);
    let run_rows = canonical_runs::Row::rows(&runtime.run_record(&run).await.unwrap()).unwrap();
    assert_eq!(run_rows.len(), 1);
    assert_eq!(run_rows[0].revision, revision);
    assert_eq!(
        run_rows[0].terminal_attempt.as_deref(),
        Some(attempt.as_str())
    );
    let manifest_rows =
        canonical_result_manifests::Row::rows(&runtime.result_manifest(&attempt).await.unwrap())
            .unwrap();
    assert_eq!(manifest_rows.len(), 1);
    let completion =
        computation_runs::Row::rows(&result.table("runtime.computation_runs").unwrap()).unwrap();
    assert_eq!(completion.len(), 1);
    assert_eq!(completion[0].run_id, result.run_id);
    assert_eq!(
        completion[0].trajectory_termination,
        Some(native::Termination::Completed)
    );
    assert_eq!(completion[0].completed_samples, Some(samples as i64));
    assert_eq!(completion[0].completed_time, Some(2.0));
    assert!(completion[0].candidate_available && completion[0].error.is_none());
    assert_eq!(completion[0].feasible, Some(true));
    assert_eq!(
        completion[0].qualification,
        pse_model::generated::enums::NativeQualification::Feasible
    );
    assert!(completion[0].validation_error.is_none());
    let assessments =
        candidate_assessments::Row::rows(&result.table("runtime.candidate_assessments").unwrap())
            .unwrap();
    assert_eq!(assessments.len(), 1);
    assert!(assessments[0].permits_result);
    // Simulation completion carries endpoint/sample feasibility above. This
    // optional field is a scalar SolveReport quality projection; no scalar
    // solve report is supplied by ModelingSimulation::finish.
    assert_eq!(assessments[0].validated, None);
    assert!(assessments[0].refusals.is_empty());
    let sets =
        pse_operations::canonical_execution::decode_result_descriptors(&manifest_rows[0]).unwrap();
    let dense_set = sets
        .iter()
        .find(|set| set.name == simulation_samples::RELATION_ID.to_string())
        .unwrap();
    assert_eq!(dense_set.row_count, (states * samples) as u64);
    assert!(dense_set.batch_count > 0);
    mark(&mut phase_seconds, "original_completion_checks", started);
    let started = Instant::now();
    let controls = AnalysisControls {
        roots: vec![],
        direction: AnalysisDirection::Downstream,
    };
    let analysis = runtime
        .result_analysis(&run, &attempt, &controls, cancel.token())
        .await
        .unwrap();
    let header = canonical_analyses::Row::rows(&analysis.header().await.unwrap())
        .unwrap()
        .remove(0);
    assert_eq!(header.revision, revision);
    assert_eq!(
        header.interpretation,
        pse_operations::generated::surreal::INTERPRETATION
    );
    assert_eq!(
        header.configuration.as_slice(),
        serde_json::to_vec(&(1_u8, &controls, &run, &attempt)).unwrap()
    );
    assert_eq!(header.node_count as usize, states + 2 + input_sources.len());
    assert_eq!(
        header.edge_count as usize,
        2 * states * samples + input_sources.len()
    );
    assert!(header.node_count <= 4096 && header.edge_count <= 8192);
    mark(&mut phase_seconds, "analysis_activation", started);
    let started = Instant::now();
    let mut expected = Expected {
        run,
        attempt,
        scientific_run: result.run_id,
        symbols,
        time_quantity,
        time_unit,
        parameter,
        sources: input_sources,
        selected,
        samples,
        dense: ContentHash::from_bytes([0; 32]),
        dense_batches: dense_set.batch_count,
        output: ContentHash::from_bytes([0; 32]),
        graph: ContentHash::from_bytes([0; 32]),
        graph_key: analysis.key().into(),
        header,
        // A cleanup page removes one payload or up to 64 indexes. This
        // conservative original-descriptor bound also allows seed/cursor pages.
        reclamation_pages: sets
            .iter()
            .map(|set| set.batch_count + 2 * set.row_count + 1)
            .sum::<u64>()
            + states as u64
            + 4,
    };
    let mut original =
        simulation_samples::Row::rows(&result.table("runtime.simulation_samples").unwrap())
            .unwrap();
    original.sort_by_key(|row| (row.symbol_id, row.sample));
    assert_eq!(original.len(), states * samples);
    let mut dense = hasher("trajectory");
    let mut output = hasher("trajectory");
    for row in &original {
        check_sample(row, &expected);
        row.frame(&mut dense);
        if row.symbol_id == selected {
            row.frame(&mut output);
        }
    }
    expected.dense = dense.finish_hash();
    expected.output = output.finish_hash();
    let responses =
        response_sensitivities::Row::rows(&result.table("runtime.response_sensitivities").unwrap())
            .unwrap();
    assert_eq!(responses.len(), states * samples);
    let mut response_grid = BTreeSet::new();
    for row in &responses {
        assert_eq!(row.run_id, result.run_id);
        assert_eq!(row.experiment_id, experiment);
        assert_eq!(row.parameter_id, parameter);
        assert_eq!(row.parameter_unit_id, parameter_unit);
        assert!(expected.symbols.contains_key(&row.output_id));
        assert_eq!(row.output_unit_id, time_unit);
        assert!(row.value.is_finite());
        let sample = usize::try_from(row.sample).unwrap();
        assert!(sample < samples);
        assert!(response_grid.insert((row.output_id, sample)));
        let reference = 2.0 * sample as f64 / (samples - 1) as f64;
        assert_eq!(row.time.unwrap().to_bits(), reference.to_bits());
        assert!((row.value - reference).abs() <= ENGINEERING_ACCURACY * reference.abs().max(1.0));
    }
    for output in expected.symbols.keys() {
        for sample in 0..samples {
            assert!(response_grid.contains(&(*output, sample)));
        }
    }
    // Evidence refers to actual immutable original IPC row coordinates, not to
    // a derivative copied into graph metadata. Every receipt retains the
    // original output/parameter endpoints; this setup-only map is dropped.
    let mut reader = runtime
        .results(
            &expected.run,
            &expected.attempt,
            "runtime.response_sensitivities",
            0,
            u64::MAX,
            cancel.token(),
        )
        .await
        .unwrap();
    let mut evidence = BTreeMap::new();
    let mut retained_responses = hasher("response");
    let mut original_responses = hasher("response");
    let response_set = reader
        .selection()
        .sets()
        .iter()
        .find(|set| set.name == response_sensitivities::RELATION_ID.to_string())
        .unwrap()
        .clone();
    let mut ordinal = 0_u64;
    let mut start = 0_u64;
    for row in &responses {
        row.frame(&mut original_responses);
    }
    while let Some(batch) = reader.next_relation_batch().await.unwrap() {
        // An unrestricted relation read yields each admitted self-contained
        // batch once, in frozen ordinal order and contiguous stored coverage.
        // Derive its public immutable coordinate from the closed descriptor.
        assert!(ordinal < response_set.batch_count);
        let receipt = pse_operations::canonical_execution::result_batch_key(
            &expected.attempt,
            &response_set.key,
            ordinal,
        );
        let view = response_sensitivities::View::from_checked(&batch).unwrap();
        for index in 0..view.len() {
            let row = view.row(index).unwrap();
            let original = &responses[usize::try_from(start).unwrap() + index];
            assert_eq!(&row, original);
            row.frame(&mut retained_responses);
            assert!(
                evidence
                    .insert(
                        format!(
                            "ipc:{}:{receipt}:{}:value",
                            response_sensitivities::RELATION_ID,
                            start + index as u64
                        ),
                        (original.output_id, original.parameter_id)
                    )
                    .is_none()
            );
        }
        start = start.checked_add(view.len() as u64).unwrap();
        ordinal = ordinal.checked_add(1).unwrap();
    }
    assert_eq!(ordinal, response_set.batch_count);
    assert_eq!(start, response_set.row_count);
    assert_eq!(
        retained_responses.finish_hash(),
        original_responses.finish_hash()
    );
    assert_eq!(evidence.len(), states * samples);
    let dense = trajectory(runtime, &expected, false).await;
    let selected = trajectory(runtime, &expected, true).await;
    assert_eq!(dense.digest, expected.dense);
    assert_eq!(dense.rows as usize, states * samples);
    assert_eq!(dense.pages, expected.dense_batches);
    assert!(dense.pages > 0);
    assert_eq!(selected.digest, expected.output);
    assert_eq!(selected.rows as usize, samples);
    assert!(selected.pages > 0);
    let graph = graph(&analysis, &expected, Some(&evidence)).await;
    expected.graph = graph.digest;
    mark(
        &mut phase_seconds,
        "original_transport_oracle_checks",
        started,
    );
    let setup = json!({"phase_seconds":phase_seconds,"physical_source_bytes":physical_bytes,
        "physical_source_documents":physical_documents,"physical_source_blocks":physical_blocks,
        "physical_source_block_bytes":physical_block_bytes,"physical_source_largest_document_bytes":physical_largest_document,
        "physical_padding_bytes":physical_padding_bytes,"physical_source_hash":physical_hash.to_hex(),
        "physical_source_growth_scope":"declared comment-byte padding of the same physical declarations; no scientific-model growth",
        "physical_source_ingress":if publish_sources { "canonical stage, one package activation, protected exact reopen and physical admission" } else { "owned document admission" },"run":expected.run,"attempt":expected.attempt,"scientific_run":expected.scientific_run,
        "revision":revision,"manifest":manifest_rows[0].digest,"analysis":expected.graph_key,
        "method":expected.header.method,"dense_rows":dense.rows,"dense_batches":dense.pages,
        "selected_rows":selected.rows,"selected_batches":selected.pages,
        "response_rows":responses.len(),"analysis_nodes":expected.header.node_count,
        "analysis_edges":expected.header.edge_count,"graph_pages":graph.pages,
        "original_dense_digest":expected.dense.to_hex(),"original_output_digest":expected.output.to_hex(),
        "original_graph_digest":expected.graph.to_hex(),"engineering_relative_fraction":ENGINEERING_ACCURACY,
        "analytical_comparison":"global fraction times max(abs(reference), canonical-unit floor)","original_completion":completion,
        "original_assessments":assessments,"original_model_checks":retained_checks.len(),
        "authored_output_sample_checks":authored_grid.len(),"trajectory_states":states,"trajectory_samples":samples,
        "trajectory_growth_scope":"joint coordinate and sample growth; no isolated per-factor speed claim",
        "numerical_observations":numerical.json()});
    drop(reader);
    drop(analysis);
    drop(original);
    drop(responses);
    drop(result);
    drop(prepared);
    drop(package);
    (expected, setup)
}

fn measure_retained(
    c: &mut Criterion,
    spec: &Value,
    output: &std::path::Path,
    phases: &phases::Phases,
) {
    let name = spec["id"].as_str().unwrap();
    assert_eq!(spec["operation"], "result_analysis");
    assert_eq!(spec["result_reads"], 3);
    let states = usize::try_from(spec["blocks"].as_u64().unwrap()).unwrap();
    let samples = usize::try_from(spec["samples"].as_u64().unwrap()).unwrap();
    let threads = usize::try_from(spec["threads"].as_u64().unwrap()).unwrap();
    let executor = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(threads)
        .enable_all()
        .build()
        .unwrap();
    let owner = WorkflowRuntime::with_threads(NonZeroUsize::new(threads).unwrap()).unwrap();
    let runtime = runtime(&owner);
    let setup_start = Instant::now();
    let (expected, setup) = executor.block_on(prepare(&runtime, &owner, states, samples, false, 0));
    let setup_seconds = setup_start.elapsed().as_secs_f64();
    owner.runtime.reset_observation_peak();
    phases.reset();
    let records_name = format!("{name}-observations.jsonl");
    let mut records =
        std::io::BufWriter::new(std::fs::File::create(output.join(&records_name)).unwrap());
    let mut iterations = 0_u64;
    let mut inclusive_phases = BTreeMap::new();
    let mut group = c.benchmark_group("process");
    group
        .sample_size(10)
        .sampling_mode(criterion::SamplingMode::Flat)
        .warm_up_time(Duration::from_millis(250))
        .measurement_time(Duration::from_secs(1));
    group.bench_function(name, |b| b.iter_custom(|count| {
        let mut elapsed = Duration::ZERO;
        for _ in 0..count {
            let started = Instant::now();
            let record = executor.block_on(read_all(&runtime, &expected, states, samples));
            let duration = started.elapsed();
            elapsed += duration;
            let (dense, selected, graph, dense_seconds, output_seconds, graph_seconds) = record;
            for (name, seconds) in [("exact_trajectory", dense_seconds), ("selected_output", output_seconds), ("analysis_pages", graph_seconds)] {
                *inclusive_phases.entry(name).or_insert(0.0) += seconds;
            }
            // Observation streaming and process/pool inspection are untimed.
            serde_json::to_writer(&mut records, &json!({"seconds":duration.as_secs_f64(),
                "dense_rows":dense.rows,"dense_batches":dense.pages,"selected_rows":selected.rows,
                "selected_batches":selected.pages,"graph_rows":graph.rows,"graph_pages":graph.pages})).unwrap();
            records.write_all(b"\n").unwrap();
            iterations += 1;
        }
        elapsed
    }));
    group.finish();
    records.flush().unwrap();
    assert!(iterations > 0);
    let peak = owner.runtime.observation_peak_bytes();
    let rss = owner.runtime.report().unwrap().process_peak_rss_bytes;
    let pool = owner.runtime.pool();
    let retained = pool.reserved();
    // Every analysis handle and reader is scoped to setup or one completed
    // iteration. Withdraw its retention only after those owners have dropped.
    executor.block_on(retire(&runtime, &expected));
    let after_case_teardown = pool.reserved();
    drop(expected);
    drop(runtime);
    executor.block_on(owner.cleanup_fixtures()).unwrap();
    drop(owner);
    executor.block_on(tokio::task::yield_now());
    let after = pool.reserved();
    assert_eq!(
        after, 0,
        "result-analysis workload escaped runtime ownership"
    );
    for value in inclusive_phases.values_mut() {
        *value /= iterations as f64;
    }
    let summary = json!({"id":name,"workload":spec,"iterations":iterations,
        "variables_observed":[states],"threads":threads,"native_threads":1,
        "pool_peak_bytes":peak,"process_peak_rss_bytes":rss,
        "retained_runtime_bytes":retained,"after_case_teardown_bytes":after_case_teardown,
        "after_retained_runtime_teardown_bytes":after,"phase_seconds":inclusive_phases,
        "compiler_phases":phases.report(iterations),"numerical_observations":setup["numerical_observations"],
        "setup_seconds":setup_seconds,"setup":setup,"observations_file":records_name,
        "scope":"exact retained trajectory, indexed output and complete bounded result-evidence graph reads",
        "timed_scope":"all three complete native selections, original row/digest assertions and graph-page completion",
        "untimed_scope":"source admission, preparation, one original simulation, analysis admission, oracle checks, observation output, explicit bounded result retirement and fixture teardown",
        "sampling":"10 flat Criterion samples, 250 ms warmup, 1 s target measurement time; slow complete operations extend sampling",
        "memory_scope":"accounted pool peak during selections; process lifetime RSS includes untimed source/solve/analysis setup"});
    std::fs::write(
        output.join(format!("{name}-memory.json")),
        serde_json::to_vec_pretty(&summary).unwrap(),
    )
    .unwrap();
}

async fn read_all(
    runtime: &Runtime,
    expected: &Expected,
    states: usize,
    samples: usize,
) -> (
    ReadObservation,
    ReadObservation,
    ReadObservation,
    f64,
    f64,
    f64,
) {
    let begin = Instant::now();
    let dense = trajectory(runtime, expected, false).await;
    assert_eq!(dense.digest, expected.dense);
    assert_eq!(dense.rows as usize, states * samples);
    assert_eq!(dense.pages, expected.dense_batches);
    assert!(dense.pages > 0);
    let dense_seconds = begin.elapsed().as_secs_f64();
    let begin = Instant::now();
    let selected = trajectory(runtime, expected, true).await;
    assert_eq!(selected.digest, expected.output);
    assert_eq!(selected.rows as usize, samples);
    assert!(selected.pages > 0);
    let output_seconds = begin.elapsed().as_secs_f64();
    let begin = Instant::now();
    let analysis = runtime.analysis(&expected.graph_key).await.unwrap();
    let graph = graph(&analysis, expected, None).await;
    assert_eq!(graph.digest, expected.graph);
    let graph_seconds = begin.elapsed().as_secs_f64();
    (
        dense,
        selected,
        graph,
        dense_seconds,
        output_seconds,
        graph_seconds,
    )
}

async fn retire(runtime: &Runtime, expected: &Expected) {
    runtime
        .forget_analysis_results(&expected.graph_key)
        .await
        .unwrap();
    let deadline = Instant::now() + pse_operations::canonical::REQUEST_TIMEOUT;
    loop {
        match runtime
            .canonical_store()
            .forget_run_results(&expected.run)
            .await
        {
            Ok(()) => break,
            Err(error)
                if error
                    .to_string()
                    .contains("run results have protected readers")
                    && Instant::now() < deadline =>
            {
                // Dropped readers release protections asynchronously. Wait
                // only for that named transient; all other failures surface.
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            Err(error) => panic!("explicit result retirement failed: {error}"),
        }
    }
    let mut complete = false;
    for _ in 0..expected.reclamation_pages {
        if runtime
            .canonical_store()
            .reclaim_result_page(&expected.run)
            .await
            .unwrap()
            .complete
        {
            complete = true;
            break;
        }
    }
    assert!(complete, "original descriptor reclamation bound exhausted");
}

pub(super) fn measure(
    c: &mut Criterion,
    spec: &Value,
    output: &std::path::Path,
    phases: &phases::Phases,
) {
    match spec["reuse"].as_str().unwrap() {
        "admitted-retained" => measure_retained(c, spec, output, phases),
        "publish-chain" => measure_complete(c, spec, output, phases),
        other => panic!("unsupported result-analysis operation scope {other}"),
    }
}

fn measure_complete(
    c: &mut Criterion,
    spec: &Value,
    output: &std::path::Path,
    phases: &phases::Phases,
) {
    let name = spec["id"].as_str().unwrap();
    assert_eq!(spec["operation"], "result_analysis");
    assert_eq!(spec["result_reads"], 3);
    let states = usize::try_from(spec["blocks"].as_u64().unwrap()).unwrap();
    let samples = usize::try_from(spec["samples"].as_u64().unwrap()).unwrap();
    let threads = usize::try_from(spec["threads"].as_u64().unwrap()).unwrap();
    let executor = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(threads)
        .enable_all()
        .build()
        .unwrap();
    let physical_padding_bytes = usize::try_from(
        spec.get("physical_padding_bytes")
            .map_or(0, |value| value.as_u64().unwrap()),
    )
    .unwrap();
    let records_name = format!("{name}-observations.jsonl");
    let mut records =
        std::io::BufWriter::new(std::fs::File::create(output.join(&records_name)).unwrap());
    let mut iterations = 0_u64;
    let mut inclusive_phases = BTreeMap::<String, f64>::new();
    let mut peak = 0;
    let mut rss = 0;
    let mut retained = 0;
    let mut after_case = 0;
    let mut last_setup = Value::Null;
    phases.reset();
    let mut group = c.benchmark_group("process");
    group
        .sample_size(10)
        .sampling_mode(criterion::SamplingMode::Flat)
        .warm_up_time(Duration::from_millis(250))
        .measurement_time(Duration::from_secs(1));
    group.bench_function(name, |b| {
        b.iter_custom(|count| {
            let mut elapsed = Duration::ZERO;
            for _ in 0..count {
                // The canonical fixture is fresh per operation. Its creation and
                // destruction are outside the source-to-results operation timer.
                let owner =
                    WorkflowRuntime::with_threads(NonZeroUsize::new(threads).unwrap()).unwrap();
                let runtime = runtime(&owner);
                owner.runtime.reset_observation_peak();
                let started = Instant::now();
                let (expected, setup) = executor.block_on(prepare(
                    &runtime,
                    &owner,
                    states,
                    samples,
                    true,
                    physical_padding_bytes,
                ));
                let (dense, selected, graph, dense_seconds, output_seconds, graph_seconds) =
                    executor.block_on(read_all(&runtime, &expected, states, samples));
                let duration = started.elapsed();
                elapsed += duration;
                for (phase, value) in setup["phase_seconds"].as_object().unwrap() {
                    *inclusive_phases.entry(phase.clone()).or_default() += value.as_f64().unwrap();
                }
                for (phase, seconds) in [
                    ("exact_trajectory", dense_seconds),
                    ("selected_output", output_seconds),
                    ("analysis_pages", graph_seconds),
                ] {
                    *inclusive_phases.entry(phase.into()).or_default() += seconds;
                }
                peak = peak.max(owner.runtime.observation_peak_bytes());
                rss = rss.max(
                    owner
                        .runtime
                        .report()
                        .unwrap()
                        .process_peak_rss_bytes
                        .unwrap(),
                );
                let pool = owner.runtime.pool();
                retained = retained.max(pool.reserved());
                // No original reports/readers survive into reclamation. Retirement,
                // observation output and fixture cleanup are explicitly untimed.
                executor.block_on(retire(&runtime, &expected));
                after_case = after_case.max(pool.reserved());
                drop(expected);
                drop(runtime);
                executor.block_on(owner.cleanup_fixtures()).unwrap();
                drop(owner);
                executor.block_on(tokio::task::yield_now());
                assert_eq!(
                    pool.reserved(),
                    0,
                    "complete result operation escaped runtime ownership"
                );
                serde_json::to_writer(
                    &mut records,
                    &json!({"seconds":duration.as_secs_f64(),
                "dense_rows":dense.rows,"dense_batches":dense.pages,"selected_rows":selected.rows,
                "selected_batches":selected.pages,"graph_rows":graph.rows,"graph_pages":graph.pages,
                "source_to_analysis":setup}),
                )
                .unwrap();
                records.write_all(b"\n").unwrap();
                last_setup = setup;
                iterations += 1;
            }
            elapsed
        })
    });
    group.finish();
    records.flush().unwrap();
    assert!(iterations > 0);
    for seconds in inclusive_phases.values_mut() {
        *seconds /= iterations as f64;
    }
    let summary = json!({"id":name,"workload":spec,"iterations":iterations,
        "variables_observed":[states],"threads":threads,"native_threads":1,
        "pool_peak_bytes":peak,"process_peak_rss_bytes":rss,"retained_runtime_bytes":retained,
        "after_case_teardown_bytes":after_case,"after_retained_runtime_teardown_bytes":0,
        "phase_seconds":inclusive_phases,"compiler_phases":phases.report(iterations),
        "numerical_observations":last_setup["numerical_observations"],"setup":last_setup,
        "observations_file":records_name,
        "scope":"complete canonical source-to-result publication and analysis operation with exact retained reads",
        "timed_scope":"fixture source loading, physical document stage/activation/protected reopen, modeling publication, simulation preparation, simulation and result publication, analysis activation, original scientific/transport oracle checks and three complete selections",
        "untimed_scope":"runtime and canonical fixture creation, observation output, explicit bounded retirement, runtime and fixture teardown",
        "phase_boundary":"simulation and publication share the public completion interface and are reported together",
        "sampling":"10 flat Criterion samples, 250 ms warmup, 1 s target measurement time; slow complete operations extend sampling",
        "memory_scope":"accounted pool peak across each complete operation; process lifetime RSS also includes fixture creation and teardown"});
    std::fs::write(
        output.join(format!("{name}-memory.json")),
        serde_json::to_vec_pretty(&summary).unwrap(),
    )
    .unwrap();
}
