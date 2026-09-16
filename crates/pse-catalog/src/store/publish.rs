// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Validate and finish immutable objects before exposing any mutable ref.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use bytes::Bytes;
use datafusion::arrow::array::RecordBatch;
use object_store::path::Path;
use object_store::{PutMode, PutOptions};
use pse_ids::{CancellationToken, CanonicalizeOptions};
use pse_relations::columnar::FieldCheckedBatch;

use super::membership::{self, AdmissionContext};
use super::open::{Catalog, infrastructure};
use super::verify::admission;
use crate::snapshot::ManifestRef;
use crate::{
    CatalogError, EncodingPolicy, EncodingRecord, LoadedRelation, Manifest, RelationContract,
    RelationMember, Snapshot,
};

/// A relation candidate under an exact registry contract. It has no planner constraints
/// until all semantic admission completes.
#[derive(Clone, Debug)]
pub struct RelationDraft {
    /// Exact registry projection; every component is checked directly.
    pub contract: Arc<RelationContract>,
    /// Complete content; empty relations may use an empty batch vector.
    pub batches: Vec<RecordBatch>,
}

/// One complete publication candidate with explicit semantic bindings.
#[derive(Clone, Debug)]
pub struct BundleDraft {
    /// Provenance template. Relation membership must be empty; timestamp, snapshot ID
    /// and relation records are constructed from the admitted content.
    pub manifest: Manifest,
    /// Exactly the declared member ports, including empty relations.
    pub relations: BTreeMap<String, RelationDraft>,
    /// Admitted parents and exact stage producer.
    pub context: AdmissionContext,
}

pub(super) struct PreparedRelation {
    pub(super) relation: LoadedRelation,
    pub(super) objects: Vec<(Path, Bytes)>,
}

/// Snapshot relations use canonical key order; staged operation ordinals address
/// the encoded input order. Both compute identity using the canonical contract.
#[derive(Clone, Copy)]
pub(super) enum RowOrder {
    CanonicalKeys,
    OperationOrdinals,
}

impl Catalog {
    /// Admit actual rows, evaluate registered invariants, canonicalize, finish encodings,
    /// then create immutable objects and the manifest.
    /// Moving a mutable ref is the separate conditional operation over this result.
    ///
    /// # Errors
    /// Contract, semantic, resource, cancellation, corruption or backend failures.
    pub async fn publish_bundle(
        &self,
        draft: BundleDraft,
        cancel: &CancellationToken,
    ) -> Result<Arc<Snapshot>, CatalogError> {
        Ok(self
            .prepare_bundle_publication(draft, cancel)?
            .execute(cancel)
            .await?
            .into_value())
    }

    /// Capture candidate ports, exact declarations and publication policy without writing.
    /// Complete value and cross-port obligations execute in the native publication body.
    /// # Errors
    /// Incompatible declaration/schema, native preparation, policy or resource failure.
    pub fn prepare_bundle_publication(
        &self,
        draft: BundleDraft,
        cancel: &CancellationToken,
    ) -> Result<super::operation::PreparedStoreOperation<Arc<Snapshot>>, CatalogError> {
        use datafusion::common::TableReference;
        use pse_schema::model::provider::{OperationPurpose, ProviderScope};
        let mut session = self.context_session(&draft.context, cancel)?;
        for (port, relation) in &draft.relations {
            cancel.checkpoint()?;
            self.bind_draft(
                &mut session,
                TableReference::full("publication", "inputs", port.clone()),
                relation,
            )?;
        }
        self.prepare_store_operation_in(
            &session,
            crate::store::operation::StoreCommand {
                name: "store.publish_bundle",
                scope: ProviderScope::Schema("store".into(), "manifests".into()),
                purpose: OperationPurpose::Publish,
                arguments: vec![datafusion::logical_expr::lit(
                    draft.manifest.snapshot_kind.as_str(),
                )],
            },
            Box::new(move |catalog, _session, cancel| {
                Box::pin(async move {
                    let snapshot = catalog.publish_bundle_inner(draft, &cancel).await?;
                    let count = u64::try_from(snapshot.relations().len())
                        .map_err(|_| super::encode::overflow())?;
                    Ok((snapshot, count))
                })
            }),
            cancel,
        )
    }

    async fn publish_bundle_inner(
        &self,
        draft: BundleDraft,
        cancel: &CancellationToken,
    ) -> Result<Arc<Snapshot>, CatalogError> {
        cancel.checkpoint()?;
        let inventory = membership::inventory(
            &self.registry,
            &draft.manifest,
            &draft.context,
            &self.admission,
        )?;
        if draft
            .relations
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>()
            != inventory.keys().map(String::as_str).collect()
        {
            return Err(membership::refused(
                "candidate does not contain every declared output port exactly once",
            ));
        }
        let mut candidates = BTreeMap::new();
        for (port, relation) in &draft.relations {
            let spec = inventory[port];
            relation
                .contract
                .validate_against_registry(&self.registry, spec)?;
            let batch = combine(self, relation, spec, cancel)?;
            candidates.insert(port.clone(), batch);
        }
        let raw = candidates
            .iter()
            .map(|(port, batch)| (port.clone(), batch.batch().clone()))
            .collect();
        self.admit_ports(
            draft.manifest.snapshot_kind,
            &candidates,
            &inventory,
            &draft.context,
            cancel,
        )
        .await?;
        self.admit_stage(&raw, &draft.context, cancel).await?;
        self.publish_admitted_bundle(draft, candidates, cancel)
            .await
    }

