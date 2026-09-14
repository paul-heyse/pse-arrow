// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "canonical contract fixtures fail with direct diagnostics"
)]
//! Metamorphic and independently decoded evidence for the frozen canonical IPC frame.

use arrow::compute::take_record_batch;
use arrow_array::types::Int8Type;
use arrow_array::{
    Array, ArrayRef, DictionaryArray, Float64Array, Int8Array, Int32Array, ListArray, RecordBatch,
    StringArray, StructArray, UInt32Array,
};
use arrow_buffer::{NullBuffer, OffsetBuffer, ScalarBuffer};
use arrow_ipc::reader::StreamReader;
use arrow_schema::{DataType, Field, Schema};
use pse_ids::{
    CancellationToken, CanonError, CanonicalContract, CanonicalizeOptions, ContentHash, Envelope,
    FieldPath, FixedBudget, SchemaVersion, SemanticId, canonicalize, logical_hash,
};
use std::collections::{BTreeMap, HashMap};
use std::io::Cursor;
use std::sync::Arc;

fn fixture(alternate: bool) -> (CanonicalContract, RecordBatch) {
    let list_child = Arc::new(Field::new("item", DataType::Int32, true));
    let struct_child = Arc::new(Field::new("hidden", DataType::Int32, true));
    let fields = vec![
        Field::new("key", DataType::UInt32, false),
        Field::new("float", DataType::Float64, true),
        Field::new(
            "enum",
            DataType::Dictionary(Box::new(DataType::Int8), Box::new(DataType::Utf8)),
            true,
        ),
        Field::new("list", DataType::List(Arc::clone(&list_child)), true),
        Field::new(
            "struct",
            DataType::Struct(vec![Arc::clone(&struct_child)].into()),
            true,
        ),
    ];
    let schema = Arc::new(Schema::new_with_metadata(
        fields,
        HashMap::from([("pse.contract.note".to_owned(), "fixture".to_owned())]),
    ));
    let enums = BTreeMap::from([(
        FieldPath::root().child(2),
        Arc::from(["A".to_owned(), "B".to_owned()]),
    )]);
    let contract = CanonicalContract::try_new(
        SemanticId::from_bytes([1; 16]),
        SchemaVersion(1),
        ContentHash::from_bytes([2; 32]),
        Arc::clone(&schema),
        &["key"],
        &enums,
    )
    .unwrap();
    let nan = if alternate {
        f64::from_bits(0xfff8_0000_0000_0002)
    } else {
        f64::from_bits(0x7ff8_0000_0000_0001)
    };
    let float: ArrayRef = Arc::new(Float64Array::new(
        ScalarBuffer::from(vec![-0.0, if alternate { 900.0 } else { -27.0 }, nan]),
        Some(NullBuffer::from(vec![true, false, true])),
    ));
    let dictionary = dictionary_fixture(alternate);
    let list: ArrayRef = if alternate {
        Arc::new(
            ListArray::try_new(
                list_child,
                OffsetBuffer::new(ScalarBuffer::from(vec![0, 2, 2, 3])),
                Arc::new(Int32Array::from(vec![Some(3), None, Some(5)])),
                Some(NullBuffer::from(vec![true, false, true])),
            )
            .unwrap(),
        )
    } else {
        Arc::new(
            ListArray::try_new(
                list_child,
                OffsetBuffer::new(ScalarBuffer::from(vec![0, 2, 4, 5])),
                Arc::new(Int32Array::from(vec![
                    Some(3),
                    None,
                    Some(888),
                    None,
                    Some(5),
                ])),
                Some(NullBuffer::from(vec![true, false, true])),
            )
            .unwrap(),
        )
    };
    let structure: ArrayRef = Arc::new(
        StructArray::try_new(
            vec![struct_child].into(),
            vec![Arc::new(Int32Array::from(vec![
                Some(3),
                if alternate { Some(999) } else { None },
                Some(5),
            ]))],
            Some(NullBuffer::from(vec![true, false, true])),
        )
        .unwrap(),
    );
    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(UInt32Array::from(vec![3, 1, 2])),
            float,
            dictionary,
            list,
            structure,
        ],
    )
    .unwrap();
    (contract, batch)
}
fn dictionary_fixture(alternate: bool) -> ArrayRef {
    if alternate {
        Arc::new(
            DictionaryArray::<Int8Type>::try_new(
                Int8Array::from(vec![Some(1), None, Some(0)]),
                Arc::new(StringArray::from(vec!["B", "A", "UNUSED INVALID MEMBER"])),
            )
            .unwrap(),
        )
    } else {
        Arc::new(
            DictionaryArray::<Int8Type>::try_new(
                Int8Array::from(vec![Some(0), None, Some(1)]),
                Arc::new(StringArray::from(vec!["A", "B"])),
            )
            .unwrap(),
        )
    }
}
fn options() -> CanonicalizeOptions {
    CanonicalizeOptions {
        keep_preimage: true,
        keep_sorted: true,
        ..Default::default()
    }
}

