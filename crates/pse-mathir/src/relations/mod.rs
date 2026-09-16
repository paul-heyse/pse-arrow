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
    BoundIndexId, ConversionId, DomainId, InvariantId, OperationId, QuantityTypeId, ReductionKind,
    UnitConvertSpec, UnitId, WeightNormalization, infer::BuiltInRule,
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
        quantity_type: Option<QuantityTypeId>,
        scope: Option<SemanticId>,
        hash: ContentHash,
    ) -> Result<(), MathIrError>;
    /// Write one ordered argument row.
    ///
    /// # Errors
    /// Propagates the sink adapter's write or admission failure.
    fn expr_arg(&mut self, parent: NodeId, ordinal: u16, child: NodeId) -> Result<(), MathIrError>;
    /// Write a symbol reference.
    ///
    /// # Errors
    /// Propagates the sink adapter's write or admission failure.
    fn symbol_ref(&mut self, node: NodeId, symbol: crate::ValueRef) -> Result<(), MathIrError>;
    /// Write an authored-unit floating literal.
    ///
    /// # Errors
    /// Propagates the sink adapter's write or admission failure.
    fn float_constant(&mut self, node: NodeId, value: f64, unit: UnitId)
    -> Result<(), MathIrError>;
    /// Write an integer literal.
    ///
    /// # Errors
    /// Propagates the sink adapter's write or admission failure.
    fn int_constant(&mut self, node: NodeId, value: i64) -> Result<(), MathIrError>;
    /// Write an ordered affine payload.
    ///
    /// # Errors
    /// Propagates the sink adapter's write or admission failure.
    fn affine(
        &mut self,
        node: NodeId,
        constant: f64,
        constant_quantity_type: Option<QuantityTypeId>,
        constant_unit: Option<UnitId>,
        terms: &[(f64, NodeId)],
    ) -> Result<(), MathIrError>;
    /// Write an ordered weighted mean.
    ///
    /// # Errors
    /// Propagates the sink adapter's write or admission failure.
    fn weighted_mean(
        &mut self,
        node: NodeId,
        pairs: &[(NodeId, NodeId)],
        normalization: WeightNormalization,
        certificate: Option<InvariantId>,
    ) -> Result<(), MathIrError>;
    /// Write a reduction and its filter reference.
    ///
    /// # Errors
    /// Propagates the sink adapter's write or admission failure.
    fn reduction(
        &mut self,
        node: NodeId,
        kind: ReductionKind,
        domain: crate::DomainRef,
        bound_index: BoundIndexId,
        filter: Option<crate::GuardRef>,
    ) -> Result<(), MathIrError>;
    /// Write a group read with explicit coordinate positions.
    ///
    /// # Errors
    /// Propagates the sink adapter's write or admission failure.
    fn gather(
        &mut self,
        node: NodeId,
        group: SemanticId,
        coordinates: &[(BoundIndexId, u16)],
    ) -> Result<(), MathIrError>;
    /// Preserve an exact normalized source-relative path and its ordered index nodes.
    /// # Errors
    /// A sink without normalized path storage refuses the request.
    fn pending_path(
        &mut self,
        node: NodeId,
        _source_id: SemanticId,
        _path_id: u64,
        _indices: &[NodeId],
    ) -> Result<(), MathIrError> {
        Err(MathIrError::malformed_at(
            node,
            "sink cannot retain an unresolved instance path",
        ))
    }
    /// Preserve a normalized read's ordered actual index expressions.
    /// # Errors
    /// A compiled-only sink refuses the unresolved source request.
    fn pending_gather(
        &mut self,
        node: NodeId,
        _group: SemanticId,
        _indices: &[NodeId],
    ) -> Result<(), MathIrError> {
        Err(MathIrError::malformed_at(
            node,
            "sink cannot retain pending indexed read",
        ))
    }
    /// Write an explicit broadcast.
    ///
    /// # Errors
    /// Propagates the sink adapter's write or admission failure.
    fn broadcast(
        &mut self,
        node: NodeId,
        domain: crate::DomainRef,
        bound_index: BoundIndexId,
    ) -> Result<(), MathIrError>;
    /// Write a derivative.
    ///
    /// # Errors
    /// Propagates the sink adapter's write or admission failure.
    fn derivative(
        &mut self,
        node: NodeId,
        domain: crate::DomainRef,
        order: u8,
    ) -> Result<(), MathIrError>;
    /// Write an integral.
    ///
    /// # Errors
    /// Propagates the sink adapter's write or admission failure.
    fn integral(
        &mut self,
        node: NodeId,
        domain: crate::DomainRef,
        bound_index: BoundIndexId,
        policy: Option<SemanticId>,
        filter: Option<crate::GuardRef>,
    ) -> Result<(), MathIrError>;
    /// Write a smoothing parameter.
    ///
    /// # Errors
    /// Propagates the sink adapter's write or admission failure.
    fn smooth_op(&mut self, node: NodeId, eps: f64) -> Result<(), MathIrError>;
    /// Write a normalized unit-bearing epsilon without erasing its physical representation.
    /// # Errors
    /// The default refuses adapters that only admit resolved compiled payloads.
    fn pending_smooth_op(
        &mut self,
        node: NodeId,
        _eps: f64,
        _unit: UnitId,
    ) -> Result<(), MathIrError> {
        Err(MathIrError::malformed_at(
            node,
            "sink cannot retain a pending smoothing tolerance",
        ))
    }

    /// Write the conditional guard.
    ///
    /// # Errors
    /// Propagates the sink adapter's write or admission failure.
    fn conditional(&mut self, node: NodeId, guard: crate::GuardRef) -> Result<(), MathIrError>;
    /// Write a kernel call's output selection.
    ///
    /// # Errors
    /// Propagates the sink adapter's write or admission failure.
    fn kernel_call(
        &mut self,
        node: NodeId,
        binding: SemanticId,
        output: u16,
    ) -> Result<(), MathIrError>;
    /// Write an implicit unknown reference.
    ///
    /// # Errors
    /// Propagates the sink adapter's write or admission failure.
    fn implicit_ref(
        &mut self,
        node: NodeId,
        system: SemanticId,
        unknown: u16,
    ) -> Result<(), MathIrError>;
    /// Write an exact unit representation edge.
    ///
    /// # Errors
    /// Propagates the sink adapter's write or admission failure.
    fn unit_convert(
        &mut self,
        node: NodeId,
        scale: f64,
        offset: f64,
        from: UnitId,
        to: UnitId,
    ) -> Result<(), MathIrError>;
    /// Write a normalized conversion request lacking a complete source quantity context.
    /// # Errors
    /// Compiled-only sinks refuse unresolved conversion requests by default.
    fn pending_unit_convert(&mut self, node: NodeId, _to: UnitId) -> Result<(), MathIrError> {
        Err(MathIrError::malformed_at(
            node,
            "unresolved unit conversion cannot enter compiled storage",
        ))
    }
    /// Write declared piecewise-linear coordinates.
    ///
    /// # Errors
    /// Propagates the sink adapter's write or admission failure.
    fn piecewise_linear(
        &mut self,
        node: NodeId,
        points: &[(f64, f64)],
        input: QuantityTypeId,
        output: QuantityTypeId,
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
            Some(node.quantity_type),
            node.scope,
            node.subtree_hash,
        )?;
        emit_args(id, &node.children, sink)?;
        emit_payload(id, &node.payload, sink)?;
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
        sink.expr_node(id, node.opcode, node.quantity_type, node.scope, hash)?;
        emit_args(id, &node.children, sink)?;
        emit_payload(id, &node.payload, sink)?;
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

