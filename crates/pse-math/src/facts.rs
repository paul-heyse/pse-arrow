// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Compiler-established mathematical facts, independent of any solver library.
use crate::{MathError, assembly::CasePlan, coefficients::Coefficients, convexity::Convexity};
use pse_model::generated::enums::ModelingVariableDomain;
use std::sync::atomic::AtomicBool;
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
    /// Objectives optimized together: none, one, or the lexicographic levels (ADR-0111),
    /// which only a native lexicographic route optimizes at once.
    pub objectives: usize,
    /// Every row is a finite equality.
    pub equalities: bool,
    /// One source-declared domain per free variable.
    pub domains: Vec<ModelingVariableDomain>,
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
    /// Native constraint handlers the structure requires (ADR-0104), sorted and unique.
    pub native: Vec<pse_model::generated::enums::NativeConstraintForm>,
    /// Structural requirements the formulation's lowerings place on the route, such as the
    /// l1 exact penalty an authored `penalty(l1)` realization states (ADR-0104 §5).
    pub requirements: Vec<pse_model::generated::enums::ModelingStructuralRequirement>,
    /// Exact convexity established at preparation (ADR-0121): a coefficient program's
    /// objective quadratic by an exact Gram certificate, any other program by the curvature
    /// pass. It carries the identity of the values it consumed and rebinds with them.
    pub convexity: Convexity,
}
impl ProblemFacts {
    /// Derive facts from a plan and its optional, current coefficient snapshot. A
    /// coefficient snapshot's objective quadratic is certified here, in the objective's
    /// minimization sense.
    ///
    /// # Errors
    /// A snapshot of other values or structure, or cancellation.
    pub fn from_plan(
        plan: &CasePlan,
        coefficients: Option<&Coefficients>,
        facts: &crate::presolve::Facts,
        cancel: &AtomicBool,
    ) -> Result<Self, MathError> {
        if facts.structure != plan.structure().key()
            || coefficients.is_some_and(|c| !c.matches_facts(facts))
        {
            return Err(MathError::Contract(
                "class coefficients do not match plan".into(),
            ));
        }
        let sign = plan.structure().objective().map_or(1.0, |o| o.sense.sign());
        let convexity = match (coefficients, &facts.curvature) {
            (Some(c), _) => Convexity::of_coefficients(c, sign, cancel)?,
            (None, Some(fact)) => fact.clone(),
            (None, None) => Convexity::not_assessed(facts.key),
        };
        Ok(Self {
            variables: plan.columns().len(),
            rows: plan.structure().rows().len(),
            objective: plan.structure().objective().is_some(),
            objectives: plan.structure().objectives().len(),
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
                        v.lower.is_some() || v.domain == ModelingVariableDomain::Binary,
                        v.upper.is_some() || v.domain == ModelingVariableDomain::Binary,
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
            native: plan
                .structure()
                .native()
                .iter()
                .map(pse_model::forms::NativeConstraint::form)
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect(),
            requirements: plan.structure().requirements().to_vec(),
            convexity,
        })
    }
}
impl crate::presolve::Facts {
    /// The one coefficient-class rule: every row is proved affine, the objective degree is
    /// at most two and every retained domain obligation is discharged under these
    /// assumptions. Preparation, routing facts and coefficient projection all consume it.
    #[must_use]
    pub fn coefficient_eligible(&self) -> bool {
        self.affine.iter().all(Option::is_some)
            && self.objective_degree.is_some_and(|d| d <= 2)
            && self.lexicographic_degree.is_some_and(|d| d <= 1)
            && self
                .obligations
                .values()
                .all(|s| *s == crate::presolve::ObligationStatus::Discharged)
    }
}

#[cfg(test)]
mod tests {
    use crate::presolve::{AffineRow, Facts, ObligationStatus};
    use pse_ids::{ContentHash, SemanticId};
    use std::collections::BTreeMap;
    fn facts() -> Facts {
        Facts {
            key: ContentHash::from_bytes([1; 32]),
            structure: ContentHash::from_bytes([2; 32]),
            values: BTreeMap::new(),
            affine: vec![Some(AffineRow {
                entries: BTreeMap::from([(0, 1.0)]),
                constant: 0.0,
            })],
            row_sources: vec![vec![]],
            tapes: vec![],
            complete: vec![true],
            has_guards: false,
            signs: BTreeMap::new(),
            obligations: BTreeMap::from([(
                SemanticId::from_bytes([3; 16]),
                ObligationStatus::Discharged,
            )]),
            objective_linear: vec![true],
            objective_degree: Some(2),
            curvature: None,
            lexicographic_degree: Some(0),
        }
    }
    #[test]
    fn coefficient_eligible_requires_affine_rows_quadratic_degree_and_discharged_obligations() {
        assert!(facts().coefficient_eligible());
        let mut opaque = facts();
        opaque.affine[0] = None;
        assert!(!opaque.coefficient_eligible());
        let mut cubic = facts();
        cubic.objective_degree = Some(3);
        assert!(!cubic.coefficient_eligible());
        let mut unknown = facts();
        unknown.objective_degree = None;
        assert!(!unknown.coefficient_eligible());
        for status in [ObligationStatus::Violated, ObligationStatus::Unestablished] {
            let mut guarded = facts();
            guarded.obligations.values_mut().for_each(|s| *s = status);
            assert!(!guarded.coefficient_eligible());
        }
    }
}
