// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original edit payload, owner routing and session activation controls on disposable roots.
use super::*;
use std::io;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
fn write(root: &Path, path: &str, text: &str) -> io::Result<()> {
    let path = root.join(path);
    fs::create_dir_all(
        path.parent()
            .ok_or_else(|| io::Error::other("fixture parent"))?,
    )?;
    fs::write(path, text)
}
fn git(root: &Path, args: &[&str]) -> Result {
    let status = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .status()?;
    assert!(status.success());
    Ok(())
}

#[test]
fn edit_payloads_preserve_all_patch_paths_and_reject_unknown_edits() -> Result {
    let patch = "*** Begin Patch\n*** Update File: a.rs\n*** Move to: b.rs\n@@\n-x\n+y\n*** Delete File: c.rs\n*** Add File: docs/generated/new.md\n+x\n*** End Patch";
    assert_eq!(
        edit_paths(&serde_json::json!({"tool_input": {"command": patch}}))?,
        ["a.rs", "b.rs", "c.rs", "docs/generated/new.md"]
    );
    for (key, path) in [("file_path", "a.rs"), ("notebook_path", "a.ipynb")] {
        assert_eq!(
            edit_paths(&serde_json::json!({"tool_input": {key: path}}))?,
            [path]
        );
    }
    for payload in [
        serde_json::json!([]),
        serde_json::json!({"tool_input": null}),
        serde_json::json!({"tool_input": {"file_path": 3}}),
        serde_json::json!({"tool_input": {"notebook_path": ""}}),
        serde_json::json!({"tool_input": {"command": "echo unchecked"}}),
        serde_json::json!({"tool_input": {"command": "*** Begin Patch\n*** Add File: x"}}),
    ] {
        assert!(edit_paths(&payload).is_err(), "{payload}");
    }
    Ok(())
}

#[test]
fn normalized_generated_and_vcs_paths_stay_protected_when_the_map_is_unavailable() -> Result {
    let root = tempfile::tempdir()?;
    for path in [
        "docs/../docs/generated/a",
        "./external/a",
        ".git/config",
        "build/out",
        "target/bin",
        "crates/a/src/generated/b.rs",
        "crates/pse-operations/src/generated/schema.surql",
        "crates/pse-ipopt-sys/src/bindings.rs",
        "scripts/generated/settings.rs",
        "xtask/src/generated/item.rs",
    ] {
        assert!(protected(root.path(), Path::new(path))?.is_some(), "{path}");
    }
    for path in [
        "crates/a/src/lib.rs",
        ".codex/skills/example/build/probe.rs",
        "docs/plans/build/note.md",
    ] {
        assert!(protected(root.path(), Path::new(path))?.is_none(), "{path}");
    }
    assert!(protected(root.path(), &root.path().join("new.rs"))?.is_none());
    Ok(())
}

#[cfg(unix)]
#[test]
fn symlinks_cannot_hide_protected_paths_and_shared_external_skills_remain_editable() -> Result {
    let root = tempfile::tempdir()?;
    let other = tempfile::tempdir()?;
    std::os::unix::fs::symlink(root.path().join("external"), root.path().join("alias"))?;
    // The final target is deliberately absent: path resolution must still follow it.
    assert!(protected(root.path(), Path::new("alias/reading.rs"))?.is_some());
    std::os::unix::fs::symlink(other.path(), root.path().join("shared"))?;
    assert!(protected(root.path(), Path::new("shared/build/reference.rs"))?.is_none());
    assert!(protected(root.path(), &other.path().join("elsewhere/new.rs"))?.is_none());
    std::os::unix::fs::symlink("loop", root.path().join("loop"))?;
    assert!(protected(root.path(), Path::new("loop/file")).is_err());
    Ok(())
}

#[test]
fn design_and_adr_edits_are_advisory_without_changing_their_required_workflows_or_personal_marker()
-> Result {
    let root = tempfile::tempdir()?;
    let adr = "docs/adr/0001-example.md";
    write(root.path(), adr, "---\nstatus: accepted\n---\n# Decision\n")?;
    let EditRoute::Allowed {
        additional_context: Some(context),
    } = route(root.path(), Path::new(adr))?
    else {
        panic!("accepted ADR receives its supersession route")
    };
    assert!(context.contains("immutable") && context.contains("supersession"));
    assert!(protected(root.path(), Path::new(adr))?.is_none());
    write(root.path(), adr, "---\nstatus: proposed\n---\n# Decision\n")?;
    assert!(protected(root.path(), Path::new(adr))?.is_none());
    write(root.path(), ".design-edit", "personal existing content\n")?;
    let EditRoute::Allowed {
        additional_context: Some(context),
    } = route(
        root.path(),
        Path::new("docs/authoritative_design/blueprint.md"),
    )?
    else {
        panic!("architecture receives its design route")
    };
    assert!(context.contains("decision/design") && context.contains("revision row"));
    assert_eq!(
        fs::read_to_string(root.path().join(".design-edit"))?,
        "personal existing content\n"
    );
    Ok(())
}

