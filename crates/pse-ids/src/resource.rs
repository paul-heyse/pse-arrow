// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The accounted-allocation interface (blueprint §14.3, ADR-0046).
//!
//! Canonicalization, binding, transfer and result buffers are platform allocations that
//! DataFusion's query pool never sees. ADR-0046 requires them to **reserve before
//! allocating** against the same shared budget, so that two snapshot sessions in one
//! process cannot each consume a whole deployment budget.
//!
//! `pse-ids` declares the interface and nothing else, because it is the lowest crate in
//! the graph and must stay free of `datafusion`. `pse-runtime` implements it over a
//! `FairSpillPool` behind `TrackConsumersPool` and `PeakRecordingPool`; [`FixedBudget`]
//! here is the reference implementation that tests and benchmarks use, and is not the
//! deployment budget.
//!
//! What the interface does **not** promise (ADR-0046, and the blueprint says so in the
//! same breath): protection from the global allocator, a solver process, or any
//! unregistered allocation. Arrow's own tracking pools have infallible reserve and cannot
//! reject; the guarantee here covers accounted consumers only.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crate::error::{CanonError, ReserveError};

/// A budget that hands out reservations.
///
/// `Send + Sync` because one budget is shared across sessions and threads; `Debug` because
/// a resource report names its budget.
pub trait MemoryReserver: Send + Sync + std::fmt::Debug {
    /// Opens a reservation attributed to `owner`.
    ///
    /// `owner` is what a `runtime::resource_limit` names, so it should identify the
    /// consumer a reader can act on — `canonicalize:authored.stoichiometry@1`, not
    /// `buffer`.
    fn open(&self, owner: &str) -> Box<dyn Reservation>;
}

/// One consumer's claim on a [`MemoryReserver`].
///
/// `Send` but not `Sync`: a reservation is grown and shrunk by its owner, and shared
/// immutable buffers have exactly one reservation owner for their lifetime so that a
/// reader does not double-charge them (§14.3).
///
/// Dropping a reservation releases everything it holds, so an early return on an error or
/// a cancellation cannot leak budget.
pub trait Reservation: Send + std::fmt::Debug {
    /// Reserves `bytes` more, **before** the allocation they account for.
    ///
    /// # Errors
    ///
    /// [`ReserveError::Exhausted`] when the budget cannot cover the request; the caller
    /// must not then allocate.
    fn try_grow(&mut self, bytes: usize) -> Result<(), ReserveError>;

    /// Returns `bytes` to the budget, saturating at the amount actually held.
    fn shrink(&mut self, bytes: usize);

    /// What this reservation currently holds.
    fn size(&self) -> usize;

    /// Returns everything to the budget, leaving the reservation open at zero.
    fn release(&mut self);
}

/// The shared state of a [`FixedBudget`], which outlives the handle it was opened from.
#[derive(Debug)]
struct BudgetState {
    limit_bytes: usize,
    reserved: AtomicUsize,
}

impl BudgetState {
    /// Compare-and-swap loop: a grow either fits within the limit or changes nothing.
    fn try_grow(&self, owner: &str, bytes: usize) -> Result<(), ReserveError> {
        let mut current = self.reserved.load(Ordering::Acquire);
        loop {
            let refused = || ReserveError::Exhausted {
                owner: owner.to_owned(),
                requested: bytes,
                reserved: current,
                limit_hint: format!("FixedBudget limit_bytes={}", self.limit_bytes),
            };
            let Some(next) = current.checked_add(bytes) else {
                return Err(refused());
            };
            if next > self.limit_bytes {
                return Err(refused());
            }
            match self.reserved.compare_exchange_weak(
                current,
                next,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return Ok(()),
                Err(observed) => current = observed,
            }
        }
    }

    /// Returns `bytes` to the budget.
    fn give_back(&self, bytes: usize) {
        if bytes > 0 {
            self.reserved.fetch_sub(bytes, Ordering::AcqRel);
        }
    }
}

