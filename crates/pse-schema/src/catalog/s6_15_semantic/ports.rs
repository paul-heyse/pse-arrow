// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit state-template interface declarations, including expression symbols.
use super::{N, RegistryBuilder, S, T, column, derived, enumeration, index, relation};
pub(super) fn declare(builder: &mut RegistryBuilder) {
    port_binding_paths(builder);
    enumeration(builder, "ConnectionExpansion", ["equality"]);
    relation(
        builder,
        N::Reference,
        "connection_bindings",
        S::Model,
        &["rule_template_id"],
        vec![
            column("rule_template_id", T::id()).with_fk("authored.templates", "template_id"),
            column("expansion", T::enumeration("ConnectionExpansion")),
        ],
        "An actual declared connection template binds to its executable equality expansion; an arbitrary template identity does not establish connection semantics.",
    );
    derived(
        builder,
        N::Compiled,
        "port_member_groups",
        &["group_id"],
        vec![
            column("group_id", T::id()),
            column("port_id", T::id()),
            column("ordinal", T::nonnegative(i64::from(u16::MAX))),
            column("quantity_type_id", T::id()),
            column("product_id", T::id()),
        ],
        "Complete ordered port-state prefix plus symbol-local member collection, proved against actual P5 targets and P7 symbol groups.",
    );
    derived(
        builder,
        N::Inferred,
        "port_state_targets",
        &["port_id", "state_index"],
        vec![
            column("port_id", T::id()),
            column("state_index", index()),
            column("state_instance_id", T::id()),
        ],
        "Complete actual state collection behind a port; scalar binding has exactly one empty-index target.",
    );
    derived(
        builder,
        N::Inferred,
        "port_state_domains",
        &["port_id"],
        vec![
            column("port_id", T::id()),
            column("domain_ids", T::list(T::id())),
            column("product_id", T::id()),
        ],
        "Ordered domains and declared product of the complete bound state collection.",
    );
    relation(
        builder,
        N::Authored,
        "template_port_members",
        S::Model,
        &["template_id", "ordinal"],
        vec![
            column("template_id", T::id()).with_fk("authored.templates", "template_id"),
            column("ordinal", T::nonnegative(i64::from(u16::MAX))),
            column("symbol_decl_id", T::id())
                .with_fk("authored.template_symbols", "symbol_decl_id"),
            column("symbol_group", T::native(arrow_schema::DataType::Utf8)),
        ],
        "The bound state template declares its complete port interface; variable and expression symbol declarations are equally supported.",
    );
    derived(
        builder,
        N::Inferred,
        "port_member_domains",
        &["port_id", "ordinal"],
        vec![
            column("port_id", T::id()),
            column("ordinal", T::nonnegative(i64::from(u16::MAX))),
            column("domain_ids", T::list(T::id())),
            column("product_id", T::id()),
        ],
        "Complete port-state collection prefix and ordered symbol-local domain product; actual owners are retained solely in port_state_targets.",
    );
}

fn port_binding_paths(builder: &mut RegistryBuilder) {
    derived(
        builder,
        N::Normalized,
        "port_binding_lengths",
        &["template_id", "name"],
        vec![
            column("template_id", T::id()),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("length", T::nonnegative(i64::MAX)),
        ],
        "Parsed port binding path length: self has zero steps. Parsing establishes syntax only; actual child targets require native relational resolution.",
    );
    derived(
        builder,
        N::Normalized,
        "port_binding_steps",
        &["template_id", "name", "position"],
        vec![
            column("template_id", T::id()),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("position", T::nonnegative(i64::MAX)),
            column("child_name", T::native(arrow_schema::DataType::Utf8)),
        ],
        "Ordered exact child names parsed once from the authored port binding; no instance membership or cardinality is inferred by the parser.",
    );
}
