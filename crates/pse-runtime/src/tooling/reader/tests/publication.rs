// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded cross-process diagnostic publication inquiry; no solver, math runtime or database.
use super::*;
use pse_backend_native::solve::{Event, EventSource, Metric, Progress};
use pse_diagnostics::{DrainFailurePolicy, StderrCapture, SupervisedChild};
use std::{
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

const FIXTURE: &str = "PSE_READER_PUBLICATION_FIXTURE";
#[derive(Serialize, Deserialize)]
struct Evidence {
    checkpoint: PathBuf,
    operation: OperationId,
    event: crate::workflow::ProgressEventDocument,
    dropped: u64,
}
fn wait(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !path.exists() {
        assert!(
            Instant::now() < deadline,
            "fixture handoff {}",
            path.display()
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}
#[test]
fn checkpoint_publication_fixture() {
    let Some(root) = std::env::var_os(FIXTURE).map(PathBuf::from) else {
        return;
    };
    let owner = Diagnostics::new(
        Policy {
            record_capacity: 1,
            summary_retention: Duration::from_millis(200),
            ..Policy::default()
        },
        root.join("checkpoints"),
    );
    let operation = owner.hub().start("cooperating native progress owner");
    let context = operation.context();
    context.declare_summary("original numerical checkpoints, not permission");
    let _entered = context.enter();
    let progress = Progress::new(1);
    for ordinal in 1..=3 {
        progress.push(Event {
            source: Some(EventSource::new(
                "nonlinear_iteration",
                Some("physical_model"),
                Some("scaled_native"),
            )),
            phase: "native iteration".into(),
            elapsed: Duration::from_millis(ordinal),
            values: std::collections::BTreeMap::from([
                ("iteration".into(), Metric::Integer(ordinal as i64)),
                ("obj_value".into(), Metric::Real(17.25)),
                ("inf_pr".into(), Metric::Real(f64::INFINITY)),
                (
                    "native_missing".into(),
                    Metric::Unavailable(
                        pse_model::generated::enums::EvidenceUnavailableReason::NotComputed,
                    ),
                ),
            ]),
            incumbent: None,
        });
    }
    owner.publish().unwrap();
    let evidence = Evidence {
        checkpoint: owner.checkpoint_path(),
        operation: context.id(),
        event: crate::workflow::ProgressEventDocument::from(progress.current().unwrap()),
        dropped: progress.snapshot().1,
    };
    std::fs::write(
        root.join("source.json"),
        serde_json::to_vec(&evidence).unwrap(),
    )
    .unwrap();
    std::fs::write(root.join("active.ready"), []).unwrap();
    wait(&root.join("finish"));
    operation.finish(Outcome::Failed);
    owner.publish().unwrap();
    std::fs::write(root.join("terminal.ready"), []).unwrap();
    wait(&root.join("expire"));
    std::thread::sleep(Duration::from_millis(250));
    owner.publish().unwrap();
    std::fs::write(root.join("expired.ready"), []).unwrap();
    wait(&root.join("close"));
    owner.release().unwrap();
}
fn page(evidence: &Evidence, correlation: &str) -> serde_json::Value {
    let bytes = request(
        correlation,
        Request::Page {
            source: CheckpointSelection::Retained {
                checkpoint: evidence.checkpoint.clone(),
                expected: None,
            },
            operation_id: evidence.operation,
            section: Section::Records,
            producer: None,
            limit: 1,
            cursor: None,
        },
    );
    let reply = encode(&bytes).unwrap();
    assert_eq!(reply.exit_code(), 0);
    serde_json::from_slice(reply.bytes()).unwrap()
}
#[test]
fn separate_process_reads_existing_native_numerics_loss_and_expiry_without_new_publication() {
    let root = tempfile::tempdir().unwrap();
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--exact",
            "tooling::reader::tests::publication::checkpoint_publication_fixture",
            "--nocapture",
        ])
        .env(FIXTURE, root.path())
        .stdout(Stdio::null());
    let mut child = SupervisedChild::spawn(
        &mut command,
        DrainFailurePolicy::Park,
        Some(StderrCapture {
            bytes: 65536,
            stack_bytes: 256 * 1024,
        }),
    )
    .unwrap();
    wait(&root.path().join("active.ready"));
    let evidence: Evidence =
        serde_json::from_slice(&std::fs::read(root.path().join("source.json")).unwrap()).unwrap();
    let original = std::fs::read(&evidence.checkpoint).unwrap();
    let source = read_checkpoint(&evidence.checkpoint).unwrap();
    let observed = page(&evidence, "separate-reader-one");
    let data = &observed["payload"]["page"];
    let record = &data["records"]["records"][0];
    let native = evidence.event.source.as_ref().unwrap();
    assert_eq!(
        record["producer"],
        serde_json::to_value(native.producer.unwrap()).unwrap()
    );
    assert_eq!(record["generation"], native.producer_generation.unwrap());
    assert_eq!(record["sequence"], native.producer_sequence.unwrap());
    assert_eq!(
        record["family_sequence"],
        serde_json::to_value(native.producer_family_sequence).unwrap()
    );
    let fields = record["values"].as_array().unwrap();
    let objective = fields
        .iter()
        .find(|field| field["name"] == "obj_value")
        .unwrap();
    assert_eq!(objective["unit"], "native_objective");
    assert_eq!(objective["value"]["value"], 17.25);
    let residual = fields
        .iter()
        .find(|field| field["name"] == "inf_pr")
        .unwrap();
    assert_eq!(residual["unit"], "native_primal_residual");
    assert_eq!(residual["value"]["kind"], "NonFinite");
    assert_eq!(evidence.dropped, 2);
    assert_eq!(data["known_records"], 1);
    assert_eq!(data["total"], serde_json::Value::Null);
    assert_eq!(data["partial"], true);
    assert!(!data["clock"].is_null());
    let again = page(&evidence, "separate-reader-two");
    assert_eq!(again["payload"]["page"]["records"], data["records"]);
    assert_eq!(
        std::fs::read(&evidence.checkpoint).unwrap(),
        original,
        "observers never publish or rewrite the source"
    );
    std::fs::write(root.path().join("finish"), []).unwrap();
    wait(&root.path().join("terminal.ready"));
    let terminal = page(&evidence, "terminal-reader");
    let terminal = &terminal["payload"]["page"];
    assert_eq!(
        terminal["outcome"],
        serde_json::to_value(Outcome::Failed).unwrap()
    );
    assert!(!terminal["summary"]["expires_at"].is_null());
    assert_eq!(terminal["publication_sequence"], source.sequence + 1);
    std::fs::write(root.path().join("expire"), []).unwrap();
    wait(&root.path().join("expired.ready"));
    let expired = page(&evidence, "expired-reader");
    assert_eq!(expired["payload"]["kind"], "unavailable");
    let expired_source = read_checkpoint(&evidence.checkpoint).unwrap();
    assert_eq!(expired_source.sequence, source.sequence + 2);
    eprintln!(
        "bounded cross-process diagnostic inquiry: first checkpoint {} bytes; 3 existing native publications; Progress retention dropped {}; no added publisher or history files",
        original.len(),
        evidence.dropped
    );
    std::fs::write(root.path().join("close"), []).unwrap();
    child
        .wait_ready(None, Some(Instant::now() + Duration::from_secs(5)))
        .unwrap();
    assert!(child.wait().unwrap().success());
}
