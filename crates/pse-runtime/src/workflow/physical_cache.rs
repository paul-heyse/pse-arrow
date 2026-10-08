// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Two bounded owner-local physical products; native attempts never enter retention.
use super::{Operations, PhysicalContext, PhysicalSource, Runtime, WorkflowError, contract};
use datafusion::execution::memory_pool::{MemoryConsumer, MemoryReservation};
use pse_engine::cache_service::{CacheComponent, CacheEntryReport, CacheReport};
use pse_operations::canonical::{CanonicalStore, ProtectedSelection, Revision};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

const RETAINED_BYTES: usize = 16 << 20;
const NAME: &str = "pse.cache.physical_admission";

/// Active immutable values retain their own protections beyond cache eviction.
#[derive(Debug)]
pub(super) struct PhysicalOwner {
    store: CanonicalStore,
    pins: Vec<ProtectedSelection>,
    _allocation: Arc<pse_columnar::AllocationLease>,
    retained_metadata: Option<Arc<pse_columnar::AllocationLease>>,
    handle: tokio::runtime::Handle,
}
impl Drop for PhysicalOwner {
    fn drop(&mut self) {
        let store = self.store.clone();
        let pins = std::mem::take(&mut self.pins);
        self.handle.spawn(async move {
            for pin in pins {
                let _ = store.release(&pin).await;
            }
        });
        // Executor shutdown can cancel bookkeeping; server expiry remains bounded.
    }
}
#[derive(Debug)]
struct Admission {
    source: PhysicalSource,
    operations: Operations,
    registry: Arc<pse_schema::Registry>,
    sessions: Arc<pse_engine::session::EngineFactory>,
    physical: Arc<PhysicalContext>,
    bytes: usize,
    _metadata: Arc<pse_columnar::AllocationLease>,
}
#[derive(Debug)]
struct Rows {
    operations: Operations,
    registry: Arc<pse_schema::Registry>,
    sessions: Arc<pse_engine::session::EngineFactory>,
    physical: PhysicalContext,
    revisions: Vec<Revision>,
    _owner: Arc<PhysicalOwner>,
    bytes: usize,
    _metadata: Arc<pse_columnar::AllocationLease>,
}
#[derive(Debug)]
pub(super) struct PhysicalRows {
    pub(super) revisions: Vec<Revision>,
    _owner: Arc<PhysicalOwner>,
}
#[derive(Debug, Default)]
pub(super) struct PhysicalCache {
    admission: Mutex<Option<Admission>>,
    rows: Mutex<Option<Rows>>,
    admission_fence: pse_columnar::retention::RetentionFence,
    rows_fence: pse_columnar::retention::RetentionFence,
    hits: AtomicUsize,
    misses: AtomicUsize,
    #[cfg(test)]
    admission_pause: Mutex<
        Option<(
            tokio::sync::oneshot::Sender<()>,
            tokio::sync::oneshot::Receiver<()>,
        )>,
    >,
    #[cfg(test)]
    admission_start_pause: Mutex<
        Option<(
            tokio::sync::oneshot::Sender<()>,
            tokio::sync::oneshot::Receiver<()>,
        )>,
    >,
}
impl PhysicalCache {
    pub(super) fn clear(&self) {
        self.admission_fence.clear(|| {
            self.admission
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take();
        });
        self.rows_fence.clear(|| {
            self.rows
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take();
        });
    }
    fn admission(
        &self,
        runtime: &Runtime,
        operations: &Operations,
        source: &PhysicalSource,
    ) -> Option<Arc<PhysicalContext>> {
        let entry = self
            .admission
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let physical = entry
            .as_ref()
            .filter(|entry| {
                entry.source == *source
                    && entry.operations.same_physical_owner(operations)
                    && Arc::ptr_eq(&entry.registry, &runtime.registry)
                    && Arc::ptr_eq(&entry.sessions, &runtime.sessions)
            })
            .map(|entry| entry.physical.clone());
        if physical.is_some() {
            self.hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
        }
        physical
    }
    pub(super) fn rows(
        &self,
        runtime: &Runtime,
        operations: &Operations,
        physical: &PhysicalContext,
    ) -> Option<Vec<Revision>> {
        let entry = self
            .rows
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let rows = entry
            .as_ref()
            .filter(|entry| {
                entry.operations.same_physical_owner(operations)
                    && Arc::ptr_eq(&entry.registry, &runtime.registry)
                    && Arc::ptr_eq(&entry.sessions, &runtime.sessions)
                    && entry.physical.same_sources(physical)
            })
            .map(|entry| entry.revisions.clone());
        if rows.is_some() {
            self.hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
        }
        rows
    }
    pub(super) fn begin_rows(&self) -> u64 {
        let generation = self.rows_fence.generation();
        self.rows_fence.clear(|| {
            self.rows
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take();
        });
        generation.saturating_add(1)
    }
    pub(super) fn rows_generation(&self) -> u64 {
        self.rows_fence.generation()
    }
    pub(super) fn retain_rows(
        &self,
        generation: u64,
        runtime: &Runtime,
        operations: &Operations,
        physical: &PhysicalContext,
        rows: &PhysicalRows,
    ) {
        let revisions = &rows.revisions;
        let bytes = physical.retained_bytes().saturating_add(
            revisions
                .iter()
                .map(|r| r.key.len() + r.problem.len() + 256)
                .sum::<usize>(),
        );
        if bytes > RETAINED_BYTES {
            return;
        }
        let Some(metadata) = metadata(
            operations,
            physical.sources.len() * 512 + revisions.len() * 512 + 1024,
        ) else {
            return;
        };
        let entry = Rows {
            operations: operations.clone(),
            registry: runtime.registry.clone(),
            sessions: runtime.sessions.clone(),
            physical: physical.clone(),
            revisions: revisions.to_vec(),
            _owner: rows._owner.clone(),
            bytes,
            _metadata: metadata,
        };
        self.rows_fence.admit(generation, || {
            *self
                .rows
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(entry);
        });
    }
    pub(super) async fn protect_rows(
        &self,
        operations: &Operations,
        revisions: &[Revision],
    ) -> Result<PhysicalRows, WorkflowError> {
        Ok(PhysicalRows {
            revisions: revisions.to_vec(),
            _owner: protect(operations, revisions, 1024 + revisions.len() * 512).await?,
        })
    }
    pub(super) async fn protected_rows(
        &self,
        generation: u64,
        runtime: &Runtime,
        operations: &Operations,
        physical: &PhysicalContext,
        revisions: &[Revision],
    ) -> Result<PhysicalRows, WorkflowError> {
        // Replace the cached pin with fresh native admission even on an exact hit.
        let rows = self.protect_rows(operations, revisions).await?;
        self.retain_rows(generation, runtime, operations, physical, &rows);
        Ok(rows)
    }
}

