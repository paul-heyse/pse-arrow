// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Session construction: the engine profile, the extension registry, semantic admission
//! and the typed configuration path (blueprint §5.4, §14.3, §18.8).
//!
//! Two of those settings are here rather than in the (deferred) configuration module,
//! because `pse-runtime`'s resource budget needs them now: a runtime that cannot state
//! its thread budget cannot validate it, and §18.8 is explicit that oversubscription "is
//! a configuration error, not a runtime surprise" — a sentence that only becomes true
//! when something checks it.
//!
//! Everything assembled here goes through typed `ConfigOptions`. `SessionConfig::set_str`
//! panics on an invalid value instead of returning one, which turns a `config::invalid`
//! row in the §23.2 taxonomy into a process abort; it is banned in `clippy.toml` and by a
//! governance grep.
//!
//! Phase 0 lands the two configuration types. The profile, rule catalog, extension
//! registry, admission rules and session builder are packet B-session.

pub mod admission;
pub mod config;
pub mod profile;
pub mod registry;

use std::num::NonZeroUsize;

use crate::error::CatalogError;

/// The DataFusion setting that bounds query parallelism.
const KEY_TARGET_PARTITIONS: &str = "datafusion.execution.target_partitions";

/// The default `RecordBatch` size DataFusion uses.
const DEFAULT_BATCH_SIZE: usize = 8_192;

/// The only spill compression the platform selects: none.
const DEFAULT_SPILL_COMPRESSION: &str = "uncompressed";

/// The default cap on one spill file: 1 GiB.
const DEFAULT_MAX_SPILL_FILE_SIZE_BYTES: u64 = 1 << 30;

/// The default sort spill reservation: 10 MiB.
const DEFAULT_SORT_SPILL_RESERVATION_BYTES: usize = 10 << 20;

/// The one timezone the registry writes (blueprint §4.4, `Timestamp(ns, "UTC")`).
const DEFAULT_TIME_ZONE: &str = "UTC";

/// How many threads the process may use, and how many partitions a query may use
/// (blueprint §18.8).
///
/// One configuration owns the whole budget: the DataFusion `tokio` runtime and
/// `target_partitions`, the `rayon` pool for kernels and passes, artifact hashing, the
/// diagnostics linear algebra and the solver's threads. Two of those asking for the same
/// cores is not a slowdown that shows up in a profile — it is a configuration error, and
/// [`Self::validate`] is where it is caught.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ThreadBudget {
    /// Threads in the shared pool.
    pub pool_threads: NonZeroUsize,
    /// Partitions a query may be split into.
    pub target_partitions: NonZeroUsize,
}

impl ThreadBudget {
    /// Checks the budget against itself.
    ///
    /// # Errors
    ///
    /// [`CatalogError::ConfigInvalid`] naming `datafusion.execution.target_partitions`
    /// when more partitions are asked for than the pool has threads. §18.8 calls this
    /// oversubscription and requires it to be a configuration error.
    pub fn validate(&self) -> Result<(), CatalogError> {
        if self.target_partitions > self.pool_threads {
            return Err(CatalogError::ConfigInvalid {
                key: KEY_TARGET_PARTITIONS.to_owned(),
                reason: format!(
                    "{} partitions oversubscribe a pool of {} threads (blueprint §18.8)",
                    self.target_partitions, self.pool_threads
                ),
            });
        }
        Ok(())
    }
}

/// The execution settings a snapshot session is built with (blueprint §5.4, §14.3).
///
/// Every one of these reaches the engine through typed `ConfigOptions`, and every one is
/// recorded in the engine profile: §20.4's reproduction re-executes "under recorded
/// policies", and a setting that is not recorded is a policy nobody can reproduce.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExecutionSettings {
    /// Rows per `RecordBatch`.
    pub batch_size: usize,
    /// Spill compression; `uncompressed` unless a measurement says otherwise.
    pub spill_compression: String,
    /// The cap on a single spill file.
    pub max_spill_file_size_bytes: u64,
    /// Bytes a sort reserves up front for its own spilling.
    pub sort_spill_reservation_bytes: usize,
    /// The session timezone.
    pub time_zone: String,
}

impl Default for ExecutionSettings {
    fn default() -> Self {
        Self {
            batch_size: DEFAULT_BATCH_SIZE,
            spill_compression: DEFAULT_SPILL_COMPRESSION.to_owned(),
            max_spill_file_size_bytes: DEFAULT_MAX_SPILL_FILE_SIZE_BYTES,
            sort_spill_reservation_bytes: DEFAULT_SORT_SPILL_RESERVATION_BYTES,
            time_zone: DEFAULT_TIME_ZONE.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `NonZeroUsize` from a literal the test knows is non-zero.
    fn threads(count: usize) -> NonZeroUsize {
        let Some(value) = NonZeroUsize::new(count) else {
            panic!("the fixture counts are non-zero");
        };
        value
    }

    #[test]
    fn a_budget_within_its_pool_is_accepted() {
        for (pool, partitions) in [(8, 8), (8, 4), (1, 1)] {
            let budget = ThreadBudget {
                pool_threads: threads(pool),
                target_partitions: threads(partitions),
            };
            assert!(budget.validate().is_ok(), "{pool} threads, {partitions}");
        }
    }

    #[test]
    fn more_partitions_than_threads_is_a_configuration_error() {
        let budget = ThreadBudget {
            pool_threads: threads(4),
            target_partitions: threads(8),
        };
        let refused = budget.validate();
        assert!(matches!(
            refused,
            Err(CatalogError::ConfigInvalid { ref key, .. })
                if key == "datafusion.execution.target_partitions"
        ));
    }

    #[test]
    fn the_execution_defaults_are_the_declared_ones() {
        let settings = ExecutionSettings::default();
        assert_eq!(settings.batch_size, 8_192);
        assert_eq!(settings.spill_compression, "uncompressed");
        assert_eq!(settings.max_spill_file_size_bytes, 1_073_741_824);
        assert_eq!(settings.sort_spill_reservation_bytes, 10_485_760);
        assert_eq!(settings.time_zone, "UTC");
    }
}
