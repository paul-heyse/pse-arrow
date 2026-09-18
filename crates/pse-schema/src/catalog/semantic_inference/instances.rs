// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual instance membership and containment use finite positive relation closure.
use super::{N, RegistryBuilder, S, T, assertion, column, provenance, relation};
pub(super) fn declare(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Inferred,
        "instance_reachability",
        S::Derived,
        &["ancestor_id", "descendant_id"],
        vec![
            column("ancestor_id", T::id()),
            column("descendant_id", T::id()),
            provenance(),
        ],
        "Finite containment closure; P5 validates acyclicity and computes exact tree depths.",
    );
    assertion(builder, "inferred.instances", "instance_assertions");
    assertion(
        builder,
        "inferred.instance_reachability",
        "instance_reachability_assertions",
    );
    guard_checks(builder);
    depth(builder);
}
fn guard_checks(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Inferred,
        "instance_guard_violations",
        S::Derived,
        &["instance_id", "reason"],
        vec![
            column("instance_id", T::id()),
            column("reason", T::native(arrow_schema::DataType::Utf8)),
            provenance(),
        ],
        "Missing, unresolved or malformed scalar child guards refuse publication rather than silently disabling instances.",
    );
    assertion(
        builder,
        "inferred.instance_guard_violations",
        "instance_guard_violation_assertions",
    );
    let keys = ["instance_id", "reason"];
    super::super::inv::declare(
        builder,
        "inferred.instance_guard_violations",
        "child_guards_decided",
        crate::model::InvariantKind::Check,
        &keys,
        "SELECT instance_id, reason FROM inferred.instance_guard_violations",
        &["inferred.instance_guard_violations"],
        "Every eligible child guard must have one complete scalar true/false outcome.",
    );
}
fn depth(builder: &mut RegistryBuilder) {
    assertion(
        builder,
        "inferred.instance_tree",
        "instance_tree_assertions",
    );
    let keys = ["ancestor_id", "descendant_id"];
    super::super::inv::declare(
        builder,
        "inferred.instance_reachability",
        "containment_acyclic",
        crate::model::InvariantKind::Check,
        &keys,
        "SELECT ancestor_id, descendant_id FROM inferred.instance_reachability WHERE ancestor_id = descendant_id",
        &["inferred.instance_reachability"],
        "Containment must be acyclic; physical recycle edges are a separate relation.",
    );
}
