// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Ordered typed sharing and canonical storage numbering (blueprint §7.4).
//!
//! This module checks structural and supplied type/index admission. Physical inference
//! is a separate prerequisite: assigning a type ID or computing a hash cannot prove it.
use crate::{ExprGraph, MathIrError, Node, NodeId, Opcode, Payload};
use pse_ids::{ContentHash, SemanticId};
use pse_quantity::{IndexSet, QuantityTypeId, registry::QuantityRegistry};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

/// An immutable numbered node with admitted type and index metadata.
#[derive(Clone, Debug)]
pub struct CanonicalNode {
    /// Mathematical operator.
    pub opcode: Opcode,
    /// Typed payload, with every embedded node reference remapped.
    pub payload: Payload,
    /// Children in authoritative argument order.
    pub children: Vec<NodeId>,
    /// Supplied complete quantity contract.
    pub quantity_type: QuantityTypeId,
    /// Shared scope, absent if contributing scopes conflict.
    pub scope: Option<SemanticId>,
    /// Content identity; never evidence of successful physical inference.
    pub subtree_hash: ContentHash,
    /// Supplied resolved free-index identities.
    pub free_indices: IndexSet,
}
/// Immutable post-order storage for a typed graph.
#[derive(Clone, Debug, Default)]
pub struct CanonicalGraph {
    nodes: Vec<CanonicalNode>,
    pub(crate) roots: Vec<NodeId>,
    pub(crate) equations: Vec<crate::equation::EquationRecord>,
    pub(crate) selections: Vec<crate::relations::vec_sink::QuantitySelection>,
    pub(crate) kernel_bindings: BTreeMap<SemanticId, crate::relations::vec_sink::KernelBinding>,
    pub(crate) node_mapping: BTreeMap<NodeId, NodeId>,
    pub(crate) source_nodes: BTreeMap<NodeId, BTreeSet<NodeId>>,
}
impl CanonicalGraph {
    /// A numbered node.
    ///
    /// # Errors
    /// Rejects an ordinal outside this graph.
    pub fn node(&self, id: NodeId) -> Result<&CanonicalNode, MathIrError> {
        usize::try_from(id.0)
            .ok()
            .and_then(|index| self.nodes.get(index))
            .ok_or_else(|| MathIrError::malformed_at(id, "no such canonical node"))
    }
    /// Roots in the supplied semantic order.
    pub fn roots(&self) -> &[NodeId] {
        &self.roots
    }
    /// Nodes in deterministic storage order.
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &CanonicalNode)> {
        self.nodes
            .iter()
            .zip(0_i64..)
            .map(|(node, index)| (NodeId(index), node))
    }
    /// Indexed equation rows with every expression reference in canonical storage.
    pub fn equations(&self) -> &[crate::equation::EquationRecord] {
        &self.equations
    }
    /// Recorded physical selections; recording alone is not proof of inference.
    pub fn selections(&self) -> &[crate::relations::vec_sink::QuantitySelection] {
        &self.selections
    }
    /// Kernel-binding rows with every input-node reference remapped.
    pub fn kernel_bindings(
        &self,
    ) -> &BTreeMap<SemanticId, crate::relations::vec_sink::KernelBinding> {
        &self.kernel_bindings
    }
    /// The complete previous-ordinal to canonical-ordinal map for retained nodes.
    pub fn node_mapping(&self) -> &BTreeMap<NodeId, NodeId> {
        &self.node_mapping
    }
    /// Actual source occurrences contributing to each retained canonical node.
    /// This includes source operations folded to literals and contextual conversions.
    /// It records inference correspondence; physical admission remains mandatory.
    pub fn source_nodes(&self) -> &BTreeMap<NodeId, BTreeSet<NodeId>> {
        &self.source_nodes
    }
    /// The number of stored nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    /// Whether no nodes are stored.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
    /// A deterministic diagnostic listing; never a serialization or hash input.
    pub fn listing(&self) -> String {
        let mut out = String::new();
        for (id, node) in self.iter() {
            let _ = writeln!(
                out,
                "{id} {} type={} children={:?} payload={:?}",
                node.opcode, node.quantity_type, node.children, node.payload
            );
        }
        let _ = writeln!(out, "roots={:?}", self.roots);
        out
    }
}
/// Share exact typed structures and number them in ordered post-order.
///
/// Call only after physical inference has established every supplied type and index set.
/// This utility does not infer operation contracts; its checks establish structural and
/// registry admission only. It compares complete keys, so digest collisions cannot merge
/// unequal structures. Unreachable nodes are excluded from the result.
///
/// # Errors
/// Rejects absent roots, untyped/unregistered nodes, missing index metadata, or index kinds
/// inconsistent with the complete type shape. Kernel calls require the later binding-aware
/// driver and are explicitly refused here because their input dependencies are external.
pub fn number_typed_graph(
    graph: &ExprGraph,
    roots: &[NodeId],
    indices: &BTreeMap<NodeId, IndexSet>,
    registry: &QuantityRegistry,
) -> Result<CanonicalGraph, MathIrError> {
    number_typed_graph_with_bindings(graph, roots, indices, registry, &BTreeMap::new())
}
/// Number typed structures with their actual bound-kernel dependencies.
/// Physical inference remains a separate prerequisite; binding IDs and hashes do not prove it.
///
/// # Errors
/// Rejects the same incomplete type/index facts as [`number_typed_graph`], and missing or
/// cyclic binding inputs. All surviving binding input references are remapped together.
pub fn number_typed_graph_with_bindings(
    graph: &ExprGraph,
    roots: &[NodeId],
    indices: &BTreeMap<NodeId, IndexSet>,
    registry: &QuantityRegistry,
    bindings: &BTreeMap<SemanticId, crate::relations::vec_sink::KernelBinding>,
) -> Result<CanonicalGraph, MathIrError> {
    let order = crate::topo::postorder_with_bindings(graph, roots, bindings)?;
    let mut result = CanonicalGraph::default();
    let mut remap = BTreeMap::new();
    let mut exact = BTreeMap::new();
    for old in order {
        let original = graph.node(old)?;
        require_compiled_payload(old, &original.payload)?;
        let quantity_type = original
            .quantity_type
            .ok_or_else(|| MathIrError::malformed_at(old, "node has no inferred quantity type"))?;
        let declared = registry
            .quantity_type(quantity_type)
            .map_err(|source| MathIrError::Quantity { node: old, source })?;
        let free_indices = indices
            .get(&old)
            .ok_or_else(|| MathIrError::malformed_at(old, "node has no resolved index set"))?
            .clone();
        admit_indices(old, &declared.key.shape, &free_indices)?;
        let map = |id: NodeId| {
            remap
                .get(&id)
                .copied()
                .ok_or_else(|| MathIrError::malformed_at(id, "dependency not numbered"))
        };
        let children = original
            .children
            .iter()
            .map(|id| map(*id))
            .collect::<Result<Vec<_>, _>>()?;
        let mut payload = original.payload.clone();
        payload.map_node_references(map)?;
        let mut key = crate::graph::structural_key(original.opcode, &payload, &children);
        key.extend_from_slice(quantity_type.as_id().as_bytes());
        for index in &free_indices {
            key.extend_from_slice(index.bound_index.as_id().as_bytes());
            key.extend_from_slice(index.domain.as_id().as_bytes());
            key.extend_from_slice(index.kind.as_str().as_bytes());
            key.push(0);
        }
        if let Some(existing) = exact.get(&key).copied() {
            let node: &mut CanonicalNode = &mut result.nodes[existing];
            if node.scope != original.scope {
                node.scope = None;
            }
            remap.insert(old, NodeId::from_index(existing)?);
            continue;
        }
        let node = Node {
            opcode: original.opcode,
            payload,
            children,
            quantity_type: Some(quantity_type),
            scope: original.scope,
        };
        let dependencies = node
            .children
            .iter()
            .copied()
            .chain(node.payload.referenced_nodes())
            .map(|id| result.node(id).map(|node| node.subtree_hash))
            .collect::<Result<Vec<_>, _>>()?;
        let subtree_hash = hash_numbered_node(
            &node,
            old,
            quantity_type,
            &dependencies,
            bindings,
            &remap,
            &result,
        )?;
        let id = NodeId::from_index(result.nodes.len())?;
        exact.insert(key, result.nodes.len());
        remap.insert(old, id);
        result.nodes.push(CanonicalNode {
            opcode: node.opcode,
            payload: node.payload,
            children: node.children,
            quantity_type,
            scope: node.scope,
            subtree_hash,
            free_indices,
        });
    }
    result.roots = roots
        .iter()
        .map(|id| {
            remap
                .get(id)
                .copied()
                .ok_or_else(|| MathIrError::malformed_at(*id, "root not numbered"))
        })
        .collect::<Result<_, _>>()?;
    result.kernel_bindings = remap_bindings(&result, bindings, &remap)?;
    result.node_mapping = remap;
    Ok(result)
}

