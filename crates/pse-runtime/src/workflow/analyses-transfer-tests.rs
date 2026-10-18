// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Real-store transfer controls over an admitted recorded partial trajectory.
#![allow(unsafe_code, reason = "finite recorded fixtures explicitly declare partial availability")]
use super::*;
use pse_operations::canonical_execution::{RunRequest, TerminalClass, SeedEligibility};
use pse_relations::columnar::RelationRow;
use pse_model::generated::identities::RunId;
use pse_relations::generated::{enums::*, runtime::{candidate_assessments, run_lineage, canonical_analysis_edges}};
use std::time::Duration;

fn batch<R: RelationRow>(runtime: &Runtime, rows: Vec<R>) -> pse_relations::columnar::FieldCheckedBatch {
    let mut builder = R::builder(runtime.registry(), rows.len(), runtime.validation_context().as_ref()).unwrap();
    for row in rows { R::push(&mut builder, row).unwrap(); }
    R::finish(builder).unwrap()
}
async fn fixture() -> (Runtime, String, String) {
    let runtime = super::super::durable_tests::durable_runtime();
    let store = runtime.canonical_store().unwrap();
    let key = format!("recorded-transfer:{}", pse_operations::mint_id::<RunId>());
    let source = store.edit(&key, None, "source", &[]).await.unwrap();
    store.begin_run(&RunRequest { key: key.clone(), revision: source, sources: vec![], request: vec![1], source_selection: vec![2], attestation: vec![3] }).await.unwrap();
    let fence = store.claim_run(&key, "claim", "worker", Duration::from_secs(60)).await.unwrap();
    let evidence = response_evidence_unit::transient();
    let context = &evidence.contexts[0].row;
    let assessment = candidate_assessments::Row {
        run_id: context.run_id, step: 0, native_termination: None, numerically_feasible: None,
        closure: ClosureAssessment::Unclosed, policy: ClosurePolicy::AllowUnclosed,
        usability: CandidateUse::QualifiedUnclosed, reason: "recorded partial fixture".into(),
        incumbent_policy: IncumbentPolicy::Refuse, candidate_kind: None,
        qualification: Some(NativeQualification::Unqualified), validated: None,
        bound_origin: None, bound: None, absolute_gap: None, relative_gap: None,
        qualifiers: vec![], refusals: vec![], permits_result: true, permits_seed: false,
    };
    let lineage = run_lineage::Row {
        run_id: context.run_id, step: 0, model_id: context.model_id,
        revision: context.source_identity, case_id: context.case_id, instance_id: context.instance_id,
        fit_id: context.fit_id, request_identity: context.source_identity,
        preparation_identity: context.preparation_identity, profile_identity: context.profile_identity,
        numerical_identity: context.profile_identity, physical_identity: context.physical_identity,
        environment_identity: context.source_identity,
    };
    let mut samples = evidence.samples.iter().map(|v| v.row.clone()).collect::<Vec<_>>();
    let mut initial = samples[0].clone(); initial.sample = 0; initial.time = 0.; samples.insert(0, initial);
    let batches = vec![
        batch(&runtime, evidence.contexts.into_iter().map(|v| v.row).collect()),
        batch(&runtime, evidence.computations.into_iter().map(|v| v.row).collect()),
        batch(&runtime, evidence.endpoints.into_iter().map(|v| v.row).collect()),
        batch(&runtime, samples),
        batch(&runtime, evidence.responses.into_iter().map(|v| v.row).collect()),
        batch(&runtime, evidence.events.into_iter().map(|v| v.row).collect()),
        batch(&runtime, vec![assessment]), batch(&runtime, vec![lineage]),
    ];
    let pool = runtime.shared.pool();
    let acks = super::super::durable::PublicationAcks::new(&pool);
    for checked in batches {
        let relation = checked.relation_id();
        let cursor = super::super::ResultCursor::from_retained(checked.batch().schema(), relation, std::slice::from_ref(&checked)).unwrap();
        super::super::result_projection::store_result_table(store, &fence, relation, cursor, &pool, &acks).await.unwrap();
    }
    let closed = store.close_result_ingestion(&fence, "close").await.unwrap();
    let manifest = store.reconcile_closed_attempt(&closed).await.unwrap();
    // SAFETY: only recorded finite evidence is published; no completion or derivative certification is asserted.
    unsafe { store.seal_attempt(&manifest, "seal", TerminalClass::Partial, &[1], SeedEligibility::Unavailable).await.unwrap(); }
    (runtime, fence.run().to_owned(), fence.attempt().to_owned())
}
fn controls() -> AnalysisControls { AnalysisControls { roots: vec![], direction: AnalysisDirection::Downstream } }