    // Only admission and the registered producer executor can reach this boundary.
    pub(crate) async fn publish_admitted_bundle(
        &self,
        draft: BundleDraft,
        candidates: BTreeMap<String, FieldCheckedBatch>,
        cancel: &CancellationToken,
    ) -> Result<Arc<Snapshot>, CatalogError> {
        cancel.checkpoint()?;
        let metadata = self.publication_metadata(&draft)?;
        let inventory = membership::inventory(
            &self.registry,
            &draft.manifest,
            &draft.context,
            &self.admission,
        )?;
        let mut manifest = draft.manifest;
        manifest.created_at = self.clock.now_rfc3339_utc();
        manifest.schema_registry_fingerprint = self.registry.fingerprint();
        let mut objects = Vec::new();
        let mut loaded = BTreeMap::new();
        for (port, relation) in draft.relations {
            let spec = inventory[&port];
            let prepared = self.prepare_relation(
                port.clone(),
                relation,
                spec,
                &candidates[&port],
                RowOrder::CanonicalKeys,
                cancel,
            )?;
            manifest
                .relations
                .push(prepared.relation.member.as_ref().clone());
            let member = &prepared.relation.member;
            loaded.insert(member.port.clone(), Arc::new(prepared.relation));
            objects.extend(prepared.objects);
        }
        manifest.snapshot_id = pse_ids::snapshot_id(&manifest.frame())?;
        manifest.admission_binding = self
            .publish_admission_binding(&draft.context, cancel)
            .await?
            .map(|encoding_checksum| super::manifest::AdmissionBindingRef { encoding_checksum });
        manifest.validate(self.registry.fingerprint())?;
        let bytes = super::encode::control(
            &manifest,
            self.reserver.as_ref(),
            "store:manifest-encode",
            self.limits.max_control_bytes,
        )?;
        let checksum = pse_ids::encoding_checksum(&bytes);
        // No visibility change occurs until every object below exists. Existing objects
        // are compared against the complete intended finished bytes, never just names.
        for (path, bytes) in objects {
            self.ensure_create(&path, bytes, cancel).await?;
        }
        self.ensure_create(&super::layout::manifest_path(&checksum), bytes, cancel)
            .await?;
        let reference = ManifestRef {
            snapshot_id: manifest.snapshot_id,
            manifest_checksum: checksum,
        };
        let metadata =
            super::control::retain(metadata, super::control::snapshot_extent(&manifest)?)?;
        Ok(Arc::new(Snapshot {
            manifest: super::control::OwnedControl::new(manifest, metadata),
            manifest_ref: reference,
            relations: Arc::new(loaded),
            parents: Arc::new(draft.context.parents),
            stage_pass: draft.context.stage_pass,
            invocation: draft.context.invocation,
            admission: Arc::clone(&self.admission),
        }))
    }

    fn publication_metadata(
        &self,
        draft: &BundleDraft,
    ) -> Result<Box<dyn pse_ids::Reservation>, CatalogError> {
        if !draft.manifest.relations.is_empty() {
            return Err(admission(
                "publication template",
                "relation claims must be empty; membership is derived from actual admitted rows",
            ));
        }
        if draft.manifest.schema_registry_fingerprint != self.registry.fingerprint() {
            return Err(admission(
                "publication template",
                "registry fingerprint differs from the bound registry",
            ));
        }
        let mut metadata = self.reserver.open("store:publication-metadata");
        let forecast = draft.relations.iter().try_fold(
            super::control::snapshot_extent(&draft.manifest)?,
            |bytes, (port, relation)| {
                let names =
                    super::control::add(port.capacity(), relation.contract.name.capacity())?;
                let names = super::control::add(names, relation.contract.namespace.capacity())?;
                super::control::add(
                    bytes,
                    super::control::add(super::control::mul(names, 8)?, 8192)?,
                )
            },
        )?;
        metadata.try_grow(forecast)?;
        Ok(metadata)
    }

