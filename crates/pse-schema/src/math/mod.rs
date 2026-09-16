// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Mathematical declarations shared by generated relations and their consumers.
//! This module contains contracts, not graph implementations or execution state.
use pse_quantity::Opcode;
use serde::{Deserialize, Serialize};

pub mod operators;

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
/// use pse_schema::math::{Arity, arity};
/// use pse_quantity::Opcode;
///
/// assert_eq!(arity(Opcode::Add), Arity::Fixed(2));
/// assert_eq!(arity(Opcode::Conditional), Arity::Fixed(2));
/// assert_eq!(arity(Opcode::Affine), Arity::Variadic);
/// assert_eq!(arity(Opcode::WeightedMean), Arity::Payload);
/// ```
pub const fn arity(opcode: Opcode) -> Arity {
    operators::operator_spec(opcode).arity
}

pse_quantity::closed_enum! {
    /// The authoritative equation-sense dictionary of blueprint §6.9.
    pub enum Sense {
        /// Equality to the supplied bound.
        Eq => "eq",
        /// Body no greater than the upper bound.
        Le => "le",
        /// Body no less than the lower bound.
        Ge => "ge",
        /// Body between both bounds.
        Range => "range",
        /// A defining equality.
        Definition => "definition",
    }
}
/// Which declared template member supplies an unresolved value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TemplateValueKind {
    /// A declared parameter.
    Parameter,
    /// A declared feature.
    Feature,
    /// A declared port.
    Port,
}
impl TemplateValueKind {
    /// Closed declaration vocabulary, used by schema projections.
    pub const ALL: [Self; 3] = [Self::Parameter, Self::Feature, Self::Port];
    /// Stable declaration spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Parameter => "parameter",
            Self::Feature => "feature",
            Self::Port => "port",
        }
    }
}
/// Normalized declaration discriminators; actual keys remain on the selected arm.
pub const NORMALIZED_REFERENCE_KINDS: [&str; 6] = [
    "symbol",
    TemplateValueKind::Parameter.as_str(),
    TemplateValueKind::Feature.as_str(),
    TemplateValueKind::Port.as_str(),
    "domain",
    "index",
];
/// An unresolved instance path, distinct from an already bound declaration.
pub const PENDING_PATH_KIND: &str = "path";
