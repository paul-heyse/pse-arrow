// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Authoring sections map to exact declared rows (blueprint §22.1, ADR-0051).

use crate::builder::RegistryBuilder;
use crate::model::{DocumentKind, DocumentSection, DocumentSpec, DslSyntax, ExpressionOwnerKind};

/// Declare each package surface once. Generated DTOs and the loader share these mappings.
pub fn declare(builder: &mut RegistryBuilder) {
    builder.declare_document(DocumentSpec {
        name: "package_header",
        kind: DocumentKind::PackageHeader,
        path_glob: "package.toml",
        sections: vec![
            DocumentSection {
                key: "package",
                relation: "authored.packages",
                repeated: false,
                identity_column: Some("package_id"),
                entity_kind: None,
                name_column: None,
                naming_scope_column: None,
                expression_owner_column: None,
                expression_owner_kind: None,
                expression_fields: &[],
                doc: "The package header; content_hash is supplied by the loader.",
            },
            DocumentSection {
                key: "unit_sets",
                relation: "authored.package_unit_sets",
                repeated: true,
                identity_column: None,
                entity_kind: None,
                name_column: None,
                naming_scope_column: None,
                expression_owner_column: None,
                expression_owner_kind: None,
                expression_fields: &[],
                doc: "Optional explicit representation-unit selector, at most one per package.",
            },
        ],
        doc: "Package identity, exact dependencies and authored identity policy.",
    });
    declare_materials(builder);
    declare_methods(builder);
    declare_properties(builder);
    declare_templates(builder);
    declare_laws(builder);
    declare_costing(builder);
    declare_cases(builder);
    entities(
        builder,
        "computation_models",
        "computation_models/*.yaml",
        &[
            "authored.computation_models",
            "authored.dynamic_cases",
            "authored.native_providers",
            "authored.physical_balances",
            "authored.fit_cases",
        ],
    );
    entities(
        builder,
        "instances",
        "instances/*.yaml",
        &[
            "authored.instances",
            "authored.instance_equations",
            "authored.instance_domain_bindings",
            "authored.flowsheets",
            "authored.scopes",
            "authored.selector_terms",
            "authored.connections",
        ],
    );
    entities(
        builder,
        "assertions",
        "assertions/*.yaml",
        &["provenance.assertions"],
    );
}

fn declare_materials(builder: &mut RegistryBuilder) {
    entities(
        builder,
        "materials",
        "materials/*.yaml",
        &[
            "reference.dimensions",
            "reference.units",
            "reference.unit_sets",
            "reference.quantity_kinds",
            "reference.bases",
            "reference.reference_states",
            "reference.quantity_types",
            "reference.conversion_rules",
            "reference.quantity_operations",
            "reference.quantity_operation_reductions",
            "reference.quantity_preconditions",
            "reference.constants",
            "reference.elements",
            "reference.math_context",
            "authored.domains",
            "authored.domain_members",
            "authored.continuous_domains",
            "authored.species",
            "authored.species_elements",
            "authored.phases",
            "authored.phase_species",
            "authored.henry_declarations",
            "authored.material_systems",
            "authored.reactions",
            "authored.stoichiometry",
            "authored.reaction_methods",
            "authored.reaction_packages",
            "authored.parameter_values",
        ],
    );
}

fn declare_methods(builder: &mut RegistryBuilder) {
    entities(
        builder,
        "methods",
        "methods/*.yaml",
        &[
            "reference.property_kinds",
            "reference.method_specs",
            "reference.method_precedence",
            "reference.method_dependencies",
            "reference.method_provisions",
            "reference.method_parameters",
            "reference.method_state_parameters",
            "reference.method_parameter_axes",
            "reference.method_kernel_inputs",
        ],
    );
}

fn declare_properties(builder: &mut RegistryBuilder) {
    entities(
        builder,
        "properties",
        "properties/*.yaml",
        &[
            "authored.property_packages",
            "authored.state_bounds",
            "authored.phase_equilibrium_pairs",
            "authored.method_selections",
            "authored.default_scaling",
        ],
    );
}

