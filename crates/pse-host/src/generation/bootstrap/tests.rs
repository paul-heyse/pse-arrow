// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;

#[test]
fn host_construction_exclusion_does_not_block_passive_generation_protection() {
    let root = tempfile::tempdir().unwrap();
    let lock = owned_file(&root.path().join(".construction.lock"), true).unwrap();
    rustix::fs::flock(&lock, rustix::fs::FlockOperation::LockExclusive).unwrap();
    let passive = owned_file(&root.path().join(".update.lock"), true).unwrap();
    rustix::fs::flock(&passive, rustix::fs::FlockOperation::NonBlockingLockShared).unwrap();
    let publishing = owned_file(&root.path().join(".update.lock"), true).unwrap();
    assert_eq!(
        rustix::fs::flock(
            &publishing,
            rustix::fs::FlockOperation::NonBlockingLockExclusive
        ),
        Err(rustix::io::Errno::WOULDBLOCK)
    );
    drop(passive);
    rustix::fs::flock(
        &publishing,
        rustix::fs::FlockOperation::NonBlockingLockExclusive,
    )
    .unwrap();
}

fn checkout() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    for directory in [
        "host/src/bin",
        ".cargo",
        "target/host-tools",
        "crates/pse-study-module",
    ] {
        fs::create_dir_all(root.path().join(directory)).unwrap();
    }
    fs::write(
        root.path().join("Cargo.toml"),
        "[workspace]\nmembers=['host']\nresolver='3'\n",
    )
    .unwrap();
    fs::write(
        root.path().join("host/Cargo.toml"),
        "[package]\nname='pse-host'\nversion='0.0.1'\nedition='2024'\n",
    )
    .unwrap();
    fs::write(
        root.path().join(".cargo/config.toml"),
        "# explicit fixture\n",
    )
    .unwrap();
    fs::write(
        root.path().join("rust-toolchain.toml"),
        include_str!("../../../../../rust-toolchain.toml"),
    )
    .unwrap();
    for name in TOOLS {
        fs::write(
            root.path().join("host/src/bin").join(format!("{name}.rs")),
            "fn main() { println!(\"fixture protocol 1\"); }\n",
        )
        .unwrap();
    }
    for path in EXTERNAL_INPUTS {
        fs::write(root.path().join(path), "# embedded input\n").unwrap();
    }
    root
}

fn environment() -> Environment {
    // Cargo/nextest may set its own selected-toolchain marker; this fixture uses its
    // checked-in toolchain and never changes the process-global environment.
    let mut environment: Environment = std::env::vars_os().collect();
    environment.remove(OsStr::new("RUSTUP_TOOLCHAIN"));
    build_environment(&environment).unwrap()
}

#[test]
fn native_construction_publishes_authenticated_bytes_and_passive_staleness() {
    let fixture = checkout();
    let environment = environment();
    let lock = Command::new("cargo")
        .args(["generate-lockfile", "--offline"])
        .current_dir(fixture.path())
        .env_clear()
        .envs(&environment)
        .status()
        .unwrap();
    assert!(lock.success());
    let built = construct(
        fixture.path(),
        &environment,
        None,
        BootstrapMode::CopyOrBuild,
    )
    .unwrap();
    assert_eq!(built.exit_code, 0);
    assert_eq!(built.settlement, EntrySettlement::Unresolved);
    let prepared = prepared(fixture.path()).unwrap().unwrap();
    assert_eq!(prepared.protocol, 1);
    assert_eq!(prepared.manifest_digest.len(), 64);
    assert_eq!(
        prepared
            .executable(fixture.path(), HostTool::Host)
            .unwrap()
            .file_name()
            .unwrap(),
        "pse-host"
    );
    let mut forged = prepared.clone();
    forged.generation = "../arbitrary".into();
    assert!(forged.executable(fixture.path(), HostTool::Pse).is_err());
    forged = prepared.clone();
    forged.manifest_digest = "forged".into();
    assert!(forged.executable(fixture.path(), HostTool::Pse).is_err());
    forged = prepared.clone();
    forged.protocol = 999;
    assert!(forged.executable(fixture.path(), HostTool::Pse).is_err());
    assert_eq!(
        freshness(fixture.path(), &environment).unwrap(),
        Freshness::Current
    );
    fs::write(
        fixture.path().join(EXTERNAL_INPUTS[0]),
        "# changed embedded input\n",
    )
    .unwrap();
    assert_eq!(
        freshness(fixture.path(), &environment).unwrap(),
        Freshness::Stale
    );
    let prior = fs::read_link(fixture.path().join("target/host-tools/current")).unwrap();
    fs::write(
        fixture.path().join("host/src/bin/pse.rs"),
        "cannot compile this source\n",
    )
    .unwrap();
    assert_ne!(
        construct(
            fixture.path(),
            &environment,
            None,
            BootstrapMode::CopyOrBuild
        )
        .unwrap()
        .exit_code,
        0
    );
    assert_eq!(
        fs::read_link(fixture.path().join("target/host-tools/current")).unwrap(),
        prior
    );
    assert_eq!(self::prepared(fixture.path()).unwrap().unwrap(), prepared);
}

