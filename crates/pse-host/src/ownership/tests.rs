// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;

fn source_metadata(root: &Path) -> crate::catalog::MetadataSnapshot {
    let package = |name: &str, relative: &str| {
        serde_json::json!({
            "name": name, "version": "0.1.0", "id": format!("path+file:///{name}#{name}@0.1.0"),
            "manifest_path": root.join(relative).join("Cargo.toml"), "dependencies": [],
            "targets": [], "features": {}
        })
    };
    crate::catalog::MetadataSnapshot {
        metadata: serde_json::from_value(serde_json::json!({
            "packages": [package("outer", "crates/outer"), package("nested", "crates/outer/nested"), package("external", "vendor/external")],
            "workspace_members": ["path+file:///outer#outer@0.1.0", "path+file:///nested#nested@0.1.0"],
            "workspace_default_members": ["path+file:///outer#outer@0.1.0"],
            "workspace_root": root, "target_directory": root.join("target"), "version": 1
        })).unwrap(), digest: "retained-source-bytes".into()
    }
}

#[test]
fn source_scopes_use_captured_workspace_members_and_deepest_manifest() {
    let root = tempfile::tempdir().unwrap();
    let snapshot = source_metadata(root.path());
    let result = source_owner(
        root.path(),
        Path::new("crates/outer/nested/src/actual.rs"),
        Some(&snapshot),
    )
    .unwrap();
    assert!(matches!(
        result.owning_plan,
        PlanOwnership::NotDeclared { .. }
    ));
    match result.package {
        PackageOwnership::WorkspacePackage {
            name,
            package_id,
            metadata_digest,
            ..
        } => {
            assert_eq!(name, "nested");
            assert_eq!(package_id, "path+file:///nested#nested@0.1.0");
            assert_eq!(metadata_digest, "retained-source-bytes");
        }
        other => panic!("unexpected {other:?}"),
    }
    for path in [
        "crates/outer-extra/file.rs",
        "vendor/external/src/lib.rs",
        "docs/model.md",
    ] {
        assert!(matches!(
            source_owner(root.path(), Path::new(path), Some(&snapshot))
                .unwrap()
                .package,
            PackageOwnership::Unmapped { .. }
        ));
    }
    assert!(matches!(
        source_owner(root.path(), Path::new("crates/outer/src/lib.rs"), None)
            .unwrap()
            .package,
        PackageOwnership::Unavailable { .. }
    ));
    assert!(source_owner(root.path(), Path::new("../outside.rs"), Some(&snapshot)).is_err());
    let other = tempfile::tempdir().unwrap();
    assert!(source_owner(other.path(), Path::new("src/file.rs"), Some(&snapshot)).is_err());
    let mut malformed = source_metadata(root.path());
    malformed.metadata.packages[0].manifest_path = "relative/Cargo.toml".into();
    assert!(source_owner(root.path(), Path::new("src/file.rs"), Some(&malformed)).is_err());
}

#[cfg(unix)]
#[test]
fn source_scope_rejects_symlink_escape_without_requiring_selected_file_to_exist() {
    let root = tempfile::tempdir().unwrap();
    let other = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink(other.path(), root.path().join("escaped")).unwrap();
    assert!(source_owner(root.path(), Path::new("escaped/missing.rs"), None).is_err());
}

#[cfg(unix)]
#[test]
fn document_and_source_owner_wire_schemas_preserve_native_path_bytes() {
    use std::os::unix::ffi::OsStringExt as _;
    let path = OsString::from_vec(b"opaque-\xff.md".to_vec());
    let record = document::DocumentOwnership {
        path: path.clone(),
        doc_owner: None,
        historical_owner: None,
        doc_role: None,
        doc_retention: None,
        status: None,
        sections: vec![],
        workflow: document::Workflow::DocumentOwner,
        coverage_limits: vec![],
    };
    let value = serde_json::to_value(&record).unwrap();
    let schema = serde_json::to_value(schemars::schema_for!(document::DocumentOwnership)).unwrap();
    jsonschema::validator_for(&schema)
        .unwrap()
        .validate(&value)
        .unwrap();
    assert_eq!(
        serde_json::from_value::<document::DocumentOwnership>(value)
            .unwrap()
            .path,
        path
    );
    let root = tempfile::tempdir().unwrap();
    let record = source_owner(root.path(), Path::new(&path), None).unwrap();
    let value = serde_json::to_value(record).unwrap();
    let schema = serde_json::to_value(schemars::schema_for!(SourceOwnerQuery)).unwrap();
    jsonschema::validator_for(&schema)
        .unwrap()
        .validate(&value)
        .unwrap();
}

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
