// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(clippy::unwrap_used, reason = "native validation assertions")]
use super::*;
use datafusion::{execution::context::SessionContext, physical_plan::collect};
use pse_ids::SemanticId;
use pse_relations::generated::authored::documents;
use pse_schema::{Registry, RegistryBuilder, model::*};

fn registry() -> Registry {
    let mut registry = RegistryBuilder::new();
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
fn nested(document: SemanticId, start: i64, end: i64) -> Cell {
    Cell::List(vec![Cell::Struct(vec![Cell::Struct(vec![
        Cell::Id(document),
        Cell::I64(start),
        Cell::I64(end),
    ])])])
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
        pse_relations::cells::batch_from_cells(
            &registry,
            spec,
            &[vec![Cell::Id(document), nested(document, 2, 1)]],
        )
        .is_err(),
        "local declared validation already rejects reversed offsets"
    );
    for (value, has_documents, valid) in [
        (nested(document, 0, 2), true, true),
        (nested(document, 0, 3), true, false),
        (nested(SemanticId::NIL, 0, 0), true, false),
        (nested(document, 0, 2), false, false),
        (Cell::Null, false, true),
        (Cell::List(vec![]), false, true),
        (
            Cell::List(vec![Cell::Struct(vec![Cell::Null])]),
            false,
            true,
        ),
    ] {
        let batch = pse_relations::cells::batch_from_cells(
            &registry,
            spec,
            &[vec![Cell::Id(document), value]],
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
                .map(datafusion::arrow::array::RecordBatch::num_rows)
                .sum::<usize>()
                == 0,
            valid
        );
    }
}
