// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native Delta maintenance steps shared by catalog-driven collection
//! ([`super::collect`]): a fencing commit, the effective retention of protected versions,
//! and the ordered checkpoint, vacuum and log cleanup that keeps them.
use datafusion::{
    common::{DataFusionError, Result},
    execution::session_state::SessionState,
};
use deltalake::{
    kernel::transaction::CommitProperties, operations::vacuum::VacuumMode, protocol::checkpoints,
};
use pse_columnar::CancellationToken;
use pse_relations::generated::runtime::maintenance_outcomes;
use std::{collections::BTreeSet, sync::Arc};

pub(super) async fn maintenance_fence(
    table: deltalake::DeltaTable,
    state: &SessionState,
) -> Result<deltalake::DeltaTable> {
    let commit = super::operation::commit_policy(
        CommitProperties::default(),
        super::operation::CommitKind::Maintenance,
    )
    .with_metadata([(
        "pse.maintenance_fence".into(),
        serde_json::json!(uuid::Uuid::now_v7().to_string()),
    )]);
    // Native metadata commit with unchanged properties: no application state store
    // and no reconstruction of Delta's transaction or replay algorithms.
    let fenced = table
        .set_tbl_properties()
        .with_properties(std::collections::HashMap::new())
        .with_commit_properties(commit)
        .await
        .map_err(super::settlement::unresolved);
    // Even a lost response may have committed. Invalidate on both outcomes.
    if let Some(service) = state
        .config()
        .get_extension::<crate::cache_service::DeltaCacheService>()
    {
        service.native().invalidate();
    }
    fenced
}

pub(super) struct EffectiveRetention {
    ranges: Vec<(u64, u64)>,
    pub(super) versions: BTreeSet<u64>,
    pub(super) changes: bool,
    pub(super) attempts: bool,
    reservation: pse_columnar::MemoryReservation,
}

impl EffectiveRetention {
    pub(super) fn new(pool: &Arc<dyn pse_columnar::MemoryPool>) -> Self {
        Self {
            ranges: Vec::new(),
            versions: BTreeSet::new(),
            changes: false,
            attempts: false,
            reservation: pse_columnar::MemoryConsumer::new("delta:retained-versions")
                .register(pool),
        }
    }
    pub(super) fn protect(&mut self, start: u64, end: u64) -> Result<()> {
        self.reservation.try_grow(32).map_err(external)?;
        self.ranges.push((start, end));
        Ok(())
    }
    // Keep compact intervals through admission, then expand once for Delta keep_versions.
    pub(super) fn expand(&mut self, cancel: &CancellationToken) -> Result<()> {
        self.ranges.sort_unstable();
        let mut through = None;
        for (start, end) in &self.ranges {
            let start = through.map_or(*start, |previous: u64| {
                (*start).max(previous.saturating_add(1))
            });
            if through == Some(u64::MAX) {
                break;
            }
            for version in start..=*end {
                cancel.checkpoint().map_err(external)?;
                if !self.versions.contains(&version) {
                    self.reservation.try_grow(64).map_err(external)?;
                    self.versions.insert(version);
                }
            }
            through = Some(through.map_or(*end, |previous| previous.max(*end)));
        }
        Ok(())
    }
}

#[derive(Debug)]
struct MaintenancePolicy {
    data_age: chrono::TimeDelta,
    log_cutoff_ms: i64,
    mode: VacuumMode,
    retain_attempt_logs: bool,
}
impl MaintenancePolicy {
    fn resolve(
        data_age: std::time::Duration,
        log_age: std::time::Duration,
        requested_cutoff: i64,
        now: i64,
        changes: bool,
        attempts: bool,
    ) -> Result<Self> {
        if requested_cutoff < 0 || requested_cutoff > now {
            return Err(invalid("invalid log cleanup cutoff"));
        }
        let rounded_age = log_age
            .as_millis()
            .checked_add(u128::from(
                !log_age.subsec_nanos().is_multiple_of(1_000_000),
            ))
            .ok_or_else(|| invalid("log retention age overflows"))?;
        let log_age = i64::try_from(rounded_age)
            .map_err(|error| DataFusionError::External(Box::new(error)))?;
        let floor = now.checked_sub(log_age).unwrap_or(0).max(0);
        Ok(Self {
            data_age: chrono::TimeDelta::from_std(data_age)
                .map_err(|error| DataFusionError::External(Box::new(error)))?,
            log_cutoff_ms: requested_cutoff.min(floor),
            mode: if changes {
                VacuumMode::Lite
            } else {
                VacuumMode::Full
            },
            retain_attempt_logs: attempts,
        })
    }
}
pub(super) fn now_ms() -> Result<i64> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| DataFusionError::External(Box::new(error)))?;
    i64::try_from(now.as_millis()).map_err(|error| DataFusionError::External(Box::new(error)))
}

