// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(clippy::unwrap_used, reason = "exact native literal fixtures")]
use super::*;
use arrow_array::{Float32Array, Float64Array};
use arrow_schema::{DataType, Field, IntervalUnit, TimeUnit, UnionFields, UnionMode};

#[test]
fn scalar_adapter_keeps_nested_metadata_and_binding_uses_declared_execution_fields() {
    let registry = crate::registry().unwrap();
    let contract = crate::model::FieldContract::native(DataType::List(Arc::new(
        Field::new("item", DataType::Float32, true).with_metadata(std::collections::HashMap::from(
            [("custom.meaning".into(), "retained".into())],
        )),
    )));
    let declared = Arc::new(contract.field().clone());
    let literal = NativeLiteral::from_json(declared, r#"["list",[["f32","7fc00001"]]]"#).unwrap();
    let bound = literal.bind(registry).unwrap();
    assert_eq!(
        bound.field().as_ref(),
        &crate::arrow::field_for(registry, &contract).unwrap()
    );
    assert_eq!(bound.as_json(), literal.as_json());
    assert_eq!(
        bound.scalar().unwrap().to_array().unwrap().data_type(),
        bound.field().data_type()
    );
}

#[test]
fn malformed_native_types_are_rejected_before_null_construction() {
    for kind in [
        DataType::Time32(TimeUnit::Nanosecond),
        DataType::Time64(TimeUnit::Second),
        DataType::FixedSizeBinary(-1),
        DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Int32, true)), -1),
        DataType::Decimal32(10, 0),
        DataType::Dictionary(Box::new(DataType::Float32), Box::new(DataType::Int32)),
        DataType::RunEndEncoded(
            Arc::new(Field::new("ends", DataType::UInt32, false)),
            Arc::new(Field::new("values", DataType::Int32, true)),
        ),
        DataType::Map(
            Arc::new(Field::new("entries", DataType::Int32, false)),
            false,
        ),
        DataType::Union(UnionFields::empty(), UnionMode::Dense),
    ] {
        assert!(
            NativeLiteral::from_json(
                Arc::new(Field::new("value", kind, true)),
                r#"["null",null]"#,
            )
            .is_err()
        );
    }
}

#[test]
fn field_directed_roundtrips_preserve_native_layout_and_metadata() {
    let child = Arc::new(Field::new("child", DataType::Float32, true).with_metadata(
        std::collections::HashMap::from([("custom.meaning".into(), "sample".into())]),
    ));
    let entries = Arc::new(Field::new(
        "entries",
        DataType::Struct(
            vec![
                Arc::new(Field::new("key", DataType::Utf8, false)),
                Arc::clone(&child),
            ]
            .into(),
        ),
        false,
    ));
    let cases = vec![
        (DataType::Float16, r#"["f16","fe01"]"#),
        (DataType::Float32, r#"["f32","7fc00001"]"#),
        (DataType::UInt64, r#"["u64",18446744073709551615]"#),
        (
            DataType::Decimal256(76, 7),
            r#"["decimal","1234567890123456789012345678901234567890"]"#,
        ),
        (
            DataType::Timestamp(TimeUnit::Microsecond, Some("UTC".into())),
            r#"["i64",-5]"#,
        ),
        (DataType::Time32(TimeUnit::Second), r#"["i64",11]"#),
        (DataType::Date32, r#"["i64",-9]"#),
        (
            DataType::Duration(TimeUnit::Nanosecond),
            r#"["i64",9223372036854775807]"#,
        ),
        (
            DataType::Interval(IntervalUnit::DayTime),
            r#"["interval",[-7,55]]"#,
        ),
        (
            DataType::Interval(IntervalUnit::MonthDayNano),
            r#"["interval",[7,-8,9223372036854775807]]"#,
        ),
        (DataType::BinaryView, r#"["bytes","00ff78"]"#),
        (DataType::FixedSizeBinary(3), r#"["bytes","00ff78"]"#),
        (
            DataType::ListView(Arc::clone(&child)),
            r#"["list",[["f32","80000000"],["null",null]]]"#,
        ),
        (
            DataType::LargeListView(Arc::clone(&child)),
            r#"["list",[]]"#,
        ),
        (
            DataType::Map(entries, false),
            r#"["map",[["struct",[["text","a"],["f32","3f800000"]]]]]"#,
        ),
        (
            DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Float32)),
            r#"["f32","ffc00021"]"#,
        ),
        (
            DataType::RunEndEncoded(
                Arc::new(Field::new("ends", DataType::Int32, false)),
                Arc::clone(&child),
            ),
            r#"["f32","80000000"]"#,
        ),
        (
            DataType::Union(
                UnionFields::try_new(
                    vec![2, 7],
                    vec![
                        Field::new("a", DataType::Int32, true),
                        Field::new("b", DataType::Float32, true),
                    ],
                )
                .unwrap(),
                UnionMode::Dense,
            ),
            r#"["union",[7,["null",null]]]"#,
        ),
    ];
    for (kind, text) in cases {
        let field = Arc::new(Field::new("literal", kind, true));
        let literal = NativeLiteral::from_json(Arc::clone(&field), text).unwrap();
        assert_eq!(literal.as_json(), text, "{field:?}");
        assert_eq!(literal.array().data_type(), field.data_type());
        assert_eq!(
            NativeLiteral::new(field, Arc::clone(literal.array())).unwrap(),
            literal
        );
    }
}
#[test]
fn exact_float_bits_survive_without_semantic_validation_or_normalization() {
    for bits in [0x8000_0000, 0x7fc0_0001, 0xffc0_0137] {
        let field = Arc::new(Field::new("value", DataType::Float32, false));
        let literal = NativeLiteral::new(
            Arc::clone(&field),
            Arc::new(Float32Array::from(vec![f32::from_bits(bits)])),
        )
        .unwrap();
        let decoded = NativeLiteral::from_json(field, literal.as_json()).unwrap();
        assert_eq!(
            decoded
                .array()
                .as_any()
                .downcast_ref::<Float32Array>()
                .unwrap()
                .value(0)
                .to_bits(),
            bits
        );
    }
    let field = Arc::new(Field::new("value", DataType::Float64, false));
    let literal = NativeLiteral::new(
        Arc::clone(&field),
        Arc::new(Float64Array::from(vec![f64::from_bits(
            0xfff8_0000_0000_0017,
        )])),
    )
    .unwrap();
    assert_eq!(literal.as_json(), r#"["f64","fff8000000000017"]"#);
    assert!(
        NativeLiteral::from_json(
            Arc::new(Field::new("n", DataType::Int8, false)),
            r#"["i64",128]"#
        )
        .is_err()
    );
    assert!(NativeLiteral::from_json(field, r#"["f64","FFF8000000000017"]"#).is_err());
}
