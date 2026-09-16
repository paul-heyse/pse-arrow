// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Truth and incompatible payloads are retained without first/last-writer precedence.
#[path = "support/strata_fixture.rs"]
mod fixture;
#[path = "support/row_key.rs"]
mod row_key;
use fixture::{Fixture, builder, declare, head, id, input};
use pse_schema::model::{
    Cell, ConflictPolicy, FieldContract, FieldContract as T, RuleDecl, RuleExpr as E, RuleHead,
    RulePlan as P,
};

fn program(policy: ConflictPolicy, second: bool, classify: bool) -> pse_schema::RegistryBuilder {
    let mut builder = builder();
    input(
        &mut builder,
        vec![
            id("id"),
            FieldContract::payload(
                "flag",
                T::native(datafusion::arrow::datatypes::DataType::Boolean),
                "Known or missing predicate",
            )
            .optional(),
            FieldContract::payload(
                "value",
                T::native(datafusion::arrow::datatypes::DataType::Float64),
                "Actual payload",
            ),
        ],
        &["id"],
    );
    head(
        &mut builder,
        "facts",
        "fact_assertions",
        &["id"],
        vec![
            id("id"),
            FieldContract::payload(
                "value",
                T::native(datafusion::arrow::datatypes::DataType::Float64),
                "Actual payload",
            ),
        ],
    );
    let rule = RuleDecl::new(
        "predicate",
        "1",
        0,
        RuleHead::Relation("inferred.facts".to_owned()),
        P::Project {
            input: Box::new(predicate(classify)),
            columns: vec![
                ("id".into(), E::col("id")),
                ("value".into(), E::col("value")),
            ],
        },
    )
    .assertions("provenance.fact_assertions")
    .conflicts(policy);
    declare(&mut builder, rule);
    if second {
        declare(
            &mut builder,
            RuleDecl::new(
                "competitor",
                "1",
                0,
                RuleHead::Relation("inferred.facts".to_owned()),
                P::Project {
                    input: Box::new(predicate(classify)),
                    columns: vec![
                        ("id".into(), E::col("id")),
                        ("value".into(), E::Lit(Cell::F64(-0.0))),
                    ],
                },
            )
            .assertions("provenance.fact_assertions")
            .conflicts(policy),
        );
    }
    builder
}
fn predicate(classify: bool) -> P {
    let input = Box::new(P::Scan {
        relation: "authored.input".to_owned(),
        port: "source",
    });
    let predicate = E::col("flag");
    if classify {
        P::Assert { input, predicate }
    } else {
        P::Filter { input, predicate }
    }
}

#[tokio::test]
async fn ordinary_filter_does_not_assert_false_or_unknown_rows() {
    let fixture = Fixture::new(
        program(ConflictPolicy::Reject, false, false),
        vec![
            vec![Cell::U64(1), Cell::Bool(true), Cell::F64(1.0)],
            vec![Cell::U64(2), Cell::Bool(false), Cell::F64(2.0)],
            vec![Cell::U64(3), Cell::Null, Cell::F64(3.0)],
        ],
        1,
    );
    let result = fixture.run(5).await.unwrap();
    assert_eq!(
        fixture.rows(&result, "inferred.facts"),
        vec![vec![Cell::U64(1), Cell::F64(1.0)]]
    );
    for name in [
        "inferred.rule_outcomes",
        "provenance.fact_assertions",
        "provenance.rule_support_edges",
    ] {
        assert_eq!(
            fixture.rows(&result, name).len(),
            1,
            "selection emits only its matching source: {name}"
        );
    }
}

