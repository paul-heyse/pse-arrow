// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Conservation construction preserves descriptor order and remaining free coordinates.
#![allow(
    clippy::unwrap_used,
    reason = "explicit mathematical test fixture construction"
)]
use pse_ids::{CancellationToken, SemanticId};
use pse_mathir::{
    ExprGraph, Opcode, Payload, ValueRef,
    relations::{LoadedMath, VecSink, emit_untyped, load_untyped},
};
use pse_quantity::IndexSet;
use pse_templates::{
    BindingValue, InstantiationEnvironment,
    laws::{ConservationTerm, conservation},
};

fn id(value: u8) -> SemanticId {
    SemanticId::from_bytes([value; 16])
}
fn source() -> LoadedMath {
    let mut graph = ExprGraph::new();
    let a = graph.symbol(id(1)).unwrap();
    let b = graph.symbol(id(2)).unwrap();
    let mut sink = VecSink::new();
    emit_untyped(&graph, &[a, b], &mut sink).unwrap();
    load_untyped(&sink, &[a, b]).unwrap()
}
fn terms(source: &LoadedMath) -> Vec<ConservationTerm> {
    let mut env = InstantiationEnvironment::new(id(3));
    for value in [1, 2] {
        env.values.insert(
            ValueRef::ActualSymbol(id(value)),
            BindingValue::Symbol(id(value)),
        );
    }
    source
        .roots
        .iter()
        .enumerate()
        .map(|(index, root)| ConservationTerm {
            contribution: id(u8::try_from(index + 5).unwrap()),
            root: *root,
            environment: env.clone(),
            reductions: Vec::new(),
            broadcasts: Vec::new(),
            sign: if index == 0 { 1 } else { -1 },
            conversion: None,
        })
        .collect()
}
#[test]
fn signed_descriptors_retain_order_without_numerical_aggregation() {
    let source = source();
    let terms = terms(&source);
    let mut graph = ExprGraph::new();
    let body = conservation(
        &source,
        id(3),
        &IndexSet::new(),
        &terms,
        &mut graph,
        &CancellationToken::new(),
    )
    .unwrap();
    let node = graph.node(body.body).unwrap();
    assert_eq!(node.opcode, Opcode::Affine);
    let Payload::Affine {
        constant, terms, ..
    } = &node.payload
    else {
        panic!("conservation must produce an affine body")
    };
    assert_eq!(constant.to_bits(), 0.0_f64.to_bits());
    assert_eq!(
        terms
            .iter()
            .map(|term| term.coefficient)
            .collect::<Vec<_>>(),
        [1.0, -1.0]
    );
    for (term, value) in terms.iter().zip([1, 2]) {
        assert_eq!(
            graph.node(term.child).unwrap().payload,
            Payload::SymbolRef {
                symbol: ValueRef::ActualSymbol(id(value))
            }
        );
    }
}
#[test]
fn duplicate_contribution_and_invalid_orientation_cannot_emit_a_balance() {
    let source = source();
    let mut terms = terms(&source);
    terms[1].contribution = terms[0].contribution;
    assert!(
        conservation(
            &source,
            id(3),
            &IndexSet::new(),
            &terms,
            &mut ExprGraph::new(),
            &CancellationToken::new()
        )
        .is_err()
    );
    terms[1].contribution = id(9);
    terms[1].sign = 0;
    assert!(
        conservation(
            &source,
            id(3),
            &IndexSet::new(),
            &terms,
            &mut ExprGraph::new(),
            &CancellationToken::new()
        )
        .is_err()
    );
}
#[test]
fn missing_law_coordinate_and_cancellation_are_explicit_refusals() {
    let source = source();
    let terms = terms(&source);
    let cancel = CancellationToken::new();
    cancel.cancel();
    let mut graph = ExprGraph::new();
    assert!(
        conservation(
            &source,
            id(3),
            &IndexSet::new(),
            &terms,
            &mut graph,
            &cancel
        )
        .is_err()
    );
    assert!(graph.is_empty());
    let free = IndexSet::try_from_iter([pse_quantity::BoundIndexRef::new(
        pse_quantity::BoundIndexId::from_id(id(10)),
        pse_quantity::DomainId::from_id(id(11)),
        pse_quantity::DomainKind::Species,
    )])
    .unwrap();
    assert!(
        conservation(
            &source,
            id(3),
            &free,
            &terms,
            &mut graph,
            &CancellationToken::new()
        )
        .is_err()
    );
    assert!(graph.is_empty());
}

#[test]
fn intensive_equality_uses_subtraction_in_orientation_order() {
    let source = source();
    let mut terms = terms(&source);
    terms.reverse();
    let mut graph = ExprGraph::new();
    let result = pse_templates::laws::equality(
        &source,
        id(3),
        &IndexSet::new(),
        &terms,
        &mut graph,
        &CancellationToken::new(),
    )
    .unwrap();
    let node = graph.node(result.body).unwrap();
    assert_eq!(node.opcode, Opcode::Sub);
    for (child, value) in node.children.iter().zip([1, 2]) {
        assert_eq!(
            graph.node(*child).unwrap().payload,
            Payload::SymbolRef {
                symbol: ValueRef::ActualSymbol(id(value))
            }
        );
    }
}

