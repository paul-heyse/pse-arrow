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
//!   [`Pinned::discrete`] pins a [`Relaxed`] mixed-integer case under a [`Commitment`]:
//!   the continuous problem of a discrete assignment.
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
/// them, are conditional on (ADR-0118 items 4 and 9; PS-12): the committed columns by
/// identity, in column order, at their values in original coordinates.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Commitment {
    /// Committed columns and their values.
    pub columns: Vec<(SemanticId, f64)>,
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
                .ok_or_else(|| ProblemError::Contract("pinned column out of range or repeated".into()))?;
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
    /// ADR-0118 item 9). Each committed column is pinned at its value:
    /// - an integer, binary or semi-integer column must be committed, at a member of its
    ///   domain;
    /// - a semi-continuous column is committed at zero, its off branch, or stays free over
    ///   its declared active interval;
    /// - a continuous column may be committed inside its box, such as an SOS or cardinality
    ///   member held at zero.
    ///
    /// # Errors
    /// A committed identity that is not a column or is repeated, an integer-valued column
    /// left free, or a value outside its column's domain or box.
    pub fn discrete(relaxed: Relaxed, commitment: &Commitment) -> Result<Self, ProblemError> {
        use ModelingVariableDomain as D;
        let Relaxed { inner, domains } = relaxed;
        let mut contract = inner.contract().clone();
        let mut committed = std::collections::BTreeMap::new();
        for (id, value) in &commitment.columns {
            let column = contract
                .variables
                .iter()
                .position(|v| v.id == *id)
                .filter(|c| !committed.contains_key(c))
                .ok_or_else(|| {
                    ProblemError::Contract(format!("committed column {id} absent or repeated"))
                })?;
            committed.insert(column, *value);
        }
        for (column, (variable, domain)) in contract.variables.iter_mut().zip(&domains).enumerate()
        {
            let admitted = match (domain, committed.get(&column)) {
                (D::Continuous, Some(v)) => v.is_finite() && *v >= variable.lower && *v <= variable.upper,
                (d, Some(v)) => d.contains(*v, variable.lower, variable.upper),
                (D::Continuous | D::Semicontinuous, None) => true,
                (_, None) => false,
            };
            if !admitted {
                return Err(ProblemError::Unsupported(format!(
                    "a discrete assignment commits {} ({}) at a member of its domain",
                    variable.id,
                    domain.as_str()
                )));
            }
            if let Some(v) = committed.get(&column) {
                variable.lower = *v;
                variable.upper = *v;
            }
        }
        Ok(Self { inner, contract })
    }
}
impl NlpOracle for Pinned {
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
