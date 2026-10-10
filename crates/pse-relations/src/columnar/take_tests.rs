// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Allocation and index-domain controls for the actual checked Arrow selection path.

#![allow(clippy::unwrap_used, reason = "isolated checked-take fixtures")]

use super::*;
use crate::native::execution::context::SessionContext;
use arrow_array::{
    ArrayRef, FixedSizeBinaryArray, Float64Array, ListArray, StringArray, UInt32Array,
};
use arrow_schema::{DataType, Field, UnionFields};
use pse_schema::model::{Authority, FieldContract, Namespace, RelationDecl, SnapshotClass};

fn declared_data(data: arrow::array::ArrayData, declared: &DataType) -> arrow::array::ArrayData {
    // Construct the fixture with the registry's nested field annotations before
    // admission. The buffers, offsets, validity and physical storage are unchanged.
    assert!(data.data_type().equals_datatype(declared));
    let children = match declared {
        DataType::Dictionary(_, values) => vec![values.as_ref()],
        _ => pse_columnar::native_field::children(declared)
            .into_iter()
            .map(Field::data_type)
            .collect(),
    };
    assert_eq!(data.child_data().len(), children.len());
    let children = data
        .child_data()
        .iter()
        .cloned()
        .zip(children)
        .map(|(data, declared)| declared_data(data, declared))
        .collect();
    data.into_builder()
        .data_type(declared.clone())
        .child_data(children)
        .build()
        .unwrap()
}

fn checked(columns: Vec<ArrayRef>, rows: usize) -> FieldCheckedBatch {
    let names = ["a", "b", "c", "d", "e", "f", "g"];
    let mut builder = pse_schema::RegistryBuilder::new();
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "take_fixture",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "checked take fixture",
        )
        .columns(
            columns
                .iter()
                .zip(names)
                .map(|(column, name)| {
                    let field = FieldContract::payload(
                        name,
                        FieldContract::native(column.data_type().clone()),
                        "fixture value",
                    );
                    if column.null_count() != 0 {
                        field.optional()
                    } else {
                        field
                    }
                })
                .collect(),
        )
        .pk(&[]),
    );
    let registry = builder.build().unwrap();
    let spec = registry.relation("authored.take_fixture").unwrap();
    let schema = pse_schema::arrow::relation_schema_ref(&registry, spec).unwrap();
    let columns = columns
        .iter()
        .zip(schema.fields())
        .map(|(column, field)| {
            arrow_array::make_array(declared_data(column.to_data(), field.data_type()))
        })
        .collect();
    let batch = RecordBatch::try_new_with_options(
        schema,
        columns,
        &RecordBatchOptions::new().with_row_count(Some(rows)),
    )
    .unwrap();
    FieldCheckedBatch::admit(
        &registry,
        spec,
        batch,
        &crate::validate::ValidationContext::new(&registry, SessionContext::new().state()),
        &pse_columnar::CancellationToken::new(),
    )
    .unwrap()
}

fn pool(bytes: usize) -> Arc<dyn pse_columnar::MemoryPool> {
    Arc::new(pse_columnar::GreedyMemoryPool::new(bytes))
}

#[test]
fn checked_take_trajectory_permutation_fits_bounded_pool() {
    // Same physical width and 128 * 31 row count as the failed trajectory sort.
    let rows = 128 * 31;
    let mut columns: Vec<ArrayRef> = (0..4)
        .map(|_| {
            let array: ArrayRef = Arc::new(
                FixedSizeBinaryArray::try_from_iter((0..rows as u128).map(u128::to_be_bytes))
                    .unwrap(),
            );
            array
        })
        .collect();
    for _ in 0..3 {
        columns.push(Arc::new(Float64Array::from_iter_values(
            (0..rows).map(|row| row as f64),
        )));
    }
    let source = checked(columns, rows);
    let pool = pool(32 * 1024 * 1024);
    let selected = source
        .take_reserved(
            &UInt32Array::from_iter_values((0..rows as u32).rev()),
            &pool,
            &pse_columnar::CancellationToken::new(),
        )
        .unwrap();
    assert_eq!(selected.batch().num_rows(), rows);
    assert_eq!(selected.batch().num_columns(), 7);
    assert!(selected.local_admission().is_some());
    let values = selected
        .batch()
        .column(6)
        .as_any()
        .downcast_ref::<Float64Array>()
        .unwrap();
    for row in 0..rows {
        assert_eq!(values.value(row), (rows - 1 - row) as f64);
    }
    assert!(pool.reserved() > 0);
    drop(selected);
    assert_eq!(pool.reserved(), 0);
}

