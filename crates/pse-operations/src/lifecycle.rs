// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The attempt lifecycle: one pure transition table, the only authority for legality
//! (ADR-0112 Outcome 12, finding T13). Repository functions apply it inside a transaction
//! under a row lock; SQL enforces only the value domain.

use std::fmt;
use std::str::FromStr;

/// The durable lifecycle state of an attempt (DP-19).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AttemptState {
    /// Registered; not yet queued.
    Planned,
    /// Waiting for a worker.
    Queued,
    /// Owned by a worker under a lease.
    Running,
    /// Finished with a complete result.
    Completed,
    /// Finished with a usable but incomplete result.
    Partial,
    /// Finished without a usable result.
    Failed,
    /// Stopped on request.
    Cancelled,
    /// The running worker's lease expired; the attempt's outcome is unknown.
    Stale,
    /// A stale attempt replaced by a new attempt.
    Superseded,
}

use AttemptState::{
    Cancelled, Completed, Failed, Partial, Planned, Queued, Running, Stale, Superseded,
};

/// Every legal `(from, to)` pair. Nothing else may change an attempt's state.
///
/// planned -> queued -> running -> {completed, partial, failed, cancelled}; running -> stale
/// on lease expiry; stale -> superseded when a new attempt replaces it. Work that has not
/// started may be cancelled while planned or queued.
pub const TRANSITIONS: &[(AttemptState, AttemptState)] = &[
    (Planned, Queued),
    (Planned, Cancelled),
    (Queued, Running),
    (Queued, Cancelled),
    (Running, Completed),
    (Running, Partial),
    (Running, Failed),
    (Running, Cancelled),
    (Running, Stale),
    (Stale, Superseded),
];

impl AttemptState {
    /// Every state, in lifecycle order.
    pub const ALL: [Self; 9] = [
        Planned, Queued, Running, Completed, Partial, Failed, Cancelled, Stale, Superseded,
    ];

    /// The state every attempt is created in.
    pub const INITIAL: Self = Planned;

    /// The stored spelling (registry enumeration value).
    pub const fn as_str(self) -> &'static str {
        match self {
            Planned => "planned",
            Queued => "queued",
            Running => "running",
            Completed => "completed",
            Partial => "partial",
            Failed => "failed",
            Cancelled => "cancelled",
            Stale => "stale",
            Superseded => "superseded",
        }
    }

    /// Whether `to` may follow `self`, according to [`TRANSITIONS`].
    pub fn may_become(self, to: Self) -> bool {
        TRANSITIONS.contains(&(self, to))
    }

    /// The legal successors of this state.
    pub fn successors(self) -> impl Iterator<Item = Self> {
        TRANSITIONS
            .iter()
            .filter(move |(from, _)| *from == self)
            .map(|&(_, to)| to)
    }

    /// No transition leaves this state.
    pub fn is_final(self) -> bool {
        self.successors().next().is_none()
    }

    /// The attempt stopped doing work: its finish time is recorded on entry.
    pub const fn ends_work(self) -> bool {
        matches!(self, Completed | Partial | Failed | Cancelled | Stale)
    }
}

impl fmt::Display for AttemptState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The stored text is not an attempt state.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("unknown attempt state `{0}`")]
pub struct UnknownState(pub String);

impl FromStr for AttemptState {
    type Err = UnknownState;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|state| state.as_str() == text)
            .ok_or_else(|| UnknownState(text.to_owned()))
    }
}

#[cfg(test)]
mod transition_unit {
    use super::*;

    #[test]
    fn table_admits_exactly_the_documented_lifecycle() {
        let legal: Vec<(AttemptState, AttemptState)> = AttemptState::ALL
            .into_iter()
            .flat_map(|from| AttemptState::ALL.into_iter().map(move |to| (from, to)))
            .filter(|&(from, to)| from.may_become(to))
            .collect();
        assert_eq!(legal, TRANSITIONS);
        assert_eq!(legal.len(), 10);
    }

    #[test]
    fn illegal_pairs_are_refused() {
        for (from, to) in [
            (Planned, Running),
            (Queued, Completed),
            (Completed, Running),
            (Failed, Queued),
            (Stale, Running),
            (Stale, Queued),
            (Superseded, Stale),
            (Cancelled, Running),
            (Running, Running),
            (Running, Queued),
        ] {
            assert!(!from.may_become(to), "{from} -> {to} must be illegal");
        }
    }

    #[test]
    fn finished_states_have_no_successors() {
        for state in [Completed, Partial, Failed, Cancelled, Superseded] {
            assert!(state.is_final(), "{state}");
        }
        for state in [Planned, Queued, Running, Stale] {
            assert!(!state.is_final(), "{state}");
        }
    }

    #[test]
    fn every_state_is_reachable_from_the_initial_state() {
        let mut reached = vec![AttemptState::INITIAL];
        let mut index = 0;
        while let Some(&state) = reached.get(index) {
            for next in state.successors() {
                if !reached.contains(&next) {
                    reached.push(next);
                }
            }
            index += 1;
        }
        reached.sort();
        assert_eq!(reached, AttemptState::ALL);
    }

    #[test]
    fn spelling_round_trips_and_rejects_unknown_text() {
        for state in AttemptState::ALL {
            assert_eq!(state.as_str().parse::<AttemptState>(), Ok(state));
        }
        assert_eq!(
            "Running".parse::<AttemptState>(),
            Err(UnknownState("Running".to_owned()))
        );
    }
}
