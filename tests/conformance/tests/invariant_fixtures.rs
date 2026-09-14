// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Every actual registered invariant has independently inspectable input/key fixtures.
#[path = "../src/fixture.rs"]
mod fixture;

#[tokio::test]
async fn every_registered_invariant_executes_its_valid_and_violating_fixture() {
    let registry = std::sync::Arc::new(pse_schema::catalog::assemble().expect("registry"));
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
    for invariant in registry.invariants() {
        for case in ["valid", "violating"] {
            let path = fixture::directory(invariant).join(format!("{case}.yaml"));
            let input = fixture::Fixture::load(&path);
            assert_eq!(input.invariant, invariant.qualified_name());
            let expected = input.expected(&registry);
            assert_eq!(expected.is_empty(), case == "valid");
            let actual = fixture::execute(&registry, &input).await;
            assert_eq!(actual, expected, "{} / {case}", invariant.qualified_name());
        }
    }
}

#[tokio::test]
async fn continuous_domain_applicability_units_and_detail_absence_are_independent_facts() {
    use pse_schema::model::Cell;
    let registry = std::sync::Arc::new(pse_schema::catalog::assemble().unwrap());
    let invariant = registry
        .invariants()
        .iter()
        .find(|value| value.name == "domain:continuous_context")
        .unwrap();
    for (column, value) in [("continuous", Cell::Bool(false)), ("unit_id", Cell::Null)] {
        let mut input = fixture::Fixture::load(&fixture::directory(invariant).join("valid.yaml"));
        input.rows.get_mut("authored.domains").unwrap()[0]
            .insert(column.to_owned(), value.literal_spec());
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
    discrete.rows.get_mut("authored.domains").unwrap()[0]
        .insert("continuous".to_owned(), Cell::Bool(false).literal_spec());
    assert!(
        fixture::execute(&registry, &discrete).await.is_empty(),
        "discrete domains require no continuous detail"
    );
}
