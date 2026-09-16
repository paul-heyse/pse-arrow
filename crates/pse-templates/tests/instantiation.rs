// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Generic instance substitution checks actual values, domains and lexical occurrences.
#![allow(
    clippy::unwrap_used,
    reason = "small declared native fixtures fail at construction"
)]

use pse_ids::{CancellationToken, SemanticId};
use pse_mathir::{
    DomainRef, ExprGraph, GuardRef, NodeId, Opcode, Payload, ValueRef,
    index::DomainFacts,
    relations::{LoadedMath, VecSink, emit_untyped, load_untyped},
};
use pse_quantity::{
    BoundIndexId, BoundIndexRef, DomainId, DomainKind, IndexSet, ReductionKind, UnitId,
};
use pse_templates::{
    BindingValue, GroupBinding, InstantiationEnvironment, TemplateError, instantiate,
};
use std::collections::BTreeMap;

fn id(value: u8) -> SemanticId {
    SemanticId::from_bytes([value; 16])
}
fn load(graph: &ExprGraph, roots: &[NodeId]) -> LoadedMath {
    let mut sink = VecSink::new();
    let all = graph.iter().map(|(id, _)| id).collect::<Vec<_>>();
    emit_untyped(graph, &all, &mut sink).unwrap();
    load_untyped(&sink, roots).unwrap()
}
fn source() -> LoadedMath {
    let mut graph = ExprGraph::new();
    let left = graph.symbol(id(1)).unwrap();
    let right = graph.symbol(id(2)).unwrap();
    let root = graph.sub(left, right).unwrap();
    load(&graph, &[root, left, right])
}
fn environment(instance: u8) -> InstantiationEnvironment {
    let mut env = InstantiationEnvironment::new(id(instance));
    env.values.insert(
        ValueRef::ActualSymbol(id(1)),
        BindingValue::Symbol(id(instance + 1)),
    );
    env.values.insert(
        ValueRef::ActualSymbol(id(2)),
        BindingValue::Symbol(id(instance + 2)),
    );
    env
}
#[test]
fn two_instances_bind_actual_symbols_and_preserve_noncommutative_root_order() {
    let source = source();
    let mut graph = ExprGraph::new();
    let first = instantiate(
        &source,
        &source.roots,
        &environment(10),
        &mut graph,
        &CancellationToken::new(),
    )
    .unwrap();
    let second = instantiate(
        &source,
        &source.roots,
        &environment(20),
        &mut graph,
        &CancellationToken::new(),
    )
    .unwrap();
    assert_ne!(first.roots[0], second.roots[0]);
    for result in [&first, &second] {
        assert_eq!(
            graph.node(result.roots[0]).unwrap().children,
            result.roots[1..]
        );
        assert_eq!(graph.node(result.roots[0]).unwrap().opcode, Opcode::Sub);
    }
    assert_eq!(
        graph.node(first.roots[1]).unwrap().payload,
        Payload::SymbolRef {
            symbol: ValueRef::ActualSymbol(id(11))
        }
    );
    assert_eq!(
        graph.node(second.roots[1]).unwrap().payload,
        Payload::SymbolRef {
            symbol: ValueRef::ActualSymbol(id(21))
        }
    );
}
#[test]
fn false_branch_does_not_require_a_provider_and_unknown_guard_cannot_succeed() {
    let mut graph = ExprGraph::new();
    let selected = graph.int_const(4).unwrap();
    let absent = graph.symbol(id(99)).unwrap();
    let root = graph
        .insert(
            Opcode::Conditional,
            Payload::Conditional {
                guard: GuardRef::Predicate {
                    source_id: id(3),
                    predicate_id: 7,
                },
            },
            &[selected, absent],
            None,
        )
        .unwrap();
    let source = load(&graph, &[root]);
    let mut env = InstantiationEnvironment::new(id(5));
    let mut destination = ExprGraph::new();
    assert!(
        matches!(instantiate(&source, &source.roots, &env, &mut destination, &CancellationToken::new()),
        Err(TemplateError::GuardUndecidable { source_id, predicate: 7, .. }) if source_id == id(3))
    );
    env.predicates.insert((id(3), 7), true);
    let result = instantiate(
        &source,
        &source.roots,
        &env,
        &mut destination,
        &CancellationToken::new(),
    )
    .unwrap();
    assert_eq!(destination.len(), 1);
    assert_eq!(
        destination.node(result.roots[0]).unwrap().payload,
        Payload::IntConst { value: 4 }
    );
}
fn indexed() -> (LoadedMath, InstantiationEnvironment, BoundIndexId, DomainId) {
    let old = BoundIndexId::from_id(id(3));
    let actual = BoundIndexId::from_id(id(4));
    let domain = DomainId::from_id(id(5));
    let template_domain = DomainRef::Template {
        template_id: id(1),
        domain_name: "members".to_owned(),
    };
    let mut graph = ExprGraph::new();
    let gather = graph
        .insert(
            Opcode::Gather,
            Payload::Gather {
                group: id(2),
                coordinate_map: vec![(old, 0)],
            },
            &[],
            None,
        )
        .unwrap();
    let root = graph
        .insert(
            Opcode::SumOver,
            Payload::Reduction {
                kind: ReductionKind::Sum,
                domain: template_domain.clone(),
                bound_index: old,
                filter: None,
            },
            &[gather],
            None,
        )
        .unwrap();
    let mut env = InstantiationEnvironment::new(id(9));
    env.domain_facts.insert(
        domain,
        DomainFacts {
            kind: DomainKind::Species,
            continuous: false,
            unit: None,
            members: vec![id(6), id(7)],
        },
    );
    env.domains.insert(template_domain, domain);
    env.binders
        .insert(old, BoundIndexRef::new(actual, domain, DomainKind::Species));
    env.groups.insert(
        id(2),
        GroupBinding {
            group: id(8),
            axes: vec![domain],
            members: BTreeMap::from([(vec![id(6)], id(16)), (vec![id(7)], id(17))]),
        },
    );
    (load(&graph, &[root, gather]), env, actual, domain)
}
#[test]
fn broadcast_reuses_its_lexical_axis_at_roots_and_inside_reductions() {
    let (_, mut env, actual, domain) = indexed();
    let old = BoundIndexId::from_id(id(3));
    let template_domain = DomainRef::Template {
        template_id: id(1),
        domain_name: "members".to_owned(),
    };
    let mut source = ExprGraph::new();
    let zero = source.int_const(0).unwrap();
    let broadcast = source
        .insert(
            Opcode::Broadcast,
            Payload::Broadcast {
                domain: template_domain.clone(),
                bound_index: old,
            },
            &[zero],
            None,
        )
        .unwrap();
    let sum = source
        .insert(
            Opcode::SumOver,
            Payload::Reduction {
                kind: ReductionKind::Sum,
                domain: template_domain,
                bound_index: old,
                filter: None,
            },
            &[broadcast],
            None,
        )
        .unwrap();
    for root in [sum, broadcast] {
        if root == broadcast {
            env.free_indices
                .insert(BoundIndexRef::new(actual, domain, DomainKind::Species))
                .unwrap();
        }
        let source = load(&source, &[root]);
        let mut destination = ExprGraph::new();
        instantiate(
            &source,
            &source.roots,
            &env,
            &mut destination,
            &CancellationToken::new(),
        )
        .unwrap();
        let (_, node) = destination
            .iter()
            .find(|(_, node)| node.opcode == Opcode::Broadcast)
            .unwrap();
        assert_eq!(
            node.payload,
            Payload::Broadcast {
                domain: domain.into(),
                bound_index: actual
            }
        );
        assert_eq!(
            destination.node(node.children[0]).unwrap().payload,
            Payload::IntConst { value: 0 }
        );
    }
    env.free_indices = IndexSet::default();
    env.fixed_indices.insert(old, id(6));
    let source = load(&source, &[broadcast]);
    let mut destination = ExprGraph::new();
    let output = instantiate(
        &source,
        &source.roots,
        &env,
        &mut destination,
        &CancellationToken::new(),
    )
    .unwrap();
    assert_eq!(destination.len(), 1);
    assert_eq!(output.node_mapping[&broadcast], output.node_mapping[&zero]);
    assert_eq!(
        destination.node(output.roots[0]).unwrap().payload,
        Payload::IntConst { value: 0 }
    );
    env.fixed_indices.insert(old, id(99));
    assert!(
        instantiate(
            &source,
            &source.roots,
            &env,
            &mut ExprGraph::new(),
            &CancellationToken::new(),
        )
        .is_err()
    );
}

