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

#[tokio::test]
async fn disposal_refuses_original_lease_and_live_process_then_removes_only_named_probe() {
    let (_temporary, root, store, snapshot) = fixture();
    let probe = new(&root, &store, "leased", &[], &snapshot).unwrap();
    new(&root, &store, "other", &[], &snapshot).unwrap();
    let (_probe, lease) = open(&store, "leased").unwrap();
    assert!(
        rm(&root, &store, "leased", &Environment::new())
            .await
            .is_err()
    );
    drop(lease);
    let mut process = Command::new("sleep")
        .arg("60")
        .current_dir(&probe.directory)
        .stdin(Stdio::null())
        .spawn()
        .unwrap();
    assert!(
        rm(&root, &store, "leased", &Environment::new())
            .await
            .is_err()
    );
    process.kill().unwrap();
    process.wait().unwrap();
    assert_eq!(
        rm(&root, &store, "leased", &Environment::new())
            .await
            .unwrap(),
        probe.directory
    );
    assert!(store.join("probes/other/src/main.rs").is_file());
}

#[tokio::test]
async fn invalid_names_and_unknown_versions_cannot_select_other_paths() {
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
    assert!(
        rm(&root, &store, "future", &Environment::new())
            .await
            .is_err()
    );
    assert!(probe.directory.join("src/main.rs").is_file());
}

