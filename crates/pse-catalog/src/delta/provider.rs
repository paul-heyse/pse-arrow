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
    version: i64,
    layout: &DurableLayout,
    state: Arc<SessionState>,
) -> Result<ViewTable> {
    let version = delta_version(version)?;
    let table = table_builder(location, &state)?
        .with_version(version)
        .load()
        .await
        .map_err(external)?;
    view(table, layout, state).await
}

/// Open an exact declared Delta table, checking its persisted properties and CHECK.
/// # Errors
/// Missing versions, a mismatched declaration, or native provider failure.
pub async fn open_declared_view(
    location: url::Url,
    version: i64,
    contract: &super::contract::DeclaredCheck,
    state: Arc<SessionState>,
) -> Result<ViewTable> {
    let table = table_builder(location, &state)?
        .with_version(delta_version(version)?)
        .load()
        .await
        .map_err(external)?;
    contract.verify(&table)?;
    view(table, contract.layout(), state).await
}

async fn view(
    table: deltalake::DeltaTable,
    layout: &DurableLayout,
    state: Arc<SessionState>,
) -> Result<ViewTable> {
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

/// The unsigned native Delta protocol is an external boundary, not PSE storage.
pub(super) fn delta_version(version: i64) -> Result<u64> {
    u64::try_from(version).map_err(|_| {
        DataFusionError::Plan(format!("Delta version must be nonnegative, got {version}"))
    })
}

/// Fail rather than truncating a native version outside PSE's declared domain.
pub(super) fn signed_version(version: u64) -> Result<i64> {
    i64::try_from(version).map_err(|_| {
        DataFusionError::Execution(format!(
            "Delta version {version} exceeds PSE's signed version domain"
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::{delta_version, signed_version};

    #[test]
    fn version_boundary_is_checked_in_both_directions() {
        for version in [0, 1, i64::MAX] {
            assert_eq!(
                delta_version(version).and_then(signed_version).ok(),
                Some(version)
            );
        }
        assert!(delta_version(-1).is_err());
        assert!(delta_version(i64::MIN).is_err());
        assert!(signed_version(1_u64 << 63).is_err());
        assert!(signed_version(u64::MAX).is_err());
    }
}
