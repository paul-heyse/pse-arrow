// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::workflow::{PackResultSelection, PackScientificState};
use crate::invocation::{
    InvocationEndpoints, InvocationEvidenceDocument, InvocationInterval, InvocationOutcome,
    InvocationProduct, ProductAvailability,
};
use pse_diagnostics::observation::{
    ObservationHub, Policy, ProducerDescriptor, ProducerRole, ProducerSnapshot, RecordCoverage,
    RecordField,
};
fn fixture() -> (FrozenEvidenceSet, AnalysisRequest) {
    let hub = ObservationHub::new(Policy::default());
    let op = hub.start("review fixture");
    let id = op.context().id();
    let snapshot = op.reader().snapshot(Duration::from_secs(60));
    let set = FrozenEvidenceSet {
        operations: vec![snapshot],
        ..Default::default()
    };
    let mut request = AnalysisRequest::new(id);
    request.window = RecordedWindow::Interval {
        start: Duration::ZERO,
        end: Duration::from_secs(100),
    };
    (set, request)
}
fn record(
    set: &mut FrozenEvidenceSet,
    family: &str,
    phase: &str,
    sequence: u64,
    fields: Vec<(&str, Value)>,
) -> CoherentRecord {
    let producer = ProducerId {
        operation: set.operations[0].id,
        sequence: 1,
    };
    let role = if family.starts_with("evaluator") {
        ProducerRole::Evaluator
    } else if family.starts_with("native_factor") || family.starts_with("native_refinement") {
        ProducerRole::NativeFactor
    } else {
        ProducerRole::NativeCallback
    };
    if set.operations[0].producers.is_empty() {
        set.operations[0].producers.push(ProducerSnapshot {
            descriptor: ProducerDescriptor {
                id: producer,
                role,
                name: "performing fixture owner".into(),
            },
            current: None,
            recent: vec![],
            coverage: RecordCoverage::default(),
        });
    }
    CoherentRecord {
        producer,
        generation: 0,
        sequence,
        family_sequence: Some(sequence),
        observed_at: Duration::from_secs(sequence),
        family: family.into(),
        phase: phase.into(),
        coordinates: Some("original physical".into()),
        norm: Some("source-defined".into()),
        values: fields
            .into_iter()
            .map(|(name, value)| RecordField {
                name: name.into(),
                unit: "calls".into(),
                value,
                availability: None,
            })
            .collect(),
    }
}
fn run(set: &FrozenEvidenceSet, request: &AnalysisRequest) -> Result<Analysis, ObservationError> {
    let pool: Arc<dyn pse_columnar::MemoryPool> =
        Arc::new(pse_columnar::GreedyMemoryPool::new(128 << 20));
    analyze(set, request, &pool, &pse_columnar::CancellationToken::new())
}
fn result(set: &FrozenEvidenceSet, request: &AnalysisRequest) -> AnalysisDocument {
    run(set, request).unwrap().document().clone()
}
fn has(doc: &AnalysisDocument, q: AnalysisQuestion, code: &str) -> bool {
    doc.questions
        .iter()
        .find(|question| question.question == q)
        .unwrap()
        .statements
        .iter()
        .any(|statement| statement.code == code)
}
fn invocation(id: OperationId, run: u8) -> InvocationEvidenceDocument {
    let run_id = pse_model::generated::identities::RunId::from_bytes([run; 16]);
    let mut attempts = crate::workflow::AttemptSummary::empty_for_test(run_id);
    attempts.requested_steps = 1;
    InvocationEvidenceDocument {
        version: Default::default(),
        run_id,
        attempt_id: None,
        operation: Some(id),
        family: "modeling".into(),
        clock_scope: "fixture invocation wall; earlier preparation excluded".into(),
        timing_unavailable: None,
        canonical_run: None,
        canonical_attempt: None,
        outcome: InvocationOutcome::Completed,
        endpoints: InvocationEndpoints {
            qualification_seconds: Some(2.0),
            terminal_seconds: Some(3.0),
            response_ready_seconds: Some(4.0),
        },
        intervals: vec![InvocationInterval {
            stage: "native_join_and_workflow_postsolve".into(),
            started_seconds: 0.5,
            ended_seconds: Some(2.0),
            outcome: Some(InvocationOutcome::Completed),
        }],
        omitted_intervals: 0,
        products: vec![InvocationProduct {
            role: "solution".into(),
            step: Some(0),
            availability: ProductAvailability::Available,
        }],
        omitted_product_steps: 0,
        preparations: Some(crate::math::PreparationCounts {
            views: 2,
            observations: 1,
            rebuilt: 0,
            shared: 3,
        }),
        attempts: Arc::new(attempts),
    }
}
#[test]
fn all_questions_have_independent_missing_input_limits_and_no_zero_work() {
    let (set, request) = fixture();
    let doc = result(&set, &request);
    assert_eq!(doc.questions.len(), 10);
    for question in &doc.questions {
        assert!(
            question
                .statements
                .iter()
                .any(|statement| statement.kind == StatementKind::Limitation
                    && !statement.next.is_empty())
        );
        assert!(
            !question
                .statements
                .iter()
                .any(|statement| statement.kind == StatementKind::Established)
        );
    }
}
#[test]
fn preparation_reports_actual_binding_and_keeps_pre_invocation_unknown() {
    let (mut set, request) = fixture();
    set.invocations.push(invocation(request.operation, 1));
    let doc = result(&set, &request);
    assert!(has(
        &doc,
        AnalysisQuestion::Preparation,
        "invocation.preparation_population"
    ));
    assert!(has(
        &doc,
        AnalysisQuestion::Preparation,
        "preparation.scope"
    ));
    let statement = doc.questions[0]
        .statements
        .iter()
        .find(|statement| statement.code == "invocation.preparation_population")
        .unwrap();
    assert!(
        statement
            .evidence
            .iter()
            .any(|reference| reference.pointer.contains("/preparations/"))
    );
}
#[test]
fn evaluator_full_event_identity_not_timestamp_iteration_or_field_count() {
    let (mut set, request) = fixture();
    let mut a = record(
        &mut set,
        "evaluator_demand_batch.v1",
        "evaluation_return",
        1,
        vec![
            ("math.demand.calls", Value::Count(7)),
            ("math.demand.evaluations", Value::Count(3)),
        ],
    );
    let mut b = a.clone();
    b.producer.sequence = 2;
    set.operations[0].producers.push(ProducerSnapshot {
        descriptor: ProducerDescriptor {
            id: b.producer,
            role: ProducerRole::Evaluator,
            name: "second owner".into(),
        },
        current: None,
        recent: vec![],
        coverage: RecordCoverage::default(),
    });
    a.observed_at = Duration::from_secs(1);
    b.observed_at = a.observed_at;
    set.records = vec![a, b];
    let doc = result(&set, &request);
    assert!(has(
        &doc,
        AnalysisQuestion::EvaluatorWork,
        "event.owner_tuple"
    ));
    let population = doc.questions[1]
        .statements
        .iter()
        .find(|statement| statement.code == "event.logical_population")
        .unwrap();
    assert!(population.message.contains("2 unique"));
    assert_eq!(
        doc.questions[1]
            .statements
            .iter()
            .filter(|statement| statement.code == "event.owner_tuple")
            .count(),
        2
    );
}
#[test]
fn evaluator_undeclared_owner_never_becomes_known_work() {
    let (mut set, request) = fixture();
    let event = record(
        &mut set,
        "evaluator_demand_batch.v1",
        "evaluation_return",
        1,
        vec![("math.demand.evaluations", Value::Count(12))],
    );
    set.operations[0].producers.clear();
    set.records.push(event);
    let doc = result(&set, &request);
    assert!(has(
        &doc,
        AnalysisQuestion::EvaluatorWork,
        "event.producer_unavailable"
    ));
    assert!(!has(
        &doc,
        AnalysisQuestion::EvaluatorWork,
        "event.owner_tuple"
    ));
    set.detail_pages.push(crate::observation_detail::DetailRecordPage {
        version: Version,
        records: vec![],
        producers: vec![],
        activations: vec![crate::observation_detail::DetailSourceActivation {
            producer: ProducerDescriptor {
                id: set.records[0].producer,
                role: ProducerRole::Evaluator,
                name: "activation-only original owner".into(),
            },
            family: "evaluator_demand_batch.v1".into(),
        }],
        next_offset: None,
        partial: false,
    });
    let doc = result(&set, &request);
    assert!(has(
        &doc,
        AnalysisQuestion::EvaluatorWork,
        "event.owner_tuple"
    ));
}
#[test]
fn evaluation_means_match_actual_completed_population_and_refuse_request_substitution() {
    let (mut set, request) = fixture();
    let event=record(&mut set,"evaluator_demand_batch.v1","evaluation_return",1,vec![
        ("demand",Value::Text("objective".into())),("requested_order",Value::Integer(0)),
        ("actual_evaluations_completed",Value::Count(2)),("actual_evaluations_failed",Value::Count(1)),
        ("actual_evaluations_completed_nanoseconds",Value::Count(4_000_000_000)),("actual_evaluations_failed_nanoseconds",Value::Count(9_000_000_000)),
        ("actual_evaluation_timing_scope",Value::Text("inclusive actual local evaluator plus providers and final execution-scope check".into()))]);
    set.records.push(event);
    let doc = result(&set, &request);
    assert!(has(
        &doc,
        AnalysisQuestion::EvaluationCost,
        "evaluation.actual_inclusive_mean"
    ));
    let mean = doc.questions[2]
        .statements
        .iter()
        .find(|statement| statement.code == "evaluation.actual_inclusive_mean")
        .unwrap();
    assert!(mean.message.contains("2.000000 seconds"));
    assert!(mean.message.contains("1 failed"));
    set.records[0]
        .values
        .retain(|field| field.name != "actual_evaluations_completed_nanoseconds");
    let doc = result(&set, &request);
    assert!(has(
        &doc,
        AnalysisQuestion::EvaluationCost,
        "evaluation.actual_timing_unavailable"
    ));
    assert!(!has(
        &doc,
        AnalysisQuestion::EvaluationCost,
        "evaluation.actual_inclusive_mean"
    ));
}
#[test]
fn root_trial_preserves_native_semantics_without_physical_displacement_claim() {
    let (mut set, request) = fixture();
    let event = record(
        &mut set,
        "nonlinear_iteration.v2",
        "line_search_trial",
        1,
        vec![
            ("checkpoint_kind", Value::Text("trial".into())),
            ("step_norm", Value::Number(0.02)),
        ],
    );
    set.records.push(event);
    let doc = result(&set, &request);
    assert!(has(
        &doc,
        AnalysisQuestion::RootProgress,
        "event.owner_tuple"
    ));
    assert!(has(
        &doc,
        AnalysisQuestion::RootProgress,
        "root.progress_scope"
    ));
    assert!(
        !doc.questions[3]
            .statements
            .iter()
            .any(|statement| statement.code.contains("accepted_displacement"))
    );
}
#[test]
fn dynamics_forward_and_replay_are_distinct_populations() {
    let (mut set, request) = fixture();
    let forward = record(
        &mut set,
        "dynamic_segment_summary.v1",
        "idas.forward",
        1,
        vec![("steps", Value::Count(8))],
    );
    let replay = record(
        &mut set,
        "dynamic_replay_effort.v1",
        "idas.adjoint.replay.acceptance",
        2,
        vec![("accepted_steps_this_call", Value::Count(3))],
    );
    set.records = vec![forward, replay];
    let doc = result(&set, &request);
    assert_eq!(
        doc.questions[4]
            .statements
            .iter()
            .filter(|statement| statement.code == "event.logical_population")
            .count(),
        2
    );
    assert!(has(&doc, AnalysisQuestion::DynamicWork, "dynamics.scope"));
}
#[test]
fn dependency_reports_exact_unready_endpoint_and_unresolved_link() {
    let (mut set, request) = fixture();
    set.inventories
        .push(crate::workflow::ReviewInventorySelection {
            version: Version,
            scope: "fixture".into(),
            attempts: crate::workflow::ReviewAttemptSelection::Current,
            members: vec![crate::workflow::ReviewInventoryMember {
                key: "attempt:actual".into(),
                role: "attempt".into(),
                run: None,
                attempt: None,
                operation: Some(request.operation),
                generation: Some(3),
                occurrence: Some(1),
                terminal: false,
                outcome: None,
                result_manifest: None,
                readable_prefix: Some(9),
                links: vec![crate::workflow::ReviewInventoryLink {
                    role: "dependency".into(),
                    target: "missing-experiment".into(),
                }],
            }],
            gaps: vec![crate::workflow::ReviewInventoryGap {
                member: "missing-experiment".into(),
                reason: "dependency target is outside supplied readable membership".into(),
            }],
            truncated: false,
        });
    let doc = result(&set, &request);
    assert!(has(
        &doc,
        AnalysisQuestion::DependencyReadiness,
        "dependency.readiness_unestablished"
    ));
    assert!(has(
        &doc,
        AnalysisQuestion::DependencyReadiness,
        "dependency.recorded_link"
    ));
    assert!(has(
        &doc,
        AnalysisQuestion::DependencyReadiness,
        "dependency.scope"
    ));
}
#[test]
fn factor_recovery_is_one_parent_not_one_factor_per_substitution() {
    let (mut set, request) = fixture();
    let factor = record(
        &mut set,
        "native_factor.v1",
        "factor",
        1,
        vec![
            ("call_ordinal", Value::Count(1)),
            ("mc64_retry_attempts", Value::Count(2)),
        ],
    );
    let substitution = record(
        &mut set,
        "native_refinement.v1",
        "substitute",
        2,
        vec![
            ("parent_call_ordinal", Value::Count(1)),
            ("correction_batches_issued", Value::Count(7)),
        ],
    );
    set.records = vec![factor, substitution];
    let doc = result(&set, &request);
    assert_eq!(
        doc.questions[6]
            .statements
            .iter()
            .filter(|statement| statement.code == "event.owner_tuple")
            .count(),
        2
    );
    assert!(has(
        &doc,
        AnalysisQuestion::FactorRecovery,
        "factor.recovery_contributor"
    ));
    assert!(has(&doc, AnalysisQuestion::FactorRecovery, "factor.scope"));
}
#[test]
fn waiting_reports_actual_inclusive_interval_with_no_wall_cpu_subtraction() {
    let (mut set, request) = fixture();
    set.invocations.push(invocation(request.operation, 1));
    let doc = result(&set, &request);
    assert!(has(
        &doc,
        AnalysisQuestion::Waiting,
        "invocation.inclusive_interval"
    ));
    assert!(has(&doc, AnalysisQuestion::Waiting, "waiting.scope"));
}
#[test]
fn comparison_reuses_existing_product_refusal_and_no_metadata_equality() {
    let (mut set, mut request) = fixture();
    let baseline = OperationId {
        process_instance: request.operation.process_instance,
        sequence: request.operation.sequence + 1,
    };
    request.baseline = Some(baseline);
    let a = invocation(baseline, 1);
    let mut b = invocation(request.operation, 2);
    b.products[0].role = "trajectory".into();
    set.invocations = vec![a, b];
    let doc = result(&set, &request);
    assert!(has(
        &doc,
        AnalysisQuestion::Comparability,
        "comparison.requested_products_changed"
    ));
    assert!(has(
        &doc,
        AnalysisQuestion::Comparability,
        "comparison.scope"
    ));
}
#[test]
fn physical_original_coordinates_are_not_universal_native_pullback() {
    let (mut set, request) = fixture();
    let context = crate::workflow::ReviewContextDocument::pre_handle_failure(
        Default::default(),
        Some(request.operation),
        "fixture has no admitted source closure".into(),
        vec![],
    )
    .unwrap();
    set.contexts.push(context);
    let doc = result(&set, &request);
    assert!(has(
        &doc,
        AnalysisQuestion::PhysicalAttribution,
        "physical.context_unavailable"
    ));
    assert!(has(
        &doc,
        AnalysisQuestion::PhysicalAttribution,
        "physical.mapping_scope"
    ));
}
#[test]
fn current_recent_and_explicit_copy_are_one_event_including_empty_events() {
    let (mut set, request) = fixture();
    let event = record(&mut set, "root_evaluation.v1", "return", 1, vec![]);
    set.operations[0].producers[0].current = Some(event.clone());
    set.operations[0].producers[0].recent = vec![event.clone()];
    set.records.push(event);
    let doc = result(&set, &request);
    assert_eq!(
        doc.questions[3]
            .statements
            .iter()
            .filter(|statement| statement.code == "event.owner_tuple")
            .count(),
        1
    );
}
#[test]
fn duplicate_or_conflicting_event_identity_refuses_derived_view() {
    let (mut set, request) = fixture();
    let event = record(&mut set, "root_evaluation.v1", "return", 1, vec![]);
    set.records = vec![event.clone(), event.clone()];
    assert!(run(&set, &request).is_err());
    set.records.pop();
    let mut conflicting = event;
    conflicting.phase = "changed".into();
    set.operations[0].producers[0].current = Some(conflicting);
    assert!(run(&set, &request).is_err());
}
#[test]
fn selected_window_retains_predecessor_without_counting_it_as_work() {
    let (mut set, mut request) = fixture();
    let a = record(&mut set, "root_evaluation.v1", "return", 1, vec![]);
    let b = record(&mut set, "root_evaluation.v1", "return", 2, vec![]);
    set.records = vec![a, b];
    request.window = RecordedWindow::Interval {
        start: Duration::from_millis(1500),
        end: Duration::from_secs(3),
    };
    let doc = result(&set, &request);
    assert!(has(
        &doc,
        AnalysisQuestion::RootProgress,
        "event.predecessor"
    ));
    let population = doc.questions[3]
        .statements
        .iter()
        .find(|statement| statement.code == "event.logical_population")
        .unwrap();
    assert!(population.message.contains("1 unique"));
}
#[test]
fn exact_u64_and_nonfinite_source_categories_round_trip_and_links_resolve() {
    let (mut set, request) = fixture();
    let event = record(
        &mut set,
        "root_evaluation.v1",
        "return",
        1,
        vec![
            ("huge", Value::Count((1u64 << 53) + 17)),
            ("bad", Value::number(f64::NAN)),
            ("unknown", Value::Unavailable("not observed".into())),
        ],
    );
    set.records.push(event);
    let doc = result(&set, &request);
    let encoded = serde_json::to_vec(&doc).unwrap();
    let restored: AnalysisDocument = serde_json::from_slice(&encoded).unwrap();
    let input = serde_json::to_value(&set).unwrap();
    let evidence = &restored.questions[3]
        .statements
        .iter()
        .find(|statement| statement.code == "event.owner_tuple")
        .unwrap()
        .evidence;
    assert!(
        evidence
            .iter()
            .any(|reference| reference.value == Value::Count((1u64 << 53) + 17))
    );
    assert!(
        evidence
            .iter()
            .any(|reference| matches!(reference.value, Value::NonFinite(_)))
    );
    for reference in evidence {
        assert_eq!(reference.member, "frozen");
        assert!(input.pointer(&reference.pointer).is_some());
    }
    assert!(render_markdown(&doc).contains("9007199254741009"));
}
#[test]
fn finite_source_and_statement_bounds_remain_visible_with_all_ten_questions() {
    let (mut set, mut request) = fixture();
    for sequence in 1..5 {
        let event = record(
            &mut set,
            "root_evaluation.v1",
            "return",
            sequence,
            vec![("value", Value::Count(sequence))],
        );
        set.records.push(event);
    }
    request.source_items = 1;
    request.statements = 10;
    let doc = result(&set, &request);
    assert_eq!(doc.questions.len(), 10);
    assert_eq!(doc.inspected_source_items, 1);
    assert!(doc.omitted_source_items > 0);
    assert!(doc.omitted_statements > 0);
}
#[test]
fn unsupported_policy_or_invalid_interval_refuses_without_changing_sources() {
    let (set, mut request) = fixture();
    let before = serde_json::to_vec(&set).unwrap();
    request.window = RecordedWindow::Interval {
        start: Duration::from_secs(2),
        end: Duration::from_secs(1),
    };
    assert!(run(&set, &request).is_err());
    assert_eq!(before, serde_json::to_vec(&set).unwrap());
}
#[test]
fn physical_retained_coordinate_codec_exposes_original_context_and_mapping_limit() {
    let (mut set, request) = fixture();
    let coordinates = crate::workflow::ReviewScientificCoordinates {
        run_id: pse_model::generated::identities::RunId::from_bytes([7; 16]),
        step: 0,
        model_id: None,
        fit_id: None,
        case_id: None,
        instance_id: None,
        source_identity: pse_ids::ContentHash::from_bytes([1; 32]),
        physical_identity: pse_ids::ContentHash::from_bytes([2; 32]),
        preparation_identity: None,
        profile_identity: None,
        configuration: pse_model::Bytes::new(vec![]),
        binding: None,
        state: vec![],
        inputs: vec![],
        outputs: vec![],
        integrals: vec![],
        time: None,
        mode_names: vec![],
        mode_identities: vec![],
    };
    let mut context = crate::workflow::ReviewContextDocument::pre_handle_failure(
        Default::default(),
        Some(request.operation),
        "retained context".into(),
        vec![],
    )
    .unwrap();
    context.run_id = Some(coordinates.run_id);
    context.facts.push(crate::workflow::ReviewContextFact {
        path: "/steps/0/coordinates".into(),
        codec: "scientific-coordinates.v1".into(),
        payload: serde_json::to_vec(&coordinates).unwrap(),
    });
    set.contexts.push(context);
    let doc = result(&set, &request);
    assert!(has(
        &doc,
        AnalysisQuestion::PhysicalAttribution,
        "physical.original_coordinates"
    ));
    assert!(has(
        &doc,
        AnalysisQuestion::PhysicalAttribution,
        "physical.mapping_scope"
    ));
}
#[test]
fn engine_metric_population_preserves_kinds_units_and_abandoned_coverage() {
    use pse_engine::session::*;
    let (mut set, request) = fixture();
    set.engine_observations.push(CopiedExecutionObservations {
        policy: None,
        operation: None,
        dropped: 2,
        observations: vec![CopiedPlanObservation {
            operation: Some(request.operation),
            policy: CopiedObservationPolicy::Contract,
            logical_plan: String::new(),
            physical_plan: Some(String::new()),
            rules: vec![],
            diagnostic: None,
            execution: Some(CopiedExecutionSnapshot {
                execution: 42,
                status: CopiedExecutionStatus::Abandoned,
                metrics: vec![ObservedMetric {
                    node: 0,
                    operator: "actual operator".into(),
                    name: "peak_memory_usage".into(),
                    partition: Some(1),
                    labels: vec![],
                    unit: MetricUnit::Bytes,
                    kind: MetricKind::Gauge,
                    value: Some(128),
                }],
                truncated: false,
                observed_after_start: Duration::from_secs(2),
                collection_elapsed: Duration::from_micros(4),
            }),
        }],
    });
    let doc = result(&set, &request);
    assert!(has(
        &doc,
        AnalysisQuestion::Preparation,
        "engine.actual_policy"
    ));
    assert!(has(
        &doc,
        AnalysisQuestion::EvaluationCost,
        "engine.operator_metric"
    ));
    assert!(has(
        &doc,
        AnalysisQuestion::EvaluationCost,
        "engine.collector_dropped"
    ));
    let statement = doc.questions[2]
        .statements
        .iter()
        .find(|statement| statement.code == "engine.operator_metric")
        .unwrap();
    assert!(statement.message.contains("Gauge"));
    assert!(statement.message.contains("Bytes"));
    assert!(statement.evidence[0].partial);
}
#[test]
fn same_work_changed_inclusive_cost_and_more_work_same_mean_remain_distinct() {
    for (counts, nanos, expected) in [
        (
            [2, 2],
            [4_000_000_000u64, 8_000_000_000],
            "2.000000 then 4.000000",
        ),
        (
            [2, 4],
            [4_000_000_000, 8_000_000_000],
            "2.000000 then 2.000000",
        ),
    ] {
        let (mut set, request) = fixture();
        for i in 0..2 {
            let event=record(&mut set,"evaluator_demand_batch.v1","evaluation_return",i as u64+1,vec![("demand",Value::Text("objective".into())),("requested_order",Value::Integer(0)),("actual_evaluations_completed",Value::Count(counts[i])),("actual_evaluations_failed",Value::Count(0)),("actual_evaluations_completed_nanoseconds",Value::Count(nanos[i])),("actual_evaluation_timing_scope",Value::Text("inclusive actual local evaluator plus providers and final execution-scope check".into()))]);
            set.records.push(event);
        }
        let doc = result(&set, &request);
        let contrast = doc.questions[2]
            .statements
            .iter()
            .find(|statement| statement.code == "evaluation.retained_population_contrast")
            .unwrap();
        assert!(contrast.message.contains(expected));
        assert!(
            contrast
                .message
                .contains("compatibility is not established")
        );
    }
}
#[test]
fn multipart_factor_v2_needs_all_unique_consistent_parts_before_complete_inner_work() {
    let (mut set, request) = fixture();
    for part in 0..2 {
        let event = record(
            &mut set,
            "native_factor.v2",
            "pounce.native_factor.v2",
            part + 1,
            vec![
                ("backend", Value::Text("pounce".into())),
                ("call_primitive", Value::Text("factor".into())),
                ("parent_call_present", Value::Boolean(false)),
                ("detail_group_ordinal", Value::Count(5)),
                ("call_ordinal", Value::Count(2)),
                ("detail_part", Value::Count(part)),
                ("detail_parts", Value::Count(2)),
                ("call_path", Value::Text("monolithic".into())),
                (
                    if part == 0 {
                        "mc64_retry_attempts"
                    } else {
                        "mc64_to_infnorm_fallbacks"
                    },
                    Value::Count(1),
                ),
            ],
        );
        set.records.push(event);
    }
    let doc = result(&set, &request);
    assert!(has(
        &doc,
        AnalysisQuestion::FactorRecovery,
        "factor.group_complete"
    ));
    assert_eq!(
        doc.questions[6]
            .statements
            .iter()
            .filter(|statement| statement.code == "factor.group_work")
            .count(),
        2
    );
    let second = set.records.pop().unwrap();
    let doc = result(&set, &request);
    assert!(has(
        &doc,
        AnalysisQuestion::FactorRecovery,
        "factor.group_incomplete"
    ));
    assert!(!has(
        &doc,
        AnalysisQuestion::FactorRecovery,
        "factor.group_work"
    ));
    set.records.push(second);
    let duplicate = set.records[0]
        .values
        .iter()
        .find(|field| field.name == "parent_call_present")
        .unwrap()
        .clone();
    set.records[0].values.push(duplicate);
    let doc = result(&set, &request);
    assert!(has(
        &doc,
        AnalysisQuestion::FactorRecovery,
        "factor.group_incomplete"
    ));
    assert!(!has(
        &doc,
        AnalysisQuestion::FactorRecovery,
        "factor.group_work"
    ));
}
#[test]
fn selected_cost_question_projects_timing_before_unrelated_event_fields() {
    let (mut set, mut request) = fixture();
    let mut fields = (0..24)
        .map(|_| ("unrelated", Value::Count(1)))
        .collect::<Vec<_>>();
    fields.extend([
        ("actual_evaluations_completed", Value::Count(1)),
        ("actual_evaluations_failed", Value::Count(0)),
        (
            "actual_evaluations_completed_nanoseconds",
            Value::Count(3_000_000_000),
        ),
        (
            "actual_evaluation_timing_scope",
            Value::Text("inclusive actual evaluator".into()),
        ),
    ]);
    let event = record(
        &mut set,
        "evaluator_demand_batch.v1",
        "evaluation_return",
        1,
        fields,
    );
    set.records.push(event);
    request.question = Some(AnalysisQuestion::EvaluationCost);
    request.source_items = 8;
    request.statements = 8;
    let doc = result(&set, &request);
    assert_eq!(doc.questions.len(), 1);
    assert_eq!(doc.questions[0].question, AnalysisQuestion::EvaluationCost);
    assert!(has(
        &doc,
        AnalysisQuestion::EvaluationCost,
        "evaluation.actual_inclusive_mean"
    ));
    assert!(doc.inspected_source_items <= 8);
    doc.validate().unwrap();
    let mut missing = doc.clone();
    missing.questions.clear();
    assert!(missing.validate().is_err());
}
#[test]
fn source_bound_does_not_leak_uninspected_tuple_values_in_prose() {
    let (mut set, mut request) = fixture();
    let event = record(
        &mut set,
        "root_evaluation.v1",
        "return",
        1,
        vec![(
            "private_uninspected",
            Value::Text("unique not admitted payload".into()),
        )],
    );
    set.records.push(event);
    request.source_items = 1;
    let doc = result(&set, &request);
    let rendered = render_markdown(&doc);
    assert!(!rendered.contains("unique not admitted payload"));
    assert!(doc.omitted_source_items > 0);
}

