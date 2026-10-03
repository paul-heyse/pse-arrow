// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Contextual readiness through the workflow/compiler/runtime boundary, before any attempt.
use super::*;
use crate::workflow::{ModelingPackage, ModelingSolvePreparation, tests as fixture};
use pse_compiler::workspace::ModelingCaseBindings;
use pse_ids::SemanticId;
use pse_kernels::DerivativeOrder;
use pse_modeling::{Bindings, DeclarationId, Limits};

fn package(text: &str) -> (ModelingPackage, DeclarationId) {
    let rows = pse_authoring::language::parse(
        text,
        SemanticId::NIL,
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap();
    let root = rows
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let package = fixture::runtime()
        .modeling_package(rows, fixture::physical())
        .unwrap();
    (package, root)
}

async fn prepare(
    text: &str,
    intent: SolveIntent,
    selection: SolverSelection,
) -> ModelingSolvePreparation {
    let (package, root) = package(text);
    package
        .prepare_solve(
            root,
            pse_modeling::specialize::root_instance(root),
            Bindings::default(),
            Limits::default(),
            ModelingCaseBindings::default(),
            DerivativeOrder::First,
            fixture::compiler_profile(),
            SolverProfile {
                intent,
                selection,
                ..Default::default()
            },
            NumericalInputs::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap()
}

fn ready(solve: &PreparedSolve) -> &AlgebraicCase {
    let decision = solve.route_decision().unwrap();
    assert_eq!(decision.state, routing::AssessmentState::Ready);
    assert!(decision.evidence.is_empty(), "{decision:?}");
    assert!(decision.artifacts.is_empty(), "{decision:?}");
    let Representation::Algebraic(case) = &solve.representation else {
        panic!("expected an authored algebraic representation");
    };
    assert!(
        case.case.is_some(),
        "the selected numeric program is retained"
    );
    case
}

#[cfg(any(feature = "solver-ipopt", feature = "solver-pounce"))]
#[tokio::test]
async fn auto_boxed_square_upgrades_first_to_ready_nlp_second() {
    let prepared = prepare(
        "package p { def Root { var x: Scalar; annotation bounds x(0.25, 2); annotation start x(1.5); eq square: x*x == 1; } }",
        SolveIntent::Root, SolverSelection::Auto,
    ).await;
    let original = &prepared.model.case.compiled().plan;
    assert_eq!(original.order(), DerivativeOrder::First);
    assert_eq!(original.available_order(), DerivativeOrder::Second);
    assert!(
        original
            .supports()
            .iter()
            .all(|support| support.order() == DerivativeOrder::First)
    );
    let case = ready(&prepared.solve);
    let Route::Native(backend) = prepared.solve.route() else {
        panic!("expected a native route");
    };
    assert_eq!(
        execution::adapter(backend).representation(),
        execution::Representation::Nlp
    );
    assert_eq!(prepared.solve.required_order(), DerivativeOrder::Second);
    let upgraded = &case.prepared.prepared.plan;
    assert_eq!(upgraded.columns(), original.columns());
    let bounds = |plan: &pse_math::assembly::CasePlan| {
        plan.structure()
            .variables()
            .iter()
            .map(|variable| (variable.port.id, variable.lower, variable.upper))
            .collect::<Vec<_>>()
    };
    assert_eq!(bounds(upgraded), bounds(original));
    assert_eq!(bounds(upgraded)[0].1, Some(0.25));
    assert_eq!(bounds(upgraded)[0].2, Some(2.0));
    assert!(
        upgraded
            .supports()
            .iter()
            .all(|support| support.order() == DerivativeOrder::Second)
    );
    assert_eq!(
        original.order(),
        DerivativeOrder::First,
        "the weaker product remains immutable"
    );
    assert!(
        matches!(
            case.prepared.prepared.presolve.class_status,
            pse_math::presolve::ClassStatus::Unassessed
        ),
        "Root readiness does not request coefficient objective proof"
    );
}

const LINEAR: &str = "package p { def Root { var x: Scalar; annotation bounds x(0, 2); annotation start x(1); let f: Scalar = x; annotation objective f(minimize); eq floor: x >= 0.5; } }";

#[cfg(feature = "solver-highs")]
#[tokio::test]
async fn auto_linear_retains_established_coefficient_readiness() {
    let prepared = prepare(LINEAR, SolveIntent::Optimize, SolverSelection::Auto).await;
    let case = ready(&prepared.solve);
    assert_eq!(prepared.solve.route(), Route::Native(Backend::Highs));
    assert_eq!(
        case.prepared.prepared.presolve.class_status,
        pse_math::presolve::ClassStatus::Established
    );
    let coefficients = case.prepared.prepared.coefficients.as_ref().unwrap();
    assert_eq!(
        coefficients.objective.len(),
        case.prepared.prepared.plan.columns().len()
    );
    assert_eq!(prepared.solve.required_order(), DerivativeOrder::First);
}

#[tokio::test]
async fn explicit_coefficient_cone_retains_representation_and_paired_proof() {
    let prepared = prepare(
        LINEAR,
        SolveIntent::Optimize,
        SolverSelection::Explicit(Backend::Clarabel),
    )
    .await;
    let case = ready(&prepared.solve);
    assert_eq!(prepared.solve.route(), Route::Native(Backend::Clarabel));
    assert_eq!(
        case.prepared.prepared.presolve.class_status,
        pse_math::presolve::ClassStatus::Established
    );
    assert!(case.prepared.prepared.coefficients.is_some());
    let (lowered, owner) = case.coefficient_cone.as_ref().unwrap();
    assert!(owner.size() > 0);
    assert_eq!(
        lowered.problem.contract.variables.len(),
        case.prepared.prepared.plan.columns().len()
    );
    assert!(lowered.problem.validate(&lowered.evidence).is_ok());
}

#[tokio::test]
async fn auto_recognized_cone_retains_representation_and_paired_proof() {
    let prepared = prepare(
        "package p { def Root { var x: Scalar; annotation bounds x(-1, 2); annotation start x(0.5); let f: Scalar = exp(x); annotation objective f(minimize); eq floor: x >= 0; } }",
        SolveIntent::Optimize, SolverSelection::Auto,
    ).await;
    let case = ready(&prepared.solve);
    assert_eq!(prepared.solve.route(), Route::Native(Backend::Clarabel));
    assert!(case.prepared.prepared.facts.convexity.cone());
    assert!(case.prepared.prepared.coefficients.is_none());
    assert!(case.coefficient_cone.is_none());
    let (recognized, proof, owner) = case.recognized.as_ref().unwrap();
    assert!(owner.size() > 0);
    assert!(recognized.problem.validate(proof.as_ref()).is_ok());
    assert_eq!(prepared.solve.required_order(), DerivativeOrder::First);
}
