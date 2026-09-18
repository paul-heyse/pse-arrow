// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! §6.6 template.

use super::declarations::{column, relation, relation_version, structure};
use crate::builder::RegistryBuilder;
use crate::model::{FieldContract as T, Namespace as N, SnapshotClass as S};

/// Declares the §6.6 template contracts.
pub fn declare(builder: &mut RegistryBuilder) {
    declare_authored_templates(builder);
    declare_authored_template_params(builder);
    declare_authored_template_features(builder);
    declare_authored_template_feature_rules(builder);
    declare_authored_template_guards(builder);
    declare_authored_template_domains(builder);
    declare_authored_template_symbols(builder);
    declare_authored_template_equations(builder);
    declare_authored_template_submodels(builder);
    declare_authored_template_ports(builder);
    declare_authored_template_contributions(builder);
    declare_authored_template_law_instances(builder);
    declare_authored_template_requirements(builder);
    declare_authored_template_display(builder);
    declare_authored_template_property_requirements(builder);
    declare_normalized_property_demand_seeds(builder);
    declare_template_kind_vocabulary(builder);
    declare_feature_kind_vocabulary(builder);
    declare_feature_rule_kind_vocabulary(builder);
    declare_symbol_role_vocabulary(builder);
    declare_port_kind_vocabulary(builder);
    declare_direction_vocabulary(builder);
    declare_law_family_vocabulary(builder);
    declare_orientation_vocabulary(builder);
    declare_capability_requirement_vocabulary(builder);
    declare_display_kind_vocabulary(builder);
    declare_demand_source_vocabulary(builder);
}

fn declare_authored_templates(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "templates",
        S::Model,
        &["template_id"],
        vec![
            column("template_id", T::id()),
            column("package_id", T::id()).with_fk("authored.packages", "package_id"),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("version", T::native(arrow_schema::DataType::Utf8)),
            column("kind", T::enumeration("TemplateKind")),
            column("default_initializer_template_id", T::id()).optional(),
            column("default_scaler_template_id", T::id()).optional(),
            column("idaes_class", T::native(arrow_schema::DataType::Utf8)).optional(),
            column("doc", T::native(arrow_schema::DataType::Utf8)),
        ],
        "blueprint §6.6 template: templates.",
    );
}

fn declare_authored_template_params(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "template_params",
        S::Model,
        &["template_id", "name"],
        vec![
            column("template_id", T::id()).with_fk("authored.templates", "template_id"),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("logical_type_id", T::id()),
            column("enum_id", T::id()).optional(),
            column("default", T::native(arrow_schema::DataType::Utf8)).optional(),
            column("required", T::native(arrow_schema::DataType::Boolean)),
            column("domain_spec", T::native(arrow_schema::DataType::Utf8)).optional(),
            column("doc", T::native(arrow_schema::DataType::Utf8)),
        ],
        "blueprint §6.6 template: template_params.",
    );
}

fn declare_authored_template_features(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "template_features",
        S::Model,
        &["template_id", "name"],
        vec![
            column("template_id", T::id()).with_fk("authored.templates", "template_id"),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("kind", T::enumeration("FeatureKind")),
            column("enum_id", T::id()).optional(),
            column("default", T::native(arrow_schema::DataType::Utf8)).optional(),
            column("inherit_from", T::native(arrow_schema::DataType::Utf8)).optional(),
            column("doc", T::native(arrow_schema::DataType::Utf8)),
        ],
        "blueprint §6.6 template: template_features.",
    );
}

fn declare_authored_template_feature_rules(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "template_feature_rules",
        S::Model,
        &["template_id", "rule", "antecedent", "consequent"],
        vec![
            column("template_id", T::id()).with_fk("authored.templates", "template_id"),
            column("rule", T::enumeration("FeatureRuleKind")),
            column("antecedent", T::native(arrow_schema::DataType::Utf8)),
            column("consequent", T::native(arrow_schema::DataType::Utf8)),
        ],
        "blueprint §6.6 template: template_feature_rules.",
    );
}

fn declare_authored_template_guards(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "template_guards",
        S::Model,
        &["guard_id"],
        vec![
            column("guard_id", T::id()),
            column("template_id", T::id()).with_fk("authored.templates", "template_id"),
            column(
                "predicate",
                T::extended(crate::model::ExtensionUse::ExprDsl),
            ),
            column("doc", T::native(arrow_schema::DataType::Utf8)),
        ],
        "blueprint §6.6 template: template_guards.",
    );
}

fn declare_authored_template_domains(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "template_domains",
        S::Model,
        &["template_id", "name"],
        vec![
            column("template_id", T::id()).with_fk("authored.templates", "template_id"),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("kind", T::enumeration("DomainKind")),
            column("continuous", T::native(arrow_schema::DataType::Boolean)),
            column("members_from", T::native(arrow_schema::DataType::Utf8)).optional(),
            column(
                "bounds",
                structure(vec![
                    ("lower", T::native(arrow_schema::DataType::Float64)),
                    ("upper", T::native(arrow_schema::DataType::Float64)),
                ]),
            )
            .optional(),
            column("unit_id", T::id()).optional(),
        ],
        "blueprint §6.6 template: template_domains.",
    );
}

