// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Same-point exact-real Value/First/Second enclosures for the original KKT system.
use super::*;
#[cfg(feature = "solver-root-isolation")]
use native::engineering_accuracy::KktPointArithmetic;

#[cfg(feature = "solver-root-isolation")]
pub(super) struct EnclosedKktPoint {
    pub proof: KktPointArithmetic,
    pub _charge: crate::math::jobs::WorkerCharge,
    pub _program_charge: crate::math::jobs::WorkerCharge,
}
pub(super) enum PointArithmetic {
    #[cfg(feature = "solver-root-isolation")]
    Enclosed(Box<EnclosedKktPoint>),
    Unavailable(pse_model::generated::enums::AccuracyUnavailableReason),
}

#[cfg(not(feature = "solver-root-isolation"))]
pub(super) fn original_point(
    _plan: &pse_math::assembly::CasePlan,
    _values: &CaseValues,
    _point: &[f64],
    _source: SemanticProductKey,
    _execution: &Execution,
    _budget: &Arc<WorkerBudget>,
) -> Result<PointArithmetic, ProblemError> {
    Ok(PointArithmetic::Unavailable(
        pse_model::generated::enums::AccuracyUnavailableReason::EvaluatorUncertainty,
    ))
}

#[cfg(feature = "solver-root-isolation")]
pub(super) fn original_point(
    plan: &pse_math::assembly::CasePlan,
    values: &CaseValues,
    point: &[f64],
    source: SemanticProductKey,
    execution: &Execution,
    budget: &Arc<WorkerBudget>,
) -> Result<PointArithmetic, ProblemError> {
    use native::root_isolation::{Ibex, PointArithmeticEvidence};
    use pse_kernels::DerivativeOrder;

    execution.check()?;
    let available = budget.capacity().saturating_sub(budget.used());
    let node_limit = available / 256;
    if node_limit == 0 {
        return Ok(PointArithmetic::Unavailable(
            pse_model::generated::enums::AccuracyUnavailableReason::ResourceLimit,
        ));
    }
    let projection_charge = budget
        .charge(available)
        .map_err(MathRuntimeError::into_problem)?;
    let program = match plan.point_arithmetic_program(values, node_limit, &execution.cancel) {
        Ok(Some(program)) => program,
        Ok(None) => {
            return Ok(PointArithmetic::Unavailable(
                pse_model::generated::enums::AccuracyUnavailableReason::EvaluatorUncertainty,
            ));
        }
        Err(pse_math::factorable::FactorableError::Math(error)) => return Err(error.into()),
        Err(_) => {
            return Ok(PointArithmetic::Unavailable(
                pse_model::generated::enums::AccuracyUnavailableReason::Unsupported,
            ));
        }
    };
    drop(projection_charge);
    if program.graph.inputs != point.len()
        || source.point != Some(native::square_response::point_key(point))
    {
        return Err(ProblemError::Contract(
            "KKT point arithmetic source or coordinate inventory mismatch".into(),
        ));
    }
    let _program_charge = budget
        .charge(program.retained_bytes())
        .map_err(MathRuntimeError::into_problem)?;
    let workspace =
        Ibex.point_arithmetic_workspace_bytes_for_order(&program, DerivativeOrder::Second)?;
    let _workspace_charge = budget
        .charge(workspace)
        .map_err(MathRuntimeError::into_problem)?;
    let outputs = program.graph.residuals.len();
    let n = point.len();
    let hessian_extent = outputs
        .checked_mul(n)
        .and_then(|extent| extent.checked_mul(n))
        .ok_or_else(|| ProblemError::memory("KKT point Hessian extent"))?;
    let jacobian_extent = outputs
        .checked_mul(n)
        .ok_or_else(|| ProblemError::memory("KKT point Jacobian extent"))?;
    let retained_bytes = outputs
        .checked_mul(size_of::<pse_math::implicit::ProofInterval>())
        .and_then(|values| {
            values.checked_add(
                jacobian_extent.checked_mul(size_of::<pse_math::implicit::ProofInterval>())?,
            )
        })
        .and_then(|values| {
            values.checked_add(
                hessian_extent.checked_mul(size_of::<pse_math::implicit::ProofInterval>())?,
            )
        })
        .and_then(|bytes| {
            bytes.checked_add(
                program
                    .rows
                    .capacity()
                    .checked_mul(size_of::<pse_math::factorable::PointArithmeticRow>())?,
            )
        })
        // The result charge owns the boxed payload as well as its retained vectors.
        .and_then(|bytes| bytes.checked_add(size_of::<EnclosedKktPoint>()))
        .ok_or_else(|| ProblemError::memory("KKT point arithmetic retained extent"))?;
    let charge = budget
        .charge(retained_bytes)
        .map_err(MathRuntimeError::into_problem)?;
    let evidence = Ibex.enclose_point_arithmetic_order_with_execution(
        &program,
        point,
        DerivativeOrder::Second,
        workspace,
        execution,
    )?;
    execution.check()?;
    let (intervals, jacobian, hessian) = match evidence {
        PointArithmeticEvidence::Enclosed {
            values,
            jacobian,
            hessian: Some(hessian),
            ..
        } => (values, jacobian, hessian),
        PointArithmeticEvidence::Enclosed { hessian: None, .. } => {
            return Ok(PointArithmetic::Unavailable(
                pse_model::generated::enums::AccuracyUnavailableReason::EvaluatorUncertainty,
            ));
        }
        PointArithmeticEvidence::Interrupted { .. } => return Err(ProblemError::Cancelled),
        PointArithmeticEvidence::Incomplete { reason, .. } => {
            use pse_math::implicit::SelectionProofRefusal as R;
            let unavailable = match reason {
                R::Resource => {
                    pse_model::generated::enums::AccuracyUnavailableReason::ResourceLimit
                }
                R::Boundary => pse_model::generated::enums::AccuracyUnavailableReason::Boundary,
                R::Unsupported | R::Chart | R::Coverage => {
                    pse_model::generated::enums::AccuracyUnavailableReason::EvaluatorUncertainty
                }
            };
            return Ok(PointArithmetic::Unavailable(unavailable));
        }
    };
    if intervals.len() != outputs
        || jacobian.len() != jacobian_extent
        || hessian.len() != hessian_extent
        || intervals
            .iter()
            .chain(&jacobian)
            .chain(&hessian)
            .any(|interval| !interval.valid())
    {
        return Ok(PointArithmetic::Unavailable(
            pse_model::generated::enums::AccuracyUnavailableReason::EvaluatorUncertainty,
        ));
    }
    Ok(PointArithmetic::Enclosed(Box::new(EnclosedKktPoint {
        proof: KktPointArithmetic {
            source,
            projection: program.graph.identity(),
            rows: program.rows,
            objective: program.objective,
            values: intervals,
            jacobian,
            hessian: Some(hessian),
        },
        _charge: charge,
        _program_charge,
    })))
}
