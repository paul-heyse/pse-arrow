// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use serde_json::json;
fn snapshot() -> MetadataSnapshot {
    let package = |id: &str, name: &str, version: &str| json!({"id":id,"name":name,"version":version,"source":null,"manifest_path":format!("/workspace/{id}/Cargo.toml"),"dependencies":[],"targets":[],"features":{"default":["extra"],"extra":[]}});
    let bytes = serde_json::to_vec(&json!({
        "packages":[package("first", "dep", "1.0.0"),package("second", "dep", "1.0.0"),package("newer", "dep", "2.0.0"),package("consumer", "consumer", "0.1.0")],
        "workspace_members":["consumer"],"workspace_default_members":["consumer"],"workspace_root":"/workspace","target_directory":"/workspace/target","version":1,
        "metadata":{"pse":{"pins":{"dep":"retained supplier hold"}}},
        "resolve":{"root":null,"nodes":[
            {"id":"consumer","dependencies":["first"],"deps":[{"name":"renamed_dep","pkg":"first","dep_kinds":[{"kind":"build","target":"cfg(unix)"}]}],"features":[]},
            {"id":"first","dependencies":[],"deps":[],"features":["extra"]}
        ]}
    })).unwrap();
    MetadataSnapshot {
        metadata: serde_json::from_slice(&bytes).unwrap(),
        digest: "captured-metadata-digest".into(),
    }
}
#[test]
fn equal_names_and_versions_preserve_source_ambiguity_and_exact_identity() {
    let snapshot = snapshot();
    assert!(
        matches!(inspect(&snapshot, "dep", "fixture").payload.selected, Selection::Ambiguous { candidates } if candidates.len() == 3)
    );
    assert!(
        matches!(inspect(&snapshot, "dep@1.0.0", "fixture").payload.selected, Selection::Ambiguous { candidates } if candidates.len() == 2)
    );
    assert!(matches!(
        inspect(&snapshot, "dep@2.0.0", "fixture").payload.selected,
        Selection::Exact { .. }
    ));
    let exact = inspect(&snapshot, "first", "fixture");
    let Selection::Exact { facts } = exact.payload.selected else {
        panic!("exact source was lost")
    };
    assert_eq!(facts.package.id, "first");
    assert_eq!(facts.metadata_features, Some(vec!["extra".into()]));
    assert_eq!(facts.pin_reason.as_deref(), Some("retained supplier hold"));
    let usage = &facts.workspace_uses.unwrap()[0];
    assert_eq!(usage.dependency_name, "renamed_dep");
    assert_eq!(usage.kinds[0].kind, "build");
    assert_eq!(usage.kinds[0].target.as_deref(), Some("cfg(unix)"));
}
#[test]
fn omitted_graph_is_unknown_not_empty_features_or_no_consumers() {
    let mut snapshot = snapshot();
    snapshot.metadata.resolve = None;
    let response = inspect(&snapshot, "first", "explicit_retained_snapshot");
    let Selection::Exact { facts } = response.payload.selected else {
        panic!("package absent")
    };
    assert!(facts.metadata_features.is_none());
    assert!(facts.workspace_uses.is_none());
    assert_eq!(
        inspect(&snapshot, "missing", "fixture").subject_state,
        SubjectState::NoMatch
    );
}
#[test]
fn package_paths_and_authored_feature_links_do_not_simulate_activation() {
    let mut snapshot = snapshot();
    snapshot.metadata.packages.iter_mut().find(|package| package.id.repr == "consumer").unwrap()
        .dependencies.push(serde_json::from_value(json!({
            "name":"dep", "rename":"renamed-dep", "req":"^1.0", "kind":"build",
            "target":"cfg(unix)", "path":"/workspace/first", "optional":true, "uses_default_features":false, "features":["requested-not-enabled"]
        })).unwrap());
    let consumer = snapshot
        .metadata
        .packages
        .iter_mut()
        .find(|package| package.id.repr == "consumer")
        .unwrap();
    let valid = consumer.dependencies[0].clone();
    let mut other_version = valid.clone();
    other_version.req = "^2.0".parse().unwrap();
    let mut other_target = valid.clone();
    other_target.target = Some("cfg(windows)".parse().unwrap());
    let mut other_kind = valid.clone();
    other_kind.kind = cargo_metadata::DependencyKind::Normal;
    let mut other_source = valid;
    other_source.path = Some("/workspace/elsewhere".into());
    consumer
        .dependencies
        .extend([other_version, other_target, other_kind, other_source]);
    let response = inspect_details(&snapshot, "first", "fixture", true, Some("extra"));
    let Selection::Exact { facts } = &response.payload.selected else {
        panic!("exact dependency lost")
    };
    let declaration = &facts.workspace_uses.as_ref().unwrap()[0].declarations[0];
    assert_eq!(
        facts.workspace_uses.as_ref().unwrap()[0].declarations.len(),
        1
    );
    assert!(declaration.optional);
    assert!(!declaration.uses_default_features);
    assert_eq!(declaration.features, ["requested-not-enabled"]);
    assert_eq!(declaration.target.as_deref(), Some("cfg(unix)"));
    assert_eq!(facts.metadata_features.as_ref().unwrap(), &["extra"]);
    let paths = response.payload.inclusion_paths.unwrap();
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].packages, ["consumer", "first"]);
    let features = response.payload.feature_uses.unwrap();
    assert_eq!(features.len(), 4);
    assert!(
        features
            .iter()
            .any(|usage| usage.package == "first" && usage.feature == "default")
    );
    assert!(
        response
            .observation
            .limitations
            .iter()
            .any(|value| value.contains("does not resolve or activate"))
    );
    let absent = inspect_details(&snapshot, "newer", "fixture", true, Some("absent"));
    assert!(absent.payload.inclusion_paths.unwrap().is_empty());
    assert!(absent.payload.feature_uses.unwrap().is_empty());
    let mut missing_graph = snapshot;
    missing_graph.metadata.resolve = None;
    assert!(
        inspect_details(&missing_graph, "first", "fixture", true, None)
            .payload
            .inclusion_paths
            .is_none()
    );
}
#[test]
fn actual_locked_offline_metadata_keeps_workspace_lock_and_reports_pinned_clap() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    let before = std::fs::read(root.join("Cargo.lock")).unwrap();
    let snapshot = capture(root, &std::env::vars_os().collect()).unwrap();
    let report = inspect(&snapshot, "clap", "locked_offline_metadata");
    let Selection::Exact { facts } = report.payload.selected else {
        panic!("ambiguous/missing locked clap")
    };
    let manifest: toml::Value =
        toml::from_str(&std::fs::read_to_string(root.join("Cargo.toml")).unwrap()).unwrap();
    let declared = manifest["workspace"]["dependencies"]["clap"]["version"]
        .as_str()
        .unwrap()
        .strip_prefix('=')
        .unwrap();
    assert_eq!(facts.package.version, declared);
    assert!(
        facts
            .overview_candidates
            .iter()
            .any(|path| path.ends_with("docs/libraries/clap.md"))
    );
    assert!(
        facts
            .workspace_uses
            .unwrap()
            .iter()
            .any(|usage| usage.package == "pse-host")
    );
    assert_eq!(before, std::fs::read(root.join("Cargo.lock")).unwrap());
}