#[test]
fn independent_child_clock_uses_own_interval_and_unknown_snapshot_retains_records() {
    let (mut set, mut request) = fixture();
    request.window = RecordedWindow::Interval {
        start: Duration::from_secs(10),
        end: Duration::from_secs(20),
    };
    let parent = record(
        &mut set,
        "dynamic_segment_summary.v1",
        "parent",
        15,
        vec![("steps", Value::Count(2))],
    );
    let mut child = set.operations[0].clone();
    child.id = OperationId {
        process_instance: [77, 88],
        sequence: 1,
    };
    child.producers.clear();
    child.interval_start = Duration::from_secs(1000);
    child.interval_end = Duration::from_secs(1100);
    set.operations[0]
        .children
        .push(pse_diagnostics::observation::ChildSnapshot {
            id: child.id,
            relationship: pse_diagnostics::observation::Relationship::Nested,
            observed_at: Duration::from_secs(12),
            kind: "child exact relation".into(),
            outcome: None,
            started_at: Duration::from_secs(1000),
            finished_at: None,
            available: true,
        });
    set.operations.push(child);
    let mut child_record = parent.clone();
    child_record.producer.operation = set.operations[1].id;
    child_record.observed_at = Duration::from_secs(1050);
    child_record.phase = "child independent clock".into();
    set.operations[1].producers.push(ProducerSnapshot {
        descriptor: ProducerDescriptor {
            id: child_record.producer,
            role: ProducerRole::NativeCallback,
            name: "child owner".into(),
        },
        current: None,
        recent: vec![],
        coverage: RecordCoverage::default(),
    });
    set.records = vec![parent, child_record];
    let doc = result(&set, &request);
    assert!(has(
        &doc,
        AnalysisQuestion::Waiting,
        "selection.independent_clock"
    ));
    let selection = &doc.selections[1];
    assert_eq!(selection.scope, AnalysisSelectionScope::Independent);
    assert_eq!(selection.interval_start, Some(Duration::from_secs(1000)));
    assert!(!selection.aligned_to_request);
    assert!(
        doc.questions[4]
            .statements
            .iter()
            .any(|statement| statement.message.contains("child independent clock"))
    );
    set.operations.pop();
    let doc = result(&set, &request);
    assert_eq!(doc.selections[1].scope, AnalysisSelectionScope::Unknown);
    assert_eq!(doc.selections[1].interval_end, None);
    assert!(doc.questions[4].statements.iter().any(|statement| {
        statement
            .evidence
            .iter()
            .any(|reference| reference.pointer.starts_with("/records/1"))
    }));
}

