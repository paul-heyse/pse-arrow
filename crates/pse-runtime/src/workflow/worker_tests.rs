// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The durable job queue from the runtime's side (Plan 22 O4), against isolated PostgreSQL
//! 18 databases. The worker binary's end-to-end journey is `tests/worker.rs`.
use super::durable_tests::quick;
use super::*;
use crate::math::settings::SolveSettings;
use pse_backend_native::solve::{Backend, SolveIntent};
use pse_operations::{
    jobs::{JobState, RetryPolicy},
    lifecycle::AttemptState,
    testing::TestDatabase,
};
use std::{collections::BTreeMap, path::Path, time::Duration};

pub(super) const SQUARE: &str = r#"package algebraic { def Root {
    var x:Scalar;
    eq square:x*x==4;
    annotation start x(1);
    annotation bounds x(0,10);
    annotation report x("root");
    annotation check x(x>1);
} }"#;

fn texts(root: &Path) -> BTreeMap<String, String> {
    let mut texts = BTreeMap::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else {
                let key = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .replace('\\', "/");
                texts.insert(key, std::fs::read_to_string(&path).unwrap());
            }
        }
    }
    texts
}

/// The authored sources of a one-model package over the physical primitives fixture.
pub(super) fn sources(source: &str) -> (BTreeMap<String, String>, BTreeMap<String, String>) {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/packages");
    let physical = texts(&fixtures.join("physical-primitives"));
    let mut manifest = std::fs::read_to_string(fixtures.join("minimal_explicit/package.toml"))
        .unwrap()
        .replace(r#"id_policy = "explicit""#, r#"id_policy = "named""#);
    manifest.push_str(
        "\n[[quantity_aliases]]\nname = \"Scalar\"\nquantity_type_id = \"1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f\"\n",
    );
    let modeling = BTreeMap::from([
        ("package.toml".to_owned(), manifest),
        ("models/root.pse".to_owned(), source.to_owned()),
    ]);
    (physical, modeling)
}

/// Store the sources of `source` and describe a job solving its `Root` case.
pub(super) async fn authored_job(
    runtime: &Runtime,
    source: &str,
    settings: SolveSettings,
) -> ModelingJob {
    let Durability::Durable(operations) = runtime.durability() else {
        panic!("a durable runtime stores sources");
    };
    let (physical, modeling) = sources(source);
    let cancel = crate::CancelSource::new();
    let context = runtime
        .physical_from_sources(&physical, &cancel)
        .await
        .unwrap();
    let package = runtime
        .package_from_sources(std::slice::from_ref(&modeling), context)
        .unwrap();
    let case = package
        .declarations()
        .iter()
        .find(|d| d.name == "Root")
        .unwrap()
        .declaration_id;
    ModelingJob {
        physical: operations.put_sources(&physical).await.unwrap(),
        modeling: vec![operations.put_sources(&modeling).await.unwrap()],
        case,
        route: pse_model::generated::enums::ModelingAnalysisRoute::Steady,
        settings,
        start: JobStart::Fresh,
        study: None,
    }
}

pub(super) fn ipopt() -> SolveSettings {
    SolveSettings {
        intent: SolveIntent::FeasiblePoint,
        backend: Some(Backend::Ipopt),
        presolve: pse_backend_native::presolve::PolicyKind::Off,
        ..SolveSettings::default()
    }
}

/// A durable runtime whose budget admits the default evaluation profile of a job.
pub(super) async fn job_durable(
    database: &TestDatabase,
    worker: &str,
    policy: LeasePolicy,
) -> Runtime {
    let operations = Operations::connect(database.url(), worker, policy)
        .await
        .unwrap();
    tests::job_runtime().with_durability(Durability::Durable(operations))
}

fn operations(runtime: &Runtime) -> &Operations {
    match runtime.durability() {
        Durability::Durable(operations) => operations,
        Durability::Ephemeral => panic!("ephemeral"),
    }
}

fn retry() -> RetryPolicy {
    RetryPolicy {
        max_tries: 3,
        backoff: Duration::ZERO,
        backoff_cap: Duration::ZERO,
    }
}

#[tokio::test]
async fn worker_runs_an_authored_job_and_stores_its_seed() {
    let database = TestDatabase::create().await.unwrap();
    let runtime = job_durable(&database, "worker-a", quick()).await;
    let job = authored_job(&runtime, SQUARE, ipopt()).await;
    let operations = operations(&runtime);
    let enqueued = operations
        .enqueue(&job, "square", retry(), 0)
        .await
        .unwrap();
    // The payload's sources round-trip through the store with verified hashes.
    assert_eq!(
        operations.sources(&job.modeling[0]).await.unwrap(),
        sources(SQUARE).1
    );
    let (processed, result) = runtime.work_once_with_result().await.unwrap();
    assert_eq!(
        processed.state(),
        Some(AttemptState::Completed),
        "{processed:?}"
    );
    let Processed::Ran { job: ran, record } = processed else {
        panic!()
    };
    assert_eq!(ran, enqueued.job_id());
    assert_eq!(record.attempt_id, enqueued.attempt_id());
    assert_eq!(record.solutions.len(), 1);
    // The claimed try runs as the run its job's attempts share: its result rows name
    // the run the store holds for the attempt.
    let run = record.attempt.as_ref().unwrap().run_id;
    let result = result.unwrap();
    assert_eq!(result.run_id, run);
    let lineage = &result.completion().unwrap().lineage;
    assert!(!lineage.is_empty() && lineage.iter().all(|row| row.run_id == run));
    let stored = operations.store().jobs().get(ran).await.unwrap();
    assert_eq!(stored.state, JobState::Completed);
    assert!(matches!(
        runtime.work_once().await.unwrap(),
        Processed::Idle
    ));
    drop(runtime);
    database.remove().await.unwrap();
}

#[tokio::test]
async fn unknown_payload_version_refused() {
    let database = TestDatabase::create().await.unwrap();
    let runtime = job_durable(&database, "worker-a", quick()).await;
    let operations = operations(&runtime);
    // A payload from a newer build: an unknown column version, and a known column version
    // whose document states another version. Both are written past the typed enqueue, as
    // another build would.
    let job = authored_job(&runtime, SQUARE, ipopt()).await;
    let mut restated =
        serde_json::to_value(JobPayload::new(JobTask::Modeling(Box::new(job.clone())))).unwrap();
    restated["version"] = serde_json::json!(4);
    // Version 2 described one modeling job at the top level; it is not interpreted.
    let mut former = serde_json::to_value(&job).unwrap();
    former["version"] = serde_json::json!(2);
    let mut enqueued = Vec::new();
    for (key, version, payload) in [
        (
            "future-column",
            JOB_PAYLOAD_VERSION + 1,
            serde_json::json!({ "from": "a newer build" }),
        ),
        ("former-column", 2, former),
        ("future-document", JOB_PAYLOAD_VERSION, restated),
    ] {
        enqueued.push(
            operations
                .store()
                .jobs()
                .enqueue(&pse_operations::jobs::NewJob {
                    attempt: pse_operations::attempts::NewAttempt {
                        attempt_id: pse_operations::mint_id(),
                        run_id: pse_operations::mint_id(),
                        kind: pse_operations::attempts::AttemptKind::Modeling,
                        request_identity: job.request_identity().unwrap(),
                        preparation_identity: None,
                        parent_attempt: None,
                    },
                    idempotency_key: key.to_owned(),
                    payload_version: version,
                    payload,
                    priority: 0,
                    retry: retry(),
                })
                .await
                .unwrap(),
        );
    }
    for (index, enqueued) in enqueued.into_iter().enumerate() {
        let processed = runtime.work_once().await.unwrap();
        let Processed::Ran { record, .. } = &processed else {
            panic!("{processed:?}")
        };
        assert_eq!(record.attempt_id, enqueued.attempt_id());
        let attempt = record.attempt.as_ref().unwrap();
        assert_eq!(attempt.state, AttemptState::Failed);
        // The typed cause, in the versioned termination detail.
        let detail: TerminationDetail =
            serde_json::from_str(attempt.termination_detail.as_deref().unwrap()).unwrap();
        let TerminationCause::Error { rule, message } = detail.cause else {
            panic!("{detail:?}")
        };
        if index < 2 {
            // The typed diagnostic code, with the violated named contract (X4).
            assert_eq!(
                pse_operations::attempts::TerminationCode::of(attempt).unwrap(),
                Some(pse_operations::attempts::TerminationCode::Rule(
                    pse_diagnostics::DiagnosticCode::ConfigInvalid
                ))
            );
            assert_eq!(rule, "workflow.job_payload_version");
        } else {
            assert!(message.contains("unknown document version 4"), "{message}");
        }
        // A refusal is not an infrastructure failure: it is not retried, whatever the policy.
        let job = operations
            .store()
            .jobs()
            .get(enqueued.job_id())
            .await
            .unwrap();
        assert_eq!(job.state, JobState::Failed);
        assert_eq!(job.attempt_id, enqueued.attempt_id());
    }
    assert!(matches!(
        runtime.work_once().await.unwrap(),
        Processed::Idle
    ));
    drop(runtime);
    database.remove().await.unwrap();
}

/// The request identity is framed from the typed payload, so two encodings of one job that
/// differ only in key order have one identity, and a changed setting changes it
/// (ADR-0116 Outcome 9).
#[test]
fn job_request_identity_independent_of_key_order() {
    fn reversed(value: serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::Object(map) => serde_json::Value::Object(
                map.into_iter()
                    .rev()
                    .map(|(key, value)| (key, reversed(value)))
                    .collect(),
            ),
            serde_json::Value::Array(items) => {
                serde_json::Value::Array(items.into_iter().map(reversed).collect())
            }
            other => other,
        }
    }
    let job = ModelingJob {
        physical: pse_ids::ContentHash::from_bytes([1; 32]),
        modeling: vec![pse_ids::ContentHash::from_bytes([2; 32])],
        case: pse_model::generated::identities::DeclarationId::from_bytes([3; 16]),
        route: pse_model::generated::enums::ModelingAnalysisRoute::Steady,
        settings: ipopt(),
        start: JobStart::Fresh,
        study: None,
    };
    let forward = serde_json::to_string(&job).unwrap();
    let backward = serde_json::to_string(&reversed(serde_json::to_value(&job).unwrap())).unwrap();
    assert_ne!(forward, backward);
    let decoded = |text: &str| serde_json::from_str::<ModelingJob>(text).unwrap();
    assert_eq!(
        decoded(&forward).request_identity().unwrap(),
        decoded(&backward).request_identity().unwrap()
    );
    assert_eq!(
        decoded(&forward).request_identity().unwrap(),
        job.request_identity().unwrap()
    );
    let mut changed = job.clone();
    changed.settings.controls.iterations += 1;
    assert_ne!(
        changed.request_identity().unwrap(),
        job.request_identity().unwrap()
    );
    let mut resumed = job.clone();
    resumed.start = JobStart::ResumeFromParent;
    assert_ne!(
        resumed.request_identity().unwrap(),
        job.request_identity().unwrap()
    );
    // A study point's binding is part of its request.
    let mut point = job.clone();
    point.study = Some(StudyPointBinding {
        study_id: pse_ids::SemanticId::from_bytes([4; 16]).into(),
        point_index: 1,
        binding_hash: pse_ids::ContentHash::from_bytes([5; 32]),
        overlay: PointOverlay::default(),
        predecessor: Some(0),
    });
    assert_ne!(
        point.request_identity().unwrap(),
        job.request_identity().unwrap()
    );
}

