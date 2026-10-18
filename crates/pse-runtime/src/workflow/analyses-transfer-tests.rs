// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Real-store transfer controls over an admitted recorded partial trajectory.
#![allow(
    unsafe_code,
    reason = "finite recorded fixtures explicitly declare partial availability"
)]
use super::*;
use pse_model::generated::identities::RunId;
use pse_operations::canonical_execution::{RunRequest, SeedEligibility, TerminalClass};
use pse_relations::columnar::RelationRow;
use pse_relations::generated::{
    enums::*,
    runtime::{candidate_assessments, canonical_analysis_edges, run_lineage},
};
use std::time::Duration;

type FixtureResult<T> = Result<T, Box<dyn std::error::Error>>;

fn batch<R: RelationRow>(
    runtime: &Runtime,
    rows: Vec<R>,
) -> Result<pse_relations::columnar::FieldCheckedBatch, pse_relations::RelationError> {
    let mut builder = R::builder(
        runtime.registry(),
        rows.len(),
        runtime.validation_context().as_ref(),
    )?;
    for row in rows {
        R::push(&mut builder, row)?;
    }
    R::finish(builder)
}
async fn fixture() -> FixtureResult<(Runtime, String, String)> {
    let runtime = super::super::durable_tests::durable_runtime();
    let mut evidence = response_evidence_unit::transient();
    let mut initial = evidence.samples[0].row.clone();
    initial.sample = 0;
    initial.time = 0.;
    evidence.samples.insert(
        0,
        Located {
            row: initial,
            locator: evidence.samples[0].locator.clone(),
        },
    );
    let (run, attempt) = publish(&runtime, evidence).await?;
    Ok((runtime, run, attempt))
}
async fn publish(runtime: &Runtime, evidence: ResponseEvidence) -> FixtureResult<(String, String)> {
    let store = runtime.canonical_store()?;
    let key = format!("recorded-transfer:{}", pse_operations::mint_id::<RunId>());
    let source = store
        .edit(&key, None, &format!("source:{key}"), &[])
        .await?;
    store
        .begin_run(&RunRequest {
            key: key.clone(),
            revision: source,
            sources: vec![],
            request: vec![1],
            source_selection: vec![2],
            attestation: vec![3],
        })
        .await?;
    let fence = store
        .claim_run(
            &key,
            &format!("claim:{key}"),
            "worker",
            Duration::from_secs(60),
        )
        .await?;
    let context = &evidence.contexts[0].row;
    let assessment = candidate_assessments::Row {
        run_id: context.run_id,
        step: 0,
        native_termination: None,
        numerically_feasible: None,
        closure: ClosureAssessment::Unclosed,
        policy: ClosurePolicy::AllowUnclosed,
        usability: CandidateUse::QualifiedUnclosed,
        reason: "recorded partial fixture".into(),
        incumbent_policy: IncumbentPolicy::Refuse,
        candidate_kind: None,
        qualification: Some(NativeQualification::Unqualified),
        validated: None,
        bound_origin: None,
        bound: None,
        absolute_gap: None,
        relative_gap: None,
        qualifiers: vec![],
        refusals: vec![],
        permits_result: true,
        permits_seed: false,
    };
    let lineage = run_lineage::Row {
        run_id: context.run_id,
        step: 0,
        model_id: context.model_id,
        revision: context.source_identity,
        case_id: context.case_id,
        instance_id: context.instance_id,
        fit_id: context.fit_id,
        request_identity: context.source_identity,
        preparation_identity: context.preparation_identity,
        profile_identity: context.profile_identity,
        numerical_identity: context.profile_identity,
        physical_identity: context.physical_identity,
        environment_identity: context.source_identity,
    };
    let batches = vec![
        batch(
            runtime,
            evidence.contexts.into_iter().map(|v| v.row).collect(),
        )?,
        batch(
            runtime,
            evidence.computations.into_iter().map(|v| v.row).collect(),
        )?,
        batch(
            runtime,
            evidence.endpoints.into_iter().map(|v| v.row).collect(),
        )?,
        batch(
            runtime,
            evidence.samples.into_iter().map(|v| v.row).collect(),
        )?,
        batch(
            runtime,
            evidence.responses.into_iter().map(|v| v.row).collect(),
        )?,
        batch(
            runtime,
            evidence.events.into_iter().map(|v| v.row).collect(),
        )?,
        batch(runtime, vec![assessment])?,
        batch(runtime, vec![lineage])?,
    ];
    let pool = runtime.shared.pool();
    let acks = super::super::durable::PublicationAcks::new(&pool);
    let mut queue =
        super::super::result_projection::ResultAppendQueue::new(store, &fence, &pool, &acks);
    for checked in batches {
        let relation = checked.relation_id();
        let cursor = super::super::ResultCursor::from_retained(
            checked.batch().schema(),
            relation,
            std::slice::from_ref(&checked),
        )?;
        super::super::result_projection::store_recorded_table_deferred(
            &mut queue, relation, cursor,
        )
        .await?;
    }
    queue.flush().await?;
    let closed = store
        .close_result_ingestion(&fence, &format!("close:{}", fence.attempt()))
        .await?;
    let manifest = store.reconcile_closed_attempt(&closed).await?;
    // SAFETY: only recorded finite evidence is published; no completion or derivative certification is asserted.
    unsafe {
        store
            .seal_attempt(
                &manifest,
                &format!("seal:{}", fence.attempt()),
                TerminalClass::Partial,
                &[1],
                SeedEligibility::Unavailable,
            )
            .await?;
    }
    Ok((fence.run().to_owned(), fence.attempt().to_owned()))
}
fn controls() -> AnalysisControls {
    AnalysisControls {
        roots: vec![],
        direction: AnalysisDirection::Downstream,
    }
}

