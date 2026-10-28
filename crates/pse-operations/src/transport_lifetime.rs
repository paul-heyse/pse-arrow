// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Caller admission, admitted continuation and the actual client pair's teardown.

use crate::canonical::CanonicalError;
use std::pin::Pin;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

tokio::task_local! {
    static CONTINUATION: ConnectionActivity;
}

#[cfg(all(test, feature = "canonical-tests"))]
thread_local! {
    static MINT_PAUSE:std::cell::RefCell<Option<Arc<std::sync::Barrier>>>=const{std::cell::RefCell::new(None)};
}
#[cfg(all(test, feature = "canonical-tests"))]
pub(crate) fn pause_next_caller_mint(barrier: Arc<std::sync::Barrier>) {
    MINT_PAUSE.with(|pause| *pause.borrow_mut() = Some(barrier));
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Open,
    Closing,
    Closed,
}

#[derive(Debug)]
struct State {
    phase: Phase,
    activities: usize,
    ordinary_drained: bool,
    activation_drained: bool,
    running: bool,
    round: Option<Arc<CloseRound>>,
}

#[derive(Debug)]
pub(crate) struct PhysicalLifetime {
    state: Mutex<State>,
    idle: tokio::sync::Notify,
}
impl Default for PhysicalLifetime {
    fn default() -> Self {
        Self {
            state: Mutex::new(State {
                phase: Phase::Open,
                activities: 0,
                ordinary_drained: false,
                activation_drained: false,
                running: false,
                round: None,
            }),
            idle: tokio::sync::Notify::new(),
        }
    }
}

/// Logical admission and continuation over one independently owned physical client.
#[derive(Clone, Debug, Default)]
pub struct ConnectionAccess {
    pub(crate) physical: Arc<PhysicalLifetime>,
    caller: Arc<AtomicBool>,
    continuation: Option<ConnectionActivity>,
}
/// Authority to finish already admitted work, including its required children.
/// Cloning shares that one activity; dropping an idle caller is unrelated.
#[derive(Clone, Debug)]
pub struct ConnectionActivity(Arc<ActivityOwner>);

#[derive(Debug)]
struct ActivityOwner {
    physical: Arc<PhysicalLifetime>,
}
impl Drop for ActivityOwner {
    fn drop(&mut self) {
        let mut state = self
            .physical
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.activities -= 1;
        if state.activities == 0 {
            self.physical.idle.notify_waiters();
        }
    }
}
impl ConnectionActivity {
    /// Carry continuation into a supervisor future without granting idle aliases
    /// perpetual admission. Spawned children must explicitly inherit this owner.
    pub fn scope<F: Future>(&self, future: F) -> impl Future<Output = F::Output> + use<F> {
        CONTINUATION.scope(self.clone(), future)
    }
    /// Admit synchronous children before transferring the activity to a supervisor.
    pub fn sync_scope<T>(&self, action: impl FnOnce() -> T) -> T {
        CONTINUATION.sync_scope(self.clone(), action)
    }
}
/// Capture continuation before spawning a required child. Tokio does not inherit
/// task locals; an unrelated caller without an activity acquires no authority here.
pub fn inherit_activity<F: Future>(future: F) -> impl Future<Output = F::Output> {
    let activity = CONTINUATION.try_with(Clone::clone).ok();
    InheritedActivity {
        activity,
        entered: false,
        future: Some(future),
    }
}

pin_project_lite::pin_project! {
    // Retain one inline child rather than an async argument plus awaited state.
    struct InheritedActivity<F> {
        // Ordinary unpolled destruction also releases the child before its owner.
        #[pin]
        future: Option<F>,
        activity: Option<ConnectionActivity>,
        entered: bool,
    }

    impl<F> PinnedDrop for InheritedActivity<F> {
        fn drop(this: Pin<&mut Self>) {
            let mut this = this.project();
            if *this.entered && let Some(activity) = this.activity.as_ref() {
                activity.sync_scope(|| this.future.set(None));
            }
            // Before polling, the old async wrapper had not entered its scope.
        }
    }
}