#[test]
fn row_order_batch_boundaries_null_payload_and_dictionary_codes_have_no_identity() {
    let (contract, source) = fixture(false);
    let (_, alternate) = fixture(true);
    let budget = FixedBudget::new(64 << 20);
    let expected = canonicalize(
        &contract,
        std::slice::from_ref(&source),
        budget.as_ref(),
        options(),
    )
    .unwrap();
    let permutation = take_record_batch(&alternate, &UInt32Array::from(vec![1, 2, 0])).unwrap();
    let batches = vec![permutation.slice(0, 1), permutation.slice(1, 2)];
    let actual = canonicalize(&contract, &batches, budget.as_ref(), options()).unwrap();
    assert_eq!(actual.logical_hash, expected.logical_hash);
    assert_eq!(actual.preimage, expected.preimage);
    let floats = actual
        .sorted
        .as_ref()
        .unwrap()
        .column(1)
        .as_any()
        .downcast_ref::<Float64Array>()
        .unwrap();
    assert_eq!(
        floats.value(1).to_bits(),
        0xfff8_0000_0000_0002,
        "sorted output preserves original NaN payload"
    );
    drop(actual);
    drop(expected);
    assert_eq!(budget.reserved(), 0);
}

fn framed_part<'a>(bytes: &'a [u8], offset: &mut usize) -> &'a [u8] {
    let length = usize::try_from(u64::from_le_bytes(
        bytes[*offset..*offset + 8].try_into().unwrap(),
    ))
    .unwrap();
    *offset += 8;
    let part = &bytes[*offset..*offset + length];
    *offset += length;
    part
}
fn decoded_data(preimage: &[u8]) -> RecordBatch {
    let mut offset = 0;
    assert_eq!(framed_part(preimage, &mut offset), b"pse.canon.v2");
    assert_eq!(&preimage[offset..offset + 16], &[1; 16]);
    offset += 16;
    assert_eq!(&preimage[offset..offset + 4], &1u32.to_le_bytes());
    offset += 4;
    assert_eq!(&preimage[offset..offset + 32], &[2; 32]);
    offset += 32;
    let metadata = framed_part(preimage, &mut offset);
    let data = framed_part(preimage, &mut offset);
    assert_eq!(offset, preimage.len());
    let mut reader = StreamReader::try_new(Cursor::new(metadata), None).unwrap();
    let metadata = reader.next().unwrap().unwrap();
    assert!(reader.next().is_none());
    assert_eq!(metadata.num_columns(), 3);
    assert!(metadata.schema().metadata().is_empty());
    let mut reader = StreamReader::try_new(Cursor::new(data), None).unwrap();
    let data = reader.next().unwrap().unwrap();
    assert!(reader.next().is_none());
    data
}

