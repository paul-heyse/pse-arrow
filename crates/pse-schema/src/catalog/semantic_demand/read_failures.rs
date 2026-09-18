// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! An absent actual coordinate is a violation only when its exact read is active.
use super::{N, RegistryBuilder, S, T, checks, column, index, provenance, relation};
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
            column("read_node_id", T::nonnegative(i64::MAX)).optional(),
            column("bound_index_ids", index()),
            column("source_index", index()),
            column("guard_source_id", T::id()).optional(),
            column("guard_node_id", T::nonnegative(i64::MAX)).optional(),
            column("guard_index", index()),
            column("outer_guard_source_id", T::id()).optional(),
            column("outer_guard_node_id", T::nonnegative(i64::MAX)).optional(),
            provenance(),
        ],
        "Exact source coordinate has no actual target member; this structural outcome makes no guard truth claim.",
    );
    super::super::inv::declare(
        builder,
        "inferred.read_coordinate_failures",
        "active_read_coordinate",
        crate::model::InvariantKind::Check,
        &["read_id"],
        format!(
            "SELECT s.read_id FROM inferred.read_coordinate_failures s WHERE {}",
            checks::active_predicate(),
        ),
        &[
            "inferred.read_coordinate_failures",
            "inferred.predicate_outcomes",
        ],
        "Every active or unknown property read resolves to its actual declared member; false guarded reads need no target.",
    );
    checks::guard_contracts(builder, "inferred.read_coordinate_failures");
}
