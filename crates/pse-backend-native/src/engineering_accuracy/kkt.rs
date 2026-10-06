// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Local output estimates from the actual qualified stationary KKT residual.
use crate::{
    ProblemError,
    kkt::{Curvature, KktActionEvidence, KktFactor, Licq, Unavailable},
    solve::{Execution, Qualification, SolveReport},
};
use pse_ids::ContentHash;
use pse_math::{factorable::PointArithmeticRow, implicit::ProofInterval};
use pse_model::{
    engineering_accuracy::{BoundGoal, OutputEvidence},
    generated::enums::{
        AccuracyEvidenceInterpretation as I, AccuracyEvidenceMethod as M, AccuracyGoalSubject as S,
        AccuracyObservation as O, AccuracyUnavailableReason as U,
    },
    strategy::{AccuracyClass, AccuracyEvidence, SemanticProductKey},
};
use std::sync::Arc;

/// A real original-domain/branch interior witness supplied by the evaluator owner.
#[derive(Clone, Copy, Debug)]
pub struct KktValidity {
    /// Exact scientific dependencies and actual candidate point.
    pub source: SemanticProductKey,
    /// Identity of the actual domain/branch admission.
    pub witness: ContentHash,
}

/// One protected output evaluated with its actual physical First partials.
#[derive(Clone, Debug)]
pub struct KktOutput {
    /// Goal and consumed output product/normalization identities.
    pub goal: BoundGoal,
    /// Actual consumed derivative dependencies, including point and branch.
    pub derivative_source: SemanticProductKey,
    /// Physical output at the reported candidate, in the goal's declared unit.
    pub value: f64,
    /// Physical output partials in the report's original variable order.
    pub gradient: Vec<f64>,
    /// Same-point absolute First-derivative arithmetic radii in original coordinates.
    /// Missing derivative uncertainty prevents the local estimate; it never becomes zero.
    pub gradient_uncertainty: Option<Vec<f64>>,
    /// Separately admitted supplier/evaluator uncertainty in output units. Missing
    /// material uncertainty prevents this estimate; it never becomes zero.
    pub uncertainty: Option<f64>,
}

/// Outputs and original domain admission observed at the actual recovered candidate.
#[derive(Clone, Debug)]
pub struct KktOutputs {
    /// Actual branch/interior admission, independent of native stationarity.
    pub validity: Option<KktValidity>,
    /// Same-point original-domain arithmetic enclosures. Missing or incomplete
    /// Second evidence prevents a correction estimate; it never becomes zero.
    pub arithmetic: Option<KktPointArithmetic>,
    /// Compatible goals evaluated by the protected output owner.
    pub outputs: Vec<KktOutput>,
}

/// Original-source point enclosures needed to bound arithmetic in the actual KKT
/// residual and matrix. Arrays retain the point program's physical output order.
#[derive(Clone, Debug)]
pub struct KktPointArithmetic {
    /// Exact point-stamped context of the source projection and factor.
    pub source: SemanticProductKey,
    /// Identity of the checked source projection that supplied these intervals.
    pub projection: ContentHash,
    /// Original row output positions, one per original constraint row.
    pub rows: Vec<PointArithmeticRow>,
    /// Original objective output position, when present.
    pub objective: Option<usize>,
    /// Outward original-point value intervals.
    pub values: Vec<ProofInterval>,
    /// Dense output-by-variable First intervals.
    pub jacobian: Vec<ProofInterval>,
    /// Dense output-by-variable-by-variable Second intervals. `None` means Second
    /// arithmetic was not established, never an exact-zero Hessian.
    pub hessian: Option<Vec<ProofInterval>>,
}

