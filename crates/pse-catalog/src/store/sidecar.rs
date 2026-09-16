// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed immutable control relations, explicitly excluded from snapshot membership.

use std::sync::Arc;

use super::operation::{PreparedStoreOperation, StoreCommand};
use pse_ids::{CancellationToken, CanonicalizeOptions, SemanticId, SnapshotKind};
use pse_schema::model::provider::{OperationPurpose, ProviderScope};
use pse_schema::model::{Authority, Cell, RelationSpec, SnapshotClass};
use serde::{Deserialize, Serialize};

use super::open::Catalog;
use super::publish::RelationDraft;
use super::verify::admission;
use crate::snapshot::ManifestRef;
use crate::{
    CatalogError, EncodingFormat, EncodingPolicy, LoadedRelation, RelationContract, RelationMember,
    Snapshot,
};

/// Exact physical reference to a control relation. Deserialization does not admit it;
/// `read_sidecar` checks its actual schema, values, keys and finished encoding.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SidecarRef {
    member: RelationMember,
}
impl SidecarRef {
    /// The complete encoded artifact claim, including length and physical checksum.
    pub fn member(&self) -> &RelationMember {
        &self.member
    }
}

/// A sidecar with admitted physical/schema/value/key content and retained buffer owners.
/// Cross-object revision/pass semantics are established by their typed receipt helpers.
#[derive(Clone, Debug)]
pub struct SidecarArtifact {
    reference: SidecarRef,
    relation: Arc<LoadedRelation>,
    pub(super) admission: Arc<crate::store::open::CatalogContext>,
}
impl SidecarArtifact {
    /// The immutable serializable reference; it must be admitted again when restored.
    pub fn reference(&self) -> &SidecarRef {
        &self.reference
    }
    /// The checked relation content.
    pub fn relation(&self) -> &Arc<LoadedRelation> {
        &self.relation
    }
}

/// Exact reference to a schema/value/key-admitted staged batch, not snapshot membership.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StagedRef(pub(super) SidecarRef);
/// A typed operation batch whose order, reservation and original artifact remain owned.
#[derive(Clone, Debug)]
pub struct StagedArtifact {
    reference: StagedRef,
    inner: SidecarArtifact,
}
impl StagedArtifact {
    /// Exact immutable reference for a durable operation-role port.
    pub fn reference(&self) -> &StagedRef {
        &self.reference
    }
    /// The schema/value/key-admitted batch in its original operation-ordinal order.
    pub fn relation(&self) -> &Arc<LoadedRelation> {
        self.inner.relation()
    }
    pub(super) fn belongs_to(&self, catalog: &Catalog) -> bool {
        Arc::ptr_eq(&self.inner.admission, &catalog.admission)
    }
}
#[derive(Clone, Copy)]
enum ArtifactKind {
    Sidecar,
    StagedBatch,
}

/// Exact revision-row witness retained in a mutable ref's single atomic value.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RevisionRef {
    /// Stable revision identity from the actual typed revision row.
    pub revision_id: SemanticId,
    /// Exact immutable artifact containing that row.
    pub artifact: SidecarRef,
    /// Exact complete change-set control receipt, when this is a commit revision.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change_set: Option<super::changes::ChangeSetRef>,
}
/// A revision reference admitted against its target snapshot in this catalog.
#[derive(Clone, Debug)]
pub struct RevisionReceipt {
    pub(super) reference: RevisionRef,
    pub(super) target: ManifestRef,
    pub(super) artifact: SidecarArtifact,
    pub(super) snapshot: Arc<Snapshot>,
}
impl RevisionReceipt {
    /// The exact durable row reference.
    pub fn reference(&self) -> &RevisionRef {
        &self.reference
    }
}

