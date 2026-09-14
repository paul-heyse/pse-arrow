// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Cooperative local conditional publication, using exact bytes under an OS lock.

use std::fs::{File, OpenOptions, TryLockError};
use std::io::{Read, Write};
use std::path::{Path as FsPath, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use bytes::Bytes;
use object_store::local::LocalFileSystem;
use object_store::path::Path;
use object_store::{PutMode, PutOptions, UpdateVersion};
use pse_ids::{CancellationToken, MemoryReserver};

use super::open::Catalog;
use super::verify::admission;
use crate::{CatalogError, Clock, TrustLevel};

fn infrastructure(
    error_context: &str,
    source: impl std::error::Error + Send + Sync + 'static,
) -> CatalogError {
    CatalogError::Infrastructure {
        op: error_context.to_owned(),
        source: Box::new(source),
    }
}

/// Actual observed control content, alongside native remote conditional tokens.
#[derive(Clone, Debug)]
pub(super) struct ObservedControl {
    pub bytes: Bytes,
    pub version: UpdateVersion,
}

#[derive(Debug)]
pub(crate) struct LocalFiles {
    root: PathBuf,
    store: Arc<LocalFileSystem>,
}

impl Catalog {
    /// Open a local catalog with cooperative conditional ref and stage-index updates.
    ///
    /// All writers must use this protocol and preserve its stable lock file. Unix local
    /// filesystems must honor advisory locks, atomic sibling rename and synchronization.
    /// Finished immutable files are synchronized before publication. A failure syncing
    /// after rename reports changed visibility with unconfirmed durability; reread the
    /// ref before retrying. Other platforms and failed capabilities are refused.
    ///
    /// # Errors
    /// Unsupported platform, unusable root directory, or local backend construction.
    pub fn open_local(
        root: &FsPath,
        registry: Arc<pse_schema::Registry>,
        trust: TrustLevel,
        clock: Arc<dyn Clock>,
        reserver: Arc<dyn MemoryReserver>,
    ) -> Result<Self, CatalogError> {
        if !cfg!(unix) {
            return Err(admission(
                "local catalog",
                "conditional local publication requires Unix",
            ));
        }
        std::fs::create_dir_all(root)
            .map_err(|error| infrastructure("create local root", error))?;
        let root = root
            .canonicalize()
            .map_err(|error| infrastructure("resolve local root", error))?;
        // A newly created root must itself be reachable durably before any reference
        // can be published below it. This also checks directory-sync support up front.
        for path in root.ancestors() {
            File::open(path)
                .and_then(|directory| directory.sync_all())
                .map_err(|error| infrastructure("synchronize local root ancestry", error))?;
        }
        let store = Arc::new(
            LocalFileSystem::new_with_prefix(&root)
                .map_err(|error| infrastructure("open local object store", error))?,
        );
        let files = Arc::new(LocalFiles {
            root,
            store: Arc::clone(&store),
        });
        let mut catalog = Self::open(store, registry, trust, clock, reserver);
        catalog.local = Some(files);
        Ok(catalog)
    }

    pub(super) async fn put_control(
        &self,
        path: &Path,
        expected: Option<&ObservedControl>,
        bytes: Bytes,
        cancel: &CancellationToken,
    ) -> Result<(), CatalogError> {
        cancel.checkpoint()?;
        if let Some(local) = &self.local {
            let local = Arc::clone(local);
            let path = path.clone();
            let expected = expected.map(|value| value.bytes.clone());
            let reserver = Arc::clone(&self.reserver);
            let cancel = cancel.clone();
            let limit = self.limits.max_control_bytes;
            return tokio::task::spawn_blocking(move || {
                local.replace(
                    &path,
                    expected.as_deref(),
                    &bytes,
                    limit,
                    reserver.as_ref(),
                    &cancel,
                )
            })
            .await
            .map_err(|error| infrastructure("join local conditional write", error))?;
        }
        let mode = expected.map_or(PutMode::Create, |value| {
            PutMode::Update(value.version.clone())
        });
        self.store
            .put_opts(
                path,
                bytes.into(),
                PutOptions {
                    mode,
                    ..Default::default()
                },
            )
            .await
            .map_err(|error| match error {
                object_store::Error::AlreadyExists { .. }
                | object_store::Error::Precondition { .. } => CatalogError::RefConflict {
                    name: path.to_string(),
                },
                source => infrastructure("conditionally write control object", source),
            })?;
        Ok(())
    }

    pub(super) async fn synchronize_immutable(
        &self,
        path: &Path,
        cancel: &CancellationToken,
    ) -> Result<(), CatalogError> {
        if let Some(local) = &self.local {
            let local = Arc::clone(local);
            let path = path.clone();
            let cancel = cancel.clone();
            tokio::task::spawn_blocking(move || {
                cancel.checkpoint()?;
                let path = local.path(&path)?;
                File::open(&path)
                    .and_then(|file| file.sync_all())
                    .map_err(|error| infrastructure("synchronize immutable local object", error))?;
                local.sync_parents(&path)
            })
            .await
            .map_err(|error| infrastructure("join local object synchronization", error))??;
        }
        Ok(())
    }
}

impl LocalFiles {
    fn path(&self, path: &Path) -> Result<PathBuf, CatalogError> {
        self.store
            .path_to_filesystem(path)
            .map_err(|error| infrastructure("resolve local object path", error))
    }

    fn lock(&self, cancel: &CancellationToken) -> Result<File, CatalogError> {
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(self.root.join(".pse-control.lock"))
            .map_err(|error| infrastructure("open stable local publication lock", error))?;
        loop {
            cancel.checkpoint()?;
            match lock.try_lock() {
                Ok(()) => return Ok(lock),
                Err(TryLockError::WouldBlock) => {
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                Err(TryLockError::Error(error)) => {
                    return Err(infrastructure("lock local publication", error));
                }
            }
        }
    }

    fn replace(
        &self,
        path: &Path,
        expected: Option<&[u8]>,
        bytes: &[u8],
        limit: usize,
        reserver: &dyn MemoryReserver,
        cancel: &CancellationToken,
    ) -> Result<(), CatalogError> {
        let _lock = self.lock(cancel)?;
        let destination = self.path(path)?;
        if !matches_current(&destination, expected, limit, reserver, cancel)? {
            return Err(CatalogError::RefConflict {
                name: path.to_string(),
            });
        }
        let parent = destination
            .parent()
            .ok_or_else(|| admission("local control", "missing parent"))?;
        std::fs::create_dir_all(parent)
            .map_err(|error| infrastructure("create local control directory", error))?;
        let pending = Pending::write(parent, bytes, cancel)?;
        cancel.checkpoint()?;
        std::fs::rename(&pending.path, &destination)
            .map_err(|error| infrastructure("atomically replace local control", error))?;
        // Publication has occurred. Never turn a later cancellation into a false
        // rollback report. A sync failure explicitly communicates the visible outcome.
        self.sync_parents(&destination)
            .map_err(|error| CatalogError::Infrastructure {
                op:
                    "local control replacement visible; durability unconfirmed; reread before retry"
                        .to_owned(),
                source: Box::new(error),
            })
    }

    fn sync_parents(&self, file: &FsPath) -> Result<(), CatalogError> {
        let mut parent = file.parent();
        while let Some(path) = parent {
            if !path.starts_with(&self.root) {
                break;
            }
            File::open(path)
                .and_then(|directory| directory.sync_all())
                .map_err(|error| infrastructure("synchronize local containing directory", error))?;
            parent = path.parent();
        }
        Ok(())
    }
}

fn matches_current(
    path: &FsPath,
    expected: Option<&[u8]>,
    limit: usize,
    reserver: &dyn MemoryReserver,
    cancel: &CancellationToken,
) -> Result<bool, CatalogError> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(expected.is_none()),
        Err(error) => return Err(infrastructure("reread locked local control", error)),
    };
    let Some(expected) = expected else {
        return Ok(false);
    };
    let size = file
        .metadata()
        .map_err(|error| infrastructure("inspect locked local control", error))?
        .len();
    if usize::try_from(size).ok().is_none_or(|size| size > limit) {
        return Err(admission(
            "locked local control",
            "actual file exceeds finite control bound",
        ));
    }
    if usize::try_from(size).ok() != Some(expected.len()) {
        return Ok(false);
    }
    // Compare exact bytes in a fixed, reserved chunk; the observed owner remains live.
    let mut reservation = reserver.open("store:local-control-reread");
    let capacity = expected.len().min(64 << 10);
    reservation.try_grow(capacity)?;
    let mut chunk = vec![0; capacity];
    for expected in expected.chunks(capacity.max(1)) {
        cancel.checkpoint()?;
        file.read_exact(&mut chunk[..expected.len()])
            .map_err(|error| infrastructure("read locked local control bytes", error))?;
        if expected != &chunk[..expected.len()] {
            return Ok(false);
        }
    }
    let mut extra = [0];
    file.read(&mut extra)
        .map(|size| size == 0)
        .map_err(|error| infrastructure("finish locked local control reread", error))
}

