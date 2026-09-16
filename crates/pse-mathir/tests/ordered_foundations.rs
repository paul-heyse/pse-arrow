// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact graph shape, dependency order and boundary checks, independent of hash agreement.
use pse_ids::SemanticId;
use pse_mathir::{ExprGraph, Opcode, Payload, WeightedPair, number_typed_graph};
use pse_quantity::*;
use std::collections::BTreeMap;
#[expect(
    clippy::expect_used,
    reason = "test fixture/assertion helper requires valid setup"
)]
fn registry() -> QuantityRegistry {
    let id = SemanticId::NIL;
    let mut builder = QuantityRegistryBuilder::new();
    builder
        .unit(Unit {
            id: UnitId::from_id(id),
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
            id: QuantityTypeId::from_id(id),
            key: QuantityTypeKey {
                kind: QuantityKindId::from_id(id),
                basis: None,
                reference_state: None,
                scale_kind: ScaleKind::Point,
                shape: vec![],
                subject_kind: None,
            },
            canonical_unit: UnitId::from_id(id),
            nominal_magnitude: None,
        });
    builder.build().expect("admitted fixture")
}
#[expect(
    clippy::expect_used,
    reason = "test fixture/assertion helper requires valid setup"
)]
fn graph(reverse_insertion: bool) -> (ExprGraph, pse_mathir::NodeId) {
    let mut graph = ExprGraph::new();
    let (a, b) = if reverse_insertion {
        let b = graph.int_const(2).expect("literal");
        let a = graph.int_const(1).expect("literal");
        (a, b)
    } else {
        (
            graph.int_const(1).expect("literal"),
            graph.int_const(2).expect("literal"),
        )
    };
    let root = graph
        .insert(
            Opcode::WeightedMean,
            Payload::WeightedMean {
                pairs: vec![WeightedPair {
                    weight: a,
                    value: b,
                }],
                normalization: WeightNormalization::DivideBySum,
                unit_sum_invariant: None,
            },
            &[],
            None,
        )
        .expect("mean");
    for id in graph.iter().map(|(id, _)| id).collect::<Vec<_>>() {
        graph
            .set_quantity_type(id, QuantityTypeId::from_id(SemanticId::NIL))
            .expect("known node");
    }
    (graph, root)
}
#[test]
fn independent_insertion_orders_have_same_actual_numbered_payloads() {
    let reg = registry();
    let (first, first_root) = graph(false);
    let (second, second_root) = graph(true);
    let indices = |graph: &ExprGraph| {
        graph
            .iter()
            .map(|(id, _)| (id, IndexSet::new()))
            .collect::<BTreeMap<_, _>>()
    };
    let first =
        number_typed_graph(&first, &[first_root], &indices(&first), &reg).expect("numbered");
    let second =
        number_typed_graph(&second, &[second_root], &indices(&second), &reg).expect("numbered");
    assert_eq!(first.listing(), second.listing());
    assert_eq!(first.roots(), second.roots());
    for ((left_id, left), (right_id, right)) in first.iter().zip(second.iter()) {
        assert_eq!(left_id, right_id);
        assert_eq!(left.opcode, right.opcode);
        assert_eq!(left.payload, right.payload);
        assert_eq!(left.children, right.children);
        assert_eq!(left.quantity_type, right.quantity_type);
        assert_eq!(left.free_indices, right.free_indices);
        assert_eq!(left.subtree_hash, right.subtree_hash);
    }
    let root = first.node(first.roots()[0]).expect("root");
    assert_eq!(
        root.payload.referenced_nodes(),
        vec![pse_mathir::NodeId(0), pse_mathir::NodeId(1)]
    );
}
#[test]
fn numbering_requires_actual_type_and_index_admission() {
    let reg = registry();
    let mut g = ExprGraph::new();
    let root = g.int_const(1).expect("literal");
    let indices = BTreeMap::from([(root, IndexSet::new())]);
    assert!(number_typed_graph(&g, &[root], &indices, &reg).is_err());
    g.set_quantity_type(
        root,
        QuantityTypeId::from_id(SemanticId::from_bytes([9; 16])),
    )
    .expect("node");
    assert!(number_typed_graph(&g, &[root], &indices, &reg).is_err());
    g.set_quantity_type(root, QuantityTypeId::from_id(SemanticId::NIL))
        .expect("node");
    assert!(number_typed_graph(&g, &[root], &BTreeMap::new(), &reg).is_err());
    assert!(number_typed_graph(&g, &[root], &indices, &reg).is_ok());
}
#[test]
fn payload_only_gather_and_one_child_piecewise_contract_are_enforced() {
    let mut g = ExprGraph::new();
    let body = g.int_const(1).expect("literal");
    let gather = Payload::Gather {
        group: SemanticId::NIL,
        coordinate_map: vec![],
    };
    assert!(g.insert(Opcode::Gather, gather.clone(), &[], None).is_ok());
    assert!(g.insert(Opcode::Gather, gather, &[body], None).is_err());
    let piecewise = Payload::PiecewiseLinear {
        breakpoints: vec![(0.0, 1.0), (1.0, 2.0)],
        input: QuantityTypeId::from_id(SemanticId::NIL),
        output: QuantityTypeId::from_id(SemanticId::NIL),
    };
    assert!(
        g.insert(Opcode::PiecewiseLinear, piecewise.clone(), &[], None)
            .is_err()
    );
    assert!(
        g.insert(Opcode::PiecewiseLinear, piecewise, &[body], None)
            .is_ok()
    );
    let duplicate = Payload::PiecewiseLinear {
        breakpoints: vec![(1.0, 1.0), (1.0, 2.0)],
        input: QuantityTypeId::from_id(SemanticId::NIL),
        output: QuantityTypeId::from_id(SemanticId::NIL),
    };
    assert!(
        g.insert(Opcode::PiecewiseLinear, duplicate, &[body], None)
            .is_err()
    );
}
#[test]
fn table_projection_contains_every_declaration_without_reconstruction() {
    struct Sink(Vec<Opcode>);
    impl pse_mathir::catalog::OperatorSpecSink for Sink {
        fn operator_spec(
            &mut self,
            spec: &pse_schema::math::operators::OperatorSpec,
            operations: &[OperationId],
        ) {
            self.0.push(spec.opcode);
            assert!(operations.is_empty());
            assert_eq!(spec.arity, pse_schema::math::arity(spec.opcode));
        }
    }
    let mut sink = Sink(vec![]);
    pse_mathir::catalog::emit_operator_specs(&registry(), &mut sink);
    assert_eq!(sink.0, Opcode::ALL);
}

