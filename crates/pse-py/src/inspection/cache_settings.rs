// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Python projection of the sole native cache budget declaration.
use super::errors;
use pse_runtime::CacheBudget;
use pyo3::prelude::*;
use std::{num::NonZeroUsize, time::Duration};

/// Native cache capacities in the shared process pool. Zero disables that cache.
#[pyclass(frozen, skip_from_py_object, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct CacheSettings {
    pub(super) budget: CacheBudget,
}
#[pymethods]
impl CacheSettings {
    #[new]
    #[expect(
        clippy::too_many_arguments,
        reason = "keyword-only projection of the Rust cache budget"
    )]
    #[pyo3(signature = (*, working_bytes, metadata_bytes=0, statistics_bytes=0, listing_bytes=0, listing_ttl_ms=Some(0), snapshot_bytes=0, resident_bytes=0, concurrent_loads=1, inflight_bytes=0, inspection_bytes=0, crc_replay_max_commits=0, checksum_interval=0, predicate_cache_bytes=0))]
    fn new(
        py: Python<'_>,
        working_bytes: usize,
        metadata_bytes: usize,
        statistics_bytes: usize,
        listing_bytes: usize,
        listing_ttl_ms: Option<u64>,
        snapshot_bytes: usize,
        resident_bytes: usize,
        concurrent_loads: usize,
        inflight_bytes: usize,
        inspection_bytes: usize,
        crc_replay_max_commits: u64,
        checksum_interval: u64,
        predicate_cache_bytes: usize,
    ) -> PyResult<Self> {
        let concurrent_loads = NonZeroUsize::new(concurrent_loads).ok_or_else(|| {
            errors::diagnostic(py, &errors::invalid("concurrent_loads must be positive"))
        })?;
        let budget = CacheBudget {
            working_bytes,
            metadata_bytes,
            statistics_bytes,
            listing_bytes,
            listing_ttl: listing_ttl_ms.map(Duration::from_millis),
            snapshot_bytes,
            resident_bytes,
            concurrent_loads,
            inflight_bytes,
            inspection_bytes,
            crc_replay_max_commits,
            checksum_interval,
            predicate_cache_bytes,
        };
        budget
            .validate(usize::MAX)
            .map_err(|error| errors::diagnostic(py, &error))?;
        Ok(Self { budget })
    }
    #[getter]
    fn working_bytes(&self) -> usize {
        self.budget.working_bytes
    }
    #[getter]
    fn metadata_bytes(&self) -> usize {
        self.budget.metadata_bytes
    }
    #[getter]
    fn statistics_bytes(&self) -> usize {
        self.budget.statistics_bytes
    }
    #[getter]
    fn listing_bytes(&self) -> usize {
        self.budget.listing_bytes
    }
    #[getter]
    fn listing_ttl_ms(&self) -> Option<u128> {
        self.budget.listing_ttl.map(|ttl| ttl.as_millis())
    }
    #[getter]
    fn snapshot_bytes(&self) -> usize {
        self.budget.snapshot_bytes
    }
    #[getter]
    fn resident_bytes(&self) -> usize {
        self.budget.resident_bytes
    }
    #[getter]
    fn concurrent_loads(&self) -> usize {
        self.budget.concurrent_loads.get()
    }
    #[getter]
    fn inflight_bytes(&self) -> usize {
        self.budget.inflight_bytes
    }
    #[getter]
    fn inspection_bytes(&self) -> usize {
        self.budget.inspection_bytes
    }
    #[getter]
    fn predicate_cache_bytes(&self) -> usize {
        self.budget.predicate_cache_bytes
    }
    #[getter]
    fn crc_replay_max_commits(&self) -> u64 {
        self.budget.crc_replay_max_commits
    }
    #[getter]
    fn checksum_interval(&self) -> u64 {
        self.budget.checksum_interval
    }
}