fn emit_args(
    id: NodeId,
    children: &[NodeId],
    sink: &mut dyn MathRelationSink,
) -> Result<(), MathIrError> {
    for (position, child) in children.iter().enumerate() {
        let ordinal = u16::try_from(position)
            .map_err(|_| MathIrError::malformed_at(id, "argument ordinal exceeds u16"))?;
        sink.expr_arg(id, ordinal, *child)?;
    }
    Ok(())
}
fn emit_payload(
    id: NodeId,
    payload: &Payload,
    sink: &mut dyn MathRelationSink,
) -> Result<(), MathIrError> {
    match payload {
        Payload::None => Ok(()),
        Payload::SymbolRef { symbol } => sink.symbol_ref(id, symbol.clone()),
        Payload::FloatConst { value, unit } => sink.float_constant(id, *value, *unit),
        Payload::IntConst { value } => sink.int_constant(id, *value),
        Payload::Affine {
            constant,
            constant_quantity_type,
            constant_unit,
            terms,
        } => sink.affine(
            id,
            *constant,
            *constant_quantity_type,
            *constant_unit,
            &terms
                .iter()
                .map(|term| (term.coefficient, term.child))
                .collect::<Vec<_>>(),
        ),
        Payload::WeightedMean {
            pairs,
            normalization,
            unit_sum_invariant,
        } => sink.weighted_mean(
            id,
            &pairs
                .iter()
                .map(|pair| (pair.weight, pair.value))
                .collect::<Vec<_>>(),
            *normalization,
            *unit_sum_invariant,
        ),
        Payload::Reduction {
            kind,
            domain,
            bound_index,
            filter,
        } => sink.reduction(id, *kind, domain.clone(), *bound_index, *filter),
        Payload::Gather {
            group,
            coordinate_map,
        } => sink.gather(id, *group, coordinate_map),
        Payload::Broadcast {
            domain,
            bound_index,
        } => sink.broadcast(id, domain.clone(), *bound_index),
        Payload::Derivative { wrt_domain, order } => {
            sink.derivative(id, wrt_domain.clone(), *order)
        }
        Payload::Integral {
            domain,
            bound_index,
            quadrature_policy,
            filter,
        } => sink.integral(
            id,
            domain.clone(),
            *bound_index,
            *quadrature_policy,
            *filter,
        ),
        Payload::SmoothOp { eps } => sink.smooth_op(id, *eps),
        Payload::PendingSmoothOp { eps, unit } => sink.pending_smooth_op(id, *eps, *unit),
        Payload::Conditional { guard } => sink.conditional(id, *guard),
        Payload::KernelCall {
            kernel_binding,
            output_ordinal,
        } => sink.kernel_call(id, *kernel_binding, *output_ordinal),
        Payload::ImplicitRef {
            implicit_system,
            unknown_ordinal,
        } => sink.implicit_ref(id, *implicit_system, *unknown_ordinal),
        Payload::PendingUnitConvert { to } => sink.pending_unit_convert(id, *to),
        Payload::PendingGather { group, indices } => sink.pending_gather(id, *group, indices),
        Payload::PendingPath {
            source_id,
            path_id,
            indices,
        } => sink.pending_path(id, *source_id, *path_id, indices),
        Payload::UnitConvert(UnitConvertSpec {
            scale,
            offset,
            from,
            to,
        }) => sink.unit_convert(id, *scale, *offset, *from, *to),
        Payload::PiecewiseLinear {
            breakpoints,
            input,
            output,
        } => sink.piecewise_linear(id, breakpoints, *input, *output),
    }
}
