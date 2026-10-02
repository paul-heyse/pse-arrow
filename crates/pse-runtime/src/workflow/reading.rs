// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Reading publications through catalog reader leases (ADR-0114 Outcome 7; finding T02;
//! Plan 22 X10, X12).
//!
//! A reader takes a lease row in one short transaction, which returns the complete record
//! and the workspace's maintenance epoch; it then reads Delta files holding no database
//! session. A [`ReaderLeaseGuard`] renews the lease in short transactions at a third of
//! its lifetime, is retained by the publication's session (so streams keep it), and
//! releases it when the last owner drops. If a renewal finds the lease lapsed, the guard
//! cancels its token and reports [`OperationsError::ReaderLeaseLapsed`]. The session
//! installs the grant's [`ReadScope`], the lookup input of every shared cache.
//!
//! An export is a reader lease held by `export:<destination>` for a stated time: the
//! record, the lease and its expiry, the epoch and the store fingerprint are written as a
//! one-row manifest an offline reader opens without the store.
use super::{Runtime, WorkflowError, contract};
use pse_catalog::delta::{
    publication::{Publication, PublicationSelection},
    scope::ReadScope,
};
use pse_columnar::CancellationToken;
use pse_operations::catalog::RuntimeOperationalReaderLeasesRow;
use pse_operations::{
    OperationsError, Store,
    catalog::{PublicationId, PublicationRecord, ReadTarget, ReaderLeaseId, WorkspaceId},
};
use pse_relations::generated::runtime::publication_manifests;
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

/// The lifetime of a reader lease between renewals; a guard renews at a third of it.
pub const READER_LEASE: Duration = Duration::from_secs(60);

/// A live catalog reader lease. Renewed in the background while any owner holds it;
/// released when the last owner drops.
#[derive(Debug)]
pub struct ReaderLeaseGuard {
    lease: RuntimeOperationalReaderLeasesRow,
    store: Store,
    lapse: CancellationToken,
    lapsed: Arc<AtomicBool>,
    renewal: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl ReaderLeaseGuard {
    fn start(
        lease: RuntimeOperationalReaderLeasesRow,
        store: Store,
        ttl: Duration,
        cancel: &CancellationToken,
    ) -> Arc<Self> {
        let lapse = cancel.child_token();
        let lapsed = Arc::new(AtomicBool::new(false));
        let renewal = tokio::runtime::Handle::try_current().ok().map(|handle| {
            let (store, lapse, lapsed, id) = (
                store.clone(),
                lapse.clone(),
                Arc::clone(&lapsed),
                lease.lease_id,
            );
            let deadline = std::time::Instant::now() + ttl;
            handle.spawn(async move { renew(store, id, ttl, deadline, lapse, lapsed).await })
        });
        Arc::new(Self {
            lease,
            store,
            lapse,
            lapsed,
            renewal: Mutex::new(renewal),
        })
    }
    /// The lease as granted: the resolved publication, holder and first expiry.
    pub const fn lease(&self) -> &RuntimeOperationalReaderLeasesRow {
        &self.lease
    }
    /// Cancelled when the lease lapses (or the opener's token is cancelled): readers
    /// pass it to their reads.
    pub const fn cancellation(&self) -> &CancellationToken {
        &self.lapse
    }
    /// Whether the lease is still protecting the publication.
    /// # Errors
    /// [`OperationsError::ReaderLeaseLapsed`] once a renewal found it lapsed.
    pub fn check(&self) -> Result<(), WorkflowError> {
        if self.lapsed.load(Ordering::Acquire) {
            return Err(OperationsError::ReaderLeaseLapsed {
                lease: self.lease.lease_id,
            }
            .into());
        }
        Ok(())
    }
}

impl Drop for ReaderLeaseGuard {
    fn drop(&mut self) {
        if let Ok(mut renewal) = self.renewal.lock()
            && let Some(renewal) = renewal.take()
        {
            renewal.abort();
        }
        // Release in a short transaction; without a runtime the lease simply expires.
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            let (store, lease) = (self.store.clone(), self.lease.lease_id);
            handle.spawn(async move {
                let _ = store.catalog().release_reader_lease(lease).await;
            });
        }
    }
}

/// Renew at a third of the lifetime until the lease lapses; a transient failure is
/// retried until the lease's own expiry.
async fn renew(
    store: Store,
    lease: ReaderLeaseId,
    ttl: Duration,
    mut deadline: std::time::Instant,
    lapse: CancellationToken,
    lapsed: Arc<AtomicBool>,
) {
    let period = (ttl / 3).max(Duration::from_millis(10));
    loop {
        tokio::time::sleep(period).await;
        match store.catalog().renew_reader_lease(lease, ttl).await {
            Ok(_) => deadline = std::time::Instant::now() + ttl,
            Err(error) if error.is_retryable() && std::time::Instant::now() < deadline => {}
            Err(_) => {
                lapsed.store(true, Ordering::Release);
                lapse.cancel();
                return;
            }
        }
    }
}

