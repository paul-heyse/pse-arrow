// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Local lazy readers and destructive maintenance share one OS lock per table.
//! This coordinates file retention only; Delta transactions remain write authority.
use datafusion::common::{DataFusionError, Result};
use pse_columnar::CancellationToken;
use std::{
    fs::{File, OpenOptions, TryLockError},
    io::{Read, Seek, SeekFrom, Write},
    sync::Arc,
};

#[derive(Debug)]
pub(crate) struct ReadLease {
    // The last owner closes the file and releases its shared lock.
    _file: File,
    pub(crate) generation: Generation,
}

pub(crate) async fn read(
    location: &url::Url,
    cancel: &CancellationToken,
) -> Result<Option<Arc<ReadLease>>> {
    if location.scheme() != "file" {
        return Ok(None);
    }
    let mut file = lock_file(location, false, false)?;
    acquire(&file, false, cancel).await?;
    let generation = locked_generation(location, &mut file)?;
    Ok(Some(Arc::new(ReadLease {
        _file: file,
        generation,
    })))
}

/// Writers participate before creating a table or emitting data files. Directory
/// creation here grants no Delta state; the native transaction remains authority.
pub(crate) async fn write(
    location: &url::Url,
    cancel: &CancellationToken,
) -> Result<Option<Arc<ReadLease>>> {
    if location.scheme() != "file" {
        return Ok(None);
    }
    let path = location
        .to_file_path()
        .map_err(|()| DataFusionError::Plan("invalid local Delta location".into()))?;
    std::fs::create_dir_all(path).map_err(external)?;
    let mut file = lock_file(location, true, true)?;
    acquire(&file, false, cancel).await?;
    if file.metadata().map_err(external)?.len() == 0 {
        file.unlock().map_err(external)?;
        acquire(&file, true, cancel).await?;
        if file.metadata().map_err(external)?.len() == 0 {
            renew(&mut file)?;
        }
        file.unlock().map_err(external)?;
        acquire(&file, false, cancel).await?;
    }
    let generation = locked_generation(location, &mut file)?;
    Ok(Some(Arc::new(ReadLease {
        _file: file,
        generation,
    })))
}

#[derive(Debug)]
pub(crate) struct MaintenanceLease {
    _file: File,
    path: std::path::PathBuf,
}

impl MaintenanceLease {
    pub(super) fn covers(&self, location: &url::Url) -> Result<()> {
        let path = location
            .to_file_path()
            .map_err(|()| DataFusionError::Plan("maintenance location must be local".into()))?
            .canonicalize()
            .map_err(external)?;
        if path != self.path {
            return Err(DataFusionError::Plan(
                "maintenance lease belongs to another table".into(),
            ));
        }
        Ok(())
    }
}

/// Destructive operations require a local exclusive OS lease. Refuse active
/// readers/writers immediately rather than deadlocking a caller retaining a reader.
pub(crate) fn maintenance(
    location: &url::Url,
    cancel: &CancellationToken,
) -> Result<MaintenanceLease> {
    cancel.checkpoint().map_err(external)?;
    let mut file = lock_file(location, true, false)?;
    file.try_lock().map_err(|error| match error {
        TryLockError::WouldBlock => DataFusionError::Execution(
            "Delta maintenance is blocked by an active reader or writer".into(),
        ),
        TryLockError::Error(error) => external(error),
    })?;
    let path = location
        .to_file_path()
        .map_err(|()| DataFusionError::Plan("maintenance location must be local".into()))?
        .canonicalize()
        .map_err(external)?;
    // Durable invalidation precedes every possible destructive action. A failed
    // maintenance attempt may invalidate reuse, but cannot resurrect old entries.
    renew(&mut file)?;
    Ok(MaintenanceLease { _file: file, path })
}

