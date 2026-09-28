// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The durable job queue from the runtime's side (Plan 22 O4), against isolated PostgreSQL
//! 18 databases. The worker binary's end-to-end journey is `tests/worker.rs`.
use super::durable_tests::quick;
use super::*;
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
    profile: JobProfile,
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
        profile,
    }
}

pub(super) fn ipopt() -> JobProfile {
    JobProfile {
        intent: SolveIntent::FeasiblePoint,
        backend: Some(Backend::Ipopt),
        presolve: JobPresolve::Off,
        ..JobProfile::default()
    }
}

/// A durable runtime whose budget admits the default evaluation profile of a job.
async fn job_durable(database: &TestDatabase, worker: &str, policy: LeasePolicy) -> Runtime {
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
        .enqueue_modeling(&job, "square", retry(), 0)
        .await
        .unwrap();
    // The payload's sources round-trip through the store with verified hashes.
    assert_eq!(
        operations.sources(&job.modeling[0]).await.unwrap(),
        sources(SQUARE).1
    );
    let processed = runtime.work_once().await.unwrap();
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
    let enqueued = operations
        .enqueue(
            MODELING_JOB_VERSION + 1,
            serde_json::json!({ "from": "a newer build" }),
            "future",
            retry(),
            0,
        )
        .await
        .unwrap();
    let processed = runtime.work_once().await.unwrap();
    let Processed::Ran { record, .. } = &processed else {
        panic!("{processed:?}")
    };
    let attempt = record.attempt.as_ref().unwrap();
    assert_eq!(attempt.state, AttemptState::Failed);
    assert_eq!(
        attempt.termination_rule.as_deref(),
        Some("workflow.job_payload_version")
    );
    // A refusal is not an infrastructure failure: it is not retried, whatever the policy.
    let job = operations
        .store()
        .jobs()
        .get(enqueued.job_id())
        .await
        .unwrap();
    assert_eq!(job.state, JobState::Failed);
    assert_eq!(job.attempt_id, enqueued.attempt_id());
    assert!(matches!(
        runtime.work_once().await.unwrap(),
        Processed::Idle
    ));
    drop(runtime);
    database.remove().await.unwrap();
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
    sqlx_lock::Lock,
    tokio::task::JoinHandle<Result<Processed, WorkflowError>>,
) {
    let job = authored_job(runtime, SQUARE, ipopt()).await;
    let enqueued = operations(runtime)
        .enqueue_modeling(&job, "blocked", retry(), 0)
        .await
        .unwrap();
    let lock = sqlx_lock::Lock::source_bundles(database).await;
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
mod sqlx_lock {
    use pse_operations::testing::{Session, TestDatabase};

    pub(super) struct Lock {
        lock: Session,
        probe: Session,
    }

    const LISTENERS: &str = "FROM pg_stat_activity \
        WHERE datname = current_database() AND query LIKE 'LISTEN%pse_ops_cancel%'";

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
                .count(&format!("SELECT count(*) {LISTENERS}"))
                .await
                .unwrap()
        }

        /// Drop every cancellation listener's connection; the listeners reconnect.
        pub(super) async fn terminate_listeners(&self) {
            let terminated = self
                .probe
                .count(&format!(
                    "SELECT count(*) FILTER (WHERE pg_terminate_backend(pid)) {LISTENERS}"
                ))
                .await
                .unwrap();
            assert!(terminated > 0, "no listener to terminate");
        }

        /// Record a cancellation request without its notification.
        pub(super) async fn set_cancel_silently(&self, attempt: pse_operations::attempts::AttemptId) {
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
