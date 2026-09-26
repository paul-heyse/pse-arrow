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
//! The fallible guarantee covers *accounted* consumers: DataFusion's reserving operators and
//! the platform buffers that reserve through [`pse_columnar::MemoryPool`] before
//! allocating. It does not cover the global allocator, a solver process, or any Arrow
//! array built outside a reservation. ADR-0046 states this in the same breath as the
//! guarantee, and so does this type, because a budget whose scope is assumed rather than
//! read is a budget that will be blamed for the wrong outage.

use std::num::NonZeroUsize;
use std::path::PathBuf;

pub use pse_catalog::cache_service::DeltaCacheBudget;
use pse_engine::{ExecutionSettings, ThreadBudget};

use crate::error::RuntimeError;

/// The configuration key naming the spill directory.
const KEY_SPILL_DIR: &str = "pse.runtime.spill_dir";

/// Everything one process may consume (blueprint §14.3, §18.8).
#[derive(Clone, Debug, PartialEq, Eq)]
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
    /// Native cache capacity, load staging and query headroom in the same pool.
    pub cache: DeltaCacheBudget,
    /// Finite compiler and native math allowances in this same pool.
    pub math: crate::math::MathPolicy,
    /// Whether artifact hashing may take pool threads (`blake3::update_rayon`).
    ///
    /// This grants permission; it does not require parallel hashing or change canonical
    /// framing. The caller selects the scheduling policy for its workload.
    pub hashing_may_use_pool: bool,
}

impl ResourceBudget {
    /// Checks the budget against the host and against itself.
    ///
    /// # Errors
    ///
    /// - [`RuntimeError::Catalog`] wrapping `config::invalid` when the thread or selected
    ///   execution settings are invalid.
    /// - [`RuntimeError::ConfigInvalid`] when the spill directory is absent, is not a
    ///   directory, is marked read-only, or the memory limit exceeds addressable storage.
    ///
    /// The spill directory is checked here, at construction, rather than at the first
    /// spill: a full or missing temp filesystem discovered mid-pass is
    /// `runtime::infrastructure` after an hour of work, and discovered here it is a
    /// configuration error before any.
    pub fn validate(&self) -> Result<(), RuntimeError> {
        self.execution.validate()?;
        self.math.validate()?;
        self.cache.validate(self.memory_limit_bytes.get())?;
        if isize::try_from(self.memory_limit_bytes.get()).is_err() {
            return Err(RuntimeError::ConfigInvalid {
                key: "datafusion.runtime.memory_limit".to_owned(),
                reason: "the memory limit must fit the platform allocation envelope".to_owned(),
            });
        }
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

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

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
            cache: DeltaCacheBudget::disabled(1),
            math: Default::default(),
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
    fn hashing_pool_permission_is_a_selected_policy() {
        let directory = scratch("hashing");
        let mut budget = budget(directory);
        budget.hashing_may_use_pool = true;
        assert!(budget.validate().is_ok());
    }

    #[test]
    fn partitions_may_exceed_worker_threads() {
        let directory = scratch("threads");
        let mut budget = budget(directory);
        budget.threads.target_partitions = count(16);
        assert!(budget.validate().is_ok());
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn workstation_resource_policy_is_not_a_fixture_sized_ceiling() {
        let mut budget = budget(scratch("workstation"));
        budget.memory_limit_bytes = count(32 << 30);
        budget.max_temp_dir_bytes = 256 << 30;
        assert!(budget.validate().is_ok());
    }
}
