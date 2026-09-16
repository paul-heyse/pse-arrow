// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Durable operation ordinals address the encoded batch, independent of key order.
#[path = "../../../tests/support/session_factory.rs"]
pub(crate) mod session_factory;

use pse_catalog::{
    Catalog, EncodingPolicy, FixedClock, RelationContract, TrustLevel,
    store::publish::RelationDraft,
};
use pse_ids::{CancellationToken, FixedBudget, SemanticId};
use pse_relations::generated::{authored::templates, enums::TemplateKind};
use std::sync::Arc;

#[tokio::test]
async fn staged_batch_preserves_noncanonical_operation_ordinals_on_reopen() {
    let registry = Arc::new(pse_schema::catalog::assemble().unwrap());
    let catalog = Catalog::open(
        Arc::new(object_store::memory::InMemory::new()),
        registry.clone(),
        TrustLevel::Untrusted,
        Arc::new(FixedClock("2026-09-15T00:00:00Z".into())),
        session_factory::factory(FixedBudget::new(2usize << 30)),
    );
    let mut builder = templates::Builder::with_registry(&registry, 2).unwrap();
    for value in [2, 1] {
        builder
            .push(templates::Row {
                template_id: SemanticId::from_bytes([value; 16]),
                package_id: SemanticId::from_bytes([9; 16]),
                name: format!("template_{value}"),
                version: "1.0.0".into(),
                kind: TemplateKind::Unit,
                doc: String::new(),
                default_initializer_template_id: None,
                default_scaler_template_id: None,
                idaes_class: None,
            })
            .unwrap();
    }
    let cancel = CancellationToken::new();
    let staged = catalog
        .publish_staged_batch(
            RelationDraft {
                contract: Arc::new(
                    RelationContract::from_spec(
                        &registry,
                        templates::spec(&registry).unwrap(),
                        EncodingPolicy::IpcFile,
                    )
                    .unwrap(),
                ),
                batches: vec![builder.finish().unwrap().into_batch()],
            },
            &cancel,
        )
        .await
        .unwrap();
    let reopened = catalog
        .read_staged_batch(staged.reference(), &cancel)
        .await
        .unwrap();
    for artifact in [&staged, &reopened] {
        let rows = templates::View::from_checked(artifact.relation().checked())
            .unwrap()
            .rows()
            .unwrap();
        assert_eq!(
            rows.iter().map(|row| row.template_id).collect::<Vec<_>>(),
            [2, 1].map(|value| SemanticId::from_bytes([value; 16]))
        );
    }
}
