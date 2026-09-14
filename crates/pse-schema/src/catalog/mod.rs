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
//! Modules that packets A-1 to A-6 fill are present and empty. They are wired in now
//! rather than added later because the list is what a reviewer checks, and a module that
//! appears halfway through the wave is a change nobody diffed.

pub mod documents;
pub mod enums_platform;
pub mod expr_family;
pub mod inv;
pub mod invariants;
pub mod manifest;
pub mod s14_passes;
pub mod s22_change_sets;
pub mod s4_schema;
pub mod s5_2_revisions;
pub mod s6_10_cases;
pub mod s6_11_numerical;
pub mod s6_12_derived;
pub mod s6_13_runtime;
pub mod s6_14_idaes_enums;
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

    enums_platform::declare(&mut builder);
    s4_schema::declare(&mut builder);
    s6_1_identity::declare(&mut builder);
    s5_2_revisions::declare(&mut builder);
    s6_2_physical::declare(&mut builder);
    s6_3_domains::declare(&mut builder);
    s6_4_material::declare(&mut builder);
    s6_5_property::declare(&mut builder);
    s6_6_templates::declare(&mut builder);
    s6_7_instances::declare(&mut builder);
    s6_8_symbols::declare(&mut builder);
    s6_9_math::declare(&mut builder);
    s7_operators::declare(&mut builder);
    expr_family::declare(&mut builder);
    s6_10_cases::declare(&mut builder);
    s6_11_numerical::declare(&mut builder);
    s6_12_derived::declare(&mut builder);
    s6_13_runtime::declare(&mut builder);
    s6_14_idaes_enums::declare(&mut builder);
    s14_passes::declare(&mut builder);
    s22_change_sets::declare(&mut builder);
    invariants::declare(&mut builder);
    documents::declare(&mut builder);
    manifest::declare(&mut builder);

    builder.build()
}
