// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Compiler-established mathematical facts, independent of any solver library.
use crate::{MathError, assembly::CasePlan, binding::VariableDomain, coefficients::Coefficients};
/// Admitted interval shape; values remain owned by the original case structure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoundShape {
    /// No finite endpoint.
    Free,
    /// Only a lower endpoint.
    Lower,
    /// Only an upper endpoint.
    Upper,
    /// A nonnegative half-line with a zero endpoint.
    Nonnegative,
    /// A nonpositive half-line with a zero endpoint.
    Nonpositive,
    /// Two finite endpoints.
    Boxed,
}
/// Immutable class facts derived from admitted source and coefficient products.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProblemFacts {
    /// Free variable count; zero selects original constant evaluation.
    pub variables: usize,
    /// Complete selected constraint count.
    pub rows: usize,
    /// Has an authored optimization objective.
    pub objective: bool,
    /// Every row is a finite equality.
    pub equalities: bool,
    /// One source-declared domain per free variable.
    pub domains: Vec<VariableDomain>,
    /// Exact available and admitted smooth derivative order.
    pub derivatives: pse_kernels::DerivativeOrder,
    /// Order in the prepared artifact requests; independent of mathematical availability.
    pub prepared_derivatives: pse_kernels::DerivativeOrder,
    /// Per-coordinate interval class, derived from admitted bounds and discrete domains.
    pub bounds: Vec<BoundShape>,
    /// Source obligations remain present after algebraic normalization.
    pub guarded: bool,
    /// Coefficient extraction established affine rows and degree <=2 objective.
    pub coefficients: bool,
    /// Mathematical row class from shared bound facts, independent of requested representation.
    pub affine_rows: Vec<bool>,
    /// Admitted polynomial degree bound, independent of coefficient materialization.
    pub objective_degree: Option<u8>,
    /// Bound/value snapshot establishing class and guard facts.
    pub bound_assumptions: pse_ids::ContentHash,
    /// Extracted objective has nonzero quadratic entries; does not prove convexity.
    pub quadratic: bool,
}
impl ProblemFacts {
    /// Derive facts from a plan and its optional, current coefficient snapshot.
    pub fn from_plan(
        plan: &CasePlan,
        coefficients: Option<&Coefficients>,
        facts: &crate::presolve::Facts,
    ) -> Result<Self, MathError> {
        if facts.structure != plan.structure().key()
            || coefficients.is_some_and(|c| !c.matches_facts(facts))
        {
            return Err(MathError::Contract(
                "class coefficients do not match plan".into(),
            ));
        }
        Ok(Self {
            variables: plan.columns().len(),
            rows: plan.structure().rows().len(),
            objective: plan.structure().objective().is_some(),
            equalities: plan
                .structure()
                .rows()
                .iter()
                .all(|r| r.lower.is_finite() && r.lower == r.upper),
            domains: plan
                .structure()
                .variables()
                .iter()
                .filter(|v| !v.fixed)
                .map(|v| v.domain)
                .collect(),
            derivatives: plan.available_order(),
            prepared_derivatives: plan.order(),
            bounds: plan
                .structure()
                .variables()
                .iter()
                .filter(|v| !v.fixed)
                .map(|v| {
                    match (
                        v.lower.is_some() || v.domain == VariableDomain::Binary,
                        v.upper.is_some() || v.domain == VariableDomain::Binary,
                    ) {
                        (false, false) => BoundShape::Free,
                        (true, false) if v.lower == Some(0.0) => BoundShape::Nonnegative,
                        (true, false) => BoundShape::Lower,
                        (false, true) if v.upper == Some(0.0) => BoundShape::Nonpositive,
                        (false, true) => BoundShape::Upper,
                        (true, true) => BoundShape::Boxed,
                    }
                })
                .collect(),
            guarded: plan.bodies().values().any(|b| b.has_obligations()),
            coefficients: coefficients.is_some(),
            affine_rows: facts.affine.iter().map(Option::is_some).collect(),
            objective_degree: facts.objective_degree,
            bound_assumptions: facts.key,
            quadratic: coefficients.is_some_and(|c| c.hessian.val().iter().any(|v| *v != 0.0)),
        })
    }
}
