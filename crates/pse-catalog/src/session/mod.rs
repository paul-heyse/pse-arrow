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
//! Sessions retain their profile, function implementations, extension registry and
//! exact admitted or unpublished relation inventory through execution and plan decoding.

pub mod admission;
mod candidate;
pub mod capture;
mod commands;
mod computed;
pub mod config;
pub mod execution;
mod factory;
mod facts;
mod functions;
mod inspection;
mod materialized;
pub mod mutation;
mod native;
mod observation;
pub mod operation;
pub mod output;
mod physical_fields;
pub mod planner;
pub mod policy;
mod preparation;
pub(crate) mod query_schema;
pub mod resolution;
mod resources;
mod roles;
mod schema_transform;
mod trace;
pub use preparation::{CompletedComputation, OwnedComputationStream, PreparedComputation};
pub mod plan_codec;
pub use observation::PlanObservation;
pub mod aggregate;
pub mod profile;
pub mod registry;
pub mod scalar;
mod snapshot_session;
pub(crate) use snapshot_session::engine;

pub use factory::SessionFactory;
pub use facts::RelationFacts;
pub use profile::{EngineProfile, EngineRules, native_engine_profile};
pub use snapshot_session::{
    SessionSemantics, SnapshotSession, build_candidate_session,
    build_candidate_session_with_cancel, build_session,
};

use std::num::NonZeroUsize;

use crate::error::CatalogError;

/// The default `RecordBatch` size DataFusion uses.
const DEFAULT_BATCH_SIZE: usize = 8_192;

/// The default spill compression; callers may select another supported codec.
const DEFAULT_SPILL_COMPRESSION: &str = "uncompressed";

/// The default cap on one spill file: 1 GiB.
const DEFAULT_MAX_SPILL_FILE_SIZE_BYTES: u64 = 1 << 30;

/// The default sort spill reservation: 10 MiB.
const DEFAULT_SORT_SPILL_RESERVATION_BYTES: usize = 10 << 20;

/// The one timezone the registry writes (blueprint §4.4, `Timestamp(ns, "UTC")`).
const DEFAULT_TIME_ZONE: &str = "UTC";

/// Shared worker capacity and native query partition count. Partitions are work
/// units scheduled on workers, not additional operating-system threads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ThreadBudget {
    /// Threads in the shared pool.
    pub pool_threads: NonZeroUsize,
    /// Partitions a query may be split into.
    pub target_partitions: NonZeroUsize,
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

impl ExecutionSettings {
    /// Validate selected settings using the pinned engine's typed configuration.
    /// # Errors
    /// An invalid selected value or zero batch size.
    pub fn validate(&self) -> Result<(), CatalogError> {
        config::validate(self)
    }
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
            assert!(config::build(ExecutionSettings::default(), budget).is_ok());
        }
    }

    #[test]
    fn partitions_are_scheduled_work_units() {
        assert!(
            config::build(
                ExecutionSettings::default(),
                ThreadBudget {
                    pool_threads: threads(2),
                    target_partitions: threads(8)
                }
            )
            .is_ok()
        );
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
