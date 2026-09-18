// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual native completion and located witnesses replace row replay callbacks.
#![allow(clippy::unwrap_used, reason = "fixed native plan and witness fixtures")]
#[path = "support/strata_fixture.rs"]
mod fixture;
use datafusion_expr::{LogicalPlan, LogicalPlanBuilder, col, lit, when};
use fixture::{Fixture, builder, declare, head, id, input};
use pse_ids::{CancellationToken, SemanticId};
use pse_rules::strata::{
    LocatedRuleInput, RuleInputLocation,
    native_input::{NativeInput, NativeWitness},
};
use pse_schema::model::{Cell, FieldContract, RuleDecl};
use std::{collections::BTreeMap, sync::Arc};

fn fixture() -> Fixture {
    let mut registry = builder();
    input(&mut registry, vec![id("id")], &["id"]);
    head(
        &mut registry,
        "native_source",
        "unused_native_assertions",
        &["id"],
        vec![
            id("id"),
            FieldContract::payload(
                "value",
                FieldContract::native(datafusion::arrow::datatypes::DataType::UInt32),
                "Actual expression output",
            ),
            FieldContract::provenance(
                "derivation_id",
                FieldContract::id(),
                "Actual native pass derivation",
            ),
        ],
    );
    head(
        &mut registry,
        "selected",
        "selected_assertions",
        &["id"],
        vec![id("id")],
    );
    declare(
        &mut registry,
        RuleDecl::new(
            "native_consumer",
            "1",
            0,
            "inferred.selected",
            "SELECT id FROM inferred.native_source",
            vec![fixture::read("inferred.native_source", "native")],
        )
        .assertions("provenance.selected_assertions"),
    );
    Fixture::new(registry, vec![vec![Cell::U64(1)], vec![Cell::U64(2)]], 1)
}

fn witness(fixture: &Fixture, keys: Option<Vec<String>>) -> NativeWitness {
    NativeWitness {
        when: None,
        port: "actual_source".to_owned(),
        key_columns: keys,
        input: LocatedRuleInput {
            relation: fixture.registry.relation("authored.input").unwrap().key,
            location: RuleInputLocation::Facts(Arc::clone(&fixture.facts)),
        },
    }
}

fn source(fixture: &Fixture, bad_key: bool, conflict: bool) -> LogicalPlan {
    let key = fixture.registry.relation("authored.input").unwrap().key;
    let scan =
        LogicalPlanBuilder::scan("source", fixture.session.table_source(&key).unwrap(), None)
            .unwrap()
            .build()
            .unwrap();
    LogicalPlanBuilder::from(scan)
        .project(vec![
            if conflict {
                lit(1_u32).alias("id")
            } else {
                col("id")
            },
            col("id").alias("value"),
            if bad_key {
                lit(99_u32).alias("source:key")
            } else {
                col("id").alias("source:key")
            },
        ])
        .unwrap()
        .build()
        .unwrap()
}

#[tokio::test]
async fn singleton_witness_requires_a_row_and_scope_remains_explicit() {
    use datafusion::arrow::array::{Array, FixedSizeBinaryArray, StringArray};

    for (source_rows, positive, output_rows, succeeds) in [
        (0, true, 1, false),
        (1, true, 1, true),
        (0, false, 1, true),
        (1, true, 2, false),
        (1, true, 0, true),
    ] {
        let mut registry = builder();
        let fields = vec![FieldContract::payload(
            "value",
            FieldContract::native(datafusion::arrow::datatypes::DataType::UInt32),
            "Singleton value",
        )];
        input(&mut registry, fields.clone(), &[]);
        head(
            &mut registry,
            "singleton",
            "singleton_assertions",
            &[],
            fields,
        );
        let fixture = Fixture::new(registry, vec![vec![Cell::U64(7)]; source_rows], 1);
        let target = fixture.registry.relation("inferred.singleton").unwrap();
        let mut plan = LogicalPlanBuilder::empty(output_rows > 0)
            .project([lit(9_u32).alias("value")])
            .unwrap()
            .build()
            .unwrap();
        if output_rows == 2 {
            let second = LogicalPlanBuilder::empty(true)
                .project([lit(10_u32).alias("value")])
                .unwrap()
                .build()
                .unwrap();
            plan = LogicalPlanBuilder::from(plan)
                .union(second)
                .unwrap()
                .build()
                .unwrap();
        }
        let result = NativeInput::build(
            plan,
            target.key,
            SemanticId::NIL,
            vec![("value".into(), "value".into())],
            vec![witness(&fixture, positive.then(Vec::new))],
            &fixture.session,
            &CancellationToken::new(),
        )
        .await;
        assert_eq!(
            result.is_ok(),
            succeeds,
            "source={source_rows}, positive={positive}, output={output_rows}: {result:?}"
        );
        if let Ok(native) = result {
            assert_eq!(native.batch().num_rows(), output_rows);
            let mapping = native.support_mapping().batch();
            let keys = mapping
                .column_by_name("input_key")
                .unwrap()
                .as_any()
                .downcast_ref::<FixedSizeBinaryArray>()
                .unwrap();
            let kinds = mapping
                .column_by_name("support_kind")
                .unwrap()
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap();
            if positive {
                assert_eq!(mapping.num_rows(), output_rows);
            } else {
                assert!(mapping.num_rows() > 0);
            }
            for row in 0..mapping.num_rows() {
                assert_eq!(keys.is_valid(row), positive);
                assert_eq!(kinds.value(row) == "absence", !positive);
            }
        }
    }
}

