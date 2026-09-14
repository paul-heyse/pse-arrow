// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The node of the expression DAG and its arity (blueprint §6.9, §7.1, §7.2).
//!
//! A node is an opcode, an ordered child list, a typed payload and — after P10 — a
//! resolved quantity type. The ordering of `children` *is* `math_expr_args.argument_ordinal`
//! (§7.1 constraint 1): it is the authoritative argument order, and nothing here sorts it,
//! not even for commutative operators, because §7.4 step 1 forbids reassociating a graph
//! whose evaluation order is observable.
//!
//! [`arity`] projects the single `OPERATOR_TABLE`; no second arity declaration exists.

use pse_ids::SemanticId;
use pse_quantity::{Opcode, QuantityTypeId};
use serde::{Deserialize, Serialize};

use crate::payload::Payload;

/// An artifact-local node number (blueprint §5.1, §6.9 `math_expr_nodes.node_id`).
///
/// An ordinal, not an identity: it is meaningful only inside the graph that assigned it,
/// and it is never derived from a name or a value. A canonical graph renumbers in
/// deterministic post-order (§7.4 step 5), so a `NodeId` from an uncanonicalized
/// [`ExprGraph`] is not the `NodeId` the stored relation will carry.
///
/// [`ExprGraph`]: crate::ExprGraph
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct NodeId(pub u64);

impl core::fmt::Display for NodeId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "#{}", self.0)
    }
}

/// How many children an operator takes (blueprint §7.3, `operator_specs.arity`).
///
/// `Display` renders the registry spelling — `fixed 2`, `variadic`, `payload` — so the
/// relation and this type cannot drift into two different vocabularies.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum Arity {
    /// Exactly this many children, in argument order.
    Fixed(u8),
    /// Any number of children; the payload says what they mean.
    Variadic,
    /// No children at all: every operand lives in the payload.
    Payload,
}

impl Arity {
    /// The exact child count, when the arity fixes one.
    pub const fn child_count(self) -> Option<u8> {
        match self {
            Self::Fixed(count) => Some(count),
            Self::Variadic | Self::Payload => None,
        }
    }

    /// Does a child list of this length satisfy the arity?
    pub const fn admits(self, children: usize) -> bool {
        match self {
            Self::Fixed(count) => children == count as usize,
            Self::Variadic => true,
            Self::Payload => children == 0,
        }
    }
}

impl core::fmt::Display for Arity {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Fixed(count) => write!(f, "fixed {count}"),
            Self::Variadic => f.write_str("variadic"),
            Self::Payload => f.write_str("payload"),
        }
    }
}

/// The arity of an operator (blueprint §7.2).
///
/// Three families are worth reading twice, because each puts an operand somewhere other
/// than the child list:
///
/// - `Conditional` is `Fixed(2)` — the two branches. The guard is in the payload, so that
///   sharing a subtree with a branch cannot hoist its evaluation out of the branch
///   (§7.4 step 1).
/// - `SumOver`, `ProdOver`, `MinOver` and `MaxOver` are `Fixed(1)` — the body. The domain,
///   the binder and the optional filter are in the payload.
/// - `WeightedMean` is `Payload`: its ordered weight/value pairs reference nodes directly,
///   because a flat child list could not say which weight belongs to which value.
///
/// ```
/// use pse_mathir::{Arity, Opcode, arity};
///
/// assert_eq!(arity(Opcode::Add), Arity::Fixed(2));
/// assert_eq!(arity(Opcode::Conditional), Arity::Fixed(2));
/// assert_eq!(arity(Opcode::Affine), Arity::Variadic);
/// assert_eq!(arity(Opcode::WeightedMean), Arity::Payload);
/// ```
pub const fn arity(opcode: Opcode) -> Arity {
    crate::opspec::operator_spec(opcode).arity
}

/// One node of the expression DAG (blueprint §6.9 `compiled.math_expr_nodes`).
///
/// `quantity_type` is `None` until P10 has run and never `None` after it: unit inference
/// is a pass, not a runtime check (§7.1 constraint 3). `scope` names the instance the node
/// belongs to; it is excluded from structural hashing (§7.4 step 1), so the same
/// expression in two instances is one node with one hash.
#[derive(Clone, PartialEq, Debug)]
pub struct Node {
    /// Which operator this node applies.
    pub opcode: Opcode,
    /// The typed operator payload; `Payload::None` for the operators that need none.
    pub payload: Payload,
    /// The children, in `argument_ordinal` order. Never sorted, not even for `Add`.
    pub children: Vec<NodeId>,
    /// The complete quantity type, resolved by P10 (`None` before it).
    pub quantity_type: Option<QuantityTypeId>,
    /// The instance the node belongs to; excluded from the structural hash.
    pub scope: Option<SemanticId>,
}

impl Node {
    /// The arity this node's operator declares.
    pub const fn arity(&self) -> Arity {
        arity(self.opcode)
    }
}

#[cfg(test)]
mod tests {
    use pse_quantity::Opcode;

    use super::{Arity, NodeId, arity};

    /// Every opcode has an arity, and the four families partition the 43 operators.
    #[test]
    fn every_opcode_has_an_arity() {
        let mut payload = 0;
        let mut binary = 0;
        let mut unary = 0;
        let mut variadic = 0;
        for opcode in Opcode::ALL {
            match arity(*opcode) {
                Arity::Payload => payload += 1,
                Arity::Fixed(2) => binary += 1,
                Arity::Fixed(1) => unary += 1,
                Arity::Variadic => variadic += 1,
                Arity::Fixed(other) => panic!("unexpected fixed arity {other} for {opcode}"),
            }
        }
        assert_eq!(payload, 6);
        assert_eq!(binary, 8);
        assert_eq!(unary, 28);
        assert_eq!(variadic, 1);
        assert_eq!(payload + binary + unary + variadic, Opcode::ALL.len());
    }

    /// The operators that keep an operand out of the child list (blueprint §7.2).
    #[test]
    fn the_payload_carrying_operators_take_no_children() {
        for opcode in [
            Opcode::Const,
            Opcode::SymbolRef,
            Opcode::WeightedMean,
            Opcode::KernelCall,
            Opcode::ImplicitRef,
            Opcode::Gather,
        ] {
            assert_eq!(arity(opcode), Arity::Payload);
            assert!(arity(opcode).admits(0));
            assert!(!arity(opcode).admits(1));
        }
    }

    /// A reduction takes only its body; the binder and the filter are payload.
    #[test]
    fn a_reduction_takes_only_its_body() {
        for opcode in [
            Opcode::SumOver,
            Opcode::ProdOver,
            Opcode::MinOver,
            Opcode::MaxOver,
        ] {
            assert_eq!(arity(opcode), Arity::Fixed(1));
        }
    }

    /// A conditional takes the two branches; the guard is payload, so sharing a subtree
    /// cannot hoist evaluation out of a branch.
    #[test]
    fn a_conditional_takes_its_two_branches() {
        assert_eq!(arity(Opcode::Conditional), Arity::Fixed(2));
        assert_eq!(arity(Opcode::Conditional).child_count(), Some(2));
    }

    #[test]
    fn arity_renders_the_registry_spelling() {
        assert_eq!(Arity::Fixed(2).to_string(), "fixed 2");
        assert_eq!(Arity::Variadic.to_string(), "variadic");
        assert_eq!(Arity::Payload.to_string(), "payload");
        assert_eq!(Arity::Variadic.child_count(), None);
    }

    #[test]
    fn a_node_id_renders_as_an_artifact_local_number() {
        assert_eq!(NodeId(7).to_string(), "#7");
    }
}
