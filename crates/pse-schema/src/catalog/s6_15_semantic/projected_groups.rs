// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact original/member/domain correspondence of a projected group (ADR-0063).
use super::{N, RegistryBuilder, T, column, derived};

pub(super) fn declare(builder: &mut RegistryBuilder) {
    derived(
        builder,
        N::Compiled,
        "group_projections",
        &["group_id"],
        vec![
            column("group_id", T::id()).with_fk("compiled.symbol_groups", "group_id"),
            column("source_group_id", T::id()).with_fk("compiled.symbol_groups", "group_id"),
            column(
                "fixed_coordinates",
                T::list(T::structure(vec![
                    T::nonnegative(i64::from(u16::MAX))
                        .with_name("axis")
                        .with_nullable(false),
                    T::id().with_name("member_id").with_nullable(false),
                ])),
            ),
            column("product_id", T::id()).with_fk("normalized.domain_products", "product_id"),
        ],
        "Exact source-group restriction on ordered fixed members; remaining factors and complete projected tuples must agree with actual group rows.",
    );
}
