// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native descriptor custody, bound to original commitments and never serialized.
use super::{ContentIdentity, EvidenceFile, address, invalid, validate};
use crate::{HostError, artifacts::DescriptorIdentity};
use sha2::{Digest as _, Sha256};
use std::{
    fs::File,
    io::{Read as _, Seek as _, Write},
    os::unix::fs::MetadataExt as _,
    path::Path,
};

/// One original ordinal and checked descriptor. No path is reopened after acquisition.
#[derive(Debug)]
pub(crate) struct OpenedFile {
    dependency: usize,
    file: File,
    original: DescriptorIdentity,
    expected: ContentIdentity,
}

impl OpenedFile {
    pub(crate) fn dependency(&self) -> usize {
        self.dependency
    }

    pub(crate) fn read_range(
        &mut self,
        offset: u64,
        length: std::num::NonZeroU64,
    ) -> Result<crate::artifacts::DescriptorRead, HostError> {
        self.identity()?;
        let read = crate::artifacts::read_descriptor_borrowed(
            &mut self.file,
            offset,
            Some(length),
            Some(&self.expected.sha256),
        )?;
        self.identity()?;
        Ok(read)
    }

    fn identity(&self) -> Result<DescriptorIdentity, HostError> {
        let metadata = self.file.metadata()?;
        if !metadata.is_file()
            || metadata.uid() != rustix::process::geteuid().as_raw()
            || metadata.mode() & 0o022 != 0
        {
            return Err(invalid(
                "opened dependency no longer has private original file custody",
            ));
        }
        let identity = crate::artifacts::descriptor_identity(&self.file)?;
        if identity.device != self.original.device
            || identity.inode != self.original.inode
            || identity.bytes != self.expected.byte_length
        {
            return Err(invalid(format!(
                "opened dependency {} changed identity or extent",
                self.dependency
            )));
        }
        Ok(identity)
    }

    /// Verify precisely the bytes transferred, with finite growth detection and fresh
    /// before/after descriptor checks. An open fd is not filesystem snapshot isolation.
    pub(crate) fn copy_verified(
        &mut self,
        output: &mut impl Write,
    ) -> Result<ContentIdentity, HostError> {
        let before = self.identity()?;
        let maximum =
            self.expected.byte_length.checked_add(1).ok_or_else(|| {
                invalid("original dependency extent cannot include growth detection")
            })?;
        self.file.rewind()?;
        let mut input = (&mut self.file).take(maximum);
        let mut buffer = [0u8; 64 * 1024];
        let mut hash = Sha256::new();
        let mut byte_length = 0u64;
        loop {
            let count = input.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            byte_length += count as u64;
            if byte_length > self.expected.byte_length {
                return Err(invalid(
                    "opened dependency grew beyond its original commitment",
                ));
            }
            hash.update(&buffer[..count]);
            output.write_all(&buffer[..count])?;
        }
        let actual = ContentIdentity {
            byte_length,
            sha256: hash
                .finalize()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
        };
        if self.identity()? != before || actual != self.expected {
            return Err(invalid(format!(
                "opened dependency {} changed during reading or differs from its original commitment",
                self.dependency
            )));
        }
        Ok(actual)
    }
}

/// Acquire every originally committed member; original omissions are never opened.
/// Earlier descriptors are dropped if any later acquisition fails.
pub(crate) fn acquire(
    workflow: &Path,
    files: &[EvidenceFile],
) -> Result<Vec<OpenedFile>, HostError> {
    validate(files)?;
    let mut opened = Vec::new();
    for (dependency, commitment) in files.iter().enumerate() {
        if let Some(member) = acquire_member(workflow, dependency, commitment)? {
            opened.push(member);
        }
    }
    Ok(opened)
}

/// The compact profile materializes one member then releases its fd. It shares
/// admission with the retained-fd profile without requiring all fds simultaneously.
pub(super) fn acquire_member(
    workflow: &Path,
    dependency: usize,
    commitment: &EvidenceFile,
) -> Result<Option<OpenedFile>, HostError> {
    let Some(expected) = &commitment.content else {
        return Ok(None);
    };
    let file = crate::test_inventory::open_private(&address(
        workflow,
        commitment.root,
        &commitment.path,
    )?)?;
    let original = crate::artifacts::descriptor_identity(&file)?;
    let mut member = OpenedFile {
        dependency,
        file,
        original,
        expected: expected.clone(),
    };
    member.copy_verified(&mut std::io::sink())?;
    Ok(Some(member))
}

#[cfg(test)]
mod tests {
    use super::super::{EvidenceRole, EvidenceRoot, SourceFile, capture, seal, validate_members};
    use super::*;
    use std::{fs, os::unix::fs::PermissionsExt as _};

