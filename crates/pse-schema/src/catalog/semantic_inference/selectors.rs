// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Structural selector lowering preserves every ordered source child.
use super::{N, RegistryBuilder, S, T, column, provenance, relation};
#[expect(
    clippy::too_many_lines,
    reason = "one declarative catalog family keeps its native rules and field declarations together"
)]
pub(super) fn declare(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Normalized,
        "instance_binding_products",
        S::Derived,
        &["instance_id"],
        vec![
            column("instance_id", T::id()),
            column("product_id", T::id()),
            column("index", T::extended(crate::model::ExtensionUse::IndexTuple)),
            provenance(),
        ],
        "Exact prospective child multiplicity product and tuple; P5 admits the instance only after tuple validity.",
    );
    relation(
        builder,
        N::Normalized,
        "domain_product_projections",
        S::Derived,
        &["projection_id"],
        vec![
            column("projection_id", T::id()),
            column("product_id", T::id()),
            column("parent_product_id", T::id()),
            column("positions", T::list(T::nonnegative(i64::from(u16::MAX)))),
            provenance(),
        ],
        "Exact ordered parent-factor positions for each prospective subset product, including repeated factor identities.",
    );
    relation(
        builder,
        N::Normalized,
        "candidate_index_tuples",
        S::Derived,
        &["product_id", "tuple"],
        vec![
            column("product_id", T::id()),
            column("tuple", T::extended(crate::model::ExtensionUse::IndexTuple)),
            provenance(),
        ],
        "P3 finite Cartesian candidates; this declares no phase/species or guard validity.",
    );
    relation(
        builder,
        N::Normalized,
        "candidate_index_members",
        S::Derived,
        &["product_id", "tuple", "position"],
        vec![
            column("product_id", T::id()),
            column("tuple", T::extended(crate::model::ExtensionUse::IndexTuple)),
            column("position", T::nonnegative(i64::from(u16::MAX))),
            column("domain_id", T::id()),
            column("member_id", T::id()),
            provenance(),
        ],
        "Exact ordered Cartesian tuple-to-domain/member correspondence.",
    );
    relation(
        builder,
        N::Normalized,
        "domain_product_sources",
        S::Derived,
        &["product_id", "source_relation_id", "source_key"],
        vec![
            column("product_id", T::id()),
            column("source_relation_id", T::id()),
            column("source_key", T::row_key()),
            provenance(),
        ],
        "Actual declaration and binding rows behind each finite prospective ordered product.",
    );
    super::super::declarations::enumeration(
        builder,
        "SelectorNodeOp",
        [
            "constant",
            "identity",
            "union",
            "intersection",
            "difference",
            "include",
            "exclude",
            "self",
            "kind_is",
            "descendant_of",
            "instance_parameter",
        ],
    );
    relation(
        builder,
        N::Normalized,
        "selector_nodes",
        S::Derived,
        &["node_id"],
        vec![
            column("node_id", T::id()),
            column("source_term_id", T::id()),
            column("scope_decl_id", T::id()),
            column("op", T::enumeration("SelectorNodeOp")),
            column("left_node_id", T::id()).optional(),
            column("right_node_id", T::id()).optional(),
            column("entity_id", T::id()).optional(),
            column("entity_kind", T::enumeration("EntityKind")).optional(),
            column("parameter_name", T::native(arrow_schema::DataType::Utf8)).optional(),
            column("constant", T::native(arrow_schema::DataType::Boolean)).optional(),
            column("fold_position", T::nonnegative(i64::from(u16::MAX))).optional(),
            provenance(),
        ],
        "P3 binary selector witnesses, including exact source correspondence; no set membership is decided here.",
    );
    relation(
        builder,
        N::Normalized,
        "selector_roots",
        S::Derived,
        &["scope_decl_id"],
        vec![
            column("scope_decl_id", T::id()),
            column("node_id", T::id()),
            provenance(),
        ],
        "Actual declaration root after structural selector lowering.",
    );
    relation(
        builder,
        N::Normalized,
        "selector_children",
        S::Derived,
        &["source_term_id", "position"],
        vec![
            column("source_term_id", T::id()),
            column("position", T::nonnegative(i64::from(u16::MAX))),
            column("source_ordinal", T::nonnegative(i64::from(u16::MAX))),
            column("child_term_id", T::id()),
            provenance(),
        ],
        "Every original selector child is retained in actual declared ordinal order.",
    );
}
