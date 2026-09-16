// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Cyclic closure, exact support and finite resource failure through real DataFusion plans.
#[path = "support/strata_fixture.rs"]
mod fixture;
use fixture::{Fixture, builder, declare, head, id, input};
use pse_schema::model::{Cell, NullEquality, RuleDecl, RuleExpr as E, RuleHead, RulePlan as P};

fn scan(name: &str, port: &'static str) -> P {
    P::Scan {
        relation: name.to_owned(),
        port,
    }
}
fn program() -> pse_schema::RegistryBuilder {
    let mut builder = builder();
    input(&mut builder, vec![id("from"), id("to")], &["from", "to"]);
    head(
        &mut builder,
        "reach",
        "reach_assertions",
        &["origin", "reached"],
        vec![id("origin"), id("reached")],
    );
    declare(
        &mut builder,
        RuleDecl::new(
            "seed",
            "1",
            0,
            RuleHead::Relation("inferred.reach".to_owned()),
            P::Project {
                input: Box::new(scan("authored.input", "edges")),
                columns: vec![
                    ("origin".into(), E::col("from")),
                    ("reached".into(), E::col("to")),
                ],
            },
        )
        .assertions("provenance.reach_assertions"),
    );
    declare(
        &mut builder,
        RuleDecl::new(
            "step",
            "1",
            0,
            RuleHead::Relation("inferred.reach".to_owned()),
            P::Project {
                input: Box::new(P::EquiJoin {
                    left: Box::new(scan("inferred.reach", "reach")),
                    right: Box::new(scan("authored.input", "edges")),
                    keys: (vec![("reach.reached", "edges.from")])
                        .into_iter()
                        .map(|(left, right)| (left.into(), right.into()))
                        .collect(),
                    null_equality: NullEquality::NullEqualsNothing,
                }),
                columns: vec![
                    ("origin".into(), E::col("reach.origin")),
                    ("reached".into(), E::col("edges.to")),
                ],
            },
        )
        .assertions("provenance.reach_assertions"),
    );
    builder
}
fn cycle() -> Vec<Vec<Cell>> {
    vec![
        vec![Cell::U64(1), Cell::U64(2)],
        vec![Cell::U64(2), Cell::U64(3)],
        vec![Cell::U64(3), Cell::U64(1)],
    ]
}
#[tokio::test]
async fn stratified_fixed_point_cycles_retain_finite_support_and_all_reachable_pairs() {
    let fixture = Fixture::new(program(), cycle(), 1);
    let result = fixture.run(12).await.unwrap();
    let rows = fixture.rows(&result, "inferred.reach");
    let expected = (1..=3)
        .flat_map(|a| (1..=3).map(move |b| vec![Cell::U64(a), Cell::U64(b)]))
        .collect::<Vec<_>>();
    assert_eq!(rows, expected);
    assert_eq!(result.rounds[&0], 5); // Final round also settles independently discovered supports.
    let supports = fixture.rows(&result, "provenance.rule_support_edges");
    assert_eq!(supports.len(), 21); // Three seed edges, two located edges per recursive fact.
    assert!(
        supports
            .iter()
            .any(|row| row[9] == Cell::Enum("workspace") && row[8] == Cell::Null)
    );
    assert!(
        supports
            .iter()
            .any(|row| row[9] == Cell::Enum("facts") && row[8] == Cell::Null)
    );
    assert!(
        result
            .plans
            .iter()
            .any(|plan| plan.explain_pgjson().contains("Join"))
    );
}
#[tokio::test]
async fn stratified_fixed_point_shuffled_partitioned_sources_have_identical_values_and_support() {
    let first = Fixture::new(program(), cycle(), 1);
    let mut shuffled = cycle();
    shuffled.reverse();
    let second = Fixture::new(program(), shuffled, 3);
    let a = first.run(12).await.unwrap();
    let b = second.run(12).await.unwrap();
    for name in [
        "inferred.reach",
        "provenance.reach_assertions",
        "inferred.rule_outcomes",
        "provenance.rule_support_edges",
    ] {
        assert_eq!(first.rows(&a, name), second.rows(&b, name), "{name}");
    }
}
#[tokio::test]
async fn unfinished_round_budget_refuses_the_entire_stratum() {
    let fixture = Fixture::new(program(), cycle(), 1);
    assert!(matches!(
        fixture::rule_cause(&fixture.run(2).await.unwrap_err()),
        pse_rules::RuleError::ResourceLimit { .. }
    ));
}
#[tokio::test]
async fn incomplete_rule_producer_inventory_and_extra_outputs_are_refused() {
    let mut fixture = Fixture::new(program(), cycle(), 1);
    let removed = fixture.rules.pop().unwrap();
    fixture.bindings.remove(&removed.id);
    assert!(fixture.run(12).await.is_err());
}

