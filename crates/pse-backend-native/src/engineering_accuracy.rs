// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Local regular-square output estimates using an already admitted sparse factor.
//! This is an evidence operation, not a nonlinear solver or a forward certificate.
mod kkt;
mod producers;
use crate::{
    ProblemError,
    solve::Execution,
    square_response::{ActionEvidence, SparseFactor},
};
pub(crate) use kkt::defer_kkt;
pub use kkt::{DeferredKktAccuracy, KktEstimates, KktOutput, KktOutputObserver, KktOutputs, KktPointArithmetic, KktValidity, estimate_kkt};
pub use producers::{Comparison, QualifiedScalar, dynamic_comparison, objective_interval};
use pse_ids::ContentHash;
use pse_model::{
    engineering_accuracy::{BoundGoal, OutputEvidence},
    generated::enums::{
        AccuracyCriterionStatus, AccuracyEvidenceInterpretation, AccuracyEvidenceMethod,
        AccuracyGoalSubject, AccuracyGoalUse, AccuracyObservation,
        AccuracyUnavailableReason,
    },
    strategy::{AccuracyClass, AccuracyEvidence, SemanticProductKey},
};
use pse_math::engineering_accuracy::{
    GoalWorkDemand, ResidualRowDemand, refinement_allowance,
};

/// Actual caller-admitted local interior/branch witness at the factor point.
#[derive(Clone, Copy, Debug)]
pub struct SquareValidity {
    /// Mathematical dependencies of the witness; must match the factor exactly.
    pub source: SemanticProductKey,
    /// Identity of the real neighborhood/branch admission, never a placeholder.
    pub witness: ContentHash,
}

/// A fresh original equality residual, including subtraction of original bounds.
#[derive(Clone, Copy, Debug)]
pub struct SquareResidual<'a> {
    /// Actual evaluation dependencies, including point, parameters and branch.
    pub source: SemanticProductKey,
    /// Original physical residual in the factor's admitted row order.
    pub values: &'a [f64],
    /// Absolute arithmetic uncertainty of each physical residual, from a matching
    /// original-point enclosure. Missing means the contribution is unavailable.
    pub residual_uncertainty: Option<&'a [f64]>,
    /// Absolute physical Jacobian arithmetic uncertainty, dense row-major
    /// (original rows × state columns), from the same source and point.
    pub jacobian_uncertainty: Option<&'a [f64]>,
}

/// One already admitted scalar output evaluated at the factor's physical point.
#[derive(Clone, Copy, Debug)]
pub struct SquareOutput<'a> {
    /// Physical goal bound to the output program and actual consumed dependencies.
    pub goal: &'a BoundGoal,
    /// Actual output/First derivative product identity.
    pub product: ContentHash,
    /// Actual output error magnitude coordinates.
    pub normalization: ContentHash,
    /// Source of both output and protected First derivatives, with branch admission.
    pub derivative_source: SemanticProductKey,
    /// Original physical output value in the goal's unit representation.
    pub value: f64,
    /// Physical output First partials per state, in the factor's state order.
    pub gradient: &'a [f64],
    /// Absolute First arithmetic uncertainty, in the same physical state order.
    /// None withholds the output; exact coordinate projections supply zero radii.
    pub gradient_uncertainty: Option<&'a [f64]>,
    /// Admitted output-level supplier/evaluator uncertainty in output units. None
    /// means a material contribution is unavailable, not an exact zero.
    pub uncertainty: Option<f64>,
}

/// Evidence from one shared action. Its reservation and work admission remain with
/// the existing factor/Execution owners; callers must not recharge it per output.
#[derive(Debug)]
pub struct SquareEstimates {
    /// Actual factor dependencies retained even when the optional operation fails.
    pub source: SemanticProductKey,
    /// Actual local admission supplied by the physical evaluator owner.
    pub validity: Option<SquareValidity>,
    /// Per-input-output evidence or a typed reason, in the input order.
    pub outputs: Vec<Result<OutputEvidence, AccuracyUnavailableReason>>,
    /// Optional non-certifying residual-row demand aligned with each output row.
    pub work_demands: Vec<Option<GoalWorkDemand>>,
    /// Backward error and actual shared backsolve count; never forward error.
    pub action: Option<ActionEvidence>,
    /// Actual physical residual correction, shared by all compatible outputs.
    pub correction: Option<Vec<f64>>,
    /// Exact operational cause when the actual sparse action did not complete.
    pub failure: Option<ProblemError>,
}

