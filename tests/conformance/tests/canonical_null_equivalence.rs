// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Independent representation changes preserve canonical bytes after direct row admission.

#[path = "../src/canon_fixtures.rs"]
mod canon_fixtures;

use canon_fixtures::{canonical, fixture, replace_float};
use datafusion::arrow::{
    array::{RecordBatch, UInt32Array, UInt64Array},
    buffer::NullBuffer,
    compute::take_record_batch,
};

#[test]
fn hidden_nulls_nested_masks_dictionary_codes_slices_and_batch_order_are_equivalent() {
    let (_reg, contract, base) = fixture(false);
    let (_, _, alternate) = fixture(true);
    let expected = canonical(&contract, std::slice::from_ref(&base));
    // One deterministic witness covers payload normalization and a split slice.
    // Generated cases own the broader ordering/split exploration.
    let reordered =
        take_record_batch(&alternate, &UInt32Array::from(vec![2, 0, 1])).expect("permutation");
    let actual = canonical(&contract, &[reordered.slice(0, 1), reordered.slice(1, 2)]);
    assert_eq!(actual.preimage, expected.preimage);
    assert_eq!(actual.logical_hash, expected.logical_hash);
    let values = base
        .column(0)
        .as_any()
        .downcast_ref::<UInt64Array>()
        .expect("keys");
    let mut columns = base.columns().to_vec();
    columns[0] = std::sync::Arc::new(UInt64Array::new(
        values.values().clone(),
        Some(NullBuffer::new_valid(3)),
    ));
    let all_valid = RecordBatch::try_new(base.schema(), columns).expect("bitmap");
    assert_eq!(
        canonical(&contract, &[all_valid]).preimage,
        expected.preimage
    );
}

#[test]
fn visible_values_signed_zero_and_empty_relations_remain_distinct() {
    let (_, contract, base) = fixture(false);
    let expected = canonical(&contract, std::slice::from_ref(&base));
    for changed in [replace_float(&base, 0, 0.0), replace_float(&base, 0, 1.0)] {
        let actual = canonical(&contract, &[changed]);
        assert_ne!(actual.preimage, expected.preimage);
        assert_ne!(actual.logical_hash, expected.logical_hash);
    }
    let empty = canonical(&contract, &[base.slice(0, 0)]);
    assert_eq!(empty.row_count, 0);
    assert_ne!(empty.preimage, expected.preimage);
}
