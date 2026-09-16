// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual committed sources remain the authority for model publication and reopening.
#[path = "../../support/native_pipeline.rs"]
mod native_pipeline;
#[path = "../../support/physical_source.rs"]
mod physical_source;

use datafusion::arrow::array::{RecordBatch, StringArray};
use pse_catalog::store::{
    membership::AdmissionContext,
    publish::{BundleDraft, RelationDraft},
};
use pse_ids::{CancellationToken, SnapshotKind};
use std::sync::Arc;

#[tokio::test]
async fn changed_package_values_cannot_replace_the_actual_committed_source() {
    let mut fixture = native_pipeline::Fixture::new();
    let source = pse_authoring::document::load_package_texts(
        physical_source::physical_texts(&fixture.registry),
        &fixture.registry,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap();
    let tip = fixture.commit(vec![source]).await;
    let committed = Arc::clone(&tip.parents()["model"]);
    let context = AdmissionContext {
        traversal: Arc::default(),
        invocation: None,
        parents: committed.parents().clone(),
        stage_pass: committed.stage_pass(),
    };
    let cancel = CancellationToken::new();
    let reopened = fixture
        .catalog
        .read_manifest(committed.manifest_ref(), &context, &cancel)
        .await
        .unwrap();
    assert_eq!(reopened.snapshot_id(), committed.snapshot_id());
    let mut changed = false;
    let relations = committed
        .relations()
        .values()
        .map(|relation| {
            let mut batch = relation.batch().clone();
            if relation.contract().namespace == "authored" && relation.contract().name == "packages"
            {
                let index = batch.schema().index_of("name").unwrap();
                let mut columns = batch.columns().to_vec();
                columns[index] = Arc::new(StringArray::from(vec![
                    "changed_without_source_edit";
                    batch.num_rows()
                ]));
                batch = RecordBatch::try_new(batch.schema(), columns).unwrap();
                changed = true;
            }
            (
                relation.member().port.clone(),
                RelationDraft {
                    contract: Arc::clone(relation.contract()),
                    batches: vec![batch],
                },
            )
        })
        .collect();
    assert!(changed);
    let manifest = fixture
        .catalog
        .manifest_template(SnapshotKind::Model, &context)
        .unwrap();
    let error = fixture
        .catalog
        .publish_bundle(
            BundleDraft {
                manifest,
                context,
                relations,
            },
            &cancel,
        )
        .await
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("differ from exact source projection"),
        "{error}"
    );
}
