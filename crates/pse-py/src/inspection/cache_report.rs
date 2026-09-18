// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Read-only projection of native cache accounting; unavailable metrics stay None.
use pyo3::prelude::*;
/// A constant-space observation of one native cache, independent of model identity.
#[pyclass(frozen, skip_from_py_object, get_all, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct CacheReport {
    name: String,
    policy_limit_bytes: usize,
    capacity_bytes: usize,
    retained_bytes: usize,
    live_bytes: Option<usize>,
    pinned_bytes: Option<usize>,
    inflight_bytes: Option<usize>,
    active_loads: Option<usize>,
    entries: usize,
    hits: usize,
    misses: usize,
    bypasses: usize,
    evictions: Option<usize>,
}
impl From<pse_catalog::cache_service::CacheReport> for CacheReport {
    fn from(value: pse_catalog::cache_service::CacheReport) -> Self {
        Self {
            name: value.name,
            policy_limit_bytes: value.policy_limit_bytes,
            capacity_bytes: value.capacity_bytes,
            retained_bytes: value.retained_bytes,
            live_bytes: value.live_bytes,
            pinned_bytes: value.pinned_bytes,
            inflight_bytes: value.inflight_bytes,
            active_loads: value.active_loads,
            entries: value.entries,
            hits: value.hits,
            misses: value.misses,
            bypasses: value.bypasses,
            evictions: value.evictions,
        }
    }
}