#[test]
fn analysis_preparation_refuses_tiny_pool_and_holds_owner_through_encoding() {
    let (set, request) = fixture();
    let tiny: Arc<dyn pse_columnar::MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(1));
    let cancel = pse_columnar::CancellationToken::new();
    assert!(
        analyze(&set, &request, &tiny, &cancel)
            .unwrap_err()
            .to_string()
            .contains("admission refused")
    );
    assert_eq!(tiny.reserved(), 0);
    let pool: Arc<dyn pse_columnar::MemoryPool> =
        Arc::new(pse_columnar::GreedyMemoryPool::new(128 << 20));
    let output = analyze(&set, &request, &pool, &cancel).unwrap();
    assert_eq!(pool.reserved(), output.retained_bytes());
    assert!(pool.reserved() > 0);
    serde_json::to_vec(&output).unwrap();
    assert_eq!(pool.reserved(), output.retained_bytes());
    drop(output);
    assert_eq!(pool.reserved(), 0);
    cancel.cancel();
    assert!(analyze(&set, &request, &pool, &cancel).is_err());
    assert_eq!(pool.reserved(), 0);
}

#[test]
fn canonical_inventory_without_operation_joins_run_attempt_and_explicit_gap_only_selection() {
    let (mut set, mut request) = fixture();
    request.question = Some(AnalysisQuestion::DependencyReadiness);
    let mut invocation = invocation(request.operation, 1);
    invocation.canonical_run = Some("canonical:run/exact".into());
    invocation.canonical_attempt = Some("canonical:attempt/exact".into());
    set.invocations.push(invocation);
    set.inventories
        .push(crate::workflow::ReviewInventorySelection {
            version: Version,
            scope: "retained canonical study page".into(),
            attempts: crate::workflow::ReviewAttemptSelection::Retained,
            members: vec![crate::workflow::ReviewInventoryMember {
                key: "canonical:attempt/exact".into(),
                role: "canonical_attempt".into(),
                run: Some("canonical:run/exact".into()),
                attempt: Some("canonical:attempt/exact".into()),
                operation: None,
                generation: Some(3),
                occurrence: Some(2),
                terminal: false,
                outcome: Some("running".into()),
                result_manifest: None,
                readable_prefix: Some(7),
                links: vec![],
            }],
            gaps: vec![],
            truncated: false,
        });
    let doc = result(&set, &request);
    assert!(has(
        &doc,
        AnalysisQuestion::DependencyReadiness,
        "dependency.readiness_unestablished"
    ));
    assert!(
        doc.questions[0]
            .statements
            .iter()
            .any(|statement| statement.message.contains("readable_prefix=Some(7)"))
    );
    set.inventories
        .push(crate::workflow::ReviewInventorySelection {
            version: Version,
            scope: "exact gap only page".into(),
            attempts: crate::workflow::ReviewAttemptSelection::Current,
            members: vec![],
            gaps: vec![crate::workflow::ReviewInventoryGap {
                member: "missing exact attempt".into(),
                reason: "retained generation unavailable".into(),
            }],
            truncated: true,
        });
    request.inventory_members = vec![1];
    let doc = result(&set, &request);
    assert!(has(
        &doc,
        AnalysisQuestion::DependencyReadiness,
        "dependency.inventory_gap"
    ));
    assert!(!has(
        &doc,
        AnalysisQuestion::DependencyReadiness,
        "dependency.readiness_unestablished"
    ));
    request.inventory_members = vec![1, 1];
    assert!(run(&set, &request).is_err());
}

