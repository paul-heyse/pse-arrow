// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Blueprint syntax examples and negative grammar/resource controls.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "fixture failures retain the precise parser error"
)]

use pse_authoring::dsl::{
    BinaryOp, DslError, ExprKind, parse_equation, parse_expr, parse_predicate, render_equation,
    render_expr, render_predicate,
};

#[test]
fn blueprint_expressions_preserve_their_actual_structure() {
    let mut snapshots = Vec::new();
    for text in [
        "A + B*t + C*t^2 + D*t^3 + E/t^2 where t = T/1000{K}",
        "arrhenius_const * exp(-energy_activation / (R * state.temperature))",
        "sum(p in phase | control_volume.properties_out[t].enth_flow_phase[p])",
        "prod(j in species where j != excluded | composition[j]^order[j])",
        "integral(x in length | d(temperature[x])/dx)",
        "if compressor then work*efficiency else work/efficiency",
        "kernel.cubic.compress_fact(A, B, eos_type)",
        "2{mol/(m^3*s)} * concentration[t, species.CO2]",
        "(-x)^2",
        "-(x^2)",
        "x^y^z",
        "x^-2",
        "min(a, b)",
        "max(a, b)",
    ] {
        let ast = parse_expr(text).unwrap_or_else(|error| panic!("{text}: {error}"));
        let rendered = render_expr(&ast);
        let reparsed = parse_expr(&rendered).unwrap_or_else(|error| panic!("{rendered}: {error}"));
        assert!(ast.structural_eq(&reparsed), "{text} -> {rendered}");
        assert_eq!(ast.span.start, 0);
        assert_eq!(usize::try_from(ast.span.end).unwrap(), text.len());
        snapshots.push((text, ast));
    }
    insta::assert_debug_snapshot!("blueprint_expressions", snapshots);
}

#[test]
fn blueprint_equations_and_predicates_round_trip() {
    let mut equations = Vec::new();
    for text in [
        "sum(p in phase | control_volume.properties_out[t].enth_flow_phase[p]) - sum(p in phase | control_volume.properties_in[t].enth_flow_phase[p]) - heat_duty[t] == 0",
        "if compressor then work_isentropic[t] == work_mechanical[t]*efficiency_isentropic[t] else work_mechanical[t] == work_isentropic[t]*efficiency_isentropic[t]",
        "pressure[t] >= 2{bar}",
    ] {
        let ast = parse_equation(text).unwrap();
        let rendered = render_equation(&ast);
        assert!(
            ast.structural_eq(&parse_equation(&rendered).unwrap()),
            "{rendered}"
        );
        equations.push((text, ast));
    }
    insta::assert_debug_snapshot!("blueprint_equations", equations);
    let mut predicates = Vec::new();
    for text in [
        "has_heat_transfer and energy_balance_type != none",
        "not (a in phase or b == null)",
        "(x + 1) <= 3 and true",
        "false or null",
        "(a and b) or (c and not d)",
    ] {
        let ast = parse_predicate(text).unwrap();
        let rendered = render_predicate(&ast);
        assert!(
            ast.structural_eq(&parse_predicate(&rendered).unwrap()),
            "{rendered}"
        );
        predicates.push((text, ast));
    }
    insta::assert_debug_snapshot!("blueprint_predicates", predicates);
}

#[test]
fn power_is_right_associative_and_ambiguous_unary_power_is_refused() {
    let ast = parse_expr("x^y^z").unwrap();
    let ExprKind::Binary {
        op: BinaryOp::Pow,
        rhs,
        ..
    } = ast.kind
    else {
        panic!("power expected")
    };
    assert!(matches!(
        rhs.kind,
        ExprKind::Binary {
            op: BinaryOp::Pow,
            ..
        }
    ));
    for text in ["-x^2", "-(x)^2"] {
        assert!(matches!(
            parse_expr(text),
            Err(DslError::AmbiguousUnaryPower { offset: 0 })
        ));
    }
}

#[test]
fn explicit_broadcast_preserves_its_value_and_lexical_index() {
    let text = "sum(k in j | h[k] - broadcast(0{J/mol}, k))";
    let ast = parse_expr(text).unwrap();
    let rendered = render_expr(&ast);
    assert!(ast.structural_eq(&parse_expr(&rendered).unwrap()));
    for invalid in [
        "broadcast()",
        "broadcast(1)",
        "broadcast(1, k, j)",
        "broadcast(1, k+1)",
        "broadcast(1, group[k])",
    ] {
        assert!(parse_expr(invalid).is_err(), "{invalid}");
    }
}

#[test]
fn hostile_or_incomplete_forms_fail_with_offsets_without_panicking() {
    for text in ["NaN", "inf", "1e999"] {
        assert!(matches!(
            parse_expr(text),
            Err(DslError::NonFiniteNumber { .. })
        ));
    }
    for text in [
        "if x then a",
        "sum(i in phase a)",
        "smooth_min(x for i in phase, eps=1)",
        "a +",
        "f(a)",
        "safe_log(a, eps=1, x)",
        "1{m/}",
        "x[]",
    ] {
        assert!(parse_expr(text).is_err(), "{text}");
    }
    assert!(matches!(
        parse_expr(&"x".repeat(65_536)),
        Err(DslError::Budget { limit: "bytes", .. })
    ));
    assert!(matches!(
        parse_expr(&format!("{}x{}", "(".repeat(100), ")".repeat(100))),
        Err(DslError::Budget { limit: "depth", .. })
    ));
    let ast = parse_expr("  α[2] + state.β  ").unwrap();
    assert_eq!(ast.span.start, 2);
    assert_eq!(ast.span.end, 18);
    assert_eq!(ast.paths().len(), 2);
}
