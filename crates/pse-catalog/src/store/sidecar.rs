// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed immutable control relations, explicitly excluded from snapshot membership.

use std::sync::Arc;

use pse_ids::{CancellationToken, CanonicalizeOptions, SemanticId, SnapshotKind};
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
    pub(super) admission: Arc<()>,
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

/// Exact reference to one schema/value/key-admitted staged row, not snapshot membership.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StagedRef(pub(super) SidecarRef);
/// One typed operation-role row whose reservation and original artifact remain owned.
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
    /// The schema/value/key-admitted row.
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
    StagedRow,
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
        self.publish_artifact(draft, ArtifactKind::Sidecar, cancel)
            .await
    }

    /// Store one authored/reference preimage or replacement row under its exact declared
    /// schema. This admits representation/values/key only, never a complete snapshot.
    /// # Errors
    /// Invalid class, row count, schema, values, resource or storage failures.
    pub async fn publish_staged_row(
        &self,
        draft: RelationDraft,
        cancel: &CancellationToken,
    ) -> Result<StagedArtifact, CatalogError> {
        let inner = self
            .publish_artifact(draft, ArtifactKind::StagedRow, cancel)
            .await?;
        Ok(StagedArtifact {
            reference: StagedRef(inner.reference.clone()),
            inner,
        })
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
        if matches!(kind, ArtifactKind::StagedRow) && candidate.num_rows() != 1 {
            return Err(admission(
                "staged row",
                "an operation-role artifact must contain exactly one row",
            ));
        }
        if matches!(kind, ArtifactKind::Sidecar) {
            self.admit_sidecar_semantics(spec, &candidate, cancel)
                .await?;
        }
        let port = pse_ids::model_port_name(spec.key.namespace.as_str(), spec.id);
        let prepared = self.prepare_relation(port, draft, spec, &candidate, cancel)?;
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
        self.read_artifact(reference, ArtifactKind::Sidecar, cancel)
            .await
    }

    /// Reopen and directly admit one exact staged row. No snapshot is manufactured.
    /// # Errors
    /// Invalid artifact identity, actual row count/schema/values or storage failures.
    pub async fn read_staged_row(
        &self,
        reference: &StagedRef,
        cancel: &CancellationToken,
    ) -> Result<StagedArtifact, CatalogError> {
        let inner = self
            .read_artifact(&reference.0, ArtifactKind::StagedRow, cancel)
            .await?;
        Ok(StagedArtifact {
            reference: reference.clone(),
            inner,
        })
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
        let batch = super::verify::ipc_file(
            &bytes,
            &self.registry,
            spec,
            self.reserver.as_ref(),
            cancel,
            self.limits.envelope,
        )?;
        if matches!(kind, ArtifactKind::StagedRow) && batch.num_rows() != 1 {
            return Err(admission(
                "staged row",
                "an operation-role artifact must contain exactly one row",
            ));
        }
        if matches!(kind, ArtifactKind::Sidecar) {
            self.admit_sidecar_semantics(spec, &batch, cancel).await?;
        }
        let output = pse_ids::canonicalize(
            &contract.canonical,
            &[batch],
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
        let batch = output
            .sorted
            .ok_or_else(|| admission("sidecar", "canonical sorted output missing"))?;
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
        validator
            .validate_sidecar(&self.registry, &rows, cancel)
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
                        ArtifactKind::StagedRow => {
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
