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

fn texts(root: &Path) -> BTreeMap<String, Vec<u8>> {
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
                texts.insert(key, std::fs::read(&path).unwrap());
            }
        }
    }
    texts
}

/// A manifest dependency on the physical primitives fixture package.
pub(super) const PRIMITIVES: &str = r#"dependencies = [{ package_id = "5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a", version_req = { operator = "exact", major = 1, minor = 0, patch = 0 } }]"#;
/// The authored sources of a one-model package over the physical primitives fixture.
pub(super) fn sources(source: &str) -> (BTreeMap<String, Vec<u8>>, BTreeMap<String, Vec<u8>>) {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/packages");
    let physical = texts(&fixtures.join("physical-primitives"));
    let manifest = std::fs::read_to_string(fixtures.join("minimal_explicit/package.toml"))
        .unwrap()
        .replace(r#"id_policy = "explicit""#, r#"id_policy = "named""#)
        // The primitives declare `Scalar`; depending on them makes it visible (ADR-0123
        // Outcome 6).
        .replace("dependencies = []", PRIMITIVES);
    let modeling = BTreeMap::from([
        ("package.toml".to_owned(), manifest.into_bytes()),
        ("models/root.pse".to_owned(), source.as_bytes().to_vec()),
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

#[cfg_attr(
    not(feature = "native-solvers"),
    ignore = "needs the linked native solvers"
)]
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
    // Unknown column and document versions are written past typed enqueue, as another
    // build would. Neither historical scientific inputs nor malformed current inputs
    // are reinterpreted.
    let job = authored_job(&runtime, SQUARE, ipopt()).await;
    let mut restated =
        serde_json::to_value(JobPayload::new(JobTask::Modeling(Box::new(job.clone())))).unwrap();
    restated["version"] = serde_json::json!(5);
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
        ("historical-column", 5, restated.clone()),
        ("historical-document", JOB_PAYLOAD_VERSION, restated),
        (
            "malformed-document",
            JOB_PAYLOAD_VERSION,
            serde_json::json!({}),
        ),
    ] {
        let queued = operations
            .store()
            .jobs()
            .enqueue(&pse_operations::jobs::NewJob {
                attempt: pse_operations::attempts::NewAttempt {
                    attempt_id: pse_operations::mint_id(),
                    run_id: pse_operations::mint_id(),
                    kind: pse_operations::attempts::AttemptKind::Modeling,
                    operational_job_identity:
                        pse_ids::roles::RecordedOperationalJobIdentity::current(
                            job.operational_job_identity(key).unwrap(),
                        ),
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
            .unwrap();
        let retained = operations
            .store()
            .jobs()
            .get(queued.job_id())
            .await
            .unwrap();
        enqueued.push((queued, retained.payload_version, retained.payload));
    }
    for (index, (enqueued, version, payload)) in enqueued.into_iter().enumerate() {
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
        let TerminationCause::Error { diagnostic } = detail.cause else {
            panic!("{detail:?}")
        };
        if index < 4 {
            // The typed diagnostic code, with the violated named contract (X4).
            assert_eq!(
                pse_operations::attempts::TerminationCode::of(attempt).unwrap(),
                Some(pse_operations::attempts::TerminationCode::Rule(
                    pse_diagnostics::DiagnosticCode::ConfigInvalid
                ))
            );
            assert_eq!(
                diagnostic.rule,
                pse_diagnostics::DiagnosticRule::WorkflowJobPayloadVersion
            );
        } else {
            assert_eq!(
                diagnostic.rule,
                pse_diagnostics::DiagnosticRule::WorkflowInput
            );
            assert!(!diagnostic.observations.is_empty());
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
        assert_eq!(job.payload_version, version);
        assert_eq!(job.payload, payload);
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
    };
    let forward = serde_json::to_string(&job).unwrap();
    let backward = serde_json::to_string(&reversed(serde_json::to_value(&job).unwrap())).unwrap();
    assert_ne!(forward, backward);
    let decoded = |text: &str| serde_json::from_str::<ModelingJob>(text).unwrap();
    assert_eq!(
        decoded(&forward).operational_job_identity("scope").unwrap(),
        decoded(&backward)
            .operational_job_identity("scope")
            .unwrap()
    );
    assert_eq!(
        decoded(&forward).operational_job_identity("scope").unwrap(),
        job.operational_job_identity("scope").unwrap()
    );
    assert_ne!(
        job.operational_job_identity("scope").unwrap(),
        job.operational_job_identity("another-scope").unwrap()
    );
    let mut changed = job.clone();
    changed.settings.controls.iterations += 1;
    assert_ne!(
        changed.operational_job_identity("scope").unwrap(),
        job.operational_job_identity("scope").unwrap()
    );
    let mut resumed = job.clone();
    resumed.start = JobStart::ResumeFromParent;
    assert_ne!(
        resumed.operational_job_identity("scope").unwrap(),
        job.operational_job_identity("scope").unwrap()
    );
}

#[test]
fn study_job_v8_codec_unit_retains_binding_policy_and_operation() {
    use pse_model::study::*;
    let hash = pse_ids::ContentHash::from_bytes([1; 32]);
    let binding = AdmittedBinding {
        revision: hash.into(),
        context: hash,
        entries: BTreeMap::new(),
    };
    let operation = StudyOperation {
        version: pse_model::document::Version,
        admitted_horizon: None,
        source: OperationSource {
            revision: hash.into(),
            physical_context: hash,
        },
        preparation: PreparationSettings::default(),
        operation: OperationRequest::DeclaredCase(CaseOperation {
            case: pse_model::generated::identities::DeclarationId::from_bytes([3; 16]),
            route: pse_model::generated::enums::ModelingAnalysisRoute::Steady,
            settings: ipopt(),
        }),
    };
    let point = StudyPointBinding {
        study_id: pse_operations::mint_id(),
        point_index: 9,
        binding_hash: binding.identity(),
        binding,
        policy: PointPolicy {
            key: OccurrenceKey(9),
            dependencies: vec![Dependency::Ordering(OccurrenceKey(3))],
            seed_need: SeedNeed::Required,
            start: StartPolicy::Fresh,
            attempt_limit: 1,
        },
        operation,
    };
    let payload = JobPayload::new(JobTask::StudyOperation(Box::new(StudyOperationJob {
        physical: hash,
        modeling: vec![hash],
        point,
    })));
    let encoded = serde_json::to_value(payload).unwrap();
    assert_eq!(encoded["version"], 8);
    assert_eq!(encoded["task"]["point"]["operation"]["version"], 4);
    assert_eq!(encoded["task"]["kind"], "study_operation");
    let decoded: JobPayload = serde_json::from_value(encoded.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), encoded);
    for broken in [
        {
            let mut value = encoded.clone();
            value["version"] = serde_json::json!(5);
            value
        },
        {
            let mut value = encoded.clone();
            value["version"] = serde_json::json!(7);
            value
        },
        {
            let mut value = encoded.clone();
            value["task"]["point"]["binding"]["values"] = serde_json::json!({"x":2});
            value
        },
        {
            let mut value = encoded;
            value["task"]["point"]["policy"]["start"]["kind"] =
                serde_json::json!("implicit_fallback");
            value
        },
    ] {
        assert!(serde_json::from_value::<JobPayload>(broken).is_err());
    }
}

#[test]
fn termination_detail_v2_codec_unit_preserves_diagnostic_start_and_attempt_lifecycle() {
    use pse_model::study::*;
    let diagnostic = WorkflowError::Input("bad study binding".into()).boundary_diagnostic();
    let point = PointAttemptOutcome {
        attempt_id: Some(pse_operations::mint_id()),
        lifecycle: Some(AttemptState::Failed),
        diagnostic: Some(diagnostic.clone()),
        scientific: ScientificFacts::default(),
        start: None,
        effect: EffectState::Absent,
    };
    let detail = TerminationDetail {
        version: pse_model::document::Version,
        cause: TerminationCause::Error { diagnostic },
        point: Some(point),
        retry_failure: Some(RetryFailure::Deterministic),
        effect: EffectState::Absent,
    };
    let value = serde_json::to_value(detail).unwrap();
    assert_eq!(value["version"], 2);
    assert_eq!(value["point"]["lifecycle"], "failed");
    assert!(value["cause"]["diagnostic"]["rule"].is_string());
    let decoded: TerminationDetail = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), value);
    let mut future = value;
    future["version"] = serde_json::json!(1);
    assert!(serde_json::from_value::<TerminationDetail>(future).is_err());
    assert!(
        serde_json::from_value::<SourceManifest>(
            serde_json::json!({"version":1,"paths":["package.toml"]})
        )
        .is_ok()
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
#[cfg_attr(
    not(feature = "native-solvers"),
    ignore = "needs the linked native solvers"
)]
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

#[test]
fn automatic_job_payload_roundtrip_retains_constraints_and_typed_identity() {
    use pse_model::strategy::{AccuracyClass, CompositionPolicy, StartOrigin, WorkLimits};
    let mut settings = ipopt();
    settings.composition.limits = Some(WorkLimits {
        attempts: 4,
        evaluations: Some(50),
        iterations: Some(30),
        factorizations: None,
        proof_steps: Some(128),
    });
    settings.composition.recovery = vec![StartOrigin::Auxiliary];
    settings.reconstruction = Some(pse_backend_native::derived::ReconstructionAccuracy {
        point: 1e-8,
        action: 1e-7,
        class: AccuracyClass::Certified,
        refinement: pse_math::derived::RefinementLimits {
            rounds: 3,
            proof_cells: 128,
        },
    });
    let job = ModelingJob {
        physical: pse_ids::ContentHash::from_bytes([1; 32]),
        modeling: vec![pse_ids::ContentHash::from_bytes([2; 32])],
        case: pse_model::generated::identities::DeclarationId::from_bytes([3; 16]),
        route: pse_model::generated::enums::ModelingAnalysisRoute::Steady,
        settings,
        start: JobStart::Fresh,
    };
    let payload = JobPayload::new(JobTask::Modeling(Box::new(job.clone())));
    let encoded = serde_json::to_value(&payload).unwrap();
    assert_eq!(encoded["version"], JOB_PAYLOAD_VERSION);
    let decoded: JobPayload = serde_json::from_value(encoded.clone()).unwrap();
    assert_eq!(serde_json::to_value(&decoded).unwrap(), encoded);
    let JobTask::Modeling(decoded) = decoded.task else {
        panic!("actual modeling payload required");
    };
    assert_eq!(decoded.settings.composition.policy, CompositionPolicy::Auto);
    assert_eq!(decoded.settings.composition, job.settings.composition);
    assert_eq!(
        serde_json::to_value(decoded.settings.reconstruction).unwrap(),
        serde_json::to_value(job.settings.reconstruction).unwrap()
    );
    assert_eq!(
        decoded.operational_job_identity("automatic-case").unwrap(),
        job.operational_job_identity("automatic-case").unwrap()
    );
    let mut changed = job.clone();
    changed
        .settings
        .composition
        .limits
        .as_mut()
        .unwrap()
        .evaluations = Some(49);
    assert_ne!(
        changed.operational_job_identity("automatic-case").unwrap(),
        job.operational_job_identity("automatic-case").unwrap()
    );
    let mut changed = job.clone();
    changed
        .settings
        .reconstruction
        .as_mut()
        .unwrap()
        .refinement
        .proof_cells = 127;
    assert_ne!(
        changed.operational_job_identity("automatic-case").unwrap(),
        job.operational_job_identity("automatic-case").unwrap()
    );
    let mut undeclared = job.clone();
    undeclared.settings.composition = Default::default();
    undeclared.settings.reconstruction = None;
    assert_ne!(
        undeclared
            .operational_job_identity("automatic-case")
            .unwrap(),
        job.operational_job_identity("automatic-case").unwrap()
    );
    let encoded =
        serde_json::to_value(JobPayload::new(JobTask::Modeling(Box::new(undeclared)))).unwrap();
    let decoded: JobPayload = serde_json::from_value(encoded).unwrap();
    let JobTask::Modeling(decoded) = decoded.task else {
        panic!("modeling payload required");
    };
    assert_eq!(decoded.settings.composition, Default::default());
    assert!(decoded.settings.reconstruction.is_none());
}
