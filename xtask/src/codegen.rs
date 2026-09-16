// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Filesystem boundary for pure schema generators (ADR-0051).

#[cfg(feature = "package-fixtures")]
mod physical;

mod ipopt;
pub(super) mod python_stubs;

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
const PYTHON_ROOT: &str = "python/pse/contracts";

pub(super) fn run(root: &Path, check: bool, only: Option<Target>) -> Result<()> {
    let languages = match only {
        Some(Target::Relations | Target::RustContracts) => vec![Language::Rust],
        Some(Target::Python) => vec![Language::Python],
        Some(Target::Docs) => vec![Language::Markdown],
        Some(Target::Bindgen) => return ipopt::run(root, check),
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
        if language == Language::Rust {
            if only == Some(Target::RustContracts) {
                contract_roots(&mut tree);
            } else {
                append_physical(root, registry, &mut tree)?;
            }
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
        ipopt::run(root, check)?;
    }

    println!(
        "codegen{}: OK ({} schema target(s))",
        if check { " --check" } else { "" },
        trees.len()
    );
    Ok(())
}

#[cfg(feature = "package-fixtures")]
fn append_physical(
    root: &Path,
    registry: &pse_schema::Registry,
    tree: &mut GeneratedTree,
) -> Result<()> {
    physical::append(root, registry, tree)
}

#[cfg(not(feature = "package-fixtures"))]
fn append_physical(_: &Path, _: &pse_schema::Registry, _: &mut GeneratedTree) -> Result<()> {
    bail!(
        "complete Rust generation requires package-fixtures; use codegen-bootstrap to rebuild its admitted package loader"
    )
}

// The pure registry emitter has no physical package data. Its bootstrap phase
// must not claim those empty roots and prune the previously compiled fixtures.
fn contract_roots(tree: &mut GeneratedTree) {
    tree.roots
        .retain(|root| tree.files.keys().any(|path| path.starts_with(root)));
}

fn add_python_manifest(tree: &mut GeneratedTree) -> Result<()> {
    let root = Path::new(PYTHON_ROOT);
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

fn inventory(root: &Path, tree: &GeneratedTree) -> Result<BTreeSet<PathBuf>> {
    let mut files = BTreeSet::new();
    for relative in &tree.roots {
        reject_symlinks(root, relative)?;
        collect_files(root, relative, &mut files)?;
    }
    // Inspect every path first: a cache directory cannot hide symlinks or other
    // non-files. Explicit generator output always remains subject to comparison.
    files.retain(|path| tree.files.contains_key(path) || !python_bytecode_path(path));
    Ok(files)
}

fn python_bytecode_path(relative: &Path) -> bool {
    if !relative.starts_with(PYTHON_ROOT)
        || relative.parent().and_then(Path::file_name) != Some("__pycache__".as_ref())
    {
        return false;
    }
    let Some(stem) = relative
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_suffix(".pyc"))
    else {
        return false;
    };
    let Some((module, tag)) = stem.rsplit_once(".cpython-") else {
        return false;
    };
    let (version, optimization) = tag
        .split_once(".opt-")
        .map_or((tag, None), |(version, level)| (version, Some(level)));
    !module.is_empty()
        && !version.is_empty()
        && version.bytes().all(|byte| byte.is_ascii_digit())
        && optimization.is_none_or(|level| {
            !level.is_empty() && level.bytes().all(|byte| byte.is_ascii_alphanumeric())
        })
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
    let existing = inventory(root, tree)?;
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
    let expected = inventory(scratch, tree)?;
    let actual = inventory(root, tree)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_bootstrap_preserves_package_fixture_outputs() {
        let checkout = tempfile::tempdir().unwrap();
        let mut tree = GeneratedTree::empty(Language::Rust.roots());
        tree.files.insert(
            PathBuf::from("crates/pse-authoring/src/generated/mod.rs"),
            b"new contracts".to_vec(),
        );
        let fixture = checkout
            .path()
            .join("crates/pse-quantity/src/generated/mod.rs");
        fs::create_dir_all(fixture.parent().unwrap()).unwrap();
        fs::write(&fixture, b"retained fixture").unwrap();
        contract_roots(&mut tree);
        write_tree(checkout.path(), &tree).unwrap();
        assert_eq!(fs::read(fixture).unwrap(), b"retained fixture");
        assert_eq!(
            tree.roots,
            vec![PathBuf::from("crates/pse-authoring/src/generated")]
        );
    }

    fn tree() -> GeneratedTree {
        GeneratedTree {
            roots: vec![PathBuf::from("generated")],
            files: [(PathBuf::from("generated/a.txt"), b"exact bytes\n".to_vec())].into(),
        }
    }

    fn python_tree() -> GeneratedTree {
        GeneratedTree {
            roots: vec![PathBuf::from(PYTHON_ROOT)],
            files: [(Path::new(PYTHON_ROOT).join("a.py"), b"value = 1\n".to_vec())].into(),
        }
    }

    #[test]
    fn python_caches_survive_writing_and_do_not_hide_source_changes() {
        let checkout = tempfile::tempdir().unwrap();
        let scratch = tempfile::tempdir().unwrap();
        let tree = python_tree();
        write_tree(checkout.path(), &tree).unwrap();
        write_tree(scratch.path(), &tree).unwrap();
        let cache = checkout.path().join(PYTHON_ROOT).join("__pycache__");
        fs::create_dir(&cache).unwrap();
        let caches = [
            "a.cpython-313.pyc",
            "a.cpython-314.pyc",
            "a.cpython-313.opt-1.pyc",
            "a.cpython-314.opt-2.pyc",
        ];
        for name in caches {
            fs::write(cache.join(name), name.as_bytes()).unwrap();
        }
        compare_tree(checkout.path(), scratch.path(), &tree).unwrap();
        let source = checkout.path().join(PYTHON_ROOT).join("a.py");
        fs::write(&source, b"changed source\n").unwrap();
        assert!(compare_tree(checkout.path(), scratch.path(), &tree).is_err());
        assert_eq!(fs::read(&source).unwrap(), b"changed source\n");
        fs::remove_file(&source).unwrap();
        assert!(compare_tree(checkout.path(), scratch.path(), &tree).is_err());
        assert!(!source.exists());
        write_tree(checkout.path(), &tree).unwrap();
        let stale = checkout.path().join(PYTHON_ROOT).join("stale.py");
        fs::write(&stale, b"stale source\n").unwrap();
        assert!(compare_tree(checkout.path(), scratch.path(), &tree).is_err());
        write_tree(checkout.path(), &tree).unwrap();
        assert!(!stale.exists());
        compare_tree(checkout.path(), scratch.path(), &tree).unwrap();
        for name in caches {
            assert_eq!(fs::read(cache.join(name)).unwrap(), name.as_bytes());
        }
    }

    #[test]
    fn cache_directories_do_not_hide_unexpected_source_or_malformed_cache_names() {
        let checkout = tempfile::tempdir().unwrap();
        let scratch = tempfile::tempdir().unwrap();
        let tree = python_tree();
        write_tree(checkout.path(), &tree).unwrap();
        write_tree(scratch.path(), &tree).unwrap();
        let cache = checkout.path().join(PYTHON_ROOT).join("__pycache__");
        fs::create_dir(&cache).unwrap();
        for name in [
            "unexpected.py",
            "unexpected.txt",
            "a.pyc",
            "a.cpython-.pyc",
            "a.cpython-source.pyc",
            "a.cpython-314.opt-.pyc",
            "a.cpython-314.pyc.py",
        ] {
            let path = cache.join(name);
            fs::write(&path, b"unexpected source\n").unwrap();
            assert!(compare_tree(checkout.path(), scratch.path(), &tree).is_err());
            assert!(path.is_file());
            write_tree(checkout.path(), &tree).unwrap();
            assert!(!path.exists());
        }
        compare_tree(checkout.path(), scratch.path(), &tree).unwrap();
    }

    #[test]
    fn bytecode_exclusion_requires_the_python_root_and_immediate_cache_parent() {
        for relative in [
            "docs/generated/__pycache__/a.cpython-314.pyc",
            "python/pse/contracts/a.cpython-314.pyc",
            "python/pse/contracts/__pycache__/nested/a.cpython-314.pyc",
        ] {
            let checkout = tempfile::tempdir().unwrap();
            let path = checkout.path().join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, b"unexpected file\n").unwrap();
            let tree = GeneratedTree::empty(vec![
                PathBuf::from("docs/generated"),
                PathBuf::from(PYTHON_ROOT),
            ]);
            assert_eq!(
                inventory(checkout.path(), &tree).unwrap(),
                [PathBuf::from(relative)].into()
            );
        }
    }

    #[test]
    fn explicit_generator_outputs_are_compared_even_with_bytecode_names() {
        let checkout = tempfile::tempdir().unwrap();
        let scratch = tempfile::tempdir().unwrap();
        let mut tree = python_tree();
        let relative = Path::new(PYTHON_ROOT).join("__pycache__/a.cpython-314.pyc");
        tree.files
            .insert(relative.clone(), b"declared bytes".to_vec());
        write_tree(checkout.path(), &tree).unwrap();
        write_tree(scratch.path(), &tree).unwrap();
        compare_tree(checkout.path(), scratch.path(), &tree).unwrap();
        let path = checkout.path().join(relative);
        fs::write(&path, b"changed bytes").unwrap();
        assert!(compare_tree(checkout.path(), scratch.path(), &tree).is_err());
        fs::remove_file(path).unwrap();
        assert!(compare_tree(checkout.path(), scratch.path(), &tree).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn bytecode_names_and_cache_directories_cannot_hide_symlinks() {
        let checkout = tempfile::tempdir().unwrap();
        let scratch = tempfile::tempdir().unwrap();
        let external = tempfile::tempdir().unwrap();
        let tree = python_tree();
        write_tree(checkout.path(), &tree).unwrap();
        write_tree(scratch.path(), &tree).unwrap();
        let cache = checkout.path().join(PYTHON_ROOT).join("__pycache__");
        fs::create_dir(&cache).unwrap();
        let target = external.path().join("untouched");
        fs::write(&target, b"external source\n").unwrap();
        let link = cache.join("a.cpython-314.pyc");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        assert!(compare_tree(checkout.path(), scratch.path(), &tree).is_err());
        assert!(write_tree(checkout.path(), &tree).is_err());
        assert!(fs::symlink_metadata(&link).unwrap().is_symlink());
        assert_eq!(fs::read(&target).unwrap(), b"external source\n");
        fs::remove_file(link).unwrap();
        fs::remove_dir(&cache).unwrap();
        std::os::unix::fs::symlink(external.path(), &cache).unwrap();
        assert!(compare_tree(checkout.path(), scratch.path(), &tree).is_err());
        assert!(write_tree(checkout.path(), &tree).is_err());
        assert_eq!(fs::read(target).unwrap(), b"external source\n");
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