#[test]
fn actual_diffsol_setup_recovery_and_idas_latest_gauges_survive_without_segment_summary() {
    let hub = ObservationHub::new(Policy::default());
    let op = hub.start("actual shaped native scalars");
    let context = op.context();
    context.count_value(
        "diffsol.work.number_of_linear_solver_setups_from_checkpoint",
        "count",
        3,
        2,
    );
    context.count_value(
        "diffsol.work.number_of_linear_solver_setups_from_error_test_fail",
        "count",
        4,
        2,
    );
    context.count("diffsol.klu.refactor.conditioning_rejected", "count", 1);
    context.gauge("idas.newton.shift", "inverse-time", 0.25);
    context.integer("idas.checkpoints.current", "checkpoints", 4);
    let set = FrozenEvidenceSet {
        operations: vec![op.reader().snapshot(Duration::from_secs(60))],
        ..Default::default()
    };
    let request = AnalysisRequest::new(context.id());
    let doc = result(&set, &request);
    assert!(has(
        &doc,
        AnalysisQuestion::DynamicWork,
        "dynamics.setup_cause"
    ));
    assert!(has(
        &doc,
        AnalysisQuestion::FactorRecovery,
        "factor.conditioning_recovery"
    ));
    let gauge = doc.questions[4]
        .statements
        .iter()
        .find(|statement| {
            statement.code == "owner.metric" && statement.message.starts_with("idas.newton.shift:")
        })
        .unwrap();
    assert!(gauge.evidence[0].pointer.ends_with("/latest"));
    assert_eq!(gauge.evidence[0].value, Value::number(0.25));
    let checkpoint = doc.questions[4]
        .statements
        .iter()
        .find(|statement| {
            statement.code == "owner.metric"
                && statement.message.starts_with("idas.checkpoints.current:")
        })
        .unwrap();
    assert!(checkpoint.evidence[0].pointer.ends_with("/latest"));
    assert_eq!(checkpoint.evidence[0].value, Value::Integer(4));
}

