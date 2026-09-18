// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Connection admission and actual cut sets are declared relational operators.
use super::{N, RegistryBuilder, S, T, assertion, column, provenance, relation};
pub(super) fn declare(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Inferred,
        "connection_violations",
        S::Derived,
        &["connection_id", "reason"],
        vec![
            column("connection_id", T::id()),
            column("reason", T::native(arrow_schema::DataType::Utf8)),
            provenance(),
        ],
        "Exact missing endpoint or incompatible member contracts prevent topology publication.",
    );
    relation(
        builder,
        N::Inferred,
        "scope_port_states",
        S::Derived,
        &["scope_id", "port_id", "state_index"],
        vec![
            column("scope_id", T::id()),
            column("port_id", T::id()),
            column(
                "state_index",
                T::extended(crate::model::ExtensionUse::IndexTuple),
            ),
            provenance(),
        ],
        "Actual state tuples included by their state, owning instance, or explicit port membership.",
    );
    relation(
        builder,
        N::Inferred,
        "scope_port_decisions",
        S::Derived,
        &["scope_id", "port_id", "state_index"],
        vec![
            column("scope_id", T::id()),
            column("port_id", T::id()),
            column(
                "state_index",
                T::extended(crate::model::ExtensionUse::IndexTuple),
            ),
            column("included", T::native(arrow_schema::DataType::Boolean)),
            provenance(),
        ],
        "Complete endpoint state membership, retaining false as explicit positive data.",
    );
    for (head, name) in [
        (
            "inferred.connection_violations",
            "connection_violation_assertions",
        ),
        ("inferred.topology_edges", "topology_edge_assertions"),
        ("inferred.scope_port_states", "scope_port_state_assertions"),
        (
            "inferred.scope_port_decisions",
            "scope_port_decision_assertions",
        ),
        (
            "inferred.boundary_crossings",
            "boundary_crossing_assertions",
        ),
    ] {
        assertion(builder, head, name);
    }
    let keys = ["connection_id", "reason"];
    super::super::inv::declare(
        builder,
        "inferred.connection_violations",
        "connections_admitted",
        crate::model::InvariantKind::Check,
        &keys,
        "SELECT connection_id, reason FROM inferred.connection_violations",
        &["inferred.connection_violations"],
        "All connection endpoints, domains, ordered members and full physical contracts must be compatible.",
    );
}