#[tokio::test]
async fn canonical_acknowledged_analysis_facts_match_independent_artifact_after_retirement() {
    let (runtime, run, attempt) = fixture().await;
    let activity = runtime.admit_activity().unwrap();
    let operation = activity.0.observation.as_ref().unwrap().reader();
    let analysis = activity.scope(
        runtime.result_analysis(&run, &attempt, &controls(), CancellationToken::new())).await.unwrap();
    let recorded = operation.snapshot(Duration::from_secs(60));
    drop(activity);
    let count = |name: &str| recorded.metrics.iter().filter(|metric| metric.name == name)
        .map(|metric| match &metric.cumulative { pse_diagnostics::observation::Value::Count(value) => *value,
            other => panic!("additive transfer counter required: {other:?}") }).sum::<u64>();
    assert_eq!(count("canonical.artifact.scientific_selection_requests"), 1);
    assert_eq!(count("canonical.artifact.scientific_source_requests"), 1);
    assert_eq!(count("canonical.artifact.scientific_encodes"), 8);
    assert_eq!(count("canonical.result.native_decodes"), 8,
        "dispatch and promotion share the two admitted family members");
    assert_eq!(count("canonical.result.standalone_decodes"), 0,
        "in-process graph construction consumes acknowledged facts without reopening member bytes");
    let edges = canonical_analysis_edges::Row::rows(&analysis.edges(None).await.unwrap()).unwrap();
    let response = edges.iter().find(|edge| edge.kind == "recorded_transient_response").unwrap();
    assert_eq!(response.evidence_field.as_deref(), Some("value"));
    assert_eq!(response.evidence_row, Some(0));
    assert!(!edges.iter().any(|edge| edge.kind.contains("parametric")));
    let artifact = runtime.scientific_artifact(response.evidence_artifact.as_deref().unwrap()).await.unwrap();
    pse_operations::testing::drain_result_releases(runtime.canonical_store().unwrap()).await.unwrap();
    runtime.canonical_store().unwrap().forget_run_results(&run).await.unwrap();
    let reopened = runtime.analysis(analysis.key()).await.unwrap();
    assert_eq!(canonical_analysis_edges::Row::rows(&reopened.edges(None).await.unwrap()).unwrap(), edges);
    let original = artifact.evidence(&ArtifactLocator {artifact: artifact.header().key.clone(), block: response.evidence_block.unwrap(), row: response.evidence_row, field: response.evidence_field.clone()}, CancellationToken::new()).await.unwrap();
    let rows = pse_relations::generated::runtime::response_sensitivities::Row::rows(&original).unwrap();
    assert_eq!(rows, vec![response_evidence_unit::transient().responses[0].row.clone()]);
}

