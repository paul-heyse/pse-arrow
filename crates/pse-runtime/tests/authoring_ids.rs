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
        "constants:\n  - name: probe\n    value: 2.0\n    unit_id: '00000000000000000000000000000011'\n    quantity_kind_id: '00000000000000000000000000000021'\n    doc: ''\n",
        "constants: [{name: probe, value: 2.0, unit_id: '00000000000000000000000000000011', quantity_kind_id: '00000000000000000000000000000021', doc: ''}]\n",
    ] {
        let mut texts = BTreeMap::from([
            ("package.toml".to_owned(), header.to_owned()),
            ("materials/constants.yaml".to_owned(), document.to_owned()),
        ]);
        let edits = assign_ids(&texts, registry, ParseBudget::default(), &mut || {
            SemanticId::from_bytes([2; 16])
        })
        .unwrap();
        assert_eq!(edits.len(), 1);
        let original = texts.clone();
        apply_edits(&mut texts, &edits).unwrap();
        assert!(texts["materials/constants.yaml"].contains("name: probe"));
        load_package_texts(texts.clone(), registry, ParseBudget::default()).unwrap();
        assert!(apply_edits(&mut texts, &edits).is_err());
        let mut stale = original.clone();
        stale.insert("materials/constants.yaml".to_owned(), "changed".to_owned());
        let before = stale.clone();
        assert!(apply_edits(&mut stale, &edits).is_err());
        assert_eq!(stale, before);
    }
}
