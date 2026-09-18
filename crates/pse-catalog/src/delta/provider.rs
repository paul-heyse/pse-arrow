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
    let state = crate::cache_service::bind_state(&location, &state)?;
    let lease = super::lease::read(&location, &pse_ids::CancellationToken::new()).await?;
    let version = delta_version(version)?;
    let opened = open_native(
        location,
        Some(version),
        crate::cache_service::snapshot::LoadRequirement::Query,
        &state,
    )
    .await?;
    view(opened, layout, state, lease).await
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
    let lease = super::lease::read(&location, &pse_ids::CancellationToken::new()).await?;
    declared_view(location, version, contract, state, lease).await
}

pub(super) async fn open_maintained_view(
    location: url::Url,
    version: i64,
    contract: &super::contract::DeclaredCheck,
    state: Arc<SessionState>,
    lease: &super::lease::MaintenanceLease,
) -> Result<ViewTable> {
    lease.covers(&location)?;
    declared_view(location, version, contract, state, None).await
}

async fn declared_view(
    location: url::Url,
    version: i64,
    contract: &super::contract::DeclaredCheck,
    state: Arc<SessionState>,
    lease: Option<Arc<super::lease::ReadLease>>,
) -> Result<ViewTable> {
    let state = crate::cache_service::bind_state(&location, &state)?;
    let opened = open_native(
        location,
        Some(delta_version(version)?),
        crate::cache_service::snapshot::LoadRequirement::Query,
        &state,
    )
    .await?;
    contract.verify(&opened.table)?;
    view(opened, contract.layout(), state, lease).await
}

pub(crate) struct Opened {
    pub(crate) table: deltalake::DeltaTable,
    pub(crate) owner: Option<Arc<crate::cache_service::snapshot::RetainedTable>>,
}
pub(crate) async fn open_native(
    location: url::Url,
    version: Option<u64>,
    requirement: crate::cache_service::snapshot::LoadRequirement,
    state: &Arc<SessionState>,
) -> Result<Opened> {
    if let Some(service) = state
        .config()
        .get_extension::<crate::cache_service::NativeCacheService>()
    {
        let owner = service
            .open_snapshot(location, version, requirement, Arc::clone(state))
            .await?;
        Ok(Opened {
            table: owner.table.clone(),
            owner: Some(owner),
        })
    } else {
        let mut builder = table_builder(location, state)?;
        if let Some(version) = version {
            builder = builder.with_version(version);
        }
        if requirement == crate::cache_service::snapshot::LoadRequirement::Metadata {
            builder = builder.without_files();
        }
        let table = builder.load().await.map_err(external)?;
        Ok(Opened { table, owner: None })
    }
}

async fn view(
    opened: Opened,
    layout: &DurableLayout,
    state: Arc<SessionState>,
    lease: Option<Arc<super::lease::ReadLease>>,
) -> Result<ViewTable> {
    let provider = opened
        .table
        .table_provider()
        .with_session(state.clone())
        .build()
        .await?;
    let provider: Arc<dyn datafusion::catalog::TableProvider> = Arc::new(provider);
    let provider = super::leased::retain_reader_budget(provider, &state);
    let provider = match opened.owner {
        Some(owner) => super::leased::retain_snapshot(provider, owner),
        None => provider,
    };
    let provider = super::leased::retain(provider, lease);
    let input =
        LogicalPlanBuilder::scan("delta_version", provider_as_source(provider), None)?.build()?;
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
    let policy = state
        .config()
        .get_extension::<crate::cache_service::NativeCacheService>();
    Ok(DeltaTableBuilder::from_url(location.clone())
        .map_err(external)?
        .with_storage_backend(store, location)
        .with_crc_replay_max_commits(
            policy
                .as_ref()
                .map_or(0, |service| service.policy().crc_replay_max_commits),
        ))
}
/// Best-effort native acceleration after a known durable commit. No failure here
/// changes the mutation outcome or causes its input plan to run a second time.
pub(crate) async fn committed(table: &deltalake::DeltaTable, state: &SessionState) {
    let Some(service) = state
        .config()
        .get_extension::<crate::cache_service::NativeCacheService>()
    else {
        return;
    };
    if let Err(error) = service.remember_committed(table, state).await {
        crate::cache_service::metrics::record(state, |metrics| {
            &metrics.committed_snapshot_refusals
        });
        tracing::warn!(%error, "committed Delta snapshot was not retained");
    }
    let interval = service.policy().checksum_interval;
    if interval != 0
        && table
            .version()
            .is_some_and(|version| version % interval == 0)
    {
        let result = async {
            table
                .snapshot()?
                .snapshot()
                .snapshot_ref()
                .clone()
                .write_checksum(table.log_store().engine(None))
                .await
        }
        .await;
        match result {
            Ok(_) => {
                crate::cache_service::metrics::record(state, |metrics| &metrics.checksum_successes);
            }
            Err(error) => {
                crate::cache_service::metrics::record(state, |metrics| &metrics.checksum_failures);
                tracing::warn!(%error, "committed Delta checksum acceleration failed");
            }
        }
    }
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
