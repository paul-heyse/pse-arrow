// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Independently authored wire failures challenge native interpretation and public disclosure.
use super::*;
use crate::envelope::{ErrorRecord, RedactionPolicy};
use base64::Engine as _;
use connectrpc::{ConnectError, ErrorCode, ErrorDetail};
use std::error::Error as _;

fn native(bytes: &[u8]) -> ErrorDetail {
    ErrorDetail {
        type_url: "type.googleapis.com/pse.code_intelligence.v1.Failure".into(),
        value: Some(base64::engine::general_purpose::STANDARD.encode(bytes)),
        debug: None,
    }
}
fn rpc(error: &Error) -> &RpcFailure {
    let Error::Rpc(error) = error else {
        panic!("RPC cause lost")
    };
    error
}
fn received(error: ConnectError) -> Error {
    // A real remote boundary has no server-local Rust source, retaining wire status/details.
    let encoded = serde_json::to_vec(&error).unwrap();
    rpc::transport_error(serde_json::from_slice(&encoded).unwrap())
}

#[test]
fn semantic_rpc_failure_envelopes_redact_opaque_backend_data_without_changing_originals() {
    use crate::envelope::{
        CliInvocationId, CommandOutcome, Envelope, OutputFormat, OutputOptions, Presentation,
    };
    use buffa::Message as _;
    use serde_json::json;

    let secrets = ["alpha-private-token", "bravo-private-token"];
    let policy =
        RedactionPolicy::new(secrets.map(|secret| ("EXPLICIT_SECRET".into(), secret.into())));
    let public_key = "<redacted:EXPLICIT_SECRET>";
    let reserved_key = format!("{public_key}<redacted-key-collision:1>");
    let data = json!({
        secrets[0]: "first secret key's public evidence",
        secrets[1]: "second secret key's public evidence",
        public_key: "public key's evidence",
        reserved_key.clone(): "public suffix-shaped key's evidence",
        "array": [secrets[0], {"nested": secrets[1]}, 17, false, null],
        "provider": "original provider",
    });
    let mut original = Failure {
        diagnostic_code: Some("provider.refusal".into()),
        context: None,
        state: FailureState::Failed,
        phase: FailurePhase::Provider,
        message: "original public provider refusal".into(),
        backend_code: Some(-32000),
        backend_data: Some(data),
        observed_cancellation: None,
        validation: None,
    };
    original.observed_cancellation = Some(Box::new(CancellationObservation {
        completion: Settlement::LspReturned,
        original: OriginalOutcome::Failed(Box::new(original.clone())),
    }));
    let exact_native = serde_json::to_value(&original).unwrap();
    let wire = wire::failure_to_pb(original.clone()).unwrap();
    assert_eq!(
        serde_json::to_value(wire::failure_from_pb(wire.clone()).unwrap()).unwrap(),
        exact_native,
        "raw failure codec retains original opaque evidence",
    );
    let transport = ConnectError::unavailable("independent original transport message")
        .with_detail(native(&wire.encode_to_vec()))
        .with_source(std::io::Error::new(
            std::io::ErrorKind::ConnectionReset,
            "original local cause",
        ));
    let exact_transport = serde_json::to_value(&transport).unwrap();
    for error in [
        Error::Refusal(Box::new(original)),
        rpc::transport_error(transport),
    ] {
        let mut response = Envelope::response(
            "inspect entity refusal",
            "semantic request did not produce a current answer",
            Failure::from_error_with_policy(&error, &policy),
        );
        response.outcome = CommandOutcome::Failure;
        response
            .failures
            .push(ErrorRecord::from_typed(&error, &policy));
        let output = OutputOptions {
            format: OutputFormat::Json,
            presentation: Presentation::default(),
            invocation_id: CliInvocationId::new(),
            artifact_directory: None,
        };
        let (code, frame) = output.encode_frame(response).unwrap();
        assert_eq!(code, 1);
        let text = String::from_utf8(frame).unwrap();
        for secret in secrets {
            assert!(!text.contains(secret), "{text}");
        }
        let disclosed: serde_json::Value = serde_json::from_str(&text).unwrap();
        let expected = json!({
            public_key: "public key's evidence",
            reserved_key.clone(): "public suffix-shaped key's evidence",
            format!("{public_key}<redacted-key-collision:2>"):
                "first secret key's public evidence",
            format!("{public_key}<redacted-key-collision:3>"):
                "second secret key's public evidence",
            "array": [public_key, {"nested": public_key}, 17, false, null],
            "provider": "original provider",
        });
        assert_eq!(disclosed["payload"]["backend_data"], expected);
        assert_eq!(
            disclosed["payload"]["observed_cancellation"]["original"]["failure"]["backend_data"],
            expected,
        );
        assert_eq!(disclosed["payload"]["diagnostic_code"], "provider.refusal");
        assert_eq!(disclosed["payload"]["backend_code"], -32000);
        assert_eq!(
            serde_json::to_value(error.native_failure().unwrap()).unwrap(),
            exact_native,
            "disclosure must only mutate the copied failure",
        );
        if let Error::Rpc(failure) = &error {
            assert_eq!(
                serde_json::to_value(&failure.original).unwrap(),
                exact_transport,
            );
            assert_eq!(
                failure
                    .original
                    .source()
                    .unwrap()
                    .downcast_ref::<std::io::Error>()
                    .unwrap()
                    .kind(),
                std::io::ErrorKind::ConnectionReset,
            );
        }
    }
}

