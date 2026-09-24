// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed row tokens qualify pinned native encoding across independent conversions.
#![allow(clippy::unwrap_used, reason = "native key qualification")]

use datafusion::{
    arrow::{
        array::{
            Array, ArrayRef, DictionaryArray, FixedSizeBinaryArray, Int8Array, Int64Array,
            RecordBatch, StringArray, StructArray,
        },
        datatypes::{DataType, Field, Int8Type, Schema},
    },
    execution::context::SessionContext,
    logical_expr::{Expr, col},
};
use pse_ids::ContentHash;
use std::sync::Arc;

async fn keys(batch: RecordBatch, names: &[&str]) -> Vec<ContentHash> {
    scoped_keys(batch, names, pse_ids::SemanticId::NIL).await
}

async fn scoped_keys(
    batch: RecordBatch,
    names: &[&str],
    relation: pse_ids::SemanticId,
) -> Vec<ContentHash> {
    let context = SessionContext::new();
    context
        .read_batch(batch)
        .unwrap()
        .select(vec![pse_relations::identity::key(
            relation,
            names.iter().map(|name| (*name, col(*name))).collect(),
        )])
        .unwrap()
        .collect()
        .await
        .unwrap()
        .iter()
        .flat_map(|batch| {
            assert_eq!(
                batch.schema().field(0).metadata()[pse_schema::arrow::KEY_ROW_KEY_ENCODING],
                pse_schema::model::ROW_KEY_ENCODING
            );
            batch
                .column(0)
                .as_any()
                .downcast_ref::<FixedSizeBinaryArray>()
                .unwrap()
                .iter()
                .map(|value| ContentHash::try_from_slice(value.unwrap()).unwrap())
                .collect::<Vec<_>>()
        })
        .collect()
}

#[tokio::test]
async fn dictionary_layout_is_not_key_identity_but_relation_scope_is() {
    let dictionary = |keys: Vec<i8>, values: Vec<&str>| {
        key_batch(vec![(
            "id",
            Arc::new(
                DictionaryArray::<Int8Type>::try_new(
                    Int8Array::from(keys),
                    Arc::new(StringArray::from(values)),
                )
                .unwrap(),
            ),
        )])
        .unwrap()
    };
    let first = dictionary(vec![0, 1, 0], vec!["alpha", "beta"]);
    let remapped = dictionary(vec![1, 0, 1], vec!["beta", "alpha", "unused"]);
    assert_eq!(
        keys(first.clone(), &["id"]).await,
        keys(remapped, &["id"]).await
    );
    assert_ne!(
        keys(first.clone(), &["id"]).await,
        scoped_keys(first, &["id"], pse_ids::SemanticId::from_bytes([1; 16])).await
    );
}

#[tokio::test]
async fn keys_survive_payload_edits_chunking_slicing_and_independent_converters() {
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("payload", DataType::Utf8, false),
    ]));
    let batch = |ids: Vec<i64>, payload: Vec<&str>| {
        RecordBatch::try_new(
            Arc::clone(&schema),
            vec![
                Arc::new(Int64Array::from(ids)),
                Arc::new(StringArray::from(payload)),
            ],
        )
        .unwrap()
    };
    let all = batch(vec![10, 20, 30], vec!["a", "b", "c"]);
    let baseline = keys(all.clone(), &["id"]).await;
    let sliced = keys(all.slice(1, 1), &["id"]).await;
    let changed = keys(
        batch(vec![30, 10, 20], vec!["changed", "", "other"]),
        &["id"],
    )
    .await;
    assert_eq!(sliced, [baseline[1]]);
    assert_eq!(changed, [baseline[2], baseline[0], baseline[1]]);
    assert_ne!(baseline[0], baseline[1]);
    assert_eq!(
        keys(all, &[]).await,
        vec![keys(batch(vec![9], vec!["x"]), &[]).await[0]; 3]
    );
}

#[tokio::test]
async fn key_contract_distinguishes_names_types_and_composite_boundaries() {
    let batch = key_batch(vec![
        ("a", Arc::new(StringArray::from(vec!["ab", "a"]))),
        ("b", Arc::new(StringArray::from(vec!["c", "bc"]))),
    ])
    .unwrap();
    let ordered = keys(batch.clone(), &["a", "b"]).await;
    assert_ne!(ordered[0], ordered[1]);
    assert_ne!(ordered, keys(batch, &["b", "a"]).await);
    let numeric = key_batch(vec![("a", Arc::new(Int64Array::from(vec![1])))]).unwrap();
    let text = key_batch(vec![("a", Arc::new(StringArray::from(vec!["1"])))]).unwrap();
    assert_ne!(keys(numeric, &["a"]).await, keys(text, &["a"]).await);
}

#[tokio::test]
async fn native_row_tokens_mask_null_parent_payloads() {
    let children = vec![Arc::new(Field::new("child", DataType::Int64, false))].into();
    let structure = StructArray::new(
        children,
        vec![Arc::new(Int64Array::from(vec![1, 2, 1]))],
        Some(vec![false, false, true].into()),
    );
    let batch = key_batch(vec![("key", Arc::new(structure))]).unwrap();
    let tokens = keys(batch, &["key"]).await;
    assert_eq!(tokens[0], tokens[1]);
    assert_ne!(tokens[0], tokens[2]);
}

#[test]
fn native_key_expression_has_no_text_encoding_path() {
    let expression =
        pse_relations::identity::key(pse_ids::SemanticId::NIL, vec![("id", col("id"))]);
    let Expr::ScalarFunction(call) = expression else {
        panic!("native scalar function")
    };
    assert_eq!(call.func.name(), "pse_row_key");
}

#[test]
fn native_output_cannot_relabel_an_ordinary_hash_or_another_key_encoding() {
    let registry = pse_engine::validation::registry().unwrap();
    let expected = pse_schema::arrow::field_for(
        registry,
        &pse_schema::model::FieldContract::row_key().with_name("key"),
    )
    .unwrap();
    let ordinary = pse_schema::arrow::field_for(
        registry,
        &pse_schema::model::FieldContract::hash().with_name("hash"),
    )
    .unwrap();
    let mut another = expected.clone();
    another.metadata_mut().insert(
        pse_schema::arrow::KEY_ROW_KEY_ENCODING.into(),
        "unqualified-encoding".into(),
    );
    for actual in [ordinary, another] {
        assert!(pse_engine::session::output::check_field_output(&actual, &expected).is_err());
        let actual = Field::new("nested", DataType::List(Arc::new(actual)), false);
        let declared = Field::new("nested", DataType::List(Arc::new(expected.clone())), false);
        assert!(pse_engine::session::output::check_field_output(&actual, &declared).is_err());
    }
}

#[tokio::test]
async fn pinned_row_encoding_has_a_frozen_contract_vector() {
    let batch = key_batch(vec![("id", Arc::new(Int64Array::from(vec![42])))]).unwrap();
    let actual = keys(batch, &["id"]).await;
    // Independently framed from the v2 wire contract in pse-ids::row_token units.
    assert_eq!(
        actual[0].to_string(),
        "811bfe61cc46de019bff708ee5f5b8534a3b4705c2effbb8a4f99352adb58ab5"
    );
}

fn key_batch(
    columns: Vec<(&str, ArrayRef)>,
) -> Result<RecordBatch, datafusion::arrow::error::ArrowError> {
    RecordBatch::try_from_iter(columns)
}
