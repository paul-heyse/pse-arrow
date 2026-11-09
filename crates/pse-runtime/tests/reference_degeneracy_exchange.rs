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
#[path = "reference_support/scalar.rs"]
mod reference_scalar;
#[path = "reference_support/scalar_exchange.rs"]
mod reference_scalar_exchange;

#[path = "reference_degeneracy_support/mod.rs"]
mod support;

use futures_util::FutureExt;
use pse_ids::SemanticId;
use pse_model::reference_exchange::{ReferenceDegenerateCertificate, ReferenceDegeneracyResult, ReferenceMeaning};
use reference_scalar::Inputs;
use reference_scalar_exchange::Oracle;
use std::{collections::{BTreeMap, BTreeSet}, panic::AssertUnwindSafe};
use support::{Candidate, CandidateReport, LinearCase, NativeCandidate, Result};

const SUPPORTS: [(&[&str], &[f64]); 2] = [
    (&["first", "second"], &[2.0, -1.0]),
    (&["third", "fourth", "fifth"], &[1.0, 1.0, -1.0]),
];
type Multipliers = BTreeMap<String, f64>;
type IndependentSupports = [Option<Multipliers>; SUPPORTS.len()];

fn proportional(actual: &Multipliers, expected: &Multipliers) -> std::result::Result<(), String> {
    if actual.keys().ne(expected.keys()) || actual.is_empty() {
        return Err("multiplier support mismatch".into());
    }
    if !actual.values().chain(expected.values()).all(|value| value.is_finite() && *value != 0.0) {
        return Err("multipliers must be finite and nonzero on their support".into());
    }
    let pivot = actual.keys().next().unwrap();
    for (row, multiplier) in actual {
        let actual_ratio = multiplier / actual[pivot];
        let expected_ratio = expected[row] / expected[pivot];
        if !actual_ratio.is_finite() || (actual_ratio - expected_ratio).abs() > (1e-6 * expected_ratio.abs()).max(1e-12) {
            return Err(format!("multiplier ratio differs at {row}"));
        }
    }
    Ok(())
}

fn weights(rows: &[&str], multipliers: &[f64]) -> Multipliers {
    rows.iter().zip(multipliers).map(|(row, multiplier)| ((*row).to_owned(), *multiplier)).collect()
}

fn validate_reference(
    meaning: &ReferenceMeaning,
    expected: &[Vec<SemanticId>; SUPPORTS.len()],
    result: &ReferenceDegeneracyResult,
) -> Result<IndependentSupports> {
    if result.version != 1 || &result.meaning != meaning || result.supports.len() != expected.len() {
        return Err("reference degeneracy result differs from its original context or support population".into());
    }
    let mut selected: IndependentSupports = Default::default();
    for (index, (support, expected_ids)) in result.supports.iter().zip(expected).enumerate() {
        if &support.rows != expected_ids {
            return Err("reference degeneracy support rows differ from their exact original identity/order".into());
        }
        if let ReferenceDegenerateCertificate::Reported { multipliers, irreducible_at_tolerance } = &support.certificate {
            let (rows, original) = SUPPORTS[index];
            if !irreducible_at_tolerance || multipliers.len() != rows.len() {
                return Err("reference degeneracy certificate is incomplete or not irreducible".into());
            }
            let actual = weights(rows, multipliers);
            proportional(&actual, &weights(rows, original))?;
            selected[index] = Some(actual);
        }
    }
    if selected.iter().all(Option::is_none) {
        return Err("IDAES must report at least one independent irreducible support".into());
    }
    Ok(selected)
}

