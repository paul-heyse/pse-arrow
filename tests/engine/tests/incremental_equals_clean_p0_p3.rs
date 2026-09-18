// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::unwrap_used,
    reason = "test fixture construction and exact independent value assertions"
)]
//! Native document edits and a fresh source produce identical typed normalization.
#[path = "../../support/native_pipeline.rs"]
mod native_pipeline;

use pse_authoring::{
    ParseBudget,
    document::{DocumentBundle, load_package_texts},
};
use pse_ids::CancellationToken;
use pse_relations::generated::authored;
use pse_schema::Registry;
use std::collections::BTreeMap;
const ADDED_SPECIES: &str = "  - id: '01991d6a-13a0-7000-8000-000000000003'\n    name: oxygen\n    formula: O2\n    mw: 0.031998\n    component_type: Component\n    charge: 0\n    doc: Molecular oxygen.\n";

fn source(registry: &Registry, changed: bool) -> DocumentBundle {
    let mut species =
        include_str!("../../fixtures/packages/minimal_explicit/materials/species.yaml").to_owned();
    if changed {
        species.push_str(ADDED_SPECIES);
    }
    // This is a complete model package. Cases are absent so the only source edit is
    // the added species; a case revision binding would introduce a second mutation.
    load_package_texts(
        BTreeMap::from([
            (
                "package.toml".to_owned(),
                include_str!("../../fixtures/packages/minimal_explicit/package.toml").to_owned(),
            ),
            ("materials/species.yaml".to_owned(), species),
            (
                "templates/model.yaml".to_owned(),
                include_str!("../../fixtures/packages/minimal_explicit/templates/model.yaml")
                    .to_owned(),
            ),
        ]),
        registry,
        ParseBudget::default(),
    )
    .unwrap()
}

#[tokio::test]
async fn changed_species_document_plan_matches_clean_normalization_and_refuses_stale_edit() {
    let fixture = native_pipeline::Fixture::new();
    let original = source(&fixture.registry, false);
    let changed = source(&fixture.registry, true);
    let before = original
        .documents
        .iter()
        .find(|d| d.path == "materials/species.yaml")
        .unwrap();
    let after = changed
        .documents
        .iter()
        .find(|d| d.path == before.path)
        .unwrap();
    let mut edits = authored::document_edits::Builder::new().unwrap();
    edits
        .push(authored::document_edits::Row {
            document_id: before.id,
            path: before.path.clone(),
            before: before.text.clone(),
            after: after.text.clone(),
        })
        .unwrap();
    let request = edits.finish().unwrap();
    let cancel = CancellationToken::new();
    let (session, documents) = fixture.source(vec![original]);
    let session = session
        .with_checked_workspace(
            BTreeMap::from([(authored::document_edits::RELATION_KEY, request)]),
            &cancel,
        )
        .unwrap();
    let name = session
        .table_reference(&authored::document_edits::RELATION_KEY)
        .unwrap();
    let request = session
        .relation_plan(&datafusion::common::ResolvedTableReference {
            catalog: name.catalog().unwrap().into(),
            schema: name.schema().unwrap().into(),
            table: name.table().into(),
        })
        .unwrap();
    let edited = pse_compiler::native::edit::documents_plan(
        &session,
        pse_compiler::native::Value::from_relation(documents.clone()),
        pse_compiler::native::Value::from_relation(request.clone()),
        &cancel,
    )
    .await
    .unwrap();
    let edited = edited.relation().clone();
    let before = fixture
        .evaluate((session.clone(), documents), "P3")
        .await
        .unwrap();
    let actual = fixture
        .evaluate((session.clone(), edited.clone()), "P3")
        .await
        .unwrap();
    let expected = fixture
        .evaluate(fixture.source(vec![changed]), "P3")
        .await
        .unwrap();
    // Compare complete relation multisets: native joins may choose a different row order.
    for (key, actual) in &actual {
        let spec = fixture.registry.relation_by_key(*key).unwrap();
        let rows = |batch: &pse_relations::columnar::FieldCheckedBatch| {
            let mut rows =
                pse_relations::cells::cells_from_batch(&fixture.registry, spec, batch.batch())
                    .unwrap()
                    .into_iter()
                    .map(|row| pse_schema::model::Cell::Struct(row).literal_spec())
                    .collect::<Vec<_>>();
            rows.sort();
            rows
        };
        assert_eq!(rows(actual), rows(&expected[key]), "{key}");
    }
    assert_eq!(
        before[&authored::species::RELATION_KEY].batch().num_rows(),
        1
    );
    assert_eq!(
        actual[&authored::species::RELATION_KEY].batch().num_rows(),
        2
    );
    let stale = pse_compiler::native::edit::documents_plan(
        &session,
        pse_compiler::native::Value::from_relation(edited),
        pse_compiler::native::Value::from_relation(request),
        &cancel,
    )
    .await
    .unwrap();
    assert!(
        session
            .prepare_rule_plan(stale.relation().plan().clone(), &cancel)
            .unwrap()
            .execute(&cancel)
            .await
            .is_err()
    );
}
