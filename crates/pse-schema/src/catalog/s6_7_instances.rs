// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! §6.7 instance.

use super::declarations::{column, relation, relation_version, structure};
use crate::builder::RegistryBuilder;
use crate::model::{FieldContract as T, Namespace as N, SnapshotClass as S};

/// Declares the §6.7 instance contracts.
pub fn declare(builder: &mut RegistryBuilder) {
    declare_authored_instances(builder);
    declare_authored_instance_domain_bindings(builder);
    declare_authored_flowsheets(builder);
    declare_authored_scopes(builder);
    declare_authored_selector_terms(builder);
    declare_authored_connections(builder);
    declare_inferred_instance_tree(builder);
    declare_inferred_instance_features(builder);
    declare_inferred_instances(builder);
    declare_inferred_ports(builder);
    declare_inferred_port_members(builder);
    declare_inferred_connection_equations(builder);
    declare_inferred_initialization_order(builder);
    declare_inferred_scope_members(builder);
    declare_inferred_boundary_crossings(builder);
    declare_inferred_topology_edges(builder);
    declare_inferred_undecided(builder);
    declare_tri_state_vocabulary(builder);
    declare_selector_op_vocabulary(builder);
    declare_crossing_vocabulary(builder);
}

fn declare_authored_instances(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "instances",
        S::Model,
        &["instance_id"],
        vec![
            column("instance_id", T::id()),
            column("parent_instance_id", T::id())
                .optional()
                .with_fk("authored.instances", "instance_id"),
            column("template_id", T::id()).with_fk("authored.templates", "template_id"),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column(
                "param_values",
                T::list(structure(vec![
                    ("name", T::native(arrow_schema::DataType::Utf8)),
                    ("value", T::native(arrow_schema::DataType::Utf8)),
                ])),
            ),
            column(
                "feature_values",
                T::list(structure(vec![
                    ("name", T::native(arrow_schema::DataType::Utf8)),
                    ("value", T::native(arrow_schema::DataType::Utf8)),
                ])),
            ),
            column("property_package_id", T::id()).optional(),
            column("reaction_package_id", T::id()).optional(),
            column("doc", T::native(arrow_schema::DataType::Utf8)),
        ],
        "blueprint §6.7 instance: instances.",
    );
}

fn declare_authored_flowsheets(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "flowsheets",
        S::Model,
        &["instance_id"],
        vec![
            column("instance_id", T::id()).with_fk("authored.instances", "instance_id"),
            column("time_domain_id", T::id()),
            column("dynamic", T::enumeration("TriState")),
            column("default_property_package_id", T::id()).optional(),
        ],
        "blueprint §6.7 instance: flowsheets.",
    );
}

fn declare_authored_scopes(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "scopes",
        S::Model,
        &["scope_id"],
        vec![column("scope_id", T::id()), column("root_term_id", T::id())],
        "blueprint §6.7 instance: scopes.",
    );
}

fn declare_authored_selector_terms(builder: &mut RegistryBuilder) {
    relation_version(
        builder,
        N::Authored,
        "selector_terms",
        2,
        S::Model,
        &["term_id"],
        vec![
            column("term_id", T::id()),
            column("scope_id", T::id()).with_fk("authored.scopes", "scope_id"),
            column("parent_term_id", T::id())
                .optional()
                .with_fk("authored.selector_terms", "term_id"),
            column("ordinal", T::nonnegative(i64::from(u16::MAX))),
            column("op", T::enumeration("SelectorOp")),
            column("entity_id", T::id()).optional(),
            column("entity_kind", T::enumeration("EntityKind")).optional(),
            column("tag", T::native(arrow_schema::DataType::Utf8)).optional(),
            column("parameter_name", T::native(arrow_schema::DataType::Utf8)).optional(),
        ],
        "blueprint §6.7 instance: selector_terms.",
    );
}

fn declare_authored_connections(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "connections",
        S::Model,
        &["connection_id"],
        vec![
            column("connection_id", T::id()),
            column("from_port_id", T::id()),
            column("to_port_id", T::id()),
            column("rule_template_id", T::id()),
            column("tear_cost", T::native(arrow_schema::DataType::Float64)).optional(),
            column("doc", T::native(arrow_schema::DataType::Utf8)),
        ],
        "blueprint §6.7 instance: connections.",
    );
}

fn declare_inferred_instance_tree(builder: &mut RegistryBuilder) {
    relation_version(
        builder,
        N::Inferred,
        "instance_tree",
        2,
        S::Derived,
        &["ancestor_id", "descendant_id"],
        vec![
            column("ancestor_id", T::id()),
            column("descendant_id", T::id()),
            column("depth", T::nonnegative(i64::from(u16::MAX))),
            crate::model::FieldContract::provenance(
                "derivation_id",
                T::id(),
                "Exact source derivation.",
            ),
        ],
        "blueprint §6.7 instance: instance_tree.",
    );
}

fn declare_inferred_instance_features(builder: &mut RegistryBuilder) {
    relation_version(
        builder,
        N::Inferred,
        "instance_features",
        2,
        S::Derived,
        &["instance_id", "name"],
        vec![
            column("instance_id", T::id()),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("value", super::s6_15_semantic::config_value_type()),
            crate::model::FieldContract::provenance(
                "derivation_id",
                T::id(),
                "Exact source derivation.",
            ),
        ],
        "blueprint §6.7 instance: instance_features.",
    );
}

