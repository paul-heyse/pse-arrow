// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;

struct Fixture {
    _directory: tempfile::TempDir,
    root: PathBuf,
    tools: PathBuf,
    metadata: Metadata,
    context: BuildContext,
}
impl Fixture {
    fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("checkout");
        fs::create_dir(&root).unwrap();
        fs::create_dir(root.join(".cargo")).unwrap();
        fs::write(root.join(".cargo/config.toml"), "# fixture\n").unwrap();
        fs::write(
            root.join("rust-toolchain.toml"),
            include_str!("../../../../rust-toolchain.toml"),
        )
        .unwrap();
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers=['host','support','unrelated']\nresolver='3'\n",
        )
        .unwrap();
        for (path, name) in [
            ("host", "pse-host"),
            ("support", "pse-support"),
            ("unrelated", "pse-unrelated"),
        ] {
            fs::create_dir_all(root.join(path).join("src")).unwrap();
            let dependencies = if path == "host" {
                "[dependencies]\npse-support={path='../support'}\n[dev-dependencies]\npse-unrelated={path='../unrelated'}\n"
            } else {
                ""
            };
            fs::write(
                root.join(path).join("Cargo.toml"),
                format!(
                    "[package]\nname='{name}'\nversion='0.0.1'\nedition='2024'\n{dependencies}"
                ),
            )
            .unwrap();
            fs::write(
                root.join(path).join("src/lib.rs"),
                "pub fn value() -> u8 { 1 }\n",
            )
            .unwrap();
        }
        let metadata = MetadataCommand::new()
            .current_dir(&root)
            .other_options(vec!["--offline".into()])
            .exec()
            .unwrap();
        let tools = directory.path().join("tools");
        private_directory(&tools, true).unwrap();
        let context = BuildContext::new(
            "rustc fixture".into(),
            "cargo fixture".into(),
            "dev".into(),
            BTreeSet::new(),
            &Environment::new(),
        );
        Self {
            _directory: directory,
            root,
            tools,
            metadata,
            context,
        }
    }
    fn fingerprint(&self) -> SourceFingerprint {
        SourceFingerprint::from_metadata(&self.root, &self.metadata, self.context.clone()).unwrap()
    }
    fn generation(&self) -> PathBuf {
        let generation = self.tools.join("generation.fixture");
        private_directory(&generation, true).unwrap();
        fs::write(generation.join("pse-host"), b"prepared executable bytes").unwrap();
        let before = self.fingerprint();
        let manifest =
            GenerationManifest::capture(&generation, before.clone(), &before, &["pse-host"])
                .unwrap();
        manifest.write(&generation).unwrap();
        generation
    }
}

pub(super) fn installed_fixture() -> (tempfile::TempDir, PathBuf) {
    let fixture = Fixture::new();
    fixture.generation();
    fs::create_dir(fixture.root.join("target")).unwrap();
    let tools = fixture.root.join("target/host-tools");
    fs::rename(&fixture.tools, &tools).unwrap();
    (fixture._directory, tools)
}

pub(super) fn transfer_fixture() -> (tempfile::TempDir, PathBuf) {
    transfer_fixture_with_input(false)
}

pub(super) fn transfer_fixture_with_input(future: bool) -> (tempfile::TempDir, PathBuf) {
    let fixture = Fixture::new();
    let source = fixture.fingerprint();
    for name in ["generation.source", "generation.target"] {
        let mut source = source.clone();
        if future && name == "generation.target" {
            source.context.environment.insert(
                "future_build_input".into(),
                Some(digest(b"new-policy-input")),
            );
        }
        let directory = fixture.tools.join(name);
        private_directory(&directory, true).unwrap();
        fs::copy(std::env::current_exe().unwrap(), directory.join("pse-host")).unwrap();
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(
            directory.join("pse-host"),
            fs::Permissions::from_mode(0o500),
        )
        .unwrap();
        GenerationManifest::capture(&directory, source.clone(), &source, &["pse-host"])
            .unwrap()
            .write(&directory)
            .unwrap();
    }
    fs::create_dir(fixture.root.join("target")).unwrap();
    let tools = fixture.root.join("target/host-tools");
    fs::rename(&fixture.tools, &tools).unwrap();
    (fixture._directory, tools)
}

#[test]
fn selected_product_source_uses_the_same_closure_owner_without_widening_host_membership() {
    let fixture = Fixture::new();
    let source = SourceFingerprint::from_metadata_for(
        &fixture.root,
        &fixture.metadata,
        fixture.context.clone(),
        "pse-support",
    )
    .unwrap();
    assert_eq!(
        source.package_roots,
        BTreeSet::from([PathBuf::from("support")])
    );
    fs::write(fixture.root.join("host/src/lib.rs"), "unselected host edit").unwrap();
    assert!(
        source
            .matches_sources(&fixture.root, &fixture.context)
            .unwrap()
    );
    fs::write(
        fixture.root.join("support/src/lib.rs"),
        "selected product edit",
    )
    .unwrap();
    assert!(
        !source
            .matches_sources(&fixture.root, &fixture.context)
            .unwrap()
    );
    assert!(
        SourceFingerprint::from_metadata_for(
            &fixture.root,
            &fixture.metadata,
            fixture.context.clone(),
            "missing-product"
        )
        .is_err()
    );
}

