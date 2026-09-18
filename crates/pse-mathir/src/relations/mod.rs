// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Arrow-free relation boundaries (blueprint §6.9, §7.1).
//!
//! Primitive callbacks mirror declared relation fields. Sources replay their rows through
//! the same interface; a loader checks every row and reference before constructing a DAG.
//! Adapters over generated Arrow relations belong in `pse-compiler`.
use crate::{CanonicalGraph, ExprGraph, MathIrError, NodeId, Opcode, Payload};
use pse_ids::{ContentHash, SemanticId};
use pse_quantity::{
    BoundIndexId, ConversionId, DomainId, OperationId, QuantityTypeId, UnitId, infer::BuiltInRule,
};
use pse_schema::math::Sense;

mod load;
pub mod vec_sink;
pub use load::{LoadedMath, load_canonical, load_untyped};
pub use vec_sink::VecSink;

/// An ordered kernel parameter binding: name, symbol, literal value and literal unit.
pub type ParameterBinding = (String, Option<SemanticId>, Option<f64>, Option<UnitId>);
/// An ordered named kernel input expression.
pub type InputBinding = (String, NodeId);

/// Primitive relation writer. Each callback may fail rather than partially assert success.
/// All implementations must preserve input argument and nested-list order.
#[expect(
    clippy::too_many_arguments,
    reason = "callbacks mirror registry relation fields"
)]
pub trait MathRelationSink {
    /// Write one expression-node row.
    ///
    /// # Errors
    /// Propagates the sink adapter's write or admission failure.
    fn expr_node(
        &mut self,
        node: NodeId,
        opcode: Opcode,
        children: &[NodeId],
        payload: &Payload,
        quantity_type: Option<QuantityTypeId>,
        scope: Option<SemanticId>,
        hash: ContentHash,
    ) -> Result<(), MathIrError>;
    /// Write an indexed equation; free indices use their separate callback.
    ///
    /// # Errors
    /// Propagates the sink adapter's write or admission failure.
    fn indexed_equation(
        &mut self,
        id: SemanticId,
        owner: SemanticId,
        declaration: Option<SemanticId>,
        name: &str,
        product: Option<SemanticId>,
        filter: Option<NodeId>,
        body: NodeId,
        sense: Sense,
        lower: Option<NodeId>,
        upper: Option<NodeId>,
        residual: Option<QuantityTypeId>,
        law: Option<SemanticId>,
        derivation: SemanticId,
    ) -> Result<(), MathIrError>;
    /// Write one equation free-index axis.
    ///
    /// # Errors
    /// Propagates the sink adapter's write or admission failure.
    fn free_index(
        &mut self,
        equation: SemanticId,
        binder: BoundIndexId,
        domain: DomainId,
        position: u16,
    ) -> Result<(), MathIrError>;
    /// Write a physical operation selection and conversions in operand order.
    ///
    /// # Errors
    /// Propagates the sink adapter's write or admission failure.
    fn quantity_selection(
        &mut self,
        node: NodeId,
        operation: Option<OperationId>,
        builtin: Option<BuiltInRule>,
        permutation: &[u16],
        conversions: &[(u16, ConversionId)],
        deferred_static_check: bool,
    ) -> Result<(), MathIrError>;
    /// Write bound kernel parameters and ordered input-node references.
    ///
    /// # Errors
    /// Propagates the sink adapter's write or admission failure.
    fn kernel_binding(
        &mut self,
        binding: SemanticId,
        kernel: SemanticId,
        scope: SemanticId,
        parameters: &[ParameterBinding],
        inputs: &[InputBinding],
    ) -> Result<(), MathIrError>;
}
/// A relation source replays actual rows. It must not silently discard duplicate keys.
pub trait MathRelationSource {
    /// Stream all rows, including empty-family absence, to a primitive sink.
    ///
    /// # Errors
    /// Propagates read/adapter or sink admission failures.
    fn read(&self, sink: &mut dyn MathRelationSink) -> Result<(), MathIrError>;
}