#[test]
fn named_physical_conversion_retains_exact_selection_on_the_operand_edge() {
    let source = source();
    let mut terms = terms(&source);
    let conversion = pse_quantity::ConversionId::from_id(id(10));
    let spec = pse_quantity::UnitConvertSpec {
        from: pse_quantity::UnitId::from_id(id(11)),
        to: pse_quantity::UnitId::from_id(id(12)),
        scale: 2.0,
        offset: 3.0,
    };
    terms[0].conversion = Some((conversion, spec));
    let mut graph = ExprGraph::new();
    let result = conservation(
        &source,
        id(3),
        &IndexSet::new(),
        &terms,
        &mut graph,
        &CancellationToken::new(),
    )
    .unwrap();
    let [selection] = result.selections.as_slice() else {
        panic!("one explicit conversion must produce one selection")
    };
    assert_eq!(selection.conversions, [(0, conversion)]);
    assert_eq!(
        selection.builtin,
        Some(pse_quantity::infer::BuiltInRule::UnitConvert)
    );
    assert_eq!(
        graph.node(selection.node).unwrap().payload,
        Payload::UnitConvert(spec)
    );
}

#[test]
fn explicit_law_broadcast_adds_one_axis_without_replicating_scalar_equations() {
    use pse_mathir::{DomainRef, index::DomainFacts};
    use pse_quantity::{BoundIndexId, BoundIndexRef, DomainId, DomainKind};
    let source = source();
    let mut terms = terms(&source);
    let domain = DomainId::from_id(id(20));
    let index = BoundIndexRef::new(BoundIndexId::from_id(id(21)), domain, DomainKind::Stage);
    for term in &mut terms {
        term.environment.domain_facts.insert(
            domain,
            DomainFacts {
                kind: DomainKind::Stage,
                continuous: false,
                unit: None,
                members: vec![id(22), id(23)],
            },
        );
        term.broadcasts = vec![index];
    }
    let free = IndexSet::try_from_iter([index]).unwrap();
    let mut graph = ExprGraph::new();
    let result = conservation(
        &source,
        id(3),
        &free,
        &terms,
        &mut graph,
        &CancellationToken::new(),
    )
    .unwrap();
    let Payload::Affine { terms: parts, .. } = &graph.node(result.body).unwrap().payload else {
        panic!("indexed law")
    };
    assert_eq!(parts.len(), 2);
    for part in parts {
        assert_eq!(
            graph.node(part.child).unwrap().payload,
            Payload::Broadcast {
                domain: DomainRef::Actual(domain),
                bound_index: index.bound_index
            }
        );
    }
    assert!(result.projected_groups.is_empty());
    terms[0].broadcasts.push(index);
    assert!(
        conservation(
            &source,
            id(3),
            &free,
            &terms,
            &mut ExprGraph::new(),
            &CancellationToken::new()
        )
        .is_err()
    );
}
#[test]
fn fixed_phase_law_slice_retains_actual_species_group_providers() {
    use pse_mathir::index::DomainFacts;
    use pse_quantity::{BoundIndexId, BoundIndexRef, DomainId, DomainKind};
    use pse_templates::GroupBinding;
    use std::collections::BTreeMap;
    let phase = DomainId::from_id(id(30));
    let species = DomainId::from_id(id(31));
    let p = BoundIndexId::from_id(id(32));
    let j = BoundIndexId::from_id(id(33));
    let mut graph = ExprGraph::new();
    let root = graph
        .insert(
            Opcode::Gather,
            Payload::Gather {
                group: id(34),
                coordinate_map: vec![(p, 0), (j, 1)],
            },
            &[],
            None,
        )
        .unwrap();
    let mut sink = VecSink::new();
    emit_untyped(&graph, &[root], &mut sink).unwrap();
    let source = load_untyped(&sink, &[root]).unwrap();
    let mut env = InstantiationEnvironment::new(id(3));
    for (domain, kind, members) in [
        (phase, DomainKind::Phase, vec![id(40), id(41)]),
        (species, DomainKind::Species, vec![id(42), id(43)]),
    ] {
        env.domain_facts.insert(
            domain,
            DomainFacts {
                kind,
                continuous: false,
                unit: None,
                members,
            },
        );
    }
    env.binders
        .insert(p, BoundIndexRef::new(p, phase, DomainKind::Phase));
    let species_index = BoundIndexRef::new(j, species, DomainKind::Species);
    env.binders.insert(j, species_index);
    env.fixed_indices.insert(p, id(41));
    env.free_indices = IndexSet::try_from_iter([species_index]).unwrap();
    env.groups.insert(
        id(34),
        GroupBinding {
            group: id(34),
            axes: vec![phase, species],
            members: BTreeMap::from([
                (vec![id(40), id(42)], id(50)),
                (vec![id(40), id(43)], id(51)),
                (vec![id(41), id(42)], id(52)),
                (vec![id(41), id(43)], id(53)),
            ]),
        },
    );
    let free = env.free_indices.clone();
    let term = ConservationTerm {
        contribution: id(60),
        root: source.roots[0],
        environment: env,
        reductions: vec![],
        broadcasts: vec![],
        sign: 1,
        conversion: None,
    };
    let mut out = ExprGraph::new();
    let result = conservation(
        &source,
        id(3),
        &free,
        &[term],
        &mut out,
        &CancellationToken::new(),
    )
    .unwrap();
    assert_eq!(result.projected_groups.len(), 1);
    let projection = result.projected_groups.values().next().unwrap();
    assert_eq!(projection.source_group, id(34));
    assert_eq!(projection.fixed_coordinates, vec![(0, id(41))]);
    assert_eq!(projection.group.axes, vec![species]);
    assert_eq!(
        projection.group.members,
        BTreeMap::from([(vec![id(42)], id(52)), (vec![id(43)], id(53))])
    );
    assert_eq!(
        out.iter()
            .filter(|(_, node)| node.opcode == Opcode::SumOver)
            .count(),
        0,
        "a fixed phase is sliced, never summed"
    );
}
