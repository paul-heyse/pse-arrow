// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use buffa::Message as _;
use pse_contract_proto::support::native_into;
use pse_host::{
    developer_evidence_contracts::{self as contracts, pb::pse::developer::evidence::v1 as pb},
    envelope::{ErrorRecord, RedactionPolicy, Settlement},
    test_workflow::execute::{Action, Outcome, ReportObservation, ReportReference, TerminalRecord},
};

// Independent literal source documents challenge the generic query. These committed
// portable observations are deliberately not claims about a performing native verdict.
fn member(
    root: &std::path::Path,
    selected: Option<usize>,
    failure: bool,
    complete: bool,
    settled: bool,
) -> Member {
    let id = CliInvocationId::new();
    let failures = if failure {
        let mut error = ErrorRecord::from_host(
            &pse_host::HostError::Ownership("original prerequisite refusal".into()),
            &RedactionPolicy::default(),
        );
        error.related.push(error.clone());
        error.causes.push("independent cleanup refusal".into());
        vec![error]
    } else {
        vec![]
    };
    let report = ReportObservation {
        version: 1,
        invocation: id.clone(),
        action: Action::Test,
        outcome: Outcome::Incomplete,
        inventory: None,
        selected_tests: selected,
        required_capture_complete: complete,
        failures,
        inventory_content: None,
        evidence: None,
    };
    let bytes = serde_json::to_vec(&report).unwrap();
    let identity = ContentIdentity::of(&bytes);
    let terminal = TerminalRecord {
        protocol: "pse.test-workflow".into(),
        version: 1,
        invocation: id.clone(),
        action: Action::Test,
        outcome: Outcome::Incomplete,
        report: ReportReference {
            path: "report.json".into(),
            sha256: identity.sha256,
            byte_length: identity.byte_length,
        },
        required_capture_complete: complete,
    };
    let wire = pb::CapturedWorkflowReport {
        invocation: native_into(id.clone(), "invocation").unwrap().into(),
        report_original: bytes,
        report_observation: native_into(report, "report").unwrap().into(),
        terminal: native_into(terminal, "terminal").unwrap().into(),
        scope: native_into(EvidenceScope::Report, "scope").unwrap().into(),
        recorded_manager_settlement: settled
            .then(|| native_into(Settlement::Complete, "settlement").unwrap())
            .into(),
        recorded_native_lifetime_pending: settled.then_some(false),
        ..Default::default()
    };
    let descriptors = root.join("descriptor.bin");
    std::fs::write(&descriptors, contracts::DESCRIPTORS).unwrap();
    let payload = root.join(format!("{id}.pb"));
    std::fs::write(&payload, wire.encode_to_vec()).unwrap();
    Member {
        descriptors,
        payload,
    }
}
fn request(members: Vec<Member>) -> Request {
    Request {
        members,
        memory_bytes: NonZeroUsize::new(64 << 20).unwrap(),
    }
}

#[tokio::test]
async fn original_member_join_groups_causes_and_preserves_exact_counts_empty_and_unknown() {
    let root = tempfile::tempdir().unwrap();
    let high = 9_007_199_254_740_993_usize;
    let first = member(root.path(), Some(high), true, false, false);
    let second = member(root.path(), Some(0), true, false, false);
    let third = member(root.path(), None, false, true, true);
    let answer = analyze(
        request(vec![first, second, third]),
        &CancellationToken::new(),
    )
    .await
    .unwrap();
    let document = answer.document();
    assert_eq!(document.sources.len(), 3);
    assert_eq!(document.patterns.len(), 3);
    let diagnostic = document
        .patterns
        .iter()
        .find(|p| p.diagnostic_kind.as_deref() == Some("diagnostic"))
        .unwrap();
    assert_eq!(
        diagnostic.invocations, 2,
        "related copies do not double-count invocations"
    );
    assert_eq!(diagnostic.members, [0, 1]);
    assert_eq!(
        diagnostic.references,
        [
            Reference {
                member: 0,
                pointer: Some("/failures/0".into())
            },
            Reference {
                member: 0,
                pointer: Some("/failures/0/related/0".into())
            },
            Reference {
                member: 1,
                pointer: Some("/failures/0".into())
            },
            Reference {
                member: 1,
                pointer: Some("/failures/0/related/0".into())
            },
        ]
    );
    assert_eq!(diagnostic.selected_tests_min, Some(0));
    assert_eq!(diagnostic.selected_tests_max, Some(high as u64));
    assert!(!diagnostic.required_capture_complete);
    assert!(diagnostic.recorded_manager_settlement.is_none());
    let cleanup = document
        .patterns
        .iter()
        .find(|p| p.diagnostic_kind.as_deref() == Some("ordinary_source"))
        .unwrap();
    assert_eq!(cleanup.members, [0, 1]);
    assert!(cleanup.code.is_none());
    let empty = document
        .patterns
        .iter()
        .find(|p| p.diagnostic_kind.is_none())
        .unwrap();
    assert_eq!(empty.members, [2]);
    assert_eq!(
        empty.references,
        [Reference {
            member: 2,
            pointer: None
        }]
    );
    assert_eq!(empty.invocations, 1);
    assert!(empty.selected_tests_min.is_none());
    assert!(empty.selected_tests_max.is_none());
    assert_eq!(
        empty.recorded_manager_settlement.as_deref(),
        Some("complete")
    );
    assert_eq!(empty.recorded_native_lifetime_pending, Some(false));
    assert!(
        document
            .sources
            .iter()
            .all(|s| !s.report_sha256.is_empty() && !s.limitations.is_empty())
    );
    assert!(
        document
            .limitations
            .iter()
            .any(|s| s.contains("source-clock"))
    );
}

