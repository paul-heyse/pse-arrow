// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Immutable staging and candidate ownership; detached DTO APIs remain explicit.
use super::{CandidateSnapshot, ChangeSet};
use pse_ids::{Reservation, ReservationLease};
use std::{ops::Deref, sync::Arc};

#[derive(Debug)]
struct Changes {
    value: ChangeSet,
    _lease: Arc<ReservationLease>,
}
/// Accounted immutable operation envelope; clones share all data and its lease.
#[derive(Clone, Debug)]
pub struct OwnedChangeSet(Arc<Changes>);
impl OwnedChangeSet {
    pub(crate) fn new(value: ChangeSet, reservation: Box<dyn Reservation>) -> Self {
        Self(Arc::new(Changes {
            value,
            _lease: ReservationLease::new(reservation),
        }))
    }
}
impl Deref for OwnedChangeSet {
    type Target = ChangeSet;
    fn deref(&self) -> &ChangeSet {
        &self.0.value
    }
}
#[derive(Debug)]
struct Candidate {
    value: CandidateSnapshot,
    _lease: Arc<ReservationLease>,
}
/// Accounted immutable candidate retaining metadata and its complete operation proof.
/// Changed Arrow buffers retain their independent construction leases.
#[derive(Clone, Debug)]
pub struct OwnedCandidateSnapshot(Arc<Candidate>);
impl OwnedCandidateSnapshot {
    pub(crate) fn new(value: CandidateSnapshot, reservation: Box<dyn Reservation>) -> Self {
        Self(Arc::new(Candidate {
            value,
            _lease: ReservationLease::new(reservation),
        }))
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
