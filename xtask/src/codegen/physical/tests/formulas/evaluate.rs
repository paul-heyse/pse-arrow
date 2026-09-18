// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Test-only scalar probe of the already parsed source. This is no production backend.

use pse_authoring::dsl::{BinaryOp, Expr, ExprKind, Function};
use pse_quantity::QuantityRegistry;
use std::collections::BTreeMap;

pub(super) fn expression(
    expr: &Expr,
    values: &BTreeMap<String, f64>,
    registry: &QuantityRegistry,
) -> f64 {
    match &expr.kind {
        ExprKind::Number(number) => number.unit.as_ref().map_or(number.value, |symbol| {
            let unit = registry
                .units()
                .find(|unit| unit.symbol == *symbol)
                .expect("declared natural unit");
            number.value * unit.scale_to_canonical + unit.offset_to_canonical
        }),
        ExprKind::Path(path) => {
            assert_eq!(
                path.segments.len(),
                1,
                "scalar coefficient probe has no member-path shortcut"
            );
            let segment = &path.segments[0];
            assert!(segment.indices.is_empty() || segment.indices.len() == 1);
            values[&segment.name]
        }
        ExprKind::Neg(value) => -expression(value, values, registry),
        ExprKind::Binary { op, lhs, rhs } => {
            let left = expression(lhs, values, registry);
            let right = expression(rhs, values, registry);
            match op {
                BinaryOp::Add => left + right,
                BinaryOp::Sub => left - right,
                BinaryOp::Mul => left * right,
                BinaryOp::Div => left / right,
                BinaryOp::Pow => left.powf(right),
            }
        }
        ExprKind::Call {
            function,
            args,
            named,
        } => {
            if *function == Function::Broadcast {
                assert_eq!(args.len(), 2);
                assert!(named.is_empty());
                // This oracle evaluates one already selected scalar member.
                // Broadcast's second argument names its index, not a value.
                return expression(&args[0], values, registry);
            }
            assert_eq!(args.len(), 1, "{function:?}");
            assert!(named.is_empty());
            let value = expression(&args[0], values, registry);
            match function {
                Function::Exp => value.exp(),
                Function::Log => value.ln(),
                other => panic!("unexercised scalar formula function {other:?}"),
            }
        }
        ExprKind::Let { bindings, body } => {
            let mut local = values.clone();
            for (name, value) in bindings {
                local.insert(name.clone(), expression(value, &local, registry));
            }
            expression(body, &local, registry)
        }
        other => panic!("unexercised scalar probe syntax {other:?}"),
    }
}
