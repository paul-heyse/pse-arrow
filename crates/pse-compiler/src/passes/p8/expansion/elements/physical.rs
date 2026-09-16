// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Every broadcast, product and reduction is checked by the ordinary quantity engine.
use super::{
    BoundIndexRef, CompilerError, Context, DomainKind, ExprGraph, IndexSet, NodeId, Opcode,
    Payload, QuantityTypeId, SemanticId, invalid,
};
use pse_mathir::{DomainRef, relations::vec_sink::QuantitySelection};
use pse_quantity::{
    OperationId, ReductionKind,
    infer::{OpRequest, Operand, OperationSelection, infer_with_evidence},
};

pub(super) struct Value {
    pub root: NodeId,
    pub quantity: QuantityTypeId,
    pub indices: IndexSet,
}
pub(super) fn broadcast(
    context: &Context<'_, '_>,
    value: &mut Value,
    target: &IndexSet,
    owner: SemanticId,
    graph: &mut ExprGraph,
) -> Result<(), CompilerError> {
    for index in target
        .iter()
        .filter(|index| !value.indices.contains(index))
        .copied()
        .collect::<Vec<_>>()
    {
        let typed = infer_with_evidence(
            &OpRequest::Broadcast { index },
            &[Operand {
                quantity_type: value.quantity,
                indices: &value.indices,
            }],
            context.physical,
            context.checker,
        )?;
        value.root = graph.insert(
            Opcode::Broadcast,
            Payload::Broadcast {
                domain: DomainRef::Actual(index.domain),
                bound_index: index.bound_index,
            },
            &[value.root],
            Some(owner),
        )?;
        value.quantity = typed.result;
        value.indices = typed.indices;
    }
    Ok(())
}
pub(super) fn multiply(
    context: &Context<'_, '_>,
    flow: &Value,
    coefficient: &Value,
    expected: OperationId,
    owner: SemanticId,
    graph: &mut ExprGraph,
) -> Result<(Value, QuantitySelection), CompilerError> {
    let typed = infer_with_evidence(
        &OpRequest::Mul,
        &[
            Operand {
                quantity_type: flow.quantity,
                indices: &flow.indices,
            },
            Operand {
                quantity_type: coefficient.quantity,
                indices: &coefficient.indices,
            },
        ],
        context.physical,
        context.checker,
    )?;
    let OperationSelection::Registered {
        operation,
        operand_permutation,
    } = typed.selected
    else {
        return Err(invalid(
            "element projection requires its declared physical multiplication",
        ));
    };
    if operation != expected || !typed.conversions.is_empty() {
        return Err(invalid(
            "element multiplication disagrees with its explicit source contract",
        ));
    }
    let root = graph.insert(
        Opcode::Mul,
        Payload::None,
        &[flow.root, coefficient.root],
        Some(owner),
    )?;
    Ok((
        Value {
            root,
            quantity: typed.result,
            indices: typed.indices,
        },
        QuantitySelection {
            node: root,
            operation: Some(operation),
            builtin: None,
            permutation: operand_permutation,
            conversions: vec![],
            deferred_static_check: false,
        },
    ))
}
pub(super) fn reduce(
    context: &Context<'_, '_>,
    value: &mut Value,
    reductions: &[BoundIndexRef],
    owner: SemanticId,
    graph: &mut ExprGraph,
) -> Result<(), CompilerError> {
    for bound in reductions {
        if bound.kind == DomainKind::Element {
            return Err(invalid("element projection cannot reduce its Element axis"));
        }
        let typed = infer_with_evidence(
            &OpRequest::Reduce {
                kind: ReductionKind::Sum,
                bound: *bound,
            },
            &[Operand {
                quantity_type: value.quantity,
                indices: &value.indices,
            }],
            context.physical,
            context.checker,
        )?;
        value.root = graph.insert(
            Opcode::SumOver,
            Payload::Reduction {
                kind: ReductionKind::Sum,
                domain: DomainRef::Actual(bound.domain),
                bound_index: bound.bound_index,
                filter: None,
            },
            &[value.root],
            Some(owner),
        )?;
        value.quantity = typed.result;
        value.indices = typed.indices;
    }
    Ok(())
}
