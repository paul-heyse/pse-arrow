// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::unwrap_used,
    reason = "explicit Arrow fixture construction and result assertions"
)]

//! Native container meaning follows actual primitive children, never opaque storage.

use datafusion::arrow::datatypes::{DataType, Field};
use pse_catalog::session::output::check_field_output;
use pse_schema::model::FieldContract;
use std::sync::Arc;

#[test]
fn primitive_nested_labels_are_inferred_but_semantic_children_are_not() {
    let registry = pse_schema::RegistryBuilder::new().build().unwrap();
    let primitive = pse_schema::arrow::field_for(
        &registry,
        &FieldContract::payload(
            "values",
            FieldContract::list(FieldContract::native(DataType::Int64)),
            "Primitive array",
        ),
    )
    .unwrap();
    let native = Field::new(
        "values",
        DataType::List(Arc::new(Field::new("item", DataType::Int64, false))),
        false,
    );
    assert!(check_field_output(&native, &primitive).is_ok());
    let semantic = pse_schema::arrow::field_for(
        &registry,
        &FieldContract::payload(
            "values",
            FieldContract::list(FieldContract::id()),
            "Semantic array",
        ),
    )
    .unwrap();
    let blobs = Field::new(
        "values",
        DataType::List(Arc::new(Field::new(
            "item",
            DataType::FixedSizeBinary(16),
            false,
        ))),
        false,
    );
    assert!(check_field_output(&blobs, &semantic).is_err());
    let nullable_child = Field::new(
        "values",
        DataType::List(Arc::new(Field::new("item", DataType::Int64, true))),
        false,
    );
    assert!(check_field_output(&nullable_child, &primitive).is_err());
}

use datafusion::{
    arrow::{
        array::{Array, FixedSizeBinaryArray, ListArray},
        buffer::OffsetBuffer,
    },
    common::{ScalarValue, config::ConfigOptions},
    execution::runtime_env::RuntimeEnv,
    logical_expr::{ColumnarValue, LogicalPlanBuilder, ScalarFunctionArgs, lit},
};
use pse_catalog::session::{
    ExecutionSettings, SessionFactory, SnapshotSession, ThreadBudget, native_engine_profile,
};
use pse_ids::{CancellationToken, FixedBudget};
use std::collections::BTreeMap;

fn session() -> SnapshotSession {
    use pse_schema::model::{Authority, Namespace, RelationDecl, SnapshotClass};
    let mut registry = pse_schema::RegistryBuilder::new();
    registry.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "list_values",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "Nested identity fixture.",
        )
        .pk(&["id"])
        .columns(vec![
            FieldContract::key("id", FieldContract::native(DataType::UInt64), "Key."),
            FieldContract::payload(
                "ids",
                FieldContract::list(FieldContract::id()),
                "Established identity children.",
            ),
        ]),
    );
    registry.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "nested_values",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "Nested selection input.",
        )
        .pk(&["id"])
        .columns(vec![
            FieldContract::key("id", FieldContract::native(DataType::UInt64), "Key."),
            FieldContract::payload(
                "nested",
                FieldContract::list(FieldContract::list(FieldContract::id())),
                "Nested identity children.",
            ),
        ]),
    );
    let registry = Arc::new(registry.build().unwrap());
    SessionFactory::new(
        Arc::new(RuntimeEnv::default()),
        FixedBudget::new(64 << 20),
        ExecutionSettings::default(),
        ThreadBudget {
            pool_threads: 1.try_into().unwrap(),
            target_partitions: 2.try_into().unwrap(),
        },
        native_engine_profile(),
    )
    .unwrap()
    .candidate_checked(BTreeMap::new(), registry, &CancellationToken::new())
    .unwrap()
}
fn declared_list(session: &SnapshotSession) -> Field {
    pse_schema::arrow::field_for(
        session.registry(),
        &FieldContract::payload(
            "ids",
            FieldContract::list(FieldContract::id()),
            "Established identity children.",
        ),
    )
    .unwrap()
}
fn list(field: Arc<Field>) -> Arc<ListArray> {
    let values =
        FixedSizeBinaryArray::try_from_iter([&[7_u8; 16], &[9_u8; 16]].into_iter()).unwrap();
    Arc::new(ListArray::new(
        field,
        OffsetBuffer::new(vec![0, 2].into()),
        Arc::new(values),
        None,
    ))
}
fn invoke(
    session: &SnapshotSession,
    value: ColumnarValue,
    expected: Field,
    rows: usize,
) -> datafusion::common::Result<ColumnarValue> {
    let expected = Arc::new(expected);
    session
        .scalar_function("pse_preserve_field")
        .unwrap()
        .invoke_with_args(ScalarFunctionArgs {
            args: vec![value],
            arg_fields: vec![Arc::clone(&expected)],
            number_rows: rows,
            return_field: expected,
            config_options: Arc::new(ConfigOptions::default()),
        })
}

