// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native scientific candidate challenged by an isolated IDAES/Pyomo oracle.
//! Register this target with required-features = ["solver-highs"]. Reference absence fails.
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

#[path = "reference_degeneracy_support/mod.rs"]
mod support;

use futures_util::FutureExt;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, panic::AssertUnwindSafe, process::Command};
use support::{Candidate, CandidateReport, LinearCase, NativeCandidate, Result};

#[derive(Serialize)]
struct Request {
    version: u32,
    case: LinearCase,
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
    reference: Vec<support::RowSet>,
    comparisons: Vec<Comparison>,
}

fn exchange(request: &Request) -> Result<Response> {
    let python = reference_environment::interpreter("PSE_DEGENERACY_REFERENCE_PYTHON");
    let interpreter_digest = reference_environment::image_digest(&python)?;
    let solver = reference_environment::prepared("SCIPOPTDIR", "scip")?;
    let scratch = tempfile::tempdir()?;
    let input = scratch.path().join("request.json");
    let output = scratch.path().join("response.json");
    let limit = reference_process::legacy_response_limit(request.candidates.len())?;
    std::fs::write(&input, serde_json::to_vec(request)?)?;
    let mut command = Command::new(&python);
    command
        .arg("-I")
        .arg(support::root().join("tests/reference/legacy_reference_driver.py"))
        .arg("degeneracy")
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
    reference_environment::verify_stage(&response.environment, "degeneracy.search", &solver, 2..=3)
        .map_err(|error| execution.failure("solver evidence refused", error))?;
    assert!(
        !response.reference.is_empty(),
        "IDAES must supply an independent nonempty set"
    );
    Ok(response)
}

fn record(id: &str, report: Option<CandidateReport>) -> Candidate {
    Candidate {
        id: id.into(),
        report,
    }
}

#[tokio::test]
async fn native_degeneracy_matches_independent_reference_and_rejects_bad_populations() -> Result<()>
{
    let native = NativeCandidate::new()?;
    let outcome = AssertUnwindSafe(async {
        let case = LinearCase::dependent();
        let report = native.run(&case).await?;
        assert!(
            report.complete,
            "native analysis must visit every anchor: {:?}",
            report.unavailable
        );
        assert_eq!(
            report.sets.len(),
            2,
            "both independently constructed degenerate sets are required"
        );
        assert!(report.sets.iter().all(|set| set.irreducible_at_tolerance));

        let mut wrong = report.clone();
        // Preserve the row sets and completeness while corrupting only a multiplier.
        let set = wrong
            .sets
            .iter_mut()
            .find(|set| set.rows.iter().any(|row| row == "first"))
            .unwrap();
        *set.multipliers.get_mut("first").unwrap() *= 1.25;
        let mut incomplete = report.clone();
        incomplete.complete = false;
        incomplete
            .unavailable
            .push("deliberately unvisited anchor".into());
        let mut missing_set = report.clone();
        missing_set.sets.pop();
        let mut reducible = report.clone();
        reducible.sets[0].irreducible_at_tolerance = false;
        let request = Request {
            version: 1,
            case,
            pins: reference_environment::pins(&reference_support::root())?,
            candidates: vec![
                record("native", Some(report.clone())),
                record("wrong-multiplier", Some(wrong)),
                record("missing", None),
                record("incomplete", Some(incomplete)),
                record("missing-set", Some(missing_set)),
                record("reducible", Some(reducible)),
                record("native-after-failures", Some(report)),
            ],
        };
        let response = tokio::task::spawn_blocking(move || exchange(&request)).await??;
        let expected = [
            ("native", "passed", ""),
            ("wrong-multiplier", "failed", "multiplier ratio differs"),
            ("missing", "missing", "native result missing"),
            ("incomplete", "failed", "all-anchor analysis incomplete"),
            ("missing-set", "failed", "both constructed sets"),
            ("reducible", "failed", "not irreducible at tolerance"),
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
                "input order and identity must survive mixed results"
            );
            assert_eq!(actual.status, status, "{actual:?}");
            assert_eq!(actual.reasons.is_empty(), status == "passed", "{actual:?}");
            if status != "passed" {
                assert!(
                    actual.reasons.iter().any(|actual| actual.contains(reason)),
                    "{actual:?}"
                );
            }
        }
        Ok(())
    })
    .catch_unwind()
    .await;
    // Settle native lanes even when scientific or reference-environment assertions fail.
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
