// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The primitive relation adapters preserve ordered structure and reject invalid actual rows.
#![allow(clippy::unwrap_used, reason = "fixture assertions")]
use pse_compiler::mathir_relations::{Family, RelationSink, RelationSource};
use pse_ids::{CancellationToken, FixedBudget, SemanticId};
use pse_mathir::{ExprGraph, Opcode, Payload, ValueRef};
use pse_relations::{columnar::FieldCheckedBatch, generated::compiled};
use pse_schema::math::TemplateValueKind;

#[test]
fn whole_node_sink_refuses_affine_operand_disagreement_and_signed_overflow() {
    use pse_mathir::{AffineTerm, NodeId, relations::MathRelationSink};
    let registry = pse_schema::registry().unwrap();
    let budget = FixedBudget::new(1 << 20);
    let cancel = CancellationToken::new();
    let mut sink = RelationSink::new(registry, Family::Compiled, budget.as_ref(), &cancel);
    let payload = Payload::Affine {
        constant: 0.0,
        constant_quantity_type: None,
        constant_unit: None,
        terms: vec![AffineTerm {
            coefficient: 2.0,
            child: NodeId(7),
        }],
    };
    let hash = pse_ids::ContentHash::from_bytes([0; 32]);
    assert!(
        sink.expr_node(
            NodeId(9),
            Opcode::Affine,
            &[NodeId(8)],
            &payload,
            None,
            None,
            hash
        )
        .is_err()
    );
    assert!(
        sink.expr_node(
            NodeId(-1),
            Opcode::Const,
            &[],
            &Payload::IntConst { value: 3 },
            None,
            None,
            hash
        )
        .is_err()
    );
    assert!(sink.into_batches().unwrap().is_empty());
}

#[test]
fn relation_round_trip_preserves_children_and_rejects_duplicate_node_rows() {
    let registry = pse_schema::registry().unwrap();
    let mut graph = ExprGraph::new();
    let first = graph.int_const(2).unwrap();
    let second = graph.symbol(SemanticId::from_bytes([4; 16])).unwrap();
    let root = graph.sub(first, second).unwrap();
    let budget = FixedBudget::new(2 << 30);
    let cancel = CancellationToken::new();
    let mut sink = RelationSink::new(registry, Family::Compiled, budget.as_ref(), &cancel);
    pse_mathir::relations::emit_untyped(&graph, &[root], &mut sink).unwrap();
    let mut rows = sink.into_batches().unwrap();
    let source = RelationSource::from_checked(
        &rows,
        registry,
        pse_compiler::mathir_relations::SourceFamily::Compiled,
    )
    .unwrap();
    let loaded = pse_mathir::relations::load_untyped(&source, &[root]).unwrap();
    let node = loaded.graph.node(loaded.roots[0]).unwrap();
    assert_eq!(node.opcode, Opcode::Sub);
    assert!(matches!(
        loaded.graph.node(node.children[0]).unwrap().payload,
        Payload::IntConst { value: 2 }
    ));
    assert!(matches!(
        loaded.graph.node(node.children[1]).unwrap().payload,
        Payload::SymbolRef { .. }
    ));
    let spec = registry.relation("compiled.math_expr_nodes").unwrap();
    let constants = rows[&spec.key].clone();
    let view = compiled::math_expr_nodes::View::from_checked(&constants).unwrap();
    let mut duplicate = compiled::math_expr_nodes::Builder::with_registry(registry, 1).unwrap();
    duplicate.push(view.row(0).unwrap()).unwrap();
    rows.insert(
        spec.key,
        FieldCheckedBatch::concat(registry, spec, &[constants, duplicate.finish().unwrap()])
            .unwrap(),
    );
    let source = RelationSource::from_checked(
        &rows,
        registry,
        pse_compiler::mathir_relations::SourceFamily::Compiled,
    )
    .unwrap();
    assert!(pse_mathir::relations::load_untyped(&source, &[root]).is_err());
}
#[test]
fn compiled_sink_cannot_emit_unresolved_template_values_or_conversion_requests() {
    let registry = pse_schema::registry().unwrap();
    let mut graph = ExprGraph::new();
    let symbol = graph
        .insert(
            Opcode::SymbolRef,
            Payload::SymbolRef {
                symbol: ValueRef::Template {
                    template_id: SemanticId::from_bytes([2; 16]),
                    kind: TemplateValueKind::Parameter,
                    name: "x".to_owned(),
                },
            },
            &[],
            None,
        )
        .unwrap();
    let budget = FixedBudget::new(2 << 30);
    let cancel = CancellationToken::new();
    let mut sink = RelationSink::new(registry, Family::Compiled, budget.as_ref(), &cancel);
    assert!(pse_mathir::relations::emit_untyped(&graph, &[symbol], &mut sink).is_err());
    let mut graph = ExprGraph::new();
    let value = graph.int_const(1).unwrap();
    let request = graph
        .insert(
            Opcode::UnitConvert,
            Payload::PendingUnitConvert {
                to: pse_quantity::UnitId::from_id(SemanticId::from_bytes([3; 16])),
            },
            &[value],
            None,
        )
        .unwrap();
    let budget = FixedBudget::new(2 << 30);
    let cancel = CancellationToken::new();
    let mut sink = RelationSink::new(registry, Family::Compiled, budget.as_ref(), &cancel);
    assert!(pse_mathir::relations::emit_untyped(&graph, &[request], &mut sink).is_err());
}

