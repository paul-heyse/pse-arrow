// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Authored constraint forms and disjunctions through the real pipeline to HiGHS (ADR-0104).
use super::*;
use crate::math::solves::{NumericalInputs, Outcome};
use crate::workflow::tests as fixture;
use pse_backend_native::{
    ProblemError,
    solve::{Backend, SolveIntent, SolverSelection},
};
use pse_compiler::workspace::ModelingCaseBindings;
use pse_kernels::DerivativeOrder;
use pse_model::forms::{NativeConstraint, NativeForm};

fn package(text: &str) -> (ModelingPackage, SemanticId) {
    let physical = fixture::physical();
    let mut names = fixture::discrete_names();
    names.insert(
        "Scalar".into(),
        physical.quantities.neutral_dimensionless().unwrap(),
    );
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
    let package = fixture::runtime()
        .modeling_package(rows, physical, names)
        .unwrap();
    (package, root)
}
fn profile(selection: SolverSelection) -> crate::math::solves::SolverProfile {
    let mut profile = fixture::profile();
    profile.intent = SolveIntent::Optimize;
    profile.selection = selection;
    profile
}
fn case(values: &[(&str, f64)]) -> ModelingCaseBindings {
    ModelingCaseBindings {
        values: values.iter().map(|(p, v)| ((*p).to_owned(), *v)).collect(),
        variables: BTreeMap::new(),
    }
}
async fn prepare(
    package: &ModelingPackage,
    root: SemanticId,
    case: ModelingCaseBindings,
    selection: SolverSelection,
) -> Result<super::super::ModelingSolvePreparation, WorkflowError> {
    package
        .prepare_solve(
            root,
            root,
            Bindings::default(),
            Limits::default(),
            case,
            DerivativeOrder::First,
            fixture::compiler_profile(),
            profile(selection),
            NumericalInputs::default(),
            &crate::CancelSource::new(),
        )
        .await
}
/// Solve with automatic routing, require HiGHS and an accepted candidate, and return a report.
async fn optimum(
    package: &ModelingPackage,
    root: SemanticId,
    case: ModelingCaseBindings,
    report: &str,
) -> f64 {
    let prepared = prepare(package, root, case, SolverSelection::Auto)
        .await
        .unwrap();
    let result = package
        .solve_case(
            prepared,
            fixture::compiler_profile(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let Outcome::Native(native) = &result.outcome else {
        panic!("expected a native outcome: {:?}", result.outcome);
    };
    assert_eq!(native.backend, Backend::Highs);
    assert!(
        native
            .evidence
            .coefficient
            .as_ref()
            .is_some_and(|c| c.discrete),
        "{:?}",
        native.evidence
    );
    assert!(
        result.accepted,
        "{:?}; {:?}",
        result.validation_error, result.checks
    );
    result
        .reports
        .iter()
        .find(|r| r.label == report)
        .unwrap()
        .value
}
/// The typed routing refusal of a native realization, with the forms it names.
fn native_refusal(error: &WorkflowError) -> String {
    match error {
        WorkflowError::Math(crate::math::MathRuntimeError::Solve(ProblemError::Unsupported(
            message,
        ))) => message.clone(),
        other => panic!("expected a typed unsupported refusal, got {other:?}"),
    }
}

const GDP: &str =
    "package p { def Root { param demand: Power = 40{W}; var output: Power; var cost: Power;
    eq need: output >= demand;
    disjunction route {
      alternative small { eq capacity: output <= 50{W}; eq price: cost == 10{W} + 0.2*output; }
      alternative large { eq capacity: output <= 120{W}; eq price: cost == 30{W} + 0.1*output; }
      alternative idle { eq capacity: output <= 0{W}; eq price: cost == 0{W}; }
    }
    realize how on route using REALIZATION;
    annotation bounds output(0{W}, 150{W}); annotation bounds cost(0{W}, 100{W});
    annotation start output(0{W}); annotation start cost(0{W});
    annotation objective cost(minimize); annotation report cost(\"cost\"); } }";
/// Enumerated oracle: the cheapest feasible alternative at the demand.
fn gdp_oracle(demand: f64) -> f64 {
    [(50.0, 10.0, 0.2), (120.0, 30.0, 0.1), (0.0, 0.0, 0.0)]
        .into_iter()
        .filter(|(cap, _, _)| demand <= *cap)
        .map(|(_, fixed, slope)| fixed + slope * demand)
        .fold(f64::INFINITY, f64::min)
}

#[tokio::test]
async fn gdp_hull_and_bigm_same_optimum() {
    for realization in ["hull", "bigm(derived)", "bigm(500{W})", "hull(0.0001)"] {
        let (package, root) = package(&GDP.replace("REALIZATION", realization));
        for demand in [40.0, 60.0, 0.0] {
            let cost = optimum(&package, root, case(&[("demand", demand)]), "cost").await;
            assert!(
                (cost - gdp_oracle(demand)).abs() < 1e-6,
                "{realization} at {demand}: {cost}"
            );
        }
    }
}

#[tokio::test]
async fn indicator_realization_requires_native_backend() {
    let (package, root) = package(&GDP.replace("REALIZATION", "indicator"));
    // Preparation records one native indicator per disjunct row, over its alternative.
    let resolution = package
        .resolve_case(
            root,
            root,
            Bindings::default(),
            Limits::default(),
            case(&[]),
            DerivativeOrder::First,
            fixture::compiler_profile(),
            profile(SolverSelection::Auto),
            NumericalInputs::default(),
            BTreeMap::new(),
            BTreeMap::new(),
            false,
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let compiled = resolution.model.case.compiled();
    let native = compiled.plan.structure().native();
    assert_eq!(native.len(), 6);
    assert!(
        native
            .iter()
            .all(|c| matches!(c, NativeConstraint::Indicator { active: true, .. }))
    );
    assert_eq!(compiled.facts.native, vec![NativeForm::Indicator]);
    for selection in [
        SolverSelection::Auto,
        SolverSelection::Explicit(Backend::Highs),
        SolverSelection::Explicit(Backend::Ipopt),
    ] {
        let error = prepare(&package, root, case(&[]), selection)
            .await
            .err()
            .unwrap();
        assert!(
            native_refusal(&error).contains("native indicator realization"),
            "{error}"
        );
    }
}

const INDICATOR: &str =
    "package p { def Root { param charge: Power = 50{W}; var on: Indicator in binary; var x: Power;
    eq cap when on: x <= 80{W};
    annotation start on(0{1});
    eq off when not on: x <= 0{W};
    REALIZE
    let net: Power = x - charge*on;
    annotation bounds x(0{W}, 100{W}); annotation start x(0{W});
    annotation objective net(maximize); annotation report net(\"net\"); } }";

#[tokio::test]
async fn indicator_linear_lowering_matches_native() {
    // Enumerated oracle: fixing the indicator leaves a linear program per branch.
    let (linear, root) = package(&INDICATOR.replace("REALIZE", ""));
    // A charge above 80 W, where idling is optimal, is not compared: the pinned HiGHS MIP
    // presolve reports the unit on (net -10 W, zero gap) for that instance, and the same
    // model solves correctly with presolve off. The defect is upstream of the lowering.
    for charge in [20.0, 50.0] {
        let mut branches = Vec::new();
        for on in [0.0, 1.0] {
            let mut fixed = case(&[("charge", charge), ("on", on)]);
            fixed.variables.insert(
                "on".into(),
                pse_compiler::workspace::ModelingVariableState {
                    fixed: Some(true),
                    ..Default::default()
                },
            );
            let prepared = prepare(&linear, root, fixed, SolverSelection::Auto)
                .await
                .unwrap();
            let result = linear
                .solve_case(
                    prepared,
                    fixture::compiler_profile(),
                    &crate::CancelSource::new(),
                )
                .await
                .unwrap();
            assert!(result.accepted, "{:?}", result.validation_error);
            branches.push(
                result
                    .reports
                    .iter()
                    .find(|r| r.label == "net")
                    .unwrap()
                    .value,
            );
        }
        // Off forces x to zero; on admits x up to 80 W.
        assert!(branches[0].abs() < 1e-6 && (branches[1] - (80.0 - charge)).abs() < 1e-6);
        let net = optimum(&linear, root, case(&[("charge", charge)]), "net").await;
        let best = branches.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        assert!((net - best).abs() < 1e-6, "{charge}: {net} vs {best}");
    }
    // The native realization keeps the authored rows with their indicator metadata.
    let (native, root) = package(&INDICATOR.replace(
        "REALIZE",
        "realize r1 on cap using indicator; realize r2 on off using indicator;",
    ));
    let resolution = native
        .resolve_case(
            root,
            root,
            Bindings::default(),
            Limits::default(),
            case(&[]),
            DerivativeOrder::First,
            fixture::compiler_profile(),
            profile(SolverSelection::Auto),
            NumericalInputs::default(),
            BTreeMap::new(),
            BTreeMap::new(),
            false,
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let compiled = resolution.model.case.compiled();
    let actives = compiled
        .plan
        .structure()
        .native()
        .iter()
        .map(|c| match c {
            NativeConstraint::Indicator { active, .. } => *active,
            other => panic!("{other:?}"),
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(actives, [false, true].into_iter().collect());
    // Both conditional rows are the authored relations, unrelaxed.
    assert_eq!(compiled.plan.structure().rows().len(), 2);
    let error = prepare(&native, root, case(&[]), SolverSelection::Auto)
        .await
        .err()
        .unwrap();
    assert!(native_refusal(&error).contains("indicator"), "{error}");
}

const PIECEWISE: &str = "package p {
    entity kind knot {} entity knot k0 {} entity knot k1 {} entity knot k2 {} entity knot k3 {}
    set knots: Set<knot> = {k0, k1, k2, k3};
    table points[k: knot]: {x: Power, y: Power};
    dataset curve_data: points source \"synthetic nonconvex curve\" {
      [k0] = [0{W}, 0{W}]; [k1] = [10{W}, 20{W}]; [k2] = [20{W}, 5{W}]; [k3] = [30{W}, 25{W}]; }
    def Root { param at: Power = 20{W}; var x: Power; var y: Power;
      piecewise curve[k in knots]: y == x at (points[k].x, points[k].y);
      REALIZE
      eq pin: x == at;
      annotation bounds x(0{W}, 30{W}); annotation bounds y(0{W}, 30{W});
      annotation start x(0{W}); annotation start y(0{W});
      annotation objective y(maximize); annotation report y(\"y\"); } }";
/// Linear interpolation over the breakpoints; the convex relaxation would exceed it.
fn interpolate(x: f64) -> f64 {
    let points = [(0.0, 0.0), (10.0, 20.0), (20.0, 5.0), (30.0, 25.0)];
    points
        .windows(2)
        .find(|w| x >= w[0].0 && x <= w[1].0)
        .map(|w| w[0].1 + (w[1].1 - w[0].1) * (x - w[0].0) / (w[1].0 - w[0].0))
        .unwrap()
}

#[tokio::test]
async fn piecewise_sos2_matches_incremental() {
    for realize in ["", "realize r on curve using incremental;"] {
        let (package, root) = package(&PIECEWISE.replace("REALIZE", realize));
        for at in [20.0, 25.0, 7.5] {
            let y = optimum(&package, root, case(&[("at", at)]), "y").await;
            assert!(
                (y - interpolate(at)).abs() < 1e-6,
                "{realize:?} at {at}: {y}"
            );
        }
    }
}

const SOS: &str = "package p {
    entity kind item {} entity item a {} entity item b {} entity item c {}
    set items: Set<item> = {a, b, c};
    table order[i: item]: Scalar;
    dataset ranks: order source \"synthetic\" { [a] = [1]; [b] = [2]; [c] = [3]; }
    def Root { var flow[i in items]: Power;
      sos1 pick[i in items]: flow[i] weight order[i];
      REALIZE
      eq supply: flow[a] + flow[b] + flow[c] <= 10{W};
      let value: Power = flow[a] + 2*flow[b] + 3*flow[c];
      annotation bounds flow(0{W}, 4{W}); annotation start flow(0{W});
      annotation objective value(maximize); annotation report value(\"value\"); } }";

#[tokio::test]
async fn native_only_realization_refused_on_highs() {
    // The linear SOS1 lowering admits one nonzero member: the best is 3 × 4 W.
    let (linear, root) = package(&SOS.replace("REALIZE", ""));
    let value = optimum(&linear, root, case(&[]), "value").await;
    assert!((value - 12.0).abs() < 1e-6, "{value}");
    let (native, root) = package(&SOS.replace("REALIZE", "realize r on pick using native;"));
    for selection in [
        SolverSelection::Auto,
        SolverSelection::Explicit(Backend::Highs),
    ] {
        let error = prepare(&native, root, case(&[]), selection)
            .await
            .err()
            .unwrap();
        assert!(
            native_refusal(&error).contains("native sos1 realization"),
            "{error}"
        );
    }
}

#[tokio::test]
async fn authored_gdp_fixture_selects_the_enumerated_alternative() {
    use pse_model::generated::enums::ModelingConformanceStatus as Status;
    let text = include_str!("../../../../../packages/reference/seed-data/models/gdp.pse");
    let rows = pse_authoring::language::parse(
        text,
        SemanticId::NIL,
        pse_authoring::language::IdentityPolicy::Explicit,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap();
    let physical = fixture::physical();
    let mut names = fixture::discrete_names();
    names.insert(
        "Scalar".into(),
        physical.quantities.neutral_dimensionless().unwrap(),
    );
    let package = fixture::runtime()
        .modeling_package(rows, physical, names)
        .unwrap();
    let mut policy = ModelingConformancePolicy {
        compiler: fixture::compiler_profile(),
        solver: profile(SolverSelection::Auto),
        numerical: NumericalInputs::default(),
        limits: Limits::default(),
        derivatives: pse_backend_native::derivative_diagnostics::Policy {
            perturbation: 1e-6,
            relative_tolerance: 1e-4,
            maximum_cells: 100,
        },
        fixture_policies: BTreeMap::new(),
        maximum_fixtures: 10,
        maximum_checks: 50,
    };
    policy.solver.intent = SolveIntent::Optimize;
    let report = package
        .conform(policy, &crate::CancelSource::new())
        .await
        .unwrap();
    assert!(report.passed(), "{:?}", report.checks);
    assert!(
        report
            .fixture_statuses
            .values()
            .all(|s| *s == Status::Passed)
    );
    let result = report.results.values().next().unwrap();
    let Outcome::Native(native) = &result.outcome else {
        panic!("expected a native GDP outcome");
    };
    assert_eq!(native.backend, Backend::Highs);
    // The case keeps the lowering record of its declared hull.
    assert!(
        result
            .prepared
            .model
            .model
            .compiled()
            .model
            .lowerings
            .iter()
            .any(|l| l.realization == pse_model::generated::enums::ModelingRealizationPolicy::Hull)
    );
}
