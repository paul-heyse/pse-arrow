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

#[path = "reference_support/mod.rs"]
mod reference_support;

#[path = "reference_degeneracy_support/mod.rs"]
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

fn pins() -> Result<BTreeMap<String, String>> {
    // Consume the existing declarations rather than introduce another pin owner.
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
    let python = PathBuf::from(std::env::var_os("PSE_DEGENERACY_REFERENCE_PYTHON")
        .expect("select an existing locked reference interpreter with PSE_DEGENERACY_REFERENCE_PYTHON; absence is failure"));
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
        .arg(support::root().join("tests/reference/oracles/degeneracy.py"))
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
    assert_eq!(response.environment["idaes-pse"], request.pins["idaes-pse"]);
    assert_eq!(response.environment["pyomo"], request.pins["pyomo"]);
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
            pins: pins()?,
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
