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
    value: f64,
    error: f64,
    class: AccuracyClass,
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
        primal.value,
        radius,
        class,
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
    if variation < pair.resolution_floor {
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
    let error = variation + pair.base_uncertainty + pair.contrasting_uncertainty
        + pair.resolution_floor;
    let error = error.next_up();
    if !error.is_finite() {
        return Err(U::Nonfinite);
    }
    Ok(evidence(
        goal,
        pair.base,
        error,
        AccuracyClass::Estimated,
        witness.finish_hash(),
        I::EmpiricalOutputVariation,
        M::DynamicComparison,
        None,
        "Estimated empirical paired output variation plus same-point authored evaluation uncertainty and representational spacing; no convergence-order or certified integration-error claim",
    ))
}
