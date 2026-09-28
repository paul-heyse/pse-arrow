// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Catalog-owned retention (ADR-0114 Outcome 7; finding T16; Plan 22 X10).
//!
//! The catalog decides what stays reachable and serializes maintainers of a workspace;
//! `pse-catalog` removes and collects member tables exactly as instructed. Every step is
//! idempotent, so an interrupted run is completed by rerunning it:
//! - **retire**: mark a publication expiring (never the head; the workspace epoch
//!   advances), wait until its reader leases are released or expired, remove the tables
//!   no other undeleted publication selects, mark it deleted;
//! - **collect**: advance the epoch and fix each table's protected ranges, then verify,
//!   fence, checkpoint and vacuum each table;
//! - **reclaim**: fence (abandon) every unpublished intent that can never commit, remove
//!   its member prefix, record it reclaimed.
use super::{Runtime, WorkflowError, contract};
use pse_catalog::delta::collect::{CollectAction, CollectTarget};
use pse_columnar::CancellationToken;
use pse_operations::catalog::{ProtectedRange, PublicationId, WorkspaceId};
use pse_relations::generated::runtime::{maintenance_outcomes, retained_versions};
use std::time::Duration;

/// What retiring publications did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RetireReport {
    /// The publications now deleted.
    pub retired: Vec<PublicationId>,
    /// The member tables removed.
    pub removed_tables: Vec<String>,
    /// The objects removed.
    pub removed_objects: u64,
}

/// What collecting a workspace did.
#[derive(Clone, Debug, PartialEq)]
pub struct CollectReport {
    /// The epoch the workspace advanced to before any effect.
    pub maintenance_epoch: i64,
    /// One outcome per maintained table.
    pub tables: Vec<maintenance_outcomes::Row>,
}

/// What reclaiming unpublished intents did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReclaimReport {
    /// The intents reclaimed.
    pub reclaimed: Vec<PublicationId>,
    /// The objects removed under their prefixes.
    pub removed_objects: u64,
}

fn url(text: &str) -> Result<url::Url, WorkflowError> {
    url::Url::parse(text).map_err(|error| contract(format!("{text}: {error}")))
}

fn engine(error: datafusion::common::DataFusionError) -> WorkflowError {
    WorkflowError::Engine(pse_engine::session::engine(error))
}

impl Runtime {
    /// Retire publications of a workspace: each is marked expiring, its readers are
    /// awaited for up to `wait`, the tables only it selects are removed, and it is marked
    /// deleted. A retirement interrupted at any step is completed by rerunning it.
    /// # Errors
    /// An ephemeral runtime; the head (protected); readers still active after `wait` (the
    /// publication stays expiring); removal or store failures.
    pub async fn retire_publications(
        &self,
        workspace: WorkspaceId,
        publications: &[PublicationId],
        wait: Duration,
        cancel: &CancellationToken,
    ) -> Result<RetireReport, WorkflowError> {
        let catalog = self.operations()?.store().catalog();
        let state = self.sessions.native_state();
        let mut report = RetireReport::default();
        for publication in publications {
            cancel
                .checkpoint()
                .map_err(pse_engine::EngineError::from)?;
            catalog.mark_expiring(workspace, *publication).await?;
            catalog
                .wait_for_readers(
                    *publication,
                    Duration::from_millis(50).min(wait),
                    wait,
                )
                .await?;
            let tables = catalog.deletion_plan(workspace, *publication).await?;
            let locations = tables
                .iter()
                .map(|table| url(table))
                .collect::<Result<Vec<_>, _>>()?;
            report.removed_objects += pse_catalog::delta::collect::remove_tables(&locations, state)
                .await
                .map_err(engine)?;
            catalog.mark_deleted(workspace, *publication).await?;
            report.removed_tables.extend(tables);
            report.retired.push(*publication);
        }
        Ok(report)
    }

    /// Collect a workspace: after its epoch advances, every table an undeleted
    /// publication selects is checkpointed and vacuumed keeping every protected version.
    /// # Errors
    /// An ephemeral runtime; a protected version whose history is gone (nothing is
    /// removed from that table); maintenance or store failures.
    pub async fn collect(
        &self,
        workspace: WorkspaceId,
        cancel: &CancellationToken,
    ) -> Result<CollectReport, WorkflowError> {
        let plan = self.operations()?.store().catalog().begin_collect(workspace).await?;
        let now = chrono::Utc::now().timestamp_millis();
        let mut tables = Vec::with_capacity(plan.tables.len());
        for (table, ranges) in &plan.tables {
            cancel
                .checkpoint()
                .map_err(pse_engine::EngineError::from)?;
            let session =
                self.sessions
                    .candidate(std::collections::BTreeMap::new(), self.registry.clone(), cancel)?;
            let retained = ranges.iter().map(retained).collect();
            let target = CollectTarget {
                reference: datafusion::common::ResolvedTableReference {
                    catalog: "maintenance".into(),
                    schema: workspace.to_string().into(),
                    table: table.clone().into(),
                },
                location: url(table)?,
                action: CollectAction::Collect,
                log_cutoff_ms: now,
            };
            let completed =
                pse_catalog::delta::collect::prepare_collect(&session, target, retained, cancel)?
                    .execute(cancel)
                    .await?;
            for batch in completed.batches() {
                let view =
                    maintenance_outcomes::View::try_from_batch_with_registry(&self.registry, batch)
                        .map_err(super::relation)?;
                for index in 0..batch.num_rows() {
                    tables.push(view.row(index).map_err(super::relation)?);
                }
            }
        }
        Ok(CollectReport {
            maintenance_epoch: plan.maintenance_epoch,
            tables,
        })
    }

    /// Reclaim a workspace's unpublished intents that can never commit: each is fenced
    /// (abandoned) before its member prefix is removed, then recorded reclaimed.
    /// # Errors
    /// An ephemeral runtime; removal or store failures (a rerun resumes).
    pub async fn reclaim_unpublished(
        &self,
        workspace: WorkspaceId,
        cancel: &CancellationToken,
    ) -> Result<ReclaimReport, WorkflowError> {
        let catalog = self.operations()?.store().catalog();
        let state = self.sessions.native_state();
        let mut report = ReclaimReport::default();
        for intent in catalog.claim_reclaimable(workspace).await? {
            cancel
                .checkpoint()
                .map_err(pse_engine::EngineError::from)?;
            report.removed_objects +=
                pse_catalog::delta::collect::remove_prefix(&url(&intent.member_prefix)?, state)
                    .await
                    .map_err(engine)?;
            catalog.mark_reclaimed(intent.publication_id).await?;
            report.reclaimed.push(intent.publication_id);
        }
        Ok(report)
    }
}

fn retained(range: &ProtectedRange) -> retained_versions::Row {
    retained_versions::Row {
        table_uri: range.table_uri.clone(),
        from_version: range.from_version,
        through_version: range.through_version,
        reason: range.reason,
    }
}
