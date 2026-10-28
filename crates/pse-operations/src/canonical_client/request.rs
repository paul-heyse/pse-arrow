// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Local request lifetime. These fields never become RPC session state.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};
use std::time::Instant;

use crate::canonical::{ConnectionActivity, ProtectedSelection};
use datafusion_execution::memory_pool::MemoryReservation;
use surrealdb_types::{ConnectionError, Error};

// These values retain already-issued custody. They cannot mint caller admission,
// byte allowance, scientific permission or a new lifetime from attribution.
#[derive(Clone, Debug)]
enum LifetimeCustody {
    Connection { _owner: ConnectionActivity },
    Selection { _owner: ProtectedSelection },
    Retained { _owner: super::RetainedRequest },
}

// Only this closed, already typed custody enum enters existential storage.
// The private boundary breaks the recursive Send graph through an issued read's
// store/connection bootstrap. It cannot accept arbitrary Debug owners or issue
// authority; physical retention and release use the same original cells.
trait IssuedCustody: std::fmt::Debug + Send + Sync {}
impl IssuedCustody for LifetimeCustody {}

use tokio::sync::Notify;

/// Whether transport transmission has started. A disconnected dispatched
/// request has an unknown server outcome; it is never safe to replay blindly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchState {
    /// No native request was transmitted.
    NotDispatched,
    /// Native transmission started; effects may require receipt settlement.
    Dispatched,
    /// Native response decoding completed.
    Completed,
}

#[derive(Debug)]
struct Lifecycle {
    deadline: Instant,
    control: bool,
    outbound: Arc<super::outbound::Progress>,
    cancelled: AtomicBool,
    state: AtomicU8,
    enqueued: AtomicBool,
    wake: Notify,
}

/// Shared original deadline and cancellation for one logical operation.
#[derive(Debug, Clone)]
pub struct RequestContext(
    Arc<Lifecycle>,
    Option<PayloadOwnership>,
    Option<RequestLifetimeOwnership>,
    Option<ResponseLimits>,
);

/// Finite decoded metadata envelope for one actual request. Counts apply to
/// occurrences across response frames before owned conversion.
#[derive(Debug, Clone, Copy)]
pub struct ResponseLimits {
    /// Cumulative native wire bytes.
    pub wire_bytes: usize,
    /// Cumulative decoded value occurrences.
    pub nodes: usize,
    /// Maximum native value nesting.
    pub depth: usize,
    /// Cumulative native container members.
    pub container_items: usize,
    /// Cumulative UTF-8 bytes.
    pub string_bytes: usize,
    /// Maximum individual UTF-8 string bytes.
    pub string_length: usize,
    /// Cumulative application binary bytes.
    pub blob_bytes: usize,
    /// Maximum individual application binary bytes.
    pub blob_length: usize,
}

/// Non-byte local activity ownership. Metadata clones share clearable cells;
/// actual response handling retains separate holds until its local work drains.
#[derive(Debug, Clone)]
pub(crate) struct RequestLifetimeOwnership(Arc<RequestLifetimeContents>);

#[derive(Debug)]
enum RequestLifetimeContents {
    Owner(std::sync::Mutex<Option<Arc<dyn IssuedCustody>>>),
    Joined(RequestLifetimeOwnership, RequestLifetimeOwnership),
}

/// Actual driver work, independent of retired request metadata.
#[derive(Debug, Default, Clone)]
pub(crate) struct RequestLifetimeRetention(Vec<Arc<dyn IssuedCustody>>);

