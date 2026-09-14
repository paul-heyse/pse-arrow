// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! A catalog pins exact manifests, fully admits their rows and retains their owners.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use bytes::Bytes;
use datafusion::arrow::array::RecordBatch;
use object_store::path::Path;
use object_store::{GetResult, ObjectMeta, ObjectStore, ObjectStoreExt};
use pse_ids::{CancellationToken, CanonicalizeOptions, Envelope, MemoryReserver, ReservationLease};
use pse_schema::Registry;
use pse_schema::model::RelationKey;

use super::manifest::Manifest;
use super::membership::{self, AdmissionContext, SemanticValidator};
use super::verify::admission;
use crate::snapshot::ManifestRef;
use crate::{
    CatalogError, Clock, EncodingFormat, EncodingPolicy, LoadedRelation, RelationContract,
    Snapshot, TrustLevel,
};

type LoadedCandidates = BTreeMap<String, LoadedRelation>;
type AlternateEncodings = BTreeMap<String, Vec<RecordBatch>>;

/// Finite bounds for supported object and control encodings. Relation construction has
/// the separately declared canonical envelope; deployments may lower all three.
#[derive(Clone, Copy, Debug)]
pub struct StoreLimits {
    /// Largest accepted physical object, checked against actual bytes before decode.
    pub max_object_bytes: usize,
    /// Largest accepted JSON manifest, ref or stage sidecar.
    pub max_control_bytes: usize,
    /// Declared supported row and normalized-buffer envelope.
    pub envelope: Envelope,
}
impl Default for StoreLimits {
    fn default() -> Self {
        Self {
            max_object_bytes: 512 << 20,
            max_control_bytes: 16 << 20,
            envelope: Envelope::default(),
        }
    }
}