    pub(super) fn prepare_relation(
        &self,
        port: String,
        relation: RelationDraft,
        spec: &pse_schema::model::RelationSpec,
        candidate: &FieldCheckedBatch,
        order: RowOrder,
        cancel: &CancellationToken,
    ) -> Result<PreparedRelation, CatalogError> {
        let mut metadata = self.reserver.open("store:relation-metadata");
        let forecast = super::control::add(port.capacity(), spec.key.name.len())?;
        metadata.try_grow(super::control::add(
            super::control::mul(forecast, 8)?,
            8192,
        )?)?;
        let mut objects = Vec::new();
        cancel.checkpoint()?;
        let (sorted, canonical) = candidate.canonicalize(
            &self.registry,
            spec,
            self.reserver.as_ref(),
            CanonicalizeOptions {
                keep_sorted: true,
                keep_preimage: false,
                envelope: self.limits.envelope,
                cancel: Some(cancel.clone()),
            },
        )?;
        let stored = match order {
            RowOrder::CanonicalKeys => sorted,
            RowOrder::OperationOrdinals => candidate.clone(),
        };
        let mut encodings = vec![super::encode::ipc_file(
            stored.batch(),
            self.reserver.as_ref(),
            cancel,
        )?];
        if relation.contract.encodings == EncodingPolicy::IpcFileAndParquet {
            encodings.push(super::encode::parquet_file(
                stored.batch(),
                self.reserver.as_ref(),
                cancel,
            )?);
        }
        let mut records = Vec::new();
        for encoded in encodings {
            // The native writer consumes the admitted Arrow values in the artifact order.
            // Decode/admission belongs to external reopen, not a local replay.
            let path = super::layout::relation_path(
                spec.key.namespace.as_str(),
                spec.key.name,
                relation.contract.canonical.schema_version,
                &encoded.checksum,
                encoded.format,
            );
            records.push(EncodingRecord {
                format: encoded.format,
                writer_version: encoded.writer_version,
                encoding_checksum: encoded.checksum,
                bytes: u64::try_from(encoded.bytes.len()).map_err(|_| super::encode::overflow())?,
                path: path.to_string(),
            });
            objects.push((path, encoded.bytes));
        }
        let member = RelationMember {
            port,
            namespace: spec.key.namespace.as_str().to_owned(),
            relation_id: spec.id,
            name: spec.key.name.to_owned(),
            version: relation.contract.canonical.schema_version,
            logical_hash: canonical.logical_hash,
            rows: canonical.row_count,
            encodings: records,
        };
        let member = super::control::own_member(member, metadata)?;
        Ok(PreparedRelation {
            relation: LoadedRelation {
                contract: relation.contract,
                batch: stored,
                member,
            },
            objects,
        })
    }

    pub(crate) async fn ensure_create(
        &self,
        path: &Path,
        bytes: Bytes,
        cancel: &CancellationToken,
    ) -> Result<(), CatalogError> {
        cancel.checkpoint()?;
        if bytes.len() > self.limits.max_object_bytes {
            return Err(admission(
                path.as_ref(),
                "encoded object exceeds supported extent",
            ));
        }
        let outcome = cancel
            .until_cancelled(self.store.put_opts(
                path,
                bytes.clone().into(),
                PutOptions {
                    mode: PutMode::Create,
                    ..Default::default()
                },
            ))
            .await?;
        cancel.checkpoint()?;
        match outcome {
            Ok(_) => self.synchronize_immutable(path, cancel).await,
            Err(object_store::Error::AlreadyExists { .. }) => {
                let existing = self.read_bytes(path, bytes.len(), cancel).await?;
                if existing.as_ref() != bytes.as_ref() {
                    return Err(admission(
                        path.as_ref(),
                        "existing immutable object differs from the intended admitted finished bytes",
                    ));
                }
                self.synchronize_immutable(path, cancel).await
            }
            Err(source) => Err(infrastructure("create immutable object", source)),
        }
    }
}

pub(super) fn combine(
    catalog: &Catalog,
    draft: &RelationDraft,
    spec: &pse_schema::model::RelationSpec,
    cancel: &CancellationToken,
) -> Result<FieldCheckedBatch, CatalogError> {
    let mut extent = 4096usize;
    for batch in &draft.batches {
        let concat = pse_ids::owned_buffer::retained_buffer_bytes(batch)?
            .checked_mul(2)
            .ok_or_else(super::encode::overflow)?;
        extent = extent
            .checked_add(membership::validation_extent(batch)?)
            .and_then(|value| value.checked_add(concat))
            .ok_or_else(super::encode::overflow)?;
    }
    let mut reservation = catalog.reserver.open("store:candidate-concat");
    // validation_extent already includes temporary value/index copies. Concat adds
    // one output allocation and a growth margin, not three simultaneous validators.
    reservation.try_grow(extent)?;
    let mut inputs = Vec::with_capacity(draft.batches.len());
    for batch in &draft.batches {
        cancel.checkpoint()?;
        inputs.push(FieldCheckedBatch::admit(
            &catalog.registry,
            spec,
            batch.clone(),
        )?);
    }
    let batch = FieldCheckedBatch::concat(&catalog.registry, spec, &inputs)?;
    let retained = pse_ids::owned_buffer::retained_buffer_bytes(batch.batch())?;
    reservation.shrink(reservation.size().saturating_sub(retained));
    // Retain the native concat allocation in the checked owner.
    let batch = batch.retained(catalog.reserver.as_ref(), cancel)?;
    drop(reservation);
    Ok(batch)
}
