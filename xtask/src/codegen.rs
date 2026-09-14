// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Filesystem boundary for pure schema generators (ADR-0051).

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use pse_schema::codegen::{GeneratedTree, Language};
use sha2::{Digest, Sha256};

use crate::Target;

const BINDINGS: &str = "crates/pse-ipopt-sys/src/bindings.rs";

pub(super) fn run(root: &Path, check: bool, only: Option<Target>) -> Result<()> {
    if only == Some(Target::Bindgen) {
        if std::env::var_os("IPOPT_DIR").is_none() {
            eprintln!(
                "bindgen requires IPOPT_DIR from the solver container; the arm remains deferred (R-3)"
            );
            std::process::exit(2);
        }
        if !check {
            eprintln!(
                "bindgen generation requires the deferred solver-container arm (R-3); IPOPT_DIR alone does not implement it"
            );
            std::process::exit(2);
        }
        return bindings_hygiene(root);
    }
    let languages = match only {
        Some(Target::Relations) => vec![Language::Rust],
        Some(Target::Python) => vec![Language::Python],
        Some(Target::Docs) => vec![Language::Markdown],
        Some(Target::Bindgen) => return bindings_hygiene(root),
        None => Language::ALL.to_vec(),
    };
    let registry = pse_schema::registry()?;
    let mut trees = Vec::new();
    for language in languages {
        let mut tree = pse_schema::codegen::generate(registry, language)?;
        if tree.roots != language.roots() {
            bail!(
                "{} generator changed its declared output roots",
                language.as_str()
            );
        }
        if language == Language::Python {
            add_python_manifest(&mut tree)?;
        }
        validate_tree(&tree)?;
        trees.push(tree);
    }
    for tree in &trees {
        if check {
            let scratch = tempfile::tempdir().context("creating codegen comparison directory")?;
            write_tree(scratch.path(), tree)?;
            compare_tree(root, scratch.path(), tree)?;
            untracked_check(root, &tree.roots)?;
        } else {
            write_tree(root, tree)?;
        }
    }
    if only.is_none() {
        if check {
            bindings_hygiene(root)?;
        } else {
            println!("codegen: bindgen skipped; R-3 remains deferred (solver-container generator)");
        }
    }
    println!(
        "codegen{}: OK ({} schema target(s))",
        if check { " --check" } else { "" },
        trees.len()
    );
    Ok(())
}

fn add_python_manifest(tree: &mut GeneratedTree) -> Result<()> {
    let root = Path::new("python/pse/contracts");
    let manifest_path = root.join("GENERATED.sha256");
    if tree.files.contains_key(&manifest_path) {
        bail!("the Python generator must leave GENERATED.sha256 to the xtask writer");
    }
    let mut manifest = String::new();
    for (path, bytes) in &tree.files {
        let relative = path
            .strip_prefix(root)
            .context("Python output escaped its root")?;
        let name = relative
            .to_str()
            .context("generated filename is not UTF-8")?
            .replace('\\', "/");
        for byte in Sha256::digest(bytes) {
            write!(manifest, "{byte:02x}")?;
        }
        writeln!(manifest, "  {name}")?;
    }
    tree.files.insert(manifest_path, manifest.into_bytes());
    Ok(())
}

fn normal_relative(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn validate_tree(tree: &GeneratedTree) -> Result<()> {
    for root in &tree.roots {
        if !normal_relative(root) {
            bail!("invalid generated root: {}", root.display());
        }
    }
    for path in tree.files.keys() {
        if !normal_relative(path)
            || !tree
                .roots
                .iter()
                .any(|root| path != root && path.starts_with(root))
        {
            bail!(
                "generated file lies outside the declared roots: {}",
                path.display()
            );
        }
    }
    Ok(())
}

fn reject_symlinks(root: &Path, relative: &Path) -> Result<()> {
    let mut path = root.to_owned();
    for component in relative.components() {
        path.push(component.as_os_str());
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                bail!("generated path traverses a symlink: {}", path.display())
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error).with_context(|| format!("inspecting {}", path.display()));
            }
        }
    }
    Ok(())
}

fn inventory(root: &Path, roots: &[PathBuf]) -> Result<BTreeSet<PathBuf>> {
    let mut files = BTreeSet::new();
    for relative in roots {
        reject_symlinks(root, relative)?;
        collect_files(root, relative, &mut files)?;
    }
    Ok(files)
}

fn collect_files(root: &Path, relative: &Path, files: &mut BTreeSet<PathBuf>) -> Result<()> {
    let path = root.join(relative);
    let metadata = match fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error).with_context(|| format!("reading {}", path.display())),
    };
    if metadata.file_type().is_symlink() {
        bail!("generated tree contains a symlink: {}", path.display());
    }
    if metadata.is_file() {
        files.insert(relative.to_owned());
    } else if metadata.is_dir() {
        for entry in fs::read_dir(path)? {
            collect_files(root, &relative.join(entry?.file_name()), files)?;
        }
    } else {
        bail!("generated tree contains a non-file: {}", path.display());
    }
    Ok(())
}

