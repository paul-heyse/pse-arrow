// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact indexed root environments and conditional equation source correspondence.
use super::{N, RegistryBuilder, T, column, derived};

pub(super) fn declare(builder: &mut RegistryBuilder) {
    derived(
        builder,
        N::Compiled,
        "expression_root_indices",
        &["owner_id", "role", "ordinal", "position"],
        vec![
            column("owner_id", T::id()),
            column("role", T::enumeration("ExpressionRootRole")),
            column("ordinal", T::native(arrow_schema::DataType::UInt16)),
            column("position", T::native(arrow_schema::DataType::UInt16)),
            column("bound_index_id", T::id()),
            column("domain_id", T::id()).with_fk("normalized.domains", "domain_id"),
        ],
        "Complete source-ordered actual free-index environment for one expression root; scalar roots have no index rows.",
    );
    derived(
        builder,
        N::Compiled,
        "equation_branches",
        &["indexed_equation_id"],
        vec![
            column("indexed_equation_id", T::id()),
            column("parent_indexed_equation_id", T::id()),
            column("instance_id", T::id()),
            column("equation_decl_id", T::id()),
            column("source_id", T::id()),
            column(
                "equation_node_id",
                T::native(arrow_schema::DataType::UInt64),
            ),
            column(
                "branch_guards",
                T::list(T::structure(vec![
                    T::id().with_name("source_id").with_nullable(false),
                    T::native(arrow_schema::DataType::UInt64)
                        .with_name("predicate_id")
                        .with_nullable(false),
                    T::native(arrow_schema::DataType::Boolean)
                        .with_name("expected")
                        .with_nullable(false),
                ])),
            ),
        ],
        "Exact conditional-equation leaf correspondence with complete complementary actual predicate guards; indexed equations retain their declared senses.",
    );
}
