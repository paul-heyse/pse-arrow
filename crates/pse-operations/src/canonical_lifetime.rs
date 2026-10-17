// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Caller admission, admitted continuation and the actual client pair's teardown.

use super::CanonicalError;
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

#[derive(Clone, Debug, Default)]
pub(crate) struct ConnectionAccess {
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
    async move {
        match activity {
            Some(activity) => activity.scope(future).await,
            None => future.await,
        }
    }
}

impl ConnectionAccess {
    pub(crate) fn admit(&self) -> Result<ConnectionActivity, CanonicalError> {
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
    pub(crate) fn close_caller(&self) {
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
    #[cfg(any(test, feature = "test-support", feature = "canonical-tests"))]
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
    /// protected releases. Seal new admission before joining actual SDK terminal
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
            tokio::spawn(async move {
                let result = async {
                    readers().await?;
                    let ordinary_done = owner
                        .physical
                        .state
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .ordinary_drained;
                    if !ordinary_done {
                        ordinary().await?;
                        owner
                            .physical
                            .state
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            .ordinary_drained = true;
                    }
                    let activation_done = owner
                        .physical
                        .state
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .activation_drained;
                    if !activation_done {
                        activation().await?;
                        owner
                            .physical
                            .state
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            .activation_drained = true;
                    }
                    Ok(())
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
}
