// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use std::{ffi::OsStr, fs, process::Command};

fn git(root: &Path, arguments: &[&OsStr]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(root)
        .args([
            "-c",
            "user.name=repository fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.hooksPath=/dev/null",
        ])
        .args(arguments)
        .status()
        .expect("Git fixture command");
    assert!(status.success(), "fixture Git command {arguments:?}");
}
fn words(root: &Path, arguments: &[&str]) {
    git(root, &arguments.iter().map(OsStr::new).collect::<Vec<_>>());
}
fn fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("fixture");
    words(
        root.path(),
        &["init", "--quiet", "--initial-branch", "main"],
    );
    root
}
fn observed(root: &Path) -> RepositoryObservation {
    observe(root, Arc::new(AtomicBool::new(false))).expect("status")
}

#[test]
fn native_status_preserves_staged_unstaged_untracked_and_original_index() {
    let root = fixture();
    fs::write(root.path().join("tracked.rs"), "initial\n").expect("source");
    fs::write(root.path().join(".gitignore"), "ignored\n").expect("ignores");
    words(root.path(), &["add", "tracked.rs", ".gitignore"]);
    words(root.path(), &["commit", "--quiet", "-m", "initial"]);
    fs::write(root.path().join("tracked.rs"), "staged\n").expect("staged source");
    words(root.path(), &["add", "tracked.rs"]);
    fs::write(root.path().join("tracked.rs"), "unstaged\n").expect("unstaged source");
    fs::write(root.path().join("ignored"), "ignored\n").expect("ignored source");
    fs::create_dir(root.path().join("new-directory")).expect("untracked directory");
    fs::write(root.path().join("new-directory/a.rs"), "new\n").expect("untracked source");
    fs::write(root.path().join("new-directory/b.rs"), "new\n").expect("untracked source");
    let index_before = fs::read(root.path().join(".git/index")).expect("original index");
    let status = observed(root.path());
    assert_eq!(status.dirty_path_count, 3);
    assert_eq!(status.changes.len(), 4);
    assert_eq!(
        status
            .changes
            .iter()
            .filter(|change| change.path.as_bytes() == b"tracked.rs")
            .map(|change| change.layer)
            .collect::<Vec<_>>(),
        vec![ChangeLayer::Staged, ChangeLayer::Worktree]
    );
    assert!(
        status
            .changes
            .iter()
            .filter(|change| change.layer == ChangeLayer::Untracked)
            .all(|change| change.kind == ChangeKind::Added)
    );
    assert_eq!(status.head_before, status.head_after);
    assert!(matches!(status.head_before, HeadState::Attached { .. }));
    assert_eq!(
        fs::read(root.path().join(".git/index")).expect("index after observation"),
        index_before
    );
    assert!(!root.path().join(".git/index.lock").exists());
}

#[test]
fn unborn_and_detached_head_and_linked_worktrees_keep_distinct_meanings() {
    let root = fixture();
    let initial = observed(root.path());
    assert!(matches!(initial.head_before, HeadState::Unborn { .. }));
    assert_eq!(initial.dirty_path_count, 0);
    words(
        root.path(),
        &["commit", "--quiet", "--allow-empty", "-m", "initial"],
    );
    let worktree = root.path().join("linked");
    git(
        root.path(),
        &[
            OsStr::new("worktree"),
            OsStr::new("add"),
            OsStr::new("--quiet"),
            OsStr::new("--detach"),
            worktree.as_os_str(),
        ],
    );
    let linked = observed(&worktree);
    assert!(matches!(linked.head_before, HeadState::Detached { .. }));
    assert_ne!(linked.git_dir, linked.common_dir);
    assert_eq!(linked.common_dir, observed(root.path()).common_dir);
}

#[test]
fn non_unicode_git_paths_round_trip_without_lossy_identity() {
    let root = fixture();
    let name = OsStr::from_bytes(b"source-\xff.rs");
    fs::write(root.path().join(name), "new source\n").expect("byte path");
    let status = observed(root.path());
    assert_eq!(status.changes.len(), 1);
    let path = &status.changes[0].path;
    assert!(matches!(path, GitName::Bytes(_)));
    assert_eq!(path.as_bytes(), name.as_bytes());
    let json = serde_json::to_string(&status).expect("serializable byte identities");
    assert!(!json.contains('\u{fffd}'));
}

#[test]
fn cancelled_or_unavailable_observation_is_not_clean_status() {
    let root = fixture();
    assert!(observe(root.path(), Arc::new(AtomicBool::new(true))).is_err());
    let absent = root.path().join("absent");
    assert!(observe(&absent, Arc::new(AtomicBool::new(false))).is_err());
}
