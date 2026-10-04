// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Pure bounded numerical step adaptation. Scientific endpoints and overlays remain
//! with their consumers; this owner changes only the next numerical displacement.
use pse_backend_native::ProblemError;
use pse_model::generated::enums::NumericalAttemptObservation;

pub(crate) struct StepControl {
    step: f64,
    minimum: f64,
    maximum: f64,
    growth: f64,
    rejections: usize,
}

impl StepControl {
    pub(crate) fn new(
        initial: f64,
        minimum: f64,
        maximum: f64,
        growth: f64,
        rejections: usize,
    ) -> Result<Self, ProblemError> {
        if [initial, minimum, maximum, growth]
            .iter()
            .any(|value| !value.is_finite() || *value <= 0.)
            || minimum > initial
            || initial > maximum
            || growth < 1.
        {
            return Err(ProblemError::Contract(
                "invalid bounded numerical step control".into(),
            ));
        }
        Ok(Self {
            step: initial,
            minimum,
            maximum,
            growth,
            rejections,
        })
    }
    pub(crate) fn step(&self) -> f64 {
        self.step
    }
    /// A qualified step is the only event allowed to grow its successor. The caller
    /// supplies the remaining authored distance; reaching an endpoint needs no step.
    pub(crate) fn accepted(&mut self, remaining: f64) -> Result<(), ProblemError> {
        if !remaining.is_finite() || remaining < 0. {
            return Err(ProblemError::Contract(
                "invalid remaining path distance".into(),
            ));
        }
        self.step = (self.step * self.growth).min(self.maximum).min(remaining);
        Ok(())
    }
    /// A fixed-displacement tracker starts its next qualified segment at its
    /// declared displacement. This resets no rejection allowance or task clock.
    pub(crate) fn accepted_reset(&mut self, remaining: f64) -> Result<(), ProblemError> {
        if !remaining.is_finite() || remaining < 0. {
            return Err(ProblemError::Contract(
                "invalid remaining path distance".into(),
            ));
        }
        self.step = self.maximum.min(remaining);
        Ok(())
    }
    /// Numerical trajectory failure may consume one declared subdivision. A limited
    /// attempt without stagnation, contract failure or terminal latch cannot do so.
    pub(crate) fn rejected(&mut self, observation: NumericalAttemptObservation) -> bool {
        use NumericalAttemptObservation as O;
        if !matches!(observation, O::NumericalFailure | O::Stalled)
            || self.rejections == 0
            || self.step * 0.5 < self.minimum
        {
            return false;
        }
        self.rejections -= 1;
        self.step *= 0.5;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_numerical_failure_consumes_subdivision_and_only_acceptance_grows() {
        use NumericalAttemptObservation as O;
        let mut steps = StepControl::new(0.5, 0.125, 1., 2., 2).unwrap();
        for observation in [
            O::Cancelled,
            O::ResourceExhausted,
            O::ContractFailure,
            O::OperationalFailure,
            O::Limited,
            O::Panic,
        ] {
            assert!(!steps.rejected(observation));
            assert_eq!(steps.step(), 0.5);
        }
        assert!(steps.rejected(O::NumericalFailure));
        assert_eq!(steps.step(), 0.25);
        steps.accepted(0.3).unwrap();
        assert_eq!(steps.step(), 0.3);
        assert!(steps.rejected(O::Stalled));
        assert_eq!(steps.step(), 0.15);
        assert!(!steps.rejected(O::NumericalFailure));
        steps.accepted_reset(1.).unwrap();
        assert_eq!(steps.step(), 1.);
        assert!(
            !steps.rejected(O::NumericalFailure),
            "acceptance resets no rejection allowance"
        );
        steps.accepted(0.).unwrap();
        assert_eq!(steps.step(), 0.);
    }
    #[test]
    fn invalid_policy_or_remaining_distance_is_refused() {
        for initial in [0., f64::NAN, f64::INFINITY, 2.] {
            assert!(StepControl::new(initial, 0.1, 1., 1., 1).is_err());
        }
        let mut steps = StepControl::new(0.5, 0.1, 1., 1., 1).unwrap();
        for remaining in [-1., f64::NAN, f64::INFINITY] {
            assert!(steps.accepted(remaining).is_err());
            assert_eq!(steps.step(), 0.5);
        }
    }
}
