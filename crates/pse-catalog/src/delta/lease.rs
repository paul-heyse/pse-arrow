// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Local lazy readers and destructive maintenance share one OS lock per table.
//! This coordinates file retention only; Delta transactions remain write authority.
use datafusion::common::{DataFusionError, Result};
use pse_ids::CancellationToken;
use std::{
    fs::{File, OpenOptions, TryLockError},
    sync::Arc,
};

#[derive(Debug)]
pub(crate) struct ReadLease {
    // The last owner closes the file and releases its shared lock.
    _file: File,
}

pub(crate) async fn read(
    location: &url::Url,
    cancel: &CancellationToken,
) -> Result<Option<Arc<ReadLease>>> {
    if location.scheme() != "file" {
        return Ok(None);
    }
    let file = lock_file(location)?;
    acquire(&file, false, cancel).await?;
    Ok(Some(Arc::new(ReadLease { _file: file })))
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
    read(location, cancel).await
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
    let file = lock_file(location)?;
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
    Ok(MaintenanceLease { _file: file, path })
}

fn lock_file(location: &url::Url) -> Result<File> {
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
        .write(true)
        .create(true)
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
fn external(error: impl std::error::Error + Send + Sync + 'static) -> DataFusionError {
    DataFusionError::External(Box::new(error))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn shared_owner_blocks_deletion_until_last_clone_and_cancel_is_observed() {
        let directory = tempfile::tempdir().unwrap();
        let location = url::Url::from_directory_path(directory.path()).unwrap();
        let cancel = CancellationToken::new();
        let reader = read(&location, &cancel).await.unwrap().unwrap();
        let retained = Arc::clone(&reader);
        assert!(maintenance(&location, &cancel).is_err());
        drop(reader);
        let contender = lock_file(&location).unwrap();
        assert!(matches!(
            contender.try_lock(),
            Err(TryLockError::WouldBlock)
        ));
        let cancelled = CancellationToken::new();
        cancelled.cancel();
        assert!(acquire(&contender, true, &cancelled).await.is_err());
        drop(retained);
        let maintenance = lock_file(&location).unwrap();
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
        let file = lock_file(&location).unwrap();
        assert!(matches!(
            file.try_lock_shared(),
            Err(TryLockError::WouldBlock)
        ));
        drop(exclusive);
        assert!(maintenance(&url::Url::parse("memory:///remote").unwrap(), &cancel).is_err());
    }
}
