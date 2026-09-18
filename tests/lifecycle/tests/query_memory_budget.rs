// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual scan admission and engine operators share the charged snapshot budget.

mod support;
use pse_catalog::CatalogError;
use pse_catalog::session::SnapshotSession;
use pse_ids::{CancellationToken, MemoryReserver, ReserveError};

#[tokio::test]
async fn two_sessions_share_input_charges_and_only_64_kib_of_query_headroom() {
    const LIMIT: usize = 512 << 20;
    let directory = tempfile::tempdir().expect("store and spill root");
    let runtime = support::runtime(directory.path(), LIMIT);
    let reg = support::registry();
    let cancel = CancellationToken::default();
    let left = support::session(&runtime, reg, 200_000);
    let right = left.clone();
    let snapshot_bytes = runtime.reserver().reserved();
    assert!(
        snapshot_bytes >= 200_000 * 16,
        "both full columns remain charged"
    );
    let mut competing = runtime.reserver().open("fixture:competing-work");
    competing
        .try_grow(LIMIT - snapshot_bytes - (64 << 10))
        .expect("leave exact query headroom");
    let held = runtime.reserver().reserved();
    assert_eq!(LIMIT - held, 64 << 10);
    let error = left
        .sql("SELECT id FROM authored.samples ORDER BY value", &cancel)
        .await
        .expect_err("actual scan validation cannot fit the remaining budget");
    assert!(
        matches!(&error, CatalogError::Reserve(ReserveError::Exhausted { owner, .. }) if !owner.is_empty())
            || matches!(&error, CatalogError::ResourceLimit { consumer, .. } if !consumer.is_empty()),
        "{error}"
    );

    assert_eq!(
        runtime.reserver().reserved() - observation_bytes(&runtime),
        held,
        "failed query releases every operator claim; attempt observations remain owned"
    );
    assert_cancelled(&right, &runtime, held + observation_bytes(&runtime)).await;
    drop(competing);
    drop(left);
    drop(right);
    assert_eq!(runtime.reserver().reserved(), 0);
    let report = runtime.report().expect("separate pool/process observation");
    assert!(report.pool_peak_bytes <= LIMIT);
    assert!(report.pool_peak_bytes >= held);
    assert!(report.process_peak_rss_bytes.is_some());
}

const EXPANSION_SQL: &str = "SELECT a.id AS a_id, b.id AS b_id, c.id AS c_id, \
    d.id AS d_id, e.id AS e_id \
    FROM authored.samples a CROSS JOIN authored.samples b \
    CROSS JOIN authored.samples c CROSS JOIN authored.samples d \
    CROSS JOIN authored.samples e \
    ORDER BY a.value, b.value, c.value, d.value, e.value";

#[tokio::test]
async fn small_input_admission_fits_64_kib_but_expansion_exhausts_the_engine() {
    const LIMIT: usize = 512 << 20;
    const HEADROOM: usize = 64 << 10;
    let directory = tempfile::tempdir().expect("store and spill root");
    let runtime = support::runtime(directory.path(), LIMIT);
    let reg = support::registry();
    let cancel = CancellationToken::default();
    let left = support::session(&runtime, reg, 8);
    let right = left.clone();
    let snapshot_bytes = runtime.reserver().reserved();
    assert!(
        snapshot_bytes >= 8 * 16,
        "the source columns remain charged"
    );
    let mut competing = runtime.reserver().open("fixture:competing-work");
    competing
        .try_grow(LIMIT - snapshot_bytes - HEADROOM)
        .expect("leave exact query headroom after opening both sessions");
    let held = runtime.reserver().reserved();
    assert_eq!(LIMIT - held, HEADROOM);

    // This successful public query proves that actual source admission and result
    // ownership fit the remaining pool; no validation stage is bypassed for the join.
    let scan = right
        .sql("SELECT id, value FROM authored.samples", &cancel)
        .await
        .expect("small scan fits");
    assert_eq!(
        scan.iter()
            .map(pse_relations::RecordBatch::num_rows)
            .sum::<usize>(),
        8
    );
    drop(scan);
    assert_eq!(
        runtime.reserver().reserved() - observation_bytes(&runtime),
        held
    );

    let error = left
        .sql(EXPANSION_SQL, &cancel)
        .await
        .expect_err("finite engine operator budget");
    assert!(
        matches!(&error, CatalogError::ResourceLimit { consumer, config_keys, .. }
        if !consumer.is_empty() && consumer != "unattributed"
        && config_keys.iter().any(|key| key == "datafusion.runtime.memory_limit")),
        "expected a named engine consumer with the bound pool configuration, got {error:?}"
    );
    assert_eq!(
        runtime.reserver().reserved() - observation_bytes(&runtime),
        held,
        "all failed operator claims are released"
    );
    assert_cancelled(&right, &runtime, held + observation_bytes(&runtime)).await;

    drop(competing);
    assert_eq!(
        runtime.reserver().reserved() - observation_bytes(&runtime),
        snapshot_bytes
    );
    let result = left
        .sql(EXPANSION_SQL, &cancel)
        .await
        .expect("the same expansion succeeds after releasing the competitor");
    assert_eq!(
        result
            .iter()
            .map(pse_relations::RecordBatch::num_rows)
            .sum::<usize>(),
        32_768,
        "all 8^5 actual join rows survive the sort"
    );
    assert!(
        runtime.reserver().reserved() > snapshot_bytes,
        "returned values retain their pool ownership"
    );
    drop(result);
    assert_eq!(
        runtime.reserver().reserved() - observation_bytes(&runtime),
        snapshot_bytes
    );
    drop((left, right));
    assert_eq!(runtime.reserver().reserved(), 0);
    let report = runtime.report().expect("shared pool observation");
    assert!(report.pool_peak_bytes <= LIMIT);
    assert!(report.pool_peak_bytes >= held);
}

async fn assert_cancelled(
    session: &SnapshotSession,
    runtime: &pse_runtime::SharedRuntime,
    held: usize,
) {
    let cancelled = CancellationToken::default();
    cancelled.cancel();
    assert!(matches!(
        session
            .sql("SELECT id FROM authored.samples", &cancelled)
            .await,
        Err(CatalogError::Cancelled | CatalogError::Canon(pse_ids::CanonError::Cancelled))
    ));
    assert_eq!(
        runtime.reserver().reserved(),
        held,
        "cancelling one session retains only live shared owners"
    );
}

#[expect(
    clippy::expect_used,
    reason = "test-only accounting helper requires a valid runtime report"
)]
fn observation_bytes(runtime: &pse_runtime::SharedRuntime) -> usize {
    runtime
        .report()
        .expect("accounted attempt observations")
        .top_consumers
        .iter()
        .filter(|(name, _)| {
            matches!(
                name.as_str(),
                "session:execution-observations"
                    | "session:plan-observation"
                    | "session:physical-plan-observation"
            )
        })
        .map(|(_, bytes)| *bytes)
        .sum()
}