#[test]
fn a_reduction_rebinds_actual_axes_without_scalarizing_the_body() {
    let (source, env, actual, domain) = indexed();
    let mut graph = ExprGraph::new();
    let output = instantiate(
        &source,
        &source.roots[..1],
        &env,
        &mut graph,
        &CancellationToken::new(),
    )
    .unwrap();
    assert_eq!(graph.len(), 2);
    let root = graph.node(output.roots[0]).unwrap();
    assert_eq!(
        root.payload,
        Payload::Reduction {
            kind: ReductionKind::Sum,
            domain: DomainRef::Actual(domain),
            bound_index: actual,
            filter: None
        }
    );
    assert_eq!(
        graph.node(root.children[0]).unwrap().payload,
        Payload::Gather {
            group: id(8),
            coordinate_map: vec![(actual, 0)]
        }
    );
}
#[test]
fn global_binder_presence_cannot_hide_a_reference_outside_lexical_scope() {
    let (source, env, _, _) = indexed();
    let mut graph = ExprGraph::new();
    assert!(
        instantiate(
            &source,
            &source.roots[1..],
            &env,
            &mut graph,
            &CancellationToken::new()
        )
        .is_err()
    );
}
#[test]
fn wrong_domain_and_foreign_members_are_rejected_from_actual_contents() {
    let (source, mut env, _, domain) = indexed();
    env.groups
        .get_mut(&id(2))
        .unwrap()
        .members
        .insert(vec![id(100)], id(101));
    assert!(
        instantiate(
            &source,
            &source.roots[..1],
            &env,
            &mut ExprGraph::new(),
            &CancellationToken::new()
        )
        .is_err()
    );
    env.groups
        .get_mut(&id(2))
        .unwrap()
        .members
        .remove(&vec![id(100)]);
    env.domain_facts.get_mut(&domain).unwrap().kind = DomainKind::Phase;
    assert!(env.validate().is_err());
}
#[test]
fn target_units_and_unit_bearing_tolerances_survive_for_physical_inference() {
    let mut graph = ExprGraph::new();
    let symbol = graph.symbol(id(1)).unwrap();
    let unit = UnitId::from_id(id(7));
    let conversion = graph
        .insert(
            Opcode::UnitConvert,
            Payload::PendingUnitConvert { to: unit },
            &[symbol],
            None,
        )
        .unwrap();
    let smooth = graph
        .insert(
            Opcode::SmoothAbs,
            Payload::PendingSmoothOp { eps: 0.1, unit },
            &[conversion],
            None,
        )
        .unwrap();
    let source = load(&graph, &[smooth]);
    let mut output = ExprGraph::new();
    let instantiated = instantiate(
        &source,
        &source.roots,
        &environment(10),
        &mut output,
        &CancellationToken::new(),
    )
    .unwrap();
    let smooth = output.node(instantiated.roots[0]).unwrap();
    assert_eq!(smooth.payload, Payload::PendingSmoothOp { eps: 0.1, unit });
    assert_eq!(
        output.node(smooth.children[0]).unwrap().payload,
        Payload::PendingUnitConvert { to: unit }
    );
}
#[test]
fn literal_index_resolves_the_exact_member_and_never_a_numeric_symbol_standin() {
    let (_, mut env, _, domain) = indexed();
    env.integer_members.insert((domain, 42), id(7));
    let mut graph = ExprGraph::new();
    let coordinate = graph.int_const(42).unwrap();
    let root = graph
        .insert(
            Opcode::Gather,
            Payload::PendingGather {
                group: id(2),
                indices: vec![coordinate],
            },
            &[],
            None,
        )
        .unwrap();
    let source = load(&graph, &[root]);
    let mut output = ExprGraph::new();
    let result = instantiate(
        &source,
        &source.roots,
        &env,
        &mut output,
        &CancellationToken::new(),
    )
    .unwrap();
    assert_eq!(output.len(), 1);
    assert_eq!(
        output.node(result.roots[0]).unwrap().payload,
        Payload::SymbolRef {
            symbol: ValueRef::ActualSymbol(id(17))
        }
    );
    env.integer_members.clear();
    assert!(
        instantiate(
            &source,
            &source.roots,
            &env,
            &mut ExprGraph::new(),
            &CancellationToken::new()
        )
        .is_err()
    );
}
#[test]
fn cancellation_refuses_before_any_instantiated_output_is_available() {
    let source = source();
    let cancel = CancellationToken::new();
    cancel.cancel();
    let mut graph = ExprGraph::new();
    assert!(
        instantiate(
            &source,
            &source.roots,
            &environment(10),
            &mut graph,
            &cancel
        )
        .is_err()
    );
    assert!(graph.is_empty());
}

