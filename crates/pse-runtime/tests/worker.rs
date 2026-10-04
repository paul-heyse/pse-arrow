// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The `pse-worker` binary end to end (Plan 22 O4, O6, G8): a job is enqueued in an
//! isolated operational store, the worker runs it in a child process and records its
//! attempt, and another process reuses the seed the worker stored. A worker killed in a
//! long SCIP solve is resumed from its stored incumbent, and a cancellation from another
//! process stops SCIP. Run with `just worker-test`.
// Match the library's depth for the shared runtime's nested async `Send` proof.
#![recursion_limit = "256"]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "integration assertions and the child-process protocol"
)]

use pse_backend_native::{
    presolve::PolicyKind,
    solve::{Backend, Metric, SolveIntent},
};
use pse_operations::{
    attempts::{AttemptId, NativeTermination, RuntimeTermination, TerminationCode},
    cancellation::CancelOutcome,
    jobs::{Finished, JobState, RetryPolicy},
    lifecycle::AttemptState,
    streams::ProgressValue,
    testing::TestDatabase,
};
use pse_runtime::{
    CancelSource, SharedRuntime,
    authoring_driver::document::{OwnedDocumentSet, load_package_documents_owned},
    math::{settings::SolveSettings, solves::Outcome},
    workflow::{
        Durability, JobStart, LeasePolicy, ModelingJob, ModelingPackage, Operations,
        PhysicalContext, Processed, RunDurability, RunReport, RunRequest, Runtime, StartSource,
        StoredStart,
    },
};
use std::{collections::BTreeMap, num::NonZeroUsize, path::Path, sync::Arc};

const SQUARE: &str = r#"package algebraic { def Root {
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

fn sources() -> (BTreeMap<String, Vec<u8>>, BTreeMap<String, Vec<u8>>) {
    sources_of(SQUARE)
}

