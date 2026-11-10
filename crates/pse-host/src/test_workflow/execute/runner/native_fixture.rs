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

    pub(super) fn with_rerun_helper(self) -> Self {
        use std::io::Write as _;

        writeln!(
            fs::OpenOptions::new()
                .append(true)
                .open(self.root.join("Cargo.toml"))
                .unwrap(),
            "\n[[bin]]\nname = 'rerun-helper'\npath = 'src/helper.rs'\ntest = false"
        )
        .unwrap();
        fs::write(
            self.root.join("src/helper.rs"),
            r#"fn main() { println!("ordinary helper"); }"#,
        )
        .unwrap();
        fs::write(
            self.root.join("tests/rerun.rs"),
            r#"
#[test]
fn rerun_controlled_helper_failure_probe() {
    let helper = std::env::var_os("CARGO_BIN_EXE_rerun-helper").expect("captured helper");
    let output = std::process::Command::new(helper).output().unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout, b"ordinary helper\n");
    assert!(std::env::var_os("PSE_TEST_RERUN_FAILURE_PROBE").is_none(), "explicit native rerun qualification failure");
}
"#,
        )
        .unwrap();
        self
    }

    pub(super) fn with_weighted_wakeup(mut self) -> Self {
        use std::io::Write as _;

        let rendezvous = self.scratch.path().join("weighted-wakeup");
        fs::create_dir(&rendezvous).unwrap();
        self.environment.insert(
            "PSE_TEST_WEIGHTED_RENDEZVOUS".into(),
            rendezvous.into_os_string(),
        );
        // Make the tiny runner fixture independent of the developer's ordinary lane count.
        self.environment
            .insert("NEXTEST_TEST_THREADS".into(), "4".into());
        fs::OpenOptions::new()
            .append(true)
            .open(self.root.join(".config/nextest.toml"))
            .unwrap()
            .write_all(
                br#"
[test-groups.weighted]
max-threads = 1

[[profile.local.overrides]]
filter = "test(=weighted_light)"
test-group = "weighted"
priority = 100

[[profile.local.overrides]]
filter = "test(=weighted_waiter)"
test-group = "weighted"
threads-required = 3
priority = 90

[[profile.local.overrides]]
filter = "test(=weighted_outside)"
threads-required = 3
priority = 80

[[profile.local.overrides]]
filter = "test(=weighted_witness)"
priority = 70

"#,
            )
            .unwrap();
        fs::write(
            self.root.join("src/lib.rs"),
            r#"
#![cfg(test)]

use std::{
    fs,
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};

fn marker(name: &str) -> PathBuf {
    PathBuf::from(std::env::var_os("PSE_TEST_WEIGHTED_RENDEZVOUS").unwrap()).join(name)
}

fn publish(name: &str) {
    fs::write(marker(name), []).unwrap();
}

fn await_marker(name: &str) {
    let path = marker(name);
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match path.try_exists() {
            Ok(true) => return,
            Ok(false) if Instant::now() < deadline => thread::sleep(Duration::from_millis(5)),
            Ok(false) => panic!("weighted runner did not reach {name}"),
            Err(error) => panic!("cannot read {name}: {error}"),
        }
    }
}

#[test]
fn weighted_light() {
    publish("light-started");
    await_marker("outside-started");
    publish("light-returning");
}

#[test]
fn weighted_waiter() {
    // The witness still holds one slot, so the three-slot waiter can only
    // start once the three-slot outside test has actually returned.
    assert!(marker("outside-returning").try_exists().unwrap());
    assert!(marker("witness-started").try_exists().unwrap());
    publish("waiter-started");
}

#[test]
fn weighted_outside() {
    await_marker("light-started");
    publish("outside-started");
    await_marker("witness-started");
    publish("outside-returning");
}

#[test]
fn weighted_witness() {
    // All four slots were occupied until the light test returned.
    assert!(marker("light-returning").try_exists().unwrap());
    publish("witness-started");
    await_marker("waiter-started");
}
"#,
        )
        .unwrap();
        self
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