/// Payload version 3 is one typed document per task: a modeling job carries its study
/// point's binding and overlay, a finalization names its study; unknown fields, tasks and
/// versions are refused.
#[test]
fn job_payload_v3_is_typed_per_task() {
    let binding = StudyPointBinding {
        study_id: pse_ids::SemanticId::from_bytes([4; 16]).into(),
        point_index: 2,
        binding_hash: pse_ids::ContentHash::from_bytes([5; 32]),
        overlay: PointOverlay {
            values: BTreeMap::from([("feed.flow".to_owned(), 2.5)]),
            parameters: BTreeMap::from([(pse_ids::SemanticId::from_bytes([6; 16]), 0.25)]),
        },
        predecessor: Some(1),
    };
    let job = ModelingJob {
        physical: pse_ids::ContentHash::from_bytes([1; 32]),
        modeling: vec![],
        case: pse_model::generated::identities::DeclarationId::from_bytes([3; 16]),
        route: pse_model::generated::enums::ModelingAnalysisRoute::Steady,
        settings: ipopt(),
        start: JobStart::Fresh,
        study: Some(binding.clone()),
    };
    let value = serde_json::to_value(JobPayload::new(JobTask::Modeling(Box::new(job)))).unwrap();
    assert_eq!(value["version"], 3);
    assert_eq!(value["task"]["kind"], "modeling");
    assert_eq!(value["task"]["study"]["point_index"], 2);
    let decoded: JobPayload = serde_json::from_value(value.clone()).unwrap();
    let JobTask::Modeling(decoded) = decoded.task else {
        panic!("a modeling task")
    };
    assert_eq!(decoded.study, Some(binding));
    let finalization = serde_json::to_value(JobPayload::new(JobTask::StudyFinalization(
        StudyFinalization {
            study_id: pse_ids::SemanticId::from_bytes([4; 16]).into(),
        },
    )))
    .unwrap();
    assert_eq!(finalization["task"]["kind"], "study_finalization");
    assert!(serde_json::from_value::<JobPayload>(finalization.clone()).is_ok());
    for refused in [
        {
            let mut v = value.clone();
            v["version"] = serde_json::json!(2);
            v
        },
        {
            let mut v = value.clone();
            v["task"]["study"]["overlay"]["unknown"] = serde_json::json!(1);
            v
        },
        {
            let mut v = finalization;
            v["task"]["kind"] = serde_json::json!("reindex");
            v
        },
    ] {
        assert!(serde_json::from_value::<JobPayload>(refused).is_err());
    }
}

