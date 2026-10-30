// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;

fn fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    let owner = |producer: &str, path: &str, kind, contributors| GeneratorOwner {
        producer: producer.into(),
        outputs: vec![Scope {
            path: path.into(),
            kind,
        }],
        regenerate: vec!["just".into(), "codegen".into()],
        inputs: vec![Scope {
            path: "authored".into(),
            kind: ScopeKind::Tree,
        }],
        contributors,
    };
    let record = OutputMap {
        schema_version: VERSION,
        comment: "generated fixture".into(),
        coverage_limits: vec!["declared fixture inputs only".into()],
        owners: vec![
            owner("docs", "docs/generated", ScopeKind::Tree, None),
            owner(
                "schemas",
                "docs/generated/schema",
                ScopeKind::Tree,
                Some("docs/relationships.json".into()),
            ),
            owner("exact", "other/output.json", ScopeKind::File, None),
            owner(MAP_PRODUCER, MAP_PATH, ScopeKind::File, None),
        ],
    };
    fs::create_dir_all(root.path().join("scripts/generated")).unwrap();
    fs::write(
        root.path().join(MAP_PATH),
        serde_json::to_vec(&record).unwrap(),
    )
    .unwrap();
    let relationships = RelationshipDocument {
        comment: "original fixture".into(),
        schema_version: 1,
        producer: "schemas".into(),
        coverage_limits: vec!["authored location is unknown".into()],
        relationships: vec![SourceRelationship {
            declaration_key: "Actual@1".into(),
            declaration_kind: "relation".into(),
            source_path: None,
            source_symbol: None,
            source_role: "semantic_declaration_without_source_location".into(),
            source_identity: Some("original producer identity".into()),
            generator_path: "authored/generator.rs".into(),
            generator_symbol: "emit".into(),
            output_path: "docs/generated/schema/actual.json".into(),
            direction: None,
            contract_sections: vec![],
            basis: "declared".into(),
        }],
    };
    fs::create_dir_all(root.path().join("docs")).unwrap();
    fs::write(
        root.path().join("docs/relationships.json"),
        serde_json::to_vec(&relationships).unwrap(),
    )
    .unwrap();
    root
}

#[test]
fn longest_component_scope_and_exact_file_ownership_do_not_infer_prefixes() {
    let root = fixture();
    let map = OwnershipMap::read(root.path()).unwrap();
    assert_eq!(
        map.owner(Path::new("docs/generated/schema/actual.json"))
            .unwrap()
            .unwrap()
            .0
            .producer,
        "schemas"
    );
    assert_eq!(
        map.owner(Path::new("docs/generated/other.md"))
            .unwrap()
            .unwrap()
            .0
            .producer,
        "docs"
    );
    assert!(
        map.owner(Path::new("docs/generated-extra/file"))
            .unwrap()
            .is_none()
    );
    assert!(
        map.owner(Path::new("other/output.json/child"))
            .unwrap()
            .is_none()
    );
    assert!(map.owner(Path::new("../escape")).is_err());
    assert_eq!(
        map.input_owners(Path::new("authored/input.rs"))
            .unwrap()
            .len(),
        4
    );
}

#[test]
fn original_contributors_keep_unknown_locations_and_failure_is_not_empty_lineage() {
    let root = fixture();
    let response = query(root.path(), Path::new("docs/generated/schema/actual.json")).unwrap();
    let Contributors::Available {
        records,
        coverage_limits,
        ..
    } = response.contributors
    else {
        panic!("original sidecar available")
    };
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].declaration_key, "Actual@1");
    assert!(records[0].source_path.is_none());
    assert_eq!(coverage_limits, ["authored location is unknown"]);
    fs::remove_file(root.path().join("docs/relationships.json")).unwrap();
    let response = query(root.path(), Path::new("docs/generated/schema/actual.json")).unwrap();
    assert!(matches!(
        response.ownership,
        OutputOwnership::Declared { .. }
    ));
    assert!(matches!(
        response.contributors,
        Contributors::Unavailable { .. }
    ));
    assert!(matches!(
        response.failures.as_slice(),
        [OwnershipError::Io(_)]
    ));
}

#[test]
fn missing_malformed_or_duplicate_maps_are_typed_failures_without_rebuilding() {
    let root = fixture();
    let original = fs::read(root.path().join(MAP_PATH)).unwrap();
    fs::remove_file(root.path().join(MAP_PATH)).unwrap();
    assert!(matches!(
        OwnershipMap::read(root.path()),
        Err(OwnershipError::Io(_))
    ));
    fs::write(root.path().join(MAP_PATH), b"not JSON").unwrap();
    assert!(matches!(
        OwnershipMap::read(root.path()),
        Err(OwnershipError::Json(_))
    ));
    let mut record: OutputMap = serde_json::from_slice(&original).unwrap();
    record.owners.push(record.owners[0].clone());
    fs::write(
        root.path().join(MAP_PATH),
        serde_json::to_vec(&record).unwrap(),
    )
    .unwrap();
    assert!(matches!(
        OwnershipMap::read(root.path()),
        Err(OwnershipError::Invalid(_))
    ));
}

#[test]
fn normalized_native_non_utf8_paths_agree_with_the_actual_serde_schema() {
    use std::os::unix::ffi::OsStringExt as _;
    let root = fixture();
    let path = PathBuf::from(OsString::from_vec(
        b"docs/generated/opaque-\xff.md".to_vec(),
    ));
    let response = query(root.path(), &path).unwrap();
    let value = serde_json::to_value(&response).unwrap();
    let schema = serde_json::to_value(schemars::schema_for!(OwnerQuery)).unwrap();
    jsonschema::validator_for(&schema)
        .unwrap()
        .validate(&value)
        .unwrap();
    assert!(matches!(
        response.ownership,
        OutputOwnership::Declared { .. }
    ));
    assert!(
        value["path"]["Unix"]
            .as_array()
            .unwrap()
            .iter()
            .any(|byte| byte == 255)
    );
}