impl RequestLifetimeOwnership {
    fn new(owner: LifetimeCustody) -> Self {
        Self(Arc::new(RequestLifetimeContents::Owner(
            std::sync::Mutex::new(Some(Arc::new(owner))),
        )))
    }
    fn join(self, other: Self) -> Self {
        if Arc::ptr_eq(&self.0, &other.0) {
            return self;
        }
        Self(Arc::new(RequestLifetimeContents::Joined(self, other)))
    }
    /// Snapshot only actual live work; released metadata grants no new hold.
    pub(super) fn retain(&self) -> RequestLifetimeRetention {
        let mut retained = RequestLifetimeRetention::default();
        self.retain_into(&mut retained);
        retained
    }
    fn retain_into(&self, retained: &mut RequestLifetimeRetention) {
        match self.0.as_ref() {
            RequestLifetimeContents::Owner(owner) => {
                let owner = owner
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if let Some(owner) = owner.as_ref() {
                    retained.0.push(owner.clone());
                }
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
                let owner = owner
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .take();
                drop(owner);
            }
            RequestLifetimeContents::Joined(first, second) => {
                first.release_after_drain();
                second.release_after_drain();
            }
        }
    }
}

#[derive(Clone, Debug)]
enum AllocationCustody {
    Response {
        _reservation: Arc<MemoryReservation>,
    },
    Scientific {
        _lease: Arc<pse_columnar::AllocationLease>,
    },
}

/// Shared local allocation drain state. Joined metadata keeps only clearable
/// allocation leaves, never an independently retained immutable grant.
#[derive(Debug, Clone)]
pub(crate) struct PayloadOwnership(Arc<PayloadContents>);

/// Actual allocation holds for live response bytes. Native driver snapshots
/// additionally witness physical response handling; caller holds do not block
/// that join and keep the delivered values charged while it waits.
#[derive(Debug, Default)]
pub(crate) struct PayloadRetention(Vec<PayloadHold>);

#[derive(Debug)]
struct PayloadHold {
    // Fields drop in order: release actual bytes before publishing driver drain.
    _owner: AllocationCustody,
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
    owner: std::sync::Mutex<Option<AllocationCustody>>,
    drivers: AtomicUsize,
    released: AtomicBool,
    wake: Notify,
}

