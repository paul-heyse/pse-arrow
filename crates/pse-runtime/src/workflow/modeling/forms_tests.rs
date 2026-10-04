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
use pse_model::{forms::NativeConstraint, generated::enums::NativeConstraintForm};

fn package(text: &str) -> (ModelingPackage, DeclarationId) {
    package_on(text, fixture::runtime())
}
/// A package whose native jobs admit a foreign allowance SCIP can take as its memory limit.
fn scip_package(text: &str) -> (ModelingPackage, DeclarationId) {
    package_on(text, fixture::runtime_with(16 << 20, 16 << 20, 2 << 30))
}
fn package_on(text: &str, runtime: Runtime) -> (ModelingPackage, DeclarationId) {
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
    let package = runtime.modeling_package(rows, physical).unwrap();
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
        members: BTreeMap::new(),
        values: values.iter().map(|(p, v)| ((*p).to_owned(), *v)).collect(),
        variables: BTreeMap::new(),
    }
}
async fn prepare(
    package: &ModelingPackage,
    root: DeclarationId,
    case: ModelingCaseBindings,
    selection: SolverSelection,
) -> Result<ModelingSolvePreparation, WorkflowError> {
    package
        .prepare_solve(
            root,
            pse_modeling::specialize::root_instance(root),
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
    root: DeclarationId,
    case: ModelingCaseBindings,
    report: &str,
) -> f64 {
    let [value] = optimal(package, root, case, [report]).await;
    value
}
/// Solve as [`optimum`] and return several reports, in the order requested.
async fn optimal<const N: usize>(
    package: &ModelingPackage,
    root: DeclarationId,
    case: ModelingCaseBindings,
    reports: [&str; N],
) -> [f64; N] {
    optimal_on(package, root, case, reports, Backend::Highs).await
}
/// Solve with automatic routing, require `backend` and an accepted candidate, and return
/// reports in the order requested. HiGHS solves the linear lowerings as a discrete
/// coefficient model; SCIP consumes native forms through the factorable export.
async fn optimal_on<const N: usize>(
    package: &ModelingPackage,
    root: DeclarationId,
    case: ModelingCaseBindings,
    reports: [&str; N],
    backend: Backend,
) -> [f64; N] {
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
    assert_eq!(native.backend, backend);
    if backend == Backend::Highs {
        assert!(
            native
                .evidence
                .coefficient
                .as_ref()
                .is_some_and(|c| c.discrete),
            "{:?}",
            native.evidence
        );
    } else {
        assert!(native.evidence.global.is_some(), "{:?}", native.evidence);
    }
    assert!(
        result.accepted,
        "{:?}; {:?}; {:?} {:?} {:?} {:?}",
        result.validation_error,
        result.checks,
        native.termination,
        native.qualification,
        native.quality,
        native.metrics
    );
    reports.map(|label| {
        result
            .reports
            .iter()
            .find(|r| r.label == label)
            .unwrap()
            .value
    })
}
/// The typed routing refusal of a native realization, with the forms it names.
fn native_refusal(error: &WorkflowError) -> String {
    let cause = match error {
        WorkflowError::Math(cause) => cause,
        WorkflowError::ModelingAdmission { diagnostic, cause } => {
            assert_eq!(
                diagnostic.code,
                pse_diagnostics::DiagnosticCode::NativeStructural
            );
            assert_eq!(
                diagnostic.stage,
                pse_diagnostics::DiagnosticStage::ModelingAdmission
            );
            cause.as_ref()
        }
        other => panic!("expected a typed routing refusal, got {other}"),
    };
    match cause {
        crate::math::MathRuntimeError::Solve(ProblemError::RouteRefused(decision)) => {
            let SolverSelection::Explicit(backend) = decision.selection else {
                panic!("expected an explicit backend refusal: {decision}");
            };
            assert!(decision.selected.is_none());
            assert!(matches!(
                decision.refusal.as_ref(),
                Some(pse_backend_native::routing::Refusal::Ineligible(refused)) if *refused == backend
            ));
            decision
                .eligibility
                .iter()
                .find(|assessment| assessment.backend == backend)
                .unwrap()
                .reasons
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("; ")
        }
        other => panic!("expected a preserved typed routing refusal, got {other}"),
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
async fn gdp_indicator_matches_hull() {
    let (package, root) = scip_package(&GDP.replace("REALIZATION", "indicator"));
    // Preparation records one native indicator per disjunct row, over its alternative.
    let resolution = package
        .resolve_case(
            root,
            pse_modeling::specialize::root_instance(root),
            Bindings::default(),
            Limits::default(),
            case(&[]),
            DerivativeOrder::First,
            fixture::compiler_profile(),
            profile(SolverSelection::Auto),
            NumericalInputs::default(),
            Default::default(),
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
    assert_eq!(compiled.facts.native, vec![NativeConstraintForm::Indicator]);
    // Adapters without the handler refuse the native realization, with no conversion.
    for backend in [Backend::Highs, Backend::Ipopt] {
        let error = prepare(
            &package,
            root,
            case(&[]),
            SolverSelection::Explicit(backend),
        )
        .await
        .err()
        .unwrap();
        assert!(
            native_refusal(&error).contains("native indicator realization"),
            "{error}"
        );
    }
    // Automatic routing selects SCIP, which consumes the indicators natively and reaches
    // the hull reformulation's optimum on HiGHS at every demand.
    let (hull, hull_root) = self::package(&GDP.replace("REALIZATION", "hull"));
    for demand in [40.0, 60.0, 0.0] {
        let [cost] = optimal_on(
            &package,
            root,
            case(&[("demand", demand)]),
            ["cost"],
            Backend::Scip,
        )
        .await;
        let reference = optimum(&hull, hull_root, case(&[("demand", demand)]), "cost").await;
        assert!(
            (cost - reference).abs() < 1e-6,
            "{demand}: {cost} vs {reference}"
        );
        assert!((cost - gdp_oracle(demand)).abs() < 1e-6, "{demand}: {cost}");
    }
}

/// Value-only study points share one prepared view (A6). A derived big-M is a value: a
/// point that changes a parameter its enclosure consumed recomputes it with the products
/// that depend on it, and never prepares the structure again (ADR-0104).
#[tokio::test]
async fn derived_big_m_follows_value_only_bindings() {
    let (package, root) = package(
        &GDP.replace("REALIZATION", "bigm(derived)")
            .replace(
                "var output: Power;",
                "param limit: Power = 50{W}; var output: Power;",
            )
            .replace(
                "alternative small { eq capacity: output <= 50{W};",
                "alternative small { eq capacity: output <= limit;",
            ),
    );
    let analysis = ModelingAnalysis {
        root,
        instance: pse_modeling::specialize::root_instance(root),
        bindings: Bindings::default(),
        limits: Limits::default(),
        case: case(&[("demand", 60.0)]),
        order: DerivativeOrder::First,
        compiler: fixture::compiler_profile(),
        solver: profile(SolverSelection::Auto),
        numerical: NumericalInputs::default(),
    };
    let limits = [50.0, 70.0, 90.0];
    let cancel = crate::CancelSource::new();
    let service = package.runtime.native().clone();
    let before = service.preparations();
    let mut staged = crate::workflow::staged::Staged::open(&package.runtime, None).unwrap();
    let mut results = Vec::new();
    for (index, limit) in limits.iter().enumerate() {
        let mut analysis = analysis.clone();
        analysis.case.values.insert("limit".into(), *limit);
        let preparation = package.prepare_analysis(&analysis, &cancel).await.unwrap();
        results.push(
            staged
                .run(
                    preparation,
                    assessment::Obligations::Final,
                    pse_operations::mint_id(),
                    index,
                    None,
                    &cancel,
                )
                .await
                .unwrap(),
        );
    }
    staged.close().await;
    let end = service.preparations();
    for (limit, result) in limits.into_iter().zip(&results) {
        assert!(result.accepted, "{limit}: {:?}", result.diagnostic());
        let cost = result.reports.iter().find(|r| r.label == "cost").unwrap();
        // The small unit serves 60 W only when its limit admits it.
        let expected = if limit >= 60.0 { 22.0 } else { 36.0 };
        assert!(
            (cost.value - expected).abs() < 1e-6,
            "{limit}: {}",
            cost.value
        );
        // Each alternative's capacity M is sup(output − capacity) over output ∈ [0, 150 W],
        // widened outward; the small unit's follows the point's limit.
        let case = &result.prepared.model;
        let derived = &case.case.compiled().derived.values;
        let capacities = case
            .model
            .compiled()
            .model
            .symbols
            .values()
            .filter(|s| s.lineage.path.ends_with("capacity.big_m_upper"))
            .map(|s| {
                assert_eq!(case.values.scalars[&s.id], derived[&s.id]);
                derived[&s.id]
            })
            .collect::<Vec<_>>();
        assert_eq!(capacities.len(), 3);
        assert!(
            capacities
                .iter()
                .any(|m| (m - (150.0 - limit) * (1.0 + 1e-6)).abs() < 1e-9),
            "{limit}: {capacities:?}"
        );
    }
    assert_eq!(end.views - before.views, 1);
    assert_eq!(end.rebuilt - before.rebuilt, 2);
}

const INDICATOR: &str =
    "package p { def Root { param charge: Power = 50{W}; var on: Indicator in binary; var x: Power;
    eq cap when on: x <= 80{W};
    annotation start on(0{1});
    eq off when not on: x <= 0{W};
    REALIZE
    let net: Power = x - charge*on;
    annotation bounds x(0{W}, 100{W}); annotation start x(0{W});
    annotation objective net(maximize); annotation report net(\"net\"); annotation report on(\"on\"); } }";

#[tokio::test]
async fn indicator_linear_lowering_matches_native() {
    // Enumerated oracle: fixing the indicator leaves a linear program per branch. Running
    // the unit is optimal below an 80 W charge and idling above it; the idle charge guards
    // the MIP whose presolve HiGHS 1.14.3 solved as running (net −10 W, zero gap).
    let (linear, root) = package(&INDICATOR.replace("REALIZE", ""));
    let linear_root = root;
    for charge in [20.0, 50.0, 90.0] {
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
        let [net, on] = optimal(&linear, root, case(&[("charge", charge)]), ["net", "on"]).await;
        let best = branches.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        assert!((net - best).abs() < 1e-6, "{charge}: {net} vs {best}");
        let running = if charge < 80.0 { 1.0 } else { 0.0 };
        assert!((on - running).abs() < 1e-6, "{charge}: on = {on}");
    }
    // The native realization keeps the authored rows with their indicator metadata.
    let (native, root) = scip_package(&INDICATOR.replace(
        "REALIZE",
        "realize r1 on cap using indicator; realize r2 on off using indicator;",
    ));
    let resolution = native
        .resolve_case(
            root,
            pse_modeling::specialize::root_instance(root),
            Bindings::default(),
            Limits::default(),
            case(&[]),
            DerivativeOrder::First,
            fixture::compiler_profile(),
            profile(SolverSelection::Auto),
            NumericalInputs::default(),
            Default::default(),
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
    // Both conditional rows are the authored relations, unrelaxed. HiGHS refuses them;
    // automatic routing hands them to SCIP's indicator handler, which agrees with the
    // linear lowering at every charge.
    assert_eq!(compiled.plan.structure().rows().len(), 2);
    let error = prepare(
        &native,
        root,
        case(&[]),
        SolverSelection::Explicit(Backend::Highs),
    )
    .await
    .err()
    .unwrap();
    assert!(native_refusal(&error).contains("indicator"), "{error}");
    for charge in [20.0, 50.0, 90.0] {
        let [net, on] = optimal_on(
            &native,
            root,
            case(&[("charge", charge)]),
            ["net", "on"],
            Backend::Scip,
        )
        .await;
        let [lowered, lowered_on] = optimal(
            &linear,
            linear_root,
            case(&[("charge", charge)]),
            ["net", "on"],
        )
        .await;
        assert!((net - lowered).abs() < 1e-6, "{charge}: {net} vs {lowered}");
        assert!((on - lowered_on).abs() < 1e-6, "{charge}: on = {on}");
    }
}

const PIECEWISE: &str = "package p { entity kind source provenance { attribute title: Text; } enum role { given } entity source s { title = \"synthetic test data\" }
    entity kind knot {} entity knot k0 {} entity knot k1 {} entity knot k2 {} entity knot k3 {}
    set knots: Set<knot> = {k0, k1, k2, k3};
    table points[k: knot]: {x: Power, y: Power} complete_over(k in knots);
    dataset curve_data: points provenance(s, role.given) {
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

const SOS: &str = "package p { entity kind source provenance { attribute title: Text; } enum role { given } entity source s { title = \"synthetic test data\" }
    entity kind item {} entity item a {} entity item b {} entity item c {}
    set items: Set<item> = {a, b, c};
    table order[i: item]: Scalar complete_over(i in items);
    dataset ranks: order provenance(s, role.given) { [a] = [1]; [b] = [2]; [c] = [3]; }
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
    let (native, root) = scip_package(&SOS.replace("REALIZE", "realize r on pick using native;"));
    let error = prepare(
        &native,
        root,
        case(&[]),
        SolverSelection::Explicit(Backend::Highs),
    )
    .await
    .err()
    .unwrap();
    assert!(
        native_refusal(&error).contains("native sos1 realization"),
        "{error}"
    );
    // Automatic routing selects SCIP's SOS1 handler, which agrees with the lowering.
    let [native_value] = optimal_on(&native, root, case(&[]), ["value"], Backend::Scip).await;
    assert!((native_value - value).abs() < 1e-6, "{native_value}");
}

#[tokio::test]
async fn indicator_realization_requires_native_backend() {
    // The native realization keeps its rows for SCIP's indicator handler, which HiGHS
    // lacks: an explicit HiGHS solve is refused before any native work.
    let (native, root) = scip_package(&INDICATOR.replace(
        "REALIZE",
        "realize r1 on cap using indicator; realize r2 on off using indicator;",
    ));
    let error = prepare(
        &native,
        root,
        case(&[]),
        SolverSelection::Explicit(Backend::Highs),
    )
    .await
    .err()
    .unwrap();
    assert!(
        native_refusal(&error).contains("native indicator realization"),
        "{error}"
    );
}

#[tokio::test]
async fn authored_gdp_fixture_selects_the_enumerated_alternative() {
    use pse_model::generated::enums::ModelingConformanceStatus as Status;
    // The fixture's oracle is declared by the seed's references and the domain's
    // provenance module (ADR-0123 Outcome 5).
    let rows = [
        include_str!("../../../../../packages/reference/seed-data/models/gdp.pse"),
        include_str!("../../../../../packages/reference/data/references/models/references.pse"),
        include_str!("../../../../../packages/reference/domain/models/provenance.pse"),
        include_str!("../../../../../packages/reference/domain/models/properties.pse"),
        include_str!("../../../../../packages/reference/domain/models/constants.pse"),
        include_str!("../../../../../packages/reference/physical/models/chemistry.pse"),
        include_str!("../../../../../packages/reference/physical/models/compatibility.pse"),
    ]
    .into_iter()
    .flat_map(|text| {
        pse_authoring::language::parse(
            text,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Explicit,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap()
    })
    .collect::<Vec<_>>();
    let physical = fixture::physical();
    // The full imported schema closure coexists with source indexing and checked
    // declarations during admission; this is larger than the local toy fixtures.
    let package = fixture::runtime_with_workspace(64 << 20)
        .modeling_package(rows, physical)
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
        maximum_fixtures: 10,
        maximum_checks: 50,
        fixtures: Default::default(),
        diagnostics: None,
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

/// The authored CHKS smoothing function the reference `math` package declares.
const SMOOTH_MIN: &str = "fn smooth_min<Q>(a: Q, b: Q, eps: Delta<Q>) -> Q valid(eps > eps - eps) = a - eps*(d + sqrt(d*d + 1))/2 where d = (a - b)/eps;";

#[tokio::test]
async fn smooth_complementarity_product_equals_eps_sq_over_4() {
    // 0 <= a ⊥ b >= 0 smoothed by CHKS: at a fixed a the solve finds a·b = eps²/4, and the
    // width is a value, so continuing it rebinds the prepared structure.
    let (package, root) = package(&format!(
        "package p {{ {SMOOTH_MIN} def Root {{ param eps: Scalar = 0.1; param a0: Scalar = 0.2; var a: Scalar; var b: Scalar; eq pin: a == a0;
        complements c: (a >= 0, b >= 0); realize r on c using smooth(smooth_min, eps);
        annotation bounds b(0, 10); annotation start a(0.2); annotation start b(1);
        annotation objective b(minimize); annotation report a(\"a\"); annotation report b(\"b\"); }} }}"
    ));
    let mut structures = std::collections::BTreeSet::new();
    for eps in [0.1, 0.02, 0.004] {
        let prepared = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                case(&[("eps", eps)]),
                DerivativeOrder::Second,
                fixture::compiler_profile(),
                profile(SolverSelection::Explicit(Backend::Ipopt)),
                NumericalInputs::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        structures.insert(prepared.model.case.compiled().plan.structure().key());
        let result = package
            .solve_case(
                prepared,
                fixture::compiler_profile(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        assert!(result.accepted, "{:?}", result.validation_error);
        let report = |label: &str| {
            result
                .reports
                .iter()
                .find(|r| r.label == label)
                .unwrap()
                .value
        };
        let product = report("a") * report("b");
        assert!(
            (product - eps * eps / 4.0).abs() < 1e-9,
            "eps={eps}: a·b = {product}"
        );
    }
    assert_eq!(structures.len(), 1, "the width is continued by value");
}

#[tokio::test]
async fn disjunctive_complementarity_refused_on_highs() {
    // max a + b over a <= 0.6, b <= 0.7, a + b <= 1 with 0 <= a ⊥ b >= 0: the
    // complementarity leaves one member nonzero, so the optimum is b = 0.7.
    let text = "package p { def Root { var a: Scalar; var b: Scalar; let value: Scalar = a + b;
        eq total: a + b <= 1; complements c: (a >= 0, b >= 0); realize r on c using disjunctive;
        annotation bounds a(0, 0.6); annotation bounds b(0, 0.7); annotation start a(0); annotation start b(0);
        annotation objective value(maximize); annotation report a(\"a\"); annotation report b(\"b\"); } }";
    let (native, root) = scip_package(text);
    let error = prepare(
        &native,
        root,
        case(&[]),
        SolverSelection::Explicit(Backend::Highs),
    )
    .await
    .err()
    .unwrap();
    assert!(
        native_refusal(&error).contains("native sos1 realization"),
        "{error}"
    );
    // Automatic routing selects SCIP's SOS1 handler.
    let [a, b] = optimal_on(&native, root, case(&[]), ["a", "b"], Backend::Scip).await;
    assert!(a.abs() < 1e-6 && (b - 0.7).abs() < 1e-6, "a={a} b={b}");
}

/// ADR-0104 §5, ADR-0109 and finding T14: an authored `penalty(l1)` realization is the
/// explicit selection of POUNCE's l1 exact penalty. Automatic routing reaches POUNCE with
/// that method because the author selected it, the result records the selection and that
/// the penalty covers every row, and a backend or method that cannot honour the
/// requirement is refused with the typed `method` reason.
#[cfg(feature = "solver-pounce")]
#[tokio::test]
async fn authored_l1_realization_selects_route() {
    use pse_backend_native::{
        execution::BackendSettings,
        routing::{Ineligible, Route},
        settings::pounce::{Method, Settings},
        solve::OptionValue,
    };
    let (package, root) = package(
        "package p { def Root { param a0: Scalar = 0.2; var a: Scalar; var b: Scalar; eq pin: a == a0;
        complements c: (a >= 0, b >= 0); realize r on c using penalty(l1);
        annotation bounds b(0, 10); annotation start a(0.2); annotation start b(1);
        annotation objective b(minimize); annotation report a(\"a\"); annotation report b(\"b\"); } }",
    );
    // POUNCE takes second derivatives.
    let cancel = crate::CancelSource::new();
    let second = |profile| {
        package.prepare_solve(
            root,
            pse_modeling::specialize::root_instance(root),
            Bindings::default(),
            Limits::default(),
            case(&[]),
            DerivativeOrder::Second,
            fixture::compiler_profile(),
            profile,
            NumericalInputs::default(),
            &cancel,
        )
    };
    let prepared = second(profile(SolverSelection::Auto)).await.unwrap();
    assert_eq!(prepared.solve.route(), Route::Native(Backend::Pounce));
    for e in prepared.solve.eligibility() {
        let method = e
            .reasons
            .iter()
            .any(|r| matches!(r, Ineligible::Method { .. }));
        assert_eq!(method, e.backend != Backend::Pounce, "{e}");
    }
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
    assert_eq!(native.backend, Backend::Pounce);
    assert_eq!(
        native.options["l1_exact_penalty_barrier"],
        OptionValue::Bool(true)
    );
    assert!(
        native.provenance["method.selection"].contains("penalty(l1)"),
        "{:?}",
        native.provenance
    );
    assert!(native.provenance["method.penalty_scope"].contains("every constraint row"));
    assert!(result.accepted, "{:?}", result.validation_error);
    let report = |label: &str| {
        result
            .reports
            .iter()
            .find(|r| r.label == label)
            .unwrap()
            .value
    };
    assert!((report("a") - 0.2).abs() < 1e-6, "a = {}", report("a"));
    assert!(report("b").abs() < 1e-6, "b = {}", report("b"));
    // A backend, or a POUNCE method, that cannot honour the requirement is refused.
    let error = second(profile(SolverSelection::Explicit(Backend::Ipopt)))
        .await
        .err()
        .unwrap();
    assert!(
        native_refusal(&error).contains("l1_exact_penalty"),
        "{error}"
    );
    let mut interior = profile(SolverSelection::Explicit(Backend::Pounce));
    interior.backend = BackendSettings::Pounce(Settings {
        method: Method::InteriorPoint,
        ..Settings::default()
    });
    let error = second(interior).await.err().unwrap();
    assert!(
        native_refusal(&error).contains("l1_exact_penalty"),
        "{error}"
    );
}
