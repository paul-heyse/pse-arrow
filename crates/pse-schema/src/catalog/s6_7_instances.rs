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
    declare_tri_state_vocabulary(builder);
    declare_selector_op_vocabulary(builder);
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
