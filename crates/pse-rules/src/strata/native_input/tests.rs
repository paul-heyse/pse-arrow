// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;
use datafusion::{
    arrow::datatypes::DataType,
    execution::runtime_env::RuntimeEnv,
    logical_expr::{ColumnarValue, Volatility, create_udf, lit},
};
use pse_catalog::session::{
    ExecutionSettings, SessionFactory, ThreadBudget, native_engine_profile,
};
use pse_schema::model::{Authority, FieldContract, Namespace, RelationDecl, SnapshotClass};
use std::sync::atomic::{AtomicUsize, Ordering};

#[tokio::test]
async fn constraints_check_one_completed_execution_and_never_export_conflicting_values() {
    let mut builder = pse_schema::RegistryBuilder::new();
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "observed",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "Actual values from one execution",
        )
        .columns(vec![FieldContract::key(
            "key",
            FieldContract::native(DataType::Int64),
            "Exact native key",
        )])
        .pk(&["key"]),
    );
    let registry = Arc::new(builder.build().unwrap());
    let target = registry.relation("authored.observed").unwrap();
    let factory = SessionFactory::new(
        Arc::new(RuntimeEnv::default()),
        pse_ids::FixedBudget::new(64 << 20),
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: 1.try_into().unwrap(),
            target_partitions: 1.try_into().unwrap(),
        },
        native_engine_profile(),
    )
    .unwrap();
    let cancel = CancellationToken::new();
    let session = factory
        .candidate(BTreeMap::new(), Arc::clone(&registry), &cancel)
        .unwrap();
    for (second, conflicts) in [(2_i64, false), (1_i64, true)] {
        let calls = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&calls);
        let function = create_udf(
            "observe_once",
            vec![DataType::Int64],
            DataType::Int64,
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
        let plan = LogicalPlanBuilder::values(vec![vec![lit(1_i64)], vec![lit(second)]])
            .unwrap()
            .project([scalar::require_nonnull(function.call(vec![col("column1")])).alias("key")])
            .unwrap()
            .build()
            .unwrap();
        let result = materialize_with_constraint(
            plan,
            target,
            &session,
            &cancel,
            |values| witness::duplicate_keys(values, target),
            "conflicting completed keys",
        )
        .await;
        assert_eq!(
            calls.load(Ordering::SeqCst),
            2,
            "constraint checks must not execute the producer again"
        );
        if conflicts {
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("conflicting completed keys")
            );
        } else {
            assert_eq!(result.unwrap().batch().num_rows(), 2);
        }
    }
}
