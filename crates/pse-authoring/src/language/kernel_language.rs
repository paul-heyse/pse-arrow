// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::ParseBudget;
use pse_ids::SemanticId;

fn parse_named(s: &str) -> Vec<Declaration> {
    parse(
        s,
        SemanticId::NIL,
        IdentityPolicy::Named,
        ParseBudget::default(),
    )
    .unwrap()
}
#[test]
fn roundtrip_all_declarations_and_explicit_identity() {
    let source = r#"package synthetic {
 use other @ "1.0.0";
 entity kind component { attribute mass: Mass; }
 entity component a { mass = 2{kg} }
 enum Choice { first, second }
 set members: Set<component> = {a};
 table coefficient[j: component]: Mass missing required;
 dataset values: coefficient source "synthetic" { [a] = [2{kg}]; }
 fn square<Q>(x: Q) -> Q^2 = x*x;
 interface I { fn f(x: Mass) -> Mass; let doubled: Mass = x+x; }
 def D(enabled: Boolean = true) : I {
 param p: Mass = 2{kg};
 var x: Mass defined by x == p;
 override let doubled: Mass = 2*x;
 alias y: Mass = x;
 child c: I = Other();
 port out: Mass = x;
 eq residual: x == p;
 when enabled { require p > 0{kg} : "positive"; }
 accumulate balance: Mass conservation tolerance 1e-9{kg};
 contribute balance role inflow = p;
 contribute balance role outflow = x;
 connect out -> c.input;
 annotation start x(p);
 scope published: Mass = p;
 implicit density { var rho: Mass; }
 domain time: Time from 0{s} to 10{s};
 collocation radau alpha(1) beta(0) right(true);
 difference backward order(1) offsets(-1,0) weights(-1,1) quadrature(0.5,0.5);
 discretize time_mesh on time using radau(elements = 5, order = 3);
 realize density_policy on density using nested;
 realize accelerated_policy on density using accelerated("cubic_roots");
 relax feasibility on residual nominal 1{kg};
 continue ramp on p from 1{kg} to 2{kg};
 stage ideal { eq start: x == p; }
 }
 preset Small = D(enabled=true);
 case run { child root = Small(); }
 test check { expect square(2) == 4 tolerance 1e-9; }
 }"#;
    let rows = parse_named(source);
    let printed = render(&rows).unwrap();
    let again = parse(
        &printed,
        SemanticId::NIL,
        IdentityPolicy::Explicit,
        ParseBudget::default(),
    )
    .unwrap();
    let semantic = |rows: Vec<Declaration>| {
        rows.into_iter()
            .map(|mut r| {
                r.source_start = 0;
                r.source_end = 0;
                r
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(semantic(rows), semantic(again));
}
#[test]
fn source_creation_and_rename_preserve_explicit_ids() {
    let source = assign_ids(
        "package p { def D { var x: Mass; } }",
        SemanticId::NIL,
        ParseBudget::default(),
    )
    .unwrap();
    let before = parse(
        &source,
        SemanticId::NIL,
        IdentityPolicy::Explicit,
        ParseBudget::default(),
    )
    .unwrap();
    let after = parse(
        &source.replace("var x:", "var renamed:"),
        SemanticId::NIL,
        IdentityPolicy::Explicit,
        ParseBudget::default(),
    )
    .unwrap();
    assert_eq!(
        before.iter().map(|r| r.declaration_id).collect::<Vec<_>>(),
        after.iter().map(|r| r.declaration_id).collect::<Vec<_>>()
    );
    assert_eq!(
        assign_ids(&source, SemanticId::NIL, ParseBudget::default()).unwrap(),
        source
    );
}
#[test]
fn errors_bounds_and_utf8_spans() {
    let text = "package p { entity kind k { attribute 'µ': Mass; } }";
    let rows = parse_named(text);
    let row = rows.last().unwrap();
    assert!(text[row.source_start as usize..row.source_end as usize].contains("µ"));
    assert!(
        parse(
            "package p {}",
            SemanticId::NIL,
            IdentityPolicy::Explicit,
            ParseBudget::default()
        )
        .is_err()
    );
    assert!(
        parse(
            "package p {",
            SemanticId::NIL,
            IdentityPolicy::Named,
            ParseBudget::default()
        )
        .is_err()
    );
    assert!(
        parse(
            "package p {}",
            SemanticId::NIL,
            IdentityPolicy::Named,
            ParseBudget {
                max_bytes: 3,
                ..ParseBudget::default()
            }
        )
        .is_err()
    );
    let long = format!("// {}\npackage p {{}}", "x".repeat(70_000));
    assert_eq!(parse_named(&long).len(), 1);
    assert!(
        parse(
            "package p { def d {} }",
            SemanticId::NIL,
            IdentityPolicy::Named,
            ParseBudget {
                max_depth: 1,
                ..ParseBudget::default()
            }
        )
        .is_err()
    );
}
#[test]
fn package_calls_and_partials_share_expression_parser() {
    for source in [
        "correlation(x)",
        "methods.correlation(x, y)",
        "partial(f, x)(a)",
        "partial(f, n[j], T)(a, b)",
        "∂(f, x)(a)",
    ] {
        let ast = crate::dsl::parse_expr(source).unwrap();
        let rendered = crate::dsl::render_expr(&ast);
        let again = crate::dsl::parse_expr(&rendered).unwrap();
        assert_eq!(crate::dsl::render_expr(&again), rendered);
    }
}

#[test]
fn named_identity_is_stable_when_a_sibling_is_inserted() {
    let a = parse_named("package p { entity kind K {} entity K a {} entity K b {} }");
    let b =
        parse_named("package p { entity kind K {} entity K extra {} entity K a {} entity K b {} }");
    for row in a {
        assert_eq!(
            row.declaration_id,
            b.iter()
                .find(|r| r.name == row.name)
                .unwrap()
                .declaration_id
        );
    }
}

#[test]
fn function_reference_types_and_partial_selectors_roundtrip() {
    let rows = parse_named(
        "package p { entity kind item {} set empty: Set<item>={}; fn apply(method: Fn(x: Scalar)->Scalar, x: Scalar)->Scalar=method(x); }",
    );
    let printed = render(&rows).unwrap();
    let reparsed = parse(
        &printed,
        SemanticId::NIL,
        IdentityPolicy::Explicit,
        ParseBudget::default(),
    )
    .unwrap();
    assert_eq!(
        rows.iter().map(|r| &r.value).collect::<Vec<_>>(),
        reparsed.iter().map(|r| &r.value).collect::<Vec<_>>()
    );
    let mut expression = crate::dsl::parse_expr("partial(f, x[j], y[k])(a,b)").unwrap();
    let names = expression
        .free_paths()
        .into_iter()
        .map(crate::dsl::render_path)
        .collect::<Vec<_>>();
    assert_eq!(
        names.into_iter().collect::<std::collections::BTreeSet<_>>(),
        ["j", "k", "a", "b"]
            .into_iter()
            .map(str::to_owned)
            .collect()
    );
    expression
        .try_walk_mut(|node| -> Result<(), std::convert::Infallible> {
            if let crate::dsl::ExprKind::Path(path) = &mut node.kind
                && path.segments[0].name == "j"
            {
                path.segments[0].name = "renamed".into();
            }
            Ok(())
        })
        .unwrap();
    assert!(crate::dsl::render_expr(&expression).contains("x[renamed]"));
}

#[test]
fn var_domain_parses_and_renders() {
    use pse_model::generated::enums::ModelingVariableDomain as Domain;
    for domain in Domain::ALL {
        let facet = if domain == Domain::Continuous {
            String::new()
        } else {
            format!(" in {}", domain.as_str())
        };
        let source = format!(
            "package domains {{ entity kind unit {{}} set units: Set<unit> = {{}}; def D {{ param p: Count = 1; var x[u in units]: Count{facet}; var y: Power; var z: Power in continuous; }} }}"
        );
        let rows = parse_named(&source);
        let binding = |name: &str| {
            rows.iter()
                .find(|r| r.name == name)
                .and_then(|r| r.value.binding.clone())
                .unwrap()
        };
        assert_eq!(binding("x").domain, Some(domain));
        assert_eq!(binding("x").indices[0].domain, "units");
        assert_eq!(
            render_type(binding("x").r#type.as_deref().unwrap()).unwrap(),
            "Count"
        );
        // Continuous is the default: the omitted and explicit spellings are one declaration.
        assert_eq!(binding("y").domain, Some(Domain::Continuous));
        assert_eq!(binding("z").domain, Some(Domain::Continuous));
        assert_eq!(binding("p").domain, None);
        let printed = render(&rows).unwrap();
        assert!(!printed.contains(" in continuous"));
        assert!(printed.contains(&facet));
        let again = parse(
            &printed,
            SemanticId::NIL,
            IdentityPolicy::Explicit,
            ParseBudget::default(),
        )
        .unwrap();
        assert_eq!(
            rows.iter().map(|r| &r.value).collect::<Vec<_>>(),
            again.iter().map(|r| &r.value).collect::<Vec<_>>()
        );
    }
    for invalid in [
        "package p { def D { var x: Count in natural; } }",
        "package p { def D { param p: Count in integer = 1; } }",
        "package p { def D { let l: Count in binary = 1; } }",
    ] {
        assert!(
            parse(
                invalid,
                SemanticId::NIL,
                IdentityPolicy::Named,
                ParseBudget::default()
            )
            .is_err(),
            "{invalid}"
        );
    }
}

#[test]
fn fixture_intent_parses_and_renders() {
    use pse_model::generated::enums::NativeSolveIntent as Intent;
    let fixture = |rows: &[Declaration]| {
        rows.iter()
            .find(|r| r.name == "t")
            .and_then(|r| r.value.scope.as_ref())
            .and_then(|s| s.fixture.clone())
            .unwrap()
    };
    let roundtrip = |rows: &[Declaration]| {
        let printed = render(rows).unwrap();
        let again = parse(
            &printed,
            SemanticId::NIL,
            IdentityPolicy::Explicit,
            ParseBudget::default(),
        )
        .unwrap();
        assert_eq!(
            rows.iter().map(|r| &r.value).collect::<Vec<_>>(),
            again.iter().map(|r| &r.value).collect::<Vec<_>>()
        );
        printed
    };
    for intent in Intent::ALL {
        let source = format!(
            "package p {{ def D {{ var x: Scalar; }} test t fixture {{ dof 0; run steady; intent {}; value root.x = 1; }} {{ child root: D = D(); }} }}",
            intent.as_str()
        );
        let rows = parse_named(&source);
        assert_eq!(fixture(&rows).intent, Some(intent));
        let printed = roundtrip(&rows);
        assert!(printed.contains(&format!("intent {};", intent.as_str())));
    }
    // Without the clause the fixture leaves the intent to the runtime policy.
    let rows = parse_named(
        "package p { def D { var x: Scalar; } test t fixture { dof 0; run steady; } { child root: D = D(); } }",
    );
    assert_eq!(fixture(&rows).intent, None);
    assert!(!roundtrip(&rows).contains("intent"));
    for invalid in [
        "package p { def D { var x: Scalar; } test t fixture { dof 0; intent prove; } { child root: D = D(); } }",
        "package p { def D { var x: Scalar; } test t fixture { dof 0; intent certify; intent optimize; } { child root: D = D(); } }",
    ] {
        assert!(
            parse(
                invalid,
                SemanticId::NIL,
                IdentityPolicy::Named,
                ParseBudget::default()
            )
            .is_err(),
            "{invalid}"
        );
    }
}

/// ADR-0119: a fixture's execution policy is typed fixture data, printed in one canonical
/// order and parsed back to the same declaration.
#[test]
fn fixture_policy_parses_and_renders() {
    use pse_model::generated::enums::{NativeBackend, PresolvePolicyKind};
    let fixture = |rows: &[Declaration]| {
        rows.iter()
            .find(|r| r.name == "t")
            .and_then(|r| r.value.scope.as_ref())
            .and_then(|s| s.fixture.clone())
            .unwrap()
    };
    let roundtrip = |rows: &[Declaration]| {
        let printed = render(rows).unwrap();
        let again = parse_named(&printed);
        assert_eq!(
            rows.iter().map(|r| &r.value).collect::<Vec<_>>(),
            again.iter().map(|r| &r.value).collect::<Vec<_>>()
        );
        printed
    };
    let source = |policy: &str| {
        format!(
            "package p {{ def D {{ var x: Scalar; }} test t fixture {{ dof 0; run steady; {policy} value root.x = 1; }} {{ child root: D = D(); }} }}"
        )
    };
    // Every setting, written out of canonical order.
    let rows = parse_named(&source(
        "policy { limits foreign_bytes(2147483648) body_slots(512) items(1000000) body_occurrences(65536); derivatives cells(8000000) tolerance(1e-4) step(1e-9); presolve off; backend ipopt; }",
    ));
    let policy = fixture(&rows).policy.unwrap();
    assert_eq!(policy.backend, Some(NativeBackend::Ipopt));
    assert_eq!(policy.presolve, Some(PresolvePolicyKind::Off));
    assert_eq!(policy.derivative_step, Some(1e-9));
    assert_eq!(policy.derivative_tolerance, Some(1e-4));
    assert_eq!(policy.derivative_cells, Some(8_000_000));
    assert_eq!(policy.items, Some(1_000_000));
    assert_eq!(policy.body_occurrences, Some(65_536));
    assert_eq!(policy.body_slots, Some(512));
    assert_eq!(policy.foreign_bytes, Some(2 << 30));
    assert!(roundtrip(&rows).contains(
        "policy { backend ipopt; presolve off; derivatives step(1e-9) tolerance(0.0001) cells(8000000); limits items(1000000) body_occurrences(65536) body_slots(512) foreign_bytes(2147483648); }"
    ));
    // One setting leaves the others to the run.
    let rows = parse_named(&source("policy { presolve auto; }"));
    let policy = fixture(&rows).policy.unwrap();
    assert_eq!(policy.presolve, Some(PresolvePolicyKind::Auto));
    assert_eq!(
        (
            policy.backend,
            policy.derivative_step,
            policy.items,
            policy.foreign_bytes
        ),
        (None, None, None, None)
    );
    assert!(roundtrip(&rows).contains("policy { presolve auto; }"));
    // Control: without the clause the fixture carries no policy and prints none.
    let rows = parse_named(&source(""));
    assert_eq!(fixture(&rows).policy, None);
    assert!(!roundtrip(&rows).contains("policy"));
}
/// An unknown, repeated or malformed fixture policy setting is refused where it is written.
#[test]
fn kernel_conformance_refuses_unknown_fixture_policy_setting() {
    for (policy, at, expected) in [
        ("policy { time_limit(600); }", "time_limit", "backend, presolve, derivatives or limits"),
        ("policy { backend ipopt; backend kinsol; }", "backend kinsol", "one backend setting"),
        ("policy { backend newton; }", "newton", "native backend"),
        ("policy { presolve explicit; }", "explicit", "auto or off"),
        ("policy { derivatives step(1e-9) step(1e-6); }", "step(1e-6)", "one step option"),
        ("policy { derivatives cells(-1); }", "-1", "nonnegative integer"),
        ("policy { derivatives step(small); }", "small", "number"),
        ("policy { derivatives; }", ";", "step, tolerance, cells"),
        ("policy { limits members(10); }", "members", "items, body_occurrences, body_slots, foreign_bytes"),
        ("policy { limits foreign_bytes(1024) foreign_bytes(2048); }", "foreign_bytes(2048)", "one foreign_bytes option"),
        ("policy { limits foreign_bytes(-1); }", "-1", "nonnegative integer"),
        ("policy { }", "}", "a fixture policy setting"),
        ("policy { presolve off; } policy { presolve auto; }", "policy { presolve auto", "one fixture policy"),
    ] {
        let text = format!(
            "package p {{ def D {{ var x: Scalar; }} test t fixture {{ dof 0; run steady; {policy} }} {{ child root: D = D(); }} }}"
        );
        let error = parse(
            &text,
            SemanticId::NIL,
            IdentityPolicy::Named,
            ParseBudget::default(),
        )
        .unwrap_err();
        let crate::AuthoringError::Syntax {
            offset,
            expected: said,
            ..
        } = &error
        else {
            panic!("{policy}: {error}");
        };
        assert_eq!(said, expected, "{policy}");
        let written = text.find("policy").unwrap() + policy.find(at).unwrap();
        assert_eq!(*offset as usize, written, "{policy}: {error}");
    }
}
#[test]
fn constraint_forms_and_disjunctions_parse_and_render() {
    use pse_model::generated::enums::{
        ModelingDeclarationKind as Kind, ModelingRealizationPolicy as Policy,
    };
    let source = r#"package forms {
 entity kind k {}
 set ks: Set<k> = {};
 def D {
  var y[i in ks]: Indicator in binary;
  var x[i in ks]: Power;
  eq on[i in ks] when y[i]: x[i] <= 10{W};
  eq off[i in ks] when not y[i]: x[i] == 0{W};
  sos1 pick[i in ks]: x[i] weight w[i];
  sos2 curve[i in ks]: x[i] weight w[i];
  atmost few[i in ks]: 2 of y[i];
  atleast some[i in ks]: 1 of y[i];
  exactly single[i in ks]: 1 of y[i];
  piecewise cost[i in ks]: c == x[i] at (bx[i], by[i]);
  logic rule: a implies (b or not c);
  disjunction route {
   alternative left { eq l: x[a] <= 1{W}; annotation bounds x(0{W}, 5{W}); }
   alternative right { disjunction inner { alternative up { eq u: x[a] >= 2{W}; } } }
  }
  realize r1 on route using bigm(derived);
  realize r2 on route using bigm(derived, 0.001);
  realize r3 on inner using bigm(100{W});
  realize r4 on route using hull;
  realize r5 on route using hull(0.0001);
  realize r6 on on using indicator;
  realize r7 on pick using native;
  realize r8 on cost using incremental;
  realize r9 on cost using sos2;
  realize r10 on rule using linear;
 }
}"#;
    let rows = parse_named(source);
    let row = |name: &str| rows.iter().find(|r| r.name == name).unwrap();
    let condition = |name: &str| row(name).value.equation.as_ref().unwrap().condition.clone();
    assert!(condition("on").is_some_and(|c| c.active && c.variable == "y[i]"));
    assert!(condition("off").is_some_and(|c| !c.active));
    assert_eq!(row("pick").value.kind, Kind::Sos1);
    assert_eq!(
        row("curve").value.ordered_set.as_ref().unwrap().weight,
        "w[i]"
    );
    assert_eq!(row("few").value.cardinality.as_ref().unwrap().count, "2");
    assert_eq!(row("single").value.kind, Kind::Exactly);
    let piecewise = row("cost").value.piecewise.as_ref().unwrap();
    assert_eq!(
        (piecewise.output.as_str(), piecewise.input.as_str()),
        ("c", "x[i]")
    );
    assert_eq!(
        row("rule").value.logic.as_ref().unwrap().proposition,
        "a implies (b or not c)"
    );
    assert_eq!(row("route").value.kind, Kind::Disjunction);
    assert_eq!(row("up").value.kind, Kind::Alternative);
    let realization = |name: &str| {
        let r = row(name).value.realization.as_ref().unwrap();
        (r.policy, r.argument.clone())
    };
    assert_eq!(realization("r1"), (Policy::DerivedBigM, None));
    assert_eq!(
        realization("r2"),
        (Policy::DerivedBigM, Some("0.001".into()))
    );
    assert_eq!(realization("r3"), (Policy::BigM, Some("100{W}".into())));
    assert_eq!(realization("r5"), (Policy::Hull, Some("0.0001".into())));
    assert_eq!(realization("r6"), (Policy::Indicator, None));
    assert_eq!(realization("r8"), (Policy::Incremental, None));
    let printed = render(&rows).unwrap();
    let again = parse(
        &printed,
        SemanticId::NIL,
        IdentityPolicy::Explicit,
        ParseBudget::default(),
    )
    .unwrap();
    assert_eq!(
        rows.iter().map(|r| &r.value).collect::<Vec<_>>(),
        again.iter().map(|r| &r.value).collect::<Vec<_>>()
    );
    for invalid in [
        "package p { def D { realize r on t using bigm; } }",
        "package p { def D { realize r on t using big_m; } }",
        "package p { def D { sos1 s: x; } }",
        "package p { def D { atmost a: 2 y; } }",
        "package p { def D { piecewise f: y == x; } }",
    ] {
        assert!(
            parse(
                invalid,
                SemanticId::NIL,
                IdentityPolicy::Named,
                ParseBudget::default()
            )
            .is_err(),
            "{invalid}"
        );
    }
}

#[test]
fn objective_members_parse_and_render() {
    use pse_model::generated::enums::NativeObjectiveSense as Sense;
    let source = r#"package p {
 def D {
 var cost: Money; var co2: Mass; var yield_: Scalar;
 annotation objective cost(minimize, relative_tolerance = 1e-3, priority = -1, weight = 0.7, normalization = 1e6{USD}, absolute_tolerance = 0.01);
 annotation objective co2(minimize, priority = -1, weight = 0.3, normalization = 1{t});
 annotation objective yield_(maximize, priority = 2);
 annotation start co2(1{t});
 }
}"#;
    let annotation = |rows: &[Declaration], kind: &str, target: &str| {
        rows.iter()
            .filter_map(|r| r.value.annotation.clone())
            .find(|a| a.kind.as_str() == kind && a.target == target)
            .unwrap()
    };
    let rows = parse_named(source);
    let cost = annotation(&rows, "objective", "cost");
    assert!(cost.arguments.is_empty());
    let members = cost.objective.unwrap();
    assert_eq!(members.sense, Sense::Minimize);
    assert_eq!(members.priority, Some(-1));
    assert_eq!(members.weight.as_deref(), Some("0.7"));
    assert_eq!(members.normalization.as_deref(), Some("1e6{USD}"));
    assert_eq!(members.absolute_tolerance.as_deref(), Some("0.01"));
    assert_eq!(members.relative_tolerance.as_deref(), Some("1e-3"));
    let yield_ = annotation(&rows, "objective", "yield_").objective.unwrap();
    assert_eq!((yield_.sense, yield_.priority), (Sense::Maximize, Some(2)));
    assert_eq!(yield_.weight, None);
    // Other annotations keep positional arguments and carry no objective members.
    let start = annotation(&rows, "start", "co2");
    assert_eq!(
        (start.arguments, start.objective),
        (vec!["1{t}".into()], None)
    );
    // The canonical rendering lists the sense, then the members in declared order, and
    // parses back to the same declarations.
    let printed = render(&rows).unwrap();
    assert!(printed.contains(
        "annotation objective cost(minimize, priority = -1, weight = 0.7, normalization = 1e6{USD}, absolute_tolerance = 0.01, relative_tolerance = 1e-3);"
    ), "{printed}");
    assert!(printed.contains("annotation objective yield_(maximize, priority = 2);"));
    let again = parse(
        &printed,
        SemanticId::NIL,
        IdentityPolicy::Explicit,
        ParseBudget::default(),
    )
    .unwrap();
    assert_eq!(
        rows.iter().map(|r| &r.value).collect::<Vec<_>>(),
        again.iter().map(|r| &r.value).collect::<Vec<_>>()
    );
    for invalid in [
        "package p { def D { annotation objective x(least); } }",
        "package p { def D { annotation objective x(minimize, priority = 1.5); } }",
        "package p { def D { annotation objective x(minimize, priority = 1, priority = 2); } }",
        "package p { def D { annotation objective x(minimize, weight = 1, weight = 2); } }",
        "package p { def D { annotation objective x(minimize, scale = 2); } }",
        "package p { def D { annotation objective x(minimize, 2); } }",
    ] {
        assert!(
            parse(
                invalid,
                SemanticId::NIL,
                IdentityPolicy::Named,
                ParseBudget::default()
            )
            .is_err(),
            "{invalid}"
        );
    }
}

#[test]
fn complements_parses_and_renders() {
    use pse_model::generated::enums::{
        ModelingDeclarationKind as Kind, ModelingRealizationPolicy as Policy,
    };
    let source = r#"package p {
 entity kind k {}
 set ks: Set<k> = {};
 def D {
 var s[j in ks]: DeltaTemperature; var f[j in ks]: Flow; var a: Scalar; var b: Scalar;
 param eps: Scalar = 1e-4;
 complements phase[j in ks]: (s[j]/1{K} >= 0, f[j]/1{mol/s} >= 0{1});
 complements pair: (a >= 0, b >= 0);
 realize smoothed on pair using smooth(math.smooth_min, eps);
 realize penalized on phase using penalty(l1);
 realize switched on pair using disjunctive;
 }
}"#;
    let rows = parse_named(source);
    let pair = rows.iter().find(|r| r.name == "pair").unwrap();
    assert_eq!(pair.value.kind, Kind::Complementarity);
    let phase = rows
        .iter()
        .find(|r| r.name == "phase")
        .and_then(|r| r.value.complementarity.clone())
        .unwrap();
    assert_eq!(
        (phase.first.as_str(), phase.second.as_str()),
        ("s[j]/1{K}", "f[j]/1{mol/s}")
    );
    assert_eq!(phase.indices[0].domain, "ks");
    let realization = |name: &str| {
        let v = rows
            .iter()
            .find(|r| r.name == name)
            .and_then(|r| r.value.realization.clone())
            .unwrap();
        (v.policy, v.function, v.argument)
    };
    assert_eq!(
        realization("smoothed"),
        (
            Policy::Smooth,
            Some("math.smooth_min".into()),
            Some("eps".into())
        )
    );
    assert_eq!(realization("penalized"), (Policy::PenaltyL1, None, None));
    assert_eq!(realization("switched"), (Policy::Disjunctive, None, None));
    let printed = render(&rows).unwrap();
    for rendered in [
        "complements pair: (a >= 0, b >= 0);",
        "complements phase[j in ks]: (s[j]/1{K} >= 0, f[j]/1{mol/s} >= 0);",
        "realize smoothed on pair using smooth(math.smooth_min, eps);",
        "realize penalized on phase using penalty(l1);",
        "realize switched on pair using disjunctive;",
    ] {
        assert!(printed.contains(rendered), "{rendered}\n{printed}");
    }
    let again = parse(
        &printed,
        SemanticId::NIL,
        IdentityPolicy::Explicit,
        ParseBudget::default(),
    )
    .unwrap();
    assert_eq!(
        rows.iter().map(|r| &r.value).collect::<Vec<_>>(),
        again.iter().map(|r| &r.value).collect::<Vec<_>>()
    );
    for invalid in [
        "package p { def D { complements c: (a >= 1, b >= 0); } }",
        "package p { def D { complements c: (a > 0, b >= 0); } }",
        "package p { def D { complements c: (a >= 0); } }",
        "package p { def D { realize r on c using smooth(eps); } }",
        "package p { def D { realize r on c using smooth; } }",
        "package p { def D { realize r on c using penalty(l2); } }",
        "package p { def D { realize r on c using penalty_l1; } }",
    ] {
        assert!(
            parse(
                invalid,
                SemanticId::NIL,
                IdentityPolicy::Named,
                ParseBudget::default()
            )
            .is_err(),
            "{invalid}"
        );
    }
}