#[derive(Debug)]
enum PayloadContents {
    Allocation(PayloadAllocation),
    Joined(PayloadOwnership, PayloadOwnership),
}
impl PayloadRetention {
    /// A body continuation copies its already issued physical holds, never
    /// attempts admission through a retired request-metadata cell.
    pub(super) fn fork_driver(&self) -> Self {
        Self(
            self.0
                .iter()
                .map(|hold| PayloadHold {
                    _owner: hold._owner.clone(),
                    _driver: hold._driver.as_ref().map(|driver| {
                        if let PayloadContents::Allocation(allocation) = driver.0.0.as_ref() {
                            allocation.drivers.fetch_add(1, Ordering::AcqRel);
                        }
                        DriverRetention(driver.0.clone())
                    }),
                })
                .collect(),
        )
    }
}
impl PayloadOwnership {
    fn new(owner: AllocationCustody) -> Self {
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
    pub(super) fn retain(&self) -> PayloadRetention {
        self.retain_kind(false)
    }
    /// Native driver-only hold, released after frame/decoded send ownership drains.
    pub(super) fn retain_driver(&self) -> PayloadRetention {
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
                        _owner: owner.clone(),
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
    pub(super) fn release_after_drain(&self) {
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
    #[cfg(test)]
    fn leaves(&self, leaves: &mut Vec<PayloadOwnership>) {
        match self.0.as_ref() {
            PayloadContents::Allocation(_) => leaves.push(self.clone()),
            PayloadContents::Joined(first, second) => {
                first.leaves(leaves);
                second.leaves(leaves);
            }
        }
    }
    #[cfg(test)]
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
pub(crate) struct UndispatchedPayloadDrain {
    context: Option<RequestContext>,
}
impl UndispatchedPayloadDrain {
    pub(super) fn caller(context: Option<RequestContext>) -> Self {
        Self { context }
    }
    pub(super) fn update_context(&mut self, context: Option<RequestContext>) {
        self.context = context;
    }
}
impl Drop for UndispatchedPayloadDrain {
    fn drop(&mut self) {
        if let Some(context) = &self.context {
            if context.dispatch_state() == DispatchState::NotDispatched
                && !context.0.enqueued.load(Ordering::Acquire)
            {
                context.release_payload_after_drain();
                context.release_lifetime_after_drain();
            }
        }
    }
}

impl RequestContext {
    /// Bind one request to its original monotonic deadline.
    pub fn new(deadline: Instant) -> Self {
        Self::with_class(deadline, false)
    }

    /// Reserved control request; shares the same finite connection envelope.
    pub fn control(deadline: Instant) -> Self {
        Self::with_class(deadline, true)
    }

    fn with_class(deadline: Instant, control: bool) -> Self {
        Self(
            Arc::new(Lifecycle {
                deadline,
                control,
                outbound: Arc::new(super::outbound::Progress::default()),
                cancelled: AtomicBool::new(false),
                state: AtomicU8::new(0),
                enqueued: AtomicBool::new(false),
                wake: Notify::new(),
            }),
            None,
            None,
            None,
        )
    }

    /// Apply the endpoint's finite borrowed metadata preflight.
    pub fn with_response_limits(mut self, limits: ResponseLimits) -> Self {
        self.3 = Some(limits);
        self
    }
    /// Return the explicit finite response envelope, when supplied.
    pub fn response_limits(&self) -> Option<ResponseLimits> {
        self.3
    }

    /// Keep non-byte local activity through actual terminal completion or drain.
    /// This does not classify payload bytes or authorize replay or remote effects.
    pub fn with_lifetime_owner(self, owner: ConnectionActivity) -> Self {
        self.with_custody(LifetimeCustody::Connection { _owner: owner })
    }
    /// Retain a checked protection through native response completion. This is
    /// custody of an existing pin, never admission of another operation.
    pub(crate) fn with_selection_custody(self, selection: ProtectedSelection) -> Self {
        self.with_custody(LifetimeCustody::Selection { _owner: selection })
    }
    fn with_custody(mut self, owner: LifetimeCustody) -> Self {
        let next = RequestLifetimeOwnership::new(owner);
        self.2 = Some(match self.2.take() {
            Some(previous) => previous.join(next),
            None => next,
        });
        self
    }
    /// Retain actual issued read/release guards through native physical completion.
    /// Neither allocation allowance nor execution authority is created here.
    pub fn with_retained_request(self, owner: impl Into<super::RetainedRequest>) -> Self {
        self.with_custody(LifetimeCustody::Retained {
            _owner: owner.into(),
        })
    }
    /// Shared clearable activity metadata; taking a hold is driver-only.
    pub(crate) fn lifetime_owner(&self) -> Option<RequestLifetimeOwnership> {
        self.2.clone()
    }
    /// Driver-only local terminal cleanup, independent of remote outcome.
    pub(crate) fn release_lifetime_after_drain(&self) {
        if let Some(owner) = &self.2 {
            owner.release_after_drain();
        }
    }
    /// Keep the actual prepaid request allocation through local transport drain.
    /// This owner never becomes a binding, session value or replay permission.
    pub fn with_payload_owner(mut self, owner: Arc<MemoryReservation>) -> Self {
        let next = PayloadOwnership::new(AllocationCustody::Response {
            _reservation: owner,
        });
        self.1 = Some(match self.1.take() {
            Some(previous) => previous.join(next),
            None => next,
        });
        self
    }
    /// Retain existing scientific staging bytes; this grants no response allowance.
    pub(crate) fn with_scientific_payload_owner(
        mut self,
        owner: Arc<pse_columnar::AllocationLease>,
    ) -> Self {
        let next = PayloadOwnership::new(AllocationCustody::Scientific { _lease: owner });
        self.1 = Some(match self.1.take() {
            Some(previous) => previous.join(next),
            None => next,
        });
        self
    }
    /// Preserve both independent drain cells when a builder receives a current lifecycle.
    /// Copying the allocation out would strand it in retired old metadata.
    pub(crate) fn inherit_request_ownership(mut self, previous: &Self) -> Self {
        if self.3.is_none() {
            self.3 = previous.3;
        }
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
    #[cfg(test)]
    pub(crate) fn same_lifecycle(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
    #[cfg(test)]
    pub(crate) fn payload_leaf_count(&self) -> usize {
        let mut leaves = Vec::new();
        if let Some(owner) = &self.1 {
            owner.leaves(&mut leaves);
        }
        leaves.len()
    }
    /// Shared driver drain ownership, independent of epoch metadata reclamation.
    pub(crate) fn payload_owner(&self) -> Option<PayloadOwnership> {
        self.1.clone()
    }
    /// The caller must prove the actual bytes drained, not merely a clock event.
    pub(crate) fn release_payload_after_drain(&self) {
        if let Some(owner) = &self.1 {
            owner.release_after_drain();
        }
    }

    /// Join physical native response handling, independently of the caller's
    /// decoded-value hold. The caller supplies this context's original clock.
    #[cfg(test)]
    pub(crate) async fn payload_driver_drained(&self) {
        if let Some(owner) = &self.1 {
            owner.driver_drained().await;
        }
    }

    pub(super) fn outbound(&self) -> Arc<super::outbound::Progress> {
        self.0.outbound.clone()
    }
    pub(super) fn outbound_is_drained(&self) -> bool {
        self.0.outbound.is_drained()
    }
    pub(super) async fn outbound_drained(&self) {
        self.0.outbound.wait().await;
    }
    pub(crate) fn deadline(&self) -> Instant {
        self.0.deadline
    }
    pub(crate) fn is_control(&self) -> bool {
        self.0.control
    }
    pub(crate) fn was_dispatched(&self) -> bool {
        self.0.state.load(Ordering::Acquire) != 0
    }

    /// Stop caller admission without claiming physical native drain.
    pub fn cancel(&self) {
        self.0.cancelled.store(true, Ordering::Release);
        self.0.wake.notify_waiters();
    }

    pub(crate) fn is_interrupted(&self) -> bool {
        self.0.cancelled.load(Ordering::Acquire) || Instant::now() >= self.0.deadline
    }

    pub(crate) fn dispatch_state(&self) -> DispatchState {
        match self.0.state.load(Ordering::Acquire) {
            0 => DispatchState::NotDispatched,
            1 => DispatchState::Dispatched,
            _ => DispatchState::Completed,
        }
    }

    /// Transport-only marker after successful channel enqueue; no permission.
    pub(crate) fn mark_enqueued(&self) {
        self.0.enqueued.store(true, Ordering::Release);
    }

    /// Driver-only transmission marker, immediately before start_send.
    pub(crate) fn mark_dispatched(&self) {
        self.0.state.store(1, Ordering::Release);
    }

    /// Driver-only complete response marker. It does not assert scientific drain.
    pub(crate) fn mark_completed(&self) {
        self.0.state.store(2, Ordering::Release);
    }

    pub(crate) fn interruption_error(&self) -> Error {
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
    pub(crate) async fn interrupted(&self) {
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

#[cfg(test)]
mod custody_tests {
    use super::*;
    use pse_columnar::{AllocationLease, GreedyMemoryPool, MemoryConsumer, MemoryPool};
    #[tokio::test]
    async fn scientific_lease_and_response_reservation_keep_independent_physical_custody() {
        let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(96));
        let staging = MemoryConsumer::new("test:scientific-request").register(&pool);
        staging.try_grow(64).unwrap();
        let response = MemoryConsumer::new("test:response-request").register(&pool);
        response.try_grow(32).unwrap();
        let context = RequestContext::new(Instant::now() + std::time::Duration::from_secs(1))
            .with_scientific_payload_owner(AllocationLease::new(staging))
            .with_payload_owner(Arc::new(response));
        let retained = context.payload_owner().unwrap().retain_driver();
        context.release_payload_after_drain();
        assert_eq!(
            pool.reserved(),
            96,
            "retired metadata does not release live native bytes"
        );
        drop(retained);
        context.payload_driver_drained().await;
        assert_eq!(
            pool.reserved(),
            0,
            "both independently prepaid allocations physically drain"
        );
    }
}
