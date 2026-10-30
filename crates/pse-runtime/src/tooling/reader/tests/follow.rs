// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use pse_diagnostics::observation::{ProducerRole, RecordField, Value};

fn read(request: Request) -> serde_json::Value {
    let response = encode(&super::request("observer", request)).unwrap();
    serde_json::from_slice(response.bytes()).unwrap()
}
fn next(response: &serde_json::Value) -> Request {
    serde_json::from_value(response["next_request"].clone()).unwrap()
}
#[test]
fn follow_preserves_source_coordinates_unchanged_gap_reset_and_terminal_failure() {
    let root = tempfile::tempdir().unwrap();
    let owner = Diagnostics::new(Policy::default(), root.path().to_owned());
    let operation = owner.hub().start("original native owner");
    let context = operation.context();
    let producer = context.producer(ProducerRole::Workflow, "performing owner");
    let record = producer.record(
        "fixture.v1",
        "preparation",
        None,
        None,
        vec![RecordField {
            name: "residual".into(),
            unit: "original_norm".into(),
            value: Value::number(f64::INFINITY),
            availability: None,
        }],
    );
    assert!(producer.publish(&record));
    owner.publish().unwrap();
    let source = CheckpointSelection::Retained {
        checkpoint: owner.checkpoint_path(),
        expected: None,
    };
    let first = read(Request::Follow {
        source,
        request: InspectionRequest::new(context.id()),
        invocation: None,
        resume: None,
    });
    let sample = &first["payload"]["sample"];
    assert_eq!(sample["continuity"], "initial");
    assert_eq!(sample["terminal"], false);
    assert_eq!(
        sample["cursor"]["producers"][0]["sequence"],
        record.sequence
    );
    let second = read(next(&first));
    assert_eq!(second["payload"]["sample"]["continuity"], "unchanged");
    let original = std::fs::read(owner.checkpoint_path()).unwrap();
    let mut image = read_checkpoint(&owner.checkpoint_path()).unwrap();
    image.sequence += 3;
    let performed = &mut image
        .operations
        .iter_mut()
        .find(|value| value.id == context.id())
        .unwrap()
        .producers[0];
    performed.current.as_mut().unwrap().sequence += 4;
    performed.recent.clear();
    std::fs::write(owner.checkpoint_path(), serde_json::to_vec(&image).unwrap()).unwrap();
    let skipped = read(next(&second));
    assert_eq!(skipped["payload"]["sample"]["skipped_publications"], 2);
    assert_eq!(skipped["payload"]["sample"]["gap"], true);
    image.sequence += 1;
    image
        .operations
        .iter_mut()
        .find(|value| value.id == context.id())
        .unwrap()
        .producers[0]
        .current
        .as_mut()
        .unwrap()
        .generation += 1;
    std::fs::write(owner.checkpoint_path(), serde_json::to_vec(&image).unwrap()).unwrap();
    let reset = read(next(&skipped));
    assert_eq!(reset["payload"]["sample"]["reset"], true);
    assert_eq!(reset["payload"]["sample"]["continuity"], "resynchronized");
    // Restore the publisher's own file before its next native publication.
    std::fs::write(owner.checkpoint_path(), original).unwrap();
    operation.finish(Outcome::Failed);
    owner.publish().unwrap();
    let terminal = read(next(&second));
    assert_eq!(terminal["payload"]["sample"]["terminal"], true);
    assert_eq!(
        terminal["payload"]["sample"]["observation"]["document"]["operation"]["outcome"],
        serde_json::json!(Outcome::Failed)
    );
    assert!(terminal["next_request"].is_null());
    owner.release().unwrap();
}
#[test]
fn follow_refuses_foreign_resume_and_reports_missing_source_without_next_request() {
    let root = tempfile::tempdir().unwrap();
    let owner = Diagnostics::new(Policy::default(), root.path().to_owned());
    let operation = owner.hub().start("still alive");
    let context = operation.context();
    owner.publish().unwrap();
    let first = read(Request::Follow {
        source: CheckpointSelection::Retained {
            checkpoint: owner.checkpoint_path(),
            expected: None,
        },
        request: InspectionRequest::new(context.id()),
        invocation: None,
        resume: None,
    });
    let Request::Follow {
        source,
        mut request,
        invocation,
        resume,
    } = next(&first)
    else {
        panic!("follow request")
    };
    request.window = std::time::Duration::from_secs(31);
    let refused = read(Request::Follow {
        source,
        request,
        invocation,
        resume,
    });
    assert_eq!(refused["category"], "refused");
    assert!(refused["next_request"].is_null());
    std::fs::remove_file(owner.checkpoint_path()).unwrap();
    let absent = read(next(&first));
    assert_eq!(absent["payload"]["kind"], "unavailable");
    assert!(absent["next_request"].is_null());
    // The original operation remains under its owner, without any observer control link.
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
