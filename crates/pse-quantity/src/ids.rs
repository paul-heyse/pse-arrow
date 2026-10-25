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
//! The wrappers are declared through [`pse_ids::semantic_id_newtype!`], which every typed
//! identity in the workspace shares: storage, serde and framing see the base identity.
//!
//! [`SemanticId`]: pse_ids::SemanticId

pse_ids::semantic_id_newtype! {
    /// Package-declared entity kind used by physical axes and subjects.
    EntityKindId: RegistrySemanticId,
    /// Identifies a unit (`reference.units`, blueprint §6.2).
    UnitId: RegistrySemanticId,
    /// Identifies a unit set, the per-package choice of base units (`reference.unit_sets`).
    UnitSetId,
    /// Identifies a quantity kind — what is measured, before basis and datum
    /// (`reference.quantity_kinds`).
    QuantityKindId: RegistrySemanticId,
    /// Identifies a basis: what a specific quantity is specific to (`reference.bases`).
    BasisId: RegistrySemanticId,
    /// Identifies a reference state: the datum an origin-sensitive quantity is measured
    /// from (`reference.reference_states`).
    ReferenceStateId: RegistrySemanticId,
    /// Identifies a complete quantity type: the §8.1 tuple
    /// (`reference.quantity_types`).
    QuantityTypeId: RegistrySemanticId,
    /// Identifies a registered conversion between two quantity types
    /// (`reference.conversion_rules`).
    ConversionId,
    /// Identifies a registered quantity operation — the §8.3 composition policy
    /// (`reference.quantity_operations`).
    OperationId,
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
mod identity_boundary_tests {
    #![allow(clippy::unwrap_used, reason = "pure schema and identity contract assertions")]
    use super::*;

    #[test]
    fn physical_identity_schema_retains_domain_and_wire_bytes() {
        let quantity = QuantityTypeId::from_bytes([0x31; 16]);
        let unit = UnitId::from_bytes([0x31; 16]);
        assert_eq!(quantity.as_bytes(), unit.as_bytes());
        assert_eq!(serde_json::to_value(quantity).unwrap(), serde_json::to_value(unit).unwrap());
        let quantity_schema = serde_json::to_value(schemars::schema_for!(QuantityTypeId)).unwrap();
        let unit_schema = serde_json::to_value(schemars::schema_for!(UnitId)).unwrap();
        assert_eq!(quantity_schema["x-pse-identity"], "QuantityTypeId");
        assert_eq!(unit_schema["x-pse-identity"], "UnitId");
        assert_ne!(quantity_schema["x-pse-identity"], unit_schema["x-pse-identity"]);
    }
}
