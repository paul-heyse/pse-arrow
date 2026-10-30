// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public reader protocol contracts; no canonical service or scientific provider is required.
use proptest::prelude::*;
use pse_diagnostics::observation::{
    OperationId, Outcome, Policy, ProducerRole, RecordField, Relationship, Value,
};
use pse_model::document::Version;
use pse_runtime::{
    observation::{Diagnostics, read_checkpoint},
    tooling::{inspection::InspectionRequest, reader::*},
};
use serde_json::{Value as Json, json};
use sha2::{Digest as _, Sha256};
use std::io::{self, Read};

fn request(correlation: &str, request: Request) -> Vec<u8> {
    serde_json::to_vec(&ReaderRequest {
        protocol: Protocol::ProductReader,
        version: Version,
        correlation: correlation.into(),
        request,
    })
    .unwrap()
}
fn response(bytes: &[u8]) -> Json {
    let encoded = encode(bytes).unwrap();
    assert_eq!(encoded.bytes().last(), Some(&b'\n'));
    let wire: Json = serde_json::from_slice(encoded.bytes()).unwrap();
    assert_eq!(
        encoded.exit_code(),
        if wire["category"] == "success" { 0 } else { 2 }
    );
    wire
}
fn fixture() -> (tempfile::TempDir, std::sync::Arc<Diagnostics>, OperationId) {
    let root = tempfile::tempdir().unwrap();
    let owner = Diagnostics::new(
        Policy {
            budget_bytes: 16 * 1024 * 1024,
            ..Policy::default()
        },
        root.path().to_owned(),
    );
    let operation = owner.hub().start("native preparation");
    let context = operation.context();
    context.gauge("native.temperature", "K", f64::NAN);
    for index in 0..3 {
        let child = context.child(format!("related {index}"), Relationship::DependencyWait);
        child.context().declare_summary("independent waiter");
        child.finish(Outcome::Completed);
    }
    let producer = context.producer(ProducerRole::Workflow, "original preparation");
    for index in 0..3 {
        let record = producer.record(
            "fixture.native.v1",
            "preparation",
            Some("original"),
            None,
            vec![RecordField {
                name: "temperature".into(),
                unit: "K".into(),
                value: match index {
                    0 => Value::number(f64::INFINITY),
                    1 => Value::Unavailable("owner did not supply a value".into()),
                    _ => Value::number(273.15),
                },
                availability: None,
            }],
        );
        assert!(producer.publish(&record));
    }
    let id = context.id();
    operation.finish(Outcome::Failed);
    owner.publish().unwrap();
    let mut image = read_checkpoint(&owner.checkpoint_path()).unwrap();
    image.clock = None;
    std::fs::write(owner.checkpoint_path(), serde_json::to_vec(&image).unwrap()).unwrap();
    (root, owner, id)
}
fn retained(owner: &Diagnostics) -> CheckpointSelection {
    CheckpointSelection::Retained {
        checkpoint: owner.checkpoint_path(),
        expected: None,
    }
}
#[test]
fn passive_protocol_preserves_units_nonfinite_and_original_failure() {
    let (_root, owner, id) = fixture();
    let path = owner.checkpoint_path();
    let before = std::fs::read(&path).unwrap();
    let wire = response(&request(
        "observer-one",
        Request::Inspect {
            source: retained(&owner),
            request: InspectionRequest::new(id),
            invocation: None,
        },
    ));
    assert_eq!(wire["category"], "success");
    let payload = &wire["payload"];
    assert_eq!(
        payload["document"]["operation"]["outcome"],
        json!(Outcome::Failed)
    );
    assert_eq!(payload["document"]["transport"]["partial"], true);
    assert_eq!(
        payload["process"]["process"],
        format!(
            "{:016x}-{:016x}",
            id.process_instance[0], id.process_instance[1]
        )
    );
    let expected_digest = Sha256::digest(&before)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(payload["source"]["identity"], expected_digest);
    assert_eq!(payload["source"]["atomic_snapshot"], false);
    assert!(payload["association"].is_null());
    let metric = payload["document"]["operation"]["metrics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|metric| metric["name"] == "native.temperature")
        .unwrap();
    assert_eq!(metric["unit"], "K");
    assert_eq!(metric["latest"]["kind"], "NonFinite");
    assert_eq!(std::fs::read(path).unwrap(), before);
    owner.release().unwrap();
}
#[test]
fn original_invocation_association_survives_distinct_observers() {
    let (root, owner, id) = fixture();
    let run = pse_model::generated::identities::RunId::from_bytes(Default::default());
    let attempt = pse_model::generated::identities::AttemptId::from_bytes(Default::default());
    let mut evidence = json!({"version":2,"run_id":run,"attempt_id":attempt,"operation":id,"family":"fixture","clock_scope":"original invocation","timing_unavailable":null,"canonical_run":"run:original","canonical_attempt":"attempt:original","outcome":"failed","endpoints":{},"intervals":[],"omitted_intervals":0,"products":[],"omitted_product_steps":0,"preparations":null,"attempts":{"run_id":run,"requested_steps":0,"steps":[],"native_attempts":[],"omitted_native_attempts":0,"native_record_coverage":"strategy_attempts","omitted_steps":0,"failure":null,"usable":false}});
    let invocation = root.path().join("invocation.evidence");
    std::fs::write(&invocation, serde_json::to_vec(&evidence).unwrap()).unwrap();
    for correlation in ["observer-a", "observer-b"] {
        let wire = response(&request(
            correlation,
            Request::Inspect {
                source: retained(&owner),
                request: InspectionRequest::new(id),
                invocation: Some(invocation.clone()),
            },
        ));
        assert_eq!(wire["correlation"], correlation);
        assert_eq!(wire["category"], "success");
        assert_eq!(wire["payload"]["association"]["run_id"], json!(run));
        assert_eq!(wire["payload"]["association"]["attempt_id"], json!(attempt));
        assert_eq!(wire["payload"]["association"]["outcome"], "failed");
    }
    evidence["operation"]["sequence"] = json!(id.sequence + 1);
    std::fs::write(&invocation, serde_json::to_vec(&evidence).unwrap()).unwrap();
    assert_eq!(
        response(&request(
            "observer-c",
            Request::Inspect {
                source: retained(&owner),
                request: InspectionRequest::new(id),
                invocation: Some(invocation)
            }
        ))["category"],
        "refused"
    );
    owner.release().unwrap();
}
#[test]
fn record_pages_keep_whole_records_and_reject_changed_continuations() {
    let (_root, owner, id) = fixture();
    let query = |section, cursor| {
        request(
            "page",
            Request::Page {
                source: retained(&owner),
                operation_id: id,
                section,
                producer: None,
                limit: 1,
                cursor,
            },
        )
    };
    let first = response(&query(Section::Records, None));
    let page = &first["payload"]["page"];
    let records = page["records"]["records"].as_array().unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["values"].as_array().unwrap().len(), 1);
    assert_eq!(records[0]["values"][0]["unit"], "K");
    let cursor: PageCursor = serde_json::from_value(page["continuation"].clone()).unwrap();
    assert_eq!(
        response(&query(Section::Records, Some(cursor.clone())))["category"],
        "success"
    );
    assert_eq!(
        response(&query(Section::Related, Some(cursor.clone())))["category"],
        "refused"
    );
    let mut image = read_checkpoint(&owner.checkpoint_path()).unwrap();
    image.sequence += 1;
    std::fs::write(owner.checkpoint_path(), serde_json::to_vec(&image).unwrap()).unwrap();
    assert_eq!(
        response(&query(Section::Records, Some(cursor)))["category"],
        "refused"
    );
    owner.release().unwrap();
}
#[test]
fn compact_terminal_record_keeps_tuple_and_partial_history() {
    let root = tempfile::tempdir().unwrap();
    let owner = Diagnostics::new(Policy::default(), root.path().to_owned());
    let operation = owner.hub().start("terminal owner");
    let context = operation.context();
    context.declare_summary("retained failed preparation");
    let producer = context.producer(ProducerRole::Workflow, "preparation owner");
    let original = producer.record(
        "fixture.native.v1",
        "setup",
        None,
        None,
        vec![RecordField {
            name: "pressure".into(),
            unit: "Pa".into(),
            value: Value::Unavailable("original owner input absent".into()),
            availability: None,
        }],
    );
    assert!(producer.publish(&original));
    let id = context.id();
    operation.finish(Outcome::Failed);
    owner.publish().unwrap();
    let image = read_checkpoint(&owner.checkpoint_path()).unwrap();
    assert!(image.operations[0].producers[0].recent.is_empty());
    let wire = response(&request(
        "compact",
        Request::Page {
            source: retained(&owner),
            operation_id: id,
            section: Section::Records,
            producer: Some(producer.id()),
            limit: 1,
            cursor: None,
        },
    ));
    let page = &wire["payload"]["page"];
    assert_eq!(page["records"]["records"][0], json!(original));
    assert_eq!(page["partial"], true);
    assert!(page["total"].is_null());
    assert_eq!(page["known_records"], 1);
    assert!(page["continuation"].is_null());
    assert_eq!(page["outcome"], json!(Outcome::Failed));
    assert_eq!(page["operation"], json!(id));
    assert_eq!(page["coverage"], json!(image.operations[0].coverage));
    owner.release().unwrap();
}
#[test]
fn process_catalog_preserves_bad_rows_and_rejects_changed_source() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("a.json"), b"invalid original checkpoint").unwrap();
    std::os::unix::fs::symlink(root.path().join("absent"), root.path().join("b.json")).unwrap();
    std::os::unix::fs::symlink(
        root.path().join("unrelated"),
        root.path().join("unrelated.tmp"),
    )
    .unwrap();
    let query = |directory, cursor| {
        request(
            "list",
            Request::Processes {
                directory,
                limit: 1,
                cursor,
            },
        )
    };
    let first = response(&query(root.path().to_owned(), None));
    let payload = &first["payload"];
    assert_eq!(payload["page"]["processes"].as_array().unwrap().len(), 1);
    assert!(!payload["page"]["processes"][0]["failure"].is_null());
    assert_eq!(payload["page"]["processes"][0]["partial"], true);
    assert_eq!(payload["source"]["examined_sources"], 2);
    let cursor: PageCursor = serde_json::from_value(payload["continuation"].clone()).unwrap();
    assert_eq!(
        response(&query(root.path().to_owned(), Some(cursor.clone())))["category"],
        "success"
    );
    std::fs::write(root.path().join("c.json"), b"new checkpoint").unwrap();
    assert_eq!(
        response(&query(root.path().to_owned(), Some(cursor)))["category"],
        "refused"
    );
    let missing = root.path().join("never-created");
    assert_eq!(
        response(&query(missing.clone(), None))["payload"]["kind"],
        "unavailable"
    );
    assert!(!missing.exists());
}
#[test]
fn invalid_protocol_version_and_process_identity_are_refused() {
    let (_root, owner, id) = fixture();
    let mut expected = owner.checkpoint_identity();
    expected.locator.start_ticks = expected.locator.start_ticks.map(|ticks| ticks + 1);
    let body = request(
        "identity",
        Request::Inspect {
            source: CheckpointSelection::Current {
                checkpoint: owner.checkpoint_path(),
                expected,
            },
            request: InspectionRequest::new(id),
            invocation: None,
        },
    );
    assert_eq!(response(&body)["category"], "refused");
    let mut value: Json = serde_json::from_slice(&body).unwrap();
    value["version"] = json!(9);
    assert_eq!(
        response(&serde_json::to_vec(&value).unwrap())["category"],
        "refused"
    );
    value["version"] = json!(1);
    value["protocol"] = json!("other-reader");
    assert_eq!(
        response(&serde_json::to_vec(&value).unwrap())["category"],
        "refused"
    );
    assert_eq!(response(b"not JSON")["category"], "refused");
    owner.release().unwrap();
}
#[test]
fn discovery_schema_and_framing_need_no_source_or_service() {
    let root = tempfile::tempdir().unwrap();
    assert_eq!(
        response(&request("discover", Request::Discover))["category"],
        "success"
    );
    for contract in [Contract::Request, Contract::Response] {
        let wire = response(&request("schema", Request::Schema { contract }));
        assert_eq!(
            wire["payload"]["document"]["$id"],
            wire["payload"]["identity"]
        );
        assert_eq!(
            schema(contract).as_value()["$id"],
            wire["payload"]["identity"]
        );
    }
    let mut frame = Vec::new();
    assert_eq!(
        serve(
            request("transport", Request::Discover).as_slice(),
            &mut frame
        )
        .unwrap(),
        0
    );
    assert_eq!(frame.last(), Some(&b'\n'));
    assert_eq!(
        serde_json::from_slice::<Json>(&frame).unwrap()["correlation"],
        "transport"
    );
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
}