impl Catalog {
    /// Publish a declared sidecar relation after schema/value/key admission and a
    /// finished IPC round trip. It cannot become semantic snapshot membership.
    ///
    /// # Errors
    /// Wrong class/contract, malformed values or keys, resource/cancel/backend failures.
    pub async fn publish_sidecar(
        &self,
        draft: RelationDraft,
        cancel: &CancellationToken,
    ) -> Result<SidecarArtifact, CatalogError> {
        Ok(self
            .prepare_sidecar_publication(draft, cancel)?
            .execute(cancel)
            .await?
            .into_value())
    }

    /// Store an authored/reference preimage or replacement batch with stable row
    /// ordinals. This admits representation/values/key only, never a complete snapshot.
    /// # Errors
    /// Invalid class, row count, schema, values, resource or storage failures.
    pub async fn publish_staged_batch(
        &self,
        draft: RelationDraft,
        cancel: &CancellationToken,
    ) -> Result<StagedArtifact, CatalogError> {
        Ok(self
            .prepare_staged_publication(draft, cancel)?
            .execute(cancel)
            .await?
            .into_value())
    }

    /// Prepare immutable sidecar validation and publication without executing rows or I/O.
    /// # Errors
    /// Declaration, binding, policy or resource failure.
    pub fn prepare_sidecar_publication(
        &self,
        draft: RelationDraft,
        cancel: &CancellationToken,
    ) -> Result<PreparedStoreOperation<SidecarArtifact>, CatalogError> {
        self.prepare_artifact_publication(
            draft,
            ArtifactKind::Sidecar,
            std::convert::identity,
            cancel,
        )
    }

    /// Prepare an ordered operation batch in the same native publication lifecycle.
    /// # Errors
    /// Declaration, binding, policy or resource failure.
    pub fn prepare_staged_publication(
        &self,
        draft: RelationDraft,
        cancel: &CancellationToken,
    ) -> Result<PreparedStoreOperation<StagedArtifact>, CatalogError> {
        self.prepare_artifact_publication(draft, ArtifactKind::StagedBatch, staged, cancel)
    }

