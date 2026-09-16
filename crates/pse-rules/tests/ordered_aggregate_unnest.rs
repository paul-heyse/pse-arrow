// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Descriptor ordering and explicit null/empty policies use real engine operators.
#[path = "support/strata_fixture.rs"]
mod fixture;
use fixture::{Fixture, builder, declare, head, id, input};
use pse_schema::model::{
    AggregateEmptyPolicy as Empty, AggregateNullPolicy as Null, Cell, EmptyListPolicy,
    FieldContract, FieldContract as T, NullListPolicy, RuleAggregate, RuleAggregateFn as F,
    RuleDecl, RuleExpr as E, RuleHead, RulePlan as P,
};

fn collect(null: Null, empty: Empty) -> pse_schema::RegistryBuilder {
    let mut builder = builder();
    input(
        &mut builder,
        vec![
            id("ordinal"),
            FieldContract::payload(
                "member",
                T::native(datafusion::arrow::datatypes::DataType::UInt32),
                "Descriptor identity",
            )
            .optional(),
        ],
        &["ordinal"],
    );
    head(
        &mut builder,
        "collected",
        "collection_assertions",
        &["id"],
        vec![
            id("id"),
            FieldContract::payload(
                "members",
                T::list(T::native(datafusion::arrow::datatypes::DataType::UInt32)),
                "Explicitly ordered descriptors",
            ),
        ],
    );
    let plan = P::Project {
        input: Box::new(P::Aggregate {
            input: Box::new(P::Scan {
                relation: "authored.input".to_owned(),
                port: "source",
            }),
            group: vec![],
            aggregates: vec![RuleAggregate {
                function: F::CollectOrdered,
                input: Some(E::col("member")),
                output_name: ("members").into(),
                order_by: (vec![("ordinal", true)])
                    .into_iter()
                    .map(|(name, ascending)| (name.into(), ascending))
                    .collect(),
                null_policy: null,
                empty_policy: empty,
            }],
        }),
        columns: vec![
            ("id".into(), E::Lit(Cell::U64(1))),
            ("members".into(), E::col("members")),
        ],
    };
    declare(
        &mut builder,
        RuleDecl::new(
            "collect",
            "1",
            0,
            RuleHead::Relation("inferred.collected".to_owned()),
            plan,
        )
        .assertions("provenance.collection_assertions"),
    );
    builder
}
#[tokio::test]
async fn ordered_aggregate_tracks_actual_members_and_skipped_nulls_under_partitioning() {
    let fixture = Fixture::new(
        collect(Null::SkipMissing, Empty::EmptyList),
        vec![
            vec![Cell::U64(3), Cell::U64(30)],
            vec![Cell::U64(1), Cell::U64(10)],
            vec![Cell::U64(2), Cell::Null],
        ],
        3,
    );
    let result = fixture.run(5).await.unwrap();
    assert_eq!(
        fixture.rows(&result, "inferred.collected"),
        vec![vec![
            Cell::U64(1),
            Cell::List(vec![Cell::U64(10), Cell::U64(30)])
        ]]
    );
    assert_eq!(
        fixture.rows(&result, "provenance.rule_support_edges").len(),
        3
    );
}
#[tokio::test]
async fn ordered_aggregate_empty_input_has_explicit_empty_list_and_complete_absence_support() {
    let fixture = Fixture::new(collect(Null::SkipMissing, Empty::EmptyList), vec![], 1);
    let result = fixture.run(5).await.unwrap();
    assert_eq!(
        fixture.rows(&result, "inferred.collected"),
        vec![vec![Cell::U64(1), Cell::List(vec![])]]
    );
    let support = fixture.rows(&result, "provenance.rule_support_edges");
    assert_eq!(support.len(), 1);
    assert_eq!(support[0][9], Cell::Enum("absence"));
    assert_eq!(support[0][10], Cell::Null);
}
#[tokio::test]
async fn ordered_aggregate_rejects_declared_null_and_empty_errors() {
    let null = Fixture::new(
        collect(Null::Reject, Empty::EmptyList),
        vec![vec![Cell::U64(1), Cell::Null]],
        1,
    );
    assert!(null.run(5).await.is_err());
    let empty = Fixture::new(collect(Null::Reject, Empty::Error), vec![], 1);
    assert!(empty.run(5).await.is_err());
}
fn unnest(null_list: NullListPolicy) -> pse_schema::RegistryBuilder {
    let mut builder = builder();
    input(
        &mut builder,
        vec![
            id("id"),
            FieldContract::payload(
                "members",
                T::list(T::native(datafusion::arrow::datatypes::DataType::UInt32)),
                "Ordered members",
            )
            .optional(),
        ],
        &["id"],
    );
    head(
        &mut builder,
        "expanded",
        "expansion_assertions",
        &["id", "member"],
        vec![id("id"), id("member")],
    );
    let plan = P::Project {
        input: Box::new(P::Unnest {
            input: Box::new(P::Scan {
                relation: "authored.input".to_owned(),
                port: "source",
            }),
            column: ("members").into(),
            value_name: ("member").into(),
            null_list,
            empty_list: EmptyListPolicy::NoMembers,
        }),
        columns: vec![
            ("id".into(), E::col("source.id")),
            ("member".into(), E::col("member")),
        ],
    };
    declare(
        &mut builder,
        RuleDecl::new(
            "unnest",
            "1",
            0,
            RuleHead::Relation("inferred.expanded".to_owned()),
            plan,
        )
        .assertions("provenance.expansion_assertions"),
    );
    builder
}

