// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Conservation candidate membership and complete, ordered participation decisions.
use super::declarations::column;
use crate::{
    RegistryBuilder,
    model::{
        AggregateEmptyPolicy, AggregateNullPolicy, Authority, Cell, CmpOp, DerivationGranularity,
        FieldContract, FieldContract as T, Namespace as N, NullEquality, RelationDecl,
        RuleAggregate, RuleAggregateFn, RuleDecl, RuleExpr as E, RuleHead, RulePlan as P,
        SnapshotClass as S,
    },
};

/// Declare the exact conservation candidate, partition and ordered-term contracts.
#[expect(
    clippy::too_many_lines,
    reason = "Keep the declarative relation and native rule family together for schema review"
)]
pub fn declare(builder: &mut RegistryBuilder) {
    let Some(application) = builder
        .declared_relations()
        .iter()
        .find(|spec| spec.key.qualified_name() == "compiled.law_applications")
        .cloned()
    else {
        return;
    };
    let mut context = application
        .columns
        .iter()
        .filter(|column| {
            ![
                "law_family",
                "subject_kind",
                "source_family",
                "subject_projection",
                "expansion",
            ]
            .contains(&column.name())
        })
        .cloned()
        .collect::<Vec<_>>();
    context.extend([
        column("balance_enum_id", T::id()),
        column("balance_member", T::native(arrow_schema::DataType::Utf8)),
    ]);
    relation(builder, "law_contexts", &["application_id"], context);
    relation(
        builder,
        "law_axes",
        &["application_id", "position"],
        vec![
            column("application_id", T::id()),
            column("position", T::native(arrow_schema::DataType::UInt16)),
            column("domain_id", T::id()),
            column("bound_index_id", T::id()),
            provenance(),
        ],
    );
    for name in [
        "law_candidates",
        "law_subject_matches",
        "law_internal_transfers",
    ] {
        relation(
            builder,
            name,
            &["application_id", "contribution_id"],
            vec![
                column("application_id", T::id()),
                column("contribution_id", T::id()),
                provenance(),
            ],
        );
    }
    relation(
        builder,
        "law_participation_decisions",
        &["application_id", "contribution_id"],
        vec![
            column("application_id", T::id()),
            column("contribution_id", T::id()),
            column("decision", T::enumeration("ParticipationDecision")),
            column("reason", T::enumeration("ParticipationReason")),
            column("sign", T::native(arrow_schema::DataType::Int32)),
            provenance(),
        ],
    );
    relation(
        builder,
        "law_ordered_terms",
        &["application_id"],
        vec![
            column("application_id", T::id()),
            column("contribution_ids", T::list(T::id())),
            provenance(),
        ],
    );
    relation(
        builder,
        "law_empty_applications",
        &["application_id"],
        vec![column("application_id", T::id()), provenance()],
    );
    for (head, assertion_name) in [
        ("compiled.law_applications", "law_application_assertions"),
        ("inferred.law_candidates", "law_candidate_assertions"),
        (
            "inferred.law_subject_matches",
            "law_subject_match_assertions",
        ),
        (
            "inferred.law_internal_transfers",
            "law_internal_transfer_assertions",
        ),
        (
            "inferred.law_participation_decisions",
            "law_participation_assertions",
        ),
        ("inferred.law_ordered_terms", "law_ordered_term_assertions"),
        (
            "inferred.law_empty_applications",
            "law_empty_application_assertions",
        ),
    ] {
        assertion(builder, head, assertion_name);
    }
    let bindings = join(
        scan("inferred.law_contexts", "context"),
        scan("reference.law_bindings", "binding"),
        vec![
            ("context.law_template_id", "binding.law_template_id"),
            ("context.balance_enum_id", "binding.balance_enum_id"),
            ("context.balance_member", "binding.balance_member"),
        ],
    );
    let columns = application
        .columns
        .iter()
        .map(|column| {
            let source = match column.name() {
                "law_family" => "binding.family",
                "subject_kind" => "binding.subject_kind",
                "source_family" => "binding.source_family",
                "subject_projection" => "binding.subject_projection",
                "expansion" => "binding.expansion",
                "law_template_id" => "context.law_template_id",
                _ => column.name(),
            };
            (column.name(), E::col(source.to_owned()))
        })
        .collect();
    rule(
        builder,
        "P8.law_applications",
        0,
        "compiled.law_applications",
        "provenance.law_application_assertions",
        project(bindings, columns),
    );
    let scoped = join(
        scan("compiled.law_applications", "law"),
        scan("inferred.scope_members", "scope"),
        vec![("law.scope_id", "scope.scope_id")],
    );
    let candidates = join(
        scoped,
        scan("compiled.contributions", "term"),
        vec![("scope.entity_id", "term.owner_instance_id")],
    );
    rule(
        builder,
        "P8.law_candidates",
        1,
        "inferred.law_candidates",
        "provenance.law_candidate_assertions",
        project(
            candidates,
            vec![
                ("application_id", E::col("law.application_id")),
                ("contribution_id", E::col("term.contribution_id")),
                ("derivation_id", E::col("term.derivation_id")),
            ],
        ),
    );
    let subject = E::Or(vec![
        literal_eq("law.subject_kind", "total"),
        E::And(vec![
            literal_eq("law.subject_kind", "species"),
            enum_in("term.subject_kind", &["species", "phase_species"]),
        ]),
        E::And(vec![
            literal_eq("law.subject_kind", "element"),
            literal_eq("law.subject_projection", "species_to_element"),
            enum_in("term.subject_kind", &["species", "phase_species"]),
        ]),
        eq(E::col("law.subject_kind"), E::col("term.subject_kind")),
    ]);
    let compatible = E::And(vec![
        subject,
        E::Or(vec![
            E::IsNull(Box::new(E::col("law.subject_id"))),
            E::IsNull(Box::new(E::col("term.subject_id"))),
            eq(E::col("law.subject_id"), E::col("term.subject_id")),
            literal_eq("law.subject_projection", "species_to_element"),
        ]),
        E::Or(vec![
            E::IsNull(Box::new(E::col("law.phase_id"))),
            E::IsNull(Box::new(E::col("term.phase_id"))),
            eq(E::col("law.phase_id"), E::col("term.phase_id")),
        ]),
    ]);
    rule(
        builder,
        "P8.law_subject_matches",
        2,
        "inferred.law_subject_matches",
        "provenance.law_subject_match_assertions",
        keys(filter(candidate_values(), compatible)),
    );
    internal(builder);
    participation(builder);
    let included = filter(
        scan("inferred.law_participation_decisions", "terms"),
        literal_eq("decision", "included"),
    );
    let ordered = P::Aggregate {
        input: Box::new(included),
        group: (vec!["application_id"])
            .into_iter()
            .map(Into::into)
            .collect(),
        aggregates: vec![
            RuleAggregate {
                function: RuleAggregateFn::CollectOrdered,
                input: Some(E::col("contribution_id")),
                output_name: ("contribution_ids").into(),
                order_by: (vec![("contribution_id", true)])
                    .into_iter()
                    .map(|(name, ascending)| (name.into(), ascending))
                    .collect(),
                null_policy: AggregateNullPolicy::Reject,
                empty_policy: AggregateEmptyPolicy::EmptyList,
            },
            RuleAggregate {
                function: RuleAggregateFn::Min,
                input: Some(E::col("derivation_id")),
                output_name: ("derivation_id").into(),
                order_by: vec![],
                null_policy: AggregateNullPolicy::Reject,
                empty_policy: AggregateEmptyPolicy::Error,
            },
        ],
    };
    rule(
        builder,
        "P8.law_ordered_terms",
        4,
        "inferred.law_ordered_terms",
        "provenance.law_ordered_term_assertions",
        ordered,
    );
    let empty = anti(
        scan("compiled.law_applications", "law"),
        scan("inferred.law_ordered_terms", "ordered"),
        vec![("law.application_id", "ordered.application_id")],
    );
    rule(
        builder,
        "P8.law_empty_applications",
        5,
        "inferred.law_empty_applications",
        "provenance.law_empty_application_assertions",
        project(
            empty,
            vec![
                ("application_id", E::col("law.application_id")),
                ("derivation_id", E::col("law.derivation_id")),
            ],
        ),
    );
}
fn internal(builder: &mut RegistryBuilder) {
    let paired = join(
        candidate_values(),
        scan("compiled.contributions", "paired"),
        vec![(
            "term.transfer_connection_id",
            "paired.transfer_connection_id",
        )],
    );
    let crossing = join(
        paired,
        scan("inferred.boundary_crossings", "cut"),
        vec![
            ("law.scope_id", "cut.scope_id"),
            ("term.transfer_connection_id", "cut.connection_id"),
        ],
    );
    let paired_scope = join(
        crossing,
        scan("inferred.scope_members", "paired_scope"),
        vec![
            ("law.scope_id", "paired_scope.scope_id"),
            ("paired.owner_instance_id", "paired_scope.entity_id"),
        ],
    );
    let exact = E::And(vec![
        literal_eq("cut.classification", "internal"),
        eq(E::col("term.law_family"), E::col("paired.law_family")),
        E::Not(Box::new(eq(
            E::col("term.contribution_id"),
            E::col("paired.contribution_id"),
        ))),
        E::Not(Box::new(eq(
            E::col("term.owner_instance_id"),
            E::col("paired.owner_instance_id"),
        ))),
        E::Or(vec![
            E::And(vec![
                literal_eq("term.orientation", "into_scope"),
                literal_eq("paired.orientation", "out_of_scope"),
            ]),
            E::And(vec![
                literal_eq("term.orientation", "out_of_scope"),
                literal_eq("paired.orientation", "into_scope"),
            ]),
        ]),
        eq(
            E::col("term.quantity_type_id"),
            E::col("paired.quantity_type_id"),
        ),
        eq(E::col("term.product_id"), E::col("paired.product_id")),
        eq(E::col("term.subject_kind"), E::col("paired.subject_kind")),
        same("term.subject_id", "paired.subject_id"),
        same("term.phase_id", "paired.phase_id"),
        same("term.subject_axis", "paired.subject_axis"),
        same("term.phase_axis", "paired.phase_axis"),
    ]);
    rule(
        builder,
        "P8.law_internal_transfers",
        2,
        "inferred.law_internal_transfers",
        "provenance.law_internal_transfer_assertions",
        keys(filter(paired_scope, exact)),
    );
}
fn participation(builder: &mut RegistryBuilder) {
    let family = eq(E::col("law.source_family"), E::col("term.law_family"));
    let matched_family = filter(candidate_values(), family.clone());
    let subject_keys = || scan("inferred.law_subject_matches", "subject");
    let key_pairs = vec![
        ("law.application_id", "subject.application_id"),
        ("term.contribution_id", "subject.contribution_id"),
    ];
    let subjects = join(matched_family.clone(), subject_keys(), key_pairs.clone());
    let internal_keys = || scan("inferred.law_internal_transfers", "internal");
    let internal_pairs = vec![
        ("law.application_id", "internal.application_id"),
        ("term.contribution_id", "internal.contribution_id"),
    ];
    for (name, input, decision, reason) in [
        (
            "P8.law_family_exclusion",
            filter(candidate_values(), E::Not(Box::new(family))),
            "excluded",
            "family_mismatch",
        ),
        (
            "P8.law_subject_exclusion",
            anti(matched_family, subject_keys(), key_pairs),
            "excluded",
            "subject_mismatch",
        ),
        (
            "P8.law_internal_exclusion",
            join(subjects.clone(), internal_keys(), internal_pairs.clone()),
            "excluded",
            "internal_transfer",
        ),
        (
            "P8.law_inclusion",
            anti(subjects, internal_keys(), internal_pairs),
            "included",
            "matched",
        ),
    ] {
        let signed = if decision == "included" {
            [
                (&["into_scope", "generation"][..], 1),
                (&["out_of_scope", "accumulation"][..], -1),
            ]
            .into_iter()
            .map(|(orientations, sign)| {
                (
                    filter(input.clone(), enum_in("term.orientation", orientations)),
                    sign,
                )
            })
            .collect::<Vec<_>>()
        } else {
            // Excluded terms retain their reason and source without contributing
            // an oriented coefficient to the conservation expression.
            vec![(input, 0)]
        };
        let parts = signed
            .into_iter()
            .map(|(input, sign)| {
                project(
                    input,
                    vec![
                        ("application_id", E::col("law.application_id")),
                        ("contribution_id", E::col("term.contribution_id")),
                        ("decision", E::Lit(Cell::Enum(decision))),
                        ("reason", E::Lit(Cell::Enum(reason))),
                        ("sign", E::Lit(Cell::I64(sign))),
                        ("derivation_id", E::col("term.derivation_id")),
                    ],
                )
            })
            .collect();
        rule(
            builder,
            name,
            3,
            "inferred.law_participation_decisions",
            "provenance.law_participation_assertions",
            P::Union(parts),
        );
    }
}
fn candidate_values() -> P {
    join(
        join(
            scan("inferred.law_candidates", "candidate"),
            scan("compiled.law_applications", "law"),
            vec![("candidate.application_id", "law.application_id")],
        ),
        scan("compiled.contributions", "term"),
        vec![("candidate.contribution_id", "term.contribution_id")],
    )
}
fn keys(input: P) -> P {
    project(
        input,
        vec![
            ("application_id", E::col("law.application_id")),
            ("contribution_id", E::col("term.contribution_id")),
            ("derivation_id", E::col("term.derivation_id")),
        ],
    )
}
fn provenance() -> FieldContract {
    FieldContract::provenance(
        "derivation_id",
        T::id(),
        "Exact actual law source and rule support.",
    )
}
fn relation(
    builder: &mut RegistryBuilder,
    name: &'static str,
    keys: &[&'static str],
    columns: Vec<FieldContract>,
) {
    builder.declare_relation(
        RelationDecl::new(
            N::Inferred,
            name,
            1,
            Authority::Derived,
            S::Derived,
            "Declared conservation expansion substrate.",
        )
        .pk(keys)
        .columns(columns)
        .granularity(DerivationGranularity::Rule),
    );
}
fn assertion(builder: &mut RegistryBuilder, head: &str, name: &'static str) {
    if let Some(spec) = builder
        .declared_relations()
        .iter()
        .find(|spec| spec.key.qualified_name() == head)
    {
        let columns = crate::model::rule::assertion_columns(&spec.columns);
        builder.declare_relation(
            RelationDecl::new(
                N::Provenance,
                name,
                1,
                Authority::Derived,
                S::Derived,
                "Typed conservation rule assertion.",
            )
            .pk(&["assertion_id"])
            .columns(columns)
            .granularity(DerivationGranularity::Rule),
        );
    }
}
fn rule(
    builder: &mut RegistryBuilder,
    name: &'static str,
    stratum: u16,
    head: &'static str,
    assertion: &'static str,
    plan: P,
) {
    builder.declare_rule(
        RuleDecl::new(
            name,
            "1",
            100 + stratum,
            RuleHead::Relation(head.to_owned()),
            P::Union(vec![plan]),
        )
        .assertions(assertion)
        .stratified_negation(),
    );
}
fn scan(name: &str, port: &'static str) -> P {
    P::Scan {
        relation: name.to_owned(),
        port,
    }
}
fn project(input: P, columns: Vec<(&str, E)>) -> P {
    P::Project {
        input: Box::new(input),
        columns: (columns)
            .into_iter()
            .map(|(name, expression)| (name.to_owned().into(), expression))
            .collect(),
    }
}
fn filter(input: P, predicate: E) -> P {
    P::Filter {
        input: Box::new(input),
        predicate,
    }
}
fn join(left: P, right: P, keys: Vec<(&'static str, &'static str)>) -> P {
    P::EquiJoin {
        left: Box::new(left),
        right: Box::new(right),
        keys: (keys)
            .into_iter()
            .map(|(left, right)| (left.into(), right.into()))
            .collect(),
        null_equality: NullEquality::NullEqualsNothing,
    }
}
fn anti(left: P, right: P, keys: Vec<(&'static str, &'static str)>) -> P {
    P::AntiJoin {
        left: Box::new(left),
        right: Box::new(right),
        keys: (keys)
            .into_iter()
            .map(|(left, right)| (left.into(), right.into()))
            .collect(),
    }
}
fn eq(left: E, right: E) -> E {
    E::cmp(CmpOp::Eq, left, right)
}
fn literal_eq(name: &'static str, value: &'static str) -> E {
    eq(E::col(name), E::Lit(Cell::Enum(value)))
}
fn same(left: &'static str, right: &'static str) -> E {
    E::IsNotDistinctFrom(Box::new(E::col(left)), Box::new(E::col(right)))
}
fn enum_in(name: &'static str, members: &[&'static str]) -> E {
    E::InList {
        expr: Box::new(E::col(name)),
        list: members.iter().map(|member| Cell::Enum(member)).collect(),
    }
}
