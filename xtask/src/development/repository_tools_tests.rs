// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Pin projection custody and actual Git eligibility, without compiling or acquiring tools.
use super::*;

fn pins(tree: &str) -> Pins {
    BTreeMap::from([
        (
            "ci".into(),
            format!("{PREFIX}ci-{tree}@sha256:{}", "a".repeat(64)),
        ),
        (
            "dev".into(),
            format!("{PREFIX}dev-{tree}@sha256:{}", "b".repeat(64)),
        ),
    ])
}
fn write(root: &Path, relative: &str, text: &str) -> Result<()> {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().context("fixture parent")?)?;
    fs::write(path, text)?;
    Ok(())
}
fn git(root: &Path, arguments: &[&str]) -> Result<()> {
    let result = Command::new("git")
        .args(arguments)
        .current_dir(root)
        .output()?;
    ensure!(
        result.status.success(),
        "Git fixture {arguments:?}: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}

#[test]
fn pins_require_exact_stage_keys_matching_recipe_trees_and_immutable_digests() -> Result<()> {
    let valid = pins("0123456789ab");
    validate(&valid)?;
    let mut missing = valid.clone();
    missing.remove("dev");
    let mut extra = valid.clone();
    extra.insert("other".into(), "image".into());
    let mut mismatched = valid.clone();
    mismatched.insert("dev".into(), pins("fedcba987654")["dev"].clone());
    let mut mutable = valid.clone();
    mutable.insert("dev".into(), format!("{PREFIX}dev-0123456789ab"));
    let mut wrong_stage = valid.clone();
    wrong_stage.insert("dev".into(), valid["ci"].clone());
    let mut uppercase = valid.clone();
    uppercase.insert(
        "ci".into(),
        valid["ci"].replace("0123456789ab", "0123456789AB"),
    );
    for invalid in [missing, extra, mismatched, mutable, wrong_stage, uppercase] {
        assert!(validate(&invalid).is_err(), "{invalid:?}");
    }
    Ok(())
}

#[test]
fn runtime_override_is_content_addressed_and_applies_to_either_stage() -> Result<()> {
    let pins = pins("0123456789ab");
    assert_eq!(runtime(ImageStage::Ci, &pins, None)?, pins["ci"]);
    assert_eq!(runtime(ImageStage::Dev, &pins, Some(""))?, pins["dev"]);
    for immutable in [
        format!("sha256:{}", "c".repeat(64)),
        format!("local.registry:5000/team/image@sha256:{}", "d".repeat(64)),
    ] {
        assert_eq!(runtime(ImageStage::Ci, &pins, Some(&immutable))?, immutable);
        assert_eq!(
            runtime(ImageStage::Dev, &pins, Some(&immutable))?,
            immutable
        );
    }
    for mutable in [
        "latest",
        "image:tag",
        "sha256:abc",
        "local@sha256:abc",
        " sha256:abc",
        "image\n@sha256:abc",
    ] {
        assert!(
            runtime(ImageStage::Dev, &pins, Some(mutable)).is_err(),
            "{mutable}"
        );
    }
    Ok(())
}

#[test]
fn projections_replace_only_solver_literals_and_preserve_surrounding_authored_bytes() -> Result<()>
{
    let fixture = tempfile::tempdir()?;
    let root = fixture.path();
    let pins = pins("0123456789ab");
    let yaml = format!(
        "# retained comment\ncontainer: {PREFIX}ci-old.tag\nother: example/image:old\nsecond: {PREFIX}dev-old-tag@sha256:{}\n",
        "0".repeat(64)
    );
    write(root, ".github/workflows/example.yml", &yaml)?;
    write(
        root,
        ".github/workflows/not-selected.yaml",
        "untouched alternative suffix",
    )?;
    write(
        root,
        ".devcontainer/devcontainer.json",
        &format!("{{\"image\":\"{PREFIX}dev-old\",\"custom\":true}}\n"),
    )?;
    let updates = projections(root, &pins)?;
    assert_eq!(updates.len(), 2);
    assert_eq!(
        updates[&root.join(".github/workflows/example.yml")],
        format!(
            "# retained comment\ncontainer: {}\nother: example/image:old\nsecond: {}\n",
            pins["ci"], pins["dev"]
        )
    );
    assert_eq!(
        updates[&root.join(".devcontainer/devcontainer.json")],
        format!("{{\"image\":\"{}\",\"custom\":true}}\n", pins["dev"])
    );
    assert_eq!(
        fs::read_to_string(root.join(".github/workflows/example.yml"))?,
        yaml
    );
    Ok(())
}

#[test]
fn check_is_read_only_sync_converges_and_update_changes_the_single_pin_owner() -> Result<()> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path();
    let first = pins("0123456789ab");
    write(root, MANIFEST, &serde_json::to_string_pretty(&first)?)?;
    write(
        root,
        ".github/workflows/example.yml",
        &format!("container: {PREFIX}ci-old\n"),
    )?;
    write(
        root,
        ".devcontainer/devcontainer.json",
        &format!("{{\"image\":\"{PREFIX}dev-old\"}}\n"),
    )?;
    let original_manifest = fs::read(root.join(MANIFEST))?;
    let original_workflow = fs::read(root.join(".github/workflows/example.yml"))?;
    assert_eq!(solver_images(root, SolverImagesCommand::Check)?, 1);
    assert_eq!(fs::read(root.join(MANIFEST))?, original_manifest);
    assert_eq!(
        fs::read(root.join(".github/workflows/example.yml"))?,
        original_workflow
    );
    assert_eq!(solver_images(root, SolverImagesCommand::Sync)?, 0);
    assert_eq!(solver_images(root, SolverImagesCommand::Check)?, 0);
    assert_eq!(fs::read(root.join(MANIFEST))?, original_manifest);
    let next = pins("fedcba987654");
    assert_eq!(
        solver_images(
            root,
            SolverImagesCommand::Update {
                ci: next["ci"].clone(),
                dev: next["dev"].clone()
            }
        )?,
        0
    );
    assert_eq!(
        serde_json::from_slice::<Pins>(&fs::read(root.join(MANIFEST))?)?,
        next
    );
    assert_eq!(solver_images(root, SolverImagesCommand::Check)?, 0);
    assert!(fs::read(root.join(MANIFEST))?.ends_with(b"\n"));
    let before = fs::read(root.join(MANIFEST))?;
    let workflow = fs::read(root.join(".github/workflows/example.yml"))?;
    assert!(
        solver_images(
            root,
            SolverImagesCommand::Update {
                ci: "mutable".into(),
                dev: next["dev"].clone()
            }
        )
        .is_err()
    );
    assert_eq!(fs::read(root.join(MANIFEST))?, before);
    assert_eq!(
        fs::read(root.join(".github/workflows/example.yml"))?,
        workflow
    );
    Ok(())
}

#[test]
fn projection_preflight_failure_leaves_manifest_and_other_projection_bytes_untouched() -> Result<()>
{
    let fixture = tempfile::tempdir()?;
    let root = fixture.path();
    let original = pins("0123456789ab");
    write(root, MANIFEST, &serde_json::to_string(&original)?)?;
    write(
        root,
        ".github/workflows/example.yml",
        "retained unrelated bytes\n",
    )?;
    let next = pins("fedcba987654");
    assert!(
        solver_images(
            root,
            SolverImagesCommand::Update {
                ci: next["ci"].clone(),
                dev: next["dev"].clone()
            }
        )
        .is_err()
    );
    assert_eq!(
        serde_json::from_slice::<Pins>(&fs::read(root.join(MANIFEST))?)?,
        original
    );
    assert_eq!(
        fs::read_to_string(root.join(".github/workflows/example.yml"))?,
        "retained unrelated bytes\n"
    );
    Ok(())
}

#[test]
fn reuse_snapshot_uses_git_eligible_untracked_files_without_ignored_nested_acquisition()
-> Result<()> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path();
    git(root, &["init", "-q"])?;
    write(
        root,
        ".gitignore",
        "**/.venv/\n**/target/\ntracked-ignored\n",
    )?;
    write(root, "tracked.txt", "tracked current worktree bytes")?;
    write(root, "deleted.txt", "removed after index addition")?;
    write(root, "replaced/source.rs", "former directory child")?;
    write(root, "tracked-ignored", "tracked remains eligible")?;
    git(
        root,
        &[
            "add",
            ".gitignore",
            "tracked.txt",
            "deleted.txt",
            "replaced/source.rs",
        ],
    )?;
    git(root, &["add", "--force", "tracked-ignored"])?;
    fs::remove_file(root.join("deleted.txt"))?;
    fs::remove_dir_all(root.join("replaced"))?;
    write(root, "replaced", "current regular replacement")?;
    write(
        root,
        "new-project/src/new\tfile\nname.rs",
        "eligible implementation",
    )?;
    write(root, "new-project/.venv/vendor.py", "ignored acquisition")?;
    write(
        root,
        "new-project/target/compiled",
        "ignored compiled output",
    )?;
    let snapshot = tempfile::tempdir()?;
    let selected = source_paths(root)?;
    assert!(selected.contains(&PathBuf::from("new-project/src/new\tfile\nname.rs")));
    assert!(!selected.contains(&PathBuf::from("new-project/.venv/vendor.py")));
    copy_sources(root, snapshot.path(), &selected)?;
    assert_eq!(
        fs::read_to_string(snapshot.path().join("tracked.txt"))?,
        "tracked current worktree bytes"
    );
    assert_eq!(
        fs::read_to_string(snapshot.path().join("tracked-ignored"))?,
        "tracked remains eligible"
    );
    assert_eq!(
        fs::read_to_string(snapshot.path().join("new-project/src/new\tfile\nname.rs"))?,
        "eligible implementation"
    );
    assert!(!snapshot.path().join("new-project/.venv").exists());
    assert!(!snapshot.path().join("new-project/target").exists());
    assert!(!snapshot.path().join("deleted.txt").exists());
    assert_eq!(
        fs::read_to_string(snapshot.path().join("replaced"))?,
        "current regular replacement"
    );
    assert!(!snapshot.path().join(".git").exists());
    Ok(())
}