fn declare_inferred_instances(builder: &mut RegistryBuilder) {
    relation_version(
        builder,
        N::Inferred,
        "instances",
        2,
        S::Derived,
        &["instance_id"],
        vec![
            column("instance_id", T::id()),
            column("parent_instance_id", T::id()).optional(),
            column("template_id", T::id()),
            column("path", T::native(arrow_schema::DataType::Utf8)),
            column("index", T::extended(crate::model::ExtensionUse::IndexTuple)),
            crate::model::FieldContract::provenance(
                "derivation_id",
                T::id(),
                "Exact source derivation.",
            ),
        ],
        "blueprint §6.7 instance: instances.",
    );
}

fn declare_inferred_ports(builder: &mut RegistryBuilder) {
    relation_version(
        builder,
        N::Inferred,
        "ports",
        2,
        S::Derived,
        &["port_id"],
        vec![
            column("port_id", T::id()),
            column("instance_id", T::id()),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("kind", T::enumeration("PortKind")),
            column("direction", T::enumeration("Direction")),
            column("state_instance_id", T::id()).optional(),
        ],
        "blueprint §6.7 instance: ports.",
    );
}

fn declare_inferred_port_members(builder: &mut RegistryBuilder) {
    relation_version(
        builder,
        N::Inferred,
        "port_members",
        2,
        S::Derived,
        &["port_id", "ordinal"],
        vec![
            column("port_id", T::id()),
            column("ordinal", T::nonnegative(i64::from(u16::MAX))),
            column("symbol_group", T::native(arrow_schema::DataType::Utf8)),
            column("symbol_decl_id", T::id()),
            column("quantity_type_id", T::id()),
            crate::model::FieldContract::provenance(
                "derivation_id",
                T::id(),
                "Exact source derivation.",
            ),
        ],
        "blueprint §6.7 instance: port_members.",
    );
}

fn declare_inferred_scope_members(builder: &mut RegistryBuilder) {
    relation_version(
        builder,
        N::Inferred,
        "scope_members",
        2,
        S::Derived,
        &["scope_id", "entity_id"],
        vec![
            column("scope_id", T::id()),
            column("entity_id", T::id()),
            crate::model::FieldContract::provenance(
                "derivation_id",
                T::id(),
                "Exact source derivation.",
            ),
        ],
        "blueprint §6.7 instance: scope_members.",
    );
}

fn declare_inferred_connection_equations(builder: &mut RegistryBuilder) {
    relation_version(
        builder,
        N::Inferred,
        "connection_equations",
        2,
        S::Derived,
        &["connection_id", "member_ordinal"],
        vec![
            column("connection_id", T::id()).with_fk("authored.connections", "connection_id"),
            column("member_ordinal", T::nonnegative(i64::from(u16::MAX))),
            column("product_id", T::id()),
            column("equation_id", T::id()),
        ],
        "blueprint §12.3: generated equation identity for each actual connection member and ordered index tuple.",
    );
}

fn declare_inferred_initialization_order(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Inferred,
        "initialization_order",
        S::Derived,
        &["instance"],
        vec![
            column("instance", T::id()),
            column("ordinal", T::nonnegative(i64::from(u16::MAX))),
        ],
        "blueprint §17.3: instance and plug-in preparation order; finalization traverses this order in reverse. Ordinal uses the bounded signed contract shared with §6.11 init_stages.",
    );
}

fn declare_inferred_boundary_crossings(builder: &mut RegistryBuilder) {
    relation_version(
        builder,
        N::Inferred,
        "boundary_crossings",
        2,
        S::Derived,
        &["scope_id", "connection_id"],
        vec![
            column("scope_id", T::id()),
            column("connection_id", T::id()),
            column("classification", T::enumeration("Crossing")),
            crate::model::FieldContract::provenance(
                "derivation_id",
                T::id(),
                "Exact source derivation.",
            ),
        ],
        "blueprint §6.7 instance: boundary_crossings.",
    );
}

fn declare_inferred_topology_edges(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Inferred,
        "topology_edges",
        S::Derived,
        &["from_instance_id", "to_instance_id", "connection_id"],
        vec![
            column("from_instance_id", T::id()),
            column("to_instance_id", T::id()),
            column("connection_id", T::id()),
        ],
        "blueprint §6.7 instance: topology_edges.",
    );
}

fn declare_inferred_undecided(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Inferred,
        "undecided",
        S::Derived,
        &["undecided_id"],
        vec![
            column("undecided_id", T::id()),
            column("rule_id", T::id()),
            column("head_relation_id", T::id()),
            column("key", T::row_key()),
            column("truth", T::enumeration("TruthValue")),
            column("reason", T::native(arrow_schema::DataType::Utf8)),
            column(
                "supporting",
                T::list(structure(vec![
                    ("relation_id", T::id()),
                    ("row_key", T::row_key()),
                ])),
            ),
            column("derivation_id", T::id()),
        ],
        "blueprint §6.7 instance: undecided.",
    );
}

fn declare_tri_state_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(builder, "TriState", ["true", "false", "inherit"]);
}

fn declare_selector_op_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "SelectorOp",
        [
            "self",
            "instance_parameter",
            "descendant_of",
            "kind_is",
            "tagged_with",
            "union",
            "intersection",
            "difference",
            "include",
            "exclude",
        ],
    );
}

fn declare_crossing_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "Crossing",
        ["internal", "external", "inbound", "outbound"],
    );
}

/// Explicit authored domain realization for a template-local domain name (ADR-0053).
fn declare_authored_instance_domain_bindings(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "instance_domain_bindings",
        S::Model,
        &["instance_id", "domain_name"],
        vec![
            column("instance_id", T::id()).with_fk("authored.instances", "instance_id"),
            column("domain_name", T::native(arrow_schema::DataType::Utf8)),
            column("domain_id", T::id()).with_fk("authored.domains", "domain_id"),
        ],
        "Actual domain bound to an instance's declared template-local domain name.",
    );
}
