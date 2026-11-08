// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use serde_json::{Value, json};
use std::{ffi::OsStr, fs, process::Command};

fn metadata(root: &Path) -> Metadata {
    let dependency = |name: &str, path: &str, kind: &str, optional: bool| json!({"name":name,"path":root.join(path),"source":null,"req":"*","kind":kind,"optional":optional,"uses_default_features":true,"features":[],"target":"cfg(unix)","rename":"renamed","registry":null});
    let package = |name: &str, directory: &str, dependencies: Vec<Value>| {
        json!({
            "name":name,"version":"1.0.0","id":format!("opaque:{name}@1#source"),"manifest_path":root.join(directory).join("Cargo.toml"),"dependencies":dependencies,
            "targets":[{"name":name,"kind":["lib"],"crate_types":["lib"],"src_path":root.join(directory).join("src/lib.rs"),"edition":"2024"},
                {"name":"native_journey","kind":["test"],"crate_types":["bin"],"src_path":root.join(directory).join("tests/journey.rs"),"edition":"2024","required-features":["native"]}],"features":{"default":[],"native":[]}
        })
    };
    serde_json::from_value(json!({"version":1,"workspace_root":root,"target_directory":root.join("target"),"resolve":null,
        "workspace_members":["opaque:pse-backend-native@1#source","opaque:consumer@1#source","opaque:downstream@1#source","opaque:unrelated@1#source","opaque:nested@1#source"],
        "workspace_default_members":["opaque:unrelated@1#source"],
        "packages":[package("pse-backend-native","crates/base",vec![]),package("consumer","crates/consumer",vec![dependency("pse-backend-native","crates/base","build",true)]),
            package("downstream","tests/downstream",vec![dependency("consumer","crates/consumer","dev",false)]),package("unrelated","crates/unrelated",vec![]),package("nested","crates/base/nested",vec![]),package("external","vendor/external",vec![])]
    })).unwrap()
}
fn map(root: &Path, input: Option<&str>, output: &str) {
    use ownership::protocol::{
        GeneratorOwner, MAP_PATH, MAP_PRODUCER, OutputMap, Scope, ScopeKind, VERSION,
    };
    let record = OutputMap {
        schema_version: VERSION,
        comment: "independent declaration".into(),
        coverage_limits: vec![],
        owners: vec![
            GeneratorOwner {
                producer: MAP_PRODUCER.into(),
                outputs: vec![Scope {
                    path: MAP_PATH.into(),
                    kind: ScopeKind::File,
                }],
                regenerate: vec!["just".into(), "codegen".into()],
                inputs: vec![],
                contributors: None,
            },
            GeneratorOwner {
                producer: "independent-generator".into(),
                outputs: vec![Scope {
                    path: output.into(),
                    kind: ScopeKind::Tree,
                }],
                regenerate: vec!["just".into(), "codegen".into(), "rust".into()],
                inputs: input
                    .into_iter()
                    .map(|path| Scope {
                        path: path.into(),
                        kind: ScopeKind::Tree,
                    })
                    .collect(),
                contributors: None,
            },
        ],
    };
    fs::create_dir_all(root.join("scripts/generated")).unwrap();
    fs::write(root.join(MAP_PATH), serde_json::to_vec(&record).unwrap()).unwrap();
}
fn changes(root: &Path, paths: &[&[u8]]) -> ChangeSet {
    let head = HeadState::Detached {
        commit: "original-head".into(),
    };
    ChangeSet {
        base: "HEAD".into(),
        base_commit: "original-head".into(),
        committed_paths: paths.iter().map(|path| GitName::new(path)).collect(),
        live: RepositoryObservation {
            worktree: GitName::new(root.as_os_str().as_bytes()),
            git_dir: GitName::new(b"/fixture/.git"),
            common_dir: GitName::new(b"/fixture/.git"),
            index_path: GitName::new(b"/fixture/.git/index"),
            head_before: head.clone(),
            head_after: head,
            changes: vec![],
            dirty_path_count: 0,
        },
    }
}
fn ids(preview: &Preview) -> Vec<&str> {
    preview
        .selected_packages
        .iter()
        .map(|package| package.id.as_str())
        .collect()
}