impl<F: Future> Future for InheritedActivity<F> {
    type Output = F::Output;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut this = self.project();
        *this.entered = true;
        let mut poll = || {
            let Some(future) = this.future.as_mut().as_pin_mut() else {
                return std::task::Poll::Pending;
            };
            let result = future.poll(cx);
            if result.is_ready() {
                this.future.set(None);
            }
            result
        };
        let result = match this.activity.as_ref() {
            Some(activity) => activity.sync_scope(poll),
            None => poll(),
        };
        if result.is_ready() {
            // A completed TaskLocalFuture released its captured value as well.
            drop(this.activity.take());
        }
        result
    }
}

impl ConnectionAccess {
    /// Drain this independently owned connection with the existing bounded lifetime owner.
    /// The caller supplies only its own transport teardown; no scientific client is exposed.
    pub async fn disconnect<F, O>(&self, ordinary: O) -> Result<(), CanonicalError>
    where
        O: FnOnce() -> F + Send + 'static,
        F: Future<Output = Result<(), CanonicalError>> + Send,
    {
        self.physical
            .close(|| async { Ok(()) }, ordinary, || async { Ok(()) })
            .await
    }
    /// Retain one bounded activity; closed callers cannot admit independent work.
    /// # Errors
    /// Closed caller, unavailable physical connection or activity count overflow.
    pub fn admit(&self) -> Result<ConnectionActivity, CanonicalError> {
        if let Some(activity) = &self.continuation {
            return Ok(activity.clone());
        }
        if let Ok(activity) = CONTINUATION.try_with(Clone::clone)
            && Arc::ptr_eq(&activity.0.physical, &self.physical)
        {
            return Ok(activity);
        }
        let mut state = self
            .physical
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if self.caller.load(Ordering::Acquire) {
            return Err(CanonicalError::CallerClosed);
        }
        if state.phase != Phase::Open {
            return Err(CanonicalError::ConnectionUnavailable);
        }
        state.activities = state
            .activities
            .checked_add(1)
            .ok_or(CanonicalError::PayloadLimit)?;
        Ok(ConnectionActivity(Arc::new(ActivityOwner {
            physical: self.physical.clone(),
        })))
    }
    pub(crate) fn with_activity(&self) -> Result<Self, CanonicalError> {
        Ok(Self {
            continuation: Some(self.admit()?),
            ..self.clone()
        })
    }
    /// Finish construction (including fixture membership) under the same lock
    /// that decides final ownership and seals physical admission. The callback
    /// must only construct aliases; it must not re-enter activity admission.
    pub(crate) fn independent_with<T>(
        &self,
        mint: impl FnOnce(Self) -> T,
    ) -> Result<T, CanonicalError> {
        let state = self
            .physical
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.phase != Phase::Open {
            return Err(CanonicalError::ConnectionUnavailable);
        }
        #[cfg(all(test, feature = "canonical-tests"))]
        MINT_PAUSE.with(|pause| {
            let barrier = pause.borrow_mut().take();
            if let Some(barrier) = barrier {
                barrier.wait();
                barrier.wait();
            }
        });
        Ok(mint(Self {
            physical: self.physical.clone(),
            caller: Arc::default(),
            continuation: None,
        }))
    }
    /// Seal only this logical caller, preserving already admitted work.
    pub fn close_caller(&self) {
        let _state = self
            .physical
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.caller.store(true, Ordering::Release);
    }
}

#[derive(Debug, Default)]
struct CloseRound {
    outcome: Mutex<Option<Result<(), Arc<CanonicalError>>>>,
    changed: tokio::sync::Notify,
}

