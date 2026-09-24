// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;
use datafusion::arrow::{
    array::{Array, ArrayRef, FixedSizeBinaryArray, ListArray},
    buffer::OffsetBuffer,
    datatypes::Field,
};

fn source(members: &[Option<[u8; 16]>]) -> (ArrayRef, FieldRef) {
    let registry = crate::validation::registry().unwrap();
    let child = Arc::new(
        pse_schema::arrow::field_for(
            registry,
            &FieldContract::payload("item", FieldContract::id(), "Actual semantic identity."),
        )
        .unwrap()
        .with_nullable(true),
    );
    let values: ArrayRef = Arc::new(
        FixedSizeBinaryArray::try_from_sparse_iter_with_size(
            members.iter().map(|value| value.as_ref()),
            16,
        )
        .unwrap(),
    );
    let array: ArrayRef = Arc::new(
        ListArray::try_new(
            Arc::clone(&child),
            OffsetBuffer::new(vec![0, i32::try_from(members.len()).unwrap()].into()),
            values,
            None,
        )
        .unwrap(),
    );
    (
        array,
        Arc::new(Field::new("source", DataType::List(child), false)),
    )
}

fn invoke(function: &ScalarUDF, source: ArrayRef, field: FieldRef) -> Result<ArrayRef> {
    let output = function.return_field_from_args(ReturnFieldArgs {
        arg_fields: &[Arc::clone(&field)],
        scalar_arguments: &[None],
    })?;
    function
        .invoke_with_args(ScalarFunctionArgs {
            args: vec![ColumnarValue::Array(source)],
            arg_fields: vec![field],
            number_rows: 1,
            return_field: output,
            config_options: Arc::default(),
        })?
        .into_array(1)
}

#[test]
fn tuple_constructor_preserves_order_and_retains_native_allocation_until_final_reader() {
    let budget: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(1 << 20));
    let function = function(budget.clone());
    let (array, field) = source(&[Some([2; 16]), Some([1; 16]), Some([2; 16])]);
    let result = invoke(&function, array, field).unwrap();
    assert_eq!(
        result.data_type(),
        &FieldContract::extended(ExtensionUse::IndexTuple).data_type()
    );
    let values = result
        .as_any()
        .downcast_ref::<ListArray>()
        .unwrap()
        .value(0);
    let ids = values
        .as_any()
        .downcast_ref::<FixedSizeBinaryArray>()
        .unwrap();
    assert_eq!(ids.value(0), &[2; 16]);
    assert_eq!(ids.value(1), &[1; 16]);
    assert_eq!(ids.value(2), &[2; 16]);
    assert!(budget.reserved() > 0);
    let reader = Arc::clone(&result);
    drop(result);
    assert!(budget.reserved() > 0);
    drop(values);
    drop(reader);
    assert_eq!(budget.reserved(), 0);
}

#[test]
fn tuple_constructor_refuses_unknown_identity_fields_actual_null_members_and_budget_exhaustion() {
    let budget: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(1 << 20));
    let function = function(budget.clone());
    let unknown = Arc::new(Field::new(
        "unknown",
        DataType::List(Arc::new(Field::new(
            "item",
            DataType::FixedSizeBinary(16),
            false,
        ))),
        false,
    ));
    assert!(
        function
            .return_field_from_args(ReturnFieldArgs {
                arg_fields: &[unknown],
                scalar_arguments: &[None],
            })
            .is_err()
    );
    let (array, field) = source(&[Some([1; 16]), None]);
    assert!(invoke(&function, array, field).is_err());
    assert_eq!(budget.reserved(), 0);
    let tiny: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(1));
    let (array, field) = source(&[Some([1; 16])]);
    assert!(invoke(&super::function(tiny.clone()), array, field).is_err());
    assert_eq!(tiny.reserved(), 0);
}