#[test]
fn declared_new_output_scopes_block_with_the_actual_producer_and_regeneration_verb() -> Result {
    use ownership::protocol::{
        GeneratorOwner, MAP_PATH, MAP_PRODUCER, OutputMap, Scope, ScopeKind, VERSION,
    };
    let root = tempfile::tempdir()?;
    let owner = |producer: &str, path: &str, kind| GeneratorOwner {
        producer: producer.into(),
        outputs: vec![Scope {
            path: path.into(),
            kind,
        }],
        regenerate: vec!["just".into(), "codegen".into()],
        inputs: vec![],
        contributors: None,
    };
    let map = OutputMap {
        schema_version: VERSION,
        comment: "fixture".into(),
        coverage_limits: vec![],
        owners: vec![
            owner("new-actual-producer", "new-output", ScopeKind::Tree),
            owner(MAP_PRODUCER, MAP_PATH, ScopeKind::File),
        ],
    };
    write(root.path(), MAP_PATH, &serde_json::to_string(&map)?)?;
    let reason = protected(root.path(), Path::new("new-output/file.rs"))?
        .ok_or("missing actual output protection")?;
    assert!(reason.contains("new-actual-producer") && reason.contains("just codegen"));
    assert!(protected(root.path(), Path::new("new-output-other/file.rs"))?.is_none());
    write(root.path(), MAP_PATH, "broken")?;
    assert!(protected(root.path(), Path::new("authored/lib.rs"))?.is_none());
    assert!(protected(root.path(), Path::new("new/generated/output.rs"))?.is_some());
    Ok(())
}

#[test]
fn session_root_selects_the_payload_checkout_and_rejects_unrelated_git_roots() -> Result {
    let root = tempfile::tempdir()?;
    let worktree = tempfile::tempdir()?;
    git(worktree.path(), &["init", "--quiet"])?;
    let payload = serde_json::json!({"cwd": worktree.path()});
    assert_eq!(session_root(root.path(), &payload)?, root.path());
    write(
        worktree.path(),
        "scripts/pse-env",
        "native environment frontdoor",
    )?;
    assert_eq!(
        session_root(root.path(), &payload)?.canonicalize()?,
        worktree.path().canonicalize()?
    );
    assert_eq!(session_root(root.path(), &Value::Null)?, root.path());
    let selected = session_root(root.path(), &payload)?;
    // A guard must read the selected checkout's map, not its launcher's unrelated root.
    let protocol = ownership::protocol::OutputMap {
        schema_version: ownership::protocol::VERSION,
        comment: "fixture".into(),
        coverage_limits: vec![],
        owners: [
            (
                "payload-owner",
                "selected-output",
                ownership::protocol::ScopeKind::Tree,
            ),
            (
                ownership::protocol::MAP_PRODUCER,
                ownership::protocol::MAP_PATH,
                ownership::protocol::ScopeKind::File,
            ),
        ]
        .into_iter()
        .map(
            |(producer, path, kind)| ownership::protocol::GeneratorOwner {
                producer: producer.into(),
                outputs: vec![ownership::protocol::Scope {
                    path: path.into(),
                    kind,
                }],
                regenerate: vec!["just".into(), "codegen".into()],
                inputs: vec![],
                contributors: None,
            },
        )
        .collect(),
    };
    write(
        worktree.path(),
        ownership::protocol::MAP_PATH,
        &serde_json::to_string(&protocol)?,
    )?;
    let reason = protected(&selected, Path::new("selected-output/file.rs"))?
        .ok_or("missing selected checkout protection")?;
    assert!(reason.contains("payload-owner"));
    Ok(())
}

#[cfg(unix)]
#[test]
fn session_activation_appends_only_the_fixed_program_and_does_not_invent_environment_values()
-> Result {
    use std::os::unix::fs::PermissionsExt;
    let root = tempfile::tempdir()?;
    write(
        root.path(),
        "scripts/pse-env",
        "#!/bin/sh\n[ \"$1\" = --print ] || exit 1\nprintf '%s\\n' 'fixed_native_activation'\n",
    )?;
    fs::set_permissions(
        root.path().join("scripts/pse-env"),
        fs::Permissions::from_mode(0o755),
    )?;
    write(
        root.path(),
        ".envrc.local",
        "export PRIVATE_VALUE='must not copy'\n",
    )?;
    let output = root.path().join("claude-env");
    fs::write(&output, "prior\n")?;
    session_env(root.path(), Some(&output))?;
    assert_eq!(
        fs::read_to_string(&output)?,
        "prior\nfixed_native_activation\n"
    );
    assert!(!fs::read_to_string(&output)?.contains("PRIVATE_VALUE"));
    session_env(root.path(), None)?;
    write(root.path(), "scripts/pse-env", "#!/bin/sh\nexit 7\n")?;
    assert!(session_env(root.path(), Some(&output)).is_err());
    assert_eq!(
        fs::read_to_string(&output)?,
        "prior\nfixed_native_activation\n"
    );
    Ok(())
}
