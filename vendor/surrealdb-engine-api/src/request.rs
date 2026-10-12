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
    async fn completed_semaphore_does_not_release_live_payload_copies() {
        let owner = Arc::new(vec![0u8; 4096]);
        let weak = Arc::downgrade(&owner);
        let context = RequestContext::new(Instant::now() + std::time::Duration::from_secs(30))
            .with_payload_owner(owner.clone());
        let semaphore = Arc::new(tokio::sync::Semaphore::new(1));
        let permit = AdmissionPermit::new(
            semaphore.clone().acquire_owned().await.unwrap(),
            context.payload_owner(),
        );
        drop(owner);
        context.mark_completed();
        let retired = context.clone();
        permit.release();
        assert_eq!(semaphore.available_permits(), 1);
        assert_eq!(
            weak.upgrade().unwrap().len(),
            4096,
            "byte drain outlives semaphore completion"
        );
        context.release_payload_after_drain();
        assert!(
            weak.upgrade().is_none(),
            "retired context and permit metadata cannot strand a drained grant"
        );
        retired.release_payload_after_drain(); // Explicit drain is idempotent.
        drop(permit);
    }

    #[test]
    fn response_retention_outlives_cleared_joined_metadata() {
        let first = Arc::new(vec![0u8; 2048]);
        let second = Arc::new(vec![1u8; 2048]);
        let first_weak = Arc::downgrade(&first);
        let second_weak = Arc::downgrade(&second);
        let context = RequestContext::new(Instant::now() + std::time::Duration::from_secs(30))
            .with_payload_owner(first)
            .with_payload_owner(second);
        let retired = context.clone();
        let retained = context.payload_owner().unwrap().retain();
        context.mark_completed();
        context.release_payload_after_drain();
        assert!(first_weak.upgrade().is_some());
        assert!(second_weak.upgrade().is_some());
        assert!(
            context.payload_owner().unwrap().retain().0.is_empty(),
            "retired metadata is empty while actual response bytes retain the grants"
        );
        drop(retained);
        assert!(first_weak.upgrade().is_none());
        assert!(second_weak.upgrade().is_none());
        retired.release_payload_after_drain();
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
    enqueued: AtomicBool,
    wake: Notify,
}

/// Shared original deadline and cancellation for one logical operation.
#[derive(Debug, Clone)]
pub struct RequestContext(Arc<Lifecycle>, Option<PayloadOwnership>);

/// Shared local allocation drain cell. Retired correlation metadata keeps this
/// cell, never an independently retained immutable allocation grant.
#[derive(Debug, Clone)]
pub struct PayloadOwnership(Arc<std::sync::Mutex<Option<PayloadContents>>>);

/// Actual allocation holds for live response bytes, independent of the
/// clearable correlation cell. Keep this lexical owner after those bytes;
/// never store it in retired request metadata.
#[derive(Debug, Default)]
pub struct PayloadRetention(Vec<Arc<dyn std::fmt::Debug + Send + Sync>>);

#[derive(Debug)]
enum PayloadContents {
    Allocation(Arc<dyn std::fmt::Debug + Send + Sync>),
    Joined(PayloadOwnership, PayloadOwnership),
}
impl PayloadOwnership {
    fn new(owner: Arc<dyn std::fmt::Debug + Send + Sync>) -> Self {
        Self(Arc::new(std::sync::Mutex::new(Some(
            PayloadContents::Allocation(owner),
        ))))
    }
    fn join(self, other: Self) -> Self {
        if Arc::ptr_eq(&self.0, &other.0) {
            return self;
        }
        Self(Arc::new(std::sync::Mutex::new(Some(
            PayloadContents::Joined(self, other),
        ))))
    }
    /// Snapshot the actual grants before response decoding or cell completion.
    /// The serial driver owns the corresponding pending request at this point.
    pub fn retain(&self) -> PayloadRetention {
        let mut retained = PayloadRetention::default();
        self.retain_into(&mut retained);
        retained
    }
    fn retain_into(&self, retained: &mut PayloadRetention) {
        let contents = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match contents.as_ref() {
            Some(PayloadContents::Allocation(owner)) => retained.0.push(Arc::clone(owner)),
            Some(PayloadContents::Joined(first, second)) => {
                first.retain_into(retained);
                second.retain_into(retained);
            }
            None => {}
        }
    }
    /// Driver-only: all actual command/message/socket copies must have drained.
    /// Lifecycle completion, cancellation and metadata retirement do not prove it.
    pub fn release_after_drain(&self) {
        let contents = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        match contents {
            Some(PayloadContents::Allocation(owner)) => drop(owner),
            Some(PayloadContents::Joined(first, second)) => {
                first.release_after_drain();
                second.release_after_drain();
            }
            None => {}
        }
    }
}

