// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded process-local listing slices. Durable progress belongs to the catalog.
//!
//! Object stores need not order listings or provide restart cursors. A retained stream
//! completes one enumeration generation; a replacement stream starts at the root.
use datafusion::{
    common::{DataFusionError, Result},
    execution::session_state::SessionState,
};
use futures_util::{StreamExt, stream::BoxStream};
use pse_columnar::{CancellationToken, MemoryConsumer, MemoryPool, MemoryReservation};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

const MAX_LOCAL_DEPTH: usize = 64;

fn invalid(message: impl Into<String>) -> DataFusionError {
    DataFusionError::Execution(message.into())
}
fn external(error: impl std::error::Error + Send + Sync + 'static) -> DataFusionError {
    DataFusionError::External(Box::new(error))
}

/// One observed object. A syntactic member prefix is not ownership evidence.
#[derive(Clone, Debug)]
pub struct ObservedObject {
    /// Object URL under the registered workspace root.
    pub location: url::Url,
    /// Candidate unit, including the publication prefix when its layout is recognized.
    pub candidate_prefix: url::Url,
    /// Physical size reported by the store.
    pub size: u64,
}

/// Owned, accounted slice; `complete` means this stream reached its end.
#[derive(Debug)]
pub struct ListingPage {
    /// Observations to persist before advancing the next slice.
    pub objects: Vec<ObservedObject>,
    /// Local confinement entries or native objects examined in this slice.
    pub examined: usize,
    /// Successful end of this uninterrupted enumeration, not a storage snapshot.
    pub complete: bool,
    _reservation: MemoryReservation,
}

/// One stream retained by one runtime. No provider cursor is fabricated.
pub struct RootListing {
    root: url::Url,
    path: object_store::path::Path,
    store: Arc<dyn object_store::ObjectStore>,
    guard: Option<walkdir::IntoIter>,
    stream: Option<BoxStream<'static, object_store::Result<object_store::ObjectMeta>>>,
    pending: Option<object_store::ObjectMeta>,
    pending_charge: usize,
    pool: Arc<dyn MemoryPool>,
    _reservation: MemoryReservation,
    complete: bool,
}
impl std::fmt::Debug for RootListing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RootListing")
            .field("root", &self.root)
            .field("confining", &self.guard.is_some())
            .field("complete", &self.complete)
            .finish()
    }
}

impl RootListing {
    /// Admit an established root and keep its native unordered stream.
    /// Local traversal first rejects symlinks without following them, over bounded slices.
    /// # Errors
    /// Unregistered, ambiguous or missing roots; local symlinks; exhausted accounting.
    pub fn new(root: url::Url, state: &SessionState, pool: Arc<dyn MemoryPool>) -> Result<Self> {
        validate_root(&root)?;
        let path = object_store::path::Path::from_url_path(root.path()).map_err(external)?;
        let store = state.runtime_env().object_store_registry.get_store(&root)?;
        let reservation = MemoryConsumer::new("catalog:orphan-listing-state").register(&pool);
        // WalkDir's max_open buffers a directory when exhausted. Keeping a handle for
        // every admitted depth avoids that unbounded entry buffer; no sorting is used.
        reservation.try_grow((MAX_LOCAL_DEPTH + 2) * 8192 + root.as_str().len())?;
        let guard = if root.scheme() == "file" {
            let local = root
                .to_file_path()
                .map_err(|()| invalid("invalid local workspace root"))?;
            check_ancestors(&local)?;
            if !local.is_dir() {
                return Err(invalid("workspace root is not an established directory"));
            }
            Some(
                walkdir::WalkDir::new(local)
                    .follow_links(false)
                    .follow_root_links(false)
                    .max_depth(MAX_LOCAL_DEPTH + 1)
                    .max_open(MAX_LOCAL_DEPTH + 2)
                    .into_iter(),
            )
        } else {
            None
        };
        Ok(Self {
            root,
            path,
            store,
            guard,
            stream: None,
            pending: None,
            pending_charge: 0,
            pool,
            _reservation: reservation,
            complete: false,
        })
    }

