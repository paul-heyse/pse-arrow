// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The deployment resource budget: memory, spill, threads and execution settings
//! (blueprint §14.3, §18.8, ADR-0046).
//!
//! One budget per process, shared by every snapshot session. ADR-0046 exists because the
//! alternative — a full deployment budget per session — means two sessions in one process
//! can each consume the whole of it, and the limit stops being a limit exactly when it
//! matters.
//!
//! # What the budget does not promise
//!
//! The fallible guarantee covers *accounted* consumers: DataFusion's query operators and
//! the platform buffers that reserve through [`pse_ids::MemoryReserver`] before
//! allocating. It does not cover the global allocator, a solver process, or any Arrow
//! array built outside a reservation. ADR-0046 states this in the same breath as the
//! guarantee, and so does this type, because a budget whose scope is assumed rather than
//! read is a budget that will be blamed for the wrong outage.

use std::num::NonZeroUsize;
use std::path::PathBuf;

use pse_catalog::{ExecutionSettings, ThreadBudget};

use crate::error::RuntimeError;

/// The configuration key naming the spill directory.
const KEY_SPILL_DIR: &str = "pse.runtime.spill_dir";

/// The configuration key naming the hashing thread policy.
const KEY_HASHING_MAY_USE_POOL: &str = "pse.runtime.hashing_may_use_pool";

/// Everything one process may consume (blueprint §14.3, §18.8).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceBudget {
    /// The accounted memory limit. Non-zero because a pool that reports an unbounded
    /// limit cannot refuse anything, and §14.3's guarantee is the refusal.
    pub memory_limit_bytes: NonZeroUsize,
    /// Where the disk manager writes spill files.
    pub spill_dir: PathBuf,
    /// The cap on the total spill directory size.
    pub max_temp_dir_bytes: u64,
    /// How many consumers a resource report names.
    pub top_consumers: NonZeroUsize,
    /// The shared thread budget (§18.8).
    pub threads: ThreadBudget,
    /// The session execution settings.
    pub execution: ExecutionSettings,
    /// Whether artifact hashing may take pool threads (`blake3::update_rayon`).
    ///
    /// `false` in phase 0. §18.8 gives the pool to DataFusion and says hashing and
    /// diagnostics never take pool threads during a solve; until there is a measurement
    /// that says which side wins when they compete, the honest setting is the one that
    /// cannot compete.
    pub hashing_may_use_pool: bool,
}

impl ResourceBudget {
    /// Checks the budget against the host and against itself.
    ///
    /// # Errors
    ///
    /// - [`RuntimeError::Catalog`] wrapping `config::invalid` when the thread budget
    ///   oversubscribes its pool (§18.8).
    /// - [`RuntimeError::ConfigInvalid`] when the spill directory is absent, is not a
    ///   directory, or is marked read-only, and when `hashing_may_use_pool` is set in
    ///   phase 0.
    ///
    /// The spill directory is checked here, at construction, rather than at the first
    /// spill: a full or missing temp filesystem discovered mid-pass is
    /// `runtime::infrastructure` after an hour of work, and discovered here it is a
    /// configuration error before any.
    pub fn validate(&self) -> Result<(), RuntimeError> {
        self.threads.validate()?;

        let metadata =
            std::fs::metadata(&self.spill_dir).map_err(|error| RuntimeError::ConfigInvalid {
                key: KEY_SPILL_DIR.to_owned(),
                reason: format!("`{}` cannot be read: {error}", self.spill_dir.display()),
            })?;
        if !metadata.is_dir() {
            return Err(RuntimeError::ConfigInvalid {
                key: KEY_SPILL_DIR.to_owned(),
                reason: format!("`{}` is not a directory", self.spill_dir.display()),
            });
        }
        if metadata.permissions().readonly() {
            return Err(RuntimeError::ConfigInvalid {
                key: KEY_SPILL_DIR.to_owned(),
                reason: format!("`{}` is read-only", self.spill_dir.display()),
            });
        }

        if self.hashing_may_use_pool {
            return Err(RuntimeError::ConfigInvalid {
                key: KEY_HASHING_MAY_USE_POOL.to_owned(),
                reason: "phase 0 hashes on the calling thread; §18.8 gives the pool to \
                         DataFusion and register R-23 owns any change"
                    .to_owned(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use pse_catalog::CatalogError;

    use super::*;

    /// `NonZeroUsize` from a literal the test knows is non-zero.
    fn count(value: usize) -> NonZeroUsize {
        let Some(value) = NonZeroUsize::new(value) else {
            panic!("the fixture counts are non-zero");
        };
        value
    }

    /// A directory this test owns, named after the case so two cases never collide.
    fn scratch(name: &str) -> PathBuf {
        let directory = std::env::temp_dir().join(format!("pse-runtime-budget-{name}"));
        let Ok(()) = fs::create_dir_all(&directory) else {
            panic!("the scratch directory is creatable");
        };
        directory
    }

    fn budget(spill_dir: PathBuf) -> ResourceBudget {
        ResourceBudget {
            memory_limit_bytes: count(256 << 20),
            spill_dir,
            max_temp_dir_bytes: 4 << 30,
            top_consumers: count(5),
            threads: ThreadBudget {
                pool_threads: count(8),
                target_partitions: count(8),
            },
            execution: ExecutionSettings::default(),
            hashing_may_use_pool: false,
        }
    }

    #[test]
    fn a_well_formed_budget_is_accepted() {
        let directory = scratch("accepted");
        assert!(budget(directory).validate().is_ok());
    }

    #[test]
    fn a_missing_spill_directory_is_a_configuration_error() {
        let directory = scratch("missing").join("no-such-subdirectory");
        let refused = budget(directory).validate();
        assert!(matches!(
            refused,
            Err(RuntimeError::ConfigInvalid { ref key, .. }) if key == "pse.runtime.spill_dir"
        ));
    }

    #[test]
    fn a_spill_path_that_is_a_file_is_refused() {
        let directory = scratch("file");
        let file = directory.join("not-a-directory");
        let Ok(()) = fs::write(&file, b"") else {
            panic!("the scratch file is writable");
        };
        let refused = budget(file).validate();
        assert!(matches!(
            refused,
            Err(RuntimeError::ConfigInvalid { ref reason, .. })
                if reason.contains("is not a directory")
        ));
    }

    #[test]
    fn hashing_may_not_take_pool_threads_in_phase_0() {
        let directory = scratch("hashing");
        let mut budget = budget(directory);
        budget.hashing_may_use_pool = true;
        let refused = budget.validate();
        assert!(matches!(
            refused,
            Err(RuntimeError::ConfigInvalid { ref key, .. })
                if key == "pse.runtime.hashing_may_use_pool"
        ));
    }

    #[test]
    fn an_oversubscribed_thread_budget_is_refused_with_the_catalog_class() {
        let directory = scratch("threads");
        let mut budget = budget(directory);
        budget.threads.target_partitions = count(16);
        let refused = budget.validate();
        assert!(matches!(
            refused,
            Err(RuntimeError::Catalog(CatalogError::ConfigInvalid { ref key, .. }))
                if key == "datafusion.execution.target_partitions"
        ));
    }
}
