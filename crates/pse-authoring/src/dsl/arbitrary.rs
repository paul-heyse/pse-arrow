// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Bounded source AST generators for structural parser/renderer qualification.

use super::{
    BinaryOp, Binder, Expr, ExprKind, Function, Number, Path, PathSegment, Predicate,
    PredicateKind, ReduceKind, Span,
};
use proptest::prelude::*;

fn expression(kind: ExprKind) -> Expr {
    Expr {
        kind,
        span: Span::default(),
    }
}
fn path(name: &str) -> Path {
    Path {
        segments: vec![PathSegment {
            name: name.to_owned(),
            indices: vec![],
        }],
    }
}

/// Canonical unit products over a few symbols with small rational exponents, including
/// the empty product `1`.
fn unit_product() -> impl Strategy<Value = pse_quantity::UnitProduct> {
    prop::collection::vec(
        (
            prop::sample::select(vec!["J", "K", "mol", "m", "s", "degC"]),
            -4_i32..=4,
            prop::sample::select(vec![1_i32, 2, 3]),
        ),
        0..4,
    )
    .prop_filter_map("representable product", |factors| {
        pse_quantity::UnitProduct::from_factors(
            factors
                .into_iter()
                .filter_map(|(symbol, num, den)| {
                    Some((symbol.to_owned(), pse_quantity::Ratio::new(num, den).ok()?))
                })
                .collect::<Vec<_>>(),
        )
        .ok()
    })
}

impl Arbitrary for Expr {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
        let leaf = prop_oneof![
            (0_u32..100_000, prop::option::of(unit_product())).prop_map(|(value, unit)| {
                expression(ExprKind::Number(Number {
                    exact_integer: None,
                    value: f64::from(value),
                    unit,
                }))
            }),
            "value_[a-z_]{0,8}".prop_map(|name| expression(ExprKind::Path(path(&name)))),
        ];
        leaf.prop_recursive(5, 96, 3, |inner| {
            prop_oneof![
                (
                    inner.clone(),
                    inner.clone(),
                    prop::sample::select(vec![
                        BinaryOp::Add,
                        BinaryOp::Sub,
                        BinaryOp::Mul,
                        BinaryOp::Div,
                        BinaryOp::Pow
                    ])
                )
                    .prop_map(|(lhs, rhs, op)| expression(ExprKind::Binary {
                        op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs)
                    })),
                inner
                    .clone()
                    .prop_map(|value| expression(ExprKind::Neg(Box::new(value)))),
                inner.clone().prop_map(|value| expression(ExprKind::Call {
                    function: Function::Exp,
                    args: vec![value],
                })),
                inner.clone().prop_map(|body| expression(ExprKind::Reduce {
                    kind: ReduceKind::Sum,
                    binder: Box::new(Binder {
                        var: "i".to_owned(),
                        domain: path("domain"),
                        filter: None
                    }),
                    body: Box::new(body)
                })),
                inner
                    .clone()
                    .prop_map(|index| expression(ExprKind::Path(Path {
                        segments: vec![PathSegment {
                            name: "indexed".to_owned(),
                            indices: vec![index]
                        }]
                    }))),
                (any::<bool>(), inner.clone(), inner.clone()).prop_map(
                    |(guard, then, otherwise)| expression(ExprKind::Conditional {
                        guard: Box::new(Predicate {
                            kind: PredicateKind::Bool(guard),
                            span: Span::default()
                        }),
                        then: Box::new(then),
                        otherwise: Box::new(otherwise)
                    })
                ),
                (inner.clone(), inner).prop_map(|(value, body)| expression(ExprKind::Let {
                    bindings: vec![("local".to_owned(), value)],
                    body: Box::new(body)
                })),
            ]
        })
        .boxed()
    }
}
