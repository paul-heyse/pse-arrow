// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded, passive measurements from existing executor and selected-future owners.

use serde::{Deserialize, Serialize};
use std::{
    future::Future,
    sync::{
        Arc, Mutex, OnceLock, Weak,
        atomic::{AtomicU64, Ordering},
    },
    time::SystemTime,
};

const MAX_EXECUTORS: usize = 8;

pub use pse_diagnostics::{ExecutorRole, NativeTaskClass, TaskMetricsScope};

fn class_index(class: NativeTaskClass) -> usize {
    match class {
        NativeTaskClass::Workflow => 0,
        NativeTaskClass::MathSubmission => 1,
        NativeTaskClass::MathJob => 2,
    }
}

fn monitors() -> &'static [tokio_metrics::TaskMonitor; 3] {
    static MONITORS: OnceLock<[tokio_metrics::TaskMonitor; 3]> = OnceLock::new();
    MONITORS.get_or_init(|| std::array::from_fn(|_| tokio_metrics::TaskMonitor::new()))
}

/// Measure a selected future with the library's poll/wake instrumentation.
/// Dropping this wrapper says nothing about a separately running native job.
pub fn instrument<F: Future>(class: NativeTaskClass, future: F) -> tokio_metrics::Instrumented<F> {
    monitors()[class_index(class)].instrument(future)
}

#[derive(Debug)]
struct ExecutorSource {
    identity: u64,
    role: ExecutorRole,
    registered_unix_micros: u128,
    handle: tokio::runtime::Handle,
}

#[derive(Debug, Default)]
struct Registry {
    sources: Vec<Weak<ExecutorSource>>,
}

fn registry() -> &'static Mutex<Registry> {
    static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();
    REGISTRY.get_or_init(Mutex::default)
}

static NEXT_EXECUTOR: AtomicU64 = AtomicU64::new(1);
static REFUSED_EXECUTORS: AtomicU64 = AtomicU64::new(0);

/// Owned only by the existing executor composition root, never a diagnostic reader.
/// A Tokio handle does not prevent the actual runtime from shutting down.
#[derive(Debug)]
pub struct ExecutorRegistration {
    _source: Option<Arc<ExecutorSource>>,
}

/// Register one existing executor without building or scheduling another runtime.
/// Drop the token before its executor owner; readers keep no registration or handle.
pub fn register_executor(
    handle: &tokio::runtime::Handle,
    role: ExecutorRole,
) -> ExecutorRegistration {
    let Ok(mut registry) = registry().lock() else {
        REFUSED_EXECUTORS.fetch_add(1, Ordering::Relaxed);
        return ExecutorRegistration { _source: None };
    };
    registry.sources.retain(|source| source.strong_count() != 0);
    if registry.sources.len() == MAX_EXECUTORS {
        REFUSED_EXECUTORS.fetch_add(1, Ordering::Relaxed);
        return ExecutorRegistration { _source: None };
    }
    let source = Arc::new(ExecutorSource {
        identity: NEXT_EXECUTOR.fetch_add(1, Ordering::Relaxed),
        role,
        registered_unix_micros: unix_micros(),
        handle: handle.clone(),
    });
    registry.sources.push(Arc::downgrade(&source));
    ExecutorRegistration {
        _source: Some(source),
    }
}

/// Coverage of the bounded executor source registry at this read.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RuntimeMetricsCoverage {
    /// False when source synchronization was unavailable; an empty list then is not zero load.
    pub executor_registry_available: bool,
    /// Cumulative refused registrations, including registry pressure or synchronization failure.
    pub refused_executor_registrations: u64,
    /// Only explicitly instrumented futures are covered by task class measurements.
    pub task_scope: TaskMetricsScope,
}

/// Copied stable Tokio facts. Gauges are independently read, not an atomic scheduler census.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExecutorMetricsSnapshot {
    /// Process-local registration identity, scoped by the enclosing process locator.
    pub identity: u64,
    /// Fixed executor composition root.
    pub role: ExecutorRole,
    /// Wall time this source was registered; counters originate at runtime construction.
    pub registered_unix_micros: u128,
    /// Configured worker count, in threads.
    pub workers: usize,
    /// Tokio's approximate current count, in tasks; not a settlement proof.
    pub alive_tasks: usize,
    /// Current global injection queue depth, in tasks; excludes worker-local queues.
    pub global_queue_depth: usize,
    /// Sum of cumulative worker busy durations, in nanoseconds since runtime construction.
    /// Unavailable on targets without 64-bit atomics; not process or operation CPU time.
    pub worker_busy_nanos: Option<u128>,
}

