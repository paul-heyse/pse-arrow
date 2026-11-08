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
#[path = "reference_support/scalar.rs"]
mod reference_scalar;
#[path = "reference_covariance_support/mod.rs"]
mod support;

use futures_util::FutureExt;
use reference_scalar::{Inputs, Oracle};
use std::{collections::BTreeMap, panic::AssertUnwindSafe, sync::Arc};
use support::{Candidate, CandidateReport, LineCase, Result};

struct Reference {
    estimates: BTreeMap<String, f64>,
    covariance: Vec<f64>,
    inverse_information: Vec<f64>,
}

fn exchange(
    case: &LineCase,
    inputs: Inputs,
    pool: Arc<dyn pse_columnar::MemoryPool>,
) -> Result<Reference> {
    let values = reference_scalar::exchange(inputs, Oracle {
        role: "PSE_COVARIANCE_REFERENCE_PYTHON",
        driver: "covariance_reference_driver.py",
        diagnostic_prefix: "covariance_reference_environment ",
        provider: "IPOPT_DIR",
        solver: "ipopt",
        stages: &[("parmest.fit", 1..=1), ("parmest.covariance", 30..=30)],
    }, pool)?;
    let matrix = |kind: &str| case.parameters.iter().flat_map(|row| {
        let values = &values;
        case.parameters.iter().map(move |column| values[&format!("{kind}:{row}:{column}")])
    }).collect();
    Ok(Reference {
        estimates: case.parameters.iter().map(|name| (name.clone(), values[&format!("estimate:{name}")])).collect(),
        covariance: matrix("covariance"),
        inverse_information: matrix("inverse_information"),
    })
}

fn compare(report: &CandidateReport, case: &LineCase, reference: &Reference) -> std::result::Result<(), String> {
    let require = |condition, reason: &str| if condition { Ok(()) } else { Err(reason.to_owned()) };
    require(report.complete, "native fit incomplete")?;
    let names = &case.parameters;
    require(report.estimates.len() == names.len() && names.iter().all(|name| report.estimates.contains_key(name)), "native estimates are incomplete")?;
    require(report.response_unit == case.response_unit, "response unit differs")?;
    // The independently selected numerical allowance is distinct from engineering budgets.
    for name in names {
        let actual = report.estimates[name];
        require(actual.is_finite() && (actual - reference.estimates[name]).abs() <= 1e-6,
            &format!("estimate differs at {name}"))?;
    }
    require(report.estimate_allowances.len() == names.len() && names.iter().all(|name| {
        report.estimate_allowances.get(name).is_some_and(|budget| budget.is_finite() && *budget > 0.0)
    }), "native engineering estimate budgets are incomplete or invalid")?;
    let covariance = &report.covariance;
    require(covariance.parameters.len() == names.len() && names.iter().all(|name| covariance.parameters.contains(name)),
        "covariance parameters are incomplete or duplicated")?;
    require(covariance.values.len() == names.len() * names.len(), "covariance is incomplete")?;
    require(covariance.coordinates == "physical", "covariance coordinates differ")?;
    require(covariance.layout == "row_major", "covariance layout differs")?;
    require(covariance.convention == case.covariance_convention, "covariance convention differs")?;
    require(covariance.parameter_units == case.parameter_units, "covariance parameter units differ")?;
    require(covariance.parameter_scales == [1.0, 1.0], "covariance parameter scales differ")?;
    let close = |actual: f64, expected: f64| {
        actual.is_finite() && expected.is_finite()
            && (actual - expected).abs() <= (1e-6 * expected.abs()).max(1e-12)
    };
    let order = |name: &str| covariance.parameters.iter().position(|parameter| parameter == name).unwrap();
    for (row, p) in names.iter().enumerate() {
        for (column, q) in names.iter().enumerate() {
            let actual = covariance.values[order(p) * names.len() + order(q)];
            let position = row * names.len() + column;
            require(close(actual, reference.covariance[position]), &format!("covariance differs at ({p}, {q})"))?;
            require(close(actual, reference.inverse_information[position]), &format!("covariance differs from inverse information at ({p}, {q})"))?;
        }
    }
    Ok(())
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
        let (report, contract) = support::run(&native, &case).await?;
        let mut wrong_estimate = report.clone();
        *wrong_estimate.estimates.get_mut("a").unwrap() += 0.01;
        wrong_estimate.estimate_allowances.insert("a".into(), 1.0);
        let mut nonfinite_estimate = report.clone();
        *nonfinite_estimate.estimates.get_mut("a").unwrap() = f64::NAN;
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
        let candidates = [
                record("native", Some(report.clone())),
                record("wrong-estimate", Some(wrong_estimate)),
                record("nonfinite-estimate", Some(nonfinite_estimate)),
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
        ];
        let reference_case = case.clone();
        let pool = native.shared.pool();
        let reference = tokio::task::spawn_blocking(move || exchange(&reference_case, contract, pool)).await??;
        let expected = [
            ("native", "passed", ""),
            ("wrong-estimate", "failed", "estimate differs"),
            ("nonfinite-estimate", "failed", "estimate differs"),
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
        assert_eq!(candidates.len(), expected.len());
        for (candidate, (id, status, reason)) in candidates.iter().zip(expected) {
            let (actual, diagnostic) = match &candidate.report {
                Some(report) => match compare(report, &case, &reference) {
                    Ok(()) => ("passed", String::new()),
                    Err(reason) => ("failed", reason),
                },
                None => ("missing", "native result missing".into()),
            };
            assert_eq!(candidate.id, id, "retain input order through mixed outcomes");
            assert_eq!(actual, status, "{id}: {diagnostic}");
            assert_eq!(diagnostic.is_empty(), status == "passed", "{id}: {diagnostic}");
            assert!(diagnostic.contains(reason), "{id}: {diagnostic}");
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