#[test]
fn construction_rejects_changes_before_and_after_staging_without_selector_publication() {
    let fixture = checkout();
    let environment = environment();
    assert!(
        Command::new("cargo")
            .args(["generate-lockfile", "--offline"])
            .current_dir(fixture.path())
            .env_clear()
            .envs(&environment)
            .status()
            .unwrap()
            .success()
    );
    let context = context("rustc fixture".into(), "cargo fixture".into(), &environment);
    let before = capture(fixture.path(), &environment, context.clone()).unwrap();
    fs::write(
        fixture.path().join(EXTERNAL_INPUTS[0]),
        "# source changed\n",
    )
    .unwrap();
    let after = capture(fixture.path(), &environment, context.clone()).unwrap();
    assert!(
        publish(
            fixture.path(),
            &fixture.path().join("target/debug"),
            None,
            before.clone(),
            &after,
            || Ok(after.clone())
        )
        .is_err()
    );
    assert!(!fixture.path().join("target/host-tools/current").exists());
    assert!(
        Command::new("cargo")
            .args(["build", "--locked", "--package", "pse-host", "--bins"])
            .current_dir(fixture.path())
            .env_clear()
            .envs(&environment)
            .status()
            .unwrap()
            .success()
    );
    assert!(
        publish(
            fixture.path(),
            &fixture.path().join("target/debug"),
            None,
            after.clone(),
            &after,
            || Ok(before)
        )
        .is_err()
    );
    assert!(!fixture.path().join("target/host-tools/current").exists());
}

#[test]
fn builder_environment_is_explicit_and_private_values_are_digest_only() {
    let inputs = Environment::from([
        ("HOME".into(), "/home/example".into()),
        ("SYMBOLICA_LICENSE".into(), "private-license".into()),
        ("PSE_UNDECLARED_BUILD_INPUT".into(), "ignored".into()),
        ("RUSTC_WRAPPER".into(), "/old/wrapper".into()),
        ("SCCACHE_DIR".into(), "/shared/compiler-cache".into()),
    ]);
    let selected = build_environment(&inputs).unwrap();
    assert_eq!(selected.len(), 3);
    assert_eq!(
        selected.get(OsStr::new("SCCACHE_DIR")),
        Some(&OsString::from("/shared/compiler-cache"))
    );
    let value = context("rustc fixture".into(), "cargo fixture".into(), &selected);
    assert!(
        !String::from_utf8(encoded(&value).unwrap())
            .unwrap()
            .contains("private-license")
    );
    let override_target = Environment::from([("CARGO_BUILD_TARGET".into(), "other-target".into())]);
    assert!(build_environment(&override_target).is_err());
}

#[test]
fn effective_config_includes_are_observed_without_making_equivalent_checkouts_differ() {
    let first = checkout();
    let second = checkout();
    let home = tempfile::tempdir().unwrap();
    fs::write(
        home.path().join("config.toml"),
        "include=[{path='optional.toml',optional=true}]\n",
    )
    .unwrap();
    let environment = Environment::from([("CARGO_HOME".into(), home.path().as_os_str().into())]);
    let observe = |root: &Path| {
        observed_context(
            root,
            "rustc fixture".into(),
            "cargo fixture".into(),
            &environment,
        )
        .unwrap()
    };
    let original = observe(first.path());
    assert_eq!(original, observe(second.path()));
    fs::write(home.path().join("optional.toml"), "[build]\njobs=32\n").unwrap();
    assert_ne!(original, observe(first.path()));
    fs::write(home.path().join("config"), "[build]\njobs=32\n").unwrap();
    let legacy_selected = observe(first.path());
    fs::write(home.path().join("config.toml"), "[build]\njobs=1\n").unwrap();
    assert_eq!(legacy_selected, observe(first.path()));
    fs::write(
        first.path().join(".cargo/config.toml"),
        "include=['extra.toml']\n",
    )
    .unwrap();
    fs::write(first.path().join(".cargo/extra.toml"), "[build]\njobs=32\n").unwrap();
    let before = observe(first.path());
    fs::write(first.path().join(".cargo/extra.toml"), "[build]\njobs=31\n").unwrap();
    assert_ne!(before, observe(first.path()));
}