async fn protect(
    operations: &Operations,
    revisions: &[Revision],
    bytes: usize,
) -> Result<Arc<PhysicalOwner>, WorkflowError> {
    let allocation = MemoryConsumer::new(NAME).register(&operations.pool);
    allocation
        .try_grow(bytes)
        .map_err(pse_engine::EngineError::from)?;
    let mut owner = PhysicalOwner {
        store: operations.store().clone(),
        pins: Vec::with_capacity(revisions.len()),
        _allocation: pse_columnar::AllocationLease::new(allocation),
        retained_metadata: None,
        handle: tokio::runtime::Handle::current(),
    };
    for revision in revisions {
        let pin = operations
            .store()
            .protect(revision.clone(), std::time::Duration::from_secs(3600))
            .await?;
        if pin.revision() != revision {
            return Err(contract("physical receipt differs from protected revision"));
        }
        owner.pins.push(pin);
    }
    Ok(Arc::new(owner))
}
fn metadata(operations: &Operations, bytes: usize) -> Option<Arc<pse_columnar::AllocationLease>> {
    let allocation =
        MemoryConsumer::new("canonical:physical-cache-metadata").register(&operations.pool);
    allocation.try_grow(bytes).ok()?;
    Some(pse_columnar::AllocationLease::new(allocation))
}
impl Runtime {
    pub(super) async fn physical_source(
        &self,
        source: &PhysicalSource,
        cancel: &crate::CancelSource,
    ) -> Result<PhysicalContext, WorkflowError> {
        checkpoint(cancel)?;
        let operations = self.operations()?;
        let generation = self.physical_cache.admission_fence.generation();
        let cached = self.physical_cache.admission(self, operations, source);
        // Evict and capture before the first asynchronous source/guard lookup. An
        // external clear during protection must not become this old load's epoch.
        let generation = if cached.is_none() {
            self.physical_cache.admission_fence.clear(|| {
                self.physical_cache
                    .admission
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .take();
            });
            generation.saturating_add(1)
        } else {
            generation
        };
        #[cfg(test)]
        {
            let pause = self
                .physical_cache
                .admission_start_pause
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take();
            if let Some((ready, resume)) = pause {
                let _ = ready.send(());
                let _ = resume.await;
            }
        }
        let revision = operations
            .store()
            .revision(&source.revision)
            .await?
            .ok_or_else(|| contract("physical source revision absent"))?;
        if revision.problem != format!("physical:documents:{}", source.identity) {
            return Err(contract("physical document receipt interpretation differs"));
        }
        let mut protection = protect(
            operations,
            &[revision],
            1024 + cached
                .as_ref()
                .map_or(0, |physical| physical.sources.len() * 512),
        )
        .await?;
        checkpoint(cancel)?;
        if let Some(physical) = cached {
            if let Some(owner) = Arc::get_mut(&mut protection) {
                owner.retained_metadata = physical
                    ._source_owner
                    .as_ref()
                    .and_then(|owner| owner.retained_metadata.clone());
            }
            let mut physical = physical.as_ref().clone();
            physical._source_owner = Some(protection);
            return Ok(physical);
        }
        let sources = operations.sources(source).await?;
        checkpoint(cancel)?;
        let mut physical = self.physical_from_sources(&sources, cancel).await?;
        checkpoint(cancel)?;
        let bytes = physical.retained_bytes();
        let metadata = metadata(
            operations,
            physical.sources.len() * 512 + source.revision.len() + 1024,
        );
        if let Some(owner) = Arc::get_mut(&mut protection) {
            owner.retained_metadata = metadata.clone();
        }
        physical._source_owner = Some(protection);
        #[cfg(test)]
        {
            let pause = self
                .physical_cache
                .admission_pause
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take();
            if let Some((ready, resume)) = pause {
                let _ = ready.send(());
                let _ = resume.await;
            }
        }
        checkpoint(cancel)?;
        if let Some(metadata) = metadata.filter(|_| bytes <= RETAINED_BYTES) {
            let entry = Admission {
                source: source.clone(),
                operations: operations.clone(),
                registry: self.registry.clone(),
                sessions: self.sessions.clone(),
                physical: Arc::new(physical.clone()),
                bytes,
                _metadata: metadata,
            };
            self.physical_cache.admission_fence.admit(generation, || {
                *self
                    .physical_cache
                    .admission
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(entry);
            });
        }
        Ok(physical)
    }
}

