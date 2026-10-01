// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Simultaneous dynamic optimization of authored fixtures (Plan 22 Y5a): the seed
//! `SaturatedProcess` and `SwitchedProcess` definitions of `control_fixtures`, a first-order
//! lag `lag·dx/dt = u − x`, x(0) = 0, over 2 s, tracking a target with a bounded input. The
//! same `SaturatedProcess` definition also integrates with its input scheduled.
use super::fixtures::*;
use pse_backend_native::solve::{Backend, Qualification, SolverSelection};
use pse_ids::SemanticId;
use pse_runtime::{
    CancelSource,
    math::solves::{Outcome, SolverProfile},
    workflow::{ModelingPackage, ModelingResult, RunResult},
};

/// `test saturated_simultaneous`, `saturated_integrated` and `switched_simultaneous`.
const SATURATED: &str = "f33e2cf83c7e4f1b8df6da7f66212bd5";
const INTEGRATED: &str = "50b334c432114f34ade0f9e24ff460e5";
const SWITCHED: &str = "bbf77b04ed344205a780917531a693f4";
/// ∫₀² (x − 1)² dt at the saturated input u = 1, x = 1 − e^{−t}.
fn saturated_tracking() -> f64 {
    (1.0 - (-4.0f64).exp()) / 2.0
}
/// ∫ (x − 0.5)² dt at the optimal input: u = 1 until x = 0.5 at t = ln 2, then u = 0.5.
fn half_tracking() -> f64 {
    std::f64::consts::LN_2 / 4.0 - 0.125
}
/// ∫₀² (x − 0.5)² dt at the always-on input.
fn half_always_on() -> f64 {
    saturated_tracking() - (1.0 - (-2.0f64).exp()) + 0.5
}
/// The seed package with the dynamic-optimization tests of `control_fixtures` restored.
async fn package(owner: &WorkflowRuntime) -> ModelingPackage {
    let package = seed_package(owner).await;
    let path = fixture("")
        .ancestors()
        .find(|p| p.join("packages/reference").is_dir())
        .unwrap()
        .join("packages/reference/seed-data/models/control-fixtures.pse");
    let rows = pse_authoring::language::parse(
        &std::fs::read_to_string(path).unwrap(),
        SemanticId::NIL,
        pse_authoring::language::IdentityPolicy::Explicit,
        Default::default(),
    )
    .unwrap();
    let tests = [SATURATED, INTEGRATED, SWITCHED].map(|id| SemanticId::parse_hex(id).unwrap());
    let mut selected = package.declarations().to_vec();
    selected.extend(rows.into_iter().filter(|r| {
        tests.iter().any(|id| {
            r.declaration_id.as_id() == *id || r.parent_id.is_some_and(|p| p.as_id() == *id)
        })
    }));
    package.with_declarations(selected).unwrap()
}
fn optimize(selection: SolverSelection) -> SolverProfile {
    let mut solver = profile(Backend::Ipopt, true);
    solver.selection = selection;
    solver.controls.time_limit = std::time::Duration::from_secs(300);
    solver
}
/// The holdup at the element boundaries 0.5 s and 1 s and at the horizon.
const HOLDUPS: [(f64, &str); 3] = [
    (0.5, "root.holdup[0.5{s}]"),
    (1.0, "root.holdup[1{s}]"),
    (2.0, "root.holdup[2{s}]"),
];
/// Solve an authored simultaneous fixture, with `target` replacing the authored target;
/// returns the run and the identities of the demanded `paths`.
async fn simultaneous(
    package: &ModelingPackage,
    fixture: &str,
    solver: SolverProfile,
    target: Option<f64>,
    paths: &[&str],
) -> (std::sync::Arc<RunResult>, Vec<SemanticId>) {
    let cancel = CancelSource::new();
    let mut analysis = package
        .declared_execution(
            SemanticId::parse_hex(fixture).unwrap().into(),
            compiler(),
            solver,
            Default::default(),
            seed_limits(),
            &cancel,
        )
        .await
        .unwrap()
        .analysis;
    if let Some(target) = target {
        analysis.case.values.insert("root.target".into(), target);
    }
    analysis
        .bindings
        .demand
        .extend(paths.iter().map(|p| (*p).to_owned()));
    let prepared = package.prepare_analysis(&analysis, &cancel).await.unwrap();
    let known = &prepared.model.model.compiled().model.paths;
    let ids = paths
        .iter()
        .map(|p| *known.get(*p).unwrap_or_else(|| panic!("{p}")))
        .collect();
    (prepared.start().unwrap().wait().await.unwrap(), ids)
}
fn tracking(result: &ModelingResult) -> f64 {
    result
        .reports
        .iter()
        .find(|r| r.label == "tracking")
        .unwrap()
        .value
}
/// Every solved value of the indexed member `name`, one per mesh coordinate.
fn values(result: &ModelingResult, name: &str) -> Vec<(SemanticId, f64)> {
    let model = &result.prepared.model.model.compiled().model;
    let suffix = format!("root.{name}");
    let found = result
        .values
        .scalars
        .iter()
        .filter(|(id, _)| {
            model
                .symbols
                .get(id)
                .is_some_and(|s| s.lineage.path.ends_with(&suffix))
        })
        .map(|(id, v)| (*id, *v))
        .collect::<Vec<_>>();
    assert!(!found.is_empty(), "no member {name}");
    found
}
/// Free variables less equations, as the fixture declares them.
fn degrees_of_freedom(result: &ModelingResult) -> usize {
    let facts = &result.prepared.model.case.compiled().facts;
    facts.variables - facts.rows
}
fn native(result: &ModelingResult) -> &pse_backend_native::solve::SolveReport {
    match &result.outcome {
        Outcome::Native(native) => native,
        other => panic!("{other:?}"),
    }
}

