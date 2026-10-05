// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Physical goal classification and contributor demands; this module executes no solver.
use crate::MathError;
use pse_model::engineering_accuracy::{AccuracyGoal, BoundGoal, OutputEvidence};
use pse_model::generated::enums::{
    AccuracyCriterionStatus as Criterion, AccuracyEvidenceInterpretation as Interpretation,
    AccuracyEvidenceMethod as Method, AccuracyGoalStatus as Status, AccuracyGoalSubject as Subject,
    AccuracyResolutionStatus as Resolution, AccuracyUnavailableReason as Unavailable,
};
use pse_model::strategy::{AccuracyClass, AccuracyDemand, AccuracyEvidence};
use pse_ids::{ContentHash, SemanticId};

/// One physical original-row residual allowance derived from a local output adjoint.
/// It is operational work guidance only and grants no feasibility permission.
#[derive(Clone, Debug, PartialEq)]
pub struct ResidualRowDemand {
    /// Original equality row whose residual is measured in `physical_allowance` units.
    pub row: SemanticId,
    /// Absolute physical residual allowance allocated to this row.
    pub physical_allowance: f64,
}

/// Estimated work demand retained beside (never serialized into) goal evidence.
/// The adjoint is local and does not certify a nonlinear output error.
#[derive(Clone, Debug, PartialEq)]
pub struct GoalWorkDemand {
    /// Source-bound product whose local gradient generated this demand.
    pub product: ContentHash,
    /// Original point and numerical dependencies used by the derivative action.
    pub source: pse_model::strategy::SemanticProductKey,
    /// Effective resolution/decision-boundary allowance in physical output units.
    pub physical_output_allowance: f64,
    /// Backward error of the performed sparse adjoint solve.
    pub adjoint_backward_error: f64,
    /// Per-row limits whose current local contributions are allocated against the
    /// remaining output allowance. These are estimates, not a certificate or acceptance limits.
    pub rows: Vec<ResidualRowDemand>,
}

