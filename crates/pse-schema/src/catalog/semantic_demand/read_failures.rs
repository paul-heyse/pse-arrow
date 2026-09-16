// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! An absent actual coordinate is a violation only when its exact read is active.
use super::{
    E, N, RegistryBuilder, S, T, checks, closure, column, index, project, provenance, relation,
};
pub(super) fn declare(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Inferred,
        "read_coordinate_failures",
        S::Derived,
        &["read_id"],
        vec![
            column("read_id", T::id()),
            column("seed_id", T::id()),
            column("requester_instance_id", T::id()),
            column("source_id", T::id()),
            column("read_node_id", T::native(arrow_schema::DataType::UInt64)).optional(),
            column("bound_index_ids", index()),
            column("source_index", index()),
            column("guard_source_id", T::id()).optional(),
            column("guard_node_id", T::native(arrow_schema::DataType::UInt64)).optional(),
            column("guard_index", index()),
            column("outer_guard_source_id", T::id()).optional(),
            column(
                "outer_guard_node_id",
                T::native(arrow_schema::DataType::UInt64),
            )
            .optional(),
            provenance(),
        ],
        "Exact source coordinate has no actual target member; this structural outcome makes no guard truth claim.",
    );
    let names = [
        "read_id",
        "seed_id",
        "requester_instance_id",
        "source_id",
        "read_node_id",
        "bound_index_ids",
        "source_index",
        "guard_source_id",
        "guard_node_id",
        "guard_index",
        "outer_guard_source_id",
        "outer_guard_node_id",
        "derivation_id",
    ];
    let qualified = [
        "reads.read_id",
        "reads.seed_id",
        "reads.requester_instance_id",
        "reads.source_id",
        "reads.read_node_id",
        "reads.bound_index_ids",
        "reads.source_index",
        "reads.guard_source_id",
        "reads.guard_node_id",
        "reads.guard_index",
        "reads.outer_guard_source_id",
        "reads.outer_guard_node_id",
        "reads.derivation_id",
    ];
    let active =
        closure::seeds::active_input("inferred.read_coordinate_failures", &names, &qualified);
    super::super::inv::declare(
        builder,
        "inferred.read_coordinate_failures",
        "active_read_coordinate",
        crate::model::InvariantKind::Check,
        &["read_id"],
        project(active, vec![("read_id", E::col("read_id"))]),
        "Every active or unknown property read resolves to its actual declared member; false guarded reads need no target.",
    );
    checks::guard_contracts(builder, "inferred.read_coordinate_failures");
}