fn observed(error: &Error, expected: RpcDisposition, phase: FailurePhase) {
    let original = rpc(error);
    assert_eq!(original.observation().disposition, expected);
    let failure = Failure::from_error(error);
    assert_eq!(
        std::mem::discriminant(&failure.phase),
        std::mem::discriminant(&phase)
    );
    assert_eq!(
        original.observation().grpc_code,
        original.original.code.grpc_code()
    );
    let cause = error
        .source()
        .unwrap()
        .downcast_ref::<ConnectError>()
        .unwrap();
    assert_eq!(cause.message, original.original.message);
    assert_eq!(cause.details.len(), original.original.details.len());
}

#[test]
fn semantic_rpc_authored_failure_overrides_no_transport_status_or_diagnostic_identity() {
    // Literal protobuf: phase CONTEXT=2, code="new.code", message="old", state STALE=3,
    // backend_code=-7. No production encoder chooses these expectations.
    let detail = native(&[
        8, 2, 18, 8, b'n', b'e', b'w', b'.', b'c', b'o', b'd', b'e', 26, 3, b'o', b'l', b'd', 32,
        249, 255, 255, 255, 255, 255, 255, 255, 255, 1, 48, 3,
    ]);
    for code in [
        ErrorCode::InvalidArgument,
        ErrorCode::Unavailable,
        ErrorCode::Internal,
    ] {
        let error = received(
            ConnectError::new(code, "transport status is independent").with_detail(detail.clone()),
        );
        observed(&error, RpcDisposition::Native, FailurePhase::Context);
        let failure = Failure::from_error(&error);
        assert!(matches!(failure.state, FailureState::Stale));
        assert_eq!(failure.backend_code, Some(-7));
        assert_eq!(failure.diagnostic_code.as_deref(), Some("new.code"));
        assert_eq!(failure.message, "old");
        assert!(failure.backend_data.is_none());
        let record = ErrorRecord::from_host(
            &crate::HostError::Semantic(Box::new(error)),
            &RedactionPolicy::default(),
        );
        let observation = record.semantic_rpc.unwrap();
        assert_eq!(
            observation.native_diagnostic_code.as_deref(),
            Some("new.code")
        );
        assert!(matches!(
            observation.native_state,
            Some(FailureState::Stale)
        ));
        assert!(matches!(
            observation.native_phase,
            Some(FailurePhase::Context)
        ));
    }
}

