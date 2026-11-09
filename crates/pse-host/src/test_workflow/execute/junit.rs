// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bound the exact native JUnit stream before it reaches the retained report store.

use crate::{HostError, managed::payload::PayloadPolicy, test_workflow::WorkflowError};
use rustix::fs::{CWD, Mode, OFlags, RenameFlags, mkfifoat, open, renameat_with};
use std::{
    fs,
    io,
    os::unix::fs::{FileTypeExt as _, MetadataExt as _, PermissionsExt as _},
    path::{Path, PathBuf},
    process::ExitStatus,
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncReadExt as _, AsyncWriteExt as _},
    net::unix::pipe,
    process::Child,
};

pub(super) const MAX_BYTES: u64 = 64 * 1024 * 1024;

pub(crate) fn transport_path(report: &Path) -> PathBuf {
    report.with_extension("xml.fifo")
}

struct OriginalPipe {
    path: PathBuf,
    device: u64,
    inode: u64,
}

impl Drop for OriginalPipe {
    fn drop(&mut self) {
        if fs::symlink_metadata(&self.path).is_ok_and(|current| {
            current.file_type().is_fifo()
                && current.dev() == self.device
                && current.ino() == self.inode
        }) {
            let _cleanup = fs::remove_file(&self.path);
        }
    }
}

impl OriginalPipe {
    fn open(&self, access: OFlags) -> io::Result<fs::File> {
        let fd = open(
            &self.path, access | OFlags::NONBLOCK | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )?;
        let file = fs::File::from(fd);
        let found = file.metadata()?;
        if !found.file_type().is_fifo() || found.nlink() != 1
            || found.uid() != rustix::process::geteuid().as_raw()
            || found.dev() != self.device || found.ino() != self.inode
        {
            return Err(io::Error::other("original JUnit capture pipe changed"));
        }
        Ok(file)
    }
}

pub(super) struct Relay {
    // Retain ownership of the name until native wait and stream drain have ended.
    name: OriginalPipe,
    keep_writer: fs::File,
    reader: pipe::Receiver,
    staged: tempfile::NamedTempFile,
    output: tokio::fs::File,
    report: PathBuf,
}

impl Relay {
    pub(super) fn start(report: &Path) -> Result<Self, WorkflowError> {
        let parent = report.parent().ok_or_else(|| {
            WorkflowError::Invalid("original JUnit report has no parent directory".into())
        })?;
        let staged = tempfile::NamedTempFile::new_in(parent)?;
        let output = tokio::fs::File::from_std(staged.reopen()?);
        let path = transport_path(report);
        mkfifoat(CWD, &path, Mode::RUSR | Mode::WUSR).map_err(io::Error::from)?;
        let metadata = fs::symlink_metadata(&path)?;
        let name = OriginalPipe { path, device: metadata.dev(), inode: metadata.ino() };
        let reader = pipe::Receiver::from_file(name.open(OFlags::RDONLY)?)?;
        // An empty read must not be mistaken for EOF before Nextest opens the path.
        let keep_writer = name.open(OFlags::WRONLY)?;
        Ok(Self { name, keep_writer, reader, staged, output, report: report.to_owned() })
    }

    pub(super) async fn wait(
        self,
        child: &mut Child,
        policy: PayloadPolicy,
    ) -> (io::Result<ExitStatus>, Instant, Result<(), WorkflowError>) {
        self.wait_bounded(child, policy, MAX_BYTES).await
    }