fn declare_templates(builder: &mut RegistryBuilder) {
    entities(
        builder,
        "templates",
        "templates/*.yaml",
        &[
            "authored.templates",
            "authored.template_params",
            "authored.template_features",
            "authored.template_feature_rules",
            "authored.template_guards",
            "authored.template_domains",
            "authored.template_domain_bindings",
            "authored.template_material_constraints",
            "authored.template_symbols",
            "authored.template_symbol_contracts",
            "authored.template_derivatives",
            "authored.template_scopes",
            "authored.template_symbol_expressions",
            "authored.template_symbol_properties",
            "authored.template_equations",
            "authored.template_submodels",
            "authored.template_ports",
            "authored.template_port_members",
            "authored.template_contributions",
            "authored.template_contribution_contracts",
            "authored.template_law_instances",
            "authored.template_law_contracts",
            "authored.template_requirements",
            "authored.template_display",
            "authored.template_display_indices",
            "authored.template_property_requirements",
            "reference.connection_bindings",
        ],
    );
}

fn declare_laws(builder: &mut RegistryBuilder) {
    entities(
        builder,
        "laws",
        "laws/*.yaml",
        &[
            "authored.templates",
            "authored.template_law_instances",
            "authored.template_law_contracts",
            "reference.law_bindings",
            "reference.element_projection_contracts",
        ],
    );
}

fn declare_costing(builder: &mut RegistryBuilder) {
    entities(
        builder,
        "costing",
        "costing/*.yaml",
        &[
            "authored.templates",
            "reference.method_specs",
            "reference.constants",
        ],
    );
}

fn declare_cases(builder: &mut RegistryBuilder) {
    entities(
        builder,
        "cases",
        "cases/*.yaml",
        &[
            "authored.instances",
            "authored.instance_domain_bindings",
            "authored.flowsheets",
            "authored.scopes",
            "authored.selector_terms",
            "authored.connections",
            "authored.cases",
            "authored.case_specs",
            "authored.case_activations",
            "authored.case_objectives",
            "authored.case_policies",
            "authored.datasets",
            "authored.observations",
            "authored.scenarios",
            "authored.case_sets",
            "authored.case_set_samples",
        ],
    );
}

