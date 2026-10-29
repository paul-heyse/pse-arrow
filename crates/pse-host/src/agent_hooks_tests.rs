// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original edit payload, authority and session activation controls on disposable roots.
use super::*;
use std::io;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
fn write(root: &Path, path: &str, text: &str) -> io::Result<()> {
    let path = root.join(path);
    fs::create_dir_all(path.parent().ok_or_else(|| io::Error::other("fixture parent"))?)?;
    fs::write(path, text)
}
fn git(root: &Path, args: &[&str]) -> Result {
    let status = Command::new("git").arg("-C").arg(root).args(args).status()?;
    assert!(status.success()); Ok(())
}

#[test]
fn edit_payloads_preserve_all_patch_paths_and_reject_unknown_edits() -> Result {
    let patch = "*** Begin Patch\n*** Update File: a.rs\n*** Move to: b.rs\n@@\n-x\n+y\n*** Delete File: c.rs\n*** Add File: docs/generated/new.md\n+x\n*** End Patch";
    assert_eq!(edit_paths(&serde_json::json!({"tool_input": {"command": patch}}))?, ["a.rs", "b.rs", "c.rs", "docs/generated/new.md"]);
    for (key, path) in [("file_path", "a.rs"), ("notebook_path", "a.ipynb")] {
        assert_eq!(edit_paths(&serde_json::json!({"tool_input": {key: path}}))?, [path]);
    }
    for payload in [serde_json::json!([]), serde_json::json!({"tool_input": null}), serde_json::json!({"tool_input": {"file_path": 3}}), serde_json::json!({"tool_input": {"notebook_path": ""}}), serde_json::json!({"tool_input": {"command": "echo unchecked"}}), serde_json::json!({"tool_input": {"command": "*** Begin Patch\n*** Add File: x"}})] {
        assert!(edit_paths(&payload).is_err(), "{payload}");
    }
    Ok(())
}

#[test]
fn normalized_generated_and_vcs_paths_stay_protected_under_design_authority() -> Result {
    let root = tempfile::tempdir()?;
    for path in ["docs/../docs/generated/a", "./external/a", ".git/config", "build/out", "target/bin", "crates/a/src/generated/b.rs", "crates/pse-operations/src/generated/schema.surql", "crates/pse-operations-queries/Cargo.toml", "python/pse/contracts/a.py", "python/pse/generated/project.py", "python/pse/_native.pyi", "crates/pse-ipopt-sys/src/bindings.rs", "scripts/generated/settings.rs", "xtask/src/generated/item.rs"] {
        assert!(protected(root.path(), Path::new(path), true)?.is_some(), "{path}");
    }
    for path in ["crates/a/src/lib.rs", ".codex/skills/example/build/probe.rs", "docs/plans/build/note.md"] { assert!(protected(root.path(), Path::new(path), false)?.is_none(), "{path}"); }
    assert!(protected(root.path(), &root.path().join("new.rs"), false)?.is_none());
    Ok(())
}

#[cfg(unix)]
#[test]
fn symlinks_cannot_hide_protected_paths_and_shared_external_skills_remain_editable() -> Result {
    let root = tempfile::tempdir()?;
    let other = tempfile::tempdir()?;
    std::os::unix::fs::symlink(root.path().join("external"), root.path().join("alias"))?;
    // The final target is deliberately absent: path resolution must still follow it.
    assert!(protected(root.path(), Path::new("alias/reading.rs"), false)?.is_some());
    std::os::unix::fs::symlink(other.path(), root.path().join("shared"))?;
    assert!(protected(root.path(), Path::new("shared/build/reference.rs"), false)?.is_none());
    assert!(protected(root.path(), &other.path().join("elsewhere/new.rs"), false)?.is_none());
    std::os::unix::fs::symlink("loop", root.path().join("loop"))?;
    assert!(protected(root.path(), Path::new("loop/file"), false).is_err());
    Ok(())
}

#[test]
fn design_exception_is_narrow_and_only_untracked_marker_grants_it() -> Result {
    let root = tempfile::tempdir()?;
    let adr = "docs/adr/0001-example.md";
    write(root.path(), adr, "---\nstatus: accepted\n---\n# Decision\n")?;
    assert!(protected(root.path(), Path::new(adr), false)?.is_some());
    assert!(protected(root.path(), Path::new(adr), true)?.is_none());
    write(root.path(), adr, "---\nstatus: proposed\n---\n# Decision\n")?;
    assert!(protected(root.path(), Path::new(adr), false)?.is_none());
    assert!(protected(root.path(), Path::new("docs/authoritative_design/blueprint.md"), false)?.is_some());
    assert!(!design_edit_authorized(root.path(), None)?);
    assert!(design_edit_authorized(root.path(), Some(OsStr::new("1")))?);
    git(root.path(), &["init", "--quiet"])?;
    write(root.path(), DESIGN_EDIT_MARKER, "why: authorized design revision\n")?;
    assert!(design_edit_authorized(root.path(), None)?);
    git(root.path(), &["add", DESIGN_EDIT_MARKER])?;
    assert!(!design_edit_authorized(root.path(), None)?);
    fs::remove_file(root.path().join(DESIGN_EDIT_MARKER))?;
    assert!(!design_edit_authorized(root.path(), None)?);
    Ok(())
}

#[test]
fn session_root_selects_the_payload_checkout_and_rejects_unrelated_git_roots() -> Result {
    let root = tempfile::tempdir()?;
    let worktree = tempfile::tempdir()?;
    git(worktree.path(), &["init", "--quiet"])?;
    let payload = serde_json::json!({"cwd": worktree.path()});
    assert_eq!(session_root(root.path(), &payload)?, root.path());
    write(worktree.path(), "scripts/pse-env", "native environment frontdoor")?;
    assert_eq!(session_root(root.path(), &payload)?.canonicalize()?, worktree.path().canonicalize()?);
    assert_eq!(session_root(root.path(), &Value::Null)?, root.path());
    Ok(())
}

#[cfg(unix)]
#[test]
fn session_activation_appends_only_the_fixed_program_and_does_not_invent_environment_values() -> Result {
    use std::os::unix::fs::PermissionsExt;
    let root = tempfile::tempdir()?;
    write(root.path(), "scripts/pse-env", "#!/bin/sh\n[ \"$1\" = --print ] || exit 1\nprintf '%s\\n' 'fixed_native_activation'\n")?;
    fs::set_permissions(root.path().join("scripts/pse-env"), fs::Permissions::from_mode(0o755))?;
    write(root.path(), ".envrc.local", "export PRIVATE_VALUE='must not copy'\n")?;
    let output = root.path().join("claude-env");
    fs::write(&output, "prior\n")?;
    session_env(root.path(), Some(&output))?;
    assert_eq!(fs::read_to_string(&output)?, "prior\nfixed_native_activation\n");
    assert!(!fs::read_to_string(&output)?.contains("PRIVATE_VALUE"));
    session_env(root.path(), None)?;
    write(root.path(), "scripts/pse-env", "#!/bin/sh\nexit 7\n")?;
    assert!(session_env(root.path(), Some(&output)).is_err());
    assert_eq!(fs::read_to_string(&output)?, "prior\nfixed_native_activation\n");
    Ok(())
}
