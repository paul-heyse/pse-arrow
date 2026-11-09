// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use buffa::Message as _;
use pse_contract_proto::support::native_into;
use pse_host::{
    developer_evidence_contracts::{self as contracts, pb::pse::developer::evidence::v1 as pb},
    envelope::{ErrorRecord, RedactionPolicy, Settlement},
    test_workflow::execute::{
        Action, EvidenceFile, EvidenceOmission, EvidenceRole, EvidenceRoot, Outcome,
        ReportObservation, ReportReference, TerminalRecord,
    },
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
    member_pending(root, selected, failure, complete, settled, None)
}
fn member_pending(
    root: &std::path::Path,
    selected: Option<usize>,
    failure: bool,
    complete: bool,
    settled: bool,
    pending: Option<&str>,
) -> Member {
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
        version: 3,
        invocation: CliInvocationId::new(),
        action: Action::Test,
        outcome: Outcome::Incomplete,
        inventory: None,
        selected_tests: selected,
        required_capture_complete: complete,
        failures,
        inventory_content: None,
        evidence: Some(vec![input_omission()]),
    };
    committed_member(root, report, settled, pending)
}

fn committed_member(
    root: &std::path::Path,
    report: ReportObservation,
    settled: bool,
    pending: Option<&str>,
) -> Member {
    let extra = pending.map(|value| ("pending", value));
    committed_member_with(root, report, settled, extra)
}

fn committed_member_with(
    root: &std::path::Path,
    report: ReportObservation,
    settled: bool,
    extra: Option<(&str, &str)>,
) -> Member {
    let id = report.invocation.clone();
    let mut bytes = serde_json::to_vec(&report).unwrap();
    if let Some((name, value)) = extra {
        assert_eq!(bytes.pop(), Some(b'}'));
        bytes.extend_from_slice(format!(",\"{name}\":{value}").as_bytes());
        bytes.push(b'}');
    }
    let identity = ContentIdentity::of(&bytes);
    let terminal = TerminalRecord {
        protocol: "pse.test-workflow".into(),
        version: 1,
        invocation: id.clone(),
        action: report.action,
        outcome: report.outcome,
        report: ReportReference {
            path: "report.json".into(),
            sha256: identity.sha256,
            byte_length: identity.byte_length,
        },
        required_capture_complete: report.required_capture_complete,
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

#[tokio::test]
async fn original_group_failure_views_preserve_setup_compiler_and_omitted_references() {
    let root = tempfile::tempdir().unwrap();
    let report = |action| ReportObservation {
        version: 3,
        invocation: CliInvocationId::new(),
        action,
        outcome: Outcome::Failed,
        inventory: None,
        selected_tests: None,
        required_capture_complete: true,
        failures: vec![],
        inventory_content: None,
        evidence: Some(vec![input_omission()]),
    };
    let check = committed_member_with(
        root.path(),
        report(Action::Check),
        true,
        Some((
            "observations",
            r#"[
        {"kind":"check","group":2,"observation":{"failures":[{"code":"E0382"}],"omitted_failure_records":3}}
    ]"#,
        )),
    );
    let run = committed_member_with(
        root.path(),
        report(Action::Test),
        true,
        Some((
            "observations",
            r#"[
        {"kind":"test","group":0,"observation":{"failures":[]},"omitted_failure_records":2},
        {"kind":"test","group":4,"observation":{"failures":[
            {"identity":{"group":4,"package_id":"package","binary_id":"unit","test_name":"setup"},
             "target_setup_refusals":["native_service_inputs"]},
            {"identity":{"group":4,"package_id":"package","binary_id":"unit","test_name":"construction"},
             "target_setup_refusals":["native_service_construction"]},
            {"identity":{"group":4,"package_id":"package","binary_id":"unit","test_name":"assertion"}}
        ]},"omitted_failure_records":0}
    ]"#,
        )),
    );
    let absent = committed_member(root.path(), report(Action::Test), false, None);
    let answer = analyze(request(vec![check, run, absent]), &CancellationToken::new())
        .await
        .unwrap();
    let document = answer.document();
    let sources = &document.sources;
    let check = &sources[0].group_failure_views.as_ref().unwrap()[0];
    assert_eq!((check.group, check.references, check.omitted), (2, 1, 3));
    let groups = sources[1].group_failure_views.as_ref().unwrap();
    assert_eq!((groups[0].references, groups[0].omitted), (0, 2));
    assert_eq!(
        (groups[1].group, groups[1].pointer.as_str()),
        (4, "/observations/1")
    );
    assert!(sources[2].group_failure_views.is_none());
    for (kind, code, class, member, pointer) in [
        (
            "compiler_group_failure",
            Some("E0382"),
            None,
            0,
            "/observations/0/observation/failures/0",
        ),
        (
            "test_group_setup_refusal",
            None,
            Some("native_service_inputs"),
            1,
            "/observations/1/observation/failures/0/target_setup_refusals/0",
        ),
        (
            "test_group_setup_refusal",
            None,
            Some("native_service_construction"),
            1,
            "/observations/1/observation/failures/1/target_setup_refusals/0",
        ),
        (
            "test_group_failure",
            None,
            None,
            1,
            "/observations/1/observation/failures/2",
        ),
    ] {
        let pattern = document
            .patterns
            .iter()
            .find(|row| {
                row.diagnostic_kind.as_deref() == Some(kind) && row.class.as_deref() == class
            })
            .unwrap();
        assert_eq!(pattern.code.as_deref(), code);
        assert_eq!(pattern.class.as_deref(), class);
        assert_eq!(
            pattern.references,
            [Reference {
                member,
                pointer: Some(pointer.into())
            }]
        );
    }
    let foreign_action = committed_member_with(
        root.path(),
        report(Action::List),
        false,
        Some((
            "observations",
            r#"[{"kind":"check","group":0,"observation":{"failures":[],"omitted_failure_records":0}}]"#,
        )),
    );
    assert!(matches!(
        analyze(request(vec![foreign_action]), &CancellationToken::new()).await,
        Err(Error::Evidence(_))
    ));
}
fn request(members: Vec<Member>) -> Request {
    Request {
        members,
        memory_bytes: NonZeroUsize::new(64 << 20).unwrap(),
    }
}

