// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Controls for the named benchmark workload and Criterion's mode boundary.
#[allow(
    dead_code,
    reason = "the selection controls exercise admission without starting fixture processes"
)]
#[path = "../benches/admission/mod.rs"]
mod admission;

use pse_benches::CriterionMode;
use serde_json::Value;

#[test]
fn criterion_smoke_mode_wins_over_cargos_benchmark_flag() {
    assert_eq!(
        CriterionMode::from_args(["--bench", "--test"]),
        CriterionMode::Smoke
    );
    assert_eq!(
        CriterionMode::from_args(["--test", "--bench"]),
        CriterionMode::Smoke
    );
    assert_eq!(CriterionMode::from_args(["--test"]), CriterionMode::Smoke);
    assert_eq!(
        CriterionMode::from_args(["--bench"]),
        CriterionMode::Measure
    );
    assert_eq!(
        CriterionMode::from_args(["--bench", "--list"]),
        CriterionMode::List
    );
    assert_eq!(
        CriterionMode::from_args(["--bench", "--profile-time=2"]),
        CriterionMode::Profile
    );
}

#[test]
fn benchmark_selection_distinguishes_local_fixture_and_linked_solver_inputs() {
    let preparation: Value =
        serde_json::from_str(include_str!("../../.config/preparation-cases.json")).unwrap();
    let preparation = preparation["workloads"].as_array().unwrap();
    let named = |id: &str| preparation.iter().find(|case| case["id"] == id).unwrap();
    assert!(admission::preparation_inputs(named("k4-demand-orders-selected-output")).local());
    let canonical = admission::preparation_inputs(named("pcsaft-02"));
    assert!(canonical.canonical);
    assert_eq!(canonical.solver_feature, None);
    let both = admission::preparation_inputs(named("pr-nested"));
    assert!(both.canonical);
    assert_eq!(both.solver_feature, Some("solver-kinsol"));

    let process: Value =
        serde_json::from_str(include_str!("../../.config/process-cases.json")).unwrap();
    let process = process["workloads"].as_array().unwrap();
    let named = |id: &str| process.iter().find(|case| case["id"] == id).unwrap();
    let solver = admission::process_inputs(named("compact-root-diagonal-4"));
    assert!(!solver.canonical);
    assert_eq!(solver.solver_feature, Some("native-process"));
    assert_eq!(solver.authored, Some(admission::AuthoredInput::Physical));
    let seeded = admission::process_inputs(named("cold-small-1"));
    assert!(seeded.canonical);
    assert_eq!(
        seeded.authored,
        Some(admission::AuthoredInput::SeedTransport)
    );
    let managed = named("k4-study-managed-durable-16");
    assert!(admission::process_inputs(managed).unavailable.is_some());
    let record = admission::selection_record(
        "native_process",
        CriterionMode::Smoke,
        &[managed],
        process,
        admission::process_inputs,
    );
    assert!(
        record["unavailable"]
            .as_array()
            .unwrap()
            .iter()
            .any(|case| case["id"] == managed["id"])
    );
}