#[tokio::test]
async fn canonical_acknowledged_analysis_facts_match_independent_artifact_after_retirement() {
    let (runtime, run, attempt) = fixture().await.expect("publish recorded transfer fixture");
    let activity = runtime.admit_activity().unwrap();
    let operation = activity.0.observation.as_ref().unwrap().reader();
    let analysis = activity
        .scope(runtime.result_analysis(&run, &attempt, &controls(), CancellationToken::new()))
        .await
        .unwrap();
    let recorded = operation.snapshot(Duration::from_secs(60));
    drop(activity);
    let count = |name: &str| {
        recorded
            .metrics
            .iter()
            .filter(|metric| metric.name == name)
            .map(|metric| match &metric.cumulative {
                pse_diagnostics::observation::Value::Count(value) => *value,
                other => panic!("additive transfer counter required: {other:?}"),
            })
            .sum::<u64>()
    };
    assert_eq!(count("canonical.artifact.scientific_selection_requests"), 1);
    assert_eq!(count("canonical.artifact.scientific_source_requests"), 1);
    assert_eq!(count("canonical.artifact.scientific_encodes"), 8);
    assert_eq!(
        count("canonical.result.native_decodes"),
        8,
        "dispatch and promotion share the two admitted family members"
    );
    assert_eq!(
        count("canonical.result.standalone_decodes"),
        0,
        "in-process graph construction consumes acknowledged facts without reopening member bytes"
    );
    let edges = canonical_analysis_edges::Row::rows(&analysis.edges(None).await.unwrap()).unwrap();
    let response = edges
        .iter()
        .find(|edge| edge.kind == "recorded_transient_response")
        .unwrap();
    assert_eq!(response.evidence_field.as_deref(), Some("value"));
    assert_eq!(response.evidence_row, Some(0));
    assert!(!edges.iter().any(|edge| edge.kind.contains("parametric")));
    let artifact = runtime
        .scientific_artifact(response.evidence_artifact.as_deref().unwrap())
        .await
        .unwrap();
    pse_operations::testing::drain_result_releases(runtime.canonical_store().unwrap())
        .await
        .unwrap();
    runtime
        .canonical_store()
        .unwrap()
        .forget_run_results(&run)
        .await
        .unwrap();
    let reopened = runtime.analysis(analysis.key()).await.unwrap();
    assert_eq!(
        canonical_analysis_edges::Row::rows(&reopened.edges(None).await.unwrap()).unwrap(),
        edges
    );
    let original = artifact
        .evidence(
            &ArtifactLocator {
                artifact: artifact.header().key.clone(),
                block: response.evidence_block.unwrap(),
                row: response.evidence_row,
                field: response.evidence_field.clone(),
            },
            CancellationToken::new(),
        )
        .await
        .unwrap();
    let rows =
        pse_relations::generated::runtime::response_sensitivities::Row::rows(&original).unwrap();
    assert_eq!(
        rows,
        vec![response_evidence_unit::transient().responses[0].row.clone()]
    );
}

