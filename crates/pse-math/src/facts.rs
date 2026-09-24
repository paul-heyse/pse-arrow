// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Compiler-established mathematical facts, independent of any solver library.
use crate::{MathError, assembly::CasePlan, binding::VariableDomain, coefficients::Coefficients};
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
    /// Coefficient extraction established affine rows and degree <=2 objective.
    pub coefficients: bool,
    /// Extracted objective has nonzero quadratic entries; does not prove convexity.
    pub quadratic: bool,
}
impl ProblemFacts {
    /// Derive facts from a plan and its optional, current coefficient snapshot.
    pub fn from_plan(
        plan: &CasePlan,
        coefficients: Option<&Coefficients>,
    ) -> Result<Self, MathError> {
        if coefficients.is_some_and(|c| c.structure() != plan.structure().key()) {
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
            derivatives: plan.order(),
            coefficients: coefficients.is_some(),
            quadratic: coefficients.is_some_and(|c| c.hessian.val().iter().any(|v| *v != 0.0)),
        })
    }
}
