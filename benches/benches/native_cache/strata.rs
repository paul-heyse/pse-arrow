// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The actual cyclic inference operator, with an independent all-pairs oracle.
use super::strata_fixture::{self as fixture, Fixture};
use datafusion::execution::{
    memory_pool::{GreedyMemoryPool, MemoryPool, PeakRecordingPool},
    runtime_env::RuntimeEnvBuilder,
};
use pse_catalog::{
    cache_service::{CacheBudget, NativeCacheService},
    session::{ExecutionSettings, SessionFactory, ThreadBudget, native_engine_profile},
};
use pse_ids::FixedBudget;
use pse_schema::model::{Cell, RuleDecl};
use std::{collections::BTreeMap, sync::Arc, time::Instant};

fn program() -> pse_schema::RegistryBuilder {
    let mut builder = fixture::builder();
    fixture::input(
        &mut builder,
        vec![fixture::id("from"), fixture::id("to")],
        &["from", "to"],
    );
    fixture::head(
        &mut builder,
        "reach",
        "reach_assertions",
        &["origin", "reached"],
        vec![fixture::id("origin"), fixture::id("reached")],
    );
    fixture::declare(
        &mut builder,
        RuleDecl::new(
            "seed",
            "1",
            0,
            "inferred.reach",
            r#"SELECT "from" AS origin, "to" AS reached FROM authored.input"#,
            vec![fixture::read("authored.input", "edges")],
        )
        .assertions("provenance.reach_assertions"),
    );
    fixture::declare(&mut builder, RuleDecl::new(
        "step", "1", 0, "inferred.reach",
        r#"SELECT reach.origin, edges."to" AS reached FROM inferred.reach AS reach JOIN authored.input AS edges ON reach.reached = edges."from""#,
        vec![fixture::read("inferred.reach", "reach"), fixture::read("authored.input", "edges")],
    ).assertions("provenance.reach_assertions"));
    builder
}

pub(super) async fn measure(nodes: u32) -> [Option<usize>; 4] {
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
    .with_cache_service(caches.clone())
    .with_query_planner(Arc::new(
        pse_catalog::session::planner::UnifiedPlanner::new(vec![Arc::new(
            pse_rules::strata::native::RuleExtensionPlanner,
        )]),
    ));
    let started = Instant::now();
    let fixture = Fixture::with_factory(
        program(),
        (0..nodes)
            .map(|node| {
                vec![
                    Cell::U64(u64::from(node)),
                    Cell::U64(u64::from((node + 1) % nodes)),
                ]
            })
            .collect(),
        &factory,
    );
    let setup_seconds = started.elapsed().as_secs_f64();
    let started = Instant::now();
    let result = Box::pin(fixture.run(nodes + 4)).await.unwrap();
    let execute_seconds = started.elapsed().as_secs_f64();
    let expected = (0..nodes)
        .flat_map(|origin| {
            (0..nodes).map(move |reached| {
                vec![Cell::U64(u64::from(origin)), Cell::U64(u64::from(reached))]
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(fixture.rows(&result, "inferred.reach"), expected);
    assert_eq!(result.rounds[&0], nodes + 2);
    let counts = caches
        .execution_report()
        .into_iter()
        .collect::<BTreeMap<_, _>>();
    assert!(counts["reusable_executions"].unwrap() > counts["physical_plans"].unwrap());
    println!(
        "{}",
        serde_json::json!({"experiment":"fixed_point_rounds","nodes":nodes,"rules":2,"heads":1,"rounds":result.rounds,"setup_seconds":setup_seconds,"execute_seconds":execute_seconds,"counts":counts,"pool_peak_bytes":peak.max_reserved(),"process_peak_rss_bytes":super::cache_journey::process_peak_rss(),"phase_costs":"pse_rules tracing events in this log"})
    );
    [
        "sql_bindings",
        "analyses",
        "optimizations",
        "physical_plans",
    ]
    .map(|name| counts[name])
}
