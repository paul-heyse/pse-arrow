// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(clippy::unwrap_used, reason = "semantic token equivalence fixtures")]
use super::*;
use arrow_array::{DictionaryArray, Float64Array, Int32Array, StringArray, types::Int32Type};
use arrow_schema::{DataType, Field};
use std::{collections::HashMap, sync::Arc};

#[test]
fn generated_result_row_identity_survives_repacking_slices_and_partitions() {
    use arrow_array::{BooleanArray, StructArray};
    use arrow_buffer::NullBuffer;
    use proptest::{
        prelude::*,
        test_runner::{Config, RngSeed, TestRunner},
    };

    // The logical rows distinguish a null struct from a valid struct with a null child.
    type Row = (Option<bool>, Option<Option<bool>>);
    let identity = |rows: &[Row], repacked: bool, partition: usize| {
        let padding = (Some(false), Some(Some(false)));
        let mut physical = Vec::with_capacity(rows.len() + usize::from(repacked));
        if repacked {
            physical.push(padding);
        }
        physical.extend_from_slice(rows);
        let keys = Int32Array::from(
            physical
                .iter()
                .map(|row| row.0.map(|value| i32::from(value ^ repacked)))
                .collect::<Vec<_>>(),
        );
        let dictionary = DictionaryArray::<Int32Type>::try_new(
            keys,
            Arc::new(StringArray::from(if repacked {
                vec!["high", "low"]
            } else {
                vec!["low", "high"]
            })),
        )
        .unwrap();
        let nested = StructArray::new(
            vec![Arc::new(Field::new("flag", DataType::Boolean, true))].into(),
            vec![Arc::new(BooleanArray::from(
                physical
                    .iter()
                    .map(|row| row.1.unwrap_or(Some(repacked)))
                    .collect::<Vec<_>>(),
            ))],
            Some(NullBuffer::from(
                physical
                    .iter()
                    .map(|row| row.1.is_some())
                    .collect::<Vec<_>>(),
            )),
        );
        let arrays: [ArrayRef; 2] = [Arc::new(dictionary), Arc::new(nested)];
        let fields = [
            Arc::new(Field::new("label", arrays[0].data_type().clone(), true)),
            Arc::new(Field::new("record", arrays[1].data_type().clone(), true)),
        ];
        let mut result = Vec::with_capacity(rows.len());
        for start in (0..rows.len()).step_by(partition) {
            let count = partition.min(rows.len() - start);
            let offset = start + usize::from(repacked);
            let tokens = tokens(
                pse_ids::SemanticId::NIL,
                &[
                    ("label", fields[0].clone(), arrays[0].slice(offset, count)),
                    ("record", fields[1].clone(), arrays[1].slice(offset, count)),
                ],
                count,
            )
            .unwrap();
            result.extend((0..count).map(|index| tokens.value(index).to_vec()));
        }
        result
    };
    let cases = (
        proptest::collection::vec(
            (
                proptest::option::of(any::<bool>()),
                proptest::option::of(proptest::option::of(any::<bool>())),
            ),
            2..17,
        ),
        1usize..8,
    );
    TestRunner::new(Config {
        cases: 32,
        rng_seed: RngSeed::Fixed(0x5053_452d_524f_5731),
        failure_persistence: None,
        ..Config::default()
    })
    .run(&cases, |(mut rows, width)| {
        let original = identity(&rows, false, rows.len());
        prop_assert_eq!(&original, &identity(&rows, true, width));
        for (left, row) in rows.iter().enumerate() {
            for (right, other) in rows[..left].iter().enumerate() {
                prop_assert_eq!(original[left] == original[right], row == other);
            }
        }
        rows[0].0 = Some(!rows[0].0.unwrap_or(false));
        let changed = identity(&rows, true, width);
        prop_assert_ne!(&original[0], &changed[0]);
        prop_assert_eq!(&original[1..], &changed[1..]);
        Ok(())
    })
    .unwrap();
}

