// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Dedicated native thread completion, including TLS and foreign destruction.
use crate::ExecutionContext;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
pub(crate) struct NativeThread;
pub(crate) struct NativeJoin<T>(tokio::task::JoinHandle<std::thread::Result<T>>);
/// The queued joiner owns the OS thread even before Tokio starts polling it.
struct NativeCustody<T, F: FnOnce()> {
    thread: Option<std::thread::JoinHandle<T>>,
    on_joined: Option<F>,
}
impl<T, F: FnOnce()> NativeCustody<T, F> {
    fn join(mut self) -> std::thread::Result<T> {
        let outcome: std::thread::Result<T> = match self.thread.take() {
            Some(thread) => thread.join(),
            None => Err(Box::new("native join owner absent")),
        };
        if let Some(joined) = self.on_joined.take() { joined(); }
        outcome
    }
}
impl<T, F: FnOnce()> Drop for NativeCustody<T, F> {
    fn drop(&mut self) {
        if let Some(thread) = self.thread.take() { let _ = thread.join(); }
        if let Some(joined) = self.on_joined.take() {
            // A callback cannot turn cancellation or an existing panic into a double panic.
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(joined));
        }
    }
}
impl NativeThread {
    pub(crate) fn spawn<T: Send + 'static>(
        name: &str,
        stack: usize,
        context: ExecutionContext,
        on_dispatched: impl FnOnce(),
        on_joined: impl FnOnce() + Send + 'static,
        work: impl FnOnce() -> T + Send + 'static,
    ) -> std::io::Result<NativeJoin<T>> {
        let thread = std::thread::Builder::new()
            .name(name.into())
            .stack_size(stack)
            .spawn(move || {
                let _context = context.enter_native();
                work()
            })?;
        let custody = NativeCustody { thread: Some(thread), on_joined: Some(on_joined) };
        if let Err(panic) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(on_dispatched)) {
            // Even an unexpected observer panic cannot abandon a successfully spawned thread.
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| custody.join()));
            std::panic::resume_unwind(panic);
        }
        Ok(NativeJoin(tokio::task::spawn_blocking(move || custody.join())))
    }
}
impl<T> Future for NativeJoin<T> {
    type Output = Result<std::thread::Result<T>, tokio::task::JoinError>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.0).poll(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        cell::RefCell,
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
    };
    struct Destruction(Arc<AtomicBool>);
    impl Drop for Destruction {
        fn drop(&mut self) {
            self.0.store(true, Ordering::Release);
        }
    }
    thread_local! { static FOREIGN: RefCell<Option<Destruction>> = const { RefCell::new(None) }; }
    #[tokio::test]
    async fn join_callback_observes_tls_destruction_in_success_and_panic() {
        for panic in [false, true] {
            let destroyed = Arc::new(AtomicBool::new(false));
            let observed = Arc::new(AtomicBool::new(false));
            let thread_destroyed = destroyed.clone();
            let joined_destroyed = destroyed.clone();
            let joined_observed = observed.clone();
            let joined = NativeThread::spawn(
                "pse-native-join-control",
                2 << 20,
                ExecutionContext::capture(),
                || {},
                move || {
                    assert!(joined_destroyed.load(Ordering::Acquire));
                    joined_observed.store(true, Ordering::Release);
                },
                move || {
                    assert_eq!(
                        std::thread::current().name(),
                        Some("pse-native-join-control")
                    );
                    let _ =
                        FOREIGN.with(|state| state.replace(Some(Destruction(thread_destroyed))));
                    assert!(!panic, "native test panic");
                    17
                },
            )
            .unwrap()
            .await
            .unwrap();
            assert_eq!(joined.is_err(), panic);
            assert!(destroyed.load(Ordering::Acquire));
            assert!(observed.load(Ordering::Acquire));
        }
    }
    #[tokio::test]
    async fn dispatch_callback_failure_joins_native_tls_before_original_panic_escapes() {
        let destroyed = Arc::new(AtomicBool::new(false));
        let joined = Arc::new(AtomicBool::new(false));
        let thread_destroyed = destroyed.clone();
        let joined_destroyed = destroyed.clone();
        let observed = joined.clone();
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            NativeThread::spawn(
                "pse-dispatch-callback-failure", 2 << 20, ExecutionContext::capture(),
                || panic!("original dispatch callback failure"),
                move || {
                    assert!(joined_destroyed.load(Ordering::Acquire));
                    observed.store(true, Ordering::Release);
                },
                move || { FOREIGN.set(Some(Destruction(thread_destroyed))); },
            )
        }));
        let original = match outcome { Err(panic) => panic, Ok(_) => panic!("dispatch callback must fail") };
        assert_eq!(original.downcast_ref::<&str>(), Some(&"original dispatch callback failure"));
        assert!(destroyed.load(Ordering::Acquire));
        assert!(joined.load(Ordering::Acquire));
    }

    struct QueuedDestruction {
        entered: std::sync::mpsc::Sender<()>,
        release: std::sync::mpsc::Receiver<()>,
        destroyed: Arc<AtomicBool>,
    }
    impl Drop for QueuedDestruction {
        fn drop(&mut self) {
            let _ = self.entered.send(());
            let _ = self.release.recv();
            self.destroyed.store(true, Ordering::Release);
        }
    }
    thread_local! { static QUEUED: RefCell<Option<QueuedDestruction>> = const { RefCell::new(None) }; }

    #[test]
    fn cancelled_queued_joiner_keeps_native_tls_owned_through_runtime_shutdown() {
        struct Release(Option<std::sync::mpsc::Sender<()>>);
        impl Drop for Release {
            fn drop(&mut self) { if let Some(sender) = self.0.take() { let _ = sender.send(()); } }
        }
        let (blocker_entered, blocker_ready) = std::sync::mpsc::channel();
        let (release_blocker, blocking) = std::sync::mpsc::channel();
        let mut release_blocker = Release(Some(release_blocker));
        let (entered, destroying) = std::sync::mpsc::channel();
        let (release, gate) = std::sync::mpsc::channel();
        let mut release = Release(Some(release));
        let (stopping, shutdown) = std::sync::mpsc::channel();
        let (joined, completion) = std::sync::mpsc::channel();
        let destroyed = Arc::new(AtomicBool::new(false));
        let native_destroyed = destroyed.clone();
        let observed_destroyed = destroyed.clone();
        let owner = std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all().max_blocking_threads(1).build().unwrap();
            runtime.spawn_blocking(move || {
                blocker_entered.send(()).unwrap();
                let _ = blocking.recv();
            });
            blocker_ready.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
            let entry = runtime.enter();
            let queued = NativeThread::spawn(
                "pse-queued-join-control", 2 << 20, ExecutionContext::capture(), || {},
                move || { assert!(observed_destroyed.load(Ordering::Acquire)); let _ = joined.send(()); },
                move || { QUEUED.set(Some(QueuedDestruction { entered, release: gate, destroyed: native_destroyed })); },
            ).unwrap();
            // Only this private fixture aborts a join task, before its blocking closure starts.
            queued.0.abort();
            drop(queued);
            drop(entry);
            stopping.send(()).unwrap();
            runtime.shutdown_background();
        });
        destroying.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
        shutdown.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
        assert!(!destroyed.load(Ordering::Acquire));
        assert!(matches!(completion.try_recv(), Err(std::sync::mpsc::TryRecvError::Empty)));
        release_blocker.0.take().unwrap().send(()).unwrap();
        assert!(!destroyed.load(Ordering::Acquire));
        release.0.take().unwrap().send(()).unwrap();
        completion.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
        owner.join().unwrap();
        assert!(destroyed.load(Ordering::Acquire));
    }
}
