// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native original-unit KKT sensitivity challenged by an isolated sIPOPT oracle.
//! Register this target with required-features = ["solver-ipopt"]. Reference absence fails.
#![recursion_limit = "512"]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "reference setup and independent scientific assertions fail directly"
)]

#[path = "reference_support/environment.rs"]
mod reference_environment;
#[path = "reference_support/process.rs"]
mod reference_process;
#[path = "reference_support/mod.rs"]
mod reference_support;
#[path = "reference_sensitivity_support/mod.rs"]
mod support;

use futures_util::FutureExt;
use reference_support::{Result, root};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, panic::AssertUnwindSafe, process::Command};
use support::{Candidate, CandidateReport, NativeCandidate, SensitivityCase};

#[derive(Serialize)]
struct Request {
    version: u32,
    case: SensitivityCase,
    pins: BTreeMap<String, String>,
    candidates: Vec<Candidate>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Comparison {
    id: String,
    status: String,
    reasons: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Response {
    version: u32,
    case_id: String,
    environment: BTreeMap<String, String>,
    reference: BTreeMap<String, f64>,
    comparisons: Vec<Comparison>,
}

fn exchange(request: &Request) -> Result<Response> {
    let python = reference_environment::interpreter("PSE_SENSITIVITY_REFERENCE_PYTHON");
    let interpreter_digest = reference_environment::image_digest(&python)?;
    let solver = reference_environment::prepared("IPOPT_DIR", "ipopt_sens")?;
    let scratch = tempfile::tempdir()?;
    let input = scratch.path().join("request.json");
    let output = scratch.path().join("response.json");
    let limit = reference_process::legacy_response_limit(request.candidates.len())?;
    std::fs::write(&input, serde_json::to_vec(request)?)?;
    let mut command = Command::new(&python);
    command
        .arg("-I")
        .arg(root().join("tests/reference/legacy_reference_driver.py"))
        .arg("sensitivity")
        .arg(&input)
        .arg(&output)
        .current_dir(scratch.path())
        .env_remove("PYTHONPATH")
        .env_remove("PYTHONHOME");
    let execution =
        reference_process::execute(&mut command, scratch, reference_process::DIAGNOSTIC_BYTES);
    if let Some((phase, cause)) = &execution.refusal {
        return Err(format!("reference {phase}: {cause}\n{}", execution.diagnostics).into());
    }
    let response: Response = execution.decode_legacy(&output, limit)?;
    assert_eq!(response.version, 1);
    assert_eq!(response.case_id, request.case.id);
    reference_environment::verify(
        &response.environment,
        &python,
        &interpreter_digest,
        &request.pins,
    )
    .map_err(|error| execution.failure("environment refused", error))?;
    reference_environment::verify_stage(
        &response.environment,
        "sensitivity.sipopt",
        &solver,
        1..=1,
    )
    .map_err(|error| execution.failure("solver evidence refused", error))?;
    assert_eq!(
        response
            .reference
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["x", "y", "z"]
    );
    assert!(response.reference.values().all(|value| value.is_finite()));
    Ok(response)
}

fn record(id: &str, report: Option<CandidateReport>) -> Candidate {
    Candidate {
        id: id.into(),
        report,
    }
}

#[tokio::test]
async fn native_sensitivity_matches_sipopt_and_rejects_bad_populations() -> Result<()> {
    let native = NativeCandidate::new()?;
    let outcome = AssertUnwindSafe(async {
        let case = SensitivityCase::quadratic();
        let report = native.run(&case).await?;
        let mut wrong = report.clone();
        let x = wrong.quantities.get_mut("x").unwrap();
        // A coherent but wrong physical Jacobian must fail even if prediction=base+increment.
        *x.derivative.get_mut("a").unwrap() += 1.0;
        x.allowance = 10.0;
        wrong.parameter_scales.insert("a".into(), 0.01);
        x.increment = x
            .derivative
            .iter()
            .map(|(p, derivative)| derivative * (case.perturbed[p] - case.nominal[p]))
            .sum();
        x.predicted = x.base + x.increment;
        let mut incomplete = report.clone();
        incomplete.complete = false;
        let mut missing_target = report.clone();
        missing_target.quantities.remove("z");
        let mut missing_parameter = report.clone();
        missing_parameter
            .quantities
            .get_mut("y")
            .unwrap()
            .derivative
            .remove("b");
        let mut wrong_increment = report.clone();
        let x = wrong_increment.quantities.get_mut("x").unwrap();
        x.increment += 0.01;
        x.allowance = 1.0;
        x.predicted = x.base + x.increment;
        let mut wrong_base = report.clone();
        let y = wrong_base.quantities.get_mut("y").unwrap();
        y.base += 0.01;
        y.allowance = 1.0;
        y.predicted = y.base + y.increment;
        let mut wrong_prediction = report.clone();
        let z = wrong_prediction.quantities.get_mut("z").unwrap();
        z.predicted += 0.01;
        z.allowance = 1.0;
        let mut wrong_unit = report.clone();
        wrong_unit.unit = "Pa".into();
        let mut wrong_coordinates = report.clone();
        wrong_coordinates.coordinates = "normalized".into();
        let request = Request {
            version: 1,
            case,
            pins: reference_environment::pins(&reference_support::root())?,
            candidates: vec![
                record("native", Some(report.clone())),
                record("wrong-response", Some(wrong)),
                record("missing", None),
                record("incomplete", Some(incomplete)),
                record("missing-target", Some(missing_target)),
                record("missing-parameter", Some(missing_parameter)),
                record("wrong-increment", Some(wrong_increment)),
                record("wrong-base", Some(wrong_base)),
                record("wrong-prediction", Some(wrong_prediction)),
                record("wrong-unit", Some(wrong_unit)),
                record("wrong-coordinates", Some(wrong_coordinates)),
                record("native-after-failures", Some(report)),
            ],
        };
        let response = tokio::task::spawn_blocking(move || exchange(&request)).await??;
        let expected = [
            ("native", "passed", ""),
            ("wrong-response", "failed", "physical derivative differs"),
            ("missing", "missing", "native result missing"),
            ("incomplete", "failed", "native response incomplete"),
            (
                "missing-target",
                "failed",
                "native target population incomplete",
            ),
            (
                "missing-parameter",
                "failed",
                "native derivative population incomplete",
            ),
            ("wrong-increment", "failed", "physical increment differs"),
            ("wrong-base", "failed", "nominal solution differs"),
            ("wrong-prediction", "failed", "predicted solution differs"),
            ("wrong-unit", "failed", "native sensitivity unit differs"),
            (
                "wrong-coordinates",
                "failed",
                "native sensitivity coordinates differ",
            ),
            ("native-after-failures", "passed", ""),
        ];
        assert_eq!(
            response.comparisons.len(),
            expected.len(),
            "missing comparisons cannot disappear"
        );
        for (actual, (id, status, reason)) in response.comparisons.iter().zip(expected) {
            assert_eq!(
                actual.id, id,
                "input identity/order must survive mixed results"
            );
            assert_eq!(actual.status, status, "{actual:?}");
            assert_eq!(actual.reasons.is_empty(), status == "passed", "{actual:?}");
            if status != "passed" {
                assert!(
                    actual.reasons.iter().any(|text| text.contains(reason)),
                    "{actual:?}"
                );
            }
        }
        Ok(())
    })
    .catch_unwind()
    .await;
    // Always settle local runtime/native lanes, including failed scientific assertions.
    let closed = native.close().await;
    match outcome {
        Ok(result) => {
            closed?;
            result
        }
        Err(panic) => {
            let _ = closed;
            std::panic::resume_unwind(panic)
        }
    }
}
