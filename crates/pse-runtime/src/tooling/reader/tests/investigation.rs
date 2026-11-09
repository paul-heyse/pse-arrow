// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;

#[test]
fn cancelled_request_refuses_at_the_public_encoder_before_source_access() {
    let directory = tempfile::tempdir().unwrap();
    let cancel = pse_columnar::CancellationToken::new();
    cancel.cancel();
    let request = ReaderRequest {
        protocol: Protocol::ProductReader,
        version: Version,
        correlation: "stopped-before-source".into(),
        access: Access::Passive,
        request: Request::Investigate {
            request: InvestigationRequest::FrozenInspect {
                evidence: directory.path().join("missing-original.json"),
                request: FrozenInspectionRequest {
                    version: Version,
                    inspection: InspectionRequest::new(operation()),
                    window: RecordedWindow::Recorded,
                },
            },
        },
    };
    let encoded = encode(&serde_json::to_vec(&request).unwrap(), &cancel).unwrap();
    assert_ne!(encoded.exit_code(), 0);
    let answer: serde_json::Value = serde_json::from_slice(encoded.bytes()).unwrap();
    assert_eq!(answer["correlation"], "stopped-before-source");
    assert_eq!(
        answer["payload"]["failure"]["native"]["code"],
        pse_diagnostics::DiagnosticCode::RuntimeCancelled.as_str()
    );
}

#[test]
fn developer_cancellation_response_retains_the_original_native_control_code() {
    let cancel = pse_columnar::CancellationToken::new();
    cancel.cancel();
    let cause = cancel.checkpoint().unwrap_err();
    let error = ReaderError::Developer(crate::tooling::review_query::developer::Error::Columnar(
        cause,
    ));
    let encoded = encode_response(response(
        "controlled-original-cancellation".into(),
        Err(error),
    ))
    .unwrap();
    assert_ne!(encoded.exit_code(), 0);
    let response: serde_json::Value = serde_json::from_slice(encoded.bytes()).unwrap();
    assert_eq!(response["payload"]["failure"]["code"], "request.refused");
    assert_eq!(
        response["payload"]["failure"]["native"]["code"],
        pse_diagnostics::DiagnosticCode::RuntimeCancelled.as_str()
    );
    assert!(
        response["payload"]["failure"]["native"]["message"]
            .as_str()
            .unwrap()
            .contains("cancelled")
    );
}

#[test]
fn cancelled_read_and_independent_drain_failure_keep_both_native_causes() {
    let cancel = pse_columnar::CancellationToken::new();
    cancel.cancel();
    let error = ReaderError::Settlement {
        operation: Box::new(ReaderError::Columnar(cancel.checkpoint().unwrap_err())),
        cleanup: Box::new(ReaderError::Io(std::io::Error::other(
            "independent original connection drain failure",
        ))),
    };
    let encoded = encode_response(response("cancel-and-drain".into(), Err(error))).unwrap();
    let answer: serde_json::Value = serde_json::from_slice(encoded.bytes()).unwrap();
    let causes = answer["payload"]["failure"]["native"]["children"]
        .as_array()
        .unwrap();
    assert_eq!(causes.len(), 2);
    assert_eq!(
        causes[0]["code"],
        pse_diagnostics::DiagnosticCode::RuntimeCancelled.as_str()
    );
    assert!(
        causes[1]["message"]
            .as_str()
            .unwrap()
            .contains("independent original connection drain failure")
    );
}

#[test]
fn encoded_refusal_retains_original_native_owners_through_failed_delivery() {
    let cancel = pse_columnar::CancellationToken::new();
    cancel.cancel();
    let encoded = encode_response(response(
        "original-three-causes".into(),
        Err(ReaderError::Columnar(cancel.checkpoint().unwrap_err())),
    ))
    .unwrap();
    let settled =
        encoded.settlement_failed(ReaderError::Io(std::io::Error::other("original drain")));
    let encoded = encode_response(response("original-three-causes".into(), Err(settled))).unwrap();
    let failure = encoded.delivery_failed(ReaderError::Io(std::io::Error::new(
        std::io::ErrorKind::BrokenPipe,
        "original output",
    )));
    let ReaderError::Delivery {
        response,
        transport,
    } = failure
    else {
        panic!("original refusal and transport both survive");
    };
    assert!(
        matches!(*transport, ReaderError::Io(ref cause) if cause.kind() == std::io::ErrorKind::BrokenPipe)
    );
    let ReaderError::Settlement { operation, cleanup } = *response else {
        panic!("original native settlement owner survives encoding");
    };
    assert!(matches!(
        *operation,
        ReaderError::Columnar(pse_columnar::CanonError::Cancelled)
    ));
    assert!(
        matches!(*cleanup, ReaderError::Io(ref cause) if cause.to_string() == "original drain")
    );
}