#[test]
fn semantic_rpc_absent_unknown_and_canonical_only_details_never_invent_user_refusal() {
    for details in [
        Vec::new(),
        vec![ErrorDetail {
            type_url: "future.v2.Failure".into(),
            value: Some("%%%".into()),
            debug: Some(serde_json::json!({"original":true})),
        }],
        vec![ErrorDetail::from_message(
            "buf.validate.Violations",
            &protovalidate_buffa::proto::Violations::default(),
        )],
    ] {
        let count = details.len();
        let mut source =
            ConnectError::invalid_argument("schema or validator classification unavailable");
        source.details = details;
        let original = serde_json::to_value(&source).unwrap();
        let error = received(source);
        observed(
            &error,
            if count == 0 {
                RpcDisposition::Absent
            } else {
                RpcDisposition::Unrecognized
            },
            FailurePhase::Transport,
        );
        assert_eq!(
            serde_json::to_value(&rpc(&error).original).unwrap(),
            original
        );
        assert!(rpc(&error).observation().native_phase.is_none());
        assert!(matches!(
            Failure::from_error(&error).state,
            FailureState::Failed
        ));
    }
}

#[test]
fn semantic_rpc_malformed_and_uninterpretable_details_keep_original_and_conversion_cause() {
    let malformed = [
        ErrorDetail {
            type_url: "pse.code_intelligence.v1.Failure".into(),
            value: None,
            debug: None,
        },
        ErrorDetail {
            type_url: "pse.code_intelligence.v1.Failure".into(),
            value: Some("%%%".into()),
            debug: None,
        },
        native(&[0xff]),
    ];
    for detail in malformed {
        let error =
            received(ConnectError::invalid_argument("do not infer constraint").with_detail(detail));
        observed(&error, RpcDisposition::Malformed, FailurePhase::Schema);
        assert!(rpc(&error).conversion_error().is_none());
    }
    for detail in [native(&[8, 99]), native(&[48, 99]), native(&[74, 0])] {
        let error = received(
            ConnectError::invalid_argument("unknown native enum or missing context reason")
                .with_detail(detail),
        );
        observed(
            &error,
            RpcDisposition::ConversionRefused,
            FailurePhase::Schema,
        );
        assert!(matches!(
            rpc(&error).conversion_error(),
            Some(Error::Schema(_))
        ));
        let record = ErrorRecord::from_typed(&error, &RedactionPolicy::default());
        assert_eq!(record.related.len(), 1);
        assert!(rpc(&error).observation().native_state.is_none());
    }
}

#[test]
fn semantic_rpc_multiple_and_oversized_details_do_not_select_first_native_failure() {
    let valid = native(&[8, 2, 48, 3]);
    let malformed = native(&[0xff]);
    for details in [
        vec![valid.clone(), malformed.clone()],
        vec![malformed, valid.clone()],
        vec![valid.clone(), valid.clone()],
    ] {
        let mut error = ConnectError::invalid_argument("ambiguous");
        error.details = details;
        let error = received(error);
        observed(&error, RpcDisposition::Ambiguous, FailurePhase::Schema);
        assert_eq!(rpc(&error).observation().native_detail_count, 2);
        assert!(rpc(&error).observation().native_phase.is_none());
    }
    for details in [
        vec![native(&vec![0; 65536])],
        vec![
            valid,
            ErrorDetail {
                type_url: "unknown".into(),
                value: Some("x".repeat(65536)),
                debug: None,
            },
        ],
        vec![
            ErrorDetail {
                type_url: "unknown".into(),
                value: None,
                debug: None
            };
            65
        ],
    ] {
        let mut original = ConnectError::internal("entire source is retained");
        original.details = details;
        let original_json = serde_json::to_value(&original).unwrap();
        let error = received(original);
        observed(&error, RpcDisposition::Oversized, FailurePhase::Transport);
        assert_eq!(
            serde_json::to_value(&rpc(&error).original).unwrap(),
            original_json
        );
        let record = ErrorRecord::from_typed(&error, &RedactionPolicy::default());
        assert!(serde_json::to_vec(&record).unwrap().len() < 2048);
    }
}

