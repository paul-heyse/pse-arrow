// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original-source editing and before-image admission.
use pse_ids::SemanticId;

#[test]
fn assignment_edits_original_block_and_flow_rows_and_checks_all_preimages() {
    use pse_authoring::ParseBudget;
    use pse_runtime::authoring_driver::document::{apply_edits, assign_ids, load_package_texts};
    use std::collections::BTreeMap;
    let registry = pse_engine::validation::registry().unwrap();
    let header = "[package]\nid='00000000000000000000000000000001'\nname='example'\nversion='1.0.0'\nkind='reference'\nid_policy='explicit'\ndependencies=[]\ndoc=''\n";
    for document in [
        "elements:\n  - symbol: H\n    name: Hydrogen\n    atomic_mass: 0.001\n",
        "elements: [{symbol: H, name: Hydrogen, atomic_mass: 0.001}]\n",
    ] {
        let mut texts = BTreeMap::from([
            ("package.toml".to_owned(), header.to_owned()),
            ("materials/element.yaml".to_owned(), document.to_owned()),
        ]);
        let edits = assign_ids(&texts, registry, ParseBudget::default(), &mut || {
            SemanticId::from_bytes([2; 16])
        })
        .unwrap();
        assert_eq!(edits.len(), 1);
        let original = texts.clone();
        apply_edits(&mut texts, &edits).unwrap();
        assert!(texts["materials/element.yaml"].contains("symbol: H"));
        load_package_texts(texts.clone(), registry, ParseBudget::default()).unwrap();
        assert!(apply_edits(&mut texts, &edits).is_err());
        let mut stale = original.clone();
        stale.insert("materials/element.yaml".to_owned(), "changed".to_owned());
        let before = stale.clone();
        assert!(apply_edits(&mut stale, &edits).is_err());
        assert_eq!(stale, before);
    }
}