#[tokio::test]
async fn union_preserves_distinct_branch_literals_and_widens_nullable_payloads() {
    use pse_schema::model::{FieldContract, FieldContract as T};
    let mut registry = builder();
    input(
        &mut registry,
        vec![
            id("key"),
            FieldContract::payload(
                "optional",
                T::native(datafusion::arrow::datatypes::DataType::UInt32),
                "Nullable source",
            )
            .optional(),
        ],
        &["key"],
    );
    head(
        &mut registry,
        "union_values",
        "union_value_assertions",
        &["key"],
        vec![
            id("key"),
            FieldContract::payload(
                "value",
                T::native(datafusion::arrow::datatypes::DataType::UInt32),
                "Exact payload",
            )
            .optional(),
        ],
    );
    let branch = |key, value| P::Project {
        input: Box::new(scan("authored.input", "input")),
        columns: vec![
            ("key".into(), E::Lit(Cell::U64(key))),
            ("value".into(), value),
        ],
    };
    declare(
        &mut registry,
        RuleDecl::new(
            "union_values",
            "1",
            0,
            RuleHead::Relation("inferred.union_values".into()),
            P::Union(vec![
                branch(1, E::col("optional")),
                branch(2, E::col("key")),
            ]),
        )
        .assertions("provenance.union_value_assertions"),
    );
    let fixture = Fixture::new(registry, vec![vec![Cell::U64(7), Cell::Null]], 2);
    let result = fixture.run(4).await.unwrap();
    assert_eq!(
        fixture.rows(&result, "inferred.union_values"),
        vec![
            vec![Cell::U64(1), Cell::Null],
            vec![Cell::U64(2), Cell::U64(7)]
        ]
    );
}

