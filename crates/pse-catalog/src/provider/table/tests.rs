// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native scan construction, projection and execution contracts.

use super::*;
use datafusion::arrow::array::{
    Array, ArrayRef, DictionaryArray, Int8Array, StringArray, StructArray, UInt64Array,
};
use datafusion::arrow::datatypes::{DataType, Field, Int8Type, Schema};
use datafusion::common::{Constraint, ScalarValue};
use datafusion::execution::context::SessionContext;
use datafusion::logical_expr::{Volatility, col, create_udf, lit};
use datafusion::physical_plan::{collect, filter::FilterExec};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

fn fixture() -> RecordBatch {
    let child = Arc::new(Field::new("value", DataType::UInt64, false).with_metadata(
        HashMap::from([("fixture.child".to_owned(), "preserved".to_owned())]),
    ));
    let values: ArrayRef = Arc::new(UInt64Array::from(vec![8, 9, 10, 11]));
    let nested = StructArray::from(vec![(child, values)]);
    let dictionary = DictionaryArray::<Int8Type>::try_new(
        Int8Array::from(vec![Some(0), Some(1), None, Some(0)]),
        Arc::new(StringArray::from(vec!["yes", "no"])),
    )
    .expect("valid dictionary fixture");
    let schema = Arc::new(Schema::new_with_metadata(
        vec![
            Field::new("id", DataType::UInt64, false),
            Field::new("status", dictionary.data_type().clone(), true),
            Field::new("payload", nested.data_type().clone(), false).with_metadata(HashMap::from(
                [("fixture.parent".to_owned(), "preserved".to_owned())],
            )),
        ],
        HashMap::from([("fixture.schema".to_owned(), "preserved".to_owned())]),
    ));
    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(UInt64Array::from(vec![1, 2, 3, 4])),
            Arc::new(dictionary),
            Arc::new(nested),
        ],
    )
    .expect("declared schema matches arrays")
}

fn constraints() -> Constraints {
    Constraints::new_unverified(vec![Constraint::PrimaryKey(vec![0])])
}

#[tokio::test]
async fn planning_never_evaluates_rows_and_native_execution_does() {
    let context = SessionContext::new();
    let state = context.state();
    let evaluated = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&evaluated);
    let function = create_udf(
        "scan_counter",
        vec![DataType::UInt64],
        DataType::UInt64,
        Volatility::Volatile,
        Arc::new(move |args| {
            counter.fetch_add(1, Ordering::Relaxed);
            Ok(args[0].clone())
        }),
    );
    let filter = function.call(vec![col("id")]).eq(lit(3_u64));
    let plan = native_scan(
        &fixture(),
        &constraints(),
        &state,
        Some(&[0]),
        &[filter],
        None,
    )
    .expect("build native plan");
    assert_eq!(evaluated.load(Ordering::Relaxed), 0);
    assert!(plan.is::<FilterExec>());
    let result = collect(plan, state.task_ctx())
        .await
        .expect("execute native filter");
    assert!(evaluated.load(Ordering::Relaxed) > 0);
    assert_eq!(result.iter().map(RecordBatch::num_rows).sum::<usize>(), 1);
    assert_eq!(
        ScalarValue::try_from_array(result[0].column(0), 0).expect("result"),
        ScalarValue::UInt64(Some(3))
    );
}

#[tokio::test]
async fn predicate_columns_remain_available_when_projection_omits_them() {
    let batch = fixture();
    let context = SessionContext::new();
    let state = context.state();
    let filter = col("id").in_list(vec![lit(2_u64), lit(4_u64)], false);
    let plan = native_scan(
        &batch,
        &constraints(),
        &state,
        Some(&[2, 1]),
        &[filter],
        Some(1),
    )
    .expect("build native filtered projection");
    assert!(plan.is::<GlobalLimitExec>());
    let filter = plan.children()[0]
        .downcast_ref::<FilterExec>()
        .expect("native filter");
    assert_eq!(filter.input().schema().fields().len(), 3);
    assert_eq!(
        plan.schema().as_ref(),
        &batch.schema().project(&[2, 1]).expect("projection")
    );
    let result = collect(plan, state.task_ctx()).await.expect("execute");
    assert_eq!(result.iter().map(RecordBatch::num_rows).sum::<usize>(), 1);
    let nested = result[0]
        .column(0)
        .as_any()
        .downcast_ref::<StructArray>()
        .expect("nested payload");
    assert_eq!(
        ScalarValue::try_from_array(nested.column(0), 0).expect("nested value"),
        ScalarValue::UInt64(Some(9))
    );
}

#[tokio::test]
async fn dictionary_nulls_and_limit_are_applied_in_query_order() {
    let context = SessionContext::new();
    let state = context.state();
    for (filter, limit, expected) in [
        (col("status").is_null(), None, vec![3_u64]),
        (col("status").eq(lit("yes")), Some(1), vec![1]),
        (col("id").eq(lit(4_u64)), Some(1), vec![4]),
        (col("id").eq(lit(99_u64)), None, vec![]),
        (col("id").is_not_null(), Some(0), vec![]),
    ] {
        let plan = native_scan(
            &fixture(),
            &constraints(),
            &state,
            Some(&[0]),
            &[filter],
            limit,
        )
        .expect("build native filter");
        let result = collect(plan, state.task_ctx()).await.expect("execute");
        let actual: Vec<_> = result
            .iter()
            .flat_map(|batch| {
                batch
                    .column(0)
                    .as_any()
                    .downcast_ref::<UInt64Array>()
                    .expect("id column")
                    .values()
                    .iter()
                    .copied()
            })
            .collect();
        assert_eq!(actual, expected);
    }
}

#[test]
fn projected_constraints_only_describe_retained_keys() {
    let context = SessionContext::new();
    let state = context.state();
    for (projection, expected) in [
        (
            vec![2, 0],
            Constraints::new_unverified(vec![Constraint::PrimaryKey(vec![1])]),
        ),
        (vec![2], Constraints::default()),
    ] {
        let plan = native_scan(
            &fixture(),
            &constraints(),
            &state,
            Some(&projection),
            &[],
            None,
        )
        .expect("build projected native source");
        assert_eq!(
            plan.properties().equivalence_properties().constraints(),
            &expected
        );
    }
}

#[test]
fn invalid_projection_refuses_during_construction() {
    let context = SessionContext::new();
    assert!(
        native_scan(
            &fixture(),
            &constraints(),
            &context.state(),
            Some(&[99]),
            &[],
            None
        )
        .is_err()
    );
}
