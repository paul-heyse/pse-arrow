// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Selected values retain semantic fields through native partial/final aggregation.

use datafusion::{
    arrow::array::{Array, FixedSizeBinaryArray, FixedSizeBinaryBuilder, RecordBatch, UInt64Array},
    logical_expr::{LogicalPlanBuilder, Partitioning, col, lit},
};
use pse_columnar::CancellationToken;
use pse_engine::session::{ExecutionSettings, ThreadBudget};
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::{
    RegistryBuilder,
    model::{Authority, FieldContract, Namespace, RelationDecl, SnapshotClass},
};
use std::{collections::BTreeMap, sync::Arc};

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "one semantic-field scenario follows actual values through preparation, execution and output admission"
)]
async fn selected_ids_retain_values_metadata_and_empty_group_nullability() {
    let mut registry = RegistryBuilder::new();
    registry.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "selected_ids",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "Actual optional semantic IDs.",
        )
        .pk(&["row"])
        .columns(vec![
            FieldContract::key(
                "row",
                FieldContract::native(datafusion::arrow::datatypes::DataType::UInt64),
                "Input row.",
            ),
            FieldContract::payload("value", FieldContract::id(), "Selected value.").optional(),
        ]),
    );
    let registry = Arc::new(registry.build().unwrap());
    let spec = registry.relation("authored.selected_ids").unwrap();
    let key = spec.key;
    let schema = Arc::new(pse_schema::arrow::relation_schema(&registry, spec).unwrap());
    let metadata = schema.field(1).metadata().clone();
    let mut values = FixedSizeBinaryBuilder::with_capacity(4, 16);
    for value in [Some(7), None, Some(3), Some(9)] {
        if let Some(value) = value {
            values.append_value([value; 16]).unwrap();
        } else {
            values.append_null();
        }
    }
    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(UInt64Array::from(vec![0, 1, 2, 3])),
            Arc::new(values.finish()),
        ],
    )
    .unwrap();
    let checked = FieldCheckedBatch::admit(&registry, spec, batch).unwrap();
    let cancel = CancellationToken::new();
    let session = pse_testkit::factory(
        Arc::new(pse_columnar::GreedyMemoryPool::new(64 << 20)),
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: 1.try_into().unwrap(),
            target_partitions: 2.try_into().unwrap(),
        },
    )
    .unwrap()
    .candidate_checked(BTreeMap::from([(key, checked)]), registry, &cancel)
    .unwrap();
    let input = LogicalPlanBuilder::scan("values", session.table_source(&key).unwrap(), None)
        .unwrap()
        .repartition(Partitioning::RoundRobinBatch(2))
        .unwrap()
        .build()
        .unwrap();
    for mode in ["values", "nulls", "empty", "groups"] {
        let groups = if mode == "groups" {
            vec![col("row")]
        } else {
            vec![]
        };
        let input = LogicalPlanBuilder::from(input.clone())
            .filter(match mode {
                "empty" => lit(false),
                "nulls" => col("value").is_null(),
                _ => lit(true),
            })
            .unwrap();
        let plan = input
            .aggregate(
                groups,
                [
                    datafusion::functions_aggregate::expr_fn::min(col("value")).alias("minimum"),
                    datafusion::functions_aggregate::expr_fn::max(col("value")).alias("maximum"),
                ],
            )
            .unwrap()
            .build()
            .unwrap();
        let prepared = session.prepare(plan, &cancel).unwrap();
        let complete = prepared.execute(&cancel).await.unwrap();
        let mut rows = Vec::new();
        for batch in complete.batches() {
            let minimum = batch
                .column_by_name("minimum")
                .unwrap()
                .as_any()
                .downcast_ref::<FixedSizeBinaryArray>()
                .unwrap();
            let maximum = batch
                .column_by_name("maximum")
                .unwrap()
                .as_any()
                .downcast_ref::<FixedSizeBinaryArray>()
                .unwrap();
            for name in ["minimum", "maximum"] {
                let schema = batch.schema();
                let field = schema.field_with_name(name).unwrap();
                assert_eq!(field.metadata(), &metadata, "{mode}");
                assert!(field.is_nullable());
            }
            for row in 0..batch.num_rows() {
                rows.push((
                    (!minimum.is_null(row)).then(|| minimum.value(row)[0]),
                    (!maximum.is_null(row)).then(|| maximum.value(row)[0]),
                ));
            }
        }
        rows.sort();
        assert_eq!(
            rows,
            match mode {
                "values" => vec![(Some(3), Some(9))],
                "groups" => vec![
                    (None, None),
                    (Some(3), Some(3)),
                    (Some(7), Some(7)),
                    (Some(9), Some(9))
                ],
                _ => vec![(None, None)],
            },
            "{mode}"
        );
    }
}
