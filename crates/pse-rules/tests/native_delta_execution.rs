// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Input deltas and fresh affected-stratum recomputation through the public executor.

#[path = "support/strata_fixture.rs"]
mod fixture;
use fixture::{Fixture, builder, declare, head, id, input};
use pse_schema::model::{Cell, FieldContract, RuleDecl};

fn recursive_program() -> pse_schema::RegistryBuilder {
    let mut registry = builder();
    input(
        &mut registry,
        vec![
            id("witness"),
            id("from"),
            id("to"),
            FieldContract::payload(
                "enabled",
                FieldContract::native(datafusion::arrow::datatypes::DataType::Boolean),
                "Actual seed membership",
            ),
        ],
        &["witness"],
    );
    head(
        &mut registry,
        "reach",
        "reach_assertions",
        &["origin", "reached"],
        vec![id("origin"), id("reached")],
    );
    declare(
        &mut registry,
        RuleDecl::new(
            "seed",
            "1",
            0,
            "inferred.reach",
            r#"SELECT "from" AS origin, "to" AS reached FROM authored.input WHERE enabled"#,
            vec![fixture::read("authored.input", "seed")],
        )
        .assertions("provenance.reach_assertions"),
    );
    declare(
        &mut registry,
        RuleDecl::new(
            "self_join",
            "1",
            0,
            "inferred.reach",
            r#"SELECT "left".origin, "right".reached FROM inferred.reach AS "left" JOIN inferred.reach AS "right" ON "left".reached = "right".origin"#,
            vec![fixture::read("inferred.reach", "left"), fixture::read("inferred.reach", "right")],
        )
        .assertions("provenance.reach_assertions"),
    );
    registry
}

fn edge(witness: u64, from: u64, to: u64) -> Vec<Cell> {
    vec![
        Cell::U64(witness),
        Cell::U64(from),
        Cell::U64(to),
        Cell::Bool(true),
    ]
}

#[tokio::test]
async fn self_join_replaces_one_input_occurrence_and_preserves_new_old_matches() {
    let rows = (1..=5).map(|from| edge(from, from, from + 1)).collect();
    let fixture = Fixture::new(recursive_program(), rows, 3);
    let result = fixture.run(16).await.unwrap();
    let expected = (1..=5)
        .flat_map(|from| ((from + 1)..=6).map(move |to| vec![Cell::U64(from), Cell::U64(to)]))
        .collect::<Vec<_>>();
    assert_eq!(fixture.rows(&result, "inferred.reach"), expected);
    // Length-three paths need a new length-two edge joined with an old length-one
    // edge. Substituting both occurrences with delta loses those exact results.
    assert!(
        fixture
            .rows(&result, "inferred.reach")
            .contains(&vec![Cell::U64(1), Cell::U64(4)])
    );
    assert!(
        result
            .plans
            .iter()
            .any(|plan| plan.explain_pgjson().contains("__pse_delta"))
    );
}

#[tokio::test]
async fn support_removal_recomputes_cyclic_closure_without_self_supported_facts() {
    let both = Fixture::new(
        recursive_program(),
        vec![edge(1, 1, 2), edge(2, 1, 2), edge(3, 2, 1)],
        1,
    );
    let original = both.run(16).await.unwrap();
    assert_eq!(both.rows(&original, "inferred.reach").len(), 4);
    let fewer = Fixture::new(recursive_program(), vec![edge(2, 1, 2), edge(3, 2, 1)], 1);
    let retained = fewer.run(16).await.unwrap();
    assert_eq!(
        fewer.rows(&retained, "inferred.reach"),
        both.rows(&original, "inferred.reach")
    );
    assert_eq!(
        both.rows(&original, "provenance.rule_support_edges").len(),
        fewer.rows(&retained, "provenance.rule_support_edges").len() + 1
    );
    let removed = Fixture::new(recursive_program(), vec![], 1);
    let empty = removed.run(16).await.unwrap();
    assert!(removed.rows(&empty, "inferred.reach").is_empty());
    assert!(
        removed
            .rows(&empty, "provenance.rule_support_edges")
            .is_empty()
    );
    // The old immutable result remains valid in its original context, while no old
    // assertion or support is imported into the fresh affected program.
    assert_eq!(both.rows(&original, "inferred.reach").len(), 4);
}