/// Per-point deferred-output owner. Native qualification alone never launches
/// protected goal work: the owning workflow retains this bounded artifact until
/// all original model checks have completed.
pub trait KktOutputObserver: std::fmt::Debug {
    /// Frozen actual scientific context, with no proposed future point substituted.
    fn source(&self) -> SemanticProductKey;
    /// Requested protected outputs, retained even if qualification prevents evaluation.
    fn goals(&self) -> &[BoundGoal];
    /// Reserve and retain the step's actual factor in this point's ephemeral owner.
    /// No protected evaluation or correction action may run here.
    /// # Errors
    /// Cancellation or finite storage admission failed.
    fn defer(
        &mut self,
        artifact: DeferredKktAccuracy,
        execution: &Execution,
    ) -> Result<(), ProblemError>;
}

/// One point's pending output operation. The existing library factor is never
/// serialized or put into the report; the workflow holds it under a finite owner
/// until original-model admission permits output evidence, or drops it on refusal.
#[derive(Debug)]
pub struct DeferredKktAccuracy {
    factor: KktFactor,
    source: SemanticProductKey,
    goals: Vec<BoundGoal>,
}
impl DeferredKktAccuracy {
    /// Actual point-stamped scientific context of the original factor.
    pub fn source(&self) -> SemanticProductKey {
        self.source
    }
    /// Original requested goals, bound to the actual factor point.
    pub fn goals(&self) -> &[BoundGoal] {
        &self.goals
    }
    /// Conservative retained factor and goal metadata extent. Overflow saturates
    /// so the ordinary finite reservation owner must refuse the allocation.
    pub fn bytes(&self) -> usize {
        self.factor
            .bytes()
            .saturating_add(size_of::<Self>())
            .saturating_add(self.goals.capacity().saturating_mul(size_of::<BoundGoal>()))
            .saturating_add(self.goals.iter().fold(0usize, |bytes, goal| {
                bytes.saturating_add(goal.declaration.provenance.capacity())
            }))
    }
    /// Consume this artifact after full original-model admission and protected
    /// output observation. Point, multipliers, normalization and qualification are
    /// checked again; no changed candidate may consume the old stationary factor.
    pub fn estimate(
        self,
        report: &SolveReport,
        observed: KktOutputs,
        execution: &Execution,
    ) -> KktEstimates {
        if observed.outputs.len() != self.goals.len()
            || observed
                .outputs
                .iter()
                .zip(&self.goals)
                .any(|(output, goal)| output.goal != *goal)
        {
            return withheld(
                self.source,
                self.goals,
                U::InvalidValidity,
                Some(Arc::new(ProblemError::Contract(
                    "deferred KKT output observer changed its requested goals".into(),
                ))),
            );
        }
        estimate_kkt(Some(&self.factor), report, self.source, observed, execution)
    }
}

/// One shared actual-RHS correction and its output estimates. Only Estimated local
/// claims are produced; original qualification and output strength remain independent.
#[derive(Clone, Debug)]
pub struct KktEstimates {
    /// Actual point-stamped original scientific context.
    pub source: SemanticProductKey,
    /// Actual local domain/branch admission, when available.
    pub validity: Option<KktValidity>,
    /// Goal identities retained in the same order as the output results.
    pub goals: Vec<BoundGoal>,
    /// Each actual estimate or the typed reason it was withheld.
    pub outputs: Vec<Result<OutputEvidence, U>>,
    /// Actual-RHS backward-error receipt for the single direct FERAL substitution.
    pub action: Option<KktActionEvidence>,
    /// Original physical KKT correction, including multiplier coordinates.
    pub correction: Option<Vec<f64>>,
    /// Exact operational cause; no native failure becomes output accuracy.
    pub failure: Option<Arc<ProblemError>>,
}