/// Cumulative library task monitor facts for a bounded class, in counts and nanoseconds.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TaskClassMetricsSnapshot {
    /// Fixed selected future class.
    pub class: NativeTaskClass,
    /// Number of wrappers created, not native jobs admitted.
    pub instrumented_count: u64,
    /// Wrappers dropped, including cancellation; never a native settlement counter.
    pub dropped_count: u64,
    /// Recorded future polls.
    pub poll_count: u64,
    /// Cumulative time within future polls, in nanoseconds; not CPU time.
    pub poll_nanos: u128,
    /// Recorded wake-to-poll schedules.
    pub scheduled_count: u64,
    /// Cumulative wake-to-poll time, in nanoseconds.
    pub scheduled_nanos: u128,
    /// Recorded pending-to-wake idle intervals.
    pub idle_count: u64,
    /// Cumulative pending-to-wake idle time, in nanoseconds.
    pub idle_nanos: u128,
}

/// One read of the same typed sources used by checkpoints and optional OTel export.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RuntimeMetricsSnapshot {
    /// Actual process/start locator; absence of start ticks limits identity evidence.
    pub locator: crate::observation::ProcessLocator,
    /// Wall time immediately before independently sampled source reads.
    pub sampled_unix_micros: u128,
    /// Explicit registry losses and selected task coverage.
    pub coverage: RuntimeMetricsCoverage,
    /// At most eight actual registered executors.
    pub executors: Vec<ExecutorMetricsSnapshot>,
    /// Exactly three process-wide selected future classes.
    pub task_classes: Vec<TaskClassMetricsSnapshot>,
}

/// Copy current and cumulative native facts without retaining an executor or native lease.
pub fn snapshot() -> RuntimeMetricsSnapshot {
    let sampled_unix_micros = unix_micros();
    let sources = registry().lock().map(|mut registry| {
        registry.sources.retain(|source| source.strong_count() != 0);
        registry
            .sources
            .iter()
            .filter_map(Weak::upgrade)
            .collect::<Vec<_>>()
    });
    let available = sources.is_ok();
    let executors = sources
        .unwrap_or_default()
        .iter()
        .map(|source| {
            let metrics = source.handle.metrics();
            let workers = metrics.num_workers();
            #[cfg(target_has_atomic = "64")]
            let worker_busy_nanos = Some((0..workers).fold(0_u128, |total, worker| {
                total.saturating_add(metrics.worker_total_busy_duration(worker).as_nanos())
            }));
            #[cfg(not(target_has_atomic = "64"))]
            let worker_busy_nanos = None;
            ExecutorMetricsSnapshot {
                identity: source.identity,
                role: source.role,
                registered_unix_micros: source.registered_unix_micros,
                workers,
                alive_tasks: metrics.num_alive_tasks(),
                global_queue_depth: metrics.global_queue_depth(),
                worker_busy_nanos,
            }
        })
        .collect();
    let task_classes = NativeTaskClass::ALL
        .iter()
        .copied()
        .map(|class| {
            let metrics = monitors()[class_index(class)].cumulative();
            TaskClassMetricsSnapshot {
                class,
                instrumented_count: metrics.instrumented_count,
                dropped_count: metrics.dropped_count,
                poll_count: metrics.total_poll_count,
                poll_nanos: metrics.total_poll_duration.as_nanos(),
                scheduled_count: metrics.total_scheduled_count,
                scheduled_nanos: metrics.total_scheduled_duration.as_nanos(),
                idle_count: metrics.total_idled_count,
                idle_nanos: metrics.total_idle_duration.as_nanos(),
            }
        })
        .collect();
    RuntimeMetricsSnapshot {
        locator: crate::observation::ProcessLocator::current(),
        sampled_unix_micros,
        coverage: RuntimeMetricsCoverage {
            executor_registry_available: available,
            refused_executor_registrations: REFUSED_EXECUTORS.load(Ordering::Relaxed),
            task_scope: TaskMetricsScope::SelectedProcessFutures,
        },
        executors,
        task_classes,
    }
}

