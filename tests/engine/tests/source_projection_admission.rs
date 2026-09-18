// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Source projection reads its actual document child, independent of other bindings.
#[path = "../../support/native_pipeline.rs"]
mod native_pipeline;
#[path = "../../support/physical_source.rs"]
mod physical_source;
use pse_ids::CancellationToken;
use pse_relations::{columnar::FieldCheckedBatch, generated::authored};
use std::collections::BTreeMap;
#[tokio::test]
async fn unrelated_package_values_cannot_replace_the_actual_source_child() {
    let fixture = native_pipeline::Fixture::new();
    let source = pse_authoring::document::load_package_texts(
        physical_source::physical_texts(&fixture.registry),
        &fixture.registry,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap();
    let package = source.batches[&authored::packages::RELATION_ID].clone();
    let expected = authored::packages::View::from_checked(&package)
        .unwrap()
        .rows()
        .unwrap();
    let mut rows = authored::packages::Builder::new().unwrap();
    for mut row in expected.clone() {
        row.name = "not_the_document".into();
        rows.push(row).unwrap();
    }
    let forged: FieldCheckedBatch = rows.finish().unwrap();
    let (session, documents) = fixture.source(vec![source]);
    let cancel = CancellationToken::new();
    let session = session
        .with_checked_workspace(
            BTreeMap::from([(authored::packages::RELATION_KEY, forged)]),
            &cancel,
        )
        .unwrap();
    let plan = pse_compiler::native::model::source(&session, documents, &cancel)
        .await
        .unwrap();
    let values = fixture.capture(&plan).await.unwrap();
    let actual = authored::packages::View::from_checked(&values[&authored::packages::RELATION_KEY])
        .unwrap()
        .rows()
        .unwrap();
    assert_eq!(actual, expected);
}