#[tokio::test]
async fn canonical_transfer_cancellation_and_lost_ack_preserve_original_settlement() {
    let (runtime, run, attempt) = fixture().await.expect("publish recorded transfer fixture");
    let before = CANCEL_TRANSFER_AT
        .scope(
            TransferCheckpoint::BeforeDispatch,
            runtime.result_analysis(&run, &attempt, &controls(), CancellationToken::new()),
        )
        .await;
    assert!(before.is_err());
    for phase in [
        TransferCheckpoint::BeforeArtifactRead,
        TransferCheckpoint::BeforeGraph,
        TransferCheckpoint::AfterGraphAck,
    ] {
        let result = CANCEL_TRANSFER_AT
            .scope(
                phase,
                runtime.result_analysis(&run, &attempt, &controls(), CancellationToken::new()),
            )
            .await;
        settle(&runtime, result, phase == TransferCheckpoint::AfterGraphAck)
            .await
            .expect("recover cancelled transfer with its original settlement");
    }
    let lost = super::super::artifacts::SCIENTIFIC_HANDOFF_REPLY_LOSS
        .scope(
            true,
            runtime.result_analysis(&run, &attempt, &controls(), CancellationToken::new()),
        )
        .await;
    settle(&runtime, lost, false)
        .await
        .expect("recover lost scientific handoff ACK");
}
async fn settle(
    runtime: &Runtime,
    result: Result<AnalysisHandle, WorkflowError>,
    graph_acked: bool,
) -> FixtureResult<()> {
    let error = match result {
        Err(error) => error,
        Ok(_) => {
            return Err(
                "expected an unsettled artifact transfer, got an acknowledged analysis".into(),
            );
        }
    };
    let WorkflowError::ArtifactTransferUnsettled {
        analysis,
        artifact,
        owner_kind,
        owner,
        ..
    } = error
    else {
        return Err(error.into());
    };
    assert_eq!(analysis.is_some(), graph_acked);
    let recovery = runtime.recover_artifact_intent(&serde_json::to_vec(&artifact)?)?;
    let reopened = runtime.scientific_artifact(&artifact.key).await?;
    assert_eq!(reopened.header().key, artifact.key);
    recovery.release(&owner_kind, &owner).await?;
    recovery.release(&owner_kind, &owner).await?;
    if let Some(key) = analysis {
        assert!(
            runtime
                .canonical_store()?
                .analysis(&key)
                .await?
                .active
        );
        assert!(
            runtime
                .analysis(&key)
                .await?
                .edges(None)
                .await?
                .batch()
                .num_rows()
                > 0
        );
        assert!(
            runtime.scientific_artifact(&artifact.key).await.is_ok(),
            "graph retains independent artifact after staging release"
        );
    }
    Ok(())
}

#[tokio::test]
async fn canonical_final_graph_ack_loss_retains_exact_intent_and_independent_evidence() {
    let (runtime, run, attempt) = fixture().await.expect("publish recorded transfer fixture");
    let result = GRAPH_HANDOFF_REPLY_LOSS
        .scope(
            true,
            runtime.result_analysis(&run, &attempt, &controls(), CancellationToken::new()),
        )
        .await;
    let Err(WorkflowError::ArtifactTransferUnsettled {
        analysis: Some(key),
        source,
        ..
    }) = &result
    else {
        panic!("final graph ACK loss must preserve both occurrences: {result:?}");
    };
    let WorkflowError::Canonical(pse_operations::canonical::CanonicalError::AnalysisUnsettled {
        intent,
        ..
    }) = source.as_ref()
    else {
        panic!("original graph intent is required: {source:?}");
    };
    assert_eq!(&intent.key, key);
    assert!(
        !intent.active,
        "original request header is never rewritten as the ACK"
    );
    let actual = runtime
        .canonical_store()
        .unwrap()
        .analysis(key)
        .await
        .unwrap();
    check_analysis_ack(&runtime.shared.pool(), intent, &actual).unwrap();
    settle(&runtime, result, true)
        .await
        .expect("recover lost final graph ACK");
}

