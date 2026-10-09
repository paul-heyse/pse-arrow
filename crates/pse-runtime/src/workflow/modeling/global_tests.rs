// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Authored global, discrete and certified journeys through the real pipeline to SCIP
//! (Plan 22 G4, G5, G7).
use super::*;
use crate::math::solves::{NumericalInputs, Outcome, SolverProfile};
use crate::workflow::tests as fixture;
use pse_backend_native::{
    execution::{BackendSettings, ScipSettings},
    solve::{Assurance, Backend, Metric, Qualification, SolveIntent, SolverSelection},
};
use pse_compiler::workspace::ModelingCaseBindings;
use pse_kernels::DerivativeOrder;

/// A package on a runtime whose native jobs admit a foreign allowance SCIP takes as its
/// memory limit.
async fn package(text: &str) -> (ModelingPackage, DeclarationId) {
    let physical = fixture::physical();
    let rows = pse_authoring::language::parse(
        text,
        SemanticId::NIL,
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap();
    let root = rows
        .iter()
        .find(|r| r.name == "Root")
        .unwrap()
        .declaration_id;
    let package = fixture::runtime_with(16 << 20, 16 << 20, 2 << 30)
        .modeling_package(rows, physical)
        .await
        .unwrap();
    (package, root)
}
fn profile(intent: SolveIntent, selection: SolverSelection) -> SolverProfile {
    let mut profile = fixture::profile();
    profile.intent = intent;
    profile.selection = selection;
    profile
}
fn analysis(root: DeclarationId, solver: SolverProfile) -> ModelingAnalysis {
    ModelingAnalysis {
        root,
        instance: pse_modeling::specialize::root_instance(root),
        bindings: Bindings::default(),
        limits: Limits::default(),
        case: ModelingCaseBindings::default(),
        order: DerivativeOrder::First,
        compiler: fixture::compiler_profile(),
        solver,
        numerical: NumericalInputs::default(),
    }
}
async fn solve(package: &ModelingPackage, analysis: &ModelingAnalysis) -> ModelingResult {
    let cancel = crate::CancelSource::new();
    let prepared = package.prepare_analysis(analysis, &cancel).await.unwrap();
    package
        .solve_case(prepared, analysis.compiler, &cancel)
        .await
        .unwrap()
}
fn report(result: &ModelingResult, label: &str) -> f64 {
    result
        .reports
        .iter()
        .find(|r| r.label == label)
        .unwrap()
        .value
}
/// The retained global gap is in original objective units; its relative branch
/// uses the smaller same-sign magnitude, as production global qualification does.
fn objective_allowance(result: &ModelingResult, expected: f64) -> f64 {
    let Outcome::Native(native) = &result.outcome else {
        panic!("expected global native evidence")
    };
    let gap = native.evidence.global.unwrap();
    let actual = native.observation.as_ref().unwrap().objective.unwrap();
    let relative = if actual.signum() == expected.signum() {
        gap.gap_relative * actual.abs().min(expected.abs())
    } else {
        0.
    };
    let allowance = gap.gap_absolute.max(relative);
    assert!(allowance.is_finite() && allowance > 0.);
    allowance
}

/// The price-taker with a quadratic operating cost: an MIQP. Enumerating the eight
/// commitments, only h3 is worth running: its interior optimum x = 1.2/0.01 = 120 W earns
/// 144 − 72 − 50 = 22 W. Running h2 alone loses 18 W and h2 with h3 shares the 150 W
/// budget at 55 W and 95 W for −2.25 W.
const PRICE_TAKER: &str =
    include_str!("../../../../../tests/fixtures/models/native-price-taker.pse");

#[tokio::test]
async fn selected_scip_preparation_budget_refuses_resource_without_relaxing_or_rerouting() {
    use pse_model::diagnostic::BoundaryClass;
    let source = "package p { def Root { var x:Scalar; eq root:x*x==1; annotation bounds x(-2,2); annotation start x(1); } }";
    for worker_bytes in [512, 1 << 30] {
        let rows = pse_authoring::language::parse(
            source,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let runtime = fixture::runtime_on(
            2 << 30,
            crate::math::MathPolicy {
                worker_bytes,
                workspace_bytes: 1 << 30,
                foreign_bytes: 16 << 20,
                ..Default::default()
            },
        );
        let package = runtime
            .modeling_package(rows, fixture::physical())
            .await
            .unwrap();
        let mut request = analysis(
            root,
            profile(
                SolveIntent::FeasiblePoint,
                SolverSelection::Explicit(Backend::Scip),
            ),
        );
        // Artifact construction is admitted before complete attempt storage. The
        // low-capacity leg must refuse before publishing a route, while the same
        // scientific compiler policy succeeds with an ordinary resource allowance.
        request.compiler.evaluation.scratch_bytes = 512;
        let prepared = package
            .prepare_analysis(&request, &crate::CancelSource::new())
            .await;
        if worker_bytes == 512 {
            let error = prepared.unwrap_err();
            let diagnostic = error.boundary_diagnostic();
            assert_eq!(
                diagnostic.class,
                BoundaryClass::ResourceLimit,
                "{diagnostic:?}"
            );
            assert!(
                error.to_string().contains("artifact construction capacity"),
                "{error}"
            );
        } else {
            let prepared = prepared.unwrap();
            assert_eq!(
                prepared.solve.route(),
                pse_backend_native::routing::Route::Native(Backend::Scip)
            );
            assert_eq!(
                prepared.solve.route_decision().unwrap().state,
                pse_backend_native::routing::AssessmentState::Ready
            );
        }
    }
}

#[tokio::test]
async fn price_taker_quadratic_cost_miqp() {
    let (package, root) = package(PRICE_TAKER).await;
    // Only SCIP represents a mixed-integer quadratic program; HiGHS refuses the class.
    let explicit = analysis(
        root,
        profile(
            SolveIntent::Optimize,
            SolverSelection::Explicit(Backend::Highs),
        ),
    );
    assert!(
        package
            .prepare_analysis(&explicit, &crate::CancelSource::new())
            .await
            .is_err()
    );
    let mut request = analysis(root, profile(SolveIntent::Optimize, SolverSelection::Auto));
    request.bindings.demand = ["h1", "h2", "h3"]
        .into_iter()
        .flat_map(|period| {
            [
                format!("on[{period}]"),
                format!("output[{period}]"),
                format!("load[{period}]"),
            ]
        })
        .collect();
    let result = solve(&package, &request).await;
    let Outcome::Native(native) = &result.outcome else {
        panic!("{:?}", result.outcome);
    };
    assert_eq!(native.backend, Backend::Scip);
    assert!(result.accepted, "{:?}", result.validation_error);
    let model = &result.prepared.model.model.compiled().model;
    let numerics = result.prepared.solve.numerics();
    let at = |path: &str| result.values.scalars[&model.paths[path]];
    let variable_budget = |path: &str| {
        fixture::engineering_target(
            numerics,
            pse_model::generated::enums::NumericalTarget::Variable,
            model.paths[path],
        )
        .budget
    };
    let row_budget = |suffix: &str| {
        let budgets = model
            .equations
            .iter()
            .filter(|row| row.lineage.path.ends_with(suffix))
            .map(|row| {
                fixture::engineering_target(
                    numerics,
                    pse_model::generated::enums::NumericalTarget::Row,
                    row.id,
                )
                .budget
            })
            .collect::<Vec<_>>();
        assert!(!budgets.is_empty());
        assert!(budgets.iter().all(|budget| *budget == budgets[0]));
        budgets[0]
    };
    let mut margin = 0.0;
    let mut energy = 0.0;
    for (period, price, on) in [("h1", -0.5, 0.0), ("h2", 0.8, 0.0), ("h3", 1.2, 1.0)] {
        let output_path = format!("output[{period}]");
        let load_path = format!("load[{period}]");
        let output = at(&output_path);
        let load = at(&load_path);
        assert_eq!(at(&format!("on[{period}]")), on);
        assert!(
            output >= -variable_budget(&output_path)
                && output <= 150.0 + variable_budget(&output_path)
        );
        assert!(load >= -variable_budget(&load_path) && load <= 1.5 + variable_budget(&load_path));
        assert!(output <= 150.0 * on + row_budget(".most"));
        assert!(output >= 40.0 * on - row_budget(".least"));
        assert!((output - 100.0 * load).abs() <= row_budget(".loading"));
        energy += output;
        margin += price * output - 50.0 * load * load - 50.0 * on;
    }
    assert!(energy <= 150.0 + row_budget(".energy"));
    let objective_budget = objective_allowance(&result, 22.);
    assert!(
        (margin - 22.0).abs() <= objective_budget,
        "actual original margin={margin}"
    );
    assert!((report(&result, "margin") - margin).abs() <= objective_budget);
    assert_eq!(
        report(&result, "margin"),
        native.candidate.as_ref().unwrap().objective.unwrap()
    );
    assert_eq!(report(&result, "h3"), at("output[h3]"));
    assert_eq!(native.qualification, Qualification::GapQualified);
    assert_eq!(native.termination.assurance, Assurance::GlobalBound);
}

#[tokio::test]
async fn limited_native_incumbent_is_original_feasible_and_explicitly_nonoptimal() {
    use pse_model::generated::enums::{CandidateQualifier, CandidateUse, IncumbentPolicy};
    let (package, root) = package(PRICE_TAKER).await;
    let mut solver = profile(
        SolveIntent::Optimize,
        SolverSelection::Explicit(Backend::Scip),
    );
    // A deterministic solution-count limit ends native search at its first
    // feasible incumbent, before claiming an optimality or gap certificate.
    solver.controls.options.insert(
        "limits/solutions".into(),
        pse_backend_native::solve::OptionValue::Integer(1),
    );
    solver.numerics.incumbent = IncumbentPolicy::AcceptFeasible;
    let result = solve(&package, &analysis(root, solver.clone())).await;
    let Outcome::Native(native) = &result.outcome else {
        panic!("{:?}", result.outcome)
    };
    assert_eq!(
        native.termination.category,
        pse_backend_native::solve::Termination::SolutionLimit,
        "{native:?}"
    );
    assert!(native.candidate.is_some());
    assert_eq!(
        native.candidate.as_ref().unwrap().kind,
        pse_backend_native::solve::CandidateKind::FinalIterate,
        "source={:?}, quality={:?}, qualification={:?}, resolve={:?}",
        native
            .evidence
            .global
            .as_ref()
            .map(|evidence| evidence.primal),
        native.quality,
        native.qualification,
        native
            .metrics
            .iter()
            .filter(|(name, _)| name.starts_with("resolve."))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        native.evidence.global.as_ref().unwrap().primal,
        pse_backend_native::solve::PrimalSource::FixedAssignment
    );
    assert!(
        !native
            .candidate
            .as_ref()
            .unwrap()
            .commitment
            .as_ref()
            .unwrap()
            .columns
            .is_empty()
    );
    assert!(native.quality.as_ref().unwrap().feasible());
    assert_eq!(native.qualification, Qualification::Feasible);
    assert!(result.accepted, "{:?}", result.validation_error);
    assert_eq!(result.completion.decision.usability, CandidateUse::Usable);
    assert!(
        result
            .completion
            .decision
            .qualifiers
            .contains(&CandidateQualifier::AcceptedIncumbentFeasible)
    );
    assert!(result.checks.iter().all(|check| check.satisfied));
    assert_ne!(native.termination.assurance, Assurance::GlobalBound);
    assert!(
        report(&result, "margin") < 22. - objective_allowance(&result, 22.),
        "the limit incumbent unexpectedly proves the independently known optimum"
    );

    solver.numerics.incumbent = IncumbentPolicy::Refuse;
    let refused = solve(&package, &analysis(root, solver)).await;
    assert!(!refused.accepted);
    let Outcome::Native(native) = &refused.outcome else {
        panic!("{:?}", refused.outcome)
    };
    assert!(
        native.candidate.is_some(),
        "policy refusal must preserve the actual incumbent"
    );
    assert_eq!(
        native.termination.category,
        pse_backend_native::solve::Termination::SolutionLimit
    );
}

/// A nested implicit block: y solves y² = x on [0.5, 3]. The objective
/// (y − 1)²(y − 2.5)² + y/10 has two basins: its global minimum near y = 0.98 and a
/// higher one near y = 2.49.
const IMPLICIT: &str = "package p { def Root { var x: Scalar;
    implicit root select branch(y>=0) { var y: Scalar; eq residual: y*y == x; annotation bounds y(0.5, 3); annotation start y(1); }
    realize policy on root using nested;
    let f: Scalar = (root.y-1)*(root.y-1)*(root.y-2.5)*(root.y-2.5) + 0.1*root.y;
    annotation bounds x(0.25, 9); annotation start x(2);
    annotation objective f(minimize); annotation report f(\"f\"); } }";
/// The minimum of the objective over y ∈ [lower, upper], by a dense grid refined by
/// golden-section search: the independent oracle of the certified value.
fn implicit_oracle(lower: f64, upper: f64) -> f64 {
    let f = |y: f64| (y - 1.0).powi(2) * (y - 2.5).powi(2) + 0.1 * y;
    let n = 100_000;
    let step = (upper - lower) / f64::from(n);
    let best = (0..=n)
        .map(|i| lower + step * f64::from(i))
        .min_by(|a, b| f(*a).total_cmp(&f(*b)))
        .unwrap();
    let (mut a, mut b) = ((best - step).max(lower), (best + step).min(upper));
    let ratio = (5.0_f64.sqrt() - 1.0) / 2.0;
    for _ in 0..200 {
        let (c, d) = (b - ratio * (b - a), a + ratio * (b - a));
        if f(c) < f(d) {
            b = d;
        } else {
            a = c;
        }
    }
    f(0.5 * (a + b))
}

#[tokio::test]
async fn certify_exports_implicit_residuals_exactly() {
    let (package, root) = package(IMPLICIT).await;
    let result = solve(
        &package,
        &analysis(root, profile(SolveIntent::Certify, SolverSelection::Auto)),
    )
    .await;
    let Outcome::Native(native) = &result.outcome else {
        panic!("{:?}", result.outcome);
    };
    assert_eq!(native.backend, Backend::Scip);
    // The block's residual is exported in place of its nested realization, so the
    // program is exact and the certificate is a gap claim.
    assert_eq!(
        native.metrics["export.fidelity"],
        Metric::Text("exact".into())
    );
    assert_eq!(native.qualification, Qualification::GapQualified);
    assert_eq!(native.termination.assurance, Assurance::GlobalBound);
    assert!(result.accepted, "{:?}", result.validation_error);
    let global = implicit_oracle(0.5, 3.0);
    assert!(
        (report(&result, "f") - global).abs() <= objective_allowance(&result, global),
        "{} vs {global}",
        report(&result, "f")
    );
    // A case bound on the unknown replaces its bound hint in the exported interval: with
    // y ≥ 1.5 the certified minimum is the other basin's.
    let mut bounded = analysis(root, profile(SolveIntent::Certify, SolverSelection::Auto));
    bounded.case.variables.insert(
        "root.y".into(),
        pse_compiler::workspace::ModelingVariableState {
            lower: Some(Some(1.5)),
            ..Default::default()
        },
    );
    // The inner solve's deterministic start lies inside the case interval.
    bounded.case.values.insert("root.y".into(), 2.0);
    let result = solve(&package, &bounded).await;
    assert!(result.accepted, "{:?}", result.validation_error);
    let restricted = implicit_oracle(1.5, 3.0);
    assert!(restricted > global + 0.1);
    assert!(
        (report(&result, "f") - restricted).abs() <= objective_allowance(&result, restricted),
        "{} vs {restricted}",
        report(&result, "f")
    );
}

const OBSTRUCTION: &str = "package p { def Root { var x:Scalar;
    eq lo: x*x >= 4; eq hi: x*x <= 1; eq spare: x*x <= 100;
    annotation start x(1.5); annotation bounds x(-3, 3); } }";

#[tokio::test]
async fn certified_infeasibility_beside_local_explanation() {
    let (package, root) = package(OBSTRUCTION).await;
    let mut solver = profile(SolveIntent::FeasiblePoint, SolverSelection::Auto);
    solver.backend = BackendSettings::Default;
    let analysis = analysis(root, solver);
    let certificate = package
        .certify_infeasibility(&analysis, &crate::CancelSource::new())
        .await
        .unwrap();
    // A global conclusion with its own assurance; the elastic explanation stays local.
    assert!(certificate.proven());
    assert_eq!(certificate.assurance, Assurance::ProvenInfeasible);
    assert_eq!(
        certificate.fidelity,
        Some(pse_math::factorable::Fidelity::Exact)
    );
    assert!(certificate.irreducible);
    let row = |name: &str| {
        certificate
            .result
            .prepared
            .model
            .model
            .compiled()
            .model
            .equations
            .iter()
            .find(|r| r.lineage.path.ends_with(name))
            .unwrap()
            .id
    };
    let names = certificate
        .result
        .prepared
        .model
        .model
        .compiled()
        .model
        .equations
        .iter()
        .filter(|r| certificate.rows.contains(&r.id))
        .map(|r| r.lineage.path.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        certificate.rows,
        [row(".lo"), row(".hi")].into_iter().collect(),
        "{names:?} {:?}",
        certificate.members
    );
    assert!(!certificate.result.accepted);
    // A feasible model yields no proof.
    let (feasible, root) = package_feasible().await;
    let certificate = feasible
        .certify_infeasibility(
            &self::analysis(
                root,
                profile(SolveIntent::FeasiblePoint, SolverSelection::Auto),
            ),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    assert!(!certificate.proven());
    assert!(certificate.members.is_empty());
}
async fn package_feasible() -> (ModelingPackage, DeclarationId) {
    package(&OBSTRUCTION.replace("eq hi: x*x <= 1;", "")).await
}

#[tokio::test]
async fn solution_pool_published() {
    use pse_relations::{columnar::RelationRow, generated::runtime::solution_pool};
    let (package, root) = package(PRICE_TAKER).await;
    let mut solver = profile(SolveIntent::Optimize, SolverSelection::Auto);
    solver.backend = BackendSettings::Scip(ScipSettings {
        pool: 3,
        ..ScipSettings::default()
    });
    let cancel = crate::CancelSource::new();
    let prepared = package
        .prepare_analysis(&analysis(root, solver), &cancel)
        .await
        .unwrap();
    let result = prepared.start().unwrap().wait().await.unwrap();
    let table = result.table("runtime.solution_pool").unwrap();
    let rows = solution_pool::Row::rows(&table).unwrap();
    assert!(!rows.is_empty());
    // Nine free variables per ranked solution, ranks from zero, every one re-qualified.
    let ranks: std::collections::BTreeSet<_> = rows.iter().map(|r| r.rank).collect();
    assert_eq!(ranks.iter().next(), Some(&0));
    for rank in &ranks {
        let solution: Vec<_> = rows.iter().filter(|r| r.rank == *rank).collect();
        assert_eq!(solution.len(), 9);
        assert!(solution.iter().all(|r| r.feasible == Some(true)));
    }
    let best = rows.iter().find(|r| r.rank == 0).unwrap();
    let crate::workflow::RunReport::Modeling(reports) = result.report().unwrap() else {
        panic!("expected modeling report")
    };
    assert_eq!(reports.len(), 1);
    let report = &reports[0];
    assert!(report.accepted, "{:?}", report.validation_error);
    let Outcome::Native(native) = &report.outcome else {
        panic!("{:?}", report.outcome)
    };
    assert_eq!(native.qualification, Qualification::GapQualified);
    assert!((best.objective.unwrap() - 22.0).abs() <= objective_allowance(report, 22.));
    for pair in ranks.iter().collect::<Vec<_>>().windows(2) {
        let objective = |rank: i64| {
            rows.iter()
                .find(|r| r.rank == rank)
                .unwrap()
                .objective
                .unwrap()
        };
        assert!(objective(*pair[0]) >= objective(*pair[1]));
    }
}
