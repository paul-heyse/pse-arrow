// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Phase disappearance authored as complementarity through the real pipeline (Plan 22 M5a,
//! ADR-0104 §5). The authored `ComplementarityVLE` of the reference `equilibrium` package
//! and its `bt_ideal` flash fixtures state every equation; the realizations only lower the
//! complementarity pairs.
use super::fixtures::*;
use pse_backend_native::solve::Backend;
use pse_ids::SemanticId;
use pse_model::generated::enums::{ModelingStructuralRequirement as Requirement, NumericalTarget};
use pse_runtime::{
    CancelSource,
    math::solves::{Outcome, SolverProfile},
    workflow::{ModelingPackage, ModelingSolvePreparation, RunReport},
};
use std::collections::BTreeMap;
use std::time::Duration;

#[derive(Debug)]
struct FlashEvidence {
    vapor: f64,
    temperature: f64,
    temperature_allowance: f64,
    temperature_binding_budget: f64,
    scalar_budget: f64,
    fraction_resolution: f64,
}

// Read the frozen input bank, not the production saturation evaluator. The
// independent Wagner expression brackets the bubble/dew temperatures away
// from the two feeds (358/378 K), without using the solved phase fraction.
fn independent_phase_brackets(delta: f64) {
    use datafusion::arrow::array::{Float64Array, StringArray};
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/reference/data/oracles/idaes-2.13/data/rpp4_pressure_oracles.parquet");
    let reader = ParquetRecordBatchReaderBuilder::try_new(std::fs::File::open(path).unwrap())
        .unwrap().build().unwrap();
    let mut pressures = BTreeMap::new();
    for batch in reader {
        let batch = batch.unwrap();
        let subjects = batch.column_by_name("subject").unwrap().as_any()
            .downcast_ref::<StringArray>().unwrap();
        for row in 0..batch.num_rows() {
            if !matches!(subjects.value(row), "71-43-2" | "108-88-3") { continue; }
            let value = |name| batch.column_by_name(name).unwrap().as_any()
                .downcast_ref::<Float64Array>().unwrap().value(row);
            let saturation = |temperature: f64| {
                let tau = 1. - temperature / value("tc");
                value("pc") * ((value("a")*tau + value("b")*tau.powf(1.5)
                    + value("c")*tau.powi(3) + value("d")*tau.powi(6)) / (1.-tau)).exp()
            };
            pressures.insert(subjects.value(row).to_owned(), [saturation(363.), saturation(373.)]);
        }
    }
    assert_eq!(pressures.len(), 2);
    let mut bubble_residual_upper = 0.;
    let mut dew_residual_lower = 0.;
    for pressure in pressures.values() {
        // Rachford-Rice is increasing in K and decreasing in beta. Account
        // separately for log-fugacity, material, beta-bound and normalization
        // residuals using their common conservative resolved upper budget.
        for (k, beta, upper) in [
            (pressure[0]/101325. * delta.exp(), -delta, true),
            (pressure[1]/101325. * (-delta).exp(), 1.+delta, false),
        ] {
            let denominator = 1. + beta*(k-1.);
            assert!(denominator > 0.);
            let coefficient = (k-1.)/denominator;
            let weight = 0.5 + if (coefficient >= 0.) == upper { delta } else { -delta };
            if upper { bubble_residual_upper += weight*coefficient; }
            else { dew_residual_lower += weight*coefficient; }
        }
    }
    // Even the perturbed original equations cannot have a root outside these
    // brackets: the remaining normalization residual is bounded by delta.
    assert!(bubble_residual_upper < -delta, "bubble bracket: {bubble_residual_upper}, budget {delta}");
    assert!(dew_residual_lower > delta, "dew bracket: {dew_residual_lower}, budget {delta}");
}