#[tokio::test]
async fn original_member_query_retains_commitment_io_duplicate_and_allocation_refusals() {
    let root = tempfile::tempdir().unwrap();
    let original = member(root.path(), None, true, false, false);
    assert!(
        matches!(analyze(request(vec![original.clone(), original.clone()]), &CancellationToken::new()).await,
        Err(Error::Invalid(reason)) if reason.contains("duplicate original"))
    );
    let bytes = std::fs::read(&original.payload).unwrap();
    let mut wire = pb::CapturedWorkflowReport::decode_from_slice(&bytes).unwrap();
    wire.report_original.push(b' ');
    std::fs::write(&original.payload, wire.encode_to_vec()).unwrap();
    assert!(matches!(
        analyze(request(vec![original.clone()]), &CancellationToken::new()).await,
        Err(Error::Evidence(_))
    ));
    std::fs::write(&original.payload, bytes).unwrap();
    let mut tiny = request(vec![original.clone()]);
    tiny.memory_bytes = NonZeroUsize::new(1).unwrap();
    let refusal = analyze(tiny, &CancellationToken::new()).await.unwrap_err();
    assert!(matches!(
        refusal,
        Error::Engine(datafusion::error::DataFusionError::ResourcesExhausted(_))
    ));
    assert_eq!(
        pse_diagnostics::TypedDiagnostic::diagnostic_code(&refusal),
        Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit)
    );
    std::fs::remove_file(&original.payload).unwrap();
    assert!(matches!(
        analyze(request(vec![original]), &CancellationToken::new()).await,
        Err(Error::Io(_))
    ));
}

#[tokio::test]
async fn typed_original_views_keep_missing_generation_clock_and_selected_count_null() {
    let root = tempfile::tempdir().unwrap();
    let input = member(root.path(), None, false, false, false);
    let descriptors =
        buffa_descriptor::generated::descriptor::FileDescriptorSet::decode_from_slice(
            contracts::DESCRIPTORS,
        )
        .unwrap();
    let facts = developer_evidence::inspect_facts(
        &descriptors,
        "pse.developer.evidence.v1.CapturedWorkflowReport",
        &std::fs::read(input.payload).unwrap(),
        &developer_evidence::binary_options(),
    )
    .unwrap();
    let pool: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(4 << 20));
    let batch = tables::invocations(&[facts], &pool, &CancellationToken::new()).unwrap();
    for name in [
        "selected_tests",
        "producer_generation",
        "clock_scope",
        "recorded_manager_settlement",
        "recorded_native_lifetime_pending",
    ] {
        assert!(batch.column_by_name(name).unwrap().is_null(0), "{name}");
    }
    drop(batch);
    assert_eq!(
        pool.reserved(),
        0,
        "escaped Arrow buffer lease releases with the view"
    );
    let empty = analyze(request(vec![]), &CancellationToken::new())
        .await
        .unwrap();
    assert!(empty.document().patterns.is_empty());
    assert!(empty.document().sources.is_empty());
    let cancelled = CancellationToken::new();
    cancelled.cancel();
    assert!(analyze(request(vec![]), &cancelled).await.is_err());
}

#[test]
fn existing_passive_reader_protocol_executes_the_registered_developer_query() {
    use crate::tooling::reader::{self, Access, InvestigationRequest, Protocol, ReaderRequest};
    let root = tempfile::tempdir().unwrap();
    let input = member(root.path(), None, false, false, false);
    let mut request = ReaderRequest {
        protocol: Protocol::ProductReader,
        version: Version,
        correlation: "original-developer-patterns".into(),
        access: Access::Passive,
        request: reader::Request::Investigate {
            request: InvestigationRequest::DeveloperAnalyze {
                request: request(vec![input]),
            },
        },
    };
    let encoded = reader::encode(&serde_json::to_vec(&request).unwrap()).unwrap();
    assert_eq!(encoded.exit_code(), 0);
    let response: serde_json::Value = serde_json::from_slice(encoded.bytes()).unwrap();
    let answer = &response["payload"]["result"]["analysis"];
    assert_eq!(answer["sources"].as_array().unwrap().len(), 1);
    assert_eq!(answer["patterns"][0]["members"], serde_json::json!([0]));
    assert!(answer["patterns"][0]["selected_tests_min"].is_null());
    assert!(response["managed_invocation"].is_null());
    let schema = serde_json::to_value(reader::schema(reader::Contract::Request)).unwrap();
    assert!(schema.to_string().contains("developer_analyze"));
    let reader::Request::Investigate {
        request: InvestigationRequest::DeveloperAnalyze { request: selected },
    } = &mut request.request
    else {
        panic!("original registered request changed")
    };
    selected.memory_bytes = NonZeroUsize::new(1).unwrap();
    let encoded = reader::encode(&serde_json::to_vec(&request).unwrap()).unwrap();
    assert_ne!(encoded.exit_code(), 0);
    let refusal: serde_json::Value = serde_json::from_slice(encoded.bytes()).unwrap();
    assert_eq!(refusal["payload"]["failure"]["code"], "request.refused");
    assert_eq!(
        refusal["payload"]["failure"]["native"]["code"],
        pse_diagnostics::DiagnosticCode::RuntimeResourceLimit.as_str()
    );
    assert!(
        !refusal["payload"]["failure"]["native"]["causes"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}
