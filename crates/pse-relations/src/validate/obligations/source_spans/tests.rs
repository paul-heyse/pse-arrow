// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(clippy::unwrap_used, reason = "native validation assertions")]
use super::*;

#[tokio::test]
async fn span_projection_does_not_capture_an_unrelated_value_column() {
    let registry = pse_schema::registry().unwrap();
    let context = SessionContext::new();
    let spec = registry.relation("authored.case_specs").unwrap();
    let schema = pse_schema::arrow::relation_schema(registry, spec).unwrap();
    let input = context
        .read_batch(arrow::record_batch::RecordBatch::new_empty(
            std::sync::Arc::new(schema),
        ))
        .unwrap()
        .into_unoptimized_plan();
    for plan in plans(&input, None).unwrap() {
        context.state().create_physical_plan(&plan).await.unwrap();
    }
}
use crate::generated::authored::documents;
use crate::native::{execution::context::SessionContext, physical_plan::collect};
use pse_ids::SemanticId;
use pse_schema::{Registry, RegistryBuilder, model::*};

fn registry() -> Registry {
    let mut registry = RegistryBuilder::new();
    pse_schema::catalog::declare(&mut registry);
    registry.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "nested_sources",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "Nested source spans for native validation.",
        )
        .pk(&["id"])
        .columns(vec![
            FieldContract::key("id", FieldContract::id(), "identity"),
            FieldContract::payload(
                "nested",
                FieldContract::list(FieldContract::structure(vec![
                    FieldContract::extended(ExtensionUse::SourceSpan)
                        .with_name("span")
                        .with_nullable(true),
                ])),
                "nested source claims",
            )
            .optional(),
        ]),
    );
    registry.build().unwrap()
}
fn nested(document: SemanticId, start: i64, end: i64) -> serde_json::Value {
    serde_json::json!([
        "list",
        vec![serde_json::json!([
            "struct",
            vec![serde_json::json!([
                "struct",
                vec![
                    serde_json::json!(["id", (document).to_hex()]),
                    serde_json::json!(["i64", start]),
                    serde_json::json!(["i64", end]),
                ]
            ])]
        ])]
    ])
}

#[tokio::test]
async fn nested_source_claims_use_selected_utf8_bytes_and_native_joins() {
    let registry = registry();
    let spec = registry.relation("authored.nested_sources").unwrap();
    let document = SemanticId::from_bytes([1; 16]);
    let context = SessionContext::new();
    let mut documents = documents::Builder::new().unwrap();
    documents
        .push(documents::Row {
            document_id: document,
            package_id: document,
            path: "source.yaml".into(),
            source_text: "é".into(),
        })
        .unwrap();
    let documents = context
        .read_batch(documents.finish().unwrap().into_batch())
        .unwrap()
        .into_unoptimized_plan();
    assert!(
        crate::testing::batch_from_literals(
            &registry,
            spec,
            &[vec![
                serde_json::json!(["id", (document).to_hex()]),
                nested(document, 2, 1)
            ]],
        )
        .is_err(),
        "local declared validation already rejects reversed offsets"
    );
    for (value, has_documents, valid) in [
        (nested(document, 0, 2), true, true),
        (nested(document, 0, 3), true, false),
        (nested(SemanticId::NIL, 0, 0), true, false),
        (nested(document, 0, 2), false, false),
        (serde_json::json!(["null", null]), false, true),
        (serde_json::json!(["list", []]), false, true),
        (
            serde_json::json!([
                "list",
                vec![serde_json::json!([
                    "struct",
                    vec![serde_json::json!(["null", null])]
                ])]
            ]),
            false,
            true,
        ),
    ] {
        let batch = crate::testing::batch_from_literals(
            &registry,
            spec,
            &[vec![serde_json::json!(["id", (document).to_hex()]), value]],
        )
        .unwrap();
        let input = context.read_batch(batch).unwrap().into_unoptimized_plan();
        let checks = plans(&input, has_documents.then_some(&documents)).unwrap();
        assert_eq!(checks.len(), 1);
        assert!(checks[0].display_indent().to_string().contains("Unnest"));
        let state = context.state();
        let result = collect(
            state.create_physical_plan(&checks[0]).await.unwrap(),
            state.task_ctx(),
        )
        .await
        .unwrap();
        assert_eq!(
            result
                .iter()
                .map(arrow::array::RecordBatch::num_rows)
                .sum::<usize>()
                == 0,
            valid
        );
    }
}
