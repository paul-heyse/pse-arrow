//! Local request lifetime. These fields never become RPC session state.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::time::Instant;

use surrealdb_types::{ConnectionError, Error};
use tokio::sync::Notify;

/// Whether transport transmission has started. A disconnected dispatched
/// request has an unknown server outcome; it is never safe to replay blindly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchState {
    NotDispatched,
    Dispatched,
    Completed,
}

#[cfg(test)]
mod lifecycle_tests {
    use super::*;

    #[tokio::test]
    async fn cancellation_wakes_waiter_and_preserves_dispatch_uncertainty() {
        let context = RequestContext::new(Instant::now() + std::time::Duration::from_secs(30));
        let other = context.clone();
        let cancel = async move {
            other.mark_dispatched();
            other.cancel();
        };
        tokio::join!(context.interrupted(), cancel);
        assert_eq!(context.dispatch_state(), DispatchState::Dispatched);
        assert!(
            context
                .interruption_error()
                .to_string()
                .contains("unknown server outcome")
        );
        context.mark_completed();
        assert!(context.was_dispatched());
    }

    #[tokio::test]
    async fn expired_original_clock_never_becomes_a_new_wait() {
        let context = RequestContext::new(Instant::now() - std::time::Duration::from_secs(1));
        context.interrupted().await;
        assert_eq!(context.dispatch_state(), DispatchState::NotDispatched);
        assert!(
            context
                .interruption_error()
                .to_string()
                .contains("not dispatched")
        );
    }
}

#[derive(Debug)]
struct Lifecycle {
    deadline: Instant,
    control: bool,
    selection_checkpoint: Option<uuid::Uuid>,
    cancelled: AtomicBool,
    state: AtomicU8,
    wake: Notify,
}

/// Shared original deadline and cancellation for one logical operation.
#[derive(Debug, Clone)]
pub struct RequestContext(Arc<Lifecycle>);

impl RequestContext {
    pub fn new(deadline: Instant) -> Self {
        Self::with_class(deadline, false, None)
    }

    /// Reserved control request; shares the same finite connection envelope.
    pub fn control(deadline: Instant) -> Self {
        Self::with_class(deadline, true, None)
    }

    /// Final acknowledged owner authentication of a serialized read-only
    /// selection sequence. Reuse one opaque identity per client and selection.
    /// Only adjacent completed same-identity auth / exact USE / auth triples
    /// are replaced; unrelated session setup retains its complete ordering.
    pub fn selection_checkpoint(deadline: Instant, identity: uuid::Uuid) -> Self {
        Self::with_class(deadline, true, Some(identity))
    }

    pub fn selection_checkpoint_identity(&self) -> Option<uuid::Uuid> {
        self.0.selection_checkpoint
    }

    fn with_class(
        deadline: Instant,
        control: bool,
        selection_checkpoint: Option<uuid::Uuid>,
    ) -> Self {
        Self(Arc::new(Lifecycle {
            deadline,
            control,
            selection_checkpoint,
            cancelled: AtomicBool::new(false),
            state: AtomicU8::new(0),
            wake: Notify::new(),
        }))
    }

    pub fn deadline(&self) -> Instant {
        self.0.deadline
    }
    pub fn is_control(&self) -> bool {
        self.0.control
    }
    pub fn was_dispatched(&self) -> bool {
        self.0.state.load(Ordering::Acquire) != 0
    }

    pub fn cancel(&self) {
        self.0.cancelled.store(true, Ordering::Release);
        self.0.wake.notify_waiters();
    }

    pub fn is_interrupted(&self) -> bool {
        self.0.cancelled.load(Ordering::Acquire) || Instant::now() >= self.0.deadline
    }

    pub fn dispatch_state(&self) -> DispatchState {
        match self.0.state.load(Ordering::Acquire) {
            0 => DispatchState::NotDispatched,
            1 => DispatchState::Dispatched,
            _ => DispatchState::Completed,
        }
    }

    /// Driver-only transmission marker, immediately before start_send.
    pub fn mark_dispatched(&self) {
        self.0.state.store(1, Ordering::Release);
    }

    /// Driver-only complete response marker. It does not assert scientific drain.
    pub fn mark_completed(&self) {
        self.0.state.store(2, Ordering::Release);
    }

    pub fn interruption_error(&self) -> Error {
        let effect = match self.dispatch_state() {
            DispatchState::NotDispatched => "not dispatched",
            DispatchState::Dispatched => "unknown server outcome",
            DispatchState::Completed => "response completed",
        };
        Error::connection(
            format!("Request interrupted: {effect}"),
            ConnectionError::ConnectionFailed,
        )
    }

    /// Wake promptly on cancellation or the original deadline, without resetting it.
    pub async fn interrupted(&self) {
        loop {
            let wake = self.0.wake.notified();
            tokio::pin!(wake);
            wake.as_mut().enable();
            if self.is_interrupted() {
                return;
            }
            tokio::select! {
                _ = tokio::time::sleep_until(self.0.deadline.into()) => return,
                _ = wake => {},
            }
        }
    }
}

/// Cancels local admission when its owning request future is dropped. Completed
/// responses keep their historical dispatch fact and need no drain cancellation.
pub(crate) struct CancelOnDrop(pub(crate) Option<RequestContext>);
impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        if let Some(context) = &self.0 {
            if context.dispatch_state() != DispatchState::Completed {
                context.cancel();
            }
        }
    }
}

/// Admission ownership with explicit completion, independent of concurrent-map
/// epoch reclamation. Release is idempotent across temporary map snapshots.
#[derive(Debug)]
pub struct AdmissionPermit(std::sync::Mutex<Option<tokio::sync::OwnedSemaphorePermit>>);
impl AdmissionPermit {
    pub(crate) fn new(permit: tokio::sync::OwnedSemaphorePermit) -> Self {
        Self(std::sync::Mutex::new(Some(permit)))
    }
    pub fn release(&self) {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
    }
}
