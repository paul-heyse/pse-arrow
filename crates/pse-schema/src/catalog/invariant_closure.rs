// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit package/material closure and finite parent-cycle rules (blueprint §6, §14.2).
use super::inv::{declare as invariant, filter, project, scan};
use crate::RegistryBuilder;
use crate::model::{
    Cell, CmpOp, DepthBound, EmptyListPolicy, InvariantKind, NullEquality, NullListPolicy,
    RuleExpr, RulePlan,
};

pub(super) fn declare(builder: &mut RegistryBuilder) {
    material_members(builder);
    package_dependencies(builder);
    stoichiometry(builder);
    for (relation, identity, parent) in [
        ("authored.entities", "entity_id", "parent_entity_id"),
        ("authored.instances", "instance_id", "parent_instance_id"),
        ("authored.cases", "case_id", "parent_case_id"),
        (
            "authored.model_revisions",
            "model_revision_id",
            "parent_revision_id",
        ),
    ] {
        acyclic(builder, relation, identity, parent);
    }
}
fn unnest(input: RulePlan, column: &'static str, value_name: &'static str) -> RulePlan {
    RulePlan::Unnest {
        input: Box::new(input),
        column: (column).into(),
        value_name: (value_name).into(),
        null_list: NullListPolicy::NoMembers,
        empty_list: EmptyListPolicy::NoMembers,
    }
}
fn join(left: RulePlan, right: RulePlan, keys: Vec<(&'static str, &'static str)>) -> RulePlan {
    RulePlan::EquiJoin {
        left: Box::new(left),
        right: Box::new(right),
        keys: (keys)
            .into_iter()
            .map(|(left, right)| (left.into(), right.into()))
            .collect(),
        null_equality: NullEquality::NullEqualsNothing,
    }
}
fn anti(left: RulePlan, right: RulePlan, keys: Vec<(&'static str, &'static str)>) -> RulePlan {
    RulePlan::AntiJoin {
        left: Box::new(left),
        right: Box::new(right),
        keys: (keys)
            .into_iter()
            .map(|(left, right)| (left.into(), right.into()))
            .collect(),
    }
}
fn renamed(input: RulePlan, columns: &[(&'static str, &'static str)]) -> RulePlan {
    RulePlan::Project {
        input: Box::new(input),
        columns: (columns
            .iter()
            .map(|(to, from)| (*to, RuleExpr::col(*from)))
            .collect::<Vec<_>>())
        .into_iter()
        .map(|(name, expression)| (name.to_owned().into(), expression))
        .collect(),
    }
}
fn material_members(builder: &mut RegistryBuilder) {
    for (list, target, target_key, name) in [
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
        let missing = anti(
            unnest(
                scan("authored.material_systems", "subject"),
                list,
                "__member",
            ),
            scan(target, "members"),
            vec![("__member", target_key)],
        );
        invariant(
            builder,
            "authored.material_systems",
            name,
            InvariantKind::Closure,
            &["material_system_id"],
            RulePlan::Distinct(Box::new(project(missing, &["material_system_id"]))),
            "Every explicitly listed material-system member exists in its declared relation.",
        );
    }
}
fn package_dependencies(builder: &mut RegistryBuilder) {
    let dependencies = RulePlan::Project {
        input: Box::new(unnest(
            scan("authored.packages", "subject"),
            "dependencies",
            "__dependency",
        )),
        columns: (vec![
            ("package_id", RuleExpr::col("package_id")),
            (
                "__package",
                RuleExpr::Field {
                    expr: Box::new(RuleExpr::col("__dependency")),
                    name: "package_id".into(),
                },
            ),
            (
                "__version",
                RuleExpr::call(
                    "regexp_replace",
                    vec![
                        RuleExpr::Field {
                            expr: Box::new(RuleExpr::col("__dependency")),
                            name: "version_req".into(),
                        },
                        RuleExpr::Lit(Cell::text("^=")),
                        RuleExpr::Lit(Cell::text("")),
                    ],
                    crate::model::FieldContract::native(arrow_schema::DataType::Utf8),
                    true,
                ),
            ),
        ])
        .into_iter()
        .map(|(name, expression)| (name.to_owned().into(), expression))
        .collect(),
    };
    let target = renamed(
        scan("authored.packages", "packages"),
        &[
            ("__target_package", "package_id"),
            ("__target_version", "version"),
        ],
    );
    let missing = anti(
        dependencies,
        target,
        vec![
            ("__package", "__target_package"),
            ("__version", "__target_version"),
        ],
    );
    invariant(
        builder,
        "authored.packages",
        "closure:packages.dependencies_resolved",
        InvariantKind::Closure,
        &["package_id"],
        RulePlan::Distinct(Box::new(project(missing, &["package_id"]))),
        "Every package dependency resolves its actual identity and exact version; ranges are not phase-0 bindings.",
    );
}
fn acyclic(
    builder: &mut RegistryBuilder,
    relation: &'static str,
    identity: &'static str,
    parent: &'static str,
) {
    let edges = filter(
        scan(relation, "subject"),
        RuleExpr::IsNotNull(Box::new(RuleExpr::col(parent))),
    );
    let seed = renamed(edges.clone(), &[("__origin", identity), ("__next", parent)]);
    let continuing = filter(
        RulePlan::RecursiveRef { name: "ancestors" },
        RuleExpr::cmp(
            CmpOp::NotEq,
            RuleExpr::col("__origin"),
            RuleExpr::col("__next"),
        ),
    );
    let step_edges = renamed(edges, &[("__edge", identity), ("__parent", parent)]);
    let step = renamed(
        join(continuing, step_edges, vec![("__next", "__edge")]),
        &[("__origin", "__origin"), ("__next", "__parent")],
    );
    let closure = RulePlan::Recursive {
        name: "ancestors",
        seed: Box::new(seed),
        step: Box::new(step),
        is_distinct: false,
        depth_bound: DepthBound::SeedRows,
    };
    let cycles = filter(
        closure,
        RuleExpr::cmp(
            CmpOp::Eq,
            RuleExpr::col("__origin"),
            RuleExpr::col("__next"),
        ),
    );
    let head = RulePlan::Distinct(Box::new(renamed(cycles, &[(identity, "__origin")])));
    invariant(
        builder,
        relation,
        "acyclic:parents",
        InvariantKind::Acyclic,
        &[identity],
        head,
        "Actual parent edges cannot return to the same identity; finite recursion stops extending a discovered cycle.",
    );
}
fn stoichiometry(builder: &mut RegistryBuilder) {
    let keys = ["reaction_id", "phase_id", "species_id"];
    let species = renamed(
        scan("authored.species", "species"),
        &[
            ("__species", "species_id"),
            ("valid_phase_types", "valid_phase_types"),
        ],
    );
    let phases = renamed(
        scan("authored.phases", "phases"),
        &[("__phase", "phase_id"), ("phase_type", "phase_type")],
    );
    let joined = join(
        join(
            scan("authored.stoichiometry", "subject"),
            species,
            vec![("species_id", "__species")],
        ),
        phases,
        vec![("phase_id", "__phase")],
    );
    let defaults = filter(
        joined.clone(),
        RuleExpr::And(vec![
            RuleExpr::IsNull(Box::new(RuleExpr::col("valid_phase_types"))),
            RuleExpr::cmp(
                CmpOp::NotEq,
                RuleExpr::col("phase_type"),
                RuleExpr::Lit(Cell::Enum(pse_material::PhaseType::Aqueous.as_str())),
            ),
        ]),
    );
    let explicit = filter(
        unnest(joined, "valid_phase_types", "__valid_phase"),
        RuleExpr::cmp(
            CmpOp::Eq,
            RuleExpr::col("phase_type"),
            RuleExpr::col("__valid_phase"),
        ),
    );
    let compatible = RulePlan::Distinct(Box::new(RulePlan::Union(vec![
        project(defaults, &keys),
        project(explicit, &keys),
    ])));
    let restrictions = renamed(
        scan("authored.phase_species", "restrictions"),
        &[
            ("__restricted_phase", "phase_id"),
            ("__restricted_species", "species_id"),
        ],
    );
    let unrestricted = anti(
        compatible.clone(),
        project(restrictions.clone(), &["__restricted_phase"]),
        vec![("phase_id", "__restricted_phase")],
    );
    let allowed = join(
        compatible,
        restrictions,
        vec![
            ("phase_id", "__restricted_phase"),
            ("species_id", "__restricted_species"),
        ],
    );
    let valid = RulePlan::Distinct(Box::new(RulePlan::Union(vec![
        project(unrestricted, &keys),
        project(allowed, &keys),
    ])));
    let missing = anti(
        scan("authored.stoichiometry", "subject"),
        valid,
        keys.iter().map(|key| (*key, *key)).collect(),
    );
    invariant(
        builder,
        "authored.stoichiometry",
        "closure:stoichiometry_species_in_phase",
        InvariantKind::Closure,
        &keys,
        project(missing, &keys),
        "Stoichiometric species satisfy declared phase types, the explicit null default, and any per-phase species restriction.",
    );
}
