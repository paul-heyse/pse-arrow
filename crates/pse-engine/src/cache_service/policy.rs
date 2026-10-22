// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One deployment declaration for native cache admission (ADR-0070).

use crate::EngineError;
use std::num::NonZeroUsize;

/// Cache capacities share the query pool; they never establish a second budget.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CacheBudget {
    /// Native SQL syntax retention, independent of plan binding.
    pub syntax_bytes: usize,
    /// Completed pure model outputs retained across invocations.
    pub model_result_bytes: usize,
    /// Concurrent outer native queries; nested queries inherit their caller admission.
    pub concurrent_queries: NonZeroUsize,
    /// Independent output work admitted by one deployment.
    pub concurrent_outputs: NonZeroUsize,
    /// Maximum simultaneous native open/decode operations.
    pub concurrent_loads: NonZeroUsize,
    /// Aggregate allowance for native replay/decode staging.
    pub inflight_bytes: usize,
    /// Maximum staging allocation for explicit entry-detail inspection.
    pub inspection_bytes: usize,
    /// Per-reader native Parquet predicate result cache; every reader reserves this amount.
    pub predicate_cache_bytes: usize,
    /// Aggregate limit across active readers, including their prefetched files.
    pub predicate_total_bytes: usize,
    /// Pool bytes cache admission must leave available to query operators.
    pub working_bytes: usize,
}

impl CacheBudget {
    /// Explicit cache-off policy; execution still uses the same native providers.
    #[must_use]
    pub fn disabled(working_bytes: usize) -> Self {
        Self {
            syntax_bytes: 0,
            model_result_bytes: 0,
            concurrent_queries: NonZeroUsize::MIN,
            concurrent_outputs: NonZeroUsize::MIN,
            concurrent_loads: NonZeroUsize::MIN,
            inflight_bytes: 0,
            inspection_bytes: 0,
            working_bytes,
            predicate_cache_bytes: 0,
            predicate_total_bytes: 0,
        }
    }

    /// Conservative deployment policy selected by the Plan 09 cache matrix.
    /// These fractions are admission limits, not a measured optimal configuration.
    #[must_use]
    pub fn for_memory(memory_bytes: usize) -> Self {
        let portion = memory_bytes / 32;
        Self {
            syntax_bytes: (memory_bytes / 256).min(4 << 20),
            model_result_bytes: portion,
            concurrent_queries: NonZeroUsize::new(2).unwrap_or(NonZeroUsize::MIN),
            concurrent_outputs: NonZeroUsize::new(4).unwrap_or(NonZeroUsize::MIN),
            predicate_total_bytes: portion.min(16 << 20),
            // Opt-in until the complete physical scan fanout is qualified at I19.
            // A nominal per-reader limit alone cannot bound an arbitrary join.
            predicate_cache_bytes: 0,
            concurrent_loads: NonZeroUsize::new(if portion >= (1 << 20) { 2 } else { 1 })
                .unwrap_or(NonZeroUsize::MIN),
            inflight_bytes: portion * 2,
            inspection_bytes: portion.min(4 << 20),
            ..Self::disabled((memory_bytes / 2).max(1))
        }
    }

    fn commitment(&self) -> Option<usize> {
        [
            self.syntax_bytes,
            self.model_result_bytes,
            self.inflight_bytes,
            self.inspection_bytes,
            self.working_bytes,
            self.predicate_total_bytes,
        ]
        .into_iter()
        .try_fold(0usize, usize::checked_add)
    }

    /// Validate policy arithmetic and fit mandatory commitments in the shared pool.
    /// # Errors
    /// Policy arithmetic overflows or mandatory capacities exceed the pool.
    pub fn validate(&self, memory_bytes: usize) -> Result<(), EngineError> {
        if self.commitment().is_none_or(|total| total > memory_bytes) || self.working_bytes == 0 {
            return Err(EngineError::ConfigInvalid {
                key: "pse.runtime.cache".into(),
                reason:
                    "cache, staging and positive query working capacities must fit the shared pool"
                        .into(),
            });
        }
        if self
            .predicate_cache_bytes
            .checked_mul(2)
            .is_none_or(|bytes| bytes > self.predicate_total_bytes)
        {
            return Err(EngineError::ConfigInvalid { key: "pse.runtime.cache.predicate_total_bytes".into(), reason: "aggregate predicate capacity must cover an active and a prefetched file per admitted reader".into() });
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
    fn aggregate_overflow_is_rejected() {
        let mut policy = CacheBudget::disabled(1);
        policy.syntax_bytes = usize::MAX;
        assert!(policy.validate(usize::MAX).is_err());
    }
}