#[test]
fn kernel_parameters_round_trip_one_symbol_or_one_complete_literal() {
    use pse_mathir::relations::MathRelationSink;
    let registry = pse_schema::registry().unwrap();
    let budget = FixedBudget::new(1 << 24);
    let cancel = CancellationToken::new();
    let binding = SemanticId::from_bytes([7; 16]);
    let symbol = SemanticId::from_bytes([8; 16]);
    let unit = pse_quantity::UnitId::from_id(SemanticId::from_bytes([9; 16]));
    let parameters = vec![
        ("variable".into(), Some(symbol), None, None),
        ("constant".into(), None, Some(2.5), Some(unit)),
    ];
    for (family, source_family) in [
        (
            Family::Compiled,
            pse_compiler::mathir_relations::SourceFamily::Compiled,
        ),
        (
            Family::Inferred,
            pse_compiler::mathir_relations::SourceFamily::Inferred,
        ),
    ] {
        let mut sink = RelationSink::new(registry, family, budget.as_ref(), &cancel);
        for invalid in [
            ("missing".into(), None, Some(1.0), None),
            ("overlap".into(), Some(symbol), Some(1.0), Some(unit)),
            ("nonfinite".into(), None, Some(f64::NAN), Some(unit)),
        ] {
            assert!(
                sink.kernel_binding(binding, symbol, symbol, &[invalid], &[])
                    .is_err()
            );
        }
        sink.kernel_binding(binding, symbol, symbol, &parameters, &[])
            .unwrap();
        let mut batches = sink.into_batches().unwrap();
        if matches!(
            source_family,
            pse_compiler::mathir_relations::SourceFamily::Inferred
        ) {
            for spec in registry.relations().iter().filter(|spec| {
                spec.key.namespace == pse_schema::model::Namespace::Inferred
                    && (spec.key.name.starts_with("math_") || spec.key.name == "kernel_bindings")
            }) {
                if let std::collections::btree_map::Entry::Vacant(entry) = batches.entry(spec.key) {
                    entry.insert(
                        FieldCheckedBatch::concat_reserved(
                            registry,
                            spec,
                            &[],
                            budget.as_ref(),
                            &cancel,
                        )
                        .unwrap(),
                    );
                }
            }
        }
        let source = RelationSource::from_checked(&batches, registry, source_family).unwrap();
        let loaded = pse_mathir::relations::load_untyped(&source, &[]).unwrap();
        assert_eq!(loaded.kernel_bindings[&binding].parameters, parameters);
    }
}
