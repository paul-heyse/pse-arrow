// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Validate and finish immutable objects before exposing any mutable ref.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use bytes::Bytes;
use datafusion::arrow::array::RecordBatch;
use object_store::path::Path;
use object_store::{PutMode, PutOptions};
use pse_ids::{CancellationToken, CanonicalizeOptions, ReservationLease};

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

impl Catalog {
    /// Admit actual rows, evaluate registered invariants, canonicalize, finish encodings,
    /// verify their decoded values, then create immutable objects and the manifest.
    /// Moving a mutable ref is the separate conditional operation over this result.
    ///
    /// # Errors
    /// Contract, semantic, resource, cancellation, corruption or backend failures.
    pub async fn publish_bundle(
        &self,
        draft: BundleDraft,
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
        self.admit_ports(
            draft.manifest.snapshot_kind,
            &candidates,
            &inventory,
            &draft.context,
            cancel,
        )
        .await?;
        self.admit_stage(&candidates, &draft.context, cancel)
            .await?;
        let mut manifest = draft.manifest;
        manifest.created_at = self.clock.now_rfc3339_utc();
        manifest.schema_registry_fingerprint = self.registry.fingerprint();
        let mut objects = Vec::new();
        let mut loaded = BTreeMap::new();
        for (port, relation) in draft.relations {
            let spec = inventory[&port];
            let prepared =
                self.prepare_relation(port.clone(), relation, spec, &candidates[&port], cancel)?;
            manifest
                .relations
                .push(prepared.relation.member.as_ref().clone());
            let member = &prepared.relation.member;
            loaded.insert(member.port.clone(), Arc::new(prepared.relation));
            objects.extend(prepared.objects);
        }
        manifest.snapshot_id = pse_ids::snapshot_id(&manifest.frame())?;
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
        candidate: &RecordBatch,
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
        let canonical = pse_ids::canonicalize(
            &relation.contract.canonical,
            std::slice::from_ref(candidate),
            self.reserver.as_ref(),
            CanonicalizeOptions {
                keep_sorted: true,
                keep_preimage: true,
                envelope: self.limits.envelope,
                cancel: Some(cancel.clone()),
            },
        )?;
        let sorted = canonical
            .sorted
            .ok_or_else(|| admission(&port, "canonical sorted content is absent"))?;
        let mut encodings = vec![super::encode::ipc_file(
            &sorted,
            self.reserver.as_ref(),
            cancel,
        )?];
        if relation.contract.encodings == EncodingPolicy::IpcFileAndParquet {
            encodings.push(super::encode::parquet_file(
                &sorted,
                self.reserver.as_ref(),
                cancel,
            )?);
        }
        let mut records = Vec::new();
        for encoded in encodings {
            let decoded = super::verify::decode_file(
                encoded.format,
                &encoded.bytes,
                &self.registry,
                spec,
                self.reserver.as_ref(),
                cancel,
                self.limits.envelope,
            )?;
            let roundtrip = pse_ids::canonicalize(
                &relation.contract.canonical,
                &[decoded],
                self.reserver.as_ref(),
                CanonicalizeOptions {
                    keep_preimage: true,
                    envelope: self.limits.envelope,
                    cancel: Some(cancel.clone()),
                    ..Default::default()
                },
            )?;
            if canonical.preimage != roundtrip.preimage {
                return Err(admission(
                    &port,
                    "finished encoding changes actual normalized logical values",
                ));
            }
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
                batch: sorted,
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
        let outcome = self
            .store
            .put_opts(
                path,
                bytes.clone().into(),
                PutOptions {
                    mode: PutMode::Create,
                    ..Default::default()
                },
            )
            .await;
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
) -> Result<RecordBatch, CatalogError> {
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
    for batch in &draft.batches {
        cancel.checkpoint()?;
        pse_relations::validate::validate_batch(&catalog.registry, spec, batch).map_err(
            |errors| {
                admission(
                    &spec.key.to_string(),
                    &errors
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join("; "),
                )
            },
        )?;
    }
    let batch = datafusion::arrow::compute::concat_batches(
        &draft.contract.canonical.schema,
        &draft.batches,
    )
    .map_err(super::encode::arrow)?;
    let retained = pse_ids::owned_buffer::retained_buffer_bytes(&batch)?;
    reservation.shrink(reservation.size().saturating_sub(retained));
    Ok(pse_ids::owned_buffer::attach_reservation(
        batch,
        ReservationLease::new(reservation),
    )?)
}
