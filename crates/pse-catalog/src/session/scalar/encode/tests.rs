// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Mathematical wire expectations, independent of the optional Cell adapter.

#![allow(clippy::unwrap_used, reason = "small exact Arrow fixtures")]

use super::*;
use datafusion::arrow::{
    array::{Int32Array, UInt64Array},
    buffer::NullBuffer,
};
use std::sync::Arc;

#[test]
fn dictionary_codes_and_slices_encode_actual_meanings() {
    let dictionary = DictionaryArray::<Int32Type>::try_new(
        Int32Array::from(vec![1, 0, 1]),
        Arc::new(StringArray::from(vec!["liquid", "vapor"])),
    )
    .unwrap();
    let dictionary = dictionary.slice(1, 2);
    let field = Field::new("phase", dictionary.data_type().clone(), false);
    let mut text = String::new();
    value(&dictionary, &field, 0, &mut text).unwrap();
    assert_eq!(text, "[\"enum\",\"liquid\"]");
    text.clear();
    value(&dictionary, &field, 1, &mut text).unwrap();
    assert_eq!(text, "[\"enum\",\"vapor\"]");
}

#[test]
fn visible_parent_nulls_mask_nested_payload_and_list_offsets_are_exact() {
    let list =
        ListArray::from_iter_primitive::<datafusion::arrow::array::types::UInt64Type, _, _>([
            Some(vec![Some(5), Some(u64::MAX)]),
            Some(vec![Some(8)]),
        ]);
    let list = list.slice(1, 1);
    let field = Field::new("items", list.data_type().clone(), false);
    let mut text = String::new();
    value(&list, &field, 0, &mut text).unwrap();
    assert_eq!(text, "[\"list\",[[\"u64\",8]]]");
    let child = Arc::new(Field::new("index", DataType::UInt64, false));
    let nested = StructArray::new(
        vec![child].into(),
        vec![Arc::new(UInt64Array::from(vec![u64::MAX, 4]))],
        Some(NullBuffer::from(vec![false, true])),
    );
    let field = Field::new("nested", nested.data_type().clone(), true);
    text.clear();
    value(&nested, &field, 0, &mut text).unwrap();
    assert_eq!(text, "[\"null\",null]");
    text.clear();
    value(&nested, &field, 1, &mut text).unwrap();
    assert_eq!(text, "[\"struct\",[[\"u64\",4]]]");
}