#[test]
fn typed_occurrences_of_one_literal_remain_distinct_before_hash_consing() {
    let mut graph = ExprGraph::new();
    let payload = Payload::FloatConst {
        value: 1.0,
        unit: UnitId::from_id(SemanticId::NIL),
    };
    let point = QuantityTypeId::from_id(SemanticId::from_bytes([1; 16]));
    let difference = QuantityTypeId::from_id(SemanticId::from_bytes([2; 16]));
    let raw = graph
        .insert(Opcode::Const, payload.clone(), &[], None)
        .expect("untyped literal");
    graph
        .set_quantity_type(raw, point)
        .expect("point occurrence");
    let diff = graph
        .insert_typed(Opcode::Const, payload.clone(), &[], difference, None)
        .expect("difference occurrence");
    assert_ne!(raw, diff);
    assert_eq!(
        graph
            .insert_typed(Opcode::Const, payload.clone(), &[], point, None)
            .expect("same point"),
        raw
    );
    let untyped = graph
        .insert(Opcode::Const, payload, &[], None)
        .expect("another untyped occurrence");
    assert_ne!(untyped, raw);
    assert_ne!(untyped, diff);
    assert_eq!(graph.node(raw).expect("point").quantity_type, Some(point));
    assert_eq!(
        graph.node(diff).expect("difference").quantity_type,
        Some(difference)
    );
    assert_eq!(graph.node(untyped).expect("untyped").quantity_type, None);
}