#[cfg(all(test, feature = "canonical-tests"))]
mod physical_admission_cache_unit {
    #![allow(
        clippy::unwrap_used,
        reason = "canonical cache fixtures require successful durable source admission before exercising owner reuse"
    )]
    use super::super::{Durability, durable_tests::durable_runtime, worker_tests::sources};
    use super::*;

    async fn fixture() -> (Runtime, PhysicalSource) {
        let runtime = durable_runtime();
        let (documents, _) = sources(
            "package p { def Root { param x:Scalar=3; annotation report x(\"constant\"); } }",
        );
        let source = runtime
            .operations()
            .unwrap()
            .put_sources(&documents)
            .await
            .unwrap();
        (runtime, source)
    }
    #[tokio::test]
    async fn canonical_physical_admission_and_rows_reuse_exact_original_owners() {
        let (runtime, source) = fixture().await;
        let cancel = crate::CancelSource::new();
        let first = runtime.physical_source(&source, &cancel).await.unwrap();
        let second = runtime.physical_source(&source, &cancel).await.unwrap();
        assert!(first.same_sources(&second));
        assert!(
            !Arc::ptr_eq(
                first._source_owner.as_ref().unwrap(),
                second._source_owner.as_ref().unwrap()
            ),
            "each hit has a fresh native protection"
        );
        let operations = runtime.operations().unwrap();
        let first_rows = operations
            .put_physical_rows(&runtime, &first)
            .await
            .unwrap();
        let hits = runtime.physical_cache.hits.load(Ordering::Relaxed);
        let second_rows = operations
            .put_physical_rows(&runtime, &second)
            .await
            .unwrap();
        assert_eq!(first_rows.revisions, second_rows.revisions);
        assert_eq!(
            runtime.physical_cache.hits.load(Ordering::Relaxed),
            hits + 1
        );
        let mut changed = second.clone();
        let key = *changed.sources.keys().next().unwrap();
        let table = changed.sources.get(&key).unwrap();
        changed.sources.insert(key, table.slice(0, 0).unwrap());
        assert_eq!(first.identity(), changed.identity());
        assert!(
            !first.same_sources(&changed),
            "equal physical meaning never hides changed support rows"
        );
        assert!(
            runtime
                .physical_cache
                .rows(&runtime, operations, &changed)
                .is_none()
        );
        let mut package_changed = second.clone();
        package_changed
            .package
            .as_mut()
            .unwrap()
            .name
            .push_str("changed");
        assert!(
            runtime
                .physical_cache
                .rows(&runtime, operations, &package_changed)
                .is_none()
        );
        runtime.clear_program_cache();
        assert_eq!(runtime.physical_cache.report()[0].entries, 0);
        assert_eq!(
            first_rows.revisions, second_rows.revisions,
            "active receipt consumers survive eviction"
        );
        assert!(first.same_sources(&second));
    }
    #[tokio::test]
    async fn canonical_physical_admission_clear_fences_actual_inflight_load() {
        let (runtime, source) = fixture().await;
        let (ready, waiting) = tokio::sync::oneshot::channel();
        let (resume, paused) = tokio::sync::oneshot::channel();
        *runtime.physical_cache.admission_pause.lock().unwrap() = Some((ready, paused));
        let worker = runtime.clone();
        let requested = source.clone();
        let pending = tokio::spawn(async move {
            worker
                .physical_source(&requested, &crate::CancelSource::new())
                .await
                .unwrap()
        });
        waiting.await.unwrap();
        runtime.clear_program_cache();
        resume.send(()).unwrap();
        let active = pending.await.unwrap();
        assert_eq!(
            runtime.physical_cache.report()[0].entries,
            0,
            "pre-clear load cannot republish"
        );
        let fresh = runtime
            .physical_source(&source, &crate::CancelSource::new())
            .await
            .unwrap();
        assert!(!Arc::ptr_eq(&active.quantities, &fresh.quantities));
        assert_eq!(active.identity(), fresh.identity());
    }
    #[tokio::test]
    async fn canonical_physical_admission_clear_before_first_source_await_fences_load() {
        let (runtime, source) = fixture().await;
        let (ready, waiting) = tokio::sync::oneshot::channel();
        let (resume, paused) = tokio::sync::oneshot::channel();
        *runtime.physical_cache.admission_start_pause.lock().unwrap() = Some((ready, paused));
        let worker = runtime.clone();
        let pending = tokio::spawn(async move {
            worker
                .physical_source(&source, &crate::CancelSource::new())
                .await
                .unwrap()
        });
        waiting.await.unwrap();
        runtime.clear_program_cache();
        resume.send(()).unwrap();
        let active = pending.await.unwrap();
        assert!(!active.sources.is_empty());
        assert_eq!(
            runtime.physical_cache.report()[0].entries,
            0,
            "clear before native guard awaits must fence the original load"
        );
    }
    #[tokio::test]
    async fn canonical_physical_admission_owner_revision_and_retirement_fences() {
        let (runtime, source) = fixture().await;
        let cancel = crate::CancelSource::new();
        let physical = runtime.physical_source(&source, &cancel).await.unwrap();
        let operations = runtime.operations().unwrap();
        let mut other = runtime.clone();
        other.sessions = Arc::new(
            runtime
                .shared
                .session_factory(pse_engine::session::native_engine_profile())
                .unwrap(),
        );
        assert!(
            runtime
                .physical_cache
                .admission(&other, operations, &source)
                .is_none()
        );
        other.registry = Arc::new(pse_schema::catalog::assemble().unwrap());
        other.sessions = runtime.sessions.clone();
        assert!(
            runtime
                .physical_cache
                .admission(&other, operations, &source)
                .is_none()
        );
        let new_operations = Operations::from_store(
            operations.store().clone(),
            "other-owner",
            operations.policy(),
            runtime.shared.pool(),
        );
        assert!(
            runtime
                .physical_cache
                .admission(&runtime, &new_operations, &source)
                .is_none()
        );
        other.durability = Durability::Durable(new_operations);
        let other_store = pse_operations::testing::canonical_fixture_store().unwrap();
        let foreign = Operations::from_store(
            other_store.clone(),
            operations.worker(),
            operations.policy(),
            runtime.shared.pool(),
        );
        assert!(
            runtime
                .physical_cache
                .admission(&runtime, &foreign, &source)
                .is_none()
        );
        other_store.remove_isolated_fixture().await.unwrap();
        let mut wrong = source.clone();
        wrong.revision.push_str("absent");
        assert!(
            runtime
                .physical_cache
                .admission(&runtime, operations, &wrong)
                .is_none()
        );
        assert!(runtime.physical_source(&wrong, &cancel).await.is_err());
        let restored = runtime.physical_source(&source, &cancel).await.unwrap();
        let stopped = crate::CancelSource::new();
        stopped.cancel();
        assert!(runtime.physical_source(&source, &stopped).await.is_err());
        let revision = operations
            .store()
            .revision(&source.revision)
            .await
            .unwrap()
            .unwrap();
        operations
            .store()
            .edit(
                &revision.problem,
                Some(&revision.key),
                "retire-physical-manifest",
                &[super::super::durable::source_edit(
                    "manifest".into(),
                    "physical:documents".into(),
                    "manifest".into(),
                    "physical:documents:manifest:v1",
                    b"{}".to_vec(),
                )],
            )
            .await
            .unwrap();
        operations.store().forget_history(&revision).await.unwrap();
        // Fixture: end the cached protection while retaining its stale immutable value.
        // This exercises the fresh server check, rather than clearing away the cache hit.
        for context in [&physical, &restored] {
            for pin in &context._source_owner.as_ref().unwrap().pins {
                operations.store().release(pin).await.unwrap();
            }
        }
        let page = operations
            .store()
            .reclaim_page(&revision.problem, "")
            .await
            .unwrap();
        assert!(page.memberships > 0);
        assert!(
            runtime
                .physical_cache
                .admission(&runtime, operations, &source)
                .is_some()
        );
        assert!(
            runtime.physical_source(&source, &cancel).await.is_err(),
            "reclaimed revision cannot be recovered from an in-memory admission"
        );
    }
    #[tokio::test]
    async fn canonical_physical_admission_actual_changed_revision_evicts_old_entry() {
        let (runtime, source) = fixture().await;
        let cancel = crate::CancelSource::new();
        let first = runtime.physical_source(&source, &cancel).await.unwrap();
        let operations = runtime.operations().unwrap();
        let revision = operations
            .store()
            .revision(&source.revision)
            .await
            .unwrap()
            .unwrap();
        let next = operations
            .store()
            .edit(
                &revision.problem,
                Some(&revision.key),
                "equivalent-physical-revision",
                &[],
            )
            .await
            .unwrap();
        let revised = PhysicalSource {
            revision: next.key,
            identity: source.identity,
        };
        let second = runtime.physical_source(&revised, &cancel).await.unwrap();
        assert_eq!(first.identity(), second.identity());
        assert!(
            !first.same_sources(&second),
            "changed exact revision re-admits original source rows even when physical meaning is equal"
        );
        assert!(
            runtime
                .physical_cache
                .admission(&runtime, operations, &source)
                .is_none()
        );
        assert_eq!(
            runtime.physical_cache.report()[0].entries,
            1,
            "replacement keeps admission retention bounded to one exact revision"
        );
        assert!(
            !first.sources.is_empty(),
            "eviction preserves the old active immutable admission"
        );
    }
}
fn checkpoint(cancel: &crate::CancelSource) -> Result<(), WorkflowError> {
    if cancel.token().is_cancelled() {
        Err(crate::math::MathRuntimeError::Cancelled.into())
    } else {
        Ok(())
    }
}
impl CacheComponent for PhysicalCache {
    fn invalidate(&self) {
        self.clear();
    }
    fn report(&self) -> Vec<CacheReport> {
        let admission = self
            .admission
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let rows = self
            .rows
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        vec![CacheReport {
            name: NAME.into(),
            capacity_bytes: 0,
            policy_limit_bytes: 2 * RETAINED_BYTES,
            retained_bytes: admission.as_ref().map_or(0, |e| e.bytes)
                + rows.as_ref().map_or(0, |e| e.bytes),
            entries: usize::from(admission.is_some()) + usize::from(rows.is_some()),
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            bypasses: 0,
            live_bytes: None,
            pinned_bytes: None,
            inflight_bytes: None,
            active_loads: None,
            evictions: None,
        }]
    }
    fn details(
        &self,
        rows: &mut Vec<CacheEntryReport>,
        limit: usize,
        owner: &MemoryReservation,
        budget: usize,
    ) -> datafusion::common::Result<()> {
        for report in self.report() {
            if report.entries > 0 && rows.len() < limit {
                pse_engine::cache_service::details::reserve_inventory(owner, 256, budget)?;
                rows.push(CacheEntryReport {
                    cache: report.name,
                    key: "exact physical owners".into(),
                    bytes: report.retained_bytes,
                    hits: report.hits,
                });
            }
        }
        Ok(())
    }
    fn execution_report(&self) -> Vec<(&'static str, Option<usize>)> {
        Vec::new()
    }
}
