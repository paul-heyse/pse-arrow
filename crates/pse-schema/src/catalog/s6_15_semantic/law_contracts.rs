// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Declared physical subjects and conservation bindings, independent of display names.
use super::{N, RegistryBuilder, S, T, column, enumeration, relation};

pub(super) fn declare(builder: &mut RegistryBuilder) {
    super::super::declarations::relation_version(
        builder,
        N::Authored,
        "template_contribution_contracts",
        2,
        S::Model,
        &["contribution_decl_id"],
        vec![
            column("contribution_decl_id", T::id())
                .with_fk("authored.template_contributions", "contribution_decl_id"),
            column(
                "indexed_by",
                T::list(T::native(arrow_schema::DataType::Utf8)),
            ),
            column("quantity_type_id", T::id())
                .with_fk("reference.quantity_types", "quantity_type_id"),
            column("subject_kind", T::enumeration("ContributionSubjectKind")),
            column("subject_axis", T::native(arrow_schema::DataType::UInt16)).optional(),
            column("phase_axis", T::native(arrow_schema::DataType::UInt16)).optional(),
            column("subject_id", T::id()).optional(),
            column("phase_id", T::id()).optional(),
            column(
                "transfer_port_name",
                T::native(arrow_schema::DataType::Utf8),
            )
            .optional(),
            column(
                "transfer_member_ordinal",
                T::native(arrow_schema::DataType::UInt16),
            )
            .optional(),
        ],
        "Exact contribution axes and physical subject claim; a subject is fixed or a declared axis, never inferred from its name.",
    );
    relation(
        builder,
        N::Authored,
        "template_law_contracts",
        S::Model,
        &["law_instance_decl_id"],
        vec![
            column("law_instance_decl_id", T::id())
                .with_fk("authored.template_law_instances", "law_instance_decl_id"),
            column(
                "indexed_by",
                T::list(T::native(arrow_schema::DataType::Utf8)),
            ),
            column("quantity_type_id", T::id())
                .with_fk("reference.quantity_types", "quantity_type_id"),
            column("balance_enum_id", T::id()).with_fk("reference.schema_enums", "enum_id"),
            column(
                "default_state_child",
                T::native(arrow_schema::DataType::Utf8),
            )
            .optional(),
            column(
                "default_feature_name",
                T::native(arrow_schema::DataType::Utf8),
            )
            .optional(),
            column("subject_axis", T::native(arrow_schema::DataType::UInt16)).optional(),
            column("phase_axis", T::native(arrow_schema::DataType::UInt16)).optional(),
            column("subject_id", T::id()).optional(),
            column("phase_id", T::id()).optional(),
        ],
        "Actual law index and required physical result; typed balance choice selects one declared law binding.",
    );
    declare_bindings(builder);
}

fn declare_bindings(builder: &mut RegistryBuilder) {
    enumeration(
        builder,
        "LawSubjectProjection",
        ["identity", "species_to_element"],
    );
    enumeration(
        builder,
        "LawExpansion",
        ["conservation", "isothermal", "pressure_total"],
    );
    relation(
        builder,
        N::Reference,
        "law_bindings",
        S::Model,
        &["law_template_id", "balance_enum_id", "balance_member"],
        vec![
            column("law_template_id", T::id()).with_fk("authored.templates", "template_id"),
            column("balance_enum_id", T::id()).with_fk("reference.schema_enums", "enum_id"),
            column("balance_member", T::native(arrow_schema::DataType::Utf8)),
            column("family", T::enumeration("LawFamily")),
            column("source_family", T::enumeration("LawFamily")),
            column("subject_projection", T::enumeration("LawSubjectProjection")),
            column("expansion", T::enumeration("LawExpansion")),
            column("subject_kind", T::enumeration("ContributionSubjectKind")),
        ],
        "Explicit supported balance dictionary member and mathematical expansion; an absent binding is unsupported.",
    );
    relation(
        builder,
        N::Authored,
        "template_display_indices",
        S::Model,
        &["template_id", "kind", "label"],
        vec![
            column("template_id", T::id()).with_fk("authored.templates", "template_id"),
            column("kind", T::enumeration("DisplayKind")),
            column("label", T::native(arrow_schema::DataType::Utf8)),
            column(
                "indexed_by",
                T::list(T::native(arrow_schema::DataType::Utf8)),
            ),
        ],
        "Declared outer axes for an indexed display expression; scalar expressions omit this companion.",
    );
}
