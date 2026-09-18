// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Graph-cycle and exact derivative algebra at the typed mathematical boundary.

use super::invalid;
use crate::{CompilerError, quantity_relations::RelationSymbolSource};
use pse_ids::{CancellationToken, SemanticId};
use pse_mathir::{NodeId, Payload, ValueRef, infer::SymbolTypeSource, relations::LoadedMath};
use pse_quantity::{
    DomainId, IndexSet, QuantityRegistry,
    infer::{OpRequest, Operand, infer_with_evidence},
};
use pse_relations::generated::{compiled, inferred};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn validate(
    expressions: &[compiled::symbol_expressions::Row],
    links: &[inferred::math_dae_links::Row],
    loaded: &LoadedMath,
    source: &RelationSymbolSource,
    quantities: &QuantityRegistry,
    cancel: &CancellationToken,
) -> Result<(), CompilerError> {
    // The actual registered input producer owns symbol/body correspondence. This map is
    // the specialized cycle algorithm's index, not a second relation validator.
    let bodies: BTreeMap<SemanticId, NodeId> = expressions
        .iter()
        .map(|row| (row.symbol_id, NodeId(row.node_id)))
        .collect();
    let mut dependencies = BTreeMap::new();
    for (symbol, root) in &bodies {
        cancel.checkpoint()?;
        let mut pending = vec![*root];
        let mut visited = BTreeSet::new();
        let mut reads = BTreeSet::new();
        while let Some(id) = pending.pop() {
            if !visited.insert(id) {
                continue;
            }
            let node = loaded.graph.node(id)?;
            pending.extend(&node.children);
            pending.extend(node.payload.referenced_nodes());
            match &node.payload {
                Payload::SymbolRef {
                    symbol: ValueRef::ActualSymbol(id),
                } => {
                    if bodies.contains_key(id) {
                        reads.insert(*id);
                    }
                }
                Payload::Gather { group, .. } => {
                    let group = source
                        .group(*group)
                        .ok_or_else(|| invalid("expression group dependency is absent"))?;
                    reads.extend(
                        group
                            .members
                            .values()
                            .filter(|id| bodies.contains_key(id))
                            .copied(),
                    );
                }
                _ => {}
            }
        }
        dependencies.insert(*symbol, reads);
    }
    let mut complete = BTreeSet::new();
    for start in bodies.keys() {
        if complete.contains(start) {
            continue;
        }
        let mut stack = vec![(*start, false)];
        let mut active = Vec::new();
        while let Some((symbol, exit)) = stack.pop() {
            cancel.checkpoint()?;
            if exit {
                active.pop();
                complete.insert(symbol);
                continue;
            }
            if complete.contains(&symbol) {
                continue;
            }
            if let Some(position) = active.iter().position(|id| *id == symbol) {
                let path = active[position..]
                    .iter()
                    .chain(std::iter::once(&symbol))
                    .map(|id| bodies[id])
                    .collect();
                return Err(pse_mathir::MathIrError::Cycle { path }.into());
            }
            active.push(symbol);
            stack.push((symbol, true));
            stack.extend(dependencies[&symbol].iter().rev().map(|id| (*id, false)));
        }
    }
    derivatives(links, source, quantities, cancel)
}

fn derivatives(
    links: &[inferred::math_dae_links::Row],
    source: &RelationSymbolSource,
    quantities: &QuantityRegistry,
    cancel: &CancellationToken,
) -> Result<(), CompilerError> {
    for link in links {
        cancel.checkpoint()?;
        let domain = DomainId::from_id(link.wrt_domain_id);
        let facts = source
            .domain(domain)
            .ok_or_else(|| invalid("derivative domain absent"))?;
        if !facts.continuous || link.derivative_order == 0 {
            return Err(invalid(
                "derivative requires a continuous coordinate and positive order",
            ));
        }
        let unit = facts
            .unit
            .ok_or_else(|| invalid("derivative coordinate has no physical unit"))?;
        let state = source
            .symbol_type(link.state_symbol_id)
            .ok_or_else(|| invalid("derivative state physical type absent"))?;
        let derivative = source
            .symbol_type(link.derivative_symbol_id)
            .ok_or_else(|| invalid("derivative physical type absent"))?;
        let indices = IndexSet::new();
        let inferred = infer_with_evidence(
            &OpRequest::Derivative {
                domain_unit: unit,
                domain_kind: facts.kind,
                order: u8::try_from(link.derivative_order)
                    .map_err(|_| invalid("derivative order exceeds u8"))?,
            },
            &[Operand {
                quantity_type: state,
                indices: &indices,
            }],
            quantities,
            source.invariant_checker(NodeId(0)),
        )?;
        pse_quantity::admission::require_same_contract(inferred.result, derivative, quantities)?;
    }
    Ok(())
}