fn input_omission() -> EvidenceFile {
    EvidenceFile {
        root: EvidenceRoot::Invocation,
        path: "workflow-input.json".into(),
        role: EvidenceRole::Input,
        group: None,
        content: None,
        omission: Some(EvidenceOmission::Unsettled),
        failure: None,
    }
}

#[tokio::test]
async fn original_dependency_diagnostics_keep_nested_pointers_without_inventing_operation_failure()
{
    let root = tempfile::tempdir().unwrap();
    let mut failure = ErrorRecord::from_host(
        &pse_host::HostError::Ownership("original compiler-context capture refusal".into()),
        &RedactionPolicy::default(),
    );
    failure.related.push(failure.clone());
    failure.causes.push("original ordinary source".into());
    let original = committed_member(
        root.path(),
        ReportObservation {
            version: 3,
            invocation: CliInvocationId::new(),
            action: Action::Test,
            outcome: Outcome::Incomplete,
            inventory: None,
            selected_tests: None,
            required_capture_complete: false,
            failures: vec![],
            inventory_content: None,
            evidence: Some(vec![
                input_omission(),
                EvidenceFile {
                    root: EvidenceRoot::Workflow,
                    path: "explicit-original/compiler-context.json".into(),
                    role: EvidenceRole::CompilerContext,
                    group: Some(3),
                    content: None,
                    omission: Some(EvidenceOmission::Refused),
                    failure: Some(failure),
                },
            ]),
        },
        false,
        None,
    );
    let analysis = analyze(request(vec![original]), &CancellationToken::new())
        .await
        .unwrap();
    let patterns = &analysis.document().patterns;
    assert_eq!(patterns.len(), 2);
    let typed = patterns
        .iter()
        .find(|pattern| pattern.diagnostic_kind.as_deref() == Some("evidence_diagnostic"))
        .unwrap();
    assert_eq!(typed.code.as_deref(), Some("host.ownership"));
    assert_eq!(typed.invocations, 1);
    assert_eq!(
        typed.references,
        [
            Reference {
                member: 0,
                pointer: Some("/evidence/1/failure".into())
            },
            Reference {
                member: 0,
                pointer: Some("/evidence/1/failure/related/0".into())
            },
        ]
    );
    let ordinary = patterns
        .iter()
        .find(|pattern| pattern.diagnostic_kind.as_deref() == Some("evidence_ordinary_source"))
        .unwrap();
    assert_eq!(ordinary.code, None);
    assert_eq!(
        ordinary.references,
        [Reference {
            member: 0,
            pointer: Some("/evidence/1/failure/causes/0".into())
        }]
    );
}