fn entities(
    builder: &mut RegistryBuilder,
    name: &'static str,
    path_glob: &'static str,
    relations: &[&'static str],
) {
    builder.declare_document(DocumentSpec {
        name,
        kind:DocumentKind::Entities,
        path_glob,
        sections:relations.iter().map(|relation|DocumentSection {
            key:relation.rsplit_once('.').map_or(*relation,|(_,name)|name),
            relation,
            repeated:true,
            identity_column: projection(relation).0,
            entity_kind: projection(relation).1,
            name_column: projection(relation).2,
            naming_scope_column: projection(relation).3,
            expression_owner_column: expression_owner(relation),
            expression_owner_kind: expression_owner(relation).map(|_| {
                if *relation == "authored.instance_equations" { ExpressionOwnerKind::Instance }
                else { ExpressionOwnerKind::Template }
            }),
            expression_fields: expression_fields(relation),
            doc:"Typed rows under the declared relation contract.",
        }).collect(),
        doc:"Package sections use declared relation fields; identity and provenance are resolved before typed decoding.",
    });
}

// Explicit projections are shared by every document exposing a relation. A missing
// entry means a relationship row, never permission to guess identity from its key.
type Projection = (
    Option<&'static str>,
    Option<&'static str>,
    Option<&'static str>,
    Option<&'static str>,
);
fn projection(relation: &str) -> Projection {
    let (identity, kind, name, scope) = match relation {
        "reference.units" => (Some("unit_id"), Some("unit"), Some("symbol"), None),
        "reference.unit_sets" => (Some("unit_set_id"), Some("unit_set"), Some("name"), None),
        "reference.quantity_kinds" => (
            Some("quantity_kind_id"),
            Some("quantity_kind"),
            Some("name"),
            None,
        ),
        "reference.constants" => (Some("constant_id"), Some("constant"), Some("name"), None),
        "reference.elements" => (Some("element_id"), Some("element"), Some("symbol"), None),
        "reference.method_specs" => (Some("method_id"), Some("method"), Some("name"), None),
        "authored.species" => (Some("species_id"), Some("species"), Some("name"), None),
        "authored.phases" => (Some("phase_id"), Some("phase"), Some("name"), None),
        "authored.material_systems" => (
            Some("material_system_id"),
            Some("material_system"),
            Some("name"),
            None,
        ),
        "authored.reactions" => (Some("reaction_id"), Some("reaction"), Some("name"), None),
        "authored.templates" => (Some("template_id"), Some("template"), Some("name"), None),
        "authored.template_symbols" => (
            Some("symbol_decl_id"),
            Some("symbol_declaration"),
            Some("name"),
            Some("template_id"),
        ),
        "authored.template_equations" => (
            Some("equation_decl_id"),
            Some("equation_declaration"),
            Some("name"),
            Some("template_id"),
        ),
        "authored.template_contributions" => (
            Some("contribution_decl_id"),
            Some("contribution_declaration"),
            Some("name"),
            Some("template_id"),
        ),
        "authored.instance_equations" => (
            Some("equation_decl_id"),
            Some("equation_declaration"),
            Some("name"),
            Some("instance_id"),
        ),
        "authored.method_selections" => (Some("selection_id"), None, None, None),
        "authored.instances" => (
            Some("instance_id"),
            Some("instance"),
            Some("name"),
            Some("parent_instance_id"),
        ),
        "authored.cases" => (Some("case_id"), Some("case"), Some("name"), None),
        "authored.datasets" => (Some("dataset_id"), Some("dataset"), Some("name"), None),
        "reference.bases" => (Some("basis_id"), None, None, None),
        "reference.reference_states" => (Some("reference_state_id"), None, None, None),
        "reference.quantity_types" => (Some("quantity_type_id"), None, None, None),
        "reference.conversion_rules" => (Some("conversion_id"), None, None, None),
        "reference.quantity_operations" => (Some("operation_id"), None, None, None),
        "reference.property_kinds" => (Some("property_kind_id"), None, None, None),
        "authored.domains" => (Some("domain_id"), None, None, None),
        "authored.domain_members" => (Some("member_id"), None, None, None),
        "authored.property_packages" => (
            Some("property_package_id"),
            Some("property_package"),
            Some("name"),
            None,
        ),
        "authored.reaction_packages" => (
            Some("reaction_package_id"),
            Some("reaction_package"),
            Some("name"),
            None,
        ),
        "authored.template_guards" => (Some("guard_id"), None, None, None),
        "authored.template_law_instances" => (Some("law_instance_decl_id"), None, None, None),
        "authored.template_property_requirements" => (Some("requirement_id"), None, None, None),
        "authored.scopes" => (Some("scope_id"), None, None, None),
        "authored.selector_terms" => (Some("term_id"), None, None, None),
        "authored.connections" => (Some("connection_id"), None, None, None),
        "authored.case_specs" => (Some("spec_id"), None, None, None),
        "authored.case_activations" => (Some("activation_id"), None, None, None),
        "authored.observations" => (Some("observation_id"), None, None, None),
        "authored.scenarios" => (Some("scenario_id"), None, None, None),
        "authored.case_sets" => (Some("case_set_id"), None, None, None),
        "provenance.assertions" => (Some("assertion_id"), None, None, None),
        _ => (None, None, None, None),
    };
    (identity, kind, name, scope)
}

fn expression_owner(relation: &str) -> Option<&'static str> {
    match relation {
        "authored.instance_equations" => Some("instance_id"),
        "authored.template_guards"
        | "authored.template_symbols"
        | "authored.template_symbol_expressions"
        | "authored.template_equations"
        | "authored.template_submodels"
        | "authored.template_contributions"
        | "authored.template_display"
        | "authored.template_property_requirements" => Some("template_id"),
        _ => None,
    }
}

fn expression_fields(relation: &str) -> &'static [(&'static str, DslSyntax)] {
    use DslSyntax::{Equation, Expression, Predicate};
    match relation {
        "authored.template_guards" => &[("predicate", Predicate)],
        "authored.template_symbols" => &[("reference_to", Expression)],
        "authored.template_equations" | "authored.instance_equations" => {
            &[("filter", Predicate), ("expression", Equation)]
        }
        "authored.template_submodels" => &[("bindings[].value", Expression)],
        "authored.template_contributions" => &[("expression", Expression)],
        "authored.template_display" | "authored.template_symbol_expressions" => {
            &[("expression", Expression)]
        }
        "authored.template_property_requirements" => &[("guard", Predicate)],
        _ => &[],
    }
}