#[test]
fn evaluated_arithmetic_gather_preserves_free_axis_and_refuses_foreign_scope() {
    let (_, mut env, actual, domain) = indexed();
    let old = BoundIndexId::from_id(id(3));
    let mut graph = ExprGraph::new();
    let index = graph
        .insert(
            Opcode::SymbolRef,
            Payload::SymbolRef {
                symbol: ValueRef::Index(old),
            },
            &[],
            None,
        )
        .unwrap();
    let one = graph.int_const(1).unwrap();
    let expression = graph.add(index, one).unwrap();
    let root = graph
        .insert(
            Opcode::Gather,
            Payload::PendingGather {
                group: id(2),
                indices: vec![expression],
            },
            &[],
            None,
        )
        .unwrap();
    let source = load(&graph, &[root]);
    let admitted = source.roots[0];
    env.evaluated_gathers.insert(
        admitted,
        pse_templates::EvaluatedGather {
            group: GroupBinding {
                group: id(81),
                axes: vec![domain],
                members: BTreeMap::from([(vec![id(6)], id(17)), (vec![id(7)], id(16))]),
            },
            coordinates: vec![old],
        },
    );
    assert!(
        instantiate(
            &source,
            &source.roots,
            &env,
            &mut ExprGraph::new(),
            &CancellationToken::new()
        )
        .is_err()
    );
    env.free_indices
        .insert(BoundIndexRef::new(actual, domain, DomainKind::Species))
        .unwrap();
    let mut output = ExprGraph::new();
    let result = instantiate(
        &source,
        &source.roots,
        &env,
        &mut output,
        &CancellationToken::new(),
    )
    .unwrap();
    assert_eq!(output.len(), 1);
    assert_eq!(
        output.node(result.roots[0]).unwrap().payload,
        Payload::Gather {
            group: id(81),
            coordinate_map: vec![(actual, 0)]
        }
    );
    env.evaluated_gathers
        .get_mut(&admitted)
        .unwrap()
        .group
        .members
        .insert(vec![id(99)], id(17));
    assert!(
        instantiate(
            &source,
            &source.roots,
            &env,
            &mut ExprGraph::new(),
            &CancellationToken::new()
        )
        .is_err()
    );
}

