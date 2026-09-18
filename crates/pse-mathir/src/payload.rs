// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The typed operator payloads (blueprint §6.9 `compiled.math_*`, §7.2).
//!
//! Every expression node carries one declared tagged payload struct. This enum is
//! the bounded algorithm view of that value; it has no independent persistence path.
//!
//! Two payloads hold node references rather than values, and both do so for a reason §7.4
//! names:
//!
//! - [`Payload::Conditional`] holds the guard, so that a subtree shared with a branch
//!   cannot have its evaluation hoisted out of the branch (§7.4 step 1, step 3).
//! - [`Payload::WeightedMean`] holds ordered weight/value pairs, because a flat child list
//!   cannot say which weight belongs to which value, and the `divide_by_sum` normalization
//!   evaluates an ordered numerator over an ordered denominator (§7.2).
//!
//! Everything a payload holds is a value or an identity. Nothing here is a rendering, and
//! the floats are hashed through `pse_ids::canonical_f64_bits` rather than compared with
//! `==` (ADR-0030).

use pse_ids::SemanticId;
use pse_quantity::{
    BoundIndexId, InvariantId, QuantityTypeId, ReductionKind, UnitConvertSpec, UnitId,
    WeightNormalization,
};

use crate::node::NodeId;
use crate::{DomainRef, GuardRef, ValueRef};

/// One ordered term of an [`Payload::Affine`] node (blueprint §6.9 `math_expr_nodes.payload.affine.terms`).
///
/// The child need not be linear: an `Affine` node records that its *own* combination is a
/// constant plus weighted terms, not that the terms are affine in the decision variables.
/// Establishing the latter is P14's job on the case-bound view (§7.5).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct AffineTerm {
    /// The coefficient the child is multiplied by.
    pub coefficient: f64,
    /// The child node; it also appears in the node's ordered child list.
    pub child: NodeId,
}

/// One ordered weight/value pair of a [`Payload::WeightedMean`] node
/// (blueprint §6.9 `math_expr_nodes.payload.weighted_mean.pairs`).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct WeightedPair {
    /// The dimensionless weight.
    pub weight: NodeId,
    /// The value being averaged; every value shares one complete quantity type (§8.3).
    pub value: NodeId,
}

/// The typed payload of one node (blueprint §6.9).
#[derive(Clone, PartialEq, Debug)]
#[non_exhaustive]
pub enum Payload {
    /// No payload: the operator's operands are exactly its children.
    None,

    /// A reference to a symbol (`math_expr_nodes.payload.symbol`); the symbol supplies the type.
    SymbolRef {
        /// The symbol's identity.
        symbol: ValueRef,
    },

    /// A floating-point literal with the unit it was written in
    /// (`math_expr_nodes.payload.float`).
    FloatConst {
        /// The value; always finite in a well-formed graph (§7.6: a null is never a value,
        /// and an infinity is never a bound).
        value: f64,
        /// The unit the literal was written in.
        unit: UnitId,
    },

    /// An integer literal (`math_expr_nodes.payload.integer`).
    IntConst {
        /// The value.
        value: i64,
    },

    /// An ordered constant plus coefficient/child terms (`math_expr_nodes.payload.affine`).
    Affine {
        /// The additive constant.
        constant: f64,
        /// Complete constant type; paired with its unit and required for a nonzero value.
        constant_quantity_type: Option<QuantityTypeId>,
        /// Canonical unit of the complete constant type; never inferred from the parent.
        constant_unit: Option<UnitId>,
        /// The ordered terms; their children are the node's children, in the same order.
        terms: Vec<AffineTerm>,
    },

    /// An ordered weighted mean with an explicit normalization (`math_expr_nodes.payload.weighted_mean`).
    WeightedMean {
        /// The ordered weight/value pairs; never empty.
        pairs: Vec<WeightedPair>,
        /// How the weights are normalized.
        normalization: WeightNormalization,
        /// The invariant certifying a unit weight sum; required by
        /// [`WeightNormalization::CertifiedUnitSum`].
        unit_sum_invariant: Option<InvariantId>,
    },

    /// A reduction over a bound index of a domain (`math_expr_nodes.payload.reduction`).
    Reduction {
        /// Which reduction; it must agree with the node's opcode.
        kind: ReductionKind,
        /// The domain reduced over.
        domain: DomainRef,
        /// The binder introduced by the reduction.
        bound_index: BoundIndexId,
        /// An optional filter on the bound index.
        filter: Option<GuardRef>,
    },