/// Emit typed expression rows in canonical storage order.
///
/// # Errors
/// Propagates sink errors or an argument count exceeding the declared ordinal type.
pub fn emit(graph: &CanonicalGraph, sink: &mut dyn MathRelationSink) -> Result<(), MathIrError> {
    for (id, node) in graph.iter() {
        sink.expr_node(
            id,
            node.opcode,
            &node.children,
            &node.payload,
            Some(node.quantity_type),
            node.scope,
            node.subtree_hash,
        )?;
    }
    for equation in graph.equations() {
        vec_sink::emit_equation(equation, sink)?;
    }
    for selection in graph.selections() {
        vec_sink::emit_selection(selection, sink)?;
    }
    for row in graph.kernel_bindings().values() {
        sink.kernel_binding(
            row.binding,
            row.kernel,
            row.scope,
            &row.parameters,
            &row.inputs,
        )?;
    }
    Ok(())
}
/// Emit an untyped graph after checking its reachable structure.
/// Root order is explicit call context; there is no invented roots relation.
///
/// # Errors
/// Rejects missing roots, unresolved bound-kernel dependencies or sink failures.
pub fn emit_untyped(
    graph: &ExprGraph,
    roots: &[NodeId],
    sink: &mut dyn MathRelationSink,
) -> Result<(), MathIrError> {
    emit_untyped_with_bindings(graph, roots, &std::collections::BTreeMap::new(), sink)
}
/// Emit an expression graph together with all named bound-kernel inputs.
///
/// # Errors
/// Rejects missing/cyclic references and incomplete bound-kernel dependencies.
pub fn emit_untyped_with_bindings(
    graph: &ExprGraph,
    roots: &[NodeId],
    bindings: &std::collections::BTreeMap<SemanticId, vec_sink::KernelBinding>,
    sink: &mut dyn MathRelationSink,
) -> Result<(), MathIrError> {
    let order = crate::topo::postorder_with_bindings(graph, roots, bindings)?;
    let mut hashes = std::collections::BTreeMap::new();
    for id in order {
        let node = graph.node(id)?;
        let dependencies = node
            .children
            .iter()
            .copied()
            .chain(node.payload.referenced_nodes())
            .map(|dependency| {
                hashes.get(&dependency).copied().ok_or_else(|| {
                    MathIrError::malformed_at(dependency, "dependency hash absent after traversal")
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let hash = if let Payload::KernelCall { kernel_binding, .. } = node.payload {
            let binding = bindings
                .get(&kernel_binding)
                .ok_or(MathIrError::UnknownBinding {
                    node: id,
                    binding: kernel_binding,
                })?;
            let inputs = binding
                .inputs
                .iter()
                .map(|(_, input)| {
                    hashes.get(input).copied().ok_or_else(|| {
                        MathIrError::malformed_at(*input, "bound input hash missing")
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            crate::hash::subtree_hash_with_binding(node, node.quantity_type, &inputs, binding)?
        } else if let Some(quantity) = node.quantity_type {
            crate::hash::subtree_hash(node, quantity, &dependencies)?
        } else {
            crate::hash::structural_hash(node, &dependencies)?
        };
        hashes.insert(id, hash);
        sink.expr_node(
            id,
            node.opcode,
            &node.children,
            &node.payload,
            node.quantity_type,
            node.scope,
            hash,
        )?;
    }
    for row in bindings.values() {
        sink.kernel_binding(
            row.binding,
            row.kernel,
            row.scope,
            &row.parameters,
            &row.inputs,
        )?;
    }
    Ok(())
}
/// Emit every admitted source row, including equations, selections and binding inputs.
/// This preserves declarations but does not promote supplied types to inference evidence.
///
/// # Errors
/// Propagates missing dependencies or sink failures.
pub fn emit_loaded(
    loaded: &LoadedMath,
    sink: &mut dyn MathRelationSink,
) -> Result<(), MathIrError> {
    let all: Vec<_> = loaded.graph.iter().map(|(id, _)| id).collect();
    emit_untyped_with_bindings(&loaded.graph, &all, &loaded.kernel_bindings, sink)?;
    for row in &loaded.equations {
        vec_sink::emit_equation(row, sink)?;
    }
    for row in &loaded.selections {
        vec_sink::emit_selection(row, sink)?;
    }
    Ok(())
}
