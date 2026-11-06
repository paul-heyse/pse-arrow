// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Fixed-parameter admission for original-coordinate output estimates.
//! Source tangent constancy is deliberately narrower than numerical recovery.
use super::{SquareEstimates, SquareOutput, SquareResidual, SquareValidity};
use crate::{
    ProblemError,
    solve::Execution,
    square_response::{SparseFactor, SparseRequest, Withheld},
};
use faer::sparse::SparseColMat;
use pounce_presolve::linear_eq_plan::VarRecovery;
use pse_ids::{ContentHash, FramedHasher};
use pse_math::{
    assembly::CasePlan,
    binding::CaseValues,
    presolve::{AffineRow, Facts},
};
#[cfg(test)]
mod tests;

/// Sealed original-point admission. It grants only an output-accuracy factor,
/// never a parameter-response or predictor capability. Borrowed inputs cannot
/// change between admission and the full original First evaluation.
#[derive(Debug)]
pub struct SquareAccuracyFrame<'a> {
    request: SparseRequest<'a>,
    _source: (&'a CasePlan, &'a CaseValues, &'a Facts),
    allowance: usize,
    witness: ContentHash,
}

/// A full original sparse factor whose constant-bound exemption cannot escape
/// into public parameter response. Every residual and partial remains original.
#[derive(Debug)]
pub struct SquareAccuracyFactor {
    factor: SparseFactor,
    witness: ContentHash,
}

// Seeds are source unary equalities only. Binary propagation needs an already
// anchored endpoint; equal bounds and a closed cycle never seed this closure.
fn anchored(
    rows: &[Option<AffineRow>],
    n: usize,
    execution: &Execution,
) -> Result<Vec<bool>, ProblemError> {
    let mut constants = vec![false; n];
    let mut adjacency = vec![Vec::new(); n];
    let mut pending = std::collections::VecDeque::new();
    for row in rows.iter().flatten() {
        execution.check()?;
        if row.entries.len() == 1
            && let Some((&column, &coefficient)) = row.entries.first_key_value()
            && column < n
            && coefficient.is_finite()
            && coefficient != 0.0
        {
            constants[column] = true;
            pending.push_back(column);
        }
        if row.entries.len() == 2 {
            let mut entries = row.entries.iter();
            if let (Some((&a, &ca)), Some((&b, &cb))) = (entries.next(), entries.next())
                && a < n
                && b < n
                && ca.is_finite()
                && cb.is_finite()
                && ca != 0.0
                && cb != 0.0
            {
                adjacency[a].push(b);
                adjacency[b].push(a);
            }
        }
    }
    while let Some(column) = pending.pop_front() {
        execution.check()?;
        for &neighbor in &adjacency[column] {
            if !constants[neighbor] {
                constants[neighbor] = true;
                pending.push_back(neighbor);
            }
        }
    }
    Ok(constants)
}