/// ADR-0123 Outcome 1: absence is the registry missing policy; a word outside it is
/// refused where it is written, and the default policy carries its value.
#[test]
fn missing_policy_is_enum() {
    use pse_model::generated::enums::ModelingMissingPolicy as Missing;
    let table = |policy: &str| {
        let rows = parse_named(&format!(
            "package p {{ entity kind k {{}} table t[j: k]: Mass{policy}; }}"
        ));
        let table = rows
            .iter()
            .find(|r| r.name == "t")
            .and_then(|r| r.value.table.clone())
            .unwrap();
        assert_eq!(render(&rows).map(|_| ()).ok(), Some(()));
        (table.missing_policy, table.default_value)
    };
    assert_eq!(table(""), (Missing::Required, None));
    assert_eq!(table(" missing optional"), (Missing::Optional, None));
    assert_eq!(
        table(" missing default 1{kg}"),
        (Missing::Default, Some("1{kg}".into()))
    );
    assert_eq!(
        Missing::ALL.map(Missing::as_str),
        ["required", "optional", "default"]
    );
    let error = parse(
        "package p { entity kind k {} table t[j: k]: Mass missing sometimes; }",
        SemanticId::NIL,
        IdentityPolicy::Named,
        ParseBudget::default(),
    )
    .unwrap_err();
    assert!(
        matches!(&error, crate::AuthoringError::Syntax { expected, found, .. }
            if expected == "required, optional or default" && found == "sometimes"),
        "{error}"
    );
}