#[test]
fn canonical_preflight_neither_connects_nor_creates_missing_state() {
    use pse_runtime::workflow::{AttemptObservationRequest, StudyObservationRequest};
    let root = tempfile::tempdir().unwrap();
    let state = root.path().join("never-created");
    let selection = || CanonicalReadSelection {
        state: state.clone(),
        database: None,
        memory_bytes: std::num::NonZeroUsize::new(256 << 20).unwrap(),
        timeout_ms: 5000,
    };
    let wire = response(&request(
        "invalid-history",
        Request::Study {
            canonical: selection(),
            request: StudyObservationRequest {
                study: "original-study".into(),
                maximum_members: 1,
                maximum_events: 0,
            },
            cursor: None,
        },
    ));
    assert_eq!(wire["correlation"], "invalid-history");
    assert_eq!(wire["category"], "refused");
    assert!(
        wire["payload"]["failure"]["message"]
            .as_str()
            .unwrap()
            .contains("allowance outside bounds")
    );
    let wire = response(&request(
        "missing-owner",
        Request::Attempt {
            canonical: selection(),
            request: AttemptObservationRequest {
                run: "original-run".into(),
                attempt: "original-attempt".into(),
                maximum_events: 1,
                cursor: None,
            },
        },
    ));
    assert_eq!(wire["category"], "refused");
    assert_eq!(wire["payload"]["kind"], "refusal");
    assert!(!state.exists());
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
    let mut invalid = selection();
    invalid.timeout_ms = 25_001;
    let wire = response(&request(
        "invalid-timeout",
        Request::Attempt {
            canonical: invalid,
            request: AttemptObservationRequest {
                run: "original-run".into(),
                attempt: "original-attempt".into(),
                maximum_events: 1,
                cursor: None,
            },
        },
    ));
    assert_eq!(wire["category"], "refused");
    assert!(
        wire["payload"]["failure"]["message"]
            .as_str()
            .unwrap()
            .contains("1..=25000ms")
    );
    assert!(!state.exists());
}

