// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Comparable completed executions under observation and contention policies.
use criterion::Criterion;
use pse_columnar::CancellationToken;
use pse_engine::{
    cache_service::CacheBudget,
    session::{ExecutionSettings, PreparedComputation, ThreadBudget, assurance::ObservationPolicy},
};
use pse_testkit::capture::{Capture, Coverage};
use std::{collections::BTreeMap, hint::black_box};

pub(super) fn measure(c: &mut Criterion) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let registry = pse_schema::shared_registry().unwrap();
    let cancel = CancellationToken::new();
    let mut receipts = Vec::new();
    let mut group = c.benchmark_group("integrated/observation");
    for threads in [1, 4] {
        for rows in [1_i64, 65_536] {
            for concurrency in [1, 4] {
                for policy in [
                    ObservationPolicy::Off,
                    ObservationPolicy::Contract,
                    ObservationPolicy::Diagnostic,
                ] {
                    let mut cache = CacheBudget::disabled(128 << 20);
                    cache.concurrent_queries = threads.try_into().unwrap();
                    let fixture = pse_testkit::NativeFixture::with_settings(
                        (128 << 20).try_into().unwrap(),
                        ThreadBudget {
                            pool_threads: threads.try_into().unwrap(),
                            target_partitions: threads.try_into().unwrap(),
                        },
                        ExecutionSettings::default(),
                        cache,
                    )
                    .unwrap();
                    let session = fixture
                        .factory
                        .clone()
                        .with_observation(policy)
                        .candidate_checked(BTreeMap::new(), registry.clone(), &cancel)
                        .unwrap();
                    let prepared = runtime.block_on(session.prepare_sql(
                        &format!("SELECT sum(value) AS total FROM (SELECT unnest(range(0, {rows})) AS value) WHERE value % 2 = 0"),
                        &cancel,
                    )).unwrap();
                    let even = (rows + 1) / 2;
                    let expected = even * (even - 1);
                    let before = fixture
                        .resources
                        .caches
                        .execution_report()
                        .into_iter()
                        .collect::<BTreeMap<_, _>>();
                    let evidence =
                        runtime.block_on(run(&prepared, &cancel, concurrency, policy, expected));
                    let after = fixture
                        .resources
                        .caches
                        .execution_report()
                        .into_iter()
                        .collect::<BTreeMap<_, _>>();
                    group.bench_function(
                        format!("{policy:?}/rows-{rows}/workers-{threads}/queries-{concurrency}"),
                        |b| {
                            b.iter(|| {
                                black_box(runtime.block_on(run(
                                    &prepared,
                                    &cancel,
                                    concurrency,
                                    policy,
                                    expected,
                                )))
                            });
                        },
                    );
                    receipts.push(serde_json::json!({
                        "policy": format!("{policy:?}"), "rows": rows, "pool_threads": threads,
                        "target_partitions": threads, "concurrent_query_limit": threads,
                        "requested_queries": concurrency, "pool_budget_bytes": 128 << 20,
                        "result_cache_bytes": 0, "capture_spans_per_query": 4096,
                        "capture_bytes_per_query": 1 << 20, "captured_spans_and_field_bytes": evidence,
                        "counts_before_probe": before, "counts_after_probe": after,
                        "pool_peak_bytes": fixture.resources.peak.max_reserved(),
                        "pool_retained_bytes": fixture.resources.pool.reserved(),
                        "preparation": "retained; each execution is drained; result caching disabled"
                    }));
                }
            }
        }
    }
    group.finish();
    if let Some(output) = std::env::var_os("PSE_ACCEPTANCE_OUTPUT") {
        std::fs::write(
            std::path::PathBuf::from(output).join("observation-counts.json"),
            serde_json::to_vec_pretty(&receipts).unwrap(),
        )
        .unwrap();
    }
}

async fn run(
    prepared: &PreparedComputation,
    cancel: &CancellationToken,
    concurrency: usize,
    policy: ObservationPolicy,
    expected: i64,
) -> Vec<(usize, usize)> {
    use tracing::instrument::WithSubscriber;
    futures_util::future::join_all((0..concurrency).map(|_| async {
        let coverage = if policy == ObservationPolicy::Off {
            Coverage::Disabled
        } else {
            Coverage::Complete
        };
        let capture = Capture::new(4096, 1 << 20).with_coverage(coverage);
        let result = prepared
            .clone()
            .execute(cancel)
            .with_subscriber(capture.dispatch())
            .await
            .unwrap();
        assert_eq!(result.batches().len(), 1);
        let batch = &result.batches()[0];
        assert_eq!(batch.num_rows(), 1);
        assert_eq!(
            batch
                .column(0)
                .as_any()
                .downcast_ref::<arrow::array::Int64Array>()
                .unwrap()
                .value(0),
            expected
        );
        drop(result);
        assert_eq!(capture.coverage(), coverage);
        let (spans, truncated) = capture.snapshot();
        assert!(!truncated);
        let bytes = spans
            .iter()
            .flat_map(|span| span.fields.values())
            .map(String::len)
            .sum();
        (spans.len(), bytes)
    }))
    .await
}