#[test]
fn distinct_child_occurrences_retain_identity_and_index_node_order() {
    let mut graph = ExprGraph::new();
    let left = graph
        .insert(
            Opcode::SymbolRef,
            Payload::PendingPath {
                source_id: id(40),
                path_id: 0,
                indices: vec![],
            },
            &[],
            None,
        )
        .unwrap();
    let right = graph
        .insert(
            Opcode::SymbolRef,
            Payload::PendingPath {
                source_id: id(40),
                path_id: 1,
                indices: vec![],
            },
            &[],
            None,
        )
        .unwrap();
    assert_ne!(left, right);
    let root = graph.sub(left, right).unwrap();
    let source = load(&graph, &[root]);
    let mut env = InstantiationEnvironment::new(id(10));
    env.paths.insert(
        (id(40), 0),
        pse_templates::PathBinding::Value(BindingValue::Symbol(id(11))),
    );
    env.paths.insert(
        (id(40), 1),
        pse_templates::PathBinding::Value(BindingValue::Symbol(id(12))),
    );
    let mut output = ExprGraph::new();
    let result = instantiate(
        &source,
        &source.roots,
        &env,
        &mut output,
        &CancellationToken::new(),
    )
    .unwrap();
    let root = output.node(result.roots[0]).unwrap();
    assert_eq!(
        output.node(root.children[0]).unwrap().payload,
        Payload::SymbolRef {
            symbol: ValueRef::ActualSymbol(id(11))
        }
    );
    assert_eq!(
        output.node(root.children[1]).unwrap().payload,
        Payload::SymbolRef {
            symbol: ValueRef::ActualSymbol(id(12))
        }
    );
    env.paths.remove(&(id(40), 1));
    assert!(
        instantiate(
            &source,
            &source.roots,
            &env,
            &mut ExprGraph::new(),
            &CancellationToken::new()
        )
        .is_err()
    );
}

