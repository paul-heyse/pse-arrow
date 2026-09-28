// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The attempt lifecycle: one pure transition table, the only authority for legality
//! (ADR-0114 Outcome 12, finding T13). Repository functions apply it inside a transaction
//! under a row lock; SQL enforces only the value domain. The state spellings are the
//! registry enumeration `AttemptState` (DP-19).

use pse_model::generated::enums::AttemptKind;
pub use pse_model::generated::enums::AttemptState;

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

/// Every legal `(from, to)` pair of a coordinating attempt (a study, Plan 22 O7), which
/// replaces [`TRANSITIONS`] for that kind.
///
/// A coordinating attempt holds no lease and does no work of its own: the attempts it
/// coordinates do. It is queued while they run and ends from queued, in the transaction
/// that ends the last of them; it may be cancelled before that. It is never running, so it
/// never goes stale, and a publication naming it can always commit once it has ended.
pub const COORDINATING: &[(AttemptState, AttemptState)] = &[
    (Planned, Queued),
    (Planned, Cancelled),
    (Queued, Completed),
    (Queued, Partial),
    (Queued, Failed),
    (Queued, Cancelled),
];

/// Whether attempts of `kind` coordinate other attempts ([`COORDINATING`]).
pub const fn coordinates(kind: AttemptKind) -> bool {
    matches!(kind, AttemptKind::Study)
}

/// The transition table of an attempt of `kind`.
pub const fn table(kind: AttemptKind) -> &'static [(AttemptState, AttemptState)] {
    if coordinates(kind) {
        COORDINATING
    } else {
        TRANSITIONS
    }
}

/// Whether an attempt of `kind` may change from `from` to `to`.
pub fn legal(kind: AttemptKind, from: AttemptState, to: AttemptState) -> bool {
    table(kind).contains(&(from, to))
}

/// The state every attempt is created in.
pub const INITIAL: AttemptState = Planned;

/// Lifecycle rules of an attempt state, read from [`TRANSITIONS`].
pub trait Lifecycle: Copy {
    /// Whether `to` may follow `self`.
    fn may_become(self, to: Self) -> bool;
    /// The legal successors of this state.
    fn successors(self) -> impl Iterator<Item = Self>;
    /// No transition leaves this state.
    fn is_final(self) -> bool;
    /// The attempt stopped doing work: its finish time is recorded on entry.
    fn ends_work(self) -> bool;
}

impl Lifecycle for AttemptState {
    fn may_become(self, to: Self) -> bool {
        TRANSITIONS.contains(&(self, to))
    }

    fn successors(self) -> impl Iterator<Item = Self> {
        TRANSITIONS
            .iter()
            .filter(move |(from, _)| *from == self)
            .map(|&(_, to)| to)
    }

    fn is_final(self) -> bool {
        self.successors().next().is_none()
    }

    fn ends_work(self) -> bool {
        matches!(self, Completed | Partial | Failed | Cancelled | Stale)
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
            assert!(
                !from.may_become(to),
                "{} -> {} must be illegal",
                from.as_str(),
                to.as_str()
            );
        }
    }

    #[test]
    fn finished_states_have_no_successors() {
        for state in [Completed, Partial, Failed, Cancelled, Superseded] {
            assert!(state.is_final(), "{}", state.as_str());
        }
        for state in [Planned, Queued, Running, Stale] {
            assert!(!state.is_final(), "{}", state.as_str());
        }
    }

    #[test]
    fn every_state_is_reachable_from_the_initial_state() {
        let mut reached = vec![INITIAL];
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
        let mut all = AttemptState::ALL.to_vec();
        all.sort();
        assert_eq!(reached, all);
    }

    #[test]
    fn coordinating_attempts_end_from_queued_and_never_run() {
        for kind in AttemptKind::ALL {
            let coordinating = coordinates(kind);
            assert_eq!(
                coordinating,
                kind == AttemptKind::Study,
                "{}",
                kind.as_str()
            );
            for (from, to) in [(Queued, Completed), (Queued, Partial), (Queued, Failed)] {
                assert_eq!(legal(kind, from, to), coordinating);
            }
            for (from, to) in [(Queued, Running), (Running, Stale)] {
                assert_eq!(legal(kind, from, to), !coordinating);
            }
            // Both tables admit creation, release and cancellation before work.
            for (from, to) in [(Planned, Queued), (Planned, Cancelled), (Queued, Cancelled)] {
                assert!(legal(kind, from, to));
            }
        }
        // A coordinating attempt's finished states are final too.
        for state in [Completed, Partial, Failed, Cancelled] {
            assert!(!COORDINATING.iter().any(|(from, _)| *from == state));
        }
    }

    #[test]
    fn spelling_round_trips_and_rejects_unknown_text() {
        for state in AttemptState::ALL {
            assert_eq!(state.as_str().parse::<AttemptState>().ok(), Some(state));
        }
        assert!("Running".parse::<AttemptState>().is_err());
    }
}
