// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Descriptor ordering and explicit null/empty policies use real engine operators.
#[path = "support/strata_fixture.rs"]
mod fixture;
use fixture::{Fixture, builder, declare, head, id, input};
use pse_schema::model::{Cell, FieldContract, FieldContract as T, RuleDecl};

fn collect(reject_null: bool, reject_empty: bool) -> pse_schema::RegistryBuilder {
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
    let aggregate = if reject_null {
        "array_agg(pse_require_nonnull(member) ORDER BY ordinal)"
    } else {
        "array_agg(member ORDER BY ordinal) FILTER (WHERE member IS NOT NULL)"
    };
    let members = if reject_empty {
        format!("pse_require_nonnull({aggregate})")
    } else {
        format!("coalesce({aggregate}, [])")
    };
    let plan =
        format!("SELECT CAST(1 AS INT UNSIGNED) AS id, {members} AS members FROM authored.input");
    declare(
        &mut builder,
        RuleDecl::new(
            "collect",
            "1",
            0,
            "inferred.collected",
            plan,
            vec![fixture::read("authored.input", "source")],
        )
        .assertions("provenance.collection_assertions"),
    );
    builder
}
#[tokio::test]
async fn ordered_aggregate_tracks_actual_members_and_skipped_nulls_under_partitioning() {
    let fixture = Fixture::new(
        collect(false, false),
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
    let fixture = Fixture::new(collect(false, false), vec![], 1);
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
        collect(true, false),
        vec![vec![Cell::U64(1), Cell::Null]],
        1,
    );
    assert!(null.run(5).await.is_err());
    let empty = Fixture::new(collect(true, true), vec![], 1);
    assert!(empty.run(5).await.is_err());
}
fn unnest(reject_null: bool) -> pse_schema::RegistryBuilder {
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
    let members = if reject_null {
        "pse_require_nonnull(members)"
    } else {
        "coalesce(members, [])"
    };
    let plan = format!("SELECT id, unnest({members}) AS member FROM authored.input");
    declare(
        &mut builder,
        RuleDecl::new(
            "unnest",
            "1",
            0,
            "inferred.expanded",
            plan,
            vec![fixture::read("authored.input", "source")],
        )
        .assertions("provenance.expansion_assertions"),
    );
    builder
}

fn integer_reduction(count: bool, reject_empty: bool) -> pse_schema::RegistryBuilder {
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
    // Native decimal accumulation avoids UInt64 overflow before the exact cast.
    let aggregate = if count {
        "count(value)"
    } else {
        "sum(CAST(value AS DECIMAL(38, 0)))"
    };
    let value = if reject_empty {
        format!("pse_require_nonnull(CASE WHEN count(*) = 0 THEN NULL ELSE {aggregate} END)")
    } else {
        format!("coalesce({aggregate}, 0)")
    };
    let plan = format!(
        "SELECT CAST(1 AS INT UNSIGNED) AS id, CAST({value} AS BIGINT UNSIGNED) AS value FROM authored.input"
    );
    declare(
        &mut builder,
        RuleDecl::new(
            "integer_reduction",
            "1",
            0,
            "inferred.reduced",
            plan,
            vec![fixture::read("authored.input", "source")],
        )
        .assertions("provenance.reduction_assertions"),
    );
    builder
}
#[tokio::test]
async fn checked_integer_aggregate_rejects_overflow_and_preserves_exact_large_values() {
    let exact = Fixture::new(
        integer_reduction(false, false),
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
        integer_reduction(false, false),
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
    let rejected = Fixture::new(integer_reduction(true, true), vec![], 1);
    assert!(rejected.run(5).await.is_err());
    let zero = Fixture::new(integer_reduction(true, false), vec![], 1);
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
    let fixture = Fixture::new(unnest(false), rows.clone(), 2);
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
    let rejected = Fixture::new(unnest(true), rows, 1);
    assert!(rejected.run(5).await.is_err());
}
