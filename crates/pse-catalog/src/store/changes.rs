// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Durable operation receipts; C2 owns operation application semantics.

mod metadata;
mod operation;
mod validate;

use std::collections::BTreeMap;
use std::sync::Arc;

use pse_ids::{CancellationToken, ContentHash, EncodingChecksum, SemanticId};
use serde::{Deserialize, Serialize};

use super::open::Catalog;
use super::refs::RefState;
use super::sidecar::{
    RevisionReceipt, RevisionRef, SidecarArtifact, SidecarRef, StagedArtifact, StagedRef,
};
use super::verify::admission;
use crate::snapshot::ManifestRef;
use crate::{CatalogError, Snapshot};

const WIRE_VERSION: u32 = 2;

/// Exact immutable receipt pointer; checksums only identify transport bytes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangeSetRef {
    /// Stable identity from the actual typed header row.
    pub change_set_id: SemanticId,
    /// Exact complete encoded receipt.
    pub encoding_checksum: ContentHash,
}
/// Actual already stored typed header, operations and every referenced operation-role row.
#[derive(Clone, Debug)]
pub struct ChangeSetDraft {
    /// Exactly one `authored.change_sets` header.
    pub header: SidecarArtifact,
    /// Complete ordered `authored.change_ops` rows, including an empty batch.
    pub operations: SidecarArtifact,
    /// Exact referenced operation-role inventory, without hidden extras.
    pub staged: BTreeMap<String, StagedArtifact>,
    /// Exact supporting revision rows, including the model revision for a case tip.
    pub supporting_revisions: Vec<RevisionRef>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RevisionBinding {
    revision: RevisionRef,
    manifest: ManifestRef,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ChangeSetWire {
    version: u32,
    change_set_id: SemanticId,
    header: SidecarRef,
    operations: SidecarRef,
    #[serde(deserialize_with = "super::verify::unique_map")]
    staged: BTreeMap<String, StagedRef>,
    supporting_revisions: Vec<RevisionRef>,
    base: Option<RevisionBinding>,
    output: RevisionBinding,
}
/// Physically and structurally admitted receipt retaining every actual referenced row.
/// It proves completeness of the staged references. C2 replays the
/// operation semantics against the exact base; P2 admits the resulting snapshot.
#[derive(Clone, Debug)]
pub struct ChangeSetReceipt {
    reference: ChangeSetRef,
    wire: Arc<ChangeSetWire>,
    draft: Arc<ChangeSetDraft>,
    admission: Arc<crate::store::open::CatalogContext>,
    _metadata: Arc<pse_ids::ReservationLease>,
}
impl ChangeSetReceipt {
    /// Exact immutable receipt reference.
    pub fn reference(&self) -> &ChangeSetRef {
        &self.reference
    }
    /// Actual typed control and operation-role artifacts, suitable for C2 replay.
    pub fn artifacts(&self) -> &ChangeSetDraft {
        &self.draft
    }
    /// Exact immutable base revision and snapshot, absent for first creation.
    pub fn base(&self) -> Option<(&RevisionRef, ManifestRef)> {
        self.wire
            .base
            .as_ref()
            .map(|base| (&base.revision, base.manifest))
    }
    /// Exact resulting revision and snapshot.
    pub fn output(&self) -> (&RevisionRef, ManifestRef) {
        (&self.wire.output.revision, self.wire.output.manifest)
    }
}

impl Catalog {
    /// Persist the complete control envelope after direct typed-reference checks.
    /// Exact source text lives in the typed base/output document relations.
    ///
    /// # Errors
    /// Missing/foreign base, staged rows or source objects; typed receipt mismatches;
    /// resource, cancellation or immutable creation failures.
    async fn publish_change_set_inner(
        &self,
        draft: ChangeSetDraft,
        base: Option<(&RefState, &Snapshot)>,
        output: &RevisionReceipt,
        cancel: &CancellationToken,
    ) -> Result<ChangeSetReceipt, CatalogError> {
        let metadata = metadata::publication(self, &draft, base, output)?;
        let base_binding = base
            .map(|(state, snapshot)| {
                if !Arc::ptr_eq(&state.admission, &self.admission)
                    || !Arc::ptr_eq(&snapshot.admission, &self.admission)
                    || state.manifest_ref() != snapshot.manifest_ref()
                {
                    return Err(admission(
                        "change set",
                        "base must be the exact observed admitted revision",
                    ));
                }
                let revision = state
                    .revision_ref()
                    .ok_or_else(|| admission("change set", "base ref has no typed revision"))?
                    .clone();
                Ok(RevisionBinding {
                    revision,
                    manifest: state.manifest_ref(),
                })
            })
            .transpose()?;
        if !Arc::ptr_eq(&output.artifact.admission, &self.admission)
            || output.reference.change_set.is_some()
        {
            return Err(admission(
                "change set",
                "output must be a fresh admitted revision receipt",
            ));
        }
        let base_id = base_binding
            .as_ref()
            .map_or(SemanticId::NIL, |base| base.revision.revision_id);
        let change_set_id = validate::envelope(self, &draft, base_id)?;
        validate::supporting(self, &draft.supporting_revisions, output, cancel).await?;
        let wire = ChangeSetWire {
            version: WIRE_VERSION,
            change_set_id,
            header: draft.header.reference().clone(),
            operations: draft.operations.reference().clone(),
            staged: draft
                .staged
                .iter()
                .map(|(port, artifact)| (port.clone(), artifact.reference().clone()))
                .collect(),
            supporting_revisions: draft.supporting_revisions.clone(),
            base: base_binding,
            output: RevisionBinding {
                revision: output.reference.clone(),
                manifest: output.target,
            },
        };
        let bytes = super::encode::control(
            &wire,
            self.reserver.as_ref(),
            "store:change-set-receipt",
            self.limits.max_control_bytes,
        )?;
        let checksum = pse_ids::encoding_checksum(&bytes);
        self.ensure_create(&super::layout::change_set_path(&checksum), bytes, cancel)
            .await?;
        let retained = super::control::add(
            metadata::wire_extent(&wire)?,
            metadata::draft_extent(&draft)?,
        )?;
        let metadata = super::control::retain(metadata, retained)?;
        Ok(ChangeSetReceipt {
            reference: ChangeSetRef {
                change_set_id,
                encoding_checksum: checksum.0,
            },
            wire: Arc::new(wire),
            draft: Arc::new(draft),
            admission: Arc::clone(&self.admission),
            _metadata: metadata,
        })
    }

    /// Reopen every exact staged/control object before exposing a receipt.
    /// Full snapshot contexts and operation replay remain explicit separate admissions.
    ///
    /// # Errors
    /// Corrupt objects, unknown encoding, incomplete or mismatched references and budgets.
    async fn read_change_set_inner(
        &self,
        reference: &ChangeSetRef,
        cancel: &CancellationToken,
    ) -> Result<ChangeSetReceipt, CatalogError> {
        let checksum = EncodingChecksum(reference.encoding_checksum);
        let path = super::layout::change_set_path(&checksum);
        let bytes = self
            .read_bytes(&path, self.limits.max_control_bytes, cancel)
            .await?;
        if pse_ids::encoding_checksum(&bytes) != checksum {
            return Err(admission(
                "change set",
                "receipt bytes differ from exact reference",
            ));
        }
        let metadata = super::control::decode_reservation(
            &bytes,
            self.reserver.as_ref(),
            "store:change-set-metadata-read",
        )?;
        let wire: ChangeSetWire = serde_json::from_slice(&bytes)
            .map_err(|error| admission("change set", &error.to_string()))?;
        if wire.version != WIRE_VERSION
            || wire.change_set_id != reference.change_set_id
            || wire.output.revision.change_set.is_some()
        {
            return Err(admission(
                "change set",
                "unsupported receipt version, identity or recursive output reference",
            ));
        }
        let mut staged = BTreeMap::new();
        for (port, reference) in &wire.staged {
            staged.insert(
                port.clone(),
                self.read_staged_batch(reference, cancel).await?,
            );
        }
        let draft = ChangeSetDraft {
            header: self.read_sidecar(&wire.header, cancel).await?,
            operations: self.read_sidecar(&wire.operations, cancel).await?,
            staged,
            supporting_revisions: wire.supporting_revisions.clone(),
        };
        if validate::envelope(
            self,
            &draft,
            wire.base
                .as_ref()
                .map_or(SemanticId::NIL, |base| base.revision.revision_id),
        )? != wire.change_set_id
        {
            return Err(admission(
                "change set",
                "header identity differs from receipt",
            ));
        }
        for binding in wire.base.iter().chain(std::iter::once(&wire.output)) {
            validate::revision_binding(self, binding, cancel).await?;
        }
        for revision in &wire.supporting_revisions {
            self.read_sidecar(&revision.artifact, cancel).await?;
        }
        let retained = super::control::add(
            metadata::wire_extent(&wire)?,
            metadata::draft_extent(&draft)?,
        )?;
        let metadata = super::control::retain(metadata, retained)?;
        Ok(ChangeSetReceipt {
            reference: reference.clone(),
            wire: Arc::new(wire),
            draft: Arc::new(draft),
            admission: Arc::clone(&self.admission),
            _metadata: metadata,
        })
    }

    /// Pair a complete receipt with the exact output revision before the final ref CAS.
    /// # Errors
    /// A foreign receipt or a different actual output revision/manifest.
    pub fn with_change_set(
        &self,
        revision: &RevisionReceipt,
        changes: &ChangeSetReceipt,
    ) -> Result<RevisionReceipt, CatalogError> {
        self.check_change_target(changes, &revision.reference, revision.target)?;
        let mut result = revision.clone();
        result.reference.change_set = Some(changes.reference.clone());
        Ok(result)
    }
    pub(super) async fn check_reopened_change_context(
        &self,
        changes: &ChangeSetReceipt,
        revision: &RevisionReceipt,
        cancel: &CancellationToken,
    ) -> Result<(), CatalogError> {
        self.check_change_target(changes, revision.reference(), revision.target)?;
        validate::supporting(
            self,
            &changes.artifacts().supporting_revisions,
            revision,
            cancel,
        )
        .await
    }
    pub(super) fn check_change_target(
        &self,
        changes: &ChangeSetReceipt,
        revision: &RevisionRef,
        target: ManifestRef,
    ) -> Result<(), CatalogError> {
        let mut bare = revision.clone();
        bare.change_set = None;
        if !Arc::ptr_eq(&changes.admission, &self.admission)
            || changes.wire.output.revision != bare
            || changes.wire.output.manifest != target
        {
            return Err(admission(
                "change set",
                "receipt targets a different exact revision or manifest",
            ));
        }
        Ok(())
    }

    pub(super) fn check_change_base(
        &self,
        changes: &ChangeSetReceipt,
        expected: Option<&RefState>,
    ) -> Result<(), CatalogError> {
        if !Arc::ptr_eq(&changes.admission, &self.admission)
            || expected.is_some_and(|state| !Arc::ptr_eq(&state.admission, &self.admission))
        {
            return Err(admission(
                "change set",
                "foreign receipt or conditional reference",
            ));
        }
        match (&changes.wire.base, expected) {
            (None, None) => Ok(()),
            (Some(base), Some(expected))
                if expected.manifest_ref() == base.manifest
                    && expected.revision_ref() == Some(&base.revision) =>
            {
                Ok(())
            }
            _ => Err(admission(
                "change set",
                "durable receipt base differs from the exact conditional ref precondition",
            )),
        }
    }

    /// Verify receipt completeness against explicitly reopened actual snapshot contexts.
    /// This checks required supporting revision rows; C2 still
    /// owns application/rename replay over the returned operation artifacts.
    /// # Errors
    /// Different/foreign contexts, omitted or changed source edits and revision mismatches.
    async fn validate_change_context_inner(
        &self,
        changes: &ChangeSetReceipt,
        base: Option<&Snapshot>,
        output: &Snapshot,
        cancel: &CancellationToken,
    ) -> Result<(), CatalogError> {
        if !Arc::ptr_eq(&changes.admission, &self.admission)
            || !Arc::ptr_eq(&output.admission, &self.admission)
            || output.manifest_ref() != changes.wire.output.manifest
            || base.map(Snapshot::manifest_ref)
                != changes.wire.base.as_ref().map(|base| base.manifest)
            || base.is_some_and(|base| !Arc::ptr_eq(&base.admission, &self.admission))
        {
            return Err(admission(
                "change set",
                "explicit snapshot contexts differ from durable exact references",
            ));
        }
        let artifact = self
            .read_sidecar(&changes.wire.output.revision.artifact, cancel)
            .await?;
        let revision =
            self.revision_receipt(&artifact, changes.wire.output.revision.revision_id, output)?;
        validate::supporting(self, &changes.wire.supporting_revisions, &revision, cancel).await
    }
}