/// Lexical cleanup for bytes which never reached a transport. Declare this
/// before the actual byte-owning locals (or as their last struct field).
/// Caller guards stop at authenticated enqueue; driver guards stop at dispatch.
#[derive(Debug)]
pub struct UndispatchedPayloadDrain {
    context: Option<RequestContext>,
    caller: bool,
}
impl UndispatchedPayloadDrain {
    pub fn caller(context: Option<RequestContext>) -> Self {
        Self {
            context,
            caller: true,
        }
    }
    pub fn driver(context: Option<RequestContext>) -> Self {
        Self {
            context,
            caller: false,
        }
    }
    pub fn update_context(&mut self, context: Option<RequestContext>) {
        self.context = context;
    }
}
impl Drop for UndispatchedPayloadDrain {
    fn drop(&mut self) {
        if let Some(context) = &self.context {
            if context.dispatch_state() == DispatchState::NotDispatched
                && (!self.caller || !context.0.enqueued.load(Ordering::Acquire))
            {
                context.release_payload_after_drain();
            }
        }
    }
}

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
        Self(
            Arc::new(Lifecycle {
                deadline,
                control,
                selection_checkpoint,
                cancelled: AtomicBool::new(false),
                state: AtomicU8::new(0),
                enqueued: AtomicBool::new(false),
                wake: Notify::new(),
            }),
            None,
        )
    }

    /// Keep the actual prepaid request allocation through local transport drain.
    /// This owner never becomes a binding, session value or replay permission.
    pub fn with_payload_owner(mut self, owner: Arc<dyn std::fmt::Debug + Send + Sync>) -> Self {
        let next = PayloadOwnership::new(owner);
        self.1 = Some(match self.1.take() {
            Some(previous) => previous.join(next),
            None => next,
        });
        self
    }
    /// Preserve the same drain cell when a query receives a current lifecycle.
    /// Copying the allocation out would strand it in retired old metadata.
    pub fn inherit_payload_ownership(mut self, previous: &Self) -> Self {
        if let Some(previous) = &previous.1 {
            self.1 = Some(match self.1.take() {
                Some(current) => current.join(previous.clone()),
                None => previous.clone(),
            });
        }
        self
    }
    /// Shared driver drain ownership, independent of epoch metadata reclamation.
    pub fn payload_owner(&self) -> Option<PayloadOwnership> {
        self.1.clone()
    }
    /// The caller must prove the actual bytes drained, not merely a clock event.
    pub fn release_payload_after_drain(&self) {
        if let Some(owner) = &self.1 {
            owner.release_after_drain();
        }
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

    /// Transport-only marker after successful channel enqueue; no permission.
    pub fn mark_enqueued(&self) {
        self.0.enqueued.store(true, Ordering::Release);
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
pub struct AdmissionPermit {
    semaphore: std::sync::Mutex<Option<tokio::sync::OwnedSemaphorePermit>>,
    // Semaphore completion and physical byte drain are distinct lifetimes.
    _payload_owner: Option<PayloadOwnership>,
}
impl AdmissionPermit {
    pub(crate) fn new(
        permit: tokio::sync::OwnedSemaphorePermit,
        owner: Option<PayloadOwnership>,
    ) -> Self {
        Self {
            semaphore: std::sync::Mutex::new(Some(permit)),
            _payload_owner: owner,
        }
    }
    pub fn release(&self) {
        self.semaphore
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
    }
}
