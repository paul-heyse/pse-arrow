// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Real nested Cargo and Nextest with outputs owned by the test, not its outer runner.
use crate::{environment::Environment, generation::GenerationManifest, selection};
use std::{
    ffi::OsStr,
    fs,
    os::unix::fs::{PermissionsExt as _, symlink},
    path::{Path, PathBuf},
};

pub(super) struct NativeFixture {
    pub(super) scratch: tempfile::TempDir,
    pub(super) root: PathBuf,
    pub(super) environment: Environment,
}

impl NativeFixture {
    pub(super) fn new() -> Self {
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap();
        let scratch = tempfile::tempdir().unwrap();
        let root = scratch.path().canonicalize().unwrap().join("workspace");
        let tools = root.join("target/host-tools");
        let generation = tools.join("generation.native-workflow-control");
        for directory in ["src", "tests", "scripts", ".cargo", ".config"] {
            fs::create_dir_all(root.join(directory)).unwrap();
        }
        fs::create_dir_all(&generation).unwrap();
        for directory in [&tools, &generation] {
            fs::set_permissions(directory, fs::Permissions::from_mode(0o700)).unwrap();
        }
        gix::init(&root).unwrap();
        fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = 'native-workflow-control'\nversion = '0.0.0'\nedition = '2024'\n",
        )
        .unwrap();
        fs::write(
            root.join("Cargo.lock"),
            "version = 4\n\n[[package]]\nname = 'native-workflow-control'\nversion = '0.0.0'\n",
        )
        .unwrap();
        fs::write(
            root.join(".cargo/config.toml"),
            "[build]\ntarget-dir = 'artifact'\nbuild-dir = 'intermediate'\n",
        )
        .unwrap();
        fs::write(
            root.join("src/lib.rs"),
            r#"
#[test]
fn ordinary_case() {
    assert_eq!(6_u32.checked_div(2), Some(3));
}

#[test]
fn expected_refusal() {
    assert_eq!(6_u32.checked_div(0), None);
}

#[test]
fn rerun_controlled_failure_probe() {
    assert!(std::env::var_os("PSE_TEST_RERUN_FAILURE_PROBE").is_none(), "explicit native rerun qualification failure");
}
"#,
        )
        .unwrap();
        fs::write(
            root.join("tests/native.rs"),
            "#[test]\n#[ignore]\nfn deliberately_unselected() {}\n",
        )
        .unwrap();
        fs::write(
            root.join(".config/nextest.toml"),
            r#"
experimental = ["setup-scripts"]

[scripts.setup.restore-cargo]
command = { command-line = "scripts/nextest-test-env", relative-to = "workspace-root" }

[[profile.default.scripts]]
filter = "all()"
setup = "restore-cargo"

[profile.default.junit]
path = "junit.xml"
store-failure-output = true

[profile.local]
inherits = "default"

[profile.local.junit]
path = "junit.xml"
store-failure-output = true
"#,
        )
        .unwrap();
        for file in [
            "rust-toolchain.toml",
            ".config/diagnostic-tools.toml",
            "scripts/nextest-test-env",
        ] {
            fs::copy(source.join(file), root.join(file)).unwrap();
        }

        // Only the real, authenticated correctness adapter is needed in this workspace.
        // Import its original source fingerprint without building the host a second time.
        let prepared = crate::generation::prepared(source).unwrap().unwrap();
        let original = source.join("target/host-tools").join(&prepared.generation);
        let source_manifest = GenerationManifest::load(&original).unwrap();
        assert_eq!(source_manifest.digest().unwrap(), prepared.manifest_digest);
        let adapter = generation.join("cargo-pse-correctness");
        // A hard link would change the original image's ctime and interrupt other
        // invocation owners that are authenticating that same prepared generation.
        crate::discovery::capture::executable(&original.join("cargo-pse-correctness"), &adapter)
            .unwrap();
        let source = source_manifest.source();
        GenerationManifest::capture(
            &generation,
            source.clone(),
            source,
            &["cargo-pse-correctness"],
        )
        .unwrap()
        .write(&generation)
        .unwrap();
        symlink(generation.file_name().unwrap(), tools.join("current")).unwrap();

        let mut environment =
            crate::correctness::standalone_adapter_environment(&std::env::vars_os().collect())
                .unwrap();
        for key in [
            "CARGO_TARGET_DIR",
            "CARGO_BUILD_TARGET_DIR",
            "CARGO_BUILD_BUILD_DIR",
        ] {
            environment.remove(OsStr::new(key));
        }
        environment.insert(
            "XDG_STATE_HOME".into(),
            scratch.path().join("state").into_os_string(),
        );
        Self {
            scratch,
            root,
            environment,
        }
    }

    pub(super) fn scope(only_lib: bool) -> selection::Scope {
        selection::Scope::Packages(vec![selection::PackageIntent {
            package: "native-workflow-control".into(),
            target: only_lib.then(|| selection::TargetIntent {
                name: "native_workflow_control".into(),
                kind: Some("lib".into()),
            }),
            features: selection::Features {
                defaults: false,
                ..selection::Features::default()
            },
        }])
    }
}