fn negative_scope_program() -> pse_schema::RegistryBuilder {
    let mut registry = recursive_program();
    head(
        &mut registry,
        "missing",
        "missing_assertions",
        &["witness"],
        vec![id("witness")],
    );
    declare(
        &mut registry,
        RuleDecl::new(
            "missing",
            "1",
            1,
            "inferred.missing",
            r#"SELECT source.witness FROM authored.input AS source LEFT ANTI JOIN inferred.reach AS closure ON source."from" = closure.origin AND source."to" = closure.reached"#,
            vec![fixture::read("authored.input", "source"), fixture::negate("inferred.reach", "closure")],
        )
        .assertions("provenance.missing_assertions")
        .stratified_negation(),
    );
    registry
}

#[tokio::test]
async fn changed_source_recomputes_the_complete_downstream_negative_scope() {
    let old = Fixture::new(negative_scope_program(), vec![edge(1, 1, 2)], 1);
    let before = old.run(16).await.unwrap();
    assert!(old.rows(&before, "inferred.missing").is_empty());
    let mut changed = edge(1, 1, 2);
    changed[3] = Cell::Bool(false);
    let new = Fixture::new(negative_scope_program(), vec![changed], 1);
    let after = new.run(16).await.unwrap();
    assert!(new.rows(&after, "inferred.reach").is_empty());
    assert_eq!(
        new.rows(&after, "inferred.missing"),
        vec![vec![Cell::U64(1)]]
    );
    let supports = new.rows(&after, "provenance.rule_support_edges");
    let absence = supports
        .iter()
        .filter(|row| row[9] == Cell::Enum("absence"))
        .collect::<Vec<_>>();
    assert_eq!(absence.len(), 1);
    assert_eq!(absence[0][10], Cell::Null);
    assert!(old.rows(&before, "inferred.missing").is_empty());
}

#[tokio::test]
async fn a_later_round_producer_conflict_aborts_before_the_next_stratum() {
    let mut registry = builder();
    input(&mut registry, vec![id("id")], &["id"]);
    head(
        &mut registry,
        "delay",
        "delay_assertions",
        &["id"],
        vec![id("id")],
    );
    let values = vec![
        id("id"),
        FieldContract::payload(
            "value",
            FieldContract::native(datafusion::arrow::datatypes::DataType::UInt32),
            "Exact fact value",
        ),
    ];
    head(
        &mut registry,
        "facts",
        "facts_assertions",
        &["id"],
        values.clone(),
    );
    head(
        &mut registry,
        "consumer",
        "consumer_assertions",
        &["id"],
        values,
    );
    declare(
        &mut registry,
        RuleDecl::new(
            "delay",
            "1",
            0,
            "inferred.delay",
            "SELECT id FROM authored.input",
            vec![fixture::read("authored.input", "input")],
        )
        .assertions("provenance.delay_assertions"),
    );
    for (name, source, value) in [
        ("first", "authored.input", 1),
        ("late", "inferred.delay", 2),
    ] {
        declare(
            &mut registry,
            RuleDecl::new(
                name,
                "1",
                0,
                "inferred.facts",
                format!("SELECT id, CAST({value} AS INT UNSIGNED) AS value FROM {source}"),
                vec![fixture::read(source, "input")],
            )
            .assertions("provenance.facts_assertions"),
        );
    }
    declare(
        &mut registry,
        RuleDecl::new(
            "consumer",
            "1",
            1,
            "inferred.consumer",
            "SELECT id, value FROM inferred.facts",
            vec![fixture::read("inferred.facts", "facts")],
        )
        .assertions("provenance.consumer_assertions"),
    );
    let fixture = Fixture::new(registry, vec![vec![Cell::U64(1)]], 1);
    let error = fixture.run(8).await.unwrap_err();
    let pse_rules::RuleError::Conflict {
        stratum,
        assertions,
        ..
    } = fixture::rule_cause(&error)
    else {
        panic!("competing finite assertions must reject the stratum");
    };
    assert_eq!(*stratum, 0);
    assert_eq!(assertions.len(), 2);
    for value in [1, 2] {
        assert!(
            assertions
                .iter()
                .any(|row| row.ends_with(&format!(",[\"u64\",{value}]]"))),
            "the diagnostic retains the actual competing payload: {assertions:?}"
        );
    }
}
