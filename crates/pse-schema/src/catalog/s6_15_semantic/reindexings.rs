// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact finite arithmetic index correspondence (ADR-0063).
use super::{N, RegistryBuilder, T, column, derived, index};

pub(super) fn declare(builder: &mut RegistryBuilder) {
    derived(
        builder,
        N::Compiled,
        "group_reindexings",
        &["group_id"],
        vec![
            column("group_id", T::id()).with_fk("compiled.symbol_groups", "group_id"),
            column("source_group_id", T::id()).with_fk("compiled.symbol_groups", "group_id"),
            column("instance_id", T::id()),
            column("source_id", T::id()).with_fk("normalized.expression_sources", "source_id"),
            column("source_node_id", T::native(arrow_schema::DataType::UInt64)),
            column("bound_indices", T::list(T::id())),
            column(
                "fixed_indices",
                T::list(T::structure(vec![
                    T::id().with_name("bound_index_id").with_nullable(false),
                    T::id().with_name("member_id").with_nullable(false),
                ])),
            ),
            column("product_id", T::id()).with_fk("normalized.domain_products", "product_id"),
        ],
        "Actual source arithmetic subscripts under an explicit free/fixed binder environment.",
    );
    derived(
        builder,
        N::Compiled,
        "group_reindexing_members",
        &["group_id", "index"],
        vec![
            column("group_id", T::id()).with_fk("compiled.group_reindexings", "group_id"),
            column("index", index()),
            column("source_index", index()),
            column("symbol_id", T::id()).with_fk("compiled.symbols", "symbol_id"),
        ],
        "Complete P5-valid free tuple to evaluated source tuple and actual provider correspondence.",
    );
}
