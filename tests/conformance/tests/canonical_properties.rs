// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded generated representation and distinguishing-value canonicalization cases.
#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "bounded test fixture construction reports direct assertion failures"
)]

#[path = "../src/canon_fixtures.rs"]
mod canon_fixtures;

use datafusion::arrow::{
    array::{
        ArrayRef, DictionaryArray, Float64Array, Int32Array, ListArray, RecordBatch, StringArray,
        StructArray, UInt32Array, UInt64Array,
    },
    buffer::{NullBuffer, OffsetBuffer, ScalarBuffer},
    compute::{cast, take_record_batch},
    datatypes::{DataType, Int32Type},
};
use proptest::prelude::*;
use pse_schema::model::Cell;
use std::sync::Arc;

type GeneratedRow = (u64, Vec<u64>, bool, bool);

fn logical_rows(rows: &[GeneratedRow]) -> Vec<Vec<Cell>> {
    rows.iter()
        .enumerate()
        .map(|(index, (bits, children, valid, choice))| {
            let valid = *valid || index == 0;
            vec![
                Cell::U64(u64::try_from(index).expect("bounded row") + 1),
                if valid {
                    Cell::F64(f64::from_bits(*bits))
                } else {
                    Cell::Null
                },
                if valid {
                    Cell::Enum(if *choice { "a" } else { "b" })
                } else {
                    Cell::Null
                },
                if valid {
                    Cell::List(children.iter().copied().map(Cell::U64).collect())
                } else {
                    Cell::Null
                },
                if valid {
                    Cell::Struct(vec![Cell::U64(children.first().copied().unwrap_or(0))])
                } else {
                    Cell::Null
                },
            ]
        })
        .collect()
}

fn alternate(source: &RecordBatch, rows: &[GeneratedRow], hidden: u64) -> RecordBatch {
    let valid: Vec<_> = rows
        .iter()
        .enumerate()
        .map(|(i, row)| row.2 || i == 0)
        .collect();
    let mask = Some(NullBuffer::from(valid.clone()));
    let floats: ArrayRef = Arc::new(Float64Array::new(
        ScalarBuffer::from(
            rows.iter()
                .zip(&valid)
                .map(|(row, valid)| {
                    if *valid {
                        f64::from_bits(row.0)
                    } else {
                        f64::from_bits(hidden)
                    }
                })
                .collect::<Vec<_>>(),
        ),
        mask.clone(),
    ));
    let dictionary: ArrayRef = Arc::new(
        DictionaryArray::<Int32Type>::try_new(
            Int32Array::from(
                rows.iter()
                    .zip(&valid)
                    .map(|(row, valid)| valid.then_some(i32::from(row.3)))
                    .collect::<Vec<_>>(),
            ),
            Arc::new(StringArray::from(vec!["b", "a", "unused"])),
        )
        .expect("independent dictionary code assignment"),
    );
    let (list, structure) = nested(source, rows, hidden, &valid, mask);
    RecordBatch::try_new(
        source.schema(),
        vec![
            Arc::clone(source.column(0)),
            floats,
            cast(dictionary.as_ref(), &DataType::Utf8).expect("native dictionary decode"),
            list,
            structure,
        ],
    )
    .expect("alternate physical representation")
}

