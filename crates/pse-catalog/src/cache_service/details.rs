// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit bounded diagnostic enumeration, separate from constant-space counters.
use super::NativeCacheService;
use datafusion::{
    common::{DataFusionError, Result},
    execution::{
        cache::Cache,
        memory_pool::{MemoryConsumer, MemoryReservation},
    },
};
/// One native diagnostic key; never an executable provider or persisted identity.
#[derive(Debug)]
pub struct CacheEntryReport {
    /// Native cache name.
    pub cache: String,
    /// Qualified native diagnostic key.
    pub key: String,
    /// Library-reported retained value extent.
    pub bytes: usize,
    /// Library-reported entry hits.
    pub hits: usize,
}
/// The staging reservation remains alive until callers finish projecting rows.
#[derive(Debug)]
pub struct CacheEntries {
    rows: Vec<CacheEntryReport>,
    _owner: MemoryReservation,
}
impl CacheEntries {
    /// Borrow the bounded result without losing its staging owner.
    pub fn rows(&self) -> &[CacheEntryReport] {
        &self.rows
    }
}
impl NativeCacheService {
    /// Enumerate native keys after reserving staging under each insertion guard.
    /// `DefaultCache` clones its entire inventory internally, so a small result limit
    /// cannot justify an unbounded intermediate clone. Oversize requests refuse.
    /// # Errors
    /// The configured inspection budget or shared pool cannot hold the native inventory.
    pub fn inspect_entries(&self, limit: usize) -> Result<CacheEntries> {
        let owner = MemoryConsumer::new("pse.cache.inspection").register(&self.pool);
        if limit == 0 {
            return Ok(CacheEntries {
                rows: vec![],
                _owner: owner,
            });
        }
        let stores = self
            .stores
            .lock()
            .map_err(|_| DataFusionError::Internal("store identity lock poisoned".into()))?;
        let store_bytes = stores
            .iter()
            .try_fold(0usize, |sum, binding| {
                sum.checked_add(binding.reservation.size())
            })
            .ok_or_else(|| {
                DataFusionError::ResourcesExhausted("inspection identity extent overflow".into())
            })?;
        let budget = self.policy.inspection_bytes.saturating_sub(store_bytes);
        let mut rows = Vec::new();
        envelope_rows(&self.metadata, &mut rows, limit, &owner, budget)?;
        envelope_rows(&self.statistics, &mut rows, limit, &owner, budget)?;
        envelope_rows(&self.listing, &mut rows, limit, &owner, budget)?;
        self.snapshots.details(&mut rows, limit, &owner, budget)?;
        self.resident.details(&mut rows, limit, &owner, budget)?;
        rows.sort_by(|left, right| (&left.cache, &left.key).cmp(&(&right.cache, &right.key)));
        Ok(CacheEntries {
            rows,
            _owner: owner,
        })
    }
}

// Each caller holds the cache's insertion guard from sampling through native
// enumeration. Other operations can only shrink the sampled extent.
pub(super) fn reserve_inventory(
    owner: &MemoryReservation,
    retained: usize,
    budget: usize,
) -> Result<()> {
    let bytes = retained
        .checked_mul(4)
        .and_then(|bytes| bytes.checked_add(512))
        .ok_or_else(|| DataFusionError::ResourcesExhausted("inspection extent overflow".into()))?;
    if owner
        .size()
        .checked_add(bytes)
        .is_none_or(|size| size > budget)
    {
        return Err(DataFusionError::ResourcesExhausted(
            "native entry enumeration exceeds the declared inspection budget".into(),
        ));
    }
    owner.try_grow(bytes)
}

fn envelope_rows<
    K: datafusion::execution::cache::CacheKey + std::fmt::Debug,
    V: datafusion::execution::cache::CacheValue,
>(
    cache: &super::envelope::Envelope<K, V>,
    rows: &mut Vec<CacheEntryReport>,
    limit: usize,
    owner: &MemoryReservation,
    budget: usize,
) -> Result<()> {
    if rows.len() >= limit {
        return Ok(());
    }
    let _guard = cache
        .admission
        .lock()
        .map_err(|_| DataFusionError::Internal("cache admission lock poisoned".into()))?;
    reserve_inventory(owner, cache.report().retained_bytes, budget)?;
    for (key, value) in cache.list_entries() {
        if rows.len() >= limit {
            break;
        }
        rows.push(CacheEntryReport {
            cache: cache.name(),
            key: format!("{key:?}"),
            bytes: value.size_bytes,
            hits: value.hits,
        });
    }
    Ok(())
}
