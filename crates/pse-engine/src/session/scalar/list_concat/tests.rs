// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;
use datafusion::arrow::{
    array::{Array, ArrayRef, FixedSizeBinaryArray, ListArray},
    datatypes::Field,
};

fn values(rows: &[Vec<u8>]) -> (ArrayRef, FieldRef) {
    // Exercise the declared IndexTuple field directly; no model relation is needed.
    let field = Arc::new(
        pse_schema::arrow::field_for(
            crate::validation::registry().unwrap(),
            &pse_schema::model::FieldContract::payload(
                "tuple",
                pse_schema::model::FieldContract::extended(
                    pse_schema::model::ExtensionUse::IndexTuple,
                ),
                "Ordered tuple test input",
            ),
        )
        .unwrap(),
    );
    let DataType::List(child) = field.data_type() else {
        panic!("list expected")
    };
    let mut builder = datafusion::arrow::array::ListBuilder::new(
        datafusion::arrow::array::FixedSizeBinaryBuilder::new(16),
    )
    .with_field(child.clone());
    for row in rows {
        for value in row {
            builder.values().append_value([*value; 16]).unwrap();
        }
        builder.append(true);
    }
    (Arc::new(builder.finish()), field)
}

#[test]
fn typed_concat_preserves_order_fields_and_last_reader_reservation() {
    let budget: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(1 << 20));
    let pool: Arc<dyn MemoryPool> = budget.clone();
    let function = function(pool);
    let (left, field) = values(&[vec![1], vec![3]]);
    let (right, _) = values(&[vec![2], vec![4, 5]]);
    let fields = vec![Arc::clone(&field), Arc::clone(&field)];
    let output = function
        .return_field_from_args(ReturnFieldArgs {
            arg_fields: &fields,
            scalar_arguments: &[None, None],
        })
        .unwrap();
    assert_eq!(output.as_ref(), field.as_ref());
    let result = function
        .invoke_with_args(ScalarFunctionArgs {
            args: vec![ColumnarValue::Array(left), ColumnarValue::Array(right)],
            arg_fields: fields,
            number_rows: 2,
            return_field: output,
            config_options: Arc::default(),
        })
        .unwrap()
        .into_array(2)
        .unwrap();
    let lists = result.as_any().downcast_ref::<ListArray>().unwrap();
    let actual = (0..lists.len())
        .map(|row| {
            let values = lists.value(row);
            let values = values
                .as_any()
                .downcast_ref::<FixedSizeBinaryArray>()
                .unwrap();
            (0..values.len())
                .map(|position| values.value(position)[0])
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, vec![vec![1, 2], vec![3, 4, 5]]);
    assert!(budget.reserved() > 0);
    let last_reader = Arc::clone(&result);
    drop(result);
    assert!(budget.reserved() > 0);
    drop(last_reader);
    assert_eq!(budget.reserved(), 0);
}

#[test]
fn typed_concat_rejects_unknown_semantics_and_budget_before_native_allocation() {
    let budget: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(1));
    let function = function(budget.clone());
    let (value, field) = values(&[vec![1]]);
    // IndexTuple declares its member semantics through the outer extension;
    // its physical child intentionally carries no separate semantic-ID label.
    let unknown = Arc::new(Field::new(field.name(), field.data_type().clone(), false));
    assert!(
        function
            .return_field_from_args(ReturnFieldArgs {
                arg_fields: &[Arc::clone(&field), unknown],
                scalar_arguments: &[None, None],
            })
            .is_err()
    );
    // An ordinary List<SemanticId> instead establishes meaning on the child.
    let typed = Arc::new(
        pse_schema::arrow::field_for(
            crate::validation::registry().unwrap(),
            &pse_schema::model::FieldContract::payload(
                "ids",
                pse_schema::model::FieldContract::list(pse_schema::model::FieldContract::id()),
                "Actual ordered identities.",
            ),
        )
        .unwrap(),
    );
    let DataType::List(child) = typed.data_type() else {
        panic!("list expected");
    };
    let unknown = Arc::new(
        typed
            .as_ref()
            .clone()
            .with_data_type(DataType::List(Arc::new(Field::new(
                "item",
                child.data_type().clone(),
                child.is_nullable(),
            )))),
    );
    assert!(
        function
            .return_field_from_args(ReturnFieldArgs {
                arg_fields: &[typed, unknown],
                scalar_arguments: &[None, None]
            })
            .is_err()
    );
    let output = function
        .return_field_from_args(ReturnFieldArgs {
            arg_fields: &[Arc::clone(&field)],
            scalar_arguments: &[None],
        })
        .unwrap();
    assert!(
        function
            .invoke_with_args(ScalarFunctionArgs {
                args: vec![ColumnarValue::Array(value)],
                arg_fields: vec![field],
                number_rows: 1,
                return_field: output,
                config_options: Arc::default()
            })
            .is_err()
    );
    assert_eq!(budget.reserved(), 0);
}
