// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native package/material closure and finite parent-cycle queries.
use super::inv::{declare as invariant, identifier, table};
use crate::{RegistryBuilder, model::InvariantKind};
pub(super) fn declare(builder: &mut RegistryBuilder) {
    for (list, target, key, name) in [
        (
            "species_ids",
            "authored.species",
            "species_id",
            "closure:material_system_species_exist",
        ),
        (
            "phase_ids",
            "authored.phases",
            "phase_id",
            "closure:material_system_phases_exist",
        ),
    ] {
        invariant(
            builder,
            "authored.material_systems",
            name,
            InvariantKind::Closure,
            &["material_system_id"],
            format!(
                "SELECT DISTINCT s.material_system_id FROM (SELECT material_system_id, unnest({list}) AS member FROM authored.material_systems) s WHERE NOT EXISTS (SELECT 1 FROM {target} t WHERE t.{key} = s.member)"
            ),
            &["authored.material_systems", target],
            "Every explicitly listed material-system member exists in its declared relation.",
        );
    }
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
    invariant(
        builder,
        "authored.stoichiometry",
        "closure:stoichiometry_species_in_phase",
        InvariantKind::Closure,
        &["reaction_id", "phase_id", "species_id"],
        r"SELECT DISTINCT s.reaction_id, s.phase_id, s.species_id
            FROM authored.stoichiometry s
            LEFT JOIN authored.species sp ON sp.species_id = s.species_id
            LEFT JOIN authored.phases p ON p.phase_id = s.phase_id
            LEFT JOIN (SELECT DISTINCT phase_id FROM authored.phase_species) restricted
                ON restricted.phase_id = s.phase_id
            LEFT JOIN authored.phase_species allowed
                ON allowed.phase_id = s.phase_id AND allowed.species_id = s.species_id
            WHERE sp.species_id IS NULL OR p.phase_id IS NULL
                OR ((sp.valid_phase_types IS NULL AND p.phase_type <> 'aqueous')
                    OR array_has(sp.valid_phase_types, p.phase_type)) IS NOT TRUE
                OR (restricted.phase_id IS NOT NULL AND allowed.species_id IS NULL)",
        &[
            "authored.stoichiometry",
            "authored.species",
            "authored.phases",
            "authored.phase_species",
        ],
        "Stoichiometric species satisfy declared phase types, the explicit null default, and any per-phase species restriction.",
    );
    for (relation, identity, parent) in [
        ("authored.entities", "entity_id", "parent_entity_id"),
        ("authored.instances", "instance_id", "parent_instance_id"),
        ("authored.cases", "case_id", "parent_case_id"),
    ] {
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
