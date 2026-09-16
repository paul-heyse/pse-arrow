// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual registry contracts with equivalent Arrow storage representations.
#![allow(
    dead_code,
    clippy::expect_used,
    clippy::panic,
    reason = "shared fixture operations are exercised by separate integration binaries"
)]

use datafusion::arrow::{
    array::{
        Array, ArrayRef, DictionaryArray, Float64Array, Int32Array, ListArray, RecordBatch,
        StringArray, StructArray, UInt64Array,
    },
    buffer::{NullBuffer, OffsetBuffer, ScalarBuffer},
    datatypes::{DataType, Int32Type},
};
use pse_catalog::{EncodingPolicy, RelationContract};
use pse_schema::{
    Registry, RegistryBuilder,
    model::{
        Authority, Cell, EnumDecl, EnumMember, FieldContract, Namespace, RelationDecl,
        SnapshotClass,
    },
};
use std::sync::Arc;

/// A schema-admitted fixture whose null payload and dictionary codes vary independently.
pub(crate) fn fixture(alternate: bool) -> (Arc<Registry>, RelationContract, RecordBatch) {
    let reg = registry();
    let spec = reg.relation("authored.values").expect("relation");
    let rows = vec![
        vec![
            Cell::U64(3),
            Cell::F64(-0.0),
            Cell::Enum("a"),
            Cell::List(vec![Cell::U64(3)]),
            Cell::Struct(vec![Cell::U64(3)]),
        ],
        vec![Cell::U64(1), Cell::Null, Cell::Null, Cell::Null, Cell::Null],
        vec![
            Cell::U64(2),
            Cell::F64(f64::NAN),
            Cell::Enum("b"),
            Cell::List(vec![Cell::U64(5)]),
            Cell::Struct(vec![Cell::U64(5)]),
        ],
    ];
    let source = pse_relations::cells::batch_from_cells(&reg, spec, &rows).expect("typed rows");
    let contract =
        RelationContract::from_spec(&reg, spec, EncodingPolicy::IpcFile).expect("contract");
    let fields = source.schema();
    let mask = Some(NullBuffer::from(vec![true, false, true]));
    let float: ArrayRef = Arc::new(Float64Array::new(
        ScalarBuffer::from(vec![
            -0.0,
            if alternate { 999.0 } else { -27.0 },
            f64::from_bits(if alternate {
                0xfff8_0000_0000_0002
            } else {
                0x7ff8_0000_0000_0001
            }),
        ]),
        mask.clone(),
    ));
    let dictionary: ArrayRef = Arc::new(
        DictionaryArray::<Int32Type>::try_new(
            Int32Array::from(if alternate {
                vec![Some(1), None, Some(0)]
            } else {
                vec![Some(0), None, Some(1)]
            }),
            Arc::new(StringArray::from(if alternate {
                vec!["b", "a", "unused"]
            } else {
                vec!["a", "b"]
            })),
        )
        .expect("dictionary"),
    );
    let DataType::List(child) = fields.field(3).data_type() else {
        panic!("list");
    };
    let list: ArrayRef = Arc::new(
        ListArray::try_new(
            Arc::clone(child),
            OffsetBuffer::new(ScalarBuffer::from(if alternate {
                vec![0, 1, 1, 2]
            } else {
                vec![0, 1, 3, 4]
            })),
            Arc::new(UInt64Array::from(if alternate {
                vec![3, 5]
            } else {
                vec![3, 88, 99, 5]
            })),
            mask.clone(),
        )
        .expect("list"),
    );
    let DataType::Struct(children) = fields.field(4).data_type() else {
        panic!("struct");
    };
    let structure: ArrayRef = Arc::new(
        StructArray::try_new(
            children.clone(),
            vec![Arc::new(UInt64Array::from(vec![
                3,
                if alternate { 88 } else { 0 },
                5,
            ]))],
            mask,
        )
        .expect("struct"),
    );
    let batch = RecordBatch::try_new(
        fields,
        vec![source.column(0).clone(), float, dictionary, list, structure],
    )
    .expect("physical fixture");
    pse_relations::validate::validate_batch(&reg, spec, &batch).expect("actual value admission");
    (reg, contract, batch)
}

/// Keep complete preimages and sorted batches so tests compare evidence, not hashes alone.
pub(crate) fn canonical(
    contract: &RelationContract,
    batches: &[RecordBatch],
) -> pse_ids::CanonicalOutput {
    pse_ids::canonicalize(
        &contract.canonical,
        batches,
        pse_ids::FixedBudget::new(64 << 20).as_ref(),
        pse_ids::CanonicalizeOptions {
            keep_preimage: true,
            keep_sorted: true,
            ..Default::default()
        },
    )
    .expect("canonicalization")
}

/// Rewrite a visible floating-point value while retaining the exact schema/null mask.
pub(crate) fn replace_float(batch: &RecordBatch, row: usize, value: f64) -> RecordBatch {
    let old = batch
        .column(1)
        .as_any()
        .downcast_ref::<Float64Array>()
        .expect("float");
    let mut values = old.values().to_vec();
    values[row] = value;
    let mut columns = batch.columns().to_vec();
    columns[1] = Arc::new(Float64Array::new(
        ScalarBuffer::from(values),
        old.nulls().cloned(),
    ));
    RecordBatch::try_new(batch.schema(), columns).expect("changed value")
}

fn registry() -> Arc<Registry> {
    let mut builder = RegistryBuilder::new();
    builder.declare_enum(EnumDecl::platform(
        "Choice",
        vec![EnumMember::new("a", "A"), EnumMember::new("b", "B")],
    ));
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "values",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "Canonical fixture",
        )
        .pk(&["id"])
        .columns(vec![
            FieldContract::key("id", FieldContract::native(DataType::UInt64), "Key"),
            FieldContract::payload("value", FieldContract::native(DataType::Float64), "Float")
                .optional(),
            FieldContract::payload("choice", FieldContract::enumeration("Choice"), "Choice")
                .optional(),
            FieldContract::payload(
                "list",
                FieldContract::list(FieldContract::native(DataType::UInt64)),
                "List",
            )
            .optional(),
            FieldContract::payload(
                "structure",
                FieldContract::structure(vec![
                    FieldContract::native(DataType::UInt64).with_name("child"),
                ]),
                "Struct",
            )
            .optional(),
        ]),
    );
    Arc::new(builder.build().expect("registry"))
}
