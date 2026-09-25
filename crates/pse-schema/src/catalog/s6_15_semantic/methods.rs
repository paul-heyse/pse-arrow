// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Method selection, dependency and provision contracts are separate from realization.
use super::{N, RegistryBuilder, S, T, column, enumeration, relation};

#[expect(
    clippy::too_many_lines,
    reason = "Keep the declarative relation and native rule family together for schema review"
)]
pub(super) fn declare(builder: &mut RegistryBuilder) {
    enumeration(
        builder,
        "ParameterSourceCoordinate",
        ["member", "ref_entity", "phase_species_pair"],
    );
    relation(
        builder,
        N::Reference,
        "method_parameter_axes",
        S::Model,
        &["method_id", "parameter_kind", "position"],
        vec![
            column("method_id", T::id()).with_fk("reference.method_specs", "method_id"),
            column("parameter_kind", T::native(arrow_schema::DataType::Utf8)),
            column("position", T::nonnegative(i64::from(u16::MAX))),
            column(
                "source_coordinate",
                T::enumeration("ParameterSourceCoordinate"),
            ),
        ],
        "Complete ordered local parameter axis to actual source-data coordinate projection.",
    );
    relation(
        builder,
        N::Reference,
        "method_state_parameters",
        S::Model,
        &["method_id"],
        vec![
            column("method_id", T::id()).with_fk("reference.method_specs", "method_id"),
            column("parameter_name", T::native(arrow_schema::DataType::Utf8)),
        ],
        "Exact required SemanticId template parameter receiving the requesting actual state instance.",
    );
    enumeration(
        builder,
        "MethodDependencyTarget",
        ["property", "state_symbol"],
    );
    enumeration(builder, "MethodScopeMap", ["same_state"]);
    enumeration(
        builder,
        "IndexMapKind",
        ["source_axis", "fixed_member", "bound_domain"],
    );
    enumeration(
        builder,
        "MethodOutputKind",
        ["template_symbol", "kernel_output"],
    );
    enumeration(builder, "RequirementSource", ["seed", "requirement"]);
    enumeration(
        builder,
        "MethodCandidateReason",
        [
            "applicable",
            "scope_mismatch",
            "family_mismatch",
            "missing_provision",
            "incompatible_signature",
            "missing_parameter",
            "shadowed",
        ],
    );
    relation(
        builder,
        N::Reference,
        "method_precedence",
        S::Model,
        &["is_default", "property_specific", "scope_kind"],
        vec![
            column("is_default", T::native(arrow_schema::DataType::Boolean)),
            column(
                "property_specific",
                T::native(arrow_schema::DataType::Boolean),
            ),
            column("scope_kind", T::enumeration("ScopeKind")),
            column("rank", T::nonnegative(i64::from(u16::MAX))),
        ],
        "Complete registry-versioned method preference; equal distinct winners are ambiguous.",
    );
    relation(
        builder,
        N::Reference,
        "method_dependencies",
        S::Model,
        &["method_id", "ordinal"],
        vec![
            column("method_id", T::id()).with_fk("reference.method_specs", "method_id"),
            column("ordinal", T::nonnegative(i64::from(u16::MAX))),
            column("target_kind", T::enumeration("MethodDependencyTarget")),
            column("target_id", T::id()),
            column("scope_map", T::enumeration("MethodScopeMap")),
            column(
                "index_map",
                T::list(T::structure(vec![
                    T::enumeration("DomainKind").with_name("domain_kind"),
                    index_coordinate().with_name("source"),
                ])),
            ),
        ],
        "Each target axis binds a source axis, fixed member or finite domain expansion.",
    );
    relation(
        builder,
        N::Reference,
        "method_provisions",
        S::Model,
        &["method_id", "property_kind_id"],
        vec![
            column("method_id", T::id()).with_fk("reference.method_specs", "method_id"),
            column("property_kind_id", T::id())
                .with_fk("reference.property_kinds", "property_kind_id"),
            column("output", method_output()),
            column("quantity_type_id", T::id())
                .with_fk("reference.quantity_types", "quantity_type_id"),
            column("natural_unit_id", T::id()).with_fk("reference.units", "unit_id"),
            column("indexed_by", T::list(T::enumeration("DomainKind"))),
        ],
        "Complete advertised output signature and realization-specific correspondence.",
    );
    relation(
        builder,
        N::Reference,
        "method_parameters",
        S::Model,
        &["method_id", "name"],
        vec![
            column("method_id", T::id()).with_fk("reference.method_specs", "method_id"),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("quantity_type_id", T::id())
                .with_fk("reference.quantity_types", "quantity_type_id"),
            column("natural_unit_id", T::id()).with_fk("reference.units", "unit_id"),
            column("indexed_by", T::list(T::enumeration("DomainKind"))),
            column("required", T::native(arrow_schema::DataType::Boolean)),
        ],
        "Complete parameter physical type and natural coordinate.",
    );
}

fn index_coordinate() -> T {
    T::structure(vec![
        T::enumeration("IndexMapKind").with_name("kind"),
        T::structure(vec![
            T::nonnegative(i64::from(u16::MAX)).with_name("position"),
        ])
        .with_name("source_axis")
        .optional(),
        T::structure(vec![T::id().with_name("member_id")])
            .with_name("fixed_member")
            .optional(),
    ])
    .with_alternative(
        &crate::model::TaggedAlternative::new(
            "kind",
            [
                ("source_axis".into(), "source_axis".into()),
                ("fixed_member".into(), "fixed_member".into()),
            ],
        )
        .with_unit("bound_domain"),
    )
}

/// A provision names a template symbol or a kernel output position, never both.
fn method_output() -> T {
    T::structure(vec![
        T::enumeration("MethodOutputKind").with_name("kind"),
        T::structure(vec![T::id().with_name("symbol_decl_id")])
            .with_name("template_symbol")
            .optional(),
        T::structure(vec![
            T::nonnegative(i64::from(u16::MAX)).with_name("ordinal"),
        ])
        .with_name("kernel_output")
        .optional(),
    ])
    .with_alternative(&crate::model::TaggedAlternative::new(
        "kind",
        [
            ("template_symbol".into(), "template_symbol".into()),
            ("kernel_output".into(), "kernel_output".into()),
        ],
    ))
}