#[test]
fn semantic_rpc_local_transport_cause_survives_while_public_record_omits_original_details() {
    let original = ConnectError::unavailable("private-token transport message")
        .with_http_status(http::StatusCode::BAD_GATEWAY)
        .with_source(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "private-token local cause",
        ))
        .with_detail(ErrorDetail {
            type_url: "unknown".into(),
            value: Some("private-token raw bytes".into()),
            debug: Some(serde_json::json!({"private-token":"debug"})),
        });
    let error = rpc::transport_error(original);
    let transport = error
        .source()
        .unwrap()
        .downcast_ref::<ConnectError>()
        .unwrap();
    assert_eq!(rpc(&error).observation().http_status, 502);
    assert_eq!(
        transport
            .source()
            .unwrap()
            .downcast_ref::<std::io::Error>()
            .unwrap()
            .kind(),
        std::io::ErrorKind::ConnectionRefused
    );
    let policy = RedactionPolicy::new([("TEST_SECRET".into(), "private-token".into())]);
    let record = ErrorRecord::from_host(&crate::HostError::Semantic(Box::new(error)), &policy);
    assert!(
        !serde_json::to_string(&record)
            .unwrap()
            .contains("private-token")
    );
    assert!(record.causes.is_empty());
    assert_eq!(
        record.semantic_rpc.unwrap().disposition,
        RpcDisposition::Unrecognized
    );
}

#[test]
fn semantic_rpc_investigation_retains_authored_disposition_and_recovery() {
    let error: crate::investigation::Error = rpc::transport_error(
        ConnectError::invalid_argument("transport cannot select scientific meaning")
            .with_detail(native(&[8, 2, 48, 3])),
    )
    .into();
    assert!(matches!(
        error.phase(),
        crate::investigation::DiagnosticPhase::Context
    ));
    assert!(matches!(
        error.disposition(),
        crate::investigation::FailureDisposition::Stale
    ));
    assert!(matches!(
        error.native_refusal().unwrap().state,
        FailureState::Stale
    ));
    assert_eq!(
        error
            .record(&RedactionPolicy::default())
            .semantic_rpc
            .unwrap()
            .disposition,
        RpcDisposition::Native
    );
    let error: crate::investigation::Error =
        rpc::transport_error(ConnectError::invalid_argument("unknown origin")).into();
    assert!(matches!(
        error.phase(),
        crate::investigation::DiagnosticPhase::Delivery
    ));
    assert!(matches!(
        error.disposition(),
        crate::investigation::FailureDisposition::Failed
    ));
    assert!(error.native_refusal().is_none());
}

