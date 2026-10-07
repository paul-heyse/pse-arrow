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
    workflow::{ModelingPackage, ModelingSolvePreparation, RunReport},
};
use std::collections::BTreeMap;
use std::time::Duration;

/// Select real complementarity fixture identities from the intact loaded package.
async fn complementarity_fixtures(package: &ModelingPackage) -> BTreeMap<String, SemanticId> {
    let tests = package
        .declarations()
        .await
        .unwrap()
        .iter()
        .filter(|r| r.value.kind.as_str() == "test" && r.name.starts_with("complementarity_"))
        .map(|r| (r.name.clone(), r.declaration_id.as_id()))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(tests.len(), 9, "3 feeds × 3 realizations");
    tests
}
/// An explicit route to a feasible point of the fixture's system.
fn feasible(backend: Backend) -> SolverProfile {
    let mut solver = profile(backend, false);
    solver.intent = pse_backend_native::solve::SolveIntent::FeasiblePoint;
    solver.controls.time_limit = Duration::from_secs(300);
    solver
}
/// Solve an authored fixture and return its vapor fraction and equilibrium temperature.
async fn solve(
    package: &ModelingPackage,
    case: SemanticId,
    solver: SolverProfile,
) -> (f64, f64, f64, f64) {
    let prepared = seed_prepare(package, case, solver, &CancelSource::new())
        .await
        .unwrap();
    solve_prepared(&prepared).await
}
/// Execute the same admitted preparation whose route and closure were inspected.
async fn solve_prepared(prepared: &ModelingSolvePreparation) -> (f64, f64, f64, f64) {
    let result = prepared.start().unwrap().wait().await.unwrap();
    let report = authored_success(&result);
    assert!(matches!(result.report(), Ok(RunReport::Modeling(_))));
    let vapor = report
        .reports
        .iter()
        .find(|r| r.label == "vapor fraction")
        .unwrap();
    let registry = &report.prepared.model.case.compiled().quantities;
    let numerics = report.prepared.solve.numerics();
    let vapor_allowance = resolved_physical_allowance(
        numerics,
        registry,
        vapor,
        SemanticId::parse_hex("8a097841b11d4824b0d5041250553949").unwrap(),
    );
    let vapor = vapor.value;
    let temperature = report
        .reports
        .iter()
        .find(|r| r.label == "equilibrium temperature")
        .unwrap();
    let temperature_allowance = resolved_physical_allowance(
        numerics,
        registry,
        temperature,
        SemanticId::parse_hex("13874d4b57684720bb42a48164325812").unwrap(),
    );
    let temperature = temperature.value;
    (vapor, temperature, vapor_allowance, temperature_allowance)
}

#[tokio::test]
async fn flash_phase_disappearance_agrees_across_realizations() {
    let owner = WorkflowRuntime::new().unwrap();
    let package = seed_package(&owner).await;
    let cases = complementarity_fixtures(&package).await;
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
        for (name, (vapor, temperature, allowance, _)) in
            [("smooth", smooth), ("disjunctive", disjunctive)]
        {
            assert!(
                (vapor - beta).abs() <= allowance,
                "{feed} {name}: vapor fraction {vapor}"
            );
            assert!(temperature.is_finite(), "{feed} {name}");
        }
        // Empirical agreement uses the resolved shared temperature floor.
        // Bound feasibility budgets do not guarantee forward output accuracy.
        assert!(
            (smooth.1 - disjunctive.1).abs() <= smooth.3.min(disjunctive.3),
            "{feed}: smooth {} K, disjunctive {} K",
            smooth.1,
            disjunctive.1
        );
        // penalty(l1): nonnegative members, a product row and the l1 exact-penalty
        // requirement of its case structure, which selects POUNCE's l1 method: the
        // author's selection (ADR-0104 §5, Plan 22 M5b).
        let penalty_case = cases[&format!("complementarity_{feed}_penalty")];
        let prepared = seed_prepare(
            &package,
            penalty_case,
            feasible(Backend::Pounce),
            &CancelSource::new(),
        )
        .await
        .unwrap();
        assert_eq!(
            prepared
                .model
                .case
                .compiled()
                .plan
                .structure()
                .requirements(),
            [Requirement::L1ExactPenalty]
        );
        let penalty = solve_prepared(&prepared).await;
        assert!(
            (penalty.0 - beta).abs() <= penalty.2,
            "{feed} penalty: vapor fraction {}",
            penalty.0
        );
        // The realizations agree within the shared physical output resolution.
        assert!(
            (penalty.1 - disjunctive.1).abs() <= penalty.3.min(disjunctive.3),
            "{feed}: penalty {} K, disjunctive {} K",
            penalty.1,
            disjunctive.1
        );
        // Another backend cannot honour the requirement.
        assert!(
            seed_prepare(
                &package,
                penalty_case,
                feasible(Backend::Ipopt),
                &CancelSource::new()
            )
            .await
            .is_err()
        );
    }
}