#[test]
fn checked_take_repeated_long_string_and_list_refuse_insufficient_pool() {
    let long = "x".repeat(32 * 1024);
    let text = checked(vec![Arc::new(StringArray::from(vec!["s", &long]))], 2);
    let list = checked(
        vec![Arc::new(ListArray::from_iter_primitive::<
            arrow_array::types::Int64Type,
            _,
            _,
        >([
            Some(vec![Some(1)]),
            Some((0..8192).map(Some).collect()),
        ]))],
        2,
    );
    for (source, budget) in [(text, 2 * 1024 * 1024), (list, 8 * 1024 * 1024)] {
        let pool = pool(budget);
        let cancel = pse_columnar::CancellationToken::new();
        let once = source
            .take_reserved(&UInt32Array::from(vec![1, 0]), &pool, &cancel)
            .unwrap();
        assert_eq!(once.batch().num_rows(), 2);
        drop(once);
        assert_eq!(pool.reserved(), 0);
        let result = source.take_reserved(&UInt32Array::from(vec![1; 32]), &pool, &cancel);
        assert!(
            matches!(result, Err(RelationError::Canon(pse_columnar::CanonError::NativeResource(datafusion_common::DataFusionError::ResourcesExhausted(message)))) if message.contains("relations:checked-take"))
        );
        assert_eq!(
            pool.reserved(),
            0,
            "refused take releases occurrence-count scratch"
        );
    }
}

#[test]
fn checked_take_aliasing_children_keep_conservative_allowance() {
    let long = "x".repeat(32 * 1024);
    let fields = UnionFields::try_new([0], [Field::new("text", DataType::Utf8, false)]).unwrap();
    let dense: ArrayRef = Arc::new(
        arrow_array::UnionArray::try_new(
            fields,
            vec![0i8; 16].into(),
            Some(vec![0i32; 16].into()),
            vec![Arc::new(StringArray::from(vec![long.as_str()]))],
        )
        .unwrap(),
    );
    let nested: ArrayRef = Arc::new(
        ListArray::try_new(
            Arc::new(Field::new("item", dense.data_type().clone(), false)),
            arrow::buffer::OffsetBuffer::new((0..=16i32).collect::<Vec<_>>().into()),
            dense.clone(),
            None,
        )
        .unwrap(),
    );
    let runs: ArrayRef = Arc::new(
        arrow_array::RunArray::<arrow_array::types::Int32Type>::try_new(
            &arrow_array::Int32Array::from(vec![8, 16]),
            &StringArray::from(vec![long.as_str(), "s"]),
        )
        .unwrap(),
    );
    // Alternating logical runs produces repeated physical values despite unique indices.
    let indices = UInt32Array::from_iter_values((0..8).flat_map(|row| [row, row + 8]));
    for column in [dense, nested, runs] {
        let source = checked(vec![column], 16);
        let pool = pool(2 * 1024 * 1024);
        // A whole-source allowance alone fits, so refusal demonstrates the alias guard.
        assert!(
            pse_columnar::allocation_extent::algorithm_decode_extent(source.batch()).unwrap()
                < 2 * 1024 * 1024
        );
        let result = source.take_reserved(&indices, &pool, &pse_columnar::CancellationToken::new());
        assert!(
            matches!(result, Err(RelationError::Canon(pse_columnar::CanonError::NativeResource(datafusion_common::DataFusionError::ResourcesExhausted(message)))) if message.contains("relations:checked-take"))
        );
        assert_eq!(pool.reserved(), 0);
    }
}

