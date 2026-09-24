// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Every actual registered invariant has independently inspectable input/key fixtures.
#[path = "../src/fixture.rs"]
mod fixture;

#[path = "generated/invariant_cases.rs"]
mod generated;

#[test]
fn every_registered_invariant_has_exactly_its_fixture_directory() {
    let registry = std::sync::Arc::new(pse_schema::catalog::assemble().unwrap());
    let expected_directories: std::collections::BTreeSet<_> = registry
        .invariants()
        .iter()
        .map(fixture::directory)
        .collect();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/invariants");
    let actual_directories = std::fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_dir())
        .collect();
    assert_eq!(
        expected_directories, actual_directories,
        "no stale or unregistered fixture directory"
    );
}

#[expect(
    clippy::unwrap_used,
    reason = "generated fixture names resolve the reviewed registry"
)]
async fn assert_fixture(name: &str, case: &str) {
    let registry = std::sync::Arc::new(pse_schema::catalog::assemble().unwrap());
    let invariant = registry
        .invariants()
        .iter()
        .find(|value| value.qualified_name() == name)
        .unwrap();
    let path = fixture::directory(invariant).join(format!("{case}.yaml"));
    let input = fixture::Fixture::load(&path);
    assert_eq!(input.invariant, name);
    let expected = input.expected(&registry);
    assert_eq!(expected.is_empty(), case == "valid");
    let actual = fixture::execute(&registry, &input).await;
    assert_eq!(actual, expected, "{name} / {case}");
}

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
