// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native Delta/Parquet experiment fixtures; fixture construction is separately timed.
use datafusion::{
    arrow::{
        array::{ArrayRef, Int64Array, RecordBatch},
        datatypes::{DataType, Field, Schema},
    },
    execution::{
        memory_pool::{GreedyMemoryPool, MemoryPool, PeakRecordingPool},
        runtime_env::RuntimeEnvBuilder,
        session_state::{SessionState, SessionStateBuilder},
    },
    logical_expr::{col, lit},
    prelude::SessionContext,
};
use deltalake::{DeltaTable, kernel::transaction::CommitProperties};
use pse_catalog::{
    cache_service::{DeltaCacheBudget, DeltaCacheService},
    delta::{
        layout::DurableLayout,
        provider::{open_view, table_builder},
    },
};
use pse_engine::session::planner::UnifiedPlanner;
use pse_testkit::counting_store::CountingStore;
use std::{collections::HashMap, sync::Arc, time::Instant};

pub(super) struct Environment {
    pub(super) state: Arc<SessionState>,
    pub(super) caches: Arc<DeltaCacheService>,
    pub(super) store: Arc<CountingStore>,
    pub(super) pool: Arc<dyn MemoryPool>,
    peak: Arc<PeakRecordingPool>,
}
impl Environment {
    pub(super) fn new(enabled: bool, predicate: usize, partitions: usize, crc: u64) -> Self {
        let peak = Arc::new(PeakRecordingPool::new(Arc::new(GreedyMemoryPool::new(
            256 << 20,
        ))));
        let pool: Arc<dyn MemoryPool> = peak.clone();
        let runtime = RuntimeEnvBuilder::new()
            .with_memory_pool(pool.clone())
            .build_arc()
            .unwrap();
        let root = url::Url::parse("file:///").unwrap();
        let store = CountingStore::new(runtime.object_store_registry.get_store(&root).unwrap());
        runtime.register_object_store(&root, store.clone());
        let mut policy = if enabled {
            DeltaCacheBudget::for_memory(256 << 20)
        } else {
            DeltaCacheBudget::disabled(128 << 20)
        };
        policy.native.predicate_cache_bytes = predicate;
        policy.crc_replay_max_commits = crc;
        let caches = DeltaCacheService::new(policy, &pool).unwrap();
        let mut config =
            datafusion::execution::config::SessionConfig::new().with_target_partitions(partitions);
        config.options_mut().execution.parquet.pushdown_filters = true;
        config
            .options_mut()
            .execution
            .parquet
            .max_predicate_cache_size = Some(predicate);
        let state = Arc::new(
            SessionStateBuilder::new()
                .with_default_features()
                .with_runtime_env(runtime)
                .with_config(
                    config
                        .with_extension(caches.native().clone())
                        .with_extension(caches.clone()),
                )
                .with_query_planner(Arc::new(UnifiedPlanner::new(
                    pse_catalog::assembly::planners(),
                )))
                .build(),
        );
        Self {
            state,
            caches,
            store,
            pool,
            peak,
        }
    }
    pub(super) fn report(&self) -> serde_json::Value {
        serde_json::json!({"io":self.store.report(),"reserved_bytes":self.pool.reserved(), "pool_peak_bytes":self.peak.max_reserved(), "process_peak_rss_bytes":super::cache_journey::process_peak_rss(),
            "cache":self.caches.native().report().iter().map(|r| serde_json::json!({"name":r.name,"hits":r.hits,"misses":r.misses,"bypasses":r.bypasses,"retained_bytes":r.retained_bytes,"live_bytes":r.live_bytes,"pinned_bytes":r.pinned_bytes})).collect::<Vec<_>>()})
    }
}
fn schema() -> Arc<Schema> {
    Arc::new(Schema::new(
        (0..16)
            .map(|i| Field::new(format!("v{i}"), DataType::Int64, true))
            .collect::<Vec<_>>(),
    ))
}
fn batch(rows: usize) -> RecordBatch {
    RecordBatch::try_new(
        schema(),
        (0..16)
            .map(|i| -> ArrayRef {
                Arc::new(Int64Array::from(
                    (0..rows)
                        .map(|r| {
                            if i != 0 && r % 5 == 0 {
                                None
                            } else {
                                Some(i64::try_from(r).unwrap())
                            }
                        })
                        .collect::<Vec<_>>(),
                ))
            })
            .collect(),
    )
    .unwrap()
}
pub(super) async fn create(root: &url::Url, rows: usize, state: &Arc<SessionState>) -> DeltaTable {
    let values = batch(rows);
    let layout = DurableLayout::new(values.schema()).unwrap();
    // These native Int64 columns have identical execution/storage layouts;
    // persist the real generated descriptors used by selected-view reopening.
    let values = RecordBatch::try_new(
        Arc::clone(layout.storage_schema()),
        values.columns().to_vec(),
    )
    .unwrap();
    table_builder(root.clone(), state)
        .unwrap()
        .build()
        .unwrap()
        .write([values])
        .with_session_state(state.clone())
        .with_session_fallback_policy(
            deltalake::delta_datafusion::SessionFallbackPolicy::RequireSessionState,
        )
        .with_configuration([
            ("delta.enableChangeDataFeed", Some("true")),
            ("delta.checkpointInterval", Some("10")),
        ])
        .await
        .unwrap()
}
fn copy_tree(source: &std::path::Path, destination: &std::path::Path) {
    std::fs::create_dir_all(destination).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let path = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &path);
        } else {
            std::fs::copy(entry.path(), path).unwrap();
        }
    }
}
async fn open(
    root: &url::Url,
    version: i64,
    env: &Environment,
) -> datafusion::datasource::ViewTable {
    open_view(
        root.clone(),
        version,
        &DurableLayout::new(schema()).unwrap(),
        env.state.clone(),
    )
    .await
    .unwrap()
}
async fn scan(
    root: &url::Url,
    version: i64,
    env: &Environment,
    selected: bool,
) -> serde_json::Value {
    let started = Instant::now();
    let provider = open(root, version, env).await;
    let context = SessionContext::new_with_state(env.state.as_ref().clone());
    let frame = context.read_table(Arc::new(provider)).unwrap();
    let frame = if selected {
        frame
            .filter(col("v0").lt(lit(8_i64)))
            .unwrap()
            .select_columns(&["v0", "v1"])
            .unwrap()
    } else {
        frame
    };
    let plan = frame.create_physical_plan().await.unwrap();
    let planning = started.elapsed().as_secs_f64();
    let started = Instant::now();
    let batches = datafusion::physical_plan::collect(plan.clone(), context.task_ctx())
        .await
        .unwrap();
    let rows = batches.iter().map(RecordBatch::num_rows).sum::<usize>();
    if selected {
        assert_eq!(rows, 8);
    }
    let mut metrics = Vec::new();
    collect_metrics(&plan, &mut metrics);
    serde_json::json!({"planning_seconds":planning,"execution_seconds":started.elapsed().as_secs_f64(),"rows":rows,"native_metrics":metrics,"resources":env.report()})
}
fn collect_metrics(
    plan: &Arc<dyn datafusion::physical_plan::ExecutionPlan>,
    out: &mut Vec<String>,
) {
    if let Some(metrics) = plan.metrics() {
        out.push(format!("{}: {metrics:?}", plan.name()));
    }
    for child in plan.children() {
        collect_metrics(child, out);
    }
}