    /// A read of a group member at an explicit coordinate map (`math_expr_nodes.payload.gather`).
    Gather {
        /// The group being read from.
        group: SemanticId,
        /// Which binder supplies which coordinate position of the group's shape.
        coordinate_map: Vec<(BoundIndexId, u16)>,
    },

    /// Normalized indexed source read awaiting actual instance/domain lowering.
    PendingGather {
        /// Actual declared group identity before instance expansion.
        group: SemanticId,
        /// Ordered index expressions; none may be discarded or replaced by a hash.
        indices: Vec<NodeId>,
    },

    /// Normalized source occurrence retaining its complete instance-relative path.
    PendingPath {
        /// Exact source declaration identity.
        source_id: SemanticId,
        /// Source-local path ordinal; its ordered segment rows remain authoritative.
        path_id: i64,
        /// Flattened index expressions in segment order, partitioned by the path declaration.
        indices: Vec<NodeId>,
    },

    /// An explicit broadcast over a domain (`math_expr_nodes.payload.broadcast`).
    Broadcast {
        /// The domain broadcast over.
        domain: DomainRef,
        /// The binder introduced by the broadcast.
        bound_index: BoundIndexId,
    },

    /// A derivative with respect to a continuous domain (`math_expr_nodes.payload.derivative`).
    Derivative {
        /// The continuous domain differentiated with respect to.
        wrt_domain: DomainRef,
        /// The order of the derivative.
        order: u8,
    },

    /// An integral over a continuous domain (`math_expr_nodes.payload.integral`).
    Integral {
        /// The continuous domain integrated over.
        domain: DomainRef,
        /// Lexical binder introduced for the integration variable.
        bound_index: BoundIndexId,
        /// The quadrature policy, when the case pins one.
        quadrature_policy: Option<SemanticId>,
        /// Optional guarded integration domain predicate.
        filter: Option<GuardRef>,
    },

    /// Normalized unit-bearing tolerance awaiting the child's complete physical context.
    PendingSmoothOp {
        /// Positive finite authored tolerance value.
        eps: f64,
        /// Actual declared representation unit; never inferred from the operand name.
        unit: UnitId,
    },

    /// The smoothing parameter of a smooth or safe operator (`math_expr_nodes.payload.smooth`).
    SmoothOp {
        /// The smoothing parameter; §7.2 requires it to be positive.
        eps: f64,
    },

    /// The guard of a conditional (`math_expr_nodes.payload.conditional`).
    Conditional {
        /// The guard node. Phase 0 admits a boolean-kind `SymbolRef` or an `IntConst` 0/1
        /// decided by feature selection (§7.2).
        guard: GuardRef,
    },

    /// A call into a kernel binding (`math_expr_nodes.payload.kernel_call`).
    KernelCall {
        /// The kernel binding invoked.
        kernel_binding: SemanticId,
        /// Which declared output of the kernel this node is.
        output_ordinal: u16,
    },

    /// A reference to one unknown of an implicit system (`math_expr_nodes.payload.implicit_ref`).
    ImplicitRef {
        /// The implicit system.
        implicit_system: SemanticId,
        /// Which unknown of the system this node is.
        unknown_ordinal: u16,
    },

    /// A declared unit-conversion edge (`math_expr_nodes.payload.unit_convert`).
    UnitConvert(UnitConvertSpec),

    /// Normalized target-unit request, resolved only with a complete operand quantity.
    PendingUnitConvert {
        /// Explicit requested representation unit.
        to: UnitId,
    },

    /// A piecewise-linear interpolation over declared breakpoints
    /// (`math_expr_nodes.payload.piecewise_linear`).
    PiecewiseLinear {
        /// The `(x, y)` breakpoints, in the order they were declared.
        breakpoints: Vec<(f64, f64)>,
        /// The quantity type of the input coordinate.
        input: QuantityTypeId,
        /// The quantity type of the output value.
        output: QuantityTypeId,
    },
}