/// ADR-0123 Outcome 7: an import carries a typed exact requirement; ranges, prereleases
/// and words are refused where they are written.
#[test]
fn import_requirement_is_typed() {
    use pse_model::generated::enums::ModelingVersionOperator as Operator;
    for written in ["\"1.2.3\"", "\"=1.2.3\""] {
        let rows = parse_named(&format!("package p {{ use other @{written} as o; }}"));
        let import = rows
            .iter()
            .find_map(|r| r.value.import.clone())
            .unwrap();
        assert_eq!(
            (
                import.version.operator,
                import.version.major,
                import.version.minor,
                import.version.patch,
                import.alias.as_deref()
            ),
            (Operator::Exact, 1, 2, 3, Some("o"))
        );
        assert!(render(&rows).unwrap().contains("use other @ \"1.2.3\" as o;"));
    }
    assert_eq!(Operator::ALL.map(Operator::as_str), ["exact"]);
    for refused in ["\"^1.0\"", "\"1.0.0-rc.1\"", "\"latest\"", "\">=1.0.0\""] {
        let text = format!("package p {{ use other @{refused}; }}");
        let error = parse(
            &text,
            SemanticId::NIL,
            IdentityPolicy::Named,
            ParseBudget::default(),
        )
        .unwrap_err();
        assert!(
            matches!(&error, crate::AuthoringError::Syntax { expected, offset, .. }
                if expected == "exact package version" && *offset as usize == text.find(refused).unwrap()),
            "{refused}: {error}"
        );
    }
}

