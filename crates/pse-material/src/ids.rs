// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The identity newtypes of the material system (blueprint §5.1, §6.4).
//!
//! Declared with `pse_quantity::semantic_id_newtype!`, so these are the same shape as the
//! physical-typing identities: a [`SemanticId`] the compiler refuses to confuse with
//! another. A `PhaseId` where a `SpeciesId` belongs would otherwise produce an empty join
//! rather than an error, and §6.4's relations are almost entirely joins between these four.
//!
//! An element is `reference` and a species, phase or reaction is `authored` (§6.4), which
//! is why an element identity is a named-policy identity — `blake3_128("pse:named:v1" ‖
//! package ‖ qualified_name)` — while the other three are assigned when the entity is
//! written (§5.1).
//!
//! [`SemanticId`]: pse_ids::SemanticId

pse_quantity::semantic_id_newtype! {
    /// Identifies a chemical element (`reference.elements`, blueprint §6.4).
    ElementId,
    /// Identifies a species (`authored.species`, blueprint §6.4).
    SpeciesId,
    /// Identifies a phase (`authored.phases`, blueprint §6.4).
    PhaseId,
    /// Identifies a reaction (`authored.reactions`, blueprint §6.4).
    ReactionId,
}

#[cfg(test)]
mod tests {
    use pse_ids::SemanticId;

    use super::{ElementId, PhaseId, SpeciesId};

    #[test]
    fn a_newtype_round_trips_through_its_identity() {
        let raw = SemanticId::from_bytes([0x2a; 16]);
        assert_eq!(SpeciesId::from_id(raw).as_id(), raw);
        assert_eq!(SemanticId::from(PhaseId::from_id(raw)), raw);
    }

    #[test]
    fn display_renders_the_underlying_hexadecimal() {
        let element = ElementId::from_id(SemanticId::from_bytes([0x0e; 16]));
        assert_eq!(element.to_string(), "0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e");
    }
}
