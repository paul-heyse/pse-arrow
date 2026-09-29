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

/// Cell path segments: plain identifiers first, then segments that need quoting.
fn cell_path() -> impl Strategy<Value = Vec<String>> {
    (
        prop::sample::select(vec!["benzene", "chem", "a_b", "liquidPhase", "x1"]),
        prop::collection::vec(
            prop::sample::select(vec!["species", "two words", "°C", "dotted.segment", "q"]),
            0..3,
        ),
    )
        .prop_map(|(first, rest)| {
            std::iter::once(first)
                .chain(rest)
                .map(str::to_owned)
                .collect()
        })
}

/// Scalar cell values: the arms a key cell of a keyed-row reference may take.
fn scalar_values() -> impl Strategy<Value = crate::language::CellValue> {
    use crate::language::{
        CellBoolean, CellIdentifier, CellInteger, CellQuantity, CellReference, CellText,
        CellValue, unit_factors,
    };
    let magnitude = prop_oneof![
        prop::num::f64::NORMAL | prop::num::f64::SUBNORMAL | prop::num::f64::ZERO,
        (-1000_i32..1000).prop_map(f64::from),
    ];
    prop_oneof![
        any::<bool>().prop_map(|value| CellValue::from_boolean(CellBoolean { value })),
        (i64::MIN + 1..=i64::MAX).prop_map(|value| CellValue::from_integer(CellInteger { value })),
        (magnitude, prop::option::of(unit_product())).prop_map(|(magnitude, unit)| {
            CellValue::from_quantity(CellQuantity {
                magnitude,
                unit: unit.as_ref().map(unit_factors),
            })
        }),
        "[a-z \\\\\"\n\t\u{b5}]{0,12}".prop_map(|value| CellValue::from_text(CellText { value })),
        (cell_path(), "[0-9A-Za-z\\-\" ]{0,12}").prop_map(|(scheme, value)| {
            CellValue::from_identifier(CellIdentifier { scheme, value })
        }),
        cell_path().prop_map(|path| CellValue::from_reference(CellReference { path })),
    ]
}

/// Well-formed cells over every variant, with and without an uncertainty (ADR-0123
/// Outcome 1): integers across the 64-bit range, finite magnitudes including negative
/// zero and exponents, canonical unit products, escaped text, quoted path segments and
/// keyed-row references over scalar key cells (Plan 23 KR5).
pub fn cells() -> impl Strategy<Value = crate::language::Cell> {
    use crate::language::{
        Cell, CellPath, CellReferences, CellRow, CellUncertainty, CellValue,
        ModelingUncertaintyKind, cell, key_cell,
    };
    let value = prop_oneof![
        Just(CellValue::from_missing()),
        scalar_values(),
        prop::collection::vec(cell_path(), 0..3).prop_map(|paths| {
            CellValue::from_references(CellReferences {
                paths: paths.into_iter().map(|path| CellPath { path }).collect(),
            })
        }),
        (cell_path(), prop::collection::vec(scalar_values(), 1..4)).prop_map(|(target, keys)| {
            CellValue::from_row(CellRow {
                target,
                keys: keys
                    .into_iter()
                    .filter_map(|value| key_cell(&cell(value)).ok())
                    .collect(),
            })
        }),
    ];
    let uncertainty = prop::option::of(
        (
            prop::sample::select(ModelingUncertaintyKind::ALL.to_vec()),
            prop::num::f64::POSITIVE | prop::num::f64::NORMAL | prop::num::f64::ZERO,
        )
            .prop_map(|(kind, magnitude)| CellUncertainty { kind, magnitude }),
    );
    (value, uncertainty).prop_map(|(value, uncertainty)| Cell { value, uncertainty })
}

/// A plain name for a key, a column or a set path segment.
fn plain_name() -> impl Strategy<Value = String> {
    prop::sample::select(vec!["j", "k", "a_b", "x1", "value", "species", "s"]).prop_map(str::to_owned)
}

