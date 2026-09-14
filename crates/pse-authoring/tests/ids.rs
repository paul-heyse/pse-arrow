// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Identity policy checks preserve explicit bytes and verify named-policy correspondence.

#![allow(
    clippy::unwrap_used,
    reason = "fixture failures retain the typed authoring diagnostic"
)]

use pse_authoring::{
    AuthoringError, SourceSpan,
    ids::{IdPolicy, entity_id, parse_id, uuid_v7},
};
use pse_ids::{SemanticId, named_id};

#[test]
fn explicit_and_named_identity_policies_have_distinct_creation_semantics() {
    let package = SemanticId::from_bytes([1; 16]);
    let at = SourceSpan::head(package);
    assert!(matches!(
        entity_id(IdPolicy::Explicit, package, None, "pkg.object", at),
        Err(AuthoringError::MissingId { .. })
    ));
    let explicit = uuid_v7();
    assert_eq!(explicit.as_bytes()[6] >> 4, 7);
    for name in ["pkg.before", "pkg.after"] {
        assert_eq!(
            entity_id(
                IdPolicy::Explicit,
                package,
                Some(&explicit.to_hex()),
                name,
                at
            )
            .unwrap(),
            explicit
        );
    }
    assert_eq!(
        entity_id(IdPolicy::Named, package, None, "pkg.object", at).unwrap(),
        named_id(package, "pkg.object")
    );
    assert!(
        entity_id(
            IdPolicy::Named,
            package,
            Some(&explicit.to_hex()),
            "pkg.object",
            at
        )
        .is_err()
    );
}

#[test]
fn compact_and_hyphenated_id_spellings_preserve_the_same_identity() {
    let at = SourceSpan::head(SemanticId::NIL);
    let compact = parse_id("01900000000070008000000000000001", at).unwrap();
    let uuid = parse_id("01900000-0000-7000-8000-000000000001", at).unwrap();
    assert_eq!(compact, uuid);
    for malformed in ["", "xyz", "0190000000007000800000000000000"] {
        assert!(parse_id(malformed, at).is_err());
    }
}

#[test]
fn assignment_edits_original_block_and_flow_rows_and_checks_all_preimages() {
    use pse_authoring::{
        ParseBudget,
        document::{apply_edits, assign_ids, load_package_texts},
    };
    use std::collections::BTreeMap;
    let registry = pse_schema::registry().unwrap();
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
