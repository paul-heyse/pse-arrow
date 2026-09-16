// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Source-backed indexed element coefficients and their explicitly declared products.
use super::{N, RegistryBuilder, S, T, column, derived, enumeration, index, relation};
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
    derived(
        builder,
        N::Compiled,
        "element_projection_groups",
        &["group_id"],
        vec![
            column("group_id", T::id()),
            column("product_id", T::id()),
            column("application_id", T::id()),
            column("contribution_id", T::id()),
            column("owner_instance_id", T::id()),
            column("law_template_id", T::id()),
            column("source_basis_id", T::id()),
            column("element_domain_id", T::id()),
            column("species_domain_id", T::id()),
            column("domain_ids", T::list(T::id())),
            column("quantity_type_id", T::id()),
        ],
        "Generated ordered Element/Species product tied to actual law and contribution axes; P10 rechecks correspondence.",
    );
    derived(
        builder,
        N::Compiled,
        "element_projection_coefficients",
        &["group_id", "index"],
        vec![
            column("group_id", T::id()).with_fk("compiled.element_projection_groups", "group_id"),
            column("index", index()),
            column("symbol_id", T::id()),
            column("node_id", T::native(arrow_schema::DataType::UInt64)),
            column("element_id", T::id()),
            column("species_id", T::id()),
            column("count", T::native(arrow_schema::DataType::Float64)),
            column(
                "molecular_weight",
                T::native(arrow_schema::DataType::Float64),
            )
            .optional(),
            column("value", T::native(arrow_schema::DataType::Float64)),
        ],
        "Immutable generated coefficient from exact actual species composition and optional kg/mol molecular weight, never an initial guess.",
    );
}
