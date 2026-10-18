// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Independent controls for admitted family, exact scope and withheld claims.
use super::*;
use pse_model::generated::{enums::*, identities::*, runtime::*, structures::*};

fn id(value: u8) -> pse_ids::SemanticId {
    pse_ids::SemanticId::from_bytes([value; 16])
}
fn hash(value: u8) -> pse_ids::ContentHash {
    pse_ids::ContentHash::from_bytes([value; 32])
}
fn run() -> RunId {
    RunId::from_id(id(1))
}
fn located<R>(row: R, block: u64, index: u64) -> Located<R> {
    Located {
        row,
        locator: ArtifactLocator {
            artifact: "independent-evidence".into(),
            block,
            row: Some(index),
            field: None,
        },
    }
}
fn coordinate(value: u8) -> ScientificArtifactCoordinate {
    ScientificArtifactCoordinate {
        id: id(value),
        quantity_id: id(8),
        unit_id: id(9),
        scale: 10.,
        offset: 2.,
    }
}
fn context(step: i64) -> scientific_contexts::Row {
    scientific_contexts::Row {
        run_id: run(),
        step,
        model_id: Some(DeclarationId::from_id(id(2))),
        fit_id: None,
        case_id: None,
        instance_id: Some(InstanceId::from_id(id(3))),
        source_identity: hash(1),
        physical_identity: hash(2),
        preparation_identity: hash(3),
        profile_identity: hash(4),
        configuration: vec![1].into(),
        binding: None,
        state: vec![coordinate(4)],
        inputs: vec![coordinate(5)],
        outputs: vec![coordinate(4)],
        integrals: vec![],
        time: None,
        mode_names: vec![],
        mode_identities: vec![],
    }
}
fn route(step: i64, intent: NativeSolveIntent) -> route_decisions::Row {
    route_decisions::Row {
        request_identity: hash(10 + step as u8),
        step,
        intent,
        selection: NativeRouteSelection::Auto,
        requested_backend: None,
        classes: vec![],
        state: NativeAssessmentState::Ready,
        snapshot: hash(2),
        pending_backend: None,
        evidence: vec![],
        artifacts: vec![],
        evidence_classes: vec![],
        required_order: Some(1),
        artifact_representations: vec![],
        eligibility: vec![],
        selected: None,
        backend: None,
        representation: None,
        lexicographic: None,
        refusal: None,
        detail: None,
    }
}
fn validity(step: i64, family: ResponseFamily) -> local_validity::Row {
    let root = family == ResponseFamily::Root;
    local_validity::Row {
        run_id: run(),
        step,
        quantity: DerivedQuantity::ParametricSensitivity,
        validity: LocalValidity {
            certified: true,
            reason: None,
            detail: None,
            conditional: false,
            licq: (!root).then_some(true),
            strict_complementarity: (!root).then_some(true),
            second_order: (!root).then_some(true),
            weakly_active: (!root).then_some(0),
            condition_1norm: Some(1.),
            residual: Some(0.),
            root_rank: root.then_some(1),
            root_rank_cutoff: root.then_some(1e-10),
            root_rank_relative_cutoff: root.then_some(1e-10),
            root_backward_error: root.then_some(0.),
            root_backward_error_limit: root.then_some(1e-10),
            root_neighborhood: root.then_some("same branch, inactive bounds".into()),
        },
    }
}
fn parametric(step: i64) -> parametric_sensitivities::Row {
    parametric_sensitivities::Row {
        run_id: run(),
        step,
        parameter_id: id(5),
        target_kind: NumericalTarget::Variable,
        target_id: id(4),
        parameter_unit_id: id(9),
        target_unit_id: id(9),
        primal: Some(3.),
        dual: None,
    }
}
fn scalar() -> ResponseEvidence {
    ResponseEvidence {
        contexts: vec![located(context(0), 0, 0)],
        routes: vec![located(route(0, NativeSolveIntent::Root), 1, 0)],
        validity: vec![located(validity(0, ResponseFamily::Root), 2, 0)],
        parametric: vec![located(parametric(0), 3, 0)],
        ..Default::default()
    }
}
fn pool() -> Arc<dyn pse_columnar::MemoryPool> {
    Arc::new(pse_columnar::GreedyMemoryPool::new(128 * 1024 * 1024))
}
fn graph(evidence: &ResponseEvidence) -> Graph {
    let mut graph = Graph::new();
    let mut nodes = BTreeMap::new();
    let attempt = vertex(&mut graph, &mut nodes, "attempt:exact".into(), "attempt").unwrap();
    project_response_evidence(evidence, &mut graph, &mut nodes, attempt, &pool()).unwrap();
    graph
}
fn count(graph: &Graph, kind: &str) -> usize {
    graph
        .edge_weights()
        .filter(|(name, _)| name == kind)
        .count()
}

