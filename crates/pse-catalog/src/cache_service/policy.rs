// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One deployment declaration for native cache admission (ADR-0070).

use crate::CatalogError;
use std::{num::NonZeroUsize, time::Duration};

/// Cache capacities share the query pool; they never establish a second budget.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CacheBudget {
    /// Native file metadata capacity, including a conservative key/node allowance.
    pub metadata_bytes: usize,
    /// Native file statistics capacity.
    pub statistics_bytes: usize,
    /// Native directory listing capacity.
    pub listing_bytes: usize,
    /// Directory listing freshness; zero disables reuse of listings.
    pub listing_ttl: Option<Duration>,
    /// Retained native snapshot capacity.
    pub snapshot_bytes: usize,
    /// Retained decoded exact-selection capacity.
    pub resident_bytes: usize,
    /// Maximum simultaneous native open/decode operations.
    pub concurrent_loads: NonZeroUsize,
    /// Aggregate allowance for native replay/decode staging.
    pub inflight_bytes: usize,
    /// Maximum staging allocation for explicit entry-detail inspection.
    pub inspection_bytes: usize,
    /// Per-reader native Parquet predicate result cache; every reader reserves this amount.
    pub predicate_cache_bytes: usize,
    /// Maximum native checksum-summary replay tail; zero disables incremental CRC replay.
    pub crc_replay_max_commits: u64,
    /// Native checksum publication cadence after durable commits; zero disables it.
    pub checksum_interval: u64,
    /// Pool bytes cache admission must leave available to query operators.
    pub working_bytes: usize,
}

impl CacheBudget {
    /// Explicit cache-off policy; execution still uses the same native providers.
    #[must_use]
    pub fn disabled(working_bytes: usize) -> Self {
        Self {
            metadata_bytes: 0,
            statistics_bytes: 0,
            listing_bytes: 0,
            listing_ttl: Some(Duration::ZERO),
            snapshot_bytes: 0,
            resident_bytes: 0,
            concurrent_loads: NonZeroUsize::MIN,
            inflight_bytes: 0,
            inspection_bytes: 0,
            working_bytes,
            predicate_cache_bytes: 0,
            crc_replay_max_commits: 0,
            checksum_interval: 0,
        }
    }

    /// Conservative deployment policy selected by the Plan 09 cache matrix.
    /// These fractions are admission limits, not a measured optimal configuration.
    #[must_use]
    pub fn for_memory(memory_bytes: usize) -> Self {
        let portion = memory_bytes / 32;
        Self {
            metadata_bytes: portion.min(50 << 20),
            snapshot_bytes: portion * 2,
            resident_bytes: portion * 4,
            inflight_bytes: portion * 2,
            inspection_bytes: portion.min(4 << 20),
            ..Self::disabled((memory_bytes / 2).max(1))
        }
    }

    /// Validate overflow and leave an explicit query working set.
    /// # Errors
    /// The sum exceeds the deployment pool, or listing reuse has no freshness bound.
    pub fn validate(&self, memory_bytes: usize) -> Result<(), CatalogError> {
        let total = [
            self.metadata_bytes,
            self.statistics_bytes,
            self.listing_bytes,
            self.snapshot_bytes,
            self.resident_bytes,
            self.inflight_bytes,
            self.inspection_bytes,
            self.working_bytes,
            self.predicate_cache_bytes,
        ]
        .into_iter()
        .try_fold(0usize, usize::checked_add);
        if total.is_none_or(|total| total > memory_bytes) || self.working_bytes == 0 {
            return Err(CatalogError::ConfigInvalid {
                key: "pse.runtime.cache".into(),
                reason:
                    "cache, staging and positive query working capacities must fit the shared pool"
                        .into(),
            });
        }
        if self.listing_bytes > 0 && self.listing_ttl.is_none() {
            return Err(CatalogError::ConfigInvalid {
                key: "pse.runtime.cache.listing_ttl".into(),
                reason: "mutable directory listings require a finite freshness bound".into(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn disabled_and_fractional_policies_preserve_working_memory() {
        for bytes in [1, 1024, 1 << 20, usize::MAX] {
            assert!(CacheBudget::disabled(bytes).validate(bytes).is_ok());
            if bytes > 1 {
                assert!(CacheBudget::for_memory(bytes).validate(bytes).is_ok());
            }
        }
    }
    #[test]
    fn overflow_and_unbounded_listing_freshness_are_rejected() {
        let mut policy = CacheBudget::disabled(1);
        policy.metadata_bytes = usize::MAX;
        assert!(policy.validate(usize::MAX).is_err());
        policy.metadata_bytes = 0;
        policy.listing_bytes = 1;
        policy.listing_ttl = None;
        assert!(policy.validate(1024).is_err());
    }
}
