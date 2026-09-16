// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Signed storage retains domain bounds at imported and generated boundaries.
#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "independent contract assertions"
)]

use ::arrow::buffer::NullBuffer;
use arrow_array::{Array, ArrayRef, Int64Array, StructArray};
use arrow_schema::{DataType, Field};
use pse_schema::{
    RegistryBuilder, arrow,
    model::{ExtensionUse, FieldContract, IntegerRange},
};
use std::sync::Arc;

#[test]
fn visible_values_obey_bounds_while_null_parents_mask_children() {
    let registry = RegistryBuilder::new().build().unwrap();
    let child = IntegerRange::nonnegative(255).field("offset");
    let field = Field::new("parent", DataType::Struct(vec![child.clone()].into()), true);
    for (value, visible, valid) in [
        (-1, true, false),
        (0, true, true),
        (255, true, true),
        (256, true, false),
        (-1, false, true),
    ] {
        let structure = StructArray::new(
            vec![child.clone()].into(),
            vec![Arc::new(Int64Array::from(vec![value]))],
            Some(NullBuffer::from(vec![visible])),
        );
        assert_eq!(
            pse_relations::validate::validate_column(&registry, &field, &structure).is_ok(),
            valid,
            "{value}/{visible}"
        );
    }
}

#[test]
fn source_span_offsets_reject_negative_oversized_and_reversed_ranges() {
    let registry = pse_schema::registry().unwrap();
    let field =
        arrow::field_for(registry, &FieldContract::extended(ExtensionUse::SourceSpan)).unwrap();
    let DataType::Struct(children) = field.data_type() else {
        panic!("span")
    };
    let document: ArrayRef = Arc::new(
        arrow_array::FixedSizeBinaryArray::try_from_iter([[0_u8; 16]].into_iter()).unwrap(),
    );
    for (start, end, valid) in [
        (0, 0, true),
        (0, i64::from(u32::MAX), true),
        (-1, 0, false),
        (0, i64::from(u32::MAX) + 1, false),
        (2, 1, false),
    ] {
        let structure = StructArray::new(
            children.clone(),
            vec![
                Arc::clone(&document),
                Arc::new(Int64Array::from(vec![start])),
                Arc::new(Int64Array::from(vec![end])),
            ],
            None,
        );
        assert_eq!(
            pse_relations::validate::validate_column(registry, &field, &structure).is_ok(),
            valid,
            "{start}..{end}"
        );
        assert_eq!(structure.slice(0, 1).data_type(), field.data_type());
    }
}

#[test]
fn ordinal_domain_rejects_negative_values_and_missing_domain_metadata() {
    use pse_schema::model::{Authority, Namespace, RelationDecl, SnapshotClass};
    let mut builder = RegistryBuilder::new();
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "ordinals",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "ordinal domain",
        )
        .pk(&["id"])
        .columns(vec![
            FieldContract::key("id", FieldContract::id(), "identity"),
            FieldContract::payload(
                "reference",
                FieldContract::extended(ExtensionUse::OrdinalRef {
                    target: "authored.ordinals",
                }),
                "ordinal",
            ),
        ]),
    );
    let registry = builder.build().unwrap();
    let registry = &registry;
    let field = arrow::field_for(
        registry,
        &FieldContract::extended(ExtensionUse::OrdinalRef {
            target: "authored.ordinals",
        }),
    )
    .unwrap();
    for (value, valid) in [(-1, false), (0, true), (i64::MAX, true)] {
        assert_eq!(
            pse_relations::validate::validate_column(
                registry,
                &field,
                &Int64Array::from(vec![value])
            )
            .is_ok(),
            valid
        );
    }
    let mut missing = field;
    missing
        .metadata_mut()
        .remove(pse_schema::model::integer_range::KEY_INTEGER_RANGE);
    assert!(pse_relations::validate::validate_field(registry, &missing).is_err());
}
