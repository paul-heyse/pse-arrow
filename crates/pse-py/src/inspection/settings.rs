// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Immutable projection of the sole Rust runtime configuration.
use super::errors;
use pse_catalog::{CatalogError, ExecutionSettings, ThreadBudget};
use pse_runtime::ResourceBudget;
use pyo3::prelude::*;
use std::{num::NonZeroUsize, path::PathBuf};

/// Explicit shared process budget. Optional settings inherit the Rust defaults.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct EngineSettings {
    pub(super) budget: ResourceBudget,
}

#[pymethods]
impl EngineSettings {
    #[new]
    #[expect(
        clippy::too_many_arguments,
        clippy::needless_pass_by_value,
        reason = "keyword-only Python projection of the existing Rust resource and execution settings"
    )]
    #[pyo3(signature = (*, memory_limit_bytes, threads, spill_dir: "str", max_spill_bytes, batch_size, target_partitions=None, top_consumers=None, hashing_may_use_pool=false, spill_compression=None, max_spill_file_size_bytes=None, sort_spill_reservation_bytes=None, time_zone=None, cache=None))]
    fn new(
        py: Python<'_>,
        memory_limit_bytes: usize,
        threads: usize,
        spill_dir: PathBuf,
        max_spill_bytes: u64,
        batch_size: usize,
        target_partitions: Option<usize>,
        top_consumers: Option<usize>,
        hashing_may_use_pool: bool,
        spill_compression: Option<String>,
        max_spill_file_size_bytes: Option<u64>,
        sort_spill_reservation_bytes: Option<usize>,
        time_zone: Option<String>,
        cache: Option<PyRef<'_, super::CacheSettings>>,
    ) -> PyResult<Self> {
        let cache = cache.map(|cache| cache.budget.clone());
        let result = py.detach(|| {
            let defaults = ExecutionSettings::default();
            let threads = positive(threads, "threads must be positive")?;
            let budget = ResourceBudget {
                memory_limit_bytes: positive(memory_limit_bytes, "memory limit must be positive")?,
                spill_dir: spill_dir.canonicalize().map_err(|source| {
                    CatalogError::Infrastructure {
                        op: "resolve explicit spill directory".to_owned(),
                        source: Box::new(source),
                    }
                })?,
                max_temp_dir_bytes: max_spill_bytes,
                top_consumers: positive(
                    top_consumers.unwrap_or(1),
                    "top_consumers must be positive",
                )?,
                threads: ThreadBudget {
                    pool_threads: threads,
                    target_partitions: positive(
                        target_partitions.unwrap_or(threads.get()),
                        "target_partitions must be positive",
                    )?,
                },
                execution: ExecutionSettings {
                    batch_size,
                    spill_compression: spill_compression.unwrap_or(defaults.spill_compression),
                    max_spill_file_size_bytes: max_spill_file_size_bytes
                        .unwrap_or(defaults.max_spill_file_size_bytes),
                    sort_spill_reservation_bytes: sort_spill_reservation_bytes
                        .unwrap_or(defaults.sort_spill_reservation_bytes),
                    time_zone: time_zone.unwrap_or(defaults.time_zone),
                },
                hashing_may_use_pool,
                cache: cache
                    .unwrap_or_else(|| pse_runtime::CacheBudget::for_memory(memory_limit_bytes)),
            };
            budget
                .validate()
                .map_err(|error| CatalogError::Semantic(std::sync::Arc::new(error)))?;
            Ok(Self { budget })
        });
        result.map_err(|error: CatalogError| errors::diagnostic(py, &error))
    }
    #[getter]
    fn memory_limit_bytes(&self) -> usize {
        self.budget.memory_limit_bytes.get()
    }
    #[getter]
    fn cache(&self) -> super::CacheSettings {
        super::CacheSettings {
            budget: self.budget.cache.clone(),
        }
    }
    #[getter]
    fn threads(&self) -> usize {
        self.budget.threads.pool_threads.get()
    }
    #[getter]
    fn target_partitions(&self) -> usize {
        self.budget.threads.target_partitions.get()
    }
    #[getter]
    fn spill_dir(&self) -> String {
        self.budget.spill_dir.to_string_lossy().into_owned()
    }
    #[getter]
    fn max_spill_bytes(&self) -> u64 {
        self.budget.max_temp_dir_bytes
    }
    #[getter]
    fn top_consumers(&self) -> usize {
        self.budget.top_consumers.get()
    }
    #[getter]
    fn hashing_may_use_pool(&self) -> bool {
        self.budget.hashing_may_use_pool
    }
    #[getter]
    fn batch_size(&self) -> usize {
        self.budget.execution.batch_size
    }
    #[getter]
    fn spill_compression(&self) -> &str {
        &self.budget.execution.spill_compression
    }
    #[getter]
    fn max_spill_file_size_bytes(&self) -> u64 {
        self.budget.execution.max_spill_file_size_bytes
    }
    #[getter]
    fn sort_spill_reservation_bytes(&self) -> usize {
        self.budget.execution.sort_spill_reservation_bytes
    }
    #[getter]
    fn time_zone(&self) -> &str {
        &self.budget.execution.time_zone
    }
}

fn positive(value: usize, reason: &str) -> Result<NonZeroUsize, CatalogError> {
    NonZeroUsize::new(value).ok_or_else(|| errors::invalid(reason))
}