/// One object-store catalog bound to an exact registry and one shared memory budget.
/// Every open path admits actual content; `TrustLevel` records provenance and never
/// disables semantic validation.
#[derive(Debug)]
pub struct Catalog {
    pub(crate) store: Arc<dyn ObjectStore>,
    pub(crate) registry: Arc<Registry>,
    pub(crate) clock: Arc<dyn Clock>,
    pub(crate) reserver: Arc<dyn MemoryReserver>,
    pub(crate) limits: StoreLimits,
    pub(crate) admission: Arc<()>,
    pub(crate) local: Option<Arc<super::local::LocalFiles>>,
    trust: TrustLevel,
    pub(crate) validator: Option<Arc<dyn SemanticValidator>>,
}
impl Catalog {
    /// Construct publication provenance from the pinned build and explicit admitted
    /// parent roles. Relation claims and snapshot identity remain empty until admission.
    /// Engine/policy selections, package inventory and kernels are supplied by the driver.
    ///
    /// # Errors
    /// Foreign/missing parent bindings or an unknown registered producer.
    pub fn manifest_template(
        &self,
        kind: pse_ids::SnapshotKind,
        context: &AdmissionContext,
    ) -> Result<Manifest, CatalogError> {
        let passes = context
            .stage_pass
            .map(|id| {
                self.registry
                    .passes()
                    .iter()
                    .find(|pass| pass.id == id)
                    .ok_or_else(|| {
                        admission("manifest template", "producer pass is not registered")
                    })
            })
            .transpose()?
            .into_iter()
            .map(|pass| crate::PassRef {
                pass_id: pass.id,
                version: pass.version.to_owned(),
            })
            .collect();
        let manifest = Manifest {
            manifest_version: crate::MANIFEST_VERSION.to_owned(),
            snapshot_kind: kind,
            snapshot_id: pse_ids::SnapshotId(pse_ids::ContentHash::NIL),
            membership_profile: pse_ids::SNAPSHOT_PROFILE.to_owned(),
            created_at: String::new(),
            schema_registry_fingerprint: self.registry.fingerprint(),
            relations: Vec::new(),
            packages: Vec::new(),
            compiler: crate::CompilerRef {
                version: pse_buildinfo::VERSION.to_owned(),
                passes,
            },
            engine_profile: None,
            numerical_policy: None,
            toolchain: crate::ToolchainRef {
                lockfile_hash: pse_ids::encoding_checksum(pse_buildinfo::CARGO_LOCK).content_hash(),
                canonicalization: pse_ids::CANON_VERSION.to_owned(),
            },
            kernels: Vec::new(),
            semantic_parents: context
                .parents
                .iter()
                .map(|(role, parent)| pse_ids::SnapshotParent {
                    role: role.clone(),
                    snapshot_id: parent.snapshot_id(),
                })
                .collect(),
            evidence: Vec::new(),
        };
        membership::inventory(&self.registry, &manifest, context, &self.admission)?;
        Ok(manifest)
    }
    /// Bind a catalog without reading any object or manufacturing a snapshot handle.
    pub fn open(
        store: Arc<dyn ObjectStore>,
        registry: Arc<Registry>,
        trust: TrustLevel,
        clock: Arc<dyn Clock>,
        reserver: Arc<dyn MemoryReserver>,
    ) -> Self {
        Self {
            store,
            registry,
            trust,
            validator: None,
            clock,
            reserver,
            limits: StoreLimits::default(),
            admission: Arc::new(()),
            local: None,
        }
    }
    /// Tighten the supported read/construction envelope.
    ///
    /// # Errors
    /// Zero object/control bounds or a raised supported envelope.
    pub fn with_limits(mut self, limits: StoreLimits) -> Result<Self, CatalogError> {
        let supported = StoreLimits::default();
        if limits.max_object_bytes == 0
            || limits.max_control_bytes == 0
            || limits.max_object_bytes > supported.max_object_bytes
            || limits.max_control_bytes > supported.max_control_bytes
        {
            return Err(admission(
                "store limits",
                "object/control bounds must be positive and may only lower the supported envelope",
            ));
        }
        Envelope::lowered(
            limits.envelope.max_rows,
            limits.envelope.max_normalized_bytes,
        )?;
        self.limits = limits;
        Ok(self)
    }
    /// Install the rule layer that executes the registry's semantic invariants.
    #[must_use]
    pub fn with_semantic_validator(mut self, validator: Arc<dyn SemanticValidator>) -> Self {
        self.validator = Some(validator);
        self
    }
    pub(crate) async fn admit_content(
        &self,
        kind: pse_ids::SnapshotKind,
        candidates: &BTreeMap<RelationKey, RecordBatch>,
        context: &AdmissionContext,
        cancel: &CancellationToken,
    ) -> Result<(), CatalogError> {
        let mut reservation = self.reserver.open("store:semantic-admission");
        reservation.try_grow(membership::context_validation_extent(candidates, context)?)?;
        let rows = membership::validate_content(&self.registry, candidates, context)?;
        self.admit_documents(&rows, cancel).await?;
        if let Some(validator) = &self.validator {
            validator.validate(&self.registry, &rows, cancel).await?;
        } else if self.registry.invariants().iter().any(|invariant| {
            rows.keys()
                .any(|key| key.qualified_name() == invariant.relation)
        }) {
            return Err(admission(
                "semantic validator",
                "registered invariants require the executable rule validator before snapshot admission",
            ));
        }
        if matches!(
            kind,
            pse_ids::SnapshotKind::Model | pse_ids::SnapshotKind::Case
        ) && rows.iter().any(|(key, batch)| {
            key.qualified_name() == "authored.documents" && batch.num_rows() != 0
        }) {
            let validator = self.validator.as_ref().ok_or_else(|| {
                membership::refused(
                    "source-bearing snapshot requires executable document-to-row correspondence validation",
                )
            })?;
            cancel.checkpoint()?;
            validator
                .validate_snapshot_sources(self, kind, context, candidates, cancel)
                .await?;
            cancel.checkpoint()?;
        }
        Ok(())
    }

    pub(crate) async fn admit_stage(
        &self,
        candidates: &BTreeMap<String, RecordBatch>,
        context: &AdmissionContext,
        cancel: &CancellationToken,
    ) -> Result<(), CatalogError> {
        if context.stage_pass.is_some()
            && let Some(validator) = &self.validator
        {
            validator
                .validate_stage(self, context, candidates, cancel)
                .await?;
        }
        Ok(())
    }