#[tokio::test]
async fn canonical_member_transfer_cancellation_keeps_unacknowledged_facts_out_of_graph() {
    let (runtime, run, attempt) = fixture().await.expect("publish recorded transfer fixture");
    let cancel = CancellationToken::new();
    let interrupted = cancel.clone();
    let result = pse_operations::testing::cancel_after_artifact_append_ack(
        move || interrupted.cancel(),
        runtime.result_analysis(&run, &attempt, &controls(), cancel.clone()),
    )
    .await;
    assert!(cancel.is_cancelled());
    let Err(WorkflowError::ArtifactTransferUnsettled {
        analysis: None,
        artifact,
        owner_kind,
        owner,
        read: None,
        ..
    }) = &result
    else {
        panic!("member cancellation must retain exact original transfer intent: {result:?}");
    };
    assert!(!artifact.active);
    assert!(
        runtime.scientific_artifact(&artifact.key).await.is_err(),
        "an attached member group is not acknowledged active scientific content"
    );
    let mut source = runtime
        .results(
            &run,
            &attempt,
            "runtime.scientific_contexts",
            0,
            u64::MAX,
            CancellationToken::new(),
        )
        .await
        .unwrap();
    assert_eq!(
        source.next_batch().await.unwrap().unwrap().num_rows(),
        1,
        "the exact protected producer remains readable after interrupted transfer"
    );
    drop(source);
    pse_operations::testing::drain_result_releases(runtime.canonical_store().unwrap())
        .await
        .unwrap();
    assert!(
        runtime
            .canonical_store()
            .unwrap()
            .forget_run_results(&run)
            .await
            .is_err(),
        "original inactive transfer staging still protects producer input"
    );
    let recovery = runtime
        .recover_artifact_intent(&serde_json::to_vec(artifact).unwrap())
        .unwrap();
    recovery.release(owner_kind, owner).await.unwrap();
    recovery.release(owner_kind, owner).await.unwrap();
    assert!(
        !recovery.collect().await.unwrap(),
        "original creation window remains unsettled"
    );
}