fn cause_reason(error: &ProblemError) -> AccuracyUnavailableReason {
    match error {
        ProblemError::Cancelled => AccuracyUnavailableReason::Cancelled,
        ProblemError::Limit { .. } => AccuracyUnavailableReason::ResourceLimit,
        ProblemError::Contract(_) | ProblemError::Structural { .. } => {
            AccuracyUnavailableReason::InvalidValidity
        }
        ProblemError::Numerical { .. } | ProblemError::Linear { .. } => {
            AccuracyUnavailableReason::Regularity
        }
        _ => AccuracyUnavailableReason::Failed,
    }
}

/// Obtain Estimated output errors from one `-J^-1 r` action. The caller has already
/// admitted derivatives, local branch, units and material evaluator/supplier inputs.
/// A Certified request still receives actual Estimated evidence, which the shared
/// classifier must refuse at that strength. No acceptance or specification is changed.
///
/// Scratch/result reservations are admitted by the caller before invoking this bounded
/// operation. The factor owns sparse backsolve admission and charging, once per batch.
pub fn estimate_square(
    factor: &SparseFactor,
    residual: SquareResidual<'_>,
    validity: Option<SquareValidity>,
    outputs: &[SquareOutput<'_>],
    execution: &Execution,
) -> SquareEstimates {
    use AccuracyUnavailableReason as U;
    let source = factor.key();
    let common = if residual.source != source || validity.is_none_or(|v| v.source != source) {
        Some(U::InvalidValidity)
    } else if residual.values.len() != factor.states().len()
        || residual.residual_uncertainty.is_some_and(|values| values.len() != factor.rows().len())
        || residual.jacobian_uncertainty.is_some_and(|values| {
            factor.states().len().checked_mul(factor.rows().len()) != Some(values.len())
        })
    {
        Some(U::InvalidValidity)
    } else if residual.residual_uncertainty.is_none() || residual.jacobian_uncertainty.is_none() {
        Some(U::EvaluatorUncertainty)
    } else if residual.values.iter().any(|v| !v.is_finite()) {
        Some(U::Nonfinite)
    } else if residual
        .residual_uncertainty
        .into_iter()
        .flatten()
        .chain(residual.jacobian_uncertainty.into_iter().flatten())
        .any(|v| !v.is_finite())
    {
        Some(U::Nonfinite)
    } else if residual
        .residual_uncertainty
        .into_iter()
        .flatten()
        .chain(residual.jacobian_uncertainty.into_iter().flatten())
        .any(|v| *v < 0.0)
    {
        Some(U::InvalidValidity)
    } else {
        None
    };
    let reasons = outputs
        .iter()
        .map(|output| {
            common.or_else(|| {
                if pse_model::engineering_accuracy::validate_goal(&output.goal.declaration).is_err()
                    || output.goal.source != source
                    || output.derivative_source != source
                    || output.product != output.goal.product
                    || output.normalization != output.goal.normalization
                    || output.gradient.len() != factor.states().len()
                    || output.gradient_uncertainty.is_some_and(|radii| radii.len() != factor.states().len()
                        || radii.iter().any(|radius| *radius < 0.0))
                {
                    Some(U::InvalidValidity)
                } else if output.goal.declaration.subject != AccuracyGoalSubject::SelectedOutput {
                    Some(U::Unsupported)
                } else if output.goal.declaration.observation != AccuracyObservation::Steady {
                    Some(U::UnsupportedObservation)
                } else if !output.value.is_finite()
                    || output.gradient.iter().any(|g| !g.is_finite())
                    || output.gradient_uncertainty.into_iter().flatten().any(|radius| !radius.is_finite())
                    || output
                        .uncertainty
                        .is_some_and(|u| !u.is_finite() || u < 0.0)
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
    let mut result = SquareEstimates {
        source,
        validity,
        outputs: Vec::with_capacity(outputs.len()),
        work_demands: Vec::with_capacity(outputs.len()),
        action: None,
        correction: None,
        failure: None,
    };
    if reasons.iter().all(Option::is_some) {
        result.outputs = reasons
            .into_iter()
            .map(|reason| Err(reason.unwrap_or(U::MissingEvidence)))
            .collect();
        result.work_demands.resize_with(outputs.len(), || None);
        return result;
    }
    let rhs = residual.values.iter().map(|r| -*r).collect::<Vec<_>>();
    let (correction, action) = match factor.action(&rhs, execution) {
        Ok(action) => action,
        Err(error) => {
            let reason = cause_reason(&error);
            result.outputs = reasons
                .into_iter()
                .map(|r| Err(r.unwrap_or(reason)))
                .collect();
            result.work_demands.resize_with(outputs.len(), || None);
            result.failure = Some(error);
            return result;
        }
    };
    result.action = Some(action);
    for (output, reason) in outputs.iter().zip(reasons) {
        if let Some(reason) = reason {
            result.outputs.push(Err(reason));
            result.work_demands.push(None);
            continue;
        }
        if let Err(error) = execution.check() {
            result.outputs.push(Err(cause_reason(&error)));
            result.work_demands.push(None);
            result.failure = Some(error);
            continue;
        }
        let propagated = output
            .gradient
            .iter()
            .zip(&correction)
            .map(|(gradient, delta)| gradient.abs() * delta.abs())
            .sum::<f64>();
        let residual_uncertainty = residual.residual_uncertainty.unwrap_or_default();
        let jacobian_uncertainty = residual.jacobian_uncertainty.unwrap_or_default();
        // Exact-zero interval radii require no adjoint merely to establish an exact-zero
        // arithmetic contribution. An unresolved permitted goal may still need one for
        // its work-demand allocation below.
        let arithmetic_possible = residual_uncertainty.iter().any(|value| *value > 0.0)
            || correction.iter().any(|value| *value != 0.0)
                && jacobian_uncertainty.iter().any(|value| *value > 0.0);
        let mut adjoint = None;
        if arithmetic_possible {
            match factor.transpose_action(output.gradient, execution) {
                Ok(value) => adjoint = Some(value),
                Err(error) => {
                    result.outputs.push(Err(cause_reason(&error)));
                    result.work_demands.push(None);
                    if result.failure.is_none() {
                        result.failure = Some(error);
                    }
                    continue;
                }
            }
        }
        let arithmetic_uncertainty = if let Some((multipliers, _)) = &adjoint {
            let n = factor.states().len();
            multipliers
                .iter()
                .enumerate()
                .map(|(row, multiplier)| {
                    let jacobian_row = &jacobian_uncertainty[row * n..(row + 1) * n];
                    multiplier.abs()
                        * (residual_uncertainty[row]
                            + jacobian_row
                                .iter()
                                .zip(&correction)
                                .map(|(radius, delta)| radius * delta.abs())
                                .sum::<f64>())
                })
                .sum::<f64>()
        } else {
            0.0
        };
        if !arithmetic_uncertainty.is_finite() {
            result.outputs.push(Err(U::Nonfinite));
            result.work_demands.push(None);
            continue;
        }
        // Neighbor spacing prevents an Estimated zero from claiming exactness. It
        // is explicitly an estimate floor, not a rounding-error upper bound.
        let spacing = [output.value.next_up(), output.value.next_down()]
            .into_iter()
            .filter(|neighbor| neighbor.is_finite())
            .map(|neighbor| (neighbor - output.value).abs())
            .fold(0.0, f64::max);
        let derivative_uncertainty = output.gradient_uncertainty.unwrap_or_default().iter()
            .zip(&correction).map(|(radius, delta)| radius * delta.abs()).sum::<f64>();
        // Fixed evaluator reserve is consumed once in the estimate and once when
        // allocating the remaining allowance; it is never assigned to residual rows.
        let evaluator_reserve = output.uncertainty.unwrap_or(0.0) + derivative_uncertainty + spacing;
        let error = propagated + arithmetic_uncertainty + evaluator_reserve;
        if !error.is_finite() || error <= 0.0 {
            result.outputs.push(Err(U::PrecisionLimit));
            result.work_demands.push(None);
            continue;
        }
        let goal = &output.goal.declaration;
        let output_evidence = OutputEvidence {
            target: goal.target_id, target_kind: goal.target_kind,
            quantity: goal.quantity_id, unit: goal.unit_id,
            observation: goal.observation, time: goal.time, value: Some(output.value),
            accuracy: AccuracyEvidence { product: output.product, normalization: output.normalization,
                class: AccuracyClass::Estimated, error: Some(error) },
            source, validity: validity.map(|v| v.witness),
            interpretation: AccuracyEvidenceInterpretation::OutputError,
            method: AccuracyEvidenceMethod::SquareCorrection, interval: None,
            limitation: "Estimated local first-order residual correction plus output First and sparse-adjoint residual/Jacobian arithmetic uncertainty, admitted evaluator/supplier uncertainty and representational spacing; no nonlinear remainder or global branch/error certificate".into(),
        };
        let classified = pse_math::engineering_accuracy::classify(
            output.goal,
            Some(&output_evidence),
        );
        let mut demand = None;
        if classified.status == pse_model::generated::enums::AccuracyGoalStatus::Unresolved
            && output.goal.declaration.refine
            && output.goal.declaration.required_class == AccuracyClass::Estimated
            && !(output.goal.declaration.use_policy == AccuracyGoalUse::RequireSatisfied
                && classified.criterion == AccuracyCriterionStatus::Violated)
            && let Some(allowance) = refinement_allowance(output.goal, output.value)
        {
            if output.uncertainty.is_some() {
                let remainder = allowance - evaluator_reserve - arithmetic_uncertainty;
                if remainder.is_finite() && remainder > 0.0 {
                    if adjoint.is_none() {
                        match factor.transpose_action(output.gradient, execution) {
                            Ok(value) => adjoint = Some(value),
                            Err(error) => {
                                if result.failure.is_none() {
                                    result.failure = Some(error);
                                }
                            }
                        }
                    }
                    if let Some((multipliers, adjoint_evidence)) = adjoint.as_ref() {
                            let contributions = multipliers
                                .iter()
                                .zip(residual.values)
                                .map(|(lambda, residual)| lambda.abs() * residual.abs())
                                .collect::<Vec<_>>();
                            let total = contributions.iter().sum::<f64>();
                            if total.is_finite() && total > 0.0 {
                                let mut rows = Vec::new();
                                let mut complete = true;
                                for ((row, lambda), contribution) in factor
                                    .rows()
                                    .iter()
                                    .copied()
                                    .zip(multipliers)
                                    .zip(contributions)
                                {
                                    if contribution <= 0.0 {
                                        continue;
                                    }
                                    let physical_allowance = remainder * contribution
                                        / (total * lambda.abs());
                                    if !lambda.is_finite()
                                        || *lambda == 0.0
                                        || !physical_allowance.is_finite()
                                        || physical_allowance <= 0.0
                                    {
                                        complete = false;
                                        break;
                                    }
                                    rows.push(ResidualRowDemand { row, physical_allowance });
                                }
                                if complete && !rows.is_empty() {
                                    demand = Some(GoalWorkDemand {
                                        product: output.product,
                                        source,
                                        physical_output_allowance: allowance,
                                        adjoint_backward_error: adjoint_evidence.backward_error,
                                        rows,
                                    });
                                }
                            }
                    }
                }
            }
        }
        result.outputs.push(Ok(output_evidence));
        result.work_demands.push(demand);
    }
    result.correction = Some(correction);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        OracleContract, Variable,
        quality::Tolerances,
        solve::Controls,
        square_response::{SparseRequest, SquareScope, point_key},
    };
    use faer::sparse::{Pair, SparseColMat, SymbolicSparseColMat};
    use pse_ids::SemanticId;
    use pse_math::{normalization::Normalization, binding::ObjectiveSense};
    use pse_model::{
        engineering_accuracy::AccuracyGoal,
        generated::enums::{AccuracyGoalStatus, AccuracyGoalUse, NumericalSource, NumericalTarget},
    };
    use std::sync::Arc;
    #[derive(Debug, Default)]
    struct Work {
        observations: std::sync::Mutex<Vec<crate::solve::WorkEvidence>>,
    }
    impl crate::solve::WorkAdmission for Work {
        fn admit(&self, _: crate::solve::WorkEvidence) -> Result<(), ProblemError> {
            Ok(())
        }
        fn observe(&self, work: crate::solve::WorkEvidence) -> Result<(), ProblemError> {
            self.observations.lock().unwrap().push(work);
            Ok(())
        }
    }
    fn hash(n: u8) -> ContentHash {
        ContentHash::from_bytes([n; 32])
    }
    fn id(n: u8) -> SemanticId {
        SemanticId::from_bytes([n; 16])
    }
    fn factor(execution: &Execution) -> SparseFactor {
        factor_with_bound(execution, 0.0)
    }
    fn factor_with_bound(execution: &Execution, bound: f64) -> SparseFactor {
        let contract = OracleContract {
            identity: hash(1),
            variables: vec![Variable {
                id: id(1),
                lower: -10.0,
                upper: 10.0,
            }],
            rows: vec![id(2)],
            derivatives: pse_kernels::DerivativeOrder::First,
            smoothness: pse_kernels::DerivativeOrder::First,
        };
        let (pattern, _) =
            SymbolicSparseColMat::try_new_from_indices(1, 1, &[Pair::new(0, 0)]).unwrap();
        let scope =
            SquareScope::admit(&contract, pattern.as_ref(), &[(bound, bound)], None).unwrap();
        let normalization = Normalization {
            variables: vec![1.0],
            rows: vec![1.0],
            objective: 1.0,
        };
        let key = SemanticProductKey {
            structure: contract.identity,
            binding: hash(2),
            numerical_policy: Some(hash(3)),
            normalization: Some(normalization.key()),
            point: Some(point_key(&[1.0])),
            parameters: Some(hash(4)),
            derivation: None,
            branch: Some(hash(5)),
            accuracy: Some(hash(6)),
        };
        SparseFactor::prepare(
            SparseRequest {
                scope: &scope,
                point: &[1.0],
                values: &[0.001],
                tolerances: &Tolerances {
                    variables: vec![1e-8],
                    rows: vec![0.002],
                    integrality: 1e-8,
                },
                normalization: &normalization,
                key,
                bytes: 1 << 20,
            },
            || Ok(SparseColMat::new(pattern, vec![0.001])),
            execution,
        )
        .unwrap()
    }
    fn goal(factor: &SparseFactor, n: u8, class: AccuracyClass) -> BoundGoal {
        BoundGoal {
            declaration: AccuracyGoal {
                goal_id: id(n).into(),
                model_id: None,
                case_id: None,
                instance_id: None,
                fit_id: None,
                target_id: id(n),
                target_kind: NumericalTarget::Observable,
                quantity_id: id(3),
                unit_id: id(4),
                subject: AccuracyGoalSubject::SelectedOutput,
                observation: AccuracyObservation::Steady,
                time: None,
                resolution: Some(3.0),
                criterion_lower: None,
                criterion_upper: None,
                required_class: class,
                use_policy: AccuracyGoalUse::Assess,
                refine: true,
                source: NumericalSource::Analysis,
                priority: 0,
                provenance: "analytic affine output accuracy".into(),
            },
            source: factor.key(),
            product: hash(n),
            normalization: hash(8),
        }
    }
    fn output<'a>(goal: &'a BoundGoal, gradient: &'a [f64]) -> SquareOutput<'a> {
        SquareOutput {
            goal,
            product: goal.product,
            normalization: goal.normalization,
            derivative_source: goal.source,
            value: 2.0,
            gradient,
            gradient_uncertainty: Some(&[0.0]),
            uncertainty: Some(0.0),
        }
    }
    #[test]
    fn engineering_accuracy_square_affine_amplification_shares_action_and_remains_estimated() {
        let work = Arc::new(Work::default());
        let mut execution = Execution::new(Arc::default(), &Controls::default());
        execution.work_admission = Some(work.clone());
        let factor = factor(&execution);
        let estimated = goal(&factor, 10, AccuracyClass::Estimated);
        let certified = goal(&factor, 11, AccuracyClass::Certified);
        // Independent affine oracle: .001*x=0 at x=1 has residual .001,
        // so exact state error is 1 and q=2*x has exact output error 2.
        let outputs = [output(&estimated, &[2.0]), output(&certified, &[2.0])];
        let batch = estimate_square(
            &factor,
            SquareResidual {
                source: factor.key(),
                values: &[0.001],
                residual_uncertainty: Some(&[0.0]),
                jacobian_uncertainty: Some(&[0.0]),
            },
            Some(SquareValidity {
                source: factor.key(),
                witness: hash(9),
            }),
            &outputs,
            &execution,
        );
        assert_eq!(batch.correction.as_ref().unwrap(), &[-1.0]);
        let action = batch.action.unwrap();
        assert_eq!(action.backsolves, 1);
        assert!(action.backward_error <= action.limit);
        assert!(batch.failure.is_none());
        assert!(batch.work_demands.iter().all(Option::is_none));
        let observations = work.observations.lock().unwrap();
        // One construction and one action, even though two goals consume it.
        assert_eq!(observations.len(), 2);
        assert_eq!(observations[0].factorizations, Some(1));
        assert_eq!(observations[1].factorizations, Some(0));
        assert_eq!(observations[1].iterations, Some(0));
        for evidence in &batch.outputs {
            let evidence = evidence.as_ref().unwrap();
            assert_eq!(evidence.accuracy.class, AccuracyClass::Estimated);
            assert!((evidence.accuracy.error.unwrap() - 2.0).abs() < 1e-14);
            assert_eq!(evidence.source.point, factor.key().point);
            assert_eq!(evidence.validity, Some(hash(9)));
        }
        assert_eq!(
            pse_math::engineering_accuracy::classify(&estimated, batch.outputs[0].as_ref().ok())
                .status,
            AccuracyGoalStatus::Satisfied
        );
        assert_eq!(
            pse_math::engineering_accuracy::classify(&certified, batch.outputs[1].as_ref().ok())
                .unavailable,
            Some(AccuracyUnavailableReason::InsufficientStrength)
        );
    }
    #[test]
    fn unresolved_refinable_estimated_goal_retains_adjoint_row_demand() {
        let execution = Execution::new(Arc::default(), &Controls::default());
        let factor = factor(&execution);
        let mut declaration = goal(&factor, 12, AccuracyClass::Estimated);
        declaration.declaration.resolution = Some(0.5);
        let output = output(&declaration, &[2.0]);
        let batch = estimate_square(
            &factor,
            SquareResidual { source: factor.key(), values: &[0.001], residual_uncertainty: Some(&[0.0]), jacobian_uncertainty: Some(&[0.0]) },
            Some(SquareValidity { source: factor.key(), witness: hash(9) }),
            &[output],
            &execution,
        );
        assert!(batch.failure.is_none());
        let demand = batch.work_demands[0].as_ref().expect("unresolved goal demand");
        assert_eq!(demand.product, declaration.product);
        assert_eq!(demand.source, factor.key());
        assert_eq!(demand.rows.len(), 1);
        assert_eq!(demand.rows[0].row, id(2));
        // lambda=2/.001 and the residual contribution is 2, so the remaining
        // output budget is assigned to the row in physical residual units.
        assert!((demand.rows[0].physical_allowance - 0.00025).abs() < 1e-15);
        assert!(demand.adjoint_backward_error.is_finite());
    }
    #[test]
    fn fixed_supplier_uncertainty_exhausts_allowance_without_adjoint_work() {
        let work = Arc::new(Work::default());
        let mut execution = Execution::new(Arc::default(), &Controls::default());
        execution.work_admission = Some(work.clone());
        let factor = factor(&execution);
        let mut declaration = goal(&factor, 13, AccuracyClass::Estimated);
        declaration.declaration.resolution = Some(0.1);
        let mut output = output(&declaration, &[2.0]);
        output.uncertainty = Some(0.2);
        let batch = estimate_square(
            &factor,
            SquareResidual { source: factor.key(), values: &[0.001], residual_uncertainty: Some(&[0.0]), jacobian_uncertainty: Some(&[0.0]) },
            Some(SquareValidity { source: factor.key(), witness: hash(9) }),
            &[output],
            &execution,
        );
        assert!(batch.outputs[0].is_ok());
        assert_eq!(batch.outputs[0].as_ref().unwrap().accuracy.class, AccuracyClass::Estimated);
        assert!(batch.work_demands[0].is_none());
        // One construction and the forward correction only; the fixed uncertainty
        // leaves no output budget for a transposed solve.
        assert_eq!(work.observations.lock().unwrap().len(), 2);
    }

    #[test]
    fn residual_and_jacobian_roundoff_propagate_once_and_exhaust_refinement_allowance() {
        let work = Arc::new(Work::default());
        let mut execution = Execution::new(Arc::default(), &Controls::default());
        execution.work_admission = Some(work.clone());
        let factor = factor(&execution);
        let mut declaration = goal(&factor, 14, AccuracyClass::Estimated);
        declaration.declaration.resolution = Some(0.01);
        let output = output(&declaration, &[2.0]);
        // For J=.001, r=.001, d=-1 and lambda=2000.  The
        // first-order arithmetic contribution is 2000*(1e-5 + 1e-5*1)=.04.
        let batch = estimate_square(
            &factor,
            SquareResidual {
                source: factor.key(),
                values: &[0.001],
                residual_uncertainty: Some(&[1e-5]),
                jacobian_uncertainty: Some(&[1e-5]),
            },
            Some(SquareValidity { source: factor.key(), witness: hash(9) }),
            &[output],
            &execution,
        );
        let evidence = batch.outputs[0].as_ref().unwrap();
        assert!((evidence.accuracy.error.unwrap() - 2.04).abs() < 1e-10);
        assert!(batch.work_demands[0].is_none());
        assert!(batch.failure.is_none());
        // Factor construction, forward correction, then the one adjoint shared by
        // arithmetic propagation and the (exhausted) demand calculation.
        assert_eq!(work.observations.lock().unwrap().len(), 3);
    }

    #[test]
    fn output_first_uncertainty_is_a_fixed_reserve_once_and_missing_is_unavailable() {
        let work = Arc::new(Work::default());
        let mut execution = Execution::new(Arc::default(), &Controls::default());
        execution.work_admission = Some(work.clone());
        let factor = factor(&execution);
        let mut declaration = goal(&factor, 17, AccuracyClass::Estimated);
        declaration.declaration.resolution = Some(0.01);
        let mut output = output(&declaration, &[2.0]);
        output.gradient_uncertainty = Some(&[0.04]);
        let residual = SquareResidual { source: factor.key(), values: &[0.001],
            residual_uncertainty: Some(&[0.0]), jacobian_uncertainty: Some(&[0.0]) };
        let validity = Some(SquareValidity { source: factor.key(), witness: hash(9) });
        // J=.001 and r=.001 give d=-1: the derivative reserve is .04*1.
        let batch = estimate_square(&factor, residual, validity, &[output], &execution);
        assert!((batch.outputs[0].as_ref().unwrap().accuracy.error.unwrap() - 2.04).abs() < 1e-10);
        assert!(batch.work_demands[0].is_none());
        assert_eq!(work.observations.lock().unwrap().len(), 2);
        output.gradient_uncertainty = None;
        let withheld = estimate_square(&factor, residual, validity, &[output], &execution);
        assert!(matches!(withheld.outputs[0], Err(AccuracyUnavailableReason::EvaluatorUncertainty)));
        assert!(withheld.action.is_none());
        assert_eq!(work.observations.lock().unwrap().len(), 2);
    }

    #[test]
    fn residual_roundoff_is_retained_when_the_nominal_correction_cancels_to_zero() {
        let execution = Execution::new(Arc::default(), &Controls::default());
        let factor = factor(&execution);
        let goal = goal(&factor, 15, AccuracyClass::Estimated);
        let batch = estimate_square(
            &factor,
            SquareResidual {
                source: factor.key(),
                values: &[0.0],
                residual_uncertainty: Some(&[1e-5]),
                jacobian_uncertainty: Some(&[0.0]),
            },
            Some(SquareValidity { source: factor.key(), witness: hash(9) }),
            &[output(&goal, &[2.0])],
            &execution,
        );
        assert_eq!(batch.correction.as_ref().unwrap(), &[0.0]);
        let error = batch.outputs[0].as_ref().unwrap().accuracy.error.unwrap();
        assert!(error >= 0.02);
    }

    #[test]
    fn missing_point_arithmetic_uncertainty_is_unavailable_not_spacing() {
        let execution = Execution::new(Arc::default(), &Controls::default());
        let factor = factor(&execution);
        let goal = goal(&factor, 16, AccuracyClass::Estimated);
        let batch = estimate_square(
            &factor,
            SquareResidual {
                source: factor.key(),
                values: &[0.001],
                residual_uncertainty: Some(&[0.0]),
                jacobian_uncertainty: None,
            },
            Some(SquareValidity { source: factor.key(), witness: hash(9) }),
            &[output(&goal, &[2.0])],
            &execution,
        );
        assert!(matches!(
            batch.outputs[0],
            Err(AccuracyUnavailableReason::EvaluatorUncertainty)
        ));
        assert!(batch.action.is_none());
    }

    #[test]
    fn engineering_accuracy_square_withholds_missing_uncertainty_and_stale_branch_without_action() {
        let execution = Execution::new(Arc::default(), &Controls::default());
        let factor = factor(&execution);
        let goal = goal(&factor, 10, AccuracyClass::Estimated);
        let validity = Some(SquareValidity {
            source: factor.key(),
            witness: hash(9),
        });
        let mut missing = output(&goal, &[2.0]);
        missing.uncertainty = None;
        let batch = estimate_square(
            &factor,
            SquareResidual {
                source: factor.key(),
                values: &[0.001],
                residual_uncertainty: Some(&[0.0]),
                jacobian_uncertainty: Some(&[0.0]),
            },
            validity,
            &[missing],
            &execution,
        );
        assert!(matches!(
            batch.outputs[0],
            Err(AccuracyUnavailableReason::EvaluatorUncertainty)
        ));
        assert!(batch.action.is_none());
        assert!(batch.work_demands[0].is_none());
        let mut stale = output(&goal, &[2.0]);
        stale.derivative_source.branch = Some(hash(99));
        let batch = estimate_square(
            &factor,
            SquareResidual {
                source: factor.key(),
                values: &[0.001],
                residual_uncertainty: Some(&[0.0]),
                jacobian_uncertainty: Some(&[0.0]),
            },
            validity,
            &[stale],
            &execution,
        );
        assert!(matches!(
            batch.outputs[0],
            Err(AccuracyUnavailableReason::InvalidValidity)
        ));
        assert!(batch.action.is_none());
        assert!(batch.work_demands[0].is_none());
    }
    #[test]
    fn engineering_accuracy_square_consumes_residual_after_original_equality_bound() {
        let execution = Execution::new(Arc::default(), &Controls::default());
        let factor = factor_with_bound(&execution, 0.0005);
        let goal = goal(&factor, 10, AccuracyClass::Estimated);
        // .001*x=.0005 has exact root .5. At x=1, subtracting the original
        // equality bound gives residual .0005 and exact q=2*x error 1.
        let batch = estimate_square(
            &factor,
            SquareResidual {
                source: factor.key(),
                values: &[0.0005],
                residual_uncertainty: Some(&[0.0]),
                jacobian_uncertainty: Some(&[0.0]),
            },
            Some(SquareValidity {
                source: factor.key(),
                witness: hash(9),
            }),
            &[output(&goal, &[2.0])],
            &execution,
        );
        assert_eq!(batch.correction.as_ref().unwrap(), &[-0.5]);
        assert!((batch.outputs[0].as_ref().unwrap().accuracy.error.unwrap() - 1.0).abs() < 1e-14);
    }
    #[test]
    fn engineering_accuracy_square_zero_correction_retains_spacing_and_cancellation_cause() {
        let execution = Execution::new(Arc::default(), &Controls::default());
        let factor = factor(&execution);
        let goal = goal(&factor, 10, AccuracyClass::Estimated);
        let validity = Some(SquareValidity {
            source: factor.key(),
            witness: hash(9),
        });
        let outputs = [output(&goal, &[0.0])];
        let batch = estimate_square(
            &factor,
            SquareResidual {
                source: factor.key(),
                values: &[0.0],
                residual_uncertainty: Some(&[0.0]),
                jacobian_uncertainty: Some(&[0.0]),
            },
            validity,
            &outputs,
            &execution,
        );
        assert!(batch.outputs[0].as_ref().unwrap().accuracy.error.unwrap() > 0.0);
        execution
            .cancel
            .store(true, std::sync::atomic::Ordering::Release);
        let cancelled = estimate_square(
            &factor,
            SquareResidual {
                source: factor.key(),
                values: &[0.0],
                residual_uncertainty: Some(&[0.0]),
                jacobian_uncertainty: Some(&[0.0]),
            },
            validity,
            &outputs,
            &execution,
        );
        assert!(matches!(
            cancelled.outputs[0],
            Err(AccuracyUnavailableReason::Cancelled)
        ));
        assert!(matches!(cancelled.failure, Some(ProblemError::Cancelled)));
    }

    #[test]
    fn engineering_accuracy_objective_interval_retains_primal_radius_and_actual_strength() {
        let execution = Execution::new(Arc::default(), &Controls::default());
        let factor = factor(&execution);
        let mut goal = goal(&factor, 11, AccuracyClass::Certified);
        goal.declaration.subject = AccuracyGoalSubject::OptimalObjective;
        goal.declaration.target_kind = NumericalTarget::Objective;
        let primal = QualifiedScalar {
            source: goal.source, value: 10., uncertainty: Some(0.25),
            class: AccuracyClass::Estimated, validity: hash(12),
        };
        let dual = QualifiedScalar {
            source: goal.source, value: 8., uncertainty: Some(0.5),
            class: AccuracyClass::Certified, validity: hash(13),
        };
        let actual = objective_interval(&goal, primal, dual, ObjectiveSense::Minimize).unwrap();
        assert_eq!(actual.interval, Some((7.5, 10.25)));
        assert_eq!(actual.value, Some(10.));
        assert!(actual.accuracy.error.unwrap() >= 2.5);
        assert!(actual.accuracy.error.unwrap() < 2.51);
        assert_eq!(actual.accuracy.class, AccuracyClass::Estimated);
        assert_eq!(pse_math::engineering_accuracy::classify(&goal, Some(&actual)).status,
            AccuracyGoalStatus::Unresolved);
        assert!(matches!(objective_interval(&goal, primal, QualifiedScalar {
            uncertainty: None, ..dual
        }, ObjectiveSense::Minimize), Err(AccuracyUnavailableReason::EvaluatorUncertainty)));
        assert!(matches!(objective_interval(&goal, primal, QualifiedScalar {
            value: 11., ..dual
        }, ObjectiveSense::Minimize), Err(AccuracyUnavailableReason::InvalidValidity)));
        goal.declaration.subject = AccuracyGoalSubject::SelectedOutput;
        goal.declaration.target_kind = NumericalTarget::Observable;
        assert!(matches!(objective_interval(&goal, primal, dual, ObjectiveSense::Minimize),
            Err(AccuracyUnavailableReason::UnsupportedObservation)));
    }
}