#[tokio::test]
async fn disposal_refuses_external_open_file_even_when_process_cwd_is_elsewhere() {
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
    assert!(
        rm(&root, &store, "open-file", &Environment::new())
            .await
            .is_err()
    );
    process.kill().unwrap();
    process.wait().unwrap();
    rm(&root, &store, "open-file", &Environment::new())
        .await
        .unwrap();
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

#[tokio::test]
async fn unknown_nonempty_legacy_run_capture_cannot_authorize_removal() {
    let (_temporary, root, store, snapshot) = fixture();
    let probe = new(&root, &store, "legacy", &[], &snapshot).unwrap();
    let capture = probe.directory.join("runs/original");
    directory(&capture).unwrap();
    publish(
        &capture.join("metadata.stdout"),
        b"original unresolved capture",
    )
    .unwrap();
    let error = rm(&root, &store, "legacy", &Environment::new())
        .await
        .unwrap_err();
    assert!(error.to_string().contains("unknown legacy or incomplete"));
    assert_eq!(
        fs::read(capture.join("metadata.stdout")).unwrap(),
        b"original unresolved capture"
    );
}

fn recorded_fixture(
    name: &str,
    change: Option<&str>,
) -> (tempfile::TempDir, PathBuf, PathBuf, Environment) {
    let (temporary, root, _, snapshot) = fixture();
    let output = Command::new("git")
        .current_dir(&root)
        .args(["init", "--quiet", "--initial-branch=main"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let mut environment: Environment = std::env::vars_os().collect();
    environment.insert(
        "XDG_STATE_HOME".into(),
        temporary.path().join("state").into_os_string(),
    );
    let state = crate::paths::CheckoutState::resolve(&root, &environment).unwrap();
    let store = state.root.clone();
    let ledger = crate::invocation::InvocationStore::new(state);
    ledger.register().unwrap();
    new(&root, &store, name, &[], &snapshot).unwrap();
    let (probe, lease) = open(&store, name).unwrap();
    let capture = probe.directory.join("runs/recorded");
    directory(&capture).unwrap();
    publish(&capture.join("metadata.stdout"), b"fixture capture").unwrap();
    let argv = vec![
        OsString::from("/fixture/immutable-correctness"),
        "run".into(),
        "--bin".into(),
        "pse-scratch-probe".into(),
    ];
    let control = crate::generation::PreparedGeneration {
        generation: "generation.fixture".into(),
        protocol: 1,
        manifest_digest: "fixture-control".into(),
    };
    let id = crate::envelope::CliInvocationId::new();
    let run = Run {
        probe: probe.clone(),
        resolution: snapshot,
        resolution_directory: capture,
        rustc_identity: "fixture selected rustc".into(),
        cargo_identity: "fixture selected Cargo".into(),
        command: argv.clone(),
        environment: environment.clone(),
        lease,
    };
    run.record_managed(
        &root,
        &environment,
        ManagedRun {
            invocation: id.clone(),
            command_name: "probe run".into(),
            command: argv.clone(),
            control: control.clone(),
            control_checkout: root.clone(),
        },
    )
    .unwrap();
    if change != Some("missing") {
        let mut operation =
            serde_json::json!({"command":argv,"cwd":probe.directory.as_os_str().to_owned()});
        let mut verb = "probe run";
        let mut actual_control = control;
        match change {
            Some("cwd") => {
                operation["cwd"] = serde_json::to_value(root.as_os_str().to_owned()).unwrap()
            }
            Some("argv") => {
                operation["command"] =
                    serde_json::to_value(vec![OsString::from("/unrelated")]).unwrap()
            }
            Some("verb") => verb = "run",
            Some("control") => actual_control.manifest_digest = "unrelated-control".into(),
            _ => {}
        }
        ledger
            .begin(crate::invocation::InvocationSpec {
                invocation_id: id,
                command: verb.into(),
                session: None,
                parent: None,
                operation,
                control: actual_control,
                environment_digest: "fixture".into(),
                scientific: vec![],
            })
            .unwrap();
    }
    // No native pass/settlement is manufactured. remove_with below tests the policy at its
    // native-observer seam; the production path calls the actual foreground custody owner.
    drop(run.lease);
    (temporary, root, store, environment)
}
#[tokio::test]
async fn durable_reference_requires_exact_original_routing_and_keeps_unknown_native_users() {
    for change in ["missing", "cwd", "argv", "verb", "control"] {
        let (_temporary, root, store, environment) = recorded_fixture("mismatch", Some(change));
        let called = std::sync::atomic::AtomicBool::new(false);
        let result = remove_with(&root, &store, "mismatch", &environment, |_| {
            called.store(true, std::sync::atomic::Ordering::SeqCst);
            async { Ok(true) }
        })
        .await;
        assert!(result.is_err(), "{change}");
        assert!(
            !called.load(std::sync::atomic::Ordering::SeqCst),
            "wrong original routing cannot ask an unrelated native owner for deletion authority"
        );
        assert!(store.join("probes/mismatch/src/main.rs").is_file());
    }
    let (_temporary, root, store, environment) = recorded_fixture("unknown", None);
    assert!(
        rm(&root, &store, "unknown", &environment).await.is_err(),
        "matching metadata for an otherwise empty invocation is not native custody/drain authority"
    );
    assert!(store.join("probes/unknown").is_dir());
    assert!(
        remove_with(&root, &store, "unknown", &environment, |_| async {
            Err(invalid("original native custody unavailable"))
        })
        .await
        .is_err()
    );
    assert!(store.join("probes/unknown").is_dir());
    assert!(
        remove_with(&root, &store, "unknown", &environment, |_| async {
            Ok(false)
        })
        .await
        .is_err()
    );
    assert!(store.join("probes/unknown").is_dir());
    // A successful semantic/terminal record is deliberately absent. Authenticated physical
    // drain, when supplied by the real native owner, is the applicable removal fact.
    assert!(
        remove_with(&root, &store, "unknown", &environment, |_| async {
            Ok(true)
        })
        .await
        .is_ok()
    );
}
#[tokio::test]
async fn removal_holds_original_probe_lease_across_native_observation() {
    let (_temporary, root, store, environment) = recorded_fixture("race", None);
    let entered = std::sync::Arc::new(tokio::sync::Notify::new());
    let release = std::sync::Arc::new(tokio::sync::Notify::new());
    let task_root = root.clone();
    let task_store = store.clone();
    let task_environment = environment.clone();
    let task_entered = entered.clone();
    let task_release = release.clone();
    let removal = tokio::spawn(async move {
        remove_with(
            &task_root,
            &task_store,
            "race",
            &task_environment,
            move |_| {
                let entered = task_entered.clone();
                let release = task_release.clone();
                async move {
                    entered.notify_one();
                    release.notified().await;
                    Ok(true)
                }
            },
        )
        .await
    });
    tokio::time::timeout(std::time::Duration::from_secs(10), entered.notified())
        .await
        .unwrap();
    let error = prepare_run(&root, &store, "race", &environment, &[]).unwrap_err();
    assert!(error.to_string().contains("live owner"));
    assert!(store.join("probes/race/src/main.rs").is_file());
    release.notify_one();
    assert!(removal.await.unwrap().is_ok());
    assert!(!store.join("probes/race").exists());
}