#[tokio::test]
async fn native_array_selection_retains_child_fields_and_index_null_behavior() {
    use datafusion::functions_nested::expr_fn::array_element;
    use pse_catalog::session::output::checked_literal;
    let session = session();
    let field = declared_list(&session);
    let DataType::List(child) = field.data_type() else {
        panic!("ID list");
    };
    let value = list(Arc::clone(child));
    let ids = checked_literal(
        session.registry(),
        &FieldContract::payload(
            "ids",
            FieldContract::list(FieldContract::id()),
            "Actual IDs.",
        ),
        ScalarValue::List(Arc::clone(&value)),
    )
    .unwrap();
    let nested_column = FieldContract::payload(
        "nested",
        FieldContract::list(FieldContract::list(FieldContract::id())),
        "Nested IDs.",
    );
    let nested_field = pse_schema::arrow::field_for(session.registry(), &nested_column).unwrap();
    let DataType::List(nested_child) = nested_field.data_type() else {
        panic!("nested list");
    };
    let nested = ListArray::try_new(
        Arc::clone(nested_child),
        OffsetBuffer::new(vec![0, 1].into()),
        value,
        None,
    )
    .unwrap();
    let nested = checked_literal(
        session.registry(),
        &nested_column,
        ScalarValue::List(Arc::new(nested)),
    )
    .unwrap();
    let tuple = session
        .scalar_function("pse_index_tuple")
        .unwrap()
        .call(vec![ids.clone()]);
    let plan = LogicalPlanBuilder::values(vec![vec![lit(1_u64)], vec![lit(2_u64)]])
        .unwrap()
        .project([
            array_element(ids.clone(), lit(1_i64)).alias("first"),
            array_element(ids.clone(), lit(-1_i64)).alias("last"),
            array_element(ids, lit(3_i64)).alias("absent"),
            array_element(nested, lit(1_i64)).alias("nested"),
            array_element(tuple, lit(1_i64)).alias("tuple_member"),
        ])
        .unwrap()
        .build()
        .unwrap();
    let cancel = CancellationToken::new();
    let completed = session
        .prepare(plan, &cancel)
        .unwrap()
        .execute(&cancel)
        .await
        .unwrap();
    let mut count = 0;
    for batch in completed.batches() {
        for (column, expected) in [(0, 7), (1, 9)] {
            let array = batch
                .column(column)
                .as_any()
                .downcast_ref::<FixedSizeBinaryArray>()
                .unwrap();
            assert_eq!(batch.schema().field(column).metadata(), child.metadata());
            for row in 0..array.len() {
                assert_eq!(array.value(row), &[expected; 16]);
            }
        }
        assert_eq!(batch.column(2).null_count(), batch.num_rows());
        assert_eq!(batch.column(3).data_type(), field.data_type());
        assert_eq!(
            batch
                .schema()
                .field(4)
                .metadata()
                .get(pse_schema::arrow::KEY_EXTENSION_NAME)
                .map(String::as_str),
            Some("pse.semantic_id")
        );
        let members = batch
            .column(4)
            .as_any()
            .downcast_ref::<FixedSizeBinaryArray>()
            .unwrap();
        for row in 0..members.len() {
            assert_eq!(members.value(row), &[7; 16]);
        }
        count += batch.num_rows();
    }
    assert_eq!(count, 2);
}

