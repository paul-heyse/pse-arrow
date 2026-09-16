// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native collection predicates exercise actual array values and parent masks.
#![allow(
    clippy::unwrap_used,
    reason = "independent native predicate assertions"
)]

use super::*;
use datafusion::{
    arrow::{
        array::builder::{Int64Builder, MapBuilder},
        array::{
            Array, ArrayRef, BooleanArray, Float16Array, Int64Array, ListViewArray, RecordBatch,
        },
        buffer::NullBuffer,
    },
    execution::context::SessionContext,
};
use std::sync::Arc;

fn check(array: ArrayRef, expected: &[bool]) {
    let field = Field::new("value", array.data_type().clone(), true);
    let batch =
        RecordBatch::try_new(Arc::new(Schema::new(vec![field.clone()])), vec![array]).unwrap();
    let schema = DFSchema::try_from(batch.schema().as_ref().clone()).unwrap();
    let registry = pse_schema::RegistryBuilder::new().build().unwrap();
    let expression = field_value(&registry, &field, column("value"), 0)
        .unwrap()
        .resolve_lambda_variables(&schema)
        .unwrap()
        .data;
    let result = SessionContext::new()
        .state()
        .create_physical_expr(expression, &schema)
        .unwrap()
        .evaluate(&batch)
        .unwrap()
        .to_array(batch.num_rows())
        .unwrap();
    let values = result.as_any().downcast_ref::<BooleanArray>().unwrap();
    assert_eq!(values.null_count(), 0);
    assert_eq!(values.values().iter().collect::<Vec<_>>(), expected);
}

#[test]
fn map_key_and_value_domains_and_uniqueness_respect_parent_masks() {
    let mut builder = MapBuilder::new(None, Int64Builder::new(), Int64Builder::new())
        .with_keys_field(IntegerRange::nonnegative(10).field("keys"))
        .with_values_field(IntegerRange::nonnegative(255).field("values"));
    for (entries, visible) in [
        (vec![(1, 0)], true),
        (vec![(1, 256)], true),
        (vec![(11, 1)], true),
        (vec![(1, 1), (1, 2)], true),
        (vec![(11, -1)], false),
        (vec![], true),
    ] {
        for (key, value) in entries {
            builder.keys().append_value(key);
            builder.values().append_value(value);
        }
        builder.append(visible).unwrap();
    }
    check(
        Arc::new(builder.finish()),
        &[true, false, false, false, true, true],
    );
}

#[test]
fn list_view_child_ranges_respect_visible_slices_and_null_parents() {
    let child = Arc::new(IntegerRange::nonnegative(255).field("item"));
    let values: ArrayRef = Arc::new(Int64Array::from(vec![255, -1, 256]));
    let array = ListViewArray::try_new(
        child,
        vec![0, 1, 2, 1].into(),
        vec![1, 1, 1, 1].into(),
        values,
        Some(NullBuffer::from(vec![true, true, true, false])),
    )
    .unwrap();
    check(Arc::new(array), &[true, false, false, true]);
}

#[test]
fn half_precision_reals_reject_nonfinite_values() {
    type Half = <Float16Type as ArrowPrimitiveType>::Native;
    let array = Float16Array::from(vec![Half::MAX, Half::NAN, Half::NEG_INFINITY]);
    check(Arc::new(array), &[true, false, false]);
}
