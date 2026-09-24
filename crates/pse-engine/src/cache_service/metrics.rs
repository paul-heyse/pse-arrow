// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual event counters, outside all semantic dependency inventories.
use std::sync::atomic::{AtomicUsize, Ordering};
#[derive(Debug, Default)]
/// Actual event counters, never semantic identities or inferred work counts.
pub struct Metrics {
    /// Successfully constructed immutable model assemblies.
    pub assembly_builds: AtomicUsize,
    /// Reused live immutable model assemblies.
    pub assembly_reuses: AtomicUsize,
    /// Actual native SQL parser invocations, excluding syntax-cache hits.
    pub sql_parses: AtomicUsize,
    /// Native SQL binding invocations.
    pub sql_bindings: AtomicUsize,
    /// Native analyzer invocations.
    pub analyses: AtomicUsize,
    /// Native logical optimizer invocations.
    pub optimizations: AtomicUsize,
    /// Actual physical planning invocations.
    pub physical_plans: AtomicUsize,
    /// Native reusable-round execution invocations.
    pub reusable_executions: AtomicUsize,
}
impl Metrics {
    /// Read bounded aggregate counters without enumerating retained entries.
    pub fn report(&self) -> Vec<(&'static str, usize)> {
        [
            ("assembly_builds", &self.assembly_builds),
            ("assembly_reuses", &self.assembly_reuses),
            ("sql_parses", &self.sql_parses),
            ("sql_bindings", &self.sql_bindings),
            ("analyses", &self.analyses),
            ("optimizations", &self.optimizations),
            ("physical_plans", &self.physical_plans),
            ("reusable_executions", &self.reusable_executions),
        ]
        .map(|(name, value)| (name, value.load(Ordering::Relaxed)))
        .into()
    }
}
/// Increment a counter on the actual deployment service, when it is installed.
pub fn record(
    state: &datafusion::execution::session_state::SessionState,
    select: impl FnOnce(&Metrics) -> &AtomicUsize,
) {
    if let Some(service) = state.config().get_extension::<super::NativeCacheService>() {
        select(&service.metrics).fetch_add(1, Ordering::Relaxed);
    }
}
