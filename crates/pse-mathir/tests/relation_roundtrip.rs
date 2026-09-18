// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual-value relation round trips and untrusted structural admission.

use pse_ids::{ContentHash, SemanticId};
use pse_mathir::relations::{MathRelationSink, VecSink, emit, emit_untyped, load_untyped};
use pse_mathir::{ExprGraph, NodeId, Opcode, Payload};
use pse_quantity::{
    IndexSet,
    standard::{ids, standard_registry},
};
use std::collections::BTreeMap;
fn hash() -> ContentHash {
    ContentHash::from_bytes([0; 32])
}
fn sid(byte: u8) -> SemanticId {
    SemanticId::from_bytes([byte; 16])
}

#[test]
fn signed_ordinals_and_equation_bounds_are_admitted_before_remapping() {
    use pse_schema::math::Sense;
    let mut negative = VecSink::new();
    negative
        .expr_node(
            NodeId(-1),
            Opcode::Const,
            &[],
            &Payload::IntConst { value: 0 },
            None,
            None,
            hash(),
        )
        .unwrap();
    assert!(load_untyped(&negative, &[]).is_err());
    for (sense, lower, upper, valid) in [
        (Sense::Eq, None, Some(NodeId(0)), true),
        (Sense::Le, None, Some(NodeId(0)), true),
        (Sense::Ge, Some(NodeId(0)), None, true),
        (Sense::Range, Some(NodeId(0)), Some(NodeId(0)), true),
        (Sense::Eq, Some(NodeId(0)), Some(NodeId(0)), false),
        (Sense::Range, None, Some(NodeId(0)), false),
        (Sense::Le, Some(NodeId(0)), None, false),
        (Sense::Ge, None, None, false),
    ] {
        let mut rows = VecSink::new();
        rows.expr_node(
            NodeId(0),
            Opcode::Const,
            &[],
            &Payload::IntConst { value: 0 },
            None,
            None,
            hash(),
        )
        .unwrap();
        rows.indexed_equation(
            sid(1),
            sid(2),
            None,
            "comparison",
            None,
            None,
            NodeId(0),
            sense,
            lower,
            upper,
            None,
            None,
            sid(3),
        )
        .unwrap();
        assert_eq!(load_untyped(&rows, &[]).is_ok(), valid, "{sense:?}");
    }
}

