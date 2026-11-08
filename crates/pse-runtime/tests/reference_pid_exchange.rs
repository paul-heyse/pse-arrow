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

#[path = "reference_support/environment.rs"]
mod reference_environment;
#[path = "reference_support/process.rs"]
mod reference_process;
#[path = "reference_support/context.rs"]
mod reference_support;
#[path = "../../../tests/fixtures/authored-scientific-inputs.rs"]
mod scientific_inputs;
#[path = "reference_pid_support/mod.rs"]
mod support;

use futures_util::FutureExt;
use pse_model::generated::enums::TrajectoryTermination;
use pse_model::reference_exchange::ReferenceOutcome;
use pse_runtime::reference_exchange;
use std::{collections::BTreeMap, panic::AssertUnwindSafe, process::Command};
use support::{Candidate, CandidateReport, LoopCase, ReferenceInputs, Result};

struct Reference {
    exact_process: Vec<f64>,
    petsc_process: Vec<f64>,
    steady_process: f64,
    steady_integral_seconds: f64,
}

fn exchange(case: &LoopCase, inputs: ReferenceInputs) -> Result<Reference> {
    const LIMIT: usize = 1 << 20;
    let python = reference_environment::interpreter("PSE_PID_REFERENCE_PYTHON");
    let interpreter_digest = reference_environment::image_digest(&python)?;
    let solver = reference_environment::prepared("IDAES_DATA", "petsc")?;
    let scratch = tempfile::tempdir()?;
    let descriptors = scratch.path().join("descriptor.bin");
    let input = scratch.path().join("contract.pb");
    let output = scratch.path().join("reference.pb");
    std::fs::write(&descriptors, reference_exchange::generated::DESCRIPTORS)?;
    let ReferenceInputs { contract, _owners } = inputs;
    let meaning = contract.meaning.clone();
    let expected = contract
        .observables
        .iter()
        .map(|coordinate| {
            let physical = &coordinate.physical;
            (
                coordinate.output,
                physical.quantity.id.as_id(),
                physical.representation_unit.id.as_id(),
                physical.quantity.key.basis.map(|id| id.as_id()),
                physical.quantity.key.reference_state.map(|id| id.as_id()),
            )
        })
        .collect::<Vec<_>>();
    std::fs::write(
        &input,
        reference_exchange::encode_case_contract(contract, LIMIT)?,
    )?;
    let mut command = Command::new(&python);
    command
        .arg("-I")
        .arg(support::root().join("tests/reference/pid_reference_driver.py"))
        .arg(&descriptors)
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
    let mut evidence = execution
        .diagnostics
        .lines()
        .filter_map(|line| line.strip_prefix("pid_reference_environment "));
    let environment: BTreeMap<String, String> = serde_json::from_str(
        evidence
            .next()
            .ok_or_else(|| execution.failure("environment refused", "missing solver evidence"))?,
    )?;
    if evidence.next().is_some() {
        return Err(execution.failure("environment refused", "duplicate solver evidence"));
    }
    reference_environment::verify(
        &environment,
        &python,
        &interpreter_digest,
        &reference_environment::pins(&support::root())?,
    )
    .map_err(|error| execution.failure("environment refused", error))?;
    reference_environment::verify_stage(&environment, "pid.initial_conditions", &solver, 1..=1)
        .map_err(|error| execution.failure("solver evidence refused", error))?;
    reference_environment::verify_stage(&environment, "pid.integration", &solver, 4..=4)
        .map_err(|error| execution.failure("solver evidence refused", error))?;
    let bytes = reference_process::read_response(&output, LIMIT)
        .map_err(|error| execution.failure("response refused", error))?;
    let response = reference_exchange::decode(&bytes, LIMIT)
        .map_err(|error| execution.failure("response refused", error))?;
    let response = response.known();
    assert_eq!(response.meaning, meaning);
    let ReferenceOutcome::Completed { outputs, blocks } = &response.outcome else {
        return Err(execution.failure("response refused", "PID reference did not complete"));
    };
    assert!(blocks.is_empty());
    assert_eq!(outputs.len(), expected.len());
    assert_eq!(response.required.len(), outputs.len());
    for ((actual, required), (output, quantity, unit, basis, datum)) in
        outputs.iter().zip(&response.required).zip(expected)
    {
        assert_eq!(actual.id, *required);
        assert_eq!(actual.id, output);
        assert_eq!(actual.quantity, quantity);
        assert_eq!(actual.unit, unit);
        assert_eq!(actual.basis, basis);
        assert_eq!(actual.datum, datum);
        assert!(actual.value.is_finite());
    }
    let samples = case.samples_seconds.len();
    let reference = Reference {
        exact_process: outputs[..samples]
            .iter()
            .map(|output| output.value)
            .collect(),
        petsc_process: outputs[samples..2 * samples]
            .iter()
            .map(|output| output.value)
            .collect(),
        steady_process: outputs[2 * samples].value,
        steady_integral_seconds: outputs[2 * samples + 1].value,
    };
    assert!((reference.steady_process - case.setpoint).abs() <= case.engineering_fraction);
    let steady_integral = case.ti_seconds * (case.setpoint - case.bias) / case.kp;
    assert!(
        (reference.steady_integral_seconds - steady_integral).abs() <= case.engineering_fraction
    );
    for (exact, petsc) in reference.exact_process.iter().zip(&reference.petsc_process) {
        assert!((exact - petsc).abs() <= case.engineering_fraction);
    }
    Ok(reference)
}

