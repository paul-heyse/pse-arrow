// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Independent domain applicability controls.
#[path = "../src/fixture.rs"]
mod fixture;
#[tokio::test]
async fn continuous_domain_applicability_units_and_detail_absence_are_independent_facts() {
    let registry = std::sync::Arc::new(pse_schema::catalog::assemble().unwrap());
    let invariant = registry
        .invariants()
        .iter()
        .find(|value| value.name == "domain:continuous_context")
        .unwrap();
    for (column, value) in [
        ("continuous", serde_json::json!(["bool", false])),
        ("unit_id", serde_json::json!(["null", null])),
    ] {
        let mut input = fixture::Fixture::load(&fixture::directory(invariant).join("valid.yaml"));
        input.rows.get_mut("authored.domains").unwrap()[0]
            .insert(column.to_owned(), value.to_string());
        let actual = fixture::execute(&registry, &input).await;
        assert_eq!(
            actual.len(),
            1,
            "{column} cannot be replaced by an unrelated matching detail"
        );
    }
    let invariant = registry
        .invariants()
        .iter()
        .find(|value| value.name == "cardinality:continuous_detail")
        .unwrap();
    let mut discrete =
        fixture::Fixture::load(&fixture::directory(invariant).join("violating.yaml"));
    discrete.rows.get_mut("authored.domains").unwrap()[0].insert(
        "continuous".to_owned(),
        serde_json::json!(["bool", false]).to_string(),
    );
    assert!(
        fixture::execute(&registry, &discrete).await.is_empty(),
        "discrete domains require no continuous detail"
    );
}