#[test]
fn a_mixed_coordinate_projection_keeps_exact_providers_and_remaining_axis_order() {
    let (_, mut env, actual, species) = indexed();
    let phase = DomainId::from_id(id(50));
    env.domain_facts.insert(
        phase,
        DomainFacts {
            kind: DomainKind::Phase,
            continuous: false,
            unit: None,
            members: vec![id(51), id(52)],
        },
    );
    env.integer_members.insert((phase, 4), id(52));
    let old = *env.binders.keys().next().unwrap();
    let binding = *env.binders.get(&old).unwrap();
    env.free_indices.insert(binding).unwrap();
    env.groups.insert(
        id(2),
        GroupBinding {
            group: id(8),
            axes: vec![phase, species],
            members: BTreeMap::from([
                (vec![id(51), id(6)], id(61)),
                (vec![id(51), id(7)], id(62)),
                (vec![id(52), id(6)], id(63)),
                (vec![id(52), id(7)], id(64)),
            ]),
        },
    );
    let mut graph = ExprGraph::new();
    let phase_index = graph.int_const(4).unwrap();
    let species_index = graph
        .insert(
            Opcode::SymbolRef,
            Payload::SymbolRef {
                symbol: ValueRef::Index(old),
            },
            &[],
            None,
        )
        .unwrap();
    let root = graph
        .insert(
            Opcode::Gather,
            Payload::PendingGather {
                group: id(2),
                indices: vec![phase_index, species_index],
            },
            &[],
            None,
        )
        .unwrap();
    let source = load(&graph, &[root]);
    let mut output = ExprGraph::new();
    let result = instantiate(
        &source,
        &source.roots,
        &env,
        &mut output,
        &CancellationToken::new(),
    )
    .unwrap();
    let projection = result.projected_groups.values().next().unwrap();
    assert_eq!(projection.group.axes, vec![species]);
    assert_eq!(
        projection.group.members,
        BTreeMap::from([(vec![id(6)], id(63)), (vec![id(7)], id(64))])
    );
    assert_eq!(projection.fixed_coordinates, vec![(0, id(52))]);
    assert_eq!(
        output.node(result.roots[0]).unwrap().payload,
        Payload::Gather {
            group: projection.group.group,
            coordinate_map: vec![(actual, 0)]
        }
    );
}

#[test]
fn an_optional_coefficient_can_remain_unused_but_cannot_be_read_unbound() {
    let mut graph = ExprGraph::new();
    let unused = graph.int_const(4).unwrap();
    let parameter = graph.symbol(id(2)).unwrap();
    let source = load(&graph, &[unused, parameter]);
    let mut env = InstantiationEnvironment::new(id(10));
    env.values
        .insert(ValueRef::ActualSymbol(id(2)), BindingValue::Symbol(id(20)));
    env.unbound_parameters.insert(id(20));
    let cancel = CancellationToken::new();
    assert!(
        instantiate(
            &source,
            &source.roots[..1],
            &env,
            &mut ExprGraph::new(),
            &cancel
        )
        .is_ok()
    );
    assert!(
        instantiate(
            &source,
            &source.roots[1..],
            &env,
            &mut ExprGraph::new(),
            &cancel
        )
        .is_err()
    );
    env.values.insert(
        ValueRef::ActualSymbol(id(2)),
        BindingValue::Literal {
            payload: Payload::IntConst { value: 7 },
            quantity: None,
        },
    );
    let mut output = ExprGraph::new();
    let bound = instantiate(&source, &source.roots[1..], &env, &mut output, &cancel).unwrap();
    assert_eq!(
        output.node(bound.roots[0]).unwrap().payload,
        Payload::IntConst { value: 7 }
    );
}
