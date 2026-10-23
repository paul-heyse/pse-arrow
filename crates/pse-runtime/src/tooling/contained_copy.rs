// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Contained local reads and bounded streaming, shared by diagnostic and logical packs.
use std::{fs::File, io::{Read, Write}, path::{Component, Path}};

pub(crate) fn safe_relative(path: &str) -> std::io::Result<()> {
    if path.is_empty() || path.contains('\\')
        || path.split('/').any(|part| part.is_empty() || part == "." || part == "..")
        || Path::new(path).components().any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "unsafe indexed relative path"));
    }
    Ok(())
}

/// Every path component is opened relative to its already opened parent without following links.
pub(crate) fn open_contained(directory: &File, relative: &str) -> std::io::Result<File> {
    use rustix::fs::{Mode, OFlags};
    safe_relative(relative)?;
    let mut directory = directory.try_clone()?;
    let mut components = Path::new(relative).components().peekable();
    while let Some(component) = components.next() {
        let flags = OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK
            | if components.peek().is_some() { OFlags::DIRECTORY } else { OFlags::empty() };
        directory = File::from(rustix::fs::openat(&directory, component.as_os_str(), flags, Mode::empty()).map_err(std::io::Error::from)?);
    }
    if !directory.metadata()?.is_file() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "indexed member is not an ordinary local file"));
    }
    Ok(directory)
}

/// Copy exactly a selected prefix with a checkpoint before every bounded read.
/// The caller owns the original protection and destination publication boundary.
pub(crate) fn copy_exact(
    source: &mut impl Read,
    output: &mut impl Write,
    bytes: u64,
    mut checkpoint: impl FnMut() -> std::io::Result<()>,
) -> std::io::Result<()> {
    let mut remaining = bytes;
    let mut buffer = [0_u8; 64 * 1024];
    while remaining != 0 {
        checkpoint()?;
        let requested = remaining.min(buffer.len() as u64) as usize;
        let read = source.read(&mut buffer[..requested])?;
        if read == 0 {
            return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "source shorter than acknowledged readable prefix"));
        }
        output.write_all(&buffer[..read])?;
        remaining -= read as u64;
    }
    checkpoint()?;
    output.flush()
}

/// Write, synchronize and atomically publish one manifest after the caller's final checkpoint.
/// Names are single contained relative paths supplied by the owning publication protocol.
pub(crate) fn publish_document(
    root: &Path,
    temporary: &str,
    destination: &str,
    bytes: &[u8],
    checkpoint: impl FnOnce() -> std::io::Result<()>,
) -> std::io::Result<()> {
    safe_relative(temporary)?;
    safe_relative(destination)?;
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(root.join(temporary))?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    checkpoint()?;
    std::fs::rename(root.join(temporary), root.join(destination))?;
    File::open(root)?.sync_all()
}
