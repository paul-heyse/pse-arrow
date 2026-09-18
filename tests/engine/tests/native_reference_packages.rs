// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Shipped sources enter the same source commit and native normalization pipeline.
#[path = "../../support/native_pipeline.rs"]
mod native_pipeline;

#[tokio::test]
async fn shipped_reference_packages_commit_and_normalize() {
    let fixture = native_pipeline::Fixture::new();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let documents = [
        "elements",
        "physical",
        "methods",
        "states",
        "units",
        "thermo-examples",
    ]
    .map(|name| {
        pse_authoring::document::load_package(
            &root.join("packages/reference").join(name),
            &fixture.registry,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap_or_else(|error| panic!("reference package {name}: {error}"))
    })
    .into_iter()
    .collect();
    let committed = fixture.source(documents);
    let report = fixture
        .evaluate(committed, "P3")
        .await
        .unwrap_or_else(|error| panic!("shipped normalization failed: {error}"));
    let output = &report;
    let sources = native_pipeline::relation(output, "normalized", "expression_sources").unwrap();
    let sources =
        pse_relations::generated::normalized::expression_sources::View::from_checked(sources)
            .unwrap()
            .rows()
            .unwrap();
    assert!(sources.iter().any(|source| source.source_relation_id
        == pse_relations::generated::authored::template_equations::RELATION_ID));
    assert!(
        sources.iter().all(|source| source.source_relation_id
            != pse_relations::generated::authored::template_submodels::RELATION_ID),
        "configuration bindings have their own actual producer and require no parallel mathematical graph"
    );
}
