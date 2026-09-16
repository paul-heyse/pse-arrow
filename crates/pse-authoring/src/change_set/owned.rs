// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Immutable staging and candidate ownership with shared construction reservations.
use super::{CandidateSnapshot, ChangeSet};
use pse_ids::{Reservation, ReservationLease};
use std::{ops::Deref, sync::Arc};

/// Accounted immutable operation envelope; clones share all data and its lease.
#[derive(Clone, Debug)]
pub struct OwnedChangeSet(Arc<ChangeSet>);
impl OwnedChangeSet {
    pub(crate) fn new(mut value: ChangeSet, reservation: Box<dyn Reservation>) -> Self {
        value.leases.push(ReservationLease::new(reservation));
        Self(Arc::new(value))
    }
}
impl Deref for OwnedChangeSet {
    type Target = ChangeSet;
    fn deref(&self) -> &ChangeSet {
        &self.0
    }
}
#[derive(Debug)]
struct Candidate {
    value: CandidateSnapshot,
    checked: crate::document::Batches,
    _lease: Arc<ReservationLease>,
}
/// Accounted immutable candidate retaining metadata and its immutable operation construction.
/// Changed Arrow buffers retain their independent construction leases.
#[derive(Clone, Debug)]
pub struct OwnedCandidateSnapshot(Arc<Candidate>);
impl OwnedCandidateSnapshot {
    pub(crate) fn new(
        value: CandidateSnapshot,
        checked: crate::document::Batches,
        reservation: Box<dyn Reservation>,
    ) -> Self {
        Self(Arc::new(Candidate {
            value,
            checked,
            _lease: ReservationLease::new(reservation),
        }))
    }

    /// Actual field-checked source and native result owners retained by construction.
    /// This establishes local field obligations, not P2 or publication obligations.
    pub fn checked_relations(&self) -> &crate::document::Batches {
        &self.0.checked
    }
}

pub(super) const fn candidate_wrapper_extent() -> usize {
    // Two Arc allocations: Candidate and ReservationLease, each with strong and
    // weak counters. The Reservation implementation's control state is owned by
    // the reserver; its lease and all candidate-owned data are included here.
    size_of::<Candidate>() + size_of::<ReservationLease>() + 4 * size_of::<usize>()
}
impl Deref for OwnedCandidateSnapshot {
    type Target = CandidateSnapshot;
    fn deref(&self) -> &CandidateSnapshot {
        &self.0.value
    }
}