#[test]
fn frame_contains_two_finished_single_batch_streams_with_canonical_hidden_values() {
    let (contract, source) = fixture(false);
    let budget = FixedBudget::new(64 << 20);
    let result = canonicalize(&contract, &[source], budget.as_ref(), options()).unwrap();
    let normalized = decoded_data(result.preimage.as_ref().unwrap());
    assert!(normalized.schema().metadata().is_empty());
    assert_eq!(normalized.schema().field(2).data_type(), &DataType::Utf8);
    let floats = normalized
        .column(1)
        .as_any()
        .downcast_ref::<Float64Array>()
        .unwrap();
    assert!(floats.is_null(0));
    assert_eq!(floats.value(0).to_bits(), 0);
    assert_eq!(floats.value(1).to_bits(), 0x7ff8_0000_0000_0000);
    assert_eq!(floats.value(2).to_bits(), (-0.0f64).to_bits());
    let structure = normalized
        .column(4)
        .as_any()
        .downcast_ref::<StructArray>()
        .unwrap();
    let child = structure
        .column(0)
        .as_any()
        .downcast_ref::<Int32Array>()
        .unwrap();
    assert!(structure.is_null(0));
    assert!(child.is_valid(0));
    assert_eq!(child.value(0), 0);
    assert!(child.nulls().is_none());
}

#[test]
fn retained_preimage_and_individual_arrow_buffers_keep_the_reservation() {
    let (contract, source) = fixture(false);
    let budget = FixedBudget::new(64 << 20);
    let output = canonicalize(&contract, &[source], budget.as_ref(), options()).unwrap();
    let reserved = budget.reserved();
    assert!(reserved > 0);
    let bytes = output.preimage.as_ref().unwrap().slice(0..1);
    let data = output.sorted.as_ref().unwrap().column(0).to_data();
    let buffer = data.buffers()[0].clone();
    drop(data);
    drop(output);
    assert_eq!(budget.reserved(), reserved);
    drop(bytes);
    assert_eq!(budget.reserved(), reserved);
    drop(buffer);
    assert_eq!(budget.reserved(), 0);
}

#[test]
fn budget_envelope_cancellation_and_duplicate_keys_reject_without_retained_claims() {
    let (contract, source) = fixture(false);
    let tiny = FixedBudget::new(1);
    assert!(matches!(
        logical_hash(&contract, std::slice::from_ref(&source), tiny.as_ref()),
        Err(CanonError::Reservation(_))
    ));
    assert_eq!(tiny.reserved(), 0);
    let budget = FixedBudget::new(64 << 20);
    for envelope in [
        Envelope::lowered(2, 1 << 20).unwrap(),
        Envelope::lowered(10, 1).unwrap(),
    ] {
        assert!(matches!(
            canonicalize(
                &contract,
                std::slice::from_ref(&source),
                budget.as_ref(),
                CanonicalizeOptions {
                    envelope,
                    ..Default::default()
                }
            ),
            Err(CanonError::Envelope { .. })
        ));
        assert_eq!(budget.reserved(), 0);
    }
    let cancel = CancellationToken::new();
    cancel.cancel();
    assert!(matches!(
        canonicalize(
            &contract,
            std::slice::from_ref(&source),
            budget.as_ref(),
            CanonicalizeOptions {
                cancel: Some(cancel),
                ..Default::default()
            }
        ),
        Err(CanonError::Cancelled)
    ));
    assert_eq!(budget.reserved(), 0);
    assert!(matches!(
        logical_hash(&contract, &[source.clone(), source], budget.as_ref()),
        Err(CanonError::DuplicateKey { .. })
    ));
    assert_eq!(budget.reserved(), 0);
}

#[test]
fn signed_zero_differs_and_matching_contract_metadata_cannot_admit_invalid_enum() {
    let (contract, source) = fixture(false);
    let budget = FixedBudget::new(64 << 20);
    let mut columns = source.columns().to_vec();
    columns[1] = Arc::new(Float64Array::from(vec![Some(0.0), None, Some(f64::NAN)]));
    let changed = RecordBatch::try_new(source.schema(), columns.clone()).unwrap();
    assert_ne!(
        logical_hash(&contract, std::slice::from_ref(&source), budget.as_ref()).unwrap(),
        logical_hash(&contract, &[changed], budget.as_ref()).unwrap()
    );
    columns[2] = Arc::new(
        DictionaryArray::<Int8Type>::try_new(
            Int8Array::from(vec![Some(0), None, Some(1)]),
            Arc::new(StringArray::from(vec!["INVALID", "B"])),
        )
        .unwrap(),
    );
    let invalid = RecordBatch::try_new(source.schema(), columns).unwrap();
    assert!(matches!(
        logical_hash(&contract, &[invalid], budget.as_ref()),
        Err(CanonError::EnumMember { .. })
    ));
    assert_eq!(budget.reserved(), 0);
}