struct Pending {
    path: PathBuf,
}
impl Pending {
    fn write(
        parent: &FsPath,
        bytes: &[u8],
        cancel: &CancellationToken,
    ) -> Result<Self, CatalogError> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let (pending, mut file) = loop {
            cancel.checkpoint()?;
            let next = NEXT.fetch_add(1, Ordering::Relaxed);
            let path = parent.join(super::layout::local_temporary_name(
                std::process::id(),
                next,
            ));
            match File::create_new(&path) {
                Ok(file) => break (Self { path }, file),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => {
                    return Err(infrastructure("create exclusive local staging file", error));
                }
            }
        };
        file.write_all(bytes)
            .and_then(|()| file.sync_all())
            .map_err(|error| infrastructure("finish and synchronize local staging file", error))?;
        Ok(pending)
    }
}
impl Drop for Pending {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn interrupted_finished_staging_never_replaces_the_visible_value() {
        let root = std::env::temp_dir().join(format!("pse-local-staging-{}", std::process::id()));
        std::fs::create_dir_all(&root).expect("fixture directory");
        let destination = root.join("main.json");
        std::fs::write(&destination, b"old exact value").expect("old value");
        let cancel = CancellationToken::default();
        let pending = Pending::write(&root, b"new exact value", &cancel).expect("finished temp");
        assert_eq!(
            std::fs::read(&destination).expect("read"),
            b"old exact value"
        );
        let staging_path = pending.path.clone();
        drop(pending); // Failure/cancellation before rename cleans only its own temporary.
        assert!(!staging_path.exists());
        assert_eq!(
            std::fs::read(&destination).expect("read"),
            b"old exact value"
        );
        let budget = pse_ids::FixedBudget::new(32);
        assert!(
            !matches_current(
                &destination,
                Some(b"new exact value"),
                32,
                budget.as_ref(),
                &cancel
            )
            .expect("exact compare")
        );
        assert!(
            matches_current(
                &destination,
                Some(b"old exact value"),
                32,
                budget.as_ref(),
                &cancel
            )
            .expect("exact compare")
        );
        assert_eq!(budget.reserved(), 0);
        std::fs::remove_dir_all(root).expect("fixture cleanup");
    }
}
