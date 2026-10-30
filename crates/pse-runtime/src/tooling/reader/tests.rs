// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::observation::{Diagnostics, read_checkpoint};
use pse_diagnostics::observation::{
    Outcome, Policy, ProducerRole, RecordField, Relationship, Value,
};
use serde_json::json;

fn request(correlation: &str, request: Request) -> Vec<u8> {
    serde_json::to_vec(&ReaderRequest {
        protocol: Protocol::ProductReader,
        version: Version,
        correlation: correlation.into(),
        request,
    })
    .unwrap()
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
                value: if index == 0 {
                    Value::number(f64::INFINITY)
                } else if index == 1 {
                    Value::Unavailable("owner did not supply a value".into())
                } else {
                    Value::number(273.15)
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
#[test]
fn passive_protocol_preserves_native_statements_units_nonfinite_and_original_failure() {
    let (_root, owner, id) = fixture();
    let path = owner.checkpoint_path();
    let before = std::fs::read(&path).unwrap();
    let native_request = InspectionRequest::new(id);
    let direct = inspection::inspect_checkpoint(&path, native_request.clone()).unwrap();
    let response = dispatch(&request(
        "observer-one",
        Request::Inspect {
            source: CheckpointSelection::Retained {
                checkpoint: path.clone(),
                expected: None,
            },
            request: native_request,
            invocation: None,
        },
    ));
    assert_eq!(response.category, Category::Success);
    let Payload::Inspection {
        document,
        process,
        source,
        association,
    } = &response.payload
    else {
        panic!("inspection")
    };
    assert_eq!(
        serde_json::to_value(document).unwrap(),
        serde_json::to_value(direct).unwrap()
    );
    assert_eq!(
        document.operation.as_ref().unwrap().outcome,
        Some(Outcome::Failed)
    );
    assert!(
        document.transport.partial,
        "missing clock mapping remains explicit"
    );
    assert_eq!(
        process.process,
        format!(
            "{:016x}-{:016x}",
            id.process_instance[0], id.process_instance[1]
        )
    );
    assert_eq!(source.identity, hash(&before));
    assert!(!source.atomic_snapshot);
    assert!(association.is_none());
    let wire = serde_json::to_value(&response).unwrap();
    let metric = wire["payload"]["document"]["operation"]["metrics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|metric| metric["name"] == "native.temperature")
        .unwrap();
    assert_eq!(metric["unit"], "K");
    assert_eq!(metric["latest"]["kind"], "NonFinite");
    assert_eq!(
        std::fs::read(&path).unwrap(),
        before,
        "readers never rewrite or cancel their source"
    );
    owner.release().unwrap();
}
#[test]
fn original_invocation_association_survives_distinct_observer_correlations() {
    let (root, owner, id) = fixture();
    let run = pse_model::generated::identities::RunId::from_bytes(Default::default());
    let attempt = pse_model::generated::identities::AttemptId::from_bytes(Default::default());
    let evidence = json!({"version":2,"run_id":run,"attempt_id":attempt,"operation":id,"family":"fixture","clock_scope":"original invocation","timing_unavailable":null,"canonical_run":"run:original","canonical_attempt":"attempt:original","outcome":"failed","endpoints":{},"intervals":[],"omitted_intervals":0,"products":[],"omitted_product_steps":0,"preparations":null,"attempts":{"run_id":run,"requested_steps":0,"steps":[],"native_attempts":[],"omitted_native_attempts":0,"native_record_coverage":"strategy_attempts","omitted_steps":0,"failure":null,"usable":false}});
    let invocation = root.path().join("invocation.evidence");
    std::fs::write(&invocation, serde_json::to_vec(&evidence).unwrap()).unwrap();
    for correlation in ["observer-a", "observer-b"] {
        let response = dispatch(&request(
            correlation,
            Request::Inspect {
                source: CheckpointSelection::Retained {
                    checkpoint: owner.checkpoint_path(),
                    expected: None,
                },
                request: InspectionRequest::new(id),
                invocation: Some(invocation.clone()),
            },
        ));
        assert_eq!(response.correlation, correlation);
        assert_eq!(response.category, Category::Success);
        let Payload::Inspection {
            association: Some(association),
            ..
        } = response.payload
        else {
            panic!("validated original association")
        };
        assert_eq!(association.run_id, run);
        assert_eq!(association.attempt_id, Some(attempt));
        assert_eq!(association.outcome, InvocationOutcome::Failed);
    }
    let mut changed = evidence;
    changed["operation"]["sequence"] = json!(id.sequence + 1);
    std::fs::write(&invocation, serde_json::to_vec(&changed).unwrap()).unwrap();
    assert_eq!(
        dispatch(&request(
            "observer-c",
            Request::Inspect {
                source: CheckpointSelection::Retained {
                    checkpoint: owner.checkpoint_path(),
                    expected: None
                },
                request: InspectionRequest::new(id),
                invocation: Some(invocation)
            }
        ))
        .category,
        Category::Refused
    );
    owner.release().unwrap();
}
#[test]
fn source_bound_record_pages_keep_whole_records_and_reject_changed_continuations() {
    let (_root, owner, id) = fixture();
    let selection = CheckpointSelection::Retained {
        checkpoint: owner.checkpoint_path(),
        expected: None,
    };
    let first = dispatch(&request(
        "page-one",
        Request::Page {
            source: selection.clone(),
            operation_id: id,
            section: Section::Records,
            producer: None,
            limit: 1,
            cursor: None,
        },
    ));
    let Payload::Page { page } = first.payload else {
        panic!("record page")
    };
    let Records::Records(records) = page.records else {
        panic!("coherent records")
    };
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].values.len(), 1);
    assert_eq!(records[0].values[0].unit, "K");
    let cursor = page.continuation.expect("next source record");
    let next = dispatch(&request(
        "page-two",
        Request::Page {
            source: selection.clone(),
            operation_id: id,
            section: Section::Records,
            producer: None,
            limit: 1,
            cursor: Some(cursor.clone()),
        },
    ));
    assert_eq!(next.category, Category::Success);
    let unrelated = dispatch(&request(
        "wrong-query",
        Request::Page {
            source: selection.clone(),
            operation_id: id,
            section: Section::Related,
            producer: None,
            limit: 1,
            cursor: Some(cursor.clone()),
        },
    ));
    assert_eq!(unrelated.category, Category::Refused);
    let mut image = read_checkpoint(&owner.checkpoint_path()).unwrap();
    image.sequence += 1;
    std::fs::write(owner.checkpoint_path(), serde_json::to_vec(&image).unwrap()).unwrap();
    assert_eq!(
        dispatch(&request(
            "stale-page",
            Request::Page {
                source: selection,
                operation_id: id,
                section: Section::Records,
                producer: None,
                limit: 1,
                cursor: Some(cursor)
            }
        ))
        .category,
        Category::Refused
    );
    owner.release().unwrap();
}
#[test]
fn compact_terminal_current_record_keeps_whole_tuple_and_partial_history() {
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
    let response = dispatch(&request(
        "compact-terminal",
        Request::Page {
            source: CheckpointSelection::Retained {
                checkpoint: owner.checkpoint_path(),
                expected: None,
            },
            operation_id: id,
            section: Section::Records,
            producer: Some(producer.id()),
            limit: 1,
            cursor: None,
        },
    ));
    let Payload::Page { page } = response.payload else {
        panic!("terminal current page")
    };
    let Records::Records(records) = page.records else {
        panic!("whole records")
    };
    assert_eq!(
        serde_json::to_value(&records[0]).unwrap(),
        serde_json::to_value(&original).unwrap()
    );
    assert!(page.partial);
    assert_eq!(page.total, None);
    assert_eq!(page.known_records, 1);
    assert!(page.continuation.is_none());
    assert_eq!(page.outcome, Some(Outcome::Failed));
    assert_eq!(page.operation, id);
    assert_eq!(
        serde_json::to_value(page.coverage).unwrap(),
        serde_json::to_value(&image.operations[0].coverage).unwrap()
    );
    owner.release().unwrap();
}
#[test]
fn discovery_preserves_independent_bad_rows_and_source_generation() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("a.json"), b"invalid original checkpoint").unwrap();
    std::os::unix::fs::symlink(
        root.path().join("absent-source"),
        root.path().join("b.json"),
    )
    .unwrap();
    std::os::unix::fs::symlink(
        root.path().join("unrelated-absent"),
        root.path().join("unrelated.tmp"),
    )
    .unwrap();
    let response = dispatch(&request(
        "list-a",
        Request::Processes {
            directory: root.path().to_owned(),
            limit: 1,
            cursor: None,
        },
    ));
    let Payload::Processes {
        page,
        source,
        continuation,
    } = response.payload
    else {
        panic!("catalogue")
    };
    assert_eq!(page.processes.len(), 1);
    assert!(page.processes[0].failure.is_some());
    assert!(page.processes[0].partial);
    assert_eq!(source.examined_sources, 2);
    let cursor = continuation.unwrap();
    assert_eq!(
        dispatch(&request(
            "list-b",
            Request::Processes {
                directory: root.path().to_owned(),
                limit: 1,
                cursor: Some(cursor.clone())
            }
        ))
        .category,
        Category::Success
    );
    std::fs::write(root.path().join("c.json"), b"another original checkpoint").unwrap();
    assert_eq!(
        dispatch(&request(
            "changed",
            Request::Processes {
                directory: root.path().to_owned(),
                limit: 1,
                cursor: Some(cursor)
            }
        ))
        .category,
        Category::Refused
    );
    let missing = root.path().join("never-created");
    assert!(matches!(
        dispatch(&request(
            "missing",
            Request::Processes {
                directory: missing.clone(),
                limit: 1,
                cursor: None
            }
        ))
        .payload,
        Payload::Unavailable { .. }
    ));
    assert!(!missing.exists());
}
#[test]
fn checked_domain_refuses_version_protocol_and_reused_start_before_inspection() {
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
    assert_eq!(dispatch(&body).category, Category::Refused);
    let mut wrong_version: serde_json::Value = serde_json::from_slice(&body).unwrap();
    wrong_version["version"] = json!(9);
    assert_eq!(
        dispatch(&serde_json::to_vec(&wrong_version).unwrap()).category,
        Category::Refused
    );
    wrong_version["version"] = json!(1);
    wrong_version["protocol"] = json!("other-reader");
    assert_eq!(
        dispatch(&serde_json::to_vec(&wrong_version).unwrap()).category,
        Category::Refused
    );
    assert_eq!(dispatch(b"not JSON").category, Category::Refused);
    owner.release().unwrap();
}
#[test]
fn discover_schema_and_framing_do_not_touch_sources_or_start_services() {
    let root = tempfile::tempdir().unwrap();
    let discovery = dispatch(&request("discover", Request::Discover));
    assert_eq!(discovery.category, Category::Success);
    for contract in [Contract::Request, Contract::Response] {
        let response = dispatch(&request("schema", Request::Schema { contract }));
        let Payload::Schema { identity, document } = response.payload else {
            panic!("native schema")
        };
        assert_eq!(
            document.get("$id").and_then(serde_json::Value::as_str),
            Some(identity)
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
    let response: serde_json::Value = serde_json::from_slice(&frame).unwrap();
    assert_eq!(response["correlation"], "transport");
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
}