#[test]
fn reverse_declarations_include_optional_renamed_build_dev_and_native_integration_roots() {
    let root = tempfile::tempdir().unwrap();
    map(root.path(), None, "crates/consumer/src/generated");
    let metadata = metadata(root.path());
    let preview = preview(
        root.path(),
        &changes(root.path(), &[b"crates/base/src/deleted.rs"]),
        Some(&metadata),
    )
    .unwrap();
    assert_eq!(
        preview.changed_packages,
        ["opaque:pse-backend-native@1#source"]
    );
    assert_eq!(
        ids(&preview),
        [
            "opaque:consumer@1#source",
            "opaque:downstream@1#source",
            "opaque:pse-backend-native@1#source"
        ]
    );
    assert!(preview.widening.is_empty());
    assert!(preview.failures.is_empty());
    let request = selection::Request {
        operation: selection::Operation::Test,
        scope: preview
            .scope(selection::Features::default())
            .unwrap()
            .unwrap(),
        platform: None,
        filters: selection::Filters::default(),
        non_incremental: false,
        ignored: selection::IgnoredPolicy::Default,
    };
    let plan = selection::plan(&metadata, request).unwrap();
    for name in ["consumer", "downstream", "pse-backend-native"] {
        let roots: Vec<_> = plan
            .groups()
            .iter()
            .flat_map(|group| group.roots())
            .filter(|root| root.package_name() == name)
            .collect();
        assert!(roots.iter().any(|root| root.target().is_none()));
        assert!(roots.iter().any(|root| {
            root.target()
                .is_some_and(|target| target.name == "native_journey")
                && root.features().enabled.contains("native")
        }));
    }
    assert!(!root.path().join("target").exists());
    assert!(!root.path().join("build").exists());
}

#[test]
fn absent_optional_edge_keeps_conservative_reach_and_the_integration_helper() {
    let root = tempfile::tempdir().unwrap();
    map(root.path(), None, "crates/consumer/src/generated");
    let mut metadata = metadata(root.path());
    let consumer = &mut metadata.packages[1];
    let consumer_id = consumer.id.to_string();
    let dependency = &mut consumer.dependencies[0];
    assert!(dependency.optional);
    dependency.path = None;
    metadata.resolve = Some(
        serde_json::from_value(json!({
            "root": null,
            "nodes": [{"id": consumer_id, "dependencies": [], "deps": [], "features": []}]
        }))
        .unwrap(),
    );
    let preview = preview(
        root.path(),
        &changes(root.path(), &[b"crates/consumer/tests/support/helper.rs"]),
        Some(&metadata),
    )
    .unwrap();
    assert_eq!(preview.changed_packages, [consumer_id.clone()]);
    assert_eq!(
        preview.selected_packages.len(),
        metadata.workspace_members.len()
    );
    assert!(preview.widening.iter().any(|basis| {
        matches!(basis, Widening::UnknownDependency { package_id, dependency }
            if package_id == &consumer_id && dependency == "pse-backend-native")
    }));
    let scope = preview
        .scope(selection::Features::default())
        .unwrap()
        .unwrap();
    let selection::Scope::Packages(roots) = scope else {
        panic!("affected workspace scope lost its declared roots");
    };
    assert!(roots.iter().any(|root| {
        root.package == consumer_id
            && root
                .target
                .as_ref()
                .is_some_and(|target| target.name == "native_journey")
    }));
}
#[test]
fn nearest_member_component_boundaries_and_unknown_sources_widen_explicitly() {
    let root = tempfile::tempdir().unwrap();
    map(root.path(), None, "crates/consumer/src/generated");
    let metadata = metadata(root.path());
    let nested = preview(
        root.path(),
        &changes(root.path(), &[b"crates/base/nested/src/lib.rs"]),
        Some(&metadata),
    )
    .unwrap();
    assert_eq!(ids(&nested), ["opaque:nested@1#source"]);
    for path in [
        b"crates/base-other/src/lib.rs".as_slice(),
        b"docs/changed.md",
        b"tests/fixtures/scientific/changed.pse",
        b"scripts/shared-tool",
    ] {
        let wide = preview(root.path(), &changes(root.path(), &[path]), Some(&metadata)).unwrap();
        assert_eq!(wide.selected_packages.len(), 5);
        assert!(
            wide.widening
                .iter()
                .any(|reason| matches!(reason, Widening::UnknownSource { .. }))
        );
    }
    let mut root_package_metadata = metadata.clone();
    root_package_metadata.packages[3].manifest_path =
        root.path().join("Cargo.toml").to_str().unwrap().into();
    for path in [
        b"Cargo.toml".as_slice(),
        b"Cargo.lock",
        b"rust-toolchain.toml",
        b".cargo/config.toml",
        b".config/nextest.toml",
    ] {
        let wide = preview(
            root.path(),
            &changes(root.path(), &[path]),
            Some(&root_package_metadata),
        )
        .unwrap();
        assert_eq!(wide.selected_packages.len(), 5);
        assert!(
            wide.widening
                .iter()
                .any(|reason| matches!(reason, Widening::WorkspaceContext { .. }))
        );
    }
}