    fn fixture(names: &[&str]) -> (tempfile::TempDir, Vec<EvidenceFile>) {
        let root = tempfile::tempdir().unwrap();
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
        for name in names {
            let path = root.path().join(name);
            fs::write(&path, b"original").unwrap();
            fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
        }
        let (files, failures) = seal(
            root.path(),
            names
                .iter()
                .map(|name| SourceFile {
                    root: EvidenceRoot::Workflow,
                    path: (*name).into(),
                    role: EvidenceRole::Stdout,
                    group: Some(0),
                    settled: true,
                })
                .collect(),
        );
        assert!(failures.is_empty());
        (root, files)
    }

    #[test]
    fn opened_members_keep_original_inode_after_path_replacement_and_unlink() {
        let (root, files) = fixture(&["one.stdout"]);
        let mut opened = acquire(root.path(), &files).unwrap();
        let retained = root.path().join("retained");
        fs::rename(root.path().join("one.stdout"), &retained).unwrap();
        fs::write(root.path().join("one.stdout"), b"replaced").unwrap();
        fs::remove_file(retained).unwrap();
        let mut original = Vec::new();
        let observed = opened[0].copy_verified(&mut original).unwrap();
        assert_eq!(original, b"original");
        validate_members(&files, [(opened[0].dependency(), observed)]).unwrap();
        assert!(acquire(root.path(), &files).is_err());
    }

    #[test]
    fn opened_members_refuse_same_inode_mutation_and_growth_during_copy() {
        let (root, files) = fixture(&["one.stdout"]);
        let mut opened = acquire(root.path(), &files).unwrap();
        fs::write(root.path().join("one.stdout"), b"changed!").unwrap();
        assert!(opened[0].copy_verified(&mut std::io::sink()).is_err());

        fs::write(root.path().join("one.stdout"), b"original").unwrap();
        let mut opened = acquire(root.path(), &files).unwrap();
        struct Growing(File);
        impl Write for Growing {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                self.0.set_len(9)?;
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let writer = File::options()
            .write(true)
            .open(root.path().join("one.stdout"))
            .unwrap();
        assert!(opened[0].copy_verified(&mut Growing(writer)).is_err());
    }

    #[test]
    fn opened_later_missing_member_releases_earlier_descriptors_and_keeps_omissions() {
        let (root, mut files) = fixture(&["one.stdout", "two.stdout"]);
        fs::remove_file(root.path().join("two.stdout")).unwrap();
        assert!(acquire(root.path(), &files).is_err());
        let leaked = fs::read_dir("/proc/self/fd")
            .unwrap()
            .filter_map(Result::ok)
            .filter_map(|entry| fs::read_link(entry.path()).ok())
            .any(|path| path.starts_with(root.path()));
        assert!(!leaked);
        files[1].content = None;
        files[1].omission = Some(super::super::EvidenceOmission::Unsettled);
        fs::write(root.path().join("two.stdout"), b"later bytes").unwrap();
        let opened = acquire(root.path(), &files).unwrap();
        assert_eq!(opened.len(), 1);
        assert_eq!(opened[0].dependency(), 0);
    }

    #[test]
    fn opened_acquisition_and_streamed_publication_exceed_compact_limit() {
        let (root, _) = fixture(&["large.stdout"]);
        File::options()
            .write(true)
            .open(root.path().join("large.stdout"))
            .unwrap()
            .set_len(65 * 1024 * 1024)
            .unwrap();
        let (files, failures) = seal(
            root.path(),
            vec![SourceFile {
                root: EvidenceRoot::Workflow,
                path: "large.stdout".into(),
                role: EvidenceRole::Stdout,
                group: Some(0),
                settled: true,
            }],
        );
        assert!(failures.is_empty());
        assert!(capture(root.path(), &files, 64 * 1024 * 1024).is_err());
        let mut opened = acquire(root.path(), &files).unwrap();
        let destination = root.path().join("published");
        crate::invocation::publish_directory(&destination, |stage| {
            let identity = crate::invocation::publish_file(&stage.join("0"), 0o400, |file| {
                opened[0].copy_verified(file)?;
                Ok(())
            });
            identity?;
            crate::invocation::publish_bytes(
                &stage.join("manifest"),
                b"complete only after all original bytes",
                None,
                0o400,
            )
        })
        .unwrap();
        let mut published = File::open(destination.join("0")).unwrap();
        let identity = super::super::hash_file(&mut published).unwrap();
        assert_eq!(identity.byte_length, 65 * 1024 * 1024);
        validate_members(&files, [(0, identity)]).unwrap();
    }

    #[test]
    fn opened_acquisition_refuses_non_regular_member_without_blocking() {
        let (root, files) = fixture(&["one.stdout"]);
        fs::remove_file(root.path().join("one.stdout")).unwrap();
        rustix::fs::mkfifoat(
            rustix::fs::CWD,
            root.path().join("one.stdout"),
            rustix::fs::Mode::RUSR | rustix::fs::Mode::WUSR,
        )
        .unwrap();
        assert!(acquire(root.path(), &files).is_err());
    }
}