pub(super) async fn matrix(smoke: bool) {
    let directory = tempfile::tempdir().unwrap();
    let template = directory.path().join("template");
    let root = url::Url::from_directory_path(&template).unwrap();
    let env = Environment::new(true, 0, 1, 0);
    let started = Instant::now();
    let mut table = create(&root, 4096, &env.state).await;
    let commits = if smoke { 3 } else { 1000 };
    for generation in 1..commits {
        table = table
            .set_tbl_properties()
            .with_properties(HashMap::from([(
                "measurement.generation".into(),
                generation.to_string(),
            )]))
            .with_raise_if_not_exists(false)
            .with_commit_properties(
                CommitProperties::default()
                    .with_create_checkpoint(true)
                    .with_cleanup_expired_logs(Some(false)),
            )
            .await
            .unwrap();
    }
    println!(
        "{}",
        serde_json::json!({"experiment":"delta_fixture_setup","commits":commits,"seconds":started.elapsed().as_secs_f64(),"rows":4096,"columns":16,"null_every":5,"resources":env.report()})
    );
    let members = if smoke { vec![1] } else { vec![1, 8, 32] };
    let versions = if smoke { vec![0, 2] } else { vec![0, 99, 999] };
    let locations = (0..*members.last().unwrap())
        .map(|member| {
            let path = directory.path().join(format!("member-{member}"));
            copy_tree(&template, &path);
            url::Url::from_directory_path(path).unwrap()
        })
        .collect::<Vec<_>>();
    for count in members {
        for version in &versions {
            if !smoke {
                let request = serde_json::json!({"roots":locations[..count],"version":version});
                assert!(
                    std::process::Command::new(std::env::current_exe().unwrap())
                        .env("PSE_CACHE_COLD_OPEN", request.to_string())
                        .status()
                        .unwrap()
                        .success()
                );
            }
            for enabled in [false, true] {
                let env = Environment::new(enabled, 0, 1, 0);
                for phase in ["fresh_cache", "warm_cache"] {
                    env.store.reset();
                    let started = Instant::now();
                    for root in &locations[..count] {
                        drop(open(root, *version, &env).await);
                    }
                    println!(
                        "{}",
                        serde_json::json!({"experiment":"delta_open","members":count,"commits":version+1,"cache_enabled":enabled,"phase":phase,"seconds":started.elapsed().as_secs_f64(),"resources":env.report()})
                    );
                }
            }
        }
    }
    let retained = Environment::new(true, 0, 1, 16);
    if !smoke {
        checkpoint_control(&template, directory.path(), i64::from(commits - 1)).await;
    }
    drop(open(&root, i64::from(commits - 1), &retained).await);
    let table = table
        .set_tbl_properties()
        .with_properties(HashMap::new())
        .await
        .unwrap();
    retained.store.reset();
    let started = Instant::now();
    drop(
        open(
            &root,
            i64::try_from(table.version().unwrap()).unwrap(),
            &retained,
        )
        .await,
    );
    println!(
        "{}",
        serde_json::json!({"experiment":"delta_incremental_open","seconds":started.elapsed().as_secs_f64(),"resources":retained.report()})
    );
    scans_and_crc(
        &root,
        i64::try_from(table.version().unwrap()).unwrap(),
        smoke,
    )
    .await;
    super::cdf::measure(&url::Url::from_directory_path(directory.path().join("cdf")).unwrap())
        .await;
}
async fn checkpoint_control(template: &std::path::Path, directory: &std::path::Path, version: i64) {
    for checkpoints in [true, false] {
        let path = directory.join(format!("checkpoint-control-{checkpoints}"));
        copy_tree(template, &path);
        let log = path.join("_delta_log");
        if !checkpoints {
            for entry in std::fs::read_dir(&log).unwrap() {
                let entry = entry.unwrap();
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if name == "_last_checkpoint" || name.contains(".checkpoint.") {
                    // Only this disposable comparison fixture is changed. All
                    // original JSON commits remain available for native replay.
                    std::fs::remove_file(entry.path()).unwrap();
                }
            }
        }
        let files = std::fs::read_dir(&log)
            .unwrap()
            .map(|entry| entry.unwrap().metadata().unwrap())
            .filter(std::fs::Metadata::is_file)
            .collect::<Vec<_>>();
        let log_bytes: u64 = files.iter().map(std::fs::Metadata::len).sum();
        let root = url::Url::from_directory_path(&path).unwrap();
        for repetition in 0..3 {
            let env = Environment::new(false, 0, 1, 0);
            let started = Instant::now();
            let selected = open(&root, version, &env).await;
            println!(
                "{}",
                serde_json::json!({"experiment":"checkpoint_control", "checkpoints":checkpoints, "interval":10, "commits":version+1, "repetition":repetition, "seconds":started.elapsed().as_secs_f64(), "log_files":files.len(), "log_bytes":log_bytes, "resources":env.report()})
            );
            drop(selected);
        }
    }
}