fn lock_fixture(root: &Path, environment: &Environment) {
    assert!(
        Command::new("cargo")
            .args(["generate-lockfile", "--offline"])
            .current_dir(root)
            .env_clear()
            .envs(environment)
            .status()
            .unwrap()
            .success()
    );
}

#[test]
fn matching_copy_has_exact_bytes_without_destination_build_and_releases_source_use() {
    let source = checkout();
    let destination = checkout();
    let environment = environment();
    lock_fixture(source.path(), &environment);
    lock_fixture(destination.path(), &environment);
    fs::remove_dir_all(destination.path().join("target")).unwrap();
    assert_eq!(
        construct(
            source.path(),
            &environment,
            None,
            BootstrapMode::CopyOrBuild
        )
        .unwrap()
        .exit_code,
        0
    );
    let source_reference = prepared(source.path()).unwrap().unwrap();
    let outcome = construct(
        destination.path(),
        &environment,
        Some(source.path()),
        BootstrapMode::CopyOnly,
    )
    .unwrap();
    assert_eq!(outcome.exit_code, 0);
    assert_eq!(outcome.settlement, EntrySettlement::ResourcesSettled);
    assert!(
        !destination.path().join("target/debug").exists(),
        "matching copy must not invoke the compiler"
    );
    let copied = prepared(destination.path()).unwrap().unwrap();
    assert_eq!(copied.manifest_digest, source_reference.manifest_digest);
    for tool in TOOLS {
        assert_eq!(
            hash_file(
                &source
                    .path()
                    .join("target/host-tools")
                    .join(&source_reference.generation)
                    .join(tool)
            )
            .unwrap(),
            hash_file(
                &destination
                    .path()
                    .join("target/host-tools")
                    .join(&copied.generation)
                    .join(tool)
            )
            .unwrap()
        );
    }
    assert_eq!(
        GenerationUse::inspect(
            &source.path().join("target/host-tools"),
            &source_reference.generation
        )
        .unwrap(),
        GenerationUseState::Available
    );
    assert_eq!(
        freshness(destination.path(), &environment).unwrap(),
        Freshness::Current
    );
}

#[test]
fn mismatch_builds_destination_and_build_failure_preserves_its_previous_selector() {
    let source = checkout();
    let destination = checkout();
    let environment = environment();
    lock_fixture(source.path(), &environment);
    lock_fixture(destination.path(), &environment);
    assert_eq!(
        construct(
            source.path(),
            &environment,
            None,
            BootstrapMode::CopyOrBuild
        )
        .unwrap()
        .exit_code,
        0
    );
    fs::write(
        destination.path().join("host/src/bin/pse.rs"),
        "fn main() { println!(\"destination source\"); }\n",
    )
    .unwrap();
    let outcome = construct(
        destination.path(),
        &environment,
        Some(source.path()),
        BootstrapMode::CopyOrBuild,
    )
    .unwrap();
    assert_eq!(outcome.exit_code, 0);
    assert_eq!(outcome.settlement, EntrySettlement::Unresolved);
    assert!(destination.path().join("target/debug/pse").is_file());
    let prior = prepared(destination.path()).unwrap().unwrap();
    let original = prepared(source.path()).unwrap().unwrap();
    assert_ne!(
        hash_file(&prior.executable(destination.path(), HostTool::Pse).unwrap()).unwrap(),
        hash_file(&original.executable(source.path(), HostTool::Pse).unwrap()).unwrap()
    );
    fs::write(
        destination.path().join("host/src/bin/pse.rs"),
        "broken destination source\n",
    )
    .unwrap();
    assert_ne!(
        construct(
            destination.path(),
            &environment,
            Some(source.path()),
            BootstrapMode::CopyOrBuild
        )
        .unwrap()
        .exit_code,
        0
    );
    assert_eq!(prepared(destination.path()).unwrap().unwrap(), prior);
    assert_eq!(
        GenerationUse::inspect(
            &source.path().join("target/host-tools"),
            &original.generation
        )
        .unwrap(),
        GenerationUseState::Available
    );
}