#[test]
fn response_family_requires_admitted_intent_not_validity_or_finite_primal() {
    let mut evidence = scalar();
    assert_eq!(
        count(&graph(&evidence), "retained_root_parametric_primal"),
        1
    );
    evidence.routes.clear();
    let withheld = graph(&evidence);
    assert_eq!(count(&withheld, "retained_root_parametric_primal"), 0);
    assert_eq!(
        count(&withheld, "withheld_unclassified_parametric_primal"),
        1
    );
    assert_eq!(count(&withheld, "produced_evidence"), 1);
    for intent in [
        NativeSolveIntent::FeasiblePoint,
        NativeSolveIntent::Initialize,
        NativeSolveIntent::Certify,
    ] {
        evidence.routes = vec![located(route(0, intent), 1, 0)];
        assert_eq!(ResponseEvidenceIndex::new(&evidence, &pool()).unwrap().scalar_family(run(), 0), None);
    }
    // Root-looking evidence never overrides an optimizing admission.
    evidence.routes = vec![located(route(0, NativeSolveIntent::Optimize), 1, 0)];
    assert_eq!(
        ResponseEvidenceIndex::new(&evidence, &pool()).unwrap().scalar_family(run(), 0),
        Some(ResponseFamily::Optimal)
    );
    assert_eq!(
        count(&graph(&evidence), "withheld_optimal_parametric_primal"),
        1
    );
}

#[test]
fn response_validity_preserves_root_neighborhood_and_actual_optimal_conditions() {
    let mut evidence = scalar();
    evidence.validity[0].row.validity.root_neighborhood = None;
    assert_eq!(
        count(&graph(&evidence), "withheld_root_parametric_primal"),
        1
    );
    evidence.validity[0].row = validity(0, ResponseFamily::Root);
    // A root has no promised KKT multiplier response even if malformed input
    // happens to contain a finite dual value.
    evidence.parametric[0].row.dual = Some(1.);
    assert_eq!(count(&graph(&evidence), "withheld_root_parametric_dual"), 1);
    evidence.routes[0].row = route(0, NativeSolveIntent::Optimize);
    evidence.validity[0].row = validity(0, ResponseFamily::Optimal);
    let optimal = graph(&evidence);
    assert_eq!(count(&optimal, "retained_optimal_parametric_primal"), 1);
    assert_eq!(count(&optimal, "retained_optimal_parametric_dual"), 1);
    evidence.validity[0].row.validity.second_order = Some(false);
    assert_eq!(
        count(&graph(&evidence), "retained_optimal_parametric_primal"),
        0
    );
    evidence.validity[0].row.validity.certified = false;
    evidence.validity[0].row.validity.reason = Some(WithheldReason::NoLocalAnalysis);
    assert_eq!(count(&graph(&evidence), "local_validity"), 1); // original withholding survives
    assert!(
        graph(&evidence)
            .edge_weights()
            .any(|(kind, locator)| kind == "response_validity"
                && locator.as_ref()
                    == Some(&field_locator(&evidence.validity[0].locator, "validity")))
    );
}