#[test]
fn untrusted_hashes_cannot_hide_duplicate_rows_dangling_children_or_payloads() {
    let mut source = VecSink::new();
    source
        .expr_node(
            NodeId(99),
            Opcode::Const,
            &[],
            &Payload::IntConst { value: 1 },
            None,
            None,
            hash(),
        )
        .unwrap();
    let loaded = load_untyped(&source, &[NodeId(99)]).unwrap();
    assert_eq!(
        loaded.graph.node(loaded.roots[0]).unwrap().payload,
        Payload::IntConst { value: 1 }
    );
    let mut duplicate = source.clone();
    duplicate
        .expr_node(
            NodeId(99),
            Opcode::Const,
            &[],
            &Payload::IntConst { value: 1 },
            None,
            None,
            hash(),
        )
        .unwrap();
    assert!(load_untyped(&duplicate, &[NodeId(99)]).is_err());
    source
        .expr_node(
            NodeId(2),
            Opcode::Neg,
            &[NodeId(100)],
            &Payload::None,
            None,
            None,
            hash(),
        )
        .unwrap();
    assert!(load_untyped(&source, &[NodeId(2)]).is_err());
}
#[test]
fn loader_checks_unreachable_cycles_and_external_kernel_dependencies() {
    let mut source = VecSink::new();
    source
        .expr_node(
            NodeId(0),
            Opcode::Const,
            &[],
            &Payload::IntConst { value: 1 },
            None,
            None,
            hash(),
        )
        .unwrap();
    source
        .expr_node(
            NodeId(9),
            Opcode::KernelCall,
            &[],
            &Payload::KernelCall {
                kernel_binding: sid(1),
                output_ordinal: 0,
            },
            None,
            None,
            hash(),
        )
        .unwrap();
    assert!(load_untyped(&source, &[NodeId(0)]).is_err());
    source
        .kernel_binding(sid(1), sid(2), sid(3), &[], &[("x".into(), NodeId(9))])
        .unwrap();
    assert!(
        matches!(load_untyped(&source, &[NodeId(0)]), Err(pse_mathir::MathIrError::Cycle { path }) if path == vec![NodeId(9), NodeId(9)])
    );
}
#[test]
fn equations_filters_bounds_and_kernel_input_rows_are_remapped_together() {
    let mut source = VecSink::new();
    for (id, value) in [(NodeId(88), 1), (NodeId(22), 2)] {
        source
            .expr_node(
                id,
                Opcode::Const,
                &[],
                &Payload::IntConst { value },
                None,
                None,
                hash(),
            )
            .unwrap();
    }
    source
        .expr_node(
            NodeId(99),
            Opcode::KernelCall,
            &[],
            &Payload::KernelCall {
                kernel_binding: sid(1),
                output_ordinal: 0,
            },
            None,
            None,
            hash(),
        )
        .unwrap();
    source
        .kernel_binding(
            sid(1),
            sid(2),
            sid(3),
            &[],
            &[("x".into(), NodeId(88)), ("y".into(), NodeId(22))],
        )
        .unwrap();
    source
        .indexed_equation(
            sid(4),
            sid(3),
            None,
            "balance",
            None,
            Some(NodeId(88)),
            NodeId(99),
            pse_schema::math::Sense::Range,
            Some(NodeId(22)),
            Some(NodeId(88)),
            None,
            None,
            sid(5),
        )
        .unwrap();
    let loaded = load_untyped(&source, &[NodeId(99), NodeId(88)]).unwrap();
    let equation = &loaded.equations[0];
    assert_eq!(equation.body, loaded.roots[0]);
    assert_eq!(equation.filter, equation.upper);
    assert_eq!(equation.filter, Some(loaded.roots[1]));
    let inputs = &loaded.kernel_bindings[&sid(1)].inputs;
    assert_eq!(inputs[0].1, loaded.roots[1]);
    assert_eq!(Some(inputs[1].1), equation.lower);
    assert_eq!(equation.derivation, sid(5));
}
#[test]
fn ordered_typed_emit_load_emit_preserves_actual_values_and_hashes() {
    let reg = standard_registry().unwrap();
    let mut graph = ExprGraph::new();
    let ty = reg.neutral_dimensionless().unwrap();
    let unit = reg.quantity_type(ty).unwrap().canonical_unit;
    let a = graph
        .insert_typed(
            Opcode::Const,
            Payload::FloatConst { value: -0.0, unit },
            &[],
            ty,
            None,
        )
        .unwrap();
    let b = graph
        .insert_typed(
            Opcode::Const,
            Payload::FloatConst { value: 2.0, unit },
            &[],
            ty,
            None,
        )
        .unwrap();
    let root = graph
        .insert_typed(Opcode::Sub, Payload::None, &[a, b], ty, None)
        .unwrap();
    let indices = graph.iter().map(|(id, _)| (id, IndexSet::new())).collect();
    let canonical = pse_mathir::number_typed_graph(&graph, &[root], &indices, &reg).unwrap();
    let mut first = VecSink::new();
    emit(&canonical, &mut first).unwrap();
    let loaded = load_untyped(&first, canonical.roots()).unwrap();
    let indices = loaded
        .graph
        .iter()
        .map(|(id, _)| (id, IndexSet::new()))
        .collect();
    let again =
        pse_mathir::number_typed_graph(&loaded.graph, &loaded.roots, &indices, &reg).unwrap();
    assert_eq!(canonical.listing(), again.listing());
    for ((_, a), (_, b)) in canonical.iter().zip(again.iter()) {
        assert_eq!(a.children, b.children);
        assert_eq!(a.subtree_hash, b.subtree_hash);
        if let Payload::FloatConst { value, .. } = b.payload
            && value == 0.
        {
            assert!(value.is_sign_negative());
        }
    }
    let mut second = VecSink::new();
    emit(&again, &mut second).unwrap();
    assert_eq!(second.node_rows(), first.node_rows());
    let mut untyped = VecSink::new();
    emit_untyped(&graph, &[root], &mut untyped).unwrap();
    assert_eq!(load_untyped(&untyped, &[root]).unwrap().graph.len(), 3);
}
#[test]
fn bound_kernel_hashes_depend_on_actual_values_and_every_ordered_input() {
    let reg = standard_registry().unwrap();
    let ty = reg.neutral_dimensionless().unwrap();
    let mut graph = ExprGraph::new();
    let a = graph
        .insert_typed(Opcode::Const, Payload::IntConst { value: 1 }, &[], ty, None)
        .unwrap();
    let b = graph
        .insert_typed(Opcode::Const, Payload::IntConst { value: 2 }, &[], ty, None)
        .unwrap();
    let root = graph
        .insert_typed(
            Opcode::KernelCall,
            Payload::KernelCall {
                kernel_binding: sid(1),
                output_ordinal: 0,
            },
            &[],
            ty,
            None,
        )
        .unwrap();
    let indices = graph.iter().map(|(id, _)| (id, IndexSet::new())).collect();
    let binding = pse_mathir::relations::vec_sink::KernelBinding {
        binding: sid(1),
        kernel: sid(2),
        scope: sid(3),
        parameters: vec![(
            "p".into(),
            None,
            Some(4.),
            Some(reg.quantity_type(ty).unwrap().canonical_unit),
        )],
        inputs: vec![("a".into(), a), ("b".into(), b)],
    };
    let make = |binding| {
        pse_mathir::canonical::number_typed_graph_with_bindings(
            &graph,
            &[root],
            &indices,
            &reg,
            &BTreeMap::from([(sid(1), binding)]),
        )
        .unwrap()
    };
    let first = make(binding.clone());
    let mut changed = binding.clone();
    changed.parameters[0].2 = Some(5.);
    let second = make(changed);
    assert_ne!(
        first.node(first.roots()[0]).unwrap().subtree_hash,
        second.node(second.roots()[0]).unwrap().subtree_hash
    );
    let mut changed = binding;
    changed.inputs.swap(0, 1);
    let second = make(changed);
    assert_ne!(
        first.node(first.roots()[0]).unwrap().subtree_hash,
        second.node(second.roots()[0]).unwrap().subtree_hash
    );
    let stored = &first.kernel_bindings()[&sid(1)].inputs;
    assert_eq!(
        first.node(stored[0].1).unwrap().payload,
        Payload::IntConst { value: 1 }
    );
    assert_eq!(
        first.node(stored[1].1).unwrap().payload,
        Payload::IntConst { value: 2 }
    );
}