#[tokio::test]
async fn native_output_without_embedded_derivation_retains_each_actual_source() {
    let fixture = fixture();
    let target = fixture.registry.relation("inferred.selected").unwrap();
    let native = NativeInput::build(
        source(&fixture, false, false),
        target.key,
        SemanticId::NIL,
        vec![("id".into(), "id".into())],
        vec![witness(&fixture, Some(vec!["source:key".into()]))],
        &fixture.session,
        &CancellationToken::new(),
    )
    .await
    .unwrap();
    let first = native.derivations().clone();
    let second = native.derivations().clone();
    let declaration = fixture.registry.relation("provenance.derivations").unwrap();
    let rows =
        pse_relations::cells::cells_from_batch(&fixture.registry, declaration, first.batch())
            .unwrap();
    assert_eq!(rows.len(), 2);
    let mut ids = std::collections::BTreeSet::new();
    for row in &rows {
        assert_eq!(row[1], Cell::Id(target.id));
        assert_eq!(row[4], Cell::Id(SemanticId::NIL));
        ids.insert(row[0].literal_spec());
        let Cell::List(support) = &row[5] else {
            panic!("missing exact support")
        };
        assert_eq!(support.len(), 1);
    }
    assert_eq!(ids.len(), 2);
    assert_eq!(
        rows,
        pse_relations::cells::cells_from_batch(&fixture.registry, declaration, second.batch(),)
            .unwrap()
    );
    let predecessor = Arc::downgrade(&fixture.facts);
    drop(fixture);
    assert!(predecessor.upgrade().is_none());
    assert_eq!(native.batch().num_rows(), 2);
    assert_eq!(native.derivations().batch().num_rows(), 2);
}

#[tokio::test]
async fn native_construction_retains_exact_row_and_explicit_absence_support() {
    let mut fixture = fixture();
    let key = fixture
        .registry
        .relation("inferred.native_source")
        .unwrap()
        .key;
    let native = NativeInput::build(
        source(&fixture, false, false),
        key,
        SemanticId::NIL,
        vec![
            ("id".to_owned(), "id".to_owned()),
            ("value".to_owned(), "value".to_owned()),
            ("derivation_id".to_owned(), "ignored_placeholder".to_owned()),
        ],
        vec![
            witness(&fixture, Some(vec!["source:key".to_owned()])),
            witness(&fixture, None),
        ],
        &fixture.session,
        &CancellationToken::new(),
    )
    .await
    .unwrap();
    let derivations = native.derivations().clone();
    assert_eq!(derivations.batch().num_rows(), 2);
    let declaration = fixture.registry.relation("provenance.derivations").unwrap();
    let evidence =
        pse_relations::cells::cells_from_batch(&fixture.registry, declaration, derivations.batch())
            .unwrap();
    for row in evidence {
        assert_eq!(row[4], Cell::Id(SemanticId::NIL));
        let Cell::List(supporting) = &row[5] else {
            panic!("expected exact row support");
        };
        assert_eq!(supporting.len(), 1);
    }
    fixture.session = fixture
        .session
        .with_checked_workspace(
            BTreeMap::from([(key, native.checked().clone())]),
            &CancellationToken::new(),
        )
        .unwrap();
    fixture
        .bindings
        .values_mut()
        .next()
        .unwrap()
        .ports
        .get_mut("native")
        .unwrap()
        .location = RuleInputLocation::Native(native);
    let outcome = fixture.run(5).await.unwrap();
    assert_eq!(
        fixture.rows(&outcome, "inferred.selected"),
        vec![vec![Cell::U64(1)], vec![Cell::U64(2)]]
    );
    let support = fixture.rows(&outcome, "provenance.rule_support_edges");
    assert_eq!(support.len(), 4);
    assert_eq!(
        support.iter().filter(|row| row[10] != Cell::Null).count(),
        2
    );
    assert!(
        support
            .iter()
            .all(|row| row[6] == Cell::text("actual_source"))
    );
}