#[test]
fn forwarded_reader_errors_keep_their_original_native_codes() {
    use pse_diagnostics::DiagnosticCode as Code;
    for (error, expected) in [
        (
            ReaderError::Workflow(Box::new(crate::workflow::WorkflowError::CallerClosed)),
            Code::WorkflowInput,
        ),
        (
            ReaderError::Workflow(Box::new(crate::workflow::WorkflowError::Resource(
                "original allocation refusal".into(),
            ))),
            Code::RuntimeResourceLimit,
        ),
        (
            ReaderError::Canonical(Box::new(pse_operations::canonical::CanonicalError::Codec(
                pse_operations::canonical_codec::CodecError::Missing("original"),
            ))),
            Code::ValidationInvariant,
        ),
    ] {
        let encoded = encode_response(response("original-native-code".into(), Err(error))).unwrap();
        let answer: serde_json::Value = serde_json::from_slice(encoded.bytes()).unwrap();
        assert_eq!(
            answer["payload"]["failure"]["native"]["code"],
            expected.as_str()
        );
    }
}
use crate::tooling::{
    frozen_inspection::{self, FrozenEvidenceSet, FrozenInspectionRequest, RecordedWindow},
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
    let encoded = encode(
        &serde_json::to_vec(&request).unwrap(),
        &pse_columnar::CancellationToken::new(),
    )
    .unwrap();
    serde_json::from_slice(encoded.bytes()).unwrap()
}
fn operation() -> OperationId {
    OperationId {
        process_instance: [19, 3],
        sequence: 42,
    }
}
#[test]
fn frozen_inspection_preserves_owner_document_bytes_and_response_schema() {
    let root = tempfile::tempdir().unwrap();
    let evidence = FrozenEvidenceSet::default();
    let path = root.path().join("frozen.json");
    std::fs::write(&path, serde_json::to_vec(&evidence).unwrap()).unwrap();
    let request = FrozenInspectionRequest {
        version: Version,
        inspection: InspectionRequest::new(operation()),
        window: RecordedWindow::Recorded,
    };
    let document = frozen_inspection::inspect_frozen(&evidence, request.clone()).unwrap();
    let expected_document = serde_json::to_vec(&document).unwrap();
    let payload = handle(
        Request::Investigate {
            request: InvestigationRequest::FrozenInspect {
                evidence: path,
                request,
            },
        },
        &pse_columnar::CancellationToken::new(),
    )
    .unwrap();
    let mut expected =
        br#"{"kind":"investigation","result":{"action":"frozen_inspect","document":"#.to_vec();
    expected.extend_from_slice(&expected_document);
    expected.extend_from_slice(b"}}");
    assert_eq!(serde_json::to_vec(&payload).unwrap(), expected);

    let response_schema = serde_json::to_value(schema(Contract::Response)).unwrap();
    let frozen_result = response_schema["$defs"]["ResultView"]["oneOf"]
        .as_array()
        .unwrap()
        .iter()
        .find(|variant| variant["properties"]["action"]["const"] == "frozen_inspect")
        .unwrap();
    assert_eq!(
        frozen_result["properties"]["document"]["$ref"],
        "#/$defs/FrozenInspectionDocument"
    );
    let native_schema = serde_json::to_value(
        SchemaSettings::draft2020_12()
            .for_serialize()
            .into_generator()
            .into_root_schema_for::<frozen_inspection::FrozenInspectionDocument>(),
    )
    .unwrap();
    for key in ["type", "properties", "required", "additionalProperties"] {
        assert_eq!(
            response_schema["$defs"]["FrozenInspectionDocument"][key], native_schema[key],
            "original document schema {key}"
        );
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
fn scientific_capture_query_preserves_original_identity_and_unavailable_association() {
    use crate::{
        invocation::{InvocationEndpoints, InvocationEvidenceDocument},
        tooling::capture::{CaptureDocument, CaptureKind, CaptureLoss, CaptureOutput, CaptureOutputKind, CaptureStatus},
        workflow::AttemptSummary,
    };
    use pse_diagnostics::observation::RecordAvailability;
    use pse_model::generated::identities::{AttemptId, RunId};
    use std::{sync::Arc, time::Duration};

    let directory = tempfile::tempdir().unwrap();
    let embedded = directory.path().join("original-prefix-not-supplied");
    let exact = OperationId { process_instance: [u64::MAX, 7], sequence: u64::MAX };
    let distinct_process = OperationId { process_instance: [u64::MAX - 1, 7], ..exact };
    let shared = OperationId { sequence: u64::MAX - 1, ..exact };
    let invocation = |run: u8, attempt: Option<u8>, operation: Option<OperationId>| {
        let run_id = RunId::from_bytes([run; 16]);
        InvocationEvidenceDocument {
            version: Version,
            run_id,
            attempt_id: attempt.map(|id| AttemptId::from_bytes([id; 16])),
            operation,
            family: "modeling".into(),
            clock_scope: format!("original invocation {run} clock"),
            timing_unavailable: operation.is_none().then_some(RecordAvailability::Refused),
            canonical_run: None,
            canonical_attempt: None,
            outcome: InvocationOutcome::Failed,
            endpoints: InvocationEndpoints::default(),
            intervals: vec![],
            omitted_intervals: 4,
            products: vec![],
            omitted_product_steps: 0,
            preparations: None,
            attempts: Arc::new(AttemptSummary::empty_for_test(run_id)),
        }
    };
    let capture = |operation| CaptureDocument {
        version: Version,
        operation,
        kind: CaptureKind::Timeline,
        related_operations: vec![distinct_process],
        launch_process: None,
        status: CaptureStatus::Partial,
        requested_duration: Duration::from_secs(20),
        observed_duration: Duration::from_nanos(371),
        targets: vec![],
        outputs: vec![CaptureOutput {
            path: embedded.clone(),
            kind: CaptureOutputKind::Perfetto,
            bytes: 999,
            readable_prefix_bytes: 500,
            readable: true,
        }],
        loss: CaptureLoss {
            queued_packets_dropped: u64::MAX,
            unknown_clock: true,
            source_partial: true,
            ..Default::default()
        },
        symbols: Default::default(),
        open_intervals: 2,
        errors: vec![],
    };
    let set = FrozenEvidenceSet {
        captures: vec![
            capture(Some(exact)),
            capture(Some(distinct_process)),
            capture(None),
            capture(Some(shared)),
            capture(Some(exact)),
        ],
        invocations: vec![
            invocation(9, Some(2), Some(exact)),
            invocation(4, Some(3), Some(shared)),
            invocation(4, Some(4), Some(shared)),
            invocation(5, None, None),
        ],
        ..Default::default()
    };
    let evidence = directory.path().join("frozen.json");
    // Retain original whitespace to distinguish the exact source hash from a re-encoding.
    let original_bytes = serde_json::to_vec_pretty(&set).unwrap();
    std::fs::write(&evidence, &original_bytes).unwrap();
    let response = read(
        InvestigationRequest::ScientificCaptureAnalyze {
            request: crate::tooling::review_query::scientific_capture::Request {
                evidence,
                memory_bytes: NonZeroUsize::new(16 << 20).unwrap(),
            },
        },
        Access::Passive,
    );
    assert_eq!(response["category"], "success", "{response}");
    let analysis = &response["payload"]["result"]["analysis"];
    assert_eq!(analysis["evidence_sha256"], hash(&original_bytes));
    let captures = analysis["captures"].as_array().unwrap();
    assert_eq!(captures.len(), 5);
    for (index, row) in captures.iter().enumerate() {
        assert_eq!(row["pointer"], format!("/captures/{index}"));
    }
    assert_eq!(captures[0]["association"], serde_json::json!({"kind":"linked","invocation_member":0}));
    assert_eq!(captures[0]["status"], "Partial");
    assert_eq!(captures[0]["operation"]["sequence"].as_u64(), Some(u64::MAX));
    assert_eq!(captures[0]["loss"]["queued_packets_dropped"].as_u64(), Some(u64::MAX));
    assert_eq!(captures[0]["loss"]["unknown_clock"], true);
    assert_eq!(captures[0]["observed_duration"], serde_json::to_value(set.captures[0].observed_duration).unwrap());
    assert_eq!(captures[1]["association"]["kind"], "invocation_unavailable");
    assert_eq!(captures[2]["association"]["kind"], "operation_unavailable");
    assert_eq!(captures[3]["association"], serde_json::json!({"kind":"ambiguous","candidates":2}));
    assert_eq!(captures[4]["association"], captures[0]["association"]);
    let invocations = analysis["invocations"].as_array().unwrap();
    assert_eq!(invocations.len(), 4);
    assert_eq!(invocations[0]["pointer"], "/invocations/0");
    assert_eq!(invocations[0]["run_id"], serde_json::to_value(set.invocations[0].run_id).unwrap());
    assert_eq!(invocations[0]["attempt_id"], serde_json::to_value(set.invocations[0].attempt_id).unwrap());
    assert_eq!(invocations[0]["outcome"], "failed");
    assert_eq!(invocations[0]["clock_scope"], "original invocation 9 clock");
    assert_eq!(invocations[1]["clock_scope"], "original invocation 4 clock");
    assert_eq!(invocations[3]["timing_unavailable"], "refused");
    assert!(invocations[3]["attempt_id"].is_null());
    assert!(!embedded.exists());
}

#[test]
fn scientific_capture_query_distinguishes_empty_evidence_from_allocation_refusal() {
    let directory = tempfile::tempdir().unwrap();
    let evidence = directory.path().join("empty-original.json");
    std::fs::write(&evidence, serde_json::to_vec(&FrozenEvidenceSet::default()).unwrap()).unwrap();
    let request = |bytes| InvestigationRequest::ScientificCaptureAnalyze {
        request: crate::tooling::review_query::scientific_capture::Request {
            evidence: evidence.clone(),
            memory_bytes: NonZeroUsize::new(bytes).unwrap(),
        },
    };
    let empty = read(request(16 << 20), Access::Passive);
    assert_eq!(empty["category"], "success", "{empty}");
    assert_eq!(empty["payload"]["result"]["analysis"]["captures"], serde_json::json!([]));
    let refused = read(request(1), Access::Passive);
    assert_eq!(refused["category"], "refused", "{refused}");
    assert_eq!(
        refused["payload"]["failure"]["native"]["code"],
        pse_diagnostics::DiagnosticCode::RuntimeResourceLimit.as_str()
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
