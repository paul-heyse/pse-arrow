// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Phase disappearance authored as complementarity through the real pipeline (Plan 22 M5a,
//! ADR-0104 §5). The authored `ComplementarityVLE` of the reference `equilibrium` package
//! and its `bt_ideal` flash fixtures state every equation; the realizations only lower the
//! complementarity pairs.
use super::fixtures::*;
use pse_backend_native::solve::Backend;
use pse_ids::SemanticId;
use pse_model::generated::enums::ModelingStructuralRequirement as Requirement;
use pse_runtime::{
    CancelSource,
    math::solves::SolverProfile,
    workflow::{ModelingPackage, RunReport},
};
use std::collections::BTreeMap;
use std::time::Duration;

/// The seed package with the authored `complementarity_*` fixtures of `bt_ideal` restored,
/// and their identities by name.
fn with_complementarity_fixtures(
    package: &ModelingPackage,
) -> (ModelingPackage, BTreeMap<String, SemanticId>) {
    let path = fixture("")
        .ancestors()
        .find(|p| p.join("packages/reference").is_dir())
        .unwrap()
        .join("packages/reference/seed-data/models/bt-ideal.pse");
    let rows = pse_authoring::language::parse(
        &std::fs::read_to_string(path).unwrap(),
        SemanticId::NIL,
        pse_authoring::language::IdentityPolicy::Explicit,
        Default::default(),
    )
    .unwrap();
    let tests = rows
        .iter()
        .filter(|r| r.value.kind.as_str() == "test" && r.name.starts_with("complementarity_"))
        .map(|r| (r.name.clone(), r.declaration_id.as_id()))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(tests.len(), 9, "3 feeds × 3 realizations");
    let mut selected = package.declarations().to_vec();
    selected.extend(rows.into_iter().filter(|r| {
        tests.values().any(|id| {
            r.declaration_id.as_id() == *id || r.parent_id.is_some_and(|p| p.as_id() == *id)
        })
    }));
    (package.with_declarations(selected).unwrap(), tests)
}
/// An explicit route to a feasible point of the fixture's system.
fn feasible(backend: Backend) -> SolverProfile {
    let mut solver = profile(backend, false);
    solver.intent = pse_backend_native::solve::SolveIntent::FeasiblePoint;
    solver.controls.time_limit = Duration::from_secs(300);
    solver
}
/// Solve an authored fixture and return its vapor fraction and equilibrium temperature.
async fn solve(package: &ModelingPackage, case: SemanticId, solver: SolverProfile) -> (f64, f64) {
    let result = seed_prepare(package, case, solver, &CancelSource::new())
        .await
        .unwrap()
        .start()
        .unwrap()
        .wait()
        .await
        .unwrap();
    let report = authored_success(&result);
    let value = |label: &str| {
        report
            .reports
            .iter()
            .find(|r| r.label == label)
            .unwrap()
            .value
    };
    assert!(matches!(result.report(), Ok(RunReport::Modeling(_))));
    (value("vapor fraction"), value("equilibrium temperature"))
}

#[tokio::test]
async fn flash_phase_disappearance_agrees_across_realizations() {
    let owner = WorkflowRuntime::new().unwrap();
    let source = seed_package(&owner).await;
    let (package, cases) = with_complementarity_fixtures(&source);
    for (feed, beta) in [("liquid", 0.0), ("two_phase", 0.3961), ("vapor", 1.0)] {
        // smooth(math.smooth_min, 1e-4): a square system on the NLP route.
        let smooth = solve(
            &package,
            cases[&format!("complementarity_{feed}_smooth")],
            feasible(Backend::Ipopt),
        )
        .await;
        // disjunctive: slack columns and a native SOS1, on SCIP.
        let disjunctive = solve(
            &package,
            cases[&format!("complementarity_{feed}_disjunctive")],
            feasible(Backend::Scip),
        )
        .await;
        for (name, (vapor, temperature)) in [("smooth", smooth), ("disjunctive", disjunctive)] {
            assert!(
                (vapor - beta).abs() < 1e-4,
                "{feed} {name}: vapor fraction {vapor}"
            );
            assert!(temperature.is_finite(), "{feed} {name}");
        }
        // The smoothed pair leaves the equilibrium temperature within O(eps²) of the
        // exact disjunctive one.
        assert!(
            (smooth.1 - disjunctive.1).abs() < 1e-3,
            "{feed}: smooth {} K, disjunctive {} K",
            smooth.1,
            disjunctive.1
        );
        // penalty(l1) lowers to nonnegative members, a product row and the l1
        // exact-penalty requirement of its case structure. Its solve needs the l1 route
        // selection of Plan 22 M5b and is pending until then.
        let penalty = seed_prepare(
            &package,
            cases[&format!("complementarity_{feed}_penalty")],
            feasible(Backend::Ipopt),
            &CancelSource::new(),
        )
        .await
        .unwrap();
        assert_eq!(
            penalty
                .model
                .case
                .compiled()
                .plan
                .structure()
                .requirements(),
            [Requirement::L1ExactPenalty]
        );
    }
}
