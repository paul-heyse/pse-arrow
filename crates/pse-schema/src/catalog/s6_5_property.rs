// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! §6.5 property.

use super::declarations::{column, relation, relation_version, structure};
use crate::builder::RegistryBuilder;
use crate::model::{FieldContract as T, Namespace as N, SnapshotClass as S};

/// Declares the §6.5 property contracts.
pub fn declare(builder: &mut RegistryBuilder) {
    declare_reference_property_kinds(builder);
    declare_reference_method_specs(builder);
    declare_authored_property_packages(builder);
    declare_authored_state_bounds(builder);
    declare_authored_phase_equilibrium_pairs(builder);
    declare_authored_method_selections(builder);
    declare_authored_default_scaling(builder);
    declare_inferred_property_requirements(builder);
    declare_inferred_method_resolutions(builder);
    declare_inferred_state_flash_required(builder);
    declare_property_category_vocabulary(builder);
    declare_method_family_vocabulary(builder);
    declare_method_realization_vocabulary(builder);
    declare_scope_kind_vocabulary(builder);
    declare_resolution_status_vocabulary(builder);
}

fn declare_inferred_state_flash_required(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Inferred,
        "state_flash_required",
        S::Derived,
        &["state_instance"],
        vec![column("state_instance", T::id())],
        "blueprint §9.2: state instances whose phase-equilibrium and state-definition facts require a flash.",
    );
}

fn declare_reference_property_kinds(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Reference,
        "property_kinds",
        S::Model,
        &["property_kind_id"],
        vec![
            column("property_kind_id", T::id()),
            column("idaes_name", T::native(arrow_schema::DataType::Utf8)),
            column("quantity_kind_id", T::id()),
            column("basis_id", T::id()).optional(),
            column("shape", T::list(T::enumeration("DomainKind"))),
            column("category", T::enumeration("PropertyCategory")),
            column("doc", T::native(arrow_schema::DataType::Utf8)),
        ],
        "blueprint §6.5 property: property_kinds.",
    );
}

fn declare_reference_method_specs(builder: &mut RegistryBuilder) {
    relation_version(
        builder,
        N::Reference,
        "method_specs",
        2,
        S::Model,
        &["method_id"],
        vec![
            column("method_id", T::id()),
            column("family", T::enumeration("MethodFamily")),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("version", T::native(arrow_schema::DataType::Utf8)),
            column("provides", T::list(T::id())),
            column("requires", T::list(T::id())),
            column(
                "parameter_kinds",
                T::list(structure(vec![
                    ("name", T::native(arrow_schema::DataType::Utf8)),
                    ("quantity_kind_id", T::id()),
                    ("indexed_by", T::list(T::enumeration("DomainKind"))),
                    ("required", T::native(arrow_schema::DataType::Boolean)),
                ])),
            ),
            column("realization", method_realization()),
            column(
                "validity",
                T::list(structure(vec![
                    ("input", T::native(arrow_schema::DataType::Utf8)),
                    ("lower", T::extended(crate::model::ExtensionUse::Bound)),
                    ("upper", T::extended(crate::model::ExtensionUse::Bound)),
                ])),
            ),
            column("doc", T::native(arrow_schema::DataType::Utf8)),
        ],
        "blueprint §6.5 property: method_specs.",
    );
}

fn declare_authored_property_packages(builder: &mut RegistryBuilder) {
    relation_version(
        builder,
        N::Authored,
        "property_packages",
        2,
        S::Model,
        &["property_package_id"],
        vec![
            column("property_package_id", T::id()),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("package_id", T::id()).with_fk("authored.packages", "package_id"),
            column("material_system_id", T::id()),
            column("unit_set_id", T::id()),
            column("state_definition_method_id", T::id()),
            column(
                "temperature_ref",
                T::native(arrow_schema::DataType::Float64),
            ),
            column("pressure_ref", T::native(arrow_schema::DataType::Float64)),
            column(
                "include_enthalpy_of_formation",
                T::native(arrow_schema::DataType::Boolean),
            ),
            column("bubble_dew_method_id", T::id()).optional(),
            column("doc", T::native(arrow_schema::DataType::Utf8)),
        ],
        "blueprint §6.5 property: property_packages.",
    );
}

fn declare_authored_state_bounds(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "state_bounds",
        S::Model,
        &["property_package_id", "state_symbol"],
        vec![
            column("property_package_id", T::id()),
            column("state_symbol", T::native(arrow_schema::DataType::Utf8)),
            column("lower", T::extended(crate::model::ExtensionUse::Bound)),
            column("initial", T::native(arrow_schema::DataType::Float64)),
            column("upper", T::extended(crate::model::ExtensionUse::Bound)),
            column("unit_id", T::id()),
        ],
        "blueprint §6.5 property: state_bounds.",
    );
}