/// Well-formed table declarations over every relation constraint (ADR-0123 Outcome 3,
/// Plan 23 KR5): integer-range and typed keys, supplied and derived columns or a value
/// type, symmetry with either diagonal policy, uniqueness, completeness over sets, ranges
/// and open keys, each missing policy with its typed default cell, and requirements.
pub fn tables()
-> impl Strategy<Value = crate::language::AuthoredModelingDeclarationsFieldValueTable> {
    use crate::language::{
        AuthoredModelingDeclarationsFieldValueTable as Table,
        AuthoredModelingDeclarationsFieldValueTableColumnsItem as Column,
        AuthoredModelingDeclarationsFieldValueTableKeysItem as Key,
        AuthoredModelingDeclarationsFieldValueTableSymmetry as Symmetry,
        AuthoredModelingDeclarationsFieldValueTableUniqueItem as Unique, ModelingCompleteness,
        ModelingIntegerRange, parse_type,
    };
    use pse_model::generated::enums::{ModelingDiagonalPolicy, ModelingMissingPolicy};
    let ty = || {
        prop::sample::select(vec![
            "Mass",
            "Integer",
            "chemistry.species",
            "Set<k>",
            "Mass?",
            "Text",
            "Id<cas>",
        ])
        .prop_map(|text| parse_type(text, &[]).unwrap_or_default())
    };
    let range = || {
        (-3_i64..3, 0_i64..4).prop_map(|(lower, width)| ModelingIntegerRange {
            lower,
            upper: lower + width,
        })
    };
    let key = (
        plain_name(),
        prop::option::of(range()),
        ty(),
    )
        .prop_map(|(name, range, r#type)| Key {
            name,
            r#type: if range.is_some() {
                parse_type("Integer", &[]).unwrap_or_default()
            } else {
                r#type
            },
            range,
        });
    let column = (
        plain_name(),
        ty(),
        prop::option::of(prop::sample::select(vec!["2 * v", "sum(i in s | x[i])", "a.b + 1{kg}"])),
    )
        .prop_map(|(name, r#type, derived)| Column {
            name,
            r#type,
            derived: derived.map(str::to_owned),
        });
    let entry = (
        plain_name(),
        prop_oneof![
            Just((None, None)),
            prop::collection::vec(plain_name(), 1..3).prop_map(|set| (Some(set), None)),
            range().prop_map(|range| (None, Some(range))),
        ],
    )
        .prop_map(|(key, (set, range))| ModelingCompleteness { key, set, range });
    (
        prop::collection::vec(key, 0..4),
        prop_oneof![
            ty().prop_map(|value| (Some(value), Vec::new())),
            prop::collection::vec(column, 1..4).prop_map(|columns| (None, columns)),
        ],
        prop::option::of((
            plain_name(),
            plain_name(),
            prop::sample::select(ModelingDiagonalPolicy::ALL.to_vec()),
        )),
        prop::collection::vec(prop::collection::vec(plain_name(), 1..3), 0..3),
        prop::collection::vec(entry, 0..3),
        prop::sample::select(ModelingMissingPolicy::ALL.to_vec()),
        cells(),
        prop::collection::vec(
            prop::sample::select(vec!["v > 0{kg}", "present(a) and b == c", "j in s"]),
            0..3,
        ),
    )
        .prop_map(
            |(keys, (value_type, columns), symmetry, unique, complete_over, policy, default, requirements)| {
                Table {
                    keys,
                    columns,
                    value_type,
                    missing_policy: policy,
                    default_value: (policy == ModelingMissingPolicy::Default).then_some(default),
                    complete_over,
                    symmetry: symmetry.map(|(first, second, diagonal)| Symmetry {
                        first,
                        second,
                        diagonal,
                    }),
                    unique: unique.into_iter().map(|names| Unique { names }).collect(),
                    requirements: requirements.into_iter().map(str::to_owned).collect(),
                }
            },
        )
}