fn compare(
    report: &CandidateReport,
    case: &LoopCase,
    reference: &Reference,
) -> std::result::Result<(), String> {
    if !report.complete {
        return Err("native trajectory incomplete".into());
    }
    if !report.accepted {
        return Err("native trajectory not accepted".into());
    }
    if report.termination != TrajectoryTermination::Completed {
        return Err("native integration not completed".into());
    }
    if !report.fixture_passed {
        return Err("native authored fixture failed".into());
    }
    let samples = case.samples_seconds.len();
    if report.samples_seconds.len() != samples
        || report
            .samples_seconds
            .iter()
            .zip(&case.samples_seconds)
            .any(|(actual, expected)| actual.to_bits() != expected.to_bits())
    {
        return Err("native sample times differ".into());
    }
    if report.process.len() != samples || report.measurement.len() != samples {
        return Err("native sample coverage incomplete".into());
    }
    let close = |left: f64, right: f64| {
        left.is_finite() && right.is_finite() && (left - right).abs() <= case.engineering_fraction
    };
    for ((((ours, measured), exact), petsc), time) in report
        .process
        .iter()
        .zip(&report.measurement)
        .zip(&reference.exact_process)
        .zip(&reference.petsc_process)
        .zip(&case.samples_seconds)
    {
        if !close(*ours, *measured) {
            return Err(format!("plant versus measurement at {time}s"));
        }
        if !close(*ours, *exact) {
            return Err(format!("native versus exact at {time}s"));
        }
        if !close(*ours, *petsc) {
            return Err(format!("native versus PETSc at {time}s"));
        }
    }
    Ok(())
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
        let (report, contract) = support::run(&context, &case).await?;
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
        stopped.termination = TrajectoryTermination::Failed;
        let candidates = [
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
        ];
        let original_case = case.clone();
        let reference =
            tokio::task::spawn_blocking(move || exchange(&original_case, contract)).await??;
        let expected = [
            ("native", "passed", ""),
            ("wrong-trajectory", "failed", "native versus exact at 5s"),
            (
                "measurement-mismatch",
                "failed",
                "plant versus measurement at 2s",
            ),
            ("missing", "missing", "native result missing"),
            ("incomplete", "failed", "native trajectory incomplete"),
            ("missing-sample", "failed", "sample coverage"),
            ("wrong-time", "failed", "sample times"),
            ("unaccepted", "failed", "not accepted"),
            ("failed-fixture", "failed", "authored fixture failed"),
            ("failed-integration", "failed", "not completed"),
            ("native-after-failures", "passed", ""),
        ];
        assert_eq!(candidates.len(), expected.len());
        for (candidate, (id, status, reason)) in candidates.iter().zip(expected) {
            let (actual, diagnostic) = match candidate.report.as_ref() {
                Some(report) => match compare(report, &case, &reference) {
                    Ok(()) => ("passed", String::new()),
                    Err(reason) => ("failed", reason),
                },
                None => ("missing", "native result missing".into()),
            };
            assert_eq!(
                candidate.id, id,
                "mixed results must retain input identity and order"
            );
            assert_eq!(actual, status, "{id}: {diagnostic}");
            assert!(diagnostic.contains(reason), "{id}: {diagnostic}");
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
