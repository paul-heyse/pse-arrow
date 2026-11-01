// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use serde_json::json;
#[test]
fn actual_cargo_contexts_preserve_renames_feature_policy_and_host_target_test_units() {
    use crate::{selection, test_workflow};
    use std::{fs, process::Command};

    let scratch = tempfile::tempdir().unwrap();
    let root = scratch.path();
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers=[\"app\",\"supplier\",\"metadata-only\"]\nresolver=\"3\"\n",
    )
    .unwrap();
    for name in ["app", "supplier", "metadata-only"] {
        fs::create_dir_all(root.join(name).join("src")).unwrap();
        fs::write(
            root.join(name).join("src/lib.rs"),
            "pub fn value() {}\n#[test] fn unit_test() {}\n",
        )
        .unwrap();
    }
    fs::write(root.join("supplier/Cargo.toml"), "[package]\nname=\"supplier\"\nversion=\"0.0.0\"\nedition=\"2024\"\n[features]\ndefault=[\"supplier-default\"]\nsupplier-default=[]\nenabled=[]\nalternate=[]\nbuild-only=[]\n").unwrap();
    fs::write(
        root.join("metadata-only/Cargo.toml"),
        "[package]\nname=\"metadata-only\"\nversion=\"0.0.0\"\nedition=\"2024\"\n",
    )
    .unwrap();
    fs::write(
        root.join("app/Cargo.toml"),
        r#"[package]
name="app"
version="0.0.0"
edition="2024"
[dependencies]
alias={package="supplier",path="../supplier",default-features=false}
[build-dependencies]
alias={package="supplier",path="../supplier",default-features=false,features=["build-only"]}
[features]
default=["alias/enabled"]
alternate=["alias/alternate"]
"#,
    )
    .unwrap();
    fs::write(root.join("app/build.rs"), "fn main() { alias::value(); }\n").unwrap();
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    fs::copy(
        checkout.join("rust-toolchain.toml"),
        root.join("rust-toolchain.toml"),
    )
    .unwrap();
    let mut environment: Environment = std::env::vars_os().collect();
    environment.remove(std::ffi::OsStr::new("CARGO_BUILD_TARGET"));
    let lock = Command::new(crate::correctness::real_cargo_in(&environment).unwrap())
        .current_dir(root)
        .env_clear()
        .envs(&environment)
        .args(["generate-lockfile", "--offline"])
        .output()
        .unwrap();
    assert!(lock.status.success(), "{lock:?}");
    let version = Command::new("rustc")
        .current_dir(root)
        .env_clear()
        .envs(&environment)
        .arg("-vV")
        .output()
        .unwrap();
    assert!(version.status.success(), "{version:?}");
    let target = String::from_utf8(version.stdout)
        .unwrap()
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .unwrap()
        .to_owned();
    let (snapshot, original) = capture_original(root, &environment).unwrap();
    let original: serde_json::Value = serde_json::from_slice(&original).unwrap();
    assert!(
        original["packages"]
            .as_array()
            .unwrap()
            .iter()
            .any(|package| package["name"] == "metadata-only")
    );
    let Selection::Exact { facts } = inspect(&snapshot, "supplier", "actual_locked_metadata")
        .payload
        .selected
    else {
        panic!("supplier identity missing")
    };
    let supplier_id = facts.package.id.clone();
    let uses = facts.workspace_uses.as_ref().unwrap();
    assert!(
        uses.iter().any(|usage| usage.package == "app"
            && usage.dependency_name == "alias"
            && usage
                .declarations
                .iter()
                .any(|declaration| !declaration.uses_default_features
                    && declaration.kind == "normal")),
        "{uses:?}"
    );
    let request = |operation, package: &str, features| selection::Request {
        operation,
        scope: selection::Scope::Packages(vec![selection::PackageIntent {
            package: package.into(),
            target: Some(selection::TargetIntent {
                name: package.into(),
                kind: Some("lib".into()),
            }),
            features,
        }]),
        platform: Some(target.clone()),
        filters: selection::Filters::default(),
        ignored: selection::IgnoredPolicy::Default,
    };
    let defaults = test_workflow::CheckIntent::new(
        snapshot.metadata.clone(),
        request(
            selection::Operation::Check,
            "app",
            selection::Features::default(),
        ),
        environment.clone(),
    )
    .unwrap()
    .admit()
    .unwrap();
    let alternate = test_workflow::CheckIntent::new(
        snapshot.metadata.clone(),
        request(
            selection::Operation::Check,
            "app",
            selection::Features {
                defaults: false,
                enabled: ["alternate".into()].into(),
                ..Default::default()
            },
        ),
        environment.clone(),
    )
    .unwrap()
    .admit()
    .unwrap();
    let units = |admitted: &test_workflow::AdmittedCheck| {
        admitted
            .resolved()
            .admitted()
            .iter()
            .flat_map(|context| context.units.clone())
            .collect::<Vec<_>>()
    };
    let default_units = units(&defaults);
    let alternate_units = units(&alternate);
    for (selected, expected, absent) in [
        (&default_units, "enabled", "alternate"),
        (&alternate_units, "alternate", "enabled"),
    ] {
        assert!(
            !selected
                .iter()
                .any(|unit| unit.package_name == "metadata-only")
        );
        let supplier = selected
            .iter()
            .filter(|unit| unit.package_id == supplier_id)
            .collect::<Vec<_>>();
        assert!(
            supplier
                .iter()
                .any(|unit| unit.platform.as_deref() == Some(target.as_str())
                    && unit.features.contains(expected)
                    && !unit.features.contains(absent)
                    && !unit.features.contains("supplier-default")),
            "{supplier:?}"
        );
        assert!(
            supplier
                .iter()
                .any(|unit| unit.platform.is_none() && unit.features.contains("build-only")),
            "{supplier:?}"
        );
    }
    let tests = test_workflow::Intent::new(
        snapshot.metadata,
        request(
            selection::Operation::Test,
            "supplier",
            selection::Features::default(),
        ),
        "default".into(),
        environment,
    )
    .unwrap()
    .admit()
    .unwrap();
    assert!(
        tests
            .resolved()
            .admitted()
            .iter()
            .flat_map(|context| &context.units)
            .any(|unit| unit.package_id == supplier_id
                && unit.mode == "test"
                && unit.platform.as_deref() == Some(target.as_str())
                && unit.features.contains("supplier-default"))
    );
    assert!(
        !root.join("target/debug/deps").exists(),
        "actual unit admission grants no compilation or executed-test evidence"
    );
}
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
fn overview_candidates_preserve_root_nested_and_duplicate_basename_associations() {
    let scratch = tempfile::tempdir().unwrap();
    let root = scratch.path();
    let libraries = root.join("docs/libraries");
    std::fs::create_dir_all(libraries.join("nested/deeper")).unwrap();
    let direct = libraries.join("dep.md");
    let nested = libraries.join("nested/dep.md");
    let deeper = libraries.join("nested/deeper/dep.md");
    std::fs::write(&direct, "root overview").unwrap();
    std::fs::write(libraries.join("dep-extra.md"), "dep").unwrap();
    std::fs::write(libraries.join("other.md"), "dep overview by text only").unwrap();
    let mut snapshot = snapshot();
    snapshot.metadata.workspace_root = root.to_str().unwrap().into();
    let candidates = || {
        let Selection::Exact { facts } = inspect(&snapshot, "first", "fixture").payload.selected
        else {
            panic!("exact fixture dependency missing")
        };
        facts.overview_candidates
    };
    assert_eq!(candidates(), vec![direct.clone()]);
    std::fs::rename(&direct, &nested).unwrap();
    assert_eq!(candidates(), vec![nested.clone()]);
    std::fs::write(&direct, "root alternative").unwrap();
    std::fs::write(&deeper, "second nested alternative").unwrap();
    let outside = root.join("outside.md");
    std::fs::write(&outside, "unrelated outside bytes").unwrap();
    std::fs::create_dir(libraries.join("linked")).unwrap();
    std::os::unix::fs::symlink(&outside, libraries.join("linked/dep.md")).unwrap();
    let mut expected = vec![direct, nested, deeper];
    expected.sort();
    assert_eq!(
        candidates(),
        expected,
        "every exact filename remains a candidate; no arbitrary winner, text match, or followed symlink"
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
    assert!(!facts.overview_candidates.is_empty());
    assert!(facts.overview_candidates.iter().all(|path| path.is_file()
        && path.file_name() == Some(std::ffi::OsStr::new("clap.md"))
        && path.starts_with(root.join("docs/libraries"))));
    assert!(
        facts
            .workspace_uses
            .unwrap()
            .iter()
            .any(|usage| usage.package == "pse-host")
    );
    assert_eq!(before, std::fs::read(root.join("Cargo.lock")).unwrap());
}