fn absent_phase_bound(slack_lower: f64, residual: f64, fraction: f64, smooth: bool) -> f64 {
    assert!(slack_lower.is_finite() && residual.is_finite() && fraction.is_finite());
    assert!(slack_lower > residual && residual >= 0. && fraction > 0.);
    // If r=smooth_min(a,b,eps), (a-r)(b-r)=eps²/4. With
    // a>=slack_lower and |r|<=residual, b<=residual+eps²/(4(a-residual)).
    // Disjunctive pairs permit only the residual/fraction envelope; penalty
    // product bounds are tighter here because the independently bounded slack>1.
    fraction + residual + if smooth { 1e-8 / (4. * (slack_lower-residual)) } else { 0. }
}

fn assert_phase_regime(feed: &str, evidence: &FlashEvidence, smooth: bool) {
    let delta = evidence.scalar_budget.max(evidence.fraction_resolution);
    independent_phase_brackets(delta);
    let slack_lower = 5. - evidence.temperature_binding_budget - delta;
    let bound = absent_phase_bound(slack_lower, delta, evidence.fraction_resolution, smooth);
    assert!(0.1 > bound, "the independent wrong-interior control must discriminate: {bound}");
    let absent = match feed {
        "liquid" => evidence.vapor,
        "vapor" => 1.-evidence.vapor,
        "two_phase" => {
            assert!(evidence.vapor > bound && 1.-evidence.vapor > bound, "{evidence:?}");
            return;
        }
        _ => panic!("unknown authored feed"),
    };
    assert!(absent.is_finite() && absent.abs() <= bound, "{feed}: {evidence:?}; absent bound={bound}");
}

#[test]
fn phase_regime_envelope_rejects_interior_and_missing_slack() {
    let bound = absent_phase_bound(4.8, 0.001, 0.001, true);
    assert!(1e-9 <= bound);
    assert!(0.1 > bound);
    assert!(std::panic::catch_unwind(|| absent_phase_bound(0., 0.001, 0.001, true)).is_err());
    let mut evidence = FlashEvidence { vapor: 1.-1e-9, temperature: 371., temperature_allowance: 0.1,
        temperature_binding_budget: 0.1, scalar_budget: 0.001, fraction_resolution: 0.001 };
    assert_phase_regime("vapor", &evidence, true);
    evidence.vapor = 0.9;
    assert!(std::panic::catch_unwind(|| assert_phase_regime("vapor", &evidence, true)).is_err());
    evidence.vapor = 0.1;
    assert!(std::panic::catch_unwind(|| assert_phase_regime("liquid", &evidence, true)).is_err());
}