    pub(crate) async fn admit_ports(
        &self,
        kind: pse_ids::SnapshotKind,
        candidates: &BTreeMap<String, RecordBatch>,
        inventory: &BTreeMap<String, &pse_schema::model::RelationSpec>,
        context: &AdmissionContext,
        cancel: &CancellationToken,
    ) -> Result<(), CatalogError> {
        let mut groups: BTreeMap<RelationKey, Vec<&RecordBatch>> = BTreeMap::new();
        for (port, batch) in candidates {
            groups.entry(inventory[port].key).or_default().push(batch);
        }
        let repeated: BTreeSet<_> = groups
            .iter()
            .filter(|(_, rows)| rows.len() > 1)
            .map(|(key, _)| key.qualified_name())
            .collect();
        if !repeated.is_empty() {
            let ambiguous_fk = inventory.values().any(|spec| {
                spec.columns
                    .iter()
                    .any(|column| column.fk.is_some_and(|fk| repeated.contains(fk.relation)))
            });
            let ambiguous_invariant = self.registry.invariants().iter().any(|invariant| {
                repeated.contains(&invariant.relation)
                    || self.registry.rule(&invariant.rule).is_some_and(|rule| {
                        self.registry.rule_dependencies().iter().any(|dependency| {
                            dependency.rule_id == rule.id && repeated.contains(&dependency.relation)
                        })
                    })
            });
            if ambiguous_fk || ambiguous_invariant {
                return Err(membership::refused(
                    "repeated output schemas have an ambiguous relation-based FK or invariant context; explicit producer-port semantics are required",
                ));
            }
        }
        let mut unique: BTreeMap<_, _> = groups
            .iter()
            .filter(|(_, rows)| rows.len() == 1)
            .map(|(key, rows)| (*key, rows[0].clone()))
            .collect();
        if repeated.is_empty() {
            return self.admit_content(kind, &unique, context, cancel).await;
        }
        // Each output is a complete independently keyed relation. Never merge, choose
        // or let equal schemas collapse the actual registered output ports.
        for (key, rows) in groups.iter().filter(|(_, rows)| rows.len() > 1) {
            for batch in rows {
                unique.insert(*key, (*batch).clone());
                self.admit_content(kind, &unique, context, cancel).await?;
                unique.remove(key);
            }
        }
        Ok(())
    }
    /// Registry declarations bound to every admitted relation.
    pub fn registry(&self) -> &Arc<Registry> {
        &self.registry
    }
    /// Storage provenance; all handles still require semantic admission.
    pub const fn trust_level(&self) -> TrustLevel {
        self.trust
    }
    /// The common platform reservation adapter.
    pub fn reserver(&self) -> &Arc<dyn MemoryReserver> {
        &self.reserver
    }

    /// Open an exact immutable manifest with explicit admitted semantic parents.
    ///
    /// # Errors
    /// Transport, framing, contract, membership, semantic, identity and resource errors.
    pub async fn read_manifest(
        &self,
        reference: ManifestRef,
        context: &AdmissionContext,
        cancel: &CancellationToken,
    ) -> Result<Arc<Snapshot>, CatalogError> {
        let path = super::layout::manifest_path(&reference.manifest_checksum);
        let bytes = self
            .read_bytes(&path, self.limits.max_control_bytes, cancel)
            .await?;
        let actual = pse_ids::encoding_checksum(&bytes);
        if actual != reference.manifest_checksum {
            return Err(CatalogError::CorruptObject {
                path: path.to_string(),
                expected: reference.manifest_checksum.to_string(),
                actual: actual.to_string(),
            });
        }
        let metadata = super::control::decode_reservation(
            &bytes,
            self.reserver.as_ref(),
            "store:manifest-metadata",
        )?;
        let manifest = Manifest::decode(&bytes)?;
        let metadata =
            super::control::retain(metadata, super::control::snapshot_extent(&manifest)?)?;
        manifest.validate(self.registry.fingerprint())?;
        if manifest.snapshot_id != reference.snapshot_id {
            return Err(membership::refused(
                "exact manifest reference names the wrong snapshot",
            ));
        }
        let inventory = membership::inventory(&self.registry, &manifest, context, &self.admission)?;
        let actual_ports = manifest
            .relations
            .iter()
            .map(|member| member.port.as_str())
            .collect::<BTreeSet<_>>();
        if actual_ports != inventory.keys().map(String::as_str).collect() {
            return Err(membership::refused(
                "manifest does not contain exactly the declared ports, including empty relations",
            ));
        }
        let (candidates, loaded, alternatives) = self
            .load_members(&manifest, &inventory, &metadata, cancel)
            .await?;
        self.admit_ports(
            manifest.snapshot_kind,
            &candidates,
            &inventory,
            context,
            cancel,
        )
        .await?;
        self.admit_stage(&candidates, context, cancel).await?;
        let loaded = self.finalize_loaded(loaded, alternatives, cancel)?;
        Ok(Arc::new(Snapshot {
            manifest: super::control::OwnedControl::new(manifest, metadata),
            manifest_ref: reference,
            relations: Arc::new(
                loaded
                    .into_iter()
                    .map(|(key, relation)| (key, Arc::new(relation)))
                    .collect(),
            ),
            parents: Arc::new(context.parents.clone()),
            stage_pass: context.stage_pass,
            admission: Arc::clone(&self.admission),
        }))
    }

