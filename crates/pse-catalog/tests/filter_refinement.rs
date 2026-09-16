// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native filtering establishes a successful nonnull result without replacing values.
use datafusion::{
    common::ScalarValue,
    execution::runtime_env::RuntimeEnv,
    logical_expr::{LogicalPlanBuilder, col, lit},
};
use pse_catalog::session::{
    ExecutionSettings, SessionFactory, ThreadBudget, native_engine_profile,
    scalar::refine_filtered_fields,
};
use pse_ids::{CancellationToken, FixedBudget};
use std::{collections::BTreeMap, sync::Arc};

#[tokio::test]
async fn filtered_nulls_are_dropped_and_remaining_values_keep_native_nonnull_schema() {
    let registry = Arc::new(pse_schema::RegistryBuilder::new().build().unwrap());
    let factory = SessionFactory::new(
        Arc::new(RuntimeEnv::default()),
        FixedBudget::new(64 << 20),
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: 1.try_into().unwrap(),
            target_partitions: 2.try_into().unwrap(),
        },
        native_engine_profile(),
    )
    .unwrap();
    let cancel = CancellationToken::default();
    let session = factory
        .candidate(BTreeMap::new(), registry, &cancel)
        .unwrap();
    let values = LogicalPlanBuilder::values(vec![
        vec![lit(ScalarValue::Int64(None))],
        vec![lit(7_i64)],
        vec![lit(-3_i64)],
    ])
    .unwrap()
    .build()
    .unwrap();
    let filtered = LogicalPlanBuilder::from(values.clone())
        .filter(col("column1").is_not_null())
        .unwrap()
        .build()
        .unwrap();
    let refined = refine_filtered_fields(filtered).unwrap();
    assert!(!refined.schema().field(0).is_nullable());
    let completed = session
        .prepare(refined, &cancel)
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
    let count: usize = completed
        .batches()
        .iter()
        .map(|batch| batch.num_rows())
        .sum();
    assert_eq!(count, 2);
    assert!(
        completed
            .batches()
            .iter()
            .all(|batch| !batch.schema().field(0).is_nullable())
    );
    assert!(completed.observation().physical_plan().is_some());
    // Disjunction does not establish that every surviving value is present.
    let disjunction = LogicalPlanBuilder::from(values)
        .filter(col("column1").is_not_null().or(lit(true)))
        .unwrap()
        .build()
        .unwrap();
    assert!(
        refine_filtered_fields(disjunction)
            .unwrap()
            .schema()
            .field(0)
            .is_nullable()
    );
}

#[tokio::test]
async fn checked_nonnull_kernel_refuses_a_null_even_without_a_filter_premise() {
    let registry = Arc::new(pse_schema::RegistryBuilder::new().build().unwrap());
    let factory = SessionFactory::new(
        Arc::new(RuntimeEnv::default()),
        FixedBudget::new(64 << 20),
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: 1.try_into().unwrap(),
            target_partitions: 1.try_into().unwrap(),
        },
        native_engine_profile(),
    )
    .unwrap();
    let cancel = CancellationToken::default();
    let session = factory
        .candidate(BTreeMap::new(), registry, &cancel)
        .unwrap();
    let function = session.scalar_function("pse_require_nonnull").unwrap();
    let plan = LogicalPlanBuilder::values(vec![vec![lit(ScalarValue::Int64(None))]])
        .unwrap()
        .project(vec![function.call(vec![col("column1")])])
        .unwrap()
        .build()
        .unwrap();
    match session.prepare(plan, &cancel) {
        Err(_) => {} // Native constant folding may execute the checked scalar first.
        Ok(prepared) => assert!(prepared.execute(&cancel).await.is_err()),
    }
}