#[derive(Debug)]
struct CancelDuringPreparation {
    budget: pse_columnar::GreedyMemoryPool,
    cancel: pse_columnar::CancellationToken,
}
impl std::fmt::Display for CancelDuringPreparation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("cancel during analysis preparation")
    }
}
impl pse_columnar::MemoryPool for CancelDuringPreparation {
    fn name(&self) -> &'static str {
        "cancel analysis preparation"
    }
    fn register(&self, consumer: &pse_columnar::MemoryConsumer) {
        self.budget.register(consumer);
    }
    fn unregister(&self, consumer: &pse_columnar::MemoryConsumer) {
        self.budget.unregister(consumer);
    }
    fn grow(&self, reservation: &pse_columnar::MemoryReservation, bytes: usize) {
        self.budget.grow(reservation, bytes);
    }
    fn shrink(&self, reservation: &pse_columnar::MemoryReservation, bytes: usize) {
        self.budget.shrink(reservation, bytes);
    }
    fn try_grow(
        &self,
        reservation: &pse_columnar::MemoryReservation,
        bytes: usize,
    ) -> datafusion::common::Result<()> {
        self.budget.try_grow(reservation, bytes)?;
        self.cancel.cancel();
        Ok(())
    }
    fn reserved(&self) -> usize {
        self.budget.reserved()
    }
    fn memory_limit(&self) -> datafusion::execution::memory_pool::MemoryLimit {
        self.budget.memory_limit()
    }
}
#[test]
fn cancellation_during_preparation_releases_reservation_before_validation() {
    let (mut set, request) = fixture();
    // This otherwise invalid member makes validation order observable.
    let mut invocation = invocation(request.operation, 1);
    invocation.run_id = pse_model::generated::identities::RunId::from_bytes([99; 16]);
    assert!(invocation.validate().is_err());
    set.invocations.push(invocation);
    let cancel = pse_columnar::CancellationToken::new();
    let pool: Arc<dyn pse_columnar::MemoryPool> = Arc::new(CancelDuringPreparation {
        budget: pse_columnar::GreedyMemoryPool::new(128 << 20),
        cancel: cancel.clone(),
    });
    let error = analyze(&set, &request, &pool, &cancel).unwrap_err();
    assert!(error.to_string().to_lowercase().contains("cancel"));
    assert_eq!(pool.reserved(), 0);
}

#[test]
fn original_finding_follows_retained_affine_occurrence_and_source_term_without_native_claim() {
    let (mut set, mut request) = fixture();
    request.question = Some(AnalysisQuestion::PhysicalAttribution);
    let mut invocation = invocation(request.operation, 9);
    let variable = pse_ids::SemanticId::from_bytes([1; 16]);
    let row = pse_ids::SemanticId::from_bytes([2; 16]);
    let declaration = pse_ids::SemanticId::from_bytes([3; 16]);
    let instance = pse_ids::SemanticId::from_bytes([4; 16]);
    let local = pse_ids::SemanticId::from_bytes([5; 16]);
    let attempts = Arc::get_mut(&mut invocation.attempts).unwrap();
    // Build the owner's serialized empty step rather than inventing scientific behavior.
    attempts.steps.push(serde_json::from_value(serde_json::json!({
        "step":0,"backend":null,"observation":null,"accepted":false,"work":null,
        "start_submitted":null,"reused_native_state":null,"normalized_physical_violation":null,
        "effective_options":{},"native_defaults":{},"start":null,
        "intervals":{"preparation_seconds":null,"queue_seconds":null,"native_including_callbacks_seconds":null,"assessment_seconds":null,"publication_seconds":null},
        "decisions":[],"omitted_decisions":0,
        "findings":[{"source":variable,"violation":{"kind":"real","value":2.0},"tolerance":{"kind":"real","value":0.1}}],
        "omitted_findings":0,"dropped_progress":null,"native_observations":null,"raw_map":null
    })).unwrap());
    let map: crate::workflow::ModelingSourceMapDocument = serde_json::from_value(serde_json::json!({
        "version":1,"source_identity":pse_ids::ContentHash::from_bytes([8;32]),"numerical_identity":pse_ids::ContentHash::from_bytes([7;32]),
        "scope":"static_eligible_alternatives","rows":[row],"columns":[variable],"fixed_inputs":[],
        "row_nominals":[[row,4.0]],"variable_nominals":[[variable,10.0]],"objective_signs":[],
        "fields":[{"declaration":declaration,"role":"expression","position":0,"document":null,"start":null,"end":null}],
        "members":[{"member":variable,"declaration":declaration,"instance":instance,"path":"source.variable"}],
        "templates":[],"instances":[{"ordinal":0,"instance":instance,"body":pse_ids::ContentHash::from_bytes([6;32]),
            "checked_members":[[local,variable]],"slots":[{"formal":0,"member":variable,"scale":2.0,"offset":3.0,"fixed":false}],
            "contributions":[{"output":0,"target":{"kind":"row","row":row},"scale":-1.0}],"demands":[]}],
        "native_pullback_unavailable":"native dual and stationarity pullback unknown"
    })).unwrap();
    let mut context = crate::workflow::ReviewContextDocument::pre_handle_failure(
        Default::default(),
        Some(request.operation),
        "retained source context".into(),
        vec![],
    )
    .unwrap();
    context.run_id = Some(invocation.run_id);
    context.attach_source_map(0, None, &map).unwrap();
    set.contexts.push(context);
    set.invocations.push(invocation);
    let doc = result(&set, &request);
    assert!(has(
        &doc,
        AnalysisQuestion::PhysicalAttribution,
        "physical.original_finding"
    ));
    assert!(has(
        &doc,
        AnalysisQuestion::PhysicalAttribution,
        "physical.original_member"
    ));
    let occurrence = doc.questions[0]
        .statements
        .iter()
        .find(|statement| statement.code == "physical.original_occurrence")
        .unwrap();
    assert!(occurrence.message.contains("scale 2"));
    assert!(occurrence.message.contains("Some(3.0)"));
    assert!(occurrence.message.contains(&local.to_string()));
    assert!(
        occurrence
            .evidence
            .iter()
            .any(|reference| reference.pointer == "/contexts/0/facts/0/payload")
    );
    let raw = serde_json::to_value(&set).unwrap();
    assert!(
        occurrence
            .evidence
            .iter()
            .all(|reference| raw.pointer(&reference.pointer).is_some())
    );
    assert!(doc.questions[0].statements.iter().any(|statement| {
        statement
            .message
            .contains("native dual and stationarity pullback unknown")
    }));
}

#[test]
fn moderate_borrowed_source_population_fits_the_default_analysis_pool() {
    let (mut set, mut request) = fixture();
    request.window = RecordedWindow::Interval {
        start: Duration::ZERO,
        end: Duration::from_secs(2048),
    };
    for sequence in 1..=1024 {
        let event = record(
            &mut set,
            "root_evaluation.v1",
            "actual retained source",
            sequence,
            vec![("source_value", Value::Text("x".repeat(2048)))],
        );
        set.records.push(event);
    }
    // Previously the whole serialized input was charged 24 times, independently
    // of its borrowed records, exhausting the ordinary 64 MiB view allowance.
    assert!(serde_json::to_vec(&set).unwrap().len() > 2 << 20);
    let pool: Arc<dyn pse_columnar::MemoryPool> =
        Arc::new(pse_columnar::GreedyMemoryPool::new(64 << 20));
    let output = analyze(
        &set,
        &request,
        &pool,
        &pse_columnar::CancellationToken::new(),
    )
    .unwrap();
    assert_eq!(output.inspected_source_items, request.source_items);
    assert!(output.omitted_source_items > 0);
    assert!(output.retained_bytes() < 64 << 20);
    drop(output);
    assert_eq!(pool.reserved(), 0);
}

