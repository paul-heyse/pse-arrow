// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Deterministic operator-family visitation without a second operator roster.
use crate::infer::KernelBindings;
use crate::opspec::OperatorFamily;
use crate::{ExprGraph, MathIrError, Node, NodeId};
/// A visitor receives the authoritative family from `OPERATOR_TABLE`.
/// Traversal is storage/dependency order; visiting never authorizes eager evaluation.
pub trait NodeVisitor {
    /// Inspect one node and its declared operator family.
    ///
    /// # Errors
    /// Propagates a semantic failure established by the visitor.
    fn visit(&mut self, id: NodeId, node: &Node, family: OperatorFamily)
    -> Result<(), MathIrError>;
}
/// Visit all reachable nodes once, including every bound-kernel dependency.
///
/// # Errors
/// Rejects missing/cyclic references or a visitor's semantic error.
pub fn walk(
    graph: &ExprGraph,
    roots: &[NodeId],
    bindings: &KernelBindings,
    visitor: &mut dyn NodeVisitor,
) -> Result<(), MathIrError> {
    for id in crate::topo::postorder_with_bindings(graph, roots, bindings)? {
        let node = graph.node(id)?;
        visitor.visit(id, node, crate::operator_spec(node.opcode).family)?;
    }
    Ok(())
}