fn integer_reduction(function: F, empty: Empty) -> pse_schema::RegistryBuilder {
    let mut builder = builder();
    input(
        &mut builder,
        vec![
            id("ordinal"),
            FieldContract::payload(
                "value",
                T::native(datafusion::arrow::datatypes::DataType::UInt64),
                "Exact integer input",
            ),
        ],
        &["ordinal"],
    );
    head(
        &mut builder,
        "reduced",
        "reduction_assertions",
        &["id"],
        vec![
            id("id"),
            FieldContract::payload(
                "value",
                T::native(datafusion::arrow::datatypes::DataType::UInt64),
                "Exact aggregate result",
            ),
        ],
    );
    let plan = P::Project {
        input: Box::new(P::Aggregate {
            input: Box::new(P::Scan {
                relation: "authored.input".to_owned(),
                port: "source",
            }),
            group: vec![],
            aggregates: vec![RuleAggregate {
                function,
                input: Some(E::col("value")),
                output_name: ("value").into(),
                order_by: vec![],
                null_policy: Null::Reject,
                empty_policy: empty,
            }],
        }),
        columns: vec![
            ("id".into(), E::Lit(Cell::U64(1))),
            ("value".into(), E::col("value")),
        ],
    };
    declare(
        &mut builder,
        RuleDecl::new(
            "integer_reduction",
            "1",
            0,
            RuleHead::Relation("inferred.reduced".to_owned()),
            plan,
        )
        .assertions("provenance.reduction_assertions"),
    );
    builder
}
#[tokio::test]
async fn checked_integer_aggregate_rejects_overflow_and_preserves_exact_large_values() {
    let exact = Fixture::new(
        integer_reduction(F::Sum, Empty::Zero),
        vec![
            vec![Cell::U64(1), Cell::U64(u64::MAX - 1)],
            vec![Cell::U64(2), Cell::U64(1)],
        ],
        2,
    );
    let result = exact.run(5).await.unwrap();
    assert_eq!(
        exact.rows(&result, "inferred.reduced"),
        vec![vec![Cell::U64(1), Cell::U64(u64::MAX)]]
    );
    let overflow = Fixture::new(
        integer_reduction(F::Sum, Empty::Zero),
        vec![
            vec![Cell::U64(1), Cell::U64(u64::MAX)],
            vec![Cell::U64(2), Cell::U64(1)],
        ],
        2,
    );
    assert!(overflow.run(5).await.is_err());
}
#[tokio::test]
async fn count_empty_error_refuses_zero_and_zero_policy_is_explicit() {
    let rejected = Fixture::new(integer_reduction(F::Count, Empty::Error), vec![], 1);
    assert!(rejected.run(5).await.is_err());
    let zero = Fixture::new(integer_reduction(F::Count, Empty::Zero), vec![], 1);
    let result = zero.run(5).await.unwrap();
    assert_eq!(
        zero.rows(&result, "inferred.reduced"),
        vec![vec![Cell::U64(1), Cell::U64(0)]]
    );
}
#[tokio::test]
async fn unnest_retains_parent_support_without_inventing_members_for_empty_or_null_lists() {
    let rows = vec![
        vec![Cell::U64(1), Cell::List(vec![Cell::U64(4), Cell::U64(5)])],
        vec![Cell::U64(2), Cell::List(vec![])],
        vec![Cell::U64(3), Cell::Null],
    ];
    let fixture = Fixture::new(unnest(NullListPolicy::NoMembers), rows.clone(), 2);
    let result = fixture.run(5).await.unwrap();
    assert_eq!(
        fixture.rows(&result, "inferred.expanded"),
        vec![
            vec![Cell::U64(1), Cell::U64(4)],
            vec![Cell::U64(1), Cell::U64(5)]
        ]
    );
    assert_eq!(
        fixture.rows(&result, "provenance.rule_support_edges").len(),
        2
    );
    let rejected = Fixture::new(unnest(NullListPolicy::Reject), rows, 1);
    assert!(rejected.run(5).await.is_err());
}
