// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Deterministic class routing with explicit representation and availability failures.
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
/// Choose from proved algebraic facts. Convexity must be revalidated against the
/// current coefficient snapshot before compiling a QP; this function cannot grant it.
pub fn select(
    f: &ProblemFacts,
    intent: SolveIntent,
    selection: SolverSelection,
    convex: bool,
) -> Result<Route, ProblemError> {
    if f.variables == 0 {
        return Ok(Route::Constant);
    }
    if intent == SolveIntent::Optimize && !f.objective {
        return Err(ProblemError::Contract(
            "optimization needs an authored objective".into(),
        ));
    }
    let continuous = f.domains.iter().all(|d| *d == VariableDomain::Continuous);
    let root = f.equalities
        && f.rows == f.variables
        && !f.objective
        && continuous
        && f.derivatives >= pse_kernels::DerivativeOrder::First;
    let nlp = continuous && f.derivatives >= pse_kernels::DerivativeOrder::First;
    let linear = f.coefficients && (!f.quadratic || continuous && convex);
    let default = match intent {
        SolveIntent::Root | SolveIntent::Initialize if root => Backend::Kinsol,
        SolveIntent::Root | SolveIntent::Initialize => {
            return Err(ProblemError::Contract(
                "root intent requires a square smooth continuous equality representation".into(),
            ));
        }
        SolveIntent::Optimize if linear => Backend::Highs,
        SolveIntent::Optimize | SolveIntent::FeasiblePoint if nlp => Backend::Ipopt,
        _ => {
            return Err(ProblemError::Contract(
                "unsupported mathematical class; no relaxation or reformulation is implied".into(),
            ));
        }
    };
    let selected = match selection {
        SolverSelection::Auto => default,
        SolverSelection::Explicit(b) => b,
    };
    let eligible = |selected| match selected {
        Backend::Highs => linear && intent == SolveIntent::Optimize,
        Backend::Kinsol => root && matches!(intent, SolveIntent::Root | SolveIntent::Initialize),
        Backend::Ipopt | Backend::Pounce => {
            nlp && matches!(intent, SolveIntent::Optimize | SolveIntent::FeasiblePoint)
        }
        Backend::Clarabel => false,
    };
    if !eligible(selected) {
        return Err(ProblemError::Contract(format!(
            "{selected:?} cannot consume this representation/intent"
        )));
    }
    if !selected.available() {
        return Err(ProblemError::Unavailable {
            backend: selected,
            alternatives: [
                Backend::Ipopt,
                Backend::Pounce,
                Backend::Kinsol,
                Backend::Highs,
                Backend::Clarabel,
            ]
            .into_iter()
            .filter(|b| b.available() && eligible(*b))
            .collect(),
        });
    }
    Ok(Route::Native(selected))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn class_refusals_do_not_relax_discrete_or_square_requirements() {
        let f = ProblemFacts {
            variables: 2,
            rows: 1,
            objective: true,
            equalities: true,
            domains: vec![VariableDomain::Integer; 2],
            derivatives: pse_kernels::DerivativeOrder::Second,
            coefficients: true,
            quadratic: true,
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