#[test]
fn plan47f_unselected_event_conflicts_and_context_receipts_do_not_enter_answers() {
    let (mut set, mut request) = fixture();
    request.question = Some(AnalysisQuestion::Preparation);
    let unrelated = OperationId {
        process_instance: [901, 902],
        sequence: 903,
    };
    let event = record(&mut set, "evaluator_demand_batch.v1", "source", 1, vec![]);
    let mut outside = event.clone();
    outside.producer.operation = unrelated;
    let mut conflict = outside.clone();
    conflict.phase = "conflicting unselected mirror".into();
    set.records = vec![event, outside];
    set.detail_pages.push(crate::observation_detail::DetailRecordPage {
        version: Version,
        records: vec![conflict],
        producers: vec![],
        activations: vec![],
        next_offset: None,
        partial: true,
    });
    // Ingress validates every supported codec; question interpretation still requires exact
    // subject association. This control does not claim validation skips unselected payloads.
    set.contexts.push(
        crate::workflow::ReviewContextDocument::pre_handle_failure(
            Default::default(),
            Some(unrelated),
            "unselected original attachment".into(),
            vec![crate::workflow::ReviewContextFact {
                path: "/physical_admission/0/engine".into(),
                codec: "engine-execution-observations.v1".into(),
                payload: serde_json::to_vec(&pse_engine::session::CopiedExecutionObservations {
                    policy: Some(pse_engine::session::CopiedObservationPolicy::Off),
                    operation: Some(unrelated),
                    observations: vec![],
                    dropped: 0,
                })
                .unwrap(),
            }],
        )
        .unwrap(),
    );
    let pool: Arc<dyn pse_columnar::MemoryPool> =
        Arc::new(pse_columnar::GreedyMemoryPool::new(64 << 20));
    let output = analyze(
        &set,
        &request,
        &pool,
        &pse_columnar::CancellationToken::new(),
    )
    .unwrap();
    assert!(!has(
        &output,
        AnalysisQuestion::Preparation,
        "engine.context_association"
    ));
    drop(output);
    assert_eq!(pool.reserved(), 0);
    // Selecting the conflicting original event must fail rather than hide its disagreement.
    request.operation = unrelated;
    assert!(
        analyze(
            &set,
            &request,
            &pool,
            &pse_columnar::CancellationToken::new()
        )
        .unwrap_err()
        .to_string()
        .contains("conflicting full coherent event")
    );
    set.detail_pages.clear();
    let selected = analyze(
        &set,
        &request,
        &pool,
        &pse_columnar::CancellationToken::new(),
    )
    .unwrap();
    assert!(has(
        &selected,
        AnalysisQuestion::Preparation,
        "engine.context_association"
    ));
    drop(selected);
    assert_eq!(pool.reserved(), 0);
}

