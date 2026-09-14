// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::expect_used,
    reason = "fixture setup fails directly on invalid scratch resources or constants"
)]

//! Shared finite-pool, reservation and driver-cancellation integration fixtures.

use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use datafusion_execution::memory_pool::{MemoryLimit, UnboundedMemoryPool};
use pse_catalog::{ExecutionSettings, ThreadBudget};
use pse_ids::{MemoryReserver, ReserveError};
use pse_runtime::{CancelSource, PoolReserver, ResourceBudget, SharedRuntime};

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "pse-runtime-shared-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).expect("create owned scratch directory");
        Self(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).expect("remove owned scratch directory");
    }
}

fn budget(directory: &Scratch, limit: usize) -> ResourceBudget {
    ResourceBudget {
        memory_limit_bytes: NonZeroUsize::new(limit).expect("nonzero limit"),
        spill_dir: directory.0.clone(),
        max_temp_dir_bytes: 1024,
        top_consumers: NonZeroUsize::new(2).expect("nonzero count"),
        threads: ThreadBudget {
            pool_threads: NonZeroUsize::new(2).expect("nonzero"),
            target_partitions: NonZeroUsize::new(2).expect("nonzero"),
        },
        execution: ExecutionSettings::default(),
        hashing_may_use_pool: false,
    }
}

#[test]
fn runtime_handles_and_platform_consumers_share_one_finite_pool() {
    let directory = Scratch::new();
    let runtime = SharedRuntime::build(budget(&directory, 96)).expect("runtime builds");
    let left = runtime.runtime_env();
    let right = runtime.runtime_env();
    assert!(Arc::ptr_eq(&left, &right));
    assert!(Arc::ptr_eq(&left.memory_pool, &runtime.pool()));
    assert!(matches!(
        left.memory_pool.memory_limit(),
        MemoryLimit::Finite(96)
    ));
    assert_eq!(left.disk_manager.max_temp_directory_size(), 1024);
    let reserver = runtime.reserver();
    let mut first = reserver.open("session:first");
    first.try_grow(64).expect("fits");
    let mut second = reserver.open("session:second");
    let error = second.try_grow(64).expect_err("shared limit exceeded");
    assert!(
        matches!(error, ReserveError::Exhausted { owner, limit_hint, .. } if owner == "session:second" && limit_hint.contains("datafusion.runtime.memory_limit"))
    );
    assert_eq!(reserver.reserved(), 64);
    let report = runtime.report().expect("resource observation");
    assert_eq!(report.pool_peak_bytes, 64);
    assert_eq!(report.pool_reserved_now, 64);
    assert_eq!(
        report.top_consumers.first(),
        Some(&("session:first".to_owned(), 64))
    );
    drop(first);
    drop(second);
    assert_eq!(reserver.reserved(), 0);
    assert_eq!(runtime.report().expect("report").pool_peak_bytes, 64);
}

#[test]
fn grow_overflow_shrink_and_release_are_bounded() {
    let directory = Scratch::new();
    let runtime = SharedRuntime::build(budget(&directory, 64)).expect("runtime builds");
    let reserver = runtime.reserver();
    let mut reservation = reserver.open("overflow");
    reservation.try_grow(32).expect("fits");
    assert!(reservation.try_grow(usize::MAX).is_err());
    assert_eq!(reservation.size(), 32);
    reservation.shrink(usize::MAX);
    assert_eq!(reserver.reserved(), 0);
    reservation.try_grow(64).expect("released claim fits again");
    reservation.release();
    reservation.release();
    assert_eq!(reserver.reserved(), 0);
}

#[test]
fn unbounded_pools_and_invalid_execution_settings_are_refused() {
    assert!(PoolReserver::new(Arc::new(UnboundedMemoryPool::default())).is_err());
    let directory = Scratch::new();
    let mut settings = budget(&directory, 64);
    settings.execution.batch_size = 0;
    assert!(SharedRuntime::build(settings).is_err());
    let mut settings = budget(&directory, 64);
    settings.execution.time_zone = "local".to_owned();
    assert!(SharedRuntime::build(settings).is_err());
}

#[tokio::test]
async fn cancellation_wakes_current_and_late_waiters_and_cpu_tokens() {
    let source = CancelSource::new();
    let token = source.token();
    let waiting = source.clone();
    let task = tokio::spawn(async move {
        waiting.cancelled().await;
    });
    tokio::task::yield_now().await;
    source.cancel();
    source.cancel();
    tokio::time::timeout(std::time::Duration::from_secs(1), task)
        .await
        .expect("waiter wakes")
        .expect("task returns");
    source.cancelled().await;
    assert!(source.checkpoint().is_err());
    assert!(token.checkpoint().is_err());
}

#[test]
fn simultaneous_consumers_cannot_exceed_the_shared_limit() {
    let directory = Scratch::new();
    let runtime = SharedRuntime::build(budget(&directory, 12)).expect("runtime builds");
    let barrier = Arc::new(std::sync::Barrier::new(2));
    let threads: Vec<_> = (0..2)
        .map(|index| {
            let reserver = runtime.reserver();
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                let mut reservation = reserver.open(&format!("concurrent:{index}"));
                barrier.wait();
                let success = reservation.try_grow(8).is_ok();
                barrier.wait();
                success
            })
        })
        .collect();
    let successes = threads
        .into_iter()
        .map(|thread| usize::from(thread.join().expect("consumer returns")))
        .sum::<usize>();
    assert_eq!(successes, 1);
    assert_eq!(runtime.reserver().reserved(), 0);
    assert_eq!(runtime.report().expect("report").pool_peak_bytes, 8);
}
