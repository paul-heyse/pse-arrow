// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::tooling::{
    frozen_inspection::FrozenEvidenceSet,
    review_analysis::AnalysisRequest,
    review_pack::{PackReader, PackSelection},
    review_query::{ReviewQuery, ReviewQueryRequest},
};
use std::num::NonZeroUsize;

fn read(request: InvestigationRequest, access: Access) -> serde_json::Value {
    let mut request: ReaderRequest = serde_json::from_slice(&super::request(
        "explicit-reader",
        Request::Investigate { request },
    ))
    .unwrap();
    request.access = access;
    let encoded = encode(&serde_json::to_vec(&request).unwrap()).unwrap();
    serde_json::from_slice(encoded.bytes()).unwrap()
}
fn operation() -> OperationId {
    OperationId {
        process_instance: [19, 3],
        sequence: 42,
    }
}
#[test]
fn passive_route_refuses_export_before_creating_output_or_reading_evidence() {
    let root = tempfile::tempdir().unwrap();
    let destination = root.path().join("uncreated-pack");
    let response = read(
        InvestigationRequest::Export {
            evidence: root.path().join("not-an-existing-source"),
            request: AnalysisRequest::new(operation()),
            memory_bytes: NonZeroUsize::new(128 << 20).unwrap(),
            destination: destination.clone(),
            selection: PackSelection::default(),
            files: vec![],
        },
        Access::Passive,
    );
    assert_eq!(response["category"], "refused");
    assert!(
        response["payload"]["failure"]["message"]
            .as_str()
            .unwrap()
            .contains("explicit investigation")
    );
    assert!(!destination.exists());
}
#[test]
fn declared_query_reports_specification_without_external_execution() {
    let response = read(
        InvestigationRequest::QuerySpecification {
            request: ReviewQueryRequest {
                version: Version,
                input_member: "selected-capture".into(),
                question: ReviewQuery::Coverage,
                clock_scope: "original".into(),
                interval: None,
                timeout_seconds: 1.0,
                output_bytes: 4096,
            },
        },
        Access::Passive,
    );
    assert_eq!(response["category"], "success");
    assert_eq!(response["payload"]["result"]["executed"], false);
    assert!(
        !response["payload"]["result"]["specification"]["sql"]
            .as_str()
            .unwrap()
            .is_empty()
    );
}
#[test]
fn selected_pack_uses_only_supplied_evidence_and_retains_analysis_limitations() {
    let root = tempfile::tempdir().unwrap();
    let evidence = root.path().join("frozen.json");
    std::fs::write(
        &evidence,
        serde_json::to_vec(&FrozenEvidenceSet::default()).unwrap(),
    )
    .unwrap();
    let destination = root.path().join("selected-pack");
    let response = read(
        InvestigationRequest::Export {
            evidence,
            request: AnalysisRequest::new(operation()),
            memory_bytes: NonZeroUsize::new(128 << 20).unwrap(),
            destination: destination.clone(),
            selection: PackSelection::default(),
            files: vec![],
        },
        Access::Investigation,
    );
    assert_eq!(response["category"], "success", "{response}");
    let pack = PackReader::open(&destination, 16 << 20).unwrap();
    assert!(pack.read_frozen().unwrap().checkpoints.is_empty());
    let analysis = pack.read_analysis().unwrap();
    assert_eq!(analysis.request.operation, operation());
    assert!(
        analysis
            .questions
            .iter()
            .flat_map(|question| &question.statements)
            .any(|statement| matches!(statement.kind, pse_diagnostics::StatementKind::Limitation))
    );
}
#[test]
fn supplied_evidence_version_and_missing_source_refuse_without_starting_services() {
    let root = tempfile::tempdir().unwrap();
    let evidence = root.path().join("unknown.json");
    std::fs::write(&evidence, br#"{"version":999,"checkpoints":[]}"#).unwrap();
    let response = read(
        InvestigationRequest::Analyze {
            evidence,
            request: AnalysisRequest::new(operation()),
            memory_bytes: NonZeroUsize::new(128 << 20).unwrap(),
        },
        Access::Passive,
    );
    assert_eq!(response["category"], "refused");
}
#[test]
fn explicit_timeline_capture_joins_collector_without_finishing_original_work() {
    let root = tempfile::tempdir().unwrap();
    let owner = Diagnostics::new(Policy::default(), root.path().join("checkpoints"));
    let operation = owner.hub().start("original performing owner");
    owner.publish().unwrap();
    let output = root.path().join("selected-capture");
    let selected = InvestigationRequest::Capture {
        request: crate::tooling::capture::CaptureRequest {
            version: Version,
            checkpoint: owner.checkpoint_path(),
            operation: operation.context().id(),
            kind: crate::tooling::capture::CaptureKind::Timeline,
            output: output.clone(),
            duration: std::time::Duration::from_millis(25),
            budget: Default::default(),
        },
    };
    let refused = read(selected.clone(), Access::Passive);
    assert_eq!(refused["category"], "refused");
    assert!(!output.exists());
    let captured = read(selected, Access::Investigation);
    assert_eq!(captured["category"], "success", "{captured}");
    let document = &captured["payload"]["result"]["document"];
    assert_ne!(document["status"], "Running");
    assert!(!document["outputs"].as_array().unwrap().is_empty());
    assert_eq!(
        operation
            .reader()
            .snapshot(std::time::Duration::from_secs(1))
            .outcome,
        None
    );
    operation.finish(Outcome::Completed);
    owner.release().unwrap();
}
