// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Finite prospective instance and typed value declarations.
use super::{
    N, RegistryBuilder, S, T, column, config_value_type, derived, enumeration, index, relation,
};

#[expect(
    clippy::too_many_lines,
    reason = "Keep the declarative relation and native rule family together for schema review"
)]
pub(super) fn declare(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Reference,
        "quantity_operation_reductions",
        S::Model,
        &["operation_id"],
        vec![
            column("operation_id", T::id())
                .with_fk("reference.quantity_operations", "operation_id"),
            column("domain_kind", T::enumeration("DomainKind")),
        ],
        "Exact structural dispatch domain for a registered reduction; physical precondition failure never permits a fallback.",
    );
    enumeration(
        builder,
        "QuantityPreconditionKind",
        ["equal_operand_bases", "operand_quantity_contract"],
    );
    relation(
        builder,
        N::Reference,
        "quantity_preconditions",
        S::Model,
        &["invariant_id"],
        vec![
            column("invariant_id", T::id()),
            column("kind", T::enumeration("QuantityPreconditionKind")),
            column(
                "operand_positions",
                T::list(T::native(arrow_schema::DataType::UInt16)),
            ),
            column("required_basis_id", T::id())
                .optional()
                .with_fk("reference.bases", "basis_id"),
            column("required_quantity_type_id", T::id())
                .optional()
                .with_fk("reference.quantity_types", "quantity_type_id"),
            column("match_shape", T::native(arrow_schema::DataType::Boolean)).optional(),
        ],
        "Operation preconditions proved from exact operand contracts at application time, never by invariant identity alone.",
    );
    derived(
        builder,
        N::Normalized,
        "material_domain_members",
        &["domain_id", "member_id"],
        vec![
            column("domain_id", T::id()),
            column("member_id", T::id()),
            column("material_system_id", T::id())
                .with_fk("authored.material_systems", "material_system_id"),
            column("phase_id", T::id())
                .optional()
                .with_fk("authored.phases", "phase_id"),
            column("species_id", T::id())
                .optional()
                .with_fk("authored.species", "species_id"),
            column("element_id", T::id())
                .optional()
                .with_fk("reference.elements", "element_id"),
        ],
        "Finite candidate member to actual material subject correspondence; P4/P5 apply declared restrictions.",
    );
    relation(
        builder,
        N::Authored,
        "template_scopes",
        S::Model,
        &["template_id", "name"],
        vec![
            column("template_id", T::id()).with_fk("authored.templates", "template_id"),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("scope_id", T::id()).with_fk("authored.scopes", "scope_id"),
        ],
        "Reusable scope declaration bound once per actual template instance.",
    );
    derived(
        builder,
        N::Inferred,
        "scope_bindings",
        &["scope_decl_id", "owner_instance_id"],
        vec![
            column("scope_decl_id", T::id()).with_fk("authored.scopes", "scope_id"),
            column("owner_instance_id", T::id()),
            column("scope_id", T::id()),
        ],
        "Actual relative scope identity and source-owner correspondence.",
    );
    derived(
        builder,
        N::Normalized,
        "property_path_demands",
        &["demand_id"],
        vec![
            column("demand_id", T::id()),
            column("source_id", T::id()),
            column("path_id", T::native(arrow_schema::DataType::UInt64)),
            column(
                "guard_predicate_id",
                T::native(arrow_schema::DataType::UInt64),
            )
            .optional(),
            column("read_node_id", T::native(arrow_schema::DataType::UInt64)),
        ],
        "Exact read-node occurrence obligation resolved against actual path targets before P6 demand closure; a guard retains its source.",
    );
    enumeration(
        builder,
        "ConfigValueKind",
        [
            "boolean",
            "signed",
            "unsigned",
            "real",
            "text",
            "semantic_id",
            "enum",
            "index",
            "quantity",
        ],
    );
    enumeration(
        builder,
        "ConfigCategory",
        ["parameter", "feature", "method_option", "law_option"],
    );
    enumeration(
        builder,
        "DomainBindingSource",
        [
            "domain",
            "parameter",
            "species",
            "phase",
            "phase_species",
            "element",
        ],
    );
    enumeration(
        builder,
        "ExpressionOwnerKind",
        crate::model::ExpressionOwnerKind::ALL.map(crate::model::ExpressionOwnerKind::as_str),
    );
    relation(
        builder,
        N::Authored,
        "template_domain_bindings",
        S::Model,
        &["template_id", "name"],
        vec![
            column("template_id", T::id()).with_fk("authored.templates", "template_id"),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("source", T::enumeration("DomainBindingSource")),
            column("domain_id", T::id())
                .optional()
                .with_fk("authored.domains", "domain_id"),
            column("parameter_name", T::native(arrow_schema::DataType::Utf8)).optional(),
        ],
        "Finite domain source, with only its tagged payload present (blueprint §6.15.1).",
    );
    derived(
        builder,
        N::Normalized,
        "config_values",
        &["owner_id", "category", "name"],
        vec![
            column("owner_id", T::id()),
            column("category", T::enumeration("ConfigCategory")),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("value", config_value_type()),
            column("source_relation_id", T::id())
                .with_fk("reference.schema_relations", "relation_id"),
            column("source_key", T::row_key()),
        ],
        "Typed admitted assignment and typed source key, including declaration defaults.",
    );
    derived(
        builder,
        N::Normalized,
        "feature_inheritance",
        &["instance_id", "name"],
        vec![
            column("instance_id", T::id()),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("source_instance_id", T::id()).optional(),
            column("source_name", T::native(arrow_schema::DataType::Utf8)),
            column("source_relation_id", T::id()),
            column("source_key", T::row_key()),
        ],
        "Explicit same-instance or parent feature reference; an absent parent remains unknown.",
    );
    derived(
        builder,
        N::Normalized,
        "instance_bindings",
        &["instance_id"],
        vec![
            column("instance_id", T::id()),
            column("parent_instance_id", T::id()).optional(),
            column("template_id", T::id()).with_fk("authored.templates", "template_id"),
            column("submodel_template_id", T::id()).optional(),
            column("submodel_name", T::native(arrow_schema::DataType::Utf8)).optional(),
            column("index", index()),
            column("path", T::native(arrow_schema::DataType::Utf8)),
            column("property_package_id", T::id()).optional(),
            column("reaction_package_id", T::id()).optional(),
            column("guard_source_id", T::id()).optional(),
            column("guard_node_id", T::native(arrow_schema::DataType::UInt64)).optional(),
        ],
        "Complete finite prospective roots and children; guards are decided by P4.",
    );
    relation(
        builder,
        N::Reference,
        "math_context",
        S::Model,
        &["package_id"],
        vec![
            column("package_id", T::id()).with_fk("authored.packages", "package_id"),
            column("neutral_quantity_type_id", T::id())
                .with_fk("reference.quantity_types", "quantity_type_id"),
            column("boolean_kind_id", T::id())
                .with_fk("reference.quantity_kinds", "quantity_kind_id"),
        ],
        "Explicit physical context for canonicalization; no dimensional guessing.",
    );
    declare_instance_equations(builder);
}

fn declare_instance_equations(builder: &mut RegistryBuilder) {
    // One equation shape; only the ownership reference changes for this authoring surface.
    let Some(mut declaration) = builder
        .declared_relations()
        .iter()
        .find(|relation| relation.key.qualified_name() == "authored.template_equations")
        .cloned()
    else {
        return;
    };
    declaration.key.name = "instance_equations";
    declaration.doc = "Instance-owned equation declaration (blueprint §6.15.4).";
    for field in &mut declaration.columns {
        if field.name() == "template_id" {
            *field = column("instance_id", T::id()).with_fk("authored.instances", "instance_id");
        }
    }
    builder.declare_relation(declaration);
}
