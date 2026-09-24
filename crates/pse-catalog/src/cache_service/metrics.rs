// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Durable cache event counters, outside semantic dependencies.
use std::sync::atomic::{AtomicUsize, Ordering};
#[derive(Debug, Default)]
/// Actual event counters, never semantic identities or inferred work counts.
pub struct Metrics {
    /// Actual durable checksum successes.
    pub checksum_successes: AtomicUsize,
    /// Actual durable checksum failures.
    pub checksum_failures: AtomicUsize,
    /// Actual durable committed snapshot refusals.
    pub committed_snapshot_refusals: AtomicUsize,
    /// Actual durable cdf comparisons.
    pub cdf_comparisons: AtomicUsize,
    /// Actual durable endpoint comparisons.
    pub endpoint_comparisons: AtomicUsize,
}
impl Metrics {
    /// Read bounded aggregate counters without enumerating retained entries.
    pub fn report(&self) -> Vec<(&'static str, Option<usize>)> {
        vec![
            (
                "checksum_successes",
                Some(self.checksum_successes.load(Ordering::Relaxed)),
            ),
            (
                "checksum_failures",
                Some(self.checksum_failures.load(Ordering::Relaxed)),
            ),
            (
                "committed_snapshot_refusals",
                Some(self.committed_snapshot_refusals.load(Ordering::Relaxed)),
            ),
            (
                "cdf_comparisons",
                Some(self.cdf_comparisons.load(Ordering::Relaxed)),
            ),
            (
                "endpoint_comparisons",
                Some(self.endpoint_comparisons.load(Ordering::Relaxed)),
            ),
        ]
    }
}
/// Increment a counter on the actual deployment service, when it is installed.
pub fn record(
    state: &datafusion::execution::session_state::SessionState,
    select: impl FnOnce(&Metrics) -> &AtomicUsize,
) {
    if let Some(service) = state.config().get_extension::<super::DeltaCacheService>() {
        select(&service.metrics).fetch_add(1, Ordering::Relaxed);
    }
}
