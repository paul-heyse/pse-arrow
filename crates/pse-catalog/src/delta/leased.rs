// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Catalog supplies snapshot and budget owners; the engine retains them through native IO.
use datafusion::{
    catalog::TableProvider, execution::session_state::SessionState, physical_plan::ExecutionPlan,
};
use pse_engine::operation::ownership;
use std::sync::Arc;
pub(crate) fn supports_round_reset(plan: &dyn ExecutionPlan) -> bool {
    ownership::supports_reset(plan)
}
pub(super) fn retain_reader_budget(
    inner: Arc<dyn TableProvider>,
    state: &SessionState,
) -> Arc<dyn TableProvider> {
    let bytes = state
        .config_options()
        .execution
        .parquet
        .max_predicate_cache_size
        .unwrap_or(0);
    if bytes == 0 {
        inner
    } else {
        ownership::provider(inner, Arc::new(()), bytes)
    }
}
pub(super) fn retain_snapshot(
    inner: Arc<dyn TableProvider>,
    owner: Arc<crate::cache_service::snapshot::RetainedTable>,
) -> Arc<dyn TableProvider> {
    ownership::provider(inner, owner, 0)
}
