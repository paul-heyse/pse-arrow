// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual material membership is a declared join over finite member inventories.
use super::{Cell, E, P, RegistryBuilder, RuleDecl, RuleHead, assertion, join, project, scan};
use crate::model::{EmptyListPolicy, NullListPolicy};

pub(super) fn declare(builder: &mut RegistryBuilder) {
    assertion(
        builder,
        "inferred.phase_species",
        "phase_species_assertions",
    );
    let explicit = valid_pairs(true);
    let default = valid_pairs(false);
    let candidates = P::Union(vec![explicit, default]);
    let allowed = project(
        join(
            candidates.clone(),
            scan("normalized.phase_species", "restrictions"),
            vec![
                ("phase_id", "restrictions.phase_id"),
                ("species_id", "restrictions.species_id"),
            ],
        ),
        vec![
            ("material_system_id", E::col("material_system_id")),
            ("phase_id", E::col("restrictions.phase_id")),
            ("species_id", E::col("restrictions.species_id")),
        ],
    );
    let unrestricted = P::AntiJoin {
        left: Box::new(candidates),
        right: Box::new(scan("normalized.phase_species", "restrictions")),
        keys: (vec![("phase_id", "restrictions.phase_id")])
            .into_iter()
            .map(|(left, right)| (left.into(), right.into()))
            .collect(),
    };
    let admitted = P::Union(vec![allowed, unrestricted]);
    for (name, henry) in [
        ("P4.phase_species_henry", true),
        ("P4.phase_species_ordinary", false),
    ] {
        let keys = vec![
            ("phase_id", "henry.phase_id"),
            ("species_id", "henry.species_id"),
        ];
        let input = if henry {
            join(
                admitted.clone(),
                scan("normalized.henry_declarations", "henry"),
                keys,
            )
        } else {
            P::AntiJoin {
                left: Box::new(admitted.clone()),
                right: Box::new(scan("normalized.henry_declarations", "henry")),
                keys: (keys)
                    .into_iter()
                    .map(|(left, right)| (left.into(), right.into()))
                    .collect(),
            }
        };
        let plan = project(
            input,
            vec![
                ("material_system_id", E::col("material_system_id")),
                (
                    "phase_id",
                    E::col(if henry { "henry.phase_id" } else { "phase_id" }),
                ),
                (
                    "species_id",
                    E::col(if henry {
                        "henry.species_id"
                    } else {
                        "species_id"
                    }),
                ),
                ("henry", E::Lit(Cell::Bool(henry))),
                ("derivation_id", E::col("material_system_id")),
            ],
        );
        builder.declare_rule(
            RuleDecl::new(
                name,
                "1",
                0,
                RuleHead::Relation("inferred.phase_species".to_owned()),
                plan,
            )
            .assertions("provenance.phase_species_assertions")
            .stratified_negation(),
        );
    }
}
fn members(column: &'static str, output: &'static str, port: &'static str) -> P {
    project(
        P::Unnest {
            input: Box::new(scan("normalized.material_systems", port)),
            column: (column).into(),
            value_name: (output).into(),
            null_list: NullListPolicy::Reject,
            empty_list: EmptyListPolicy::NoMembers,
        },
        vec![
            ("material_system_id", E::col("material_system_id")),
            (output, E::col(output)),
        ],
    )
}
fn valid_pairs(explicit: bool) -> P {
    let phases = join(
        members("phase_ids", "phase_id", "phase_members"),
        scan("normalized.phases", "phases"),
        vec![("phase_id", "phases.phase_id")],
    );
    let phases = project(
        phases,
        vec![
            ("material_system_id", E::col("material_system_id")),
            ("phase_id", E::col("phases.phase_id")),
            ("phase_type", E::col("phases.phase_type")),
        ],
    );
    let species = join(
        members("species_ids", "species_id", "species_members"),
        scan("normalized.species", "species"),
        vec![("species_id", "species.species_id")],
    );
    let species = project(
        species,
        vec![
            ("species_system_id", E::col("material_system_id")),
            ("species_id", E::col("species.species_id")),
            ("valid_phase_types", E::col("species.valid_phase_types")),
        ],
    );
    let species = if explicit {
        P::Unnest {
            input: Box::new(species),
            column: ("valid_phase_types").into(),
            value_name: ("valid_phase_type").into(),
            null_list: NullListPolicy::NoMembers,
            empty_list: EmptyListPolicy::NoMembers,
        }
    } else {
        P::Filter {
            input: Box::new(species),
            predicate: E::IsNull(Box::new(E::col("valid_phase_types"))),
        }
    };
    let mut keys = vec![("material_system_id", "species_system_id")];
    if explicit {
        keys.push(("phase_type", "valid_phase_type"));
    }
    let joined = join(phases, species, keys);
    let joined = if explicit {
        joined
    } else {
        P::Filter {
            input: Box::new(joined),
            predicate: E::cmp(
                crate::model::CmpOp::NotEq,
                E::col("phase_type"),
                E::Lit(Cell::Enum("aqueousPhase")),
            ),
        }
    };
    project(
        joined,
        vec![
            ("material_system_id", E::col("material_system_id")),
            ("phase_id", E::col("phase_id")),
            ("species_id", E::col("species_id")),
        ],
    )
}