#[test]
fn local_closure_membership_and_content_invalidate_without_unrelated_crate_scan() {
    let fixture = Fixture::new();
    let fingerprint = fixture.fingerprint();
    assert_eq!(
        fingerprint.package_roots,
        BTreeSet::from([PathBuf::from("host"), PathBuf::from("support")])
    );
    fs::write(fixture.root.join("unrelated/src/lib.rs"), "unrelated edit").unwrap();
    assert!(
        fingerprint
            .matches_sources(&fixture.root, &fixture.context)
            .unwrap()
    );
    fs::write(fixture.root.join("support/src/added.rs"), "new member").unwrap();
    assert!(
        !fingerprint
            .matches_sources(&fixture.root, &fixture.context)
            .unwrap()
    );
    fs::remove_file(fixture.root.join("support/src/added.rs")).unwrap();
    assert!(
        fingerprint
            .matches_sources(&fixture.root, &fixture.context)
            .unwrap()
    );
    fs::write(fixture.root.join("host/src/lib.rs"), "changed source").unwrap();
    assert!(
        !fingerprint
            .matches_sources(&fixture.root, &fixture.context)
            .unwrap()
    );
}

#[test]
fn construction_rejects_changed_source_context_and_binary_bytes() {
    let fixture = Fixture::new();
    let generation = fixture.generation();
    let original = GenerationManifest::load(&generation).unwrap();
    let mut changed = original.source.clone();
    changed.context.profile = "different".into();
    assert!(original.source.require_unchanged(&changed).is_err());
    assert!(
        GenerationManifest::capture(
            &generation,
            original.source.clone(),
            &changed,
            &["pse-host"]
        )
        .is_err()
    );
    assert!(original.write(&generation).is_err());
    fs::write(generation.join("pse-host"), b"substituted executable").unwrap();
    assert!(GenerationManifest::load(&generation).is_err());
}

#[test]
fn completed_read_only_generations_can_be_authenticated() {
    use std::os::unix::fs::PermissionsExt;
    let fixture = Fixture::new();
    let generation = fixture.generation();
    fs::set_permissions(generation.join(MANIFEST), fs::Permissions::from_mode(0o400)).unwrap();
    fs::set_permissions(
        generation.join("pse-host"),
        fs::Permissions::from_mode(0o500),
    )
    .unwrap();
    fs::set_permissions(&generation, fs::Permissions::from_mode(0o500)).unwrap();
    assert!(GenerationManifest::load(&generation).is_ok());
    let owner = GenerationUse::acquire(&fixture.tools, "generation.fixture").unwrap();
    owner.settle().unwrap();
    // Restore access for this test fixture's ordinary TempDir cleanup.
    fs::set_permissions(&generation, fs::Permissions::from_mode(0o700)).unwrap();
}

#[test]
fn fingerprint_is_checkout_relative_and_roundtrips_with_recorded_inputs() {
    let fixture = Fixture::new();
    let original = fixture.fingerprint();
    let decoded: SourceFingerprint = serde_json::from_slice(&encoded(&original).unwrap()).unwrap();
    assert_eq!(original, decoded);
    assert_eq!(original.digest().unwrap(), decoded.digest().unwrap());
    let mut context = fixture.context.clone();
    context.rustc = "another rustc".into();
    assert!(!original.matches_sources(&fixture.root, &context).unwrap());
    assert!(!original.files.keys().any(|path| path.is_absolute()));
    fs::remove_file(fixture.root.join("Cargo.lock")).unwrap();
    assert!(
        !original
            .matches_sources(&fixture.root, &fixture.context)
            .unwrap()
    );
}

#[test]
fn explicit_discovery_matches_the_captured_metadata_graph() {
    let fixture = Fixture::new();
    let environment = std::env::vars_os().collect();
    let discovered =
        SourceFingerprint::discover(&fixture.root, &environment, fixture.context.clone()).unwrap();
    assert_eq!(fixture.fingerprint(), discovered);
    let mut changed = fixture.metadata.clone();
    for dependency in &mut changed
        .resolve
        .as_mut()
        .unwrap()
        .nodes
        .iter_mut()
        .find(|node| {
            node.id
                == changed
                    .packages
                    .iter()
                    .find(|package| package.name == "pse-host")
                    .unwrap()
                    .id
        })
        .unwrap()
        .deps
    {
        for kind in &mut dependency.dep_kinds {
            kind.kind = DependencyKind::Normal;
        }
    }
    let after =
        SourceFingerprint::from_metadata(&fixture.root, &changed, fixture.context.clone()).unwrap();
    assert!(discovered.require_unchanged(&after).is_err());
}

