// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Transformations of an NLP's callbacks that keep its coordinates (ADR-0118 item 9; Plan
//! 22 I3). Each wraps an [`NlpOracle`] and changes only a declared box or row bound, so
//! candidates, multipliers, normalization and tolerances keep the original column and row
//! order, and a consumer reads every result by the original identities.
//!
//! - [`Pinned`] holds columns at given values with the degenerate box `[v, v]`: a
//!   parameter of a parametric sensitivity (sIPOPT's pin formulation, S1) or a profile pin
//!   (S3). A pinned column stays a column, so the KKT-point analysis sees its bound as a
//!   strongly active row whose multiplier is the derivative of the optimal value along it.
//!   [`Pinned::discrete`] commits a [`Relaxed`] mixed-integer case to the closed boxes of
//!   a [`Commitment`]: the continuous problem of a discrete assignment.
//! - [`Unconstrained`] removes the bounds of rows a discrete assignment leaves unenforced
//!   (the SCIP fixed-assignment re-solve).
//!
//! One [`Commitment`] states the assignment behind conditional multipliers, for the SCIP
//! fixed-assignment re-solve and the HiGHS fixed-commitment LP alike; the candidate those
//! multipliers belong to carries it.
//!
//! Proofs tied to the inner box or row bounds (presolve facts, the compiler's structural
//! witness) are not forwarded; the transformed callbacks are analysed afresh.
use crate::{DerivativeFacts, NlpOracle, OracleContract, ProblemError};
use pse_ids::SemanticId;
use pse_model::generated::enums::ModelingVariableDomain;

/// The discrete assignment a candidate's multipliers, and every quantity derived from
/// them, are conditional on (ADR-0118 items 4 and 9; PS-12): each committed column by
/// identity, in column order, with its committed closed box `(lower, upper)` in original
/// coordinates. A degenerate box `[v, v]` fixes a column at `v` (an integer value, a semi
/// column's zero branch, a held SOS member); a semicontinuous column on its active branch
/// is committed to its active interval `[l, u]`, over which the continuous problem still
/// decides it. One box states both kinds of branch, so no second representation exists.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Commitment {
    /// Committed columns and their closed boxes.
    pub columns: Vec<(SemanticId, (f64, f64))>,
}
/// NLP callbacks over every column of a mixed-integer case, its discrete domains relaxed
/// to their declared boxes, with each column's domain. It is not itself an [`NlpOracle`]:
/// only [`Pinned::discrete`] turns it into callbacks, so no relaxation reaches a solver with
/// an integer-valued column free.
#[derive(Debug)]
pub struct Relaxed {
    inner: Box<dyn NlpOracle>,
    domains: Vec<ModelingVariableDomain>,
}
impl Relaxed {
    /// `inner` with the declared domain of each of its columns, in column order.
    ///
    /// # Errors
    /// A domain count that differs from the column count.
    pub(crate) fn new(
        inner: Box<dyn NlpOracle>,
        domains: Vec<ModelingVariableDomain>,
    ) -> Result<Self, ProblemError> {
        if domains.len() != inner.contract().variables.len() {
            return Err(ProblemError::Internal("relaxed column domains".into()));
        }
        Ok(Self { inner, domains })
    }
}