#[tokio::test]
async fn native_construction_refuses_false_witnesses_and_conflicting_complete_keys() {
    let fixture = fixture();
    let key = fixture
        .registry
        .relation("inferred.native_source")
        .unwrap()
        .key;
    for (false_witness, conflict) in [(true, false), (false, true)] {
        assert!(
            NativeInput::build(
                source(&fixture, false_witness, conflict),
                key,
                SemanticId::NIL,
                vec![
                    ("id".to_owned(), "id".to_owned()),
                    ("value".to_owned(), "value".to_owned()),
                    ("derivation_id".to_owned(), "ignored_placeholder".to_owned())
                ],
                vec![witness(&fixture, Some(vec!["source:key".to_owned()]))],
                &fixture.session,
                &CancellationToken::new()
            )
            .await
            .is_err()
        );
    }
}

#[tokio::test]
async fn witness_conditions_are_evaluated_once_beside_their_actual_output_rows() {
    use datafusion::arrow::{array::StringArray, datatypes::DataType};
    use datafusion_expr::{ColumnarValue, Volatility, create_udf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    let fixture = fixture();
    let key = fixture
        .registry
        .relation("inferred.native_source")
        .unwrap()
        .key;
    let rows_seen = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&rows_seen);
    let function = create_udf(
        "observed_witness_key",
        vec![DataType::UInt32],
        DataType::UInt32,
        Volatility::Volatile,
        Arc::new(move |values| {
            let rows = match &values[0] {
                ColumnarValue::Array(values) => values.len(),
                ColumnarValue::Scalar(_) => 1,
            };
            observed.fetch_add(rows, Ordering::SeqCst);
            Ok(values[0].clone())
        }),
    );
    let mut conditional = witness(&fixture, Some(vec!["source:key".into()]));
    conditional.when = Some(function.call(vec![col("id")]).eq(lit(1_u32)));
    let native = NativeInput::build(
        source(&fixture, false, false),
        key,
        SemanticId::NIL,
        vec![
            ("id".into(), "id".into()),
            ("value".into(), "value".into()),
            ("derivation_id".into(), "ignored_placeholder".into()),
        ],
        vec![conditional, witness(&fixture, None)],
        &fixture.session,
        &CancellationToken::new(),
    )
    .await
    .unwrap();
    assert_eq!(rows_seen.load(Ordering::SeqCst), 2);
    assert_eq!(native.batch().num_rows(), 2);
    let kinds = native
        .support_mapping()
        .batch()
        .column_by_name("support_kind")
        .unwrap()
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();
    assert_eq!(
        kinds.iter().filter(|kind| *kind == Some("facts")).count(),
        1
    );
    assert!(kinds.iter().any(|kind| kind == Some("absence")));
}

#[tokio::test]
async fn optional_witnesses_check_only_selected_rows_and_union_retains_all_branches() {
    let fixture = fixture();
    let key = fixture
        .registry
        .relation("inferred.native_source")
        .unwrap()
        .key;
    let mut branches = Vec::new();
    for selected in [1_u32, 2] {
        let plan = LogicalPlanBuilder::from(source(&fixture, false, false))
            .filter(col("id").eq(lit(selected)))
            .unwrap()
            .build()
            .unwrap();
        let mut source = witness(&fixture, Some(vec!["source:key".to_owned()]));
        source.when = Some(col("id").eq(lit(selected)));
        let native = NativeInput::build(
            plan,
            key,
            SemanticId::NIL,
            vec![
                ("id".into(), "id".into()),
                ("value".into(), "value".into()),
                ("derivation_id".into(), "ignored".into()),
            ],
            vec![source, witness(&fixture, None)],
            &fixture.session,
            &CancellationToken::new(),
        )
        .await
        .unwrap();
        branches.push(native);
    }
    let union = NativeInput::union(&branches, &fixture.session, &CancellationToken::new())
        .await
        .unwrap();
    let predecessors = branches.iter().map(Arc::downgrade).collect::<Vec<_>>();
    drop(branches);
    assert!(predecessors.iter().all(|input| input.upgrade().is_none()));
    assert_eq!(union.batch().num_rows(), 2);
    assert_eq!(union.derivations().batch().num_rows(), 2);
    let mut false_source = witness(&fixture, Some(vec!["source:key".to_owned()]));
    false_source.when = Some(col("id").eq(lit(1_u32)));
    assert!(
        NativeInput::build(
            source(&fixture, true, false),
            key,
            SemanticId::NIL,
            vec![
                ("id".into(), "id".into()),
                ("value".into(), "value".into()),
                ("derivation_id".into(), "ignored".into())
            ],
            vec![false_source],
            &fixture.session,
            &CancellationToken::new()
        )
        .await
        .is_err()
    );
}

