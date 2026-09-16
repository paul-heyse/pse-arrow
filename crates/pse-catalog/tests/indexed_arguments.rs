// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! A source ordinal follows the actual Arrow row through native reorder/partitioning.
use datafusion::{
    arrow::array::{FixedSizeBinaryArray, UInt64Array},
    execution::runtime_env::RuntimeEnv,
    logical_expr::{LogicalPlanBuilder, col},
};
use pse_catalog::session::{
    ExecutionSettings, SessionFactory, ThreadBudget, native_engine_profile,
};
use pse_ids::{CancellationToken, FixedBudget, SemanticId};
use pse_relations::generated::authored::package_unit_sets;
use std::{collections::BTreeMap, sync::Arc};

#[tokio::test]
async fn indexed_argument_positions_are_captured_before_query_reordering() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let mut builder = package_unit_sets::Builder::with_registry(&registry, 2).unwrap();
    for byte in [2, 1] {
        builder
            .push(package_unit_sets::Row {
                package_id: SemanticId::from_bytes([byte; 16]),
                unit_set_id: SemanticId::from_bytes([9; 16]),
            })
            .unwrap();
    }
    let input = builder.finish().unwrap();
    let budget = FixedBudget::new(128 << 20);
    let factory = SessionFactory::new(
        Arc::new(RuntimeEnv::default()),
        budget.clone(),
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: 1.try_into().unwrap(),
            target_partitions: 4.try_into().unwrap(),
        },
        native_engine_profile(),
    )
    .unwrap();
    let cancel = CancellationToken::default();
    let session = factory
        .candidate(BTreeMap::new(), registry, &cancel)
        .unwrap()
        .with_indexed_checked_role("constructed", &input, &cancel)
        .unwrap();
    let plan = LogicalPlanBuilder::from(session.scan_computation_role("constructed").unwrap())
        .sort([col("package_id").sort(true, false)])
        .unwrap()
        .build()
        .unwrap();
    let completed = session
        .prepare(plan, &cancel)
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
    let actual = completed
        .batches()
        .iter()
        .flat_map(|batch| {
            let ids = batch
                .column_by_name("package_id")
                .unwrap()
                .as_any()
                .downcast_ref::<FixedSizeBinaryArray>()
                .unwrap();
            let ordinals = batch
                .column_by_name("constructed_row_ordinal")
                .unwrap()
                .as_any()
                .downcast_ref::<UInt64Array>()
                .unwrap();
            (0..batch.num_rows())
                .map(|row| (ids.value(row)[0], ordinals.value(row)))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, [(1, 1), (2, 0)]);
    assert!(completed.observation().physical_plan().is_some());
}