/// A bounded input tracking a target: saturated at its bound for target 1, with the
/// analytic x = 1 − e^{−t}; switching at t = ln 2 to hold x = 0.5 for target 0.5. The
/// input is saturated at its bound over [0, ln 2) on both. The same definition, integrated
/// with its input scheduled at the saturated optimum, reproduces the trajectory.
#[tokio::test]
async fn simultaneous_dynamic_optimization_matches_analytic() {
    let owner = WorkflowRuntime::new().unwrap();
    let package = package(&owner).await;
    let paths = HOLDUPS.map(|(_, p)| p);
    let (saturated, holdups) = simultaneous(
        &package,
        SATURATED,
        optimize(SolverSelection::Auto),
        None,
        &paths,
    )
    .await;
    let result = authored_success(&saturated);
    // A linear lag with a quadratic tracking integral is a convex QP; the fixture's
    // declared degrees of freedom are its free inputs.
    assert_eq!(native(result).backend, Backend::Highs);
    assert_eq!(degrees_of_freedom(result), 24);
    near(tracking(result), saturated_tracking(), 1e-6);
    let inputs = values(result, "u");
    // One input per mesh node, the fixed left boundary included.
    assert_eq!(inputs.len(), 25);
    for (_, u) in &inputs {
        near(*u, 1.0, 1e-5);
    }
    let simultaneous_states = [0, 1, 2].map(|k| result.values.scalars[&holdups[k]]);
    for ((t, _), x) in HOLDUPS.into_iter().zip(simultaneous_states) {
        near(x, 1.0 - (-t).exp(), 1e-6);
    }
    // Tracking half the reachable range switches off the saturated input at x = 0.5.
    let (half, holdups) = simultaneous(
        &package,
        SATURATED,
        optimize(SolverSelection::Auto),
        Some(0.5),
        &paths,
    )
    .await;
    let half = authored_success(&half);
    near(tracking(half), half_tracking(), 1e-3);
    near(half.values.scalars[&holdups[2]], 0.5, 1e-3);
    let inputs = values(half, "u");
    assert!(
        inputs.iter().any(|(_, u)| (u - 0.5).abs() < 1e-3),
        "{inputs:?}"
    );
    assert!(
        inputs.iter().any(|(_, u)| (u - 1.0).abs() < 1e-3),
        "{inputs:?}"
    );

    // Integrated, with the input scheduled in two intervals at its saturated optimum.
    let cancel = CancelSource::new();
    let integrated = SemanticId::parse_hex(INTEGRATED).unwrap().into();
    let declared = package
        .declared_simulation(integrated, compiler(), None, seed_limits(), &cancel)
        .await
        .unwrap();
    let contract = declared.contract();
    let symbols = &declared.model().compiled().model.symbols;
    let input = contract
        .parameters
        .iter()
        .position(|id| symbols[id].lineage.path.contains(".u"))
        .unwrap();
    let mut schedule = declared.profile().clone();
    schedule
        .schedule
        .push(pse_backend_native::dynamics::ScheduledInput {
            parameter: input,
            times: vec![std::f64::consts::LN_2],
        });
    let simulation = package
        .declared_simulation(
            integrated,
            compiler(),
            Some(schedule),
            seed_limits(),
            &cancel,
        )
        .await
        .unwrap();
    let trajectory = simulation.run(&cancel).await.unwrap();
    assert_eq!(
        trajectory.report.termination,
        pse_backend_native::dynamics::Termination::Completed,
        "{:?}",
        trajectory.report.error
    );
    let state = simulation.contract().states[0];
    let output = simulation
        .contract()
        .outputs
        .iter()
        .position(|id| *id == state);
    for (sample, x) in trajectory.report.samples.iter().zip(simultaneous_states) {
        let integrated = output.map_or(sample.state[0], |o| sample.outputs[o]);
        near(integrated, 1.0 - (-sample.time).exp(), 1e-7);
        near(integrated, x, 1e-6);
    }
    near(
        trajectory.report.samples.last().unwrap().integrals[0],
        saturated_tracking(),
        1e-8,
    );
}

