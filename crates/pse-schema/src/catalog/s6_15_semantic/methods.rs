// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Method selection, dependency and provision contracts are separate from realization.
use super::{N, RegistryBuilder, S, T, column, derived, enumeration, relation};

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
            column("position", T::native(arrow_schema::DataType::UInt16)),
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
            column("rank", T::native(arrow_schema::DataType::UInt16)),
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
            column("ordinal", T::native(arrow_schema::DataType::UInt16)),
            column("target_kind", T::enumeration("MethodDependencyTarget")),
            column("target_id", T::id()),
            column("scope_map", T::enumeration("MethodScopeMap")),
            column(
                "index_map",
                T::list(T::structure(vec![
                    T::enumeration("IndexMapKind")
                        .with_name("kind")
                        .with_nullable(false),
                    T::native(arrow_schema::DataType::UInt16)
                        .with_name("source_axis")
                        .with_nullable(true),
                    T::id().with_name("member_id").with_nullable(true),
                    T::enumeration("DomainKind")
                        .with_name("domain_kind")
                        .with_nullable(false),
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
            column("output_kind", T::enumeration("MethodOutputKind")),
            column("symbol_decl_id", T::id()).optional(),
            column(
                "kernel_output_ordinal",
                T::native(arrow_schema::DataType::UInt16),
            )
            .optional(),
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
    derived(
        builder,
        N::Inferred,
        "requirement_support",
        &["requirement_id", "requester_id"],
        vec![
            column("requirement_id", T::id()),
            column("requester_id", T::id()),
            column("source_kind", T::enumeration("RequirementSource")),
        ],
        "Finite many-to-many direct demand support, including recursive edges.",
    );
    derived(
        builder,
        N::Inferred,
        "method_candidates",
        &["requirement_id", "selection_id", "method_id"],
        vec![
            column("requirement_id", T::id()),
            column("selection_id", T::id()),
            column("method_id", T::id()),
            column("applicable", T::native(arrow_schema::DataType::Boolean)),
            column("rank", T::native(arrow_schema::DataType::UInt16)).optional(),
            column("reason", T::enumeration("MethodCandidateReason")),
        ],
        "Complete candidate classification; absence requires the entire declared inventory.",
    );
    derived(
        builder,
        N::Compiled,
        "method_realizations",
        &["requirement_id"],
        vec![
            column("requirement_id", T::id()),
            column("method_id", T::id()),
            column("template_instance_id", T::id()).optional(),
            column("kernel_binding_id", T::id()).optional(),
            column("output_kind", T::enumeration("MethodOutputKind")),
            column("output_symbol_id", T::id()).optional(),
            column(
                "kernel_output_ordinal",
                T::native(arrow_schema::DataType::UInt16),
            )
            .optional(),
        ],
        "P9 provision witness, distinct from P6 candidate selection.",
    );
}
