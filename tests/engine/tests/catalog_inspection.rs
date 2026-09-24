// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Admitted table streams retain actual array owners across close and partial drain.
#![allow(clippy::unwrap_used, reason = "test fixtures fail directly")]

#[path = "../../support/native_publication.rs"]
mod native_publication;

use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc};

use datafusion::arrow::array::UInt64Array;
use pse_catalog::{delta::publication::Publication, inspection::TableReader};
use pse_columnar::CancellationToken;
use pse_ids::SemanticId;
use pse_schema::{
    RegistryBuilder,
    model::{Authority, FieldContract, Namespace, RelationDecl, SnapshotClass},
};

async fn fixture(
    empty: bool,
) -> (
    Publication,
    Arc<dyn pse_columnar::MemoryPool>,
    tempfile::TempDir,
) {
    let mut builder = RegistryBuilder::new();
    pse_schema::catalog::declare_diagnostics(&mut builder);
    pse_schema::catalog::declare_publications(&mut builder);
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "items",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "inspection fixture",
        )
        .pk(&["id"])
        .columns(vec![
            FieldContract::key(
                "id",
                FieldContract::native(arrow::datatypes::DataType::UInt64),
                "identity",
            ),
            FieldContract::payload(
                "nested",
                FieldContract::structure(vec![FieldContract::id().with_name("semantic")]),
                "nested identity",
            ),
        ]),
    );
    let registry = Arc::new(builder.build().unwrap());
    let spec = registry.relation("authored.items").unwrap();
    let rows = (0..5)
        .map(|value| {
            vec![
                serde_json::json!(["u64", value]),
                serde_json::json!([
                    "struct",
                    vec![serde_json::json!([
                        "id",
                        (SemanticId::from_bytes([7; 16])).to_hex()
                    ])]
                ]),
            ]
        })
        .collect::<Vec<_>>();
    let batch = pse_relations::testing::batch_from_literals(&registry, spec, &rows).unwrap();
    let batch = batch.slice(1, if empty { 0 } else { 3 });
    let key = spec.key;
    let (publication, directory, budget) =
        native_publication::publish(registry, BTreeMap::from([(key, batch)])).await;
    (publication, budget, directory)
}

#[tokio::test]
async fn bounded_streams_preserve_nested_slices_and_last_buffer_accounting() {
    let (publication, budget, _directory) = fixture(false).await;
    let expected_schema = Arc::new(
        pse_schema::arrow::relation_schema(
            publication.session().registry(),
            publication
                .session()
                .registry()
                .relation("authored.items")
                .unwrap(),
        )
        .unwrap(),
    );
    let session = publication.into_session();
    let mut reader = TableReader::new(
        &session,
        &native_publication::name("authored", "items"),
        NonZeroUsize::new(2).unwrap(),
        CancellationToken::new(),
    )
    .await
    .unwrap();
    drop(session);
    let first = reader.next_batch().await.unwrap().unwrap();
    let last = reader.next_batch().await.unwrap().unwrap();
    assert_eq!((first.num_rows(), last.num_rows()), (2, 1));
    assert_eq!(first.schema(), expected_schema);
    assert_eq!(
        last.column(0)
            .as_any()
            .downcast_ref::<UInt64Array>()
            .unwrap()
            .value(0),
        3
    );
    assert!(reader.next_batch().await.unwrap().is_none());
    reader.close();
    drop(reader);
    let retained = first.column(0).slice(1, 1);
    drop(first);
    drop(last);
    assert!(budget.reserved() > 0);
    assert_eq!(
        retained
            .as_any()
            .downcast_ref::<UInt64Array>()
            .unwrap()
            .value(0),
        2
    );
    drop(retained);
    assert_eq!(budget.reserved(), 0);
}

#[tokio::test]
async fn partial_cancel_releases_unread_owners_and_reports_cancellation() {
    let (publication, budget, _directory) = fixture(false).await;
    let cancel = CancellationToken::new();
    let session = publication.into_session();
    let mut reader = TableReader::new(
        &session,
        &native_publication::name("authored", "items"),
        NonZeroUsize::MIN,
        cancel.clone(),
    )
    .await
    .unwrap();
    drop(session);
    let batch = reader.next_batch().await.unwrap().unwrap();
    cancel.cancel();
    assert!(reader.next_batch().await.is_err());
    drop(reader);
    assert!(budget.reserved() > 0);
    drop(batch);
    assert_eq!(budget.reserved(), 0);
}

#[tokio::test]
async fn empty_stream_keeps_its_exact_schema_and_releases_sources_at_eof() {
    let (publication, budget, _directory) = fixture(true).await;
    let schema = Arc::new(
        pse_schema::arrow::relation_schema(
            publication.session().registry(),
            publication
                .session()
                .registry()
                .relation("authored.items")
                .unwrap(),
        )
        .unwrap(),
    );
    let session = publication.into_session();
    let mut reader = TableReader::new(
        &session,
        &native_publication::name("authored", "items"),
        NonZeroUsize::MIN,
        CancellationToken::new(),
    )
    .await
    .unwrap();
    drop(session);
    assert_eq!(reader.schema(), schema);
    assert!(reader.next_batch().await.unwrap().is_none());
    drop(reader);
    assert_eq!(budget.reserved(), 0);
}

#[tokio::test]
async fn wrong_name_and_port_never_select_a_different_admitted_table() {
    let (publication, _, _directory) = fixture(false).await;
    let session = publication.into_session();
    for (schema, table) in [("wrong", "items"), ("authored", "missing"), ("", "items")] {
        assert!(
            TableReader::new(
                &session,
                &native_publication::name(schema, table),
                NonZeroUsize::MIN,
                CancellationToken::new()
            )
            .await
            .is_err()
        );
    }
}