fn exchange(inputs: Inputs) -> Result<IndependentSupports> {
    const LIMIT: usize = 64 << 10;
    let meaning = inputs.contract.meaning.clone();
    let ids = inputs.contract.observables.iter().map(|coordinate| (coordinate.key.as_str(), coordinate.output)).collect::<BTreeMap<_, _>>();
    let expected = SUPPORTS.map(|(rows, _)| rows.iter().map(|row| ids[format!("row:{row}").as_str()]).collect());
    let (bytes, execution) = reference_scalar_exchange::perform(inputs, Oracle {
        role: "PSE_DEGENERACY_REFERENCE_PYTHON",
        driver: "degeneracy_reference_driver.py",
        diagnostic_prefix: "degeneracy_reference_environment ",
        provider: "SCIPOPTDIR",
        solver: "scip",
        stages: &[("degeneracy.search", 2..=3)],
    }, LIMIT)?;
    let decoded = pse_runtime::reference_exchange::decode_degeneracy_result(&bytes, LIMIT)
        .map_err(|error| execution.failure("response refused", error))?;
    let original = decoded.known();
    let reference = validate_reference(&meaning, &expected, original)
        .map_err(|error| execution.failure("reference context refused", error))?;
    let mut foreign = original.clone();
    foreign.meaning.physical = pse_ids::ContentHash::from_bytes([0x77; 32]);
    assert!(validate_reference(&meaning, &expected, &foreign).is_err());
    let mut duplicate = original.clone();
    duplicate.supports[0].rows[1] = duplicate.supports[0].rows[0];
    assert!(validate_reference(&meaning, &expected, &duplicate).is_err());
    let mut unreported = original.clone();
    for support in &mut unreported.supports {
        support.certificate = ReferenceDegenerateCertificate::Unreported;
    }
    assert!(validate_reference(&meaning, &expected, &unreported).is_err());
    Ok(reference)
}

fn compare(report: &CandidateReport, reference: &IndependentSupports) -> std::result::Result<(), String> {
    if !report.complete {
        return Err("native all-anchor analysis incomplete".into());
    }
    if !report.unavailable.is_empty() {
        return Err(format!("native analysis unavailable: {:?}", report.unavailable));
    }
    if report.sets.len() != SUPPORTS.len() {
        return Err("native result must contain both constructed sets".into());
    }
    let mut visited = BTreeSet::new();
    for item in &report.sets {
        let rows = item.rows.iter().map(String::as_str).collect::<BTreeSet<_>>();
        if rows.is_empty() || rows.len() != item.rows.len() {
            return Err("duplicate or empty rows".into());
        }
        if item.multipliers.keys().map(String::as_str).ne(rows.iter().copied()) {
            return Err("certificate and row support differ".into());
        }
        if !item.irreducible_at_tolerance {
            return Err("set is not irreducible at tolerance".into());
        }
        let index = SUPPORTS.iter().position(|(expected, _)| expected.len() == rows.len() && expected.iter().all(|row| rows.contains(row)))
            .ok_or("native result must contain both constructed sets")?;
        if !visited.insert(index) {
            return Err("duplicate degenerate set".into());
        }
        if let Some(independent) = &reference[index] {
            proportional(&item.multipliers, independent)?;
        }
        let (expected_rows, original) = SUPPORTS[index];
        proportional(&item.multipliers, &weights(expected_rows, original))?;
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
async fn native_degeneracy_matches_independent_reference_and_rejects_bad_populations() -> Result<()>
{
    let native = NativeCandidate::new()?;
    let outcome = AssertUnwindSafe(async {
        let case = LinearCase::dependent();
        let (report, contract) = native.run(&case).await?;
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
        let candidates = [
                record("native", Some(report.clone())),
                record("wrong-multiplier", Some(wrong)),
                record("missing", None),
                record("incomplete", Some(incomplete)),
                record("missing-set", Some(missing_set)),
                record("reducible", Some(reducible)),
                record("native-after-failures", Some(report)),
        ];
        let reference = tokio::task::spawn_blocking(move || exchange(contract)).await??;
        let expected = [
            ("native", "passed", ""),
            ("wrong-multiplier", "failed", "multiplier ratio differs"),
            ("missing", "missing", "native result missing"),
            ("incomplete", "failed", "all-anchor analysis incomplete"),
            ("missing-set", "failed", "both constructed sets"),
            ("reducible", "failed", "not irreducible at tolerance"),
            ("native-after-failures", "passed", ""),
        ];
        assert_eq!(candidates.len(), expected.len());
        for (candidate, (id, status, reason)) in candidates.iter().zip(expected) {
            let (actual, diagnostic) = match &candidate.report {
                Some(report) => match compare(report, &reference) {
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