#[test]
fn discovery_operations_match_the_public_request_schema() {
    let wire = response(&request("capabilities", Request::Discover));
    let mut operations = wire["payload"]["operations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    let contract = schema(Contract::Request);
    let mut declared = contract.as_value()["$defs"]["Request"]["oneOf"]
        .as_array()
        .unwrap()
        .iter()
        .map(|variant| {
            variant["properties"]["operation"]["const"]
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect::<Vec<_>>();
    declared.sort();
    operations.sort();
    assert_eq!(operations, declared);
    assert!(operations.iter().any(|operation| operation == "study"));
    assert!(operations.iter().any(|operation| operation == "attempt"));
}

struct Chunks<'a> {
    remaining: &'a [u8],
    width: usize,
}
impl Read for Chunks<'_> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        let count = output.len().min(self.width).min(self.remaining.len());
        output[..count].copy_from_slice(&self.remaining[..count]);
        self.remaining = &self.remaining[count..];
        Ok(count)
    }
}
proptest! {
    #[test]
    fn transport_chunk_boundaries_preserve_frame_and_truncated_requests_refuse(width in 1usize..128, removed in 1usize..20) {
        let bytes = request("chunked", Request::Discover);
        let expected = encode(&bytes).unwrap();
        let mut output = Vec::new();
        let code = serve(Chunks { remaining: &bytes, width }, &mut output).unwrap();
        prop_assert_eq!(code, expected.exit_code());
        prop_assert_eq!(output, expected.bytes());
        let truncated = &bytes[..bytes.len().saturating_sub(removed)];
        let mut output = Vec::new();
        prop_assert_eq!(serve(Chunks { remaining: truncated, width }, &mut output).unwrap(), 2);
        let wire: Json = serde_json::from_slice(&output).unwrap();
        prop_assert_eq!(&wire["category"], "refused");
    }
}