/// The termination detail is a typed, versioned document: its cause is tagged by kind, its
/// candidate uses are registry vocabularies, and another version is refused.
#[test]
fn termination_detail_versioned_and_typed() {
    let detail = TerminationDetail {
        version: pse_model::document::Version,
        cause: TerminationCause::Assessment {
            usable: false,
            candidate_use: vec![
                pse_model::generated::enums::CandidateUse::Usable,
                pse_model::generated::enums::CandidateUse::DiagnosticOnly,
            ],
        },
    };
    let value = serde_json::to_value(&detail).unwrap();
    assert_eq!(
        value,
        serde_json::json!({
            "version": 1,
            "cause": {"kind": "assessment", "usable": false, "candidate_use": ["usable", "diagnostic_only"]},
        })
    );
    assert_eq!(
        serde_json::from_value::<TerminationDetail>(value.clone()).unwrap(),
        detail
    );
    let mut future = value.clone();
    future["version"] = serde_json::json!(2);
    assert!(serde_json::from_value::<TerminationDetail>(future).is_err());
    let mut untyped = value;
    untyped["cause"]["candidate_use"] = serde_json::json!(["maybe"]);
    assert!(serde_json::from_value::<TerminationDetail>(untyped).is_err());
    // The source manifest is versioned the same way.
    let manifest = serde_json::json!({"version": 1, "paths": ["package.toml"]});
    assert!(serde_json::from_value::<SourceManifest>(manifest).is_ok());
    assert!(
        serde_json::from_value::<SourceManifest>(serde_json::json!({"paths": ["package.toml"]}))
            .is_err()
    );
}

