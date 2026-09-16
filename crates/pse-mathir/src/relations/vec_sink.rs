// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! An append-only plain-value sink/source for admission and deterministic round trips.
use super::{InputBinding, MathRelationSink, MathRelationSource, ParameterBinding};
use crate::equation::{EquationRecord, FreeIndex, Sense};
use crate::{AffineTerm, MathIrError, NodeId, Opcode, Payload, WeightedPair};
use pse_ids::{ContentHash, SemanticId};
use pse_quantity::{
    BoundIndexId, ConversionId, DomainId, InvariantId, OperationId, QuantityTypeId, ReductionKind,
    UnitConvertSpec, UnitId, WeightNormalization, infer::BuiltInRule,
};

/// A selection row retained as fields until semantic inference validates it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuantitySelection {
    /// Expression identity.
    pub node: NodeId,
    /// Selected registered operation, exclusive with `builtin`.
    pub operation: Option<OperationId>,
    /// Selected built-in rule, exclusive with `operation`.
    pub builtin: Option<BuiltInRule>,
    /// Original operand ordinal for each matched slot.
    pub permutation: Vec<u16>,
    /// Required conversions in original operand positions.
    pub conversions: Vec<(u16, ConversionId)>,
    /// A runtime check remains necessary; never proof that the check passed.
    pub deferred_static_check: bool,
}
/// Bound kernel relation facts. Physical port contracts live in the kernel specification.
#[derive(Clone, Debug, PartialEq)]
pub struct KernelBinding {
    /// Binding identity.
    pub binding: SemanticId,
    /// Kernel specification identity.
    pub kernel: SemanticId,
    /// Owning instance.
    pub scope: SemanticId,
    /// Ordered parameter bindings.
    pub parameters: Vec<ParameterBinding>,
    /// Ordered named input expressions.
    pub inputs: Vec<InputBinding>,
}
pub(super) type NodeRow = (
    NodeId,
    Opcode,
    Option<QuantityTypeId>,
    Option<SemanticId>,
    ContentHash,
);
/// Test and boundary sink. Append operations deliberately retain duplicate keys so loader
/// admission can reject them; writing to this sink never establishes validity.
#[derive(Clone, Debug, Default)]
pub struct VecSink {
    pub(super) nodes: Vec<NodeRow>,
    pub(super) args: Vec<(NodeId, u16, NodeId)>,
    pub(super) payloads: Vec<(NodeId, Payload)>,
    pub(super) equations: Vec<EquationRecord>,
    pub(super) indices: Vec<(SemanticId, FreeIndex)>,
    pub(super) selections: Vec<QuantitySelection>,
    pub(super) bindings: Vec<KernelBinding>,
}
impl VecSink {
    /// An empty unvalidated relation collection.
    pub fn new() -> Self {
        Self::default()
    }
    /// Number of expression rows, including any duplicates offered by the source.
    pub fn node_rows(&self) -> usize {
        self.nodes.len()
    }
}
impl MathRelationSink for VecSink {
    fn pending_path(
        &mut self,
        node: NodeId,
        source_id: SemanticId,
        path_id: u64,
        indices: &[NodeId],
    ) -> Result<(), MathIrError> {
        self.payloads.push((
            node,
            Payload::PendingPath {
                source_id,
                path_id,
                indices: indices.to_vec(),
            },
        ));
        Ok(())
    }
    fn pending_gather(
        &mut self,
        node: NodeId,
        group: SemanticId,
        indices: &[NodeId],
    ) -> Result<(), MathIrError> {
        self.payloads.push((
            node,
            Payload::PendingGather {
                group,
                indices: indices.to_vec(),
            },
        ));
        Ok(())
    }
    fn expr_node(
        &mut self,
        node: NodeId,
        opcode: Opcode,
        quantity_type: Option<QuantityTypeId>,
        scope: Option<SemanticId>,
        hash: ContentHash,
    ) -> Result<(), MathIrError> {
        self.nodes.push((node, opcode, quantity_type, scope, hash));
        Ok(())
    }
    fn expr_arg(&mut self, parent: NodeId, ordinal: u16, child: NodeId) -> Result<(), MathIrError> {
        self.args.push((parent, ordinal, child));
        Ok(())
    }
    fn symbol_ref(&mut self, node: NodeId, symbol: crate::ValueRef) -> Result<(), MathIrError> {
        self.payloads.push((node, Payload::SymbolRef { symbol }));
        Ok(())
    }
    fn float_constant(
        &mut self,
        node: NodeId,
        value: f64,
        unit: UnitId,
    ) -> Result<(), MathIrError> {
        self.payloads
            .push((node, Payload::FloatConst { value, unit }));
        Ok(())
    }
    fn int_constant(&mut self, node: NodeId, value: i64) -> Result<(), MathIrError> {
        self.payloads.push((node, Payload::IntConst { value }));
        Ok(())
    }
    fn affine(
        &mut self,
        node: NodeId,
        constant: f64,
        constant_quantity_type: Option<QuantityTypeId>,
        constant_unit: Option<UnitId>,
        terms: &[(f64, NodeId)],
    ) -> Result<(), MathIrError> {
        self.payloads.push((
            node,
            Payload::Affine {
                constant,
                constant_quantity_type,
                constant_unit,
                terms: terms
                    .iter()
                    .map(|(coefficient, child)| AffineTerm {
                        coefficient: *coefficient,
                        child: *child,
                    })
                    .collect(),
            },
        ));
        Ok(())
    }
    fn weighted_mean(
        &mut self,
        node: NodeId,
        pairs: &[(NodeId, NodeId)],
        normalization: WeightNormalization,
        certificate: Option<InvariantId>,
    ) -> Result<(), MathIrError> {
        self.payloads.push((
            node,
            Payload::WeightedMean {
                pairs: pairs
                    .iter()
                    .map(|(weight, value)| WeightedPair {
                        weight: *weight,
                        value: *value,
                    })
                    .collect(),
                normalization,
                unit_sum_invariant: certificate,
            },
        ));
        Ok(())
    }
    fn reduction(
        &mut self,
        node: NodeId,
        kind: ReductionKind,
        domain: crate::DomainRef,
        bound_index: BoundIndexId,
        filter: Option<crate::GuardRef>,
    ) -> Result<(), MathIrError> {
        self.payloads.push((
            node,
            Payload::Reduction {
                kind,
                domain,
                bound_index,
                filter,
            },
        ));
        Ok(())
    }
    fn gather(
        &mut self,
        node: NodeId,
        group: SemanticId,
        coordinates: &[(BoundIndexId, u16)],
    ) -> Result<(), MathIrError> {
        self.payloads.push((
            node,
            Payload::Gather {
                group,
                coordinate_map: coordinates.to_vec(),
            },
        ));
        Ok(())
    }
    fn broadcast(
        &mut self,
        node: NodeId,
        domain: crate::DomainRef,
        bound_index: BoundIndexId,
    ) -> Result<(), MathIrError> {
        self.payloads.push((
            node,
            Payload::Broadcast {
                domain,
                bound_index,
            },
        ));
        Ok(())
    }
    fn derivative(
        &mut self,
        node: NodeId,
        domain: crate::DomainRef,
        order: u8,
    ) -> Result<(), MathIrError> {
        self.payloads.push((
            node,
            Payload::Derivative {
                wrt_domain: domain,
                order,
            },
        ));
        Ok(())
    }
    fn integral(
        &mut self,
        node: NodeId,
        domain: crate::DomainRef,
        bound_index: BoundIndexId,
        policy: Option<SemanticId>,
        filter: Option<crate::GuardRef>,
    ) -> Result<(), MathIrError> {
        self.payloads.push((
            node,
            Payload::Integral {
                domain,
                bound_index,
                quadrature_policy: policy,
                filter,
            },
        ));
        Ok(())
    }
    fn pending_smooth_op(
        &mut self,
        node: NodeId,
        eps: f64,
        unit: UnitId,
    ) -> Result<(), MathIrError> {
        self.payloads
            .push((node, Payload::PendingSmoothOp { eps, unit }));
        Ok(())
    }
    fn smooth_op(&mut self, node: NodeId, eps: f64) -> Result<(), MathIrError> {
        self.payloads.push((node, Payload::SmoothOp { eps }));
        Ok(())
    }
    fn conditional(&mut self, node: NodeId, guard: crate::GuardRef) -> Result<(), MathIrError> {
        self.payloads.push((node, Payload::Conditional { guard }));
        Ok(())
    }
    fn kernel_call(
        &mut self,
        node: NodeId,
        binding: SemanticId,
        output: u16,
    ) -> Result<(), MathIrError> {
        self.payloads.push((
            node,
            Payload::KernelCall {
                kernel_binding: binding,
                output_ordinal: output,
            },
        ));
        Ok(())
    }
    fn implicit_ref(
        &mut self,
        node: NodeId,
        system: SemanticId,
        unknown: u16,
    ) -> Result<(), MathIrError> {
        self.payloads.push((
            node,
            Payload::ImplicitRef {
                implicit_system: system,
                unknown_ordinal: unknown,
            },
        ));
        Ok(())
    }
    fn pending_unit_convert(&mut self, node: NodeId, to: UnitId) -> Result<(), MathIrError> {
        self.payloads
            .push((node, Payload::PendingUnitConvert { to }));
        Ok(())
    }
    fn unit_convert(
        &mut self,
        node: NodeId,
        scale: f64,
        offset: f64,
        from: UnitId,
        to: UnitId,
    ) -> Result<(), MathIrError> {
        self.payloads.push((
            node,
            Payload::UnitConvert(UnitConvertSpec {
                scale,
                offset,
                from,
                to,
            }),
        ));
        Ok(())
    }
    fn piecewise_linear(
        &mut self,
        node: NodeId,
        points: &[(f64, f64)],
        input: QuantityTypeId,
        output: QuantityTypeId,
    ) -> Result<(), MathIrError> {
        self.payloads.push((
            node,
            Payload::PiecewiseLinear {
                breakpoints: points.to_vec(),
                input,
                output,
            },
        ));
        Ok(())
    }
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
    ) -> Result<(), MathIrError> {
        self.equations.push(EquationRecord {
            indexed_equation_id: id,
            owner_instance: owner,
            equation_decl: declaration,
            qualified_name: name.to_owned(),
            product,
            filter,
            body,
            sense,
            lower,
            upper,
            free_indices: vec![],
            residual_quantity_type: residual,
            law_instance: law,
            derivation,
        });
        Ok(())
    }
    fn free_index(
        &mut self,
        equation: SemanticId,
        binder: BoundIndexId,
        domain: DomainId,
        position: u16,
    ) -> Result<(), MathIrError> {
        self.indices.push((
            equation,
            FreeIndex {
                bound_index: binder,
                domain,
                position,
            },
        ));
        Ok(())
    }
    fn quantity_selection(
        &mut self,
        node: NodeId,
        operation: Option<OperationId>,
        builtin: Option<BuiltInRule>,
        permutation: &[u16],
        conversions: &[(u16, ConversionId)],
        deferred_static_check: bool,
    ) -> Result<(), MathIrError> {
        self.selections.push(QuantitySelection {
            node,
            operation,
            builtin,
            permutation: permutation.to_vec(),
            conversions: conversions.to_vec(),
            deferred_static_check,
        });
        Ok(())
    }
    fn kernel_binding(
        &mut self,
        binding: SemanticId,
        kernel: SemanticId,
        scope: SemanticId,
        parameters: &[ParameterBinding],
        inputs: &[InputBinding],
    ) -> Result<(), MathIrError> {
        self.bindings.push(KernelBinding {
            binding,
            kernel,
            scope,
            parameters: parameters.to_vec(),
            inputs: inputs.to_vec(),
        });
        Ok(())
    }
}
impl MathRelationSource for VecSink {
    fn read(&self, sink: &mut dyn MathRelationSink) -> Result<(), MathIrError> {
        for &(id, opcode, quantity, scope, hash) in &self.nodes {
            sink.expr_node(id, opcode, quantity, scope, hash)?;
        }
        for &(parent, ordinal, child) in &self.args {
            sink.expr_arg(parent, ordinal, child)?;
        }
        for (id, payload) in &self.payloads {
            super::emit_payload(*id, payload, sink)?;
        }
        for equation in &self.equations {
            emit_equation(equation, sink)?;
        }
        for &(equation, index) in &self.indices {
            sink.free_index(equation, index.bound_index, index.domain, index.position)?;
        }
        for row in &self.selections {
            emit_selection(row, sink)?;
        }
        for row in &self.bindings {
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
}
pub(super) fn emit_equation(
    row: &EquationRecord,
    sink: &mut dyn MathRelationSink,
) -> Result<(), MathIrError> {
    sink.indexed_equation(
        row.indexed_equation_id,
        row.owner_instance,
        row.equation_decl,
        &row.qualified_name,
        row.product,
        row.filter,
        row.body,
        row.sense,
        row.lower,
        row.upper,
        row.residual_quantity_type,
        row.law_instance,
        row.derivation,
    )?;
    for index in &row.free_indices {
        sink.free_index(
            row.indexed_equation_id,
            index.bound_index,
            index.domain,
            index.position,
        )?;
    }
    Ok(())
}
pub(super) fn emit_selection(
    row: &QuantitySelection,
    sink: &mut dyn MathRelationSink,
) -> Result<(), MathIrError> {
    sink.quantity_selection(
        row.node,
        row.operation,
        row.builtin,
        &row.permutation,
        &row.conversions,
        row.deferred_static_check,
    )
}

impl VecSink {
    pub(super) fn same_rows(&self, other: &Self) -> bool {
        let mut left = self.clone();
        let mut right = other.clone();
        for rows in [&mut left, &mut right] {
            rows.nodes.sort_by_key(|row| row.0);
            rows.args.sort_unstable();
            rows.payloads.sort_by_key(|row| row.0);
            rows.equations.sort_by_key(|row| row.indexed_equation_id);
            rows.indices
                .sort_by_key(|(id, index)| (*id, index.position));
            rows.selections.sort_by_key(|row| row.node);
            rows.bindings.sort_by_key(|row| row.binding);
        }
        let payload_key = |rows: &Vec<(NodeId, Payload)>| {
            rows.iter()
                .map(|(id, payload)| {
                    let mut bytes = Vec::new();
                    crate::graph::encode_payload(&mut bytes, payload);
                    (*id, bytes)
                })
                .collect::<Vec<_>>()
        };
        let binding_key = |rows: &Vec<KernelBinding>| {
            rows.iter()
                .map(|binding| {
                    let parameters: Vec<_> = binding
                        .parameters
                        .iter()
                        .map(|(name, symbol, value, unit)| {
                            (
                                name.clone(),
                                *symbol,
                                value.map(pse_ids::canonical_f64_bits),
                                *unit,
                            )
                        })
                        .collect();
                    (
                        binding.binding,
                        binding.kernel,
                        binding.scope,
                        parameters,
                        binding.inputs.clone(),
                    )
                })
                .collect::<Vec<_>>()
        };
        left.args == right.args
            && payload_key(&left.payloads) == payload_key(&right.payloads)
            && left.equations == right.equations
            && left.indices == right.indices
            && left.selections == right.selections
            && binding_key(&left.bindings) == binding_key(&right.bindings)
            && left.nodes == right.nodes
    }
}
