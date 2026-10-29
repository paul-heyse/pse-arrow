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

#[path = "reference_support/mod.rs"]
mod reference_support;
#[path = "reference_sensitivity_support/mod.rs"]
mod support;

use futures_util::FutureExt;
use reference_support::{Result, root};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    panic::AssertUnwindSafe,
    path::PathBuf,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
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

fn pins() -> Result<BTreeMap<String, String>> {
    // Reference versions have one owner: existing dependency declarations.
    let project: toml::Value =
        toml::from_str(&std::fs::read_to_string(root().join("pyproject.toml"))?)?;
    let group = project["dependency-groups"]["parity"].as_array().unwrap();
    ["idaes-pse", "pyomo"]
        .into_iter()
        .map(|name| {
            let prefix = format!("{name}==");
            let version = group
                .iter()
                .filter_map(toml::Value::as_str)
                .find_map(|entry| entry.strip_prefix(&prefix))
                .unwrap()
                .split(';')
                .next()
                .unwrap()
                .trim();
            Ok((name.to_owned(), version.to_owned()))
        })
        .collect()
}

fn exchange(request: &Request) -> Result<Response> {
    let python = PathBuf::from(std::env::var_os("PSE_SENSITIVITY_REFERENCE_PYTHON")
        .expect("select an existing locked reference interpreter with PSE_SENSITIVITY_REFERENCE_PYTHON; absence is failure"));
    assert!(
        python.is_absolute() && python.is_file(),
        "explicit reference interpreter must be an absolute existing file"
    );
    let scratch = tempfile::tempdir()?;
    let input = scratch.path().join("request.json");
    let output = scratch.path().join("response.json");
    let log = scratch.path().join("oracle.log");
    std::fs::write(&input, serde_json::to_vec(request)?)?;
    let diagnostics = std::fs::File::create(&log)?;
    let mut command = Command::new(python);
    command
        .arg("-I")
        .arg(root().join("tests/reference/oracles/sensitivity.py"))
        .arg(&input)
        .arg(&output)
        .current_dir(scratch.path())
        .env_remove("PYTHONPATH")
        .env_remove("PYTHONHOME")
        .stdin(Stdio::null())
        .stdout(Stdio::from(diagnostics.try_clone()?))
        .stderr(Stdio::from(diagnostics));
    let mut child = pse_diagnostics::SupervisedChild::spawn(
        &mut command,
        pse_diagnostics::DrainFailurePolicy::Refuse(Duration::from_secs(30)),
        None,
    )?;
    let deadline = Instant::now() + Duration::from_secs(240);
    if let Err(error) = child.wait_ready(None, Some(deadline)) {
        let drained = child.cancel();
        panic!(
            "reference execution failed: {error}; drain={drained:?}; oracle output: {}",
            std::fs::read_to_string(&log)?
        );
    }
    let status = child.wait()?;
    assert!(
        status.success(),
        "reference environment/execution failed ({status}); {}",
        std::fs::read_to_string(&log)?
    );
    let bytes =
        std::fs::read(&output).expect("successful oracle must publish its explicit response");
    let response: Response = serde_json::from_slice(&bytes)?;
    assert_eq!(response.version, 1);
    assert_eq!(response.case_id, request.case.id);
    for (name, version) in &request.pins {
        assert_eq!(&response.environment[name], version);
    }
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
        x.increment += 10.0 * x.allowance;
        x.predicted = x.base + x.increment;
        let mut wrong_base = report.clone();
        let y = wrong_base.quantities.get_mut("y").unwrap();
        y.base += 10.0 * y.allowance;
        y.predicted = y.base + y.increment;
        let mut wrong_prediction = report.clone();
        let z = wrong_prediction.quantities.get_mut("z").unwrap();
        z.predicted += 10.0 * z.allowance;
        let request = Request {
            version: 1,
            case,
            pins: pins()?,
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
                record("native-after-failures", Some(report)),
            ],
        };
        let response = tokio::task::spawn_blocking(move || exchange(&request)).await??;
        let expected = [
            ("native", "passed", ""),
            ("wrong-response", "failed", "normalized derivative differs"),
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
