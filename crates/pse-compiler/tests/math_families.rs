// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Normalized and instantiated projections preserve actual callback alternatives.
#![allow(
    clippy::unwrap_used,
    reason = "small declared mathematical family fixtures"
)]
use pse_authoring::SourceSpan;
use pse_compiler::mathir_relations::{Family, RelationSink, RelationSource, SourceFamily};
use pse_ids::{CancellationToken, FixedBudget, MemoryReserver, SemanticId};
use pse_mathir::{
    DomainRef, ExprGraph, GuardRef, NodeId, Opcode, Payload, ValueRef,
    relations::{emit_untyped, load_untyped},
};
use pse_quantity::{BoundIndexId, DomainId, QuantityTypeId, UnitId};
use pse_relations::{columnar::FieldCheckedBatch, generated::normalized::template_expr_nodes};
use pse_schema::{
    Registry,
    model::{Namespace, RelationKey},
};
use std::collections::BTreeMap;

fn id(value: u8) -> SemanticId {
    SemanticId::from_bytes([value; 16])
}
fn batches(
    reg: &Registry,
    namespace: Namespace,
    sink: RelationSink<'_>,
    budget: &dyn MemoryReserver,
    cancel: &CancellationToken,
) -> BTreeMap<RelationKey, FieldCheckedBatch> {
    let mut rows = sink.into_batches().unwrap();
    for spec in reg.relations().iter().filter(|spec| {
        spec.key.namespace == namespace
            && match namespace {
                Namespace::Normalized => {
                    spec.key.name.starts_with("template_expr_")
                        || spec.key.name.starts_with("template_math_")
                        || spec.key.name == "template_kernel_bindings"
                }
                Namespace::Inferred => {
                    spec.key.name.starts_with("math_") || spec.key.name == "kernel_bindings"
                }
                _ => false,
            }
    }) {
        rows.entry(spec.key).or_insert_with(|| {
            FieldCheckedBatch::concat_reserved(reg, spec, &[], budget, cancel).unwrap()
        });
    }
    rows
}
fn normalized() -> Family {
    Family::Normalized {
        prefix: "template",
        offset: 100,
        derivation: id(3),
        source_span: SourceSpan {
            document_id: id(4),
            start: 5,
            end: 25,
        },
    }
}
#[test]
fn normalized_reference_domains_predicates_and_pending_requests_round_trip_at_nonzero_ordinals() {
    let reg = pse_schema::catalog::assemble().unwrap();
    let budget = FixedBudget::new(512 << 20);
    let cancel = CancellationToken::new();
    let mut graph = ExprGraph::new();
    let index = BoundIndexId::from_id(id(5));
    let domain = DomainRef::Template {
        template_id: id(6),
        domain_name: "species".to_owned(),
    };
    let reference = graph
        .insert(
            Opcode::SymbolRef,
            Payload::SymbolRef {
                symbol: ValueRef::Index(index),
            },
            &[],
            None,
        )
        .unwrap();
    let gather = graph
        .insert(
            Opcode::Gather,
            Payload::PendingGather {
                group: id(7),
                indices: vec![reference],
            },
            &[],
            None,
        )
        .unwrap();
    let smooth = graph
        .insert(
            Opcode::SmoothAbs,
            Payload::PendingSmoothOp {
                eps: 2.0,
                unit: UnitId::from_id(id(8)),
            },
            &[gather],
            None,
        )
        .unwrap();
    let conversion = graph
        .insert(
            Opcode::UnitConvert,
            Payload::PendingUnitConvert {
                to: UnitId::from_id(id(9)),
            },
            &[smooth],
            None,
        )
        .unwrap();
    let root = graph
        .insert(
            Opcode::SumOver,
            Payload::Reduction {
                kind: pse_quantity::ReductionKind::Sum,
                domain,
                bound_index: index,
                filter: Some(GuardRef::Predicate {
                    source_id: id(10),
                    predicate_id: 12,
                }),
            },
            &[conversion],
            None,
        )
        .unwrap();
    let mut sink = RelationSink::new(&reg, normalized(), budget.as_ref(), &cancel);
    emit_untyped(&graph, &[root], &mut sink).unwrap();
    let rows = batches(&reg, Namespace::Normalized, sink, budget.as_ref(), &cancel);
    let source =
        RelationSource::from_checked(&rows, &reg, SourceFamily::Normalized { prefix: "template" })
            .unwrap();
    let loaded = load_untyped(&source, &[NodeId(root.0 + 100)]).unwrap();
    assert_eq!(loaded.graph.len(), graph.len());
    for (old, node) in graph.iter() {
        let mapped = loaded.node_mapping[&NodeId(old.0 + 100)];
        assert_eq!(loaded.graph.node(mapped).unwrap().payload, node.payload);
    }
}
#[test]
fn inferred_family_keeps_pending_physical_requests_and_refuses_unresolved_source_bindings() {
    let reg = pse_schema::catalog::assemble().unwrap();
    let budget = FixedBudget::new(512 << 20);
    let cancel = CancellationToken::new();
    let mut graph = ExprGraph::new();
    let symbol = graph.symbol(id(1)).unwrap();
    let root = graph
        .insert(
            Opcode::UnitConvert,
            Payload::PendingUnitConvert {
                to: UnitId::from_id(id(2)),
            },
            &[symbol],
            None,
        )
        .unwrap();
    let mut sink = RelationSink::new(&reg, Family::Inferred, budget.as_ref(), &cancel);
    emit_untyped(&graph, &[root], &mut sink).unwrap();
    let rows = batches(&reg, Namespace::Inferred, sink, budget.as_ref(), &cancel);
    let source = RelationSource::from_checked(&rows, &reg, SourceFamily::Inferred).unwrap();
    let loaded = load_untyped(&source, &[root]).unwrap();
    assert_eq!(
        loaded.graph.node(loaded.roots[0]).unwrap().payload,
        Payload::PendingUnitConvert {
            to: UnitId::from_id(id(2))
        }
    );
    let mut graph = ExprGraph::new();
    let unresolved = graph
        .insert(
            Opcode::SymbolRef,
            Payload::SymbolRef {
                symbol: ValueRef::Domain(DomainRef::Actual(DomainId::from_id(id(3)))),
            },
            &[],
            None,
        )
        .unwrap();
    assert!(
        emit_untyped(
            &graph,
            &[unresolved],
            &mut RelationSink::new(&reg, Family::Inferred, budget.as_ref(), &cancel)
        )
        .is_err()
    );
}
#[test]
fn normalized_builder_rejects_overlapping_typed_alternatives_even_with_unchanged_hashes() {
    let reg = pse_schema::catalog::assemble().unwrap();
    let budget = FixedBudget::new(512 << 20);
    let cancel = CancellationToken::new();
    let mut graph = ExprGraph::new();
    let root = graph
        .insert_typed(
            Opcode::SymbolRef,
            Payload::SymbolRef {
                symbol: ValueRef::ActualSymbol(id(1)),
            },
            &[],
            QuantityTypeId::from_id(id(2)),
            None,
        )
        .unwrap();
    let mut sink = RelationSink::new(&reg, normalized(), budget.as_ref(), &cancel);
    emit_untyped(&graph, &[root], &mut sink).unwrap();
    let batches = batches(&reg, Namespace::Normalized, sink, budget.as_ref(), &cancel);
    let key = template_expr_nodes::RELATION_KEY;
    let mut refs = template_expr_nodes::View::from_checked(&batches[&key])
        .unwrap()
        .rows()
        .unwrap();
    refs[0].payload.symbol.as_mut().unwrap().reference.index = Some(
        template_expr_nodes::NormalizedTemplateExprNodesFieldPayloadSymbolReferenceIndex {
            bound_index_id: id(99),
        },
    );
    let mut builder = template_expr_nodes::Builder::with_registry(&reg, refs.len()).unwrap();
    assert!(builder.push(refs.remove(0)).is_err());
}

