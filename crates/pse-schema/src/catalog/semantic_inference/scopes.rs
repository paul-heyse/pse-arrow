// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete positive Boolean selector decisions admit arbitrary nested difference.
use super::{N, RegistryBuilder, S, T, assertion, column, provenance, relation};
#[expect(
    clippy::too_many_lines,
    reason = "one declarative catalog family keeps its native rules and field declarations together"
)]
pub(super) fn declare(builder: &mut RegistryBuilder) {
    let context_columns = || {
        vec![
            column("scope_id", T::id()),
            column("scope_decl_id", T::id()),
            column("owner_instance_id", T::id()).optional(),
            provenance(),
        ]
    };
    relation(
        builder,
        N::Inferred,
        "scope_candidates",
        S::Derived,
        &["scope_id"],
        context_columns(),
        "Replayed finite scope identity construction from actual declarations and prospective instances.",
    );
    relation(
        builder,
        N::Inferred,
        "resolved_scopes",
        S::Derived,
        &["scope_id"],
        context_columns(),
        "Actual global or instance-bound scope contexts.",
    );
    relation(
        builder,
        N::Inferred,
        "selector_contexts",
        S::Derived,
        &["scope_id", "node_id"],
        vec![
            column("scope_id", T::id()),
            column("node_id", T::id()),
            column("op", T::enumeration("SelectorNodeOp")),
            column("left_node_id", T::id()).optional(),
            column("right_node_id", T::id()).optional(),
            column("target_entity_id", T::id()).optional(),
            column("target_kind", T::enumeration("EntityKind")).optional(),
            column("constant", T::native(arrow_schema::DataType::Boolean)).optional(),
            provenance(),
        ],
        "Replayed selector structural contexts; relative parameter leaves bind exact typed configured identities.",
    );
    relation(
        builder,
        N::Inferred,
        "scope_entities",
        S::Derived,
        &["entity_id"],
        vec![
            column("entity_id", T::id()),
            column("kind", T::enumeration("EntityKind")),
            provenance(),
        ],
        "Complete actual instance/port universe at the topology boundary.",
    );
    relation(
        builder,
        N::Inferred,
        "scope_reachability",
        S::Derived,
        &["ancestor_id", "entity_id"],
        vec![
            column("ancestor_id", T::id()),
            column("entity_id", T::id()),
            provenance(),
        ],
        "Actual containment of instances and their ports, separate from physical flow.",
    );
    relation(
        builder,
        N::Inferred,
        "selector_decisions",
        S::Derived,
        &["scope_id", "node_id", "entity_id"],
        vec![
            column("scope_id", T::id()),
            column("node_id", T::id()),
            column("entity_id", T::id()),
            column("included", T::native(arrow_schema::DataType::Boolean)),
            provenance(),
        ],
        "Complete Boolean membership facts; false is positive data, so nested set difference remains monotone.",
    );
    for (head, assertion_name) in [
        ("inferred.resolved_scopes", "scope_assertions"),
        ("inferred.scope_bindings", "scope_binding_assertions"),
        ("inferred.scope_entities", "scope_entity_assertions"),
        (
            "inferred.scope_reachability",
            "scope_reachability_assertions",
        ),
        (
            "inferred.selector_decisions",
            "selector_decision_assertions",
        ),
        ("inferred.scope_members", "scope_member_assertions"),
    ] {
        assertion(builder, head, assertion_name);
    }
}
