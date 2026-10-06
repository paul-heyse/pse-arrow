// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Output evidence from actual qualified objective bounds and paired observations.
use pse_ids::ContentHash;
use pse_model::{
    engineering_accuracy::{BoundGoal, OutputEvidence},
    generated::enums::{
        AccuracyEvidenceInterpretation as I, AccuracyEvidenceMethod as M, AccuracyGoalSubject as S,
        AccuracyObservation as O, AccuracyUnavailableReason as U,
    },
    strategy::{AccuracyClass, AccuracyEvidence, SemanticProductKey},
};

/// A separately admitted original-problem scalar. Native termination supplies none
/// of its source, strength, validity or material evaluation uncertainty.
#[derive(Clone, Copy, Debug)]
pub struct QualifiedScalar {
    /// Actual consumed scientific dependencies.
    pub source: SemanticProductKey,
    /// Actual value in the goal's physical unit.
    pub value: f64,
    /// Separately admitted material error in that unit.
    pub uncertainty: Option<f64>,
    /// Strength actually established by the scalar producer.
    pub class: AccuracyClass,
    /// Producer-issued original feasibility/bound validity receipt.
    pub validity: ContentHash,
}
fn scalar(goal: &BoundGoal, actual: QualifiedScalar) -> Result<f64, U> {
    if actual.source != goal.source {
        return Err(U::InvalidValidity);
    }
    if actual.class == AccuracyClass::Unresolved {
        return Err(U::InsufficientStrength);
    }
    let uncertainty = actual.uncertainty.ok_or(U::EvaluatorUncertainty)?;
    if !actual.value.is_finite() || !uncertainty.is_finite() || uncertainty < 0. {
        return Err(U::Nonfinite);
    }
    Ok(uncertainty)
}
fn evidence(
    goal: &BoundGoal,
    (value, error, class): (f64, f64, AccuracyClass),
    validity: ContentHash,
    interpretation: I,
    method: M,
    interval: Option<(f64, f64)>,
    limitation: &str,
) -> OutputEvidence {
    let g = &goal.declaration;
    OutputEvidence {
        target: g.target_id,
        target_kind: g.target_kind,
        quantity: g.quantity_id,
        unit: g.unit_id,
        observation: g.observation,
        time: g.time,
        value: Some(value),
        accuracy: AccuracyEvidence {
            product: goal.product,
            normalization: goal.normalization,
            class,
            error: Some(error),
        },
        source: goal.source,
        validity: Some(validity),
        interpretation,
        method,
        interval,
        limitation: limitation.into(),
    }
}
/// A qualified original primal/dual interval protects only the optimal objective.
/// The representative remains the actual primal objective, so its radius is asymmetric.
pub fn objective_interval(
    goal: &BoundGoal,
    primal: QualifiedScalar,
    dual: QualifiedScalar,
    sense: pse_math::binding::ObjectiveSense,
) -> Result<OutputEvidence, U> {
    if goal.declaration.subject != S::OptimalObjective || goal.declaration.observation != O::Steady
    {
        return Err(U::UnsupportedObservation);
    }
    let up = scalar(goal, primal)?;
    let ud = scalar(goal, dual)?;
    let (lower, upper) = match sense {
        pse_math::binding::ObjectiveSense::Minimize if dual.value - ud <= primal.value + up => {
            (dual.value - ud, primal.value + up)
        }
        pse_math::binding::ObjectiveSense::Maximize if primal.value - up <= dual.value + ud => {
            (primal.value - up, dual.value + ud)
        }
        _ => return Err(U::InvalidValidity),
    };
    let class =
        if primal.class == AccuracyClass::Certified && dual.class == AccuracyClass::Certified {
            AccuracyClass::Certified
        } else {
            AccuracyClass::Estimated
        };
    let (lower, upper) = if class == AccuracyClass::Certified {
        // Preserve genuinely exact certified zero-width intervals.
        (
            if ud == 0. && sense == pse_math::binding::ObjectiveSense::Minimize
                || up == 0. && sense == pse_math::binding::ObjectiveSense::Maximize
            {
                lower
            } else {
                lower.next_down()
            },
            if up == 0. && sense == pse_math::binding::ObjectiveSense::Minimize
                || ud == 0. && sense == pse_math::binding::ObjectiveSense::Maximize
            {
                upper
            } else {
                upper.next_up()
            },
        )
    } else {
        (lower, upper)
    };
    let radius = (primal.value - lower)
        .abs()
        .max((upper - primal.value).abs());
    let radius = if radius == 0. { 0. } else { radius.next_up() };
    if !lower.is_finite() || !upper.is_finite() || !radius.is_finite() {
        return Err(U::Nonfinite);
    }
    let mut witness = pse_ids::FramedHasher::new(pse_ids::Frame::EngineeringGoalV1);
    witness
        .str("qualified-original-objective-interval")
        .hash(&primal.validity)
        .hash(&dual.validity);
    Ok(evidence(
        goal,
        (primal.value, radius, class),
        witness.finish_hash(),
        I::ObjectiveInterval,
        M::QualifiedObjectiveInterval,
        Some((lower, upper)),
        "Original feasible primal and qualified original objective bound; interval protects the objective only",
    ))
}