fn nested(
    source: &RecordBatch,
    rows: &[GeneratedRow],
    hidden: u64,
    valid: &[bool],
    mask: Option<NullBuffer>,
) -> (ArrayRef, ArrayRef) {
    let mut offsets = vec![0_i32];
    let mut children = Vec::new();
    for (row, valid) in rows.iter().zip(valid) {
        if *valid {
            children.extend_from_slice(&row.1);
        } else {
            children.extend(std::iter::repeat_n(
                hidden,
                usize::try_from(hidden % 5).expect("bounded hidden length"),
            ));
        }
        offsets.push(i32::try_from(children.len()).expect("bounded offsets"));
    }
    let schema = source.schema();
    let DataType::List(child) = schema.field(3).data_type() else {
        panic!("list");
    };
    let list: ArrayRef = Arc::new(
        ListArray::try_new(
            Arc::clone(child),
            OffsetBuffer::new(ScalarBuffer::from(offsets)),
            Arc::new(UInt64Array::from(children)),
            mask.clone(),
        )
        .expect("masked list child garbage"),
    );
    let DataType::Struct(fields) = schema.field(4).data_type() else {
        panic!("struct");
    };
    let structure: ArrayRef = Arc::new(
        StructArray::try_new(
            fields.clone(),
            vec![Arc::new(UInt64Array::from(
                rows.iter()
                    .zip(valid)
                    .map(|(row, valid)| {
                        if *valid {
                            row.1.first().copied().unwrap_or(0)
                        } else {
                            hidden
                        }
                    })
                    .collect::<Vec<_>>(),
            ))],
            mask,
        )
        .expect("masked struct child garbage"),
    );
    (list, structure)
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 128, max_shrink_iters: 1024, ..ProptestConfig::default() })]

    #[test]
    fn generated_null_dictionary_nested_slices_and_order_preserve_complete_preimages(
        rows in prop::collection::vec((any::<u64>(), prop::collection::vec(any::<u64>(), 0..6), any::<bool>(), any::<bool>()), 1..33),
        hidden in any::<u64>(),
        rotation in 0usize..32,
        split in 0usize..33,
        reverse in any::<bool>(),
    ) {
        let (reg, contract, _) = canon_fixtures::fixture(false);
        let spec = reg.relation("authored.values").expect("registry relation");
        let cells = logical_rows(&rows);
        let source = pse_relations::cells::batch_from_cells(&reg, spec, &cells).expect("declared generated rows");
        let alternative = alternate(&source, &rows, hidden);
        pse_relations::validate::validate_batch(&reg, spec, &alternative).expect("actual recursive value admission");
        // Decode actual rows before comparing bytes; hidden children and dictionary codes
        // must preserve the declared values, including original floating-point bits.
        let decoded = pse_relations::cells::cells_from_batch(&reg, spec, &alternative).expect("decode");
        let render = |values: &[Vec<Cell>]| values.iter().map(|row| row.iter().map(Cell::literal_spec).collect::<Vec<_>>()).collect::<Vec<_>>();
        prop_assert_eq!(render(&decoded), render(&cells));
        let mut order: Vec<_> = (0..u32::try_from(rows.len()).expect("bounded")).collect();
        let count = order.len();
        order.rotate_left(rotation % count);
        if reverse { order.reverse(); }
        let permuted = take_record_batch(&alternative, &UInt32Array::from(order)).expect("permutation");
        let split = split.min(count);
        let expected = canon_fixtures::canonical(&contract, &[source]);
        let actual = canon_fixtures::canonical(&contract, &[permuted.slice(0, split), permuted.slice(split, count - split)]);
        prop_assert_eq!(actual.preimage, expected.preimage);
        prop_assert_eq!(actual.logical_hash, expected.logical_hash);
    }

    #[test]
    fn generated_visible_nested_values_always_change_the_preimage(
        rows in prop::collection::vec((any::<u64>(), prop::collection::vec(any::<u64>(), 0..6), any::<bool>(), any::<bool>()), 1..33),
        additional in any::<u64>(),
    ) {
        let (reg, contract, _) = canon_fixtures::fixture(false);
        let spec = reg.relation("authored.values").expect("registry relation");
        let mut cells = logical_rows(&rows);
        let source = pse_relations::cells::batch_from_cells(&reg, spec, &cells).expect("valid source");
        let Cell::List(children) = &mut cells[0][3] else { panic!("first row always visible"); };
        children.push(Cell::U64(additional));
        let changed = pse_relations::cells::batch_from_cells(&reg, spec, &cells).expect("valid distinct value");
        let expected = canon_fixtures::canonical(&contract, &[source]);
        let actual = canon_fixtures::canonical(&contract, &[changed]);
        prop_assert_ne!(actual.preimage, expected.preimage);
    }
}