#[cfg(unix)]
#[test]
fn reuse_snapshot_never_follows_selected_symlinks_or_substituted_parent_directories() -> Result<()>
{
    use std::os::unix::fs::symlink;
    let fixture = tempfile::tempdir()?;
    let root = fixture.path();
    git(root, &["init", "-q"])?;
    write(root, "replaced/source.rs", "original")?;
    git(root, &["add", "replaced/source.rs"])?;
    fs::remove_dir_all(root.join("replaced"))?;
    let outside = tempfile::tempdir()?;
    write(outside.path(), "source.rs", "outside acquisition")?;
    symlink(outside.path(), root.join("replaced"))?;
    symlink(outside.path().join("source.rs"), root.join("linked.rs"))?;
    let snapshot = tempfile::tempdir()?;
    copy_sources(root, snapshot.path(), &source_paths(root)?)?;
    assert!(!snapshot.path().join("replaced").exists());
    assert!(!snapshot.path().join("linked.rs").exists());
    Ok(())
}

#[cfg(unix)]
#[test]
fn reuse_runs_the_selected_pinned_cli_and_returns_its_exit_after_private_snapshot_cleanup()
-> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let fixture = tempfile::tempdir()?;
    let root = fixture.path();
    git(root, &["init", "-q"])?;
    write(root, ".gitignore", ".venv/\nsnapshot-location\n")?;
    write(root, "source.rs", "retained source")?;
    write(
        root,
        ".venv/bin/reuse",
        "#!/bin/sh\n[ \"$1\" = --root ] && [ \"$3\" = lint ] || exit 90\n[ -f \"$2/source.rs\" ] || exit 91\n[ ! -e \"$2/.venv\" ] || exit 92\nprintf '%s' \"$2\" > snapshot-location\nexit 7\n",
    )?;
    fs::set_permissions(
        root.join(".venv/bin/reuse"),
        fs::Permissions::from_mode(0o700),
    )?;
    let status = reuse_lint(root)?;
    assert_eq!(status.code(), Some(7));
    let location = fs::read_to_string(root.join("snapshot-location"))?;
    assert!(
        !Path::new(&location).exists(),
        "private source snapshot was not reclaimed"
    );
    assert_eq!(
        fs::read_to_string(root.join("source.rs"))?,
        "retained source"
    );
    Ok(())
}
