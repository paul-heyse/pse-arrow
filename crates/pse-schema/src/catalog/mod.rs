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

mod cache;
mod declarations;
pub mod documents;
pub mod enums_platform;
pub mod inv;
pub mod invariants;
mod modeling;
mod modeling_analysis;
mod modeling_knowledge;
mod modeling_native_analysis;
mod native_math;
mod native_strategy;
mod normalization;
mod operations;
mod publication;
mod row_checks;
pub mod s14_passes;
pub mod s4_schema;
pub mod s6_11_numerical;
pub mod s6_13_runtime;
pub mod s6_14_idaes_enums;
pub mod s6_15_semantic;
pub mod s6_1_identity;
pub mod s6_2_physical;
pub mod s7_operators;
pub mod solve_settings;

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
    publication::declare_profiles(builder);
}

/// Add only the canonical diagnostic relation, its severity/failure vocabularies and the
/// run identity its findings reference. Custom registries use this before native
/// integrity execution; the complete platform catalog already includes it. Duplicate
/// declarations remain errors.
pub fn declare_diagnostics(builder: &mut RegistryBuilder) {
    operations::declare_run_identity(builder);
    enums_platform::declare_failure_classes(builder);
    s6_13_runtime::declare_diagnostics(builder);
}

/// Add the publication record, artifact, dependency and retention contracts to an
/// explicit registry, with the entity identities the publication record references.
/// They are the same declarations the complete platform catalog uses.
pub fn declare_publications(builder: &mut RegistryBuilder) {
    operations::declare_publication_identities(builder);
    publication::declare(builder);
}

/// Full relation/rule authority with only P0–P3 producers, for explicit leaf fixture registries.
/// Production registries use [`declare`], which adds the real semantic stage contracts.
pub fn declare_foundations(builder: &mut RegistryBuilder) {
    crate::validation::declare(builder);
    cache::declare(builder);
    enums_platform::declare(builder);
    s4_schema::declare(builder);
    s6_1_identity::declare(builder);
    s6_2_physical::declare(builder);
    s7_operators::declare(builder);
    s6_11_numerical::declare(builder);
    s6_13_runtime::declare(builder);
    native_math::declare(builder);
    native_strategy::declare(builder);
    solve_settings::declare(builder);
    modeling::declare(builder);
    modeling_knowledge::declare(builder);
    modeling_analysis::register(builder);
    modeling_native_analysis::register(builder);
    publication::declare(builder);
    operations::declare(builder);
    s6_14_idaes_enums::declare(builder);
    s6_15_semantic::declare(builder);
    normalization::declare(builder);
    documents::declare(builder);
    invariants::declare(builder);
    s14_passes::declare(builder);
}