const ORIGINAL_PENDING: &str = r#"[
 {"kind":"native","capabilities":["compiler","klu"]},
 {"kind":"native","capabilities":[]},
 {"kind":"fixture","requirement":{"group":2,"unit":{"package_id":"original-package","package_name":"example","target_name":"original","target_kinds":["lib"],"source_path":"/original/lib.rs","features":[],"platform":null,"mode":"test","root":true},"declaration":{"id":"selected-original","target":"original","kind":"lib","ownership":"target-owned","scope":{"tests":["selected_case"]},"inputs":["study-module","server","worker","engineering-endpoint","scientific-inputs"]}},"readiness":{"ownership":"target-owned","inputs":{
  "study-module":{"state":"unprepared","code":"original.study-module.unprepared","next":["scripts/pse","fixture","prepare","study-module"]},
  "server":{"state":"selected","path":"/must-not-follow/original-server","identity":"original-owner"},
  "worker":{"state":"refused","code":"original.worker.refused","next":[]},
  "scientific-inputs":{"state":"refused","code":null,"next":[]},
  "engineering-endpoint":{"state":"owner-admission-required","path":"/must-not-follow/endpoint"}
 }}},
 {"kind":"fixture","requirement":{"group":3,"unit":{"package_id":"original-package","package_name":"example","target_name":"original","target_kinds":["lib"],"source_path":"/original/lib.rs","features":[],"platform":null,"mode":"test","root":true},"declaration":{"id":"empty-original-inputs","target":"original","kind":"lib","ownership":"preparation-only","scope":{"tests":["selected_case"]},"inputs":[]}},"readiness":{"ownership":"preparation-only","inputs":{}}}
]"#;

