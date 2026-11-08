// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native weighted fit and covariance against isolated Pyomo parmest.
//! Register with required-features = ["solver-ipopt"]; correctness also needs force-validate.
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
#[path = "reference_covariance_support/mod.rs"]
mod support;

use futures_util::FutureExt;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, panic::AssertUnwindSafe, process::Command};
use support::{Candidate, CandidateReport, Covariance, LineCase, Result};

#[derive(Serialize)]
struct Request {
    version: u32,
    case: LineCase,
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
struct Reference {
    estimates: BTreeMap<String, f64>,
    covariance: Covariance,
    inverse_information: Vec<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Response {
    version: u32,
    case_id: String,
    environment: BTreeMap<String, String>,
    reference: Reference,
    comparisons: Vec<Comparison>,
}

fn exchange(request: &Request) -> Result<Response> {
    let python = reference_environment::interpreter("PSE_COVARIANCE_REFERENCE_PYTHON");
    let interpreter_digest = reference_environment::image_digest(&python)?;
    let solver = reference_environment::prepared("IPOPT_DIR", "ipopt")?;
    let scratch = tempfile::tempdir()?;
    let input = scratch.path().join("request.json");
    let output = scratch.path().join("response.json");
    let limit = reference_process::legacy_response_limit(request.candidates.len())?;
    std::fs::write(&input, serde_json::to_vec(request)?)?;
    let mut command = Command::new(&python);
    command
        .arg("-I")
        .arg(support::root().join("tests/reference/legacy_reference_driver.py"))
        .arg("covariance")
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
    reference_environment::verify_stage(&response.environment, "parmest.fit", &solver, 1..=1)
        .map_err(|error| execution.failure("solver evidence refused", error))?;
    reference_environment::verify_stage(
        &response.environment,
        "parmest.covariance",
        &solver,
        30..=30,
    )
    .map_err(|error| execution.failure("solver evidence refused", error))?;
    assert_eq!(
        response
            .reference
            .estimates
            .keys()
            .cloned()
            .collect::<Vec<_>>(),
        request.case.parameters
    );
    assert!(
        response
            .reference
            .estimates
            .values()
            .all(|value| value.is_finite())
    );
    assert_eq!(
        response.reference.covariance.parameters,
        request.case.parameters
    );
    assert_eq!(response.reference.covariance.values.len(), 4);
    assert_eq!(response.reference.inverse_information.len(), 4);
    Ok(response)
}

fn record(id: &str, report: Option<CandidateReport>) -> Candidate {
    Candidate {
        id: id.into(),
        report,
    }
}

#[tokio::test]
async fn native_covariance_matches_parmest_and_rejects_bad_populations() -> Result<()> {
    let native = reference_support::NativeContext::new()?;
    let outcome = AssertUnwindSafe(async {
        let case = LineCase::weighted();
        let report = support::run(&native, &case).await?;
        let mut wrong_estimate = report.clone();
        *wrong_estimate.estimates.get_mut("a").unwrap() += 0.01;
        wrong_estimate.estimate_allowances.insert("a".into(), 1.0);
        let mut invalid_engineering = report.clone();
        invalid_engineering.estimate_allowances.remove("a");
        let mut wrong_covariance = report.clone();
        wrong_covariance.covariance.values[0] *= 1.25;
        let mut incomplete = report.clone();
        incomplete.complete = false;
        let mut missing_cell = report.clone();
        missing_cell.covariance.values.pop();
        let mut wrong_order = report.clone();
        wrong_order.covariance.parameters.swap(0, 1);
        let mut reordered = wrong_order.clone();
        // A valid physical reordering permutes BOTH matrix axes and metadata.
        reordered.covariance.values = [3, 2, 1, 0]
            .map(|index| report.covariance.values[index])
            .to_vec();
        reordered.covariance.parameter_units.reverse();
        reordered.covariance.parameter_scales.reverse();
        let mut wrong_units = report.clone();
        wrong_units.response_unit = "Pa".into();
        let mut wrong_convention = report.clone();
        wrong_convention.covariance.convention = "residual_variance_scaled".into();
        let request = Request {
            version: 1,
            case,
            pins: reference_environment::pins(&reference_support::root())?,
            candidates: vec![
                record("native", Some(report.clone())),
                record("wrong-estimate", Some(wrong_estimate)),
                record("missing-engineering-budget", Some(invalid_engineering)),
                record("wrong-covariance", Some(wrong_covariance)),
                record("missing", None),
                record("incomplete", Some(incomplete)),
                record("missing-covariance-cell", Some(missing_cell)),
                record("wrong-parameter-order", Some(wrong_order)),
                record("physical-reordering", Some(reordered)),
                record("wrong-response-unit", Some(wrong_units)),
                record("wrong-convention", Some(wrong_convention)),
                record("native-after-failures", Some(report)),
            ],
        };
        let response = tokio::task::spawn_blocking(move || exchange(&request)).await??;
        let expected = [
            ("native", "passed", ""),
            ("wrong-estimate", "failed", "estimate differs"),
            (
                "missing-engineering-budget",
                "failed",
                "native engineering estimate budgets",
            ),
            ("wrong-covariance", "failed", "covariance differs"),
            ("missing", "missing", "native result missing"),
            ("incomplete", "failed", "native fit incomplete"),
            (
                "missing-covariance-cell",
                "failed",
                "covariance is incomplete",
            ),
            ("wrong-parameter-order", "failed", "covariance differs"),
            ("physical-reordering", "passed", ""),
            ("wrong-response-unit", "failed", "response unit differs"),
            (
                "wrong-convention",
                "failed",
                "covariance convention differs",
            ),
            ("native-after-failures", "passed", ""),
        ];
        assert_eq!(
            response.comparisons.len(),
            expected.len(),
            "missing comparisons cannot disappear"
        );
        for (actual, (id, status, reason)) in response.comparisons.iter().zip(expected) {
            assert_eq!(actual.id, id, "retain input order through mixed outcomes");
            assert_eq!(actual.status, status, "{actual:?}");
            assert_eq!(actual.reasons.is_empty(), status == "passed", "{actual:?}");
            if status != "passed" {
                assert!(
                    actual.reasons.iter().any(|item| item.contains(reason)),
                    "{actual:?}"
                );
            }
        }
        Ok(())
    })
    .catch_unwind()
    .await;
    // Close native lanes and release physical resources on assertions and errors alike.
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
