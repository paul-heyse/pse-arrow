// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;

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
    assert_eq!(construct(fixture.path(), &environment).unwrap(), 0);
    let prepared = prepared(fixture.path()).unwrap().unwrap();
    assert_eq!(prepared.protocol, 1);
    assert_eq!(prepared.manifest_digest.len(), 64);
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
    assert_ne!(construct(fixture.path(), &environment).unwrap(), 0);
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
    assert!(publish(fixture.path(), before.clone(), &after, || Ok(after.clone())).is_err());
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
    assert!(publish(fixture.path(), after.clone(), &after, || Ok(before)).is_err());
    assert!(!fixture.path().join("target/host-tools/current").exists());
}

#[test]
fn builder_environment_is_explicit_and_private_values_are_digest_only() {
    let inputs = Environment::from([
        ("HOME".into(), "/home/example".into()),
        ("SYMBOLICA_LICENSE".into(), "private-license".into()),
        ("PSE_UNDECLARED_BUILD_INPUT".into(), "ignored".into()),
        ("RUSTC_WRAPPER".into(), "/old/wrapper".into()),
    ]);
    let selected = build_environment(&inputs).unwrap();
    assert_eq!(selected.len(), 2);
    let value = context("rustc fixture".into(), "cargo fixture".into(), &selected);
    assert!(
        !String::from_utf8(encoded(&value).unwrap())
            .unwrap()
            .contains("private-license")
    );
    let override_target = Environment::from([("CARGO_BUILD_TARGET".into(), "other-target".into())]);
    assert!(build_environment(&override_target).is_err());
}