/// Pure assessment of an actual output product. Completion owns the final retained row.
#[derive(Clone, Debug, PartialEq)]
pub struct Classification {
    /// Resolution/decision outcome under the actual evidence strength.
    pub status: Status,
    /// Output accuracy remains independently visible when a decision resolves.
    pub resolution: Resolution,
    /// A clear specification violation is not hidden by unmet value accuracy.
    pub criterion: Criterion,
    /// Actual physical envelope, never a widened specification band.
    pub interval: Option<(f64, f64)>,
    /// Error about the reported representative, including asymmetric intervals.
    pub error: Option<f64>,
    /// Why the requested interpretation could not be established.
    pub unavailable: Option<Unavailable>,
}
fn unresolved(goal: &AccuracyGoal, reason: Unavailable) -> Classification {
    Classification {
        status: Status::Unresolved,
        resolution: if goal.resolution.is_some() {
            Resolution::Unavailable
        } else {
            Resolution::NotRequested
        },
        criterion: if goal.criterion_lower.is_some() || goal.criterion_upper.is_some() {
            Criterion::Unresolved
        } else {
            Criterion::NotRequested
        },
        interval: None,
        error: None,
        unavailable: Some(reason),
    }
}
fn supports(actual: AccuracyClass, required: AccuracyClass) -> bool {
    matches!(
        (actual, required),
        (
            AccuracyClass::Certified,
            AccuracyClass::Certified | AccuracyClass::Estimated
        ) | (AccuracyClass::Estimated, AccuracyClass::Estimated)
    )
}
/// Classify a frozen physical goal from actual matching evidence, without refining.
/// Missing/invalid evidence remains unresolved; no native outcome is upgraded here.
pub fn classify(goal: &BoundGoal, evidence: Option<&OutputEvidence>) -> Classification {
    let declaration = &goal.declaration;
    let unresolved = |reason| unresolved(declaration, reason);
    if pse_model::engineering_accuracy::validate_goal(declaration).is_err() {
        return unresolved(Unavailable::Unsupported);
    }
    let Some(evidence) = evidence else {
        return unresolved(Unavailable::MissingEvidence);
    };
    if evidence.target != declaration.target_id
        || evidence.target_kind != declaration.target_kind
        || evidence.quantity != declaration.quantity_id
        || evidence.unit != declaration.unit_id
        || evidence.observation != declaration.observation
        || evidence.time != declaration.time
        || evidence.source != goal.source
        || evidence.accuracy.product != goal.product
        || evidence.accuracy.normalization != goal.normalization
    {
        return unresolved(Unavailable::InvalidValidity);
    }
    if evidence.validity.is_none() {
        return unresolved(Unavailable::InvalidValidity);
    }
    let interpretation_valid = match declaration.subject {
        Subject::OptimalObjective => evidence.interpretation == Interpretation::ObjectiveInterval,
        Subject::SelectedOutput => matches!(
            evidence.interpretation,
            Interpretation::OutputError | Interpretation::EmpiricalOutputVariation
        ),
    };
    if !interpretation_valid {
        return unresolved(Unavailable::Unsupported);
    }
    let method_valid = match evidence.method {
        Method::SquareCorrection
        | Method::KktCorrection
        | Method::CertifiedEnclosure
        | Method::ComposedOutput => evidence.interpretation == Interpretation::OutputError,
        Method::QualifiedObjectiveInterval => {
            evidence.interpretation == Interpretation::ObjectiveInterval
        }
        Method::DynamicComparison => {
            evidence.interpretation == Interpretation::EmpiricalOutputVariation
        }
    };
    if !method_valid {
        return unresolved(Unavailable::Unsupported);
    }
    if !supports(evidence.accuracy.class, declaration.required_class)
        || evidence.accuracy.class == AccuracyClass::Certified
            && evidence.interpretation == Interpretation::EmpiricalOutputVariation
        || evidence.accuracy.class == AccuracyClass::Certified
            && matches!(
                evidence.method,
                Method::SquareCorrection | Method::KktCorrection | Method::DynamicComparison
            )
    {
        return unresolved(Unavailable::InsufficientStrength);
    }
    if evidence.value.is_some_and(|v| !v.is_finite())
        || evidence
            .accuracy
            .error
            .is_some_and(|e| !e.is_finite() || e < 0.0)
    {
        return unresolved(Unavailable::Nonfinite);
    }
    let interval = match evidence.interval {
        Some((lower, upper)) if lower.is_finite() && upper.is_finite() && lower <= upper => {
            (lower, upper)
        }
        Some(_) => return unresolved(Unavailable::Nonfinite),
        None => {
            let (Some(value), Some(error)) = (evidence.value, evidence.accuracy.error) else {
                return unresolved(Unavailable::MissingObservation);
            };
            // An estimated zero is not proof of exactness; producers must retain their
            // evaluator/supplier estimate floor or report missing evidence.
            if error == 0.0 && evidence.accuracy.class == AccuracyClass::Estimated {
                return unresolved(Unavailable::EvaluatorUncertainty);
            }
            if error == 0.0 {
                (value, value)
            } else {
                ((value - error).next_down(), (value + error).next_up())
            }
        }
    };
    if !interval.0.is_finite() || !interval.1.is_finite() {
        return unresolved(Unavailable::Nonfinite);
    }
    if evidence.accuracy.class == AccuracyClass::Estimated && interval.0 == interval.1 {
        return unresolved(Unavailable::EvaluatorUncertainty);
    }
    if declaration.resolution.is_some() && evidence.value.is_none() {
        return unresolved(Unavailable::MissingObservation);
    }
    let radius = evidence.value.map(|q| {
        let distance = (interval.0 - q).abs().max((interval.1 - q).abs());
        if distance == 0.0 {
            0.0
        } else {
            distance.next_up()
        }
    });
    let error = match (radius, evidence.accuracy.error) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (a, b) => a.or(b),
    };
    let resolution = match declaration.resolution {
        None => Resolution::NotRequested,
        Some(delta) if error.is_some_and(|e| e <= delta) => Resolution::Met,
        Some(_) if error.is_some() => Resolution::Unmet,
        Some(_) => Resolution::Unavailable,
    };
    let lower = declaration.criterion_lower.unwrap_or(f64::NEG_INFINITY);
    let upper = declaration.criterion_upper.unwrap_or(f64::INFINITY);
    let inside = interval.0 >= lower && interval.1 <= upper;
    let disjoint = interval.1 < lower || interval.0 > upper;
    let criterion =
        if declaration.criterion_lower.is_none() && declaration.criterion_upper.is_none() {
            Criterion::NotRequested
        } else if inside {
            Criterion::Satisfied
        } else if disjoint {
            Criterion::Violated
        } else {
            Criterion::Unresolved
        };
    let resolution_met = matches!(resolution, Resolution::Met | Resolution::NotRequested);
    let status = if !resolution_met || criterion == Criterion::Unresolved {
        Status::Unresolved
    } else if criterion == Criterion::Violated {
        Status::Violated
    } else if matches!(criterion, Criterion::Satisfied | Criterion::NotRequested) {
        Status::Satisfied
    } else {
        Status::Unresolved
    };
    let unavailable = if !resolution_met {
        Some(Unavailable::InsufficientAccuracy)
    } else if criterion == Criterion::Unresolved {
        Some(Unavailable::Boundary)
    } else {
        None
    };
    Classification {
        status,
        resolution,
        criterion,
        interval: Some(interval),
        error,
        unavailable,
    }
}

