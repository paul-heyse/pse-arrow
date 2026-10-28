// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Dedicated native thread completion, including TLS and foreign destruction.
use crate::ExecutionContext;
use std::{future::Future, pin::Pin, task::{Context, Poll}};
pub(crate) struct NativeThread;
pub(crate) struct NativeJoin<T>(tokio::task::JoinHandle<std::thread::Result<T>>);
impl NativeThread {
    pub(crate) fn spawn<T: Send + 'static>(
        name: &str,
        stack: usize,
        context: ExecutionContext,
        on_dispatched: impl FnOnce(),
        on_joined: impl FnOnce() + Send + 'static,
        work: impl FnOnce() -> T + Send + 'static,
    ) -> std::io::Result<NativeJoin<T>> {
        let thread = std::thread::Builder::new().name(name.into()).stack_size(stack).spawn(move || {
            let _context = context.enter_native();
            work()
        })?;
        if let Err(panic) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(on_dispatched)) {
            // Even an unexpected observer panic cannot abandon a successfully spawned thread.
            let _ = thread.join();
            on_joined();
            std::panic::resume_unwind(panic);
        }
        Ok(NativeJoin(tokio::task::spawn_blocking(move || {
            let outcome = thread.join();
            on_joined();
            outcome
        })))
    }
}
impl<T> Future for NativeJoin<T> {
    type Output = Result<std::thread::Result<T>, tokio::task::JoinError>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> { Pin::new(&mut self.0).poll(cx) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, sync::{Arc, atomic::{AtomicBool, Ordering}}};
    struct Destruction(Arc<AtomicBool>);
    impl Drop for Destruction { fn drop(&mut self) { self.0.store(true, Ordering::Release); } }
    thread_local! { static FOREIGN: RefCell<Option<Destruction>> = const { RefCell::new(None) }; }
    #[tokio::test]
    async fn join_callback_observes_tls_destruction_in_success_and_panic() {
        for panic in [false, true] {
            let destroyed = Arc::new(AtomicBool::new(false));
            let observed = Arc::new(AtomicBool::new(false));
            let thread_destroyed = destroyed.clone();
            let joined_destroyed = destroyed.clone();
            let joined_observed = observed.clone();
            let joined = NativeThread::spawn("pse-native-join-control", 2 << 20, ExecutionContext::capture(), || {}, move || {
                assert!(joined_destroyed.load(Ordering::Acquire));
                joined_observed.store(true, Ordering::Release);
            }, move || {
                assert_eq!(std::thread::current().name(), Some("pse-native-join-control"));
                let _ = FOREIGN.with(|state| state.replace(Some(Destruction(thread_destroyed))));
                assert!(!panic, "native test panic");
                17
            }).unwrap().await.unwrap();
            assert_eq!(joined.is_err(), panic);
            assert!(destroyed.load(Ordering::Acquire));
            assert!(observed.load(Ordering::Acquire));
        }
    }
}
