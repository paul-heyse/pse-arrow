// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact long trajectory/output reads and complete retained-evidence graph pages.
//! Source admission, original simulation and graph admission are untimed setup.
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
use std::io::Write;

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
) -> (Expected, Value) {
    assert_eq!(
        states, 128,
        "this declared bounded campaign has 128 original Time states"
    );
    assert_eq!(
        samples, 31,
        "this declared bounded campaign has 31 original output times"
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
    let sources = support::sources(&source);
    let physical = support::physical(runtime, owner, &sources).await;
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
    let result = prepared.start().unwrap().wait().await.unwrap();
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
    assert!(dense_set.batch_count >= states as u64 && dense_set.batch_count > 64);
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
    assert!(dense.pages >= states as u64);
    assert_eq!(selected.digest, expected.output);
    assert_eq!(selected.rows as usize, samples);
    assert!(selected.pages < dense.pages);
    let graph = graph(&analysis, &expected, Some(&evidence)).await;
    expected.graph = graph.digest;
    let setup = json!({"run":expected.run,"attempt":expected.attempt,"scientific_run":expected.scientific_run,
        "revision":revision,"manifest":manifest_rows[0].digest,"analysis":expected.graph_key,
        "method":expected.header.method,"dense_rows":dense.rows,"dense_batches":dense.pages,
        "selected_rows":selected.rows,"selected_batches":selected.pages,
        "response_rows":responses.len(),"analysis_nodes":expected.header.node_count,
        "analysis_edges":expected.header.edge_count,"graph_pages":graph.pages,
        "original_dense_digest":expected.dense.to_hex(),"original_output_digest":expected.output.to_hex(),
        "original_graph_digest":expected.graph.to_hex(),"engineering_relative_fraction":ENGINEERING_ACCURACY,
        "analytical_comparison":"global fraction times max(abs(reference), canonical-unit floor)","original_completion":completion,
        "original_assessments":assessments,"original_model_checks":retained_checks.len(),
        "authored_output_sample_checks":authored_grid.len(),"numerical_observations":numerical.json()});
    drop(reader);
    drop(analysis);
    drop(original);
    drop(responses);
    drop(result);
    drop(prepared);
    drop(package);
    (expected, setup)
}

pub(super) fn measure(
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
    let (expected, setup) = executor.block_on(prepare(&runtime, &owner, states, samples));
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
            let record = executor.block_on(async {
                let begin = Instant::now();
                let dense = trajectory(&runtime, &expected, false).await;
                assert_eq!(dense.digest, expected.dense);
                assert_eq!(dense.rows as usize, states * samples);
                assert!(dense.pages >= states as u64);
                let dense_seconds = begin.elapsed().as_secs_f64();
                let begin = Instant::now();
                let selected = trajectory(&runtime, &expected, true).await;
                assert_eq!(selected.digest, expected.output);
                assert_eq!(selected.rows as usize, samples);
                assert!(selected.pages < dense.pages);
                let output_seconds = begin.elapsed().as_secs_f64();
                let begin = Instant::now();
                let analysis = runtime.analysis(&expected.graph_key).await.unwrap();
                let graph = graph(&analysis, &expected, None).await;
                assert_eq!(graph.digest, expected.graph);
                let graph_seconds = begin.elapsed().as_secs_f64();
                (dense, selected, graph, dense_seconds, output_seconds, graph_seconds)
            });
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
    executor.block_on(async {
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
    });
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
