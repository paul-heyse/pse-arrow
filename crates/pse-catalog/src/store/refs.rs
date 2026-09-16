// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Conditional mutable refs; only admitted handles can become visible targets.

use std::sync::Arc;

use object_store::UpdateVersion;
use pse_ids::{CancellationToken, ContentHash, EncodingChecksum, SnapshotId};
use serde::{Deserialize, Serialize};

use super::local::ObservedControl;
use super::membership::AdmissionContext;
use super::open::{Catalog, infrastructure};
use super::sidecar::{RevisionReceipt, RevisionRef};
use super::verify::admission;
use crate::snapshot::ManifestRef;
use crate::{CatalogError, RefName, Snapshot};

/// Exact observed alias paired with its fully admitted immutable snapshot.
pub type PinnedRef = (RefState, Arc<Snapshot>);

/// A ref retaining exact observed bytes and native conditional tokens under ownership.
#[derive(Clone, Debug)]
pub struct RefState {
    name: Arc<RefName>,
    manifest: ManifestRef,
    revision: Option<Arc<RevisionRef>>,
    observed: ObservedControl,
    pub(super) admission: Arc<crate::store::open::CatalogContext>,
    _metadata: Arc<pse_ids::ReservationLease>,
}
impl RefState {
    /// The observed mutable alias.
    pub fn name(&self) -> &RefName {
        &self.name
    }
    /// The exact immutable manifest the alias names.
    pub const fn manifest_ref(&self) -> ManifestRef {
        self.manifest
    }
    /// The exact admitted authored revision row paired atomically with this target.
    pub fn revision_ref(&self) -> Option<&RevisionRef> {
        self.revision.as_deref()
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReferenceWire {
    snapshot_id: ContentHash,
    manifest_checksum: ContentHash,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    revision: Option<RevisionRef>,
    #[serde(default)]
    updated_at: String,
}
impl From<ManifestRef> for ReferenceWire {
    fn from(reference: ManifestRef) -> Self {
        Self {
            snapshot_id: reference.snapshot_id.0,
            manifest_checksum: reference.manifest_checksum.0,
            revision: None,
            updated_at: String::new(),
        }
    }
}
impl From<ReferenceWire> for ManifestRef {
    fn from(reference: ReferenceWire) -> Self {
        Self {
            snapshot_id: SnapshotId(reference.snapshot_id),
            manifest_checksum: EncodingChecksum(reference.manifest_checksum),
        }
    }
}

impl Catalog {
    /// Read a ref and retain the actual observed conditional predicate.
    ///
    /// # Errors
    /// Backend failures, malformed refs or unsupported object extents.
    pub async fn read_ref(
        &self,
        name: &RefName,
        cancel: &CancellationToken,
    ) -> Result<Option<RefState>, CatalogError> {
        Ok(self
            .prepare_ref_read(name, cancel)?
            .execute(cancel)
            .await?
            .into_value())
    }

    /// Capture a ref lookup and its scoped read policy before backend access.
    /// # Errors
    /// Native preparation, policy or resource refusal.
    pub fn prepare_ref_read(
        &self,
        name: &RefName,
        cancel: &CancellationToken,
    ) -> Result<super::operation::PreparedStoreOperation<Option<RefState>>, CatalogError> {
        use pse_schema::model::provider::{OperationPurpose, ProviderScope};
        let name = name.clone();
        self.prepare_store_operation(
            "store.read_ref",
            ProviderScope::Table("store".into(), "refs".into(), name.as_str().into()),
            OperationPurpose::Resolve,
            vec![datafusion::logical_expr::lit(name.as_str())],
            Box::new(move |catalog, _session, cancel| {
                Box::pin(async move {
                    let state = catalog.read_ref_inner(&name, &cancel).await?;
                    let count = u64::from(state.is_some());
                    Ok((state, count))
                })
            }),
            cancel,
        )
    }

    async fn read_ref_inner(
        &self,
        name: &RefName,
        cancel: &CancellationToken,
    ) -> Result<Option<RefState>, CatalogError> {
        let path = super::layout::ref_path(name);
        let (bytes, meta) = match self
            .read_versioned(&path, self.limits.max_control_bytes, cancel)
            .await
        {
            Ok(result) => result,
            Err(CatalogError::Infrastructure { source, .. })
                if source
                    .downcast_ref::<object_store::Error>()
                    .is_some_and(|error| matches!(error, object_store::Error::NotFound { .. })) =>
            {
                return Ok(None);
            }
            Err(error) => return Err(error),
        };
        let metadata = super::control::decode_reservation(
            &bytes,
            self.reserver.as_ref(),
            "store:ref-metadata",
        )?;
        let mut wire: ReferenceWire = serde_json::from_slice(&bytes)
            .map_err(|error| admission(path.as_ref(), &error.to_string()))?;
        if let Some(revision) = &wire.revision {
            let artifact = self.read_sidecar(&revision.artifact, cancel).await?;
            let kind = match artifact.relation().member().name.as_str() {
                "model_revisions" => pse_ids::SnapshotKind::Model,
                "case_revisions" => pse_ids::SnapshotKind::Case,
                _ => {
                    return Err(admission(
                        "revision ref",
                        "artifact is not an authored revision relation",
                    ));
                }
            };
            self.check_revision(
                &artifact,
                revision.revision_id,
                SnapshotId(wire.snapshot_id),
                kind,
            )?;
        }
        let retained = super::control::add(
            1024 + name.as_str().len(),
            wire.revision
                .as_ref()
                .map_or(Ok(0), super::control::revision_extent)?,
        )?;
        let manifest = ManifestRef {
            snapshot_id: SnapshotId(wire.snapshot_id),
            manifest_checksum: EncodingChecksum(wire.manifest_checksum),
        };
        let revision = wire.revision.take().map(Arc::new);
        drop(wire);
        let metadata = super::control::retain(metadata, retained)?;
        if let Some(revision) = &revision
            && let Some(reference) = &revision.change_set
        {
            let changes = self.read_change_set(reference, cancel).await?;
            self.check_change_target(&changes, revision, manifest)?;
        }
        Ok(Some(RefState {
            name: Arc::new(name.clone()),
            manifest,
            revision,
            observed: ObservedControl {
                bytes,
                version: UpdateVersion {
                    e_tag: meta.e_tag,
                    version: meta.version,
                },
            },
            admission: Arc::clone(&self.admission),
            _metadata: metadata,
        }))
    }

    /// Resolve an alias to its exact manifest and admit all relation content.
    ///
    /// # Errors
    /// The ref and complete snapshot admission errors; a missing alias returns `None`.
    pub async fn read_snapshot(
        &self,
        name: &RefName,
        context: &AdmissionContext,
        cancel: &CancellationToken,
    ) -> Result<Option<Arc<Snapshot>>, CatalogError> {
        let Some(state) = self.read_ref(name, cancel).await? else {
            return Ok(None);
        };
        let snapshot = self.read_manifest(state.manifest, context, cancel).await?;
        if let Some(revision) = state.revision {
            let artifact = self.read_sidecar(&revision.artifact, cancel).await?;
            self.revision_receipt(&artifact, revision.revision_id, &snapshot)?;
        }
        Ok(Some(snapshot))
    }

    /// Create an absent ref or conditionally move the exact observed version. The
    /// destination handle can only exist after successful semantic catalog admission.
    ///
    /// # Errors
    /// A stale/foreign expected state, foreign target, missing conditional tokens,
    /// malformed serialization, cancellation or backend failure.
    pub async fn compare_and_swap_ref(
        &self,
        name: &RefName,
        expected: Option<&RefState>,
        target: &Snapshot,
        cancel: &CancellationToken,
    ) -> Result<(), CatalogError> {
        self.prepare_ref_update(name, expected, target, None, cancel)?
            .execute(cancel)
            .await
            .map(|_| ())
    }

    /// Atomically publish a snapshot alias and its exact admitted authored revision row.
    /// The revision object must already be finished and stored before this last step.
    ///
    /// # Errors
    /// Revision/target mismatch, invalid artifacts, stale tokens or backend failures.
    pub async fn compare_and_swap_revision_ref(
        &self,
        name: &RefName,
        expected: Option<&RefState>,
        target: &Snapshot,
        revision: &RevisionReceipt,
        cancel: &CancellationToken,
    ) -> Result<(), CatalogError> {
        self.prepare_ref_update(name, expected, target, Some(revision), cancel)?
            .execute(cancel)
            .await
            .map(|_| ())
    }

    /// Prepare the exact conditional target and optional authored revision before any write.
    /// The returned native plan binds the target's admitted providers and scoped ref policy.
    /// # Errors
    /// A foreign target, invalid native binding or incompatible policy/effect.
    pub fn prepare_ref_update(
        &self,
        name: &RefName,
        expected: Option<&RefState>,
        target: &Snapshot,
        revision: Option<&RevisionReceipt>,
        cancel: &CancellationToken,
    ) -> Result<super::operation::PreparedStoreOperation<()>, CatalogError> {
        use pse_schema::model::provider::{OperationPurpose, ProviderScope};
        if !Arc::ptr_eq(&self.admission, &target.admission) {
            return Err(admission(
                "ref target",
                "foreign snapshots must be reopened before publication",
            ));
        }
        let target = Arc::new(target.clone());
        let mut session = self.validation_session(cancel)?;
        self.bind_snapshot(&mut session, "publication", "target", &target, cancel)?;
        let arguments = vec![
            datafusion::logical_expr::lit(name.as_str()),
            datafusion::logical_expr::lit(target.manifest_ref().manifest_checksum.to_string()),
            datafusion::logical_expr::lit(datafusion::common::ScalarValue::Utf8(
                expected.map(|state| state.manifest.manifest_checksum.to_string()),
            )),
        ];
        let name = name.clone();
        let expected = expected.cloned();
        let revision = revision.cloned();
        self.prepare_store_operation_in(
            &session,
            crate::store::operation::StoreCommand {
                name: "store.compare_and_swap_ref",
                scope: ProviderScope::Table("store".into(), "refs".into(), name.as_str().into()),
                purpose: OperationPurpose::Publish,
                arguments,
            },
            Box::new(move |catalog, _session, cancel| {
                Box::pin(async move {
                    if let Some(revision) = revision {
                        catalog
                            .write_revision_ref(
                                &name,
                                expected.as_ref(),
                                &target,
                                &revision,
                                &cancel,
                            )
                            .await?;
                    } else {
                        catalog
                            .write_ref(&name, expected.as_ref(), &target, None, &cancel)
                            .await?;
                    }
                    Ok(((), 1))
                })
            }),
            cancel,
        )
    }

    async fn write_revision_ref(
        &self,
        name: &RefName,
        expected: Option<&RefState>,
        target: &Snapshot,
        revision: &RevisionReceipt,
        cancel: &CancellationToken,
    ) -> Result<(), CatalogError> {
        if revision.target != target.manifest_ref()
            || !Arc::ptr_eq(&revision.artifact.admission, &self.admission)
        {
            return Err(admission(
                "revision ref",
                "revision receipt belongs to a different target or catalog",
            ));
        }
        let actual = self
            .read_sidecar(&revision.reference.artifact, cancel)
            .await?;
        self.revision_receipt(&actual, revision.reference.revision_id, target)?;
        if let Some(reference) = &revision.reference.change_set {
            let changes = self.read_change_set(reference, cancel).await?;
            self.check_change_target(&changes, &revision.reference, target.manifest_ref())?;
            self.check_change_base(&changes, expected)?;
        }
        self.write_ref(name, expected, target, Some(&revision.reference), cancel)
            .await
    }

    async fn write_ref(
        &self,
        name: &RefName,
        expected: Option<&RefState>,
        target: &Snapshot,
        revision: Option<&RevisionRef>,
        cancel: &CancellationToken,
    ) -> Result<(), CatalogError> {
        cancel.checkpoint()?;
        if !Arc::ptr_eq(&self.admission, &target.admission) {
            return Err(admission(
                "ref target",
                "foreign snapshots must be reopened before publication",
            ));
        }
        let observed = match expected {
            None => None,
            Some(state) => {
                if state.name.as_ref() != name || !Arc::ptr_eq(&state.admission, &self.admission) {
                    return Err(admission(
                        "ref expected state",
                        "state was read from a different ref or catalog",
                    ));
                }
                if self.local.is_none()
                    && state.observed.version.e_tag.is_none()
                    && state.observed.version.version.is_none()
                {
                    return Err(admission(
                        "ref conditional update",
                        "backend supplied neither ETag nor version token",
                    ));
                }
                Some(&state.observed)
            }
        };
        let mut metadata = self.reserver.open("store:ref-metadata-write");
        metadata.try_grow(super::control::add(
            1024,
            revision.map_or(Ok(0), super::control::revision_extent)?,
        )?)?;
        let mut wire = ReferenceWire::from(target.manifest_ref);
        wire.revision = revision.cloned();
        wire.updated_at = self.clock.now_rfc3339_utc();
        let bytes = super::encode::control(
            &wire,
            self.reserver.as_ref(),
            "store:ref-write",
            self.limits.max_control_bytes,
        )?;
        self.put_control(&super::layout::ref_path(name), observed, bytes, cancel)
            .await
            .map_err(|source| {
                if matches!(source, CatalogError::RefConflict { .. }) {
                    CatalogError::RefConflict {
                        name: name.to_string(),
                    }
                } else {
                    source
                }
            })?;
        Ok(())
    }

    /// List valid flat ref names in byte order. No remote query catalog is registered.
    ///
    /// # Errors
    /// Backend failure or a malformed path under the ref prefix.
    pub async fn list_refs(
        &self,
        cancel: &CancellationToken,
    ) -> Result<Vec<RefName>, CatalogError> {
        Ok(self
            .prepare_ref_listing(cancel)?
            .execute(cancel)
            .await?
            .into_value())
    }

    /// Prepare complete ref discovery through the same namespace policy and native route.
    /// # Errors
    /// Native preparation, policy or resource refusal.
    pub fn prepare_ref_listing(
        &self,
        cancel: &CancellationToken,
    ) -> Result<super::operation::PreparedStoreOperation<Vec<RefName>>, CatalogError> {
        use pse_schema::model::provider::{OperationPurpose, ProviderScope};
        self.prepare_store_operation(
            "store.list_refs",
            ProviderScope::Schema("store".into(), "refs".into()),
            OperationPurpose::Resolve,
            vec![],
            Box::new(move |catalog, _session, cancel| {
                Box::pin(async move {
                    let names = catalog.list_refs_inner(&cancel).await?;
                    let count =
                        u64::try_from(names.len()).map_err(|_| super::encode::overflow())?;
                    Ok((names, count))
                })
            }),
            cancel,
        )
    }

    async fn list_refs_inner(
        &self,
        cancel: &CancellationToken,
    ) -> Result<Vec<RefName>, CatalogError> {
        cancel.checkpoint()?;
        let prefix = object_store::path::Path::from("refs");
        let listing = cancel
            .until_cancelled(self.store.list_with_delimiter(Some(&prefix)))
            .await?
            .map_err(|source| infrastructure("list refs", source))?;
        let mut names = Vec::new();
        for object in listing.objects {
            cancel.checkpoint()?;
            let path = object.location.as_ref();
            if path
                .strip_prefix("refs/")
                .is_some_and(super::layout::is_local_temporary)
            {
                continue;
            }
            let name = path
                .strip_prefix("refs/")
                .and_then(|name| name.strip_suffix(".json"))
                .ok_or_else(|| admission(path, "invalid ref path"))?;
            names.push(RefName::parse(name)?);
        }
        names.sort();
        Ok(names)
    }
}