fn write_tree(root: &Path, tree: &GeneratedTree) -> Result<()> {
    validate_tree(tree)?;
    let existing = inventory(root, &tree.roots)?;
    for path in tree.files.keys() {
        reject_symlinks(root, path)?;
    }
    for (path, bytes) in &tree.files {
        let destination = root.join(path);
        let parent = destination
            .parent()
            .context("generated file has no parent")?;
        fs::create_dir_all(parent)?;
        fs::write(&destination, bytes)
            .with_context(|| format!("writing {}", destination.display()))?;
    }
    for stale in existing
        .iter()
        .filter(|path| !tree.files.contains_key(*path))
    {
        fs::remove_file(root.join(stale))
            .with_context(|| format!("removing stale generated file {}", stale.display()))?;
    }
    Ok(())
}

fn compare_tree(root: &Path, scratch: &Path, tree: &GeneratedTree) -> Result<()> {
    let expected = inventory(scratch, &tree.roots)?;
    let actual = inventory(root, &tree.roots)?;
    let mut differences = Vec::new();
    for path in expected.union(&actual) {
        let difference = match (expected.contains(path), actual.contains(path)) {
            (true, false) => Some("missing from checkout"),
            (false, true) => Some("stale in checkout"),
            (true, true) if fs::read(root.join(path))? != fs::read(scratch.join(path))? => {
                Some("bytes differ")
            }
            _ => None,
        };
        if let Some(reason) = difference {
            differences.push(format!("{}: {reason}", path.display()));
        }
    }
    if !differences.is_empty() {
        bail!(
            "codegen --check found {} difference(s):\n{}",
            differences.len(),
            differences.join("\n")
        );
    }
    Ok(())
}

fn untracked_check(root: &Path, paths: &[PathBuf]) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "--others", "--exclude-standard", "--"])
        .args(paths)
        .output()
        .context("checking untracked generated files")?;
    if !output.status.success() {
        bail!("git ls-files failed while checking generated paths");
    }
    if !output.stdout.is_empty() {
        bail!(
            "generated files must be tracked:\n{}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
    Ok(())
}

fn bindings_hygiene(root: &Path) -> Result<()> {
    let status = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["diff", "HEAD", "--exit-code", "--stat", "--", BINDINGS])
        .status()
        .context("checking deferred bindgen output")?;
    if !status.success() {
        bail!("bindgen output differs from HEAD; the deferred arm checks hygiene only");
    }
    untracked_check(root, &[PathBuf::from(BINDINGS)])?;
    println!("codegen: bindgen hygiene only; regeneration remains deferred (R-3)");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree() -> GeneratedTree {
        GeneratedTree {
            roots: vec![PathBuf::from("generated")],
            files: [(PathBuf::from("generated/a.txt"), b"exact bytes\n".to_vec())].into(),
        }
    }

    #[test]
    fn comparison_detects_changes_missing_and_extra_files_without_writing_checkout() {
        let checkout = tempfile::tempdir().unwrap();
        let scratch = tempfile::tempdir().unwrap();
        let tree = tree();
        write_tree(checkout.path(), &tree).unwrap();
        write_tree(scratch.path(), &tree).unwrap();
        compare_tree(checkout.path(), scratch.path(), &tree).unwrap();
        let path = checkout.path().join("generated/a.txt");
        fs::write(&path, b"changed").unwrap();
        assert!(compare_tree(checkout.path(), scratch.path(), &tree).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"changed");
        fs::remove_file(&path).unwrap();
        assert!(compare_tree(checkout.path(), scratch.path(), &tree).is_err());
        assert!(!path.exists());
        write_tree(checkout.path(), &tree).unwrap();
        fs::write(checkout.path().join("generated/stale.txt"), b"stale").unwrap();
        assert!(compare_tree(checkout.path(), scratch.path(), &tree).is_err());
        write_tree(checkout.path(), &tree).unwrap();
        compare_tree(checkout.path(), scratch.path(), &tree).unwrap();
    }

    #[test]
    fn output_cannot_escape_its_declared_root() {
        let mut tree = tree();
        tree.files
            .insert(PathBuf::from("generated/../../other"), vec![]);
        assert!(validate_tree(&tree).is_err());
        tree.files.clear();
        tree.files.insert(PathBuf::from("other/file"), vec![]);
        assert!(validate_tree(&tree).is_err());
    }

    #[test]
    fn python_manifest_lists_exact_generated_bytes_and_excludes_itself() {
        let mut tree = GeneratedTree::empty(Language::Python.roots());
        tree.files
            .insert(PathBuf::from("python/pse/contracts/a.py"), b"abc".to_vec());
        add_python_manifest(&mut tree).unwrap();
        assert_eq!(
            tree.files[Path::new("python/pse/contracts/GENERATED.sha256")],
            b"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad  a.py\n"
        );
    }

    #[test]
    fn equivalent_output_still_requires_tracked_files() {
        let checkout = tempfile::tempdir().unwrap();
        assert!(
            Command::new("git")
                .arg("-C")
                .arg(checkout.path())
                .args(["init", "-q"])
                .status()
                .unwrap()
                .success()
        );
        let tree = tree();
        write_tree(checkout.path(), &tree).unwrap();
        assert!(untracked_check(checkout.path(), &tree.roots).is_err());
        assert!(
            Command::new("git")
                .arg("-C")
                .arg(checkout.path())
                .args(["add", "generated"])
                .status()
                .unwrap()
                .success()
        );
        untracked_check(checkout.path(), &tree.roots).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn writer_refuses_symlinked_output_paths() {
        let checkout = tempfile::tempdir().unwrap();
        let other = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(other.path(), checkout.path().join("generated")).unwrap();
        assert!(write_tree(checkout.path(), &tree()).is_err());
        assert!(!other.path().join("a.txt").exists());
    }
}
