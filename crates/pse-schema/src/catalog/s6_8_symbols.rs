// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! §6.8 symbol.

use super::declarations::{column, relation};
use crate::builder::RegistryBuilder;
use crate::model::{FieldContract as T, Namespace as N, SnapshotClass as S};

/// Declares the §6.8 symbol contracts.
pub fn declare(builder: &mut RegistryBuilder) {
    declare_compiled_symbols(builder);
    declare_compiled_symbol_references(builder);
    declare_compiled_symbol_groups(builder);
    declare_compiled_symbol_group_members(builder);
    declare_solver_variable_type_vocabulary(builder);
    declare_variable_semantic_role_vocabulary(builder);
    declare_variable_lifecycle_vocabulary(builder);
    declare_alias_kind_vocabulary(builder);
}

fn declare_compiled_symbols(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "symbols",
        S::Derived,
        &["symbol_id"],
        vec![
            column("symbol_id", T::id()),
            column("ordinal", T::nonnegative(i64::MAX)),
            column("owner_instance_id", T::id()),
            column("symbol_decl_id", T::id()),
            column("qualified_name", T::native(arrow_schema::DataType::Utf8)),
            column("index", T::extended(crate::model::ExtensionUse::IndexTuple)),
            column("quantity_type_id", T::id()),
            column("unit_id", T::id()),
            column("role", T::enumeration("SymbolRole")),
            column("solver_type", T::enumeration("SolverVariableType")),
            column("semantic_role", T::enumeration("VariableSemanticRole")),
            column("lifecycle", T::enumeration("VariableLifecycle")),
            column(
                "default_lower",
                T::extended(crate::model::ExtensionUse::Bound),
            ),
            column(
                "default_upper",
                T::extended(crate::model::ExtensionUse::Bound),
            ),
            column(
                "default_initial",
                T::native(arrow_schema::DataType::Float64),
            )
            .optional(),
            column("derivation_id", T::id()),
        ],
        "blueprint §6.8 symbol: symbols.",
    );
}

fn declare_compiled_symbol_references(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "symbol_references",
        S::Derived,
        &["alias_symbol_id"],
        vec![
            column("alias_symbol_id", T::id()),
            column("target_symbol_id", T::id()),
            column("kind", T::enumeration("AliasKind")),
        ],
        "blueprint §6.8 symbol: symbol_references.",
    );
}

fn declare_compiled_symbol_groups(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "symbol_groups",
        S::Derived,
        &["group_id"],
        vec![
            column("group_id", T::id()),
            column("owner_instance_id", T::id()),
            column("name", T::native(arrow_schema::DataType::Utf8)),
            column("product_id", T::id()),
        ],
        "blueprint §6.8 symbol: symbol_groups.",
    );
}

fn declare_compiled_symbol_group_members(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Compiled,
        "symbol_group_members",
        S::Derived,
        &["group_id", "tuple"],
        vec![
            column("group_id", T::id()),
            column("tuple", T::extended(crate::model::ExtensionUse::IndexTuple)),
            column("symbol_id", T::id()),
        ],
        "blueprint §6.8 symbol: symbol_group_members.",
    );
}

fn declare_solver_variable_type_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "SolverVariableType",
        ["continuous", "binary", "integer"],
    );
}

fn declare_variable_semantic_role_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "VariableSemanticRole",
        [
            "state",
            "design_capacity",
            "allocation",
            "slack",
            "aux_reformulation",
            "reporting_only",
        ],
    );
}

fn declare_variable_lifecycle_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(
        builder,
        "VariableLifecycle",
        [
            "authored",
            "generated_semantic",
            "generated_discretization",
            "generated_reformulation",
            "generated_relaxation",
            "runtime_artifact",
        ],
    );
}

fn declare_alias_kind_vocabulary(builder: &mut RegistryBuilder) {
    super::declarations::enumeration(builder, "AliasKind", ["reference", "display_alias"]);
}
