// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(
    unsafe_code,
    reason = "one serialized owner for the native HiGHS global scheduler reset"
)]
//! Shared scheduler ownership for direct HiGHS and Uno's HiGHS subproblems.
use crate::{ProblemError, solve::Execution};
#[cfg(feature = "highs")]
use std::sync::RwLockReadGuard;
use std::sync::{Condvar, Mutex, RwLock, TryLockError};
#[cfg(feature = "uno")]
use std::sync::{
    RwLockWriteGuard,
    atomic::{AtomicUsize, Ordering},
};
static LIFECYCLE: RwLock<()> = RwLock::new(());
static CHANGE: Mutex<()> = Mutex::new(());
static RELEASED: Condvar = Condvar::new();
#[cfg(feature = "uno")]
static WAITERS: AtomicUsize = AtomicUsize::new(0);
#[cfg(feature = "uno")]
static SCHEDULER_CHANGED: pse_diagnostics::WakeSignal = pse_diagnostics::WakeSignal::new();
#[cfg(feature = "uno")]
pub(crate) fn watch(waker: &std::task::Waker) -> pse_diagnostics::WakeWatch {
    SCHEDULER_CHANGED.watch(waker)
}
thread_local! {static ACTIVE:std::cell::Cell<bool>=const {std::cell::Cell::new(false)};}

struct ReleasedWake;
impl std::task::Wake for ReleasedWake {
    fn wake(self: std::sync::Arc<Self>) { notify(); }
    fn wake_by_ref(self: &std::sync::Arc<Self>) { notify(); }
}
fn stop_watches(execution: &Execution) -> (pse_diagnostics::CancellationWatch, pse_diagnostics::WakeWatch) {
    let wake = std::task::Waker::from(std::sync::Arc::new(ReleasedWake));
    (execution.cancellation_control().watch(&wake), execution.abandonment.watch(&wake))
}
fn wait(changed: std::sync::MutexGuard<'_, ()>, execution: &Execution) -> Result<(), ProblemError> {
    if let Some(deadline) = execution.scope()?.deadline() {
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        drop(RELEASED.wait_timeout(changed, remaining).unwrap_or_else(std::sync::PoisonError::into_inner));
    } else {
        drop(RELEASED.wait(changed).unwrap_or_else(std::sync::PoisonError::into_inner));
    }
    Ok(())
}
fn notify() {
    let _changed = CHANGE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    RELEASED.notify_all();
}
#[cfg(feature = "uno")]
pub(crate) fn waiting() -> bool {
    WAITERS.load(Ordering::Acquire) != 0
}
#[cfg(feature = "uno")]
struct Waiting;
#[cfg(feature = "uno")]
impl Waiting {
    fn new() -> Self {
        WAITERS.fetch_add(1, Ordering::AcqRel);
        notify();
        SCHEDULER_CHANGED.notify();
        Self
    }
}
#[cfg(feature = "uno")]
impl Drop for Waiting {
    fn drop(&mut self) {
        WAITERS.fetch_sub(1, Ordering::AcqRel);
        notify();
        SCHEDULER_CHANGED.notify();
    }
}

#[cfg(feature = "highs")]
pub(crate) fn read(execution: &Execution) -> Result<RwLockReadGuard<'static, ()>, ProblemError> {
    if ACTIVE.with(std::cell::Cell::get) {
        return Err(ProblemError::Internal(
            "nested HiGHS scheduler ownership".into(),
        ));
    }
    crate::execution::pause_compute();
    let _stop_watches = stop_watches(execution);
    loop {
        execution.check()?;
        // Registration and native-guard inspection share the release condition.
        // No release can be missed between a failed try and sleeping.
        let changed = CHANGE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        execution.check()?;
        #[cfg(feature = "uno")]
        let writer_waiting = waiting();
        #[cfg(not(feature = "uno"))]
        let writer_waiting = false;
        let gate = if writer_waiting {
            None
        } else {
            match LIFECYCLE.try_read() {
                Ok(gate) => Some(gate),
                Err(TryLockError::Poisoned(error)) => Some(error.into_inner()),
                Err(TryLockError::WouldBlock) => None,
            }
        };
        if let Some(gate) = gate {
            drop(changed);
            if let Err(error) = crate::execution::resume_compute(execution) {
                drop(gate);
                notify();
                return Err(error);
            }
            ACTIVE.with(|active| active.set(true));
            return Ok(gate);
        }
        wait(changed, execution)?;
    }
}
/// Called after the read owner's last native model is destroyed. Another live owner
/// postpones reset until its destruction; an idle owner never makes teardown block.
#[cfg(feature = "highs")]
pub(crate) fn released() {
    ACTIVE.with(|active| active.set(false));
    if let Ok(_exclusive) = LIFECYCLE.try_write() {
        reset();
    }
    notify();
}
fn reset() {
    // SAFETY: every caller holds the exclusive scheduler gate and all its native
    // HiGHS models have been destroyed before calling reset.
    unsafe { highs_sys::Highs_resetGlobalScheduler(1) };
}