#[test]
fn plan47f_growth_and_skew_analyze_export_and_relocated_readback() {
    use crate::tooling::review_pack::*;
    use crate::tooling::review_query::{ReviewTableKind, ReviewTableRequest};
    use crate::workflow::{
        ReviewAttemptSelection, ReviewInventoryGap, ReviewInventoryLink, ReviewInventoryMember,
        ReviewInventorySelection,
    };
    use datafusion::arrow::{
        array::{Array, Float64Array, UInt64Array},
        ipc::reader::StreamReader,
    };
    use std::io::{Read, Write};
    for members in [16usize, 129] {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("pack");
        let (mut set, mut request) = fixture();
        request.window = RecordedWindow::Interval {
            start: Duration::ZERO,
            end: Duration::from_secs(2048),
        };
        for sequence in 1..=1024 {
            let event = record(
                &mut set,
                "evaluator_demand_batch.v1",
                "original batch return",
                sequence,
                vec![
                    ("exact_count", Value::Count(u64::MAX - sequence)),
                    ("signed_half", Value::Number(-(sequence as f64) / 2.0)),
                ],
            );
            set.records.push(event);
        }
        // Mirrors of an original event are transport copies, never extra evaluator work.
        set.operations[0].producers[0].current = Some(set.records[0].clone());
        set.operations[0].producers[0].recent = vec![set.records[0].clone()];
        set.detail_pages.push(crate::observation_detail::DetailRecordPage {
            version: Version,
            records: set.records.clone(),
            producers: vec![],
            activations: vec![],
            next_offset: None,
            partial: false,
        });
        let payload = serde_json::to_vec(&"x".repeat(2 << 20)).unwrap();
        let opaque_payload_bytes = payload.len();
        set.contexts.push(
            crate::workflow::ReviewContextDocument::pre_handle_failure(
                Default::default(),
                Some(request.operation),
                "retained opaque original input; no executable factory".into(),
                vec![crate::workflow::ReviewContextFact {
                    path: "/original_inputs".into(),
                    codec: "physical-input-bits.v1".into(),
                    payload,
                }],
            )
            .unwrap(),
        );
        let inventory_members = (0..members)
            .map(|index| ReviewInventoryMember {
                key: format!("attempt-{index}"),
                role: "canonical_attempt".into(),
                run: Some(format!("run-{}", index / 2)),
                attempt: Some(format!("attempt-{index}")),
                operation: None,
                generation: Some((index % 2 + 1) as u64),
                occurrence: Some((index / 2) as u64),
                terminal: index % 2 == 0,
                outcome: Some(if index % 2 == 0 { "failed" } else { "running" }.into()),
                result_manifest: None,
                readable_prefix: Some(index as u64 + 3),
                links: if index == 0 {
                    vec![ReviewInventoryLink {
                        role: "shared_producer".into(),
                        target: "missing-original-attempt".into(),
                    }]
                } else {
                    vec![]
                },
            })
            .collect::<Vec<_>>();
        let current_members = inventory_members
            .iter()
            .enumerate()
            .filter(|(index, _)| index % 2 == 1 || *index == members - 1)
            .map(|(_, member)| member.clone())
            .collect();
        let gap = ReviewInventoryGap {
            member: "missing-original-attempt".into(),
            reason: "original retained retry unavailable".into(),
        };
        set.inventories = vec![
            ReviewInventorySelection {
                version: Version,
                scope: "selected retained retries".into(),
                attempts: ReviewAttemptSelection::Retained,
                members: inventory_members,
                gaps: vec![gap.clone()],
                truncated: false,
            },
            ReviewInventorySelection {
                version: Version,
                scope: "selected current pointers".into(),
                attempts: ReviewAttemptSelection::Current,
                members: current_members,
                gaps: vec![gap],
                truncated: false,
            },
        ];
        let pool: Arc<dyn pse_columnar::MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(64 << 20));
        let started = std::time::Instant::now();
        let output = analyze(
            &set,
            &request,
            &pool,
            &pse_columnar::CancellationToken::new(),
        )
        .unwrap();
        let analysis_elapsed_ns = started.elapsed().as_nanos();
        let analysis_reserved_bytes = pool.reserved();
        assert_eq!(analysis_reserved_bytes, output.retained_bytes());
        assert!(analysis_reserved_bytes <= 64 << 20);
        assert!(output.inspected_source_items <= 1024);
        assert!(output.omitted_source_items > 0);
        assert!(output.omitted_statements > 0);
        assert!(
            output
                .questions
                .iter()
                .map(|question| question.statements.len())
                .sum::<usize>()
                <= 128
        );
        let output_json_bytes = serde_json::to_vec(output.document()).unwrap().len();
        drop(output);
        assert_eq!(pool.reserved(), 0);

        // A small original-time selection independently establishes deduplication and exact values.
        request.question = Some(AnalysisQuestion::EvaluatorWork);
        request.window = RecordedWindow::Interval {
            start: Duration::from_secs(1),
            end: Duration::from_secs(8),
        };
        let selected = analyze(
            &set,
            &request,
            &pool,
            &pse_columnar::CancellationToken::new(),
        )
        .unwrap();
        let population = selected.questions[0]
            .statements
            .iter()
            .find(|statement| statement.code == "event.logical_population")
            .unwrap();
        assert!(population.message.contains("8 unique coherent"));
        assert_eq!(
            selected.questions[0]
                .statements
                .iter()
                .filter(|statement| statement.code == "event.owner_tuple")
                .count(),
            8
        );
        let exact = selected.questions[0]
            .statements
            .iter()
            .flat_map(|statement| &statement.evidence)
            .find(|reference| {
                reference
                    .event
                    .as_ref()
                    .is_some_and(|event| event.sequence == 1 && event.field_ordinal == Some(0))
            })
            .unwrap();
        assert_eq!(exact.value, Value::Count(u64::MAX - 1));

        let table = ReviewTableRequest {
            version: Version,
            kind: ReviewTableKind::Detail,
            index: 0,
            memory_bytes: 64 << 20,
        };
        let member = |id: String, role, format| PackMemberSelection {
            id,
            role,
            required: false,
            format,
            document_version: None,
            source: Some("/original/removed/descriptive-only".into()),
            references: vec![],
            result: None,
            capture_scope: None,
            diagnostic_table: None,
        };
        let mut selection = PackSelection::default();
        for index in 0..members {
            selection.members.push(member(
                format!("attempt-{index}"),
                PackMemberRole::Context,
                PackFormat::Text,
            ));
        }
        selection.members.push(member(
            "optional_profile".into(),
            PackMemberRole::Capture,
            PackFormat::ProfileBinary,
        ));
        let mut unavailable_result = member(
            "selected_result".into(),
            PackMemberRole::Result,
            PackFormat::ArrowIpcStream,
        );
        unavailable_result.result = Some(PackResultSelection {
            run: "run-0".into(),
            attempt: "attempt-0".into(),
            relation: "runtime.solve_metrics".into(),
            schema_version: 1,
            row_start: 0,
            row_end: 1,
            field: None,
            terminal_class: "failed".into(),
            predicate: None,
            usability: PackScientificState::Unusable,
        });
        selection.members.push(unavailable_result);
        let mut diagnostic = member(
            "diagnostic".into(),
            PackMemberRole::Detail,
            PackFormat::ArrowIpcStream,
        );
        diagnostic.required = true;
        diagnostic.diagnostic_table = Some(PackDiagnosticTable::new(table.clone()).unwrap());
        selection.members.push(diagnostic);
        let started = std::time::Instant::now();
        let mut session =
            PackSession::create(&path, selection, pse_columnar::CancellationToken::new()).unwrap();
        session.add_frozen(&set).unwrap();
        session.add_analysis(&selected).unwrap();
        drop(selected);
        assert_eq!(pool.reserved(), 0);
        let source = directory.path().join("selected-source");
        // Supply one small member at a time; no concatenated pack-sized payload collection.
        for index in 0..members {
            let bytes = format!(
                "original attempt-{index}; run-{}; generation {}; prefix {}\n",
                index / 2,
                index % 2 + 1,
                index + 3
            );
            std::fs::File::create(&source)
                .unwrap()
                .write_all(bytes.as_bytes())
                .unwrap();
            session
                .copy_prefix(
                    &format!("attempt-{index}"),
                    std::fs::File::open(&source).unwrap(),
                    bytes.len() as u64,
                )
                .unwrap();
        }
        session
            .unavailable(
                "optional_profile",
                "not captured; no profiler capability or measurement claimed".into(),
            )
            .unwrap();
        session
            .unavailable(
                "selected_result",
                "original failed attempt retained no selected result rows".into(),
            )
            .unwrap();
        session.export_table("diagnostic", &set, &table).unwrap();
        let index = session.seal().unwrap();
        let export_elapsed_ns = started.elapsed().as_nanos();
        let frozen_bytes = index
            .members
            .iter()
            .find(|member| member.selection.id == "frozen")
            .unwrap()
            .readable_bytes;
        let output_bytes = index
            .members
            .iter()
            .map(|member| member.readable_bytes)
            .sum::<u64>();
        assert!(output_bytes <= index.selection.maximum_output_bytes);
        assert!(!index.membership_complete);
        assert_eq!(index.members.len(), members + 7);
        std::fs::remove_file(source).unwrap();
        let relocated = directory.path().join("relocated");
        std::fs::rename(&path, &relocated).unwrap();
        std::fs::create_dir(&path).unwrap();
        std::fs::write(path.join("index.json"), b"conflicting live neighbour").unwrap();
        let reader = PackReader::open(&relocated, MAXIMUM_PACK_DOCUMENT_BYTES).unwrap();
        let restored = reader.read_frozen().unwrap();
        assert!(reader.open_member("optional_profile").is_err());
        assert!(reader.open_member("selected_result").is_err());
        assert!(reader.open_member("unrequested_result").is_err());
        let result_scope = reader
            .index()
            .members
            .iter()
            .find(|member| member.selection.id == "selected_result")
            .unwrap()
            .selection
            .result
            .as_ref()
            .unwrap();
        assert_eq!(
            (&*result_scope.run, &*result_scope.attempt),
            ("run-0", "attempt-0")
        );
        assert_eq!(result_scope.usability, PackScientificState::Unusable);
        assert_eq!(restored.records.len(), 1024);
        assert_eq!(restored.inventories[0].members.len(), members);
        assert_eq!(
            restored.contexts[0].facts[0].payload.len(),
            opaque_payload_bytes
        );
        for index in 0..members {
            let mut bytes = String::new();
            reader
                .open_member(&format!("attempt-{index}"))
                .unwrap()
                .read_to_string(&mut bytes)
                .unwrap();
            assert_eq!(
                bytes,
                format!(
                    "original attempt-{index}; run-{}; generation {}; prefix {}\n",
                    index / 2,
                    index % 2 + 1,
                    index + 3
                )
            );
        }
        // Raw source identity/value assertions are independent of the exporter-created index.
        for (index, record) in restored.records.iter().enumerate() {
            assert_eq!(record.sequence, index as u64 + 1);
            assert_eq!(
                record.values[0].value,
                Value::Count(u64::MAX - index as u64 - 1)
            );
        }
        let stream =
            StreamReader::try_new(reader.open_member("diagnostic").unwrap(), None).unwrap();
        let mut rows = 0;
        for batch in stream {
            let batch = batch.unwrap();
            let counts = batch
                .column_by_name("count_value")
                .unwrap()
                .as_any()
                .downcast_ref::<UInt64Array>()
                .unwrap();
            let numbers = batch
                .column_by_name("number_value")
                .unwrap()
                .as_any()
                .downcast_ref::<Float64Array>()
                .unwrap();
            let events = batch
                .column_by_name("event_sequence")
                .unwrap()
                .as_any()
                .downcast_ref::<UInt64Array>()
                .unwrap();
            for row in 0..batch.num_rows() {
                let sequence = (rows / 2 + 1) as u64;
                assert_eq!(events.value(row), sequence);
                if rows % 2 == 0 {
                    assert_eq!(counts.value(row), u64::MAX - sequence);
                    assert!(numbers.is_null(row));
                } else {
                    assert_eq!(
                        numbers.value(row).to_bits(),
                        (-(sequence as f64) / 2.0).to_bits()
                    );
                    assert!(counts.is_null(row));
                }
                rows += 1;
            }
        }
        assert_eq!(rows, 2048);
        // Exact current-versus-retained inventory selection is independent of record/output limits.
        request.question = Some(AnalysisQuestion::DependencyReadiness);
        request.window = RecordedWindow::Interval {
            start: Duration::ZERO,
            end: Duration::ZERO,
        };
        for inventory in [0, 1] {
            request.inventory_members = vec![inventory];
            let output = analyze(
                &restored,
                &request,
                &pool,
                &pse_columnar::CancellationToken::new(),
            )
            .unwrap();
            let keys = output.questions[0]
                .statements
                .iter()
                .filter(|statement| statement.code == "dependency.readiness_unestablished")
                .map(|statement| statement.evidence[0].value.clone())
                .collect::<Vec<_>>();
            let expected = restored.inventories[inventory]
                .members
                .iter()
                .take(keys.len())
                .map(|member| Value::Text(member.key.clone()))
                .collect::<Vec<_>>();
            assert_eq!(keys, expected);
            assert!(!keys.is_empty());
            if inventory == 1 {
                assert_eq!(keys.len(), members.div_ceil(2));
            }
            if inventory == 0 && members == 129 {
                assert!(output.omitted_statements > 0);
            }
            if members == 16 {
                assert!(has(
                    &output,
                    AnalysisQuestion::DependencyReadiness,
                    "dependency.inventory_gap"
                ));
            }
            drop(output);
            assert_eq!(pool.reserved(), 0);
        }
        println!(
            "plan47f_measurement={}",
            serde_json::json!({
                "case": "analysis_export_relocated", "inventory_members": members, "logical_events": 1024,
                "field_rows": rows, "mirrored_event_copies": 1026, "opaque_input_payload_bytes": opaque_payload_bytes,
                "frozen_input_json_bytes": frozen_bytes, "analysis_json_bytes": output_json_bytes,
                "pack_payload_output_bytes": output_bytes, "selected_pack_members": index.members.len(),
                "analysis_elapsed_ns": analysis_elapsed_ns, "export_elapsed_ns": export_elapsed_ns,
                "analysis_retained_reservation_bytes": analysis_reserved_bytes, "analysis_reservation_after_drop_bytes": pool.reserved(),
                "analysis_pool_bytes": 64 << 20, "table_pool_bytes": table.memory_bytes,
                "private_peak_memory": "not measured", "timing_gate": false, "profiler_capture": "not captured",
            })
        );
    }
}