impl<'a> SquareAccuracyFrame<'a> {
    /// Admit the current source, consumed fixed values, original point and
    /// physical budgets before evaluating the complete original Jacobian.
    /// Moving coordinates require strict pointwise interior independently of
    /// tolerance-based physical acceptance; no finite neighborhood is established.
    /// # Errors
    /// Stale facts/point, mismatched original scope, moving boundary coordinate,
    /// physical infeasibility, or operational/storage refusal.
    pub fn admit(
        request: SparseRequest<'a>,
        plan: &'a CasePlan,
        values: &'a CaseValues,
        facts: &'a Facts,
        execution: &Execution,
    ) -> Result<Self, Withheld> {
        execution
            .check()
            .map_err(|cause| Withheld::Cause(std::sync::Arc::new(cause)))?;
        // Refuse unsupported strict opaque storage before constructing source
        // adjacency or the library reduction. The caller reserves both products.
        let factor_allowance = SparseFactor::preflight_accuracy(&request, execution)?;
        let projection = crate::presolve::roots::allowance(
            plan.columns().len(),
            plan.jacobian_pattern().compute_nnz(),
        )
        .ok_or(Withheld::Memory)?;
        if factor_allowance
            .checked_add(projection)
            .is_none_or(|bytes| bytes > request.bytes)
        {
            return Err(Withheld::Memory);
        }
        if !facts.matches(plan, values)
            || plan.columns().len() != request.point.len()
            || plan.columns().iter().zip(request.point).any(|(id, value)| {
                values
                    .scalars
                    .get(id)
                    .is_none_or(|actual| actual.to_bits() != value.to_bits())
            })
        {
            return Err(Withheld::Structural("output accuracy requires matching source facts, consumed values and original point".into()));
        }
        let original = crate::assembled::contract(plan);
        original
            .square()
            .map_err(|cause| Withheld::Structural(cause.to_string()))?;
        let bounds = plan
            .structure()
            .rows()
            .iter()
            .map(|row| (row.lower, row.upper))
            .collect::<Vec<_>>();
        let mut constants = anchored(&facts.affine, original.variables.len(), execution)
            .map_err(|cause| Withheld::Cause(std::sync::Arc::new(cause)))?;
        let reduction = crate::presolve::roots::plan(
            &original,
            &crate::presolve::roots::Primal {
                facts,
                bounds: &bounds,
                policy: &crate::presolve::Policy::Auto,
            },
        )
        .map_err(|cause| Withheld::Cause(std::sync::Arc::new(cause)))?;
        execution
            .check()
            .map_err(|cause| Withheld::Cause(std::sync::Arc::new(cause)))?;
        for (column, constant) in constants.iter_mut().enumerate() {
            *constant &= reduction.as_ref().is_some_and(|plan| {
                matches!(plan.recovery.get(column), Some(VarRecovery::Constant(value)) if value.is_finite())
            });
        }
        let allowance = SparseFactor::admit_accuracy(
            &request,
            &constants,
            &original,
            &bounds,
            plan.jacobian_pattern(),
            execution,
        )?;
        let mut witness = FramedHasher::new(pse_ids::Frame::EngineeringGoalV1);
        witness
            .str("source-anchored-original-output-accuracy-v1")
            .hash(&facts.key)
            .hash(&request.key.structure)
            .hash(&request.key.binding)
            .hash(&request.normalization.key());
        // The common admission above has checked that these consumed keys exist.
        if let Some(point) = request.key.point {
            witness.hash(&point);
        }
        if let Some(accuracy) = request.key.accuracy {
            witness.hash(&accuracy);
        }
        for (id, value) in &facts.values {
            witness.id(id).u64(*value);
        }
        for (column, constant) in constants.iter().enumerate() {
            witness.u64(column as u64).bool(*constant);
        }
        for ((value, variable_tolerance), (observed, row_tolerance)) in request
            .point
            .iter()
            .zip(&request.tolerances.variables)
            .zip(request.values.iter().zip(&request.tolerances.rows))
        {
            witness
                .f64(*value)
                .f64(*variable_tolerance)
                .f64(*observed)
                .f64(*row_tolerance);
        }
        Ok(Self {
            request,
            _source: (plan, values, facts),
            allowance,
            witness: witness.finish_hash(),
        })
    }

    /// Factor the fresh complete original Jacobian after full original guard and
    /// First evaluation. No coordinate value or row defect is projected or snapped.
    /// # Errors
    /// Original sparse support, storage, execution or numerical factor failure.
    pub fn prepare(
        self,
        evaluate: impl FnOnce() -> Result<SparseColMat<usize, f64>, ProblemError>,
        execution: &Execution,
    ) -> Result<SquareAccuracyFactor, Withheld> {
        Ok(SquareAccuracyFactor {
            factor: SparseFactor::prepare_admitted(
                self.request,
                self.allowance,
                evaluate,
                execution,
            )?,
            witness: self.witness,
        })
    }
}

impl SquareAccuracyFactor {
    /// Retain the admitted allocation without exposing the private sparse factor.
    pub fn with_owner(mut self, owner: std::sync::Arc<dyn pse_math::AllocationOwner>) -> Self {
        self.factor = self.factor.with_owner(owner);
        self
    }

    /// Apply the unchanged original-equation estimator, adding the sealed source
    /// provenance to the actual original First/arithmetic validity witness.
    pub fn estimate(
        &self,
        residual: SquareResidual<'_>,
        validity: SquareValidity,
        outputs: &[SquareOutput<'_>],
        execution: &Execution,
    ) -> SquareEstimates {
        let mut witness = FramedHasher::new(pse_ids::Frame::EngineeringGoalV1);
        witness
            .str("source-anchored-full-original-first-output-estimate")
            .hash(&self.witness)
            .hash(&validity.witness);
        super::estimate_square(
            &self.factor,
            residual,
            Some(SquareValidity {
                source: validity.source,
                witness: witness.finish_hash(),
            }),
            outputs,
            execution,
        )
    }
}