fn local_reason(report: &SolveReport) -> Option<U> {
    if report.validation_failure().is_some()
        || !report
            .quality
            .as_ref()
            .is_some_and(crate::quality::Quality::feasible)
        || !matches!(
            report.qualification,
            Qualification::Stationary
                | Qualification::OptimalWithinTolerance
                | Qualification::GapQualified
        )
        || !report
            .evidence
            .kkt
            .as_ref()
            .is_some_and(|kkt| kkt.stationarity == Some(true) && kkt.complementarity == Some(true))
    {
        return Some(U::InvalidValidity);
    }
    match &report.evidence.local {
        Some(Ok(point))
            if point.licq == Licq::Independent
                && point.weakly_active() == 0
                && point.curvature == Curvature::Sufficient =>
        {
            None
        }
        Some(Ok(_)) => Some(U::Regularity),
        Some(Err(Unavailable::Hessian)) => Some(U::Unsupported),
        Some(Err(Unavailable::Limit { .. })) => Some(U::ResourceLimit),
        Some(Err(Unavailable::Failed(error))) => Some(super::cause_reason(error)),
        Some(Err(_)) => Some(U::InvalidValidity),
        None => Some(U::MissingEvidence),
    }
}

fn withheld(
    source: SemanticProductKey,
    goals: Vec<BoundGoal>,
    reason: U,
    failure: Option<Arc<ProblemError>>,
) -> KktEstimates {
    KktEstimates {
        source,
        validity: None,
        outputs: vec![Err(reason); goals.len()],
        goals,
        action: None,
        correction: None,
        failure,
    }
}

/// Retain the existing native factor without evaluating outputs or applying any
/// correction. The workflow controls when full original admission permits use.
/// A missing native prerequisite retains all goals with the actual typed reason.
pub(crate) fn defer_kkt(
    observer: &mut dyn KktOutputObserver,
    factor: Option<KktFactor>,
    report: &SolveReport,
    execution: &Execution,
) -> Option<KktEstimates> {
    let mut source = observer.source();
    source.point = report
        .candidate
        .as_ref()
        .map(|candidate| crate::square_response::point_key(&candidate.primal));
    let goals = observer
        .goals()
        .iter()
        .cloned()
        .map(|mut goal| {
            goal.source = source;
            goal
        })
        .collect::<Vec<_>>();
    if let Some(reason) = local_reason(report) {
        return Some(withheld(source, goals, reason, None));
    }
    let Some(factor) = factor else {
        return Some(withheld(source, goals, U::MissingEvidence, None));
    };
    if source.point != Some(factor.point())
        || source.normalization != Some(factor.normalization())
        || !factor.matches_report(report)
    {
        return Some(withheld(source, goals, U::InvalidValidity, None));
    }
    if let Err(error) = execution.check() {
        return Some(withheld(
            source,
            goals,
            super::cause_reason(&error),
            Some(Arc::new(error)),
        ));
    }
    let fallback = goals.clone();
    match observer.defer(
        DeferredKktAccuracy {
            factor,
            source,
            goals,
        },
        execution,
    ) {
        Ok(()) => None,
        Err(error) => Some(withheld(
            source,
            fallback,
            super::cause_reason(&error),
            Some(Arc::new(error)),
        )),
    }
}