    /// Examine at most `max_entries`, retaining at most `max_bytes` of observations.
    /// Provider-internal buffers, I/O and latency are outside this application bound.
    /// # Errors
    /// Cancellation, escaped/local linked objects, invalid bounds or store failure.
    pub async fn page(
        &mut self,
        max_entries: usize,
        max_bytes: usize,
        cancel: &CancellationToken,
    ) -> Result<ListingPage> {
        if max_entries == 0 || max_bytes < 256 {
            return Err(invalid(
                "listing bounds must be positive and byte budget at least 256",
            ));
        }
        let reservation = MemoryConsumer::new("catalog:orphan-listing-page").register(&self.pool);
        reservation.try_grow(max_bytes)?;
        let mut page = ListingPage {
            objects: Vec::new(),
            examined: 0,
            complete: self.complete,
            _reservation: reservation,
        };
        let mut bytes = 0;
        while page.examined < max_entries && !self.complete {
            cancel.checkpoint().map_err(external)?;
            if let Some(guard) = self.guard.as_mut() {
                match guard.next() {
                    Some(entry) => {
                        let entry = entry.map_err(external)?;
                        page.examined += 1;
                        if entry.path_is_symlink() {
                            return Err(invalid(format!(
                                "local symlink refuses discovery: {}",
                                entry.path().display()
                            )));
                        }
                        if entry.depth() > MAX_LOCAL_DEPTH {
                            return Err(invalid("local discovery depth exceeds admitted bound"));
                        }
                        continue;
                    }
                    None => self.guard = None,
                }
            }
            if self.stream.is_none() {
                self.stream = Some(self.store.list(Some(&self.path)));
            }
            let next = if let Some(meta) = self.pending.take() {
                self._reservation.shrink(self.pending_charge);
                self.pending_charge = 0;
                Some(Ok(meta))
            } else {
                let stream = self
                    .stream
                    .as_mut()
                    .ok_or_else(|| invalid("listing state absent"))?;
                cancel
                    .until_cancelled(stream.next())
                    .await
                    .map_err(external)?
            };
            let Some(meta) = next else {
                self.complete = true;
                break;
            };
            let meta = meta.map_err(external)?;
            let metadata_bytes = meta.location.as_ref().len()
                + meta.e_tag.as_ref().map_or(0, String::len)
                + meta.version.as_ref().map_or(0, String::len)
                + 256;
            if metadata_bytes > max_bytes {
                return Err(invalid(
                    "object metadata exceeds admitted listing byte budget",
                ));
            }
            let observation = observed(&self.root, &self.path, &meta)?;
            let cost = observation.location.as_str().len()
                + observation.candidate_prefix.as_str().len()
                + 256;
            if cost > max_bytes {
                return Err(invalid("one object exceeds admitted listing byte budget"));
            }
            if bytes + cost > max_bytes {
                self._reservation.try_grow(metadata_bytes)?;
                self.pending_charge = metadata_bytes;
                self.pending = Some(meta);
                break;
            }
            page.examined += 1;
            bytes += cost;
            page.objects.push(observation);
        }
        page.complete = self.complete;
        Ok(page)
    }
}

fn validate_root(root: &url::Url) -> Result<()> {
    if !root.path().ends_with('/')
        || root.query().is_some()
        || root.fragment().is_some()
        || !root.username().is_empty()
        || root.password().is_some()
    {
        return Err(invalid(
            "workspace root must be an unambiguous directory URL",
        ));
    }
    Ok(())
}

fn check_ancestors(path: &Path) -> Result<()> {
    for ancestor in path.ancestors() {
        let metadata = std::fs::symlink_metadata(ancestor).map_err(external)?;
        if metadata.file_type().is_symlink() {
            return Err(invalid(format!(
                "local symlink refuses maintenance: {}",
                ancestor.display()
            )));
        }
    }
    Ok(())
}