#[tokio::test]
async fn conditional_witness_ignores_unmatched_placeholder_and_checks_matched_value() {
    let fixture = fixture();
    let key = fixture
        .registry
        .relation("inferred.native_source")
        .unwrap()
        .key;
    let plan = LogicalPlanBuilder::from(source(&fixture, false, false))
        .project(vec![
            col("id"),
            col("value"),
            when(col("id").eq(lit(1_u32)), col("source:key"))
                .otherwise(lit(99_u32))
                .unwrap()
                .alias("choice_key"),
        ])
        .unwrap()
        .build()
        .unwrap();
    let mut first = witness(&fixture, Some(vec!["choice_key".into()]));
    first.when = Some(col("id").eq(lit(1_u32)));
    let mut second = witness(&fixture, Some(vec!["value".into()]));
    second.when = Some(col("id").eq(lit(2_u32)));
    let columns = vec![
        ("id".into(), "id".into()),
        ("value".into(), "value".into()),
        ("derivation_id".into(), "ignored".into()),
    ];
    let complete = NativeInput::build(
        plan.clone(),
        key,
        SemanticId::NIL,
        columns.clone(),
        vec![first.clone(), second],
        &fixture.session,
        &CancellationToken::new(),
    )
    .await
    .unwrap();
    assert_eq!(complete.derivations().batch().num_rows(), 2);
    first.when = Some(col("id").eq(lit(2_u32)));
    assert!(
        NativeInput::build(
            plan,
            key,
            SemanticId::NIL,
            columns,
            vec![first],
            &fixture.session,
            &CancellationToken::new()
        )
        .await
        .is_err()
    );
}

#[tokio::test]
async fn distinct_native_source_roles_retain_their_actual_row_membership() {
    let fixture = fixture();
    let cancel = CancellationToken::new();
    let target = fixture.registry.relation("inferred.selected").unwrap();
    let mut sources = Vec::new();
    for value in [1_u32, 2_u32] {
        let plan = LogicalPlanBuilder::from(source(&fixture, false, false))
            .filter(col("id").eq(lit(value)))
            .unwrap()
            .build()
            .unwrap();
        sources.push(
            NativeInput::build(
                plan,
                target.key,
                SemanticId::NIL,
                vec![("id".into(), "id".into())],
                vec![witness(&fixture, Some(vec!["source:key".into()]))],
                &fixture.session,
                &cancel,
            )
            .await
            .unwrap(),
        );
    }
    let owner = fixture
        .session
        .with_checked_role_inputs(
            BTreeMap::from([
                ("first".into(), sources[0].checked().clone()),
                ("second".into(), sources[1].checked().clone()),
            ]),
            &cancel,
        )
        .unwrap();
    let first = LogicalPlanBuilder::from(owner.scan_role("first").unwrap())
        .project(vec![col("id"), lit(1_u32).alias("branch")])
        .unwrap()
        .build()
        .unwrap();
    let second = LogicalPlanBuilder::from(owner.scan_role("second").unwrap())
        .project(vec![col("id"), lit(2_u32).alias("branch")])
        .unwrap()
        .build()
        .unwrap();
    let plan = LogicalPlanBuilder::from(first)
        .union(second)
        .unwrap()
        .build()
        .unwrap();
    for false_witness in [false, true] {
        let witnesses = sources
            .iter()
            .enumerate()
            .map(|(index, source)| NativeWitness {
                port: format!("role:{index}"),
                input: LocatedRuleInput {
                    relation: target.key,
                    location: RuleInputLocation::Native(Arc::clone(source)),
                },
                key_columns: Some(vec!["id".into()]),
                when: Some(col("branch").eq(lit(if false_witness {
                    2 - u32::try_from(index).unwrap()
                } else {
                    u32::try_from(index).unwrap() + 1
                }))),
            })
            .collect();
        let output = NativeInput::build(
            plan.clone(),
            target.key,
            SemanticId::NIL,
            vec![("id".into(), "id".into())],
            witnesses,
            &owner,
            &cancel,
        )
        .await;
        if false_witness {
            assert!(
                output.is_err(),
                "a row from another role is not source membership"
            );
        } else {
            let output = output.unwrap();
            assert_eq!(output.batch().num_rows(), 2);
            let derived = output.derivations().clone();
            assert_eq!(derived.batch().num_rows(), 2);
        }
    }
}