#[test]
fn response_conflicts_withhold_only_the_affected_step_and_quantity() {
    let mut evidence = scalar();
    evidence.contexts.push(located(context(1), 0, 1));
    evidence
        .routes
        .push(located(route(1, NativeSolveIntent::Optimize), 1, 1));
    evidence
        .validity
        .push(located(validity(1, ResponseFamily::Optimal), 2, 1));
    evidence.parametric.push(located(parametric(1), 3, 1));
    let complete = graph(&evidence);
    assert_eq!(count(&complete, "retained_root_parametric_primal"), 1);
    assert_eq!(count(&complete, "retained_optimal_parametric_primal"), 1);
    let mut conflict = route(1, NativeSolveIntent::Optimize);
    conflict.request_identity = hash(90);
    evidence.routes.push(located(conflict, 4, 0));
    let conflict = graph(&evidence);
    assert_eq!(count(&conflict, "retained_root_parametric_primal"), 1);
    assert_eq!(count(&conflict, "retained_optimal_parametric_primal"), 0);
    assert_eq!(
        count(&conflict, "withheld_unclassified_parametric_primal"),
        1
    );
    evidence.routes.pop();
    evidence.validity[1].row.quantity = DerivedQuantity::ReducedHessian;
    assert_eq!(
        count(&graph(&evidence), "withheld_optimal_parametric_primal"),
        1
    );
    evidence.validity[1].row = validity(1, ResponseFamily::Optimal);
    evidence.validity[1].row.run_id = RunId::from_id(id(90));
    assert_eq!(
        count(&graph(&evidence), "withheld_optimal_parametric_primal"),
        1
    );
}

