// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Release only the exact Cargo output directory of an authenticated authored probe.
use super::{ProbeError, host, invalid, settled_probe};
use crate::environment::Environment;
use rustix::{
    fd::AsFd,
    fs::{self as unix, AtFlags, Dir, FileType, Mode, OFlags, ResolveFlags},
};
use schemars::JsonSchema;
use serde::Serialize;
use std::{
    collections::BTreeSet,
    fs::File,
    io,
    os::unix::fs::{MetadataExt as _, OpenOptionsExt as _},
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

/// Original Cargo output selection and allocated footprint, independent of filesystem free space.
#[derive(Debug, Serialize, JsonSchema)]
pub struct OutputRelease {
    /// Checkout-state path selected by the probe owner.
    pub target: PathBuf,
    /// Exact directory identity required to perform the previewed release.
    pub identity: Option<String>,
    /// Allocated blocks observed once per inode before release.
    pub allocated_bytes: u64,
    /// Whether the selected directory was removed in this operation.
    pub released: bool,
}

pub(crate) fn directory(parent: impl AsFd, name: impl rustix::path::Arg) -> io::Result<File> {
    unix::openat2(
        parent,
        name,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
        Mode::empty(),
        ResolveFlags::BENEATH | ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_XDEV,
    )
    .map(File::from)
    .map_err(io::Error::from)
}

fn walk(parent: &File, seen: &mut BTreeSet<(u64, u64)>, release: bool) -> io::Result<u64> {
    let mut bytes = 0_u64;
    for entry in Dir::read_from(parent)? {
        let entry = entry?;
        let name = entry.file_name();
        if matches!(name.to_bytes(), b"." | b"..") {
            continue;
        }
        let facts = unix::statat(parent, name, AtFlags::SYMLINK_NOFOLLOW)?;
        if seen.insert((facts.st_dev, facts.st_ino)) {
            bytes = bytes.saturating_add(
                u64::try_from(facts.st_blocks)
                    .unwrap_or(0)
                    .saturating_mul(512),
            );
        }
        if FileType::from_raw_mode(facts.st_mode) == FileType::Directory {
            let child = directory(parent, name)?;
            bytes = bytes.saturating_add(walk(&child, seen, release)?);
            if release {
                unix::unlinkat(parent, name, AtFlags::REMOVEDIR)?;
            }
        } else if release {
            unix::unlinkat(parent, name, AtFlags::empty())?;
        }
    }
    Ok(bytes)
}

pub(crate) fn identity(directory: &File) -> io::Result<String> {
    let metadata = directory.metadata()?;
    let created = metadata
        .created()?
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?;
    Ok(format!(
        "{}-{}-{}-{}",
        metadata.dev(),
        metadata.ino(),
        created.as_secs(),
        created.subsec_nanos()
    ))
}

/// Inspect or unlink the owner's fixed target under its held parent and lease.
pub(crate) fn target_output(
    parent: &File,
    target_path: PathBuf,
    owned_identity: Option<&str>,
    release: bool,
) -> io::Result<OutputRelease> {
    let mut result = OutputRelease {
        target: target_path,
        identity: None,
        allocated_bytes: 0,
        released: false,
    };
    let target = match directory(parent, c"target") {
        Ok(target) => target,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(result),
        Err(error) => return Err(error),
    };
    let selected = identity(&target)?;
    if owned_identity.is_some_and(|expected| expected != selected) {
        return Err(io::Error::other("Cargo output identity changed after selection"));
    }
    let metadata = target.metadata()?;
    let mut seen = BTreeSet::from([(metadata.dev(), metadata.ino())]);
    result.allocated_bytes = metadata
        .blocks()
        .saturating_mul(512)
        .saturating_add(walk(&target, &mut seen, false)?);
    result.identity = Some(selected.clone());
    if release {
        if owned_identity.is_none() || identity(&directory(parent, c"target")?)? != selected {
            return Err(io::Error::other("Cargo output identity is unverified during release"));
        }
        walk(&target, &mut BTreeSet::new(), true)?;
        if identity(&directory(parent, c"target")?)? != selected {
            return Err(io::Error::other("Cargo output identity changed during release"));
        }
        unix::unlinkat(parent, c"target", AtFlags::REMOVEDIR).map_err(io::Error::from)?;
        result.released = true;
    }
    Ok(result)
}

/// Preview or explicitly release regenerable Cargo output after original probe users settle.
/// The authored source, lockfile, original invocation references, and diagnostic records remain.
pub async fn clean_outputs(
    root: &Path,
    store: &Path,
    name: &str,
    environment: &Environment,
    expected: Option<&str>,
) -> Result<OutputRelease, ProbeError> {
    super::name_component(name)?;
    let probe_path = store.join("probes").join(name);
    if !std::fs::symlink_metadata(&probe_path)?.is_dir() {
        return Err(invalid(
            "authored probe must be an existing owned directory",
        ));
    }
    let (probe, _lease) = settled_probe(root, store, name, environment, |reference| async move {
        crate::foreground::observe_recorded_drain(
            Path::new(&reference.checkout),
            &reference.invocation,
            environment,
        )
        .await
        .map_err(host)
    })
    .await?;
    let parent = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags((OFlags::DIRECTORY | OFlags::NOFOLLOW).bits() as i32)
        .open(&probe.directory)?;
    Ok(target_output(
        &parent,
        probe.target_directory,
        expected,
        expected.is_some(),
    )?)
}
