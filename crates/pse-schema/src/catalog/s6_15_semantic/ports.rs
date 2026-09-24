// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit state-template interface declarations, including expression symbols.
use super::{N, RegistryBuilder, S, T, column, enumeration, relation};
pub(super) fn declare(builder: &mut RegistryBuilder) {
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

}


