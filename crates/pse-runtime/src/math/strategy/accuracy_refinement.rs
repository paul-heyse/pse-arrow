// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Finite output-accuracy decisions from retained original evidence. Numerical
//! actions and their controls belong to the admitted producer, not this policy.
use pse_math::engineering_accuracy::{GoalResult, refinement_allowance};
use pse_model::{
    engineering_accuracy::BoundGoal,
    generated::enums::{
        AccuracyCriterionStatus, AccuracyGoalStatus, AccuracyGoalUse,
        AccuracyUnavailableReason as U,
    },
};

pub(crate) enum Decision {
    Finish,
    Stop(U),
    Refine,
}

/// Keep only the previous completed demand group. One completed non-improving
/// attempt stops; there is no probe ladder or tolerance-halving search.
#[derive(Default)]
pub(crate) struct Progress {
    previous: Vec<GoalResult>,
}
impl Progress {
    pub(crate) fn next(&mut self, original_satisfied: bool, goals: &[GoalResult]) -> Decision {
        if !original_satisfied || goals.is_empty() {
            return Decision::Finish;
        }
        // A required-satisfied decision stops at an established violation. Assess
        // may still request an independent unresolved value resolution.
        if goals.iter().any(|g| {
            g.goal.use_policy == AccuracyGoalUse::RequireSatisfied
                && g.classification.criterion == AccuracyCriterionStatus::Violated
        }) || goals
            .iter()
            .all(|g| g.classification.status != AccuracyGoalStatus::Unresolved)
        {
            return Decision::Finish;
        }
        let mut improved = self.previous.is_empty();
        for goal in goals {
            if goal.classification.status == AccuracyGoalStatus::Unresolved
                && let Some(reason) = goal.classification.unavailable
                && !matches!(reason, U::InsufficientAccuracy | U::Boundary)
            {
                return Decision::Stop(reason);
            }
            if let Some(previous) = self
                .previous
                .iter()
                .find(|g| g.goal.goal_id == goal.goal.goal_id)
            {
                let (Some(evidence), Some(old)) = (&goal.evidence, &previous.evidence) else {
                    return Decision::Stop(U::InvalidValidity);
                };
                let a = evidence.source;
                let b = old.source;
                if goal.goal != previous.goal
                    || a.structure != b.structure
                    || a.binding != b.binding
                    || a.numerical_policy != b.numerical_policy
                    || a.normalization != b.normalization
                    || a.parameters != b.parameters
                    || a.branch != b.branch
                    || a.derivation != b.derivation
                {
                    return Decision::Stop(U::InvalidValidity);
                }
                improved |= previous.classification.status == AccuracyGoalStatus::Unresolved
                    && goal.classification.status != AccuracyGoalStatus::Unresolved;
                use pse_model::strategy::AccuracyClass as C;
                improved |= matches!(
                    (old.accuracy.class, evidence.accuracy.class),
                    (C::Estimated, C::Certified) | (C::Unresolved, C::Estimated | C::Certified)
                );
            } else if !self.previous.is_empty() {
                return Decision::Stop(U::InvalidValidity);
            }
            if goal.classification.status != AccuracyGoalStatus::Unresolved {
                continue;
            }
            if !goal.goal.refine {
                return Decision::Stop(U::RefinementDisallowed);
            }
            if let Some(reason) = goal.classification.unavailable
                && !matches!(reason, U::InsufficientAccuracy | U::Boundary)
            {
                return Decision::Stop(reason);
            }
            let Some(evidence) = &goal.evidence else {
                return Decision::Stop(U::MissingEvidence);
            };
            let Some(value) = evidence.value else {
                return Decision::Stop(U::MissingObservation);
            };
            let bound = BoundGoal {
                declaration: goal.goal.clone(),
                source: evidence.source,
                product: evidence.accuracy.product,
                normalization: evidence.accuracy.normalization,
            };
            if refinement_allowance(&bound, value).is_none() {
                return Decision::Stop(U::Boundary);
            }
            let Some(error) = goal
                .classification
                .error
                .filter(|e| e.is_finite() && *e > 0.0)
            else {
                return Decision::Stop(U::PrecisionLimit);
            };
            if let Some(previous) = self
                .previous
                .iter()
                .find(|g| g.goal.goal_id == goal.goal.goal_id)
            {
                // The point, product and operational precision may change; the
                // frozen acceptance and scientific dependencies were checked above.
                let Some(old_error) = previous.classification.error else {
                    return Decision::Stop(U::MissingEvidence);
                };
                if error > old_error {
                    return Decision::Stop(U::Nonprogress);
                }
                improved |= error < old_error;
            }
        }
        if !improved {
            return Decision::Stop(U::Nonprogress);
        }
        self.previous = goals.to_vec();
        Decision::Refine
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_ids::{ContentHash, SemanticId};
    use pse_model::{
        engineering_accuracy::{AccuracyGoal, OutputEvidence},
        generated::enums::{
            AccuracyEvidenceInterpretation, AccuracyEvidenceMethod, AccuracyGoalSubject,
            AccuracyObservation, AccuracyResolutionStatus, NumericalSource, NumericalTarget,
        },
        strategy::{AccuracyClass, AccuracyEvidence, SemanticProductKey},
    };
    fn hash(n: u8) -> ContentHash {
        ContentHash::from_bytes([n; 32])
    }
    fn id(n: u8) -> SemanticId {
        SemanticId::from_bytes([n; 16])
    }
    fn observed(value: f64, error: f64) -> GoalResult {
        let goal = BoundGoal {
            declaration: AccuracyGoal {
                goal_id: id(1).into(),
                model_id: None,
                case_id: None,
                instance_id: None,
                fit_id: None,
                target_id: id(2),
                target_kind: NumericalTarget::Variable,
                quantity_id: id(3),
                unit_id: id(4),
                subject: AccuracyGoalSubject::SelectedOutput,
                observation: AccuracyObservation::Steady,
                time: None,
                resolution: Some(0.1),
                criterion_lower: None,
                criterion_upper: None,
                required_class: AccuracyClass::Estimated,
                use_policy: AccuracyGoalUse::Assess,
                refine: true,
                source: NumericalSource::Analysis,
                priority: 0,
                provenance: "finite original-square accuracy control".into(),
            },
            source: SemanticProductKey {
                structure: hash(1),
                binding: hash(2),
                numerical_policy: Some(hash(3)),
                normalization: Some(hash(4)),
                point: Some(hash(5)),
                parameters: Some(hash(6)),
                derivation: None,
                branch: Some(hash(7)),
                accuracy: Some(hash(8)),
            },
            product: hash(9),
            normalization: hash(10),
        };
        let evidence = OutputEvidence {
            target: goal.declaration.target_id,
            target_kind: goal.declaration.target_kind,
            quantity: goal.declaration.quantity_id,
            unit: goal.declaration.unit_id,
            observation: goal.declaration.observation,
            time: None,
            value: Some(value),
            accuracy: AccuracyEvidence {
                product: goal.product,
                normalization: goal.normalization,
                error: Some(error),
                class: AccuracyClass::Estimated,
            },
            source: goal.source,
            validity: Some(hash(11)),
            interpretation: AccuracyEvidenceInterpretation::OutputError,
            method: AccuracyEvidenceMethod::SquareCorrection,
            interval: None,
            limitation: "local original-square estimate".into(),
        };
        GoalResult::assess(&goal, Some(evidence))
    }

    #[test]
    fn engineering_refinement_stops_after_one_completed_nonprogress_and_never_retries_missing_evidence()
     {
        let initial = observed(20.0, 0.5);
        let mut progress = Progress::default();
        assert!(matches!(
            progress.next(true, std::slice::from_ref(&initial)),
            Decision::Refine
        ));
        assert!(matches!(
            progress.next(true, std::slice::from_ref(&initial)),
            Decision::Stop(U::Nonprogress)
        ));
        let missing = GoalResult::unavailable(initial.goal, U::MissingEvidence);
        assert!(matches!(
            Progress::default().next(true, &[missing]),
            Decision::Stop(U::MissingEvidence)
        ));
    }

    #[test]
    fn engineering_refinement_consumes_only_unresolved_permitted_original_output_demands() {
        let mut initial = observed(20.0, 0.5);
        assert!(matches!(
            Progress::default().next(false, &[initial.clone()]),
            Decision::Finish
        ));
        initial.goal.refine = false;
        assert!(matches!(
            Progress::default().next(true, &[initial]),
            Decision::Stop(U::RefinementDisallowed)
        ));
        let resolved = observed(20.0, 0.05);
        assert_eq!(
            resolved.classification.resolution,
            AccuracyResolutionStatus::Met
        );
        assert!(matches!(
            Progress::default().next(true, &[resolved]),
            Decision::Finish
        ));
    }

    #[test]
    fn engineering_refinement_requires_improvement_at_frozen_contract_and_branch() {
        let mut progress = Progress::default();
        assert!(matches!(
            progress.next(true, &[observed(20.0, 0.5)]),
            Decision::Refine
        ));
        let mut improved = observed(20.1, 0.2);
        improved.evidence.as_mut().unwrap().source.point = Some(hash(20));
        improved.evidence.as_mut().unwrap().source.accuracy = Some(hash(22));
        assert!(matches!(
            progress.next(true, &[improved.clone()]),
            Decision::Refine
        ));
        improved.evidence.as_mut().unwrap().source.branch = Some(hash(21));
        improved.classification.error = Some(0.15);
        assert!(matches!(
            progress.next(true, &[improved]),
            Decision::Stop(U::InvalidValidity)
        ));
    }

    #[test]
    fn engineering_refinement_counts_a_newly_resolved_goal_as_shared_progress() {
        let initial = observed(20.0, 0.5);
        let mut other = initial.clone();
        other.goal.goal_id = id(20).into();
        let mut progress = Progress::default();
        assert!(matches!(
            progress.next(true, &[initial, other.clone()]),
            Decision::Refine
        ));
        let resolved = observed(20.0, 0.05);
        assert!(matches!(
            progress.next(true, &[resolved.clone(), other.clone()]),
            Decision::Refine
        ));
        assert!(matches!(
            progress.next(true, &[resolved, other]),
            Decision::Stop(U::Nonprogress)
        ));
    }

    #[test]
    fn engineering_refinement_stops_exact_boundary_and_required_violation_without_tightening() {
        let mut boundary = observed(20.0, 0.5);
        boundary.goal.criterion_upper = Some(20.0);
        assert!(matches!(
            Progress::default().next(true, &[boundary]),
            Decision::Stop(U::Boundary)
        ));
        let mut violation = observed(20.0, 0.5);
        violation.goal.criterion_upper = Some(19.0);
        violation.classification.criterion = AccuracyCriterionStatus::Violated;
        assert!(matches!(
            Progress::default().next(true, &[violation.clone()]),
            Decision::Refine
        ));
        violation.goal.use_policy = AccuracyGoalUse::RequireSatisfied;
        assert!(matches!(
            Progress::default().next(true, &[violation]),
            Decision::Finish
        ));
    }
}
