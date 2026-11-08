// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact working inputs and disposable, independently built checkout.
use super::{Scope, delta, hex};
use crate::codegen::{normal_relative, publication, reject_symlinks};
use anyhow::{Context, Result, bail, ensure};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::{self, Read as _, Write as _},
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
};

const LOCAL_CONFIGURATION: &str = ".envrc.local";

pub(super) fn git(root: &Path) -> Command {
    let mut command = Command::new("git");
    command.current_dir(root);
    for variable in [
        "GIT_DIR",
        "GIT_COMMON_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        "GIT_NAMESPACE",
    ] {
        command.env_remove(variable);
    }
    command
}

fn git_output(root: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let output = git(root).args(args).output()?;
    ensure!(
        output.status.success(),
        "Git {} failed ({})",
        args[0],
        output.status
    );
    Ok(output.stdout)
}

pub(super) fn revision(root: &Path) -> Result<String> {
    Ok(
        String::from_utf8(git_output(root, &["rev-parse", "--verify", "HEAD"])?)?
            .trim()
            .to_owned(),
    )
}

pub(super) fn paths(root: &Path) -> Result<BTreeSet<PathBuf>> {
    let output = git_output(
        root,
        &[
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ],
    )?;
    output
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .map(|path| {
            let path = PathBuf::from(
                std::str::from_utf8(path)
                    .context("source workflow requires UTF-8 checkout paths")?,
            );
            ensure!(
                normal_relative(&path),
                "Git returned an unsafe working path"
            );
            Ok(path)
        })
        .collect()
}

fn file_set(root: &Path, scopes: &[Scope]) -> Result<BTreeSet<PathBuf>> {
    let mut paths = paths(root)?;
    paths.extend(delta::inventory(root, scopes)?);
    paths.insert(LOCAL_CONFIGURATION.into());
    Ok(paths)
}

fn symlink(root: &Path, path: &Path) -> Result<PathBuf> {
    let target = fs::read_link(root.join(path))?;
    let mut depth = path.components().count().saturating_sub(1);
    for part in target.components() {
        match part {
            Component::Normal(_) => depth += 1,
            Component::CurDir => (),
            Component::ParentDir if depth > 0 => depth -= 1,
            _ => bail!(
                "source link escapes the isolated checkout: {}",
                path.display()
            ),
        }
    }
    Ok(target)
}

fn mode(metadata: &fs::Metadata) -> u32 {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        metadata.permissions().mode() & 0o777
    }
    #[cfg(not(unix))]
    u32::from(metadata.permissions().readonly())
}

pub(super) fn fingerprint(
    root: &Path,
    scopes: &[Scope],
    exclude_generated: bool,
) -> Result<String> {
    let mut digest = Sha256::new();
    for path in file_set(root, scopes)? {
        if exclude_generated && scopes.iter().any(|scope| scope.contains(&path)) {
            continue;
        }
        reject_symlinks(root, path.parent().context("source path has no parent")?)?;
        let absolute = root.join(&path);
        let metadata = match fs::symlink_metadata(&absolute) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(error).with_context(|| format!("inspecting {}", path.display()));
            }
        };
        digest.update(path.as_os_str().as_encoded_bytes());
        digest.update([0]);
        digest.update(mode(&metadata).to_le_bytes());
        if metadata.is_file() {
            digest.update(b"file\0");
            digest.update(publication::digest(File::open(&absolute)?)?);
        } else if metadata.is_symlink() && path != Path::new(LOCAL_CONFIGURATION) {
            digest.update(b"symlink\0");
            digest.update(Sha256::digest(
                symlink(root, &path)?.as_os_str().as_encoded_bytes(),
            ));
        } else {
            bail!(
                "source workflow requires a regular file: {}",
                path.display()
            );
        }
    }
    Ok(hex(&digest.finalize()))
}

pub(super) fn isolated_command(root: &Path, program: &str) -> Command {
    let mut command = if program.contains('/') {
        Command::new(root.join(program))
    } else {
        Command::new(program)
    };
    command.current_dir(root);
    for key in [
        "CARGO_TARGET_DIR",
        "CARGO_BUILD_TARGET_DIR",
        "PSE_CARGO_TARGET_DIR",
        "CARGO_BUILD_DIR",
        "CARGO_BUILD_BUILD_DIR",
        "GIT_DIR",
        "GIT_COMMON_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        "GIT_NAMESPACE",
    ] {
        command.env_remove(key);
    }
    command
}

pub(super) fn create(source: &Path, root: &Path, at: &str, scopes: &[Scope]) -> Result<()> {
    let status = git(source)
        .args([
            "clone",
            "--shared",
            "--no-checkout",
            "--single-branch",
            "--no-tags",
            "--quiet",
            "--",
        ])
        .arg(source)
        .arg(root)
        .status()?;
    ensure!(
        status.success(),
        "could not initialize the isolated checkout ({status})"
    );
    git_output(root, &["reset", "--mixed", "--quiet", at])?;
    for path in file_set(source, scopes)? {
        reject_symlinks(source, path.parent().context("source path has no parent")?)?;
        let from = source.join(&path);
        let to = root.join(&path);
        let metadata = match fs::symlink_metadata(&from) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(error).with_context(|| format!("inspecting {}", path.display()));
            }
        };
        fs::create_dir_all(to.parent().context("snapshot path has no parent")?)?;
        if metadata.is_file() {
            fs::copy(&from, &to).with_context(|| format!("snapshotting {}", path.display()))?;
        } else if metadata.is_symlink() && path != Path::new(LOCAL_CONFIGURATION) {
            #[cfg(unix)]
            std::os::unix::fs::symlink(symlink(source, &path)?, &to)?;
            #[cfg(not(unix))]
            bail!("source symlinks require the supported Linux workflow");
        } else {
            bail!("unsupported snapshot input type: {}", path.display());
        }
    }
    Ok(())
}

pub(super) fn head_file_identity(root: &Path, path: &Path) -> Result<Option<String>> {
    let object = format!("HEAD:{}", path.display());
    let output = git(root)
        .args(["rev-parse", "--verify", "--quiet", &object])
        .output()?;
    if !output.status.success() {
        ensure!(
            output.status.code() == Some(1),
            "cannot inspect the selected HEAD input"
        );
        return Ok(None);
    }
    let mut child = git(root)
        .args(["cat-file", "blob", &object])
        .stdout(Stdio::piped())
        .spawn()?;
    let bytes = publication::digest(child.stdout.take().context("Git blob stream absent")?)?;
    ensure!(
        child.wait()?.success(),
        "cannot read the selected HEAD input"
    );
    Ok(Some(hex(&bytes)))
}

pub(super) fn bounded_read(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let mut file = File::open(path)?.take(limit + 1);
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= limit,
        "file exceeds its retained byte limit"
    );
    Ok(bytes)
}

pub(super) fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().context("retained state has no parent")?;
    let mut staged = tempfile::NamedTempFile::new_in(parent)?;
    staged.write_all(bytes)?;
    staged.as_file().sync_all()?;
    staged.persist(path).map_err(|error| error.error)?;
    File::open(parent)?.sync_all()?;
    Ok(())
}
