// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::fixtures::*;
use pse_backend_native::solve::Backend;
#[tokio::test]
async fn physical_nlp() {
    let owner = WorkflowRuntime::new().unwrap();
    let f = json("bindings.json");
    let revision = builder(&owner).await.freeze().unwrap();
    for backend in [Backend::Ipopt, Backend::Pounce, Backend::Kinsol] {
        let mut root = revision.edit();
        if backend == Backend::Kinsol {
            for c in &mut root.declaration_mut().cases {
                for v in &mut c.variables {
                    v.lower = None;
                    v.upper = None;
                }
            }
        }
        let root = root.freeze().unwrap();
        let result = solve(&root, sid(&f["root_case"]), backend, false).await;
        success(&result);
        for (name, expected, tol) in [
            ("temperature", 76.85, 1e-5),
            ("density", f["expected"]["density"].as_f64().unwrap(), 1e-5),
            ("recycle", 5., 1e-5),
        ] {
            near(
                variable(&result, sid(&f["root_ports"][name]["symbol_id"])),
                expected,
                tol,
            );
        }
        assert_eq!(
            result
                .table("authored.computation_models")
                .unwrap()
                .batch()
                .num_rows(),
            1
        );
    }
    for backend in [Backend::Ipopt, Backend::Pounce] {
        success(
            solve(&revision, sid(&f["optimization"]), backend, true)
                .await
                .as_ref(),
        );
    }
    let mut separator = revision.edit();
    for c in &mut separator.declaration_mut().cases {
        for v in &mut c.variables {
            v.upper = None;
        }
    }
    let separator = separator.freeze().unwrap();
    for case in separator
        .declaration()
        .cases
        .iter()
        .filter(|c| c.name.starts_with("separator-"))
    {
        let result = solve(&separator, case.case_id, Backend::Kinsol, false).await;
        success(&result);
        let feed = case
            .values
            .iter()
            .find(|v| v.symbol_id == case.parameters[0].symbol_id)
            .unwrap()
            .value;
        near(
            variable(&result, case.variables[0].port.symbol_id),
            0.4 * feed,
            1e-7,
        );
    }
    let flash = revision
        .declaration()
        .cases
        .iter()
        .find(|c| c.name == "flash")
        .unwrap();
    let reference = json("thermo-reference.json")["flash"].clone();
    for backend in [Backend::Ipopt, Backend::Pounce] {
        let result = solve(&revision, flash.case_id, backend, false).await;
        success(&result);
        let expected = [
            reference["liquid_density"].as_f64().unwrap(),
            reference["vapor_density"].as_f64().unwrap(),
            reference["liquid"][0].as_f64().unwrap(),
            reference["liquid"][1].as_f64().unwrap(),
            reference["vapor"][0].as_f64().unwrap(),
            reference["vapor"][1].as_f64().unwrap(),
            reference["beta"].as_f64().unwrap(),
        ];
        for (v, expected) in flash.variables.iter().zip(expected) {
            near(
                variable(&result, v.port.symbol_id),
                expected,
                2e-5 * expected.abs().max(1.),
            );
        }
    }
    // Original constrained roots cannot silently discard optimization/inequalities.
    let case = revision
        .declaration()
        .cases
        .iter()
        .find(|c| c.case_id == sid(&f["optimization"]))
        .unwrap();
    assert!(
        revision
            .prepare(
                case.case_id,
                profile(Backend::Kinsol, 2, 3, false),
                compiler(),
                false,
                &pse_runtime::CancelSource::new()
            )
            .await
            .is_err()
    );
    // Structural matching alone cannot certify a singular numerical system.
    let mut singular = revision.edit();
    let source = singular.declaration_mut();
    let first = source.cases[0].instances[0].definition_id;
    let duplicate = source.cases[0].instances[1].definition_id;
    let definition = source
        .definitions
        .iter()
        .find(|d| d.definition_id == first)
        .unwrap()
        .clone();
    let target = source
        .definitions
        .iter_mut()
        .find(|d| d.definition_id == duplicate)
        .unwrap();
    target.sources = definition.sources;
    target.providers = definition.providers;
    // Admission must reject the incompatible physical row before a solver runs.
    let changed = singular.freeze().unwrap();
    assert!(
        changed
            .prepare(
                sid(&f["root_case"]),
                profile(Backend::Kinsol, 3, 3, false),
                compiler(),
                false,
                &pse_runtime::CancelSource::new()
            )
            .await
            .is_err()
    );
}
