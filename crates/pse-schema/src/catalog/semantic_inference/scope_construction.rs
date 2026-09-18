// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Scope construction is an ordinary native rule program with exact source lineage.
use super::{N, RegistryBuilder, S, T, assertion, column, provenance, relation};
use crate::model::InvariantKind;
pub(super) fn declare(builder: &mut RegistryBuilder) {
    for (head, assertions) in [
        ("inferred.scope_candidates", "scope_candidate_assertions"),
        ("inferred.selector_contexts", "selector_context_assertions"),
    ] {
        assertion(builder, head, assertions);
    }
    relation(
        builder,
        N::Inferred,
        "selector_parameter_targets",
        S::Derived,
        &["scope_id", "node_id"],
        vec![
            column("scope_id", T::id()),
            column("node_id", T::id()),
            column("target_entity_id", T::id()),
            provenance(),
        ],
        "Actual typed configured instance targets for each relative selector context.",
    );
    assertion(
        builder,
        "inferred.selector_parameter_targets",
        "selector_parameter_target_assertions",
    );
    obligations(builder);
}
fn obligations(builder: &mut RegistryBuilder) {
    for (name, invalid, message) in [
        (
            "relative_target_present",
            "op IN ('self', 'instance_parameter') AND target_entity_id IS NULL",
            "A relative selector requires an actual, uniquely configured instance target.",
        ),
        (
            "kind_has_actual_universe",
            "op = 'kind_is' AND (target_kind IN ('instance', 'port')) IS NOT TRUE",
            "The selected entity kind must have an explicitly admitted finite universe.",
        ),
    ] {
        super::super::inv::declare(
            builder,
            "inferred.selector_contexts",
            name,
            InvariantKind::Check,
            &["scope_id", "node_id"],
            format!("SELECT scope_id, node_id FROM inferred.selector_contexts WHERE {invalid}"),
            &["inferred.selector_contexts"],
            message,
        );
    }
}
