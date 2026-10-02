// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use datafusion::execution::memory_pool::FairSpillPool;
use std::sync::atomic::Ordering;
pub(super) fn service() -> Arc<MathService> {
    service_and_cache().0
}
fn service_and_cache() -> (
    Arc<MathService>,
    Arc<pse_engine::cache_service::NativeCacheService>,
) {
    service_in(512 << 20)
}
fn service_in(
    pool: usize,
) -> (
    Arc<MathService>,
    Arc<pse_engine::cache_service::NativeCacheService>,
) {
    service_with(pool, 1 << 20)
}
/// A service on a `pool`-byte pool whose native jobs admit `foreign` bytes of
/// library-owned memory.
fn service_with(
    pool: usize,
    foreign: usize,
) -> (
    Arc<MathService>,
    Arc<pse_engine::cache_service::NativeCacheService>,
) {
    let pool: Arc<dyn MemoryPool> = Arc::new(FairSpillPool::new(pool));
    let native = pse_engine::cache_service::NativeCacheService::new(
        pse_engine::cache_service::CacheBudget::disabled(1024),
        &pool,
    )
    .unwrap();
    let service = MathService::new(
        pool,
        Arc::new(tokio::sync::Semaphore::new(2)),
        2,
        MathPolicy {
            foreign_bytes: foreign,
            worker_bytes: 8 << 20,
            workspace_bytes: 16 << 20,
            ..MathPolicy::default()
        },
        &native,
    );
    (service, native)
}
#[tokio::test]
async fn cancellation_before_entry_releases_admission_without_starting() {
    let s = service();
    let permit = s.cpu.clone().acquire_many_owned(2).await.unwrap();
    let control = FlightCancellation::default();
    let called = Arc::new(AtomicBool::new(false));
    let flag = called.clone();
    let baseline = s.pool.reserved();
    let mut job = Box::pin(s.job(2, 0, control.clone(), move |_| {
        flag.store(true, Ordering::SeqCst);
        Ok(())
    }));
    assert!(futures_util::poll!(job.as_mut()).is_pending());
    control.cancel();
    assert!(matches!(job.await, Err(MathRuntimeError::Cancelled)));
    assert!(!called.load(Ordering::SeqCst));
    assert_eq!(s.pool.reserved(), baseline);
    drop(permit);
}
#[derive(Debug)]
struct SlowDrop {
    entered: Arc<tokio::sync::Notify>,
    gate: Arc<(Mutex<bool>, std::sync::Condvar)>,
    thread: std::thread::ThreadId,
}
impl Drop for SlowDrop {
    fn drop(&mut self) {
        assert_eq!(self.thread, std::thread::current().id());
        self.entered.notify_one();
        let (lock, cv) = &*self.gate;
        let mut ready = lock.lock().unwrap();
        while !*ready {
            ready = cv.wait(ready).unwrap();
        }
    }
}
thread_local! {static TLS:std::cell::RefCell<Option<SlowDrop>>=const{std::cell::RefCell::new(None)};}
#[tokio::test]
async fn abandoned_caller_holds_pool_and_cpu_through_tls_destructor() {
    let s = service();
    let entered = Arc::new(tokio::sync::Notify::new());
    let gate = Arc::new((Mutex::new(false), std::sync::Condvar::new()));
    let e = entered.clone();
    let g = gate.clone();
    let control = FlightCancellation::default();
    let baseline = s.pool.reserved();
    let mut job = Box::pin(s.job(2, 1024, control, move |_| {
        let non_send = std::rc::Rc::new(3);
        TLS.set(Some(SlowDrop {
            entered: e,
            gate: g,
            thread: std::thread::current().id(),
        }));
        Ok(*non_send)
    }));
    assert!(futures_util::poll!(job.as_mut()).is_pending());
    entered.notified().await;
    drop(job);
    assert_eq!(s.cpu.available_permits(), 0);
    assert!(s.pool.reserved() > baseline);
    assert_eq!(s.jobs.available_permits(), s.policy.jobs - 1);
    *gate.0.lock().unwrap() = true;
    gate.1.notify_one();
    while s.jobs.available_permits() != s.policy.jobs {
        tokio::task::yield_now().await;
    }
    assert_eq!(s.cpu.available_permits(), 2);
    assert_eq!(s.pool.reserved(), baseline);
}
#[tokio::test]
async fn retained_result_capacity_transfers_without_a_second_reservation() {
    let s = service();
    let pool = s.pool.clone();
    let bytes = 1 << 20;
    let active =
        bytes + s.policy.stack_bytes + s.policy.foreign_bytes + s.policy.inner_session_bytes;
    let (result, owner) = s
        .job_retained(1, bytes, FlightCancellation::default(), move |_| {
            assert_eq!(pool.reserved(), active);
            Ok((vec![3_u64; 16], 16 * size_of::<u64>()))
        })
        .await
        .unwrap();
    assert_eq!(result.len(), 16);
    assert_eq!(owner.size(), 128);
    assert_eq!(s.pool.reserved(), 128);
    drop(result);
    drop(owner);
    assert_eq!(s.pool.reserved(), 0);
}

#[tokio::test]
async fn retained_result_beyond_the_working_allowance_is_charged_at_its_extent() {
    let s = service();
    let (result, owner) = s
        .job_retained(1, 0, FlightCancellation::default(), move |_| {
            Ok((vec![3_u64; 1 << 20], 8 << 20))
        })
        .await
        .unwrap();
    assert_eq!(owner.size(), 8 << 20);
    assert_eq!(s.pool.reserved(), 8 << 20);
    drop((result, owner));
    assert_eq!(s.pool.reserved(), 0);
    // A product the pool cannot admit is refused after the work, as a pool limit.
    let refused = s
        .job_retained(1, 0, FlightCancellation::default(), move |_| {
            Ok(((), 1 << 30))
        })
        .await;
    assert!(matches!(refused, Err(MathRuntimeError::Pool(_))));
    assert_eq!(s.pool.reserved(), 0);
}

#[tokio::test]
async fn parallel_jobs_admit_library_team_stacks_and_release_them_after_join() {
    let service = service();
    let pool = service.pool.clone();
    let expected = 1024
        + 3 * service.policy.stack_bytes
        + service.policy.foreign_bytes
        + service.policy.inner_session_bytes;
    service
        .job(2, 1024, Default::default(), move |_| {
            assert_eq!(pool.reserved(), expected);
            Ok(())
        })
        .await
        .unwrap();
    assert_eq!(service.pool.reserved(), 0);
    assert_eq!(service.cpu.available_permits(), 2);
}

#[test]
fn worker_budget_releases_dropped_capacity() {
    // The budget refuses a worker beyond its capacity and releases a dropped one.
    let budget = WorkerBudget::new(100);
    let a = budget.charge(60).unwrap();
    assert!(matches!(
        budget.charge(60),
        Err(MathRuntimeError::Limit("worker storage"))
    ));
    drop(a);
    let b = budget.charge(60).unwrap();
    assert_eq!(budget.used(), 60);
    drop(b);
    assert_eq!(budget.used(), 0);
}
