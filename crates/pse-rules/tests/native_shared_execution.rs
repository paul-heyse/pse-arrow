// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Shared native computations keep actual payloads and each source occurrence.
#[path = "support/strata_fixture.rs"]
mod fixture;
#[path = "support/row_key.rs"]
mod row_key;

use fixture::{Fixture, builder, declare, head, id, input};
use pse_schema::model::{
    Cell, ConflictPolicy, FieldContract, NullEquality, RuleDecl, RuleExpr as E, RuleHead,
    RulePlan as P,
};
use std::collections::BTreeSet;

fn scan(port: &'static str) -> P {
    P::Scan {
        relation: "authored.input".into(),
        port,
    }
}

fn program(plan: P) -> pse_schema::RegistryBuilder {
    let mut registry = builder();
    input(
        &mut registry,
        vec![
            id("witness"),
            id("id"),
            id("group"),
            FieldContract::payload(
                "value",
                FieldContract::native(datafusion::arrow::datatypes::DataType::Float64),
                "Exact source payload",
            ),
        ],
        &["witness"],
    );
    head(
        &mut registry,
        "facts",
        "fact_assertions",
        &["id"],
        vec![
            id("id"),
            FieldContract::payload(
                "value",
                FieldContract::native(datafusion::arrow::datatypes::DataType::Float64),
                "Actual result payload",
            ),
        ],
    );
    declare(
        &mut registry,
        RuleDecl::new(
            "shared",
            "1",
            0,
            RuleHead::Relation("inferred.facts".into()),
            plan,
        )
        .assertions("provenance.fact_assertions")
        .stratified_negation()
        .conflicts(ConflictPolicy::Undecided),
    );
    registry
}