#[tokio::test]
async fn original_pending_query_preserves_unknown_empty_and_each_recorded_input_without_cause_product()
 {
    let root = tempfile::tempdir().unwrap();
    let original = member_pending(
        root.path(),
        Some(0),
        true,
        false,
        false,
        Some(ORIGINAL_PENDING),
    );
    let empty = member_pending(root.path(), Some(0), false, false, false, Some("[]"));
    let unknown = member(root.path(), Some(0), false, false, false);
    let analysis = analyze(
        request(vec![original, empty, unknown]),
        &CancellationToken::new(),
    )
    .await
    .unwrap();
    let doc = analysis.document();
    assert_eq!(doc.query, SQL);
    assert_eq!(doc.prerequisite_query, PREREQUISITE_SQL);
    assert_eq!(
        doc.patterns.len(),
        3,
        "the first cause question is independent of nine pending rows"
    );
    assert_eq!(doc.prerequisite_patterns.len(), 11);
    let known_empty = doc
        .prerequisite_patterns
        .iter()
        .find(|p| p.pending_known && p.prerequisite_kind.is_none())
        .unwrap();
    assert_eq!(known_empty.pending_entries_min, Some(0));
    assert_eq!(known_empty.pending_entries_max, Some(0));
    assert_eq!(
        known_empty.references,
        [Reference {
            member: 1,
            pointer: Some("/pending".into())
        }]
    );
    let missing = doc
        .prerequisite_patterns
        .iter()
        .find(|p| !p.pending_known)
        .unwrap();
    assert_eq!(missing.pending_entries_min, None);
    assert_eq!(
        missing.references,
        [Reference {
            member: 2,
            pointer: None
        }]
    );
    for capability in ["compiler", "klu"] {
        let pattern = doc
            .prerequisite_patterns
            .iter()
            .find(|p| p.capability.as_deref() == Some(capability))
            .unwrap();
        assert_eq!(pattern.invocations, 1);
        assert_eq!(pattern.pending_entries_min, Some(4));
        assert_eq!(pattern.members, [0]);
    }
    for (input, state, code) in [
        (
            "study-module",
            "unprepared",
            Some("original.study-module.unprepared"),
        ),
        ("server", "selected", None),
        ("worker", "refused", Some("original.worker.refused")),
        ("scientific-inputs", "refused", None),
        ("engineering-endpoint", "owner-admission-required", None),
    ] {
        let pattern = doc
            .prerequisite_patterns
            .iter()
            .find(|p| p.input.as_deref() == Some(input))
            .unwrap();
        assert_eq!(pattern.state.as_deref(), Some(state));
        assert_eq!(pattern.code.as_deref(), code);
        assert_eq!(pattern.ownership.as_deref(), Some("target-owned"));
        assert_eq!(
            pattern.references,
            [Reference {
                member: 0,
                pointer: Some(format!("/pending/2/readiness/inputs/{input}"))
            }]
        );
        assert_eq!(pattern.invocations, 1);
    }
    assert!(
        doc.prerequisite_patterns
            .iter()
            .any(|p| p.prerequisite_kind.as_deref() == Some("native")
                && p.capability.is_none()
                && p.references[0].pointer.as_deref() == Some("/pending/1"))
    );
    assert!(
        doc.prerequisite_patterns
            .iter()
            .any(|p| p.fixture.as_deref() == Some("empty-original-inputs")
                && p.input.is_none()
                && p.references[0].pointer.as_deref() == Some("/pending/3"))
    );
}

