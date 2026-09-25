// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit numerical and semantic contracts of realized symbol declarations.

use super::{N, RegistryBuilder, S, T, column, relation};

pub(super) fn declare(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "template_derivatives",
        S::Model,
        &["symbol_decl_id"],
        vec![
            column("symbol_decl_id", T::id())
                .with_fk("authored.template_symbols", "symbol_decl_id"),
            column("order", T::nonnegative(i64::from(u8::MAX))),
        ],
        "Explicit positive derivative order; derivative-role reference_to binds the actual base symbol and wrt_domain names the declared continuous axis.",
    );
    relation(
        builder,
        N::Authored,
        "template_symbol_contracts",
        S::Model,
        &["symbol_decl_id"],
        vec![
            column("symbol_decl_id", T::id())
                .with_fk("authored.template_symbols", "symbol_decl_id"),
            column("solver_type", T::enumeration("SolverVariableType")),
            column("semantic_role", T::enumeration("VariableSemanticRole")),
        ],
        "Every realized symbol declaration has exactly one explicit solver and semantic role contract, including parameter, reference and expression roles.",
    );
}