/// Apply one physical `-K^-1 r` action to fresh original stationarity and active
/// feasibility residuals, shared by compatible outputs. `KktPoint::residual` is never
/// read: that field diagnoses an unrelated consistent system.
///
/// The caller reserves scratch/results before entry; the factor's bounded Execution
/// admits and charges the single actual correction action.
pub fn estimate_kkt(
    factor: Option<&KktFactor>,
    report: &SolveReport,
    source: SemanticProductKey,
    observed: KktOutputs,
    execution: &Execution,
) -> KktEstimates {
    let mut common = local_reason(report)
        .or_else(|| {
            let factor = factor?;
            if !factor.matches_report(report)
                || source.point != Some(factor.point())
                || source.normalization != Some(factor.normalization())
                || observed
                    .validity
                    .is_none_or(|validity| validity.source != source)
            {
                Some(U::InvalidValidity)
            } else {
                None
            }
        })
        .or_else(|| factor.is_none().then_some(U::MissingEvidence));
    let (radii, arithmetic_reason, arithmetic_failure) = if common.is_some() {
        (None, None, None)
    } else {
        match (factor, observed.arithmetic.as_ref()) {
            (_, None) => (None, Some(U::EvaluatorUncertainty), None),
            (None, Some(_)) => (None, Some(U::MissingEvidence), None),
            (Some(factor), Some(proof)) => match factor.arithmetic_radii(report, source, proof) {
                Ok(Some(radii)) => (Some(radii), None, None),
                Ok(None) => (None, Some(U::EvaluatorUncertainty), None),
                Err(error) => {
                    let reason = match &error {
                        ProblemError::Contract(_) => U::InvalidValidity,
                        ProblemError::Numerical { .. } | ProblemError::Linear { .. } => {
                            U::Nonfinite
                        }
                        ProblemError::Limit { .. } => U::ResourceLimit,
                        ProblemError::Cancelled => U::Cancelled,
                        ProblemError::Unsupported(_) => U::Unsupported,
                        _ => U::Failed,
                    };
                    (None, Some(reason), Some(Arc::new(error)))
                }
            },
        }
    };
    common = common.or(arithmetic_reason);
    let reasons = observed
        .outputs
        .iter()
        .map(|output| {
            common.or_else(|| {
                if pse_model::engineering_accuracy::validate_goal(&output.goal.declaration).is_err()
                    || output.goal.source != source
                    || output.derivative_source != source
                    || output.gradient.len() != report.variables.len()
                {
                    Some(U::InvalidValidity)
                } else if output.goal.declaration.subject != S::SelectedOutput {
                    Some(U::Unsupported)
                } else if output.goal.declaration.observation != O::Steady {
                    Some(U::UnsupportedObservation)
                } else if !output.value.is_finite()
                    || output.gradient.iter().any(|value| !value.is_finite())
                    || output.gradient_uncertainty.as_ref().is_some_and(|values| {
                        values.len() != report.variables.len()
                            || values
                                .iter()
                                .any(|value| !value.is_finite() || *value < 0.0)
                    })
                    || output
                        .uncertainty
                        .is_some_and(|error| !error.is_finite() || error < 0.)
                {
                    Some(U::Nonfinite)
                } else if output.uncertainty.is_none() || output.gradient_uncertainty.is_none() {
                    Some(U::EvaluatorUncertainty)
                } else {
                    None
                }
            })
        })
        .collect::<Vec<_>>();
    let mut result = KktEstimates {
        source,
        validity: observed.validity,
        goals: observed
            .outputs
            .iter()
            .map(|output| output.goal.clone())
            .collect(),
        outputs: Vec::with_capacity(observed.outputs.len()),
        action: None,
        correction: None,
        failure: arithmetic_failure,
    };
    if reasons.iter().all(Option::is_some) {
        result.outputs = reasons
            .into_iter()
            .map(|reason| Err(reason.unwrap_or(U::MissingEvidence)))
            .collect();
        return result;
    }
    let action = factor
        .ok_or_else(|| ProblemError::Contract("KKT output factor missing".into()))
        .and_then(|factor| {
            factor.original_residual(report).and_then(|residual| {
                factor.correction_action(
                    &residual.iter().map(|value| -*value).collect::<Vec<_>>(),
                    execution,
                )
            })
        });
    let (correction, receipt) = match action {
        Ok(action) => action,
        Err(error) => {
            let reason = super::cause_reason(&error);
            result.outputs = reasons
                .into_iter()
                .map(|prior| Err(prior.unwrap_or(reason)))
                .collect();
            result.failure = Some(Arc::new(error));
            return result;
        }
    };
    result.action = Some(receipt);
    for (output, reason) in observed.outputs.into_iter().zip(reasons) {
        if let Some(reason) = reason {
            result.outputs.push(Err(reason));
            continue;
        }
        if let Err(error) = execution.check() {
            result.outputs.push(Err(super::cause_reason(&error)));
            result.failure = Some(Arc::new(error));
            continue;
        }
        let propagated = output
            .gradient
            .iter()
            .zip(&correction)
            .map(|(gradient, delta)| gradient.abs() * delta.abs())
            .sum::<f64>();
        let gradient_error = output
            .gradient_uncertainty
            .as_ref()
            .map_or(0.0, |uncertainty| {
                uncertainty
                    .iter()
                    .zip(&correction)
                    .map(|(radius, delta)| radius * delta.abs())
                    .sum::<f64>()
            });
        let mut arithmetic_error = 0.0;
        if let Some(radii) = &radii
            && output.gradient.iter().any(|value| *value != 0.0)
            && (radii.residual.iter().any(|value| *value > 0.0)
                || radii.matrix.iter().any(|value| *value > 0.0))
        {
            let Some(factor) = factor else {
                result.outputs.push(Err(U::MissingEvidence));
                continue;
            };
            // The physical KKT matrix includes active multiplier coordinates.
            // A selected output depends on primal coordinates only, so its
            // corresponding multiplier partials are exactly zero.
            let mut rhs = vec![0.0; correction.len()];
            rhs[..output.gradient.len()].copy_from_slice(&output.gradient);
            let (influence, adjoint_receipt) = match factor.correction_action(&rhs, execution) {
                Ok(action) => action,
                Err(error) => {
                    result.outputs.push(Err(super::cause_reason(&error)));
                    if result.failure.is_none() {
                        result.failure = Some(Arc::new(error));
                    }
                    continue;
                }
            };
            let dim = influence.len();
            arithmetic_error = influence
                .iter()
                .enumerate()
                .map(|(row, multiplier)| {
                    let matrix_start = row * dim;
                    multiplier.abs()
                        * (radii.residual[row]
                            + radii.matrix[matrix_start..matrix_start + dim]
                                .iter()
                                .zip(&correction)
                                .map(|(radius, delta)| radius * delta.abs())
                                .sum::<f64>())
                })
                .sum::<f64>();
            if !arithmetic_error.is_finite() {
                result.outputs.push(Err(U::Nonfinite));
                continue;
            }
            if let Some(receipt) = result.action.as_mut() {
                receipt.action_invocations = receipt
                    .action_invocations
                    .saturating_add(adjoint_receipt.action_invocations);
                receipt.backsolves = receipt
                    .backsolves
                    .zip(adjoint_receipt.backsolves)
                    .and_then(|(left, right)| left.checked_add(right));
                receipt.backward_error = receipt.backward_error.max(adjoint_receipt.backward_error);
                receipt.backward_error_limit = receipt
                    .backward_error_limit
                    .min(adjoint_receipt.backward_error_limit);
            }
        }
        let spacing = [output.value.next_up(), output.value.next_down()]
            .into_iter()
            .filter(|neighbor| neighbor.is_finite())
            .map(|neighbor| (neighbor - output.value).abs())
            .fold(0., f64::max);
        let error = propagated
            + gradient_error
            + arithmetic_error
            + output.uncertainty.unwrap_or(0.)
            + spacing;
        if !error.is_finite() || error <= 0. {
            result.outputs.push(Err(U::PrecisionLimit));
            continue;
        }
        let goal = &output.goal.declaration;
        result.outputs.push(Ok(OutputEvidence { target: goal.target_id, target_kind: goal.target_kind,
            quantity: goal.quantity_id, unit: goal.unit_id, observation: goal.observation,
            time: goal.time, value: Some(output.value), accuracy: AccuracyEvidence {
                product: output.goal.product, normalization: output.goal.normalization,
                class: AccuracyClass::Estimated, error: Some(error) }, source,
            validity: result.validity.map(|validity| validity.witness), interpretation: I::OutputError,
            method: M::KktCorrection, interval: None,
            limitation: "Estimated local first-order stationary KKT correction plus output-gradient, same-point residual and matrix arithmetic uncertainty, admitted evaluator/supplier uncertainty and representational spacing; no global optimality or nonlinear remainder certificate".into() }));
    }
    result.correction = Some(correction);
    result
}