fn require_compiled_payload(node: NodeId, payload: &Payload) -> Result<(), MathIrError> {
    if matches!(
        payload,
        Payload::PendingUnitConvert { .. }
            | Payload::PendingGather { .. }
            | Payload::PendingPath { .. }
            | Payload::PendingSmoothOp { .. }
    ) {
        return Err(MathIrError::malformed_at(
            node,
            "pending normalized payload has no compiled storage contract",
        ));
    }
    Ok(())
}

fn hash_numbered_node(
    node: &Node,
    old: NodeId,
    quantity_type: QuantityTypeId,
    dependencies: &[ContentHash],
    bindings: &BTreeMap<SemanticId, crate::relations::vec_sink::KernelBinding>,
    remap: &BTreeMap<NodeId, NodeId>,
    result: &CanonicalGraph,
) -> Result<ContentHash, MathIrError> {
    if let Payload::KernelCall { kernel_binding, .. } = node.payload {
        let binding = bindings
            .get(&kernel_binding)
            .ok_or(MathIrError::UnknownBinding {
                node: old,
                binding: kernel_binding,
            })?;
        let dependencies = binding
            .inputs
            .iter()
            .map(|(_, id)| {
                let mapped = remap
                    .get(id)
                    .copied()
                    .ok_or_else(|| MathIrError::malformed_at(*id, "kernel input not numbered"))?;
                result.node(mapped).map(|node| node.subtree_hash)
            })
            .collect::<Result<Vec<_>, _>>()?;
        crate::hash::subtree_hash_with_binding(node, Some(quantity_type), &dependencies, binding)
    } else {
        crate::hash::subtree_hash(node, quantity_type, dependencies)
    }
}
fn remap_bindings(
    result: &CanonicalGraph,
    bindings: &BTreeMap<SemanticId, crate::relations::vec_sink::KernelBinding>,
    remap: &BTreeMap<NodeId, NodeId>,
) -> Result<BTreeMap<SemanticId, crate::relations::vec_sink::KernelBinding>, MathIrError> {
    let mut output = BTreeMap::new();
    let used: BTreeSet<_> = result
        .nodes
        .iter()
        .filter_map(|node| {
            if let Payload::KernelCall { kernel_binding, .. } = node.payload {
                Some(kernel_binding)
            } else {
                None
            }
        })
        .collect();
    for id in used {
        let mut binding = bindings
            .get(&id)
            .cloned()
            .ok_or_else(|| MathIrError::malformed("retained kernel binding is missing"))?;
        for (_, input) in &mut binding.inputs {
            *input = remap.get(input).copied().ok_or_else(|| {
                MathIrError::malformed_at(*input, "kernel input was not retained")
            })?;
        }
        output.insert(id, binding);
    }
    Ok(output)
}

fn admit_indices(
    old: NodeId,
    shape: &[pse_quantity::DomainKind],
    free_indices: &IndexSet,
) -> Result<(), MathIrError> {
    // Shape order is supplied by the complete type; IndexSet stores binder order.
    // Compare multiplicities of domain kinds, not incidental binder-identity ordering.
    let mut expected = shape.to_vec();
    expected.sort_unstable();
    let mut actual: Vec<_> = free_indices.iter().map(|index| index.kind).collect();
    actual.sort_unstable();
    if expected != actual {
        return Err(MathIrError::malformed_at(
            old,
            "free indices disagree with quantity shape",
        ));
    }
    Ok(())
}