#[tokio::test]
async fn empty_identity_collection_keeps_its_child_field_after_folding() {
    let session = session();
    let field = declared_list(&session);
    let empty = pse_catalog::session::scalar::id_list(vec![]);
    let concat = session
        .scalar_function("pse_array_concat")
        .unwrap()
        .call(vec![empty.clone(), empty.clone()]);
    let tuple = session
        .scalar_function("pse_index_tuple")
        .unwrap()
        .call(vec![empty.clone()]);
    let nullable = session
        .scalar_function("pse_nullable")
        .unwrap()
        .call(vec![empty.clone()]);
    let coalesced =
        datafusion::functions::core::expr_fn::coalesce(vec![empty.clone(), empty.clone()]);
    let plan = LogicalPlanBuilder::values(vec![vec![lit(1_u64)], vec![lit(2_u64)]])
        .unwrap()
        .project([
            empty.alias("ids"),
            concat.alias("concatenated"),
            tuple.alias("index"),
            nullable.alias("nullable_ids"),
            coalesced.alias("coalesced"),
        ])
        .unwrap()
        .build()
        .unwrap();
    let cancel = CancellationToken::new();
    let prepared = session.prepare(plan, &cancel).unwrap();
    assert_eq!(
        prepared.optimized_plan().schema().field(0).data_type(),
        field.data_type()
    );
    let completed = prepared.execute(&cancel).await.unwrap();
    for batch in completed.batches() {
        assert_eq!(batch.column(0).data_type(), field.data_type());
    }
}

#[tokio::test]
async fn nested_scalar_materialization_is_visible_after_constant_folding() {
    let session = session();
    let field = declared_list(&session);
    let DataType::List(child) = field.data_type() else {
        panic!("declared list");
    };
    let scalar = ScalarValue::List(list(Arc::clone(child)));
    let literal = pse_catalog::session::output::checked_literal(
        session.registry(),
        &FieldContract::payload(
            "ids",
            FieldContract::list(FieldContract::id()),
            "Established identity children.",
        ),
        scalar,
    )
    .unwrap();
    let plan = LogicalPlanBuilder::values(vec![vec![lit(1_u64)], vec![lit(2_u64)]])
        .unwrap()
        .project([literal.alias("ids")])
        .unwrap()
        .build()
        .unwrap();
    let cancel = CancellationToken::new();
    let prepared = session.prepare(plan, &cancel).unwrap();
    assert!(!prepared.contains_volatile_expression());
    assert!(
        prepared
            .optimized_plan()
            .display_indent()
            .to_string()
            .contains("pse_preserve_field")
    );
    let completed = prepared.execute(&cancel).await.unwrap();
    let mut count = 0;
    for batch in completed.batches() {
        let actual = batch
            .column(0)
            .as_any()
            .downcast_ref::<ListArray>()
            .unwrap();
        assert_eq!(actual.data_type(), field.data_type());
        for row in 0..batch.num_rows() {
            let values = actual.value(row);
            let values = values
                .as_any()
                .downcast_ref::<FixedSizeBinaryArray>()
                .unwrap();
            assert_eq!(values.value(0), &[7_u8; 16]);
            assert_eq!(values.value(1), &[9_u8; 16]);
            count += 1;
        }
    }
    assert_eq!(count, 2);
}

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "one semantic-field scenario follows actual values through preparation, execution and output admission"
)]
async fn native_collection_preserves_fields_through_ordered_distinct_and_grouped_aggregation() {
    use datafusion::{
        arrow::array::{RecordBatch, UInt64Array},
        logical_expr::{Expr, ExprFunctionExt, Partitioning, col},
    };
    use pse_relations::columnar::FieldCheckedBatch;
    let session = session();
    let spec = session.registry().relation("authored.list_values").unwrap();
    let key = spec.key;
    let field = declared_list(&session);
    let DataType::List(child) = field.data_type() else {
        panic!("list child");
    };
    let values = list(Arc::clone(child));
    let schema = pse_schema::arrow::relation_schema(session.registry(), spec).unwrap();
    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(UInt64Array::from(vec![1, 2])),
            datafusion::arrow::compute::concat(&[values.as_ref(), values.as_ref()]).unwrap(),
        ],
    )
    .unwrap();
    let checked = FieldCheckedBatch::admit(session.registry(), spec, batch).unwrap();
    let cancel = CancellationToken::new();
    let session = session
        .with_checked_workspace(BTreeMap::from([(key, checked)]), &cancel)
        .unwrap();
    let input = LogicalPlanBuilder::scan("values", session.table_source(&key).unwrap(), None)
        .unwrap()
        .unnest_column("ids")
        .unwrap()
        .repartition(Partitioning::RoundRobinBatch(2))
        .unwrap()
        .build()
        .unwrap();
    let input = session.derive_plan_fields(input, &cancel).unwrap();
    let child = input
        .schema()
        .field_with_unqualified_name("ids")
        .unwrap()
        .clone();
    for mode in ["ordered", "distinct", "groups"] {
        let collection = datafusion::functions_aggregate::expr_fn::array_agg(col("ids"));
        let aggregate = match mode {
            "ordered" => collection
                .order_by(vec![col("ids").sort(false, false)])
                .build()
                .unwrap(),
            "distinct" => collection
                .distinct()
                .order_by(vec![col("ids").sort(false, false)])
                .build()
                .unwrap(),
            _ => collection,
        };
        let groups: Vec<Expr> = if mode == "groups" {
            vec![col("id")]
        } else {
            vec![]
        };
        let output_column = groups.len();
        let plan = LogicalPlanBuilder::from(input.clone())
            .aggregate(groups, [aggregate.alias("collected")])
            .unwrap()
            .build()
            .unwrap();
        let prepared = session.prepare(plan, &cancel).unwrap();
        assert!(
            prepared
                .optimized_plan()
                .display_indent()
                .to_string()
                .contains("array_agg")
        );
        let complete = prepared
            .clone()
            .execute(&cancel)
            .await
            .unwrap_or_else(|error| {
                panic!(
                    "{mode}: {error}; {}",
                    session
                        .execution_observations()
                        .unwrap()
                        .last()
                        .unwrap()
                        .physical_plan()
                        .unwrap_or("no physical plan")
                )
            });
        let mut group_count = 0;
        for batch in complete.batches() {
            let array = batch
                .column(output_column)
                .as_any()
                .downcast_ref::<ListArray>()
                .unwrap();
            let DataType::List(actual_child) = array.data_type() else {
                panic!("actual list");
            };
            assert_eq!(actual_child.metadata(), child.metadata(), "{mode}");
            for row in 0..array.len() {
                let values = array.value(row);
                let values = values
                    .as_any()
                    .downcast_ref::<FixedSizeBinaryArray>()
                    .unwrap();
                let mut ids = (0..values.len())
                    .map(|row| values.value(row)[0])
                    .collect::<Vec<_>>();
                if mode == "groups" {
                    ids.sort_unstable();
                }
                assert_eq!(
                    ids,
                    match mode {
                        "ordered" => vec![9, 9, 7, 7],
                        "distinct" => vec![9, 7],
                        _ => vec![7, 9],
                    }
                );
                group_count += 1;
            }
        }
        assert_eq!(group_count, if mode == "groups" { 2 } else { 1 });
    }
}