#[tokio::test]
async fn canonical_transfer_cancellation_and_lost_ack_preserve_original_settlement() {
    let (runtime, run, attempt) = fixture().await;
    let before = CANCEL_TRANSFER_AT.scope(TransferCheckpoint::BeforeDispatch, runtime.result_analysis(&run, &attempt, &controls(), CancellationToken::new())).await;
    assert!(before.is_err());
    for phase in [TransferCheckpoint::BeforeArtifactRead, TransferCheckpoint::BeforeGraph, TransferCheckpoint::AfterGraphAck] {
        let result = CANCEL_TRANSFER_AT.scope(phase, runtime.result_analysis(&run, &attempt, &controls(), CancellationToken::new())).await;
        settle(&runtime, result, phase == TransferCheckpoint::AfterGraphAck).await;
    }
    let lost = super::super::artifacts::SCIENTIFIC_HANDOFF_REPLY_LOSS.scope(true,
        runtime.result_analysis(&run, &attempt, &controls(), CancellationToken::new())).await;
    settle(&runtime, lost, false).await;
}
async fn settle(runtime: &Runtime, result: Result<AnalysisHandle, WorkflowError>, graph_acked: bool) {
    let error = result.unwrap_err();
    let WorkflowError::ArtifactTransferUnsettled {analysis, artifact, owner_kind, owner, ..} = error else { panic!("unexpected transfer error: {error:?}"); };
    assert_eq!(analysis.is_some(), graph_acked);
    let recovery = runtime.recover_artifact_intent(&serde_json::to_vec(&artifact).unwrap()).unwrap();
    let reopened = runtime.scientific_artifact(&artifact.key).await.unwrap();
    assert_eq!(reopened.header().key, artifact.key);
    recovery.release(&owner_kind, &owner).await.unwrap();
    recovery.release(&owner_kind, &owner).await.unwrap();
    if let Some(key) = analysis {
        assert!(runtime.canonical_store().unwrap().analysis(&key).await.unwrap().active);
        assert!(runtime.analysis(&key).await.unwrap().edges(None).await.unwrap().batch().num_rows() > 0);
        assert!(runtime.scientific_artifact(&artifact.key).await.is_ok(), "graph retains independent artifact after staging release");
    }
}

#[tokio::test]
async fn canonical_final_graph_ack_loss_retains_exact_intent_and_independent_evidence() {
    let (runtime, run, attempt) = fixture().await;
    let result = GRAPH_HANDOFF_REPLY_LOSS.scope(true,
        runtime.result_analysis(&run, &attempt, &controls(), CancellationToken::new())).await;
    let Err(WorkflowError::ArtifactTransferUnsettled {analysis: Some(key), source, ..}) = &result else {
        panic!("final graph ACK loss must preserve both occurrences: {result:?}");
    };
    let WorkflowError::Canonical(pse_operations::canonical::CanonicalError::AnalysisUnsettled {intent, ..}) = source.as_ref() else {
        panic!("original graph intent is required: {source:?}");
    };
    assert_eq!(&intent.key, key);
    assert!(!intent.active, "original request header is never rewritten as the ACK");
    let actual = runtime.canonical_store().unwrap().analysis(key).await.unwrap();
    check_analysis_ack(&runtime.shared.pool(), intent, &actual).unwrap();
    settle(&runtime, result, true).await;
}

#[tokio::test]
async fn canonical_member_transfer_cancellation_keeps_unacknowledged_facts_out_of_graph() {
    let (runtime, run, attempt) = fixture().await;
    let cancel = CancellationToken::new();
    let interrupted = cancel.clone();
    let result = pse_operations::testing::cancel_after_artifact_append_ack(move || interrupted.cancel(),
        runtime.result_analysis(&run, &attempt, &controls(), cancel.clone())).await;
    assert!(cancel.is_cancelled());
    let Err(WorkflowError::ArtifactTransferUnsettled {analysis: None, artifact, owner_kind, owner, read: None, ..}) = &result else {
        panic!("member cancellation must retain exact original transfer intent: {result:?}");
    };
    assert!(!artifact.active);
    assert!(runtime.scientific_artifact(&artifact.key).await.is_err(),
        "an attached member group is not acknowledged active scientific content");
    let mut source = runtime.results(&run, &attempt, "runtime.scientific_contexts", 0, u64::MAX, CancellationToken::new()).await.unwrap();
    assert_eq!(source.next_batch().await.unwrap().unwrap().num_rows(), 1,
        "the exact protected producer remains readable after interrupted transfer");
    drop(source);
    pse_operations::testing::drain_result_releases(runtime.canonical_store().unwrap()).await.unwrap();
    assert!(runtime.canonical_store().unwrap().forget_run_results(&run).await.is_err(),
        "original inactive transfer staging still protects producer input");
    let recovery = runtime.recover_artifact_intent(&serde_json::to_vec(artifact).unwrap()).unwrap();
    recovery.release(owner_kind, owner).await.unwrap();
    recovery.release(owner_kind, owner).await.unwrap();
    assert!(!recovery.collect().await.unwrap(), "original creation window remains unsettled");
}