fn observed(
    root: &url::Url,
    path: &object_store::path::Path,
    meta: &object_store::ObjectMeta,
) -> Result<ObservedObject> {
    let relative = meta
        .location
        .prefix_match(path)
        .ok_or_else(|| invalid("listed object escaped workspace root"))?
        .map(|part| part.as_ref().to_owned())
        .collect::<Vec<_>>();
    if relative.is_empty() {
        return Err(invalid("listed object names workspace root"));
    }
    let mut location = root.clone();
    {
        let mut parts = location
            .path_segments_mut()
            .map_err(|()| invalid("workspace root cannot contain objects"))?;
        parts.pop_if_empty();
        parts.extend(relative.iter().map(String::as_str));
    }
    if location.scheme() == "file" {
        check_ancestors(
            &location
                .to_file_path()
                .map_err(|()| invalid("invalid local object"))?,
        )?;
    }
    let count = if relative.first().is_some_and(|part| part == "members") && relative.len() >= 3 {
        3
    } else {
        1
    };
    let mut candidate_prefix = root.clone();
    {
        let mut parts = candidate_prefix
            .path_segments_mut()
            .map_err(|()| invalid("invalid workspace root"))?;
        parts.pop_if_empty();
        parts.extend(relative.iter().take(count).map(String::as_str));
        parts.push("");
    }
    Ok(ObservedObject {
        location,
        candidate_prefix,
        size: meta.size,
    })
}

/// Check a selected prefix is strictly contained, without local symlink aliases.
/// Run under the catalog claim immediately before physical maintenance.
/// # Errors
/// Escaped prefixes, symlinks or invalid local paths.
pub fn check_selected_prefix(root: &url::Url, prefix: &url::Url) -> Result<()> {
    validate_root(root)?;
    let base = root.as_str();
    if !prefix.as_str().starts_with(base)
        || prefix.as_str() == base
        || prefix.query().is_some()
        || prefix.fragment().is_some()
    {
        return Err(invalid("selected reclaim prefix escaped workspace root"));
    }
    if root.scheme() == "file" {
        let local: PathBuf = prefix
            .to_file_path()
            .map_err(|()| invalid("invalid local selected prefix"))?;
        let mut existing = local.as_path();
        while !existing.exists() {
            existing = existing
                .parent()
                .ok_or_else(|| invalid("selected prefix has no established ancestor"))?;
        }
        check_ancestors(existing)?;
    }
    Ok(())
}

/// Remove an explicitly claimed candidate, after bounded local confinement slices.
/// The caller holds the catalog maintenance claim throughout this operation.
/// # Errors
/// Root escape, linked descendants, cancellation, listing or deletion failure.
pub async fn remove_selected_prefix(
    root: &url::Url,
    prefix: &url::Url,
    state: &SessionState,
    pool: Arc<dyn MemoryPool>,
    cancel: &CancellationToken,
) -> Result<u64> {
    check_selected_prefix(root, prefix)?;
    // An absent local prefix is already physically reclaimed. Refuse dangling
    // symlinks rather than mistake their missing targets for absence.
    if prefix.scheme() == "file" {
        let local = prefix
            .to_file_path()
            .map_err(|()| invalid("invalid local selected prefix"))?;
        match std::fs::symlink_metadata(local) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
            Err(error) => return Err(external(error)),
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(invalid("selected prefix is a local symlink"));
            }
            Ok(_) => {}
        }
        let mut listing = RootListing::new(prefix.clone(), state, pool)?;
        loop {
            if listing.page(256, 1 << 20, cancel).await?.complete {
                break;
            }
        }
    }
    cancel.checkpoint().map_err(external)?;
    super::collect::remove_prefix_cancellable(prefix, state, cancel).await
}

#[cfg(test)]
mod discovery_unit {
    use super::*;
    #[test]
    fn selected_prefix_cannot_escape_or_alias_root() {
        let root = url::Url::parse("memory:///workspace/").unwrap();
        assert!(check_selected_prefix(&root, &root.join("members/a/b/").unwrap()).is_ok());
        assert!(check_selected_prefix(&root, &root.join("../outside/").unwrap()).is_err());
        assert!(check_selected_prefix(&root, &root).is_err());
    }
    #[cfg(unix)]
    #[test]
    fn local_symlink_ancestor_refuses_maintenance() {
        let directory = tempfile::tempdir().unwrap();
        let root = url::Url::from_directory_path(directory.path()).unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(outside.path(), directory.path().join("linked")).unwrap();
        assert!(check_selected_prefix(&root, &root.join("linked/member/").unwrap()).is_err());
    }

