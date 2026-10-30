// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;

fn git(root: &Path, arguments: &[&OsStr]) {
    let output = Command::new("git")
        .current_dir(root)
        .args([
            "-c",
            "user.name=worktree fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.hooksPath=/dev/null",
        ])
        .args(arguments)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
}
fn words(root: &Path, arguments: &[&str]) {
    git(root, &arguments.iter().map(OsStr::new).collect::<Vec<_>>());
}
fn fixture() -> (tempfile::TempDir, PathBuf, Environment) {
    let scratch = tempfile::tempdir().unwrap();
    let root = scratch.path().join("source");
    fs::create_dir_all(root.join("scripts")).unwrap();
    fs::write(root.join(".gitignore"), ".envrc.local\n").unwrap();
    fs::write(root.join("file.rs"), "fn original() {}\n").unwrap();
    fs::write(root.join("scripts/bootstrap-host"),"#!/bin/sh\ntest \"$1\" = --from || exit 41\nprintf 'independent preparation refusal\\n' >&2\nexit 42\n").unwrap();
    fs::set_permissions(
        root.join("scripts/bootstrap-host"),
        fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    words(&root, &["init", "--quiet", "--initial-branch=main"]);
    words(&root, &["add", "."]);
    words(
        &root,
        &["commit", "--quiet", "-m", "initial worktree fixture"],
    );
    let mut environment: Environment = std::env::vars_os().collect();
    environment.insert(
        "XDG_STATE_HOME".into(),
        scratch.path().join("state").into_os_string(),
    );
    (scratch, root, environment)
}
fn request(name: &str) -> NewRequest {
    NewRequest {
        name: name.into(),
        base: "HEAD".into(),
        branch: None,
        warm: false,
    }
}

#[test]
fn preparation_failure_preserves_created_worktree_and_exact_local_configuration_mode() {
    let (scratch, root, environment) = fixture();
    let private = b"PSE_BUILD_MODE=unbounded\nPRIVATE=test-private-value\n";
    fs::write(root.join(".envrc.local"), private).unwrap();
    fs::set_permissions(root.join(".envrc.local"), fs::Permissions::from_mode(0o640)).unwrap();
    let response = create(&root, request("partial"), &environment).unwrap();
    assert_eq!(response.exit_code(), 1);
    assert!(response.payload.git_created);
    assert!(response.payload.destination_exists);
    assert!(!response.payload.ready);
    let destination = scratch.path().join("partial");
    assert_eq!(fs::read(destination.join(".envrc.local")).unwrap(), private);
    assert_eq!(
        fs::metadata(destination.join(".envrc.local"))
            .unwrap()
            .mode()
            & 0o7777,
        0o640
    );
    let preparation = response
        .payload
        .stages
        .iter()
        .find(|stage| stage.stage == "host_preparation")
        .unwrap();
    assert_eq!(preparation.state, StageState::Failed);
    assert_eq!(preparation.command.as_ref().unwrap().exit_code, Some(42));
    assert!(
        response
            .payload
            .stages
            .iter()
            .filter(|stage| stage.stage == "registration")
            .all(|stage| stage.state == StageState::Pending)
    );
    assert_eq!(
        response
            .payload
            .stages
            .iter()
            .find(|stage| stage.stage == "warm_launch")
            .unwrap()
            .state,
        StageState::Skipped
    );
    assert!(
        repository::observe_worktrees(&root)
            .unwrap()
            .entries
            .iter()
            .any(|entry| entry
                .worktree
                .as_ref()
                .is_some_and(|path| path.as_bytes() == destination.as_os_str().as_bytes()))
    );
    assert!(
        !serde_json::to_string(&response)
            .unwrap()
            .contains("test-private-value")
    );
}

#[test]
fn invalid_names_refuse_before_git_and_existing_destination_is_never_removed() {
    let (scratch, root, environment) = fixture();
    for name in ["../escape", ".", "..", "/absolute", "-option", ""] {
        assert!(
            create(&root, request(name), &environment).is_err(),
            "{name}"
        );
    }
    let destination = scratch.path().join("occupied");
    fs::create_dir(&destination).unwrap();
    fs::write(destination.join("authored"), "preserve").unwrap();
    let response = create(&root, request("occupied"), &environment).unwrap();
    assert!(!response.payload.git_created);
    assert!(response.payload.destination_exists);
    assert_eq!(response.payload.stages[0].state, StageState::Failed);
    assert_eq!(
        fs::read_to_string(destination.join("authored")).unwrap(),
        "preserve"
    );
}

#[test]
fn environment_copy_refuses_different_existing_bytes_without_overwrite() {
    let (scratch, root, _) = fixture();
    let destination = scratch.path().join("independent");
    fs::create_dir(&destination).unwrap();
    fs::write(root.join(".envrc.local"), "source configuration").unwrap();
    fs::write(destination.join(".envrc.local"), "concurrent configuration").unwrap();
    assert!(copied_environment(&root, &destination).is_err());
    assert_eq!(
        fs::read_to_string(destination.join(".envrc.local")).unwrap(),
        "concurrent configuration"
    );
}

#[test]
fn selected_checkout_ignores_ambient_repository_local_git_overrides() {
    let (scratch, root, mut environment) = fixture();
    let foreign = scratch.path().join("foreign");
    fs::create_dir(&foreign).unwrap();
    words(&foreign, &["init", "--quiet", "--initial-branch=main"]);
    environment.insert("GIT_DIR".into(), foreign.join(".git").into_os_string());
    environment.insert("GIT_WORK_TREE".into(), foreign.into_os_string());
    environment.insert(
        "GIT_INDEX_FILE".into(),
        scratch.path().join("foreign-index").into_os_string(),
    );
    let response = create(&root, request("selected"), &environment).unwrap();
    assert!(response.payload.git_created);
    assert!(scratch.path().join("selected/file.rs").is_file());
    assert!(!scratch.path().join("foreign-index").exists());
}

#[test]
fn standalone_registration_creates_only_the_original_owned_namespace() {
    let (_scratch, root, environment) = fixture();
    let state = CheckoutState::resolve(&root, &environment).unwrap();
    let store = InvocationStore::new(state.clone());
    assert!(!state.root.exists());
    store.register().unwrap();
    store.register().unwrap();
    assert_eq!(fs::metadata(&state.root).unwrap().mode() & 0o777, 0o700);
    assert_eq!(fs::read_dir(state.invocations()).unwrap().count(), 0);
    assert!(
        store
            .history::<serde::de::IgnoredAny>()
            .unwrap()
            .records
            .is_empty()
    );
}

#[tokio::test]
async fn list_retains_native_paths_direct_ancestry_dirty_state_and_no_preparation_effects() {
    let (scratch, root, environment) = fixture();
    let peer = scratch
        .path()
        .join(OsString::from_vec(b"peer-\xff".to_vec()));
    git(
        &root,
        &[
            OsStr::new("worktree"),
            OsStr::new("add"),
            OsStr::new("-b"),
            OsStr::new("peer"),
            OsStr::new("--"),
            peer.as_os_str(),
            OsStr::new("HEAD"),
        ],
    );
    fs::write(peer.join("added.rs"), "fn added() {}\n").unwrap();
    words(&peer, &["add", "added.rs"]);
    words(&peer, &["commit", "--quiet", "-m", "peer commit"]);
    fs::write(peer.join("file.rs"), "fn dirty() {}\n").unwrap();
    let response = list(&root, OsStr::new("HEAD"), &environment).await.unwrap();
    assert_eq!(response.exit_code(), 0);
    assert_eq!(response.payload.worktrees.len(), 2);
    let row = response
        .payload
        .worktrees
        .iter()
        .find(|row| {
            row.git
                .worktree
                .as_ref()
                .is_some_and(|path| path.as_bytes() == peer.as_os_str().as_bytes())
        })
        .unwrap();
    assert_eq!(row.dirty_paths, Some(1));
    assert_eq!(row.divergence.as_ref().unwrap().ahead, 1);
    assert_eq!(row.divergence.as_ref().unwrap().behind, 0);
    assert!(row.invocations.is_empty());
    assert!(!peer.join("target").exists());
    assert!(!scratch.path().join("state").exists());
    let wire = serde_json::to_value(response).unwrap();
    assert!(wire.to_string().contains("255"));
}
