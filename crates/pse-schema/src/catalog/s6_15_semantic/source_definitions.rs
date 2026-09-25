// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Authored expression meanings retained independently of mathematical storage.
use super::{ExtensionUse, N, RegistryBuilder, S, T, column, relation};
pub(super) fn declare(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "template_symbol_expressions",
        S::Model,
        &["symbol_decl_id"],
        vec![
            column("symbol_decl_id", T::id())
                .with_fk("authored.template_symbols", "symbol_decl_id"),
            column("template_id", T::id()).with_fk("authored.templates", "template_id"),
            column("expression", T::extended(ExtensionUse::ExprDsl)),
        ],
        "The sole expression-role symbol body; owner and role must match the symbol declaration.",
    );
}
