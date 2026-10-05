// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Engineering consumers reuse registry declarations and existing numerical products.
use crate::generated::enums::{
    AccuracyEvidenceInterpretation, AccuracyEvidenceMethod, AccuracyGoalSubject, AccuracyGoalUse,
    AccuracyObservation, NumericalTarget,
};
use crate::strategy::{AccuracyClass, AccuracyEvidence, SemanticProductKey};
use pse_ids::{ContentHash, SemanticId};

/// The single generated meaning used by authored and request-local goals.
pub type AccuracyGoal = crate::generated::authored::accuracy_goals::Row;
/// The immutable completion/publication projection.
pub type GoalAssessment = crate::generated::runtime::accuracy_goal_assessments::Row;

/// Validate goal meaning before binding its physical target and observation.
/// # Errors
/// Missing obligations, malformed physical numbers or incompatible policy options.
pub fn validate_goal(goal: &AccuracyGoal) -> Result<(), crate::ModelError> {
    let criterion = goal.criterion_lower.is_some() || goal.criterion_upper.is_some();
    if goal.resolution.is_none() && !criterion
        || goal.resolution.is_some_and(|v| !v.is_finite() || v <= 0.0)
        || goal
            .criterion_lower
            .into_iter()
            .chain(goal.criterion_upper)
            .any(|v| !v.is_finite())
        || matches!((goal.criterion_lower, goal.criterion_upper), (Some(l), Some(u)) if l > u)
        || goal.required_class == AccuracyClass::Unresolved
        || goal.use_policy == AccuracyGoalUse::RequireSatisfied && !criterion
        || goal.provenance.trim().is_empty()
        || (goal.observation == AccuracyObservation::Sample) != goal.time.is_some()
        || goal.time.is_some_and(|v| !v.is_finite())
        || goal.subject == AccuracyGoalSubject::OptimalObjective
            && goal.target_kind != NumericalTarget::Objective
    {
        return Err(crate::malformed("invalid engineering accuracy goal"));
    }
    Ok(())
}

/// A physically converted goal frozen to the actual consumed mathematical dependencies.
#[derive(Clone, Debug, PartialEq)]
pub struct BoundGoal {
    /// Limits use point conversion; resolutions use magnitude conversion.
    pub declaration: AccuracyGoal,
    /// Exact binding, point, normalization and branch expected from the producer.
    pub source: SemanticProductKey,
    /// Protected output or optimum-value product, not a residual product.
    pub product: ContentHash,
    /// Physical output-error coordinates.
    pub normalization: ContentHash,
}

/// Actual physical output evidence supplied by a class-specific numerical owner.
#[derive(Clone, Debug, PartialEq)]
pub struct OutputEvidence {
    /// Original scalar target.
    pub target: SemanticId,
    /// Original target role.
    pub target_kind: NumericalTarget,
    /// Full physical quantity meaning.
    pub quantity: SemanticId,
    /// Unit of observation, interval and error.
    pub unit: SemanticId,
    /// Observation actually produced.
    pub observation: AccuracyObservation,
    /// Actual sample coordinate; endpoint meaning remains distinct.
    pub time: Option<f64>,
    /// Reported representative; criterion-only intervals may omit it.
    pub value: Option<f64>,
    /// Actual numerical error class and product coverage.
    pub accuracy: AccuracyEvidence,
    /// Actual source/binding/point/branch covered.
    pub source: SemanticProductKey,
    /// Producer-established validity region/context identity.
    pub validity: Option<ContentHash>,
    /// Meaning of the numbers, preventing residual/gap misinterpretation.
    pub interpretation: AccuracyEvidenceInterpretation,
    /// Operation that produced the evidence.
    pub method: AccuracyEvidenceMethod,
    /// Physical enclosure or explicitly estimated envelope, including asymmetric bounds.
    pub interval: Option<(f64, f64)>,
    /// Producer limitations remain visible at completion.
    pub limitation: String,
}
