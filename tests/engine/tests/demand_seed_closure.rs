// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual public P0–P6 execution: source reads, relational guard selection and finite dependency closure.
#[path = "support/demand_source.rs"]
mod demand_source;
#[path = "../../support/native_pipeline.rs"]
mod native_pipeline;
use demand_source::id;
use pse_schema::model::Cell;
use std::collections::BTreeSet;

#[tokio::test]
async fn demand_seed_closure_uses_actual_source_guards_and_transitive_method_support() {
    let fixture = native_pipeline::Fixture::new();
    let documents = vec![demand_source::source(&fixture.registry, false)];
    let model = fixture.source(documents);
    let result = fixture
        .evaluate(model, "P6")
        .await
        .unwrap_or_else(|error| panic!("actual P3/P4/P5/P6 execution: {error}"));
    let stage = &result;
    let required = native_pipeline::relation(stage, "inferred", "property_requirements").unwrap();
    let spec = fixture
        .registry
        .relation("inferred.property_requirements")
        .unwrap();
    let rows =
        pse_relations::cells::cells_from_batch(&fixture.registry, spec, required.batch()).unwrap();
    let properties = rows
        .iter()
        .map(|row| row[2].literal_spec())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        properties,
        [100, 101, 102]
            .map(|value| Cell::Id(id(value)).literal_spec())
            .into_iter()
            .collect()
    );
    assert_eq!(
        rows.len(),
        3,
        "duplicate source reads coalesce by actual state/kind/index"
    );
    let resolutions = native_pipeline::relation(stage, "inferred", "method_resolutions").unwrap();
    assert_eq!(resolutions.batch().num_rows(), 3);
    let supports = native_pipeline::relation(stage, "inferred", "requirement_support").unwrap();
    assert!(
        supports.batch().num_rows() >= 3,
        "original seed and both dependency edges retained"
    );
    let scopes = native_pipeline::relation(stage, "inferred", "state_scopes").unwrap();
    assert_eq!(scopes.batch().num_rows(), 1);
    let reopened = fixture.roundtrip(stage).await;
    assert_eq!(
        native_pipeline::relation(&reopened, "inferred", "property_requirements")
            .unwrap()
            .batch(),
        required.batch()
    );
}

#[tokio::test]
async fn equal_rank_distinct_methods_refuse_instead_of_selecting_by_identity_order() {
    let fixture = native_pipeline::Fixture::new();
    let documents = vec![demand_source::source(&fixture.registry, true)];
    let model = fixture.source(documents);
    let result = fixture.evaluate(model, "P6").await;
    assert!(
        result.is_err(),
        "two actual equal-rank distinct methods are ambiguous"
    );
}