fn scientific_sources(root: &Path, shared_dependency: &str) {
    let shared = "4b32d03e21e647eaad63561b9e259be1";
    let packages = [
        ("shared", shared, String::new()),
        (
            "left",
            "21ca4b98ab8e4726bb4d96d986f3fdb4",
            format!(
                r#"{{ package_id = "{shared_dependency}", version_req = {{ operator = "exact", major = 1, minor = 0, patch = 0 }} }}"#
            ),
        ),
        (
            "right",
            "ed944a698ea344eb8b983d5a8aaee3c0",
            format!(
                r#"{{ package_id = "{shared}", version_req = {{ operator = "exact", major = 1, minor = 0, patch = 0 }} }}"#
            ),
        ),
    ];
    for (name, identity, dependencies) in packages {
        let package = root.join("packages/reference").join(name);
        fs::create_dir_all(package.join("models")).unwrap();
        fs::write(
            package.join("package.toml"),
            format!(
                "[package]\nname = \"fixture.{name}\"\npackage_id = \"{identity}\"\nversion = \"1.0.0\"\ndependencies = [{dependencies}]\n"
            ),
        )
        .unwrap();
        // These bytes are outside the passive reader's header budget and invalid
        // as model text. Reaching the consumer still requires no model-body read.
        fs::File::create(package.join("models/input.pse"))
            .unwrap()
            .set_len(2 << 20)
            .unwrap();
    }
    let catalog = root.join(scientific::CATALOG);
    fs::create_dir_all(catalog.parent().unwrap()).unwrap();
    fs::write(
        catalog,
        r#"
version = 1
package_inventory = "packages/reference"

[[families]]
name = "left"
roots = ["packages/reference/left"]
consumers = ["consumer"]
projection = "authentic-transport"

[[families]]
name = "right"
roots = ["packages/reference/right"]
consumers = ["unrelated"]
projection = "authentic-transport"
"#,
    )
    .unwrap();
}

#[test]
fn scientific_family_reach_seeds_reverse_cargo_scope_without_opening_model_bodies() {
    let root = tempfile::tempdir().unwrap();
    map(root.path(), None, "crates/consumer/src/generated");
    scientific_sources(root.path(), "4b32d03e21e647eaad63561b9e259be1");
    let metadata = metadata(root.path());
    let shared = preview(
        root.path(),
        &changes(root.path(), &[b"packages/reference/shared/models/input.pse"]),
        Some(&metadata),
    )
    .unwrap();
    assert_eq!(
        ids(&shared),
        [
            "opaque:consumer@1#source",
            "opaque:downstream@1#source",
            "opaque:unrelated@1#source"
        ]
    );
    assert_eq!(shared.changed_packages, ["opaque:consumer@1#source", "opaque:unrelated@1#source"]);
    assert_eq!(
        shared.scientific.iter().map(|association| association.family.as_str()).collect::<Vec<_>>(),
        ["left", "right"]
    );
    assert!(shared.widening.is_empty());

    let left = preview(
        root.path(),
        &changes(root.path(), &[b"packages/reference/left/models/input.pse"]),
        Some(&metadata),
    )
    .unwrap();
    assert_eq!(ids(&left), ["opaque:consumer@1#source", "opaque:downstream@1#source"]);
    assert_eq!(left.scientific[0].family, "left");
    assert!(left.widening.is_empty());

    let aliased = root.path().join("packages/reference/left/models/input.pse");
    fs::remove_file(&aliased).unwrap();
    std::os::unix::fs::symlink(
        root.path().join("packages/reference/right/models/input.pse"),
        &aliased,
    )
    .unwrap();
    let aliased = preview(
        root.path(),
        &changes(root.path(), &[b"packages/reference/left/models/input.pse"]),
        Some(&metadata),
    )
    .unwrap();
    assert_eq!(ids(&aliased).len(), metadata.workspace_members.len());
    assert!(aliased.scientific.is_empty());
    assert!(aliased.widening.iter().any(|basis| matches!(
        basis, Widening::ScientificAssociationUnavailable { reason } if reason.contains("alias")
    )));
    assert!(!root.path().join("target").exists());
    assert!(!root.path().join("build").exists());
}

