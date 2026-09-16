// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The catalog: every declaration the platform ships, in one fixed order.
//!
//! One module per blueprint section, and [`assemble`] calls them in a list that is itself
//! the contract. A module may name a relation, enumeration or rule another module
//! declares, because nothing is resolved until [`crate::builder::RegistryBuilder::build`]
//! has them all — so the order below is about readability and about being able to see, in
//! one screen, everything the platform declares.
//!
//! Section modules declare the production registry. Cross-section references resolve
//! together during registry construction.

mod declarations;
pub mod documents;
pub mod enums_platform;
pub mod expr_family;
pub mod inv;
mod invariant_closure;
mod invariant_domain;
mod invariant_semantic;
pub mod invariants;
pub mod manifest;
mod normalization;
mod publication;
pub mod s14_passes;
pub mod s14_semantic_passes;
pub mod s22_change_sets;
pub mod s4_schema;
pub mod s5_2_revisions;
pub mod s6_10_cases;
pub mod s6_11_numerical;
pub mod s6_12_derived;
pub mod s6_13_runtime;
pub mod s6_14_idaes_enums;
pub mod s6_15_semantic;
pub mod s6_1_identity;
pub mod s6_2_physical;
pub mod s6_3_domains;
pub mod s6_4_material;
pub mod s6_5_property;
pub mod s6_6_templates;
pub mod s6_7_instances;
pub mod s6_8_symbols;
pub mod s6_9_math;
pub mod s7_operators;
pub mod semantic_demand;
pub mod semantic_inference;
pub mod semantic_laws;
mod terminal_attempts;

use crate::builder::{Registry, RegistryBuilder};
use crate::error::SchemaError;

/// Declares everything and builds the registry.
///
/// # Errors
///
/// The errors of [`crate::builder::RegistryBuilder::build`]: a duplicate declaration, a
/// dangling reference, a derived relation without a granularity, a broken stage graph or a
/// rule that keys on a float.
pub fn assemble() -> Result<Registry, SchemaError> {
    let mut builder = RegistryBuilder::new();

    declare(&mut builder);
    builder.build()
}

/// Add the complete platform declarations to a builder before explicit fixture extensions.
pub fn declare(builder: &mut RegistryBuilder) {
    declare_foundations(builder);
    s14_semantic_passes::declare(builder);
}

/// Add only the canonical diagnostic relation and its severity/failure vocabularies.
/// Custom registries use this before native integrity execution; the complete
/// platform catalog already includes it. Duplicate declarations remain errors.
pub fn declare_diagnostics(builder: &mut RegistryBuilder) {
    enums_platform::declare_failure_classes(builder);
    s6_13_runtime::declare_diagnostics(builder);
}

/// Full relation/rule authority with only P0–P3 producers, for explicit leaf fixture registries.
/// Production registries use [`declare`], which adds the real semantic stage contracts.
pub fn declare_foundations(builder: &mut RegistryBuilder) {
    enums_platform::declare(builder);
    s4_schema::declare(builder);
    s6_1_identity::declare(builder);
    s5_2_revisions::declare(builder);
    s6_2_physical::declare(builder);
    s6_3_domains::declare(builder);
    s6_4_material::declare(builder);
    s6_5_property::declare(builder);
    s6_6_templates::declare(builder);
    s6_7_instances::declare(builder);
    s6_8_symbols::declare(builder);
    s6_9_math::declare(builder);
    s7_operators::declare(builder);
    s6_10_cases::declare(builder);
    s6_11_numerical::declare(builder);
    s6_12_derived::declare(builder);
    s6_13_runtime::declare(builder);
    publication::declare(builder);
    s6_14_idaes_enums::declare(builder);
    s6_15_semantic::declare(builder);
    s22_change_sets::declare(builder);
    normalization::declare(builder);
    expr_family::declare(builder);
    semantic_inference::declare(builder);
    semantic_demand::declare(builder);
    semantic_laws::declare(builder);
    documents::declare(builder);
    invariants::declare(builder);
    s14_passes::declare(builder);
    manifest::declare(builder);
}