fn declare_authored_template_symbols(builder: &mut RegistryBuilder) {
    relation_version(
        builder,
        N::Authored,
        "template_symbols",
        2,
        S::Model,
        &["symbol_decl_id"],
        vec![
            column("template_id", T::id()).with_fk("authored.templates", "template_id"),
            column("symbol_decl_id", T::id()),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("role", T::enumeration("SymbolRole")),
            column("quantity_type_id", T::id()),
            column(
                "indexed_by",
                T::list(T::native(arrow_schema::DataType::Utf8)),
            ),
            column(
                "default_lower",
                T::extended(crate::model::ExtensionUse::Bound),
            )
            .optional(),
            column(
                "default_upper",
                T::extended(crate::model::ExtensionUse::Bound),
            )
            .optional(),
            column(
                "default_initial",
                T::native(arrow_schema::DataType::Float64),
            )
            .optional(),
            column(
                "reference_to",
                T::extended(crate::model::ExtensionUse::ExprDsl),
            )
            .optional(),
            column("wrt_domain", T::native(arrow_schema::DataType::Utf8)).optional(),
            column("guard_id", T::id()).optional(),
            column("idaes_name", T::native(arrow_schema::DataType::Utf8)).optional(),
            column("doc", T::native(arrow_schema::DataType::Utf8)),
        ],
        "blueprint §6.6 template: template_symbols.",
    );
}

fn declare_authored_template_equations(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "template_equations",
        S::Model,
        &["equation_decl_id"],
        vec![
            column("template_id", T::id()).with_fk("authored.templates", "template_id"),
            column("equation_decl_id", T::id()),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column(
                "indexed_by",
                T::list(T::native(arrow_schema::DataType::Utf8)),
            ),
            column("filter", T::extended(crate::model::ExtensionUse::ExprDsl)).optional(),
            column(
                "expression",
                T::extended(crate::model::ExtensionUse::ExprDsl),
            ),
            column("sense", T::enumeration("Sense")),
            column("family_hint", T::enumeration("EquationFamily")).optional(),
            column("role_hint", T::enumeration("EquationRole")).optional(),
            column("guard_id", T::id()).optional(),
            column("idaes_name", T::native(arrow_schema::DataType::Utf8)).optional(),
            column("doc", T::native(arrow_schema::DataType::Utf8)),
        ],
        "blueprint §6.6 template: template_equations.",
    );
}

fn declare_authored_template_submodels(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "template_submodels",
        S::Model,
        &["template_id", "name"],
        vec![
            column("template_id", T::id()).with_fk("authored.templates", "template_id"),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("child_template_id", T::id()).optional(),
            column("child_from_param", T::native(arrow_schema::DataType::Utf8)).optional(),
            column(
                "multiplicity_domain",
                T::native(arrow_schema::DataType::Utf8),
            )
            .optional(),
            column(
                "bindings",
                T::list(structure(vec![
                    ("child_param", T::native(arrow_schema::DataType::Utf8)),
                    ("value", T::extended(crate::model::ExtensionUse::ExprDsl)),
                ])),
            ),
            column("guard_id", T::id()).optional(),
        ],
        "blueprint §6.6 template: template_submodels.",
    );
}

fn declare_authored_template_ports(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "template_ports",
        S::Model,
        &["template_id", "name"],
        vec![
            column("template_id", T::id()).with_fk("authored.templates", "template_id"),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("kind", T::enumeration("PortKind")),
            column("direction", T::enumeration("Direction")),
            column("bound_to", T::native(arrow_schema::DataType::Utf8)),
            column("guard_id", T::id()).optional(),
            column("doc", T::native(arrow_schema::DataType::Utf8)),
        ],
        "blueprint §6.6 template: template_ports.",
    );
}

fn declare_authored_template_contributions(builder: &mut RegistryBuilder) {
    relation_version(
        builder,
        N::Authored,
        "template_contributions",
        2,
        S::Model,
        &["contribution_decl_id"],
        vec![
            column("template_id", T::id()).with_fk("authored.templates", "template_id"),
            column("contribution_decl_id", T::id()),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("law_family", T::enumeration("LawFamily")),
            column(
                "expression",
                T::extended(crate::model::ExtensionUse::ExprDsl),
            ),
            column("orientation", T::enumeration("Orientation")),
            column("scope", T::native(arrow_schema::DataType::Utf8)),
            column("guard_id", T::id()).optional(),
            column("doc", T::native(arrow_schema::DataType::Utf8)),
        ],
        "blueprint §6.6 template: template_contributions.",
    );
}

fn declare_authored_template_law_instances(builder: &mut RegistryBuilder) {
    relation_version(
        builder,
        N::Authored,
        "template_law_instances",
        2,
        S::Model,
        &["law_instance_decl_id"],
        vec![
            column("template_id", T::id()).with_fk("authored.templates", "template_id"),
            column("law_instance_decl_id", T::id()),
            column("law_template_id", T::id()),
            column("scope", T::native(arrow_schema::DataType::Utf8)),
            column(
                "options",
                T::list(structure(vec![
                    ("key", T::native(arrow_schema::DataType::Utf8)),
                    ("value", T::native(arrow_schema::DataType::Utf8)),
                ])),
            ),
            column("guard_id", T::id()).optional(),
        ],
        "blueprint §6.6 template: template_law_instances.",
    );
}