#[tokio::test]
async fn original_pending_projection_retains_typed_low_pool_cancel_and_escaped_buffer_owner() {
    let root = tempfile::tempdir().unwrap();
    let input = member_pending(
        root.path(),
        None,
        false,
        false,
        false,
        Some(ORIGINAL_PENDING),
    );
    let descriptors =
        buffa_descriptor::generated::descriptor::FileDescriptorSet::decode_from_slice(
            &std::fs::read(&input.descriptors).unwrap(),
        )
        .unwrap();
    let facts = developer_evidence::inspect_facts(
        &descriptors,
        "pse.developer.evidence.v1.CapturedWorkflowReport",
        &std::fs::read(&input.payload).unwrap(),
        &developer_evidence::binary_options(),
    )
    .unwrap();
    let tiny: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(1));
    let error = tables::prerequisites(
        std::slice::from_ref(&facts),
        &tiny,
        &CancellationToken::new(),
    )
    .unwrap_err();
    assert!(matches!(
        &error,
        crate::observation_detail::CaptureError::Columnar(
            pse_columnar::CanonError::NativeResource(_)
        )
    ));
    assert_eq!(
        pse_diagnostics::TypedDiagnostic::diagnostic_code(&error),
        Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit)
    );
    let root_error = Error::Projection(error);
    assert_eq!(
        pse_diagnostics::TypedDiagnostic::diagnostic_code(&root_error),
        Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit)
    );
    assert_eq!(tiny.reserved(), 0);
    let pool: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(4 << 20));
    let batch = tables::prerequisites(
        std::slice::from_ref(&facts),
        &pool,
        &CancellationToken::new(),
    )
    .unwrap();
    let escaped = batch.column(0).clone();
    assert_eq!(batch.num_rows(), 9);
    drop(batch);
    assert!(pool.reserved() > 0);
    drop(escaped);
    assert_eq!(pool.reserved(), 0);
    let mut input_owners = Vec::new();
    let descriptor_bytes = acquire(&input.descriptors, &pool, &mut input_owners).unwrap();
    let payload_bytes = acquire(&input.payload, &pool, &mut input_owners).unwrap();
    let invocations = tables::invocations(
        std::slice::from_ref(&facts),
        &pool,
        &CancellationToken::new(),
    )
    .unwrap();
    let causes = tables::causes(
        std::slice::from_ref(&facts),
        &pool,
        &CancellationToken::new(),
    )
    .unwrap();
    let prior = pool.reserved();
    let prerequisites = tables::prerequisites(
        std::slice::from_ref(&facts),
        &pool,
        &CancellationToken::new(),
    )
    .unwrap();
    let projection_bytes = pool.reserved() - prior;
    let query_budget = prior + projection_bytes / 2;
    drop((
        prerequisites,
        causes,
        invocations,
        input_owners,
        payload_bytes,
        descriptor_bytes,
    ));
    assert_eq!(pool.reserved(), 0);
    // The same actual acquisition/table owners determine a budget that admits
    // the existing first-question tables but refuses the new prerequisite view.
    let error = analyze(
        Request {
            members: vec![input.clone()],
            memory_bytes: NonZeroUsize::new(query_budget).unwrap(),
        },
        &CancellationToken::new(),
    )
    .await
    .unwrap_err();
    assert!(matches!(
        &error,
        Error::Projection(crate::observation_detail::CaptureError::Columnar(
            pse_columnar::CanonError::NativeResource(_)
        ))
    ));
    assert_eq!(
        pse_diagnostics::TypedDiagnostic::diagnostic_code(&error),
        Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit)
    );
    let protocol_input = input.clone();
    let response = tokio::task::spawn_blocking(move || {
        use crate::tooling::reader::{self, Access, InvestigationRequest, Protocol, ReaderRequest};
        let request = ReaderRequest {
            protocol: Protocol::ProductReader,
            version: Version,
            correlation: "original-prerequisite-allocation-refusal".into(),
            access: Access::Passive,
            request: reader::Request::Investigate {
                request: InvestigationRequest::DeveloperAnalyze {
                    request: Request {
                        members: vec![protocol_input],
                        memory_bytes: NonZeroUsize::new(query_budget).unwrap(),
                    },
                },
            },
        };
        let encoded = reader::encode(
            &serde_json::to_vec(&request).unwrap(),
            &CancellationToken::new(),
        )
        .unwrap();
        assert_ne!(encoded.exit_code(), 0);
        serde_json::from_slice::<serde_json::Value>(encoded.bytes()).unwrap()
    })
    .await
    .unwrap();
    assert_eq!(
        response["payload"]["failure"]["native"]["code"],
        pse_diagnostics::DiagnosticCode::RuntimeResourceLimit.as_str()
    );
    let cancel = CancellationToken::new();
    cancel.cancel();
    let error = tables::prerequisites(&[facts], &pool, &cancel).unwrap_err();
    assert_eq!(
        pse_diagnostics::TypedDiagnostic::diagnostic_code(&error),
        Some(pse_diagnostics::DiagnosticCode::RuntimeCancelled)
    );
    assert_eq!(pool.reserved(), 0);
    let error = analyze(
        Request {
            members: vec![input],
            memory_bytes: NonZeroUsize::new(1).unwrap(),
        },
        &CancellationToken::new(),
    )
    .await
    .unwrap_err();
    assert_eq!(
        pse_diagnostics::TypedDiagnostic::diagnostic_code(&error),
        Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit)
    );
}

