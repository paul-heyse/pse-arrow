// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original-point arithmetic uncertainty uses the library's exact-real interval DAG.
use super::*;

pub(crate) struct ArithmeticUncertainty {
    pub residual: Vec<f64>,
    pub jacobian: Vec<f64>,
    pub witness: pse_ids::ContentHash,
    _owner: crate::math::jobs::WorkerCharge,
}

struct PointEnclosure<'a> {
    plan: &'a pse_math::assembly::CasePlan,
    values: &'a CaseValues,
    point: &'a [f64],
    residual: &'a [f64],
    jacobian: &'a faer::sparse::SparseColMat<usize, f64>,
    execution: &'a Execution,
    budget: &'a Arc<WorkerBudget>,
    selected_values: bool,
}

#[cfg(not(feature = "solver-root-isolation"))]
fn enclose_point(input: PointEnclosure<'_>) -> Result<Option<ArithmeticUncertainty>, ProblemError> {
    let _ = (
        input.plan,
        input.values,
        input.point,
        input.residual,
        input.jacobian,
        input.execution,
        input.budget,
        input.selected_values,
    );
    Ok(None)
}

#[cfg(feature = "solver-root-isolation")]
fn enclose_point(input: PointEnclosure<'_>) -> Result<Option<ArithmeticUncertainty>, ProblemError> {
    let PointEnclosure {
        plan,
        values,
        point,
        residual,
        jacobian,
        execution,
        budget,
        selected_values,
    } = input;
    use native::root_isolation::{Ibex, PointArithmeticEvidence};
    execution.check()?;
    let available = budget.capacity().saturating_sub(budget.used());
    let limit = available / 256;
    if limit == 0 {
        return Ok(None);
    }
    // The existing factorable projection limit bounds construction under the worker
    // allowance. Replace that temporary construction charge with its actual product.
    let projection_charge = budget
        .charge(available)
        .map_err(MathRuntimeError::into_problem)?;
    let program = match plan.point_arithmetic_program(values, limit, &execution.cancel) {
        Ok(Some(program)) => program,
        Ok(None) => return Ok(None),
        Err(pse_math::factorable::FactorableError::Math(error)) => return Err(error.into()),
        Err(_) => return Ok(None),
    };
    drop(projection_charge);
    let _program_charge = budget
        .charge(program.retained_bytes())
        .map_err(MathRuntimeError::into_problem)?;
    if program.graph.inputs != point.len()
        || program.rows.len() != residual.len()
        || jacobian.nrows() != residual.len()
        || jacobian.ncols() != point.len()
    {
        return Err(ProblemError::Contract(
            "original point arithmetic inventory mismatch".into(),
        ));
    }
    let entries = residual
        .len()
        .checked_mul(point.len())
        .ok_or_else(|| ProblemError::memory("point arithmetic uncertainty extent"))?;
    let native_entries = point
        .len()
        .checked_add(1)
        .and_then(|n| program.graph.residuals.len().checked_mul(n))
        .and_then(|n| n.checked_mul(2))
        .ok_or_else(|| ProblemError::memory("point arithmetic enclosure extent"))?;
    let retained_bytes = entries
        .checked_add(residual.len())
        .and_then(|n| n.checked_mul(size_of::<f64>()))
        .ok_or_else(|| ProblemError::memory("point arithmetic uncertainty extent"))?;
    let owner = budget
        .charge(retained_bytes)
        .map_err(MathRuntimeError::into_problem)?;
    let scratch_bytes = native_entries
        .checked_add(entries)
        .and_then(|n| n.checked_mul(size_of::<f64>()))
        .ok_or_else(|| ProblemError::memory("point arithmetic enclosure scratch extent"))?;
    let _scratch_charge = budget
        .charge(scratch_bytes)
        .map_err(MathRuntimeError::into_problem)?;
    let workspace = Ibex.point_arithmetic_workspace_bytes(&program)?;
    let _workspace_charge = budget
        .charge(workspace)
        .map_err(MathRuntimeError::into_problem)?;
    let enclosure =
        Ibex.enclose_point_arithmetic_with_execution(&program, point, workspace, execution)?;
    execution.check()?;
    let (enclosed_values, enclosed_jacobian) = match enclosure {
        PointArithmeticEvidence::Enclosed {
            values, jacobian, ..
        } => (values, jacobian),
        PointArithmeticEvidence::Interrupted { .. } => return Err(ProblemError::Cancelled),
        PointArithmeticEvidence::Incomplete { .. } => return Ok(None),
    };
    let mut dense = vec![0.; entries];
    for (column, _) in point.iter().enumerate() {
        for entry in jacobian.symbolic().col_range(column) {
            dense[jacobian.symbolic().row_idx()[entry] * point.len() + column] =
                jacobian.val()[entry];
        }
    }
    let radius = |interval: &pse_math::implicit::ProofInterval, value: f64| {
        let radius = (interval.lower - value)
            .abs()
            .max((interval.upper - value).abs());
        let radius = if radius == 0. { 0. } else { radius.next_up() };
        (radius.is_finite() && radius >= 0.).then_some(radius)
    };
    let mut residual_uncertainty = Vec::with_capacity(residual.len());
    let mut jacobian_uncertainty = Vec::with_capacity(entries);
    for (row, projection) in program.rows.iter().enumerate() {
        let Some(output) = (if selected_values {
            Some(projection.value)
        } else {
            projection
                .lower_residual
                .filter(|_| projection.lower_residual == projection.upper_residual)
        }) else {
            return Ok(None);
        };
        let Some(error) = enclosed_values
            .get(output)
            .and_then(|interval| radius(interval, residual[row]))
        else {
            return Ok(None);
        };
        residual_uncertainty.push(error);
        for column in 0..point.len() {
            let Some(error) = enclosed_jacobian
                .get(projection.value * point.len() + column)
                .and_then(|interval| radius(interval, dense[row * point.len() + column]))
            else {
                return Ok(None);
            };
            jacobian_uncertainty.push(error);
        }
    }
    let mut witness = FramedHasher::new(pse_ids::Frame::EngineeringGoalV1);
    witness
        .str("point-arithmetic-projection-and-expression")
        .hash(&program.key)
        .hash(&program.graph.identity());
    Ok(Some(ArithmeticUncertainty {
        residual: residual_uncertainty,
        jacobian: jacobian_uncertainty,
        witness: witness.finish_hash(),
        _owner: owner,
    }))
}

/// Equality residual arithmetic, including original bound subtraction.
pub(super) fn original_point(
    plan: &pse_math::assembly::CasePlan,
    values: &CaseValues,
    point: &[f64],
    residual: &[f64],
    jacobian: &faer::sparse::SparseColMat<usize, f64>,
    execution: &Execution,
    budget: &Arc<WorkerBudget>,
) -> Result<Option<ArithmeticUncertainty>, ProblemError> {
    enclose_point(PointEnclosure {
        plan,
        values,
        point,
        residual,
        jacobian,
        execution,
        budget,
        selected_values: false,
    })
}
/// Selected Member values need no bound residual; First uncertainty uses their value output.
pub(crate) fn selected_point(
    plan: &pse_math::assembly::CasePlan,
    values: &CaseValues,
    point: &[f64],
    outputs: &[f64],
    jacobian: &faer::sparse::SparseColMat<usize, f64>,
    execution: &Execution,
    budget: &Arc<WorkerBudget>,
) -> Result<Option<ArithmeticUncertainty>, ProblemError> {
    enclose_point(PointEnclosure {
        plan,
        values,
        point,
        residual: outputs,
        jacobian,
        execution,
        budget,
        selected_values: true,
    })
}
