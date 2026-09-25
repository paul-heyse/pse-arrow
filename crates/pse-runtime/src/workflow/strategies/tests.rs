// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::math::solves::{BackendSettings, Outcome};
#[cfg(feature = "solver-kinsol")]
use crate::workflow::tests::{compiler_profile, declaration};
use crate::workflow::tests::{id, physical, runtime};
pub(super) fn profile(intent: SolveIntent) -> SolverProfile {
    SolverProfile {
        intent,
        selection: SolverSelection::Auto,
        controls: Controls::default(),
        backend: BackendSettings::Default,
        presolve: Default::default(),
        numerics: Default::default(),
        convexity: Default::default(),
    }
}
#[tokio::test]
async fn explicit_cone_request_runs_without_an_algebraic_compiler_flag() {
    let physical = physical();
    let q = physical.quantities.neutral_dimensionless().unwrap();
    let unit = physical.quantities.quantity_type(q).unwrap().canonical_unit;
    let port = |symbol_id| AnalysisPort {
        symbol_id,
        quantity_id: q.as_id(),
        unit_id: unit.as_id(),
    };
    let request = ConicRequest {
        variables: vec![port(id(1))],
        rows: vec![port(id(2))],
        objective_port: port(SemanticId::NIL),
        quadratic: native::conic::Matrix::zeros((1, 1)),
        objective: vec![1.0],
        constraints: native::conic::Matrix::new(1, 1, vec![0, 1], vec![0], vec![-1.0]),
        rhs: vec![-2.0],
        cones: vec![native::conic::Cone::NonnegativeConeT(1)],
        objective_constant: 3.0,
        gram_factors: vec![],
        gram_weights: vec![],
    };
    let prepared = runtime()
        .prepare_conic(request, &physical, profile(SolveIntent::Optimize))
        .await
        .unwrap();
    assert_eq!(
        prepared.solve().route(),
        native::routing::Route::Native(Backend::Clarabel)
    );
    let result = prepared.start().unwrap().finish().await.unwrap();
    let Outcome::Native(report) = &result.outcomes[0] else {
        panic!("{:?}", result.outcomes)
    };
    assert!((report.candidate.as_ref().unwrap().primal[0] - 2.0).abs() < 1e-6);
    assert!((report.candidate.as_ref().unwrap().objective.unwrap() - 5.0).abs() < 1e-6);
    assert!(report.quality.as_ref().unwrap().feasible());
    assert_eq!(
        report.qualification,
        Qualification::OptimalWithinTolerance,
        "{report:?}"
    );
}
#[cfg(feature = "solver-kinsol")]
#[tokio::test]
async fn failed_continuation_preserves_original_bindings_and_prior_solved_unknowns() {
    use std::collections::BTreeMap;
    let mut d = declaration();
    let q = d.cases[0].variables[0].port.quantity_id;
    let unit = d.cases[0].variables[0].port.unit_id;
    d.definitions[0].sources = vec!["x*x-a".into()];
    d.definitions[0]
        .formals
        .push(serde_json::from_value(serde_json::json!({"path":"a","quantity_id":q})).unwrap());
    let c = &mut d.cases[0];
    c.variables[0].fixed = false;
    c.rows[0].lower = Some(0.0);
    c.rows[0].upper = Some(0.0);
    c.parameters.push(
        serde_json::from_value(
            serde_json::json!({"symbol_id":id(9),"quantity_id":q,"unit_id":unit}),
        )
        .unwrap(),
    );
    c.values
        .push(serde_json::from_value(serde_json::json!({"symbol_id":id(9),"value":4.0})).unwrap());
    c.instances[0].slots.push(
        serde_json::from_value(
            serde_json::json!({"source_id":id(9),"formal_quantity_id":q,"formal_unit_id":unit}),
        )
        .unwrap(),
    );
    let revision = crate::workflow::ModelBuilder::from_declaration(runtime(), d, physical())
        .freeze()
        .unwrap();
    let original = revision.identity();
    let prepared = revision
        .prepare_initialization(
            id(5),
            crate::math::initialization::InitializationProfile {
                selection: SolverSelection::Auto,
                controls: Controls {
                    start: StartPolicy::PreviousAccepted,
                    iterations: 50,
                    ..Default::default()
                },
                linear: native::kinsol::Linear::Klu,
                numerics: Default::default(),
                stages: vec![
                    BTreeMap::from([(id(9), 9.0)]),
                    BTreeMap::from([(id(9), -1.0)]),
                ],
            },
            compiler_profile(),
        )
        .await
        .unwrap();
    let report = prepared.start().unwrap().finish().await.unwrap();
    assert_eq!(report.original.scalars[&id(9)], 4.0);
    assert_eq!(report.original.scalars[&id(1)], 2.0);
    assert_eq!(report.completed_stages, 1, "{report:?}");
    assert!((report.values.scalars[&id(1)] - 3.0).abs() < 1e-6);
    assert!(!report.values.scalars.contains_key(&id(9)));
    assert_eq!(report.stages[1].candidate.scalars[&id(9)], -1.0);
    assert!(!report.stages[1].completed);
    assert!(!report.original_bindings_restored);
    assert_eq!(revision.identity(), original);
}