#[tokio::test]
async fn original_pending_null_and_duplicate_input_refuse_without_changing_readback() {
    let root = tempfile::tempdir().unwrap();
    for pending in [
        "null",
        "[{\"kind\":\"native\",\"capabilities\":[\"compiler\",\"compiler\"]}]",
    ] {
        let input = member_pending(root.path(), None, false, false, false, Some(pending));
        assert!(
            analyze(request(vec![input]), &CancellationToken::new())
                .await
                .is_err()
        );
    }
    let duplicate = ORIGINAL_PENDING.replace("\"server\":{", "\"worker\":{\"state\":\"selected\",\"path\":\"/never/read\",\"identity\":\"original\"},\"server\":{");
    let input = member_pending(root.path(), None, false, false, false, Some(&duplicate));
    assert!(
        analyze(request(vec![input]), &CancellationToken::new())
            .await
            .is_err()
    );
}

#[test]
fn original_pending_combined_population_refuses_before_retaining_projection_buffers() {
    let root = tempfile::tempdir().unwrap();
    let pending = serde_json::to_string(&serde_json::json!([{"kind":"native","capabilities":
        (0..2500).map(|n| format!("native-{n:08}")).collect::<Vec<_>>()
    }]))
    .unwrap();
    let inputs = (0..2)
        .map(|_| member_pending(root.path(), None, false, false, false, Some(&pending)))
        .collect::<Vec<_>>();
    let facts = inputs
        .iter()
        .map(|input| {
            let descriptors =
                buffa_descriptor::generated::descriptor::FileDescriptorSet::decode_from_slice(
                    &std::fs::read(&input.descriptors).unwrap(),
                )
                .unwrap();
            developer_evidence::inspect_facts(
                &descriptors,
                "pse.developer.evidence.v1.CapturedWorkflowReport",
                &std::fs::read(&input.payload).unwrap(),
                &developer_evidence::binary_options(),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    let pool: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(64 << 20));
    let error = tables::prerequisites(&facts, &pool, &CancellationToken::new()).unwrap_err();
    assert!(matches!(
        error,
        crate::observation_detail::CaptureError::Invalid(_)
    ));
    assert_eq!(pool.reserved(), 0);
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
            pointer: Some("/evidence".into())
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
    let error = analyze(request(vec![]), &cancelled).await.unwrap_err();
    assert!(matches!(
        error,
        Error::Columnar(pse_columnar::CanonError::Cancelled)
    ));
    assert_eq!(
        pse_diagnostics::TypedDiagnostic::diagnostic_code(&error),
        Some(pse_diagnostics::DiagnosticCode::RuntimeCancelled)
    );
    let table_error = tables::invocations(&[], &pool, &cancelled).unwrap_err();
    assert!(matches!(
        table_error,
        crate::observation_detail::CaptureError::Columnar(pse_columnar::CanonError::Cancelled)
    ));
    assert_eq!(
        pse_diagnostics::TypedDiagnostic::diagnostic_code(&table_error),
        Some(pse_diagnostics::DiagnosticCode::RuntimeCancelled)
    );
}

#[test]
fn explicit_fifo_refuses_without_waiting_for_a_peer_and_regular_symlinks_remain_supported() {
    use std::sync::mpsc;
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("explicit-fifo");
    rustix::fs::mkfifoat(
        rustix::fs::CWD,
        &path,
        rustix::fs::Mode::RUSR | rustix::fs::Mode::WUSR,
    )
    .unwrap();
    let (send, receive) = mpsc::channel();
    let input = path.clone();
    let owner = std::thread::spawn(move || {
        let pool: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(1 << 20));
        let result = acquire(&input, &pool, &mut Vec::new());
        send.send(matches!(result, Err(Error::Invalid(reason)) if reason.contains("regular file")))
            .unwrap();
    });
    let refused = receive.recv_timeout(std::time::Duration::from_secs(2));
    // Release and join even if a regression reintroduces blocking open. RDWR on
    // this caller-owned FIFO supplies its peer without signalling any process.
    let release = rustix::fs::open(
        &path,
        rustix::fs::OFlags::RDWR | rustix::fs::OFlags::CLOEXEC | rustix::fs::OFlags::NONBLOCK,
        rustix::fs::Mode::empty(),
    )
    .unwrap();
    owner.join().unwrap();
    drop(release);
    assert!(refused.unwrap());
    let regular = root.path().join("regular");
    std::fs::write(&regular, b"explicit selected regular bytes").unwrap();
    let link = root.path().join("explicit-link");
    std::os::unix::fs::symlink(&regular, &link).unwrap();
    let pool: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(1 << 20));
    assert_eq!(
        acquire(&link, &pool, &mut Vec::new()).unwrap(),
        b"explicit selected regular bytes"
    );
}