/// One immutable engineering assessment, projected to the generated completion relation.
#[derive(Clone, Debug, PartialEq)]
pub struct GoalResult {
    /// The admitted physical obligation.
    pub goal: AccuracyGoal,
    /// Outcomes classified once from the final evidence.
    pub classification: Classification,
    /// Actual produced evidence, including limitations and mismatched-source diagnostics.
    pub evidence: Option<OutputEvidence>,
    /// Optional local derivative-based solver-work demand; not part of durable evidence.
    pub work_demand: Option<GoalWorkDemand>,
    /// Original typed cause of a locally unavailable optional evidence operation.
    pub failure: Option<GoalFailure>,
}
/// Shared optional-action cause; repeated projections retain its original owner.
#[derive(Clone, Debug)]
pub struct GoalFailure {
    /// Original diagnostic authority, independent of its rendered explanation.
    pub cause: std::sync::Arc<pse_model::diagnostic::DiagnosticCause>,
    /// Producer-reported owned cause extent, including the shared wrapper.
    pub bytes: usize,
    owner: Option<std::sync::Arc<dyn crate::AllocationOwner>>,
}
impl GoalFailure {
    /// Retain the original cause and producer-reported extent.
    pub fn new(cause: pse_model::diagnostic::DiagnosticCause, bytes: usize) -> Self {
        Self { cause: std::sync::Arc::new(cause), bytes, owner: None }
    }
    /// Attach runtime accounting without changing diagnostic or semantic ownership.
    pub fn with_owner(mut self, owner: std::sync::Arc<dyn crate::AllocationOwner>) -> Self {
        self.owner = Some(crate::retain_allocation_owner(self.owner.take(), owner));
        self
    }
}
impl PartialEq for GoalFailure {
    fn eq(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.cause, &other.cause) && self.bytes == other.bytes
    }
}
impl GoalResult {
    /// Retain a classification at its frozen dependencies; this performs no solver work.
    pub fn assess(goal: &BoundGoal, evidence: Option<OutputEvidence>) -> Self {
        let classification = classify(goal, evidence.as_ref());
        Self {
            goal: goal.declaration.clone(),
            classification,
            evidence,
            work_demand: None,
            failure: None,
        }
    }
    /// Requested goals cannot disappear when a producer is unavailable.
    pub fn unavailable(goal: AccuracyGoal, reason: Unavailable) -> Self {
        let classification = unresolved(&goal, reason);
        Self {
            goal,
            classification,
            evidence: None,
            work_demand: None,
            failure: None,
        }
    }
    /// Copy the retained scientific interpretation, adding only execution metadata.
    pub fn row(
        &self,
        run_id: pse_model::generated::identities::RunId,
        step: i64,
    ) -> pse_model::engineering_accuracy::GoalAssessment {
        let g = &self.goal;
        let e = self.evidence.as_ref();
        let dependencies = e
            .map(|e| {
                let s = e.source;
                [
                    Some(s.structure),
                    Some(s.binding),
                    s.numerical_policy,
                    s.normalization,
                    s.point,
                    s.parameters,
                    s.derivation,
                    s.branch,
                    s.accuracy,
                ]
                .into_iter()
                .flatten()
                .collect()
            })
            .unwrap_or_default();
        pse_model::engineering_accuracy::GoalAssessment {
            run_id,
            step,
            goal_id: g.goal_id,
            model_id: g.model_id,
            case_id: g.case_id,
            instance_id: g.instance_id,
            fit_id: g.fit_id,
            target_id: g.target_id,
            target_kind: g.target_kind,
            quantity_id: g.quantity_id,
            unit_id: g.unit_id,
            subject: g.subject,
            observation: g.observation,
            time: g.time,
            value: e.and_then(|e| e.value).filter(|v| v.is_finite()),
            interval_lower: self.classification.interval.map(|v| v.0),
            interval_upper: self.classification.interval.map(|v| v.1),
            error: self.classification.error,
            accuracy_class: e.map(|e| e.accuracy.class),
            required_class: g.required_class,
            status: self.classification.status,
            resolution_status: self.classification.resolution,
            criterion_status: self.classification.criterion,
            use_policy: g.use_policy,
            resolution: g.resolution,
            criterion_lower: g.criterion_lower,
            criterion_upper: g.criterion_upper,
            refine: g.refine,
            context: e.map(|e| e.source.binding),
            product: e.map(|e| e.accuracy.product),
            point: e.and_then(|e| e.source.point),
            branch: e.and_then(|e| e.source.branch),
            validity: e.and_then(|e| e.validity),
            dependencies,
            interpretation: e.map(|e| e.interpretation),
            method: e.map(|e| e.method),
            unavailable: self.classification.unavailable,
            limitation: e.map_or_else(
                || self.failure.as_ref().map_or_else(
                    || "requested output evidence unavailable".into(),
                    |failure| format!("optional output evidence unavailable: {}", failure.cause),
                ),
                |e| e.limitation.clone(),
            ),
        }
    }
}