#[test]
fn semantic_rpc_error_record_exports_original_codes_and_correction_with_independent_tags() {
    use buffa::Message as _;
    use buffa_descriptor::{DescriptorPool, DynamicMessage, Value};
    let error = received(
        ConnectError::unavailable("independent status")
            .with_detail(native(&[8, 2, 18, 1, b'X', 48, 3]))
            .with_detail(ErrorDetail::from_message(
                "buf.validate.Violations",
                &protovalidate_buffa::proto::Violations::default(),
            )),
    );
    let record = ErrorRecord::from_typed(&error, &RedactionPolicy::default());
    let wire: crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ErrorRecord =
        pse_contract_proto::support::native_into(record, "test").unwrap();
    let descriptors =
        buffa_descriptor::generated::descriptor::FileDescriptorSet::decode_from_slice(
            crate::developer_evidence_contracts::DESCRIPTORS,
        )
        .unwrap();
    let pool = std::sync::Arc::new(DescriptorPool::new(descriptors).unwrap());
    let message = pool
        .message_index("pse.developer.evidence.v1.ErrorRecord")
        .unwrap();
    let dynamic = DynamicMessage::decode(pool, message, &wire.encode_to_vec()).unwrap();
    let Some(Value::Message(rpc)) = dynamic.field_by_number(16) else {
        panic!("missing RPC facet tag16")
    };
    assert_eq!(rpc.field_by_number(1), Some(&Value::U32(14)));
    assert_eq!(rpc.field_by_number(4), Some(&Value::U64(2)));
    assert_eq!(rpc.field_by_number(9), Some(&Value::String("X".into())));
    let Some(Value::Message(state)) = rpc.field_by_number(6) else {
        panic!("missing native state")
    };
    assert!(
        state.field_by_number(4).is_some(),
        "native stale alternative tag4"
    );
    let Some(Value::Message(phase)) = rpc.field_by_number(7) else {
        panic!("missing native phase")
    };
    assert!(
        phase.field_by_number(3).is_some(),
        "native context alternative tag3"
    );
    let Some(Value::Message(validation)) = rpc.field_by_number(8) else {
        panic!("canonical correction")
    };
    let Some(Value::Message(disclosure)) = validation.field_by_number(1) else {
        panic!("validation disclosure")
    };
    assert!(
        disclosure.field_by_number(4).is_none(),
        "unknown originating count stays absent"
    );
    let Some(Value::Message(coverage)) = disclosure.field_by_number(6) else {
        panic!("validation coverage")
    };
    assert!(
        coverage.field_by_number(1).is_some(),
        "unknown coverage is not complete"
    );
}

#[test]
fn semantic_rpc_caller_policy_redacts_native_and_canonical_only_nested_correction() {
    use crate::code_intelligence_contracts::pb::pse::code_intelligence::v1 as pb;
    use buffa::Message as _;
    use protovalidate_buffa::{FieldPath, FieldPathElement, ValidationError, Violation};
    let marker = "NONENVSECRET";
    let path = FieldPath {
        elements: vec![FieldPathElement {
            field_name: Some(format!("{marker}_field").into()),
            ..Default::default()
        }],
    };
    let canonical = ValidationError {
        violations: vec![Violation {
            field: path.clone(),
            rule: path,
            rule_id: format!("{marker}_rule").into(),
            message: format!("{marker} rejected input").into(),
            for_key: false,
        }],
        ..Default::default()
    }
    .to_proto();
    let native_failure = pb::Failure {
        phase: pb::failure::Phase::Constraint.into(),
        state: pb::failure::State::Failed.into(),
        code: format!("{marker}.code"),
        message: "public native constraint".into(),
        validation_disclosure: pb::ValidationDisclosure {
            message_type: Some(format!("{marker}.Type")),
            ..Default::default()
        }
        .into(),
        ..Default::default()
    };
    for authored in [false, true] {
        let mut original = ConnectError::invalid_argument("local transport original").with_detail(
            ErrorDetail::from_message("buf.validate.Violations", &canonical),
        );
        if authored {
            original = original.with_detail(native(&native_failure.encode_to_vec()));
        }
        let error = received(original);
        let exact = serde_json::to_value(&rpc(&error).original).unwrap();
        let policy = RedactionPolicy::new([("EXPLICIT_SECRET".into(), marker.into())]);
        let record = ErrorRecord::from_typed(&error, &policy);
        let encoded = serde_json::to_string(&record).unwrap();
        assert!(
            !encoded.contains(marker),
            "native code and canonical nested strings obey caller policy"
        );
        assert!(encoded.contains("<redacted:EXPLICIT_SECRET>"));
        assert_eq!(serde_json::to_value(&rpc(&error).original).unwrap(), exact);
        let correction = record
            .semantic_rpc
            .as_ref()
            .unwrap()
            .validation
            .as_ref()
            .unwrap();
        assert_eq!(correction.violations.len(), 1);
        assert!(
            correction.violations[0]
                .message
                .as_ref()
                .unwrap()
                .contains("<redacted:")
        );
        let wire: crate::developer_evidence_contracts::pb::pse::developer::evidence::v1::ErrorRecord =
            pse_contract_proto::support::native_into(record, "test").unwrap();
        assert!(
            !wire
                .encode_to_vec()
                .windows(marker.len())
                .any(|bytes| bytes == marker.as_bytes())
        );
        let host = crate::HostError::Semantic(Box::new(error));
        assert!(
            !serde_json::to_string(&ErrorRecord::from_host(&host, &policy))
                .unwrap()
                .contains(marker)
        );
    }
}