#[test]
fn v2_row_token_matches_an_independently_framed_int64_vector() {
    let payload = pse_ids::preimage::derive_key(Frame::NativeValuePayloadV1, br#"["i64",42]"#);
    let meaning = br#"{"data_type":"Int64","dict_id":0,"dict_is_ordered":false,"metadata":{},"name":"item","nullable":false}"#;
    let mut frame = Vec::new();
    for part in [
        &[0u8; 16][..],
        &1u64.to_le_bytes(),
        b"id",
        meaning,
        &payload,
    ] {
        frame.extend_from_slice(&u64::try_from(part.len()).unwrap().to_le_bytes());
        frame.extend_from_slice(part);
    }
    let expected = pse_ids::preimage::derive_key(Frame::RowKeyNativeValuesV2, &frame);
    assert_eq!(
        crate::ContentHash::from_bytes(expected).to_hex(),
        "811bfe61cc46de019bff708ee5f5b8534a3b4705c2effbb8a4f99352adb58ab5"
    );
    let actual = tokens(
        SemanticId::NIL,
        &[(
            "id",
            Arc::new(Field::new("id", DataType::Int64, false)),
            Arc::new(arrow_array::Int64Array::from(vec![42])),
        )],
        1,
    )
    .unwrap();
    assert_eq!(actual.value(0), &expected);
}
fn token(field: Field, values: ArrayRef) -> FixedSizeBinaryArray {
    let length = values.len();
    tokens(SemanticId::NIL, &[("key", Arc::new(field), values)], length).unwrap()
}
#[test]
fn semantic_tokens_normalize_nans_but_preserve_zero_sign_and_meaning() {
    let field = Field::new("source", DataType::Float64, false);
    let values: ArrayRef = Arc::new(Float64Array::from(vec![
        f64::from_bits(0x7ff8_0000_0000_0001),
        f64::from_bits(0xfff8_0000_0000_0009),
        -0.0,
        0.0,
    ]));
    let ordinary = token(field.clone(), Arc::clone(&values));
    assert_eq!(ordinary.value(0), ordinary.value(1));
    assert_ne!(ordinary.value(2), ordinary.value(3));
    let docs = field.clone().with_metadata(HashMap::from([(
        crate::native_field::DOCUMENTATION.into(),
        "other prose".into(),
    )]));
    assert_eq!(ordinary, token(docs, Arc::clone(&values)));
    let unknown = field.with_metadata(HashMap::from([("unknown.meaning".into(), "new".into())]));
    assert_ne!(ordinary, token(unknown, values));
}
#[test]
fn dictionary_reordering_and_slicing_do_not_change_value_tokens() {
    let first = DictionaryArray::<Int32Type>::try_new(
        Int32Array::from(vec![0, 1]),
        Arc::new(StringArray::from(vec!["a", "b"])),
    )
    .unwrap();
    let second = DictionaryArray::<Int32Type>::try_new(
        Int32Array::from(vec![0, 1, 0]),
        Arc::new(StringArray::from(vec!["b", "a"])),
    )
    .unwrap()
    .slice(1, 2);
    let field = Field::new("key", first.data_type().clone(), false);
    assert_eq!(
        token(field.clone(), Arc::new(first)),
        token(field, Arc::new(second))
    );
    assert_eq!(tokens(SemanticId::NIL, &[], 2).unwrap().len(), 2);
    assert_ne!(
        tokens(SemanticId::NIL, &[], 1).unwrap(),
        tokens(SemanticId::from_bytes([1; 16]), &[], 1).unwrap()
    );
}
#[test]
fn physical_execution_and_value_metadata_projections_answer_different_questions() {
    use crate::native_field::{MetadataPurpose, project};
    let source = Field::new("alias", DataType::Utf8, true).with_metadata(HashMap::from([
        (crate::native_field::DOCUMENTATION.into(), "prose".into()),
        ("pse.semantic.role".into(), "key".into()),
        ("custom.meaning".into(), "domain".into()),
    ]));
    assert_eq!(
        project(&source, MetadataPurpose::PhysicalObservation).unwrap(),
        source
    );
    let execution = project(&source, MetadataPurpose::ExecutionIdentity).unwrap();
    assert!(execution.metadata().contains_key("pse.semantic.role"));
    let value = project(&source, MetadataPurpose::ValueIdentity).unwrap();
    assert_eq!(value.name(), "item");
    assert!(!value.is_nullable());
    assert!(!value.metadata().contains_key("pse.semantic.role"));
    assert_eq!(value.metadata().get("custom.meaning").unwrap(), "domain");
}