#[test]
fn empty_relations_and_explicit_all_valid_bitmaps_are_canonical() {
    let (contract, source) = fixture(false);
    let budget = FixedBudget::new(64 << 20);
    let empty = canonicalize(&contract, &[], budget.as_ref(), options()).unwrap();
    assert_eq!(empty.row_count, 0);
    assert_eq!(decoded_data(empty.preimage.as_ref().unwrap()).num_rows(), 0);
    assert_eq!(
        empty.logical_hash,
        logical_hash(&contract, &[source.slice(0, 0)], budget.as_ref()).unwrap()
    );
    let mut columns = source.columns().to_vec();
    columns[0] = Arc::new(UInt32Array::new(
        ScalarBuffer::from(vec![3, 1, 2]),
        Some(NullBuffer::new_valid(3)),
    ));
    let explicit = RecordBatch::try_new(source.schema(), columns).unwrap();
    assert_eq!(
        logical_hash(&contract, &[source], budget.as_ref()).unwrap(),
        logical_hash(&contract, &[explicit], budget.as_ref()).unwrap()
    );
}

#[test]
fn fixed_list_descendants_are_forced_valid_zero_under_masked_parents() {
    use arrow_array::FixedSizeListArray;
    let child = Arc::new(Field::new("item", DataType::Int32, true));
    let schema = Arc::new(Schema::new(vec![
        Field::new("key", DataType::UInt32, false),
        Field::new(
            "fixed",
            DataType::FixedSizeList(Arc::clone(&child), 2),
            true,
        ),
    ]));
    let contract = CanonicalContract::try_new(
        SemanticId::from_bytes([1; 16]),
        SchemaVersion(1),
        ContentHash::from_bytes([2; 32]),
        Arc::clone(&schema),
        &["key"],
        &BTreeMap::new(),
    )
    .unwrap();
    let make = |hidden| {
        RecordBatch::try_new(
            Arc::clone(&schema),
            vec![
                Arc::new(UInt32Array::from(vec![3, 1, 2])),
                Arc::new(
                    FixedSizeListArray::try_new(
                        Arc::clone(&child),
                        2,
                        Arc::new(Int32Array::from(vec![
                            Some(10),
                            Some(11),
                            hidden,
                            None,
                            Some(12),
                            Some(13),
                        ])),
                        Some(NullBuffer::from(vec![true, false, true])),
                    )
                    .unwrap(),
                ),
            ],
        )
        .unwrap()
    };
    let budget = FixedBudget::new(64 << 20);
    let output = canonicalize(&contract, &[make(None)], budget.as_ref(), options()).unwrap();
    assert_eq!(
        output.logical_hash,
        logical_hash(&contract, &[make(Some(999))], budget.as_ref()).unwrap()
    );
    let normalized = decoded_data(output.preimage.as_ref().unwrap());
    let fixed = normalized
        .column(1)
        .as_any()
        .downcast_ref::<FixedSizeListArray>()
        .unwrap();
    let values = fixed
        .values()
        .as_any()
        .downcast_ref::<Int32Array>()
        .unwrap();
    assert!(fixed.is_null(0));
    assert!(values.nulls().is_none());
    assert_eq!(values.values().as_ref(), &[0, 0, 12, 13, 10, 11]);
}