/// A contributor already aggregated by source product; duplicates are an admission error.
#[derive(Clone, Copy, Debug)]
pub struct Contribution {
    /// Actual observed supplier error in its own physical normalization.
    pub evidence: AccuracyEvidence,
    /// Admitted absolute influence from supplier error into the protected output.
    pub gain: f64,
    /// Strength of the influence, independent of supplier evidence strength.
    pub gain_class: AccuracyClass,
    /// Whether this actual supplier supports improving its consumed product.
    pub reducible: bool,
}
/// No solver work is requested by an allocation itself.
#[derive(Clone, Debug, PartialEq)]
pub struct Allocation {
    /// Actual conservative sum including fixed remainder.
    pub error: Option<f64>,
    /// Compatible per-product demands; fixed contributions never receive a demand.
    pub demands: Vec<AccuracyDemand>,
    /// An unavailable allocation is not zero error.
    pub unavailable: Option<Unavailable>,
}
fn unavailable_allocation(reason: Unavailable) -> Allocation {
    Allocation {
        error: None,
        demands: vec![],
        unavailable: Some(reason),
    }
}
fn upper_add(a: f64, b: f64, certified: bool) -> f64 {
    if a == 0.0 {
        b
    } else if b == 0.0 {
        a
    } else if certified {
        (a + b).next_up()
    } else {
        a + b
    }
}
fn upper_product(a: f64, b: f64, certified: bool) -> f64 {
    if a == 0.0 || b == 0.0 {
        0.0
    } else if certified {
        (a * b).next_up()
    } else {
        a * b
    }
}
/// Reserve fixed contributions, then allocate remaining error proportionally to observed
/// reducible contributions. The returned demands never loosen the supplier's observed error.
/// # Errors
/// Invalid demands/numbers or repeated source products, which must first be aggregated.
pub fn allocate(
    allowance: f64,
    class: AccuracyClass,
    remainder: f64,
    contributions: &[Contribution],
) -> Result<Allocation, MathError> {
    if !allowance.is_finite()
        || allowance < 0.0
        || !remainder.is_finite()
        || remainder < 0.0
        || class == AccuracyClass::Unresolved
    {
        return Err(MathError::Contract(
            "invalid engineering error allocation".into(),
        ));
    }
    let certified = class == AccuracyClass::Certified;
    let mut products = std::collections::BTreeSet::new();
    let mut fixed = remainder;
    let mut reducible = 0.0;
    let mut observed = Vec::with_capacity(contributions.len());
    for contribution in contributions {
        let e = contribution.evidence;
        if !products.insert((e.product, e.normalization)) {
            return Err(MathError::Contract(
                "duplicate engineering contributor; aggregate its influence once".into(),
            ));
        }
        if !contribution.gain.is_finite() || contribution.gain < 0.0 {
            return Ok(unavailable_allocation(Unavailable::Nonfinite));
        }
        if !supports(contribution.gain_class, class) {
            return Ok(unavailable_allocation(Unavailable::InsufficientStrength));
        }
        if contribution.gain == 0.0 {
            observed.push(0.0);
            continue;
        }
        if !supports(e.class, class) {
            return Ok(unavailable_allocation(Unavailable::InsufficientStrength));
        }
        let Some(error) = e.error.filter(|e| e.is_finite() && *e >= 0.0) else {
            return Ok(unavailable_allocation(Unavailable::MissingEvidence));
        };
        let c = upper_product(contribution.gain, error, certified);
        observed.push(c);
        if contribution.reducible {
            reducible = upper_add(reducible, c, certified);
        } else {
            fixed = upper_add(fixed, c, certified);
        }
    }
    let total = upper_add(fixed, reducible, certified);
    if !total.is_finite() {
        return Ok(unavailable_allocation(Unavailable::Nonfinite));
    }
    if total <= allowance {
        return Ok(Allocation {
            error: Some(total),
            demands: vec![],
            unavailable: None,
        });
    }
    let available = if certified {
        (allowance - fixed).next_down()
    } else {
        allowance - fixed
    };
    if available <= 0.0 || reducible <= 0.0 {
        return Ok(Allocation {
            error: Some(total),
            demands: vec![],
            unavailable: Some(Unavailable::FixedErrorFloor),
        });
    }
    let mut demands = Vec::new();
    for (contribution, observed) in contributions.iter().zip(observed) {
        if !contribution.reducible || observed == 0.0 {
            continue;
        }
        // Compute the bounded fraction before multiplying to avoid avoidable overflow.
        let requested = if certified {
            let ratio = (observed / reducible).next_down().max(0.0);
            let c = (available * ratio).next_down().max(0.0);
            (c / contribution.gain).next_down().max(0.0)
        } else {
            available * (observed / reducible) / contribution.gain
        };
        if requested == 0.0 {
            return Ok(Allocation {
                error: Some(total),
                demands: vec![],
                unavailable: Some(Unavailable::PrecisionLimit),
            });
        }
        demands.push(AccuracyDemand {
            product: contribution.evidence.product,
            normalization: contribution.evidence.normalization,
            allowance: requested.min(contribution.evidence.error.unwrap_or(0.0)),
            class,
        });
    }
    Ok(Allocation {
        error: Some(total),
        demands,
        unavailable: None,
    })
}