// This is recorded storage/ownership evidence. It does not execute or certify a
// differential model; the native scientific journeys own that qualification.
fn recorded_shape(samples: usize, directions: usize) -> ResponseEvidence {
    let mut evidence = response_evidence_unit::transient();
    let coordinate = evidence.contexts[0].row.state[0].clone();
    let symbols = (20..100)
        .map(|byte| {
            let mut value = coordinate.clone();
            value.id = pse_ids::SemanticId::from_bytes([byte; 16]);
            value
        })
        .collect::<Vec<_>>();
    let inputs = (0..directions)
        .map(|n| {
            let mut value = coordinate.clone();
            value.id = pse_ids::SemanticId::from_bytes([110 + n as u8; 16]);
            value
        })
        .collect::<Vec<_>>();
    evidence.contexts[0].row.state = symbols.clone();
    evidence.contexts[0].row.outputs = symbols.clone();
    evidence.contexts[0].row.inputs = inputs.clone();
    evidence.endpoints[0].row.state_ids = symbols.iter().map(|v| v.id).collect();
    evidence.endpoints[0].row.output_ids = symbols.iter().map(|v| v.id).collect();
    evidence.endpoints[0].row.input_ids = inputs.iter().map(|v| v.id).collect();
    evidence.endpoints[0].row.state = vec![2.; 80];
    evidence.endpoints[0].row.outputs = vec![2.; 80];
    evidence.endpoints[0].row.inputs = vec![2.; directions];
    evidence.endpoints[0].row.input_columns = (0..directions as i64).collect();
    evidence.events[0].row.symbol_id = symbols[0].id;
    evidence.events[0].row.before = 2.;
    evidence.computations[0].row.completed_samples = Some(samples as i64);
    let sample_template = evidence.samples[0].clone();
    let response_template = evidence.responses[0].clone();
    evidence.samples.clear();
    evidence.responses.clear();
    for symbol in symbols {
        for sample in 0..samples {
            let time = sample as f64 / (samples - 1) as f64;
            let mut value = sample_template.clone();
            value.row.symbol_id = symbol.id;
            value.row.sample = sample as i64;
            value.row.time = time;
            value.row.value = 1. + time;
            evidence.samples.push(value);
            for (direction, input) in inputs.iter().enumerate() {
                let mut value = response_template.clone();
                value.row.output_id = symbol.id;
                value.row.sample = sample as i64;
                value.row.time = Some(time);
                value.row.parameter_id = input.id;
                value.row.parameter_column = direction as i64;
                value.row.value = time * (direction + 1) as f64;
                evidence.responses.push(value);
            }
        }
    }
    evidence
}
fn count(snapshot: &pse_diagnostics::observation::OperationSnapshot, name: &str) -> FixtureResult<u64> {
    assert!(
        snapshot.coverage.enabled && snapshot.coverage.available,
        "actual counter owner must be available: {:?}",
        snapshot.coverage
    );
    assert_eq!(snapshot.coverage.dropped_updates, 0);
    assert_eq!(snapshot.coverage.refused_items, 0);
    assert!(!snapshot.coverage.counter_overflow);
    assert!(!snapshot.coverage.curtailed);
    snapshot
        .metrics
        .iter()
        .filter(|metric| metric.name == name)
        .try_fold(0, |total, metric| match &metric.cumulative {
            pse_diagnostics::observation::Value::Count(value) => Ok(total + value),
            other => Err(format!("additive work counter required for {name}: {other:?}").into()),
        })
}
#[derive(Debug)]
struct ReadPoint {
    pool_reserved: usize,
    queries: u64,
    block_pages: u64,
    output_pages: u64,
    descriptors: u64,
    decodes: u64,
    decoded_bytes: u64,
}
fn point(
    runtime: &Runtime,
    reader: &pse_diagnostics::observation::OperationReader,
    full: bool,
) -> FixtureResult<ReadPoint> {
    let snapshot = reader.snapshot(Duration::from_secs(60));
    // Full-reader construction owns a continuation child; stream polling runs
    // in this caller's activity. Read both actual diagnostic owners once.
    let linked = snapshot
        .children
        .iter()
        .filter(|child| child.kind == "result.reopen_and_access")
        .collect::<Vec<_>>();
    assert_eq!(
        linked.len(),
        usize::from(full),
        "one exact full-reader continuation; selected readers use this activity"
    );
    let access = linked
        .into_iter()
        .map(|child| {
            assert_ne!(child.id, snapshot.id);
            assert_eq!(
                child.relationship,
                pse_diagnostics::observation::Relationship::Continuation
            );
            assert!(
                child.available,
                "linked constructor owner is not optional evidence"
            );
            let actual = runtime
                .observations()
                .reader(child.id)
                .ok_or_else(|| {
                    format!("linked constructor observation {} is unavailable", child.id)
                })?
                .snapshot(Duration::from_secs(60));
            assert_eq!(actual.id, child.id);
            Ok(actual)
        })
        .collect::<FixtureResult<Vec<_>>>()?;
    let work = |name: &str| -> FixtureResult<u64> {
        access.iter().try_fold(count(&snapshot, name)?, |total, child| {
            Ok(total + count(child, name)?)
        })
    };
    Ok(ReadPoint {
        pool_reserved: runtime.shared.pool().reserved(),
        queries: work("canonical.query.builders")?,
        block_pages: work("canonical.result.block_page_requests")?,
        output_pages: work("canonical.result.output_page_requests")?,
        descriptors: work("canonical.result.descriptor_page_requests")?,
        decodes: work("canonical.result.native_decodes")?,
        decoded_bytes: work("canonical.result.decoded_array_bytes")?,
    })
}
#[derive(Debug)]
struct ReadGrowth {
    constructor: ReadPoint,
    first: ReadPoint,
    eof: ReadPoint,
    released: usize,
    rows: usize,
    elapsed: [Duration; 3],
}
impl ReadGrowth {
    fn pool_stages(&self) -> [usize; 4] {
        [
            self.constructor.pool_reserved,
            self.first.pool_reserved,
            self.eof.pool_reserved,
            self.released,
        ]
    }
}
async fn all_edges(analysis: &AnalysisHandle) -> FixtureResult<Vec<canonical_analysis_edges::Row>> {
    let mut rows = Vec::new();
    let mut after: Option<String> = None;
    loop {
        let page = canonical_analysis_edges::Row::rows(&analysis.edges(after.as_deref()).await?)?;
        assert!(page.len() <= 64);
        if page.is_empty() {
            break;
        }
        for row in &page {
            assert!(
                after.as_ref().is_none_or(|previous| previous < &row.key),
                "native graph cursor must advance strictly"
            );
            after = Some(row.key.clone());
        }
        rows.extend(page);
        assert!(
            rows.len() <= ANALYSIS_EDGES,
            "complete graph enumeration preserves existing bound"
        );
    }
    Ok(rows)
}
enum StoredReader {
    Full(super::super::CanonicalResultReader),
    Selected(super::super::SelectedResultReader),
}
impl StoredReader {
    async fn next(
        &mut self,
    ) -> Result<Option<pse_relations::columnar::FieldCheckedBatch>, WorkflowError> {
        match self {
            Self::Full(reader) => reader.next_relation_batch().await,
            Self::Selected(reader) => reader.next_relation_batch().await,
        }
    }
}
async fn read_growth(
    runtime: &Runtime,
    run: &str,
    attempt: &str,
    output: Option<pse_ids::SemanticId>,
    expected: &[pse_relations::generated::runtime::simulation_samples::Row],
) -> FixtureResult<ReadGrowth> {
    use pse_relations::generated::runtime::simulation_samples;
    let activity = runtime.admit_activity()?;
    let observation = activity
        .0
        .observation
        .as_ref()
        .ok_or("read growth requires an admitted activity observation")?
        .reader();
    let started = std::time::Instant::now();
    let (constructor, first, eof, rows, elapsed) = activity
        .scope(async {
            let mut reader = match output {
                None => StoredReader::Full(
                    runtime
                        .results(
                            run,
                            attempt,
                            "runtime.simulation_samples",
                            0,
                            u64::MAX,
                            CancellationToken::new(),
                        )
                        .await?,
                ),
                Some(output) => StoredReader::Selected(
                    runtime
                        .output_results(
                            run,
                            attempt,
                            "runtime.simulation_samples",
                            output,
                            "value",
                            "0",
                            0,
                            u64::MAX,
                            None,
                            None,
                            None,
                            CancellationToken::new(),
                        )
                        .await?,
                ),
            };
            let constructor = point(runtime, &observation, output.is_none())?;
            let constructor_elapsed = started.elapsed();
            assert_eq!(constructor.decodes, 0);
            let first_batch = reader.next().await?;
            let first = point(runtime, &observation, output.is_none())?;
            let first_elapsed = started.elapsed();
            let mut actual = Vec::new();
            if let Some(batch) = first_batch {
                actual.extend(simulation_samples::Row::rows(&batch)?);
            }
            while let Some(batch) = reader.next().await? {
                actual.extend(simulation_samples::Row::rows(&batch)?);
            }
            let eof = point(runtime, &observation, output.is_none())?;
            let eof_elapsed = started.elapsed();
            assert_eq!(actual, expected);
            if output.is_some() {
                assert_eq!(constructor.block_pages, 0);
                assert_eq!(eof.block_pages, 0);
            } else {
                assert!(constructor.block_pages > 0);
            }
            if expected.is_empty() {
                assert_eq!(first.decodes, 0);
                assert_eq!(eof.decodes, 0);
                assert_eq!(eof.descriptors, 0);
            } else {
                assert!(first.decodes > 0);
                assert!(eof.decoded_bytes > 0);
            }
            drop(reader);
            Ok::<_, Box<dyn std::error::Error>>((
                constructor,
                first,
                eof,
                actual.len(),
                [constructor_elapsed, first_elapsed, eof_elapsed],
            ))
        })
        .await?;
    drop(activity);
    pse_operations::testing::drain_result_releases(runtime.canonical_store()?).await?;
    Ok(ReadGrowth {
        constructor,
        first,
        eof,
        released: runtime.shared.pool().reserved(),
        rows,
        elapsed,
    })
}

