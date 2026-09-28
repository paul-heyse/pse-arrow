// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native package/material closure and finite parent-cycle queries.
use super::inv::{declare as invariant, identifier, table};
use crate::{RegistryBuilder, model::InvariantKind};
pub(super) fn declare(builder: &mut RegistryBuilder) {
    invariant(
        builder,
        "authored.packages",
        "closure:packages.dependencies_resolved",
        InvariantKind::Closure,
        &["package_id"],
        r"SELECT DISTINCT s.package_id FROM
            (SELECT package_id, dependency.package_id AS target_package,
                regexp_replace(dependency.version_req, '^=', '') AS target_version
                FROM (SELECT package_id, unnest(dependencies) AS dependency FROM authored.packages)) s
            WHERE NOT EXISTS (SELECT 1 FROM authored.packages p WHERE p.package_id = s.target_package
                AND p.version = s.target_version)",
        &["authored.packages"],
        "Every package dependency resolves its actual identity and exact version; ranges are not phase-0 bindings.",
    );
    for (relation, identity, parent) in [("authored.entities", "entity_id", "parent_entity_id")] {
        let source = table(relation);
        let key = identifier(identity);
        let parent = identifier(parent);
        invariant(builder, relation, "acyclic:parents", InvariantKind::Acyclic, &[identity],
            format!("WITH RECURSIVE ancestors(origin, next) AS (
                SELECT {key}, {parent} FROM {source} WHERE {parent} IS NOT NULL
                UNION SELECT a.origin, s.{parent} FROM ancestors a JOIN {source} s ON a.next = s.{key}
                    WHERE s.{parent} IS NOT NULL)
                SELECT DISTINCT origin AS {key} FROM ancestors WHERE origin = next"),
            &[relation], "Actual parent edges cannot return to the same identity; distinct finite relation pairs bound the recursive closure.");
    }
}
