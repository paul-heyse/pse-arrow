// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Ports retain full state collections; existence and membership are relational facts.
use super::{N, RegistryBuilder, S, T, assertion, column, provenance, relation};
pub(super) fn declare(builder: &mut RegistryBuilder) {
    for (head, candidate, assertions) in [
        ("inferred.ports", "port_candidates", "port_assertions"),
        (
            "inferred.port_state_targets",
            "port_state_candidates",
            "port_state_assertions",
        ),
        (
            "inferred.port_state_domains",
            "port_state_domain_candidates",
            "port_state_domain_assertions",
        ),
        (
            "inferred.port_members",
            "port_member_candidates",
            "port_member_assertions",
        ),
        (
            "inferred.port_member_domains",
            "port_member_domain_candidates",
            "port_member_domain_assertions",
        ),
    ] {
        let Some(source) = builder
            .declared_relations()
            .iter()
            .find(|spec| spec.key.qualified_name() == head)
            .cloned()
        else {
            continue;
        };
        let Some(keys) = &source.primary_key else {
            continue;
        };
        let mut columns = source.columns.clone();
        if candidate == "port_candidates" {
            columns.push(column("guard_outcome", T::enumeration("TruthValue")));
            columns.push(provenance());
        }
        relation(
            builder,
            N::Inferred,
            candidate,
            S::Derived,
            keys,
            columns,
            "Replayed prospective port declaration and exact finite state/member context; Native queries decide actual membership.",
        );
        assertion(builder, head, assertions);
    }
    relation(
        builder,
        N::Inferred,
        "unbound_port_targets",
        S::Derived,
        &["port_id", "state_index"],
        vec![
            column("port_id", T::id()),
            column(
                "state_index",
                T::extended(crate::model::ExtensionUse::IndexTuple),
            ),
            column("state_instance_id", T::id()),
            provenance(),
        ],
        "Enabled port target has no actual admitted instance; publication is refused.",
    );
    assertion(
        builder,
        "inferred.unbound_port_targets",
        "unbound_port_assertions",
    );
    let keys = ["port_id", "state_index"];
    super::super::inv::declare(
        builder,
        "inferred.unbound_port_targets",
        "all_state_targets_bound",
        crate::model::InvariantKind::Check,
        &keys,
        "SELECT port_id, state_index FROM inferred.unbound_port_targets",
        &["inferred.unbound_port_targets"],
        "Every enabled port target must be an actual admitted instance.",
    );
}