#[test]
fn field_identity_restores_missing_children_and_reuses_already_correct_arrays() {
    let session = session();
    let field = declared_list(&session);
    let DataType::List(child) = field.data_type() else {
        panic!("declared list");
    };
    let missing = list(Arc::new(
        child
            .as_ref()
            .clone()
            .with_metadata(std::collections::HashMap::default()),
    ));
    let data = missing.values().to_data();
    let restored = invoke(&session, ColumnarValue::Array(missing), field.clone(), 1).unwrap();
    let ColumnarValue::Array(restored) = restored else {
        panic!("restored array");
    };
    assert_eq!(restored.data_type(), field.data_type());
    let restored_list = restored.as_any().downcast_ref::<ListArray>().unwrap();
    assert_eq!(
        restored_list.values().to_data().buffers()[0].as_ptr(),
        data.buffers()[0].as_ptr()
    );
    let again = invoke(
        &session,
        ColumnarValue::Array(Arc::clone(&restored)),
        field,
        1,
    )
    .unwrap();
    let ColumnarValue::Array(again) = again else {
        panic!("retained array");
    };
    assert!(Arc::ptr_eq(&again, &restored));
}

#[test]
fn field_identity_refuses_changed_child_meaning_or_layout() {
    let session = session();
    let field = declared_list(&session);
    let DataType::List(child) = field.data_type() else {
        panic!("declared list");
    };
    for changed in [
        child.as_ref().clone().with_name("other"),
        child.as_ref().clone().with_nullable(!child.is_nullable()),
        child
            .as_ref()
            .clone()
            .with_metadata(std::collections::HashMap::from([(
                "pse.logical_type".to_owned(),
                "a-different-meaning".to_owned(),
            )])),
    ] {
        assert!(
            invoke(
                &session,
                ColumnarValue::Array(list(Arc::new(changed))),
                field.clone(),
                1
            )
            .is_err()
        );
    }
}