impl Payload {
    /// Storage retained while materializing a whole-node Arrow value.
    /// Fixed enum storage and owned strings/vectors are charged before conversion.
    /// # Errors
    /// The combined allocation extent cannot be represented by this process.
    pub fn allocation_extent(&self) -> Result<usize, crate::MathIrError> {
        let vector = match self {
            Self::Affine { terms, .. } => size_of_val(terms.as_slice()),
            Self::WeightedMean { pairs, .. } => size_of_val(pairs.as_slice()),
            Self::Gather { coordinate_map, .. } => size_of_val(coordinate_map.as_slice()),
            Self::PendingGather { indices, .. } | Self::PendingPath { indices, .. } => {
                size_of_val(indices.as_slice())
            }
            Self::PiecewiseLinear { breakpoints, .. } => size_of_val(breakpoints.as_slice()),
            _ => 0,
        };
        let reference = match self {
            Self::SymbolRef {
                symbol: ValueRef::Template { name, .. },
            } => name.len(),
            Self::SymbolRef {
                symbol: ValueRef::Domain(DomainRef::Template { domain_name, .. }),
            } => domain_name.len(),
            _ => 0,
        };
        let domain = match self.domain() {
            Some(DomainRef::Template { domain_name, .. }) => domain_name.len(),
            _ => 0,
        };
        [vector, reference, domain]
            .into_iter()
            .try_fold(size_of_val(self), usize::checked_add)
            .ok_or_else(|| {
                crate::MathIrError::malformed("whole-node value allocation extent overflow")
            })
    }

    /// Optional predicate dependency, preserving normalized predicate identity.
    pub fn guard(&self) -> Option<GuardRef> {
        match self {
            Self::Conditional { guard } => Some(*guard),
            Self::Reduction { filter, .. } | Self::Integral { filter, .. } => *filter,
            _ => None,
        }
    }
    /// Domain operand, when this operator carries one.
    pub fn domain(&self) -> Option<&DomainRef> {
        match self {
            Self::Reduction { domain, .. }
            | Self::Broadcast { domain, .. }
            | Self::Integral { domain, .. }
            | Self::Derivative {
                wrt_domain: domain, ..
            } => Some(domain),
            _ => None,
        }
    }

