// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Bounded source AST and type-arena generators for structural parser/renderer
//! qualification.

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

/// A generated type before it is laid out as an arena.
#[derive(Clone, Debug)]
enum TypeShape {
    Leaf(crate::language::TypeNodeKind),
    Named(Vec<String>),
    Variable(String),
    Identifier(Vec<String>),
    Unary(crate::language::TypeNodeKind, Box<TypeShape>),
    Tuple(Vec<TypeShape>),
    Indexed(Box<TypeShape>, Vec<Vec<String>>),
    Function(Vec<TypeShape>, Box<TypeShape>),
    Binary(crate::language::TypeNodeKind, Box<TypeShape>, Box<TypeShape>),
    Power(Box<TypeShape>, pse_quantity::Ratio),
}

/// The type variables [`type_arenas`] names; parse its output with these in scope.
pub const TYPE_VARIABLES: [&str; 2] = ["Q", "R2"];

fn type_path() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(
        prop::sample::select(vec![
            "Mass",
            "MolarCp",
            "chemistry",
            "species",
            "a_b",
            "°C",
            "two words",
            "dotted.segment",
        ]),
        1..3,
    )
    .prop_map(|segments| segments.into_iter().map(str::to_owned).collect())
}

fn layout(shape: &TypeShape, nodes: &mut Vec<crate::language::TypeNode>) -> u32 {
    use crate::language::{TypeExponent, TypeNode, TypeNodeKind as K, type_node};
    let named = |kind, path: &Vec<String>| TypeNode {
        path: Some(path.clone()),
        ..type_node(kind, vec![])
    };
    let node = match shape {
        TypeShape::Leaf(kind) => type_node(*kind, vec![]),
        TypeShape::Named(path) => named(K::Named, path),
        TypeShape::Identifier(path) => named(K::Identifier, path),
        TypeShape::Variable(name) => TypeNode {
            name: Some(name.clone()),
            ..type_node(K::Variable, vec![])
        },
        TypeShape::Unary(kind, child) => type_node(*kind, vec![layout(child, nodes)]),
        TypeShape::Tuple(children) => type_node(
            K::Tuple,
            children.iter().map(|c| layout(c, nodes)).collect(),
        ),
        TypeShape::Indexed(element, axes) => {
            let mut children = vec![layout(element, nodes)];
            for axis in axes {
                nodes.push(named(K::Named, axis));
                children.push(u32::try_from(nodes.len() - 1).unwrap_or(u32::MAX));
            }
            type_node(K::Indexed, children)
        }
        TypeShape::Function(arguments, result) => {
            let mut children = Vec::new();
            for (i, argument) in arguments.iter().enumerate() {
                let ty = layout(argument, nodes);
                nodes.push(TypeNode {
                    name: Some(format!("arg{i}")),
                    ..type_node(K::Argument, vec![ty])
                });
                children.push(u32::try_from(nodes.len() - 1).unwrap_or(u32::MAX));
            }
            children.push(layout(result, nodes));
            type_node(K::Function, children)
        }
        TypeShape::Binary(kind, left, right) => {
            let left = layout(left, nodes);
            type_node(*kind, vec![left, layout(right, nodes)])
        }
        TypeShape::Power(base, exponent) => TypeNode {
            exponent: Some(TypeExponent {
                num: exponent.num(),
                den: exponent.den(),
            }),
            ..type_node(K::Power, vec![layout(base, nodes)])
        },
    };
    nodes.push(node);
    u32::try_from(nodes.len() - 1).unwrap_or(u32::MAX)
}

/// Well-formed post-order type arenas over every node kind, laid out as the parser lays
/// out the canonical spelling of the same tree (ADR-0123 Outcome 1).
pub fn type_arenas() -> impl Strategy<Value = Vec<crate::language::TypeNode>> {
    use crate::language::TypeNodeKind as K;
    let leaf = prop_oneof![
        prop::sample::select(vec![
            K::Boolean,
            K::Integer,
            K::Text,
            K::QuantityType,
            K::ReferenceState
        ])
        .prop_map(TypeShape::Leaf),
        type_path().prop_map(TypeShape::Named),
        type_path().prop_map(TypeShape::Identifier),
        prop::sample::select(TYPE_VARIABLES.to_vec())
            .prop_map(|name| TypeShape::Variable(name.to_owned())),
    ];
    leaf.prop_recursive(5, 64, 3, |inner| {
        prop_oneof![
            (
                prop::sample::select(vec![K::Optional, K::Set, K::Row, K::Table, K::Delta]),
                inner.clone()
            )
                .prop_map(|(kind, child)| TypeShape::Unary(kind, Box::new(child))),
            prop::collection::vec(inner.clone(), 1..3).prop_map(TypeShape::Tuple),
            (inner.clone(), prop::collection::vec(type_path(), 1..3))
                .prop_map(|(element, axes)| TypeShape::Indexed(Box::new(element), axes)),
            (prop::collection::vec(inner.clone(), 0..3), inner.clone()).prop_map(
                |(arguments, result)| TypeShape::Function(arguments, Box::new(result))
            ),
            (
                prop::sample::select(vec![K::Product, K::Quotient]),
                inner.clone(),
                inner.clone()
            )
                .prop_map(|(kind, left, right)| TypeShape::Binary(
                    kind,
                    Box::new(left),
                    Box::new(right)
                )),
            (
                inner,
                (-4_i32..=4).prop_flat_map(|num| (Just(num), 1_i32..=3))
            )
                .prop_filter_map("representable exponent", |(base, (num, den))| {
                    Some(TypeShape::Power(
                        Box::new(base),
                        pse_quantity::Ratio::new(num, den).ok()?,
                    ))
                }),
        ]
    })
    .prop_map(|shape| {
        let mut nodes = Vec::new();
        layout(&shape, &mut nodes);
        nodes
    })
}
