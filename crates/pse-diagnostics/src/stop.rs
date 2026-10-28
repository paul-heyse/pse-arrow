// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Executor-neutral stop races. Completion custody remains with the performing owner.
use std::{future::{Future, poll_fn}, pin::pin, task::Poll};

/// A stop request does not establish completion or authorize resource release.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopReason {
    /// Cooperative caller cancellation.
    Cancelled,
    /// The original absolute clock expired.
    Deadline,
}

/// Readable synchronously in native callbacks and asynchronously without polling timers.
pub trait StopSignal {
    /// Inspect the owner's existing state without entering an executor.
    fn check(&self) -> Option<StopReason>;
    /// Register before checking state; the owner supplies any deadline wake.
    fn stopped(&self) -> impl Future<Output = StopReason> + Send;
}
impl StopSignal for crate::Cancellation {
    fn check(&self) -> Option<StopReason> {
        self.is_cancelled().then_some(StopReason::Cancelled)
    }
    async fn stopped(&self) -> StopReason {
        self.cancelled().await;
        StopReason::Cancelled
    }
}

/// Race abandon-safe work against stop. A simultaneous stop wins.
pub async fn stop_first<T>(
    stop: impl Future<Output = StopReason>,
    work: impl Future<Output = T>,
) -> Result<T, StopReason> {
    let mut stop = pin!(stop);
    let mut work = pin!(work);
    poll_fn(|cx| {
        if let Poll::Ready(reason) = stop.as_mut().poll(cx) {
            return Poll::Ready(Err(reason));
        }
        work.as_mut().poll(cx).map(Ok)
    }).await
}

pin_project_lite::pin_project! {
    // Field order keeps the guard alive through the retained future's destruction,
    // including unwinding from its poll. The original future is pinned only once.
    struct DrainOwner<W, G> {
        #[pin]
        work: W,
        guard: Option<G>,
    }
}

/// Retain the original work after stop and keep its owner's drain guard through completion.
/// The caller must itself retain this future in a completion-owned supervisor when abandonable.
pub async fn drain_on_stop<T, G>(
    stop: impl Future<Output = StopReason>,
    work: impl Future<Output = T>,
    on_stop: impl FnOnce(StopReason) -> G,
) -> (T, Option<StopReason>) {
    let mut owner = pin!(DrainOwner { work, guard: None });
    match stop_first(stop, poll_fn(|cx| owner.as_mut().project().work.poll(cx))).await {
        Ok(value) => (value, None),
        Err(reason) => {
            *owner.as_mut().project().guard = Some(on_stop(reason));
            let value = poll_fn(|cx| owner.as_mut().project().work.poll(cx)).await;
            (value, Some(reason))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::{Arc, atomic::{AtomicBool, AtomicUsize, Ordering}}, task::{Context, Wake, Waker}};
    struct Noop;
    impl Wake for Noop { fn wake(self: Arc<Self>) {} }
    fn context() -> Waker { Waker::from(Arc::new(Noop)) }
    #[test]
    fn simultaneous_stop_wins_without_polling_abandon_safe_work() {
        let polls = AtomicUsize::new(0);
        let work = poll_fn(|_| { polls.fetch_add(1, Ordering::Relaxed); Poll::Ready(7) });
        let mut race = pin!(stop_first(std::future::ready(StopReason::Cancelled), work));
        assert_eq!(race.as_mut().poll(&mut Context::from_waker(&context())), Poll::Ready(Err(StopReason::Cancelled)));
        assert_eq!(polls.load(Ordering::Relaxed), 0);
    }
    #[test]
    fn stopped_work_keeps_original_future_and_guard_until_completion() {
        struct Guard(Arc<AtomicBool>);
        impl Drop for Guard { fn drop(&mut self) { self.0.store(false, Ordering::Release); } }
        let guarded = Arc::new(AtomicBool::new(false));
        let polls = AtomicUsize::new(0);
        let work = poll_fn(|_| {
            assert!(guarded.load(Ordering::Acquire));
            if polls.fetch_add(1, Ordering::Relaxed) == 0 { Poll::Pending } else { Poll::Ready(13) }
        });
        let mut drain = pin!(drain_on_stop(std::future::ready(StopReason::Deadline), work, |_| {
            guarded.store(true, Ordering::Release);
            Guard(guarded.clone())
        }));
        let waker = context();
        let mut cx = Context::from_waker(&waker);
        assert!(drain.as_mut().poll(&mut cx).is_pending());
        assert!(guarded.load(Ordering::Acquire));
        assert_eq!(drain.as_mut().poll(&mut cx), Poll::Ready((13, Some(StopReason::Deadline))));
        assert!(!guarded.load(Ordering::Acquire));
        assert_eq!(polls.load(Ordering::Relaxed), 2);
    }
    #[test]
    fn normal_completion_never_constructs_drain_guard() {
        let mut drain = pin!(drain_on_stop(std::future::pending(), std::future::ready(2), |_| panic!("unexpected drain")));
        assert_eq!(drain.as_mut().poll(&mut Context::from_waker(&context())), Poll::Ready((2, None)));
    }
    #[test]
    fn flat_cancellation_checks_and_wakes_without_runtime() {
        let cancellation = crate::Cancellation::default();
        assert_eq!(StopSignal::check(&cancellation), None);
        let mut stopped = pin!(cancellation.stopped());
        let waker = context();
        let mut cx = Context::from_waker(&waker);
        assert!(stopped.as_mut().poll(&mut cx).is_pending());
        cancellation.cancel();
        assert_eq!(StopSignal::check(&cancellation), Some(StopReason::Cancelled));
        assert_eq!(stopped.as_mut().poll(&mut cx), Poll::Ready(StopReason::Cancelled));
    }
    #[test]
    fn drain_guard_outlives_retained_future_destruction_when_poll_panics() {
        struct Guard(Arc<AtomicBool>);
        impl Drop for Guard { fn drop(&mut self) { self.0.store(false, Ordering::Release); } }
        struct Work(Arc<AtomicBool>);
        impl Future for Work {
            type Output = ();
            fn poll(self: std::pin::Pin<&mut Self>, _: &mut Context<'_>) -> Poll<()> { panic!("work poll failed"); }
        }
        impl Drop for Work { fn drop(&mut self) { assert!(self.0.load(Ordering::Acquire)); } }
        let live = Arc::new(AtomicBool::new(false));
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut drain = pin!(drain_on_stop(std::future::ready(StopReason::Cancelled), Work(live.clone()), |_| {
                live.store(true, Ordering::Release); Guard(live.clone())
            }));
            let _ = drain.as_mut().poll(&mut Context::from_waker(&context()));
        }));
        assert!(result.is_err());
        assert!(!live.load(Ordering::Acquire));
    }

}