#[test]
fn large_opaque_payload_pays_for_validation_scratch_without_a_whole_input_multiplier() {
    let (mut set, request) = fixture();
    let payload = serde_json::to_vec(&"original input bits".repeat(131072)).unwrap();
    assert!(payload.len() > 2 << 20);
    let context = crate::workflow::ReviewContextDocument::pre_handle_failure(
        Default::default(),
        Some(request.operation),
        "original input attachment".into(),
        vec![crate::workflow::ReviewContextFact {
            path: "/original_inputs".into(),
            codec: "physical-input-bits.v1".into(),
            payload,
        }],
    )
    .unwrap();
    set.contexts.push(context);
    let pool: Arc<dyn pse_columnar::MemoryPool> =
        Arc::new(pse_columnar::GreedyMemoryPool::new(64 << 20));
    let output = analyze(
        &set,
        &request,
        &pool,
        &pse_columnar::CancellationToken::new(),
    )
    .unwrap();
    assert!(has(
        &output,
        AnalysisQuestion::Comparability,
        "comparison.physical_inputs_retained"
    ));
    assert!(output.retained_bytes() < 16 << 20);
    drop(output);
    assert_eq!(pool.reserved(), 0);
}

#[test]
fn selected_run_context_joins_original_engine_scope_and_deduplicates_flat_mirror() {
    use pse_engine::session::*;
    for original in [
        Some(OperationId {
            process_instance: [700, 900],
            sequence: 81,
        }),
        None,
    ] {
        let (mut set, request) = fixture();
        let invocation = invocation(request.operation, 27);
        let trace = CopiedExecutionObservations {
            policy: Some(CopiedObservationPolicy::Contract),
            operation: original,
            dropped: 2,
            observations: vec![CopiedPlanObservation {
                operation: original,
                policy: CopiedObservationPolicy::Contract,
                logical_plan: String::new(),
                physical_plan: Some(String::new()),
                rules: vec![],
                diagnostic: None,
                execution: Some(CopiedExecutionSnapshot {
                    execution: 902,
                    status: CopiedExecutionStatus::Abandoned,
                    metrics: vec![ObservedMetric {
                        node: 0,
                        operator: "actual physical admission operator".into(),
                        name: "peak_memory_usage".into(),
                        partition: Some(0),
                        labels: vec![],
                        unit: MetricUnit::Bytes,
                        kind: MetricKind::Gauge,
                        value: Some(128),
                    }],
                    truncated: false,
                    observed_after_start: Duration::from_secs(7000),
                    collection_elapsed: Duration::from_micros(3),
                }),
            }],
        };
        let mut context = crate::workflow::ReviewContextDocument::pre_handle_failure(
            Default::default(),
            Some(request.operation),
            "retained actual source session".into(),
            vec![],
        )
        .unwrap();
        context.run_id = Some(invocation.run_id);
        context.facts.push(crate::workflow::ReviewContextFact {
            path: "/physical_admission/0/engine".into(),
            codec: "engine-execution-observations.v1".into(),
            payload: serde_json::to_vec(&trace).unwrap(),
        });
        set.contexts.push(context);
        set.invocations.push(invocation);
        set.engine_observations.push(trace.clone());
        let document = result(&set, &request);
        assert!(has(
            &document,
            AnalysisQuestion::Preparation,
            "engine.context_association"
        ));
        let policy = document.questions[0]
            .statements
            .iter()
            .find(|statement| statement.code == "engine.actual_session_policy")
            .unwrap();
        assert!(policy.message.contains(&format!("{original:?}")));
        assert_eq!(policy.evidence[0].pointer, "/contexts/0/facts/0/payload");
        let metric = document.questions[2]
            .statements
            .iter()
            .filter(|statement| statement.code == "engine.operator_metric")
            .collect::<Vec<_>>();
        assert_eq!(
            metric.len(),
            1,
            "a mirrored flat copy must not duplicate actual work"
        );
        assert_eq!(metric[0].evidence[0].pointer, "/contexts/0/facts/0/payload");
        assert_eq!(metric[0].evidence[0].value, Value::Count(128));
        assert!(
            metric[0]
                .message
                .contains("Decoded payload coordinate /observations/0/execution/metrics/0/value")
        );
        let lifetime = document.questions[7]
            .statements
            .iter()
            .find(|statement| statement.code == "engine.lifetime")
            .unwrap();
        assert!(lifetime.message.contains("does not align"));
        assert_eq!(document.selections.len(), 1);
        assert_eq!(document.selections[0].operation, request.operation);
        let raw = serde_json::to_value(&set).unwrap();
        assert!(
            document
                .questions
                .iter()
                .flat_map(|question| &question.statements)
                .filter(|statement| statement.code.starts_with("engine."))
                .flat_map(|statement| &statement.evidence)
                .all(|reference| raw.pointer(&reference.pointer).is_some())
        );
        // The same exact receipt outside the selected run context does not acquire
        // lineage from a name or from equal numeric clock values.
        set.engine_observations.clear();
        set.contexts[0].operation = Some(OperationId {
            process_instance: [41, 42],
            sequence: 43,
        });
        set.contexts[0].run_id = Some(pse_model::generated::identities::RunId::from_bytes(
            [28; 16],
        ));
        let unrelated = result(&set, &request);
        assert!(!has(
            &unrelated,
            AnalysisQuestion::EvaluationCost,
            "engine.operator_metric"
        ));
    }
    // A direct selected-operation flat copy also appears only once when the
    // actual retained context carries its exact mirrored receipt.
    let (mut set, request) = fixture();
    let trace = CopiedExecutionObservations {
        policy: Some(CopiedObservationPolicy::Off),
        operation: Some(request.operation),
        observations: vec![],
        dropped: 0,
    };
    let mut context = crate::workflow::ReviewContextDocument::pre_handle_failure(
        Default::default(),
        Some(request.operation),
        "retained Off session".into(),
        vec![],
    )
    .unwrap();
    context.facts.push(crate::workflow::ReviewContextFact {
        path: "/physical_admission/0/engine".into(),
        codec: "engine-execution-observations.v1".into(),
        payload: serde_json::to_vec(&trace).unwrap(),
    });
    set.contexts.push(context);
    set.engine_observations.push(trace);
    let document = result(&set, &request);
    assert_eq!(
        document.questions[0]
            .statements
            .iter()
            .filter(|statement| statement.code == "engine.actual_session_policy")
            .count(),
        1
    );
    assert_eq!(
        document.questions[2]
            .statements
            .iter()
            .filter(|statement| statement.code == "engine.off")
            .count(),
        1
    );
}

#[test]
fn actual_dependency_session_joins_exact_retained_run_context_without_relabeling_or_mirror_duplication()
 {
    use crate::workflow::{ReviewContextDocument, ReviewContextFact};
    use pse_engine::session::{CopiedExecutionObservations, CopiedObservationPolicy};
    for original in [
        None,
        Some(OperationId {
            process_instance: [91, 92],
            sequence: 93,
        }),
    ] {
        let (mut set, mut request) = fixture();
        request.question = Some(AnalysisQuestion::Preparation);
        let trace = CopiedExecutionObservations {
            operation: original,
            policy: Some(CopiedObservationPolicy::Off),
            observations: vec![],
            dropped: 3,
        };
        let payload = serde_json::to_vec(&trace).unwrap();
        let mut context = ReviewContextDocument::pre_handle_failure(
            Default::default(),
            Some(request.operation),
            "actual selected dependency".into(),
            vec![],
        )
        .unwrap();
        context.facts.push(ReviewContextFact {
            path: "/physical_admission/0/engine".into(),
            codec: "engine-execution-observations.v1".into(),
            payload: payload.clone(),
        });
        set.contexts.push(context);
        set.engine_observations.push(trace);
        let document = result(&set, &request);
        let facts = document.questions[0]
            .statements
            .iter()
            .filter(|statement| statement.code == "engine.actual_session_policy")
            .collect::<Vec<_>>();
        assert_eq!(
            facts.len(),
            1,
            "flattened receipt mirror is not another session"
        );
        assert!(facts[0].message.contains(&format!("{original:?}")));
        assert_eq!(facts[0].evidence[0].member, "frozen");
        assert_eq!(facts[0].evidence[0].pointer, "/contexts/0/facts/0/payload");
        assert_eq!(set.contexts[0].facts[0].payload, payload);
        set.contexts[0].operation = Some(OperationId {
            process_instance: [81, 82],
            sequence: 83,
        });
        let unrelated = result(&set, &request);
        assert!(
            !unrelated.questions[0]
                .statements
                .iter()
                .any(|statement| statement.code == "engine.actual_session_policy")
        );
    }
}
