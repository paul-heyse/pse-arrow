// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual event counters, outside all semantic dependency inventories.
use std::sync::atomic::{AtomicUsize, Ordering};
#[derive(Debug, Default)]
pub(crate) struct Metrics {
    pub(crate) sql_bindings: AtomicUsize,
    pub(crate) analyses: AtomicUsize,
    pub(crate) optimizations: AtomicUsize,
    pub(crate) physical_plans: AtomicUsize,
    pub(crate) reusable_executions: AtomicUsize,
    pub(crate) checksum_successes: AtomicUsize,
    pub(crate) checksum_failures: AtomicUsize,
    pub(crate) committed_snapshot_refusals: AtomicUsize,
    pub(crate) cdf_comparisons: AtomicUsize,
    pub(crate) endpoint_comparisons: AtomicUsize,
}
impl Metrics {
    pub(crate) fn report(&self) -> Vec<(&'static str, usize)> {
        [
            ("sql_bindings", &self.sql_bindings),
            ("analyses", &self.analyses),
            ("optimizations", &self.optimizations),
            ("physical_plans", &self.physical_plans),
            ("reusable_executions", &self.reusable_executions),
            ("checksum_successes", &self.checksum_successes),
            ("checksum_failures", &self.checksum_failures),
            ("cdf_comparisons", &self.cdf_comparisons),
            ("endpoint_comparisons", &self.endpoint_comparisons),
            (
                "committed_snapshot_refusals",
                &self.committed_snapshot_refusals,
            ),
        ]
        .map(|(name, value)| (name, value.load(Ordering::Relaxed)))
        .into()
    }
}
pub(crate) fn record(
    state: &datafusion::execution::session_state::SessionState,
    select: impl FnOnce(&Metrics) -> &AtomicUsize,
) {
    if let Some(service) = state.config().get_extension::<super::NativeCacheService>() {
        select(&service.metrics).fetch_add(1, Ordering::Relaxed);
    }
}
