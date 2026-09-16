// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Current ChangeSet receipts reopen complete staged batches and validate ordinals.
#[path = "../../support/native_pipeline.rs"]
mod native_pipeline;
#[path = "../../support/physical_source.rs"]
mod physical_source;

use pse_catalog::{
    EncodingPolicy, RefName, RelationContract,
    store::{publish::RelationDraft, sidecar::SidecarArtifact},
};
use pse_ids::CancellationToken;
use pse_relations::generated::authored::change_ops;
use std::sync::Arc;

async fn operations(
    fixture: &native_pipeline::Fixture,
    rows: &[change_ops::Row],
) -> SidecarArtifact {
    let mut builder = change_ops::Builder::with_registry(&fixture.registry, rows.len()).unwrap();
    for row in rows {
        builder.push(row.clone()).unwrap();
    }
    fixture
        .catalog
        .publish_sidecar(
            RelationDraft {
                contract: Arc::new(
                    RelationContract::from_spec(
                        &fixture.registry,
                        change_ops::spec(&fixture.registry).unwrap(),
                        EncodingPolicy::IpcFile,
                    )
                    .unwrap(),
                ),
                batches: vec![builder.finish().unwrap().into_batch()],
            },
            &CancellationToken::new(),
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn actual_commit_receipt_reopens_nonzero_ordinals_and_rejects_invalid_batch_coverage() {
    let mut fixture = native_pipeline::Fixture::new();
    let source = pse_authoring::document::load_package_texts(
        physical_source::physical_texts(&fixture.registry),
        &fixture.registry,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap();
    let output = fixture.commit(vec![source]).await;
    let cancel = CancellationToken::new();
    let state = fixture
        .catalog
        .read_ref(&RefName::parse("semantic_fixture").unwrap(), &cancel)
        .await
        .unwrap()
        .unwrap();
    let revision = state.revision_ref().unwrap();
    let receipt = fixture
        .catalog
        .read_change_set(revision.change_set.as_ref().unwrap(), &cancel)
        .await
        .unwrap();
    let (port, batch) = receipt
        .artifacts()
        .staged
        .iter()
        .find(|(_, artifact)| artifact.relation().rows() > 1)
        .unwrap();
    let ordinal = u64::try_from(batch.relation().rows() - 1).unwrap();
    let mut rows =
        change_ops::View::from_checked(receipt.artifacts().operations.relation().checked())
            .unwrap()
            .rows()
            .unwrap();
    let selected = rows
        .iter()
        .position(|row| row.row_key.staged_port == *port && row.row_key.staged_ordinal == ordinal)
        .unwrap();
    let artifact = fixture
        .catalog
        .read_sidecar(&revision.artifact, &cancel)
        .await
        .unwrap();
    let output = fixture
        .catalog
        .revision_receipt(&artifact, revision.revision_id, &output)
        .unwrap();

    let mut invalid = receipt.artifacts().clone();
    rows[selected].row_key.staged_ordinal = ordinal + 1;
    invalid.operations = operations(&fixture, &rows).await;
    let error = fixture
        .catalog
        .publish_change_set(invalid, None, &output, &cancel)
        .await
        .unwrap_err();
    assert!(error.to_string().contains("staged ordinal"), "{error}");

    let mut incomplete = receipt.artifacts().clone();
    let mut rows = change_ops::View::from_checked(incomplete.operations.relation().checked())
        .unwrap()
        .rows()
        .unwrap();
    rows.retain(|row| {
        !(row.row_key.staged_port == *port && row.row_key.staged_ordinal == ordinal)
            && !row
                .row
                .as_ref()
                .is_some_and(|row| row.staged_port == *port && row.staged_ordinal == ordinal)
    });
    for (ordinal, row) in rows.iter_mut().enumerate() {
        row.ordinal = u32::try_from(ordinal).unwrap();
    }
    incomplete.operations = operations(&fixture, &rows).await;
    let error = fixture
        .catalog
        .publish_change_set(incomplete, None, &output, &cancel)
        .await
        .unwrap_err();
    assert!(error.to_string().contains("unreferenced"), "{error}");
}