/// ADR-0123 Outcome 1: every annotation kind parses to its registry member, and the
/// typed members of `valid`, `scale` and `connectivity` replace their words.
#[test]
fn annotation_kinds_parse_typed_members() {
    use pse_model::generated::enums::{
        ConstraintScalingScheme, ExtrapolationPolicy, ModelingAnnotationKind as A,
    };
    let source = r#"package p {
 def D {
 var x: Mass; port o: Mass = x; eq e: x == 1{kg};
 annotation start x(1{kg});
 annotation nominal x(1{kg});
 annotation bounds x(0{kg}, 2{kg});
 annotation scale e(inverseSum);
 annotation report x(label);
 annotation valid x(0{kg}, 5{kg}, extrapolate);
 annotation check x(x > 0{kg});
 annotation objective x(minimize);
 annotation connectivity o(1, many);
 }
}"#;
    let rows = parse_named(source);
    let annotations = rows
        .iter()
        .filter_map(|r| r.value.annotation.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        annotations.iter().map(|a| a.kind).collect::<Vec<_>>(),
        A::ALL.to_vec()
    );
    let of = |kind| annotations.iter().find(|a| a.kind == kind).unwrap();
    assert_eq!(of(A::Valid).arguments, ["0{kg}", "5{kg}"]);
    assert_eq!(of(A::Valid).extrapolation, Some(ExtrapolationPolicy::Extrapolate));
    assert_eq!(
        (of(A::Scale).arguments.len(), of(A::Scale).scheme),
        (0, Some(ConstraintScalingScheme::InverseSum))
    );
    let limits = of(A::Connectivity).connectivity.clone().unwrap();
    assert_eq!((limits.incoming, limits.outgoing), (Some(1), None));
    let printed = render(&rows).unwrap();
    for spelled in [
        "annotation valid x(0{kg}, 5{kg}, extrapolate);",
        "annotation scale e(inverseSum);",
        "annotation connectivity o(1, many);",
    ] {
        assert!(printed.contains(spelled), "{spelled}\n{printed}");
    }
    assert_eq!(
        rows.iter().map(|r| &r.value).collect::<Vec<_>>(),
        parse_named(&printed).iter().map(|r| &r.value).collect::<Vec<_>>()
    );
    for invalid in [
        "package p { def D { annotation clamp x(1); } }",
        "package p { def D { annotation valid x(0, 1, clamp); } }",
        "package p { def D { annotation scale x(unknown); } }",
        "package p { def D { annotation connectivity x(1, some); } }",
        "package p { def D { annotation connectivity x(-1, many); } }",
    ] {
        assert!(
            parse(
                invalid,
                SemanticId::NIL,
                IdentityPolicy::Named,
                ParseBudget::default()
            )
            .is_err(),
            "{invalid}"
        );
    }
}
