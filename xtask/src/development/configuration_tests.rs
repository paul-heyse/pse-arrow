// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Alias custody and native instruction wiring in disposable repositories.
use super::*;

fn write(root: &Path, path: &str, text: &str) -> Result<()> {
    let path = root.join(path);
    fs::create_dir_all(path.parent().context("fixture parent")?)?;
    fs::write(path, text)?; Ok(())
}

#[test]
fn aliases_preserve_native_adapters_and_do_not_traverse_shared_bundle_links() -> Result<()> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path();
    write(root, ".codex/skills/local/SKILL.md", "local process skill")?;
    write(root, ".codex/agents/executor.toml", "custom native adapter")?;
    write(root, ".claude/agents/implementer.md", "custom other adapter")?;
    write(root, "shared/library/SKILL.md", "original shared capability")?;
    symlink(&root.join("shared/library"), &root.join(".codex/skills/library"), true)?;
    write(root, ".claude/skills", LINK_TARGET)?;
    // An existing directory exercises materialization without mocking OS behavior.
    fs::create_dir_all(root.join(".agents/skills"))?;
    assert!(synchronize(root, false)?.is_empty());
    assert!(synchronize(root, true)?.is_empty());
    assert!(fs::symlink_metadata(root.join(".agents/skills/library"))?.is_symlink());
    assert_eq!(fs::read_to_string(root.join(".codex/agents/executor.toml"))?, "custom native adapter");
    assert_eq!(fs::read_to_string(root.join(".claude/agents/implementer.md"))?, "custom other adapter");
    write(root, ".agents/skills/local/SKILL.md", "changed alias")?;
    assert_eq!(synchronize(root, true)?.len(), 1);
    assert!(synchronize(root, false).is_err());
    assert_eq!(fs::read_to_string(root.join(".agents/skills/local/SKILL.md"))?, "changed alias");
    assert_eq!(fs::read_to_string(root.join("shared/library/SKILL.md"))?, "original shared capability");
    Ok(())
}

#[test]
fn alias_conflicts_are_preflighted_before_replacing_any_placeholder() -> Result<()> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path();
    write(root, ".codex/skills/local/SKILL.md", "canonical")?;
    write(root, ".claude/skills", LINK_TARGET)?;
    write(root, ".agents/skills/unrelated/SKILL.md", "unrelated")?;
    assert!(synchronize(root, false).is_err());
    assert_eq!(fs::read_to_string(root.join(".claude/skills"))?, LINK_TARGET);
    assert_eq!(fs::read_to_string(root.join(".agents/skills/unrelated/SKILL.md"))?, "unrelated");
    Ok(())
}

#[test]
fn wrong_symlink_is_preserved_and_sync_does_not_generate_adapters() -> Result<()> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path();
    write(root, ".codex/skills/local/SKILL.md", "canonical")?;
    fs::create_dir_all(root.join(".claude"))?;
    symlink(Path::new("../other"), &root.join(".claude/skills"), true)?;
    assert!(synchronize(root, false).is_err());
    assert_eq!(fs::read_link(root.join(".claude/skills"))?, Path::new("../other"));
    fs::remove_file(root.join(".claude/skills"))?;
    synchronize(root, false)?;
    assert!(synchronize(root, true)?.is_empty());
    assert!(!root.join(".codex/agents").exists());
    assert!(!root.join(".claude/agents").exists());
    Ok(())
}

#[test]
fn library_selection_delegates_exact_repo_and_mode_without_reimplementing_policy() -> Result<()> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path();
    write(root, "shared library/manage.py", "external tool owns the manifest")?;
    let store = root.join("shared library");
    let command = library_command(root, true, Some(store.clone().into_os_string()), None, Some("/pinned/python".into()))?;
    assert_eq!(command.get_program(), "/pinned/python");
    assert_eq!(command.get_args().map(OsStr::to_owned).collect::<Vec<_>>(), [store.join("manage.py").into_os_string(), "--repo".into(), root.canonicalize()?.into_os_string(), "--check".into()]);
    let command = library_command(root, false, Some("~/shared library".into()), Some(root.as_os_str().to_owned()), None)?;
    assert_eq!(command.get_program(), root.canonicalize()?.join(".venv/bin/python"));
    assert_eq!(command.get_args().next(), Some(store.join("manage.py").as_os_str()));
    assert!(library_command(root, false, Some(root.join("missing").into_os_string()), None, None).is_err());
    Ok(())
}

