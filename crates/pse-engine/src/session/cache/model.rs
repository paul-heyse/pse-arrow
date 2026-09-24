// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Completed-only model retention. Idle entries own no session or producing plan.
use super::Cached;
use crate::{
    cache_service::{CacheComponent, CacheEntryReport, CacheReport},
    session::EngineSession,
};
use datafusion::{
    common::{Result, TableReference},
    execution::{
        cache::{Cache, CacheKey, CacheValue, default_cache::DefaultCache},
        memory_pool::MemoryReservation,
    },
};
use std::sync::{
    Arc, Mutex, Weak,
    atomic::{AtomicUsize, Ordering},
};

pub(super) struct Selection {
    // Only the logical producer owns this proof. Idle cache keys retain a Weak
    // selection, so completed entries never keep input plans or sessions alive.
    admission: Arc<super::admission::Admission>,
    reservation: MemoryReservation,
}
impl std::fmt::Debug for Selection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelResultSelection")
            .finish_non_exhaustive()
    }
}
impl Selection {
    pub(super) fn capture(
        session: &EngineSession,
        admission: &Arc<super::admission::Admission>,
    ) -> Result<Option<Arc<Self>>> {
        if !session
            .effective_policy()
            .map_err(pse_columnar::external)?
            .requirements
            .is_empty()
            || !admission.retention_eligible()
        {
            return Ok(None);
        }
        let reservation = pse_columnar::MemoryConsumer::new("native:model-result-selection")
            .register(&session.pool);
        if reservation.try_grow(size_of::<Self>() + 128).is_err() {
            return Ok(None);
        }
        Ok(Some(Arc::new(Self {
            admission: admission.clone(),
            reservation,
        })))
    }
    pub(super) fn matches(&self, session: &EngineSession) -> bool {
        self.admission.matches(session)
    }
}

/// Own both tokens: hashes locate candidates, actual owner equality decides reuse.
#[derive(Clone, Debug)]
pub(super) struct Key {
    producer: Arc<()>,
    selection: Weak<Selection>,
    selection_extent: usize,
}
impl Key {
    pub(super) fn new(producer: Arc<()>, selection: &Arc<Selection>) -> Self {
        Self {
            producer,
            selection_extent: selection
                .reservation
                .size()
                .saturating_add(selection.admission.retained_extent()),
            selection: Arc::downgrade(selection),
        }
    }
}
impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.producer, &other.producer)
            && Weak::ptr_eq(&self.selection, &other.selection)
    }
}
impl Eq for Key {}
impl std::hash::Hash for Key {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&Arc::as_ptr(&self.producer), state);
        std::hash::Hash::hash(&self.selection.as_ptr(), state);
    }
}
impl CacheKey for Key {
    fn size(&self) -> usize {
        512 + self.selection_extent
    }
    fn table_ref(&self) -> Option<&TableReference> {
        None
    }
}
#[derive(Clone)]
struct Value(Arc<Cached>);
impl CacheValue for Value {
    fn size(&self) -> usize {
        self.0.retained_bytes()
    }
}
pub(crate) struct ModelCache {
    entries: DefaultCache<Key, Value>,
    admission: Mutex<()>,
    epoch: AtomicUsize,
    live: Arc<AtomicUsize>,
    pinned: Arc<AtomicUsize>,
    hits: AtomicUsize,
    misses: AtomicUsize,
    bypasses: AtomicUsize,
}
impl ModelCache {
    pub(crate) fn new(bytes: usize) -> Arc<Self> {
        Arc::new(Self {
            entries: DefaultCache::new(bytes).with_name("pse.cache.model_results"),
            admission: Mutex::default(),
            epoch: AtomicUsize::new(0),
            live: Arc::default(),
            pinned: Arc::default(),
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
            bypasses: AtomicUsize::new(0),
        })
    }
    pub(super) fn get(&self, key: &Key) -> Option<Arc<Cached>> {
        let value = key
            .selection
            .upgrade()
            .and_then(|_| self.entries.get(key).map(|v| v.0));
        if value.is_some() {
            &self.hits
        } else {
            &self.misses
        }
        .fetch_add(1, Ordering::Relaxed);
        value
    }
    pub(super) fn epoch(&self) -> usize {
        self.epoch.load(Ordering::Acquire)
    }
    pub(super) fn complete(&self, key: &Key, value: Cached) -> Cached {
        // The completion cell still owns this value if admission declines it.
        // Only resident pages cross attempts; spill owners remain invocation-local.
        let fits = key
            .size()
            .checked_add(value.retained_bytes())
            .and_then(|bytes| bytes.checked_add(512))
            .is_some_and(|bytes| bytes <= self.entries.cache_limit());
        if !fits || !value.resident() || !value.reserve_identity(512) {
            self.bypasses.fetch_add(1, Ordering::Relaxed);
            return value;
        }
        value.account(self.live.clone(), self.pinned.clone())
    }
    pub(super) fn retain(&self, key: &Key, value: Arc<Cached>, epoch: usize) {
        let _guard = self
            .admission
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if value.live.is_some()
            && value.resident()
            && key.selection.upgrade().is_some()
            && epoch == self.epoch()
        {
            self.entries.put(key, Value(value));
        }
    }
}
impl CacheComponent for ModelCache {
    fn invalidate(&self) {
        let _guard = self
            .admission
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.epoch.fetch_add(1, Ordering::AcqRel);
        self.entries.clear();
    }
    fn report(&self) -> Vec<CacheReport> {
        vec![CacheReport {
            name: self.entries.name(),
            capacity_bytes: 0,
            retained_bytes: self.entries.memory_used(),
            policy_limit_bytes: self.entries.cache_limit(),
            live_bytes: Some(self.live.load(Ordering::Acquire)),
            pinned_bytes: Some(self.pinned.load(Ordering::Acquire)),
            inflight_bytes: None,
            active_loads: None,
            evictions: None,
            entries: self.entries.len(),
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            bypasses: self.bypasses.load(Ordering::Relaxed),
        }]
    }
    fn details(
        &self,
        rows: &mut Vec<CacheEntryReport>,
        limit: usize,
        owner: &MemoryReservation,
        budget: usize,
    ) -> Result<()> {
        let _guard = self
            .admission
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        crate::cache_service::details::reserve_inventory(
            owner,
            self.entries.memory_used(),
            budget,
        )?;
        for (key, entry) in self
            .entries
            .list_entries()
            .into_iter()
            .take(limit.saturating_sub(rows.len()))
        {
            rows.push(CacheEntryReport {
                cache: self.entries.name(),
                key: format!("producer:{:p}", Arc::as_ptr(&key.producer)),
                bytes: entry.size_bytes,
                hits: entry.hits,
            });
        }
        Ok(())
    }
    fn execution_report(&self) -> Vec<(&'static str, Option<usize>)> {
        vec![]
    }
}

impl std::fmt::Debug for ModelCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelCache")
            .field("entries", &self.entries.len())
            .finish_non_exhaustive()
    }
}
