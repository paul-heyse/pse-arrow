// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Allocation accounting for resolved products. Shared finite cells may be counted conservatively.
use super::*;
fn total(values: impl IntoIterator<Item = usize>) -> usize {
    values.into_iter().fold(0, usize::saturating_add)
}
fn vector<T>(capacity: usize) -> usize {
    capacity.saturating_mul(size_of::<T>())
}
pub(crate) fn ty(value: &Type) -> usize {
    crate::extent::ty(value).saturating_add(refinements(value))
}
fn refinements(value: &Type) -> usize {
    match value {
        Type::RefinedQuantity {
            refinement:
                crate::PhysicalRefinement::Transfer {
                    boundary: crate::BoundaryRef::Bound { coordinates, .. },
                    ..
                },
            ..
        } => vector::<crate::specialize::Value>(coordinates.capacity()).saturating_add(total(
            coordinates
                .iter()
                .map(crate::specialize::Value::retained_bytes),
        )),
        Type::Set(value)
        | Type::Continuous(_, value)
        | Type::Optional(value)
        | Type::Indexed { element: value, .. } => refinements(value),
        Type::Tuple(values) => total(values.iter().map(refinements)),
        Type::Function { arguments, result } => {
            total(arguments.iter().map(|(_, value)| refinements(value)))
                .saturating_add(refinements(result))
        }
        _ => 0,
    }
}
fn source(value: &Source) -> usize {
    value.field.role.capacity()
}
fn node(value: &Node) -> usize {
    match value {
        Node::Number(number) => number
            .unit
            .as_ref()
            .map_or(0, pse_quantity::UnitProduct::retained_bytes),
        Node::Reference(Reference::Indexed { indices, .. }) => vector::<NodeIx>(indices.capacity()),
        Node::Call { args, .. } | Node::Apply { args, .. } | Node::Provider { args, .. } => {
            vector::<NodeIx>(args.capacity())
        }
        Node::Partial { wrt, args, .. } => {
            vector::<usize>(wrt.capacity()).saturating_add(vector::<NodeIx>(args.capacity()))
        }
        Node::Let { bindings, .. } => vector::<BinderIx>(bindings.capacity()),
        Node::Sequence { prerequisites, .. } => vector::<NodeIx>(prerequisites.capacity()),
        _ => 0,
    }
}
pub(super) fn syntax(value: &ResolvedSyntax) -> usize {
    total([
        size_of::<ResolvedSyntax>(),
        value.symbols.retained_bytes(),
        arena(&value.arena),
        vector::<NodeIx>(value.outputs.capacity()),
        vector::<PredicateIx>(value.validity.capacity()),
    ])
}
pub(super) fn arena(value: &Arena) -> usize {
    total([
        size_of::<Arena>(),
        vector::<Node>(value.nodes.capacity()),
        total(value.nodes.iter().map(node)),
        vector::<ResolvedPredicate>(value.predicates.capacity()),
        vector::<Binding>(value.binders.capacity()),
        vector::<FunctionSelection>(value.functions.capacity()),
        total(value.functions.iter().map(|function| {
            vector::<String>(function.formals.capacity())
                .saturating_add(total(function.formals.iter().map(String::capacity)))
        })),
        vector::<Carrier>(value.carriers.capacity()),
        total(value.carriers.iter().map(|carrier| {
            ty(&carrier.element).saturating_add(total(
                carrier
                    .members
                    .iter()
                    .map(crate::specialize::Value::retained_bytes),
            ))
        })),
        vector::<IndexedGroup>(value.groups.capacity()),
        total(value.groups.iter().map(|group| {
            total([
                vector::<CarrierIx>(group.axes.capacity()),
                vector::<(Vec<crate::specialize::Value>, Option<SymbolIx>)>(
                    group.members.capacity(),
                ),
                total(group.members.iter().map(|(tuple, _)| {
                    vector::<crate::specialize::Value>(tuple.capacity()).saturating_add(total(
                        tuple.iter().map(crate::specialize::Value::retained_bytes),
                    ))
                })),
            ])
        })),
        vector::<EvidenceUse>(value.evidence.capacity()),
        total(value.evidence.iter().map(|evidence| total([
            evidence.node.retained_bytes(),
            vector::<NodeIx>(evidence.inputs.capacity()),
            vector::<PredicateIx>(evidence.predicates.capacity()),
        ]))),
        value.member_ranges.len().saturating_mul(size_of::<(SymbolIx, MemberRange)>() + 96),
        value.finite_folds.len().saturating_mul(size_of::<(NodeIx, FiniteFold)>() + 96),
        total(value.finite_folds.values().map(|fold| total([
            vector::<crate::specialize::Value>(fold.selected.len()),
            total(fold.selected.iter().map(crate::specialize::Value::retained_bytes)),
            ty(&fold.prototype),
        ]))),
        value
            .node_sources
            .len()
            .saturating_mul(size_of::<(NodeIx, Source)>().saturating_add(96)),
        total(value.node_sources.values().map(source)),
        value
            .predicate_sources
            .len()
            .saturating_mul(size_of::<(PredicateIx, Source)>().saturating_add(96)),
        total(value.predicate_sources.values().map(source)),
    ])
}
pub(super) fn inference(value: &pse_quantity::ResolvedInference) -> usize {
    total([
        vector::<pse_quantity::ResolvedPhysicalContract>(value.operands.capacity()),
        total(
            value
                .operands
                .iter()
                .map(pse_quantity::ResolvedPhysicalContract::heap_bytes),
        ),
        value.result.heap_bytes(),
        vector::<pse_quantity::infer::OperandConversion>(value.conversions.capacity()),
        vector::<f64>(value.operand_scales.capacity()),
        match &value.selected {
            pse_quantity::infer::OperationSelection::Registered {
                operand_permutation,
                ..
            } => vector::<u16>(operand_permutation.capacity()),
            _ => 0,
        },
    ])
}