#[test]
fn response_graph_preserves_scopes_units_scaling_and_exact_evidence_fields() {
    let mut evidence = scalar();
    evidence.contexts.push(located(context(1), 0, 1));
    evidence
        .routes
        .push(located(route(1, NativeSolveIntent::Root), 1, 1));
    evidence
        .validity
        .push(located(validity(1, ResponseFamily::Root), 2, 1));
    evidence.parametric.push(located(parametric(1), 3, 1));
    evidence.parametric.push(located(parametric(1), 4, 0));
    let graph = graph(&evidence);
    let locators = graph
        .edge_weights()
        .filter(|(kind, _)| kind == "retained_root_parametric_primal")
        .map(|(_, locator)| locator.as_ref().unwrap().clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(locators.len(), 3);
    assert!(
        locators
            .iter()
            .all(|locator| locator.artifact == "independent-evidence"
                && locator.field.as_deref() == Some("primal"))
    );
    assert!(
        locators
            .iter()
            .any(|locator| locator.block == 3 && locator.row == Some(1))
    );
    assert!(
        locators
            .iter()
            .any(|locator| locator.block == 4 && locator.row == Some(0))
    );
    assert_eq!(
        graph
            .node_weights()
            .filter(|(_, kind)| kind == "scoped_response")
            .count(),
        2
    );
    assert!(
        graph
            .edge_weights()
            .any(|(kind, locator)| kind == "response_coordinates"
                && locator.as_ref() == Some(&evidence.contexts[1].locator))
    );
    assert_eq!(evidence.contexts[1].row.inputs[0].scale, 10.);
    assert_eq!(evidence.contexts[1].row.inputs[0].offset, 2.);
    assert_eq!(
        evidence.parametric[1].row.parameter_unit_id,
        evidence.contexts[1].row.inputs[0].unit_id
    );
}

fn transient() -> ResponseEvidence {
    let mut context = context(0);
    context.time = Some(coordinate(6));
    context.mode_names = vec!["initial".into()];
    context.mode_identities = vec![hash(5)];
    let header = computation_runs::Row {
        run_id: run(),
        kind: ComputationKind::Simulation,
        source_identity: hash(1),
        profile_identity: hash(4),
        state: NativeRunState::Native,
        termination: None,
        trajectory_termination: Some(TrajectoryTermination::Event),
        backend: None,
        native_code: None,
        native_status: None,
        qualification: NativeQualification::Unqualified,
        candidate_kind: None,
        candidate_available: true,
        feasible: None,
        completed_time: Some(1.),
        completed_samples: Some(2),
        estimate_qualified: None,
        response_available: Some(true),
        response_rank: None,
        response_condition: None,
        validation_error: None,
        error: None,
    };
    let endpoint = trajectory_endpoints::Row {
        run_id: run(),
        requirement: EndpointPolicy::FixedHorizon,
        required_event: None,
        event_id: Some(id(7)),
        time: 1.,
        mode: 0,
        mode_name: "initial".into(),
        mode_identity: hash(5),
        state_ids: vec![id(4)],
        input_ids: vec![id(5)],
        output_ids: vec![id(4)],
        state: vec![1.],
        outputs: vec![1.],
        integrals: vec![],
        input_columns: vec![0],
        inputs: vec![2.],
        endpoint_satisfied: false,
        prefix_complete: false,
        missing_observations: vec![2.],
    };
    let sample = simulation_samples::Row {
        run_id: run(),
        sample: 1,
        time: 0.5,
        symbol_id: id(4),
        quantity_id: id(8),
        unit_id: id(9),
        value: 1.,
    };
    let response = response_sensitivities::Row {
        run_id: run(),
        experiment_id: InstanceId::from_id(id(3)),
        sample: 1,
        time: Some(0.5),
        output_id: id(4),
        parameter_id: id(5),
        parameter_column: 0,
        output_unit_id: id(9),
        parameter_unit_id: id(9),
        value: 0.5,
    };
    ResponseEvidence {
        contexts: vec![located(context, 0, 0)],
        computations: vec![located(header, 1, 0)],
        endpoints: vec![located(endpoint, 2, 0)],
        samples: vec![located(sample, 3, 0)],
        responses: vec![located(response, 4, 0)],
        events: vec![located(
            simulation_events::Row {
                run_id: run(),
                ordinal: 0,
                event_id: Some(id(7)),
                time: 1.,
                symbol_id: id(4),
                before: 1.,
                after: None,
            },
            5,
            0,
        )],
        ..Default::default()
    }
}

#[test]
fn response_partial_event_trajectory_retains_recording_without_scalar_certification() {
    let mut evidence = transient();
    let projected = graph(&evidence);
    assert_eq!(count(&projected, "recorded_transient_response"), 1);
    assert_eq!(count(&projected, "response_event_context"), 1);
    assert_eq!(count(&projected, "response_endpoint"), 1);
    assert!(
        projected
            .edge_weights()
            .all(|(kind, _)| !kind.contains("parametric") && !kind.contains("certified"))
    );
    assert!(
        projected
            .edge_weights()
            .any(|(kind, locator)| kind == "recorded_transient_response"
                && locator.as_ref()
                    == Some(&field_locator(&evidence.responses[0].locator, "value")))
    );
    assert_eq!(
        evidence.responses[0].row.sample,
        evidence.samples[0].row.sample
    );
    assert_eq!(
        evidence.responses[0].row.time,
        Some(evidence.samples[0].row.time)
    );
    let mut absent_sample = evidence.responses[0].row.clone();
    absent_sample.sample = 2;
    evidence.responses.push(located(absent_sample, 4, 1));
    let partial = graph(&evidence);
    assert_eq!(count(&partial, "recorded_transient_response"), 1);
    assert_eq!(count(&partial, "withheld_response_claim"), 1);
    evidence.computations[0].row.response_available = Some(false);
    assert_eq!(count(&graph(&evidence), "recorded_transient_response"), 0);
}

#[test]
fn response_transient_family_never_comes_from_rows_or_primal_success() {
    let mut evidence = transient();
    evidence.computations.clear();
    assert_eq!(count(&graph(&evidence), "recorded_transient_response"), 0);
    evidence = transient();
    evidence.computations[0].row.kind = ComputationKind::Shooting;
    assert_eq!(count(&graph(&evidence), "recorded_transient_response"), 0);
    evidence = transient();
    evidence.responses[0].row.time = Some(1.5);
    assert_eq!(count(&graph(&evidence), "withheld_response_claim"), 1);
    evidence = transient();
    evidence.responses[0].row.experiment_id = InstanceId::from_id(id(99));
    assert_eq!(count(&graph(&evidence), "withheld_response_claim"), 1);
}

#[test]
fn response_artifact_dispatch_requires_actual_owned_scientific_context() {
    use super::super::artifacts::{ScientificArtifactKind, response_artifact_kind};
    let scalar = scalar();
    assert!(matches!(
        response_artifact_kind(scalar.contexts.iter().map(|c| &c.row), [].iter()),
        Ok(ScientificArtifactKind::LocalAnalysis)
    ));
    let transient = transient();
    assert!(matches!(
        response_artifact_kind(
            transient.contexts.iter().map(|c| &c.row),
            transient.computations.iter().map(|c| &c.row)
        ),
        Ok(ScientificArtifactKind::Trajectory)
    ));
    assert!(response_artifact_kind([].iter(), [].iter()).is_err());
    assert!(response_artifact_kind(transient.contexts.iter().map(|c| &c.row), [].iter()).is_err());
    assert!(
        response_artifact_kind(
            scalar.contexts.iter().map(|c| &c.row),
            transient.computations.iter().map(|c| &c.row)
        )
        .is_err()
    );
}

#[test]
fn response_selected_parameter_keeps_original_qualification_context() {
    let evidence = scalar();
    let graph = graph(&evidence);
    let nodes = graph
        .node_indices()
        .map(|index| (graph[index].0.clone(), index))
        .collect::<BTreeMap<_, _>>();
    let selection = selected(
        &graph,
        &nodes,
        &AnalysisControls {
            roots: vec![id(5).to_string()],
            direction: AnalysisDirection::Downstream,
        },
    )
    .unwrap();
    assert!(
        selection
            .iter()
            .any(|index| graph[*index].1 == "scoped_response")
    );
    assert_eq!(
        selection
            .iter()
            .filter(|index| graph[**index].1 == "retained_context")
            .count(),
        3
    );
    assert!(!selection.contains(&nodes["attempt:exact"]));
}

#[test]
fn response_actual_campaign_shape_preserves_complete_inventory_within_graph_caps() {
    const OUTPUTS: usize = 128;
    const SAMPLES: usize = 31;
    let mut evidence = transient();
    evidence.samples.clear();
    evidence.responses.clear();
    evidence.events.clear();
    evidence.contexts[0].row.outputs = (0..OUTPUTS)
        .map(|output| coordinate(20 + output as u8))
        .collect();
    evidence.computations[0].row.completed_samples = Some(SAMPLES as i64);
    evidence.computations[0].row.trajectory_termination = Some(TrajectoryTermination::Completed);
    evidence.endpoints[0].row.endpoint_satisfied = true;
    evidence.endpoints[0].row.prefix_complete = true;
    evidence.endpoints[0].row.missing_observations.clear();
    for sample in 0..SAMPLES {
        let time = sample as f64 / (SAMPLES - 1) as f64;
        evidence.modes.push(located(
            modeling_trajectory_modes::Row {
                run_id: run(),
                sample: sample as i64,
                time,
                mode: "initial".into(),
            },
            6,
            sample as u64,
        ));
        for output in 0..OUTPUTS {
            let symbol = id(20 + output as u8);
            let index = (sample * OUTPUTS + output) as u64;
            evidence.samples.push(located(
                simulation_samples::Row {
                    run_id: run(),
                    sample: sample as i64,
                    time,
                    symbol_id: symbol,
                    quantity_id: id(8),
                    unit_id: id(9),
                    value: time,
                },
                3,
                index,
            ));
            evidence.responses.push(located(
                response_sensitivities::Row {
                    run_id: run(),
                    experiment_id: InstanceId::from_id(id(3)),
                    sample: sample as i64,
                    time: Some(time),
                    output_id: symbol,
                    parameter_id: id(5),
                    parameter_column: 0,
                    output_unit_id: id(9),
                    parameter_unit_id: id(9),
                    value: time,
                },
                4,
                index,
            ));
        }
    }
    let graph = graph(&evidence);
    assert!(graph.node_count() <= ANALYSIS_NODES);
    assert!(graph.edge_count() <= ANALYSIS_EDGES);
    let quantitative = graph
        .edge_weights()
        .filter(|(kind, _)| kind == "recorded_transient_response")
        .map(|(_, locator)| locator.as_ref().unwrap().clone())
        .collect::<BTreeSet<_>>();
    let original = evidence
        .responses
        .iter()
        .map(|item| field_locator(&item.locator, "value"))
        .collect::<BTreeSet<_>>();
    assert_eq!(quantitative, original);
    assert_eq!(quantitative.len(), OUTPUTS * SAMPLES);
    assert_eq!(count(&graph, "withheld_response_claim"), 0);
    assert_eq!(count(&graph, "produced_evidence"), OUTPUTS);
    assert!(
        graph
            .edge_weights()
            .filter(|(kind, _)| kind == "produced_evidence")
            .all(|(_, locator)| {
                let locator = locator.as_ref().unwrap();
                locator.block == 4 && locator.row.is_none() && locator.field.is_none()
            })
    );
    for kind in [
        "response_coordinates",
        "response_completion",
        "response_endpoint",
    ] {
        assert_eq!(count(&graph, kind), OUTPUTS);
        assert!(
            graph
                .edge_weights()
                .filter(|(name, _)| name == kind)
                .all(|(_, locator)| locator.as_ref().unwrap().row.is_some())
        );
    }
    assert_eq!(count(&graph, "trajectory_mode"), SAMPLES);
}

#[test]
fn response_produced_provenance_preserves_distinct_blocks_and_quantitative_rows() {
    let mut evidence = transient();
    evidence
        .responses
        .push(located(evidence.responses[0].row.clone(), 7, 0));
    let graph = graph(&evidence);
    let produced = graph
        .edge_weights()
        .filter(|(kind, _)| kind == "produced_evidence")
        .map(|(_, locator)| locator.as_ref().unwrap().clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(produced.len(), 2);
    assert_eq!(
        produced
            .iter()
            .map(|locator| locator.block)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([4, 7])
    );
    assert!(
        produced
            .iter()
            .all(|locator| locator.row.is_none() && locator.field.is_none())
    );
    let quantitative = graph
        .edge_weights()
        .filter(|(kind, _)| kind == "recorded_transient_response")
        .map(|(_, locator)| locator.as_ref().unwrap().clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(quantitative.len(), 2);
    assert!(
        quantitative
            .iter()
            .all(|locator| locator.row == Some(0) && locator.field.as_deref() == Some("value"))
    );
}


#[test]
fn response_index_preserves_ieee_membership_and_all_transient_predicates() {
    let mut evidence = transient();
    evidence.samples[0].row.time = -0.0;
    evidence.responses[0].row.time = Some(0.0);
    let index = ResponseEvidenceIndex::new(&evidence, &pool()).unwrap();
    assert!(index.transient_recorded(&evidence.responses[0].row));
    let mut row = evidence.responses[0].row.clone();
    for time in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 0.5] {
        row.time = Some(time);
        assert!(!index.transient_recorded(&row));
    }
    row.time = None;
    assert!(!index.transient_recorded(&row));
    let original = &evidence.responses[0].row;
    let mut wrong = original.clone();
    wrong.sample = -1;
    assert!(!index.transient_recorded(&wrong));
    wrong.sample = i64::MAX;
    assert!(!index.transient_recorded(&wrong));
    wrong = original.clone();
    wrong.parameter_column = i64::MIN;
    assert!(!index.transient_recorded(&wrong));
    wrong = original.clone();
    wrong.parameter_id = id(99);
    assert!(!index.transient_recorded(&wrong));
    wrong = original.clone();
    wrong.output_unit_id = id(99);
    assert!(!index.transient_recorded(&wrong));
    wrong = original.clone();
    wrong.output_id = id(99);
    assert!(!index.transient_recorded(&wrong));
    wrong = original.clone();
    wrong.run_id = RunId::from_id(id(99));
    assert!(!index.transient_recorded(&wrong));
    wrong = original.clone();
    wrong.experiment_id = InstanceId::from_id(id(99));
    assert!(!index.transient_recorded(&wrong));
    drop(index);
    evidence.samples[0].row.time = f64::NAN;
    evidence.responses[0].row.time = Some(f64::NAN);
    let index = ResponseEvidenceIndex::new(&evidence, &pool()).unwrap();
    assert!(index.samples.is_empty());
    assert!(!index.transient_recorded(&evidence.responses[0].row));
}

#[test]
fn response_index_duplicate_context_headers_endpoints_keep_conflict_classification() {
    let mut evidence = transient();
    evidence.contexts.push(located(evidence.contexts[0].row.clone(), 10, 0));
    evidence.computations.push(located(evidence.computations[0].row.clone(), 11, 0));
    evidence.endpoints.push(located(evidence.endpoints[0].row.clone(), 12, 0));
    evidence.samples.push(located(evidence.samples[0].row.clone(), 13, 0));
    assert_eq!(count(&graph(&evidence), "recorded_transient_response"), 1);
    assert_eq!(count(&graph(&evidence), "response_coordinates"), 2);
    assert_eq!(count(&graph(&evidence), "response_completion"), 2);
    assert_eq!(count(&graph(&evidence), "response_endpoint"), 2);
    for kind in 0..3 {
        let mut conflict = transient();
        match kind {
            0 => { let mut row = conflict.contexts[0].row.clone(); row.configuration = vec![2].into();
                conflict.contexts.push(located(row, 10, 0)); }
            1 => { let mut row = conflict.computations[0].row.clone(); row.completed_samples = Some(3);
                conflict.computations.push(located(row, 11, 0)); }
            _ => { let mut row = conflict.endpoints[0].row.clone(); row.endpoint_satisfied = true;
                conflict.endpoints.push(located(row, 12, 0)); }
        }
        let graph = graph(&conflict);
        assert_eq!(count(&graph, "recorded_transient_response"), 0);
        assert_eq!(count(&graph, "withheld_response_claim"), 1);
    }
}

#[test]
fn response_index_admission_precedes_graph_effects_and_releases_its_charge() {
    let evidence = transient();
    let pool: Arc<dyn pse_columnar::MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(1));
    let mut graph = Graph::new();
    let mut nodes = BTreeMap::new();
    let attempt = vertex(&mut graph, &mut nodes, "attempt".into(), "attempt").unwrap();
    assert!(project_response_evidence(&evidence, &mut graph, &mut nodes, attempt, &pool).is_err());
    assert_eq!(graph.node_count(), 1);
    assert_eq!(graph.edge_count(), 0);
    assert_eq!(pool.reserved(), 0);
    let pool = self::pool();
    let index = ResponseEvidenceIndex::new(&evidence, &pool).unwrap();
    assert!(pool.reserved() > 0);
    drop(index);
    assert_eq!(pool.reserved(), 0);
}

fn indexed_campaign(samples: usize, directions: usize) -> ResponseEvidence {
    let mut evidence = transient();
    let sample_template = evidence.samples[0].row.clone();
    let response_template = evidence.responses[0].row.clone();
    evidence.samples.clear();
    evidence.responses.clear();
    evidence.contexts[0].row.inputs = (0..directions).map(|n| coordinate(110 + n as u8)).collect();
    evidence.computations[0].row.completed_samples = Some(samples as i64);
    for sample in 0..samples {
        let time = sample as f64 / (samples - 1) as f64;
        for output in 0..80 {
            let mut row = sample_template.clone();
            row.sample = sample as i64;
            row.time = time;
            row.symbol_id = id(20 + output);
            evidence.samples.push(located(row, 3, (sample * 80 + usize::from(output)) as u64));
            for direction in 0..directions {
                let mut row = response_template.clone();
                row.sample = sample as i64;
                row.time = Some(time);
                row.output_id = id(20 + output);
                row.parameter_id = id(110 + direction as u8);
                row.parameter_column = direction as i64;
                evidence.responses.push(located(row, 4, evidence.responses.len() as u64));
            }
        }
    }
    evidence
}

#[test]
fn response_index_campaign_shapes_preserve_exact_permissions_locators_and_bounds() {
    for samples in [5, 25] {
        for directions in [1, 4] {
            let evidence = indexed_campaign(samples, directions);
            let index = ResponseEvidenceIndex::new(&evidence, &pool()).unwrap();
            assert_eq!(index.samples.len(), 80 * samples);
            assert_eq!(index.runs.len(), 1);
            for response in &evidence.responses {
                assert!(index.transient_recorded(&response.row));
                let mut missing = response.row.clone();
                missing.parameter_id = id(120);
                assert!(!index.transient_recorded(&missing));
            }
            if samples == 25 && directions == 4 {
                // Existing relation and graph limits are independent of index
                // admission. This complete shape cannot be partially published.
                let mut graph = Graph::new();
                let mut nodes = BTreeMap::new();
                let attempt = vertex(&mut graph, &mut nodes, "attempt".into(), "attempt").unwrap();
                assert!(project_response_evidence(&evidence, &mut graph, &mut nodes, attempt, &pool()).is_err());
                assert_eq!(graph.edge_count(), ANALYSIS_EDGES);
            } else {
                let graph = graph(&evidence);
                let expected = (0..80 * samples * directions).map(|row| ArtifactLocator {
                    artifact: "independent-evidence".into(), block: 4, row: Some(row as u64), field: Some("value".into()),
                }).collect::<BTreeSet<_>>();
                let actual = graph.edge_weights().filter(|(kind, _)| kind == "recorded_transient_response")
                    .map(|(_, locator)| locator.clone().unwrap()).collect::<BTreeSet<_>>();
                assert_eq!(actual, expected);
                assert_eq!(count(&graph, "produced_evidence"), 80);
                assert_eq!(count(&graph, "withheld_response_claim"), 0);
                for kind in ["response_coordinates", "response_completion", "response_endpoint", "response_event_context"] {
                    assert_eq!(count(&graph, kind), 80);
                }
            }
        }
    }
}
