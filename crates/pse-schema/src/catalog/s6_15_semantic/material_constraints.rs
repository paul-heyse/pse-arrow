// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual selected material-domain cardinalities required by a reusable template.

use super::{N, RegistryBuilder, S, T, column, relation};

pub(super) fn declare(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "template_material_constraints",
        S::Model,
        &["template_id"],
        vec![
            column("template_id", T::id()).with_fk("authored.templates", "template_id"),
            column("min_phases", T::native(arrow_schema::DataType::UInt16)),
            column("max_phases", T::native(arrow_schema::DataType::UInt16)).optional(),
            column("min_species", T::native(arrow_schema::DataType::UInt16)),
            column("max_species", T::native(arrow_schema::DataType::UInt16)).optional(),
        ],
        "Bounds on the distinct actual phase and species members of the selected material system, evaluated before realization; caller declarations of counts are not evidence.",
    );
}
