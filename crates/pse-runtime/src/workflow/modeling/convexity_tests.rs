// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Convexity as a compiler fact through the real pipeline (ADR-0121): recognized cone
//! programs route to Clarabel automatically, and numerical PSD evidence stays a
//! qualification of the one request that states it.
use super::*;
use crate::math::solves::{ConvexityPolicy, NumericalInputs, Outcome, SolverProfile};
use crate::workflow::tests as fixture;
use pse_backend_native::{
    ProblemError,
    routing::{Ineligible, Route},
    solve::{Backend, ProblemClass, SolveIntent, SolverSelection},
};
use pse_compiler::workspace::ModelingCaseBindings;
use pse_kernels::DerivativeOrder;
use pse_math::convexity::{ConvexityAssessment, ConvexityClass, Unrecognized};

async fn package(text: &str) -> (ModelingPackage, DeclarationId) {
    let runtime = fixture::runtime();
    let physical = fixture::physical();
    let rows = pse_authoring::language::parse(
        text,
        SemanticId::NIL,
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap();
    let root = rows
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    let package = runtime.modeling_package(rows, physical).await.unwrap();
    (package, root)
}
fn profile(selection: SolverSelection, convexity: ConvexityPolicy) -> SolverProfile {
    let mut profile = fixture::profile();
    profile.intent = SolveIntent::Optimize;
    profile.selection = selection;
    profile.convexity = convexity;
    profile
}
fn refused_class(error: &WorkflowError, backend: Backend) -> &[ProblemClass] {
    let WorkflowError::Math(crate::math::MathRuntimeError::Solve(ProblemError::RouteRefused(
        decision,
    ))) = error
    else {
        panic!("expected a typed routing refusal: {error}");
    };
    assert_eq!(decision.selection, SolverSelection::Explicit(backend));
    assert!(decision.selected.is_none());
    assert!(matches!(
        decision.refusal.as_ref(),
        Some(pse_backend_native::routing::Refusal::Ineligible(refused)) if *refused == backend
    ));
    decision
        .eligibility
        .iter()
        .find(|assessment| assessment.backend == backend)
        .unwrap()
        .reasons
        .iter()
        .find_map(|reason| match reason {
            Ineligible::Class { problem } => Some(problem.as_slice()),
            _ => None,
        })
        .unwrap()
}
async fn prepare(
    package: &ModelingPackage,
    root: DeclarationId,
    profile: SolverProfile,
) -> Result<ModelingSolvePreparation, WorkflowError> {
    package
        .prepare_solve(
            root,
            pse_modeling::specialize::root_instance(root),
            Bindings::default(),
            Limits::default(),
            ModelingCaseBindings::default(),
            DerivativeOrder::First,
            fixture::compiler_profile(),
            profile,
            NumericalInputs::default(),
            &crate::CancelSource::new(),
        )
        .await
}
async fn reports(
    package: &ModelingPackage,
    prepared: ModelingSolvePreparation,
    labels: &[&str],
) -> (Backend, Vec<f64>) {
    let result = package
        .solve_case(
            prepared,
            fixture::compiler_profile(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let Outcome::Native(native) = &result.outcome else {
        panic!("expected a native outcome: {:?}", result.outcome);
    };
    assert!(
        result.accepted,
        "{:?}; {:?}; {:?} {:?} {:?}",
        result.validation_error,
        native.termination,
        native.qualification,
        native.quality,
        native.metrics
    );
    let values = labels
        .iter()
        .map(|label| {
            result
                .reports
                .iter()
                .find(|r| r.label == *label)
                .unwrap()
                .value
        })
        .collect();
    (native.backend, values)
}

/// An ideal Gibbs-energy minimization: `Σ nᵢ μᵢ + Σ nᵢ log(nᵢ/N)` over a mole balance, with
/// `N = Σ nᵢ`. Every term is linear or a relative entropy, so preparation recognizes an
/// exponential-cone program and records it as a fact.
const GIBBS: &str = "package p { def Root {
    var a: Scalar; var b: Scalar;
    annotation bounds a(0.001, 1); annotation bounds b(0.001, 1);
    annotation start a(0.5); annotation start b(0.5);
    let total: Scalar = a + b;
    let g: Scalar = -a + 0.5*b + a*log(a/total) + b*log(b/total);
    annotation objective g(minimize);
    eq balance: a + b == 1;
    annotation report a(\"a\"); annotation report b(\"b\"); annotation report g(\"g\");
} }";

/// ADR-0121 Outcomes 1, 3 and 6: preparation recognizes the Gibbs problem as a continuous
/// exponential-cone program; automatic routing selects Clarabel for it, the cone runner
/// lowers the rebuilt program, and the candidate, re-evaluated against the original model,
/// is the analytic minimum `nᵢ ∝ exp(−μᵢ)`.
#[tokio::test]
async fn recognized_exp_cone_routes_to_clarabel() {
    let (package, root) = package(GIBBS).await;
    let prepared = prepare(
        &package,
        root,
        profile(SolverSelection::Auto, ConvexityPolicy::Exact),
    )
    .await
    .unwrap();
    let facts = &prepared.model.case.compiled().facts;
    let ConvexityClass::Cone(summary) = facts.convexity.class else {
        panic!("recognized: {:?}", facts.convexity)
    };
    assert_eq!(summary.exponential, 2, "{summary:?}");
    assert_eq!(prepared.solve.route(), Route::Native(Backend::Clarabel));
    let (backend, values) = reports(&package, prepared, &["a", "b", "g"]).await;
    assert_eq!(backend, Backend::Clarabel);
    // The minimum is flat: the objective meets the resolved gap budget, and the argument
    // is within its square root.
    let a = 1.0 / (1.0 + (-1.5_f64).exp());
    let b = 1.0 - a;
    let g = -a + 0.5 * b + a * a.ln() + b * b.ln();
    assert!((values[2] - g).abs() < 1e-7, "{values:?} vs {g}");
    assert!((values[0] - a).abs() < 1e-4, "{values:?} vs {a}");
    assert!((values[1] - b).abs() < 1e-4, "{values:?} vs {b}");
    assert!((values[0] + values[1] - 1.0).abs() < 1e-8, "{values:?}");
}

/// Maximizing the same Gibbs energy is not a convex program: preparation records the
/// objective as unrecognized, and no route reaches Clarabel.
#[tokio::test]
async fn unrecognized_gibbs_maximization_not_routed_to_clarabel() {
    let (package, root) = package(&GIBBS.replace("g(minimize)", "g(maximize)")).await;
    let explicit = prepare(
        &package,
        root,
        profile(
            SolverSelection::Explicit(Backend::Clarabel),
            ConvexityPolicy::Exact,
        ),
    )
    .await;
    let error = explicit.unwrap_err();
    assert!(!refused_class(&error, Backend::Clarabel).is_empty());
    if let Ok(prepared) = prepare(
        &package,
        root,
        profile(SolverSelection::Auto, ConvexityPolicy::Exact),
    )
    .await
    {
        assert_eq!(
            prepared.model.case.compiled().facts.convexity.class,
            ConvexityClass::Unrecognized(Unrecognized::Curvature { row: None })
        );
        assert_ne!(prepared.solve.route(), Route::Native(Backend::Clarabel));
    }
}

/// `x² + 2xy + c·y²` with `c` one part in 10¹² below one: exactly indefinite, numerically
/// PSD within a 10⁻⁹ tolerance.
const NEARLY_PSD: &str = "package p { def Root {
    param c: Scalar = 0.999999999999;
    var x: Scalar; var y: Scalar;
    annotation bounds x(-10, 10); annotation bounds y(-10, 10);
    annotation start x(0); annotation start y(0);
    let f: Scalar = x*x + 2*x*y + c*y*y + x;
    annotation objective f(minimize);
    eq balance: x - y == 1;
    annotation report f(\"f\");
} }";

/// ADR-0121 Outcome 4 and review F05: the exact fact is the only authority by default, so
/// an exactly indefinite quadratic is a nonconvex class that HiGHS refuses. An explicit
/// numerical policy qualifies that one request's objective as PSD and adds
/// `convex_quadratic` for it, with its evidence recorded as numerical; the fact is not
/// changed, and the next request without the policy is refused again.
#[tokio::test]
async fn numerical_psd_only_under_explicit_policy() {
    let (package, root) = package(NEARLY_PSD).await;
    let highs = SolverSelection::Explicit(Backend::Highs);
    let refused = |result: Result<ModelingSolvePreparation, WorkflowError>| {
        let error = result.unwrap_err();
        assert!(refused_class(&error, Backend::Highs).contains(&ProblemClass::NonconvexQuadratic));
    };
    refused(prepare(&package, root, profile(highs, ConvexityPolicy::Exact)).await);
    let numerical = ConvexityPolicy::Numerical {
        absolute: 1e-9,
        relative: 1e-9,
    };
    let prepared = prepare(&package, root, profile(highs, numerical))
        .await
        .unwrap();
    assert_eq!(prepared.solve.route(), Route::Native(Backend::Highs));
    // Exact coefficient evidence is an explicit class demand, independent of
    // this selected solve's numerical PSD qualification.
    let exact = package
        .runtime
        .shared
        .math()
        .discover_class(
            prepared.model.case.clone(),
            prepared.model.values.clone(),
            &profile(highs, ConvexityPolicy::Exact),
        )
        .await
        .unwrap();
    assert_eq!(
        exact.compiled().facts.convexity.class,
        ConvexityClass::Unrecognized(Unrecognized::Indefinite)
    );
    assert!(matches!(
        prepared
            .solve
            .quadratic_evidence()
            .and_then(|e| e.assessment()),
        Some(ConvexityAssessment::NumericalPsd { .. })
    ));
    assert!(
        prepared
            .solve
            .eligibility()
            .iter()
            .find(|e| e.backend == Backend::Highs)
            .is_some_and(|e| e.reasons.is_empty())
    );
    // The qualification served that request only.
    let again = prepare(&package, root, profile(highs, ConvexityPolicy::Exact)).await;
    refused(again);
    if let Ok(auto) = prepare(
        &package,
        root,
        profile(SolverSelection::Auto, ConvexityPolicy::Exact),
    )
    .await
    {
        assert_ne!(auto.solve.route(), Route::Native(Backend::Highs));
        let highs = auto
            .solve
            .eligibility()
            .iter()
            .find(|e| e.backend == Backend::Highs)
            .unwrap()
            .reasons
            .clone();
        assert!(
            highs.iter().any(|r| matches!(
                r,
                Ineligible::Class { problem } if problem.contains(&ProblemClass::NonconvexQuadratic)
            )),
            "{highs:?}"
        );
    }
}