#[tokio::test]
async fn joined_fanout_keeps_qualified_columns_and_every_actual_witness() {
    let joined = P::EquiJoin {
        left: Box::new(scan("left")),
        right: Box::new(scan("right")),
        keys: vec![("left.group".into(), "right.group".into())],
        null_equality: NullEquality::NullEqualsNothing,
    };
    let plan = P::Union(
        ["left", "right"]
            .into_iter()
            .map(|port| P::Project {
                input: Box::new(joined.clone()),
                columns: vec![
                    ("id".into(), E::col(format!("{port}.id"))),
                    ("value".into(), E::col(format!("{port}.value"))),
                ],
            })
            .collect(),
    );
    let rows = vec![
        vec![Cell::U64(10), Cell::U64(1), Cell::U64(7), Cell::F64(0.0)],
        vec![Cell::U64(20), Cell::U64(2), Cell::U64(7), Cell::F64(-0.0)],
    ];
    let fixture = Fixture::new(program(plan), rows.clone(), 2);
    let result = fixture.run(4).await.unwrap();
    let facts = fixture.rows(&result, "inferred.facts");
    assert_eq!(facts.len(), 2);
    for (fact, source) in facts.iter().zip(&rows) {
        assert_eq!(fact[0], source[1]);
        assert_eq!(fact[1].literal_spec(), source[3].literal_spec());
    }
    let source = fixture.registry.relation("authored.input").unwrap();
    let rule = &fixture.rules[0];
    let mut expected = BTreeSet::new();
    for id in [1, 2] {
        for port in ["left", "right"] {
            for row in &rows {
                expected.insert((
                    row_key::rule(rule, &fixture.registry, &[Cell::U64(id)]).await,
                    port.to_owned(),
                    row_key::relation(source, &fixture.registry, row).await,
                ));
            }
        }
    }
    let supports = fixture.rows(&result, "provenance.rule_support_edges");
    assert_eq!(supports.len(), 8);
    let actual = supports
        .into_iter()
        .map(|row| {
            assert_eq!(row[7], Cell::Id(source.id));
            assert_eq!(row[9], Cell::Enum("facts"));
            let (Cell::Hash(output), Cell::Text(port), Cell::Hash(input)) =
                (&row[5], &row[6], &row[10])
            else {
                panic!("support keys must be typed tokens")
            };
            (*output, port.clone(), *input)
        })
        .collect();
    assert_eq!(expected, actual);
    let declaration = fixture.registry.relation("provenance.derivations").unwrap();
    let mut source_keys = vec![];
    for row in &rows {
        source_keys.push(row_key::relation(source, &fixture.registry, row).await);
    }
    source_keys.sort();
    let ordered_support = Cell::List(
        source_keys
            .into_iter()
            .map(|key| Cell::Struct(vec![Cell::Id(source.id), Cell::Hash(key)]))
            .collect(),
    );
    let derivations = result
        .derivations
        .iter()
        .flat_map(|batch| {
            pse_relations::cells::cells_from_batch(&fixture.registry, declaration, batch).unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(derivations.len(), 2);
    for row in derivations {
        assert_eq!(row[5], ordered_support);
    }
    assert!(
        result
            .plans
            .iter()
            .any(|plan| plan.explain_pgjson().contains("__pse_shared_column_"))
    );
}

#[tokio::test]
async fn shared_literal_plans_do_not_equate_opposite_signed_zero() {
    let literal = |value| P::Project {
        input: Box::new(scan("source")),
        columns: vec![
            ("id".into(), E::col("id")),
            ("value".into(), E::Lit(Cell::F64(value))),
        ],
    };
    let positive = literal(0.0);
    let negative = literal(-0.0);
    let fixture = Fixture::new(
        program(P::Union(vec![
            positive.clone(),
            negative.clone(),
            positive,
            negative,
        ])),
        vec![vec![
            Cell::U64(10),
            Cell::U64(1),
            Cell::U64(7),
            Cell::F64(1.0),
        ]],
        2,
    );
    let result = fixture.run(4).await.unwrap();
    let assertions = fixture.rows(&result, "provenance.fact_assertions");
    assert_eq!(assertions.len(), 2);
    let actual = assertions
        .iter()
        .map(|row| row[4].literal_spec())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        actual,
        [Cell::F64(0.0), Cell::F64(-0.0)]
            .iter()
            .map(Cell::literal_spec)
            .collect()
    );
    assert_eq!(
        fixture.rows(&result, "provenance.rule_support_edges").len(),
        2
    );
}

#[tokio::test]
async fn empty_shared_lookup_keeps_its_exact_negative_scope() {
    let empty = P::Filter {
        input: Box::new(scan("blocked")),
        predicate: E::Lit(Cell::Bool(false)),
    };
    let plan = P::Project {
        input: Box::new(P::AntiJoin {
            left: Box::new(scan("actual")),
            right: Box::new(P::Union(vec![empty.clone(), empty])),
            keys: vec![("actual.id".into(), "id".into())],
        }),
        columns: vec![
            ("id".into(), E::col("actual.id")),
            ("value".into(), E::col("actual.value")),
        ],
    };
    let fixture = Fixture::new(
        program(plan),
        vec![vec![
            Cell::U64(10),
            Cell::U64(1),
            Cell::U64(7),
            Cell::F64(3.0),
        ]],
        2,
    );
    let result = fixture.run(4).await.unwrap();
    assert_eq!(
        fixture.rows(&result, "inferred.facts"),
        vec![vec![Cell::U64(1), Cell::F64(3.0)]]
    );
    let supports = fixture.rows(&result, "provenance.rule_support_edges");
    assert_eq!(supports.len(), 2);
    let absent = supports
        .iter()
        .find(|row| row[6] == Cell::Text("blocked".into()))
        .unwrap();
    assert_eq!(absent[9], Cell::Enum("absence"));
    assert_eq!(absent[10], Cell::Null);
    assert_eq!(absent[8], Cell::Null);
    let actual = supports
        .iter()
        .find(|row| row[6] == Cell::Text("actual".into()))
        .unwrap();
    assert_eq!(actual[9], Cell::Enum("facts"));
    assert!(matches!(actual[10], Cell::Hash(_)));
}
