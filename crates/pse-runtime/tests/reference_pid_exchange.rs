// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public native authored PI dynamics versus isolated IDAES/PETSc and exact response.
//! Required features: solver-diffsol, force-validate. Reference absence fails.
#![recursion_limit = "512"]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "reference setup and independent scientific assertions fail directly"
)]

#[path = "reference_support/context.rs"]
mod reference_support;
#[path = "../../../tests/fixtures/scientific-inputs.rs"]
mod scientific_inputs;
#[path = "reference_pid_support/mod.rs"]
mod support;

use futures_util::FutureExt;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    panic::AssertUnwindSafe,
    path::PathBuf,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
use support::{Candidate, CandidateReport, LoopCase, Result};

#[derive(Serialize)]
struct Request {
    version: u32,
    case: LoopCase,
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
    samples_seconds: Vec<f64>,
    exact_process: Vec<f64>,
    petsc_process: Vec<f64>,
    steady_process: f64,
    steady_integral_seconds: f64,
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

fn pins() -> Result<BTreeMap<String, String>> {
    let project: toml::Value = toml::from_str(&std::fs::read_to_string(
        support::root().join("pyproject.toml"),
    )?)?;
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
    let python = PathBuf::from(std::env::var_os("PSE_PID_REFERENCE_PYTHON").expect(
        "select the locked reference interpreter with PSE_PID_REFERENCE_PYTHON; absence fails",
    ));
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
        .arg(support::root().join("tests/reference/oracles/pid.py"))
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
    if let Err(error) = child.wait_ready(None, Some(Instant::now() + Duration::from_secs(240))) {
        let drained = child.cancel();
        panic!(
            "PID reference failed: {error}; drain={drained:?}; {}",
            std::fs::read_to_string(&log)?
        );
    }
    let status = child.wait()?;
    assert!(
        status.success(),
        "PID reference environment/execution failed ({status}); {}",
        std::fs::read_to_string(&log)?
    );
    let response: Response = serde_json::from_slice(&std::fs::read(&output)?)?;
    assert_eq!(response.version, 1);
    assert_eq!(response.case_id, request.case.id);
    for (name, pin) in &request.pins {
        assert_eq!(&response.environment[name], pin);
    }
    assert!(response.environment.contains_key("python"));
    assert_eq!(
        response.reference.samples_seconds,
        request.case.samples_seconds
    );
    assert_eq!(
        response.reference.exact_process.len(),
        request.case.samples_seconds.len()
    );
    assert_eq!(
        response.reference.petsc_process.len(),
        request.case.samples_seconds.len()
    );
    assert!(
        (response.reference.steady_process - request.case.setpoint).abs()
            <= request.case.engineering_fraction
    );
    let steady_integral =
        request.case.ti_seconds * (request.case.setpoint - request.case.bias) / request.case.kp;
    assert!(
        (response.reference.steady_integral_seconds - steady_integral).abs()
            <= request.case.engineering_fraction
    );
    for (exact, petsc) in response
        .reference
        .exact_process
        .iter()
        .zip(&response.reference.petsc_process)
    {
        assert!(exact.is_finite() && petsc.is_finite());
        assert!((exact - petsc).abs() <= request.case.engineering_fraction);
    }
    Ok(response)
}

fn candidate(id: &str, report: Option<CandidateReport>) -> Candidate {
    Candidate {
        id: id.into(),
        report,
    }
}

#[tokio::test]
async fn native_pi_loop_matches_petsc_and_exact_response_and_rejects_bad_populations() -> Result<()>
{
    let context = reference_support::NativeContext::new()?;
    let outcome = AssertUnwindSafe(async {
        let case = LoopCase::original();
        let report = Box::pin(support::run(&context, &case)).await?;
        assert!(report.complete);
        let mut wrong = report.clone();
        wrong.process[3] += 0.1;
        wrong.measurement[3] += 0.1;
        let mut measured = report.clone();
        measured.measurement[2] += 0.1;
        let mut incomplete = report.clone();
        incomplete.complete = false;
        let mut missing_sample = report.clone();
        missing_sample.process.pop();
        let mut wrong_time = report.clone();
        wrong_time.samples_seconds[3] += 0.25;
        let mut unaccepted = report.clone();
        unaccepted.accepted = false;
        let mut fixture_failed = report.clone();
        fixture_failed.fixture_passed = false;
        let mut stopped = report.clone();
        stopped.termination = "failed".into();
        let request = Request {
            version: 1,
            pins: pins()?,
            case,
            candidates: vec![
                candidate("native", Some(report.clone())),
                candidate("wrong-trajectory", Some(wrong)),
                candidate("measurement-mismatch", Some(measured)),
                candidate("missing", None),
                candidate("incomplete", Some(incomplete)),
                candidate("missing-sample", Some(missing_sample)),
                candidate("wrong-time", Some(wrong_time)),
                candidate("unaccepted", Some(unaccepted)),
                candidate("failed-fixture", Some(fixture_failed)),
                candidate("failed-integration", Some(stopped)),
                candidate("native-after-failures", Some(report)),
            ],
        };
        let response = tokio::task::spawn_blocking(move || exchange(&request)).await??;
        let expected = [
            ("native", "passed", ""),
            ("wrong-trajectory", "failed", "native versus exact"),
            ("measurement-mismatch", "failed", "plant versus measurement"),
            ("missing", "missing", "native result missing"),
            ("incomplete", "failed", "native trajectory incomplete"),
            ("missing-sample", "failed", "sample coverage"),
            ("wrong-time", "failed", "sample times"),
            ("unaccepted", "failed", "not accepted"),
            ("failed-fixture", "failed", "authored fixture failed"),
            ("failed-integration", "failed", "not completed"),
            ("native-after-failures", "passed", ""),
        ];
        assert_eq!(response.comparisons.len(), expected.len());
        for (actual, (id, status, reason)) in response.comparisons.iter().zip(expected) {
            assert_eq!(
                actual.id, id,
                "mixed results must retain input identity and order"
            );
            assert_eq!(actual.status, status, "{actual:?}");
            assert_eq!(actual.reasons.is_empty(), status == "passed", "{actual:?}");
            if status != "passed" {
                assert!(
                    actual.reasons.iter().any(|value| value.contains(reason)),
                    "{actual:?}"
                );
            }
        }
        Ok(())
    })
    .catch_unwind()
    .await;
    // The scientific reports leave scope before physical/native-lane cleanup, on
    // both success and panic. Failed assertions must not strand native resources.
    let closed = context.close().await;
    match outcome {
        Ok(result) => {
            closed?;
            result
        }
        Err(panic) => {
            if let Err(error) = closed {
                eprintln!("PID native cleanup failed: {error}");
            }
            std::panic::resume_unwind(panic)
        }
    }
}