    /// The declared coherent payload discriminator.
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::SymbolRef { .. } => "symbol",
            Self::FloatConst { .. } => "float",
            Self::IntConst { .. } => "integer",
            Self::Affine { .. } => "affine",
            Self::WeightedMean { .. } => "weighted_mean",
            Self::Reduction { .. } => "reduction",
            Self::Gather { .. } => "gather",
            Self::PendingGather { .. } => "pending_gather",
            Self::PendingPath { .. } => "pending_path",
            Self::Broadcast { .. } => "broadcast",
            Self::Derivative { .. } => "derivative",
            Self::Integral { .. } => "integral",
            Self::SmoothOp { .. } => "smooth",
            Self::PendingSmoothOp { .. } => "pending_smooth",
            Self::Conditional { .. } => "conditional",
            Self::KernelCall { .. } => "kernel_call",
            Self::ImplicitRef { .. } => "implicit_ref",
            Self::UnitConvert(_) => "unit_convert",
            Self::PendingUnitConvert { .. } => "pending_unit_convert",
            Self::PiecewiseLinear { .. } => "piecewise_linear",
        }
    }

    /// Remap every payload-held node reference while retaining order and all values.
    ///
    /// # Errors
    /// Propagates a missing or invalid node reported by the supplied mapping.
    pub fn map_node_references(
        &mut self,
        mut map: impl FnMut(NodeId) -> Result<NodeId, crate::MathIrError>,
    ) -> Result<(), crate::MathIrError> {
        match self {
            Self::PendingGather { indices, .. } | Self::PendingPath { indices, .. } => {
                for index in indices {
                    *index = map(*index)?;
                }
            }
            Self::Affine { terms, .. } => {
                for term in terms {
                    term.child = map(term.child)?;
                }
            }
            Self::WeightedMean { pairs, .. } => {
                for pair in pairs {
                    pair.weight = map(pair.weight)?;
                    pair.value = map(pair.value)?;
                }
            }
            Self::Reduction {
                filter: Some(node), ..
            }
            | Self::Integral {
                filter: Some(node), ..
            }
            | Self::Conditional { guard: node } => {
                if let GuardRef::Math(id) = node {
                    *id = map(*id)?;
                }
            }
            Self::None
            | Self::SymbolRef { .. }
            | Self::FloatConst { .. }
            | Self::IntConst { .. }
            | Self::Reduction { filter: None, .. }
            | Self::Gather { .. }
            | Self::Broadcast { .. }
            | Self::Derivative { .. }
            | Self::Integral { filter: None, .. }
            | Self::SmoothOp { .. }
            | Self::PendingSmoothOp { .. }
            | Self::KernelCall { .. }
            | Self::ImplicitRef { .. }
            | Self::UnitConvert(_)
            | Self::PendingUnitConvert { .. }
            | Self::PiecewiseLinear { .. } => {}
        }
        Ok(())
    }

    /// Every node this payload references, in a deterministic order.
    ///
    /// A reference in a payload is as real as a child: [`ExprGraph::insert`] checks these
    /// exist, which is what makes a cycle unrepresentable through that API.
    ///
    /// [`ExprGraph::insert`]: crate::ExprGraph::insert
    pub fn referenced_nodes(&self) -> Vec<NodeId> {
        match self {
            Self::PendingGather { indices, .. } | Self::PendingPath { indices, .. } => {
                indices.clone()
            }
            Self::Affine { terms, .. } => terms.iter().map(|term| term.child).collect(),
            Self::WeightedMean { pairs, .. } => pairs
                .iter()
                .flat_map(|pair| [pair.weight, pair.value])
                .collect(),
            Self::Reduction { filter, .. } | Self::Integral { filter, .. } => {
                filter.iter().filter_map(|guard| guard.math()).collect()
            }
            Self::Conditional { guard } => guard.math().into_iter().collect(),
            Self::None
            | Self::SymbolRef { .. }
            | Self::FloatConst { .. }
            | Self::IntConst { .. }
            | Self::Gather { .. }
            | Self::Broadcast { .. }
            | Self::Derivative { .. }
            | Self::SmoothOp { .. }
            | Self::PendingSmoothOp { .. }
            | Self::KernelCall { .. }
            | Self::ImplicitRef { .. }
            | Self::UnitConvert(_)
            | Self::PendingUnitConvert { .. }
            | Self::PiecewiseLinear { .. } => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use pse_ids::SemanticId;
    use pse_quantity::{BoundIndexId, DomainId, ReductionKind, WeightNormalization};

    use super::{AffineTerm, Payload, WeightedPair};
    use crate::node::NodeId;

    #[test]
    fn a_payload_names_its_coherent_value_alternative() {
        assert_eq!(Payload::None.kind(), "none");
        assert_eq!(Payload::IntConst { value: 1 }.kind(), "integer");
        assert_eq!(Payload::SmoothOp { eps: 1e-3 }.kind(), "smooth");
    }

    #[test]
    fn an_affine_payload_references_its_term_children() {
        let payload = Payload::Affine {
            constant: 1.0,
            constant_quantity_type: Some(pse_quantity::QuantityTypeId::from_id(SemanticId::NIL)),
            constant_unit: Some(pse_quantity::UnitId::from_id(SemanticId::NIL)),
            terms: vec![
                AffineTerm {
                    coefficient: 2.0,
                    child: NodeId(3),
                },
                AffineTerm {
                    coefficient: -1.0,
                    child: NodeId(4),
                },
            ],
        };
        assert_eq!(payload.referenced_nodes(), vec![NodeId(3), NodeId(4)]);
    }

    #[test]
    fn a_weighted_mean_references_weights_before_values() {
        let payload = Payload::WeightedMean {
            pairs: vec![WeightedPair {
                weight: NodeId(1),
                value: NodeId(2),
            }],
            normalization: WeightNormalization::DivideBySum,
            unit_sum_invariant: None,
        };
        assert_eq!(payload.referenced_nodes(), vec![NodeId(1), NodeId(2)]);
    }

    #[test]
    fn a_reduction_references_only_its_filter() {
        let with_filter = Payload::Reduction {
            kind: ReductionKind::Sum,
            domain: DomainId::from_id(SemanticId::NIL).into(),
            bound_index: BoundIndexId::from_id(SemanticId::NIL),
            filter: Some(NodeId(9).into()),
        };
        assert_eq!(with_filter.referenced_nodes(), vec![NodeId(9)]);

        let without = Payload::Reduction {
            kind: ReductionKind::Sum,
            domain: DomainId::from_id(SemanticId::NIL).into(),
            bound_index: BoundIndexId::from_id(SemanticId::NIL),
            filter: None,
        };
        assert!(without.referenced_nodes().is_empty());
    }

    #[test]
    fn a_conditional_references_its_guard() {
        let payload = Payload::Conditional {
            guard: NodeId(5).into(),
        };
        assert_eq!(payload.referenced_nodes(), vec![NodeId(5)]);
    }
}
