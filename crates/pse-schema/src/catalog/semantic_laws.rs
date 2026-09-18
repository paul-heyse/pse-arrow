// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Conservation candidate membership and complete, ordered participation decisions.
use super::declarations::column;
use crate::{
    RegistryBuilder,
    model::{
        Authority, DerivationGranularity, FieldContract, FieldContract as T, Namespace as N,
        RelationDecl, SnapshotClass as S,
    },
};
/// Declare the exact conservation candidate, partition and ordered-term contracts.
#[expect(
    clippy::too_many_lines,
    reason = "one declarative catalog family keeps its native rules and field declarations together"
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
                "subject",
                "source_family",
                "subject_projection",
                "expansion",
            ]
            .contains(&column.name())
        })
        .cloned()
        .collect::<Vec<_>>();
    context.extend([
        column(
            "coordinates",
            super::s6_15_semantic::conservation_values::coordinates(),
        ),
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
            column("position", T::nonnegative(i64::from(u16::MAX))),
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
            column(
                "decision",
                super::s6_15_semantic::conservation_values::participation(),
            ),
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
