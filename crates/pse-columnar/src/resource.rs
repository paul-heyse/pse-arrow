// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native reservations and immutable allocation lifetime (ADR-0073).

use crate::error::CanonError;
use datafusion_execution::memory_pool::MemoryReservation;
use std::sync::Arc;

/// Native lifetime anchor shared by immutable typed payloads and their consumers.
/// This is deliberately outside pure compiler inputs and semantic equality.
pub trait PayloadOwner: std::fmt::Debug + Send + Sync + 'static {}
impl<T: std::fmt::Debug + Send + Sync + 'static> PayloadOwner for T {}

/// Shared value and its original accounting owner. The inner Arc cannot escape.
#[derive(Debug)]
pub struct Leased<T: ?Sized> {
    value: Arc<T>,
    owner: Arc<dyn PayloadOwner>,
}
impl<T: ?Sized> Leased<T> {
    /// Retain the admission owner with an already allocated value.
    /// The caller reserves sufficient capacity before constructing the value.
    pub fn new(value: Arc<T>, owner: Arc<dyn PayloadOwner>) -> Self {
        Self { value, owner }
    }
    /// A second consumer can retain the same owner without a second reservation.
    pub fn owner(&self) -> Arc<dyn PayloadOwner> {
        self.owner.clone()
    }
}
impl<T: ?Sized> std::ops::Deref for Leased<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}

/// Immutable ownership of pre-admitted native capacity. Shared buffers retain this
/// owner; it exposes no mutation of the reservation after publication.
#[derive(Debug)]
pub struct AllocationLease {
    reservation: MemoryReservation,
}
impl AllocationLease {
    /// Freeze a native reservation without releasing or reacquiring its capacity.
    pub fn new(reservation: MemoryReservation) -> Arc<Self> {
        Arc::new(Self { reservation })
    }
    /// Native capacity retained until the last owner drops.
    pub fn size(&self) -> usize {
        self.reservation.size()
    }
}

/// Shared cancellation for asynchronous work and synchronous compilation loops.
///
/// Cloning shares the flag, so a driver holds one token and hands clones to the work it
/// started. `Default` is an uncancelled token.
///
/// ```
/// use pse_columnar::CancellationToken;
///
/// let token = CancellationToken::new();
/// assert!(token.checkpoint().is_ok());
///
/// let handed_out = token.clone();
/// token.cancel();
/// assert!(handed_out.is_cancelled());
/// assert!(handed_out.checkpoint().is_err());
/// ```
#[derive(Clone, Debug, Default)]
pub struct CancellationToken {
    inner: tokio_util::sync::CancellationToken,
}

impl CancellationToken {
    /// Opens an uncancelled token.
    pub fn new() -> Self {
        Self::default()
    }

    /// A child receives parent cancellation without cancelling its parent when
    /// an individual native stream is dropped.
    #[must_use]
    pub fn child_token(&self) -> Self {
        Self {
            inner: self.inner.child_token(),
        }
    }

    /// Cancels this token and every clone of it. Cancellation is one-way.
    pub fn cancel(&self) {
        self.inner.cancel();
    }

    /// Whether the token has been cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.inner.is_cancelled()
    }

    /// Wait for cancellation, including cancellation before the first poll.
    ///
    /// Every token clone wakes every registered waiter. Dropping this future does not
    /// cancel the work, and another waiter can safely be registered later.
    pub async fn cancelled(&self) {
        self.inner.cancelled().await;
    }

    /// Await cancellation-safe work with a wakeable native cancellation race.
    /// Cancellation drops the pending future. Do not use this to infer that an
    /// externally visible write was rolled back; publication must reconcile that state.
    /// # Errors
    /// [`CanonError::Cancelled`] if cancellation wins the native race.
    pub async fn until_cancelled<F: Future>(&self, future: F) -> Result<F::Output, CanonError> {
        self.inner
            .run_until_cancelled(future)
            .await
            .ok_or(CanonError::Cancelled)
    }

    /// The check a loop body performs between units of work.
    ///
    /// # Errors
    ///
    /// [`CanonError::Cancelled`] once the token is cancelled. A cancelled pass records a
    /// terminal finding; it is never an empty successful result (§14.3).
    pub fn checkpoint(&self) -> Result<(), CanonError> {
        if self.is_cancelled() {
            Err(CanonError::Cancelled)
        } else {
            Ok(())
        }
    }
}

impl pse_diagnostics::CancellationCheck for CancellationToken {
    fn checkpoint(&self) -> Result<(), pse_diagnostics::WorkError> {
        if self.is_cancelled() {
            Err(pse_diagnostics::WorkError::Cancelled)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod durability_unit {
    use super::*;
    use crate::{GreedyMemoryPool, MemoryConsumer, MemoryPool};
    #[test]
    fn typed_payload_retains_original_charge_through_last_consumer() {
        let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(128));
        let reservation = MemoryConsumer::new("leased fixture").register(&pool);
        reservation.try_grow(128).unwrap();
        let value = Arc::new(Leased::new(
            Arc::<[u8]>::from([1, 2, 3]),
            AllocationLease::new(reservation),
        ));
        let consumer = value.clone();
        let downstream_owner = consumer.owner();
        drop(value);
        assert_eq!(pool.reserved(), 128);
        assert_eq!(&**consumer, &[1, 2, 3]);
        drop(consumer);
        assert_eq!(pool.reserved(), 128);
        drop(downstream_owner);
        assert_eq!(pool.reserved(), 0);
    }
}

/// Preserve cancellation and native resource causes at a synchronous semantic boundary.
pub fn work_error(error: CanonError) -> pse_diagnostics::WorkError {
    match error {
        CanonError::Cancelled => pse_diagnostics::WorkError::Cancelled,
        error => pse_diagnostics::WorkError::Resource(Box::new(error)),
    }
}

#[cfg(test)]
mod pivot_unit {
    use super::*;
    use datafusion_execution::memory_pool::{GreedyMemoryPool, MemoryConsumer, MemoryPool};
    #[test]
    fn split_and_shared_owner_preserve_native_capacity() {
        let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(128));
        let reserve = MemoryConsumer::new("unit:owner").register(&pool);
        reserve.try_grow(128).unwrap();
        assert!(reserve.try_grow(1).is_err());
        let owner = AllocationLease::new(reserve.split(80));
        assert_eq!(pool.reserved(), 128);
        drop(reserve);
        let reader = owner.clone();
        drop(owner);
        assert_eq!(pool.reserved(), 80);
        std::thread::spawn(move || drop(reader)).join().unwrap();
        assert_eq!(pool.reserved(), 0);
    }
    #[test]
    fn cancellation_is_shared_and_child_does_not_cancel_parent() {
        let token = CancellationToken::new();
        let child = token.child_token();
        child.cancel();
        assert!(!token.is_cancelled());
        assert!(child.checkpoint().is_err());
        let clone = token.clone();
        token.cancel();
        assert!(clone.checkpoint().is_err());
    }
}