/// A worker whose lease is renewed rarely: only the `LISTEN` path can stop a try quickly.
async fn slow_heartbeat_runtime(database: &TestDatabase) -> Runtime {
    let policy = LeasePolicy {
        lease: Duration::from_secs(120),
        heartbeat: Duration::from_secs(60),
        ..quick()
    };
    let operations = Operations::connect(database.url(), "worker-slow", policy)
        .await
        .unwrap();
    tests::job_runtime().with_durability(Durability::Durable(operations))
}

/// Enqueue a job, hold its source bundles behind a table lock, and let a worker claim it:
/// the try stays running (claimed, leased, watched) while its sources cannot be read.
async fn blocked_try(
    database: &TestDatabase,
    runtime: &Runtime,
) -> (
    pse_operations::attempts::AttemptId,
    store_lock::Lock,
    tokio::task::JoinHandle<Result<Processed, WorkflowError>>,
) {
    let job = authored_job(runtime, SQUARE, ipopt()).await;
    let enqueued = operations(runtime)
        .enqueue(&job, "blocked", retry(), 0)
        .await
        .unwrap();
    let lock = store_lock::Lock::source_bundles(database).await;
    let worker = runtime.clone();
    let task = tokio::spawn(async move { worker.work_once().await });
    // Running under the worker's lease, with its cancellation watcher listening.
    let store = database.store();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(20);
    loop {
        let running = store
            .attempts()
            .get(enqueued.attempt_id())
            .await
            .unwrap()
            .state
            == AttemptState::Running;
        if running && lock.listeners().await > 0 {
            break;
        }
        assert!(tokio::time::Instant::now() < deadline, "the try never ran");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    (enqueued.attempt_id(), lock, task)
}

/// A table lock held in an open transaction on its own connection, and an autocommit probe
/// (a transaction would see one cached `pg_stat_activity` snapshot).
mod store_lock {
    use pse_operations::testing::{Session, TestDatabase};

    pub(super) struct Lock {
        lock: Session,
        probe: Session,
    }

    /// The store's listener connection, by its application name (Plan 22 X8).
    fn listeners() -> String {
        format!(
            "FROM pg_stat_activity WHERE datname = current_database() \
             AND application_name = '{}'",
            pse_operations::LISTENER_APPLICATION
        )
    }

    impl Lock {
        pub(super) async fn source_bundles(database: &TestDatabase) -> Self {
            let lock = database.session().await.unwrap();
            lock.execute("BEGIN").await.unwrap();
            lock.execute("LOCK TABLE pse_ops.source_bundles IN ACCESS EXCLUSIVE MODE")
                .await
                .unwrap();
            Self {
                lock,
                probe: database.session().await.unwrap(),
            }
        }

        pub(super) async fn listeners(&self) -> i64 {
            self.probe
                .count(&format!("SELECT count(*) {}", listeners()))
                .await
                .unwrap()
        }

        /// Drop the store listener's connection; the listener reconnects.
        pub(super) async fn terminate_listeners(&self) {
            let terminated = self
                .probe
                .count(&format!(
                    "SELECT count(*) FILTER (WHERE pg_terminate_backend(pid)) {}",
                    listeners()
                ))
                .await
                .unwrap();
            assert!(terminated > 0, "no listener to terminate");
        }

        /// Record a cancellation request without its notification.
        pub(super) async fn set_cancel_silently(
            &self,
            attempt: pse_operations::attempts::AttemptId,
        ) {
            let changed = self
                .probe
                .execute(&format!(
                    "UPDATE pse_ops.attempts SET cancel_requested = true, \
                     cancel_requested_at = now() WHERE attempt_id = '{attempt}'"
                ))
                .await
                .unwrap();
            assert_eq!(changed, 1);
        }

        pub(super) async fn release(self) {
            self.lock.execute("ROLLBACK").await.unwrap();
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn cancel_notify_stops_running_job() {
    let database = TestDatabase::create().await.unwrap();
    let runtime = slow_heartbeat_runtime(&database).await;
    let (attempt, lock, task) = blocked_try(&database, &runtime).await;
    let requested = tokio::time::Instant::now();
    assert_eq!(
        database
            .store()
            .request_cancel(attempt, "test")
            .await
            .unwrap(),
        pse_operations::cancellation::CancelOutcome::Requested
    );
    let processed = tokio::time::timeout(Duration::from_secs(20), task)
        .await
        .expect("the notification stops the try long before the next heartbeat")
        .unwrap()
        .unwrap();
    assert!(requested.elapsed() < Duration::from_secs(30));
    lock.release().await;
    assert_eq!(
        processed.state(),
        Some(AttemptState::Cancelled),
        "{processed:?}"
    );
    let Processed::Ran { job, .. } = processed else {
        panic!()
    };
    let job = database.store().jobs().get(job).await.unwrap();
    assert_eq!(job.state, JobState::Cancelled);
    drop(runtime);
    database.remove().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn cancel_survives_listener_reconnect() {
    let database = TestDatabase::create().await.unwrap();
    let runtime = slow_heartbeat_runtime(&database).await;
    let (attempt, lock, task) = blocked_try(&database, &runtime).await;
    // The request's notification is lost: the flag is set without NOTIFY, then the
    // listener's connection drops. The watcher re-reads the authority after reconnecting.
    lock.set_cancel_silently(attempt).await;
    lock.terminate_listeners().await;
    let processed = tokio::time::timeout(Duration::from_secs(20), task)
        .await
        .expect("the re-read after the reconnect observes the durable request")
        .unwrap()
        .unwrap();
    lock.release().await;
    assert_eq!(
        processed.state(),
        Some(AttemptState::Cancelled),
        "{processed:?}"
    );
    drop(runtime);
    database.remove().await.unwrap();
}

/// The `job.start` event of a durable try's stream.
fn start_event(record: &DurableRecord) -> BTreeMap<String, pse_operations::streams::ProgressValue> {
    record
        .progress
        .as_ref()
        .unwrap()
        .iter()
        .find(|event| event.phase == "job.start")
        .unwrap()
        .values
        .clone()
}

/// A job's start policy is applied and recorded as the try's first progress event (Plan 22
/// G8): a resume without a parent attempt starts fresh and says why; a stored-solution
/// start seeds every free coordinate from that solution; an unknown solution fails the
/// try instead of starting fresh.
#[tokio::test]
async fn job_start_policies_applied_and_recorded() {
    use pse_operations::streams::ProgressValue as V;
    let database = TestDatabase::create().await.unwrap();
    let runtime = job_durable(&database, "worker-a", quick()).await;
    let operations = operations(&runtime);
    let mut job = authored_job(&runtime, SQUARE, ipopt()).await;
    job.start = JobStart::ResumeFromParent;
    operations
        .enqueue(&job, "resume-first-try", retry(), 0)
        .await
        .unwrap();
    let (processed, _) = runtime.work_once_with_result().await.unwrap();
    assert_eq!(
        processed.state(),
        Some(AttemptState::Completed),
        "{processed:?}"
    );
    let Processed::Ran { record, .. } = processed else {
        panic!()
    };
    let start = start_event(&record);
    assert_eq!(start["requested"], V::Text("resume_from_parent".into()));
    assert_eq!(
        start["fresh"],
        V::Text("first try: no parent attempt".into())
    );
    assert!(matches!(start["solution"], V::Unavailable(_)));
    let [(0, solution)] = record.solutions[..] else {
        panic!("{:?}", record.solutions)
    };

    job.start = JobStart::StoredSolution { solution };
    operations
        .enqueue(&job, "stored-solution", retry(), 0)
        .await
        .unwrap();
    let (processed, result) = runtime.work_once_with_result().await.unwrap();
    assert_eq!(
        processed.state(),
        Some(AttemptState::Completed),
        "{processed:?}"
    );
    let Processed::Ran { record, .. } = processed else {
        panic!()
    };
    let start = start_event(&record);
    assert_eq!(start["requested"], V::Text("stored_solution".into()));
    assert_eq!(start["solution"], V::Text(solution.to_string()));
    let result = result.unwrap();
    let RunRequest::Modeling(requests) = result.request() else {
        panic!()
    };
    assert!(requests[0].starts.values().all(|s| *s
        == StartSource::Stored {
            solution: solution.as_id()
        }));
    let Ok(RunReport::Modeling(steps)) = result.report() else {
        panic!()
    };
    let crate::math::solves::Outcome::Native(native) = &steps[0].outcome else {
        panic!()
    };
    assert!(native.start_receipt.as_ref().unwrap().submitted);

    job.start = JobStart::StoredSolution {
        solution: pse_operations::mint_id(),
    };
    operations
        .enqueue(&job, "unknown-solution", retry(), 0)
        .await
        .unwrap();
    let processed = runtime.work_once().await.unwrap();
    assert_eq!(
        processed.state(),
        Some(AttemptState::Failed),
        "{processed:?}"
    );
    drop((runtime, result));
    database.remove().await.unwrap();
}