    fn prepare_artifact_publication<T: Send + 'static>(
        &self,
        draft: RelationDraft,
        kind: ArtifactKind,
        finish: fn(SidecarArtifact) -> T,
        cancel: &CancellationToken,
    ) -> Result<PreparedStoreOperation<T>, CatalogError> {
        let mut session = self.validation_session(cancel)?;
        let reference = datafusion::common::TableReference::full(
            "publication",
            "artifacts",
            draft.contract.canonical.relation_id.to_string(),
        );
        self.bind_draft(&mut session, reference.clone(), &draft)?;
        self.prepare_store_operation_in(
            &session,
            StoreCommand {
                name: match kind {
                    ArtifactKind::Sidecar => "store.publish_sidecar",
                    ArtifactKind::StagedBatch => "store.publish_staged_batch",
                },
                scope: ProviderScope::Table(
                    "publication".into(),
                    "artifacts".into(),
                    reference.table().into(),
                ),
                purpose: OperationPurpose::Publish,
                arguments: vec![datafusion::logical_expr::lit(
                    draft.contract.canonical.relation_id.to_string(),
                )],
            },
            Box::new(move |catalog, _session, cancel| {
                Box::pin(async move {
                    let artifact = catalog.publish_artifact(draft, kind, &cancel).await?;
                    let rows = u64::try_from(artifact.relation.rows())
                        .map_err(|_| super::encode::overflow())?;
                    Ok((finish(artifact), rows))
                })
            }),
            cancel,
        )
    }

    async fn publish_artifact(
        &self,
        draft: RelationDraft,
        kind: ArtifactKind,
        cancel: &CancellationToken,
    ) -> Result<SidecarArtifact, CatalogError> {
        let spec = self.artifact_spec(draft.contract.canonical.relation_id, kind)?;
        draft
            .contract
            .validate_against_registry(&self.registry, spec)?;
        if draft.contract.encodings != EncodingPolicy::IpcFile {
            return Err(admission(
                "sidecar",
                "control relation artifacts use one finished IPC file",
            ));
        }
        let candidate = super::publish::combine(self, &draft, spec, cancel)?;
        if matches!(kind, ArtifactKind::Sidecar) {
            self.admit_sidecar_semantics(spec, candidate.batch(), cancel)
                .await?;
        }
        let port = pse_ids::model_port_name(spec.key.namespace.as_str(), spec.id);
        let order = match kind {
            ArtifactKind::Sidecar => super::publish::RowOrder::CanonicalKeys,
            ArtifactKind::StagedBatch => super::publish::RowOrder::OperationOrdinals,
        };
        let prepared = self.prepare_relation(port, draft, spec, &candidate, order, cancel)?;
        for (path, bytes) in prepared.objects {
            self.ensure_create(&path, bytes, cancel).await?;
        }
        Ok(SidecarArtifact {
            reference: SidecarRef {
                member: prepared.relation.member.as_ref().clone(),
            },
            relation: Arc::new(prepared.relation),
            admission: Arc::clone(&self.admission),
        })
    }

    /// Reopen a sidecar by its exact artifact claim, checking actual content before
    /// comparing its logical identity. This does not create a snapshot handle.
    ///
    /// # Errors
    /// Malformed claims/bytes, schema/value/key violations or resource/backend failures.
    pub async fn read_sidecar(
        &self,
        reference: &SidecarRef,
        cancel: &CancellationToken,
    ) -> Result<SidecarArtifact, CatalogError> {
        Ok(self
            .prepare_sidecar_read(reference, cancel)?
            .execute(cancel)
            .await?
            .into_value())
    }

    /// Reopen a staged batch, retaining the encoded operation ordinals.
    /// # Errors
    /// Invalid artifact identity, actual row count/schema/values or storage failures.
    pub async fn read_staged_batch(
        &self,
        reference: &StagedRef,
        cancel: &CancellationToken,
    ) -> Result<StagedArtifact, CatalogError> {
        Ok(self
            .prepare_staged_read(reference, cancel)?
            .execute(cancel)
            .await?
            .into_value())
    }

    /// Prepare actual sidecar resolution and admission under the operation policy.
    /// # Errors
    /// Invalid native preparation or policy/resource failure.
    pub fn prepare_sidecar_read(
        &self,
        reference: &SidecarRef,
        cancel: &CancellationToken,
    ) -> Result<PreparedStoreOperation<SidecarArtifact>, CatalogError> {
        self.prepare_artifact_read(
            reference,
            ArtifactKind::Sidecar,
            std::convert::identity,
            cancel,
        )
    }

    /// Prepare actual ordered operation-batch resolution and admission.
    /// # Errors
    /// Invalid native preparation or policy/resource failure.
    pub fn prepare_staged_read(
        &self,
        reference: &StagedRef,
        cancel: &CancellationToken,
    ) -> Result<PreparedStoreOperation<StagedArtifact>, CatalogError> {
        self.prepare_artifact_read(&reference.0, ArtifactKind::StagedBatch, staged, cancel)
    }

    fn prepare_artifact_read<T: Send + 'static>(
        &self,
        reference: &SidecarRef,
        kind: ArtifactKind,
        finish: fn(SidecarArtifact) -> T,
        cancel: &CancellationToken,
    ) -> Result<PreparedStoreOperation<T>, CatalogError> {
        let reference = reference.clone();
        self.prepare_store_operation(
            match kind {
                ArtifactKind::Sidecar => "store.read_sidecar",
                ArtifactKind::StagedBatch => "store.read_staged_batch",
            },
            ProviderScope::Table(
                "publication".into(),
                "artifacts".into(),
                reference.member.relation_id.to_string(),
            ),
            OperationPurpose::Resolve,
            vec![datafusion::logical_expr::lit(
                reference.member.relation_id.to_string(),
            )],
            Box::new(move |catalog, _session, cancel| {
                Box::pin(async move {
                    let artifact = catalog.read_artifact(&reference, kind, &cancel).await?;
                    let rows = u64::try_from(artifact.relation.rows())
                        .map_err(|_| super::encode::overflow())?;
                    Ok((finish(artifact), rows))
                })
            }),
            cancel,
        )
    }

    async fn read_artifact(
        &self,
        reference: &SidecarRef,
        kind: ArtifactKind,
        cancel: &CancellationToken,
    ) -> Result<SidecarArtifact, CatalogError> {
        let member = &reference.member;
        let mut metadata = self.reserver.open("store:artifact-metadata");
        metadata.try_grow(super::control::add(
            super::control::mul(super::stage_owned::member_extent(member)?, 3)?,
            1024,
        )?)?;
        let metadata = pse_ids::ReservationLease::new(metadata);
        let spec = self.artifact_spec(member.relation_id, kind)?;
        let contract = Arc::new(RelationContract::from_spec(
            &self.registry,
            spec,
            EncodingPolicy::IpcFile,
        )?);
        if member.namespace != spec.key.namespace.as_str()
            || member.name != spec.key.name
            || member.version != contract.canonical.schema_version
            || member.port != pse_ids::model_port_name(spec.key.namespace.as_str(), spec.id)
            || member.encodings.len() != 1
        {
            return Err(admission(
                "sidecar",
                "artifact identity differs from the declared control relation",
            ));
        }
        let encoding = &member.encodings[0];
        let path = super::layout::relation_path(
            spec.key.namespace.as_str(),
            spec.key.name,
            member.version,
            &encoding.encoding_checksum,
            EncodingFormat::ArrowIpcFile,
        );
        if encoding.format != EncodingFormat::ArrowIpcFile || encoding.path != path.as_ref() {
            return Err(admission(
                "sidecar",
                "encoding path or format is not the declared IPC artifact",
            ));
        }
        let bytes = self
            .read_bytes(&path, self.limits.max_object_bytes, cancel)
            .await?;
        super::verify::encoding(&bytes, encoding)?;
        let batch = super::verify::ipc_file_checked(
            &bytes,
            &self.registry,
            spec,
            self.reserver.as_ref(),
            cancel,
            self.limits.envelope,
        )?;
        if matches!(kind, ArtifactKind::Sidecar) {
            self.admit_sidecar_semantics(spec, batch.batch(), cancel)
                .await?;
        }
        let (sorted, output) = batch.canonicalize(
            &self.registry,
            spec,
            self.reserver.as_ref(),
            CanonicalizeOptions {
                keep_sorted: true,
                envelope: self.limits.envelope,
                cancel: Some(cancel.clone()),
                ..Default::default()
            },
        )?;
        if output.logical_hash != member.logical_hash || output.row_count != member.rows {
            return Err(admission(
                "sidecar",
                "actual admitted content differs from logical identity or row count claim",
            ));
        }
        let batch = match kind {
            ArtifactKind::Sidecar => sorted,
            ArtifactKind::StagedBatch => batch,
        };
        Ok(SidecarArtifact {
            reference: reference.clone(),
            relation: Arc::new(LoadedRelation {
                contract,
                batch,
                member: super::control::OwnedControl::new(member.clone(), metadata),
            }),
            admission: Arc::clone(&self.admission),
        })
    }

    async fn admit_sidecar_semantics(
        &self,
        spec: &RelationSpec,
        batch: &datafusion::arrow::array::RecordBatch,
        cancel: &CancellationToken,
    ) -> Result<(), CatalogError> {
        if !self
            .registry
            .invariants()
            .iter()
            .any(|invariant| invariant.relation == spec.key.qualified_name())
        {
            return Ok(());
        }
        let validator = self.validator.as_ref().ok_or_else(|| {
            admission(
                "sidecar semantics",
                "declared sidecar invariants require the executable rule validator",
            )
        })?;
        cancel.checkpoint()?;
        let mut reservation = self.reserver.open("store:sidecar-semantic-admission");
        reservation.try_grow(super::membership::validation_extent(batch)?)?;
        let rows = std::collections::BTreeMap::from([(spec.key, batch.clone())]);
        let session = self.validation_session(cancel)?;
        validator
            .validate_sidecar(&self.registry, &rows, &session, cancel)
            .await?;
        cancel.checkpoint()?;
        Ok(())
    }

    fn artifact_spec(
        &self,
        id: SemanticId,
        kind: ArtifactKind,
    ) -> Result<&RelationSpec, CatalogError> {
        self.registry
            .relation_by_id(id)
            .filter(|spec| {
                self.registry.relation(&spec.key.qualified_name()) == Some(*spec)
                    && match kind {
                        ArtifactKind::Sidecar => spec.snapshot_class == SnapshotClass::Sidecar,
                        ArtifactKind::StagedBatch => {
                            matches!(spec.authority, Authority::Authored | Authority::Reference)
                        }
                    }
            })
            .ok_or_else(|| {
                admission(
                    "relation artifact",
                    "an exact current declaration of the permitted authority/class is required",
                )
            })
    }

    /// Bind the actual revision row to its already admitted target snapshot.
    /// Revision identity is supplied by the authored change; none is manufactured.
    ///
    /// # Errors
    /// Foreign handles, wrong revision kind/key or an actual snapshot target mismatch.
    pub fn revision_receipt(
        &self,
        artifact: &SidecarArtifact,
        revision_id: SemanticId,
        target: &Snapshot,
    ) -> Result<RevisionReceipt, CatalogError> {
        if !Arc::ptr_eq(&self.admission, &artifact.admission)
            || !Arc::ptr_eq(&self.admission, &target.admission)
        {
            return Err(admission(
                "revision",
                "foreign artifacts and snapshots must be reopened",
            ));
        }
        self.check_revision(
            artifact,
            revision_id,
            target.snapshot_id(),
            target.manifest.snapshot_kind,
        )?;
        Ok(RevisionReceipt {
            reference: RevisionRef {
                revision_id,
                artifact: artifact.reference.clone(),
                change_set: None,
            },
            target: target.manifest_ref(),
            artifact: artifact.clone(),
            snapshot: Arc::new(target.clone()),
        })
    }

    pub(super) fn check_revision(
        &self,
        artifact: &SidecarArtifact,
        revision_id: SemanticId,
        target: pse_ids::SnapshotId,
        kind: SnapshotKind,
    ) -> Result<(), CatalogError> {
        let (relation, key) = match kind {
            SnapshotKind::Model => ("authored.model_revisions", "model_revision_id"),
            SnapshotKind::Case => ("authored.case_revisions", "case_revision_id"),
            _ => {
                return Err(admission(
                    "revision",
                    "only model and case snapshots have authored revision rows",
                ));
            }
        };
        let spec = self
            .registry
            .relation(relation)
            .ok_or_else(|| admission("revision", "revision relation is not declared"))?;
        if artifact.relation.contract.canonical.relation_id != spec.id {
            return Err(admission(
                "revision",
                "revision artifact has the wrong relation",
            ));
        }
        let batch = artifact.relation.batch();
        let mut reservation = self.reserver.open("store:revision-row-check");
        reservation.try_grow(super::membership::validation_extent(batch)?)?;
        let rows = pse_relations::cells::cells_from_batch(&self.registry, spec, batch)
            .map_err(|error| admission("revision", &error.to_string()))?;
        let key = batch.schema().index_of(key).map_err(super::encode::arrow)?;
        let snapshot = batch
            .schema()
            .index_of("snapshot_id")
            .map_err(super::encode::arrow)?;
        if rows.len() != 1
            || rows[0][key] != Cell::Id(revision_id)
            || rows[0][snapshot] != Cell::Hash(target.0)
        {
            return Err(admission(
                "revision",
                "artifact must contain exactly the named revision row targeting this snapshot",
            ));
        }
        Ok(())
    }
}

fn staged(inner: SidecarArtifact) -> StagedArtifact {
    StagedArtifact {
        reference: StagedRef(inner.reference.clone()),
        inner,
    }
}
