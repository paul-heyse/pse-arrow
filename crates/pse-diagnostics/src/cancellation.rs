// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One cooperative stop state shared by numerical callbacks and idle waiters.
use event_listener::{Event, EventListener};
#[cfg(not(target_family = "wasm"))]
use event_listener::Listener;
#[cfg(not(target_family = "wasm"))]
use std::time::Instant;
use std::{
    future::Future,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Waker},
};

#[derive(Debug, Default)]
struct State {
    flag: Arc<AtomicBool>,
    changed: Event,
}

/// Completion-owned cooperative cancellation. Requesting stop never releases work.
#[derive(Clone, Debug, Default)]
pub struct Cancellation(Arc<State>);

impl std::ops::Deref for Cancellation {
    type Target = Arc<AtomicBool>;
    /// Borrow the same read flag for existing synchronous scientific contracts.
    /// Requests must use `cancel`, which also wakes blocked owners.
    fn deref(&self) -> &Self::Target {
        &self.0.flag
    }
}

impl Cancellation {
    /// Construct a direct execution with an explicit initial stop state.
    pub fn new(cancelled: bool) -> Self {
        let control = Self::default();
        if cancelled {
            control.cancel();
        }
        control
    }

    /// Read-only flag for synchronous scientific and foreign-library callbacks.
    /// Writers must call `cancel` so blocked owners also observe the request.
    pub fn flag(&self) -> Arc<AtomicBool> {
        self.0.flag.clone()
    }

    /// Observe the single stop state without allocating or entering a runtime.
    pub fn is_cancelled(&self) -> bool {
        self.0.flag.load(Ordering::Acquire)
    }

    /// Publish stop before waking every currently registered waiter.
    pub fn cancel(&self) {
        if !self.0.flag.swap(true, Ordering::AcqRel) {
            self.0.changed.notify(usize::MAX);
        }
    }

    /// Await stop with registration before the state check; no timer or runtime task.
    pub async fn cancelled(&self) {
        let listener = self.0.changed.listen();
        if !self.is_cancelled() {
            listener.await;
        }
    }

    /// Block until cancellation or an original absolute deadline.
    /// Returns true only when cancellation was observed; a timeout does not set stop.
    /// Blocking waits are unavailable in WASI policy modules; async notification remains usable.
    #[cfg(not(target_family = "wasm"))]
    pub fn wait_until(&self, deadline: Option<Instant>) -> bool {
        let listener = self.0.changed.listen();
        if !self.is_cancelled() {
            match deadline {
                Some(deadline) => {
                    listener.wait_deadline(deadline);
                }
                None => listener.wait(),
            }
        }
        self.is_cancelled()
    }

    /// Register a signal-only native-safe waker for the lifetime of the returned watch.
    /// A racing wake can finish after watch drop, so the waker must own its signal target.
    /// It must not access borrowed session pointers or perform numerical work.
    pub fn watch(&self, waker: &Waker) -> CancellationWatch {
        let mut listener = self.0.changed.listen();
        let ready = Pin::new(&mut listener)
            .poll(&mut Context::from_waker(waker))
            .is_ready();
        if ready || self.is_cancelled() {
            waker.wake_by_ref();
            CancellationWatch { _listener: None }
        } else {
            CancellationWatch {
                _listener: Some(listener),
            }
        }
    }
}

/// Registration retains notification only; it has no execution or cleanup permission.
#[derive(Debug)]
pub struct CancellationWatch {
    _listener: Option<EventListener>,
}

/// One-shot release/change notifications; the performing owner retains its predicate.
/// Register before checking that predicate and rearm after each observed wake.
#[derive(Debug, Default)]
pub struct WakeSignal(Event);
impl WakeSignal {
    /// A process-owned signal can be static without another worker or runtime.
    pub const fn new() -> Self {
        Self(Event::new())
    }
    /// Signal a change after publishing it at its actual owner.
    pub fn notify(&self) {
        self.0.notify(usize::MAX);
    }
    /// Register at construction, before inspecting the actual owner predicate.
    /// A notification is only a hint to inspect that predicate again.
    pub fn notified(&self) -> impl Future<Output = ()> + Send + 'static {
        self.0.listen()
    }
    /// Register a signal-only waker before inspecting the actual owner predicate.
    pub fn watch(&self, waker: &Waker) -> WakeWatch {
        let mut listener = self.0.listen();
        if Pin::new(&mut listener)
            .poll(&mut Context::from_waker(waker))
            .is_ready()
        {
            waker.wake_by_ref();
            WakeWatch { _listener: None }
        } else {
            WakeWatch {
                _listener: Some(listener),
            }
        }
    }
}
/// One waiting interval, with no capacity, execution or cleanup authority.
#[derive(Debug)]
pub struct WakeWatch {
    _listener: Option<EventListener>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::atomic::AtomicUsize, task::Wake, time::Duration};

    #[derive(Default)]
    struct Count(AtomicUsize);
    impl Wake for Count {
        fn wake(self: Arc<Self>) {
            self.0.fetch_add(1, Ordering::AcqRel);
        }
        fn wake_by_ref(self: &Arc<Self>) {
            self.0.fetch_add(1, Ordering::AcqRel);
        }
    }
    #[test]
    fn cancellation_before_and_after_registration_wakes_without_a_runtime() {
        for before in [false, true] {
            let stop = Cancellation::new(before);
            let count = Arc::new(Count::default());
            let waker = Waker::from(count.clone());
            let _watch = stop.watch(&waker);
            stop.cancel();
            assert!(stop.flag().load(Ordering::Acquire));
            assert!(count.0.load(Ordering::Acquire) >= 1);
        }
    }
    #[test]
    fn cancellation_clone_and_dropped_watch_preserve_single_state() {
        let stop = Cancellation::default();
        let clone = stop.clone();
        let count = Arc::new(Count::default());
        let waker = Waker::from(count.clone());
        drop(stop.watch(&waker));
        clone.cancel();
        assert!(stop.is_cancelled());
        assert!(Arc::ptr_eq(&stop.flag(), &clone.flag()));
        assert_eq!(count.0.load(Ordering::Acquire), 0);
    }
    #[test]
    fn cancellation_deadline_does_not_request_stop() {
        let stop = Cancellation::default();
        assert!(!stop.wait_until(Some(Instant::now())));
        assert!(!stop.is_cancelled());
        let clone = stop.clone();
        let worker = std::thread::spawn(move || {
            clone.cancel();
        });
        assert!(stop.wait_until(Some(Instant::now() + Duration::from_secs(5))));
        worker.join().unwrap();
    }
    #[test]
    fn owner_signal_registers_before_predicate_and_rearms_after_notification() {
        let signal = WakeSignal::new();
        let count = Arc::new(Count::default());
        let waker = Waker::from(count.clone());
        let watch = signal.watch(&waker);
        signal.notify();
        assert_eq!(count.0.load(Ordering::Acquire), 1);
        drop(watch);
        let _rearmed = signal.watch(&waker);
        signal.notify();
        assert_eq!(count.0.load(Ordering::Acquire), 2);
    }
}