#[test]
fn copy_owner_keeps_source_pinned_and_rejects_changed_bytes_before_probe_or_publication() {
    let source = checkout();
    let destination = checkout();
    let environment = environment();
    lock_fixture(source.path(), &environment);
    lock_fixture(destination.path(), &environment);
    assert_eq!(
        construct(
            source.path(),
            &environment,
            None,
            BootstrapMode::CopyOrBuild
        )
        .unwrap()
        .exit_code,
        0
    );
    assert_eq!(
        construct(
            destination.path(),
            &environment,
            None,
            BootstrapMode::CopyOrBuild
        )
        .unwrap()
        .exit_code,
        0
    );
    let prior = prepared(destination.path()).unwrap().unwrap();
    let copy = CopySource::acquire(source.path()).unwrap();
    let generation = copy.directory.file_name().unwrap().to_str().unwrap();
    assert_eq!(
        GenerationUse::inspect(&source.path().join("target/host-tools"), generation).unwrap(),
        GenerationUseState::Active
    );
    use std::os::unix::fs::PermissionsExt;
    let artifact = copy.directory.join("pse");
    fs::set_permissions(&artifact, fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(
        &artifact,
        b"corrupt executable bytes which must never reach the probe",
    )
    .unwrap();
    let error = construct_selected(
        destination.path(),
        &environment,
        Some(&copy),
        BootstrapMode::CopyOrBuild,
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("copied binary differs from the pinned source manifest"),
        "corruption must fail before attempting execution: {error}"
    );
    assert_eq!(prepared(destination.path()).unwrap().unwrap(), prior);
    assert_eq!(
        GenerationUse::inspect(&source.path().join("target/host-tools"), generation).unwrap(),
        GenerationUseState::Active
    );
    copy.usage.settle().unwrap();
    assert_eq!(
        GenerationUse::inspect(&source.path().join("target/host-tools"), generation).unwrap(),
        GenerationUseState::Available
    );
}

#[test]
fn source_helper_mismatch_requests_fresh_local_seed_without_compiling_destination() {
    let source = checkout();
    let destination = checkout();
    let environment = environment();
    lock_fixture(source.path(), &environment);
    lock_fixture(destination.path(), &environment);
    assert_eq!(
        construct(
            source.path(),
            &environment,
            None,
            BootstrapMode::CopyOrBuild
        )
        .unwrap()
        .exit_code,
        0
    );
    fs::write(
        destination.path().join("host/src/bin/pse.rs"),
        "new target code the old helper must not build\n",
    )
    .unwrap();
    let outcome = construct(
        destination.path(),
        &environment,
        Some(source.path()),
        BootstrapMode::CopyOnly,
    )
    .unwrap();
    assert_eq!(outcome.exit_code, LOCAL_SEED_REQUIRED);
    assert_eq!(outcome.settlement, EntrySettlement::ResourcesSettled);
    assert!(!destination.path().join("target/debug").exists());
    assert!(prepared(destination.path()).unwrap().is_none());
    let reference = prepared(source.path()).unwrap().unwrap();
    assert_eq!(
        GenerationUse::inspect(
            &source.path().join("target/host-tools"),
            &reference.generation
        )
        .unwrap(),
        GenerationUseState::Available
    );
}

#[test]
fn source_helper_assertion_requires_actual_prepared_checkout_identity() {
    let (_fixture, tools) = crate::generation::tests::installed_fixture();
    let root = tools.parent().unwrap().parent().unwrap();
    let usage = GenerationUse::acquire(&tools, "generation.fixture").unwrap();
    let reference = PreparedGeneration {
        generation: "generation.fixture".into(),
        protocol: usage.manifest().control_protocol(),
        manifest_digest: usage.manifest().digest().unwrap(),
    };
    let entry = EntryGuard {
        usage: Some(usage),
        reference: Some(reference),
    };
    assert_eq!(
        source_mode(&entry, Some(root), Some(root.as_os_str())).unwrap(),
        BootstrapMode::CopyOnly
    );
    assert_eq!(
        source_mode(&entry, Some(root), None).unwrap(),
        BootstrapMode::CopyOnly
    );
    assert!(source_mode(&entry, None, Some(root.as_os_str())).is_err());
    assert!(source_mode(&entry, Some(root), Some(tools.as_os_str())).is_err());
    let seed = EntryGuard {
        usage: None,
        reference: None,
    };
    assert!(source_mode(&seed, Some(root), Some(root.as_os_str())).is_err());
    assert_eq!(
        source_mode(&seed, Some(root), None).unwrap(),
        BootstrapMode::CopyOrBuild
    );
    entry.complete(EntrySettlement::NoEffects).unwrap();
}