    async fn wait_bounded(
        self,
        child: &mut Child,
        policy: PayloadPolicy,
        maximum: u64,
    ) -> (io::Result<ExitStatus>, Instant, Result<(), WorkflowError>) {
        let Self { name: _name, keep_writer, reader, staged, output, report } = self;
        let collect = collect(reader, output, maximum, policy.write);
        tokio::pin!(collect);
        let (waited, waited_at, capture) = tokio::select! {
            waited = child.wait() => {
                let waited_at = Instant::now();
                drop(keep_writer);
                let capture = tokio::time::timeout(policy.drain, &mut collect).await
                    .map_err(|_| WorkflowError::Invalid("original JUnit stream did not drain".into()))
                    .and_then(|outcome| outcome);
                (waited, waited_at, capture)
            }
            capture = &mut collect => {
                drop(keep_writer);
                // Before we drop the keeper, EOF is impossible. This branch is a
                // transport failure; the writer cannot be left blocked on the pipe.
                let _stop = child.start_kill();
                let waited = child.wait().await;
                (waited, Instant::now(), capture)
            }
        };
        let capture = capture.and_then(|bytes| {
            if bytes == 0 {
                // The native observer distinguishes a missing report from a completed one.
                return Ok(());
            }
            if waited.is_err() {
                return Err(WorkflowError::Invalid("JUnit writer exit was not observed".into()));
            }
            publish(staged, &report)
        });
        (waited, waited_at, capture)
    }
}

async fn collect(
    mut reader: pipe::Receiver,
    mut output: tokio::fs::File,
    limit: u64,
    write: Duration,
) -> Result<u64, WorkflowError> {
    let mut buffer = [0_u8; 16 * 1024];
    let mut extent = 0_u64;
    let mut omission = None;
    loop {
        let count = reader.read(&mut buffer).await?;
        if count == 0 {
            break;
        }
        extent = extent.saturating_add(count as u64);
        if omission.is_some() {
            continue;
        }
        if extent > limit {
            omission = Some(WorkflowError::Invalid(format!(
                "original JUnit report exceeded its {limit}-byte capture bound"
            )));
        } else {
            match tokio::time::timeout(write, output.write_all(&buffer[..count])).await {
                Ok(Ok(())) => {}
                Ok(Err(error)) => omission = Some(error.into()),
                Err(_) => omission = Some(WorkflowError::Invalid("original JUnit write timed out".into())),
            }
        }
    }
    if let Some(omission) = omission {
        return Err(omission);
    }
    tokio::time::timeout(write, output.flush())
    .await
    .map_err(|_| WorkflowError::Invalid("original JUnit synchronization timed out".into()))??;
    Ok(extent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn bounded_junit_relay_drains_overflow_and_retains_original_exit() {
        let directory = tempfile::tempdir().unwrap();
        let extent = 128 * 1024;
        let duration = Duration::from_secs(5);
        let policy = PayloadPolicy { startup: duration, drain: duration, write: duration };
        for (name, limit) in [("complete.xml", extent), ("omitted.xml", 1024)] {
            let report = directory.path().join(name);
            let relay = Relay::start(&report).unwrap();
            let pipe = transport_path(&report);
            let mut child = tokio::process::Command::new("/bin/sh")
                .args(["-c", "head -c \"$2\" /dev/zero > \"$1\" || exit 90; exit 23", "report-writer"])
                .arg(&pipe)
                .arg(extent.to_string())
                .kill_on_drop(true)
                .spawn().unwrap();
            let (status, _, capture) = tokio::time::timeout(
                duration, relay.wait_bounded(&mut child, policy, limit),
            ).await.unwrap();
            assert_eq!(status.unwrap().code(), Some(23));
            assert!(!pipe.exists());
            if limit == extent {
                capture.unwrap();
                assert_eq!(fs::metadata(&report).unwrap().len(), extent);
                assert_eq!(fs::metadata(&report).unwrap().mode() & 0o777, 0o400);
            } else {
                assert!(capture.unwrap_err().to_string().contains("capture bound"));
                assert!(!report.exists());
            }
        }
    }
}

fn publish(mut staged: tempfile::NamedTempFile, report: &Path) -> Result<(), WorkflowError> {
    staged.as_file().set_permissions(fs::Permissions::from_mode(0o400))?;
    staged.as_file().sync_all()?;
    renameat_with(CWD, staged.path(), CWD, report, RenameFlags::NOREPLACE)
        .map_err(io::Error::from)?;
    staged.disable_cleanup(true);
    if let Some(parent) = report.parent() {
        fs::File::open(parent).and_then(|file| file.sync_all()).map_err(|source| {
            HostError::PublicationDurabilityUnknown { destination: report.to_owned(), source }
        })?;
    }
    Ok(())
}