#[tokio::test]
async fn in_flight_stream_cancellation_drops_owned_stream_and_preserves_typed_cause() {
    use std::{
        pin::Pin,
        sync::atomic::{AtomicBool, Ordering},
        task::{Context, Poll},
    };
    struct Pending {
        started: Option<tokio::sync::oneshot::Sender<()>>,
        dropped: Arc<AtomicBool>,
    }
    impl futures_util::Stream for Pending {
        type Item = Result<RecordBatch, datafusion::error::DataFusionError>;
        fn poll_next(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            if let Some(started) = self.started.take() {
                let _ = started.send(());
            }
            Poll::Pending
        }
    }
    impl Drop for Pending {
        fn drop(&mut self) {
            self.dropped.store(true, Ordering::SeqCst);
        }
    }
    let dropped = Arc::new(AtomicBool::new(false));
    let (send, receive) = tokio::sync::oneshot::channel();
    let stream = datafusion::physical_plan::stream::RecordBatchStreamAdapter::new(
        Arc::new(datafusion::arrow::datatypes::Schema::empty()),
        Pending {
            started: Some(send),
            dropped: Arc::clone(&dropped),
        },
    );
    let cancel = CancellationToken::new();
    let child = cancel.clone();
    let operation = tokio::spawn(async move { read_patterns(Box::pin(stream), &child).await });
    receive.await.unwrap();
    cancel.cancel();
    let error = operation.await.unwrap().unwrap_err();
    assert!(matches!(
        error,
        Error::Columnar(pse_columnar::CanonError::Cancelled)
    ));
    assert!(
        dropped.load(Ordering::SeqCst),
        "the selected stream, not an external workload, dropped"
    );
    assert_eq!(
        pse_diagnostics::TypedDiagnostic::diagnostic_code(&error),
        Some(pse_diagnostics::DiagnosticCode::RuntimeCancelled)
    );
}

#[tokio::test]
async fn cancellation_during_last_empty_batch_or_eof_cannot_publish_success() {
    for empty_batch in [true, false] {
        let cancel = CancellationToken::new();
        let source = cancel.clone();
        let schema = Arc::new(datafusion::arrow::datatypes::Schema::empty());
        let batch_schema = Arc::clone(&schema);
        let stream = futures_util::stream::poll_fn(move |_| {
            source.cancel();
            std::task::Poll::Ready(
                empty_batch.then(|| Ok(RecordBatch::new_empty(Arc::clone(&batch_schema)))),
            )
        });
        let stream =
            datafusion::physical_plan::stream::RecordBatchStreamAdapter::new(schema, stream);
        let error = read_patterns(Box::pin(stream), &cancel).await.unwrap_err();
        assert!(matches!(
            error,
            Error::Columnar(pse_columnar::CanonError::Cancelled)
        ));
    }
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
    let encoded = reader::encode(
        &serde_json::to_vec(&request).unwrap(),
        &CancellationToken::new(),
    )
    .unwrap();
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
    let encoded = reader::encode(
        &serde_json::to_vec(&request).unwrap(),
        &CancellationToken::new(),
    )
    .unwrap();
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
