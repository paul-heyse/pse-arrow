// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Pure bounded numerical step adaptation. Scientific endpoints and overlays remain
//! with their consumers; this owner changes only the next numerical displacement.
use pse_backend_native::ProblemError;
use pse_model::generated::enums::NumericalAttemptObservation;
use pse_model::strategy::{EffortPolicy, ProfileRef};

/// Counter supplied by the actual native getter, with its complete effective profile.
#[derive(Clone, Copy, Debug)]
pub(crate) struct EffortObservation {
    profile: ProfileRef,
    iterations: u64,
}
impl EffortObservation {
    pub(crate) fn from_report(report: &pse_backend_native::solve::SolveReport) -> Option<Self> {
        use pse_backend_native::solve::NativeCounters;
        let observed = report.evidence.native_observations.as_ref()?;
        let iterations = match &observed.counters {
            NativeCounters::Ipopt { iterations, .. }
            | NativeCounters::Uno { iterations, .. }
            | NativeCounters::Kinsol { iterations, .. } => (*iterations)?,
        };
        Some(Self {
            profile: ProfileRef {
                backend: observed.backend,
                key: observed.profile,
            },
            iterations,
        })
    }
}

pub(crate) struct StepControl {
    step: f64,
    minimum: f64,
    maximum: f64,
    growth: f64,
    rejections: usize,
    effort: Option<EffortPolicy>,
    effort_profile: Option<ProfileRef>,
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
            effort: None,
            effort_profile: None,
        })
    }
    pub(crate) fn with_effort(
        mut self,
        policy: Option<EffortPolicy>,
    ) -> Result<Self, ProblemError> {
        if policy.is_some_and(|policy| !policy.is_valid()) {
            return Err(ProblemError::Contract(
                "positive continuation effort target required".into(),
            ));
        }
        self.effort = policy;
        Ok(self)
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
    /// Only the scientific owner's qualified endpoint may call this. Missing, zero
    /// or foreign-profile counters retain the declared growth/reset rule. Adaptation
    /// never restores rejection allowance or changes an enclosing work/deadline owner.
    pub(crate) fn accepted_effort(
        &mut self,
        remaining: f64,
        reset: bool,
        observed: Option<EffortObservation>,
    ) -> Result<(), ProblemError> {
        let previous = self.step;
        if reset {
            self.accepted_reset(remaining)?;
        } else {
            self.accepted(remaining)?;
        }
        let (Some(policy), Some(observed)) = (self.effort, observed) else {
            return Ok(());
        };
        if observed.iterations == 0 {
            return Ok(());
        }
        let bound = self.effort_profile.get_or_insert(observed.profile);
        if *bound != observed.profile {
            return Ok(());
        }
        let factor = (f64::from(policy.target_iterations) / observed.iterations as f64)
            .clamp(0.5, if reset { 2. } else { self.growth });
        self.step = (previous * factor)
            .max(self.minimum)
            .min(self.maximum)
            .min(remaining);
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
    fn uno_effort_uses_only_the_actual_optional_native_iteration_getter() {
        use pse_backend_native::solve::{Assurance,Backend,Controls,Execution,NativeCounters,NativeObservations,NativeTermination,ObservationCoordinates,SolveReport,StopFidelity,Termination};
        use pse_ids::ContentHash;
        use std::sync::Arc;
        let contract=pse_backend_native::OracleContract {identity:ContentHash::from_bytes([71;32]),variables:vec![],rows:vec![],
            derivatives:pse_kernels::DerivativeOrder::Value,smoothness:pse_kernels::DerivativeOrder::Value};
        let execution=Execution::new(pse_diagnostics::Cancellation::default(),&Controls::default());
        let mut report=SolveReport::new(Backend::Uno,&contract,NativeTermination{code:-1,name:"controlled unavailable outcome".into(),message:None,category:Termination::Numerical,assurance:Assurance::None},&execution);
        for iterations in [None,Some(7)] {
            report.evidence.native_observations=Some(Arc::new(NativeObservations {backend:Backend::Uno,problem:contract.identity,profile:ContentHash::from_bytes([72;32]),
                coordinates:ObservationCoordinates::SubmittedOracle,stop_fidelity:StopFidelity::EvaluationAndIterationCheckpoints,current:None,
                counters:NativeCounters::Uno {iterations,objective_evaluations:None,constraint_evaluations:None,gradient_evaluations:None,jacobian_evaluations:None,subproblems:None,cpu_seconds:None}}));
            let observed=EffortObservation::from_report(&report);
            assert_eq!(observed.map(|value|value.iterations),iterations,"unavailable iterations must not be fabricated from callbacks or status");
            if let Some(observed)=observed {assert_eq!(observed.profile.backend,Backend::Uno);assert_eq!(observed.profile.key,ContentHash::from_bytes([72;32]));}
        }
    }
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
    #[test]
    fn effort_is_opt_in_profile_scoped_and_preserves_finite_allowance() {
        use pse_model::strategy::Backend;
        let profile = ProfileRef {
            backend: Backend::Ipopt,
            key: pse_ids::ContentHash::from_bytes([1; 32]),
        };
        let observation = |iterations| {
            Some(EffortObservation {
                profile,
                iterations,
            })
        };
        let mut steps = StepControl::new(0.4, 0.1, 1., 2., 1)
            .unwrap()
            .with_effort(Some(EffortPolicy {
                target_iterations: 4,
            }))
            .unwrap();
        steps.accepted_effort(1., false, observation(16)).unwrap();
        assert_eq!(steps.step(), 0.2);
        steps.accepted_effort(1., false, observation(1)).unwrap();
        assert_eq!(steps.step(), 0.4);
        steps.accepted_effort(1., false, observation(0)).unwrap();
        assert_eq!(steps.step(), 0.8, "zero falls back to declared growth");
        let foreign = Some(EffortObservation {
            profile: ProfileRef {
                key: pse_ids::ContentHash::from_bytes([2; 32]),
                ..profile
            },
            iterations: 100,
        });
        steps.accepted_effort(0.3, true, foreign).unwrap();
        assert_eq!(
            steps.step(),
            0.3,
            "foreign profile falls back to declared reset"
        );
        assert!(steps.rejected(NumericalAttemptObservation::Stalled));
        steps.accepted_effort(0.02, false, observation(4)).unwrap();
        assert_eq!(
            steps.step(),
            0.02,
            "remaining endpoint may be below minimum"
        );
        assert!(!steps.rejected(NumericalAttemptObservation::Stalled));
        let mut baseline = StepControl::new(0.4, 0.1, 1., 2., 1).unwrap();
        baseline
            .accepted_effort(1., false, observation(100))
            .unwrap();
        assert_eq!(baseline.step(), 0.8);
        assert!(
            StepControl::new(0.4, 0.1, 1., 2., 1)
                .unwrap()
                .with_effort(Some(EffortPolicy {
                    target_iterations: 0
                }))
                .is_err()
        );
    }
}