/// Cooperating local maintenance identity; remote reuse needs its own qualified lease.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Generation {
    token: [u8; 16],
    #[cfg(unix)]
    root: (u64, u64),
    #[cfg(not(unix))]
    root: std::time::SystemTime,
}
#[cfg(test)]
pub(crate) fn test_generation(token: u8) -> Generation {
    Generation {
        token: [token; 16],
        #[cfg(unix)]
        root: (0, 0),
        #[cfg(not(unix))]
        root: std::time::SystemTime::UNIX_EPOCH,
    }
}
pub(crate) fn generation(location: &url::Url) -> Result<Option<Generation>> {
    if location.scheme() != "file" {
        return Ok(None);
    }
    let path = location
        .to_file_path()
        .map_err(|()| DataFusionError::Plan("invalid local table path".into()))?;
    let mut file = File::open(path.join(".pse-retention.lock")).map_err(external)?;
    locked_generation(location, &mut file).map(Some)
}
fn locked_generation(location: &url::Url, file: &mut File) -> Result<Generation> {
    let path = location
        .to_file_path()
        .map_err(|()| DataFusionError::Plan("invalid local table path".into()))?;
    let root = identity(&path.metadata().map_err(external)?)?;
    let held = file.metadata().map_err(external)?;
    if held.len() != 16
        || identity(&held)?
            != identity(
                &path
                    .join(".pse-retention.lock")
                    .metadata()
                    .map_err(external)?,
            )?
    {
        return Err(DataFusionError::Plan(
            "retention owner was replaced or has an invalid generation".into(),
        ));
    }
    let mut token = [0; 16];
    file.seek(SeekFrom::Start(0)).map_err(external)?;
    file.read_exact(&mut token).map_err(external)?;
    if root != identity(&path.metadata().map_err(external)?)? {
        return Err(DataFusionError::Plan(
            "table root changed while acquiring retention ownership".into(),
        ));
    }
    Ok(Generation { token, root })
}
#[cfg(unix)]
#[expect(
    clippy::unnecessary_wraps,
    reason = "shared fallible identity interface across supported platforms"
)]
fn identity(metadata: &std::fs::Metadata) -> Result<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;
    Ok((metadata.dev(), metadata.ino()))
}
#[cfg(not(unix))]
fn identity(metadata: &std::fs::Metadata) -> Result<std::time::SystemTime> {
    metadata.created().map_err(external)
}

fn renew(file: &mut File) -> Result<()> {
    file.seek(SeekFrom::Start(0)).map_err(external)?;
    file.write_all(uuid::Uuid::now_v7().as_bytes())
        .map_err(external)?;
    file.set_len(16).map_err(external)?;
    file.sync_all().map_err(external)
}

fn lock_file(location: &url::Url, writable: bool, create: bool) -> Result<File> {
    let path = location.to_file_path().map_err(|()| {
        DataFusionError::Plan(
            "destructive remote maintenance requires a qualified reader coordination provider"
                .into(),
        )
    })?;
    // Resolve aliases and refuse a missing table instead of creating one during read.
    let path = path.canonicalize().map_err(external)?;
    OpenOptions::new()
        .read(true)
        .write(writable)
        .create(create)
        .truncate(false)
        .open(path.join(".pse-retention.lock"))
        .map_err(external)
}