/// Uno requires serial HiGHS. Hold this owner until Uno and every subproblem are gone.
#[cfg(feature = "uno")]
pub(crate) struct Exclusive {
    gate: Option<RwLockWriteGuard<'static, ()>>,
}
#[cfg(feature = "uno")]
impl Drop for Exclusive {
    fn drop(&mut self) {
        reset();
        ACTIVE.with(|active| active.set(false));
        self.gate.take();
        notify();
    }
}
#[cfg(feature = "uno")]
pub(crate) fn exclusive(execution: &Execution) -> Result<Exclusive, ProblemError> {
    if ACTIVE.with(std::cell::Cell::get) {
        return Err(ProblemError::Internal(
            "Uno cannot enter while this worker retains a HiGHS model".into(),
        ));
    }
    let _waiting = Waiting::new();
    crate::execution::pause_compute();
    let _stop_watches = stop_watches(execution);
    loop {
        execution.check()?;
        let changed = CHANGE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        execution.check()?;
        let gate = match LIFECYCLE.try_write() {
            Ok(gate) => Some(gate),
            Err(TryLockError::Poisoned(error)) => Some(error.into_inner()),
            Err(TryLockError::WouldBlock) => None,
        };
        if let Some(gate) = gate {
            drop(changed);
            if let Err(error) = crate::execution::resume_compute(execution) {
                drop(gate);
                notify();
                return Err(error);
            }
            reset();
            ACTIVE.with(|active| active.set(true));
            let owner = Exclusive { gate: Some(gate) };
            return Ok(owner);
        }
        wait(changed, execution)?;
    }
}

#[cfg(all(test, feature = "highs", feature = "uno"))]
mod tests {
    use super::*;
    use crate::{
        execution::{ComputeAdmission, compute_scoped},
        solve::Controls,
    };
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    };
    #[derive(Debug)]
    struct Compute {
        paused: mpsc::Sender<()>,
        resumed: Arc<AtomicUsize>,
    }
    impl ComputeAdmission for Compute {
        fn pause(&mut self) {
            assert!(self.paused.send(()).is_ok(), "pause observer was dropped");
        }
        fn try_resume(
            &mut self,
            execution: &Execution,
        ) -> Result<crate::execution::ComputeAvailability, ProblemError> {
            self.resume(execution)?;
            Ok(crate::execution::ComputeAvailability::Acquired)
        }
        fn resume(&mut self, execution: &Execution) -> Result<(), ProblemError> {
            execution.check()?;
            self.resumed.fetch_add(1, Ordering::Release);
            Ok(())
        }
    }
    #[derive(Debug, Default)]
    struct WakeCount(AtomicUsize);
    impl std::task::Wake for WakeCount {
        fn wake(self: Arc<Self>) { self.0.fetch_add(1, Ordering::AcqRel); }
        fn wake_by_ref(self: &Arc<Self>) { self.0.fetch_add(1, Ordering::AcqRel); }
    }
    #[test]
    fn scheduler_pressure_wakes_registered_idle_owner_without_native_polling() {
        let count = Arc::new(WakeCount::default());
        let waker = std::task::Waker::from(count.clone());
        let registration = watch(&waker);
        let waiting = Waiting::new();
        assert!(self::waiting());
        assert!(count.0.load(Ordering::Acquire) > 0);
        drop(registration);
        let before = count.0.load(Ordering::Acquire);
        let registration = watch(&waker);
        drop(waiting);
        assert!(count.0.load(Ordering::Acquire) > before);
        drop(registration);
    }
    #[test]
    fn scheduler_wait_releases_compute_then_resumes_after_actual_guard() {
        let controls = Controls::default();
        let owner = exclusive(&Execution::new(pse_diagnostics::Cancellation::default(), &controls)).unwrap();
        let (paused, observed) = mpsc::channel();
        let resumed = Arc::new(AtomicUsize::new(0));
        let seen = resumed.clone();
        let thread = std::thread::spawn(move || {
            compute_scoped(
                Box::new(Compute {
                    paused,
                    resumed: seen,
                }),
                || {
                    let gate = read(&Execution::new(pse_diagnostics::Cancellation::default(), &controls)).unwrap();
                    drop(gate);
                    released();
                },
            );
        });
        observed
            .recv_timeout(std::time::Duration::from_secs(2))
            .unwrap();
        assert_eq!(resumed.load(Ordering::Acquire), 0);
        drop(owner);
        thread.join().unwrap();
        assert_eq!(resumed.load(Ordering::Acquire), 1);
    }
    #[test]
    fn scheduler_wait_preserves_cancel_without_claiming_compute() {
        let owner = exclusive(&Execution::new(pse_diagnostics::Cancellation::default(), &Controls::default())).unwrap();
        let (paused, observed) = mpsc::channel();
        let resumed = Arc::new(AtomicUsize::new(0));
        let seen = resumed.clone();
        let cancel = pse_diagnostics::Cancellation::default();
        let flag = cancel.clone();
        let thread = std::thread::spawn(move || {
            compute_scoped(
                Box::new(Compute {
                    paused,
                    resumed: seen,
                }),
                || {
                    let result = read(&Execution::new(flag, &Controls::default()));
                    assert!(result.is_err());
                },
            )
        });
        observed
            .recv_timeout(std::time::Duration::from_secs(2))
            .unwrap();
        cancel.cancel();
        thread.join().unwrap();
        assert_eq!(resumed.load(Ordering::Acquire), 0);
        drop(owner);
    }
}