struct CloseOwner {
    physical: Arc<PhysicalLifetime>,
    round: Arc<CloseRound>,
    completed: bool,
}
impl CloseOwner {
    fn complete(mut self, outcome: Result<(), CanonicalError>) {
        self.physical.complete(&self.round, outcome);
        self.completed = true;
    }
}
impl Drop for CloseOwner {
    fn drop(&mut self) {
        if !self.completed {
            self.physical.complete(
                &self.round,
                Err(CanonicalError::Configuration(
                    "physical connection drain task ended before completion".into(),
                )),
            );
        }
    }
}
impl PhysicalLifetime {
    /// The fixture association is counted while independent minting is excluded.
    /// Ordinary clones preserve the same caller: those racing after this seal
    /// are closed aliases and cannot admit new work or restore open authority.
    #[cfg(any(test, feature = "test-support"))]
    pub(crate) fn seal_if_final_fixture(&self, is_final: impl FnOnce() -> bool) -> bool {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.phase != Phase::Open {
            return true;
        }
        if !is_final() {
            return false;
        }
        state.phase = Phase::Closing;
        true
    }
    #[cfg(all(test, feature = "canonical-tests"))]
    pub(crate) fn caller_mint_holds_admission(&self) -> bool {
        matches!(
            self.state.try_lock(),
            Err(std::sync::TryLockError::WouldBlock)
        )
    }
    fn complete(&self, round: &CloseRound, outcome: Result<(), CanonicalError>) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if outcome.is_ok() {
            state.phase = Phase::Closed;
        }
        state.running = false;
        *round
            .outcome
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(outcome.map_err(Arc::new));
        round.changed.notify_waiters();
    }
    /// Terminal operation ownership has excluded escaped callers and joined
    /// protected releases. Seal new admission before joining actual client terminal
    /// work; admitted continuations retain their original authority while draining.
    pub(crate) async fn seal_and_wait_for_idle(
        &self,
        deadline: tokio::time::Instant,
    ) -> Result<(), CanonicalError> {
        {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if state.phase == Phase::Open {
                state.phase = Phase::Closing;
            }
        }
        tokio::time::timeout_at(deadline, async {
            loop {
                let idle = self.idle.notified();
                tokio::pin!(idle);
                idle.as_mut().enable();
                if self
                    .state
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .activities
                    == 0
                {
                    return;
                }
                idle.await;
            }
        })
        .await
        .map_err(|_| CanonicalError::Timeout)
    }
    /// Refusal changes no admission state. Once Closing is published, an owned
    /// task drains both clients even when the submitting waiter disappears.
    /// Failed teardown keeps its acknowledged progress for an authentic retry.
    pub(crate) async fn close<R, RF, O, OF, A, AF>(
        self: &Arc<Self>,
        readers: R,
        ordinary: O,
        activation: A,
    ) -> Result<(), CanonicalError>
    where
        R: FnOnce() -> RF + Send + 'static,
        RF: Future<Output = Result<(), CanonicalError>> + Send,
        O: FnOnce() -> OF + Send + 'static,
        OF: Future<Output = Result<(), CanonicalError>> + Send,
        A: FnOnce() -> AF + Send + 'static,
        AF: Future<Output = Result<(), CanonicalError>> + Send,
    {
        let (round, launch) = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if state.phase == Phase::Closed {
                return Ok(());
            }
            if state.activities != 0 {
                return Err(CanonicalError::ConnectionBusy {
                    activities: state.activities,
                });
            }
            state.phase = Phase::Closing;
            let launch = !state.running;
            if launch {
                state.round = Some(Arc::new(CloseRound::default()));
                state.running = true;
            }
            (
                state
                    .round
                    .get_or_insert_with(|| Arc::new(CloseRound::default()))
                    .clone(),
                launch,
            )
        };
        if launch {
            let owner = CloseOwner {
                physical: self.clone(),
                round: round.clone(),
                completed: false,
            };
            let deadline = crate::transport::current_deadline().unwrap_or_else(|| {
                crate::transport::original_deadline(crate::transport::REQUEST_TIMEOUT)
            });
            tokio::spawn(async move {
                let result = async {
                    // Readers protect both clients. Failure here forbids either
                    // disconnect, while the clients themselves are independent.
                    crate::transport::within_clock(deadline, readers()).await?;
                    let (ordinary_done, activation_done) = {
                        let state = owner
                            .physical
                            .state
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        (state.ordinary_drained, state.activation_drained)
                    };
                    let ordinary_drain = async {
                        if !ordinary_done {
                            crate::transport::within_clock(deadline, ordinary()).await?;
                            owner
                                .physical
                                .state
                                .lock()
                                .unwrap_or_else(std::sync::PoisonError::into_inner)
                                .ordinary_drained = true;
                        }
                        Ok::<(), CanonicalError>(())
                    };
                    let activation_drain = async {
                        if !activation_done {
                            crate::transport::within_clock(deadline, activation()).await?;
                            owner
                                .physical
                                .state
                                .lock()
                                .unwrap_or_else(std::sync::PoisonError::into_inner)
                                .activation_drained = true;
                        }
                        Ok::<(), CanonicalError>(())
                    };
                    match tokio::join!(ordinary_drain, activation_drain) {
                        (Ok(()), Ok(())) => Ok(()),
                        (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
                        (Err(ordinary), Err(activation)) => Err(CanonicalError::Configuration(
                            format!("ordinary driver: {ordinary}; activation driver: {activation}"),
                        )),
                    }
                }
                .await;
                owner.complete(result);
            });
        }
        loop {
            let changed = round.changed.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            {
                let outcome = round
                    .outcome
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if let Some(outcome) = &*outcome {
                    return outcome
                        .clone()
                        .map_err(|source| CanonicalError::PhysicalClose { source });
                }
            }
            changed.await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    struct ActivityProbe {
        expected: ConnectionAccess,
        observations: Arc<Mutex<Vec<Option<bool>>>>,
        behavior: u8,
    }
    impl Future for ActivityProbe {
        type Output = ();
        fn poll(self: Pin<&mut Self>, _: &mut std::task::Context<'_>) -> std::task::Poll<()> {
            self.observe();
            match self.behavior {
                0 => std::task::Poll::Ready(()),
                1 => std::task::Poll::Pending,
                _ => panic!("controlled child poll panic"),
            }
        }
    }
    impl ActivityProbe {
        fn observe(&self) {
            self.observations.lock().unwrap().push(
                CONTINUATION
                    .try_with(|activity| Arc::ptr_eq(&activity.0.physical, &self.expected.physical))
                    .ok(),
            );
            if CONTINUATION
                .try_with(|activity| Arc::ptr_eq(&activity.0.physical, &self.expected.physical))
                .unwrap_or(false)
            {
                assert!(
                    self.expected.admit().is_ok(),
                    "captured continuation remains real authority"
                );
            }
        }
    }
    impl Drop for ActivityProbe {
        fn drop(&mut self) {
            self.observe();
        }
    }

    #[tokio::test]
    async fn inherited_activity_captures_before_spawn_and_drops_ready_child_in_scope() {
        let caller = ConnectionAccess::default();
        let activity = caller.admit().unwrap();
        let observations = Arc::new(Mutex::new(Vec::new()));
        let child = activity.sync_scope(|| {
            inherit_activity(ActivityProbe {
                expected: caller.clone(),
                observations: observations.clone(),
                behavior: 0,
            })
        });
        caller.close_caller();
        drop(activity);
        assert_eq!(caller.physical.state.lock().unwrap().activities, 1);
        tokio::spawn(child).await.unwrap();
        assert_eq!(*observations.lock().unwrap(), vec![Some(true), Some(true)]);
        assert_eq!(caller.physical.state.lock().unwrap().activities, 0);
        assert!(CONTINUATION.try_with(|_| ()).is_err());
        assert!(matches!(caller.admit(), Err(CanonicalError::CallerClosed)));
    }

    #[test]
    fn inherited_activity_pending_repoll_and_cancel_drop_restore_captured_authority() {
        let caller = ConnectionAccess::default();
        let activity = caller.admit().unwrap();
        let ambient = ConnectionAccess::default().admit().unwrap();
        let observations = Arc::new(Mutex::new(Vec::new()));
        let mut child = Box::pin(activity.sync_scope(|| {
            inherit_activity(ActivityProbe {
                expected: caller.clone(),
                observations: observations.clone(),
                behavior: 1,
            })
        }));
        caller.close_caller();
        drop(activity);
        let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
        assert!(child.as_mut().poll(&mut cx).is_pending());
        ambient.sync_scope(|| {
            assert!(child.as_mut().poll(&mut cx).is_pending());
            assert!(
                CONTINUATION
                    .try_with(|value| Arc::ptr_eq(&value.0, &ambient.0))
                    .unwrap()
            );
            drop(child);
            assert!(
                CONTINUATION
                    .try_with(|value| Arc::ptr_eq(&value.0, &ambient.0))
                    .unwrap()
            );
        });
        assert_eq!(*observations.lock().unwrap(), vec![Some(true); 3]);
        assert_eq!(caller.physical.state.lock().unwrap().activities, 0);
        assert!(CONTINUATION.try_with(|_| ()).is_err());
    }

    #[test]
    fn inherited_activity_panic_drop_is_scoped_but_unpolled_and_uncaptured_are_ambient() {
        let caller = ConnectionAccess::default();
        let activity = caller.admit().unwrap();
        let ambient = ConnectionAccess::default().admit().unwrap();
        for (behavior, polled, captured, expected) in [
            (2, true, true, vec![Some(true), Some(true)]),
            (1, false, true, vec![Some(false)]),
            (0, true, false, vec![Some(false), Some(false)]),
        ] {
            let observations = Arc::new(Mutex::new(Vec::new()));
            let probe = ActivityProbe {
                expected: caller.clone(),
                observations: observations.clone(),
                behavior,
            };
            let mut child = Box::pin(if captured {
                activity.sync_scope(|| inherit_activity(probe))
            } else {
                inherit_activity(probe)
            });
            ambient.sync_scope(|| {
                if polled {
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
                        child.as_mut().poll(&mut cx)
                    }));
                    assert_eq!(result.is_err(), behavior == 2);
                }
                assert!(
                    CONTINUATION
                        .try_with(|value| Arc::ptr_eq(&value.0, &ambient.0))
                        .unwrap()
                );
                drop(child);
            });
            assert_eq!(*observations.lock().unwrap(), expected);
        }
        assert!(CONTINUATION.try_with(|_| ()).is_err());
    }

    #[test]
    fn inherited_activity_ready_releases_authority_before_completed_wrapper_is_dropped() {
        let caller = ConnectionAccess::default();
        let activity = caller.admit().unwrap();
        let observations = Arc::new(Mutex::new(Vec::new()));
        let mut child = Box::pin(activity.sync_scope(|| {
            inherit_activity(ActivityProbe {
                expected: caller.clone(),
                observations: observations.clone(),
                behavior: 0,
            })
        }));
        caller.close_caller();
        drop(activity);
        let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
        assert!(child.as_mut().poll(&mut cx).is_ready());
        assert_eq!(*observations.lock().unwrap(), vec![Some(true), Some(true)]);
        assert_eq!(caller.physical.state.lock().unwrap().activities, 0);
        assert!(matches!(caller.admit(), Err(CanonicalError::CallerClosed)));
        assert!(CONTINUATION.try_with(|_| ()).is_err());
        drop(child);
        assert_eq!(
            observations.lock().unwrap().len(),
            2,
            "completed child is destroyed exactly once"
        );
    }

    #[test]
    fn inherited_activity_retains_one_inline_child_without_async_state_duplication() {
        let payload = [0_u8; 64 * 1024];
        let child = async move {
            std::future::pending::<()>().await;
            std::hint::black_box(payload)
        };
        let bytes = size_of_val(&child);
        assert!(size_of_val(&inherit_activity(child)) <= bytes + 4 * size_of::<usize>());
    }

    #[tokio::test]
    async fn caller_aliases_close_together_but_continuation_and_independent_caller_survive() {
        let caller = ConnectionAccess::default();
        let escaped = caller.clone();
        let other = caller.independent_with(|access| access).unwrap();
        let activity = caller.admit().unwrap();
        caller.close_caller();
        assert!(matches!(escaped.admit(), Err(CanonicalError::CallerClosed)));
        assert!(other.admit().is_ok());
        activity
            .scope(async {
                assert!(escaped.admit().is_ok());
                let bound = escaped.with_activity().unwrap();
                tokio::spawn(async move {
                    assert!(bound.admit().is_ok());
                })
                .await
                .unwrap();
            })
            .await;
        drop(activity);
        assert_eq!(caller.physical.state.lock().unwrap().activities, 0);
    }

    #[tokio::test]
    async fn refused_physical_close_preserves_open_admission_and_both_drivers() {
        let caller = ConnectionAccess::default();
        let activity = caller.admit().unwrap();
        let called = Arc::new(AtomicUsize::new(0));
        let driver = called.clone();
        let result = caller
            .physical
            .close(
                || async { Ok(()) },
                move || async move {
                    driver.fetch_add(1, Ordering::AcqRel);
                    Ok(())
                },
                || async { Ok(()) },
            )
            .await;
        assert!(matches!(
            result,
            Err(CanonicalError::ConnectionBusy { activities: 1 })
        ));
        assert_eq!(called.load(Ordering::Acquire), 0);
        assert!(caller.admit().is_ok());
        assert_eq!(caller.physical.state.lock().unwrap().phase, Phase::Open);
        drop(activity);
        caller
            .physical
            .close(
                || async { Ok(()) },
                || async { Ok(()) },
                || async { Ok(()) },
            )
            .await
            .unwrap();
        assert!(matches!(
            caller.admit(),
            Err(CanonicalError::ConnectionUnavailable)
        ));
    }

    #[tokio::test]
    async fn cancelled_close_waiter_does_not_cancel_two_driver_drain() {
        let caller = ConnectionAccess::default();
        let physical = caller.physical.clone();
        let (ordinary_done, done) = tokio::sync::oneshot::channel();
        let (resume, paused) = tokio::sync::oneshot::channel();
        let waiter = tokio::spawn(async move {
            physical
                .close(
                    || async { Ok(()) },
                    move || async move {
                        ordinary_done.send(()).unwrap();
                        Ok(())
                    },
                    move || async move {
                        paused.await.unwrap();
                        Ok(())
                    },
                )
                .await
        });
        done.await.unwrap();
        waiter.abort();
        let _ = waiter.await;
        assert_eq!(caller.physical.state.lock().unwrap().phase, Phase::Closing);
        assert!(matches!(
            caller.admit(),
            Err(CanonicalError::ConnectionUnavailable)
        ));
        resume.send(()).unwrap();
        caller
            .physical
            .close(
                || async { panic!("existing close owns readers") },
                || async { panic!("ordinary already drained") },
                || async { panic!("existing close owns activation") },
            )
            .await
            .unwrap();
        assert_eq!(caller.physical.state.lock().unwrap().phase, Phase::Closed);
    }

    #[tokio::test]
    async fn failed_second_driver_keeps_closing_and_retry_skips_acknowledged_first_driver() {
        let caller = ConnectionAccess::default();
        let result = caller
            .physical
            .close(
                || async { Ok(()) },
                || async { Ok(()) },
                || async {
                    Err(CanonicalError::Configuration(
                        "activation drain failed".into(),
                    ))
                },
            )
            .await;
        assert!(matches!(result, Err(CanonicalError::PhysicalClose { .. })));
        assert_eq!(caller.physical.state.lock().unwrap().phase, Phase::Closing);
        assert!(matches!(
            caller.admit(),
            Err(CanonicalError::ConnectionUnavailable)
        ));
        caller
            .physical
            .close(
                || async { Ok(()) },
                || async { panic!("ordinary already drained") },
                || async { Ok(()) },
            )
            .await
            .unwrap();
        assert_eq!(caller.physical.state.lock().unwrap().phase, Phase::Closed);
    }

    #[tokio::test]
    async fn last_activity_release_and_new_admission_serialize_with_close() {
        for _ in 0..32 {
            let caller = ConnectionAccess::default();
            let activity = caller.admit().unwrap();
            let contender = caller.clone();
            let physical = caller.physical.clone();
            let barrier = Arc::new(tokio::sync::Barrier::new(3));
            let start = barrier.clone();
            let admission = tokio::spawn(async move {
                start.wait().await;
                contender.admit()
            });
            let start = barrier.clone();
            let closing = tokio::spawn(async move {
                start.wait().await;
                physical
                    .close(
                        || async { Ok(()) },
                        || async { Ok(()) },
                        || async { Ok(()) },
                    )
                    .await
            });
            drop(activity);
            barrier.wait().await;
            let admitted = admission.await.unwrap();
            let closed = closing.await.unwrap();
            match admitted {
                Ok(activity) => {
                    assert!(matches!(closed, Err(CanonicalError::ConnectionBusy { .. })));
                    drop(activity);
                }
                Err(CanonicalError::ConnectionUnavailable) => assert!(closed.is_ok()),
                other => panic!("unexpected admission: {other:?}"),
            }
        }
    }
    #[tokio::test]
    async fn original_close_waiter_observes_its_failure_after_a_successful_retry() {
        use std::{future::Future, task::Poll};
        let caller = ConnectionAccess::default();
        let first = caller.physical.close(
            || async { Ok(()) },
            || async { Ok(()) },
            || async {
                Err(CanonicalError::Configuration(
                    "first activation failure".into(),
                ))
            },
        );
        tokio::pin!(first);
        std::future::poll_fn(|cx| {
            assert!(first.as_mut().poll(cx).is_pending());
            Poll::Ready(())
        })
        .await;
        while caller.physical.state.lock().unwrap().running {
            tokio::task::yield_now().await;
        }
        caller
            .physical
            .close(
                || async { Ok(()) },
                || async { panic!("ordinary already drained") },
                || async { Ok(()) },
            )
            .await
            .unwrap();
        assert!(matches!(
            first.await,
            Err(CanonicalError::PhysicalClose { .. })
        ));
    }
    #[tokio::test]
    async fn failed_first_driver_still_settles_second_and_retains_each_error() {
        let caller = ConnectionAccess::default();
        let error = caller
            .physical
            .close(
                || async { Ok(()) },
                || async { Err(CanonicalError::Configuration("ordinary refusal".into())) },
                || async { Err(CanonicalError::Configuration("activation refusal".into())) },
            )
            .await
            .unwrap_err();
        let detail = error.to_string();
        assert!(detail.contains("ordinary refusal"));
        assert!(detail.contains("activation refusal"));
        assert_eq!(caller.physical.state.lock().unwrap().phase, Phase::Closing);
    }

    #[tokio::test]
    async fn stalled_first_driver_cannot_block_second_and_child_keeps_original_deadline() {
        let caller = ConnectionAccess::default();
        let completed = Arc::new(AtomicBool::new(false));
        let second = completed.clone();
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_millis(40);
        let result = crate::transport::operation_scope(
            deadline.into_std(),
            caller.physical.close(
                || async { Ok(()) },
                || async { std::future::pending::<Result<(), CanonicalError>>().await },
                move || async move {
                    second.store(true, Ordering::Release);
                    Ok(())
                },
            ),
        )
        .await;
        assert!(result.is_err());
        assert!(completed.load(Ordering::Acquire));
        let state = caller.physical.state.lock().unwrap();
        assert!(state.activation_drained);
        assert!(!state.ordinary_drained);
        assert_eq!(state.phase, Phase::Closing);
        assert!(!state.running);
    }

    #[tokio::test]
    async fn failed_reader_prerequisite_preserves_both_clients() {
        let caller = ConnectionAccess::default();
        let result = caller
            .physical
            .close(
                || async {
                    Err(CanonicalError::Configuration(
                        "escaped protected reader".into(),
                    ))
                },
                || async { panic!("ordinary disconnect before reader settlement") },
                || async { panic!("activation disconnect before reader settlement") },
            )
            .await;
        assert!(result.is_err());
        let state = caller.physical.state.lock().unwrap();
        assert!(!state.ordinary_drained && !state.activation_drained);
        assert_eq!(state.phase, Phase::Closing);
    }

    #[tokio::test]
    async fn physical_child_preserves_exact_longer_enclosing_settlement_offer() {
        let caller = ConnectionAccess::default();
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(60);
        crate::transport::operation_scope(
            deadline.into_std(),
            caller.physical.close(
                move || async move {
                    assert_eq!(crate::transport::current_deadline(), Some(deadline));
                    Ok(())
                },
                move || async move {
                    assert_eq!(crate::transport::current_deadline(), Some(deadline));
                    Ok(())
                },
                move || async move {
                    assert_eq!(crate::transport::current_deadline(), Some(deadline));
                    Ok(())
                },
            ),
        )
        .await
        .unwrap();
    }
}