#[test]
fn instruction_scan_excludes_shared_links_and_checks_local_nested_references() -> Result<()> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path();
    for path in ["AGENTS.md", ".agents/roles/executor.md", ".codex/skills/local/SKILL.md", ".codex/skills/local/references/contract.md"] { write(root, path, "`scripts/actual.rs` and `just actual`\n")?; }
    write(root, "shared/library/SKILL.md", "`scripts/never-traverse.rs`")?;
    symlink(&root.join("shared/library"), &root.join(".codex/skills/library"), true)?;
    let scanned = scanned_files(root)?;
    assert_eq!(scanned.len(), 4);
    assert!(scanned.iter().all(|path| !path.to_string_lossy().contains("library")));
    assert_eq!(path_references("scripts/actual.rs, `docs/plans/{slug}.md`\nscripts/ignored.rs agent-config: ignore\nnotdocs/bogus.rs")?, BTreeSet::from(["scripts/actual.rs".into()]));
    assert_eq!(recipe_references("`just actual --flag`\n just also-actual\n`just --list`")?, BTreeSet::from(["actual".into(), "also-actual".into()]));
    assert_eq!(body("---\npaths: [scripts/missing.rs]\n---\nactual"), "actual");
    Ok(())
}

#[test]
fn native_hook_wiring_and_anchored_deny_rules_refuse_foreign_or_ineffective_rules() -> Result<()> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path();
    let settings = root.join("settings.json");
    fs::write(&settings, serde_json::to_vec(&serde_json::json!({"hooks": {"PreToolUse": [{"hooks": [{"command": "\"${CLAUDE_PROJECT_DIR}/scripts/pse-host\" agent-hook guard"}]}]}, "permissions": {"deny": ["Edit(/target/**)", "Read(~/private/**)"]}}))?)?;
    let mut problems = Vec::new();
    check_hooks(&settings, &mut problems)?;
    assert_eq!(check_deny_rules(&settings, &mut problems)?, 2);
    assert!(problems.is_empty());
    fs::write(&settings, serde_json::to_vec(&serde_json::json!({"hooks": {"Stop": [{"hooks": [{"command": "echo not shared"}]}]}, "permissions": {"deny": ["Write(./build/**)"]}}))?)?;
    check_hooks(&settings, &mut problems)?;
    check_deny_rules(&settings, &mut problems)?;
    assert!(problems.iter().any(|problem| problem.contains("missing PreToolUse")));
    assert!(problems.iter().any(|problem| problem.contains("does not use shared native policy")));
    assert!(problems.iter().any(|problem| problem.contains("must use Edit or Read")));
    assert!(problems.iter().any(|problem| problem.contains("anchored")));
    Ok(())
}

#[test]
fn adapter_checks_require_shared_contracts_and_native_owned_defaults() -> Result<()> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path();
    write(root, ".agents/roles/worker.md", "shared")?;
    for role in ROLES {
        write(root, &format!(".agents/roles/{role}.md"), "shared role")?;
        let instructions = format!("read .agents/roles/worker.md and .agents/roles/{role}.md");
        write(root, &format!(".codex/agents/{role}.toml"), &format!("name = {role:?}\ndescription = \"native default\"\nmodel = \"configured\"\nmodel_reasoning_effort = \"high\"\ndeveloper_instructions = {instructions:?}\n"))?;
        write(root, &format!(".claude/agents/{}.md", if role == "executor" { "implementer" } else { role }), &instructions)?;
    }
    let mut problems = Vec::new();
    check_adapters(root, &mut problems)?;
    assert!(problems.is_empty(), "{problems:?}");
    write(root, ".codex/agents/executor.toml", "name = \"wrong\"\n")?;
    check_adapters(root, &mut problems)?;
    assert!(problems.iter().any(|problem| problem.contains("missing shared contract")));
    assert!(problems.iter().any(|problem| problem.contains("missing model")));
    assert!(problems.iter().any(|problem| problem.contains("name must be executor")));
    Ok(())
}
