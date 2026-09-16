// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit dependency-to-input mapping and actual generated kernel output correspondence.
use super::{ExtensionUse, N, RegistryBuilder, S, T, column, relation};

pub(super) fn declare(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Reference,
        "method_kernel_inputs",
        S::Model,
        &["method_id", "input_name"],
        vec![
            column("method_id", T::id()).with_fk("reference.method_specs", "method_id"),
            column("input_name", T::native(arrow_schema::DataType::Utf8)),
            column(
                "dependency_ordinal",
                T::native(arrow_schema::DataType::UInt16),
            ),
        ],
        "Exact kernel input name to declared method dependency ordinal; complete ordered signature admission is required.",
    );
    relation(
        builder,
        N::Compiled,
        "kernel_output_symbols",
        S::Derived,
        &["requirement_id"],
        vec![
            column("symbol_id", T::id()).with_fk("compiled.symbols", "symbol_id"),
            column("requirement_id", T::id())
                .with_fk("inferred.property_requirements", "requirement_id"),
            column("state_scope_id", T::id()).with_fk("inferred.state_scopes", "state_scope_id"),
            column("method_id", T::id()).with_fk("reference.method_specs", "method_id"),
            column("kernel_binding_id", T::id()),
            column("output_ordinal", T::native(arrow_schema::DataType::UInt16)),
            column("index", T::extended(ExtensionUse::IndexTuple)),
            column("quantity_type_id", T::id())
                .with_fk("reference.quantity_types", "quantity_type_id"),
            column("derivation_id", T::id()),
        ],
        "Actual kernel output symbol correspondence; generated identity never certifies descriptor, body or execution availability.",
    );
}
