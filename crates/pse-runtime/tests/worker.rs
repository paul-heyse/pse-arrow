// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The `pse-worker` binary end to end (Plan 22 O4, O6): a job is enqueued in an isolated
//! operational store, the worker runs it in a child process and records its attempt, and
//! another process reuses the seed the worker stored. Run with `just worker-test`.
use pse_backend_native::solve::{Backend, SolveIntent};
use pse_operations::{
    attempts::{NativeTermination, TerminationCode},
    jobs::{JobState, RetryPolicy},
    lifecycle::AttemptState,
    testing::TestDatabase,
};
use pse_runtime::{
    CancelSource, SharedRuntime,
    authoring_driver::document::{OwnedDocumentSet, load_package_texts_owned},
    workflow::{
        Durability, JobPresolve, JobProfile, LeasePolicy, ModelingJob, ModelingPackage, Operations,
        RunDurability, Runtime, StartSource, StoredStart,
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

fn sources() -> (BTreeMap<String, String>, BTreeMap<String, String>) {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/packages");
    let mut manifest = std::fs::read_to_string(fixtures.join("minimal_explicit/package.toml"))
        .unwrap()
        .replace(r#"id_policy = "explicit""#, r#"id_policy = "named""#);
    manifest.push_str(
        "\n[[quantity_aliases]]\nname = \"Scalar\"\nquantity_type_id = \"1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f\"\n",
    );
    (
        texts(&fixtures.join("physical-primitives")),
        BTreeMap::from([
            ("package.toml".to_owned(), manifest),
            ("models/root.pse".to_owned(), SQUARE.to_owned()),
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
            worker_bytes: 128 << 20,
            workspace_bytes: 128 << 20,
            foreign_bytes: 1 << 20,
            ..Default::default()
        },
        hashing_may_use_pool: false,
    })
    .unwrap();
    let registry = pse_schema::shared_registry().unwrap();
    pse_engine::validation::bind_defaults(&registry).unwrap();
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
    physical: &BTreeMap<String, String>,
    modeling: &BTreeMap<String, String>,
) -> ModelingPackage {
    let pool = shared.pool();
    let cancel = pse_columnar::CancellationToken::new();
    let load = |texts| {
        load_package_texts_owned(
            texts,
            runtime.registry(),
            pse_authoring::ParseBudget::default(),
            &pool,
            &cancel,
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
    runtime.modeling_from_documents(&modeling, context).unwrap()
}

fn profile() -> JobProfile {
    JobProfile {
        intent: SolveIntent::FeasiblePoint,
        backend: Some(Backend::Ipopt),
        presolve: JobPresolve::Off,
        ..JobProfile::default()
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
    let package = package(&shared, &local, &physical, &modeling).await;
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
        profile: profile(),
    };
    let enqueued = operations
        .enqueue_modeling(&job, "square-end-to-end", RetryPolicy::ONCE, 0)
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
                "4096",
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
        attempt.termination.as_ref().map(|t| &t.code),
        Some(&TerminationCode::Native(NativeTermination::Success))
    );
    let history: Vec<_> = store
        .attempts()
        .history(attempt.attempt_id)
        .await
        .unwrap()
        .iter()
        .map(|t| t.to)
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

/// This process reuses the seed the worker process stored for the same preparation.
async fn stored_seed_reused_across_processes(
    operations: &Operations,
    package: &ModelingPackage,
    job: &ModelingJob,
) {
    let cancel = CancelSource::new();
    let analysis = package
        .declared_analysis(
            job.case,
            job.route,
            Default::default(),
            job.profile.solver().unwrap(),
            Default::default(),
            pse_modeling::Limits::default(),
            &cancel,
        )
        .await
        .unwrap();
    let prepared = package.prepare_analysis(&analysis, &cancel).await.unwrap();
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
        .get(solution)
        .await
        .unwrap()
        .unwrap();
    assert_ne!(stored.solution.created_by, None);
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
    let pse_runtime::workflow::RunReport::Modeling(steps) = result.report().unwrap() else {
        panic!()
    };
    let pse_runtime::math::solves::Outcome::Native(native) = &steps[0].outcome else {
        panic!()
    };
    let receipt = native.start_receipt.as_ref().unwrap();
    assert!(receipt.submitted);
    let seed = receipt.seed.as_ref().unwrap();
    assert_eq!(
        seed.compatibility.layout,
        stored.solution.compatibility_stamp
    );
    // Its identity is in the result's lineage: the stored seed changes the request identity.
    let unseeded = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    let plain = unseeded.start().unwrap().wait().await.unwrap();
    let lineage =
        |r: &pse_runtime::workflow::RunResult| r.completion().unwrap().lineage[0].request_identity;
    assert_ne!(lineage(&result), lineage(&plain));
}
