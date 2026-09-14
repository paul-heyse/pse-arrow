// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Guarded and ordered floating outcomes (blueprint §7.4, ADR-0047).
use pse_ids::SemanticId;
use pse_mathir::{ExprGraph, MathIrError, NodeId, Opcode, Payload};
use pse_quantity::*;
#[expect(
    clippy::expect_used,
    reason = "test fixture/assertion helper requires valid setup"
)]
fn fixture() -> (QuantityRegistry, UnitId, QuantityTypeId) {
    let id = SemanticId::NIL;
    let unit = UnitId::from_id(id);
    let ty = QuantityTypeId::from_id(id);
    let mut b = QuantityRegistryBuilder::new();
    b.unit(Unit {
        id: unit,
        symbol: "1".into(),
        dimension: DimensionVector::DIMENSIONLESS,
        scale_to_canonical: 1.0,
        offset_to_canonical: 0.0,
        is_affine: false,
        reference_state: None,
    })
    .kind(QuantityKind {
        id: QuantityKindId::from_id(id),
        dimension: DimensionVector::DIMENSIONLESS,
        extensive: false,
        addition_kind: QuantityAdditionKind::Additive,
    })
    .quantity_type(QuantityType {
        id: ty,
        key: QuantityTypeKey {
            kind: QuantityKindId::from_id(id),
            basis: None,
            reference_state: None,
            scale_kind: ScaleKind::Point,
            shape: vec![],
            subject_kind: None,
        },
        canonical_unit: unit,
        nominal_magnitude: None,
    })
    .neutral_dimensionless(ty);
    (b.build().expect("fixture"), unit, ty)
}
#[expect(
    clippy::expect_used,
    reason = "test fixture/assertion helper requires valid setup"
)]
fn type_all(graph: &mut ExprGraph, ty: QuantityTypeId) {
    for id in graph.iter().map(|(id, _)| id).collect::<Vec<_>>() {
        graph.set_quantity_type(id, ty).expect("known node");
    }
}
#[expect(
    clippy::expect_used,
    reason = "test fixture/assertion helper requires valid setup"
)]
#[expect(
    clippy::panic,
    reason = "unexpected folded payload is a failed test assertion"
)]
fn folded_value(graph: &ExprGraph, id: NodeId) -> f64 {
    match graph.node(id).expect("node").payload {
        Payload::FloatConst { value, .. } => value,
        ref other => panic!("expected float, got {other:?}"),
    }
}
#[test]
fn nested_addition_keeps_original_rounding_and_negative_zero() {
    let (registry, unit, ty) = fixture();
    let mut g = ExprGraph::new();
    let a = g.float_const(1e16, unit).expect("a");
    let b = g.float_const(-1e16, unit).expect("b");
    let one = g.float_const(1.0, unit).expect("one");
    let inner = g.add(b, one).expect("inner");
    let root = g.add(a, inner).expect("outer");
    type_all(&mut g, ty);
    assert_eq!(
        pse_mathir::fold::fold_literals(&mut g, &[root], &registry)
            .expect("fold")
            .folded,
        2
    );
    assert_eq!(folded_value(&g, root).to_bits(), 0.0_f64.to_bits());
    let mut g = ExprGraph::new();
    let negative = g.float_const(-1.0, unit).expect("negative");
    let zero = g.float_const(0.0, unit).expect("zero");
    let root = g.mul(negative, zero).expect("product");
    type_all(&mut g, ty);
    pse_mathir::fold::fold_literals(&mut g, &[root], &registry).expect("fold");
    assert_eq!(folded_value(&g, root).to_bits(), (-0.0f64).to_bits());
}
#[test]
fn overflow_and_nonliteral_structure_remain_for_evaluation() {
    let (registry, unit, ty) = fixture();
    let mut g = ExprGraph::new();
    let a = g.float_const(1e308, unit).expect("a");
    let b = g.float_const(10.0, unit).expect("b");
    let root = g.mul(a, b).expect("overflow");
    type_all(&mut g, ty);
    let report = pse_mathir::fold::fold_literals(&mut g, &[root], &registry).expect("deferred");
    assert_eq!(report.folded, 0);
    assert_eq!(g.node(root).expect("root").opcode, Opcode::Mul);
    assert_eq!(report.deferred_static_checks, vec![root]);
    let x = g.symbol(SemanticId::NIL).expect("symbol");
    let root = g.add(x, b).expect("sum");
    type_all(&mut g, ty);
    pse_mathir::fold::fold_literals(&mut g, &[root], &registry).expect("unchanged");
    assert_eq!(g.node(root).expect("root").children, vec![x, b]);
}
#[test]
fn an_excluded_failure_is_deferred_but_an_unconditional_shared_use_still_fails() {
    let (registry, unit, ty) = fixture();
    let mut g = ExprGraph::new();
    let one = g.float_const(1.0, unit).expect("one");
    let zero = g.float_const(0.0, unit).expect("zero");
    let bad = g.div(one, zero).expect("domain checked later");
    let guard = g.int_const(0).expect("guard");
    let conditional = g
        .insert(
            Opcode::Conditional,
            Payload::Conditional {
                guard: guard.into(),
            },
            &[bad, one],
            None,
        )
        .expect("conditional");
    type_all(&mut g, ty);
    let report = pse_mathir::fold::fold_literals(&mut g, &[conditional], &registry)
        .expect("excluded branch remains");
    assert!(report.deferred_static_checks.contains(&bad));
    assert_eq!(g.node(bad).expect("bad").opcode, Opcode::Div);
    assert!(
        matches!(pse_mathir::fold::fold_literals(&mut g,&[conditional,bad],&registry),Err(MathIrError::StaticDomain {node,..}) if node==bad)
    );
    assert_eq!(g.node(bad).expect("atomic failure").opcode, Opcode::Div);
}
#[test]
fn unknown_guard_preserves_even_finite_branch_arithmetic() {
    let (registry, unit, ty) = fixture();
    let mut g = ExprGraph::new();
    let one = g.float_const(1.0, unit).expect("one");
    let sum = g.add(one, one).expect("sum");
    let guard = g.symbol(SemanticId::NIL).expect("guard");
    let conditional = g
        .insert(
            Opcode::Conditional,
            Payload::Conditional {
                guard: guard.into(),
            },
            &[sum, one],
            None,
        )
        .expect("conditional");
    type_all(&mut g, ty);
    assert_eq!(
        pse_mathir::fold::fold_literals(&mut g, &[conditional], &registry)
            .expect("guarded")
            .folded,
        0
    );
    assert_eq!(g.node(sum).expect("sum").opcode, Opcode::Add);
}