/// The sources of a one-document package over the physical primitives fixture.
fn sources_of(source: &str) -> (BTreeMap<String, Vec<u8>>, BTreeMap<String, Vec<u8>>) {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/packages");
    let manifest = std::fs::read_to_string(fixtures.join("minimal_explicit/package.toml"))
        .unwrap()
        .replace(r#"id_policy = "explicit""#, r#"id_policy = "named""#)
        // The primitives declare `Scalar`; depending on them makes it visible (ADR-0123
        // Outcome 6).
        .replace(
            "dependencies = []",
            r#"dependencies = [{ package_id = "5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a", version_req = { operator = "exact", major = 1, minor = 0, patch = 0 } }]"#,
        );
    (
        texts(&fixtures.join("physical-primitives")),
        BTreeMap::from([
            ("package.toml".to_owned(), manifest.into_bytes()),
            ("models/root.pse".to_owned(), source.as_bytes().to_vec()),
        ]),
    )
}

fn runtime() -> (Arc<SharedRuntime>, Runtime) {
    let n = |v| NonZeroUsize::new(v).unwrap();
    let shared = SharedRuntime::build(pse_runtime::ResourceBudget {
        memory_limit_bytes: n(1 << 30),
        spill_dir: std::env::temp_dir(),
        max_temp_dir_bytes: 1 << 30,
        top_consumers: n(5),
        threads: pse_engine::ThreadBudget {
            pool_threads: n(2),
            target_partitions: n(1),
        },
        execution: Default::default(),
        cache: pse_runtime::DeltaCacheBudget::disabled(1024),
        math: pse_runtime::math::MathPolicy {
            // Default compilation reserves its declared scratch allowance; the positive
            // deployment also admits the original evaluator and result buffers.
            worker_bytes: pse_compiler::workspace::Profile::default()
                .evaluation
                .scratch_bytes
                + (64 << 20),
            workspace_bytes: 128 << 20,
            foreign_bytes: 1 << 20,
            ..Default::default()
        },
        hashing_may_use_pool: false,
    })
    .unwrap();
    let registry = pse_schema::shared_registry().unwrap();
    let sessions = Arc::new(
        shared
            .session_factory(pse_engine::session::native_engine_profile())
            .unwrap(),
    );
    (
        shared.clone(),
        Runtime::from_shared(shared, registry, sessions),
    )
}

async fn package(
    shared: &SharedRuntime,
    runtime: &Runtime,
    physical: &BTreeMap<String, Vec<u8>>,
    modeling: &BTreeMap<String, Vec<u8>>,
) -> (ModelingPackage, PhysicalContext) {
    let pool = shared.pool();
    let cancel = pse_columnar::CancellationToken::new();
    let validation = runtime
        .sessions()
        .validation_context(runtime.registry())
        .unwrap();
    let load = |texts| {
        load_package_documents_owned(
            texts,
            runtime.registry(),
            pse_authoring::ParseBudget::default(),
            &pool,
            &cancel,
            &validation,
        )
        .unwrap()
    };
    let physical =
        OwnedDocumentSet::try_from_bundles(vec![load(physical)], &pool, &cancel).unwrap();
    let context = runtime
        .physical_from_documents(&physical, &cancel)
        .await
        .unwrap();
    let modeling =
        OwnedDocumentSet::try_from_bundles(vec![load(modeling)], &pool, &cancel).unwrap();
    (
        runtime
            .modeling_from_documents(&modeling, context.clone())
            .unwrap(),
        context,
    )
}

fn settings() -> SolveSettings {
    SolveSettings {
        intent: SolveIntent::FeasiblePoint,
        backend: Some(Backend::Ipopt),
        presolve: PolicyKind::Off,
        ..SolveSettings::default()
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn worker_runs_authored_case_end_to_end() {
    let database = TestDatabase::create().await.unwrap();
    let operations = Operations::connect(database.url(), "enqueuer", LeasePolicy::default())
        .await
        .unwrap();
    let (physical, modeling) = sources();
    let (shared, local) = runtime();
    let local = local.with_durability(Durability::Durable(operations.clone()));
    let (package, _) = package(&shared, &local, &physical, &modeling).await;
    let case = package
        .declarations()
        .iter()
        .find(|d| d.name == "Root")
        .unwrap()
        .declaration_id;
    let job = ModelingJob {
        physical: operations.put_sources(&physical).await.unwrap(),
        modeling: vec![operations.put_sources(&modeling).await.unwrap()],
        case,
        route: pse_model::generated::enums::ModelingAnalysisRoute::Steady,
        settings: settings(),
        start: JobStart::Fresh,
    };
    let enqueued = operations
        .enqueue(&job, "square-end-to-end", RetryPolicy::ONCE, 0)
        .await
        .unwrap();

    // The worker binary, in its own process, claims and runs the job, then stops.
    let url = database.url().to_owned();
    let status = tokio::task::spawn_blocking(move || {
        std::process::Command::new(env!("CARGO_BIN_EXE_pse-worker"))
            .args([
                "--url",
                &url,
                "--name",
                "worker-child",
                "--until-idle",
                "--heartbeat-ms",
                "200",
                "--memory-mib",
                "8192",
                "--threads",
                "2",
            ])
            .status()
            .unwrap()
    })
    .await
    .unwrap();
    assert!(status.success(), "{status}");

    let store = operations.store();
    let stored = store.jobs().get(enqueued.job_id()).await.unwrap();
    let attempt = store.attempts().get(enqueued.attempt_id()).await.unwrap();
    assert_eq!(stored.state, JobState::Completed, "{attempt:?}");
    assert_eq!(attempt.state, AttemptState::Completed);
    assert_eq!(attempt.worker.as_deref(), Some("worker-child"));
    assert_eq!(
        TerminationCode::of(&attempt).unwrap().as_ref(),
        Some(&TerminationCode::Native(NativeTermination::Success))
    );
    let history: Vec<_> = store
        .attempts()
        .history(attempt.attempt_id)
        .await
        .unwrap()
        .iter()
        .map(|t| t.to_state)
        .collect();
    assert_eq!(
        history,
        [
            AttemptState::Planned,
            AttemptState::Queued,
            AttemptState::Running,
            AttemptState::Completed
        ]
    );
    assert!(
        !store
            .streams()
            .snapshot(attempt.attempt_id)
            .await
            .unwrap()
            .is_empty(),
        "the worker streamed the solve's progress"
    );

    stored_seed_reused_across_processes(&operations, &package, &job).await;
    drop((local, shared, package, operations));
    database.remove().await.unwrap();
}

/// A package whose `Root` squares to a value its data document supplies (ADR-0125).
const DATA_SQUARE: &str = r#"package algebraic {
    entity kind source provenance { attribute title: Text; }
    enum role { given }
    entity source s { title = "KR9 durable job" }
    table target[n: 1..1]: Scalar storage {dimensionless} complete_over(n in 1..1) missing required;
    dataset targets: target provenance(s, role.given) from "data/target.parquet";
    def Root {
        var x:Scalar;
        eq square:x*x==target[1];
        annotation start x(1);
        annotation bounds x(0,10);
        annotation report x("root");
    }
}"#;

/// The Parquet bytes of `target`: one row, `n = 1`, `value = 9`.
fn target_document() -> Vec<u8> {
    use datafusion::arrow::{
        array::{ArrayRef, Float64Array, Int64Array, RecordBatch},
        datatypes::{DataType, Field, Schema},
    };
    let schema = Arc::new(Schema::new(vec![
        Field::new("n", DataType::Int64, false),
        Field::new("value", DataType::Float64, false),
    ]));
    let columns: Vec<ArrayRef> = vec![
        Arc::new(Int64Array::from(vec![1])),
        Arc::new(Float64Array::from(vec![9.0])),
    ];
    let batch = RecordBatch::try_new(Arc::clone(&schema), columns).unwrap();
    let mut bytes = Vec::new();
    let mut writer = parquet::arrow::ArrowWriter::try_new(&mut bytes, schema, None).unwrap();
    writer.write(&batch).unwrap();
    writer.close().unwrap();
    bytes
}

/// ADR-0125, review F09: a package with a data document is stored as a durable source
/// bundle whose binary document round-trips byte for byte and is verified against its
/// hashes; the worker binary loads it, admits the document's rows and solves the case.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn durable_job_round_trips_a_package_with_a_data_document() {
    let database = TestDatabase::create().await.unwrap();
    let operations = Operations::connect(database.url(), "enqueuer", LeasePolicy::default())
        .await
        .unwrap();
    let (physical, mut modeling) = sources_of(DATA_SQUARE);
    modeling.insert("data/target.parquet".to_owned(), target_document());
    let (shared, local) = runtime();
    let local = local.with_durability(Durability::Durable(operations.clone()));
    let (package, _) = package(&shared, &local, &physical, &modeling).await;
    let case = package
        .declarations()
        .iter()
        .find(|d| d.name == "Root")
        .unwrap()
        .declaration_id;
    let job = ModelingJob {
        physical: operations.put_sources(&physical).await.unwrap(),
        modeling: vec![operations.put_sources(&modeling).await.unwrap()],
        case,
        route: pse_model::generated::enums::ModelingAnalysisRoute::Steady,
        settings: settings(),
        start: JobStart::Fresh,
    };
    // The stored bundle is the package's exact bytes, its Parquet document included.
    assert_eq!(
        operations.sources(&job.modeling[0]).await.unwrap(),
        modeling
    );
    let enqueued = operations
        .enqueue(&job, "data-document-square", RetryPolicy::ONCE, 0)
        .await
        .unwrap();
    let url = database.url().to_owned();
    let status = tokio::task::spawn_blocking(move || {
        std::process::Command::new(env!("CARGO_BIN_EXE_pse-worker"))
            .args([
                "--url",
                &url,
                "--name",
                "worker-data",
                "--until-idle",
                "--heartbeat-ms",
                "200",
                "--memory-mib",
                "8192",
                "--threads",
                "2",
            ])
            .status()
            .unwrap()
    })
    .await
    .unwrap();
    assert!(status.success(), "{status}");
    let store = operations.store();
    let attempt = store.attempts().get(enqueued.attempt_id()).await.unwrap();
    assert_eq!(
        store.jobs().get(enqueued.job_id()).await.unwrap().state,
        JobState::Completed,
        "{attempt:?}"
    );
    assert_eq!(
        TerminationCode::of(&attempt).unwrap().as_ref(),
        Some(&TerminationCode::Native(NativeTermination::Success))
    );
    drop((local, shared, package, operations));
    database.remove().await.unwrap();
}

/// This process reuses the seed the worker process stored for the same preparation.
async fn stored_seed_reused_across_processes(
    operations: &Operations,
    package: &ModelingPackage,
    job: &ModelingJob,
) {
    let cancel = CancelSource::new();
    let analysis = package
        .declared_execution(
            job.case,
            Default::default(),
            job.settings.clone().profile().unwrap(),
            Default::default(),
            pse_modeling::Limits::default(),
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(analysis.route, job.route);
    let prepared = package.prepare_declared(&analysis, &cancel).await.unwrap();
    let seeded = prepared
        .with_stored_start(operations, StoredStart::Latest)
        .await
        .unwrap();
    let solution = seeded
        .starts
        .values()
        .find_map(|s| match s {
            StartSource::Stored { solution } => Some(*solution),
            _ => None,
        })
        .expect("the start sources name the stored solution");
    let stored = operations
        .store()
        .solutions()
        .get(solution.into())
        .await
        .unwrap()
        .unwrap();
    assert_ne!(stored.created_by, None);
    let result = seeded.start().unwrap().wait().await.unwrap();
    assert!(result.usable());
    let RunDurability::Durable(record) = result.durability() else {
        panic!("durable")
    };
    assert_eq!(
        record.attempt.as_ref().unwrap().state,
        AttemptState::Completed
    );
    // The native start receipt shows the stored seed was submitted.
    let RunReport::Modeling(steps) = result.report().unwrap() else {
        panic!()
    };
    let Outcome::Native(native) = &steps[0].outcome else {
        panic!()
    };
    let receipt = native.start_receipt.as_ref().unwrap();
    assert!(receipt.submitted);
    let seed = receipt.seed.as_ref().unwrap();
    assert_eq!(seed.compatibility.layout, stored.compatibility_stamp);
    // Its identity is in the result's lineage: the stored seed changes the request identity.
    let unseeded = package.prepare_declared(&analysis, &cancel).await.unwrap();
    let plain = unseeded.start().unwrap().wait().await.unwrap();
    let lineage =
        |r: &pse_runtime::workflow::RunResult| r.completion().unwrap().lineage[0].request_identity;
    assert_ne!(lineage(&result), lineage(&plain));
}

// ------------------------------------------------------ durable long solves (G8) --

/// The physical primitives with a dimensionless indicator kind and type, so an authored
/// model can declare binary decisions (ADR-0103: a discrete domain needs a count or
/// indicator quantity).
fn physical_with_indicator() -> BTreeMap<String, Vec<u8>> {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/packages");
    let mut physical = texts(&fixtures.join("physical-primitives"));
    let bytes = physical.get_mut("materials/physical.yaml").unwrap();
    let document = String::from_utf8(std::mem::take(bytes)).unwrap();
    let zero = r#"{"num": 0, "den": 1}"#;
    let dimension = [zero; 8].join(", ");
    let kind = format!(
        r#""quantity_kinds": [
    {{"quantity_kind_id": "18181818181818181818181818181818", "name": "indicator",
      "dimension": [{dimension}], "extensive": false, "addition_kind": "additive",
      "category": "indicator", "doc": "Zero-or-one decisions."}},"#
    );
    let ty = r#""quantity_types": [
    {"quantity_type_id": "1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c", "name": "Indicator",
      "quantity_kind_id": "18181818181818181818181818181818", "basis_id": null,
      "reference_state_id": null, "scale_kind": "point", "shape": [], "subject_kind": null,
      "canonical_unit_id": "0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a", "nominal_magnitude": null,
      "doc": "Zero-or-one decision."},"#;
    *bytes = document
        .replacen(r#""quantity_kinds": ["#, &kind, 1)
        .replacen(r#""quantity_types": ["#, ty, 1)
        .into_bytes();
    physical
}

/// A market-split problem with a squared deviation: `min Σᵢ sᵢ²` where
/// `sᵢ = Σⱼ aᵢⱼ·xⱼ − bᵢ` over 30 binary decisions and four rows, `bᵢ = ⌊Σⱼ aᵢⱼ / 2⌋`.
/// SCIP finds incumbents at once (the empty split is one) but proving optimality takes far
/// longer than any test allows, so a time limit ends every try. Deterministic coefficients.
fn market_split() -> String {
    const ROWS: usize = 4;
    const COLUMNS: usize = 30;
    let mut state: u64 = 0x9e37_79b9_7f4a_7c15;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state % 100
    };
    let mut body = String::new();
    for j in 0..COLUMNS {
        body.push_str(&format!(
            "var x{j}: Indicator in binary; annotation start x{j}(0{{dimensionless}}); "
        ));
    }
    let mut deviation = Vec::new();
    for i in 0..ROWS {
        let a: Vec<u64> = (0..COLUMNS).map(|_| next()).collect();
        let b = a.iter().sum::<u64>() / 2;
        let terms: Vec<String> = a
            .iter()
            .enumerate()
            .map(|(j, a)| format!("{a}*x{j}"))
            .collect();
        body.push_str(&format!(
            "var s{i}: Scalar; annotation bounds s{i}(-5000, 5000); annotation start s{i}(0); \
             eq split{i}: {} - s{i} == {b}; ",
            terms.join(" + ")
        ));
        deviation.push(format!("s{i}*s{i}"));
    }
    format!(
        "package algebraic {{ def Root {{ {body}\
         let deviation: Scalar = {}; \
         annotation objective deviation(minimize); annotation report deviation(\"deviation\"); }} }}",
        deviation.join(" + ")
    )
}

/// The modeling sources of `source`, which sees the primitives' `Scalar` and `Indicator`.
fn discrete_sources(source: &str) -> BTreeMap<String, Vec<u8>> {
    let (_, mut modeling) = sources();
    modeling.insert("models/root.pse".to_owned(), source.as_bytes().to_vec());
    modeling
}

/// The long SCIP job: the market split, optimized by SCIP under a time limit.
async fn long_scip_job(
    shared: &SharedRuntime,
    local: &Runtime,
    operations: &Operations,
    start: JobStart,
    time_limit: std::time::Duration,
) -> ModelingJob {
    let physical = physical_with_indicator();
    let modeling = discrete_sources(&market_split());
    let (package, _) = package(shared, local, &physical, &modeling).await;
    let case = package
        .declarations()
        .iter()
        .find(|d| d.name == "Root")
        .unwrap()
        .declaration_id;
    let mut settings = SolveSettings {
        intent: SolveIntent::Optimize,
        backend: Some(Backend::Scip),
        ..SolveSettings::default()
    };
    settings.controls.time_limit = time_limit;
    ModelingJob {
        physical: operations.put_sources(&physical).await.unwrap(),
        modeling: vec![operations.put_sources(&modeling).await.unwrap()],
        case,
        route: pse_model::generated::enums::ModelingAnalysisRoute::Steady,
        settings,
        start,
    }
}

/// Spawn the worker binary against `url`, serving until idle with a short lease.
fn spawn_worker(url: &str, name: &str, lease_seconds: u64) -> std::process::Child {
    std::process::Command::new(env!("CARGO_BIN_EXE_pse-worker"))
        .args([
            "--url",
            url,
            "--name",
            name,
            "--until-idle",
            "--lease-seconds",
            &lease_seconds.to_string(),
            "--heartbeat-ms",
            "200",
            "--memory-mib",
            "8192",
            "--threads",
            "2",
        ])
        .spawn()
        .unwrap()
}

/// Poll `probe` every 100 ms until it holds, for at most `limit`.
async fn until<F: Future<Output = bool>>(
    limit: std::time::Duration,
    what: &str,
    mut probe: impl FnMut() -> F,
) {
    let deadline = tokio::time::Instant::now() + limit;
    while !probe().await {
        assert!(tokio::time::Instant::now() < deadline, "timed out: {what}");
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

/// Refuse to keep waiting on an attempt that already ended: its search is over.
async fn searching(operations: &Operations, attempt: AttemptId) {
    let stored = operations.store().attempts().get(attempt).await.unwrap();
    assert!(
        !matches!(
            stored.state,
            AttemptState::Completed
                | AttemptState::Partial
                | AttemptState::Failed
                | AttemptState::Cancelled
        ),
        "the try ended before its search was observed: {stored:?}"
    );
}

/// The stored incumbents of an attempt, in order: objective and captured solution.
async fn incumbents(database: &TestDatabase, attempt: AttemptId) -> Vec<(f64, Option<String>)> {
    let session = database.session().await.unwrap();
    session
        .texts(&format!(
            "SELECT objective::text, solution_id::text FROM pse_ops.incumbents \
             WHERE attempt_id = '{attempt}'::uuid ORDER BY seq"
        ))
        .await
        .unwrap()
        .into_iter()
        .map(|row| (row[0].as_deref().unwrap().parse().unwrap(), row[1].clone()))
        .collect()
}

/// A worker process dies in a long SCIP solve once an incumbent is stored (Plan 22 S16):
/// its lease expires, the stale sweep requeues the job as a new attempt whose parent is the
/// dead one, and the next worker resumes from the stored incumbent: its start sources name
/// the solution, SCIP stores it as a start, and its result is no worse.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn killed_worker_attempt_goes_stale_and_resumes_from_incumbent() {
    let database = TestDatabase::create().await.unwrap();
    let policy = LeasePolicy {
        lease: std::time::Duration::from_secs(2),
        heartbeat: std::time::Duration::from_millis(200),
        ..LeasePolicy::default()
    };
    let operations = Operations::connect(database.url(), "enqueuer", policy)
        .await
        .unwrap();
    let (shared, local) = scip_runtime();
    let job = long_scip_job(
        &shared,
        &local,
        &operations,
        JobStart::ResumeFromParent,
        // Original seed preparation and qualification share the finite task allowance
        // with SCIP. Leave room for both while retaining the complete lifecycle bound.
        std::time::Duration::from_secs(30),
    )
    .await;
    let enqueued = operations
        .enqueue(
            &job,
            "market-split",
            RetryPolicy {
                max_tries: 2,
                backoff: std::time::Duration::ZERO,
                backoff_cap: std::time::Duration::ZERO,
            },
            0,
        )
        .await
        .unwrap();
    let first = enqueued.attempt_id();

    // The first worker runs the solve in its own process; once an incumbent with its
    // solution is stored, the process is killed.
    let started = tokio::time::Instant::now();
    let mut worker = spawn_worker(database.url(), "worker-killed", 2);
    until(
        std::time::Duration::from_secs(40),
        "a captured incumbent",
        || async {
            searching(&operations, first).await;
            incumbents(&database, first)
                .await
                .iter()
                .any(|(_, s)| s.is_some())
        },
    )
    .await;
    worker.kill().unwrap();
    let status = worker.wait().unwrap();
    assert!(!status.success(), "{status}");
    let stored = incumbents(&database, first).await;
    let (objective, solution) = stored
        .iter()
        .rev()
        .find_map(|(objective, solution)| solution.clone().map(|s| (*objective, s)))
        .unwrap();
    let store = operations.store();
    assert_eq!(
        store.attempts().get(first).await.unwrap().state,
        AttemptState::Running,
        "the killed process never ended its try"
    );

    // The lease expires: the sweep marks the attempt stale and requeues the job.
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(20);
    let requeued = loop {
        let recovery = operations.recover().await.unwrap();
        if let Some(requeue) = recovery.requeued.iter().find(|r| r.stale_attempt == first) {
            break requeue.outcome;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "timed out: the stale sweep"
        );
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    };
    let Finished::Requeued {
        attempt_id: second, ..
    } = requeued
    else {
        panic!("{requeued:?}")
    };
    assert_eq!(
        store.attempts().get(first).await.unwrap().state,
        AttemptState::Superseded
    );
    assert_eq!(
        store.attempts().get(second).await.unwrap().parent_attempt,
        Some(first)
    );

    // The next worker claims the new attempt and resumes from the stored incumbent.
    let resumer = local
        .clone()
        .with_durability(Durability::Durable(Operations::from_store(
            store.clone(),
            "worker-resume",
            policy,
        )));
    let (processed, result) = resumer.work_once_with_result().await.unwrap();
    let Processed::Ran { record, .. } = &processed else {
        panic!("{processed:?}")
    };
    assert_eq!(record.attempt_id, second);
    let result = result.unwrap_or_else(|| panic!("the resumed try did not run: {record:?}"));
    // The attempt's stream records the resume and the solution it started from.
    let progress = record.progress.as_ref().unwrap();
    let start = progress.iter().find(|e| e.phase == "job.start").unwrap();
    assert_eq!(
        start.values["requested"],
        ProgressValue::Text("resume_from_parent".into())
    );
    assert_eq!(
        start.values["solution"],
        ProgressValue::Text(solution.replace('-', ""))
    );
    let RunRequest::Modeling(requests) = result.request() else {
        panic!()
    };
    let solution_id = requests[0]
        .starts
        .values()
        .find_map(|s| match s {
            StartSource::Stored { solution } => Some(*solution),
            _ => None,
        })
        .expect("the start sources name the stored incumbent");
    assert_eq!(solution_id.to_string(), solution.replace('-', ""));
    let RunReport::Modeling(steps) = result.report().unwrap() else {
        panic!()
    };
    let Outcome::Native(native) = &steps[0].outcome else {
        panic!("{:?}", steps[0].outcome)
    };
    assert_eq!(native.backend, Backend::Scip);
    assert_eq!(
        native.metrics.get("scip.incumbent.stored"),
        Some(&Metric::Bool(true))
    );
    assert!(native.start_receipt.as_ref().unwrap().submitted);
    // Its result is no worse than the incumbent it resumed from.
    let resumed = native.candidate.as_ref().unwrap().objective.unwrap();
    assert!(
        resumed <= objective + 1e-6 * objective.abs().max(1.0),
        "resumed {resumed} vs stored {objective}"
    );
    let job = store.jobs().get(enqueued.job_id()).await.unwrap();
    assert_ne!(job.state, JobState::Running);
    assert!(
        started.elapsed() < std::time::Duration::from_secs(60),
        "{:?}",
        started.elapsed()
    );
    drop((resumer, local, shared, operations, result));
    database.remove().await.unwrap();
}

/// A cancellation requested from another process's connection stops a SCIP solve running
/// in a worker process: the durable flag reaches the native interrupt through the
/// worker's watcher, and the attempt ends cancelled long before the solve's time limit.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn cross_process_cancel_stops_scip() {
    let database = TestDatabase::create().await.unwrap();
    let operations = Operations::connect(database.url(), "enqueuer", LeasePolicy::default())
        .await
        .unwrap();
    let (shared, local) = scip_runtime();
    let job = long_scip_job(
        &shared,
        &local,
        &operations,
        JobStart::Fresh,
        std::time::Duration::from_secs(120),
    )
    .await;
    let enqueued = operations
        .enqueue(&job, "market-split-cancel", RetryPolicy::ONCE, 0)
        .await
        .unwrap();
    let attempt = enqueued.attempt_id();
    let mut worker = spawn_worker(database.url(), "worker-cancelled", 30);
    // SCIP is searching once its first incumbent is stored.
    until(
        std::time::Duration::from_secs(40),
        "a stored incumbent",
        || async {
            searching(&operations, attempt).await;
            !incumbents(&database, attempt).await.is_empty()
        },
    )
    .await;
    let requested = tokio::time::Instant::now();
    let store = database.store();
    assert_eq!(
        store.request_cancel(attempt, "test").await.unwrap(),
        CancelOutcome::Requested
    );
    let status = tokio::task::spawn_blocking(move || worker.wait().unwrap())
        .await
        .unwrap();
    assert!(status.success(), "{status}");
    assert!(
        requested.elapsed() < std::time::Duration::from_secs(30),
        "{:?}",
        requested.elapsed()
    );
    let stored = store.attempts().get(attempt).await.unwrap();
    assert_eq!(stored.state, AttemptState::Cancelled);
    assert_eq!(
        TerminationCode::of(&stored).unwrap(),
        Some(TerminationCode::Runtime(RuntimeTermination::Cancelled))
    );
    assert_eq!(
        store.jobs().get(enqueued.job_id()).await.unwrap().state,
        JobState::Cancelled
    );
    drop((local, shared, operations));
    database.remove().await.unwrap();
}

/// A runtime budgeted as the worker binary is with `--memory-mib 8192`, whose native jobs
/// admit SCIP's memory limit (the default foreign allowance).
fn scip_runtime() -> (Arc<SharedRuntime>, Runtime) {
    let n = |v| NonZeroUsize::new(v).unwrap();
    let memory: usize = 8 << 30;
    let shared = SharedRuntime::build(pse_runtime::ResourceBudget {
        memory_limit_bytes: n(memory),
        spill_dir: std::env::temp_dir(),
        max_temp_dir_bytes: 1 << 30,
        top_consumers: n(5),
        threads: pse_engine::ThreadBudget {
            pool_threads: n(2),
            target_partitions: n(1),
        },
        execution: Default::default(),
        cache: pse_runtime::DeltaCacheBudget::disabled(1024),
        math: pse_runtime::math::MathPolicy {
            workspace_bytes: memory / 8,
            worker_bytes: memory / 16,
            artifact_bytes: memory / 8,
            ..Default::default()
        },
        hashing_may_use_pool: false,
    })
    .unwrap();
    let registry = pse_schema::shared_registry().unwrap();
    let sessions = Arc::new(
        shared
            .session_factory(pse_engine::session::native_engine_profile())
            .unwrap(),
    );
    (
        shared.clone(),
        Runtime::from_shared(shared, registry, sessions),
    )
}

/// `Root` with the right-hand side of its equation as parameter `a`: each study point sets
/// its own value.
const PARAMETRIC: &str = r#"package algebraic { def Root {
    param a:Scalar = 4;
    var x:Scalar;
    eq square:x*x==a;
    annotation start x(1);
    annotation bounds x(0,10);
    annotation report x("root");
    annotation check x(x>1);
} }"#;

/// Two `pse-worker` processes claim the points of one study concurrently (Plan 22 O7,
/// architecture S15): every point runs once, each completed point's members are written
/// under the study's intent, and exactly one publication — of the study's own attempt —
/// holds the summary and every completed point's members.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn study_parallel_workers_publish_once() {
    use pse_model::study::{OccurrenceKey, StartPolicy};
    use pse_runtime::workflow::{
        BindingAssignment, BindingQuantity, BindingTarget, CaseOperation, OperationRequest,
        PackageSources, PointOverlay, PreparationSettings, StudyPlan, StudyPoint, StudyPointPolicy,
        StudyState,
    };
    const POINTS: usize = 8;
    let database = TestDatabase::create().await.unwrap();
    let operations = Operations::connect(database.url(), "enqueuer", LeasePolicy::default())
        .await
        .unwrap();
    let (physical, modeling) = sources_of(PARAMETRIC);
    let (shared, local) = runtime();
    let local = local.with_durability(Durability::Durable(operations.clone()));
    let (package, context) = package(&shared, &local, &physical, &modeling).await;
    let case = package
        .declarations()
        .iter()
        .find(|d| d.name == "Root")
        .unwrap()
        .declaration_id;
    let directory = tempfile::tempdir().unwrap();
    let workspace = local
        .register_workspace(
            "parallel",
            url::Url::from_directory_path(directory.path()).unwrap(),
        )
        .await
        .unwrap();
    let scalar = context
        .quantities()
        .quantity_types()
        .find(|quantity| quantity.name.as_deref() == Some("Scalar"))
        .unwrap();
    let points = (0..POINTS)
        .map(|index| StudyPoint {
            operation: OperationRequest::DeclaredCase(CaseOperation {
                case,
                route: pse_model::generated::enums::ModelingAnalysisRoute::Steady,
                settings: settings(),
            }),
            preparation: PreparationSettings::default(),
            overlay: PointOverlay {
                // x = index + 2, inside the bounds.
                assignments: vec![BindingAssignment {
                    target: BindingTarget::Path("a".into()),
                    value: BindingQuantity {
                        magnitude: pse_model::scalars::FiniteBound::try_new(
                            ((index + 2) * (index + 2)) as f64,
                        )
                        .unwrap(),
                        quantity: scalar.id.as_id(),
                        unit: scalar.canonical_unit.as_id(),
                    },
                }],
            },
            policy: StudyPointPolicy {
                key: OccurrenceKey(u32::try_from(index).unwrap()),
                dependencies: vec![],
                start: StartPolicy::Fresh,
                attempt_limit: 1,
            },
        })
        .collect();
    let handle = local
        .start_study(
            &workspace,
            StudyPlan {
                sources: PackageSources {
                    physical,
                    modeling: vec![modeling],
                },
                points,
                retry: RetryPolicy::ONCE,
                priority: 0,
            },
        )
        .await
        .unwrap();

    // Two worker processes serve the queue at once and stop when it is empty.
    let url = database.url().to_owned();
    let mut workers = ["worker-left", "worker-right"].map(|name| spawn_worker(&url, name, 30));
    let statuses =
        tokio::task::spawn_blocking(move || workers.each_mut().map(|w| w.wait().unwrap()))
            .await
            .unwrap();
    for status in statuses {
        assert!(status.success(), "{status}");
    }

    let status = handle.status().await.unwrap();
    assert_eq!(status.state, StudyState::Published, "{status:?}");
    assert_eq!(status.attempt_state, AttemptState::Completed);
    let store = operations.store();
    let mut workers = std::collections::BTreeSet::new();
    for point in &status.points {
        let attempt = store.attempts().get(point.attempt_id).await.unwrap();
        assert_eq!(attempt.state, AttemptState::Completed, "{point:?}");
        // Each point ran exactly once: its job's first try completed.
        assert_eq!(attempt.parent_attempt, None);
        workers.insert(attempt.worker.unwrap());
    }
    assert_eq!(
        workers.into_iter().collect::<Vec<_>>(),
        ["worker-left", "worker-right"],
        "both processes ran points"
    );

    // Exactly one publication: the workspace head, with no parent, naming the study's
    // attempt, holding the summary and every point's members under the intent's prefix.
    let published = handle.result().await.unwrap().unwrap();
    assert_eq!(
        local.head(workspace.workspace_id).await.unwrap(),
        Some(published.publication_id)
    );
    let record = store
        .catalog()
        .publication(published.publication_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(record.publication.parent_publication, None);
    assert_eq!(record.publication.attempt_id, status.attempt_id);
    let intent = store
        .catalog()
        .intent(published.publication_id)
        .await
        .unwrap()
        .unwrap();
    assert!(
        record
            .members
            .iter()
            .all(|member| member.table_uri.starts_with(&intent.member_prefix))
    );
    let catalogs: std::collections::BTreeSet<String> = record
        .members
        .iter()
        .map(|member| member.catalog_name.clone())
        .collect();
    let mut expected: std::collections::BTreeSet<String> =
        (0..POINTS).map(|index| format!("point_{index}")).collect();
    expected.insert("study".to_owned());
    assert_eq!(catalogs, expected);
    drop((local, shared, package, operations, directory));
    database.remove().await.unwrap();
}