#[tokio::test]
#[allow(
    clippy::print_stderr,
    reason = "bounded phase observations distinguish ordinary publication from read and analysis work without timing gates"
)]
async fn canonical_recorded_result_growth_covers_full_selected_absent_creation_and_reopen() {
    use pse_relations::generated::runtime::response_sensitivities;
    let runtime = super::super::durable_tests::durable_runtime();
    for (samples, directions) in [(5, 1), (5, 4), (25, 1), (25, 4)] {
        let evidence = recorded_shape(samples, directions);
        let expected_samples = evidence
            .samples
            .iter()
            .map(|v| v.row.clone())
            .collect::<Vec<_>>();
        let expected_responses = evidence
            .responses
            .iter()
            .map(|v| v.row.clone())
            .collect::<Vec<_>>();
        let selected = expected_samples[0].symbol_id;
        let selected_rows = expected_samples
            .iter()
            .filter(|v| v.symbol_id == selected)
            .cloned()
            .collect::<Vec<_>>();
        let started = std::time::Instant::now();
        let activity = runtime.admit_activity().unwrap();
        let observation = activity.0.observation.as_ref().unwrap().reader();
        let (run, attempt) = activity
            .scope(publish(&runtime, evidence))
            .await
            .expect("publish recorded growth fixture");
        let setup = observation.snapshot(Duration::from_secs(60));
        drop(activity);
        eprintln!(
            "recorded80x{samples}x{directions} setup: elapsed={:?},queries={}",
            started.elapsed(),
            count(&setup, "canonical.query.builders").expect("read additive work counter")
        );
        let full = read_growth(&runtime, &run, &attempt, None, &expected_samples)
            .await
            .expect("read full recorded trajectory");
        let selected = read_growth(&runtime, &run, &attempt, Some(selected), &selected_rows)
            .await
            .expect("read selected recorded trajectory");
        let absent = read_growth(
            &runtime,
            &run,
            &attempt,
            Some(pse_ids::SemanticId::from_bytes([250; 16])),
            &[],
        )
        .await
        .expect("read absent recorded trajectory selection");
        eprintln!(
            "recorded80x{samples}x{directions} reads: full_elapsed={:?},queries={}; selected_elapsed={:?},queries={}; absent_elapsed={:?},queries={}",
            full.elapsed,
            full.eof.queries,
            selected.elapsed,
            selected.eof.queries,
            absent.elapsed,
            absent.eof.queries
        );
        assert_eq!(full.rows, 80 * samples);
        assert_eq!(selected.rows, samples);
        assert_eq!(absent.rows, 0);
        assert!(selected.eof.decodes <= full.eof.decodes);
        assert!(selected.eof.output_pages > 0);
        assert!(absent.eof.output_pages > 0);
        let activity = runtime.admit_activity().unwrap();
        let observation = activity.0.observation.as_ref().unwrap().reader();
        let started = std::time::Instant::now();
        let created = activity
            .scope(runtime.result_analysis(&run, &attempt, &controls(), CancellationToken::new()))
            .await;
        let creation = observation.snapshot(Duration::from_secs(60));
        drop(activity);
        eprintln!(
            "recorded80x{samples}x{directions} creation: elapsed={:?},queries={},decode={},encode={}",
            started.elapsed(),
            count(&creation, "canonical.query.builders").expect("read additive work counter"),
            count(&creation, "canonical.result.native_decodes")
                .expect("read additive work counter"),
            count(&creation, "canonical.artifact.scientific_encodes")
                .expect("read additive work counter")
        );
        assert_eq!(
            count(
                &creation,
                "canonical.artifact.scientific_selection_requests"
            )
            .expect("read additive work counter"),
            1
        );
        assert_eq!(
            count(&creation, "canonical.artifact.scientific_source_requests")
                .expect("read additive work counter"),
            1
        );
        assert_eq!(
            count(&creation, "canonical.result.standalone_decodes")
                .expect("read additive work counter"),
            0
        );
        if samples == 25 && directions == 4 {
            assert!(
                matches!(created,Err(WorkflowError::Input(ref detail)) if detail.contains("analysis evidence relation exceeds bound")),
                "8000 responses exceed existing4096 evidence cap before a graph that would also exceed8192 edges: {created:?}"
            );
            // Public scientific retention keeps its own coverage; the private
            // analysis fact demand must not silently cap this recorded product.
            let retained = runtime
                .promote_scientific_artifact(
                    &run,
                    &attempt,
                    super::super::artifacts::ScientificArtifactKind::Trajectory,
                    "audit",
                    "recorded-large-coverage",
                    CancellationToken::new(),
                )
                .await
                .unwrap();
            assert_eq!(retained.header().kind, "trajectory");
            let read = runtime
                .canonical_store()
                .unwrap()
                .read_artifact(&retained.header().key)
                .await
                .unwrap();
            let mut retained_responses = 0;
            for ordinal in 0..retained.header().block_count {
                let block = read.block(ordinal).await.unwrap();
                if block.relation == "runtime.response_sensitivities" {
                    retained_responses += block.rows;
                }
            }
            read.release().await.unwrap();
            assert_eq!(retained_responses, 8000);
        } else {
            let analysis = created.unwrap();
            let started = std::time::Instant::now();
            let activity = runtime.admit_activity().unwrap();
            let observation = activity.0.observation.as_ref().unwrap().reader();
            let edges = activity
                .scope(all_edges(&analysis))
                .await
                .expect("enumerate all created analysis edges");
            let enumeration = observation.snapshot(Duration::from_secs(60));
            drop(activity);
            eprintln!(
                "recorded80x{samples}x{directions} complete_edges: elapsed={:?},queries={},rows={}",
                started.elapsed(),
                count(&enumeration, "canonical.query.builders").expect("read additive work counter"),
                edges.len()
            );
            let locators = edges
                .iter()
                .filter(|v| v.kind == "recorded_transient_response")
                .map(|v| {
                    (
                        v.evidence_artifact.clone().unwrap(),
                        v.evidence_block.unwrap(),
                        v.evidence_row.unwrap(),
                        v.evidence_field.clone().unwrap(),
                    )
                })
                .collect::<BTreeSet<_>>();
            assert_eq!(locators.len(), 80 * samples * directions);
            assert!(edges.iter().all(|v| !v.kind.contains("parametric")));
            let artifact_key = &locators.first().unwrap().0;
            let artifact = runtime.scientific_artifact(artifact_key).await.unwrap();
            assert_eq!(
                count(&creation, "canonical.result.native_decodes")
                    .expect("read additive work counter"),
                artifact.header().block_count
            );
            assert_eq!(
                count(&creation, "canonical.artifact.scientific_encodes")
                    .expect("read additive work counter"),
                artifact.header().block_count
            );
            let read = runtime
                .canonical_store()
                .unwrap()
                .read_artifact(artifact_key)
                .await
                .unwrap();
            let mut expected_locators = BTreeSet::new();
            let mut original_rows = Vec::new();
            for ordinal in 0..artifact.header().block_count {
                let block = read.block(ordinal).await.unwrap();
                if block.relation == "runtime.response_sensitivities" {
                    for row in 0..block.rows {
                        expected_locators.insert((
                            artifact_key.clone(),
                            ordinal,
                            row,
                            "value".to_owned(),
                        ));
                    }
                    original_rows.extend(
                        response_sensitivities::Row::rows(
                            &artifact
                                .block(ordinal, CancellationToken::new())
                                .await
                                .unwrap(),
                        )
                        .unwrap(),
                    );
                }
            }
            read.release().await.unwrap();
            assert_eq!(original_rows, expected_responses);
            assert_eq!(locators, expected_locators);
            let activity = runtime.admit_activity().unwrap();
            let observation = activity.0.observation.as_ref().unwrap().reader();
            let started = std::time::Instant::now();
            let reopened = activity
                .scope(async {
                    let handle = runtime.analysis(analysis.key()).await.unwrap();
                    all_edges(&handle).await
                })
                .await
                .expect("enumerate all reopened analysis edges");
            let reused = observation.snapshot(Duration::from_secs(60));
            drop(activity);
            eprintln!(
                "recorded80x{samples}x{directions} reopen_with_complete_edges: elapsed={:?},queries={}",
                started.elapsed(),
                count(&reused, "canonical.query.builders").expect("read additive work counter")
            );
            assert_eq!(reopened, edges);
            for metric in [
                "canonical.artifact.scientific_selection_requests",
                "canonical.artifact.scientific_source_requests",
                "canonical.result.native_decodes",
                "canonical.artifact.scientific_encodes",
            ] {
                assert_eq!(count(&reused, metric).expect("read additive work counter"), 0);
            }
        }
        // Pool reservations are actual participating allocations, not RSS or a
        // new capacity gate. The production native journeys establish science.
        eprintln!(
            "recorded80x{samples}x{directions}: pool_constructor_first_eof_released full={:?},selected={:?},absent={:?}; full={full:?}; selected={selected:?}; absent={absent:?}; creation_decode={},creation_encode={}",
            full.pool_stages(),
            selected.pool_stages(),
            absent.pool_stages(),
            count(&creation, "canonical.result.native_decodes")
                .expect("read additive work counter"),
            count(&creation, "canonical.artifact.scientific_encodes")
                .expect("read additive work counter")
        );
    }
}
