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
        assert_eq!(binding("x").type_name, "Count");
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
