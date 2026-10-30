// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct Payload {
    records: Vec<u32>,
    secret_detail: String,
    explanation: String,
}
fn payload() -> Payload {
    Payload {
        records: vec![1, 2, 3],
        secret_detail: "detail".into(),
        explanation: "αβ 🦀".into(),
    }
}
#[test]
fn full_and_projected_serialization_validate_against_their_actual_schemas() {
    for view in [
        Presentation::default(),
        Presentation {
            fields: Some(vec!["records".into(), "explanation".into()]),
            budget: Some(1),
        },
        Presentation {
            fields: Some(vec![]),
            budget: Some(0),
        },
    ] {
        let mut response = Envelope::response("fixture inspect", "observed fixture", payload());
        response.subject_state = SubjectState::Running;
        response.settlement = Settlement::Unknown;
        response.observation.completeness = Completeness::Partial;
        response
            .observation
            .limitations
            .push("stream ended before terminal reconciliation".into());
        let mut output = Vec::new();
        assert_eq!(
            write(&mut output, response, OutputFormat::Json, &view).unwrap(),
            1
        );
        assert!(output.starts_with(b"{\"summary\":"));
        let value: serde_json::Value = serde_json::from_slice(&output).unwrap();
        let schema = schema::<Payload>("fixture inspect", &view).unwrap();
        let validator = jsonschema::validator_for(schema.as_value()).unwrap();
        assert!(
            validator.is_valid(&value),
            "{}",
            validator
                .iter_errors(&value)
                .map(|error| error.to_string())
                .collect::<Vec<_>>()
                .join("; ")
        );
        assert_eq!(value["settlement"], "unknown");
        assert_eq!(value["observation"]["completeness"], "partial");
        assert_eq!(
            value["observation"]["limitations"][0],
            "stream ended before terminal reconciliation"
        );
        if view.fields.is_some() {
            assert!(value["payload"].get("secret_detail").is_none());
            assert_eq!(value["view"]["truncated"], true);
        }
    }
}
#[test]
fn raw_pretty_payload_cannot_break_ndjson_or_change_number_lexemes() {
    #[derive(Serialize, JsonSchema)]
    struct RawPayload {
        upstream: Box<serde_json::value::RawValue>,
    }
    let original = "{\n \"huge\":1234567890123456789012345678901234567890,\n \"repeat\": 1,\"repeat\":2,\"escaped\":\"a \\\" b\\n c\"\n}";
    let raw = serde_json::value::RawValue::from_string(original.into()).unwrap();
    let response = Envelope::response("fixture raw", "raw retained", RawPayload { upstream: raw });
    let mut output = Vec::new();
    write(
        &mut output,
        response,
        OutputFormat::Ndjson,
        &Presentation::default(),
    )
    .unwrap();
    let text = String::from_utf8(output).unwrap();
    assert_eq!(text.lines().count(), 1);
    assert!(text.contains("1234567890123456789012345678901234567890"));
    assert!(text.contains("\"repeat\":1,\"repeat\":2"));
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(value["record"], "observation_summary");
    assert_eq!(value["observation"]["subject_state"], "not_applicable");
}
#[test]
fn exit_classes_keep_launch_wait_workload_failure_and_settlement_distinct() {
    let mut response = Envelope::response("wait", "still active", ());
    response.subject_state = SubjectState::Launched;
    assert_eq!(response.exit_code(), 0);
    response.subject_state = SubjectState::Running;
    response.observation.timed_out = true;
    assert_eq!(response.exit_code(), 0);
    response.subject_state = SubjectState::TimedOut;
    assert_eq!(response.exit_code(), 1);
    response.subject_state = SubjectState::Failed;
    response.child_exit = Some(7);
    assert_eq!(response.exit_code(), 1);
    response.subject_state = SubjectState::Succeeded;
    response.settlement = Settlement::Unknown;
    assert_eq!(response.exit_code(), 1);
    response.settlement = Settlement::Complete;
    response.observation.completeness = Completeness::Unknown;
    assert_eq!(response.exit_code(), 1);
    response.outcome = CommandOutcome::BoundaryFailure;
    assert_eq!(response.exit_code(), 125);
    response.outcome = CommandOutcome::Usage;
    assert_eq!(response.exit_code(), 2);
    response.outcome = CommandOutcome::Interrupted;
    assert_eq!(response.exit_code(), 130);
}
#[test]
fn invocation_identity_roundtrips_and_rejects_unvalidated_strings() {
    let id = CliInvocationId::new();
    let encoded = serde_json::to_string(&id).unwrap();
    assert_eq!(
        serde_json::from_str::<CliInvocationId>(&encoded).unwrap(),
        id
    );
    assert!(serde_json::from_str::<CliInvocationId>("\"../not-an-id\"").is_err());
    let response = Envelope::response("fixture", "ok", payload());
    let encoded = serde_json::to_string(&response).unwrap();
    let restored: Envelope<Payload> = serde_json::from_str(&encoded).unwrap();
    assert_eq!(restored.invocation_id, response.invocation_id);
}
#[test]
fn every_settlement_failure_and_ordinary_cause_is_retained_and_redacted() {
    let failure = crate::HostError::Settlement {
        operation: Box::new(crate::HostError::Io(std::io::Error::other(
            "failed using private-value",
        ))),
        cleanup: Box::new(crate::HostError::Deadline("drain unavailable".into())),
    };
    let policy = RedactionPolicy::new([("PSE_SECRET_TOKEN".into(), "private-value".into())]);
    let record = ErrorRecord::from_typed(&failure, &policy);
    assert_eq!(record.code.as_deref(), Some("host.settlement"));
    assert_eq!(record.children.len(), 2);
    assert_eq!(record.children[0].code.as_deref(), Some("host.io"));
    assert!(record.children[0].causes[0].contains("<redacted:PSE_SECRET_TOKEN>"));
    assert_eq!(record.children[1].code.as_deref(), Some("host.deadline"));
    assert!(
        !serde_json::to_string(&record)
            .unwrap()
            .contains("private-value")
    );
}
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[error("related issue")]
#[diagnostic(
    code(test::leaf),
    url("https://example.test/\"quoted\""),
    help("correct the authored location")
)]
struct Leaf {
    #[label("non-ASCII label")]
    span: miette::SourceSpan,
}
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[error("outer issue")]
struct Parent {
    #[source_code]
    src: miette::NamedSource<String>,
    #[related]
    children: Vec<Leaf>,
}
#[test]
fn related_labels_inherit_source_identity_and_keep_byte_coordinates_and_quoted_urls() {
    let error = Parent {
        src: miette::NamedSource::new("模型.rs", "αβx".to_owned()),
        children: vec![Leaf {
            span: (4, 1).into(),
        }],
    };
    let record = ErrorRecord::from_diagnostic(&error, None, &RedactionPolicy::default());
    let child = &record.related[0];
    assert_eq!(child.labels[0].source.as_deref(), Some("模型.rs"));
    assert_eq!(child.labels[0].byte_offset, 4);
    assert_eq!(child.labels[0].column, Some(4));
    let text = serde_json::to_string(&record).unwrap();
    let restored: ErrorRecord = serde_json::from_str(&text).unwrap();
    assert_eq!(
        restored.related[0].url.as_deref(),
        Some("https://example.test/\"quoted\"")
    );
}
#[test]
fn early_format_selection_does_not_interpret_child_options() {
    let args = |values: &[&str]| {
        values
            .iter()
            .map(std::ffi::OsString::from)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        OutputFormat::early(
            &args(&["--format", "ndjson", "bad", "--other"]),
            None,
            false
        ),
        OutputFormat::Ndjson
    );
    assert_eq!(
        OutputFormat::early(
            &args(&["run", "child", "--format", "json"]),
            Some("human"),
            false
        ),
        OutputFormat::Human
    );
}