#[test]
fn checked_take_many_nested_aliases_refuse_before_kernel_allocation() {
    let long = "x".repeat(32 * 1024);
    let dense: ArrayRef = Arc::new(
        arrow_array::UnionArray::try_new(
            UnionFields::try_new([0], [Field::new("text", DataType::Utf8, false)]).unwrap(),
            vec![0i8; 1024].into(),
            Some(vec![0i32; 1024].into()),
            vec![Arc::new(StringArray::from(vec![long.as_str()]))],
        )
        .unwrap(),
    );
    let parent: ArrayRef = Arc::new(
        arrow_array::FixedSizeListArray::try_new(
            Arc::new(Field::new("item", dense.data_type().clone(), false)),
            1024,
            dense,
            None,
        )
        .unwrap(),
    );
    let source = checked(vec![parent], 1);
    let pool = pool(8 * 1024 * 1024);
    assert!(
        pse_columnar::allocation_extent::algorithm_decode_extent(source.batch()).unwrap()
            < 8 * 1024 * 1024
    );
    // One parent selection expands 1024 aliases into 32 MiB of string bytes.
    // Require the pre-kernel reservation owner, never a refusal after Arrow take.
    let result = source.take_reserved(
        &UInt32Array::from(vec![0]),
        &pool,
        &pse_columnar::CancellationToken::new(),
    );
    assert!(matches!(
        result,
        Err(RelationError::Canon(pse_columnar::CanonError::NativeResource(
            datafusion_common::DataFusionError::ResourcesExhausted(message),
        ))) if message.contains("relations:checked-take")
    ));
    assert_eq!(pool.reserved(), 0);
}

#[test]
fn checked_take_indices_refuse_before_budget_even_without_columns() {
    let cancel = pse_columnar::CancellationToken::new();
    for source in [
        checked(vec![], 2),
        checked(vec![Arc::new(StringArray::from(vec!["a", "b"]))], 2),
    ] {
        let pool = pool(0);
        let result = source.take_reserved(&UInt32Array::from(vec![0, 2]), &pool, &cancel);
        assert!(matches!(
            result,
            Err(RelationError::Arrow(
                arrow_schema::ArrowError::ComputeError(_)
            ))
        ));
        let result = source.take_reserved(&UInt32Array::from(vec![Some(0), None]), &pool, &cancel);
        assert!(matches!(result, Err(RelationError::Contract { .. })));
        assert_eq!(pool.reserved(), 0);
    }
    let empty = checked(vec![], 0);
    assert!(matches!(
        empty.take_reserved(&UInt32Array::from(vec![0]), &pool(0), &cancel),
        Err(RelationError::Arrow(
            arrow_schema::ArrowError::ComputeError(_)
        ))
    ));
    let selected = checked(vec![], 2)
        .take_reserved(&UInt32Array::from(vec![1, 0, 1]), &pool(16 * 1024), &cancel)
        .unwrap();
    assert_eq!(selected.batch().num_columns(), 0);
    assert_eq!(selected.batch().num_rows(), 3);
}

#[test]
fn checked_take_forecast_scratch_is_admitted_before_allocation() {
    let source = checked(vec![Arc::new(Float64Array::from(vec![0.0; 4096]))], 4096);
    let pool = pool(0);
    let result = source.take_reserved(
        &UInt32Array::from(vec![0]),
        &pool,
        &pse_columnar::CancellationToken::new(),
    );
    assert!(matches!(
        result,
        Err(RelationError::Canon(pse_columnar::CanonError::NativeResource(
            datafusion_common::DataFusionError::ResourcesExhausted(message),
        ))) if message.contains("relations:checked-take-forecast")
    ));
    assert_eq!(pool.reserved(), 0);
}

