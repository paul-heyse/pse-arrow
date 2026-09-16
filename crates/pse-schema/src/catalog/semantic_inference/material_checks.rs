// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Selected template cardinalities count actual distinct material members.
use super::{
    Cell, E, N, P, RegistryBuilder, RuleDecl, RuleHead, S, T, assertion, column, equals, join,
    project, provenance, relation, scan,
};
use crate::model::{
    AggregateEmptyPolicy, AggregateNullPolicy, CmpOp, EmptyListPolicy, NullListPolicy,
    RuleAggregate, RuleAggregateFn,
};

#[expect(
    clippy::too_many_lines,
    reason = "Keep the declarative relation and native rule family together for schema review"
)]
pub(super) fn declare(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Inferred,
        "material_member_counts",
        S::Derived,
        &["material_system_id", "kind"],
        vec![
            column("material_system_id", T::id()),
            column("kind", T::enumeration("DomainKind")),
            column("count", T::native(arrow_schema::DataType::UInt64)),
            provenance(),
        ],
        "Exact counts over distinct selected material IDs, including explicit empty lists.",
    );
    relation(
        builder,
        N::Inferred,
        "material_template_checks",
        S::Derived,
        &["instance_id"],
        vec![
            column("instance_id", T::id()),
            column("material_system_id", T::id()),
            column("phase_count", T::native(arrow_schema::DataType::UInt64)),
            column("species_count", T::native(arrow_schema::DataType::UInt64)),
            provenance(),
        ],
        "Actual selected material membership satisfies the template's declared bounds.",
    );
    assertion(
        builder,
        "inferred.material_member_counts",
        "material_count_assertions",
    );
    assertion(
        builder,
        "inferred.material_template_checks",
        "material_check_assertions",
    );
    for (kind, list, nonempty, empty) in [
        (
            "phase",
            "phase_ids",
            "P4.count_phases",
            "P4.count_empty_phases",
        ),
        (
            "species",
            "species_ids",
            "P4.count_species",
            "P4.count_empty_species",
        ),
    ] {
        let members = P::Unnest {
            input: Box::new(scan("normalized.material_systems", "materials")),
            column: (list).into(),
            value_name: ("member").into(),
            null_list: NullListPolicy::Reject,
            empty_list: EmptyListPolicy::NoMembers,
        };
        let members = P::Distinct(Box::new(project(
            members,
            vec![
                ("material_system_id", E::col("material_system_id")),
                ("member", E::col("member")),
            ],
        )));
        let counted = P::Aggregate {
            input: Box::new(members),
            group: (vec!["material_system_id"])
                .into_iter()
                .map(Into::into)
                .collect(),
            aggregates: vec![RuleAggregate {
                function: RuleAggregateFn::Count,
                input: None,
                output_name: ("count").into(),
                order_by: vec![],
                null_policy: AggregateNullPolicy::Reject,
                empty_policy: AggregateEmptyPolicy::Zero,
            }],
        };
        builder.declare_rule(
            RuleDecl::new(
                nonempty,
                "1",
                0,
                RuleHead::Relation("inferred.material_member_counts".to_owned()),
                project(
                    counted,
                    vec![
                        ("material_system_id", E::col("material_system_id")),
                        ("kind", E::Lit(Cell::Enum(kind))),
                        ("count", E::col("count")),
                        ("derivation_id", E::col("material_system_id")),
                    ],
                ),
            )
            .assertions("provenance.material_count_assertions"),
        );
        let missing = P::Filter {
            input: Box::new(scan("normalized.material_systems", "materials")),
            predicate: equals(E::ListLen(Box::new(E::col(list))), Cell::I64(0)),
        };
        builder.declare_rule(
            RuleDecl::new(
                empty,
                "1",
                0,
                RuleHead::Relation("inferred.material_member_counts".to_owned()),
                P::Union(vec![project(
                    missing,
                    vec![
                        ("material_system_id", E::col("material_system_id")),
                        ("kind", E::Lit(Cell::Enum(kind))),
                        ("count", E::Lit(Cell::U64(0))),
                        ("derivation_id", E::col("material_system_id")),
                    ],
                )]),
            )
            .assertions("provenance.material_count_assertions"),
        );
    }
    let checks = join(
        scan("normalized.instance_bindings", "instances"),
        scan("normalized.template_material_constraints", "constraints"),
        vec![("instances.template_id", "constraints.template_id")],
    );
    let checks = join(
        checks,
        scan("normalized.property_packages", "packages"),
        vec![(
            "instances.property_package_id",
            "packages.property_package_id",
        )],
    );
    let phases = project(
        P::Filter {
            input: Box::new(scan("inferred.material_member_counts", "phases")),
            predicate: equals(E::col("kind"), Cell::Enum("phase")),
        },
        vec![
            ("phase_material", E::col("material_system_id")),
            ("phase_count", E::col("count")),
        ],
    );
    let species = project(
        P::Filter {
            input: Box::new(scan("inferred.material_member_counts", "species")),
            predicate: equals(E::col("kind"), Cell::Enum("species")),
        },
        vec![
            ("species_material", E::col("material_system_id")),
            ("species_count", E::col("count")),
        ],
    );
    let checks = join(
        join(
            checks,
            phases,
            vec![("packages.material_system_id", "phase_material")],
        ),
        species,
        vec![("packages.material_system_id", "species_material")],
    );
    let bounds = |count, min, max| {
        E::And(vec![
            E::cmp(CmpOp::GtEq, E::col(count), E::col(min)),
            E::Or(vec![
                E::IsNull(Box::new(E::col(max))),
                E::cmp(CmpOp::LtEq, E::col(count), E::col(max)),
            ]),
        ])
    };
    let valid = E::And(vec![
        bounds(
            "phase_count",
            "constraints.min_phases",
            "constraints.max_phases",
        ),
        bounds(
            "species_count",
            "constraints.min_species",
            "constraints.max_species",
        ),
    ]);
    builder.declare_rule(
        RuleDecl::new(
            "P4.material_template_check",
            "1",
            1,
            RuleHead::Relation("inferred.material_template_checks".to_owned()),
            project(
                P::Filter {
                    input: Box::new(checks),
                    predicate: valid,
                },
                vec![
                    ("instance_id", E::col("instances.instance_id")),
                    ("material_system_id", E::col("packages.material_system_id")),
                    ("phase_count", E::col("phase_count")),
                    ("species_count", E::col("species_count")),
                    ("derivation_id", E::col("instances.derivation_id")),
                ],
            ),
        )
        .assertions("provenance.material_check_assertions"),
    );
    let keys = ["assertion_id"];
    super::super::inv::declare(
        builder,
        "provenance.material_check_assertions",
        "material_bounds_satisfied",
        crate::model::InvariantKind::Check,
        &keys,
        super::super::inv::project(
            super::super::inv::filter(
                super::super::inv::scan("provenance.material_check_assertions", "subject"),
                E::Not(Box::new(E::IsTrue(Box::new(equals(
                    E::col("truth"),
                    Cell::Enum("true"),
                ))))),
            ),
            &keys,
        ),
        "Actual phase and species counts satisfy selected template constraints.",
    );
}
