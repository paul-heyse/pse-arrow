// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native Arrow containers preserve PSE child predicates and visible-value masks.

use datafusion::{
    arrow::{
        array::{
            Array, ArrayRef, Decimal128Array, DictionaryArray, FixedSizeBinaryArray, Int64Array,
            LargeListArray, LargeListViewArray, RecordBatch, StructArray, UInt8Array,
        },
        buffer::{NullBuffer, OffsetBuffer},
        datatypes::{DataType, Field, Schema, UInt8Type},
    },
    common::TableReference,
    datasource::MemTable,
};
use pse_columnar::CancellationToken;
use pse_engine::session::EngineSession;
use std::{collections::BTreeMap, sync::Arc};

#[expect(clippy::unwrap_used, reason = "fixed declared span storage fixture")]
#[expect(
    clippy::panic,
    reason = "test assertion checks the declared extension layout"
)]
fn fixture() -> (EngineSession, Arc<Field>, ArrayRef) {
    use pse_schema::model::{ExtensionUse, FieldContract};
    let registry = Arc::new(pse_schema::RegistryBuilder::new().build().unwrap());
    let field = Arc::new(
        pse_schema::arrow::field_for(
            &registry,
            &FieldContract::payload(
                "span",
                FieldContract::extended(ExtensionUse::SourceSpan),
                "captured source range",
            ),
        )
        .unwrap(),
    );
    let DataType::Struct(children) = field.data_type() else {
        panic!("declared span is a struct")
    };
    let values: ArrayRef = Arc::new(
        StructArray::try_new(
            children.clone(),
            vec![
                Arc::new(
                    FixedSizeBinaryArray::try_from_iter([[1_u8; 16], [1_u8; 16]].into_iter())
                        .unwrap(),
                ),
                Arc::new(Int64Array::from(vec![0, 7])),
                Arc::new(Int64Array::from(vec![3, 2])),
            ],
            None,
        )
        .unwrap(),
    );
    let session = pse_testkit::NativeFixture::new((64 << 20).try_into().unwrap())
        .unwrap()
        .into_factory()
        .candidate(BTreeMap::new(), registry, &CancellationToken::new())
        .unwrap();
    (session, field, values)
}

async fn capture(
    session: &EngineSession,
    field: Field,
    array: ArrayRef,
) -> Result<(), pse_engine::EngineError> {
    let schema = Arc::new(Schema::new(vec![field]));
    let batch = RecordBatch::try_new(Arc::clone(&schema), vec![array]).map_err(|error| {
        pse_engine::EngineError::Infrastructure {
            op: "capture fixture".into(),
            source: Box::new(error),
        }
    })?;
    let provider = MemTable::try_new(schema, vec![vec![batch]]).map_err(|error| {
        pse_engine::EngineError::Infrastructure {
            op: "capture fixture provider".into(),
            source: Box::new(error),
        }
    })?;
    let cancel = CancellationToken::new();
    session
        .prepare_provider_capture(
            TableReference::full("external", "values", "nested"),
            Arc::new(provider),
            &cancel,
        )?
        .execute(&cancel)
        .await?;
    Ok(())
}

#[tokio::test]
async fn large_list_validates_visible_pse_children_and_ignores_null_parents() {
    let (session, child, values) = fixture();
    let visible: ArrayRef = Arc::new(
        LargeListArray::try_new(
            Arc::clone(&child),
            OffsetBuffer::new(vec![0_i64, 2].into()),
            Arc::clone(&values),
            None,
        )
        .unwrap(),
    );
    let field = Field::new("spans", visible.data_type().clone(), true);
    let error = capture(&session, field.clone(), visible).await.unwrap_err();
    assert_eq!(
        pse_diagnostics::TypedDiagnostic::diagnostic_code(&error),
        Some(pse_diagnostics::DiagnosticCode::ValidationInvariant),
    );
    let masked: ArrayRef = Arc::new(
        LargeListArray::try_new(
            child,
            OffsetBuffer::new(vec![0_i64, 2].into()),
            values,
            Some(NullBuffer::from(vec![false])),
        )
        .unwrap(),
    );
    capture(&session, field, masked).await.unwrap();
}

#[tokio::test]
async fn mixed_decimal_struct_and_large_list_view_keep_the_native_type_universe() {
    let (session, child, values) = fixture();
    let view: ArrayRef = Arc::new(
        LargeListViewArray::try_new(child, vec![0_i64].into(), vec![1_i64].into(), values, None)
            .unwrap(),
    );
    let decimal: ArrayRef = Arc::new(
        Decimal128Array::from(vec![123])
            .with_precision_and_scale(10, 2)
            .unwrap(),
    );
    let fields = vec![
        Arc::new(Field::new("amount", decimal.data_type().clone(), false)),
        Arc::new(Field::new("spans", view.data_type().clone(), false)),
    ]
    .into();
    let record: ArrayRef =
        Arc::new(StructArray::try_new(fields, vec![decimal, view], None).unwrap());
    capture(
        &session,
        Field::new("record", record.data_type().clone(), false),
        record,
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn dictionary_checks_only_referenced_visible_nested_values() {
    let (session, span, values) = fixture();
    let records: ArrayRef =
        Arc::new(StructArray::try_new(vec![span].into(), vec![values], None).unwrap());
    let dictionary: ArrayRef = Arc::new(
        DictionaryArray::<UInt8Type>::try_new(
            UInt8Array::from(vec![Some(0), None]),
            Arc::clone(&records),
        )
        .unwrap(),
    );
    capture(
        &session,
        Field::new("record", dictionary.data_type().clone(), true),
        dictionary,
    )
    .await
    .unwrap();
    let dictionary: ArrayRef = Arc::new(
        DictionaryArray::<UInt8Type>::try_new(UInt8Array::from(vec![1]), records).unwrap(),
    );
    assert!(
        capture(
            &session,
            Field::new("record", dictionary.data_type().clone(), false),
            dictionary
        )
        .await
        .is_err()
    );
}