#[test]
fn checked_take_small_selection_does_not_reserve_unselected_values() {
    let long = "x".repeat(1024 * 1024);
    let sources = [
        checked(
            vec![Arc::new(StringArray::from(vec!["selected", &long]))],
            2,
        ),
        checked(
            vec![Arc::new(ListArray::from_iter_primitive::<
                arrow_array::types::Int64Type,
                _,
                _,
            >([
                Some(vec![Some(7)]),
                Some((0..131072).map(Some).collect()),
            ]))],
            2,
        ),
    ];
    for source in sources {
        let pool = pool(64 * 1024);
        let cancel = pse_columnar::CancellationToken::new();
        let expected = source.batch().column(0).slice(0, 1);
        let selected = source
            .take_reserved(&UInt32Array::from(vec![0, 0, 0]), &pool, &cancel)
            .unwrap();
        assert_eq!(selected.batch().num_rows(), 3);
        for row in 0..3 {
            assert_eq!(
                selected.batch().column(0).slice(row, 1).to_data(),
                expected.to_data()
            );
        }
        let escaped = selected.batch().column(0).clone();
        drop(selected);
        drop(source);
        assert!(pool.reserved() > 0);
        assert_eq!(escaped.len(), 3);
        drop(escaped);
        assert_eq!(pool.reserved(), 0);
    }
    let source = checked(
        vec![Arc::new(Float64Array::from(vec![1.0; 131072]))],
        131072,
    );
    let pool = pool(64 * 1024);
    let empty = source
        .take_reserved(
            &UInt32Array::from(Vec::<u32>::new()),
            &pool,
            &pse_columnar::CancellationToken::new(),
        )
        .unwrap();
    assert_eq!(empty.batch().num_rows(), 0);
}

#[test]
fn checked_take_selected_nested_slices_preserve_offsets_and_nulls() {
    let strings: ArrayRef = Arc::new(StringArray::from(vec![
        Some("unselected"),
        None,
        Some("kept"),
    ]));
    let lists: ArrayRef = Arc::new(ListArray::from_iter_primitive::<
        arrow_array::types::Int64Type,
        _,
        _,
    >([
        Some(vec![Some(0); 131072]),
        None,
        Some(vec![Some(8), None]),
    ]));
    let structure = arrow_array::StructArray::from(vec![
        (
            Arc::new(Field::new("text", strings.data_type().clone(), true)),
            strings,
        ),
        (
            Arc::new(Field::new("items", lists.data_type().clone(), true)),
            lists,
        ),
    ]);
    let fixed = arrow_array::FixedSizeListArray::from_iter_primitive::<
        arrow_array::types::Int64Type,
        _,
        _,
    >(
        [
            Some(vec![Some(0), Some(1)]),
            None,
            Some(vec![Some(8), None]),
        ],
        2,
    );
    let source = checked(
        vec![Arc::new(structure.slice(1, 2)), Arc::new(fixed.slice(1, 2))],
        2,
    );
    let selected = source
        .take_reserved(
            &UInt32Array::from(vec![1, 0, 1]),
            &pool(64 * 1024),
            &pse_columnar::CancellationToken::new(),
        )
        .unwrap();
    for (output, input) in [1, 0, 1].into_iter().enumerate() {
        for column in 0..2 {
            assert_eq!(
                selected.batch().column(column).slice(output, 1).to_data(),
                source.batch().column(column).slice(input, 1).to_data()
            );
        }
    }
}

#[test]
fn checked_take_empty_dictionary_and_views_do_not_retain_unselected_values() {
    let dictionary: ArrayRef = Arc::new(
        arrow_array::DictionaryArray::<arrow_array::types::Int8Type>::try_new(
            arrow_array::Int8Array::from(vec![0]),
            Arc::new(StringArray::from(
                (0..129)
                    .map(|n| format!("{n}:{}", "x".repeat(8192)))
                    .collect::<Vec<_>>(),
            )),
        )
        .unwrap(),
    );
    let view: ArrayRef = Arc::new(arrow_array::StringViewArray::from(vec![
        "x".repeat(1024 * 1024),
    ]));
    for column in [dictionary, view] {
        let source = checked(vec![column], 1);
        let pool = pool(64 * 1024);
        let empty = source
            .take_reserved(
                &UInt32Array::from(Vec::<u32>::new()),
                &pool,
                &pse_columnar::CancellationToken::new(),
            )
            .unwrap();
        assert_eq!(empty.batch().num_rows(), 0);
        let data = empty.batch().column(0).to_data();
        // An empty UTF-8 dictionary child retains one zero offset (four bytes),
        // while its values and the view's external storage must be absent.
        assert!(data.get_buffer_memory_size() <= size_of::<i32>());
        assert!(data.child_data().iter().all(|child| child.is_empty()));
        drop(empty);
        assert_eq!(pool.reserved(), data.get_buffer_memory_size());
        drop(data);
        assert_eq!(pool.reserved(), 0);
    }
}
