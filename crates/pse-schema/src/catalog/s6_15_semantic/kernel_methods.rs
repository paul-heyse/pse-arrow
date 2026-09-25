// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit dependency-to-input mapping and actual generated kernel output correspondence.
use super::{N, RegistryBuilder, S, T, column, relation};

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
            column("dependency_ordinal", T::nonnegative(i64::from(u16::MAX))),
        ],
        "Exact kernel input name to declared method dependency ordinal; complete ordered signature admission is required.",
    );
}
