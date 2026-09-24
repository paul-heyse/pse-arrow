// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Primitive canonicalization across Arrow representations, including nonfinite bit patterns.
//! Schema agreement is required; domain validity is tested at relation admission separately.
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
    compute::cast,
    datatypes::{DataType, Int32Type},
};
use pse_columnar::CanonicalContract;
use pse_schema::{
    Registry, RegistryBuilder,
    model::{
        Authority, EnumDecl, EnumMember, FieldContract, Namespace, RelationDecl, SnapshotClass,
    },
};
use std::sync::Arc;

/// A schema-admitted fixture whose null payload and dictionary codes vary independently.
pub(crate) fn fixture(alternate: bool) -> (Arc<Registry>, CanonicalContract, RecordBatch) {
    let reg = registry();
    let spec = reg.relation("authored.values").expect("relation");
    let rows = vec![
        vec![
            serde_json::json!(["u64", 3]),
            serde_json::json!(["f64", format!("{:016x}", f64::to_bits(-0.0))]),
            serde_json::json!(["enum", "a"]),
            serde_json::json!(["list", vec![serde_json::json!(["u64", 3])]]),
            serde_json::json!(["struct", vec![serde_json::json!(["u64", 3])]]),
        ],
        vec![
            serde_json::json!(["u64", 1]),
            serde_json::json!(["null", null]),
            serde_json::json!(["null", null]),
            serde_json::json!(["null", null]),
            serde_json::json!(["null", null]),
        ],
        vec![
            serde_json::json!(["u64", 2]),
            serde_json::json!(["f64", format!("{:016x}", (f64::NAN).to_bits())]),
            serde_json::json!(["enum", "b"]),
            serde_json::json!(["list", vec![serde_json::json!(["u64", 5])]]),
            serde_json::json!(["struct", vec![serde_json::json!(["u64", 5])]]),
        ],
    ];
    let source = pse_relations::testing::untrusted_batch_from_literals(&reg, spec, &rows)
        .expect("typed rows");
    let contract = pse_relations::canonical::contract(&reg, spec).expect("contract");
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
    let choice = decoded_choices(alternate);
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
        vec![source.column(0).clone(), float, choice, list, structure],
    )
    .expect("physical fixture");
    pse_relations::validate::validate_schema(&reg, spec, batch.schema().as_ref())
        .expect("canonical input schema");
    (reg, contract, batch)
}

fn decoded_choices(alternate: bool) -> ArrayRef {
    let dictionary = DictionaryArray::<Int32Type>::try_new(
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
    .expect("dictionary");
    // Arrow decodes external dictionary encoding before exact admission to the
    // canonical Utf8 enum declaration; dictionary storage is not a PSE enum form.
    cast(&dictionary, &DataType::Utf8).expect("native dictionary decode")
}

/// Keep complete preimages and sorted batches so tests compare evidence, not hashes alone.
pub(crate) fn canonical(
    contract: &CanonicalContract,
    batches: &[RecordBatch],
) -> pse_columnar::CanonicalOutput {
    pse_columnar::canonicalize(
        contract,
        batches,
        &{
            let pool: Arc<dyn pse_columnar::MemoryPool> =
                Arc::new(pse_columnar::GreedyMemoryPool::new(64 << 20));
            pool
        },
        pse_columnar::CanonicalizeOptions {
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