/// A fixed-limit, thread-safe [`MemoryReserver`] for tests and benchmarks.
///
/// Deliberately not a deployment budget: it has no spill manager, no consumer attribution
/// beyond the owner string and no peak recording. It exists so that a canonicalization
/// test can assert "this fails *before* it allocates" without pulling DataFusion into
/// `pse-ids`.
///
/// ```
/// use pse_ids::{FixedBudget, MemoryReserver};
///
/// let budget = FixedBudget::new(1024);
/// let mut reservation = budget.open("canonicalize:demo");
/// assert!(reservation.try_grow(512).is_ok());
/// assert_eq!(budget.reserved(), 512);
///
/// // A refused grow changes nothing.
/// assert!(reservation.try_grow(1024).is_err());
/// assert_eq!(budget.reserved(), 512);
///
/// drop(reservation);
/// assert_eq!(budget.reserved(), 0);
/// ```
#[derive(Debug)]
pub struct FixedBudget {
    state: Arc<BudgetState>,
}

impl FixedBudget {
    /// Opens a budget of `limit_bytes`.
    ///
    /// Returned behind an `Arc` because a budget is shared by construction; a per-consumer
    /// budget is the mistake ADR-0046 exists to prevent.
    pub fn new(limit_bytes: usize) -> Arc<Self> {
        Arc::new(Self {
            state: Arc::new(BudgetState {
                limit_bytes,
                reserved: AtomicUsize::new(0),
            }),
        })
    }

    /// The limit this budget was opened with.
    pub fn limit_bytes(&self) -> usize {
        self.state.limit_bytes
    }

    /// Bytes reserved across every reservation this budget has open.
    pub fn reserved(&self) -> usize {
        self.state.reserved.load(Ordering::Acquire)
    }
}

impl MemoryReserver for FixedBudget {
    fn open(&self, owner: &str) -> Box<dyn Reservation> {
        Box::new(FixedReservation {
            owner: owner.to_owned(),
            state: Arc::clone(&self.state),
            size: 0,
        })
    }
}

/// One consumer's claim on a [`FixedBudget`].
#[derive(Debug)]
struct FixedReservation {
    owner: String,
    state: Arc<BudgetState>,
    size: usize,
}

impl Reservation for FixedReservation {
    fn try_grow(&mut self, bytes: usize) -> Result<(), ReserveError> {
        self.state.try_grow(&self.owner, bytes)?;
        self.size = self.size.saturating_add(bytes);
        Ok(())
    }

    fn shrink(&mut self, bytes: usize) {
        let given = bytes.min(self.size);
        self.size -= given;
        self.state.give_back(given);
    }

    fn size(&self) -> usize {
        self.size
    }

    fn release(&mut self) {
        let held = self.size;
        self.size = 0;
        self.state.give_back(held);
    }
}

impl Drop for FixedReservation {
    fn drop(&mut self) {
        self.release();
    }
}

