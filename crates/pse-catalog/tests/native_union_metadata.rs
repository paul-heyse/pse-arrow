// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! UNION exposes only shared annotations in its actual logical and physical fields.
use datafusion::{
    arrow::array::{Array, FixedSizeBinaryArray, RecordBatch, UInt64Array},
    logical_expr::{LogicalPlanBuilder, Partitioning, col, lit},
};
use pse_columnar::CancellationToken;
use pse_engine::session::{ExecutionSettings, ThreadBudget};
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::{
    RegistryBuilder,
    model::{Authority, FieldContract, Namespace, RelationDecl, SnapshotClass},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "one semantic-field scenario follows actual values through preparation, execution and output admission"
)]
async fn union_key_and_payload_retains_shared_meaning_before_distinct() {
    let mut registry = RegistryBuilder::new();
    for (name, value) in [
        (
            "products",
            FieldContract::key("value", FieldContract::id(), "Actual product."),
        ),
        (
            "uses",
            FieldContract::payload("value", FieldContract::id(), "Used product."),
        ),
    ] {
        registry.declare_relation(
            RelationDecl::new(
                Namespace::Authored,
                name,
                1,
                Authority::Authored,
                SnapshotClass::Model,
                "UNION source.",
            )
            .pk(&["row"])
            .columns(vec![
                FieldContract::key(
                    "row",
                    FieldContract::native(datafusion::arrow::datatypes::DataType::UInt64),
                    "Row.",
                ),
                value,
            ]),
        );
    }
    let registry = Arc::new(registry.build().unwrap());
    let mut inputs = BTreeMap::new();
    for (name, values) in [("authored.products", [7_u8, 9]), ("authored.uses", [9, 11])] {
        let spec = registry.relation(name).unwrap();
        let batch = RecordBatch::try_new(
            Arc::new(pse_schema::arrow::relation_schema(&registry, spec).unwrap()),
            vec![
                Arc::new(UInt64Array::from(vec![0, 1])),
                Arc::new(
                    FixedSizeBinaryArray::try_from_iter(
                        values.into_iter().map(|value| [value; 16]),
                    )
                    .unwrap(),
                ),
            ],
        )
        .unwrap();
        inputs.insert(
            spec.key,
            FieldCheckedBatch::admit(&registry, spec, batch).unwrap(),
        );
    }
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
    .candidate_checked(inputs, registry, &cancel)
    .unwrap();
    let plans = ["authored.products", "authored.uses"].map(|name| {
        let key = session.registry().relation(name).unwrap().key;
        LogicalPlanBuilder::scan(name, session.table_source(&key).unwrap(), None)
            .unwrap()
            .project([col("value")])
            .unwrap()
            .repartition(Partitioning::RoundRobinBatch(2))
            .unwrap()
            .build()
            .unwrap()
    });
    for (left, right) in [(0, 1), (1, 0)] {
        let plan = LogicalPlanBuilder::from(plans[left].clone())
            .union(plans[right].clone())
            .unwrap()
            .distinct()
            .unwrap()
            .build()
            .unwrap();
        let result = session
            .prepare(plan, &cancel)
            .unwrap()
            .execute(&cancel)
            .await
            .unwrap();
        let mut values = BTreeSet::new();
        for batch in result.batches() {
            let field = batch.schema().field(0).clone();
            assert_eq!(
                field
                    .metadata()
                    .get("ARROW:extension:name")
                    .map(String::as_str),
                Some("pse.semantic_id")
            );
            assert!(!field.metadata().contains_key("pse.semantic.role"));
            let ids = batch
                .column(0)
                .as_any()
                .downcast_ref::<FixedSizeBinaryArray>()
                .unwrap();
            assert_eq!(ids.null_count(), 0);
            for row in 0..ids.len() {
                values.insert(ids.value(row).to_vec());
            }
        }
        assert_eq!(
            values,
            [7_u8, 9, 11]
                .map(|value| vec![value; 16])
                .into_iter()
                .collect()
        );
    }
    let key = session
        .registry()
        .relation("authored.products")
        .unwrap()
        .key;
    let rows = LogicalPlanBuilder::scan("rows", session.table_source(&key).unwrap(), None)
        .unwrap()
        .project([col("row")])
        .unwrap()
        .build()
        .unwrap();
    let literal = LogicalPlanBuilder::empty(true)
        .project([lit(2_u64).alias("row")])
        .unwrap()
        .build()
        .unwrap();
    for (left, right) in [(rows.clone(), literal.clone()), (literal, rows)] {
        let plan = LogicalPlanBuilder::from(left)
            .union(right)
            .unwrap()
            .distinct()
            .unwrap()
            .build()
            .unwrap();
        let complete = session
            .prepare(plan, &cancel)
            .unwrap()
            .execute(&cancel)
            .await
            .unwrap();
        let mut values = BTreeSet::new();
        for batch in complete.batches() {
            assert!(
                batch.schema().field(0).metadata().is_empty(),
                "an unannotated alternative cannot establish the source role"
            );
            let rows = batch
                .column(0)
                .as_any()
                .downcast_ref::<UInt64Array>()
                .unwrap();
            values.extend(rows.values().iter().copied());
        }
        assert_eq!(values, BTreeSet::from([0, 1, 2]));
    }
    // Native UNION analysis can widen ordinary primitive alternatives. Annotation
    // derivation must not reject a legal plan before the analyzer sees it.
    for (left, right) in [(lit(1_i32), lit(2_i64)), (lit(2_i64), lit(1_i32))] {
        let literal = |value: datafusion::logical_expr::Expr| {
            LogicalPlanBuilder::empty(true)
                .project([value.alias("value")])
                .unwrap()
                .build()
                .unwrap()
        };
        let plan = LogicalPlanBuilder::from(literal(left))
            .union(literal(right))
            .unwrap()
            .build()
            .unwrap();
        let result = session
            .prepare(plan, &cancel)
            .unwrap()
            .execute(&cancel)
            .await
            .unwrap();
        let values = result
            .batches()
            .iter()
            .flat_map(|batch| {
                batch
                    .column(0)
                    .as_any()
                    .downcast_ref::<datafusion::arrow::array::Int64Array>()
                    .unwrap()
                    .values()
                    .iter()
                    .copied()
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(values, BTreeSet::from([1, 2]));
    }
}
