// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One phase-separated target-native measurement matrix; no timing pass threshold.
#![allow(
    clippy::unwrap_used,
    clippy::print_stdout,
    reason = "standalone qualification harness emits measurements and asserts its fixtures"
)]
#[path = "../../crates/pse-catalog/tests/support/cache_journey.rs"]
mod cache_journey;
#[path = "native_cache/cdf.rs"]
mod cdf;
#[path = "native_cache/delta.rs"]
mod delta;
#[path = "../../crates/pse-catalog/tests/support/kernel_checksum.rs"]
mod kernel_checksum;
#[path = "native_cache/store.rs"]
mod store;
#[path = "native_cache/strata.rs"]
mod strata;
#[path = "../../crates/pse-rules/tests/support/strata_fixture.rs"]
mod strata_fixture;
use datafusion::{
    arrow::{
        array::{Int64Array, RecordBatch},
        datatypes::DataType,
    },
    execution::{
        memory_pool::{GreedyMemoryPool, MemoryPool, PeakRecordingPool},
        runtime_env::RuntimeEnvBuilder,
    },
    functions_aggregate::expr_fn::count,
    logical_expr::{LogicalPlanBuilder, col},
};
use pse_catalog::{
    cache_service::{CacheBudget, NativeCacheService},
    session::{ExecutionSettings, SessionFactory, ThreadBudget, native_engine_profile},
};
use pse_ids::{CancellationToken, FixedBudget};
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::{
    RegistryBuilder,
    model::{Authority, FieldContract, Namespace, RelationDecl, SnapshotClass},
};
use std::{collections::BTreeMap, sync::Arc, time::Instant};

#[expect(
    clippy::too_many_lines,
    reason = "phase separated fixture setup, execution and independent row count assertions"
)]
async fn prepared_rounds(rows: usize, rounds: usize) -> serde_json::Value {
    let mut registry = RegistryBuilder::new();
    registry.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "round_values",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "Round measurement",
        )
        .pk(&["id"])
        .columns(vec![FieldContract::key(
            "id",
            FieldContract::native(DataType::Int64),
            "identity",
        )]),
    );
    let registry = Arc::new(registry.build().unwrap());
    let spec = registry.relation("authored.round_values").unwrap();
    let key = spec.key;
    let schema = Arc::new(pse_schema::arrow::relation_schema(&registry, spec).unwrap());
    let empty =
        FieldCheckedBatch::admit(&registry, spec, RecordBatch::new_empty(schema.clone())).unwrap();
    let values = FieldCheckedBatch::admit(
        &registry,
        spec,
        RecordBatch::try_new(
            schema,
            vec![Arc::new(Int64Array::from(
                (0..i64::try_from(rows).unwrap()).collect::<Vec<_>>(),
            ))],
        )
        .unwrap(),
    )
    .unwrap();
    let peak = Arc::new(PeakRecordingPool::new(Arc::new(GreedyMemoryPool::new(
        256 << 20,
    ))));
    let pool: Arc<dyn MemoryPool> = peak.clone();
    let runtime = RuntimeEnvBuilder::new()
        .with_memory_pool(pool.clone())
        .build_arc()
        .unwrap();
    let caches = NativeCacheService::new(CacheBudget::for_memory(256 << 20), &pool).unwrap();
    let factory = SessionFactory::new(
        runtime,
        FixedBudget::new(256 << 20),
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: 1.try_into().unwrap(),
            target_partitions: 1.try_into().unwrap(),
        },
        native_engine_profile(),
    )
    .unwrap()
    .with_cache_service(caches.clone());
    let cancel = CancellationToken::new();
    let session = factory
        .candidate(BTreeMap::new(), registry, &cancel)
        .unwrap();
    let (session, input) = session
        .with_round_inputs(
            BTreeMap::from([(key, empty.clone())]),
            BTreeMap::new(),
            &cancel,
        )
        .unwrap();
    let plan = LogicalPlanBuilder::scan(
        session.table_reference(&key).unwrap(),
        session.table_source(&key).unwrap(),
        None,
    )
    .unwrap()
    .aggregate(
        Vec::<datafusion::logical_expr::Expr>::new(),
        [count(col("id"))],
    )
    .unwrap()
    .build()
    .unwrap();
    let started = Instant::now();
    let mut prepared = session
        .prepare_rule_plan(plan, &cancel)
        .unwrap()
        .prepare_reusable(&cancel)
        .await
        .unwrap();
    let prepare_seconds = started.elapsed().as_secs_f64();
    let mut execute_seconds = Vec::new();
    for ordinal in 0..rounds {
        input
            .replace(
                &key.qualified_name(),
                if ordinal % 2 == 0 {
                    empty.clone()
                } else {
                    values.clone()
                },
            )
            .unwrap();
        let started = Instant::now();
        let result = prepared.execute(&cancel).await.unwrap();
        execute_seconds.push(started.elapsed().as_secs_f64());
        assert_eq!(
            result.batches()[0]
                .column(0)
                .as_any()
                .downcast_ref::<Int64Array>()
                .unwrap()
                .value(0),
            if ordinal % 2 == 0 {
                0
            } else {
                i64::try_from(rows).unwrap()
            }
        );
    }
    let counters = caches
        .execution_report()
        .into_iter()
        .collect::<BTreeMap<_, _>>();
    assert_eq!(counters["physical_plans"], Some(1));
    assert_eq!(counters["reusable_executions"], Some(rounds));
    serde_json::json!({"experiment":"prepared_rounds", "rows": rows,"rounds":rounds,"prepare_seconds":prepare_seconds,"execute_seconds":execute_seconds,"counts":counters,"reserved_bytes":pool.reserved(), "pool_peak_bytes":peak.max_reserved(), "process_peak_rss_bytes":cache_journey::process_peak_rss()})
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Ok(payload) = std::env::var("PSE_CACHE_COLD_OPEN") {
        delta::cold_open(&payload).await;
        return;
    }
    if let Ok(payload) = std::env::var("PSE_NATIVE_CACHE_MAINTENANCE") {
        cache_journey::maintenance_child(&payload).await;
        return;
    }
    let smoke = std::env::args().any(|argument| argument == "--test");
    tracing_subscriber::fmt()
        .with_env_filter("pse_rules=debug")
        .with_target(true)
        .with_ansi(false)
        .with_writer(std::io::stderr)
        .init();
    let mut planning_counts = None;
    for nodes in if smoke { vec![3] } else { vec![3, 8, 16] } {
        let counts = strata::measure(nodes).await;
        if let Some(expected) = planning_counts {
            assert_eq!(
                counts, expected,
                "additional inference rounds must not plan again"
            );
        }
        planning_counts = Some(counts);
    }
    let sizes = if smoke {
        vec![16]
    } else {
        vec![16, 1024, 16384]
    };
    delta::matrix(smoke).await;
    if !smoke {
        for resident_bytes in [0, 64 << 10, 8 << 20] {
            let mut policy = CacheBudget::for_memory(256 << 20);
            policy.resident_bytes = resident_bytes;
            let receipt =
                Box::pin(cache_journey::run_policy(true, 16384, false, Some(policy))).await;
            println!(
                "{}",
                serde_json::json!({"experiment":"cache_pressure","resident_bytes":resident_bytes,"receipt":receipt})
            );
        }
    }
    for rows in sizes {
        println!(
            "{}",
            prepared_rounds(rows, if smoke { 2 } else { 20 }).await
        );
        for enabled in [false, true] {
            println!("{}", Box::pin(cache_journey::run(enabled, rows)).await);
        }
    }
}
