// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Package parsing exercises generated contracts, actual identities and original spans.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "fixture assertions"
)]

use pse_authoring::ParseBudget;
use pse_relations::generated::{authored, reference};
use pse_runtime::authoring_driver::document::{load_package, load_package_documents};
use std::collections::BTreeMap;
#[path = "authoring_support/mod.rs"]
mod support;

fn fixture(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/packages")
        .join(name)
}
fn texts(name: &str) -> BTreeMap<String, Vec<u8>> {
    let loaded = load_package(
        &fixture(name),
        pse_engine::validation::registry().unwrap(),
        ParseBudget::default(),
    )
    .unwrap();
    loaded
        .documents
        .iter()
        .map(|document| (document.path.clone(), document.bytes().to_vec()))
        .collect()
}

#[test]
fn explicit_and_named_fixtures_decode_generated_rows_and_original_source_spans() {
    let registry_owner = support::registry();
    let registry = registry_owner.as_ref();
    let explicit = load_package(
        &fixture("minimal_explicit"),
        registry,
        ParseBudget::default(),
    )
    .unwrap();
    let constant = reference::constants::View::from_checked(
        &explicit.batches[&reference::constants::RELATION_ID],
    )
    .unwrap()
    .row(0)
    .unwrap();
    assert_eq!(constant.name, "probe");
    let entity =
        authored::entities::View::from_checked(&explicit.batches[&authored::entities::RELATION_ID])
            .unwrap()
            .rows()
            .unwrap()
            .into_iter()
            .find(|entity| entity.entity_id == constant.constant_id)
            .unwrap();
    assert_eq!(entity.entity_id, constant.constant_id);
    assert_eq!(entity.qualified_name, "minimal.probe");
    let span = entity.source_span.unwrap();
    let document = explicit
        .documents
        .iter()
        .find(|document| document.id == span.document_id)
        .unwrap();
    let original = &document.text().unwrap()
        [usize::try_from(span.start).unwrap()..usize::try_from(span.end).unwrap()];
    assert!(
        original.contains("name: probe"),
        "row span: {original:?}; {span:?}"
    );
    assert!(original.contains("value: 2.0"));
    let named = load_package(&fixture("minimal_named"), registry, ParseBudget::default()).unwrap();
    let entity =
        authored::entities::View::from_checked(&named.batches[&authored::entities::RELATION_ID])
            .unwrap()
            .row(0)
            .unwrap();
    assert_eq!(
        entity.entity_id,
        pse_ids::named_id(named.package.package_id.as_id(), "minimal_named.probe")
    );
    assert_eq!(entity.name, "probe");
}

#[test]
fn missing_ids_unknown_fields_duplicate_keys_and_wrong_package_context_fail() {
    let registry_owner = support::registry();
    let registry = registry_owner.as_ref();
    let original = texts("minimal_explicit");
    let document = std::str::from_utf8(&original["materials/constants.yaml"]).unwrap();
    let variants = [
        document.replace(
            "id: \"01991d6a-13a0-7000-8000-000000000002\"",
            "unused: true",
        ),
        document.replace("name: probe", "name: probe\n    typo: value"),
        document.replace("name: probe", "name: probe\n    name: duplicate"),
        document.replace(
            "name: probe",
            "name: probe\n    package_id: '00000000000000000000000000000000'",
        ),
        document.replace(
            "name: probe",
            "name: probe\n    constant_id: '00000000000000000000000000000000'",
        ),
    ];
    for text in variants {
        let mut inputs = original.clone();
        inputs.insert("materials/constants.yaml".to_owned(), text.into_bytes());
        assert!(load_package_documents(inputs, registry, ParseBudget::default()).is_err());
    }
}
