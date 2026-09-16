// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Immutable version binding with the native Delta scan and DataFusion view contracts.
use super::layout::DurableLayout;
use datafusion::{
    common::{DataFusionError, Result},
    datasource::{ViewTable, provider_as_source},
    execution::session_state::SessionState,
    logical_expr::LogicalPlanBuilder,
};
use deltalake::DeltaTableBuilder;
use std::sync::Arc;

/// Open an exact Delta version as a native semantic view. Native views reject writes;
/// mutations use the explicit validating command route. The scan owns its snapshot,
/// log store and caller runtime through the view's real logical input.
/// # Errors
/// Missing/pruned versions, incompatible declared layout or native provider failure.
pub async fn open_view(
    location: url::Url,
    version: u64,
    layout: &DurableLayout,
    state: Arc<SessionState>,
) -> Result<ViewTable> {
    let table = table_builder(location, &state)?
        .with_version(version)
        .load()
        .await
        .map_err(external)?;
    let provider = table.table_provider().with_session(state).build().await?;
    let input = LogicalPlanBuilder::scan(
        "delta_version",
        provider_as_source(Arc::new(provider)),
        None,
    )?
    .build()?;
    Ok(ViewTable::new(layout.decode(input)?, None))
}
/// Resolve Delta storage from the invocation's native object-store registry.
/// # Errors
/// The caller has not registered the location or Delta cannot bind its log store.
pub fn table_builder(location: url::Url, state: &SessionState) -> Result<DeltaTableBuilder> {
    let store = state
        .runtime_env()
        .object_store_registry
        .get_store(&location)?;
    Ok(DeltaTableBuilder::from_url(location.clone())
        .map_err(external)?
        .with_storage_backend(store, location))
}
fn external(error: deltalake::DeltaTableError) -> DataFusionError {
    DataFusionError::External(Box::new(error))
}