/// Select real complementarity fixture identities from the intact loaded package.
async fn complementarity_fixtures(package: &ModelingPackage) -> BTreeMap<String, SemanticId> {
    let tests = package
        .declarations()
        .await
        .unwrap()
        .iter()
        .filter(|r| r.value.kind.as_str() == "test" && r.name.starts_with("complementarity_"))
        .map(|r| (r.name.clone(), r.declaration_id.as_id()))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(tests.len(), 9, "3 feeds × 3 realizations");
    tests
}
/// An explicit route to a feasible point of the fixture's system.
fn feasible(backend: Backend) -> SolverProfile {
    let mut solver = profile(backend, false);
    solver.intent = pse_backend_native::solve::SolveIntent::FeasiblePoint;
    solver.controls.time_limit = Duration::from_secs(300);
    solver
}
/// Solve an authored fixture and return its vapor fraction and equilibrium temperature.
async fn solve(
    package: &ModelingPackage,
    case: SemanticId,
    solver: SolverProfile,
) -> FlashEvidence {
    let prepared = seed_prepare(package, case, solver, &CancelSource::new())
        .await
        .unwrap();
    solve_prepared(&prepared).await
}
/// Execute the same admitted preparation whose route and closure were inspected.
async fn solve_prepared(prepared: &ModelingSolvePreparation) -> FlashEvidence {
    let result = prepared.start().unwrap().wait().await.unwrap();
    if let Ok(RunReport::Modeling(reports)) = result.report() {
        for report in reports
            .iter()
            .filter(|report| !report.accepted || !result.usable())
        {
            for value in report.reports.iter().filter(|value| {
                matches!(
                    value.label.as_str(),
                    "vapor fraction" | "equilibrium temperature"
                )
            }) {
                eprintln!(
                    "flash rejected output: label={} value={:.17e}",
                    value.label, value.value
                );
            }
            for check in report
                .checks
                .iter()
                .filter(|check| !check.satisfied)
                .take(8)
            {
                eprintln!(
                    "flash rejected check: source={} target={} kind={:?} error={:.17e} tolerance={:?} satisfied={}",
                    check.source_id,
                    check.target_id,
                    check.kind,
                    check.value,
                    check.tolerance,
                    check.satisfied
                );
            }
            if let Outcome::Native(native) = &report.outcome {
                eprintln!(
                    "flash rejected native: backend={:?} code={} name={} termination={:?} qualification={:?} kkt={:?} work={:?}",
                    native.backend,
                    native.termination.code,
                    native.termination.name,
                    native.termination.category,
                    native.qualification,
                    native.evidence.kkt,
                    native.evidence.work
                );
                if let Some(quality) = &native.quality {
                    eprintln!(
                        "flash rejected native quality: normalized_max={:.17e}",
                        quality.normalized_max
                    );
                    for (kind, violation) in quality
                        .rows
                        .iter()
                        .map(|violation| ("row", violation))
                        .chain(quality.bounds.iter().map(|violation| ("bound", violation)))
                        .filter(|(_, violation)| violation.physical > violation.tolerance)
                        .take(8)
                    {
                        eprintln!(
                            "flash rejected native violation: kind={kind} source={} physical={:.17e} tolerance={:.17e}",
                            violation.id, violation.physical, violation.tolerance
                        );
                    }
                } else {
                    eprintln!("flash rejected native quality: unavailable");
                }
                for name in [
                    "tol",
                    "constr_viol_tol",
                    "dual_inf_tol",
                    "compl_inf_tol",
                    "l1_exact_penalty_barrier",
                    "hessian_approximation",
                    "max_wall_time",
                ] {
                    if let Some(value) = native.options.get(name) {
                        eprintln!("flash rejected native option: name={name} value={value:?}");
                    }
                }
            }
        }
    }
    let report = authored_success(&result);
    assert!(matches!(result.report(), Ok(RunReport::Modeling(_))));
    let Outcome::Native(native) = &report.outcome else {
        panic!("expected native complementarity realization");
    };
    assert!(
        native
            .quality
            .as_ref()
            .is_some_and(|quality| quality.feasible())
    );
    let vapor = report
        .reports
        .iter()
        .find(|r| r.label == "vapor fraction")
        .unwrap();
    let registry = &report.prepared.model.case.compiled().quantities;
    let numerics = report.prepared.solve.numerics();
    let vapor = vapor.value;
    let temperature = report
        .reports
        .iter()
        .find(|r| r.label == "equilibrium temperature")
        .unwrap();
    let temperature_allowance = resolved_physical_allowance(
        numerics,
        registry,
        temperature,
        SemanticId::parse_hex("13874d4b57684720bb42a48164325812").unwrap(),
    );
    let temperature = temperature.value;
    assert!(vapor.is_finite());
    assert!(temperature.is_finite());
    let model = &report.prepared.model.model.compiled().model;
    let row_budget = |id| numerics.targets.iter().find(|target| target.kind == NumericalTarget::Row && target.id == id)
        .expect("every original phase row has a resolved budget").budget;
    let mut scalar_budget: f64 = 0.;
    for (suffix, count) in [(".material", 2), (".fugacity", 2), (".normalization", 1)] {
        let rows = model.equations.iter().filter(|row| row.lineage.path.ends_with(suffix)).collect::<Vec<_>>();
        assert_eq!(rows.len(), count, "phase premise {suffix}");
        for row in rows { scalar_budget = scalar_budget.max(row_budget(row.id)); }
    }
    for suffix in [".liquid_disappearance", ".vapor_disappearance"] {
        let rows = model.equations.iter().filter(|row| row.lineage.path.ends_with(suffix)).collect::<Vec<_>>();
        assert!(!rows.is_empty(), "actual complementarity realization rows required");
        for row in rows { scalar_budget = scalar_budget.max(row_budget(row.id)); }
    }
    for suffix in [".beta", ".liquid_slack", ".vapor_slack"] {
        let symbols = model.symbols.values().filter(|symbol| symbol.lineage.path.ends_with(suffix)).collect::<Vec<_>>();
        assert_eq!(symbols.len(), 1, "phase premise {suffix}");
        let target = numerics.targets.iter().find(|target| target.kind == NumericalTarget::Variable && target.id == symbols[0].id)
            .expect("phase variable bound budget");
        scalar_budget = scalar_budget.max(target.budget);
    }
    let bindings = model.equations.iter().filter(|row| row.lineage.path.ends_with(".equilibrium_temperature_binding")).collect::<Vec<_>>();
    assert_eq!(bindings.len(), 1);
    let temperature_binding_budget = row_budget(bindings[0].id);
    let fraction_resolution = numerics.targets.iter().filter_map(|target| target.engineering.as_ref())
        .filter(|context| context.rule_id.is_some_and(|id| id.as_id() == SemanticId::parse_hex("8a097841b11d4824b0d5041250553949").unwrap()))
        .filter_map(|context| context.physical_allowance).reduce(f64::max)
        .expect("authored dimensionless fraction resolution");
    FlashEvidence { vapor, temperature, temperature_allowance, temperature_binding_budget, scalar_budget, fraction_resolution }
}