    async fn load_members(
        &self,
        manifest: &Manifest,
        inventory: &BTreeMap<String, &pse_schema::model::RelationSpec>,
        metadata: &Arc<ReservationLease>,
        cancel: &CancellationToken,
    ) -> Result<
        (
            BTreeMap<String, RecordBatch>,
            LoadedCandidates,
            AlternateEncodings,
        ),
        CatalogError,
    > {
        let mut candidates = BTreeMap::new();
        let mut loaded = BTreeMap::new();
        let mut alternatives = BTreeMap::new();
        for member in &manifest.relations {
            cancel.checkpoint()?;
            let spec = inventory[&member.port];
            if member.relation_id != spec.id
                || member.namespace != spec.key.namespace.as_str()
                || member.name != spec.key.name
                || member.version.0 != spec.key.version
            {
                return Err(admission(
                    &member.port,
                    "member identity/schema differs from its actual registered declaration",
                ));
            }
            let contract = Arc::new(RelationContract::from_spec(
                &self.registry,
                spec,
                if member
                    .encodings
                    .iter()
                    .any(|encoding| encoding.format == EncodingFormat::Parquet)
                {
                    EncodingPolicy::IpcFileAndParquet
                } else {
                    EncodingPolicy::IpcFile
                },
            )?);
            if !member
                .encodings
                .iter()
                .any(|encoding| encoding.format == EncodingFormat::ArrowIpcFile)
            {
                return Err(admission(
                    &member.port,
                    "an IPC encoding is required by the supported catalog policy",
                ));
            }
            let mut batches = Vec::new();
            for encoding in &member.encodings {
                let bytes = self
                    .read_bytes(
                        &Path::from(encoding.path.as_str()),
                        self.limits.max_object_bytes,
                        cancel,
                    )
                    .await?;
                super::verify::encoding(&bytes, encoding)?;
                let decoded = super::verify::decode_file(
                    encoding.format,
                    &bytes,
                    &self.registry,
                    spec,
                    self.reserver.as_ref(),
                    cancel,
                    self.limits.envelope,
                )?;
                if u64::try_from(decoded.num_rows()).ok() != Some(member.rows) {
                    return Err(admission(
                        &member.port,
                        "decoded row count differs from manifest",
                    ));
                }
                batches.push(decoded);
            }
            let batch = batches
                .pop()
                .ok_or_else(|| admission(&member.port, "no supported encoding"))?;
            alternatives.insert(member.port.clone(), batches);
            candidates.insert(member.port.clone(), batch.clone());
            loaded.insert(
                member.port.clone(),
                LoadedRelation {
                    contract,
                    batch,
                    member: super::control::OwnedControl::new(member.clone(), Arc::clone(metadata)),
                },
            );
        }
        Ok((candidates, loaded, alternatives))
    }