#[tokio::test]
async fn four_valued_rule_outcomes_false_is_not_unknown_and_true_heads_are_decided() {
    let fixture = Fixture::new(
        program(ConflictPolicy::Undecided, false, true),
        vec![
            vec![Cell::U64(1), Cell::Bool(true), Cell::F64(1.0)],
            vec![Cell::U64(2), Cell::Bool(false), Cell::F64(2.0)],
            vec![Cell::U64(3), Cell::Null, Cell::F64(3.0)],
        ],
        1,
    );
    let result = fixture.run(5).await.unwrap();
    assert_eq!(
        fixture.rows(&result, "inferred.facts"),
        vec![vec![Cell::U64(1), Cell::F64(1.0)]]
    );
    let outcomes = fixture.rows(&result, "inferred.rule_outcomes");
    assert_eq!(outcomes.len(), 3);
    let facts = fixture.registry.relation("inferred.facts").unwrap();
    for (id, truth) in [(1, "true"), (2, "false"), (3, "unknown")] {
        let key = row_key::values(
            &fixture.registry,
            facts.id,
            &[facts.column("id").unwrap()],
            &[Cell::U64(id)],
        )
        .await;
        let outcome = outcomes
            .iter()
            .find(|row| row[2] == Cell::Hash(key))
            .unwrap();
        assert_eq!(outcome[3], Cell::Enum(truth));
    }
    assert_eq!(fixture.rows(&result, "provenance.fact_assertions").len(), 3);
    assert_eq!(
        fixture.rows(&result, "provenance.rule_support_edges").len(),
        3
    );
}
#[tokio::test]
async fn native_signed_zero_equality_preserves_both_exact_provenance_assertions() {
    let fixture = Fixture::new(
        program(ConflictPolicy::Undecided, true, true),
        vec![vec![Cell::U64(1), Cell::Bool(true), Cell::F64(0.0)]],
        2,
    );
    let result = fixture.run(5).await.unwrap();
    assert_eq!(fixture.rows(&result, "inferred.facts").len(), 1);
    let assertions = fixture.rows(&result, "provenance.fact_assertions");
    assert_eq!(assertions.len(), 2);
    let mut values = assertions
        .iter()
        .map(|row| row[4].literal_spec())
        .collect::<Vec<_>>();
    values.sort();
    let mut expected = vec![
        Cell::F64(0.0).literal_spec(),
        Cell::F64(-0.0).literal_spec(),
    ];
    expected.sort();
    assert_eq!(values, expected);
    assert!(
        fixture
            .rows(&result, "inferred.rule_outcomes")
            .iter()
            .all(|row| row[3] == Cell::Enum("true"))
    );
    let support = fixture.rows(&result, "provenance.rule_support_edges");
    assert_eq!(support.len(), 2);
    assert_ne!(support[0][3], support[1][3]);
}
#[tokio::test]
async fn reject_conflict_aborts_whole_stratum_including_uncontested_keys() {
    let fixture = Fixture::new(
        program(ConflictPolicy::Reject, true, true),
        vec![
            vec![Cell::U64(1), Cell::Bool(true), Cell::F64(1.0)],
            vec![Cell::U64(2), Cell::Bool(true), Cell::F64(-0.0)],
        ],
        1,
    );
    assert!(matches!(
        fixture::rule_cause(&fixture.run(5).await.unwrap_err()),
        pse_rules::RuleError::Conflict { .. }
    ));
}
#[test]
fn undecided_same_stratum_consumers_and_mixed_conflict_policies_are_rejected() {
    let mut builder = program(ConflictPolicy::Undecided, false, true);
    head(
        &mut builder,
        "consumer",
        "consumer_assertions",
        &["id"],
        vec![
            id("id"),
            FieldContract::payload(
                "value",
                T::native(datafusion::arrow::datatypes::DataType::Float64),
                "Actual payload",
            ),
        ],
    );
    declare(
        &mut builder,
        RuleDecl::new(
            "consumer",
            "1",
            0,
            RuleHead::Relation("inferred.consumer".to_owned()),
            P::Scan {
                relation: "inferred.facts".to_owned(),
                port: "facts",
            },
        )
        .assertions("provenance.consumer_assertions"),
    );
    assert!(builder.build().is_err());
    let mut builder = program(ConflictPolicy::Undecided, false, true);
    declare(
        &mut builder,
        RuleDecl::new(
            "mixed",
            "1",
            0,
            RuleHead::Relation("inferred.facts".to_owned()),
            P::Project {
                input: Box::new(P::Scan {
                    relation: "authored.input".to_owned(),
                    port: "source",
                }),
                columns: vec![
                    ("id".into(), E::col("id")),
                    ("value".into(), E::col("value")),
                ],
            },
        )
        .assertions("provenance.fact_assertions"),
    );
    assert!(builder.build().is_err());
}
#[tokio::test]
async fn alternate_equal_producers_keep_support_without_duplicate_facts() {
    let fixture = Fixture::new(
        program(ConflictPolicy::Reject, true, true),
        vec![vec![Cell::U64(1), Cell::Bool(true), Cell::F64(-0.0)]],
        1,
    );
    let result = fixture.run(5).await.unwrap();
    assert_eq!(fixture.rows(&result, "inferred.facts").len(), 1);
    assert_eq!(fixture.rows(&result, "provenance.fact_assertions").len(), 2);
    assert_eq!(
        fixture.rows(&result, "provenance.rule_support_edges").len(),
        2
    );
}

