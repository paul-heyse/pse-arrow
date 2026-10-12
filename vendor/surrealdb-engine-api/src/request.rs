//! Local request lifetime. These fields never become RPC session state.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};
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
    async fn joined_driver_drain_waits_for_both_leaves_without_waiting_for_caller_hold() {
        let first = Arc::new(vec![0u8; 2048]);
        let second = Arc::new(vec![1u8; 2048]);
        let first_weak = Arc::downgrade(&first);
        let second_weak = Arc::downgrade(&second);
        let first_context =
            RequestContext::new(Instant::now() + std::time::Duration::from_secs(30))
                .with_payload_owner(first);
        // A native hold acquired before composition must still be joined.
        let first_driver = first_context.payload_owner().unwrap().retain_driver();
        let context = first_context.with_payload_owner(second);
        let driver = context.payload_owner().unwrap().retain_driver();
        let caller = context.payload_owner().unwrap().retain();
        let retired = context.clone();
        context.release_payload_after_drain();
        let mut drained = Box::pin(context.payload_driver_drained());
        assert!(futures::poll!(&mut drained).is_pending());
        drop(driver);
        assert!(futures::poll!(&mut drained).is_pending());
        drop(first_driver);
        drained.await;
        assert!(first_weak.upgrade().is_some() && second_weak.upgrade().is_some());
        drop(caller);
        assert!(first_weak.upgrade().is_none() && second_weak.upgrade().is_none());
        retired.release_payload_after_drain();
    }

    #[test]
    fn lifetime_composition_and_context_replacement_preserve_clearable_cells() {
        let first = Arc::new(());
        let second = Arc::new(());
        let bytes = Arc::new(vec![0u8; 8]);
        let first_weak = Arc::downgrade(&first);
        let second_weak = Arc::downgrade(&second);
        let bytes_weak = Arc::downgrade(&bytes);
        let old = RequestContext::new(Instant::now() + std::time::Duration::from_secs(30))
            .with_lifetime_owner(first).with_payload_owner(bytes);
        let current = RequestContext::new(old.deadline()).with_lifetime_owner(second)
            .inherit_request_ownership(&old);
        let retired = current.clone();
        let handling = current.lifetime_owner().unwrap().retain();
        current.mark_completed();
        assert!(first_weak.upgrade().is_some(), "dispatch fact is not local work drain");
        current.release_lifetime_after_drain();
        assert!(first_weak.upgrade().is_some() && second_weak.upgrade().is_some());
        assert!(bytes_weak.upgrade().is_some(), "non-byte completion cannot release payload bytes");
        assert!(retired.lifetime_owner().unwrap().retain().0.is_empty());
        drop(handling);
        assert!(first_weak.upgrade().is_none() && second_weak.upgrade().is_none());
        old.release_lifetime_after_drain();
        retired.release_lifetime_after_drain();
        current.release_payload_after_drain();
        assert!(bytes_weak.upgrade().is_none());
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
pub struct RequestContext(Arc<Lifecycle>, Option<PayloadOwnership>, Option<RequestLifetimeOwnership>);

/// Non-byte local activity ownership. Metadata clones share clearable cells;
/// actual response handling retains separate holds until its local work drains.
#[derive(Debug, Clone)]
pub struct RequestLifetimeOwnership(Arc<RequestLifetimeContents>);

#[derive(Debug)]
enum RequestLifetimeContents {
    Owner(std::sync::Mutex<Option<Arc<dyn std::fmt::Debug + Send + Sync>>>),
    Joined(RequestLifetimeOwnership, RequestLifetimeOwnership),
}

/// Actual driver work, independent of retired request metadata.
#[derive(Debug, Default)]
pub struct RequestLifetimeRetention(Vec<Arc<dyn std::fmt::Debug + Send + Sync>>);

impl RequestLifetimeOwnership {
    fn new(owner: Arc<dyn std::fmt::Debug + Send + Sync>) -> Self {
        Self(Arc::new(RequestLifetimeContents::Owner(std::sync::Mutex::new(Some(owner)))))
    }
    fn join(self, other: Self) -> Self {
        if Arc::ptr_eq(&self.0, &other.0) { return self; }
        Self(Arc::new(RequestLifetimeContents::Joined(self, other)))
    }
    /// Snapshot only actual live work; released metadata grants no new hold.
    pub fn retain(&self) -> RequestLifetimeRetention {
        let mut retained = RequestLifetimeRetention::default();
        self.retain_into(&mut retained);
        retained
    }
    fn retain_into(&self, retained: &mut RequestLifetimeRetention) {
        match self.0.as_ref() {
            RequestLifetimeContents::Owner(owner) => {
                let owner = owner.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                if let Some(owner) = owner.as_ref() { retained.0.push(owner.clone()); }
            }
            RequestLifetimeContents::Joined(first, second) => {
                first.retain_into(retained);
                second.retain_into(retained);
            }
        }
    }
    fn release_after_drain(&self) {
        match self.0.as_ref() {
            RequestLifetimeContents::Owner(owner) => {
                // Drop outside the slot lock: owner destructors may notify waiters.
                let owner = owner.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take();
                drop(owner);
            }
            RequestLifetimeContents::Joined(first, second) => {
                first.release_after_drain();
                second.release_after_drain();
            }
        }
    }
}

/// Shared local allocation drain state. Joined metadata keeps only clearable
/// allocation leaves, never an independently retained immutable grant.
#[derive(Debug, Clone)]
pub struct PayloadOwnership(Arc<PayloadContents>);

/// Actual allocation holds for live response bytes. Native driver snapshots
/// additionally witness physical response handling; caller holds do not block
/// that join and keep the delivered values charged while it waits.
#[derive(Debug, Default)]
pub struct PayloadRetention(Vec<PayloadHold>);

#[derive(Debug)]
struct PayloadHold {
    // Fields drop in order: release actual bytes before publishing driver drain.
    _owner: Arc<dyn std::fmt::Debug + Send + Sync>,
    _driver: Option<DriverRetention>,
}

#[derive(Debug)]
struct DriverRetention(PayloadOwnership);
impl Drop for DriverRetention {
    fn drop(&mut self) {
        if let PayloadContents::Allocation(allocation) = self.0.0.as_ref() {
            allocation.drivers.fetch_sub(1, Ordering::AcqRel);
            allocation.wake.notify_waiters();
        }
    }
}

#[derive(Debug)]
struct PayloadAllocation {
    owner: std::sync::Mutex<Option<Arc<dyn std::fmt::Debug + Send + Sync>>>,
    drivers: AtomicUsize,
    released: AtomicBool,
    wake: Notify,
}

#[derive(Debug)]
enum PayloadContents {
    Allocation(PayloadAllocation),
    Joined(PayloadOwnership, PayloadOwnership),
}
impl PayloadOwnership {
    fn new(owner: Arc<dyn std::fmt::Debug + Send + Sync>) -> Self {
        Self(Arc::new(PayloadContents::Allocation(PayloadAllocation {
            owner: std::sync::Mutex::new(Some(owner)),
            drivers: AtomicUsize::new(0),
            released: AtomicBool::new(false),
            wake: Notify::new(),
        })))
    }
    fn join(self, other: Self) -> Self {
        if Arc::ptr_eq(&self.0, &other.0) {
            return self;
        }
        Self(Arc::new(PayloadContents::Joined(self, other)))
    }
    /// Keep actual caller-owned decoded values charged through receive and drain.
    pub fn retain(&self) -> PayloadRetention {
        self.retain_kind(false)
    }
    /// Native driver-only hold, released after frame/decoded send ownership drains.
    pub fn retain_driver(&self) -> PayloadRetention {
        self.retain_kind(true)
    }
    fn retain_kind(&self, driver: bool) -> PayloadRetention {
        let mut retained = PayloadRetention::default();
        self.retain_into(&mut retained, driver);
        retained
    }
    fn retain_into(&self, retained: &mut PayloadRetention, driver: bool) {
        match self.0.as_ref() {
            PayloadContents::Allocation(allocation) => {
                let owner = allocation
                    .owner
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if let Some(owner) = owner.as_ref() {
                    // The slot lock orders snapshot admission against release.
                    // Once release takes the slot, no new driver hold can appear.
                    let guard = driver.then(|| {
                        allocation.drivers.fetch_add(1, Ordering::AcqRel);
                        DriverRetention(self.clone())
                    });
                    retained.0.push(PayloadHold {
                        _owner: Arc::clone(owner),
                        _driver: guard,
                    });
                }
            }
            PayloadContents::Joined(first, second) => {
                first.retain_into(retained, driver);
                second.retain_into(retained, driver);
            }
        }
    }
    /// Driver-only: all actual command/message/socket copies must have drained.
    /// Lifecycle completion, cancellation and metadata retirement do not prove it.
    pub fn release_after_drain(&self) {
        match self.0.as_ref() {
            PayloadContents::Allocation(allocation) => {
                let owner = allocation
                    .owner
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .take();
                if let Some(owner) = owner {
                    // A concurrent idempotent release must not publish this
                    // witness while the first release is still dropping bytes.
                    drop(owner);
                    allocation.released.store(true, Ordering::Release);
                    allocation.wake.notify_waiters();
                }
            }
            PayloadContents::Joined(first, second) => {
                first.release_after_drain();
                second.release_after_drain();
            }
        }
    }
    fn leaves(&self, leaves: &mut Vec<PayloadOwnership>) {
        match self.0.as_ref() {
            PayloadContents::Allocation(_) => leaves.push(self.clone()),
            PayloadContents::Joined(first, second) => {
                first.leaves(leaves);
                second.leaves(leaves);
            }
        }
    }
    async fn driver_drained(&self) {
        let mut leaves = Vec::new();
        self.leaves(&mut leaves);
        for leaf in leaves {
            if let PayloadContents::Allocation(allocation) = leaf.0.as_ref() {
                loop {
                    let wake = allocation.wake.notified();
                    tokio::pin!(wake);
                    wake.as_mut().enable();
                    if allocation.released.load(Ordering::Acquire)
                        && allocation.drivers.load(Ordering::Acquire) == 0
                    {
                        break;
                    }
                    wake.await;
                }
            }
        }
    }
}

/// Lexical cleanup for request ownership which never reached a transport. Declare this
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
                context.release_lifetime_after_drain();
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
            None,
        )
    }

    /// Keep non-byte local activity through actual terminal completion or drain.
    /// This does not classify payload bytes or authorize replay or remote effects.
    pub fn with_lifetime_owner(mut self, owner: Arc<dyn std::fmt::Debug + Send + Sync>) -> Self {
        let next = RequestLifetimeOwnership::new(owner);
        self.2 = Some(match self.2.take() {
            Some(previous) => previous.join(next),
            None => next,
        });
        self
    }
    /// Shared clearable activity metadata; taking a hold is driver-only.
    pub fn lifetime_owner(&self) -> Option<RequestLifetimeOwnership> { self.2.clone() }
    /// Driver-only local terminal cleanup, independent of remote outcome.
    pub fn release_lifetime_after_drain(&self) {
        if let Some(owner) = &self.2 { owner.release_after_drain(); }
    }
    /// Whether a builder needs lexical undispatched cleanup for either owner.
    pub fn has_request_ownership(&self) -> bool { self.1.is_some() || self.2.is_some() }

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
    /// Preserve both independent drain cells when a builder receives a current lifecycle.
    /// Copying the allocation out would strand it in retired old metadata.
    pub fn inherit_request_ownership(mut self, previous: &Self) -> Self {
        if let Some(previous) = &previous.1 {
            self.1 = Some(match self.1.take() {
                Some(current) => current.join(previous.clone()),
                None => previous.clone(),
            });
        }
        if let Some(previous) = &previous.2 {
            self.2 = Some(match self.2.take() {
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

    /// Join physical native response handling, independently of the caller's
    /// decoded-value hold. The caller supplies this context's original clock.
    pub async fn payload_driver_drained(&self) {
        if let Some(owner) = &self.1 {
            owner.driver_drained().await;
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
