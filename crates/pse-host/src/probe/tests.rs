// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use std::process::Stdio;

fn fixture() -> (tempfile::TempDir, PathBuf, PathBuf, MetadataSnapshot) {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path().join("checkout");
    let store = temporary.path().join("store");
    fs::create_dir(&root).unwrap();
    fs::write(
        root.join("rust-toolchain.toml"),
        "[toolchain]\nchannel = \"nightly-2026-09-29\"\n",
    )
    .unwrap();
    fs::write(root.join("Cargo.lock"), "version = 4\n").unwrap();
    let bytes = format!(r#"{{"packages":[],"workspace_members":[],"workspace_default_members":[],"resolve":{{"nodes":[],"root":null}},"target_directory":"{}","workspace_root":"{}","version":1,"metadata":null}}"#, root.join("target").display(),root.display()).into_bytes();
    let snapshot = MetadataSnapshot {
        metadata: serde_json::from_slice(&bytes).unwrap(),
        digest: digest(&bytes),
    };
    (temporary, root, store, snapshot)
}

#[test]
fn authored_workspace_copies_seed_and_toolchain_without_touching_origin() {
    let (_temporary, root, store, snapshot) = fixture();
    let before = fs::read(root.join("Cargo.lock")).unwrap();
    let probe = new(&root, &store, "authored", &[], &snapshot).unwrap();
    assert_eq!(
        fs::read(probe.directory.join("Cargo.lock")).unwrap(),
        before
    );
    assert_eq!(fs::read(root.join("Cargo.lock")).unwrap(), before);
    assert_eq!(
        fs::read(probe.directory.join("rust-toolchain.toml")).unwrap(),
        fs::read(root.join("rust-toolchain.toml")).unwrap()
    );
    assert_eq!(probe.target_directory, probe.directory.join("target"));
    let document: toml::Value =
        toml::from_str(&fs::read_to_string(probe.directory.join("Cargo.toml")).unwrap()).unwrap();
    assert!(document.get("workspace").is_some());
    fs::write(
        probe.directory.join("src/main.rs"),
        "fn main() { /* authored */ }\n",
    )
    .unwrap();
    assert!(new(&root, &store, "authored", &[], &snapshot).is_err());
    assert!(
        fs::read_to_string(probe.directory.join("src/main.rs"))
            .unwrap()
            .contains("authored")
    );
}

#[test]
fn disposal_refuses_original_lease_and_live_process_then_removes_only_named_probe() {
    let (_temporary, root, store, snapshot) = fixture();
    let probe = new(&root, &store, "leased", &[], &snapshot).unwrap();
    new(&root, &store, "other", &[], &snapshot).unwrap();
    let (_probe, lease) = open(&store, "leased").unwrap();
    assert!(rm(&store, "leased").is_err());
    drop(lease);
    let mut process = Command::new("sleep")
        .arg("60")
        .current_dir(&probe.directory)
        .stdin(Stdio::null())
        .spawn()
        .unwrap();
    assert!(rm(&store, "leased").is_err());
    process.kill().unwrap();
    process.wait().unwrap();
    assert_eq!(rm(&store, "leased").unwrap(), probe.directory);
    assert!(store.join("probes/other/src/main.rs").is_file());
}

#[test]
fn invalid_names_and_unknown_versions_cannot_select_other_paths() {
    let (_temporary, root, store, snapshot) = fixture();
    for name in ["", "..", "../escape", "a/b", "a.b", "é"] {
        assert!(new(&root, &store, name, &[], &snapshot).is_err());
    }
    let probe = new(&root, &store, "future", &[], &snapshot).unwrap();
    let mut manifest = probe.manifest;
    manifest.version = 2;
    fs::write(
        probe.directory.join("probe.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    assert!(rm(&store, "future").is_err());
    assert!(probe.directory.join("src/main.rs").is_file());
}

#[test]
fn disposal_refuses_external_open_file_even_when_process_cwd_is_elsewhere() {
    use std::io::BufRead as _;
    let (_temporary, root, store, snapshot) = fixture();
    let probe = new(&root, &store, "open-file", &[], &snapshot).unwrap();
    let mut process = Command::new("bash")
        .args([
            "-c",
            "exec 3< \"$1\"; printf 'ready\\n'; exec sleep 60",
            "probe-file-owner",
        ])
        .arg(probe.directory.join("src/main.rs"))
        .current_dir(&root)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut line = String::new();
    std::io::BufReader::new(process.stdout.take().unwrap())
        .read_line(&mut line)
        .unwrap();
    assert_eq!(line, "ready\n");
    assert!(rm(&store, "open-file").is_err());
    process.kill().unwrap();
    process.wait().unwrap();
    rm(&store, "open-file").unwrap();
}

#[test]
fn changed_copied_toolchain_refuses_before_resolution_and_retains_authored_source() {
    let (_temporary, root, store, snapshot) = fixture();
    let probe = new(&root, &store, "changed-pin", &[], &snapshot).unwrap();
    fs::write(
        probe.directory.join("rust-toolchain.toml"),
        "[toolchain]\nchannel = \"stable\"\n",
    )
    .unwrap();
    let error = prepare_run(&root, &store, "changed-pin", &Environment::new(), &[]).unwrap_err();
    assert!(error.to_string().contains("copied toolchain pin changed"));
    assert!(probe.directory.join("src/main.rs").is_file());
}
