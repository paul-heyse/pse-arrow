// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Accepted advancement permission for one complete trajectory, independent of native epochs.
use crate::{LimitKind, ProblemError};

#[derive(Clone, Debug)]
pub(super) struct StepBudget {
    limit: usize,
    used: usize,
}
impl StepBudget {
    pub(super) fn new(limit: usize) -> Self {
        Self { limit, used: 0 }
    }
    pub(super) fn remaining(&self) -> usize {
        self.limit - self.used
    }
    pub(super) fn used(&self) -> usize {
        self.used
    }
    pub(super) fn require_advance(&self, phase: &str) -> Result<(), ProblemError> {
        if self.remaining() == 0 {
            Err(Self::exhausted(phase))
        } else {
            Ok(())
        }
    }
    pub(super) fn charge(&mut self, accepted: usize, phase: &str) -> Result<(), ProblemError> {
        self.used = self
            .used
            .checked_add(accepted)
            .filter(|used| *used <= self.limit)
            .ok_or_else(|| {
                ProblemError::internal(format!("{phase} exceeded its admitted accepted-step cap"))
            })?;
        Ok(())
    }
    pub(super) fn exhausted(phase: &str) -> ProblemError {
        ProblemError::Limit {
            kind: LimitKind::Work,
            detail: format!("{phase} cumulative accepted-step allowance exhausted"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_trajectory_permission_survives_segment_and_counter_epochs() {
        let mut budget = StepBudget::new(3);
        for accepted in [1, 0, 2] {
            budget.charge(accepted, "forward").unwrap();
        }
        assert_eq!(budget.used(), 3);
        assert!(matches!(
            budget.require_advance("forward"),
            Err(ProblemError::Limit {
                kind: LimitKind::Work,
                ..
            })
        ));
        budget.charge(0, "covered output").unwrap();
        assert!(budget.charge(1, "forward").is_err());
    }
    #[test]
    fn separate_backward_directions_do_not_share_or_renew_permission() {
        let mut first = StepBudget::new(2);
        let mut second = StepBudget::new(2);
        first.charge(2, "direction 0").unwrap();
        second.charge(1, "direction 1").unwrap();
        assert_eq!((first.remaining(), second.remaining()), (0, 1));
    }
}