#[test]
fn template_domains_preserve_composite_keys() {
    use pse_mathir::DomainRef;
    use pse_quantity::{BoundIndexId, DomainId};
    let local = DomainRef::Template {
        template_id: sid(40),
        domain_name: "species".into(),
    };
    let actual = DomainId::from_id(sid(40));
    let mut graph = ExprGraph::new();
    let body = graph.int_const(1).unwrap();
    let mut roots = vec![];
    for reference in [
        local.clone(),
        DomainRef::Template {
            template_id: sid(41),
            domain_name: "species".into(),
        },
        DomainRef::Template {
            template_id: sid(40),
            domain_name: "phases".into(),
        },
        DomainRef::Actual(actual),
    ] {
        roots.push(
            graph
                .insert(
                    Opcode::Broadcast,
                    Payload::Broadcast {
                        domain: reference,
                        bound_index: BoundIndexId::from_id(sid(42)),
                    },
                    &[body],
                    None,
                )
                .unwrap(),
        );
    }
    assert_eq!(graph.len(), 5);
    let mut rows = VecSink::new();
    emit_untyped(&graph, &roots, &mut rows).unwrap();
    let loaded = load_untyped(&rows, &roots).unwrap();
    for (old, new) in roots.iter().zip(&loaded.roots) {
        assert_eq!(
            graph.node(*old).unwrap().payload.domain(),
            loaded.graph.node(*new).unwrap().payload.domain()
        );
    }
    let mut repeated = VecSink::new();
    emit_untyped(&loaded.graph, &loaded.roots, &mut repeated).unwrap();
    let second = load_untyped(&repeated, &loaded.roots).unwrap();
    for (before, after) in loaded.roots.iter().zip(&second.roots) {
        assert_eq!(
            loaded.graph.node(*before).unwrap().payload,
            second.graph.node(*after).unwrap().payload
        );
    }
    let body_hash = pse_mathir::hash::structural_hash(graph.node(body).unwrap(), &[]).unwrap();
    let hashes = roots
        .iter()
        .map(|id| {
            pse_mathir::hash::structural_hash(graph.node(*id).unwrap(), &[body_hash]).unwrap()
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(hashes.len(), 4);
}

#[test]
fn normalized_predicate_and_value_keys_survive_math_remapping_without_fake_symbols() {
    use pse_mathir::{DomainRef, GuardRef, ValueRef};
    use pse_quantity::{BoundIndexId, DomainId};
    use pse_schema::math::TemplateValueKind;
    let mut graph = ExprGraph::new();
    let mut roots = vec![];
    let values = [
        ValueRef::ActualSymbol(sid(50)),
        ValueRef::Template {
            template_id: sid(50),
            kind: TemplateValueKind::Parameter,
            name: "x".into(),
        },
        ValueRef::Template {
            template_id: sid(50),
            kind: TemplateValueKind::Feature,
            name: "x".into(),
        },
        ValueRef::Template {
            template_id: sid(50),
            kind: TemplateValueKind::Port,
            name: "x".into(),
        },
        ValueRef::Domain(DomainRef::Actual(DomainId::from_id(sid(50)))),
        ValueRef::Index(BoundIndexId::from_id(sid(50))),
    ];
    for value in &values {
        roots.push(
            graph
                .insert(
                    Opcode::SymbolRef,
                    Payload::SymbolRef {
                        symbol: value.clone(),
                    },
                    &[],
                    None,
                )
                .unwrap(),
        );
    }
    let guard = GuardRef::Predicate {
        source_id: sid(60),
        predicate_id: 0,
    };
    let conditional = graph
        .insert(
            Opcode::Conditional,
            Payload::Conditional { guard },
            &[roots[0], roots[1]],
            None,
        )
        .unwrap();
    roots.push(conditional);
    assert!(
        graph
            .node(conditional)
            .unwrap()
            .payload
            .referenced_nodes()
            .is_empty()
    );
    let mut source = VecSink::new();
    emit_untyped(&graph, &roots, &mut source).unwrap();
    let loaded = load_untyped(&source, &roots).unwrap();
    for (old, new) in roots.iter().zip(&loaded.roots) {
        assert_eq!(
            graph.node(*old).unwrap().payload,
            loaded.graph.node(*new).unwrap().payload
        );
    }
    assert_eq!(
        loaded
            .graph
            .node(*loaded.roots.last().unwrap())
            .unwrap()
            .payload
            .guard(),
        Some(guard)
    );
}

#[test]
fn pending_conversion_and_integral_keep_their_actual_normalized_context() {
    use pse_mathir::{DomainRef, GuardRef};
    use pse_quantity::BoundIndexId;
    let mut graph = ExprGraph::new();
    let value = graph.int_const(2).unwrap();
    let pending = graph
        .insert(
            Opcode::UnitConvert,
            Payload::PendingUnitConvert { to: ids::unit("K") },
            &[value],
            None,
        )
        .unwrap();
    let filter = GuardRef::Predicate {
        source_id: sid(75),
        predicate_id: 8,
    };
    let integral = graph
        .insert(
            Opcode::Integral,
            Payload::Integral {
                domain: DomainRef::Template {
                    template_id: sid(72),
                    domain_name: "time".into(),
                },
                bound_index: BoundIndexId::from_id(sid(73)),
                quadrature_policy: Some(sid(74)),
                filter: Some(filter),
            },
            &[pending],
            None,
        )
        .unwrap();
    let mut sink = VecSink::new();
    emit_untyped(&graph, &[integral], &mut sink).unwrap();
    let loaded = load_untyped(&sink, &[integral]).unwrap();
    let loaded_integral = loaded.graph.node(loaded.roots[0]).unwrap();
    assert_eq!(
        loaded_integral.payload,
        graph.node(integral).unwrap().payload
    );
    assert_eq!(loaded_integral.payload.guard(), Some(filter));
    assert!(
        matches!(loaded.graph.node(loaded_integral.children[0]).unwrap().payload, Payload::PendingUnitConvert { to } if to == ids::unit("K"))
    );
}

#[test]
fn pending_gather_preserves_ordered_index_expressions_and_refuses_physical_admission() {
    struct Empty;
    impl pse_mathir::infer::SymbolTypeSource for Empty {
        fn symbol_type(&self, _: SemanticId) -> Option<pse_quantity::QuantityTypeId> {
            None
        }
    }
    let mut graph = ExprGraph::new();
    let one = graph.int_const(1).unwrap();
    let two = graph.int_const(2).unwrap();
    let root = graph
        .insert(
            Opcode::Gather,
            Payload::PendingGather {
                group: sid(80),
                indices: vec![two, one],
            },
            &[],
            None,
        )
        .unwrap();
    assert_eq!(
        graph.node(root).unwrap().payload.referenced_nodes(),
        vec![two, one]
    );
    let mut sink = VecSink::new();
    emit_untyped(&graph, &[root], &mut sink).unwrap();
    let loaded = load_untyped(&sink, &[root]).unwrap();
    let Payload::PendingGather { group, indices } =
        &loaded.graph.node(loaded.roots[0]).unwrap().payload
    else {
        panic!("pending indexed source");
    };
    assert_eq!(*group, sid(80));
    assert!(matches!(
        loaded.graph.node(indices[0]).unwrap().payload,
        Payload::IntConst { value: 2 }
    ));
    assert!(matches!(
        loaded.graph.node(indices[1]).unwrap().payload,
        Payload::IntConst { value: 1 }
    ));
    let registry = standard_registry().unwrap();
    assert!(
        pse_mathir::canonicalize::canonicalize(
            pse_mathir::canonicalize::CanonicalizeInput::new(
                &loaded.graph,
                &loaded.roots,
                &Empty,
                &registry
            ),
            pse_mathir::canonicalize::Policy::Strict
        )
        .is_err()
    );
}