/// A publication read under a catalog reader lease.
#[derive(Debug, Clone)]
pub struct LeasedPublication {
    publication: Arc<Publication>,
    guard: Arc<ReaderLeaseGuard>,
}

impl LeasedPublication {
    /// The opened publication.
    pub const fn publication(&self) -> &Arc<Publication> {
        &self.publication
    }
    /// The lease guard, also retained by the publication's session.
    pub const fn guard(&self) -> &Arc<ReaderLeaseGuard> {
        &self.guard
    }
    /// The publication identity.
    pub fn publication_id(&self) -> PublicationId {
        self.guard.lease.publication_id
    }
}

/// The publication record as the catalog holds it, in the form Delta readers use.
fn manifest_of(
    record: &PublicationRecord,
    export: Option<(&RuntimeOperationalReaderLeasesRow, i64)>,
) -> publication_manifests::Row {
    let publication = &record.publication;
    let (exported_at, export_lease_id, export_expires_at, maintenance_epoch, store_fingerprint) =
        match export {
            Some((lease, epoch)) => (
                Some(lease.acquired_at),
                Some(lease.lease_id),
                Some(lease.expires_at),
                Some(epoch),
                Some(pse_operations::generated::SCHEMA_FINGERPRINT),
            ),
            None => (None, None, None, None, None),
        };
    publication_manifests::Row {
        publication_id: publication.publication_id,
        workspace_id: publication.workspace_id,
        parent_publication_id: publication.parent_publication,
        attempt_id: publication.attempt_id,
        kind: publication.kind,
        inputs: record.inputs.clone(),
        members: record.members.clone(),
        windows: record.windows.clone(),
        exported_at,
        export_lease_id,
        export_expires_at,
        maintenance_epoch,
        store_fingerprint,
    }
}

/// A receipt of an export: the manifest's location and the lease protecting its members.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ExportReceipt {
    /// The exported publication.
    pub publication_id: PublicationId,
    /// The manifest location.
    #[schemars(with = "String")]
    pub destination: url::Url,
    /// The lease protecting the members until it expires or is released.
    pub lease_id: ReaderLeaseId,
    /// When the export expires, microseconds since the Unix epoch.
    pub expires_at: i64,
}

#[derive(Clone, Copy)]
enum ReadConsumption {
    Current,
    Recorded,
}

impl Runtime {
    /// Open an exact publication under a catalog reader lease.
    /// # Errors
    /// An ephemeral runtime; an unknown, expiring or deleted publication; member open
    /// failures.
    pub async fn open(
        &self,
        publication: PublicationId,
        cancel: &CancellationToken,
    ) -> Result<LeasedPublication, WorkflowError> {
        self.open_leased(ReadTarget::Publication(publication), cancel)
            .await
    }

    /// Open the head of a workspace under a catalog reader lease; the lease records the
    /// resolved publication.
    /// # Errors
    /// An ephemeral runtime; an unknown workspace or an empty head; member open failures.
    pub async fn open_head(
        &self,
        workspace: WorkspaceId,
        cancel: &CancellationToken,
    ) -> Result<LeasedPublication, WorkflowError> {
        self.open_leased(ReadTarget::Head(workspace), cancel).await
    }

    async fn open_leased(
        &self,
        target: ReadTarget,
        cancel: &CancellationToken,
    ) -> Result<LeasedPublication, WorkflowError> {
        self.open_leased_interpreted(target, cancel, ReadConsumption::Current)
            .await
    }
    /// Retain an exact catalog reader lease while opening original recorded meaning
    /// for an explicit artifact migration. Source protection survives into its plans.
    /// # Errors
    /// Missing or expiring selections, invalid recorded declarations or policy refusal.
    pub async fn read_migration_source(
        &self,
        target: ReadTarget,
        cancel: &CancellationToken,
    ) -> Result<LeasedPublication, WorkflowError> {
        self.open_leased_interpreted(target, cancel, ReadConsumption::Recorded)
            .await
    }
    async fn open_leased_interpreted(
        &self,
        target: ReadTarget,
        cancel: &CancellationToken,
        consumption: ReadConsumption,
    ) -> Result<LeasedPublication, WorkflowError> {
        let operations = self.operations()?;
        let granted = operations
            .store()
            .catalog()
            .acquire_reader_lease(
                pse_operations::mint_id(),
                target,
                &format!("reader:{}", operations.worker()),
                READER_LEASE,
            )
            .await?;
        let guard = ReaderLeaseGuard::start(
            granted.lease.clone(),
            operations.store().clone(),
            READER_LEASE,
            cancel,
        );
        let scope = ReadScope {
            workspace: granted.record.publication.workspace_id,
            epoch: granted.maintenance_epoch,
        };
        let selection = PublicationSelection {
            record: manifest_of(&granted.record, None),
            scope: Some(scope),
            owner: Some(guard.clone()),
        };
        let publication = match consumption {
            ReadConsumption::Current => {
                Publication::open(selection, self.registry.clone(), &self.sessions, cancel).await?
            }
            ReadConsumption::Recorded => {
                Publication::open_recorded(selection, self.registry.clone(), &self.sessions, cancel)
                    .await?
            }
        };
        Ok(LeasedPublication {
            publication: Arc::new(publication),
            guard,
        })
    }