fn unix_micros() -> u128 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_metrics_track_actual_queue_and_registration_lifetime() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let registration = register_executor(runtime.handle(), ExecutorRole::CanonicalTools);
        let identity = registration._source.as_ref().unwrap().identity;
        let (release, pending) = tokio::sync::oneshot::channel::<()>();
        let task = runtime.spawn(async move {
            pending.await.unwrap();
        });
        let image = snapshot();
        let source = image
            .executors
            .iter()
            .find(|source| source.identity == identity)
            .unwrap();
        assert_eq!(source.workers, 1);
        assert_eq!(source.alive_tasks, 1);
        assert_eq!(source.global_queue_depth, 1);
        assert_eq!(image.locator.pid, std::process::id());
        release.send(()).unwrap();
        runtime.block_on(task).unwrap();
        let image = snapshot();
        let source = image
            .executors
            .iter()
            .find(|source| source.identity == identity)
            .unwrap();
        assert_eq!(source.alive_tasks, 0);
        assert_eq!(source.global_queue_depth, 0);
        drop(registration);
        assert!(
            !snapshot()
                .executors
                .iter()
                .any(|source| source.identity == identity)
        );
        // Copied images do not keep either the registration or executor alive.
        drop(runtime);
        assert_eq!(source.workers, 1);
    }

    #[test]
    fn runtime_metrics_task_library_records_pending_wake_and_poll() {
        let before = snapshot()
            .task_classes
            .remove(class_index(NativeTaskClass::MathJob));
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let (release, pending) = tokio::sync::oneshot::channel::<()>();
        let mut future = Box::pin(instrument(NativeTaskClass::MathJob, async move {
            pending.await.unwrap();
        }));
        runtime.block_on(async {
            assert!(futures_util::poll!(future.as_mut()).is_pending());
            release.send(()).unwrap();
            future.await;
        });
        let after = snapshot()
            .task_classes
            .remove(class_index(NativeTaskClass::MathJob));
        assert!(after.instrumented_count > before.instrumented_count);
        assert!(after.dropped_count > before.dropped_count);
        assert!(after.poll_count >= before.poll_count + 2);
        assert!(after.idle_count > before.idle_count);
        assert!(after.scheduled_count > before.scheduled_count);
    }

    #[cfg(target_has_atomic = "64")]
    #[test]
    fn runtime_metrics_worker_busy_and_queue_follow_actual_executor() {
        use std::sync::atomic::AtomicBool;
        use std::time::Duration;
        let (parked, park) = std::sync::mpsc::channel();
        let completed = Arc::new(AtomicBool::new(false));
        let on_park = completed.clone();
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .on_thread_park(move || {
                if on_park.load(Ordering::Acquire) {
                    let _ = parked.send(());
                }
            })
            .build()
            .unwrap();
        let registration = register_executor(runtime.handle(), ExecutorRole::Worker);
        let identity = registration._source.as_ref().unwrap().identity;
        let (entered, enter) = std::sync::mpsc::channel();
        let (release, wait) = std::sync::mpsc::channel();
        let task = runtime.spawn(async move {
            entered.send(()).unwrap();
            wait.recv().unwrap();
            completed.store(true, Ordering::Release);
        });
        enter.recv_timeout(Duration::from_secs(5)).unwrap();
        let queued = runtime.spawn(async {});
        let busy = snapshot();
        // Release the worker before assertions, so a failing assertion cannot strand shutdown.
        release.send(()).unwrap();
        runtime.block_on(async {
            task.await.unwrap();
            queued.await.unwrap();
        });
        park.recv_timeout(Duration::from_secs(5)).unwrap();
        // Tokio calls on_thread_park immediately before submitting its busy-time batch.
        // Synchronization establishes the work; wait only for that library publication.
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        let drained = loop {
            let image = snapshot();
            let source = image
                .executors
                .iter()
                .find(|source| source.identity == identity)
                .unwrap();
            if source.worker_busy_nanos.is_some_and(|nanos| nanos > 0)
                || std::time::Instant::now() >= deadline
            {
                break image;
            }
            std::thread::yield_now();
        };
        let busy = busy
            .executors
            .iter()
            .find(|source| source.identity == identity)
            .unwrap();
        let drained = drained
            .executors
            .iter()
            .find(|source| source.identity == identity)
            .unwrap();
        assert_eq!(busy.workers, 1);
        assert!(busy.alive_tasks >= 2);
        assert_eq!(busy.global_queue_depth, 1);
        assert!(drained.worker_busy_nanos.unwrap() > 0);
        assert!(drained.worker_busy_nanos >= busy.worker_busy_nanos);
        assert_eq!(drained.global_queue_depth, 0);
    }
}
