// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Acyclicity and deterministic storage ordering (blueprint §7.4).
use crate::{ExprGraph, MathIrError, Node, NodeId};
use std::collections::{BTreeMap, BTreeSet};

/// Deterministic post-order from caller-ordered roots, preserving all argument orders.
/// Storage ordering never authorizes evaluating an excluded conditional branch.
///
/// # Errors
/// Rejects absent roots or dangling/cyclic references.
pub fn postorder(graph: &ExprGraph, roots: &[NodeId]) -> Result<Vec<NodeId>, MathIrError> {
    traverse(roots, |id| graph.node(id))
}
/// Traverse a graph with the ordered input references stored in kernel-binding rows.
///
/// # Errors
/// Rejects missing bindings, dangling inputs or cycles through external binding inputs.
pub fn postorder_with_bindings(
    graph: &ExprGraph,
    roots: &[NodeId],
    bindings: &BTreeMap<pse_ids::SemanticId, crate::relations::vec_sink::KernelBinding>,
) -> Result<Vec<NodeId>, MathIrError> {
    let mut nodes: BTreeMap<_, _> = graph.iter().map(|(id, node)| (id, node.clone())).collect();
    for (id, node) in &mut nodes {
        if let crate::Payload::KernelCall { kernel_binding, .. } = node.payload {
            let binding = bindings
                .get(&kernel_binding)
                .ok_or(MathIrError::UnknownBinding {
                    node: *id,
                    binding: kernel_binding,
                })?;
            node.children
                .extend(binding.inputs.iter().map(|(_, input)| *input));
        }
    }
    traverse(roots, |id| {
        nodes
            .get(&id)
            .ok_or_else(|| MathIrError::malformed_at(id, "missing kernel dependency"))
    })
}
/// Check a relation-loaded graph, including unreachable components and payload references.
///
/// # Errors
/// Rejects dangling references and returns a concrete closed cycle path.
pub fn validate_nodes(nodes: &BTreeMap<NodeId, Node>) -> Result<Vec<NodeId>, MathIrError> {
    let roots: Vec<_> = nodes.keys().copied().collect();
    traverse(&roots, |id| {
        nodes
            .get(&id)
            .ok_or_else(|| MathIrError::malformed_at(id, "missing node in loaded graph"))
    })
}
fn traverse<'a>(
    roots: &[NodeId],
    mut node: impl FnMut(NodeId) -> Result<&'a Node, MathIrError>,
) -> Result<Vec<NodeId>, MathIrError> {
    let mut done = BTreeSet::new();
    let mut active = BTreeMap::new();
    let mut path = Vec::new();
    let mut output = Vec::new();
    for root in roots {
        let mut stack = vec![(*root, false)];
        while let Some((id, exiting)) = stack.pop() {
            if exiting {
                active.remove(&id);
                path.pop();
                done.insert(id);
                output.push(id);
                continue;
            }
            if done.contains(&id) {
                continue;
            }
            if let Some(start) = active.get(&id).copied() {
                let mut cycle = path[start..].to_vec();
                cycle.push(id);
                return Err(MathIrError::Cycle { path: cycle });
            }
            let value = node(id)?;
            active.insert(id, path.len());
            path.push(id);
            stack.push((id, true));
            let dependencies = value
                .children
                .iter()
                .copied()
                .chain(value.payload.referenced_nodes())
                .collect::<Vec<_>>();
            for dependency in dependencies.into_iter().rev() {
                stack.push((dependency, false));
            }
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Opcode, Payload};
    fn node(children: Vec<NodeId>) -> Node {
        Node {
            opcode: Opcode::Add,
            payload: Payload::None,
            children,
            quantity_type: None,
            scope: None,
        }
    }
    #[test]
    fn loaded_cycles_report_the_actual_path_even_through_a_guard() {
        let mut nodes = BTreeMap::from([
            (NodeId(0), node(vec![NodeId(1)])),
            (NodeId(1), node(vec![])),
        ]);
        nodes.get_mut(&NodeId(1)).expect("node").payload = Payload::Conditional {
            guard: NodeId(0).into(),
        };
        match validate_nodes(&nodes) {
            Err(MathIrError::Cycle { path }) => {
                assert_eq!(path, vec![NodeId(0), NodeId(1), NodeId(0)]);
            }
            other => panic!("expected cycle, got {other:?}"),
        }
    }
    #[test]
    fn dangling_and_unreachable_cycles_are_rejected() {
        let mut nodes = BTreeMap::from([
            (NodeId(0), node(vec![])),
            (NodeId(1), node(vec![NodeId(9)])),
        ]);
        assert!(matches!(
            validate_nodes(&nodes),
            Err(MathIrError::Malformed { .. })
        ));
        nodes.get_mut(&NodeId(1)).expect("node").children = vec![NodeId(1)];
        assert!(matches!(
            validate_nodes(&nodes),
            Err(MathIrError::Cycle { .. })
        ));
    }
    #[test]
    fn numbering_preserves_root_and_child_order_without_recursive_stack_growth() {
        let mut graph = ExprGraph::new();
        let right = graph.int_const(2).expect("literal");
        let left = graph.int_const(1).expect("literal");
        let root = graph.add(left, right).expect("sum");
        assert_eq!(
            postorder(&graph, &[root]).expect("DAG"),
            vec![left, right, root]
        );
        let mut chain = root;
        for _ in 0..10000 {
            chain = graph.neg(chain).expect("child exists");
        }
        assert_eq!(postorder(&graph, &[chain]).expect("deep DAG").len(), 10003);
    }
}