fn scalar_arrays() -> Vec<ArrayRef> {
    use arrow_array::{
        BinaryArray, BooleanArray, FixedSizeBinaryArray, Float32Array, Int16Array, Int64Array,
        TimestampNanosecondArray, UInt8Array, UInt16Array, UInt64Array,
    };
    vec![
        Arc::new(UInt32Array::from(vec![99, 3, 1, 2])),
        Arc::new(BooleanArray::from(vec![
            Some(true),
            Some(false),
            None,
            Some(true),
        ])),
        Arc::new(Int8Array::from(vec![
            Some(0),
            Some(i8::MIN),
            None,
            Some(i8::MAX),
        ])),
        Arc::new(Int16Array::from(vec![
            Some(0),
            Some(i16::MIN),
            None,
            Some(i16::MAX),
        ])),
        Arc::new(Int64Array::from(vec![
            Some(0),
            Some(i64::MIN),
            None,
            Some(i64::MAX),
        ])),
        Arc::new(UInt8Array::from(vec![
            Some(0),
            Some(u8::MAX),
            None,
            Some(1),
        ])),
        Arc::new(UInt16Array::from(vec![
            Some(0),
            Some(u16::MAX),
            None,
            Some(1),
        ])),
        Arc::new(UInt64Array::from(vec![
            Some(0),
            Some(u64::MAX),
            None,
            Some(1),
        ])),
        Arc::new(Float32Array::from(vec![
            Some(0.0),
            Some(-0.0),
            None,
            Some(f32::from_bits(0xffc0_1234)),
        ])),
        Arc::new(BinaryArray::from(vec![
            Some(&b"ignored"[..]),
            Some(&b"\x00\xff"[..]),
            None,
            Some(&b"last"[..]),
        ])),
        Arc::new(StringArray::from(vec![
            Some("ignored"),
            Some("λ"),
            None,
            Some("last"),
        ])),
        Arc::new(
            FixedSizeBinaryArray::try_from_sparse_iter_with_size(
                [Some([0u8; 16]), Some([1u8; 16]), None, Some([2u8; 16])].into_iter(),
                16,
            )
            .unwrap(),
        ),
        Arc::new(
            TimestampNanosecondArray::from(vec![Some(0), Some(-100), None, Some(100)])
                .with_timezone("UTC"),
        ),
    ]
}

#[test]
fn supported_scalar_storage_and_sliced_input_round_trip_through_canonical_ipc() {
    use arrow_array::Float32Array;
    let arrays = scalar_arrays();
    let fields = arrays
        .iter()
        .enumerate()
        .map(|(index, array)| {
            Field::new(
                if index == 0 {
                    "key".to_owned()
                } else {
                    format!("column{index}")
                },
                array.data_type().clone(),
                index != 0,
            )
        })
        .collect::<Vec<_>>();
    let schema = Arc::new(Schema::new(fields));
    let full = RecordBatch::try_new(Arc::clone(&schema), arrays).unwrap();
    let sliced = full.slice(1, 3);
    let contract = CanonicalContract::try_new(
        SemanticId::from_bytes([1; 16]),
        SchemaVersion(1),
        ContentHash::from_bytes([2; 32]),
        schema,
        &["key"],
        &BTreeMap::new(),
    )
    .unwrap();
    let budget = FixedBudget::new(64 << 20);
    let output = canonicalize(
        &contract,
        std::slice::from_ref(&sliced),
        budget.as_ref(),
        options(),
    )
    .unwrap();
    let normalized = decoded_data(output.preimage.as_ref().unwrap());
    assert_eq!(normalized.num_rows(), 3);
    let floats = normalized
        .column(8)
        .as_any()
        .downcast_ref::<Float32Array>()
        .unwrap();
    assert_eq!(floats.value(0).to_bits(), 0);
    assert_eq!(floats.value(1).to_bits(), 0x7fc0_0000);
    assert_eq!(floats.value(2).to_bits(), (-0.0f32).to_bits());
    assert_eq!(
        output.logical_hash,
        logical_hash(
            &contract,
            &[sliced.slice(2, 1), sliced.slice(0, 2)],
            budget.as_ref()
        )
        .unwrap()
    );
}
