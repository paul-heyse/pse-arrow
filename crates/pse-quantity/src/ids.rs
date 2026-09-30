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
    EntityKindId,
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
    /// Identifies a domain: what an index ranges over (`authored.domains`, §6.3).
    DomainId,
    /// Identifies a bound index — one binding occurrence of a domain in an expression,
    /// not the domain itself (§6.9, `bound_index_id`).
    BoundIndexId,
    /// Identifies an invariant a rule or an operation depends on (`schema` invariants,
    /// §4.1).
    InvariantId,
}