/// A real paired observation, with correspondence admitted by its trajectory owner.
#[derive(Clone, Copy, Debug)]
pub struct Comparison {
    /// Frozen scientific context, observation and base point of this pair.
    pub source: SemanticProductKey,
    /// Selected scalar at the production controls.
    pub base: f64,
    /// Same selected scalar at the actual contrasting controls.
    pub contrasting: f64,
    /// Actual mode/event/endpoint correspondence receipt.
    pub correspondence: ContentHash,
    /// Identity of the two controls actually executed.
    pub base_controls: ContentHash,
    /// Identity of the second controls actually executed.
    pub contrasting_controls: ContentHash,
    /// Same-point enclosure identity for the base authored scalar or state projection.
    pub base_arithmetic: ContentHash,
    /// Same-point enclosure identity for the contrasting authored scalar or state projection.
    pub contrasting_arithmetic: ContentHash,
    /// Independent authored evaluation uncertainty in the output's physical units.
    pub base_uncertainty: f64,
    /// Independent authored evaluation uncertainty in the output's physical units.
    pub contrasting_uncertainty: f64,
    /// Output representational floor retained by the physical projection owner.
    pub resolution_floor: f64,
}
/// Retain empirical output variation without claiming integration order or certification.
pub fn dynamic_comparison(goal: &BoundGoal, pair: Comparison) -> Result<OutputEvidence, U> {
    if goal.declaration.subject != S::SelectedOutput
        || !matches!(
            goal.declaration.observation,
            O::Sample | O::Endpoint | O::Integrated
        )
    {
        return Err(U::UnsupportedObservation);
    }
    if pair.source != goal.source || pair.base_controls == pair.contrasting_controls {
        return Err(U::InvalidValidity);
    }
    if !pair.base.is_finite()
        || !pair.contrasting.is_finite()
        || !pair.base_uncertainty.is_finite()
        || pair.base_uncertainty < 0.
        || !pair.contrasting_uncertainty.is_finite()
        || pair.contrasting_uncertainty < 0.
        || !pair.resolution_floor.is_finite()
        || pair.resolution_floor <= 0.
    {
        return Err(U::Nonfinite);
    }
    let variation = (pair.base - pair.contrasting).abs();
    if !variation.is_finite() {
        return Err(U::Nonfinite);
    }
    if variation < pair.resolution_floor
        && pair.base_uncertainty == 0.
        && pair.contrasting_uncertainty == 0.
    {
        return Err(U::PrecisionLimit);
    }
    let mut witness = pse_ids::FramedHasher::new(pse_ids::Frame::EngineeringGoalV1);
    witness
        .str("paired-physical-output")
        .hash(&pair.correspondence)
        .hash(&pair.base_controls)
        .hash(&pair.contrasting_controls)
        .hash(&pair.base_arithmetic)
        .hash(&pair.contrasting_arithmetic);
    // Keep the observed discrepancy unpadded for admission. Widen its positive
    // subtraction magnitude separately before accumulating arithmetic contributions.
    let mut error = if variation > 0. {
        variation.next_up()
    } else {
        0.
    };
    if !error.is_finite() {
        return Err(U::Nonfinite);
    }
    for contribution in [pair.base_uncertainty, pair.contrasting_uncertainty] {
        if contribution > 0. {
            error = (error + contribution).next_up();
            if !error.is_finite() {
                return Err(U::Nonfinite);
            }
        }
    }
    // Spacing prevents an estimated zero; it supplies no rounding-error bound.
    // Positive retained arithmetic uncertainty, rather than spacing itself, is
    // the independent support for agreement below the representation floor.
    let error = error.max(pair.resolution_floor);
    Ok(evidence(
        goal,
        (pair.base, error, AccuracyClass::Estimated),
        witness.finish_hash(),
        I::EmpiricalOutputVariation,
        M::DynamicComparison,
        None,
        "Estimated empirical paired output variation plus same-point authored evaluation uncertainty, with an explicit representational estimate floor; spacing is not a rounding-error bound, and no convergence-order or certified integration-error is claimed",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_ids::SemanticId;
    use pse_model::{
        engineering_accuracy::AccuracyGoal,
        generated::enums::{
            AccuracyCriterionStatus, AccuracyGoalStatus, AccuracyGoalUse, AccuracyResolutionStatus,
            NumericalSource, NumericalTarget,
        },
    };
    fn fixture() -> (BoundGoal, Comparison) {
        let hash = ContentHash::from_bytes([1; 32]);
        let id = SemanticId::from_bytes([1; 16]);
        let source = SemanticProductKey {
            structure: hash,
            binding: hash,
            numerical_policy: Some(hash),
            normalization: Some(hash),
            point: Some(hash),
            parameters: None,
            derivation: None,
            branch: Some(hash),
            accuracy: None,
        };
        let goal = BoundGoal {
            declaration: AccuracyGoal {
                goal_id: id.into(),
                model_id: None,
                case_id: None,
                instance_id: None,
                fit_id: None,
                target_id: id,
                target_kind: NumericalTarget::Observable,
                quantity_id: id,
                unit_id: id,
                subject: S::SelectedOutput,
                observation: O::Endpoint,
                time: None,
                resolution: Some(0.1),
                criterion_lower: Some(3.0),
                criterion_upper: Some(5.0),
                required_class: AccuracyClass::Estimated,
                use_policy: AccuracyGoalUse::Assess,
                refine: false,
                source: NumericalSource::Analysis,
                priority: 0,
                provenance: "paired endpoint with admitted arithmetic contributions".into(),
            },
            source,
            product: hash,
            normalization: hash,
        };
        let pair = Comparison {
            source,
            base: 4.,
            contrasting: 4.,
            correspondence: hash,
            base_controls: hash,
            contrasting_controls: ContentHash::from_bytes([2; 32]),
            base_arithmetic: hash,
            contrasting_arithmetic: hash,
            base_uncertainty: 0.,
            contrasting_uncertainty: 0.,
            resolution_floor: 4.0_f64.next_up() - 4.,
        };
        (goal, pair)
    }
    #[test]
    fn dynamic_comparison_zero_and_subresolution_agreement_need_other_support() {
        let (goal, mut pair) = fixture();
        assert_eq!(
            dynamic_comparison(&goal, pair).unwrap_err(),
            U::PrecisionLimit
        );
        pair.base = 0.;
        pair.contrasting = pair.resolution_floor / 2.;
        assert_eq!(
            dynamic_comparison(&goal, pair).unwrap_err(),
            U::PrecisionLimit
        );
        pair.base_uncertainty = 0.02;
        let actual = dynamic_comparison(&goal, pair).unwrap();
        assert_eq!(actual.accuracy.class, AccuracyClass::Estimated);
        assert!(actual.accuracy.error.unwrap() >= 0.02);
    }
    #[test]
    fn dynamic_comparison_supported_agreement_retains_estimated_resolution_and_criterion() {
        let (mut goal, mut pair) = fixture();
        pair.base_uncertainty = 0.02;
        pair.contrasting_uncertainty = 0.03;
        let actual = dynamic_comparison(&goal, pair).unwrap();
        assert_eq!(actual.method, M::DynamicComparison);
        assert_eq!(actual.interpretation, I::EmpiricalOutputVariation);
        assert_eq!(actual.accuracy.class, AccuracyClass::Estimated);
        assert!(actual.accuracy.error.unwrap() >= 0.05 && actual.accuracy.error.unwrap() < 0.051);
        let classified = pse_math::engineering_accuracy::classify(&goal, Some(&actual));
        assert_eq!(classified.resolution, AccuracyResolutionStatus::Met);
        assert_eq!(classified.criterion, AccuracyCriterionStatus::Satisfied);
        assert_eq!(classified.status, AccuracyGoalStatus::Satisfied);
        goal.declaration.criterion_upper = Some(4.01);
        assert_eq!(
            pse_math::engineering_accuracy::classify(&goal, Some(&actual)).criterion,
            AccuracyCriterionStatus::Unresolved
        );
        goal.declaration.required_class = AccuracyClass::Certified;
        assert_eq!(
            pse_math::engineering_accuracy::classify(&goal, Some(&actual)).unavailable,
            Some(U::InsufficientStrength)
        );
    }
    #[test]
    fn dynamic_comparison_spacing_is_only_a_floor_and_positive_sum_overflow_refuses() {
        let (goal, mut pair) = fixture();
        pair.base_uncertainty = pair.resolution_floor / 2.;
        assert_eq!(
            dynamic_comparison(&goal, pair).unwrap().accuracy.error,
            Some(pair.resolution_floor)
        );
        pair.base_uncertainty = f64::MAX;
        assert_eq!(dynamic_comparison(&goal, pair).unwrap_err(), U::Nonfinite);
        pair.base_uncertainty = f64::MAX * 0.75;
        pair.contrasting_uncertainty = f64::MAX * 0.75;
        assert_eq!(dynamic_comparison(&goal, pair).unwrap_err(), U::Nonfinite);
        pair.base = f64::MAX;
        pair.contrasting = -f64::MAX;
        assert_eq!(dynamic_comparison(&goal, pair).unwrap_err(), U::Nonfinite);
    }
}
