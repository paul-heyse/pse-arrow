// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Unambiguous rendering preserves ordered syntax and explicitly marks every grouping.

use super::ast::{Equation, EquationKind, Expr, ExprKind, Path, Predicate, PredicateKind};

/// Render an expression with all potentially ambiguous groupings explicit.
pub fn render_expr(expr: &Expr) -> String {
    match &expr.kind {
        ExprKind::Number(number) => format!(
            "{}{}",
            number.value,
            number
                .unit
                .as_ref()
                .map_or_else(String::new, |unit| format!("{{{unit}}}"))
        ),
        ExprKind::Path(path) => render_path(path),
        ExprKind::Neg(inner) => format!("(-({}))", render_expr(inner)),
        ExprKind::Binary { op, lhs, rhs } => format!(
            "({} {} {})",
            render_expr(lhs),
            op.as_str(),
            render_expr(rhs)
        ),
        ExprKind::Call {
            function,
            args,
            named,
        } => {
            let mut arguments: Vec<_> = args.iter().map(render_expr).collect();
            arguments.extend(
                named
                    .iter()
                    .map(|arg| format!("{}={}", arg.name, render_expr(&arg.value))),
            );
            format!("{}({})", function.as_str(), arguments.join(", "))
        }
        ExprKind::Kernel { name, args } => format!(
            "kernel.{name}({})",
            args.iter().map(render_expr).collect::<Vec<_>>().join(", ")
        ),
        ExprKind::Reduce { kind, binder, body } => format!(
            "{}({} in {}{} | {})",
            kind.as_str(),
            binder.var,
            render_path(&binder.domain),
            binder
                .filter
                .as_ref()
                .map_or_else(String::new, |filter| format!(
                    " where {}",
                    render_predicate(filter)
                )),
            render_expr(body)
        ),
        ExprKind::Derivative { body, wrt } => {
            format!("d({})/d {}", render_expr(body), render_path(wrt))
        }
        ExprKind::Conditional {
            guard,
            then,
            otherwise,
        } => format!(
            "(if {} then {} else {})",
            render_predicate(guard),
            render_expr(then),
            render_expr(otherwise)
        ),
        ExprKind::Let { bindings, body } => format!(
            "({} where {})",
            render_expr(body),
            bindings
                .iter()
                .map(|(name, value)| format!("{name} = {}", render_expr(value)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

/// Render a dotted member path, retaining each segment's ordered indices.
pub fn render_path(path: &Path) -> String {
    path.segments
        .iter()
        .map(|segment| {
            format!(
                "{}{}",
                segment.name,
                if segment.indices.is_empty() {
                    String::new()
                } else {
                    format!(
                        "[{}]",
                        segment
                            .indices
                            .iter()
                            .map(render_expr)
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            )
        })
        .collect::<Vec<_>>()
        .join(".")
}

/// Render a predicate with explicit logical grouping.
pub fn render_predicate(predicate: &Predicate) -> String {
    match &predicate.kind {
        PredicateKind::Compare { op, lhs, rhs } => format!(
            "({} {} {})",
            render_expr(lhs),
            op.as_str(),
            render_expr(rhs)
        ),
        PredicateKind::In { expr, domain } => {
            format!("({} in {})", render_expr(expr), render_path(domain))
        }
        PredicateKind::And(lhs, rhs) => {
            format!("({} and {})", render_predicate(lhs), render_predicate(rhs))
        }
        PredicateKind::Or(lhs, rhs) => {
            format!("({} or {})", render_predicate(lhs), render_predicate(rhs))
        }
        PredicateKind::Not(value) => format!("not ({})", render_predicate(value)),
        PredicateKind::Atom(expr) => render_expr(expr),
        PredicateKind::Bool(value) => value.to_string(),
        PredicateKind::Null => "null".to_owned(),
    }
}

/// Render an equation with the exact sense of each conditional branch preserved.
pub fn render_equation(equation: &Equation) -> String {
    match &equation.kind {
        EquationKind::Relation { lhs, sense, rhs } => format!(
            "{} {} {}",
            render_expr(lhs),
            sense.as_str(),
            render_expr(rhs)
        ),
        EquationKind::Conditional {
            guard,
            then,
            otherwise,
        } => format!(
            "if {} then {} else {}",
            render_predicate(guard),
            render_equation(then),
            render_equation(otherwise)
        ),
    }
}
