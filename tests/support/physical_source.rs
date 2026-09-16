// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use pse_schema::Registry;

/// Explicit domain declarations, without a mathematical graph or producer fixture.
pub(crate) fn physical_texts(registry: &Registry) -> std::collections::BTreeMap<String, String> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    pse_authoring::document::load_package(
        &root.join("tests/fixtures/packages/physical-primitives"),
        registry,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap()
    .documents
    .iter()
    .map(|document| (document.path.clone(), document.text.clone()))
    .collect()
}
