// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The primitive relation adapters preserve ordered structure and reject invalid actual rows.
#![allow(clippy::unwrap_used, reason = "fixture assertions")]
use pse_compiler::mathir_relations::{Family, RelationSink, RelationSource};
use pse_ids::SemanticId;
use pse_mathir::{ExprGraph, Opcode, Payload, TemplateValueKind, ValueRef};
use std::collections::BTreeMap;

#[test]
fn relation_round_trip_preserves_children_and_rejects_duplicate_payload_rows() {
    let registry = pse_schema::registry().unwrap();
    let mut graph = ExprGraph::new();
    let first = graph.int_const(2).unwrap();
    let second = graph.symbol(SemanticId::from_bytes([4; 16])).unwrap();
    let root = graph.sub(first, second).unwrap();
    let mut sink = RelationSink::new(registry, Family::Compiled);
    pse_mathir::relations::emit_untyped(&graph, &[root], &mut sink).unwrap();
    let mut rows = sink
        .into_rows()
        .into_iter()
        .map(|(key, rows)| {
            let spec = registry.relation(&key.qualified_name()).unwrap();
            (
                key,
                pse_relations::cells::batch_from_cells(registry, spec, &rows).unwrap(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let source = RelationSource::from_batches(&rows, registry).unwrap();
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
    let mut constants =
        pse_relations::cells::cells_from_batch(registry, spec, &rows[&spec.key]).unwrap();
    constants.push(constants[0].clone());
    rows.insert(
        spec.key,
        pse_relations::cells::batch_from_cells(registry, spec, &constants).unwrap(),
    );
    let source = RelationSource::from_batches(&rows, registry).unwrap();
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
    let mut sink = RelationSink::new(registry, Family::Compiled);
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
    let mut sink = RelationSink::new(registry, Family::Compiled);
    assert!(pse_mathir::relations::emit_untyped(&graph, &[request], &mut sink).is_err());
}