/// NLP callbacks with some columns held at given values by a degenerate box.
#[derive(Debug)]
pub struct Pinned {
    inner: Box<dyn NlpOracle>,
    contract: OracleContract,
}
impl Pinned {
    /// Pin each `(column, value)`: the column's box becomes `[value, value]`.
    ///
    /// # Errors
    /// A column out of range or repeated, or a value that is not finite or lies outside the
    /// column's declared box: a pin narrows a box, it never widens one.
    pub fn new(inner: Box<dyn NlpOracle>, pins: &[(usize, f64)]) -> Result<Self, ProblemError> {
        let mut contract = inner.contract().clone();
        let mut seen = std::collections::BTreeSet::new();
        for &(column, value) in pins {
            let variable = contract
                .variables
                .get_mut(column)
                .filter(|_| seen.insert(column))
                .ok_or_else(|| {
                    ProblemError::Contract("pinned column out of range or repeated".into())
                })?;
            if !value.is_finite() || value < variable.lower || value > variable.upper {
                return Err(ProblemError::Contract(format!(
                    "pin of {} at {value} lies outside its box [{}, {}]",
                    variable.id, variable.lower, variable.upper
                )));
            }
            variable.lower = value;
            variable.upper = value;
        }
        Ok(Self { inner, contract })
    }
    /// The continuous problem of a mixed-integer case under `commitment` (ADR-0105 §2,
    /// ADR-0118 item 9): each committed column keeps its coordinate with its committed box.
    /// - An integer, binary or semi-integer column must be committed to a degenerate box
    ///   at a member of its domain.
    /// - A semicontinuous column must be committed to a degenerate box at a member of its
    ///   domain (zero is its off branch), or exactly to its declared active interval.
    /// - A continuous column may be committed to a box inside its own, such as an SOS or
    ///   cardinality member held at zero.
    ///
    /// # Errors
    /// A committed identity that is not a column or is repeated, a discrete column left
    /// uncommitted, or a box its column's domain or declared box does not admit.
    pub fn discrete(relaxed: Relaxed, commitment: &Commitment) -> Result<Self, ProblemError> {
        use ModelingVariableDomain as D;
        let Relaxed { inner, domains } = relaxed;
        let mut contract = inner.contract().clone();
        let mut committed = std::collections::BTreeMap::new();
        for (id, bounds) in &commitment.columns {
            let column = contract
                .variables
                .iter()
                .position(|v| v.id == *id)
                .filter(|c| !committed.contains_key(c))
                .ok_or_else(|| {
                    ProblemError::Contract(format!("committed column {id} absent or repeated"))
                })?;
            committed.insert(column, *bounds);
        }
        for (column, (variable, domain)) in contract.variables.iter_mut().zip(&domains).enumerate()
        {
            let (lower, upper) = (variable.lower, variable.upper);
            let admitted = match (domain, committed.get(&column)) {
                (_, Some(&(l, u))) if !(l.is_finite() && u.is_finite() && l <= u) => false,
                (D::Continuous, Some(&(l, u))) => l >= lower && u <= upper,
                (D::Semicontinuous, Some(&(l, u))) if l < u => l == lower && u == upper,
                (d, Some(&(l, u))) => l == u && d.contains(l, lower, upper),
                (D::Continuous, None) => true,
                (_, None) => false,
            };
            if !admitted {
                return Err(ProblemError::Unsupported(format!(
                    "a discrete assignment commits {} ({}) to one value of its domain, or a semicontinuous column to its active interval",
                    variable.id,
                    domain.as_str()
                )));
            }
            if let Some(&(l, u)) = committed.get(&column) {
                variable.lower = l;
                variable.upper = u;
            }
        }
        Ok(Self { inner, contract })
    }
}
impl NlpOracle for Pinned {
    fn solve_separator(&self) -> Option<&crate::SolveSeparator> {
        self.inner.solve_separator()
    }
    fn normalization(&self) -> Option<&pse_math::normalization::Normalization> {
        self.inner.normalization()
    }
    fn constraint_sources(&self) -> Result<Vec<pse_math::assembly::OutputValue>, ProblemError> {
        self.inner.constraint_sources()
    }
    fn derivative_facts(&self) -> DerivativeFacts {
        self.inner.derivative_facts()
    }
    fn contract(&self) -> &OracleContract {
        &self.contract
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.inner.jacobian_pattern()
    }
    fn hessian_pattern(&self) -> Option<faer::sparse::SymbolicSparseColMatRef<'_, usize>> {
        self.inner.hessian_pattern()
    }
    fn constraint_bounds(&self) -> &[(f64, f64)] {
        self.inner.constraint_bounds()
    }
    fn objective(&mut self, x: &[f64]) -> Result<f64, ProblemError> {
        self.inner.objective(x)
    }
    fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.inner.constraints(x, out)
    }
    fn gradient(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.inner.gradient(x, out)
    }
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.inner.jacobian(x, out)
    }
    fn hessian(
        &mut self,
        x: &[f64],
        objective_weight: f64,
        multipliers: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        self.inner.hessian(x, objective_weight, multipliers, out)
    }
}

/// NLP callbacks with the bounds of some rows removed: the rows a discrete assignment
/// leaves unenforced.
#[derive(Debug)]
pub struct Unconstrained {
    inner: Box<dyn NlpOracle>,
    bounds: Vec<(f64, f64)>,
}
impl Unconstrained {
    /// Remove the bounds of `rows`, by ordinal.
    ///
    /// # Errors
    /// A row ordinal out of range.
    pub fn new(inner: Box<dyn NlpOracle>, rows: &[usize]) -> Result<Self, ProblemError> {
        let mut bounds = inner.constraint_bounds().to_vec();
        for r in rows {
            *bounds
                .get_mut(*r)
                .ok_or_else(|| ProblemError::Internal("unconstrained row ordinal".into()))? =
                (f64::NEG_INFINITY, f64::INFINITY);
        }
        Ok(Self { inner, bounds })
    }
}
impl NlpOracle for Unconstrained {
    fn solve_separator(&self) -> Option<&crate::SolveSeparator> {
        self.inner.solve_separator()
    }
    fn normalization(&self) -> Option<&pse_math::normalization::Normalization> {
        self.inner.normalization()
    }
    fn constraint_sources(&self) -> Result<Vec<pse_math::assembly::OutputValue>, ProblemError> {
        self.inner.constraint_sources()
    }
    fn derivative_facts(&self) -> DerivativeFacts {
        self.inner.derivative_facts()
    }
    fn contract(&self) -> &OracleContract {
        self.inner.contract()
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.inner.jacobian_pattern()
    }
    fn hessian_pattern(&self) -> Option<faer::sparse::SymbolicSparseColMatRef<'_, usize>> {
        self.inner.hessian_pattern()
    }
    fn constraint_bounds(&self) -> &[(f64, f64)] {
        &self.bounds
    }
    fn objective(&mut self, x: &[f64]) -> Result<f64, ProblemError> {
        self.inner.objective(x)
    }
    fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.inner.constraints(x, out)
    }
    fn gradient(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.inner.gradient(x, out)
    }
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.inner.jacobian(x, out)
    }
    fn hessian(
        &mut self,
        x: &[f64],
        objective_weight: f64,
        multipliers: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        self.inner.hessian(x, objective_weight, multipliers, out)
    }
}
