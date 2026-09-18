// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit P4–P10 contracts projected from their registered rules and native adapters.
use crate::{
    RegistryBuilder,
    model::{
        AlgorithmDecl, ArgumentSpec, Authority, Determinism, Namespace, RelationDecl, ResultSpec,
    },
};
use std::collections::BTreeSet;

/// Declare typed finite algorithm signatures; actual producers are bound native children.
#[expect(
    clippy::too_many_lines,
    reason = "Keep the declarative relation and native rule family together for schema review"
)]
pub fn declare(builder: &mut RegistryBuilder) {
    let relations = builder.declared_relations().to_vec();
    let mut graph_outputs = BTreeSet::new();
    for pass in ["P4", "P5", "P6", "P7", "P8", "P9", "P10"] {
        let mut inputs = BTreeSet::<String>::new();
        let mut outputs = BTreeSet::<String>::new();
        let rules = builder
            .declared_rules()
            .iter()
            .filter(|rule| {
                [pass]
                    .into_iter()
                    .chain((pass == "P9").then_some("P4"))
                    .chain((pass == "P9").then_some("P5"))
                    .any(|producer| {
                        rule.name
                            .strip_prefix(producer)
                            .is_some_and(|suffix| suffix.starts_with('.'))
                    })
            })
            .collect::<Vec<_>>();
        for rule in &rules {
            outputs.insert(rule.head.as_str().to_owned());
            if let Some(assertions) = &rule.assertion_relation {
                outputs.insert(assertions.clone());
            }
            for input in &rule.inputs {
                let relation = &input.relation;
                inputs.insert(relation.to_owned());
            }
        }
        let relational = matches!(pass, "P4" | "P5" | "P6" | "P8" | "P9");
        if relational {
            outputs.extend([
                "inferred.rule_outcomes".to_owned(),
                "provenance.rule_support_edges".to_owned(),
            ]);
        }
        for name in native_inputs(pass) {
            inputs.insert((*name).to_owned());
        }
        for name in native_outputs(pass) {
            outputs.insert((*name).to_owned());
        }
        if pass == "P9" {
            // Selected method roots use the sole P3 configuration algorithm and
            // run P4/P5 for the new instances. Their outputs are explicit stage
            // ports, including unchanged and empty relations.
            inputs.extend(
                P4_INPUTS
                    .iter()
                    .chain(P5_INPUTS)
                    .chain(P7_INPUTS)
                    .map(|name| (*name).to_owned()),
            );
            outputs.extend(
                native_outputs("P4")
                    .into_iter()
                    .chain(native_outputs("P5"))
                    .map(str::to_owned),
            );
            outputs.extend(SELECTED_CONFIGURATION.iter().map(|name| (*name).to_owned()));
        }
        if matches!(pass, "P4" | "P5" | "P6" | "P7" | "P9" | "P10") {
            for prefix in ["template", "instance", "display", "contribution", "guard"] {
                for relation in &relations {
                    if relation.key.namespace == Namespace::Compiled
                        && let Some(name) =
                            super::expr_family::target_name(prefix, relation.key.name)
                    {
                        inputs.insert(format!("normalized.{name}"));
                    }
                }
            }
        }
        // Predicate evaluation also resolves physical coordinates and quantities.
        // Every semantic consumer binds these actual facts before construction.
        inputs.extend(PHYSICAL_INPUTS.iter().map(|name| (*name).to_owned()));
        inputs.insert("reference.kernel_specs".to_owned());
        if pass == "P6" {
            inputs.insert("normalized.template_guards".to_owned());
            inputs.extend(
                relations
                    .iter()
                    .filter(|relation| {
                        relation.key.namespace == Namespace::Normalized
                            && relation
                                .columns
                                .iter()
                                .any(|column| column.name() == "guard_id")
                    })
                    .map(|relation| relation.key.qualified_name()),
            );
        }
        if pass == "P7" {
            outputs.extend(GRAPH_RELATIONS.iter().map(|name| (*name).to_owned()));
            outputs.extend(
                relations
                    .iter()
                    .filter(|relation| math(relation, Namespace::Inferred))
                    .map(|relation| relation.key.qualified_name()),
            );
            graph_outputs = outputs.clone();
        } else if matches!(pass, "P8" | "P9") {
            inputs.extend(graph_outputs.iter().cloned());
            outputs.extend(graph_outputs.iter().cloned());
            // Only mathematical results form the next graph input. Rule facts stay
            // with their own producer and reach consumers through declared reads.
            graph_outputs = relations
                .iter()
                .filter(|relation| {
                    relation.key.namespace == Namespace::Compiled
                        || math(relation, Namespace::Inferred)
                        || relation.key.qualified_name() == "inferred.connection_equations"
                })
                .map(|relation| relation.key.qualified_name())
                .filter(|name| outputs.contains(name))
                .collect();
        } else if pass == "P10" {
            inputs.extend(graph_outputs.iter().cloned());
            outputs.extend(
                graph_outputs
                    .iter()
                    .filter(|name| name.starts_with("compiled."))
                    .cloned(),
            );
            outputs.extend(
                relations
                    .iter()
                    .filter(|relation| math(relation, Namespace::Compiled))
                    .map(|relation| relation.key.qualified_name()),
            );
        }
        // A rule head is a private worktable inside this pass. A preserved native graph
        // output remains an explicit input from its fixed previous producer.
        let heads = rules
            .iter()
            .map(|rule| rule.head.as_str())
            .collect::<BTreeSet<_>>();
        inputs.retain(|name| {
            !(heads.contains(name.as_str())
                || native_constructed(pass).contains(&name.as_str())
                || pass == "P9" && P5_CONSTRUCTED.contains(&name.as_str()))
        });
        if pass == "P9" {
            inputs.insert("inferred.instances".to_owned());
        }
        // Obligations read explicit input/result values, including negative reads.
        // Their dependencies are algorithm arguments unless this invocation produces them.
        for invariant in builder
            .declared_invariants()
            .iter()
            .filter(|invariant| outputs.contains(&invariant.relation))
        {
            inputs.extend(
                invariant
                    .inputs
                    .iter()
                    .filter(|name| !outputs.contains(*name))
                    .cloned(),
            );
        }
        let ports = inputs
            .into_iter()
            .map(|name| {
                let declaration = relations
                    .iter()
                    .find(|relation| relation.key.qualified_name() == name);
                let port = declaration.map_or_else(|| "__undeclared_relation".into(), port_name);
                ArgumentSpec {
                    port,
                    relation: name,
                    required: true,
                    consumption: crate::model::algorithm::InputConsumption::Whole,
                }
            })
            .collect();
        let output_ports = outputs
            .iter()
            .map(|name| {
                let declaration = relations
                    .iter()
                    .find(|relation| relation.key.qualified_name() == *name);
                ResultSpec {
                    port: declaration.map_or_else(|| "__undeclared_relation".into(), port_name),
                    relation: name.clone(),
                }
            })
            .collect();
        let postconditions = builder
            .declared_invariants()
            .iter()
            .filter(|invariant| outputs.contains(&invariant.relation))
            .map(|invariant| format!("{}:{}", invariant.relation, invariant.name))
            .collect();
        let declaration = AlgorithmDecl::new(pass, "1", Determinism::Deterministic)
            .inputs(ports)
            .outputs(output_ports)
            .conditions(vec![], postconditions)
            .diagnostics(vec![
                "validation.invariant",
                "compile.math",
                "runtime.resource_limit",
                "kernel.unbound_parameter",
            ]);
        builder.declare_algorithm(declaration);
    }
}
fn port_name(relation: &RelationDecl) -> String {
    let name = if relation.key.namespace == Namespace::Authored {
        match relation.key.name {
            "templates" => "authored_templates",
            "template_submodels" => "authored_template_submodels",
            "instances" => "authored_instances",
            "template_params" => "authored_template_params",
            "template_features" => "authored_template_features",
            "template_guards" => "authored_template_guards",
            "template_domains" => "authored_template_domains",
            "template_domain_bindings" => "authored_template_domain_bindings",
            "instance_domain_bindings" => "authored_instance_domain_bindings",
            "domains" => "authored_domains",
            "domain_members" => "authored_domain_members",
            "flowsheets" => "authored_flowsheets",
            "property_packages" => "authored_property_packages",
            "reaction_packages" => "authored_reaction_packages",
            "material_systems" => "authored_material_systems",
            "species" => "authored_species",
            "phases" => "authored_phases",
            "species_elements" => "authored_species_elements",
            _ => return relation.key.qualified_name(),
        }
    } else if relation.key.namespace == Namespace::Normalized && relation.key.name == "units" {
        "normalized_units"
    } else {
        relation.key.name
    };
    name.to_owned()
}
fn math(relation: &RelationDecl, namespace: Namespace) -> bool {
    relation.authority == Authority::Derived
        && relation.key.namespace == namespace
        && (relation.key.name.starts_with("math_") || relation.key.name == "kernel_bindings")
}
fn native_inputs(pass: &str) -> &'static [&'static str] {
    match pass {
        "P4" => P4_INPUTS,
        "P5" => P5_INPUTS,
        "P6" => P6_INPUTS,
        "P7" => P7_INPUTS,
        "P8" => P8_INPUTS,
        "P9" => P9_INPUTS,
        "P10" => P10_INPUTS,
        _ => &[],
    }
}
fn native_constructed(pass: &str) -> &'static [&'static str] {
    match pass {
        "P5" => P5_CONSTRUCTED,
        "P6" => P6_CONSTRUCTED,
        "P8" => &["inferred.law_contexts", "inferred.law_axes"],
        _ => &[],
    }
}
fn native_outputs(pass: &str) -> Vec<&'static str> {
    let mut output = native_constructed(pass).to_vec();
    output.extend(match pass {
        "P4" => vec!["inferred.predicate_axes", "inferred.predicate_outcomes"],
        "P5" => vec![
            "inferred.instance_tree",
            "inferred.path_targets",
            "inferred.topology_edges",
            "inferred.boundary_crossings",
            "inferred.tear_candidates",
        ],
        "P8" => vec![
            "compiled.law_participation",
            "compiled.port_member_groups",
            "compiled.element_projection_groups",
            "compiled.element_projection_coefficients",
            "inferred.connection_equations",
        ],
        "P9" => vec![
            "compiled.method_realizations",
            "compiled.method_parameter_bindings",
            "compiled.kernel_output_symbols",
            "inferred.instances",
            "inferred.path_targets",
            "normalized.instance_bindings",
            "normalized.instance_domain_bindings",
            "normalized.config_values",
        ],
        _ => vec![],
    });
    output
}
/// Actual input families of the shared quantity/material inventory. Compiler
/// readers and provenance use this same selector as the registered pass ports.
pub const PHYSICAL_INPUTS: &[&str] = &[
    "reference.units",
    "normalized.units",
    "reference.unit_sets",
    "reference.quantity_kinds",
    "reference.bases",
    "reference.reference_states",
    "reference.quantity_types",
    "reference.conversion_rules",
    "reference.quantity_operations",
    "reference.quantity_operation_reductions",
    "reference.quantity_preconditions",
    "reference.math_context",
    "reference.elements",
];
const P4_INPUTS: &[&str] = &[
    "normalized.expression_sources",
    "normalized.predicate_nodes",
    "normalized.expression_index_bindings",
    "normalized.expression_paths",
    "normalized.instance_bindings",
    "normalized.instance_domain_bindings",
    "normalized.domains",
    "normalized.domain_members",
    "normalized.domain_products",
    "normalized.candidate_index_tuples",
    "normalized.template_submodels",
    "normalized.template_symbols",
    "normalized.template_params",
    "normalized.template_features",
    "normalized.template_ports",
    "normalized.config_values",
];
const P5_CONSTRUCTED: &[&str] = &[
    "inferred.scope_candidates",
    "inferred.selector_contexts",
    "inferred.port_candidates",
    "inferred.port_state_candidates",
    "inferred.port_state_domain_candidates",
    "inferred.port_member_candidates",
    "inferred.port_member_domain_candidates",
];
const P5_INPUTS: &[&str] = &[
    "normalized.port_binding_lengths",
    "normalized.port_binding_steps",
    "normalized.scopes",
    "normalized.template_scopes",
    "normalized.selector_nodes",
    "normalized.instance_bindings",
    "normalized.config_values",
    "normalized.template_params",
    "normalized.template_ports",
    "normalized.template_submodels",
    "normalized.template_symbols",
    "normalized.template_features",
    "normalized.template_port_members",
    "normalized.template_guards",
    "normalized.instance_domain_bindings",
    "normalized.domains",
    "normalized.domain_members",
    "normalized.domain_products",
    "normalized.connections",
    "normalized.expression_sources",
    "normalized.expression_paths",
    "normalized.expression_index_bindings",
    "normalized.predicate_nodes",
    "inferred.predicate_axes",
    "inferred.predicate_outcomes",
    "inferred.instance_features",
    "reference.quantity_types",
];
const P6_CONSTRUCTED: &[&str] = &[
    "inferred.state_scope_keys",
    "inferred.requirement_keys",
    "inferred.requirement_key_axes",
    "inferred.requirement_scope_keys",
    "inferred.state_method_selection_keys",
    "inferred.dependency_key_maps",
    "inferred.method_parameter_keys",
    "inferred.demand_request_keys",
    "inferred.demand_read_keys",
    "inferred.read_coordinate_failures",
    "inferred.demand_index_maps",
    "inferred.state_dependency_keys",
];
const P6_INPUTS: &[&str] = &[
    "reference.method_parameter_axes",
    "normalized.instance_bindings",
    "normalized.property_packages",
    "normalized.instance_domain_bindings",
    "normalized.domains",
    "normalized.domain_members",
    "normalized.domain_products",
    "normalized.material_domain_members",
    "normalized.property_demand_seeds",
    "normalized.property_path_demands",
    "normalized.template_symbols",
    "normalized.template_domains",
    "normalized.template_property_requirements",
    "normalized.template_symbol_properties",
    "normalized.expression_sources",
    "normalized.expression_index_bindings",
    "normalized.expression_paths",
    "normalized.predicate_nodes",
    "normalized.config_values",
    "inferred.instances",
    "inferred.instance_features",
    "inferred.valid_index_tuples",
    "inferred.scope_members",
    "inferred.scope_bindings",
    "inferred.path_targets",
    "inferred.predicate_outcomes",
    "inferred.predicate_axes",
    "reference.property_kinds",
    "reference.method_dependencies",
    "reference.method_provisions",
    "reference.method_parameters",
    "reference.quantity_types",
];
const P7_INPUTS: &[&str] = &[
    "normalized.connections",
    "reference.connection_bindings",
    "normalized.instance_bindings",
    "normalized.templates",
    "normalized.template_symbols",
    "normalized.template_symbol_contracts",
    "normalized.template_derivatives",
    "normalized.template_symbol_expressions",
    "normalized.template_equations",
    "normalized.instance_equations",
    "normalized.template_submodels",
    "normalized.template_params",
    "normalized.template_features",
    "normalized.template_ports",
    "normalized.template_guards",
    "normalized.instance_domain_bindings",
    "normalized.domains",
    "normalized.domain_members",
    "normalized.continuous_domains",
    "normalized.domain_products",
    "normalized.domain_product_projections",
    "normalized.config_values",
    "normalized.expression_sources",
    "normalized.expression_index_bindings",
    "normalized.equation_nodes",
    "normalized.predicate_nodes",
    "normalized.expression_paths",
    "normalized.template_contributions",
    "normalized.template_contribution_contracts",
    "normalized.template_scopes",
    "inferred.instances",
    "inferred.instance_features",
    "inferred.valid_index_tuples",
    "inferred.predicate_outcomes",
    "inferred.predicate_axes",
    "inferred.path_targets",
    "inferred.scope_bindings",
    "inferred.ports",
    "inferred.port_members",
    "inferred.port_state_targets",
    "inferred.port_state_domains",
    "inferred.port_member_domains",
    "inferred.property_requirements",
    "inferred.method_resolutions",
];
const P8_INPUTS: &[&str] = &[
    "inferred.domain_eligible_members",
    "normalized.property_packages",
    "reference.method_specs",
    "reference.element_projection_contracts",
    "reference.connection_bindings",
    "normalized.template_law_instances",
    "normalized.template_law_contracts",
    "normalized.template_scopes",
    "normalized.template_guards",
    "normalized.template_submodels",
    "normalized.template_symbols",
    "normalized.template_domains",
    "normalized.expression_sources",
    "normalized.expression_index_bindings",
    "normalized.template_ports",
    "normalized.instance_bindings",
    "normalized.instance_domain_bindings",
    "normalized.domains",
    "normalized.domain_members",
    "normalized.continuous_domains",
    "normalized.domain_products",
    "normalized.material_domain_members",
    "normalized.species",
    "normalized.species_elements",
    "normalized.connections",
    "inferred.instances",
    "inferred.instance_features",
    "inferred.predicate_outcomes",
    "inferred.scope_bindings",
    "inferred.scope_members",
    "inferred.boundary_crossings",
    "inferred.valid_index_tuples",
    "inferred.ports",
    "inferred.port_members",
    "inferred.port_state_targets",
    "inferred.port_state_domains",
    "inferred.port_member_domains",
    "inferred.property_requirements",
    "inferred.method_resolutions",
    "reference.schema_enums",
    "reference.law_bindings",
    "reference.elements",
];
const SELECTED_CONFIGURATION: &[&str] = &[
    "normalized.instance_bindings",
    "normalized.material_domain_members",
    "normalized.config_values",
    "normalized.feature_inheritance",
    "normalized.instance_domain_bindings",
    "normalized.domain_products",
    "normalized.instance_binding_products",
    "normalized.candidate_index_tuples",
    "normalized.candidate_index_members",
    "normalized.domain_product_sources",
    "normalized.domain_product_projections",
    "normalized.domains",
    "normalized.domain_members",
];
const P9_INPUTS: &[&str] = &[
    "normalized.template_symbol_properties",
    "inferred.dependency_key_maps",
    "inferred.state_dependency_keys",
    "reference.method_kernel_inputs",
    "reference.method_state_parameters",
    "reference.method_parameter_axes",
    "authored.templates",
    "authored.template_submodels",
    "authored.instances",
    "authored.template_params",
    "authored.template_features",
    "authored.template_guards",
    "authored.template_domains",
    "authored.template_domain_bindings",
    "authored.instance_domain_bindings",
    "authored.domains",
    "authored.domain_members",
    "authored.flowsheets",
    "authored.property_packages",
    "authored.reaction_packages",
    "authored.material_systems",
    "authored.species",
    "authored.phases",
    "authored.species_elements",
    "reference.elements",
    "normalized.instance_bindings",
    "normalized.templates",
    "normalized.template_symbols",
    "normalized.template_symbol_contracts",
    "normalized.template_derivatives",
    "normalized.template_symbol_expressions",
    "normalized.template_equations",
    "normalized.instance_equations",
    "normalized.template_submodels",
    "normalized.template_params",
    "normalized.template_features",
    "normalized.template_ports",
    "normalized.template_guards",
    "normalized.template_domain_bindings",
    "normalized.instance_domain_bindings",
    "normalized.domains",
    "normalized.domain_members",
    "normalized.continuous_domains",
    "normalized.domain_products",
    "normalized.domain_product_projections",
    "normalized.material_domain_members",
    "normalized.config_values",
    "normalized.expression_sources",
    "normalized.expression_index_bindings",
    "normalized.equation_nodes",
    "normalized.predicate_nodes",
    "normalized.expression_paths",
    "normalized.template_contributions",
    "normalized.template_contribution_contracts",
    "normalized.template_scopes",
    "normalized.parameter_values",
    "normalized.property_packages",
    "inferred.instances",
    "inferred.instance_features",
    "inferred.valid_index_tuples",
    "inferred.predicate_outcomes",
    "inferred.predicate_axes",
    "inferred.path_targets",
    "inferred.scope_bindings",
    "inferred.ports",
    "inferred.port_members",
    "inferred.port_state_targets",
    "inferred.port_state_domains",
    "inferred.port_member_domains",
    "inferred.state_scopes",
    "inferred.property_requirements",
    "inferred.method_resolutions",
    "reference.method_specs",
    "reference.method_provisions",
    "reference.method_parameters",
    "reference.method_dependencies",
    "reference.property_kinds",
];
const P10_INPUTS: &[&str] = &[
    "reference.element_projection_contracts",
    "reference.method_kernel_inputs",
    "normalized.species",
    "normalized.species_elements",
    "reference.elements",
    "normalized.template_symbol_properties",
    "normalized.property_packages",
    "inferred.property_requirements",
    "inferred.method_resolutions",
    "reference.method_provisions",
    "reference.method_dependencies",
    "reference.method_state_parameters",
    "reference.method_parameter_axes",
    "inferred.ports",
    "inferred.port_members",
    "inferred.port_member_domains",
    "inferred.port_state_targets",
    "inferred.port_state_domains",
    "normalized.templates",
    "normalized.template_symbols",
    "normalized.template_derivatives",
    "normalized.template_domains",
    "normalized.instance_domain_bindings",
    "normalized.instance_bindings",
    "normalized.domains",
    "normalized.domain_members",
    "normalized.continuous_domains",
    "normalized.domain_products",
    "normalized.parameter_values",
    "normalized.material_domain_members",
    "normalized.expression_sources",
    "normalized.expression_index_bindings",
    "normalized.expression_paths",
    "normalized.config_values",
    "normalized.template_params",
    "normalized.template_features",
    "normalized.template_submodels",
    "normalized.template_ports",
    "normalized.predicate_nodes",
    "inferred.instances",
    "inferred.instance_features",
    "inferred.valid_index_tuples",
    "inferred.predicate_outcomes",
    "inferred.predicate_axes",
    "inferred.path_targets",
    "inferred.state_scopes",
    "reference.method_parameters",
    "reference.method_specs",
];
const GRAPH_RELATIONS: &[&str] = &[
    "compiled.symbols",
    "compiled.symbol_groups",
    "compiled.symbol_group_members",
    "compiled.symbol_references",
    "compiled.symbol_expressions",
    "compiled.expression_roots",
    "compiled.expression_root_indices",
    "compiled.equation_branches",
    "compiled.contributions",
    "compiled.predicate_masks",
    "compiled.predicate_mask_members",
    "compiled.group_collections",
    "compiled.group_collection_members",
    "compiled.group_projections",
    "compiled.group_reindexings",
    "compiled.group_reindexing_members",
];