/// Cooperative cancellation for the long loops of canonicalization and compilation.
///
/// Cloning shares the flag, so a driver holds one token and hands clones to the work it
/// started. `Default` is an uncancelled token.
///
/// ```
/// use pse_ids::CancellationToken;
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
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    /// Opens an uncancelled token.
    pub fn new() -> Self {
        Self::default()
    }

    /// Cancels this token and every clone of it. Cancellation is one-way.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    /// Whether the token has been cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
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

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn a_reservation_grows_shrinks_and_releases() {
        let budget = FixedBudget::new(1_000);
        let mut reservation = budget.open("canonicalize:demo");
        assert_eq!(budget.limit_bytes(), 1_000);

        assert!(reservation.try_grow(400).is_ok());
        assert_eq!(reservation.size(), 400);
        assert_eq!(budget.reserved(), 400);

        reservation.shrink(100);
        assert_eq!(reservation.size(), 300);
        assert_eq!(budget.reserved(), 300);

        // Shrinking past zero returns what is held, not more.
        reservation.shrink(10_000);
        assert_eq!(reservation.size(), 0);
        assert_eq!(budget.reserved(), 0);

        assert!(reservation.try_grow(50).is_ok());
        reservation.release();
        assert_eq!(reservation.size(), 0);
        assert_eq!(budget.reserved(), 0);
    }

    #[test]
    fn a_refused_grow_names_the_owner_and_the_configuration() {
        let budget = FixedBudget::new(64);
        let mut reservation = budget.open("canonicalize:authored.stoichiometry@1");
        let refused = reservation.try_grow(65);
        match refused {
            Err(ReserveError::Exhausted {
                owner,
                requested,
                reserved,
                limit_hint,
            }) => {
                assert_eq!(owner, "canonicalize:authored.stoichiometry@1");
                assert_eq!(requested, 65);
                assert_eq!(reserved, 0);
                assert!(limit_hint.contains("limit_bytes=64"), "{limit_hint}");
            }
            Ok(()) => panic!("a 65-byte grow fit into a 64-byte budget"),
        }
        // Refusal is not partial: nothing was taken.
        assert_eq!(budget.reserved(), 0);
        assert_eq!(reservation.size(), 0);
    }

    #[test]
    fn an_overflowing_grow_is_refused_rather_than_wrapped() {
        let budget = FixedBudget::new(usize::MAX);
        let mut reservation = budget.open("overflow");
        assert!(reservation.try_grow(usize::MAX).is_ok());
        assert!(reservation.try_grow(1).is_err());
        assert_eq!(budget.reserved(), usize::MAX);
    }

    #[test]
    fn dropping_a_reservation_returns_its_bytes() {
        let budget = FixedBudget::new(1_000);
        {
            let mut reservation = budget.open("temporary");
            assert!(reservation.try_grow(999).is_ok());
            assert_eq!(budget.reserved(), 999);
        }
        assert_eq!(budget.reserved(), 0);
    }

    #[test]
    fn reservations_share_one_budget() {
        let budget = FixedBudget::new(100);
        let mut first = budget.open("first");
        let mut second = budget.open("second");
        assert!(first.try_grow(60).is_ok());
        assert!(second.try_grow(60).is_err());
        assert!(second.try_grow(40).is_ok());
        assert_eq!(budget.reserved(), 100);
    }

    #[test]
    fn concurrent_consumers_never_exceed_the_limit() {
        const THREADS: usize = 8;
        const ROUNDS: usize = 200;
        const CHUNK: usize = 16;
        let limit = THREADS * CHUNK * 4;

        let budget = FixedBudget::new(limit);
        let observed_peak = Arc::new(AtomicUsize::new(0));

        std::thread::scope(|scope| {
            for worker in 0..THREADS {
                let budget = Arc::clone(&budget);
                let observed_peak = Arc::clone(&observed_peak);
                scope.spawn(move || {
                    let mut reservation = budget.open(&format!("worker-{worker}"));
                    for _ in 0..ROUNDS {
                        if reservation.try_grow(CHUNK).is_ok() {
                            observed_peak.fetch_max(budget.reserved(), Ordering::AcqRel);
                            reservation.shrink(CHUNK);
                        }
                    }
                });
            }
        });

        assert_eq!(budget.reserved(), 0, "every worker released on drop");
        assert!(
            observed_peak.load(Ordering::Acquire) <= limit,
            "a concurrent peak of {} exceeded the {limit}-byte limit",
            observed_peak.load(Ordering::Acquire)
        );
    }

    #[test]
    fn a_cancellation_token_is_shared_and_one_way() {
        let token = CancellationToken::new();
        let clone = token.clone();
        assert!(!token.is_cancelled());
        assert!(token.checkpoint().is_ok());

        clone.cancel();
        assert!(token.is_cancelled());
        assert!(matches!(token.checkpoint(), Err(CanonError::Cancelled)));
        // Still cancelled after a second look; there is no uncancel.
        assert!(token.is_cancelled());
    }

    proptest! {
        /// Whatever sequence of grows and shrinks a consumer performs, the budget never
        /// reports more than its limit and a released reservation reports zero.
        #[test]
        fn a_budget_never_exceeds_its_limit(
            limit in 0_usize..4096,
            steps in proptest::collection::vec((any::<bool>(), 0_usize..512), 0..64),
        ) {
            let budget = FixedBudget::new(limit);
            let mut reservation = budget.open("property");
            for (grow, bytes) in steps {
                if grow {
                    let before = reservation.size();
                    if reservation.try_grow(bytes).is_err() {
                        prop_assert_eq!(reservation.size(), before);
                    }
                } else {
                    reservation.shrink(bytes);
                }
                prop_assert!(budget.reserved() <= limit);
                prop_assert_eq!(budget.reserved(), reservation.size());
            }
            reservation.release();
            prop_assert_eq!(budget.reserved(), 0);
        }

        /// A reservation dropped mid-flight returns exactly what it held.
        #[test]
        fn drop_releases_whatever_was_held(
            limit in 1_usize..4096,
            bytes in 0_usize..4096,
        ) {
            let budget = FixedBudget::new(limit);
            {
                let mut reservation = budget.open("dropped");
                let _ = reservation.try_grow(bytes);
            }
            prop_assert_eq!(budget.reserved(), 0);
        }
    }
}