async fn scans_and_crc(root: &url::Url, version: i64, smoke: bool) {
    for concurrency in if smoke { vec![1] } else { vec![1, 4] } {
        for predicate in [0, 1 << 20] {
            let env = Environment::new(true, predicate, concurrency, 16);
            for selected in [false, true] {
                env.store.reset();
                let results = futures_util::future::join_all(
                    (0..concurrency).map(|_| scan(root, version, &env, selected)),
                )
                .await;
                println!(
                    "{}",
                    serde_json::json!({"experiment":"delta_scan_pushdown","concurrency":concurrency,"predicate_bytes_per_reader":predicate,"selected":selected,"results":results})
                );
            }
        }
    }
    // Native metadata opens and full query opens have different replay obligations.
    let crc_directory = tempfile::tempdir().unwrap();
    let root = url::Url::from_directory_path(crc_directory.path()).unwrap();
    let env = Environment::new(true, 0, 1, 16);
    let table = table_builder(root.clone(), &env.state)
        .unwrap()
        .build()
        .unwrap();
    super::kernel_checksum::create_with_checksum(
        root.clone(),
        &batch(4096).schema(),
        table.log_store().engine(None),
    )
    .await;
    let table = table_builder(root.clone(), &env.state)
        .unwrap()
        .load()
        .await
        .unwrap()
        .write([batch(4096)])
        .with_session_state(env.state.clone())
        .with_session_fallback_policy(
            deltalake::delta_datafusion::SessionFallbackPolicy::RequireSessionState,
        )
        .await
        .unwrap();
    let version = i64::try_from(table.version().unwrap()).unwrap();
    let seed_crc = crc_directory
        .path()
        .join("_delta_log/00000000000000000000.crc");
    let seed_bytes = std::fs::read(&seed_crc).unwrap();
    std::fs::remove_file(&seed_crc).unwrap();
    for crc_present in [false, true] {
        let env = Environment::new(true, 0, 1, 16);
        if crc_present {
            std::fs::write(&seed_crc, &seed_bytes).unwrap();
            let table = table_builder(root.clone(), &env.state)
                .unwrap()
                .load()
                .await
                .unwrap();
            let snapshot = table.snapshot().unwrap().snapshot().snapshot_ref().clone();
            let engine = table.log_store().engine(None);
            snapshot.write_checksum(engine).await.unwrap();
        }
        for with_files in [false, true] {
            env.store.reset();
            let started = Instant::now();
            let builder = table_builder(root.clone(), &env.state)
                .unwrap()
                .with_version(u64::try_from(version).unwrap());
            let builder = if with_files {
                builder
            } else {
                builder.without_files()
            };
            let loaded = builder.load().await.unwrap();
            assert_eq!(loaded.version(), Some(u64::try_from(version).unwrap()));
            println!(
                "{}",
                serde_json::json!({"experiment":"crc_load_class","crc_present":crc_present,"files":with_files,"seconds":started.elapsed().as_secs_f64(),"resources":env.report()})
            );
        }
    }
}

pub(super) async fn cold_open(payload: &str) {
    let request: serde_json::Value = serde_json::from_str(payload).unwrap();
    let locations: Vec<url::Url> = serde_json::from_value(request["roots"].clone()).unwrap();
    let version = request["version"].as_i64().unwrap();
    let env = Environment::new(true, 0, 1, 0);
    let started = Instant::now();
    for root in &locations {
        drop(open(root, version, &env).await);
    }
    println!(
        "{}",
        serde_json::json!({"experiment":"delta_open","phase":"fresh_process_os_cache_uncontrolled","members":locations.len(),"commits":version+1,"seconds":started.elapsed().as_secs_f64(),"resources":env.report()})
    );
}