/// An on/off input at every collocation node makes the problem a mixed-integer program
/// that SCIP certifies. Its candidate is the continuous re-solve under the node
/// assignment, which the step states as its commitment (Plan 22 M2c); its tracking lies
/// between the continuous optimum and the always-on input's.
#[tokio::test]
async fn simultaneous_dynamic_optimization_with_discrete_decision() {
    let owner = WorkflowRuntime::new().unwrap();
    let package = package(&owner).await;
    let (run, _) = simultaneous(
        &package,
        SWITCHED,
        optimize(SolverSelection::Auto),
        None,
        &[],
    )
    .await;
    let result = authored_success(&run);
    assert_eq!(degrees_of_freedom(result), 8);
    let native = native(result);
    assert_eq!(native.backend, Backend::Scip);
    assert!(
        matches!(
            native.qualification,
            Qualification::GapQualified | Qualification::OptimalWithinTolerance
        ),
        "{:?}",
        native.qualification
    );
    let value = tracking(result);
    assert!(
        value >= half_tracking() - 5e-3 && value <= half_always_on() + 5e-3,
        "{value}"
    );
    // The switched inputs are the committed assignment: binary, and both on and off.
    let commitment = native
        .candidate
        .as_ref()
        .unwrap()
        .commitment
        .as_ref()
        .expect("a mixed-integer candidate states its commitment");
    assert_eq!(commitment.columns.len(), 8);
    assert!(
        commitment
            .columns
            .iter()
            .all(|(_, (l, u))| l == u && (*l == 0.0 || *l == 1.0))
    );
    assert!(commitment.columns.iter().any(|(_, (v, _))| *v == 0.0));
    assert!(commitment.columns.iter().any(|(_, (v, _))| *v == 1.0));
    let on = values(result, "on");
    // Every switch but the one fixed at the start is committed at its solved value.
    assert_eq!(on.len(), 9);
    for (id, (committed, _)) in &commitment.columns {
        let solved = on.iter().find(|(o, _)| o == id).unwrap().1;
        near(solved, *committed, 1e-9);
    }
    let runs = pse_relations::generated::runtime::solve_runs::View::from_checked(
        &run.table("runtime.solve_runs").unwrap(),
    )
    .unwrap()
    .rows()
    .unwrap();
    assert_eq!(runs[0].commitment.as_ref().map(Vec::len), Some(8));
}
