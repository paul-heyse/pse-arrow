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
#[path = "reference_support/scalar.rs"]
mod reference_scalar;
#[path = "reference_support/scalar_exchange.rs"]
mod reference_scalar_exchange;
#[path = "reference_sensitivity_support/mod.rs"]
mod support;

use futures_util::FutureExt;
use reference_scalar_exchange::Oracle;
use reference_support::Result;
use std::{collections::BTreeMap, panic::AssertUnwindSafe};
use support::{Candidate, CandidateReport, NativeCandidate, SensitivityCase};

fn compare(
    report: &CandidateReport,
    case: &SensitivityCase,
    reference: &BTreeMap<String, f64>,
) -> std::result::Result<(), String> {
    const DERIVATIVE_ERROR: f64 = 1e-7;
    const INCREMENT_ERROR: f64 = 1e-8;
    const NATIVE_POINT_ERROR: f64 = 5e-7;
    const SIPOPT_POINT_ERROR: f64 = 5e-7;
    let require = |condition, reason: &str| if condition { Ok(()) } else { Err(reason.to_owned()) };
    let close = |actual: f64, expected: f64, allowance: f64, reason: &str| {
        require(actual.is_finite() && expected.is_finite() && (actual - expected).abs() <= allowance, reason)
    };
    require(report.complete, "native response incomplete")?;
    require(report.unit == case.unit, "native sensitivity unit differs")?;
    require(report.coordinates == "physical", "native sensitivity coordinates differ")?;
    require(report.quantities.len() == case.variables.len() && case.variables.iter().all(|name| report.quantities.contains_key(name)),
        "native target population incomplete")?;
    let parameters = ["a", "b"];
    require(report.parameter_scales.len() == parameters.len() && parameters.iter().all(|name| report.parameter_scales.contains_key(*name)),
        "native parameter scales incomplete")?;
    require(report.parameter_scales.values().all(|value| value.is_finite() && *value > 0.0),
        "invalid native parameter scale")?;
    for name in &case.variables {
        let quantity = &report.quantities[name];
        require(quantity.derivative.len() == parameters.len() && parameters.iter().all(|name| quantity.derivative.contains_key(*name)),
            "native derivative population incomplete")?;
        for parameter in parameters {
            close(quantity.derivative[parameter], reference[&format!("derivative:{name}:{parameter}")],
                DERIVATIVE_ERROR, &format!("physical derivative differs for {name}/{parameter}"))?;
        }
        let nominal = reference[&format!("exact:nominal:{name}")];
        let perturbed = reference[&format!("exact:prediction:{name}")];
        close(quantity.base, nominal, NATIVE_POINT_ERROR, &format!("nominal solution differs for {name} ({:?} versus {nominal:?})", quantity.base))?;
        close(quantity.increment, perturbed - nominal, INCREMENT_ERROR, &format!("physical increment differs for {name}"))?;
        close(quantity.predicted, perturbed, NATIVE_POINT_ERROR, &format!("predicted solution differs for {name}"))?;
        close(quantity.predicted, reference[&format!("sipopt:prediction:{name}")],
            NATIVE_POINT_ERROR + SIPOPT_POINT_ERROR, &format!("native/sIPOPT difference for {name}"))?;
        require(quantity.predicted == quantity.base + quantity.increment, &format!("incoherent prediction for {name}"))?;
        let physical_increment = parameters.iter().map(|parameter| quantity.derivative[*parameter]
            * (case.perturbed[*parameter] - case.nominal[*parameter])).sum();
        close(quantity.increment, physical_increment, INCREMENT_ERROR, &format!("incoherent increment for {name}"))?;
        require(quantity.allowance.is_finite() && quantity.allowance > 0.0, "invalid native engineering budget")?;
        require(quantity.coordinate_scale.is_finite() && quantity.coordinate_scale > 0.0, "invalid native output scale")?;
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
async fn native_sensitivity_matches_sipopt_and_rejects_bad_populations() -> Result<()> {
    let native = NativeCandidate::new()?;
    let outcome = AssertUnwindSafe(async {
        let case = SensitivityCase::quadratic();
        let (report, contract) = native.run(&case).await?;
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
        let mut nonfinite = report.clone();
        *nonfinite.quantities.get_mut("x").unwrap().derivative.get_mut("a").unwrap() = f64::NAN;
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
        let candidates = [
                record("native", Some(report.clone())),
                record("wrong-response", Some(wrong)),
                record("nonfinite-response", Some(nonfinite)),
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
        ];
        let pool = native.pool();
        let reference = tokio::task::spawn_blocking(move || reference_scalar_exchange::exchange(contract, Oracle {
            role: "PSE_SENSITIVITY_REFERENCE_PYTHON",
            driver: "sensitivity_reference_driver.py",
            diagnostic_prefix: "sensitivity_reference_environment ",
            provider: "IPOPT_DIR",
            solver: "ipopt_sens",
            stages: &[("sensitivity.sipopt", 1..=1)],
        }, pool)).await??;
        for name in &case.variables {
            for stage in ["nominal", "prediction"] {
                let actual = reference[&format!("sipopt:{stage}:{name}")];
                let expected = reference[&format!("exact:{stage}:{name}")];
                assert!((actual - expected).abs() <= 5e-7, "sIPOPT {stage} differs for {name}");
            }
        }
        let expected = [
            ("native", "passed", ""),
            ("wrong-response", "failed", "physical derivative differs"),
            ("nonfinite-response", "failed", "physical derivative differs"),
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
