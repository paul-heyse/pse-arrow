// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Catalog supplies leases/snapshot owners; engine retains them through native IO.
use super::lease::ReadLease;
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
pub(super) fn retain(
    inner: Arc<dyn TableProvider>,
    lease: Option<Arc<ReadLease>>,
) -> Arc<dyn TableProvider> {
    match lease {
        Some(lease) => ownership::provider(inner, lease, 0),
        None => inner,
    }
}
pub(super) fn retain_execution(
    inner: Arc<dyn ExecutionPlan>,
    lease: Option<Arc<ReadLease>>,
) -> Arc<dyn ExecutionPlan> {
    match lease {
        Some(lease) => ownership::execution(inner, lease),
        None => inner,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::{
        arrow::datatypes::Schema, execution::TaskContext, physical_plan::empty::EmptyExec,
    };

    #[tokio::test]
    async fn physical_stream_retains_lease_after_plan_and_provider_drop() {
        let directory = tempfile::tempdir().unwrap();
        let location = url::Url::from_directory_path(directory.path()).unwrap();
        let cancel = pse_columnar::CancellationToken::new();
        let lease = super::super::lease::write(&location, &cancel)
            .await
            .unwrap()
            .unwrap();
        let plan = ownership::execution(Arc::new(EmptyExec::new(Arc::new(Schema::empty()))), lease);
        let stream = plan.execute(0, Arc::new(TaskContext::default())).unwrap();
        drop(plan);
        assert!(super::super::lease::maintenance(&location, &cancel).is_err());
        drop(stream);
        assert!(super::super::lease::maintenance(&location, &cancel).is_ok());
    }
}
