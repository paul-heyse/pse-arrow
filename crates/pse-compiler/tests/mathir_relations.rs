// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The primitive relation adapters preserve ordered structure and reject invalid actual rows.
#![allow(clippy::unwrap_used, reason = "fixture assertions")]
use pse_compiler::mathir_relations::{Family, RelationSink, RelationSource};
use pse_ids::{CancellationToken, FixedBudget, SemanticId};
use pse_mathir::{ExprGraph, Opcode, Payload, TemplateValueKind, ValueRef};
use pse_relations::{columnar::FieldCheckedBatch, generated::compiled};

#[test]
fn relation_round_trip_preserves_children_and_rejects_duplicate_payload_rows() {
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
    let spec = registry.relation("compiled.math_int_constants").unwrap();
    let constants = rows[&spec.key].clone();
    let view = compiled::math_int_constants::View::from_checked(&constants).unwrap();
    let mut duplicate = compiled::math_int_constants::Builder::with_registry(registry, 1).unwrap();
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
