// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Every error this crate returns, with its blueprint §23.2 class.
//!
//! The classes follow §23.2's `compile.math` family, spelled in Rust path form.
//!
//! [`MathIrError::Quantity`] wraps a physical-typing failure together with the node it
//! happened at. `miette`'s `#[diagnostic(transparent)]` accepts a variant with exactly one
//! field, so the variant cannot both carry the node and inherit the inner class; it
//! carries `compile::math::unit_inconsistent` — §23.2's class for incompatible physical
//! contracts — and marks the wrapped error `#[diagnostic_source]`, so the precise class
//! `pse-quantity` assigned (`quantity_operation_unsupported`, `domain_violation_static`,
//! ...) stays reachable through `miette::Diagnostic::diagnostic_source` and in the
//! rendered report.
//!
//! [`MathIrError::Malformed`] is `internal::invariant` on purpose. An arity mismatch or a
//! dangling node reference is not an authoring mistake that reached this far: P3 and the
//! DSL parser reject those, so a malformed insert means a pass built a bad graph.

use pse_ids::SemanticId;
use pse_quantity::{BoundIndexId, Opcode, QuantityError, UnitId};

use crate::node::NodeId;

/// Every failure of the mathematical IR (blueprint §7.4, §23.2).
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[non_exhaustive]
pub enum MathIrError {
    /// Shared resource/cancellation failure at the typed algorithm boundary.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Canon(#[from] pse_ids::CanonError),
    /// The expression graph contains a cycle.
    ///
    /// Unreachable through [`ExprGraph::insert`], which only accepts children that already
    /// exist; it is reachable when a graph is loaded from relations, where the node
    /// numbering is data.
    ///
    /// [`ExprGraph::insert`]: crate::ExprGraph::insert
    #[error("the expression graph is cyclic through {} nodes", .path.len())]
    #[diagnostic(code(compile::math::cyclic_expression))]
    Cycle {
        /// The cycle, in traversal order, starting and ending at the repeated node.
        path: Vec<NodeId>,
    },

    /// Physical typing failed at a node.
    ///
    /// The wrapped error keeps its own §23.2 class, reachable through
    /// `miette::Diagnostic::diagnostic_source`.
    #[error("node {node}: {source}")]
    #[diagnostic(code(compile::math::unit_inconsistent))]
    Quantity {
        /// The node whose typing failed.
        node: NodeId,
        /// The §8.3 failure, with its own §23.2 class.
        #[source]
        #[diagnostic_source]
        source: QuantityError,
    },

    /// A literal operand violates an operator's domain restriction before any solve.
    #[error("node {node}: `{opcode}` requires {restriction}, but the literal operand is {value}")]
    #[diagnostic(code(compile::math::domain_violation_static))]
    StaticDomain {
        /// The node that violates the restriction.
        node: NodeId,
        /// The operator whose restriction was violated.
        opcode: Opcode,
        /// The restriction, spelled as §7.2 spells it.
        restriction: &'static str,
        /// The literal value that violates it.
        value: f64,
    },

    /// A literal is not finite.
    ///
    /// §7.6 leaves no room for one: an unbounded bound is `pse.bound{kind: unbounded}`, a
    /// missing measurement is a null, and an invalid evaluation is a typed failure. An
    /// infinity or a NaN in a literal is therefore a malformed value, and the bits are
    /// reported verbatim — not canonicalized — because the payload of the NaN is the
    /// diagnostic (ADR-0030).
    #[error("a literal must be finite; its bits are {value_bits:#018x}")]
    #[diagnostic(code(compile::math::domain_violation_static))]
    NonFiniteLiteral {
        /// The raw IEEE-754 bits of the offending literal.
        value_bits: u64,
    },

    /// A node still carries a non-canonical unit where §7.2 allows none.
    ///
    /// After P10 an affine unit may appear only on the input of a `UnitConvert` or of a
    /// kernel port; anywhere else it means a conversion edge was never inserted.
    #[error("node {node} carries the non-canonical unit `{unit}` outside a conversion edge")]
    #[diagnostic(code(compile::math::unit_inconsistent))]
    NonCanonicalUnitEscapes {
        /// The node carrying the unit.
        node: NodeId,
        /// The unit that escaped.
        unit: UnitId,
    },

    /// A conditional's guard is not a boolean node.
    #[error("node {node} is used as a conditional guard but is not boolean")]
    #[diagnostic(code(compile::math::unit_inconsistent))]
    GuardNotBoolean {
        /// The guard node.
        node: NodeId,
    },

    /// A node references a binder that nothing in scope binds.
    #[error("node {node} is free in an unbound index `{bound_index}`")]
    #[diagnostic(code(compile::math::quantity_operation_unsupported))]
    UnboundIndex {
        /// The node that is free in the binder.
        node: NodeId,
        /// The binder that nothing binds.
        bound_index: BoundIndexId,
    },

    /// A node names a kernel binding, implicit system or group that does not resolve.
    #[error("node {node} names the unknown binding `{binding}`")]
    #[diagnostic(code(compile::math::quantity_operation_unsupported))]
    UnknownBinding {
        /// The node carrying the reference.
        node: NodeId,
        /// The identity that does not resolve.
        binding: SemanticId,
    },