#[test]
fn source_path_keeps_exact_occurrence_and_ordered_index_dependencies() {
    let reg = pse_schema::catalog::assemble().unwrap();
    let budget = FixedBudget::new(512 << 20);
    let cancel = CancellationToken::new();
    let mut graph = ExprGraph::new();
    let first = graph.int_const(42).unwrap();
    let second = graph.int_const(7).unwrap();
    let root = graph
        .insert(
            Opcode::SymbolRef,
            Payload::PendingPath {
                source_id: id(60),
                path_id: 3,
                indices: vec![second, first],
            },
            &[],
            None,
        )
        .unwrap();
    let mut sink = RelationSink::new(&reg, normalized(), budget.as_ref(), &cancel);
    emit_untyped(&graph, &[root], &mut sink).unwrap();
    let rows = batches(&reg, Namespace::Normalized, sink, budget.as_ref(), &cancel);
    let source =
        RelationSource::from_checked(&rows, &reg, SourceFamily::Normalized { prefix: "template" })
            .unwrap();
    let loaded = load_untyped(&source, &[NodeId(root.0 + 100)]).unwrap();
    let expected = Payload::PendingPath {
        source_id: id(60),
        path_id: 3,
        indices: vec![
            loaded.node_mapping[&NodeId(second.0 + 100)],
            loaded.node_mapping[&NodeId(first.0 + 100)],
        ],
    };
    assert_eq!(
        loaded.graph.node(loaded.roots[0]).unwrap().payload,
        expected
    );
    assert!(
        emit_untyped(
            &graph,
            &[root],
            &mut RelationSink::new(&reg, Family::Inferred, budget.as_ref(), &cancel)
        )
        .is_err()
    );
}
