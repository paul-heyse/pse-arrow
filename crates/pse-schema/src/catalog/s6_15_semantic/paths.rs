// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Source-relative instance paths and their exact finite realized collections.

use super::{N, RegistryBuilder, T, column, derived, enumeration, index};

pub(super) fn declare(builder: &mut RegistryBuilder) {
    enumeration(
        builder,
        "ExpressionPathSegmentKind",
        crate::model::ExpressionPathSegmentKind::ALL
            .map(crate::model::ExpressionPathSegmentKind::as_str),
    );
    enumeration(
        builder,
        "PathTargetKind",
        ["symbol", "parameter", "feature", "port"],
    );
    path_targets(builder);
    derived(
        builder,
        N::Normalized,
        "expression_paths",
        &["source_id", "path_id"],
        vec![
            column("source_id", T::id()).with_fk("normalized.expression_sources", "source_id"),
            column("path_id", T::native(arrow_schema::DataType::UInt64)),
            column("root_instance_id", T::id()).optional(),
            column(
                "segments",
                T::list(T::structure(vec![
                    T::enumeration("ExpressionPathSegmentKind")
                        .with_name("kind")
                        .with_nullable(false),
                    T::native(arrow_schema::DataType::Utf8)
                        .with_name("name")
                        .with_nullable(false),
                    T::native(arrow_schema::DataType::UInt16)
                        .with_name("index_count")
                        .with_nullable(false),
                ])),
            ),
            column("path_start", T::native(arrow_schema::DataType::UInt32)),
            column("path_end", T::native(arrow_schema::DataType::UInt32)),
        ],
        "Exact source-relative child/member keys and index partitions. Each child key is resolved under the actual selected parent template, never under one globally substituted template. Path byte offsets address the decoded source DSL string.",
    );
    derived(
        builder,
        N::Compiled,
        "group_collections",
        &["group_id"],
        vec![
            column("group_id", T::id()).with_fk("compiled.symbol_groups", "group_id"),
            column("root_instance_id", T::id()),
            column("source_id", T::id()).with_fk("normalized.expression_sources", "source_id"),
            column("path_id", T::native(arrow_schema::DataType::UInt64)),
            column("product_id", T::id()).with_fk("normalized.domain_products", "product_id"),
            column("quantity_type_id", T::id())
                .with_fk("reference.quantity_types", "quantity_type_id"),
        ],
        "Finite group collected through one actual instance path. Ordered factors and every scalar member's complete physical type are checked against actual providers.",
    );
    derived(
        builder,
        N::Compiled,
        "group_collection_members",
        &["group_id", "index"],
        vec![
            column("group_id", T::id()).with_fk("compiled.group_collections", "group_id"),
            column("index", index()),
            column("symbol_id", T::id()).with_fk("compiled.symbols", "symbol_id"),
            column("owner_instance_id", T::id()),
            column("symbol_decl_id", T::id()),
            column("source_index", index()),
        ],
        "Complete reversible collected tuple to actual owner/declaration/member correspondence; hashes and matching labels cannot prove this mapping.",
    );
}

fn path_targets(builder: &mut RegistryBuilder) {
    derived(
        builder,
        N::Inferred,
        "path_targets",
        &[
            "requester_instance_id",
            "source_id",
            "path_id",
            "path_index",
        ],
        vec![
            column("requester_instance_id", T::id()),
            column("source_id", T::id()).with_fk("normalized.expression_sources", "source_id"),
            column("path_id", T::native(arrow_schema::DataType::UInt64)),
            column("path_index", index()),
            column("path_domains", T::list(T::id())),
            column("owner_instance_id", T::id()),
            column("target_kind", T::enumeration("PathTargetKind")),
            column("symbol_decl_id", T::id())
                .optional()
                .with_fk("authored.template_symbols", "symbol_decl_id"),
            column("name", T::native(arrow_schema::DataType::Utf8)).optional(),
            column("source_index", index()),
        ],
        "Exact finite path binding before method selection. Symbol targets carry declaration IDs; parameter/feature/port targets carry the actual selected owner's composite member key. Full actual template, domain and member traversal establishes validity.",
    );
}
