// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Tiny in-memory witnesses; no compiler, persistence or solver journey.
#![allow(clippy::unwrap_used, clippy::expect_used, reason = "unit assertions")]
use super::*;
use datafusion::arrow::{
    array::{Array, Int64Array},
    datatypes::DataType,
};
use pse_catalog::session::{
    ExecutionSettings, SnapshotSession, ThreadBudget, build_candidate_session,
    native_engine_profile,
};
use pse_ids::{CancellationToken, FixedBudget};
use pse_schema::{
    RegistryBuilder,
    model::{
        Authority, Cell, DependencyMode, DerivationGranularity, FieldContract as F, Namespace,
        RelationDecl, RuleDecl, RuleInput, SnapshotClass,
    },
};
use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc};

fn fixture(sql: &str, negative: bool) -> (Arc<Registry>, SnapshotSession) {
    let mut builder = RegistryBuilder::new();
    for (name, fields) in [
        ("rows", vec!["id", "bucket", "amount"]),
        ("matches", vec!["id", "bucket"]),
    ] {
        builder.declare_relation(
            RelationDecl::new(
                Namespace::Authored,
                name,
                1,
                Authority::Authored,
                SnapshotClass::Model,
                "unit input",
            )
            .pk(&["id"])
            .columns(
                fields
                    .iter()
                    .map(|field| F::payload(field, F::native(DataType::Int64), "unit value"))
                    .collect(),
            ),
        );
    }
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Inferred,
            "result",
            1,
            Authority::Derived,
            SnapshotClass::Derived,
            "unit output",
        )
        .pk(&["bucket"])
        .columns(vec![
            F::payload("bucket", F::native(DataType::Int64), "group"),
            F::payload("total", F::native(DataType::Int64), "measure"),
        ])
        .granularity(DerivationGranularity::Rule),
    );
    let mut inputs = vec![RuleInput {
        relation: "authored.rows".into(),
        port: "r",
        mode: DependencyMode::Read,
    }];
    if negative {
        inputs.push(RuleInput {
            relation: "authored.matches".into(),
            port: "m",
            mode: if sql.contains("ANTI") {
                DependencyMode::Negate
            } else {
                DependencyMode::Read
            },
        });
    }
    let rule = RuleDecl::new("unit", "1", 0, "inferred.result", sql, inputs);
    builder.declare_rule(if negative {
        rule.stratified_negation()
    } else {
        rule
    });
    let registry = Arc::new(builder.build().unwrap());
    let mut batches = BTreeMap::new();
    for (name, values) in [
        (
            "authored.rows",
            vec![vec![1, 1, 10], vec![2, 1, 20], vec![3, 2, 7]],
        ),
        ("authored.matches", vec![vec![4, 2]]),
    ] {
        let spec = registry.relation(name).unwrap();
        let rows = values
            .into_iter()
            .map(|values| values.into_iter().map(Cell::I64).collect())
            .collect::<Vec<_>>();
        batches.insert(
            spec.key,
            pse_relations::cells::batch_from_cells(&registry, spec, &rows).unwrap(),
        );
    }
    let one = NonZeroUsize::new(1).unwrap();
    let session = build_candidate_session(
        batches,
        registry.clone(),
        Arc::new(datafusion::execution::runtime_env::RuntimeEnv::default()),
        FixedBudget::new(32 << 20),
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: one,
            target_partitions: one,
        },
        native_engine_profile(),
    )
    .unwrap();
    (registry, session)
}
async fn witnesses(sql: &str, negative: bool) -> Vec<(bool, Vec<Vec<i64>>)> {
    witnesses_at_depth(sql, negative, 0).await
}