#[test]
fn additional_actual_builder_inputs_are_digest_only_and_invalidate_reuse() {
    let fixture = Fixture::new();
    let inputs = BTreeMap::from([(
        "PROVIDER_SELECTION".into(),
        Some(OsString::from("private-value")),
    )]);
    let context = fixture.context.clone().with_additional_inputs(&inputs);
    let fingerprint =
        SourceFingerprint::from_metadata(&fixture.root, &fixture.metadata, context.clone())
            .unwrap();
    assert!(
        !String::from_utf8(encoded(&fingerprint).unwrap())
            .unwrap()
            .contains("private-value")
    );
    assert!(
        fingerprint
            .matches_sources(&fixture.root, &context)
            .unwrap()
    );
    assert!(
        !fingerprint
            .matches_sources(&fixture.root, &fixture.context)
            .unwrap()
    );
}

#[test]
fn inspection_does_not_create_use_state() {
    let fixture = Fixture::new();
    fixture.generation();
    drop(update_lock(&fixture.tools).unwrap());
    assert_eq!(
        GenerationUse::inspect(&fixture.tools, "generation.fixture").unwrap(),
        GenerationUseState::Available
    );
    assert!(!fixture.tools.join(".uses").exists());
}

#[test]
fn declared_compile_time_inputs_outside_package_roots_are_rechecked() {
    let fixture = Fixture::new();
    let path = PathBuf::from("embedded.toml");
    fs::write(fixture.root.join(&path), "embedded input").unwrap();
    let fingerprint = fixture
        .fingerprint()
        .with_additional_files(&fixture.root, [path.clone()])
        .unwrap();
    assert!(
        fingerprint
            .matches_sources(&fixture.root, &fixture.context)
            .unwrap()
    );
    fs::write(fixture.root.join(&path), "changed input").unwrap();
    assert!(
        !fingerprint
            .matches_sources(&fixture.root, &fixture.context)
            .unwrap()
    );
    assert!(
        fixture
            .fingerprint()
            .with_additional_files(&fixture.root, [PathBuf::from("../outside")])
            .is_err()
    );
}

#[test]
fn supplemental_inputs_cannot_traverse_a_symlink_directory() {
    let fixture = Fixture::new();
    std::os::unix::fs::symlink(fixture.root.join("support"), fixture.root.join("alias")).unwrap();
    assert!(
        fixture
            .fingerprint()
            .with_additional_files(&fixture.root, [PathBuf::from("alias/src/lib.rs")])
            .is_err()
    );
    fixture.generation();
    drop(update_lock(&fixture.tools).unwrap());
    std::os::unix::fs::symlink("missing", fixture.tools.join(".uses")).unwrap();
    assert!(GenerationUse::inspect(&fixture.tools, "generation.fixture").is_err());
}

#[test]
fn kernel_users_and_abandoned_claims_have_distinct_protection() {
    let fixture = Fixture::new();
    fixture.generation();
    let first = GenerationUse::acquire(&fixture.tools, "generation.fixture").unwrap();
    let second = GenerationUse::acquire(&fixture.tools, "generation.fixture").unwrap();
    assert_eq!(
        GenerationUse::inspect(&fixture.tools, "generation.fixture").unwrap(),
        GenerationUseState::Active
    );
    first.settle().unwrap();
    assert_eq!(
        GenerationUse::inspect(&fixture.tools, "generation.fixture").unwrap(),
        GenerationUseState::Active
    );
    second.settle().unwrap();
    assert_eq!(
        GenerationUse::inspect(&fixture.tools, "generation.fixture").unwrap(),
        GenerationUseState::Available
    );
    let abandoned = GenerationUse::acquire(&fixture.tools, "generation.fixture").unwrap();
    drop(abandoned);
    assert_eq!(
        GenerationUse::inspect(&fixture.tools, "generation.fixture").unwrap(),
        GenerationUseState::Unsettled
    );
    // Absent controller locks do not clear the durable claim or delete the generation.
    assert!(fixture.tools.join("generation.fixture/pse-host").exists());
}

#[test]
fn concurrent_first_users_share_private_directories_without_creation_races() {
    let fixture = Fixture::new();
    fixture.generation();
    let barrier = std::sync::Barrier::new(16);
    std::thread::scope(|scope| {
        let users: Vec<_> = (0..16)
            .map(|_| {
                scope.spawn(|| {
                    barrier.wait();
                    GenerationUse::acquire(&fixture.tools, "generation.fixture").unwrap()
                })
            })
            .collect();
        for user in users {
            user.join().unwrap().settle().unwrap();
        }
    });
    assert_eq!(
        GenerationUse::inspect(&fixture.tools, "generation.fixture").unwrap(),
        GenerationUseState::Available
    );
}

#[test]
fn traversal_aliases_and_unprepared_generations_cannot_mint_use_claims() {
    let fixture = Fixture::new();
    let generation = fixture.generation();
    assert!(GenerationUse::acquire(&fixture.tools, "../generation.fixture").is_err());
    let alias = fixture.tools.join("alias");
    std::os::unix::fs::symlink(&generation, &alias).unwrap();
    assert!(GenerationUse::acquire(&fixture.tools, "alias").is_err());
    assert!(GenerationUse::acquire(&fixture.tools, "missing").is_err());
}
