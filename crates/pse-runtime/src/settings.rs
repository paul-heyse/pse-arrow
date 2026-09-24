// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native construction authority and mechanical boundary field projections.
use crate::{DeltaCacheBudget, ResourceBudget, RuntimeError};
use pse_engine::{ExecutionSettings, ThreadBudget};
use std::num::NonZeroUsize;

/// Native input names, types, defaults and read-back; callbacks supply boundary syntax.
#[macro_export]
macro_rules! engine_settings_fields {
    ($emit:ident, $budget:ident) => { $emit! { $budget;
        memory_limit_bytes: usize, "int" => usize, $budget.memory_limit_bytes.get();
        threads: usize, "int" => usize, $budget.threads.pool_threads.get();
        spill_dir: std::path::PathBuf, "str" => String, $budget.spill_dir.to_string_lossy().into_owned();
        max_spill_bytes: u64, "int" => u64, $budget.max_temp_dir_bytes;
        batch_size: usize, "int" => usize, $budget.execution.batch_size;
        target_partitions: Option<usize>, "int | None", None => usize, $budget.threads.target_partitions.get();
        top_consumers: Option<usize>, "int | None", None => usize, $budget.top_consumers.get();
        concurrent_queries: Option<usize>, "int | None", None => usize, $budget.cache.native.concurrent_queries.get();
        concurrent_outputs: Option<usize>, "int | None", None => usize, $budget.cache.native.concurrent_outputs.get();
        model_result_bytes: Option<usize>, "int | None", None => usize, $budget.cache.native.model_result_bytes;
        hashing_may_use_pool: bool, "bool", false => bool, $budget.hashing_may_use_pool;
        spill_compression: Option<String>, "str | None", None => String, $budget.execution.spill_compression.clone();
        max_spill_file_size_bytes: Option<u64>, "int | None", None => u64, $budget.execution.max_spill_file_size_bytes;
        sort_spill_reservation_bytes: Option<usize>, "int | None", None => usize, $budget.execution.sort_spill_reservation_bytes;
        time_zone: Option<String>, "str | None", None => String, $budget.execution.time_zone.clone();
    } };
}
macro_rules! input {
    ($budget:ident; $($name:ident: $ty:ty, $hint:literal $(, $default:expr)? => $out:ty, $read:expr;)*) => {
        /// Deployment inputs; `resolve` is the construction policy.
        #[derive(Debug)]
        pub struct EngineSettingsInput { $(#[doc = concat!("Declared ", stringify!($name), " input.")] pub $name: $ty,)* }
    };
}
engine_settings_fields!(input, budget);
impl EngineSettingsInput {
    /// Apply defaults and admit filesystem, execution and cache policy.
    /// # Errors
    /// Invalid ranges, directory, native settings or aggregate capacity.
    pub fn resolve(self, cache: Option<DeltaCacheBudget>) -> Result<ResourceBudget, RuntimeError> {
        let defaults = ExecutionSettings::default();
        let positive = |value, key: &str| {
            NonZeroUsize::new(value).ok_or_else(|| RuntimeError::ConfigInvalid {
                key: key.into(),
                reason: "must be positive".into(),
            })
        };
        let threads = positive(self.threads, "threads")?;
        let spill_dir =
            self.spill_dir
                .canonicalize()
                .map_err(|error| RuntimeError::ConfigInvalid {
                    key: "spill_dir".into(),
                    reason: error.to_string(),
                })?;
        let supplied_concurrency = cache.as_ref().map(|cache| {
            (
                cache.native.concurrent_queries.get(),
                cache.native.concurrent_outputs.get(),
            )
        });
        let mut cache =
            cache.unwrap_or_else(|| DeltaCacheBudget::for_memory(self.memory_limit_bytes));
        cache.native.concurrent_queries = positive(
            self.concurrent_queries
                .or(supplied_concurrency.map(|limits| limits.0))
                .unwrap_or(threads.get().min(2)),
            "concurrent_queries",
        )?;
        cache.native.concurrent_outputs = positive(
            self.concurrent_outputs
                .or(supplied_concurrency.map(|limits| limits.1))
                .unwrap_or(threads.get().min(4)),
            "concurrent_outputs",
        )?;
        if let Some(bytes) = self.model_result_bytes {
            cache.native.model_result_bytes = bytes;
        }
        let budget = ResourceBudget {
            memory_limit_bytes: positive(self.memory_limit_bytes, "memory_limit_bytes")?,
            spill_dir,
            max_temp_dir_bytes: self.max_spill_bytes,
            top_consumers: positive(self.top_consumers.unwrap_or(1), "top_consumers")?,
            threads: ThreadBudget {
                pool_threads: threads,
                target_partitions: positive(
                    self.target_partitions.unwrap_or(threads.get()),
                    "target_partitions",
                )?,
            },
            execution: ExecutionSettings {
                batch_size: self.batch_size,
                spill_compression: self.spill_compression.unwrap_or(defaults.spill_compression),
                max_spill_file_size_bytes: self
                    .max_spill_file_size_bytes
                    .unwrap_or(defaults.max_spill_file_size_bytes),
                sort_spill_reservation_bytes: self
                    .sort_spill_reservation_bytes
                    .unwrap_or(defaults.sort_spill_reservation_bytes),
                time_zone: self.time_zone.unwrap_or(defaults.time_zone),
            },
            math: Default::default(),
            hashing_may_use_pool: self.hashing_may_use_pool,
            cache,
        };
        budget.validate()?;
        Ok(budget)
    }
}

#[cfg(test)]
mod delta_boundary_unit {
    use super::*;
    fn input() -> EngineSettingsInput {
        EngineSettingsInput {
            memory_limit_bytes: 256 << 20,
            threads: 2,
            spill_dir: std::env::current_dir().unwrap(),
            max_spill_bytes: 1 << 30,
            batch_size: 256,
            target_partitions: None,
            top_consumers: None,
            hashing_may_use_pool: false,
            spill_compression: None,
            max_spill_file_size_bytes: None,
            sort_spill_reservation_bytes: None,
            time_zone: None,
            concurrent_queries: None,
            concurrent_outputs: None,
            model_result_bytes: None,
        }
    }
    #[test]
    fn explicit_engine_limits_override_supplied_policy_and_defaults_do_not() {
        let mut cache = DeltaCacheBudget::for_memory(256 << 20);
        cache.native.concurrent_queries = NonZeroUsize::new(5).unwrap();
        cache.native.concurrent_outputs = NonZeroUsize::new(7).unwrap();
        let selected = input().resolve(Some(cache.clone())).unwrap();
        assert_eq!(selected.cache.native.concurrent_queries.get(), 5);
        assert_eq!(selected.cache.native.concurrent_outputs.get(), 7);
        let mut settings = input();
        settings.concurrent_outputs = Some(3);
        let selected = settings.resolve(Some(cache)).unwrap();
        assert_eq!(selected.cache.native.concurrent_queries.get(), 5);
        assert_eq!(selected.cache.native.concurrent_outputs.get(), 3);
    }
    #[test]
    fn construction_defaults_and_invalid_ranges_share_native_validation() {
        let budget = input().resolve(None).unwrap();
        assert_eq!(budget.threads.target_partitions.get(), 2);
        assert_eq!(budget.execution.batch_size, 256);
        assert_eq!(budget.cache.native.concurrent_queries.get(), 2);
        assert_eq!(budget.cache.native.concurrent_outputs.get(), 2);
        assert_eq!(budget.cache.native.model_result_bytes, (256 << 20) / 32);
        let mut tiny = input();
        tiny.threads = 1;
        let tiny = tiny.resolve(None).unwrap();
        assert_eq!(tiny.cache.native.concurrent_outputs.get(), 1);
        for selector in 0..7 {
            let mut value = input();
            match selector {
                0 => value.memory_limit_bytes = 0,
                1 => value.threads = 0,
                2 => value.batch_size = 0,
                3 => value.top_consumers = Some(0),
                4 => value.target_partitions = Some(0),
                5 => value.concurrent_queries = Some(0),
                _ => value.concurrent_outputs = Some(0),
            }
            assert!(value.resolve(None).is_err());
        }
    }
}