    fn finalize_loaded(
        &self,
        mut loaded: LoadedCandidates,
        mut alternatives: AlternateEncodings,
        cancel: &CancellationToken,
    ) -> Result<LoadedCandidates, CatalogError> {
        for relation in loaded.values_mut() {
            let canonical = pse_ids::canonicalize(
                &relation.contract.canonical,
                std::slice::from_ref(&relation.batch),
                self.reserver.as_ref(),
                CanonicalizeOptions {
                    keep_sorted: true,
                    keep_preimage: true,
                    envelope: self.limits.envelope,
                    cancel: Some(cancel.clone()),
                },
            )?;
            if canonical.logical_hash != relation.member.logical_hash {
                return Err(CatalogError::LogicalHashMismatch {
                    relation: relation.member.port.clone(),
                    expected: relation.member.logical_hash,
                    actual: canonical.logical_hash,
                });
            }
            for other in alternatives
                .remove(&relation.member.port)
                .unwrap_or_default()
            {
                let equivalent = pse_ids::canonicalize(
                    &relation.contract.canonical,
                    &[other],
                    self.reserver.as_ref(),
                    CanonicalizeOptions {
                        keep_preimage: true,
                        envelope: self.limits.envelope,
                        cancel: Some(cancel.clone()),
                        ..Default::default()
                    },
                )?;
                if equivalent.preimage != canonical.preimage {
                    return Err(admission(
                        &relation.member.port,
                        "encodings disagree on actual normalized logical values",
                    ));
                }
            }
            relation.batch = canonical.sorted.ok_or_else(|| CatalogError::Internal {
                message: "requested canonical sorted output is absent".to_owned(),
            })?;
        }
        Ok(loaded)
    }

    pub(crate) async fn read_bytes(
        &self,
        path: &Path,
        limit: usize,
        cancel: &CancellationToken,
    ) -> Result<Bytes, CatalogError> {
        Ok(self.read_versioned(path, limit, cancel).await?.0)
    }
    pub(crate) async fn read_versioned(
        &self,
        path: &Path,
        limit: usize,
        cancel: &CancellationToken,
    ) -> Result<(Bytes, ObjectMeta), CatalogError> {
        cancel.checkpoint()?;
        let result = self
            .store
            .get(path)
            .await
            .map_err(|source| infrastructure("read object", source))?;
        self.collect_object(result, limit, cancel).await
    }
    async fn collect_object(
        &self,
        result: GetResult,
        limit: usize,
        cancel: &CancellationToken,
    ) -> Result<(Bytes, ObjectMeta), CatalogError> {
        let size = usize::try_from(result.meta.size)
            .map_err(|_| admission("object", "size exceeds addressable memory"))?;
        if size > limit || result.range != (0..result.meta.size) {
            return Err(admission(
                "object",
                "actual object/range exceeds the finite supported read extent",
            ));
        }
        let mut reservation = self.reserver.open("store:object-read");
        // Destination and one backend-owned incoming object/chunk coexist. The backend
        // owns allocation before yielding; the platform never grows its destination
        // beyond the observed object extent.
        reservation.try_grow(size.checked_mul(2).ok_or_else(super::encode::overflow)?)?;
        let meta = result.meta.clone();
        let mut output = Vec::with_capacity(size);
        let mut stream = result.into_stream();
        while let Some(chunk) = std::future::poll_fn(|cx| stream.as_mut().poll_next(cx)).await {
            cancel.checkpoint()?;
            let chunk = chunk.map_err(|source| infrastructure("read object chunk", source))?;
            if chunk.len() > size.saturating_sub(output.len()) {
                return Err(admission(
                    "object",
                    "stream exceeds actual advertised object extent",
                ));
            }
            output.extend_from_slice(&chunk);
        }
        if output.len() != size {
            return Err(admission("object", "truncated object stream"));
        }
        drop(stream);
        reservation.shrink(reservation.size().saturating_sub(output.capacity()));
        Ok((
            pse_ids::owned_buffer::attach_bytes(output, ReservationLease::new(reservation))?,
            meta,
        ))
    }
}
pub(crate) fn infrastructure(op: &str, source: object_store::Error) -> CatalogError {
    CatalogError::Infrastructure {
        op: op.to_owned(),
        source: Box::new(source),
    }
}
