// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded reads of private checkout operation records.
use crate::HostError;
use std::{
    fs::{self, OpenOptions},
    io::Read as _,
    os::unix::fs::{MetadataExt as _, OpenOptionsExt as _},
    path::Path,
};

pub(super) fn directory(path: &Path) -> Result<(), HostError> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_dir()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o077 != 0
    {
        return Err(HostError::Ownership(
            "checkout operation directory is not owner-private".into(),
        ));
    }
    Ok(())
}

pub(super) fn read(path: &Path, maximum: usize) -> Result<Vec<u8>, HostError> {
    let flags = rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK;
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(flags.bits() as i32)
        .open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o077 != 0
        || metadata.len() > maximum as u64
    {
        return Err(HostError::Ownership(
            "checkout operation record is not bounded and owner-private".into(),
        ));
    }
    let mut bytes = Vec::new();
    file.take(maximum as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > maximum {
        return Err(HostError::Ownership(
            "checkout operation record exceeds its extent".into(),
        ));
    }
    Ok(bytes)
}