    /// A normalized declaration or predicate still requires its explicit lowering pass.
    #[error("node {node} still names an unresolved {kind} reference")]
    #[diagnostic(code(compile::math::quantity_operation_unsupported))]
    UnresolvedValue {
        /// Expression using the unresolved reference.
        node: NodeId,
        /// Explicit normalized reference category.
        kind: &'static str,
    },

    /// A template-local domain has not been resolved by instance binding.
    #[error("node {node} still names template {template_id} domain `{domain_name}`")]
    #[diagnostic(code(compile::math::quantity_operation_unsupported))]
    UnresolvedDomain {
        /// Expression requiring an actual domain.
        node: NodeId,
        /// Owning template declaration.
        template_id: SemanticId,
        /// Exact local domain name.
        domain_name: String,
    },

    /// A graph was built that does not satisfy the node model.
    #[error("malformed expression node: {detail}")]
    #[diagnostic(code(internal::invariant))]
    Malformed {
        /// The node, when one exists; `None` while it is still being built.
        node: Option<NodeId>,
        /// What specifically was wrong, naming the operator and the expectation.
        detail: String,
    },
}

impl MathIrError {
    /// A [`MathIrError::Malformed`] for a node that does not exist yet.
    pub(crate) fn malformed(detail: impl Into<String>) -> Self {
        Self::Malformed {
            node: None,
            detail: detail.into(),
        }
    }

    /// A [`MathIrError::Malformed`] naming an existing node.
    pub(crate) fn malformed_at(node: NodeId, detail: impl Into<String>) -> Self {
        Self::Malformed {
            node: Some(node),
            detail: detail.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use miette::Diagnostic as _;
    use pse_ids::SemanticId;
    use pse_quantity::{Opcode, QuantityError};

    use super::MathIrError;
    use crate::node::NodeId;

    fn code_of(error: &MathIrError) -> String {
        error
            .code()
            .map(|code| code.to_string())
            .unwrap_or_default()
    }

    #[test]
    fn each_variant_carries_its_section_23_2_class() {
        assert_eq!(
            code_of(&MathIrError::Cycle {
                path: vec![NodeId(0), NodeId(1), NodeId(0)],
            }),
            "compile::math::cyclic_expression"
        );
        assert_eq!(
            code_of(&MathIrError::NonFiniteLiteral { value_bits: 0 }),
            "compile::math::domain_violation_static"
        );
        assert_eq!(
            code_of(&MathIrError::GuardNotBoolean { node: NodeId(0) }),
            "compile::math::unit_inconsistent"
        );
        assert_eq!(
            code_of(&MathIrError::malformed("no reason")),
            "internal::invariant"
        );
    }

    /// A wrapped quantity failure keeps the class `pse-quantity` gave it, reachable as the
    /// diagnostic source beside the `compile::math` class of the wrapper.
    #[test]
    fn a_quantity_failure_keeps_its_own_class_as_its_source() {
        let error = MathIrError::Quantity {
            node: NodeId(2),
            source: QuantityError::OperationUnsupported {
                opcode: Opcode::Mul,
                input_kinds: Vec::new(),
                ordered_matches: 0,
                swapped_matches: 0,
            },
        };
        assert_eq!(code_of(&error), "compile::math::unit_inconsistent");
        assert_eq!(
            error
                .diagnostic_source()
                .and_then(miette::Diagnostic::code)
                .map(|code| code.to_string()),
            Some("compile::math::quantity_operation_unsupported".to_owned())
        );
        assert!(error.to_string().starts_with("node #2:"));
    }

    /// A non-finite literal reports its bits verbatim, so a NaN payload survives into the
    /// diagnostic.
    #[test]
    fn a_non_finite_literal_reports_its_bits() {
        let error = MathIrError::NonFiniteLiteral {
            value_bits: f64::INFINITY.to_bits(),
        };
        assert!(error.to_string().contains("0x7ff0000000000000"));
    }

    #[test]
    fn malformed_can_name_the_node_it_found() {
        let error = MathIrError::malformed_at(NodeId(4), "`Add` takes fixed 2 children");
        assert!(matches!(
            error,
            MathIrError::Malformed {
                node: Some(NodeId(4)),
                ..
            }
        ));
    }

    #[test]
    fn a_static_domain_violation_names_the_restriction() {
        let error = MathIrError::StaticDomain {
            node: NodeId(1),
            opcode: Opcode::SmoothMax,
            restriction: "eps > 0",
            value: 0.0,
        };
        assert!(error.to_string().contains("eps > 0"));
        assert_eq!(code_of(&error), "compile::math::domain_violation_static");
    }

    #[test]
    fn an_unbound_index_names_the_binder() {
        let error = MathIrError::UnboundIndex {
            node: NodeId(3),
            bound_index: pse_quantity::BoundIndexId::from_id(SemanticId::from_bytes([0x0f; 16])),
        };
        assert!(
            error
                .to_string()
                .contains("0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f")
        );
    }
}