fn declare_authored_phase_equilibrium_pairs(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "phase_equilibrium_pairs",
        S::Model,
        &["property_package_id", "phase_a_id", "phase_b_id"],
        vec![
            column("property_package_id", T::id()),
            column("phase_a_id", T::id()),
            column("phase_b_id", T::id()),
            column("state_method_id", T::id()),
            column("form_method_id", T::id()),
        ],
        "blueprint §6.5 property: phase_equilibrium_pairs.",
    );
}

fn declare_authored_method_selections(builder: &mut RegistryBuilder) {
    relation_version(
        builder,
        N::Authored,
        "method_selections",
        2,
        S::Model,
        &["selection_id"],
        vec![
            column("selection_id", T::id()),
            column("is_default", T::native(arrow_schema::DataType::Boolean)),
            column("property_package_id", T::id()),
            column("scope_kind", T::enumeration("ScopeKind")),
            column(
                "scope_ids",
                T::extended(crate::model::ExtensionUse::IndexTuple),
            ),
            column("property_kind_id", T::id()).optional(),
            column("family", T::enumeration("MethodFamily")),
            column("method_id", T::id()),
            column(
                "options",
                T::list(structure(vec![
                    ("key", T::native(arrow_schema::DataType::Utf8)),
                    ("value", T::native(arrow_schema::DataType::Utf8)),
                ])),
            ),
        ],
        "blueprint §6.5 property: method_selections.",
    );
}

fn declare_authored_default_scaling(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "default_scaling",
        S::Model,
        &["property_package_id", "property_kind_id", "index"],
        vec![
            column("property_package_id", T::id()),
            column("property_kind_id", T::id()),
            column("index", T::extended(crate::model::ExtensionUse::IndexTuple)),
            column("scaling_factor", T::native(arrow_schema::DataType::Float64)),
        ],
        "blueprint §6.5 property: default_scaling.",
    );
}

fn declare_inferred_property_requirements(builder: &mut RegistryBuilder) {
    relation_version(
        builder,
        N::Inferred,
        "property_requirements",
        2,
        S::Derived,
        &["requirement_id"],
        vec![
            column("requirement_id", T::id()),
            column("state_scope_id", T::id()),
            column("property_kind_id", T::id()),
            column("index", T::extended(crate::model::ExtensionUse::IndexTuple)),
            crate::model::FieldContract::provenance(
                "derivation_id",
                T::id(),
                "Exact source derivation.",
            ),
        ],
        "blueprint §6.5 property: property_requirements.",
    );
}

fn declare_inferred_method_resolutions(builder: &mut RegistryBuilder) {
    relation_version(
        builder,
        N::Inferred,
        "method_resolutions",
        3,
        S::Derived,
        &["requirement_id"],
        vec![
            column("requirement_id", T::id()),
            column("outcome", resolution_outcome()),
            crate::model::FieldContract::provenance(
                "derivation_id",
                T::id(),
                "Exact source derivation.",
            ),
        ],
        "blueprint §6.5 property: method_resolutions.",
    );
}

/// A resolution names one authoritative method or a complete unresolved outcome.
pub(super) fn resolution_outcome() -> T {
    T::structure(vec![
        T::enumeration("ResolutionStatus").with_name("kind"),
        T::structure(vec![
            T::id()
                .with_name("method_id")
                .with_fk("reference.method_specs", "method_id"),
        ])
        .with_name("resolved")
        .optional(),
    ])
    .with_alternative(
        &crate::model::TaggedAlternative::new("kind", [("resolved".into(), "resolved".into())])
            .with_unit("unresolved")
            .with_unit("ambiguous"),
    )
}

fn declare_property_category_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "PropertyCategory",
        ["state", "thermo", "transport", "reaction", "derived"],
    );
}

fn declare_method_family_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "MethodFamily",
        [
            "state_definition",
            "eos",
            "pure_component",
            "phase_equilibrium_form",
            "phase_equilibrium_state",
            "bubble_dew",
            "henry",
            "transport_mixing",
            "reaction_rate_form",
            "rate_constant",
            "equilibrium_form",
            "equilibrium_constant",
            "heat_of_reaction",
            "enthalpy_transport",
            "custom",
        ],
    );
}

fn declare_method_realization_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "MethodRealization",
        ["equation_template", "kernel"],
    );
}

fn declare_scope_kind_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "ScopeKind",
        ["package", "phase", "species", "phase_species", "reaction"],
    );
}

fn declare_resolution_status_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "ResolutionStatus",
        ["resolved", "unresolved", "ambiguous"],
    );
}

/// The selected producer carries exactly the reference required by its route.
fn method_realization() -> T {
    T::structure(vec![
        T::enumeration("MethodRealization").with_name("kind"),
        T::structure(vec![T::id().with_name("template_id")])
            .with_name("equation_template")
            .optional(),
        T::structure(vec![T::id().with_name("kernel_id")])
            .with_name("kernel")
            .optional(),
    ])
    .with_alternative(&crate::model::TaggedAlternative::new(
        "kind",
        [
            ("equation_template".into(), "equation_template".into()),
            ("kernel".into(), "kernel".into()),
        ],
    ))
}