#[test]
fn unresolved_scientific_dependency_and_changed_projection_each_widen() {
    let root = tempfile::tempdir().unwrap();
    map(root.path(), None, "crates/consumer/src/generated");
    scientific_sources(root.path(), "178bc2dc3edc4da79d876b290f2d11de");
    let metadata = metadata(root.path());
    let path = b"packages/reference/left/models/input.pse";
    let unknown = preview(root.path(), &changes(root.path(), &[path]), Some(&metadata)).unwrap();
    assert_eq!(ids(&unknown).len(), metadata.workspace_members.len());
    assert!(unknown.scientific.is_empty());
    assert!(unknown.widening.iter().any(|basis| matches!(
        basis, Widening::ScientificAssociationUnavailable { reason }
            if reason.contains("missing scientific dependency")
    )));
    let changed = preview(
        root.path(),
        &changes(root.path(), &[path, scientific::CATALOG.as_bytes()]),
        Some(&metadata),
    )
    .unwrap();
    assert_eq!(ids(&changed).len(), metadata.workspace_members.len());
    assert!(changed.scientific.is_empty());
    assert!(changed.widening.iter().any(|basis| matches!(
        basis, Widening::ScientificContext { path } if path.as_bytes() == scientific::CATALOG.as_bytes()
    )));
}
#[test]
fn generated_input_reach_comes_only_from_declared_map_scopes() {
    let root = tempfile::tempdir().unwrap();
    map(
        root.path(),
        Some("authored/new-generator"),
        "crates/consumer/src/generated",
    );
    let metadata = metadata(root.path());
    let selected = preview(
        root.path(),
        &changes(root.path(), &[b"authored/new-generator/model.rs"]),
        Some(&metadata),
    )
    .unwrap();
    assert_eq!(
        ids(&selected),
        ["opaque:consumer@1#source", "opaque:downstream@1#source"]
    );
    assert_eq!(selected.generators[0].producer, "independent-generator");
    assert_eq!(
        selected.generators[0].regenerate,
        ["just", "codegen", "rust"]
    );
    assert!(selected.widening.is_empty());
    map(
        root.path(),
        Some("authored/new-generator"),
        "docs/generated/new-view",
    );
    let unknown = preview(
        root.path(),
        &changes(root.path(), &[b"authored/new-generator/model.rs"]),
        Some(&metadata),
    )
    .unwrap();
    assert_eq!(unknown.selected_packages.len(), 5);
    assert!(
        unknown
            .widening
            .iter()
            .any(|reason| matches!(reason, Widening::UnknownGeneratedConsumer { .. }))
    );
}
#[test]
fn unavailable_map_and_graph_preserve_independent_errors_and_refuse_scope() {
    let root = tempfile::tempdir().unwrap();
    let metadata = metadata(root.path());
    let input = changes(root.path(), &[b"crates/base/src/lib.rs"]);
    let no_map = preview(root.path(), &input, Some(&metadata)).unwrap();
    assert_eq!(no_map.selected_packages.len(), 5);
    assert!(matches!(
        no_map.failures[0],
        AffectedError::Ownership(OwnershipError::Io(_))
    ));
    assert!(
        no_map
            .scope(selection::Features::default())
            .unwrap()
            .is_some()
    );
    let missing = preview(root.path(), &input, None).unwrap();
    assert_eq!(missing.failures.len(), 2);
    assert!(matches!(missing.graph, Graph::Unavailable { .. }));
    assert!(missing.scope(selection::Features::default()).is_err());
    let empty = preview(root.path(), &changes(root.path(), &[]), None).unwrap();
    assert!(matches!(empty.graph, Graph::NotNeeded));
    assert!(empty.failures.is_empty());
    assert!(
        empty
            .scope(selection::Features::default())
            .unwrap()
            .is_none()
    );
    for path in [
        b"../outside.rs".as_slice(),
        b"/absolute.rs",
        b"embedded\0nul",
        b"./unnormalized",
        b"",
    ] {
        assert!(
            preview(root.path(), &changes(root.path(), &[path]), None).is_err(),
            "{path:?}"
        );
    }
}
#[test]
fn non_unicode_source_identity_and_unknown_dependency_joins_are_not_erased() {
    let root = tempfile::tempdir().unwrap();
    map(root.path(), None, "crates/consumer/src/generated");
    let mut metadata = metadata(root.path());
    let selected = preview(
        root.path(),
        &changes(root.path(), &[b"crates/base/src/opaque-\xff.rs"]),
        Some(&metadata),
    )
    .unwrap();
    assert_eq!(
        selected.changes[0].as_bytes(),
        b"crates/base/src/opaque-\xff.rs"
    );
    assert_eq!(selected.selected_packages.len(), 3);
    metadata.packages[1].dependencies[0].path = None;
    let wide = preview(
        root.path(),
        &changes(root.path(), &[b"crates/base/src/lib.rs"]),
        Some(&metadata),
    )
    .unwrap();
    assert_eq!(wide.selected_packages.len(), 5);
    assert!(
        wide.widening
            .iter()
            .any(|reason| matches!(reason, Widening::UnknownDependency { .. }))
    );
}
fn git(root: &Path, args: &[&OsStr]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(root)
        .args([
            "-c",
            "user.name=affected fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "tag.gpgsign=false",
            "-c",
            "core.hooksPath=/dev/null",
        ])
        .args(args)
        .status()
        .unwrap();
    assert!(status.success(), "fixture Git command {args:?}");
}
fn words(root: &Path, args: &[&str]) {
    git(root, &args.iter().map(OsStr::new).collect::<Vec<_>>());
}
#[test]
fn native_base_head_live_union_preserves_renames_deletions_byte_paths_and_index() {
    let root = tempfile::tempdir().unwrap();
    words(
        root.path(),
        &["init", "--quiet", "--initial-branch", "main"],
    );
    for path in ["old.rs", "deleted.rs", "layered.rs"] {
        fs::write(root.path().join(path), "initial\n").unwrap();
    }
    fs::write(root.path().join(".gitignore"), "ignored\n").unwrap();
    words(root.path(), &["add", "."]);
    words(root.path(), &["commit", "--quiet", "-m", "base"]);
    words(root.path(), &["tag", "base"]);
    words(root.path(), &["mv", "old.rs", "new.rs"]);
    words(root.path(), &["commit", "--quiet", "-m", "rename"]);
    fs::remove_file(root.path().join("deleted.rs")).unwrap();
    fs::write(root.path().join("layered.rs"), "staged\n").unwrap();
    words(root.path(), &["add", "layered.rs"]);
    fs::write(root.path().join("layered.rs"), "unstaged\n").unwrap();
    fs::write(
        root.path()
            .join(OsStr::from_bytes(b"untracked-\xff\n\t.rs")),
        "new\n",
    )
    .unwrap();
    fs::write(root.path().join("ignored"), "ignored\n").unwrap();
    let before = fs::read(root.path().join(".git/index")).unwrap();
    let selected = observe(root.path(), "base", Arc::new(AtomicBool::new(false))).unwrap();
    let paths: Vec<_> = selected
        .paths()
        .iter()
        .map(|path| path.as_bytes().to_vec())
        .collect();
    for path in [
        b"old.rs".as_slice(),
        b"new.rs",
        b"deleted.rs",
        b"layered.rs",
        b"untracked-\xff\n\t.rs",
    ] {
        assert!(paths.iter().any(|actual| actual == path), "{path:?}");
    }
    assert!(!paths.iter().any(|path| path == b"ignored"));
    assert_eq!(fs::read(root.path().join(".git/index")).unwrap(), before);
    assert!(!root.path().join(".git/index.lock").exists());
    assert!(
        observe(
            root.path(),
            "missing-base",
            Arc::new(AtomicBool::new(false))
        )
        .is_err()
    );
    assert!(observe(root.path(), "--all", Arc::new(AtomicBool::new(false))).is_err());
    assert!(observe(root.path(), "HEAD", Arc::new(AtomicBool::new(true))).is_err());
}
