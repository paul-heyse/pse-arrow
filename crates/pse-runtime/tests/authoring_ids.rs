// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original-source editing and before-image admission.
use pse_ids::SemanticId;

#[test]
fn assignment_edits_original_block_and_flow_rows_and_checks_all_preimages() {
    use pse_authoring::ParseBudget;
    use pse_runtime::authoring_driver::document::{
        apply_edits, assign_ids, load_package_documents,
    };
    use std::collections::BTreeMap;
    let registry = pse_schema::registry().unwrap();
    let header = "[package]\nid='00000000000000000000000000000001'\nname='example'\nversion='1.0.0'\nkind='reference'\nid_policy='explicit'\ndependencies=[]\ndoc=''\n";
    for document in [
        "quantity_kinds:\n  - name: probe\n    dimension: [{num: 0, den: 1}, {num: 0, den: 1}, {num: 0, den: 1}, {num: 0, den: 1}, {num: 0, den: 1}, {num: 0, den: 1}, {num: 0, den: 1}, {num: 0, den: 1}]\n    category: count\n    extensive: false\n    addition_kind: additive\n    doc: ''\n",
        "quantity_kinds: [{name: probe, dimension: [{num: 0, den: 1}, {num: 0, den: 1}, {num: 0, den: 1}, {num: 0, den: 1}, {num: 0, den: 1}, {num: 0, den: 1}, {num: 0, den: 1}, {num: 0, den: 1}], category: count, extensive: false, addition_kind: additive, doc: ''}]\n",
    ] {
        let mut texts = BTreeMap::from([
            ("package.toml".to_owned(), header.as_bytes().to_vec()),
            (
                "materials/quantity-kinds.yaml".to_owned(),
                document.as_bytes().to_vec(),
            ),
        ]);
        let edits = assign_ids(
            &texts,
            registry,
            ParseBudget::default(),
            &mut || SemanticId::from_bytes([2; 16]),
            &fixture_validation(registry),
        )
        .unwrap();
        assert_eq!(edits.len(), 1);
        let original = texts.clone();
        apply_edits(&mut texts, &edits).unwrap();
        assert!(
            String::from_utf8_lossy(&texts["materials/quantity-kinds.yaml"])
                .contains("name: probe")
        );
        load_package_documents(
            texts.clone(),
            registry,
            ParseBudget::default(),
            &fixture_validation(registry),
        )
        .unwrap();
        assert!(apply_edits(&mut texts, &edits).is_err());
        let mut stale = original.clone();
        stale.insert(
            "materials/quantity-kinds.yaml".to_owned(),
            b"changed".to_vec(),
        );
        let before = stale.clone();
        assert!(apply_edits(&mut stale, &edits).is_err());
        assert_eq!(stale, before);
    }
}

fn fixture_validation(
    registry: &pse_schema::Registry,
) -> std::sync::Arc<pse_relations::validate::ValidationContext> {
    // This source-only fixture deliberately captures its fixed native session state.
    std::sync::Arc::new(pse_relations::validate::ValidationContext::new(
        registry,
        pse_engine::validation::NativeValidation(
            datafusion::prelude::SessionContext::new().state(),
        ),
    ))
}
