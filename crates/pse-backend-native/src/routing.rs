// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Deterministic class routing with explicit representation and availability failures.
use crate::solve::BackendCapabilities;
use crate::{
    ProblemError,
    solve::{Backend, SolveIntent, SolverSelection},
};
use pse_math::{binding::VariableDomain, facts::ProblemFacts};
/// Selected execution class, including the zero-variable path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Route {
    /// Direct original-model evaluation, without a native solver attempt.
    Constant,
    /// Native implementation and its separately admitted representation.
    Native(Backend),
}
/// One backend's contextual refusal; inventory alone never grants eligibility.
#[derive(Clone, Debug)]
pub struct Eligibility {
    /// Backend whose concrete representation is assessed.
    pub backend: Backend,
    /// All applicable preparation failures; empty means eligible.
    pub reasons: Vec<String>,
}
/// Requirements established by the selected mathematical representation and policy.
#[derive(Debug)]
pub struct Requirements<'a> {
    /// Routes exposed by the caller's build. None uses the linked adapter inventory.
    pub available: Option<&'a [Backend]>,
    /// Compiler-established class and derivative facts.
    pub facts: &'a ProblemFacts,
    /// Selected analysis purpose.
    pub intent: SolveIntent,
    /// Convexity qualified against this coefficient snapshot.
    pub convex: bool,
    /// Complete effective native controls.
    pub controls: &'a crate::solve::Controls,
}
/// Project an already admitted native oracle into the same contextual selector.
/// Callers supply the represented objective/equality meaning, not a backend preference.
pub fn oracle_facts(c: &crate::OracleContract, objective: bool, equalities: bool) -> ProblemFacts {
    use pse_math::facts::BoundShape;
    ProblemFacts {
        variables: c.variables.len(),
        rows: c.rows.len(),
        objective,
        equalities,
        domains: vec![VariableDomain::Continuous; c.variables.len()],
        derivatives: c.derivatives.min(c.smoothness),
        prepared_derivatives: c.derivatives,
        bounds: c
            .variables
            .iter()
            .map(|v| match (v.lower.is_finite(), v.upper.is_finite()) {
                (false, false) => BoundShape::Free,
                (true, true) => BoundShape::Boxed,
                (true, false) if v.lower == 0.0 => BoundShape::Nonnegative,
                (false, true) if v.upper == 0.0 => BoundShape::Nonpositive,
                (true, false) => BoundShape::Lower,
                (false, true) => BoundShape::Upper,
            })
            .collect(),
        guarded: false,
        coefficients: false,
        affine_rows: vec![false; c.rows.len()],
        objective_degree: None,
        bound_assumptions: c.identity,
        quadratic: false,
    }
}
impl Requirements<'_> {
    /// Assess every algebraic backend through one policy owner.
    pub fn eligibility(&self) -> Vec<Eligibility> {
        use crate::solve::HessianMode;
        use pse_kernels::DerivativeOrder;
        use pse_math::facts::BoundShape;
        let f = self.facts;
        let continuous = f.domains.iter().all(|d| *d == VariableDomain::Continuous);
        let first = f.derivatives.min(f.prepared_derivatives) >= DerivativeOrder::First;
        let root = f.equalities && f.rows == f.variables && !f.objective && continuous;
        let root_intent = matches!(self.intent, SolveIntent::Root | SolveIntent::Initialize);
        let coefficient = f.coefficients && (!f.quadratic || continuous && self.convex);
        [
            Backend::Ipopt,
            Backend::Pounce,
            Backend::Kinsol,
            Backend::Highs,
            Backend::Clarabel,
        ]
        .into_iter()
        .map(|backend| {
            let mut reasons = Vec::new();
            let mut require = |valid: bool, reason: &str| {
                if !valid {
                    reasons.push(reason.to_owned());
                }
            };
            require(
                self.available
                    .is_none_or(|routes| routes.contains(&backend)),
                "adapter not exposed by caller",
            );
            require(backend.available(), "adapter not linked");
            require(
                self.controls.threads == 1 || backend.capabilities().parallel,
                "linked profile is serial",
            );
            if root_intent {
                require(
                    root,
                    "root analysis requires square continuous equalities without an objective",
                );
            }
            if self.intent == SolveIntent::Optimize {
                require(f.objective, "optimization requires an authored objective");
            }
            match backend {
                Backend::Kinsol => {
                    require(
                        root_intent && root,
                        "equation representation requires root intent",
                    );
                    require(
                        first,
                        "equation strategy requires prepared smooth first derivatives",
                    );
                    require(
                        f.bounds.iter().all(|b| {
                            matches!(
                                b,
                                BoundShape::Free
                                    | BoundShape::Nonnegative
                                    | BoundShape::Nonpositive
                            )
                        }),
                        "KINSOL cannot represent general bounds",
                    );
                }
                Backend::Ipopt | Backend::Pounce => {
                    require(
                        continuous && first,
                        "NLP requires prepared smooth continuous derivatives",
                    );
                    require(
                        self.controls.hessian != HessianMode::Exact
                            || f.derivatives.min(f.prepared_derivatives) >= DerivativeOrder::Second,
                        "exact Hessian requires prepared second derivatives",
                    );
                }
                Backend::Highs => require(
                    coefficient && !root_intent,
                    "HiGHS requires affine constraints and an admitted LP/MILP/convex QP",
                ),
                Backend::Clarabel => {
                    require(false, "Clarabel requires an explicit cone representation")
                }
                _ => require(false, "not an algebraic backend"),
            }
            Eligibility { backend, reasons }
        })
        .collect()
    }
    /// Deterministic route; explicit selection never silently falls back.
    pub fn select(&self, selection: SolverSelection) -> Result<Route, ProblemError> {
        self.controls.validate()?;
        if self.intent == SolveIntent::Optimize && !self.facts.objective {
            return Err(ProblemError::Contract(
                "optimization needs an authored objective".into(),
            ));
        }
        if self.facts.variables == 0 {
            return Ok(Route::Constant);
        }
        let choices = self.eligibility();
        let admitted = |backend| {
            choices
                .iter()
                .any(|c| c.backend == backend && c.reasons.is_empty())
        };
        let preferred: &[Backend] =
            if matches!(self.intent, SolveIntent::Root | SolveIntent::Initialize) {
                &[Backend::Kinsol, Backend::Ipopt, Backend::Pounce]
            } else {
                &[Backend::Highs, Backend::Ipopt, Backend::Pounce]
            };
        let selected = match selection {
            SolverSelection::Explicit(b) => b,
            SolverSelection::Auto => preferred
                .iter()
                .copied()
                .find(|b| admitted(*b))
                .ok_or_else(|| {
                    ProblemError::Contract(format!("no eligible native route: {choices:?}"))
                })?,
        };
        if !self.available(selected) {
            return Err(ProblemError::Unavailable {
                backend: selected,
                alternatives: choices
                    .iter()
                    .filter(|c| c.reasons.is_empty())
                    .map(|c| c.backend)
                    .collect(),
            });
        }
        if !admitted(selected) {
            return Err(ProblemError::Contract(format!(
                "selected {selected:?} is ineligible: {choices:?}"
            )));
        }
        Ok(Route::Native(selected))
    }
    fn available(&self, backend: Backend) -> bool {
        backend.available()
            && self
                .available
                .is_none_or(|routes| routes.contains(&backend))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn caller_inventory_limits_automatic_selection() {
        let facts = root_facts();
        let requirements = Requirements {
            available: Some(&[]),
            facts: &facts,
            intent: SolveIntent::Root,
            convex: false,
            controls: &crate::solve::Controls::default(),
        };
        assert!(requirements.select(SolverSelection::Auto).is_err());
        if Backend::Ipopt.available() {
            let requirements = Requirements {
                available: Some(&[Backend::Ipopt]),
                ..requirements
            };
            assert_eq!(
                requirements.select(SolverSelection::Auto).unwrap(),
                Route::Native(Backend::Ipopt)
            );
        }
    }
    fn select(
        f: &ProblemFacts,
        intent: SolveIntent,
        selection: SolverSelection,
        convex: bool,
    ) -> Result<Route, ProblemError> {
        Requirements {
            available: None,
            facts: f,
            intent,
            convex,
            controls: &crate::solve::Controls::default(),
        }
        .select(selection)
    }
    fn root_facts() -> ProblemFacts {
        ProblemFacts {
            variables: 1,
            rows: 1,
            objective: false,
            equalities: true,
            domains: vec![VariableDomain::Continuous],
            derivatives: pse_kernels::DerivativeOrder::Second,
            prepared_derivatives: pse_kernels::DerivativeOrder::Second,
            bounds: vec![pse_math::facts::BoundShape::Free],
            guarded: false,
            coefficients: false,
            affine_rows: vec![false],
            objective_degree: Some(0),
            bound_assumptions: pse_ids::ContentHash::from_bytes([0; 32]),
            quadratic: false,
        }
    }
    #[test]
    fn contextual_bounds_derivatives_and_threads_are_preparation_facts() {
        let mut f = root_facts();
        f.bounds[0] = pse_math::facts::BoundShape::Boxed;
        let mut controls = crate::solve::Controls::default();
        let assess = |f: &ProblemFacts, c: &crate::solve::Controls| {
            Requirements {
                available: None,
                facts: f,
                intent: SolveIntent::Root,
                convex: false,
                controls: c,
            }
            .eligibility()
        };
        let q = assess(&f, &controls);
        assert!(
            q.iter()
                .find(|e| e.backend == Backend::Kinsol)
                .unwrap()
                .reasons
                .iter()
                .any(|r| r.contains("general bounds"))
        );
        assert!(
            q.iter()
                .find(|e| e.backend == Backend::Ipopt)
                .unwrap()
                .reasons
                .iter()
                .all(|r| r == "adapter not linked")
        );
        f.prepared_derivatives = pse_kernels::DerivativeOrder::Value;
        assert!(
            assess(&f, &controls)
                .iter()
                .filter(|e| matches!(
                    e.backend,
                    Backend::Ipopt | Backend::Pounce | Backend::Kinsol
                ))
                .all(|e| e.reasons.iter().any(|r| r.contains("derivative")))
        );
        f = root_facts();
        controls.threads = 2;
        assert!(
            assess(&f, &controls)
                .iter()
                .find(|e| e.backend == Backend::Kinsol)
                .unwrap()
                .reasons
                .iter()
                .any(|r| r.contains("serial"))
        );
        f.variables = 0;
        assert!(
            Requirements {
                available: None,
                facts: &f,
                intent: SolveIntent::Optimize,
                convex: false,
                controls: &controls
            }
            .select(SolverSelection::Auto)
            .is_err()
        );
    }
    #[test]
    fn class_refusals_do_not_relax_discrete_or_square_requirements() {
        let f = ProblemFacts {
            variables: 2,
            rows: 1,
            objective: true,
            equalities: true,
            domains: vec![VariableDomain::Integer; 2],
            derivatives: pse_kernels::DerivativeOrder::Second,
            prepared_derivatives: pse_kernels::DerivativeOrder::Second,
            bounds: vec![pse_math::facts::BoundShape::Free; 2],
            guarded: false,
            coefficients: true,
            quadratic: true,
            affine_rows: vec![true],
            objective_degree: Some(2),
            bound_assumptions: pse_ids::ContentHash::from_bytes([0; 32]),
        };
        assert!(select(&f, SolveIntent::Optimize, SolverSelection::Auto, true).is_err());
        assert!(select(&f, SolveIntent::Root, SolverSelection::Auto, true).is_err());
        let mut f = f;
        // Coefficient MILPs and opaque nonlinear expressions are not conic data.
        f.quadratic = false;
        assert!(
            select(
                &f,
                SolveIntent::Optimize,
                SolverSelection::Explicit(Backend::Clarabel),
                true,
            )
            .is_err()
        );
        f.domains.fill(VariableDomain::Continuous);
        f.coefficients = false;
        assert!(
            select(
                &f,
                SolveIntent::Optimize,
                SolverSelection::Explicit(Backend::Clarabel),
                true,
            )
            .is_err()
        );
        f.variables = 0;
        assert_eq!(
            select(&f, SolveIntent::Optimize, SolverSelection::Auto, false).unwrap(),
            Route::Constant
        );
    }
}
