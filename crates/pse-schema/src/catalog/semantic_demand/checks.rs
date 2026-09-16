// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Missing actual tuple evidence and unresolved demanded winners fail the pass explicitly.
use super::{Cell, E, P, RegistryBuilder, anti, eq, filter, join, literal, present, project, scan};
use crate::model::InvariantKind;

pub(super) fn declare(builder: &mut RegistryBuilder) {
    guard_contracts(builder, "inferred.demand_read_keys");
    let missing = anti(
        scan("inferred.demand_scope_requests", "requests"),
        scan("inferred.demand_obligations", "obligations"),
        vec![
            ("requests.read_id", "obligations.read_id"),
            ("requests.property_kind_id", "obligations.property_kind_id"),
        ],
    );
    check(
        builder,
        "inferred.demand_scope_requests",
        "actual_scope_binding",
        &["read_id", "property_kind_id", "scope_decl_id"],
        project(
            missing,
            vec![
                ("read_id", E::col("requests.read_id")),
                ("property_kind_id", E::col("requests.property_kind_id")),
                ("scope_decl_id", E::col("requests.scope_decl_id")),
            ],
        ),
        "Each active read scope is resolved from its actual relative owner or explicit global declaration.",
    );
    let missing = anti(
        scan("inferred.demand_obligations", "obligations"),
        scan("inferred.demand_seed_bindings", "bound"),
        vec![
            ("obligations.read_id", "bound.read_id"),
            ("obligations.property_kind_id", "bound.property_kind_id"),
        ],
    );
    check(
        builder,
        "inferred.demand_obligations",
        "actual_demand_target",
        &["read_id", "property_kind_id", "scope_id"],
        project(
            missing,
            vec![
                ("read_id", E::col("obligations.read_id")),
                ("property_kind_id", E::col("obligations.property_kind_id")),
                ("scope_id", E::col("obligations.scope_id")),
            ],
        ),
        "An active declared property read requires actual eligible state and complete member correspondence.",
    );
    let unresolved = filter(
        scan("inferred.method_resolutions", "resolved"),
        E::Not(Box::new(eq(E::col("status"), literal("resolved")))),
    );
    check(
        builder,
        "inferred.method_resolutions",
        "demanded_method_resolved",
        &["requirement_id"],
        project(
            unresolved,
            vec![("requirement_id", E::col("requirement_id"))],
        ),
        "Every demanded property has exactly one greatest-ranked compatible actual method.",
    );
}
pub(super) fn guard_contracts(builder: &mut RegistryBuilder, relation: &'static str) {
    outer_guards(builder, relation);
    let guarded = filter(scan(relation, "reads"), present("guard_source_id"));
    let missing = anti(
        guarded.clone(),
        scan("inferred.predicate_outcomes", "outcomes"),
        guard_keys(),
    );
    check(
        builder,
        relation,
        "actual_guard_tuple",
        &["read_id"],
        project(missing, vec![("read_id", E::col("reads.read_id"))]),
        "A guarded read requires its exact source-local predicate outcome tuple.",
    );
    let conflict = filter(
        join(
            guarded,
            scan("inferred.predicate_outcomes", "outcomes"),
            guard_keys(),
        ),
        eq(E::col("outcomes.outcome"), literal("conflict")),
    );
    check(
        builder,
        relation,
        "guard_not_conflict",
        &["read_id"],
        project(conflict, vec![("read_id", E::col("reads.read_id"))]),
        "Conflicting actual guard evidence is not an inactive branch.",
    );
}
fn guard_keys() -> Vec<(&'static str, &'static str)> {
    vec![
        ("reads.requester_instance_id", "outcomes.instance_id"),
        ("reads.guard_source_id", "outcomes.source_id"),
        ("reads.guard_node_id", "outcomes.predicate_id"),
        ("reads.guard_index", "outcomes.index"),
    ]
}
fn check(
    builder: &mut RegistryBuilder,
    relation: &str,
    name: &str,
    keys: &[&'static str],
    input: P,
    doc: &'static str,
) {
    super::super::inv::declare(
        builder,
        relation,
        name,
        InvariantKind::Check,
        keys,
        input,
        doc,
    );
}

fn outer_guards(builder: &mut RegistryBuilder, relation: &'static str) {
    let reads = filter(scan(relation, "reads"), present("outer_guard_source_id"));
    let outcomes = filter(
        scan("inferred.predicate_outcomes", "outcomes"),
        eq(E::ListLen(Box::new(E::col("index"))), E::Lit(Cell::U64(0))),
    );
    let keys = vec![
        ("reads.requester_instance_id", "outcomes.instance_id"),
        ("reads.outer_guard_source_id", "outcomes.source_id"),
        ("reads.outer_guard_node_id", "outcomes.predicate_id"),
    ];
    let missing = anti(reads.clone(), outcomes.clone(), keys.clone());
    check(
        builder,
        relation,
        "actual_outer_guard",
        &["read_id"],
        project(missing, vec![("read_id", E::col("reads.read_id"))]),
        "An outer declaration guard requires its actual scalar predicate outcome.",
    );
    let conflict = filter(
        join(reads, outcomes, keys),
        eq(E::col("outcomes.outcome"), literal("conflict")),
    );
    check(
        builder,
        relation,
        "outer_guard_not_conflict",
        &["read_id"],
        project(conflict, vec![("read_id", E::col("reads.read_id"))]),
        "Conflicting declaration guard evidence cannot silently suppress a demand.",
    );
}
