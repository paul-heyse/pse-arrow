// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit numerical and semantic contracts of realized symbol declarations.

use super::{N, RegistryBuilder, S, T, column, derived, index, relation};

pub(super) fn declare(builder: &mut RegistryBuilder) {
    derived(
        builder,
        N::Compiled,
        "method_parameter_bindings",
        &["method_instance_id", "symbol_decl_id", "index"],
        vec![
            column("method_instance_id", T::id()),
            column("symbol_decl_id", T::id())
                .with_fk("authored.template_symbols", "symbol_decl_id"),
            column("index", index()),
            column("source_owner", T::id()),
            column("parameter_kind", T::native(arrow_schema::DataType::Utf8)),
            column("source_index", index()),
            column("value", T::native(arrow_schema::DataType::Float64)),
            column("unit_id", T::id()).with_fk("reference.units", "unit_id"),
            column("quantity_type_id", T::id())
                .with_fk("reference.quantity_types", "quantity_type_id"),
        ],
        "Exact source coefficient and physical contract for a selected method's parameter symbol; initial guesses are not parameter bindings.",
    );

    relation(
        builder,
        N::Authored,
        "template_derivatives",
        S::Model,
        &["symbol_decl_id"],
        vec![
            column("symbol_decl_id", T::id())
                .with_fk("authored.template_symbols", "symbol_decl_id"),
            column("order", T::native(arrow_schema::DataType::UInt8)),
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