#[test]
fn semantic_rpc_default_policy_never_renders_local_only_causes_through_diagnostic_sources() {
    #[derive(Debug, thiserror::Error)]
    #[error("cleanup summary")]
    struct CleanupCause(#[source] std::io::Error);
    for authored in [false, true] {
        let private = "unclassified-original-input".repeat(10_000);
        let mut original = ConnectError::invalid_argument(private.clone()).with_source(
            protovalidate_buffa::ValidationError {
                compile_error: Some("unclassified-local-validator-cause".into()),
                ..Default::default()
            },
        );
        if authored {
            original = original.with_detail(native(&[
                8, 3, 26, 6, b'p', b'u', b'b', b'l', b'i', b'c', 48, 4,
            ]));
        }
        let error = rpc::transport_error(original);
        assert_eq!(
            rpc(&error).original.message.as_deref(),
            Some(private.as_str())
        );
        assert!(
            rpc(&error)
                .original
                .source()
                .unwrap()
                .downcast_ref::<protovalidate_buffa::ValidationError>()
                .is_some()
        );
        let host = crate::HostError::Semantic(Box::new(error));
        let record = ErrorRecord::from_host(&host, &RedactionPolicy::default());
        let encoded = serde_json::to_string(&record).unwrap();
        assert!(!encoded.contains("unclassified-"));
        assert!(encoded.len() < 8192, "large original source remains local");
        assert!(
            record.diagnostic_source.is_some(),
            "diagnostic relationship survives"
        );
        let crate::HostError::Semantic(error) = host else {
            panic!("semantic error ownership must survive host diagnostic projection")
        };
        let investigation = crate::investigation::Error::Semantic(error);
        let host = crate::HostError::Investigation(investigation);
        let record = ErrorRecord::from_host(&host, &RedactionPolicy::default());
        let encoded = serde_json::to_string(&record).unwrap();
        assert!(!encoded.contains("unclassified-"));
        assert!(encoded.len() < 8192);
        assert!(record.semantic_rpc.is_some());
        assert!(record.diagnostic_source.is_some());
        let record = ErrorRecord::from_typed(&host, &RedactionPolicy::default());
        let encoded = serde_json::to_string(&record).unwrap();
        assert!(!encoded.contains("unclassified-"));
        assert!(encoded.len() < 8192);
        assert!(
            record.semantic_rpc.is_some(),
            "comparison's direct typed host path retains the facet"
        );
        let aggregate = crate::HostError::Settlement {
            operation: Box::new(host),
            cleanup: Box::new(crate::HostError::Io(std::io::Error::other(CleanupCause(
                std::io::Error::other("independent-cleanup-cause"),
            )))),
        };
        let record = ErrorRecord::from_typed(&aggregate, &RedactionPolicy::default());
        let encoded = serde_json::to_string(&record).unwrap();
        assert!(
            !encoded.contains("unclassified-"),
            "erased related diagnostic cannot restore local RPC causes"
        );
        assert!(
            record.semantic_rpc.is_none(),
            "aggregate does not merge distinct native failures"
        );
        assert!(record.children[0].semantic_rpc.is_some());
        assert!(
            record.children[1]
                .causes
                .iter()
                .any(|cause| cause.contains("independent-cleanup-cause")),
            "unrelated typed sibling keeps its own ordinary cause"
        );
        assert_eq!(record.related.len(), 2);
    }
}