/// Bind authored and request-local goals to selected physical targets. Request overrides
/// preserve a goal's target/subject/location; new targets require new identities.
/// # Errors
/// Duplicate/conflicting identities, unselected targets or invalid physical conversions.
pub fn bind_goals(
    registry: &pse_quantity::QuantityRegistry,
    targets: &[crate::numerics::TargetSpec],
    declarations: &[AccuracyGoal],
    requested: &[AccuracyGoal],
) -> Result<Vec<AccuracyGoal>, MathError> {
    use pse_model::generated::enums::NumericalSource;
    let mut selected = std::collections::BTreeMap::new();
    for (analysis, rows) in [(false, declarations), (true, requested)] {
        let mut identities = std::collections::BTreeSet::new();
        for row in rows {
            pse_model::engineering_accuracy::validate_goal(row)
                .map_err(|e| MathError::Contract(e.to_string()))?;
            if !identities.insert((
                row.goal_id,
                if analysis {
                    NumericalSource::Analysis
                } else {
                    row.source
                },
            )) {
                return Err(MathError::Contract(
                    "duplicate same-source accuracy goal".into(),
                ));
            }
            if let Some(previous) = selected.get(&row.goal_id) {
                let previous: &AccuracyGoal = previous;
                if previous.target_id != row.target_id
                    || previous.target_kind != row.target_kind
                    || previous.subject != row.subject
                    || previous.observation != row.observation
                    || previous.time != row.time
                {
                    return Err(MathError::Contract(
                        "accuracy goal override changes protected target or observation".into(),
                    ));
                }
                if !analysis {
                    let rank = |source| match source {
                        NumericalSource::Analysis => 3,
                        NumericalSource::Case => 2,
                        NumericalSource::Model => 1,
                        _ => 0,
                    };
                    if rank(previous.source) > rank(row.source) {
                        continue;
                    }
                    if rank(previous.source) == rank(row.source) {
                        return Err(MathError::Contract(
                            "conflicting authored accuracy goal identity".into(),
                        ));
                    }
                }
            }
            let mut row = row.clone();
            if analysis {
                row.source = NumericalSource::Analysis;
            }
            selected.insert(row.goal_id, row);
        }
    }
    selected
        .into_values()
        .map(|mut row| {
            let target = targets
                .iter()
                .find(|t| t.id == row.target_id && t.kind == row.target_kind)
                .ok_or_else(|| {
                    MathError::Contract("accuracy goal names an unselected scalar target".into())
                })?;
            pse_quantity::admission::require_same_contract(
                row.quantity_id.into(),
                target.quantity,
                registry,
            )?;
            let quantity = registry.quantity_type(target.quantity)?;
            let conversion = pse_quantity::convert_spec_for_type(
                registry.unit(row.unit_id.into())?,
                registry.unit(target.unit)?,
                &quantity.key,
            )?;
            row.resolution = row.resolution.map(|v| v * conversion.scale.abs());
            row.criterion_lower = row
                .criterion_lower
                .map(|v| v * conversion.scale + conversion.offset);
            row.criterion_upper = row
                .criterion_upper
                .map(|v| v * conversion.scale + conversion.offset);
            if conversion.scale < 0.0 {
                std::mem::swap(&mut row.criterion_lower, &mut row.criterion_upper);
            }
            row.quantity_id = target.quantity.as_id();
            row.unit_id = target.unit.as_id();
            pse_model::engineering_accuracy::validate_goal(&row)
                .map_err(|e| MathError::Contract(e.to_string()))?;
            Ok(row)
        })
        .collect()
}

