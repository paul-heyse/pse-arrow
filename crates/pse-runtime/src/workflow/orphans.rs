// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Durable reports over bounded process-local enumeration generations.
use super::{Runtime, WorkflowError, contract};
use pse_catalog::delta::discovery::RootListing;
use pse_columnar::CancellationToken;
use pse_model::generated::identities::ScanId;
use pse_operations::{
    catalog::WorkspaceId,
    inventory::{CandidatePage, ObservedCandidate, ScanCheckpoint},
};
use std::collections::BTreeMap;

const MAX_ACTIVE_STREAMS: usize = 16;

/// Application work and retained observation bounds for one discovery slice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DiscoveryBudget {
    /// At most 256 local guard entries or listed objects per call.
    pub maximum_entries: usize,
    /// Accounted observation capacity. Native provider buffers and latency are excluded.
    pub maximum_bytes: usize,
}
impl Default for DiscoveryBudget {
    fn default() -> Self {
        Self {
            maximum_entries: 256,
            maximum_bytes: 1 << 20,
        }
    }
}
#[derive(Debug)]
struct ActiveScan {
    checkpoint: ScanCheckpoint,
    listing: RootListing,
}
#[derive(Debug, Default)]
pub(super) struct DiscoveryStreams {
    active: BTreeMap<ScanId, ActiveScan>,
}

/// Physical result of one explicit selected catalog claim.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrphanReclaimReport {
    /// Recorded inventory selected by the caller.
    pub scan_id: ScanId,
    /// Exactly the claimed prefix.
    pub prefix: String,
    /// Workspace fence held through deletion and disposition commit.
    pub maintenance_epoch: i64,
    /// Actual successful physical deletions.
    pub removed_objects: u64,
}

fn engine(error: datafusion::common::DataFusionError) -> WorkflowError {
    WorkflowError::Engine(pse_engine::session::engine(error))
}

impl Runtime {
    /// Discover one bounded slice under an established workspace root. Reuse `scan_id`
    /// to continue. A new runtime restarts enumeration at the root in a new generation,
    /// retaining the catalog's deduplicated candidates and report cursor.
    /// No observed prefix is deletion authority; only the catalog establishes ownership.
    /// # Errors
    /// Ephemeral runtime, invalid budget, root/symlink refusal, cancellation, accounting
    /// exhaustion or store failures. A failed slice drops its stream; retry starts a new
    /// generation rather than silently losing uncommitted observations.
    pub async fn discover_orphans(
        &self,
        workspace: WorkspaceId,
        scan_id: ScanId,
        budget: DiscoveryBudget,
        cancel: &CancellationToken,
    ) -> Result<ScanCheckpoint, WorkflowError> {
        if !(1..=pse_operations::inventory::INVENTORY_PAGE_MAX).contains(&budget.maximum_entries)
            || budget.maximum_bytes < 256
        {
            return Err(contract(
                "orphan discovery requires 1..=256 entries and at least 256 bytes",
            ));
        }
        let catalog = self.operations()?.store().catalog();
        // Serialize stream ownership, including durable acknowledgement. A replacement
        // caller never advances a stream whose preceding page failed to persist.
        let mut streams = cancel
            .until_cancelled(self.orphan_streams.lock())
            .await
            .map_err(pse_engine::EngineError::from)?;
        let active = if let Some(active) = streams.active.remove(&scan_id) {
            if active.checkpoint.workspace_id != workspace {
                streams.active.insert(scan_id, active);
                return Err(contract("scan belongs to another workspace"));
            }
            active
        } else {
            let recorded = catalog.begin_orphan_scan(scan_id, workspace).await?;
            if recorded.complete {
                return Ok(recorded);
            }
            if streams.active.len() >= MAX_ACTIVE_STREAMS {
                return Err(contract(
                    "maximum retained orphan streams reached; close a discovery stream",
                ));
            }
            let checkpoint = catalog.restart_orphan_scan(scan_id).await?;
            let root = url::Url::parse(&checkpoint.root_uri)
                .map_err(|error| contract(error.to_string()))?;
            let listing = RootListing::new(
                root,
                self.sessions.native_state(),
                self.sessions.pool().clone(),
            )
            .map_err(engine)?;
            ActiveScan {
                checkpoint,
                listing,
            }
        };
        let mut active = active;
        let page = active
            .listing
            .page(budget.maximum_entries, budget.maximum_bytes, cancel)
            .await
            .map_err(engine)?;
        let observations = page
            .objects
            .iter()
            .map(|object| ObservedCandidate {
                prefix: object.candidate_prefix.to_string(),
            })
            .collect::<Vec<_>>();
        let checkpoint = catalog
            .record_orphan_page(
                scan_id,
                active.checkpoint.generation,
                &observations,
                page.complete,
            )
            .await?;
        if !checkpoint.complete {
            active.checkpoint = checkpoint.clone();
            streams.active.insert(scan_id, active);
        }
        Ok(checkpoint)
    }

    /// Release a process-local stream without deleting durable candidates. The next
    /// discovery call starts a new generation at the root.
    pub async fn close_orphan_discovery(&self, scan_id: ScanId) {
        self.orphan_streams.lock().await.active.remove(&scan_id);
    }

    /// Read the persisted bounded candidate report, including unresolved ownership.
    /// `after_prefix` is a report cursor, never a backend enumeration cursor.
    /// # Errors
    /// Ephemeral runtime, invalid bound or driver failure.
    pub async fn orphan_candidates(
        &self,
        scan_id: ScanId,
        after_prefix: Option<&str>,
        limit: usize,
    ) -> Result<CandidatePage, WorkflowError> {
        Ok(self
            .operations()?
            .store()
            .catalog()
            .orphan_candidates(scan_id, after_prefix, limit)
            .await?)
    }

    /// Reclaim exactly one explicitly selected recorded candidate. Fresh catalog
    /// ownership/protection admission and its maintenance fence survive until physical
    /// deletion and disposition commit. No listing-completion inference grants deletion.
    /// # Errors
    /// Unrecorded, unattributable or protected candidate; escaped/linked local paths;
    /// cancellation or storage/store failure. Retry rechecks protections and is idempotent.
    pub async fn reclaim_orphan(
        &self,
        workspace: WorkspaceId,
        scan_id: ScanId,
        prefix: &str,
        cancel: &CancellationToken,
    ) -> Result<OrphanReclaimReport, WorkflowError> {
        let catalog = self.operations()?.store().catalog();
        let root = catalog
            .workspace(workspace)
            .await?
            .ok_or_else(|| contract("unknown workspace"))?;
        let root = url::Url::parse(&root.root_uri).map_err(|error| contract(error.to_string()))?;
        let selected = url::Url::parse(prefix).map_err(|error| contract(error.to_string()))?;
        pse_catalog::delta::discovery::check_selected_prefix(&root, &selected).map_err(engine)?;
        let claim = catalog.claim_orphan(scan_id, prefix).await?;
        if claim.workspace_id() != workspace {
            return Err(contract("selected candidate belongs to another workspace"));
        }
        let maintenance_epoch = claim.maintenance_epoch();
        let removed_objects = pse_catalog::delta::discovery::remove_selected_prefix(
            &root,
            &selected,
            self.sessions.native_state(),
            self.sessions.pool().clone(),
            cancel,
        )
        .await
        .map_err(engine)?;
        claim.complete_deleted().await?;
        Ok(OrphanReclaimReport {
            scan_id,
            prefix: prefix.to_owned(),
            maintenance_epoch,
            removed_objects,
        })
    }
}
