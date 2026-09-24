// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Source-backed indexed element coefficients and their explicitly declared products.
use super::{N, RegistryBuilder, S, T, column, enumeration, relation};
pub(super) fn declare(builder: &mut RegistryBuilder) {
    enumeration(
        builder,
        "ElementProjectionFormula",
        ["molar_count", "mass_count_over_mw"],
    );
    relation(
        builder,
        N::Reference,
        "element_projection_contracts",
        S::Model,
        &["law_template_id", "source_basis_id"],
        vec![
            column("law_template_id", T::id()).with_fk("authored.templates", "template_id"),
            column("source_basis_id", T::id()).with_fk("reference.bases", "basis_id"),
            column("formula", T::enumeration("ElementProjectionFormula")),
            column("source_quantity_type_id", T::id())
                .with_fk("reference.quantity_types", "quantity_type_id"),
            column("coefficient_quantity_type_id", T::id())
                .with_fk("reference.quantity_types", "quantity_type_id"),
            column("coefficient_unit_id", T::id()).with_fk("reference.units", "unit_id"),
            column("multiplication_operation_id", T::id())
                .with_fk("reference.quantity_operations", "operation_id"),
        ],
        "Closed physical projection from actual species count/MW to one declared Element balance quantity.",
    );


}
