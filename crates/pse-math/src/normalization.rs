// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Positive diagonal coordinate transport; no new evaluator or mathematical authority.
use crate::MathError;
use pounce_nlp::expression_provider::{FbbtOp as Op, FbbtTape};
use pse_ids::{ContentHash, FramedHasher, SemanticId};
use pse_model::{generated::enums::NumericalTarget, numerics::ResolvedNumericalPolicy};

/// x = Sx z, normalized rows = rows / Sr, normalized minimization objective = f / Sf.
#[derive(Clone, Debug, PartialEq)]
pub struct Normalization {
    /// Positive variable coordinate nominals; integer entries are one.
    pub variables: Vec<f64>,
    /// Positive row coordinate nominals; cone blocks may require common factors.
    pub rows: Vec<f64>,
    /// Positive objective nominal, independent of objective sense.
    pub objective: f64,
}
impl Normalization {
    /// Explicit identity, used by direct native problem callers that declare normalized data.
    pub fn identity(variables: usize, rows: usize) -> Self {
        Self {
            variables: vec![1.0; variables],
            rows: vec![1.0; rows],
            objective: 1.0,
        }
    }
    /// Derive one checked projection in the existing native coordinate order.
    pub fn from_policy(
        policy: &ResolvedNumericalPolicy,
        variables: &[SemanticId],
        rows: &[SemanticId],
    ) -> Result<Self, MathError> {
        let scale = |id, kind| {
            policy
                .targets
                .iter()
                .find(|t| t.id == id && t.kind == kind)
                .map(|t| t.coordinate_scale)
                .ok_or_else(|| MathError::Contract(format!("missing resolved coordinate {id}")))
        };
        let result = Self {
            variables: variables
                .iter()
                .map(|id| scale(*id, NumericalTarget::Variable))
                .collect::<Result<_, _>>()?,
            rows: rows
                .iter()
                .map(|id| scale(*id, NumericalTarget::Row))
                .collect::<Result<_, _>>()?,
            objective: policy
                .targets
                .iter()
                .find(|t| t.kind == NumericalTarget::Objective)
                .map_or(1.0, |t| t.coordinate_scale),
        };
        result.validate(variables.len(), rows.len())?;
        Ok(result)
    }
    /// Dimension and finite positive coordinate contract, without a backend-specific floor.
    pub fn validate(&self, n: usize, m: usize) -> Result<(), MathError> {
        if self.variables.len() != n
            || self.rows.len() != m
            || self
                .variables
                .iter()
                .chain(&self.rows)
                .chain(std::iter::once(&self.objective))
                .any(|v| !v.is_finite() || *v <= 0.0)
        {
            return Err(MathError::Contract(
                "normalization dimensions or positive finite factors".into(),
            ));
        }
        Ok(())
    }
    /// Derive native NLP row coordinates for one uniform feasibility budget.
    ///
    /// For physical row budget εᵢ and native budget τ, Srᵢ = εᵢ / τ makes
    /// |physical violationᵢ / Srᵢ| ≤ τ equivalent to its original budget.
    /// Variable/objective coordinates retain their declared meaning. This derived
    /// map does not change the declared nominals used by original KKT analysis.
    pub fn with_row_budgets(&self, budgets: &[f64], feasibility: f64) -> Result<Self, MathError> {
        self.validate(self.variables.len(), budgets.len())?;
        if budgets.iter().any(|v| !v.is_finite() || *v <= 0.0)
            || !feasibility.is_finite()
            || feasibility <= 0.0
        {
            return Err(MathError::Contract(
                "native NLP row budgets must be positive finite".into(),
            ));
        }
        let rows = budgets
            .iter()
            .map(|v| checked_ratio(*v, feasibility))
            .collect::<Result<Vec<_>, _>>()?;
        let result = Self {
            variables: self.variables.clone(),
            rows,
            objective: self.objective,
        };
        result.validate(self.variables.len(), budgets.len())?;
        Ok(result)
    }
    /// Normalization has a separate identity from native algorithmic scaling.
    pub fn key(&self) -> ContentHash {
        let mut h = FramedHasher::new(pse_ids::Frame::MathNormalizationV1);
        h.u64(self.objective.to_bits());
        for group in [&self.variables, &self.rows] {
            h.u64(group.len() as u64);
            for value in group {
                h.u64(value.to_bits());
            }
        }
        h.finish_hash()
    }
    /// Map a finite primal back to original physical coordinates.
    pub fn physical_point(&self, z: &[f64]) -> Result<Vec<f64>, MathError> {
        transform(z, &self.variables, false)
    }
    /// Map a physical primal into normalized coordinates.
    pub fn normalized_point(&self, x: &[f64]) -> Result<Vec<f64>, MathError> {
        transform(x, &self.variables, true)
    }
    /// Project the library tape mechanically, keeping original node correspondence local.
    pub fn tape(&self, tape: &FbbtTape, row: usize, limit: usize) -> Result<FbbtTape, MathError> {
        if row >= self.rows.len()
            || tape.first_invalid_slot().is_some()
            || tape.ops.is_empty()
            || tape
                .ops
                .len()
                .checked_mul(3)
                .and_then(|n| n.checked_add(2))
                .is_none_or(|n| n > limit)
        {
            return Err(MathError::Limit("normalized FBBT tape"));
        }
        let mut ops = Vec::new();
        let mut map = Vec::with_capacity(tape.ops.len());
        for op in &tape.ops {
            let next = match *op {
                Op::Var(c) => {
                    let scale = *self
                        .variables
                        .get(c)
                        .ok_or_else(|| MathError::Contract("FBBT variable coordinate".into()))?;
                    let index = ops.len();
                    ops.push(Op::Var(c));
                    ops.push(Op::Const(scale));
                    Op::Mul(index, index + 1)
                }
                Op::Const(v) => Op::Const(v),
                Op::Opaque => Op::Opaque,
                Op::Add(a, b) => Op::Add(map[a], map[b]),
                Op::Sub(a, b) => Op::Sub(map[a], map[b]),
                Op::Mul(a, b) => Op::Mul(map[a], map[b]),
                Op::Div(a, b) => Op::Div(map[a], map[b]),
                Op::PowInt(a, n) => Op::PowInt(map[a], n),
                Op::Neg(a) => Op::Neg(map[a]),
                Op::Sqrt(a) => Op::Sqrt(map[a]),
                Op::Exp(a) => Op::Exp(map[a]),
                Op::Ln(a) => Op::Ln(map[a]),
                Op::Abs(a) => Op::Abs(map[a]),
                Op::Sin(a) => Op::Sin(map[a]),
                Op::Cos(a) => Op::Cos(map[a]),
            };
            map.push(ops.len());
            ops.push(next);
        }
        let root = ops.len() - 1;
        let factor = ops.len();
        ops.push(Op::Const(self.rows[row]));
        ops.push(Op::Div(root, factor));
        Ok(FbbtTape { ops })
    }
    /// Preserve value/guard assumptions while projecting shared affine and interval facts.
    pub fn facts(
        &self,
        facts: &crate::presolve::Facts,
        limit: usize,
    ) -> Result<crate::presolve::Facts, MathError> {
        self.validate(facts.objective_linear.len(), facts.affine.len())?;
        let mut out = facts.clone();
        let mut h = FramedHasher::new(pse_ids::Frame::MathNormalizedFactsV1);
        h.hash(&facts.key).hash(&self.key());
        out.key = h.finish_hash();
        for (r, row) in out.affine.iter_mut().enumerate() {
            if let Some(row) = row {
                row.constant = checked_ratio(row.constant, self.rows[r])?;
                for (c, value) in &mut row.entries {
                    *value =
                        checked_ratio(checked_product(*value, self.variables[*c])?, self.rows[r])?;
                }
            }
        }
        let mut remaining = limit;
        out.tapes = facts
            .tapes
            .iter()
            .enumerate()
            .map(|(r, t)| {
                let projected = self.tape(t, r, remaining)?;
                remaining = remaining
                    .checked_sub(projected.ops.len())
                    .ok_or(MathError::Limit("normalized FBBT total"))?;
                Ok(projected)
            })
            .collect::<Result<_, MathError>>()?;
        Ok(out)
    }
}
/// Magnitudes and finite values may not disappear or overflow during coordinate transport.
pub fn checked_product(value: f64, factor: f64) -> Result<f64, MathError> {
    let result = value * factor;
    if !value.is_finite()
        || !factor.is_finite()
        || factor <= 0.0
        || !result.is_finite()
        || value != 0.0 && result == 0.0
    {
        return Err(MathError::Contract(
            "coordinate transport overflow or underflow".into(),
        ));
    }
    Ok(result)
}
/// Division preserves infinities only at the explicit bound call sites.
pub fn checked_ratio(value: f64, factor: f64) -> Result<f64, MathError> {
    let result = value / factor;
    if !value.is_finite()
        || !factor.is_finite()
        || factor <= 0.0
        || !result.is_finite()
        || value != 0.0 && result == 0.0
    {
        return Err(MathError::Contract(
            "coordinate transport overflow or underflow".into(),
        ));
    }
    Ok(result)
}
fn transform(values: &[f64], scales: &[f64], divide: bool) -> Result<Vec<f64>, MathError> {
    if values.len() != scales.len() {
        return Err(MathError::Contract("coordinate vector dimensions".into()));
    }
    values
        .iter()
        .zip(scales)
        .map(|(v, s)| {
            if divide {
                checked_ratio(*v, *s)
            } else {
                checked_product(*v, *s)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_row_budgets_preserve_each_physical_acceptance_and_declared_scales() {
        let declared = Normalization {
            variables: vec![2.0],
            rows: vec![3.0, 7.0],
            objective: 5.0,
        };
        let original = declared.clone();
        let budgets = [1e-8, 1e-4];
        let native = declared.with_row_budgets(&budgets, 1e-8).unwrap();
        assert_eq!(declared, original);
        assert_eq!(native.variables, declared.variables);
        assert_eq!(native.objective, declared.objective);
        assert_eq!(native.rows, vec![1.0, 10000.0]);
        for (budget, scale) in budgets.iter().zip(&native.rows) {
            for fraction in [0.5, 1.0, 2.0] {
                let violation = budget * fraction;
                assert_eq!(violation <= *budget, violation / scale <= 1e-8);
                assert!((violation / scale / 1e-8 - fraction).abs() < 1e-15);
            }
        }
        let changed = declared.with_row_budgets(&[1e-8, 2e-4], 1e-8).unwrap();
        assert_ne!(native.key(), changed.key());
        assert_eq!(declared.key(), original.key());
    }

    #[test]
    fn native_row_budgets_refuse_extent_invalidity_and_unrepresentable_ratios() {
        let declared = Normalization::identity(1, 2);
        assert!(declared.with_row_budgets(&[1e-8], 1e-8).is_err());
        for budget in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(declared.with_row_budgets(&[1e-8, budget], 1e-8).is_err());
        }
        for accuracy in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(declared.with_row_budgets(&[1e-8, 1e-4], accuracy).is_err());
        }
        assert!(
            declared
                .with_row_budgets(&[1.0, f64::MAX], f64::MIN_POSITIVE)
                .is_err()
        );
        assert!(
            declared
                .with_row_budgets(&[1.0, f64::from_bits(1)], f64::MAX)
                .is_err()
        );
    }
}
