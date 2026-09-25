// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Finite prospective instance and typed value declarations.
use super::{N, RegistryBuilder, S, T, column, derived, enumeration, index, relation};

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
                T::list(T::nonnegative(i64::from(u16::MAX))),
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
            column("source", domain_source()),
        ],
        "Finite domain source, with only its tagged payload present (blueprint §6.15.1).",
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
            column("guard_node_id", T::nonnegative(i64::MAX)).optional(),
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

fn domain_source() -> T {
    let alternative = crate::model::TaggedAlternative::new(
        "kind",
        [
            ("domain".into(), "domain".into()),
            ("parameter".into(), "parameter".into()),
        ],
    )
    .with_unit("species")
    .with_unit("phase")
    .with_unit("phase_species")
    .with_unit("element");
    T::structure(vec![
        T::enumeration("DomainBindingSource").with_name("kind"),
        T::structure(vec![
            T::id()
                .with_fk("authored.domains", "domain_id")
                .with_name("domain_id"),
        ])
        .with_name("domain")
        .optional(),
        T::structure(vec![
            T::native(arrow_schema::DataType::Utf8).with_name("name"),
        ])
        .with_name("parameter")
        .optional(),
    ])
    .with_alternative(&alternative)
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