#[tokio::test]
async fn integer_head_narrowing_preserves_values_and_rejects_overflow() {
    use pse_schema::model::{FieldContract, FieldContract as T};
    let program = || {
        let mut registry = builder();
        input(
            &mut registry,
            vec![FieldContract::key(
                "key",
                T::native(datafusion::arrow::datatypes::DataType::UInt64),
                "Exact wide input",
            )],
            &["key"],
        );
        head(
            &mut registry,
            "narrow",
            "narrow_assertions",
            &["key"],
            vec![FieldContract::key(
                "key",
                T::native(datafusion::arrow::datatypes::DataType::UInt8),
                "Checked small key",
            )],
        );
        declare(
            &mut registry,
            RuleDecl::new(
                "narrow",
                "1",
                0,
                RuleHead::Relation("inferred.narrow".into()),
                scan("authored.input", "input"),
            )
            .assertions("provenance.narrow_assertions"),
        );
        registry
    };
    let exact = Fixture::new(program(), vec![vec![Cell::U64(255)]], 1);
    assert_eq!(
        exact.rows(&exact.run(4).await.unwrap(), "inferred.narrow"),
        vec![vec![Cell::U64(255)]]
    );
    let overflow = Fixture::new(program(), vec![vec![Cell::U64(256)]], 1);
    assert!(overflow.run(4).await.is_err());
}

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "one completion lifecycle covers retention and changed input refusal"
)]
async fn completed_program_retains_actual_rows_and_refuses_changed_workspace_values() {
    use pse_ids::CancellationToken;
    use pse_rules::strata::{
        LocatedRuleInput, RuleBindings, RuleInputLocation, StratumLimits, execute_strata,
    };
    use std::{
        collections::{BTreeMap, BTreeSet},
        num::NonZeroU32,
    };
    let mut registry = builder();
    input(&mut registry, vec![id("key")], &["key"]);
    head(
        &mut registry,
        "first",
        "first_assertions",
        &["key"],
        vec![id("key")],
    );
    head(
        &mut registry,
        "second",
        "second_assertions",
        &["key"],
        vec![id("key")],
    );
    declare(
        &mut registry,
        RuleDecl::new(
            "first",
            "1",
            0,
            RuleHead::Relation("inferred.first".into()),
            scan("authored.input", "input"),
        )
        .assertions("provenance.first_assertions"),
    );
    declare(
        &mut registry,
        RuleDecl::new(
            "second",
            "1",
            1,
            RuleHead::Relation("inferred.second".into()),
            scan("inferred.first", "completed"),
        )
        .assertions("provenance.second_assertions"),
    );
    let fixture = Fixture::new(registry, vec![vec![Cell::U64(7)]], 1);
    let first = fixture.registry.rule("first@1").unwrap().clone();
    let second = fixture.registry.rule("second@1").unwrap().clone();
    let outputs = |head, assertions| {
        [
            head,
            assertions,
            "inferred.rule_outcomes",
            "provenance.rule_support_edges",
        ]
        .into_iter()
        .map(|name| fixture.registry.relation(name).unwrap().key)
        .collect::<BTreeSet<_>>()
    };
    let cancel = CancellationToken::new();
    let limits = StratumLimits {
        max_rounds: NonZeroU32::new(4).unwrap(),
    };
    let mut result = execute_strata(
        std::slice::from_ref(&first),
        &BTreeMap::from([(first.id, fixture.bindings[&first.id].clone())]),
        &fixture.session,
        &fixture.registry,
        &outputs("inferred.first", "provenance.first_assertions"),
        limits,
        &cancel,
    )
    .await
    .unwrap();
    let declaration = fixture.registry.relation("inferred.first").unwrap();
    let actual = result.completed.relation(declaration.key).unwrap();
    assert!(
        result
            .completed
            .relation(fixture.registry.relation("inferred.second").unwrap().key)
            .is_err()
    );
    let altered = pse_relations::cells::batch_from_cells(
        &fixture.registry,
        declaration,
        &[vec![Cell::U64(8)]],
    )
    .unwrap();
    result.relations.insert(declaration.key, altered.clone());
    let producer = std::sync::Arc::downgrade(&result.completed);
    drop(result);
    assert!(producer.upgrade().is_none());
    assert_eq!(
        pse_relations::cells::cells_from_batch(&fixture.registry, declaration, actual.batch())
            .unwrap(),
        vec![vec![Cell::U64(7)]]
    );
    let bindings = BTreeMap::from([(
        second.id,
        RuleBindings {
            ports: BTreeMap::from([(
                "completed".into(),
                LocatedRuleInput {
                    relation: declaration.key,
                    location: RuleInputLocation::Completed(actual.clone()),
                },
            )]),
        },
    )]);
    let wrong = fixture
        .session
        .with_workspace(BTreeMap::from([(declaration.key, altered)]), &cancel)
        .unwrap();
    assert!(
        execute_strata(
            std::slice::from_ref(&second),
            &bindings,
            &wrong,
            &fixture.registry,
            &outputs("inferred.second", "provenance.second_assertions"),
            limits,
            &cancel
        )
        .await
        .is_err()
    );
    let session = fixture
        .session
        .with_workspace(
            BTreeMap::from([(declaration.key, actual.batch().clone())]),
            &cancel,
        )
        .unwrap();
    let output = execute_strata(
        &[second],
        &bindings,
        &session,
        &fixture.registry,
        &outputs("inferred.second", "provenance.second_assertions"),
        limits,
        &cancel,
    )
    .await
    .unwrap();
    assert_eq!(
        fixture.rows(&output, "inferred.second"),
        vec![vec![Cell::U64(7)]]
    );
    assert!(
        fixture
            .rows(&output, "provenance.rule_support_edges")
            .iter()
            .any(|row| row[9] == Cell::Enum("completed") && row[8] == Cell::Null)
    );
}
