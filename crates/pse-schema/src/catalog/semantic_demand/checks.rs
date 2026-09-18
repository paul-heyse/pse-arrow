// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native queries require complete actual tuples and settled demanded winners.
use super::super::inv::{columns, declare as invariant, table};
use super::RegistryBuilder;
use crate::model::InvariantKind;
pub(super) fn declare(builder: &mut RegistryBuilder) {
    guard_contracts(builder, "inferred.demand_read_keys");
    for (relation, target, name, keys, doc) in [
        (
            "inferred.demand_scope_requests",
            "inferred.demand_obligations",
            "actual_scope_binding",
            &["read_id", "property_kind_id", "scope_decl_id"][..],
            "Each active read scope is resolved from its actual relative owner or explicit global declaration.",
        ),
        (
            "inferred.demand_obligations",
            "inferred.demand_seed_bindings",
            "actual_demand_target",
            &["read_id", "property_kind_id", "scope_id"][..],
            "An active declared property read requires actual eligible state and complete member correspondence.",
        ),
    ] {
        invariant(
            builder,
            relation,
            name,
            InvariantKind::Check,
            keys,
            format!(
                "SELECT {} FROM {} s WHERE NOT EXISTS (SELECT 1 FROM {} t WHERE t.read_id = s.read_id AND t.property_kind_id = s.property_kind_id)",
                columns(keys, "s"),
                table(relation),
                table(target),
            ),
            &[relation, target],
            doc,
        );
    }
    invariant(
        builder,
        "inferred.method_resolutions",
        "demanded_method_resolved",
        InvariantKind::Check,
        &["requirement_id"],
        "SELECT requirement_id FROM inferred.method_resolutions WHERE (outcome.kind = 'resolved') IS NOT TRUE",
        &["inferred.method_resolutions"],
        "Every demanded property has exactly one greatest-ranked compatible actual method.",
    );
}
pub(super) fn guard_contracts(builder: &mut RegistryBuilder, relation: &'static str) {
    let source = table(relation);
    for (present, tuple, missing, conflict, missing_doc, conflict_doc) in [
        (
            "s.guard_source_id IS NOT NULL",
            guard_tuple(false),
            "actual_guard_tuple",
            "guard_not_conflict",
            "A guarded read requires its exact source-local predicate outcome tuple.",
            "Conflicting actual guard evidence is not an inactive branch.",
        ),
        (
            "s.outer_guard_source_id IS NOT NULL",
            guard_tuple(true),
            "actual_outer_guard",
            "outer_guard_not_conflict",
            "An outer declaration guard requires its actual scalar predicate outcome.",
            "Conflicting declaration guard evidence cannot silently suppress a demand.",
        ),
    ] {
        invariant(
            builder,
            relation,
            missing,
            InvariantKind::Check,
            &["read_id"],
            format!(
                "SELECT s.read_id FROM {source} s WHERE {present} AND NOT EXISTS (SELECT 1 FROM inferred.predicate_outcomes o WHERE {tuple})",
            ),
            &[relation, "inferred.predicate_outcomes"],
            missing_doc,
        );
        invariant(
            builder,
            relation,
            conflict,
            InvariantKind::Check,
            &["read_id"],
            format!(
                "SELECT s.read_id FROM {source} s WHERE {present} AND EXISTS (SELECT 1 FROM inferred.predicate_outcomes o WHERE {tuple} AND o.outcome = 'conflict')",
            ),
            &[relation, "inferred.predicate_outcomes"],
            conflict_doc,
        );
    }
}
fn guard_tuple(outer: bool) -> &'static str {
    if outer {
        "s.requester_instance_id = o.instance_id AND s.outer_guard_source_id = o.source_id AND s.outer_guard_node_id = o.predicate_id AND array_length(o.index) = 0"
    } else {
        "s.requester_instance_id = o.instance_id AND s.guard_source_id = o.source_id AND s.guard_node_id = o.predicate_id AND s.guard_index = o.index"
    }
}
/// Native active-read predicate shared by coordinate admission and demand planning.
pub(in super::super) fn active_predicate() -> String {
    let guard = guard_tuple(false);
    let outer = guard_tuple(true);
    format!(
        "((s.guard_source_id IS NULL AND s.guard_node_id IS NULL)
        OR EXISTS (SELECT 1 FROM inferred.predicate_outcomes o WHERE {guard} AND o.outcome IN ('true', 'unknown')))
        AND ((s.outer_guard_source_id IS NULL AND s.outer_guard_node_id IS NULL)
        OR EXISTS (SELECT 1 FROM inferred.predicate_outcomes o WHERE {outer} AND o.outcome IN ('true', 'unknown')))",
    )
}
