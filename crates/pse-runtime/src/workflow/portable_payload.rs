// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Product-owned contained exchange and bounded scientific IPC admission.
//! Callers retain selection, reader protection and final publication authority.
use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    os::unix::fs::MetadataExt as _,
    path::{Component, Path, PathBuf},
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
    checkpoint: impl FnMut() -> std::io::Result<()>,
) -> std::io::Result<()> {
    transfer_exact(source, output, bytes, checkpoint)?;
    output.flush()
}

/// Transfer the exact selected extent without claiming sink flush or publication.
pub(crate) fn transfer_exact(
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
    checkpoint()
}

/// A staged output established by the actual exclusive file creation.
pub(crate) struct OwnedMemberStage {
    path: PathBuf,
    file: File,
}

/// Original writer custody and the last complete IPC message it acknowledged.
#[derive(Default)]
pub(crate) struct IpcExportAcknowledgement {
    pub(crate) bytes: u64,
    pub(crate) stage: Option<OwnedMemberStage>,
}

impl OwnedMemberStage {
    pub(crate) fn create(path: PathBuf) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)?;
        Ok(Self { path, file })
    }

    pub(crate) fn file(&mut self) -> &mut File {
        &mut self.file
    }

    pub(crate) fn reader(&self) -> std::io::Result<File> {
        let mut file = self.file.try_clone()?;
        file.rewind()?;
        Ok(file)
    }

    pub(crate) fn is_at(&self, path: &Path) -> bool {
        self.path == path
    }

    fn require_entry(&self, path: &Path) -> std::io::Result<u64> {
        let owned = self.file.metadata()?;
        let entry = std::fs::symlink_metadata(path)?;
        if !entry.is_file() || (entry.dev(), entry.ino()) != (owned.dev(), owned.ino()) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "staged member entry no longer names the original writer",
            ));
        }
        Ok(owned.len())
    }

    /// Publish the exact staged inode at a fresh path and retain that path even
    /// if independently removing the earlier staging entry fails.
    pub(crate) fn publish(&mut self, destination: &Path) -> std::io::Result<u64> {
        if self.path != destination {
            self.require_entry(&self.path)?;
            std::fs::hard_link(&self.path, destination)?;
            let bytes = self.require_entry(destination)?;
            let old = std::mem::replace(&mut self.path, destination.to_owned());
            self.require_entry(&old)?;
            std::fs::remove_file(old)?;
            Ok(bytes)
        } else {
            self.require_entry(destination)
        }
    }
}

pub(crate) struct MemberTransfer {
    pub bytes: u64,
    pub digest: pse_ids::ContentHash,
}

pub(crate) struct MemberTransferFailure {
    pub accepted_bytes: u64,
    pub source: std::io::Error,
}

struct Accepted<'a, W> {
    sink: &'a mut W,
    bytes: u64,
    digest: pse_ids::EncodingHasher,
}

impl<W: Write> Write for Accepted<'_, W> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let written = self.sink.write(bytes)?;
        if written > bytes.len() {
            return Err(std::io::Error::other(
                "sink reported more bytes than supplied",
            ));
        }
        self.digest.update(&bytes[..written]);
        self.bytes += written as u64;
        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.sink.flush()
    }
}

/// Commit a newly staged member or verify one previously committed member in a single
/// bounded pass. The caller owns containment, source authority and sink publication.
pub(crate) fn transfer_member(
    mut file: File,
    sink: &mut impl Write,
    bytes: u64,
    expected: Option<pse_ids::ContentHash>,
    mut checkpoint: impl FnMut() -> std::io::Result<()>,
) -> Result<MemberTransfer, MemberTransferFailure> {
    let mut accepted = Accepted {
        sink,
        bytes: 0,
        digest: pse_ids::EncodingHasher::new(),
    };
    let transfer = (|| {
        checkpoint()?;
        let observe = |file: &File| {
            pse_host::artifacts::descriptor_identity(file).map_err(std::io::Error::other)
        };
        let before = observe(&file)?;
        if before.bytes != bytes {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "member extent differs from committed extent",
            ));
        }
        transfer_exact(&mut file, &mut accepted, bytes, &mut checkpoint)?;
        let mut extra = [0];
        if file.read(&mut extra)? != 0 || observe(&file)? != before {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "member descriptor changed during transfer",
            ));
        }
        let digest = accepted.digest.clone().finish().content_hash();
        if expected.is_some_and(|expected| digest != expected) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "transferred bytes differ from the original member commitment",
            ));
        }
        checkpoint()?;
        Ok(MemberTransfer {
            bytes: accepted.bytes,
            digest,
        })
    })();
    transfer.map_err(|source| MemberTransferFailure {
        accepted_bytes: accepted.bytes,
        source,
    })
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
    let mut file = OpenOptions::new()
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
    file: impl Read + Seek,
    readable_bytes: u64,
    maximum_document_bytes: usize,
    result: &PackResultSelection,
) -> std::io::Result<()> {
    let schema = finalized_ipc_schema(file, readable_bytes, maximum_document_bytes)?;
    validate_scientific_lineage(schema.as_ref(), result)
}
pub(crate) fn finalized_ipc_schema(
    mut file: impl Read + Seek,
    readable_bytes: u64,
    maximum_document_bytes: usize,
) -> std::io::Result<pse_columnar::Leased<datafusion::arrow::datatypes::Schema>> {
    let limits = pse_columnar::ipc_stream::Limits {
        message_bytes: maximum_document_bytes.max(pse_operations::canonical_files::FILE_BYTES),
        metadata_bytes: maximum_document_bytes,
    };
    let pool =
        pse_columnar::ipc_stream::offline_pool(limits).map_err(|e| invalid(e.to_string()))?;
    file.seek(SeekFrom::Start(0))?;
    let stream = pse_columnar::ipc_stream::validate(
        file,
        readable_bytes,
        limits,
        pool,
        pse_columnar::CancellationToken::new(),
    )
    .map_err(|e| invalid(e.to_string()))?;
    Ok(pse_columnar::Leased::new(
        stream.schema(),
        stream.schema_owner(),
    ))
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
        super::result_selection::ValuePredicate::new(
            false,
            predicate.minimum_bits.map(f64::from_bits),
            predicate.maximum_bits.map(f64::from_bits),
            predicate.missing,
        )
        .map_err(|error| invalid(error.to_string()))?;
    }
    super::result_selection::validate_selection_metadata(schema.metadata(), result)
}