    #[tokio::test]
    async fn retained_unordered_stream_slices_visit_all_objects_without_offset() {
        use object_store::ObjectStoreExt;
        let context = datafusion::prelude::SessionContext::new();
        let root = url::Url::parse("memory:///workspace/").unwrap();
        let store = Arc::new(object_store::memory::InMemory::new());
        for name in ["z", "b", "a", "m", "c"] {
            store
                .put(
                    &object_store::path::Path::from(format!("workspace/members/a/p/{name}")),
                    bytes::Bytes::from_static(b"value").into(),
                )
                .await
                .unwrap();
        }
        context.runtime_env().register_object_store(&root, store);
        let pool: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(2 << 20));
        let cancel = CancellationToken::new();
        let mut listing = RootListing::new(root.clone(), &context.state(), pool.clone()).unwrap();
        let mut visited = std::collections::BTreeSet::new();
        loop {
            let page = listing.page(2, 2048, &cancel).await.unwrap();
            assert!(page.objects.len() <= 2);
            for object in &page.objects {
                assert_eq!(object.candidate_prefix, root.join("members/a/p/").unwrap());
                assert!(visited.insert(object.location.clone()));
            }
            if page.complete {
                break;
            }
        }
        assert_eq!(visited.len(), 5);
        drop(listing);
        assert_eq!(pool.reserved(), 0);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn local_guard_refuses_link_before_recording_any_candidate() {
        let directory = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(outside.path().join("foreign"), "preserve").unwrap();
        std::os::unix::fs::symlink(outside.path(), directory.path().join("linked")).unwrap();
        let root = url::Url::from_directory_path(directory.path()).unwrap();
        let context = datafusion::prelude::SessionContext::new();
        let pool: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(2 << 20));
        let mut listing = RootListing::new(root, &context.state(), pool).unwrap();
        assert!(
            listing
                .page(256, 4096, &CancellationToken::new())
                .await
                .is_err()
        );
        assert_eq!(
            std::fs::read_to_string(outside.path().join("foreign")).unwrap(),
            "preserve"
        );
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn selected_reclaim_checks_descendant_links_before_any_deletion() {
        let directory = tempfile::tempdir().unwrap();
        let selected = directory.path().join("members/a/p");
        std::fs::create_dir_all(&selected).unwrap();
        std::fs::write(selected.join("owned"), "preserve on refusal").unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(outside.path().join("foreign"), "preserve").unwrap();
        std::os::unix::fs::symlink(outside.path(), selected.join("linked")).unwrap();
        let root = url::Url::from_directory_path(directory.path()).unwrap();
        let prefix = url::Url::from_directory_path(&selected).unwrap();
        let context = datafusion::prelude::SessionContext::new();
        let pool: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(4 << 20));
        assert!(
            remove_selected_prefix(
                &root,
                &prefix,
                &context.state(),
                pool,
                &CancellationToken::new()
            )
            .await
            .is_err()
        );
        assert!(selected.join("owned").exists());
        assert!(outside.path().join("foreign").exists());
    }

    #[tokio::test]
    async fn cancellation_refuses_a_slice_and_releases_its_accounting() {
        let context = datafusion::prelude::SessionContext::new();
        let root = url::Url::parse("memory:///workspace/").unwrap();
        context
            .runtime_env()
            .register_object_store(&root, Arc::new(object_store::memory::InMemory::new()));
        let pool: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(2 << 20));
        let mut listing = RootListing::new(root, &context.state(), pool.clone()).unwrap();
        let cancel = CancellationToken::new();
        cancel.cancel();
        assert!(listing.page(256, 4096, &cancel).await.is_err());
        drop(listing);
        assert_eq!(pool.reserved(), 0);
    }
}
