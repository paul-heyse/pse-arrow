// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded reads of an already owner-checked descriptor; no path is reopened here.
use super::{ArtifactError, Content, content};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use std::{
    fs::File,
    io::{Read as _, Seek as _, SeekFrom},
    num::NonZeroU64,
    ops::Range,
    os::unix::fs::MetadataExt as _,
};
/// Observed native file identity, independent of object-store ETags and cryptographic bytes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DescriptorIdentity {
    /// Actual filesystem device.
    pub device: u64,
    /// Actual original inode.
    pub inode: u64,
    /// Observed complete file extent.
    pub bytes: u64,
    /// Observed modification seconds/nanoseconds.
    pub modified: (i64, i64),
    /// Observed change seconds/nanoseconds.
    pub changed: (i64, i64),
}
/// Observe a regular descriptor's identity without reopening a path or granting custody.
/// Callers retain their own permissions and content-commitment policy.
pub fn identity(file: &File) -> Result<DescriptorIdentity, ArtifactError> {
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err(ArtifactError::Range(
            "selected descriptor is not a regular file".into(),
        ));
    }
    Ok(DescriptorIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
        bytes: metadata.len(),
        modified: (metadata.mtime(), metadata.mtime_nsec()),
        changed: (metadata.ctime(), metadata.ctime_nsec()),
    })
}
/// Exact selected bytes from one checked descriptor, without a path-reopen or atomic claim.
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct DescriptorRead {
    /// Original descriptor metadata checked before and after observation.
    pub identity: DescriptorIdentity,
    /// Actual half-open selected byte extent.
    pub byte_range: Range<u64>,
    /// Digest of the selected bytes only.
    pub selected_sha256: String,
    /// Whole-file digest only when explicitly scanned and checked against the requested digest.
    pub verified_sha256: Option<String>,
    /// Original selected bytes, retaining Unicode or exact binary encoding.
    pub content: Content,
    /// True only if every observed file byte is selected in this output.
    pub whole_file: bool,
    /// Explicit offset for the next bounded read.
    pub next_offset: Option<u64>,
    /// Metadata/digest validation does not establish atomic filesystem snapshot isolation.
    pub atomic_snapshot: bool,
}
fn hex(bytes: impl IntoIterator<Item = u8>) -> String {
    bytes
        .into_iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
/// Consume the same owner-checked File without reopening its path. Output is range bounded.
/// Supplying `expected_sha256` scans the entire file before the selected range; metadata is
/// checked around both reads. The caller remains responsible for custody/owner authorization.
pub fn read_descriptor(
    mut file: File,
    offset: u64,
    length: Option<NonZeroU64>,
    expected_sha256: Option<&str>,
) -> Result<DescriptorRead, ArtifactError> {
    read_descriptor_borrowed(&mut file, offset, length, expected_sha256)
}

/// Read through an exclusively borrowed checked descriptor, retaining its native owner.
pub(crate) fn read_descriptor_borrowed(
    file: &mut File,
    offset: u64,
    length: Option<NonZeroU64>,
    expected_sha256: Option<&str>,
) -> Result<DescriptorRead, ArtifactError> {
    let before = identity(file)?;
    if offset > before.bytes {
        return Err(ArtifactError::Range(
            "offset exceeds checked descriptor extent".into(),
        ));
    }
    let end = length
        .map(|length| {
            offset
                .checked_add(length.get())
                .ok_or_else(|| ArtifactError::Range("offset plus length overflows".into()))
        })
        .transpose()?
        .unwrap_or(before.bytes)
        .min(before.bytes);
    let verified_sha256 = if let Some(expected) = expected_sha256 {
        file.seek(SeekFrom::Start(0))?;
        let mut hash = Sha256::new();
        let mut buffer = [0u8; 8192];
        let mut count = 0u64;
        let maximum = before.bytes.checked_add(1).ok_or_else(|| {
            ArtifactError::Range("checked descriptor extent cannot include growth detection".into())
        })?;
        let mut bounded = (&mut *file).take(maximum);
        loop {
            let bytes = bounded.read(&mut buffer)?;
            if bytes == 0 {
                break;
            }
            count += bytes as u64;
            hash.update(&buffer[..bytes]);
        }
        let observed = hex(hash.finalize());
        if count != before.bytes || observed != expected {
            return Err(ArtifactError::Range(
                "original whole-file digest changed; selected bytes were not returned".into(),
            ));
        }
        Some(observed)
    } else {
        None
    };
    file.seek(SeekFrom::Start(offset))?;
    let mut bytes = Vec::new();
    (&mut *file).take(end - offset).read_to_end(&mut bytes)?;
    if bytes.len() as u64 != end - offset || identity(file)? != before {
        return Err(ArtifactError::Range(
            "descriptor changed or truncated during observation".into(),
        ));
    }
    Ok(DescriptorRead {
        identity: before.clone(),
        byte_range: offset..end,
        selected_sha256: hex(Sha256::digest(&bytes)),
        verified_sha256,
        content: content(&bytes),
        whole_file: offset == 0 && end == before.bytes,
        next_offset: (end < before.bytes).then_some(end),
        atomic_snapshot: false,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selected_bytes_are_descriptor_bound_binary_safe_and_digest_guarded() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("original");
        std::fs::write(&path, b"a\xfforiginal").unwrap();
        let file = File::open(&path).unwrap();
        std::fs::rename(&path, directory.path().join("retained")).unwrap();
        std::fs::write(&path, b"unrelated replacement").unwrap();
        let sha = hex(Sha256::digest(b"a\xfforiginal"));
        let selected = read_descriptor(file, 1, NonZeroU64::new(1), Some(&sha)).unwrap();
        assert!(matches!(selected.content,Content::Base64(ref data) if data=="/w=="));
        assert_eq!(selected.byte_range, 1..2);
        assert_eq!(selected.verified_sha256.as_deref(), Some(sha.as_str()));
        assert!(!selected.whole_file);
        assert!(!selected.atomic_snapshot);
        assert!(
            read_descriptor(
                File::open(&path).unwrap(),
                0,
                NonZeroU64::new(1),
                Some(&sha)
            )
            .is_err()
        );
        assert!(
            read_descriptor(
                File::open(&path).unwrap(),
                u64::MAX,
                NonZeroU64::new(2),
                None
            )
            .is_err()
        );
        let final_byte = read_descriptor(File::open(&path).unwrap(), 20, None, None).unwrap();
        assert!(final_byte.next_offset.is_none());
    }
    #[test]
    fn borrowed_descriptor_owner_survives_multiple_bounded_reads() {
        let mut file = tempfile::tempfile().unwrap();
        std::io::Write::write_all(&mut file, b"original").unwrap();
        let hash = hex(Sha256::digest(b"original"));
        for offset in [0, 4] {
            let read = read_descriptor_borrowed(&mut file, offset, NonZeroU64::new(4), Some(&hash))
                .unwrap();
            assert_eq!(read.byte_range, offset..offset + 4);
            assert_eq!(read.verified_sha256.as_deref(), Some(hash.as_str()));
            assert!(!read.atomic_snapshot);
        }
        assert_eq!(file.metadata().unwrap().len(), 8);
    }
}