#[tokio::test]
async fn lower_stratum_absence_records_complete_binding_without_invented_missing_keys() {
    let mut builder = builder();
    input(
        &mut builder,
        vec![
            id("id"),
            FieldContract::payload(
                "flag",
                T::native(datafusion::arrow::datatypes::DataType::Boolean),
                "Selection",
            ),
        ],
        &["id"],
    );
    head(
        &mut builder,
        "selected",
        "selected_assertions",
        &["id"],
        vec![id("id")],
    );
    head(
        &mut builder,
        "excluded",
        "excluded_assertions",
        &["id"],
        vec![id("id")],
    );
    declare(
        &mut builder,
        RuleDecl::new(
            "select",
            "1",
            0,
            RuleHead::Relation("inferred.selected".to_owned()),
            P::Project {
                input: Box::new(P::Filter {
                    input: Box::new(P::Scan {
                        relation: "authored.input".to_owned(),
                        port: "source",
                    }),
                    predicate: E::col("flag"),
                }),
                columns: vec![("id".into(), E::col("id"))],
            },
        )
        .assertions("provenance.selected_assertions"),
    );
    declare(
        &mut builder,
        RuleDecl::new(
            "exclude",
            "1",
            1,
            RuleHead::Relation("inferred.excluded".to_owned()),
            P::Project {
                input: Box::new(P::AntiJoin {
                    left: Box::new(P::Scan {
                        relation: "authored.input".to_owned(),
                        port: "source",
                    }),
                    right: Box::new(P::Scan {
                        relation: "inferred.selected".to_owned(),
                        port: "selected",
                    }),
                    keys: (vec![("source.id", "selected.id")])
                        .into_iter()
                        .map(|(left, right)| (left.into(), right.into()))
                        .collect(),
                }),
                columns: vec![("id".into(), E::col("source.id"))],
            },
        )
        .assertions("provenance.excluded_assertions")
        .stratified_negation(),
    );
    let fixture = Fixture::new(
        builder,
        vec![
            vec![Cell::U64(1), Cell::Bool(true)],
            vec![Cell::U64(2), Cell::Bool(false)],
        ],
        1,
    );
    let result = fixture.run(8).await.unwrap();
    assert_eq!(
        fixture.rows(&result, "inferred.selected"),
        vec![vec![Cell::U64(1)]]
    );
    assert_eq!(
        fixture.rows(&result, "inferred.excluded"),
        vec![vec![Cell::U64(2)]]
    );
    let supports = fixture.rows(&result, "provenance.rule_support_edges");
    let absence = supports
        .iter()
        .filter(|row| row[9] == Cell::Enum("absence"))
        .collect::<Vec<_>>();
    assert_eq!(absence.len(), 1);
    assert_eq!(absence[0][10], Cell::Null);
    assert_eq!(absence[0][8], Cell::Null); // completed lower workspace, no fabricated snapshot
}