async fn witnesses_at_depth(sql: &str, negative: bool, depth: usize) -> Vec<(bool, Vec<Vec<i64>>)> {
    let (registry, session) = fixture(sql, negative);
    let rule = registry.rule("unit").unwrap();
    let bindings = crate::plan::PortBinding {
        ports: rule
            .inputs
            .iter()
            .map(|input| {
                (
                    input.port.to_owned(),
                    registry.relation(&input.relation).unwrap().key,
                )
            })
            .collect(),
    };
    let compiled = crate::plan::compile(rule, &bindings, &session, &registry)
        .await
        .unwrap();
    let mut plan = compiled.plan;
    for _ in 0..depth {
        let fields = columns(&plan);
        plan = LogicalPlan::Projection(
            datafusion_expr::logical_plan::Projection::try_new(fields, Arc::new(plan)).unwrap(),
        );
    }
    let mut result = Vec::new();
    for trace in
        compile_support(&plan, rule, &registry, &session, &CancellationToken::new()).unwrap()
    {
        let completed = session
            .prepare_rule_plan(trace.plan, &CancellationToken::new())
            .unwrap()
            .execute(&CancellationToken::new())
            .await
            .unwrap();
        let mut rows = Vec::new();
        for batch in completed.into_batches() {
            for row in 0..batch.num_rows() {
                rows.push(
                    batch
                        .columns()
                        .iter()
                        .map(|column| {
                            let column = column.as_any().downcast_ref::<Int64Array>().unwrap();
                            assert!(!column.is_null(row));
                            column.value(row)
                        })
                        .collect(),
                );
            }
        }
        rows.sort();
        result.push((trace.source.absence, rows));
    }
    result.sort();
    result
}
#[tokio::test]
async fn aggregate_witnesses_keep_exact_members_without_changing_the_sum() {
    assert_eq!(
        witnesses(
            "SELECT r.bucket, sum(r.amount) AS total FROM authored.rows r GROUP BY r.bucket",
            false
        )
        .await,
        vec![(false, vec![vec![1, 30, 1], vec![1, 30, 2], vec![2, 7, 3]])]
    );
}
#[tokio::test]
async fn anti_join_keeps_matching_left_rows_and_exact_absence_scope() {
    assert_eq!(witnesses("SELECT r.bucket, r.amount AS total FROM authored.rows r LEFT ANTI JOIN authored.matches m ON r.bucket=m.bucket",true).await,
        vec![(false,vec![vec![1,10,1],vec![1,20,2]]),(true,vec![vec![1,10],vec![1,20]])]);
}

#[tokio::test]
async fn window_witnesses_retain_partition_dependency_without_recomputing_values() {
    assert_eq!(witnesses("SELECT r.id AS bucket, sum(r.amount) OVER (PARTITION BY r.bucket) AS total FROM authored.rows r",false).await,
        vec![(false,vec![vec![1,30,1],vec![1,30,2],vec![2,30,1],vec![2,30,2],vec![3,7,3]])]);
}

#[tokio::test]
async fn outer_join_separates_actual_keys_from_unmatched_scope_absence() {
    assert_eq!(witnesses("SELECT r.id AS bucket, coalesce(m.id, 0) AS total FROM authored.rows r LEFT JOIN authored.matches m ON r.bucket=m.bucket",true).await,
        vec![(false,vec![vec![1,0,1],vec![2,0,2],vec![3,4,3]]),
            (false,vec![vec![3,4,4]]), (true,vec![vec![1,0],vec![2,0]])]);
}

#[tokio::test]
async fn filters_and_union_keep_only_actual_branch_witnesses() {
    assert_eq!(witnesses("SELECT r.id AS bucket, r.amount AS total FROM authored.rows r WHERE r.id = 1 UNION ALL SELECT r.id AS bucket, r.amount AS total FROM authored.rows r WHERE r.id = 3", false).await,
        vec![(false, vec![vec![1,10,1]]), (false, vec![vec![3,7,3]])]);
}

#[tokio::test]
async fn unnest_repeats_the_source_key_with_each_actual_nested_value() {
    assert_eq!(witnesses("SELECT r.id AS bucket, unnest(make_array(r.amount, r.amount + 1)) AS total FROM authored.rows r WHERE r.id = 1", false).await,
        vec![(false, vec![vec![1,10,1], vec![1,11,1]])]);
}

#[tokio::test]
async fn deep_native_projection_graph_keeps_exact_witnesses_without_recursive_recovery() {
    assert_eq!(
        witnesses_at_depth(
            "SELECT r.bucket, r.amount AS total FROM authored.rows r",
            false,
            256,
        )
        .await,
        vec![(false, vec![vec![1, 10, 1], vec![1, 20, 2], vec![2, 7, 3]])],
    );
}
