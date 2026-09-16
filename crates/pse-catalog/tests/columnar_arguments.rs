// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Co-located support survives native operators and retains its Arrow ownership.
#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "typed Arrow test fixtures"
)]
use datafusion::{
    arrow::{
        array::{
            Array, FixedSizeBinaryArray, Int64Array, ListArray, RecordBatch, StructArray,
            builder::{FixedSizeBinaryBuilder, ListBuilder, StringBuilder, StructBuilder},
        },
        datatypes::{DataType, Field, Schema},
    },
    execution::runtime_env::RuntimeEnv,
    logical_expr::{LogicalPlanBuilder, col, lit},
};
use pse_catalog::session::{
    ExecutionSettings, SessionFactory, ThreadBudget, native_engine_profile,
};
use pse_ids::{CancellationToken, FixedBudget, owned_buffer::OwnedRecordBatch};
use pse_schema::{
    Registry,
    model::{FieldContract, field::SOURCE_SUPPORT_COLUMN},
};
use std::{collections::BTreeMap, sync::Arc};

fn argument(registry: &Registry, empty: bool, duplicate: bool) -> RecordBatch {
    let support = pse_schema::arrow::field_for(registry, &FieldContract::source_support()).unwrap();
    let DataType::List(child) = support.data_type() else {
        panic!("list")
    };
    let DataType::Struct(fields) = child.data_type() else {
        panic!("struct")
    };
    let mut lists = ListBuilder::with_capacity(StructBuilder::from_fields(fields.clone(), 0), 0)
        .with_field(Arc::clone(child));
    for value in [2, 1, 3] {
        for _ in 0..if empty {
            0
        } else if duplicate {
            2
        } else {
            1
        } {
            let members = lists.values();
            members
                .field_builder::<StringBuilder>(0)
                .unwrap()
                .append_value("source");
            members
                .field_builder::<FixedSizeBinaryBuilder>(1)
                .unwrap()
                .append_value(
                    pse_relations::generated::authored::package_unit_sets::RELATION_ID.as_bytes(),
                )
                .unwrap();
            members
                .field_builder::<FixedSizeBinaryBuilder>(2)
                .unwrap()
                .append_value([value; 32])
                .unwrap();
            members.append(true);
        }
        lists.append(true);
    }
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new("value", DataType::Int64, false),
            support,
        ])),
        vec![
            Arc::new(Int64Array::from(vec![2, 1, 3])),
            Arc::new(lists.finish()),
        ],
    )
    .unwrap()
}

#[tokio::test]
async fn support_stays_with_values_across_filter_union_sort_and_owner_drop() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let budget = FixedBudget::new(128 << 20);
    let cancel = CancellationToken::default();
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
    let base = factory
        .candidate(BTreeMap::new(), registry.clone(), &cancel)
        .unwrap();
    for (empty, duplicate) in [(true, false), (false, true)] {
        let data = OwnedRecordBatch::export(
            argument(&registry, empty, duplicate),
            budget.as_ref(),
            &cancel,
        )
        .unwrap();
        assert!(
            base.with_columnar_argument("malformed", data, &cancel)
                .await
                .is_err()
        );
    }
    let input =
        OwnedRecordBatch::export(argument(&registry, false, false), budget.as_ref(), &cancel)
            .unwrap();
    let session = base
        .with_columnar_argument("constructed", input, &cancel)
        .await
        .unwrap();
    drop(base);
    assert!(
        session
            .computation_source("constructed")
            .unwrap()
            .constraints()
            .is_none_or(|keys| keys.is_empty())
    );
    let scan = session.scan_computation_role("constructed").unwrap();
    let plan = LogicalPlanBuilder::from(scan.clone())
        .union(scan)
        .unwrap()
        .filter(col("value").gt(lit(1_i64)))
        .unwrap()
        .sort([col("value").sort(true, false)])
        .unwrap()
        .build()
        .unwrap();
    let completed = session
        .prepare(plan, &cancel)
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
    let batches = completed.batches().to_vec();
    drop(completed);
    drop(session);
    drop(factory);
    assert!(budget.reserved() > 0);
    let mut actual = Vec::new();
    for batch in &batches {
        let values = batch
            .column_by_name("value")
            .unwrap()
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap();
        let lists = batch
            .column_by_name(SOURCE_SUPPORT_COLUMN)
            .unwrap()
            .as_any()
            .downcast_ref::<ListArray>()
            .unwrap();
        for row in 0..batch.num_rows() {
            let members = lists.value(row);
            let members = members.as_any().downcast_ref::<StructArray>().unwrap();
            let keys = members
                .column_by_name("source_key")
                .unwrap()
                .as_any()
                .downcast_ref::<FixedSizeBinaryArray>()
                .unwrap();
            actual.push((values.value(row), keys.value(0)[0]));
        }
    }
    assert_eq!(actual, [(2, 2), (2, 2), (3, 3), (3, 3)]);
    drop(batches);
    assert_eq!(budget.reserved(), 0);
}
