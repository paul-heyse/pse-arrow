// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Product-owned contained exchange and bounded scientific IPC admission.
//! Callers retain selection, reader protection and final publication authority.
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    path::{Component, Path},
};

pub(crate) fn safe_relative(path: &str) -> std::io::Result<()> {
    if path.is_empty()
        || path.contains('\\')
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        || Path::new(path)
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "unsafe indexed relative path",
        ));
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
        let flags = OFlags::RDONLY
            | OFlags::NOFOLLOW
            | OFlags::CLOEXEC
            | OFlags::NONBLOCK
            | if components.peek().is_some() {
                OFlags::DIRECTORY
            } else {
                OFlags::empty()
            };
        directory = File::from(
            rustix::fs::openat(&directory, component.as_os_str(), flags, Mode::empty())
                .map_err(std::io::Error::from)?,
        );
    }
    if !directory.metadata()?.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "indexed member is not an ordinary local file",
        ));
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
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "source shorter than acknowledged readable prefix",
            ));
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
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root.join(temporary))?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    checkpoint()?;
    std::fs::rename(root.join(temporary), root.join(destination))?;
    File::open(root)?.sync_all()
}

use super::PackResultSelection;

fn invalid(reason: impl Into<String>) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, reason.into())
}

/// Validate every bounded message's schema/layout/framing and exact original lineage.
/// Metadata validation never materializes record arrays.
pub(crate) fn validate_scientific_ipc(
    file: File,
    readable_bytes: u64,
    maximum_document_bytes: usize,
    result: &PackResultSelection,
) -> std::io::Result<()> {
    let schema = finalized_ipc_schema(file, readable_bytes, maximum_document_bytes)?;
    validate_scientific_lineage(schema.as_ref(), result)
}
pub(crate) fn finalized_ipc_schema(
    mut file: File,
    readable_bytes: u64,
    maximum_document_bytes: usize,
) -> std::io::Result<pse_columnar::Leased<datafusion::arrow::datatypes::Schema>> {
    let limits = pse_columnar::ipc_stream::Limits {
        message_bytes: maximum_document_bytes.max(pse_operations::canonical_files::FILE_BYTES),
        metadata_bytes: maximum_document_bytes,
    };
    let pool = pse_columnar::ipc_stream::offline_pool(limits).map_err(|e| invalid(e.to_string()))?;
    file.seek(SeekFrom::Start(0))?;
    let stream = pse_columnar::ipc_stream::validate(
        file, readable_bytes, limits, pool, pse_columnar::CancellationToken::new(),
    ).map_err(|e| invalid(e.to_string()))?;
    Ok(pse_columnar::Leased::new(stream.schema(), stream.schema_owner()))
}

pub(crate) fn validate_scientific_lineage(
    schema: &datafusion::arrow::datatypes::Schema,
    result: &PackResultSelection,
) -> std::io::Result<()> {
    if result.schema_version == 0
        || result.row_start > result.row_end
        || result.run.is_empty()
        || result.attempt.is_empty()
        || result.relation.is_empty()
    {
        return Err(invalid("invalid selected scientific coordinates"));
    }
    if let Some(predicate) = &result.predicate {
        super::result_selection::ValuePredicate::new(false,
            predicate.minimum_bits.map(f64::from_bits), predicate.maximum_bits.map(f64::from_bits), predicate.missing)
            .map_err(|error| invalid(error.to_string()))?;
    }
    for (key, expected) in super::result_selection::ScientificMetadata::from(result).entries() {
        if schema.metadata().get(key) != expected.as_ref() {
            return Err(invalid(format!("IPC lineage differs from selected {key}")));
        }
    }
    Ok(())
}