#[tokio::test]
async fn flash_phase_disappearance_agrees_across_realizations() {
    let owner = WorkflowRuntime::new().unwrap();
    let package = seed_package(&owner).await;
    let cases = complementarity_fixtures(&package).await;
    // Historical beta comparisons belong to the source-authored expectations.
    for feed in ["liquid", "two_phase", "vapor"] {
        // smooth(math.smooth_min, 1e-4): a square system on the NLP route.
        let smooth = solve(
            &package,
            cases[&format!("complementarity_{feed}_smooth")],
            feasible(Backend::Ipopt),
        )
        .await;
        // disjunctive: slack columns and a native SOS1, on SCIP.
        let disjunctive = solve(
            &package,
            cases[&format!("complementarity_{feed}_disjunctive")],
            feasible(Backend::Scip),
        )
        .await;
        // Empirical agreement uses the resolved shared temperature floor.
        // Bound feasibility budgets do not guarantee forward output accuracy.
        assert!(
            (smooth.temperature - disjunctive.temperature).abs() <= smooth.temperature_allowance.min(disjunctive.temperature_allowance),
            "{feed}: smooth {} K, disjunctive {} K",
            smooth.temperature,
            disjunctive.temperature
        );
        // penalty(l1): nonnegative members, a product row and the l1 exact-penalty
        // requirement of its case structure, which selects POUNCE's l1 method: the
        // author's selection (ADR-0104 §5, Plan 22 M5b).
        let penalty_case = cases[&format!("complementarity_{feed}_penalty")];
        let prepared = seed_prepare(
            &package,
            penalty_case,
            feasible(Backend::Pounce),
            &CancelSource::new(),
        )
        .await
        .unwrap();
        assert_eq!(
            prepared
                .model
                .case
                .compiled()
                .plan
                .structure()
                .requirements(),
            [Requirement::L1ExactPenalty]
        );
        let penalty = solve_prepared(&prepared).await;
        // The realizations agree within the shared physical output resolution.
        assert!(
            (penalty.temperature - disjunctive.temperature).abs() <= penalty.temperature_allowance.min(disjunctive.temperature_allowance),
            "{feed}: penalty {} K, disjunctive {} K",
            penalty.temperature,
            disjunctive.temperature
        );
        assert_phase_regime(feed, &smooth, true);
        assert_phase_regime(feed, &disjunctive, false);
        assert_phase_regime(feed, &penalty, false);
        // Another backend cannot honour the requirement.
        assert!(
            seed_prepare(
                &package,
                penalty_case,
                feasible(Backend::Ipopt),
                &CancelSource::new()
            )
            .await
            .is_err()
        );
    }
}