fn declare_authored_template_requirements(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "template_requirements",
        S::Model,
        &["template_id", "requirement"],
        vec![
            column("template_id", T::id()).with_fk("authored.templates", "template_id"),
            column("requirement", T::enumeration("CapabilityRequirement")),
            column("detail", T::native(arrow_schema::DataType::Utf8)).optional(),
        ],
        "blueprint §6.6 template: template_requirements.",
    );
}

fn declare_authored_template_display(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "template_display",
        S::Model,
        &["template_id", "kind", "label"],
        vec![
            column("template_id", T::id()).with_fk("authored.templates", "template_id"),
            column("kind", T::enumeration("DisplayKind")),
            column("label", T::native(arrow_schema::DataType::Utf8)),
            column(
                "expression",
                T::extended(crate::model::ExtensionUse::ExprDsl),
            ),
            column("display_unit_id", T::id()).optional(),
            column("format", T::native(arrow_schema::DataType::Utf8)).optional(),
        ],
        "blueprint §6.6 template: template_display.",
    );
}

fn declare_authored_template_property_requirements(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "template_property_requirements",
        S::Model,
        &["requirement_id"],
        vec![
            column("requirement_id", T::id()),
            column("template_id", T::id()).with_fk("authored.templates", "template_id"),
            column("operation_id", T::id()),
            column("scope_selector_id", T::id()).with_fk("authored.scopes", "scope_id"),
            column("property_kind_id", T::id()),
            column("guard", T::extended(crate::model::ExtensionUse::ExprDsl)).optional(),
            column(
                "index_domain_bindings",
                T::list(structure(vec![
                    ("index_name", T::native(arrow_schema::DataType::Utf8)),
                    ("domain_id", T::id()),
                ])),
            ),
        ],
        "blueprint §6.6 template: template_property_requirements.",
    );
}

fn declare_normalized_property_demand_seeds(builder: &mut RegistryBuilder) {
    relation_version(
        builder,
        N::Normalized,
        "property_demand_seeds",
        2,
        S::Derived,
        &["seed_id"],
        vec![
            column("seed_id", T::id()),
            column("source_id", T::id()),
            column("source_kind", T::enumeration("DemandSource")),
            column("scope_id", T::id()).with_fk("authored.scopes", "scope_id"),
            column("property_kind_id", T::id()),
            column("index", T::extended(crate::model::ExtensionUse::IndexTuple)).optional(),
            column("guard_node_id", T::nonnegative(i64::MAX)).optional(),
            crate::model::FieldContract::provenance(
                "derivation_id",
                T::id(),
                "Exact demand seed row derivation.",
            ),
            column("guard_source_id", T::id()).optional(),
            column("source_symbol_decl_id", T::id())
                .optional()
                .with_fk("authored.template_symbols", "symbol_decl_id"),
            column("read_node_id", T::nonnegative(i64::MAX)).optional(),
        ],
        "Each source-bound demand retains its exact normalized read node and guard; opaque explicit demands have no read node.",
    );
}

fn declare_template_kind_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "TemplateKind",
        [
            "unit",
            "control_volume",
            "state_block",
            "reaction_block",
            "connection_rule",
            "costing_method",
            "law",
            "initializer",
            "scaler",
            "flowsheet",
            "helper",
        ],
    );
}

fn declare_feature_kind_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(builder, "FeatureKind", ["bool", "enum", "choice"]);
}

fn declare_feature_rule_kind_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "FeatureRuleKind",
        ["implies", "excludes", "requires"],
    );
}

fn declare_symbol_role_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "SymbolRole",
        [
            "variable",
            "parameter",
            "expression",
            "derivative",
            "reference",
        ],
    );
}

fn declare_port_kind_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(builder, "PortKind", ["material", "heat", "work", "signal"]);
}

fn declare_direction_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(builder, "Direction", ["inlet", "outlet", "bidirectional"]);
}

fn declare_law_family_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "LawFamily",
        [
            "material", "energy", "momentum", "element", "charge", "cost", "utility",
        ],
    );
}

fn declare_orientation_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "Orientation",
        ["into_scope", "out_of_scope", "generation", "accumulation"],
    );
}

fn declare_capability_requirement_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "CapabilityRequirement",
        [
            "material_flow_terms",
            "enthalpy_flow_terms",
            "material_density_terms",
            "energy_density_terms",
            "diffusion_terms",
            "reaction_rate_basis",
            "phase_equilibrium",
        ],
    );
}

fn declare_display_kind_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "DisplayKind",
        [
            "variable",
            "expression",
            "parameter",
            "stream",
            "performance",
        ],
    );
}

fn declare_demand_source_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(builder, "DemandSource", ["expression", "opaque_operation"]);
}
