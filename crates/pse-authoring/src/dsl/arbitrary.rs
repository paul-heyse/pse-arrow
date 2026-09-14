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

impl Arbitrary for Expr {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
        let leaf = prop_oneof![
            (0_u32..100_000).prop_map(|value| expression(ExprKind::Number(Number {
                value: f64::from(value),
                unit: None
            }))),
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
                    named: vec![]
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