#[test]
fn same_stratum_negation_is_rejected() {
    let mut builder = builder();
    input(&mut builder, vec![id("id")], &["id"]);
    head(
        &mut builder,
        "facts",
        "fact_assertions",
        &["id"],
        vec![id("id")],
    );
    declare(
        &mut builder,
        RuleDecl::new(
            "negative_recursion",
            "1",
            0,
            RuleHead::Relation("inferred.facts".to_owned()),
            P::Project {
                input: Box::new(P::AntiJoin {
                    left: Box::new(P::Scan {
                        relation: "authored.input".to_owned(),
                        port: "source",
                    }),
                    right: Box::new(P::Scan {
                        relation: "inferred.facts".to_owned(),
                        port: "facts",
                    }),
                    keys: (vec![("source.id", "facts.id")])
                        .into_iter()
                        .map(|(left, right)| (left.into(), right.into()))
                        .collect(),
                }),
                columns: vec![("id".into(), E::col("source.id"))],
            },
        )
        .assertions("provenance.fact_assertions")
        .stratified_negation(),
    );
    assert!(builder.build().is_err());
}

#[test]
fn aggregate_feedback_cannot_invent_an_unbounded_numeric_domain() {
    use pse_schema::model::{
        AggregateEmptyPolicy, AggregateNullPolicy, RuleAggregate, RuleAggregateFn,
    };
    let mut builder = builder();
    head(
        &mut builder,
        "counts",
        "count_assertions",
        &["id"],
        vec![FieldContract::key(
            "id",
            T::native(datafusion::arrow::datatypes::DataType::UInt64),
            "Count identity",
        )],
    );
    declare(
        &mut builder,
        RuleDecl::new(
            "count_feedback",
            "1",
            0,
            RuleHead::Relation("inferred.counts".to_owned()),
            P::Aggregate {
                input: Box::new(P::Scan {
                    relation: "inferred.counts".to_owned(),
                    port: "counts",
                }),
                group: vec![],
                aggregates: vec![RuleAggregate {
                    function: RuleAggregateFn::Count,
                    input: None,
                    output_name: ("id").into(),
                    order_by: vec![],
                    null_policy: AggregateNullPolicy::Reject,
                    empty_policy: AggregateEmptyPolicy::Zero,
                }],
            },
        )
        .assertions("provenance.count_assertions"),
    );
    assert!(builder.build().is_err());
}

#[tokio::test]
async fn fact_derivation_links_the_smallest_actual_assertion_and_empty_input_stays_empty() {
    for rows in [
        vec![vec![
            Cell::U64(1),
            Cell::Id(pse_ids::SemanticId::from_bytes([17; 16])),
        ]],
        vec![],
    ] {
        let mut registry = builder();
        input(
            &mut registry,
            vec![
                id("id"),
                FieldContract::payload("origin", T::id(), "Actual origin"),
            ],
            &["id"],
        );
        head(
            &mut registry,
            "linked",
            "linked_assertions",
            &["id"],
            vec![
                id("id"),
                FieldContract::provenance("derivation_id", T::id(), "Actual assertion"),
            ],
        );
        for name in ["first_producer", "second_producer"] {
            declare(
                &mut registry,
                RuleDecl::new(
                    name,
                    "1",
                    0,
                    RuleHead::Relation("inferred.linked".to_owned()),
                    P::Project {
                        input: Box::new(P::Scan {
                            relation: "authored.input".to_owned(),
                            port: "source",
                        }),
                        columns: vec![
                            ("id".into(), E::col("id")),
                            ("derivation_id".into(), E::col("origin")),
                        ],
                    },
                )
                .assertions("provenance.linked_assertions"),
            );
        }
        let populated = !rows.is_empty();
        let fixture = Fixture::new(registry, rows, 3);
        let result = fixture.run(5).await.unwrap();
        let facts = fixture.rows(&result, "inferred.linked");
        let assertions = fixture.rows(&result, "provenance.linked_assertions");
        if populated {
            assert_eq!(facts.len(), 1);
            assert_eq!(assertions.len(), 2);
            let smallest = assertions
                .iter()
                .map(|row| match row[0] {
                    Cell::Id(id) => id,
                    _ => panic!("assertion identity must be typed"),
                })
                .min()
                .unwrap();
            assert_eq!(facts[0], vec![Cell::U64(1), Cell::Id(smallest)]);
        } else {
            assert!(facts.is_empty());
            assert!(assertions.is_empty());
        }
    }
}
