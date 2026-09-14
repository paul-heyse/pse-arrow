// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The identity newtypes of the physical-typing registry (blueprint §5.1, §6.2, §6.3).
//!
//! Every one of these is a [`SemanticId`] and nothing else at runtime; the wrapper exists
//! so that the compiler refuses the substitution the design refuses. A `UnitId` where a
//! `QuantityTypeId` belongs is the kind of mistake that produces a plausible-looking
//! lookup miss rather than a type error, and §8.1 is built entirely out of the difference
//! between these tuple components.
//!
//! `pse-ids` owns no `serde` dependency, so these types are not `Serialize`: identities
//! reach storage as `pse.semantic_id` Arrow columns, not as JSON fields.
//!
//! [`SemanticId`]: pse_ids::SemanticId

/// Declares newtypes over [`pse_ids::SemanticId`] with the identity type's own derives.
///
/// Each generated type carries `from_id`/`as_id`, the two `From` conversions and a
/// `Display` that renders the underlying lowercase hexadecimal, so a diagnostic can name
/// the identity without reaching for `Debug`.
///
/// ```
/// pse_quantity::semantic_id_newtype! {
///     /// Identifies a widget.
///     WidgetId,
/// }
///
/// let id = WidgetId::from_id(pse_ids::SemanticId::from_bytes([7; 16]));
/// assert_eq!(id.as_id().to_hex(), "07070707070707070707070707070707");
/// ```
#[macro_export]
macro_rules! semantic_id_newtype {
    (
        $(
            $(#[$meta:meta])*
            $name:ident
        ),+ $(,)?
    ) => {
        $(
            $(#[$meta])*
            #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
            pub struct $name(::pse_ids::SemanticId);

            impl $name {
                #[doc = concat!("Wraps a [`SemanticId`] as a [`", stringify!($name), "`].")]
                ///
                /// [`SemanticId`]: pse_ids::SemanticId
                pub const fn from_id(id: ::pse_ids::SemanticId) -> Self {
                    Self(id)
                }

                #[doc = concat!("The [`SemanticId`] inside this [`", stringify!($name), "`].")]
                ///
                /// [`SemanticId`]: pse_ids::SemanticId
                pub const fn as_id(self) -> ::pse_ids::SemanticId {
                    self.0
                }
            }

            impl ::core::convert::From<::pse_ids::SemanticId> for $name {
                fn from(id: ::pse_ids::SemanticId) -> Self {
                    Self(id)
                }
            }

            impl ::core::convert::From<$name> for ::pse_ids::SemanticId {
                fn from(value: $name) -> Self {
                    value.0
                }
            }

            impl ::core::fmt::Display for $name {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    ::core::fmt::Display::fmt(&self.0, f)
                }
            }
        )+
    };
}

crate::semantic_id_newtype! {
    /// Identifies a unit (`reference.units`, blueprint §6.2).
    UnitId,
    /// Identifies a unit set, the per-package choice of base units (`reference.unit_sets`).
    UnitSetId,
    /// Identifies a quantity kind — what is measured, before basis and datum
    /// (`reference.quantity_kinds`).
    QuantityKindId,
    /// Identifies a basis: what a specific quantity is specific to (`reference.bases`).
    BasisId,
    /// Identifies a reference state: the datum an origin-sensitive quantity is measured
    /// from (`reference.reference_states`).
    ReferenceStateId,
    /// Identifies a complete quantity type: the §8.1 tuple
    /// (`reference.quantity_types`).
    QuantityTypeId,
    /// Identifies a registered conversion between two quantity types
    /// (`reference.conversion_rules`).
    ConversionId,
    /// Identifies a registered quantity operation — the §8.3 composition policy
    /// (`reference.quantity_operations`).
    OperationId,
    /// Identifies a physical constant (`reference.constants`).
    ConstantId,
    /// Identifies a domain: what an index ranges over (`authored.domains`, §6.3).
    DomainId,
    /// Identifies a bound index — one binding occurrence of a domain in an expression,
    /// not the domain itself (§6.9, `bound_index_id`).
    BoundIndexId,
    /// Identifies an invariant a rule or an operation depends on (`schema` invariants,
    /// §4.1).
    InvariantId,
}

#[cfg(test)]
mod tests {
    use pse_ids::SemanticId;

    use super::{QuantityTypeId, UnitId};

    #[test]
    fn a_newtype_round_trips_through_its_identity() {
        let raw = SemanticId::from_bytes([0x11; 16]);
        let unit = UnitId::from_id(raw);
        assert_eq!(unit.as_id(), raw);
        assert_eq!(SemanticId::from(unit), raw);
        assert_eq!(UnitId::from(raw), unit);
    }

    #[test]
    fn display_renders_the_underlying_hexadecimal() {
        let id = QuantityTypeId::from_id(SemanticId::from_bytes([0xab; 16]));
        assert_eq!(id.to_string(), "abababababababababababababababab");
    }

    /// Ordering follows the identity bytes, so a `BTreeMap` keyed by one of these is
    /// deterministic without a separate comparator.
    #[test]
    fn ordering_follows_the_identity_bytes() {
        let low = UnitId::from_id(SemanticId::from_bytes([0x00; 16]));
        let high = UnitId::from_id(SemanticId::from_bytes([0x01; 16]));
        assert!(low < high);
    }
}
