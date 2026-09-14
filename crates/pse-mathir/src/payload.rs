// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The typed operator payloads (blueprint §6.9 `compiled.math_*`, §7.2).
//!
//! §6.9 stores one relation per operator family that needs data beyond its children —
//! `math_float_constants`, `math_affine`, `math_weighted_means`, `math_reductions` and the
//! rest. [`Payload`] is the in-memory union of exactly those relations, one variant per
//! relation, so the row model and the graph model cannot drift into different field sets.
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
    BoundIndexId, DomainId, InvariantId, QuantityTypeId, ReductionKind, UnitConvertSpec, UnitId,
    WeightNormalization,
};

use crate::node::NodeId;

/// One ordered term of an [`Payload::Affine`] node (blueprint §6.9 `math_affine.terms`).
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
/// (blueprint §6.9 `math_weighted_means.pairs`).
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

    /// A reference to a symbol (`math_symbol_refs`); the symbol supplies the type.
    SymbolRef {
        /// The symbol's identity.
        symbol: SemanticId,
    },

    /// A floating-point literal with the unit it was written in
    /// (`math_float_constants`).
    FloatConst {
        /// The value; always finite in a well-formed graph (§7.6: a null is never a value,
        /// and an infinity is never a bound).
        value: f64,
        /// The unit the literal was written in.
        unit: UnitId,
    },

    /// An integer literal (`math_int_constants`).
    IntConst {
        /// The value.
        value: i64,
    },

    /// An ordered constant plus coefficient/child terms (`math_affine`).
    Affine {
        /// The additive constant.
        constant: f64,
        /// The ordered terms; their children are the node's children, in the same order.
        terms: Vec<AffineTerm>,
    },

    /// An ordered weighted mean with an explicit normalization (`math_weighted_means`).
    WeightedMean {
        /// The ordered weight/value pairs; never empty.
        pairs: Vec<WeightedPair>,
        /// How the weights are normalized.
        normalization: WeightNormalization,
        /// The invariant certifying a unit weight sum; required by
        /// [`WeightNormalization::CertifiedUnitSum`].
        unit_sum_invariant: Option<InvariantId>,
    },

    /// A reduction over a bound index of a domain (`math_reductions`).
    Reduction {
        /// Which reduction; it must agree with the node's opcode.
        kind: ReductionKind,
        /// The domain reduced over.
        domain: DomainId,
        /// The binder introduced by the reduction.
        bound_index: BoundIndexId,
        /// An optional filter on the bound index.
        filter: Option<NodeId>,
    },

    /// A read of a group member at an explicit coordinate map (`math_gathers`).
    Gather {
        /// The group being read from.
        group: SemanticId,
        /// Which binder supplies which coordinate position of the group's shape.
        coordinate_map: Vec<(BoundIndexId, u16)>,
    },

    /// An explicit broadcast over a domain (`math_broadcasts`).
    Broadcast {
        /// The domain broadcast over.
        domain: DomainId,
        /// The binder introduced by the broadcast.
        bound_index: BoundIndexId,
    },

    /// A derivative with respect to a continuous domain (`math_derivatives`).
    Derivative {
        /// The continuous domain differentiated with respect to.
        wrt_domain: DomainId,
        /// The order of the derivative.
        order: u8,
    },

    /// An integral over a continuous domain (`math_integrals`).
    Integral {
        /// The continuous domain integrated over.
        domain: DomainId,
        /// The quadrature policy, when the case pins one.
        quadrature_policy: Option<SemanticId>,
    },

    /// The smoothing parameter of a smooth or safe operator (`math_smooth_ops`).
    SmoothOp {
        /// The smoothing parameter; §7.2 requires it to be positive.
        eps: f64,
    },

    /// The guard of a conditional (`math_conditionals`).
    Conditional {
        /// The guard node. Phase 0 admits a boolean-kind `SymbolRef` or an `IntConst` 0/1
        /// decided by feature selection (§7.2).
        guard: NodeId,
    },

    /// A call into a kernel binding (`math_kernel_calls`).
    KernelCall {
        /// The kernel binding invoked.
        kernel_binding: SemanticId,
        /// Which declared output of the kernel this node is.
        output_ordinal: u16,
    },

    /// A reference to one unknown of an implicit system (`math_implicit_refs`).
    ImplicitRef {
        /// The implicit system.
        implicit_system: SemanticId,
        /// Which unknown of the system this node is.
        unknown_ordinal: u16,
    },

    /// A declared unit-conversion edge (`math_unit_converts`).
    UnitConvert(UnitConvertSpec),

    /// A piecewise-linear interpolation over declared breakpoints
    /// (`math_piecewise_linear`).
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
    /// The relation name of §6.9 this payload is stored in, or `None` for
    /// [`Payload::None`].
    ///
    /// Naming the relation in a diagnostic is more useful than naming the Rust variant,
    /// because the relation is what a reader can go and query.
    pub const fn relation_name(&self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::SymbolRef { .. } => Some("math_symbol_refs"),
            Self::FloatConst { .. } => Some("math_float_constants"),
            Self::IntConst { .. } => Some("math_int_constants"),
            Self::Affine { .. } => Some("math_affine"),
            Self::WeightedMean { .. } => Some("math_weighted_means"),
            Self::Reduction { .. } => Some("math_reductions"),
            Self::Gather { .. } => Some("math_gathers"),
            Self::Broadcast { .. } => Some("math_broadcasts"),
            Self::Derivative { .. } => Some("math_derivatives"),
            Self::Integral { .. } => Some("math_integrals"),
            Self::SmoothOp { .. } => Some("math_smooth_ops"),
            Self::Conditional { .. } => Some("math_conditionals"),
            Self::KernelCall { .. } => Some("math_kernel_calls"),
            Self::ImplicitRef { .. } => Some("math_implicit_refs"),
            Self::UnitConvert(_) => Some("math_unit_converts"),
            Self::PiecewiseLinear { .. } => Some("math_piecewise_linear"),
        }
    }

    /// Every node this payload references, in a deterministic order.
    ///
    /// A reference in a payload is as real as a child: [`ExprGraph::insert`] checks these
    /// exist, which is what makes a cycle unrepresentable through that API.
    ///
    /// [`ExprGraph::insert`]: crate::ExprGraph::insert
    pub fn referenced_nodes(&self) -> Vec<NodeId> {
        match self {
            Self::Affine { terms, .. } => terms.iter().map(|term| term.child).collect(),
            Self::WeightedMean { pairs, .. } => pairs
                .iter()
                .flat_map(|pair| [pair.weight, pair.value])
                .collect(),
            Self::Reduction { filter, .. } => filter.iter().copied().collect(),
            Self::Conditional { guard } => vec![*guard],
            Self::None
            | Self::SymbolRef { .. }
            | Self::FloatConst { .. }
            | Self::IntConst { .. }
            | Self::Gather { .. }
            | Self::Broadcast { .. }
            | Self::Derivative { .. }
            | Self::Integral { .. }
            | Self::SmoothOp { .. }
            | Self::KernelCall { .. }
            | Self::ImplicitRef { .. }
            | Self::UnitConvert(_)
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
    fn a_payload_names_the_relation_it_is_stored_in() {
        assert_eq!(Payload::None.relation_name(), None);
        assert_eq!(
            Payload::IntConst { value: 1 }.relation_name(),
            Some("math_int_constants")
        );
        assert_eq!(
            Payload::SmoothOp { eps: 1e-3 }.relation_name(),
            Some("math_smooth_ops")
        );
    }

    #[test]
    fn an_affine_payload_references_its_term_children() {
        let payload = Payload::Affine {
            constant: 1.0,
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
            domain: DomainId::from_id(SemanticId::NIL),
            bound_index: BoundIndexId::from_id(SemanticId::NIL),
            filter: Some(NodeId(9)),
        };
        assert_eq!(with_filter.referenced_nodes(), vec![NodeId(9)]);

        let without = Payload::Reduction {
            kind: ReductionKind::Sum,
            domain: DomainId::from_id(SemanticId::NIL),
            bound_index: BoundIndexId::from_id(SemanticId::NIL),
            filter: None,
        };
        assert!(without.referenced_nodes().is_empty());
    }

    #[test]
    fn a_conditional_references_its_guard() {
        let payload = Payload::Conditional { guard: NodeId(5) };
        assert_eq!(payload.referenced_nodes(), vec![NodeId(5)]);
    }
}
