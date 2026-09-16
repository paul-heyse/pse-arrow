// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Admitted table streams retain actual array owners across close and partial drain.
#![allow(clippy::unwrap_used, reason = "test fixtures fail directly")]

#[path = "../../support/native_catalog.rs"]
mod native_catalog;

use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc};

use datafusion::arrow::array::UInt64Array;
use object_store::memory::InMemory;
use pse_catalog::{
    Catalog, EncodingPolicy, FixedClock, RelationContract, Snapshot, TrustLevel,
    inspection::TableReader,
    store::{
        membership::AdmissionContext,
        publish::{BundleDraft, RelationDraft},
    },
};
use pse_ids::{CancellationToken, FixedBudget, MemoryReserver, SemanticId, SnapshotKind};
use pse_schema::{
    RegistryBuilder,
    model::{Authority, Cell, FieldContract, Namespace, RelationDecl, SnapshotClass},
};

async fn fixture(
    empty: bool,
) -> (
    Arc<Snapshot>,
    Arc<FixedBudget>,
    Arc<pse_catalog::session::SessionFactory>,
) {
    let mut builder = RegistryBuilder::new();
    pse_schema::catalog::declare_diagnostics(&mut builder);
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
    let budget = FixedBudget::new(32 << 20);
    let reserver: Arc<dyn MemoryReserver> = budget.clone();
    let catalog = Catalog::open(
        Arc::new(InMemory::new()),
        Arc::clone(&registry),
        TrustLevel::Untrusted,
        Arc::new(FixedClock("2026-09-14T00:00:00Z".to_owned())),
        native_catalog::from_reserver(reserver),
    );
    let sessions = native_catalog::factory(&catalog);
    let catalog = catalog.with_semantic_validator(Arc::new(
        pse_rules::validator::InvariantValidator::new(Arc::clone(&registry)),
    ));
    let spec = registry.relation("authored.items").unwrap();
    let rows = (0..5)
        .map(|value| {
            vec![
                Cell::U64(value),
                Cell::Struct(vec![Cell::Id(SemanticId::from_bytes([7; 16]))]),
            ]
        })
        .collect::<Vec<_>>();
    let batch = pse_relations::cells::batch_from_cells(&registry, spec, &rows).unwrap();
    let batch = batch.slice(1, if empty { 0 } else { 3 });
    let context = AdmissionContext::default();
    let draft = BundleDraft {
        manifest: catalog
            .manifest_template(SnapshotKind::Model, &context)
            .unwrap(),
        relations: BTreeMap::from([(
            pse_ids::model_port_name("authored", spec.id),
            RelationDraft {
                contract: Arc::new(
                    RelationContract::from_spec(&registry, spec, EncodingPolicy::IpcFile).unwrap(),
                ),
                batches: vec![batch],
            },
        )]),
        context,
    };
    let snapshot = catalog
        .publish_bundle(draft, &CancellationToken::new())
        .await
        .unwrap();
    (snapshot, budget, sessions)
}

#[tokio::test]
async fn bounded_streams_preserve_nested_slices_and_last_buffer_accounting() {
    let (snapshot, budget, sessions) = fixture(false).await;
    let expected_schema = snapshot
        .relation("authored", "items")
        .unwrap()
        .batch()
        .schema();
    let session = sessions
        .inspect_snapshot(&snapshot, &CancellationToken::new())
        .unwrap();
    let mut reader = TableReader::new(
        &session,
        "authored.items",
        None,
        NonZeroUsize::new(2).unwrap(),
        CancellationToken::new(),
    )
    .await
    .unwrap();
    drop(session);
    drop(snapshot);
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
    let (snapshot, budget, sessions) = fixture(false).await;
    let cancel = CancellationToken::new();
    let session = sessions
        .inspect_snapshot(&snapshot, &CancellationToken::new())
        .unwrap();
    let mut reader = TableReader::new(
        &session,
        "authored.items",
        None,
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
    let (snapshot, budget, sessions) = fixture(true).await;
    let schema = snapshot
        .relation("authored", "items")
        .unwrap()
        .batch()
        .schema();
    let session = sessions
        .inspect_snapshot(&snapshot, &CancellationToken::new())
        .unwrap();
    let mut reader = TableReader::new(
        &session,
        "authored.items",
        None,
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
    let (snapshot, _, sessions) = fixture(false).await;
    let session = sessions
        .inspect_snapshot(&snapshot, &CancellationToken::new())
        .unwrap();
    for (name, port) in [
        ("items", None),
        ("authored.missing", None),
        ("authored.items", Some("wrong")),
    ] {
        assert!(
            TableReader::new(
                &session,
                name,
                port,
                NonZeroUsize::MIN,
                CancellationToken::new()
            )
            .await
            .is_err()
        );
    }
}