/// A decision-overlap request uses representable positive slack below its nearest boundary.
/// Exact boundary/precision limits yield no blind zero-accuracy retry.
pub fn refinement_allowance(goal: &BoundGoal, value: f64) -> Option<f64> {
    if !goal.declaration.refine || !value.is_finite() {
        return None;
    }
    let margin = goal
        .declaration
        .criterion_lower
        .into_iter()
        .chain(goal.declaration.criterion_upper)
        .map(|b| (value - b).abs())
        .fold(f64::INFINITY, f64::min);
    let decision = margin.next_down();
    let allowance = goal
        .declaration
        .resolution
        .unwrap_or(f64::INFINITY)
        .min(decision);
    (allowance.is_finite() && allowance > 0.0).then_some(allowance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_ids::{ContentHash, SemanticId};
    use pse_model::generated::enums::{
        AccuracyGoalUse, AccuracyObservation, NumericalSource, NumericalTarget,
    };
    use pse_model::strategy::SemanticProductKey;
    fn hash(value: u8) -> ContentHash {
        ContentHash::from_bytes([value; 32])
    }
    fn id(value: u8) -> SemanticId {
        SemanticId::from_bytes([value; 16])
    }
    fn goal() -> BoundGoal {
        BoundGoal {
            declaration: AccuracyGoal {
                goal_id: id(1).into(),
                model_id: None,
                case_id: None,
                instance_id: None,
                fit_id: None,
                target_id: id(2),
                target_kind: NumericalTarget::Observable,
                quantity_id: id(3),
                unit_id: id(4),
                subject: Subject::SelectedOutput,
                observation: AccuracyObservation::Steady,
                time: None,
                resolution: None,
                criterion_lower: None,
                criterion_upper: Some(10.0),
                required_class: AccuracyClass::Estimated,
                use_policy: AccuracyGoalUse::Assess,
                refine: true,
                source: NumericalSource::Analysis,
                priority: 0,
                provenance: "independent test specification".into(),
            },
            source: SemanticProductKey {
                structure: hash(1),
                binding: hash(2),
                numerical_policy: Some(hash(3)),
                normalization: Some(hash(4)),
                point: Some(hash(5)),
                parameters: None,
                derivation: None,
                branch: Some(hash(6)),
                accuracy: None,
            },
            product: hash(7),
            normalization: hash(4),
        }
    }
    fn evidence(goal: &BoundGoal, value: f64, interval: (f64, f64)) -> OutputEvidence {
        OutputEvidence {
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
                class: AccuracyClass::Estimated,
                error: None,
            },
            source: goal.source,
            validity: Some(hash(9)),
            interpretation: Interpretation::OutputError,
            method: Method::SquareCorrection,
            interval: Some(interval),
            limitation: "local estimate".into(),
        }
    }
    #[test]
    fn engineering_accuracy_goal_binding_converts_points_and_magnitudes_independently() {
        use pse_quantity::standard::{ids, standard_registry};
        let registry = standard_registry().unwrap();
        let mut goal = goal().declaration;
        goal.target_id = id(8);
        goal.target_kind = NumericalTarget::Variable;
        goal.quantity_id = ids::quantity("temperature.point").as_id();
        goal.unit_id = ids::unit("degF").as_id();
        goal.resolution = Some(0.18);
        goal.criterion_upper = Some(32.0);
        goal.source = NumericalSource::Model;
        let target = crate::numerics::TargetSpec {
            id: goal.target_id,
            kind: goal.target_kind,
            quantity: goal.quantity_id.into(),
            unit: ids::unit("K"),
            integer: false,
            declared_tolerance: None,
        };
        let converted = bind_goals(&registry, &[target.clone()], &[goal.clone()], &[]).unwrap();
        assert!((converted[0].resolution.unwrap() - 0.1).abs() < 1e-12);
        assert!((converted[0].criterion_upper.unwrap() - 273.15).abs() < 1e-12);
        let mut request = goal.clone();
        request.resolution = Some(0.36);
        let selected = bind_goals(
            &registry,
            &[target.clone()],
            &[goal.clone()],
            &[request.clone()],
        )
        .unwrap();
        assert!((selected[0].resolution.unwrap() - 0.2).abs() < 1e-12);
        let mut conflicting = request.clone();
        conflicting.source = NumericalSource::Case;
        assert!(
            bind_goals(
                &registry,
                &[target.clone()],
                &[],
                &[request.clone(), conflicting]
            )
            .is_err()
        );
        request.target_id = id(99);
        assert!(bind_goals(&registry, &[target], &[goal], &[request]).is_err());
    }
    #[test]
    fn engineering_accuracy_decision_only_and_combined_obligations() {
        let mut goal = goal();
        // The entire coarse envelope is below the threshold; no digit cap is implied.
        assert_eq!(
            classify(&goal, Some(&evidence(&goal, 4.0, (1.0, 7.0)))).status,
            Status::Satisfied
        );
        assert_eq!(
            classify(&goal, Some(&evidence(&goal, 12.0, (11.0, 13.0)))).status,
            Status::Violated
        );
        assert_eq!(
            classify(&goal, Some(&evidence(&goal, 10.0, (9.0, 11.0)))).status,
            Status::Unresolved
        );
        assert_eq!(
            classify(&goal, Some(&evidence(&goal, 9.0, (8.0, 10.0)))).status,
            Status::Satisfied
        );
        goal.declaration.resolution = Some(0.5);
        assert_eq!(
            classify(&goal, Some(&evidence(&goal, 4.0, (1.0, 7.0)))).status,
            Status::Unresolved
        );
        assert_eq!(
            classify(&goal, Some(&evidence(&goal, 4.0, (3.8, 4.2)))).status,
            Status::Satisfied
        );
        assert_eq!(classify(&goal, None).status, Status::Unresolved);
        let result = classify(&goal, Some(&evidence(&goal, 12.0, (11.0, 13.0))));
        assert_eq!(result.criterion, Criterion::Violated);
        assert_eq!(result.resolution, Resolution::Unmet);
        assert_eq!(result.status, Status::Unresolved);
        assert_eq!(result.unavailable, Some(Unavailable::InsufficientAccuracy));
    }
    #[test]
    fn engineering_accuracy_asymmetric_optimum_interval_uses_reported_value() {
        let mut goal = goal();
        goal.declaration.subject = Subject::OptimalObjective;
        goal.declaration.target_kind = NumericalTarget::Objective;
        goal.declaration.resolution = Some(1.5);
        goal.declaration.criterion_upper = None;
        let mut evidence = evidence(&goal, 3.0, (1.0, 3.0));
        evidence.interpretation = Interpretation::ObjectiveInterval;
        evidence.method = Method::QualifiedObjectiveInterval;
        // Gap 2 has half-gap 1, but the reported incumbent 3 is 2 from lower bound.
        assert_eq!(classify(&goal, Some(&evidence)).status, Status::Unresolved);
        evidence.value = Some(2.0);
        assert_eq!(classify(&goal, Some(&evidence)).status, Status::Satisfied);
        evidence.interpretation = Interpretation::ResidualError;
        assert_eq!(classify(&goal, Some(&evidence)).status, Status::Unresolved);
    }
    #[test]
    fn engineering_accuracy_strength_binding_and_boundary_are_not_inferred() {
        let mut goal = goal();
        goal.declaration.required_class = AccuracyClass::Certified;
        let mut evidence = evidence(&goal, 4.0, (3.0, 5.0));
        assert_eq!(
            classify(&goal, Some(&evidence)).unavailable,
            Some(Unavailable::InsufficientStrength)
        );
        evidence.accuracy.class = AccuracyClass::Certified;
        // Relabelling a local correction cannot certify it.
        assert_eq!(
            classify(&goal, Some(&evidence)).unavailable,
            Some(Unavailable::InsufficientStrength)
        );
        evidence.method = Method::CertifiedEnclosure;
        assert_eq!(classify(&goal, Some(&evidence)).status, Status::Satisfied);
        evidence.source.branch = Some(hash(99));
        assert_eq!(classify(&goal, Some(&evidence)).status, Status::Unresolved);
        evidence.source = goal.source;
        evidence.interpretation = Interpretation::EmpiricalOutputVariation;
        assert_eq!(classify(&goal, Some(&evidence)).status, Status::Unresolved);
        assert_eq!(refinement_allowance(&goal, 10.0), None);
        assert!(refinement_allowance(&goal, 9.0).unwrap() < 1.0);
        goal.declaration.refine = false;
        assert_eq!(refinement_allowance(&goal, 9.0), None);
    }
    #[test]
    fn engineering_accuracy_zero_estimates_and_missing_representatives_remain_unresolved() {
        let mut goal = goal();
        let mut evidence = evidence(&goal, 4.0, (4.0, 4.0));
        assert_eq!(
            classify(&goal, Some(&evidence)).unavailable,
            Some(Unavailable::EvaluatorUncertainty)
        );
        evidence.interval = Some((3.0, 5.0));
        evidence.value = None;
        evidence.accuracy.error = Some(1.0);
        assert_eq!(classify(&goal, Some(&evidence)).status, Status::Satisfied);
        goal.declaration.resolution = Some(2.0);
        assert_eq!(
            classify(&goal, Some(&evidence)).unavailable,
            Some(Unavailable::MissingObservation)
        );
    }
    fn contributor(index: u8, error: f64, gain: f64, reducible: bool) -> Contribution {
        Contribution {
            evidence: AccuracyEvidence {
                product: hash(index),
                normalization: hash(4),
                class: AccuracyClass::Estimated,
                error: Some(error),
            },
            gain,
            gain_class: AccuracyClass::Estimated,
            reducible,
        }
    }
    #[test]
    fn engineering_accuracy_allocation_reserves_fixed_error() {
        let allocation = allocate(
            1.0,
            AccuracyClass::Estimated,
            0.0,
            &[
                contributor(1, 0.4, 1.0, false),
                contributor(2, 0.8, 1.0, true),
            ],
        )
        .unwrap();
        assert_eq!(allocation.demands.len(), 1);
        assert!((allocation.demands[0].allowance - 0.6).abs() < 1e-15);
        // Independent budget arithmetic: 0.4 + 0.6 = 1, not 0.4 + 1.
        let blocked = allocate(
            0.3,
            AccuracyClass::Estimated,
            0.0,
            &[
                contributor(1, 0.4, 1.0, false),
                contributor(2, 0.8, 1.0, true),
            ],
        )
        .unwrap();
        assert_eq!(blocked.unavailable, Some(Unavailable::FixedErrorFloor));
        assert!(blocked.demands.is_empty());
    }
    #[test]
    fn engineering_accuracy_allocation_amplification_and_certified_rounding() {
        let inputs = [
            contributor(1, 0.2, 2.0, true),
            contributor(2, 0.3, 2.0, true),
        ];
        let allocation = allocate(0.5, AccuracyClass::Estimated, 0.0, &inputs).unwrap();
        assert!((allocation.demands[0].allowance - 0.1).abs() < 1e-15);
        assert!((allocation.demands[1].allowance - 0.15).abs() < 1e-15);
        assert_eq!(
            allocate(0.5, AccuracyClass::Certified, 0.0, &inputs)
                .unwrap()
                .unavailable,
            Some(Unavailable::InsufficientStrength)
        );
        let certified = inputs.map(|mut c| {
            c.evidence.class = AccuracyClass::Certified;
            c.gain_class = AccuracyClass::Certified;
            c
        });
        let allocation = allocate(0.5, AccuracyClass::Certified, 0.0, &certified).unwrap();
        assert!(allocation.demands[0].allowance <= 0.1);
        assert!(allocation.demands[1].allowance <= 0.15);
        assert!(allocate(0.5, AccuracyClass::Estimated, 0.0, &[inputs[0], inputs[0]]).is_err());
    }
    #[test]
    fn engineering_accuracy_unknown_zero_influence_and_unrepresentable_demands_stop() {
        let mut zero = contributor(1, 1.0, 0.0, true);
        zero.gain_class = AccuracyClass::Unresolved;
        assert_eq!(
            allocate(1.0, AccuracyClass::Estimated, 0.0, &[zero])
                .unwrap()
                .unavailable,
            Some(Unavailable::InsufficientStrength)
        );
        zero.gain_class = AccuracyClass::Certified;
        zero.evidence.error = None;
        zero.evidence.class = AccuracyClass::Unresolved;
        assert_eq!(
            allocate(1.0, AccuracyClass::Certified, 0.0, &[zero])
                .unwrap()
                .error,
            Some(0.0)
        );
        let input = contributor(2, 1.0, 2.0, true);
        assert_eq!(
            allocate(f64::from_bits(1), AccuracyClass::Estimated, 0.0, &[input])
                .unwrap()
                .unavailable,
            Some(Unavailable::PrecisionLimit)
        );
    }
}