    /// Export a publication for offline readers: a reader lease held by
    /// `export:<destination>` for `valid_for`, and a one-row manifest at `destination`
    /// recording the record, the lease, its expiry, the maintenance epoch and the store
    /// fingerprint. The lease is not renewed; release it with [`Self::release_export`].
    /// # Errors
    /// An ephemeral runtime; an unknown, expiring or deleted publication; an existing
    /// destination; write failures (the lease is then released).
    pub async fn export_publication(
        &self,
        publication: PublicationId,
        destination: url::Url,
        valid_for: Duration,
        cancel: &CancellationToken,
    ) -> Result<ExportReceipt, WorkflowError> {
        let catalog = self.operations()?.store().catalog();
        let granted = catalog
            .acquire_reader_lease(
                pse_operations::mint_id(),
                ReadTarget::Publication(publication),
                &format!("export:{destination}"),
                valid_for,
            )
            .await?;
        let record = manifest_of(
            &granted.record,
            Some((&granted.lease, granted.maintenance_epoch)),
        );
        let written = async {
            let session = self.sessions.candidate(
                std::collections::BTreeMap::new(),
                self.registry.clone(),
                cancel,
            )?;
            pse_catalog::delta::manifest::prepare_manifest(
                &session,
                destination.clone(),
                record,
                cancel,
            )?
            .execute(cancel)
            .await?;
            Ok::<_, WorkflowError>(())
        }
        .await;
        if let Err(error) = written {
            let _ = catalog.release_reader_lease(granted.lease.lease_id).await;
            return Err(error);
        }
        Ok(ExportReceipt {
            publication_id: publication,
            destination,
            lease_id: granted.lease.lease_id,
            expires_at: granted.lease.expires_at,
        })
    }

    /// Release an export's lease; its manifest's members may then be retired. Returns
    /// whether the lease was still held.
    /// # Errors
    /// An ephemeral runtime; store failures.
    pub async fn release_export(&self, receipt: &ExportReceipt) -> Result<bool, WorkflowError> {
        Ok(self
            .operations()?
            .store()
            .catalog()
            .release_reader_lease(receipt.lease_id)
            .await?)
    }
}

/// Open an exported publication offline, without the operational store: exactly the
/// members its manifest names, under the manifest's read scope.
/// # Errors
/// [`WorkflowError::ExportLeaseExpired`] after the export's expiry; a former Delta
/// control table (migration required); a manifest of another format or version; member
/// open failures.
pub async fn open_export(
    location: url::Url,
    registry: Arc<pse_schema::Registry>,
    factory: &pse_engine::session::EngineFactory,
    cancel: &CancellationToken,
) -> Result<Publication, WorkflowError> {
    let state = Arc::new(factory.native_state().clone());
    let record = pse_catalog::delta::manifest::read_manifest(location, &registry, state)
        .await
        .map_err(|error| WorkflowError::Engine(pse_engine::session::engine(error)))?;
    let (Some(expires_at), Some(epoch)) = (record.export_expires_at, record.maintenance_epoch)
    else {
        return Err(contract("an export manifest records its lease and epoch"));
    };
    let now = chrono::Utc::now().timestamp_micros();
    if now >= expires_at {
        return Err(WorkflowError::ExportLeaseExpired {
            publication: record.publication_id,
            expires_at,
        });
    }
    let scope = ReadScope {
        workspace: record.workspace_id,
        epoch,
    };
    Ok(Publication::open(
        PublicationSelection {
            record,
            scope: Some(scope),
            owner: None,
        },
        registry,
        factory,
        cancel,
    )
    .await?)
}
