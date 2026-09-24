// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;
use datafusion::arrow::array::{
    Array, ArrayRef, FixedSizeBinaryArray, StringArray, StructArray, new_null_array,
};

#[test]
fn existing_enum_meaning_cannot_be_relabelled_by_value_admission() {
    let typed = |identity: &str| {
        Field::new("value", DataType::Utf8, false).with_metadata(std::collections::HashMap::from([
            (pse_schema::arrow::KEY_ENUM.into(), identity.into()),
        ]))
    };
    assert!(compatible(&typed("one"), &typed("two")).is_err());
    assert!(compatible(&typed("one"), &typed("one")).is_ok());
    assert!(compatible(&Field::new("literal", DataType::Utf8, false), &typed("two")).is_ok());
}

fn invoke(array: ArrayRef) -> Result<ArrayRef> {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let function = function(
        registry,
        datafusion::execution::context::SessionContext::new().state(),
    );
    let relation = ScalarValue::Utf8(Some("inferred.method_resolutions".into()));
    let column = ScalarValue::Utf8(Some("outcome".into()));
    let fields = vec![
        Arc::new(Field::new("value", array.data_type().clone(), true)),
        Arc::new(Field::new("relation", DataType::Utf8, false)),
        Arc::new(Field::new("column", DataType::Utf8, false)),
    ];
    let output = function.return_field_from_args(ReturnFieldArgs {
        arg_fields: &fields,
        scalar_arguments: &[None, Some(&relation), Some(&column)],
    })?;
    let rows = array.len();
    function
        .invoke_with_args(ScalarFunctionArgs {
            args: vec![
                ColumnarValue::Array(array),
                ColumnarValue::Scalar(relation),
                ColumnarValue::Scalar(column),
            ],
            arg_fields: fields,
            number_rows: rows,
            return_field: output,
            config_options: Arc::default(),
        })?
        .into_array(rows)
}
fn outcome(kind: &str, present: bool) -> ArrayRef {
    let member: ArrayRef = if present {
        let ids: ArrayRef = Arc::new(
            FixedSizeBinaryArray::try_from_iter([[7_u8; 16].as_slice()].into_iter()).unwrap(),
        );
        Arc::new(
            StructArray::try_new(
                vec![Arc::new(Field::new(
                    "method_id",
                    ids.data_type().clone(),
                    false,
                ))]
                .into(),
                vec![ids],
                None,
            )
            .unwrap(),
        )
    } else {
        new_null_array(&DataType::Null, 1)
    };
    Arc::new(
        StructArray::try_new(
            vec![
                Arc::new(Field::new("kind", DataType::Utf8, false)),
                Arc::new(Field::new("resolved", member.data_type().clone(), true)),
            ]
            .into(),
            vec![Arc::new(StringArray::from(vec![kind])), member],
            None,
        )
        .unwrap(),
    )
}
#[test]
fn checks_selected_payload_and_unit_outcomes_in_native_arrays() {
    for (kind, present) in [
        ("resolved", true),
        ("unresolved", false),
        ("ambiguous", false),
    ] {
        assert_eq!(invoke(outcome(kind, present)).unwrap().len(), 1);
    }
    for (kind, present) in [
        ("resolved", false),
        ("unresolved", true),
        ("ambiguous", true),
        ("unregistered", false),
    ] {
        assert!(
            invoke(outcome(kind, present)).is_err(),
            "{kind}/{present} must fail actual value admission"
        );
    }
}
#[test]
fn cannot_invent_quantity_meaning_or_silently_convert_storage() {
    let plain = Field::new("value", DataType::Float64, false);
    let quantity = plain
        .clone()
        .with_metadata(std::collections::HashMap::from([(
            pse_schema::arrow::KEY_QUANTITY_TYPE.into(),
            pse_ids::SemanticId::NIL.to_hex(),
        )]));
    assert!(compatible(&plain, &quantity).is_err());
    assert!(compatible(&quantity, &plain).is_err());
    assert!(compatible(&plain, &Field::new("value", DataType::Int64, false)).is_err());
    assert!(compatible(&plain, &plain).is_ok());
}