/// Optimize (when asked), checkpoint, vacuum keeping every retained version, and remove
/// expired logs below the oldest retained version.
pub(super) async fn apply_maintenance(
    location: &url::Url,
    optimize: bool,
    log_cutoff_ms: i64,
    state: &SessionState,
    mut table: deltalake::DeltaTable,
    contract: &super::contract::DeclaredCheck,
    retained: EffectiveRetention,
) -> Result<datafusion::arrow::array::RecordBatch> {
    use deltalake::table::config::TablePropertiesExt;
    let properties = table.snapshot().map_err(external)?.table_config();
    let policy = MaintenancePolicy::resolve(
        properties.deleted_file_retention_duration(),
        properties.log_retention_duration(),
        log_cutoff_ms,
        now_ms()?,
        retained.changes,
        retained.attempts,
    )?;
    let caller = Arc::new(state.clone());
    let context = super::operation::DeltaOperationContext::new(
        caller.clone(),
        Some(contract.clone()),
        CommitProperties::default(),
        super::operation::CommitKind::Maintenance,
    )?;
    if optimize {
        table = context.optimize(table).await.map_err(external)?.0;
    }
    checkpoints::create_checkpoint(&table, None)
        .await
        .map_err(external)?;
    let keep = retained.versions.iter().copied().collect::<Vec<_>>();
    let (table, metrics) = context
        .vacuum(table)
        .with_keep_versions(&keep)
        .with_retention_period(policy.data_age)
        .with_mode(policy.mode)
        .await
        .map_err(external)?;
    let version = table
        .version()
        .ok_or_else(|| invalid("maintained table has no version"))?;
    tracing::Span::current().record("delta_version", version);
    tracing::Span::current().record("files_deleted", metrics.files_deleted.len());
    let minimum = retained.versions.first().copied().unwrap_or(version);
    // Live member attempts require complete commit evidence; otherwise expired logs
    // below the oldest retained version go.
    let deleted_logs = if policy.retain_attempt_logs {
        0
    } else {
        checkpoints::cleanup_expired_logs_for(
            minimum,
            table.log_store().as_ref(),
            policy.log_cutoff_ms,
            None,
        )
        .await
        .map_err(external)?
    };
    drop(retained);
    let mut output = maintenance_outcomes::Builder::new().map_err(external)?;
    output
        .push(maintenance_outcomes::Row {
            table_uri: location.to_string(),
            delta_version: super::provider::signed_version(version)?,
            deleted_files: metrics.files_deleted,
            deleted_logs: i64::try_from(deleted_logs)
                .map_err(|error| DataFusionError::External(Box::new(error)))?,
        })
        .map_err(external)?;
    Ok(output.finish().map_err(external)?.into_batch())
}

fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(reason.into())
}
fn external(error: impl Into<DataFusionError>) -> DataFusionError {
    error.into()
}

#[cfg(test)]
mod delta_boundary_unit {
    use super::*;
    use std::time::Duration;
    #[test]
    fn native_retention_floor_changes_and_attempts_form_one_effective_policy() {
        for changes in [false, true] {
            for attempts in [false, true] {
                let policy = MaintenancePolicy::resolve(
                    Duration::from_secs(10),
                    Duration::from_millis(30),
                    99,
                    100,
                    changes,
                    attempts,
                )
                .unwrap();
                assert_eq!(policy.log_cutoff_ms, 70);
                assert_eq!(policy.data_age.num_seconds(), 10);
                assert_eq!(
                    policy.mode,
                    if changes {
                        VacuumMode::Lite
                    } else {
                        VacuumMode::Full
                    }
                );
                assert_eq!(policy.retain_attempt_logs, attempts);
            }
        }
        assert_eq!(
            MaintenancePolicy::resolve(
                Duration::ZERO,
                Duration::from_secs(1),
                20,
                100,
                false,
                false
            )
            .unwrap()
            .log_cutoff_ms,
            0
        );
        for cutoff in [-1, 101] {
            assert!(
                MaintenancePolicy::resolve(
                    Duration::ZERO,
                    Duration::ZERO,
                    cutoff,
                    100,
                    false,
                    false
                )
                .is_err()
            );
        }
        assert!(
            MaintenancePolicy::resolve(Duration::MAX, Duration::ZERO, 0, 100, false, false)
                .is_err()
        );
        assert!(
            MaintenancePolicy::resolve(Duration::ZERO, Duration::MAX, 0, 100, false, false)
                .is_err()
        );
    }
    #[test]
    fn overlapping_protection_ranges_expand_once_under_a_finite_pool() {
        use datafusion::execution::memory_pool::{GreedyMemoryPool, MemoryPool};
        let pool: Arc<dyn MemoryPool> = Arc::new(GreedyMemoryPool::new(4096));
        let mut retained = EffectiveRetention {
            ranges: vec![(5, 7), (1, 3), (2, 6)],
            versions: BTreeSet::new(),
            changes: true,
            attempts: true,
            reservation: pse_columnar::MemoryConsumer::new("unit").register(&pool),
        };
        retained.expand(&CancellationToken::new()).unwrap();
        assert_eq!(retained.versions, (1..=7).collect());
        assert_eq!(pool.reserved(), 7 * 64);
        drop(retained);
        assert_eq!(pool.reserved(), 0);
    }
}
