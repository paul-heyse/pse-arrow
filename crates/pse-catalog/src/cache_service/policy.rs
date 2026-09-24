// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Durable cache capacities composed with the native engine resource policy.
use pse_engine::{EngineError, cache_service::CacheBudget};
/// Delta-specific reuse and persistence policy, sharing the native query pool.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeltaCacheBudget {
    /// Generic native cache admission and query working capacity.
    pub native: CacheBudget,
    /// Retained native snapshot capacity.
    pub snapshot_bytes: usize,
    /// Decoded exact-selection capacity.
    pub resident_bytes: usize,
    /// Maximum checksum replay tail; zero disables incremental replay.
    pub crc_replay_max_commits: u64,
    /// Commit checksum cadence; zero disables checksum publication.
    pub checksum_interval: u64,
}
impl DeltaCacheBudget {
    /// Disable reuse while preserving an explicit query working set.
    #[must_use]
    pub fn disabled(working_bytes: usize) -> Self {
        Self {
            native: CacheBudget::disabled(working_bytes),
            snapshot_bytes: 0,
            resident_bytes: 0,
            crc_replay_max_commits: 0,
            checksum_interval: 0,
        }
    }
    /// Conservative admission fractions, not measured optimal capacities.
    #[must_use]
    pub fn for_memory(bytes: usize) -> Self {
        Self {
            native: CacheBudget::for_memory(bytes),
            snapshot_bytes: (bytes / 32) * 2,
            resident_bytes: (bytes / 32) * 4,
            ..Self::disabled(1)
        }
    }
    /// Check all retained families against the same finite pool.
    /// # Errors
    /// Overflow, invalid native policy, or aggregate capacities exceeding the pool.
    pub fn validate(&self, bytes: usize) -> Result<(), EngineError> {
        let available = bytes
            .checked_sub(self.snapshot_bytes)
            .and_then(|b| b.checked_sub(self.resident_bytes))
            .ok_or_else(|| EngineError::ConfigInvalid {
                key: "pse.runtime.cache".into(),
                reason: "Delta retained capacities exceed the shared pool".into(),
            })?;
        self.native.validate(available)
    }
}