async fn acquire(file: &File, exclusive: bool, cancel: &CancellationToken) -> Result<()> {
    loop {
        cancel.checkpoint().map_err(external)?;
        match if exclusive {
            file.try_lock()
        } else {
            file.try_lock_shared()
        } {
            Ok(()) => return Ok(()),
            Err(TryLockError::WouldBlock) => {
                cancel
                    .until_cancelled(tokio::time::sleep(std::time::Duration::from_millis(10)))
                    .await
                    .map_err(external)?;
            }
            Err(TryLockError::Error(error)) => return Err(external(error)),
        }
    }
}
fn external(error: impl Into<DataFusionError>) -> DataFusionError {
    error.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn readers_never_initialize_or_repair_retention_state() {
        let directory = tempfile::tempdir().unwrap();
        let location = url::Url::from_directory_path(directory.path()).unwrap();
        let cancel = CancellationToken::new();
        let lock = directory.path().join(".pse-retention.lock");
        assert!(read(&location, &cancel).await.is_err());
        assert!(!lock.exists());
        let writer = write(&location, &cancel).await.unwrap();
        drop(writer);
        let before = std::fs::read(&lock).unwrap();
        let reader = read(&location, &cancel).await.unwrap().unwrap();
        assert_eq!(std::fs::read(&lock).unwrap(), before);
        drop(reader);
        std::fs::write(&lock, []).unwrap();
        assert!(read(&location, &cancel).await.is_err());
        assert!(std::fs::read(&lock).unwrap().is_empty());
    }

    #[tokio::test]
    async fn shared_owner_blocks_deletion_until_last_clone_and_cancel_is_observed() {
        let directory = tempfile::tempdir().unwrap();
        let location = url::Url::from_directory_path(directory.path()).unwrap();
        let cancel = CancellationToken::new();
        let reader = write(&location, &cancel).await.unwrap().unwrap();
        let retained = Arc::clone(&reader);
        assert!(maintenance(&location, &cancel).is_err());
        drop(reader);
        let contender = lock_file(&location, true, false).unwrap();
        assert!(matches!(
            contender.try_lock(),
            Err(TryLockError::WouldBlock)
        ));
        let cancelled = CancellationToken::new();
        cancelled.cancel();
        assert!(acquire(&contender, true, &cancelled).await.is_err());
        drop(retained);
        let maintenance = lock_file(&location, true, false).unwrap();
        acquire(&maintenance, true, &cancel).await.unwrap();
        assert!(matches!(
            contender.try_lock_shared(),
            Err(TryLockError::WouldBlock)
        ));
        drop(maintenance);
        assert!(contender.try_lock_shared().is_ok());
    }

    #[tokio::test]
    async fn writers_and_maintenance_share_the_same_local_exclusion() {
        let directory = tempfile::tempdir().unwrap();
        let location = url::Url::from_directory_path(directory.path().join("new-table")).unwrap();
        let cancel = CancellationToken::new();
        let writer = write(&location, &cancel).await.unwrap().unwrap();
        assert!(maintenance(&location, &cancel).is_err());
        drop(writer);
        let exclusive = maintenance(&location, &cancel).unwrap();
        let file = lock_file(&location, true, false).unwrap();
        assert!(matches!(
            file.try_lock_shared(),
            Err(TryLockError::WouldBlock)
        ));
        drop(exclusive);
        assert!(maintenance(&url::Url::parse("memory:///remote").unwrap(), &cancel).is_err());
    }
}

#[cfg(test)]
mod integrated_performance_unit {
    use super::*;
    #[tokio::test]
    async fn root_replacement_cannot_admit_the_old_locked_file_as_a_new_generation() {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("table");
        std::fs::create_dir(&root).unwrap();
        let location = url::Url::from_directory_path(&root).unwrap();
        let cancel = CancellationToken::new();
        let original = write(&location, &cancel).await.unwrap().unwrap();
        #[expect(
            clippy::used_underscore_binding,
            reason = "the test checks the held OS lock against a replaced pathname"
        )]
        let mut held = original._file.try_clone().unwrap();
        std::fs::rename(&root, parent.path().join("old-table")).unwrap();
        std::fs::create_dir(&root).unwrap();
        let replacement = write(&location, &cancel).await.unwrap().unwrap();
        assert_ne!(original.generation, replacement.generation);
        assert!(locked_generation(&location, &mut held).is_err());
    }
    #[tokio::test]
    async fn append_preserves_generation_and_maintenance_changes_it_before_effects() {
        let directory = tempfile::tempdir().unwrap();
        let location = url::Url::from_directory_path(directory.path()).unwrap();
        let cancel = CancellationToken::new();
        let first = write(&location, &cancel).await.unwrap().unwrap();
        let selected = first.generation.clone();
        let writer = write(&location, &cancel).await.unwrap().unwrap();
        assert_eq!(writer.generation, selected);
        assert!(maintenance(&location, &cancel).is_err());
        drop((first, writer));
        let exclusive = maintenance(&location, &cancel).unwrap();
        assert_ne!(generation(&location).unwrap().unwrap(), selected);
        drop(exclusive);
        let next = read(&location, &cancel).await.unwrap().unwrap();
        assert_ne!(next.generation, selected);
    }
}
