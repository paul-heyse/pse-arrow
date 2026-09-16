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
use pse_relations::columnar::FieldCheckedBatch;
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
type AlternateEncodings = BTreeMap<String, Vec<FieldCheckedBatch>>;

/// Selected finite limits for object, control and relation construction. Deployments
/// may raise or lower these policies within the actual representation limits.
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
            max_object_bytes: usize::try_from(32_u64 << 30).unwrap_or(isize::MAX as usize),
            max_control_bytes: 1 << 30,
            envelope: Envelope::default(),
        }
    }
}
impl StoreLimits {
    /// Check selected policies against actual addressable and canonical representation.
    ///
    /// # Errors
    /// Zero physical/control limits or bounds outside the platform representation.
    pub fn validate(self) -> Result<(), CatalogError> {
        if self.max_object_bytes == 0
            || self.max_control_bytes == 0
            || isize::try_from(self.max_object_bytes).is_err()
            || isize::try_from(self.max_control_bytes).is_err()
        {
            return Err(admission(
                "store limits",
                "object/control bounds must be positive and fit the platform allocation envelope",
            ));
        }
        Envelope::new(self.envelope.max_rows, self.envelope.max_normalized_bytes)?;
        Ok(())
    }
}

/// One object-store catalog bound to an exact registry and one shared memory budget.
/// Every open path admits actual content; `TrustLevel` records provenance and never
/// disables semantic validation.
#[derive(Debug, Clone)]
pub struct Catalog {
    pub(crate) admission: Arc<CatalogContext>,
    pub(crate) reserver: Arc<dyn MemoryReserver>,
    pub(crate) local: Option<Arc<super::local::LocalFiles>>,
    pub(crate) publication: Option<super::publication::PublicationJournal>,
    pub(crate) execution: Option<crate::session::SnapshotSession>,
    trust: TrustLevel,
}

/// Actual immutable admission environment retained by snapshots and receipts.
/// Changing any configured component creates a new environment; existing handles
/// continue to retain the one under which they were admitted.
#[derive(Clone, Debug)]
pub struct CatalogContext {
    pub(crate) store: Arc<dyn ObjectStore>,
    pub(crate) registry: Arc<Registry>,
    pub(crate) clock: Arc<dyn Clock>,
    pub(crate) sessions: Arc<crate::session::SessionFactory>,
    pub(crate) limits: StoreLimits,
    pub(crate) validator: Option<Arc<dyn SemanticValidator>>,
}
impl std::ops::Deref for Catalog {
    type Target = CatalogContext;
    fn deref(&self) -> &Self::Target {
        &self.admission
    }
}
impl Catalog {
    /// Open an existing local directory without creating or synchronizing files.
    /// The returned catalog uses the ordinary active semantic admission path. This
    /// constructor does not install the cooperative local publication protocol.
    ///
    /// # Errors
    /// A missing/non-directory root or local backend construction failure.
    pub fn open_existing_local(
        root: &std::path::Path,
        registry: Arc<Registry>,
        trust: TrustLevel,
        clock: Arc<dyn Clock>,
        sessions: Arc<crate::session::SessionFactory>,
    ) -> Result<Self, CatalogError> {
        let root = root
            .canonicalize()
            .map_err(|source| CatalogError::Infrastructure {
                op: "resolve existing local root".to_owned(),
                source: Box::new(source),
            })?;
        if !root.is_dir() {
            return Err(admission(
                "local catalog",
                "existing root must be a directory",
            ));
        }
        let store = object_store::local::LocalFileSystem::new_with_prefix(root)
            .map_err(|source| infrastructure("open existing local store", source))?;
        Ok(Self::open(
            Arc::new(store),
            registry,
            trust,
            clock,
            sessions,
        ))
    }

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
            admission_binding: None,
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
        sessions: Arc<crate::session::SessionFactory>,
    ) -> Self {
        Self {
            reserver: Arc::clone(sessions.reserver()),
            admission: Arc::new(CatalogContext {
                store,
                registry,
                clock,
                sessions,
                limits: StoreLimits::default(),
                validator: None,
            }),
            trust,
            local: None,
            publication: None,
            execution: None,
        }
    }
    /// Select the read/construction resource envelope for this catalog context.
    ///
    /// # Errors
    /// Zero object/control bounds or bounds beyond addressable/declared representation.
    pub fn with_limits(mut self, limits: StoreLimits) -> Result<Self, CatalogError> {
        limits.validate()?;
        self.admission = Arc::new(CatalogContext {
            limits,
            ..self.admission.as_ref().clone()
        });
        Ok(self)
    }
    /// Install the rule layer that executes the registry's semantic invariants.
    #[must_use]
    pub fn with_semantic_validator(mut self, validator: Arc<dyn SemanticValidator>) -> Self {
        self.admission = Arc::new(CatalogContext {
            validator: Some(validator),
            ..self.admission.as_ref().clone()
        });
        self
    }
    pub(crate) async fn admit_content(
        &self,
        kind: pse_ids::SnapshotKind,
        candidates: &BTreeMap<RelationKey, FieldCheckedBatch>,
        context: &AdmissionContext,
        cancel: &CancellationToken,
    ) -> Result<(), CatalogError> {
        let mut reservation = self.reserver.open("store:semantic-admission");
        reservation.try_grow(membership::context_validation_extent(candidates, context)?)?;
        let rows = membership::complete_context(&self.registry, candidates, context)?;
        let changed = candidates.keys().copied().collect();
        if let Some(validator) = &self.validator {
            let session = self.validation_session(cancel)?;
            validator
                .validate_affected(&self.registry, &rows, &changed, &session, cancel)
                .await?;
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
            key.qualified_name() == "authored.documents" && batch.batch().num_rows() != 0
        }) {
            let validator = self.validator.as_ref().ok_or_else(|| {
                membership::refused(
                    "source-bearing snapshot requires executable document-to-row correspondence validation",
                )
            })?;
            let candidates = candidates
                .iter()
                .map(|(key, batch)| (*key, batch.batch().clone()))
                .collect();
            cancel.checkpoint()?;
            validator
                .validate_snapshot_sources(self, kind, context, &candidates, cancel)
                .await?;
            cancel.checkpoint()?;
        }
        Ok(())
    }

    pub(crate) fn validation_session(
        &self,
        cancel: &CancellationToken,
    ) -> Result<crate::session::SnapshotSession, CatalogError> {
        match &self.execution {
            Some(session) => Ok(session.clone()),
            None => self
                .sessions
                .candidate(BTreeMap::new(), Arc::clone(&self.registry), cancel),
        }
    }

    pub(crate) async fn admit_stage(
        &self,
        candidates: &BTreeMap<String, RecordBatch>,
        context: &AdmissionContext,
        cancel: &CancellationToken,
    ) -> Result<(), CatalogError> {
        if context.stage_pass.is_some() {
            super::stage_admission::validate(self, context, candidates, cancel).await?;
        }
        Ok(())
    }

    pub(crate) async fn admit_ports(
        &self,
        kind: pse_ids::SnapshotKind,
        candidates: &BTreeMap<String, FieldCheckedBatch>,
        inventory: &BTreeMap<String, &pse_schema::model::RelationSpec>,
        context: &AdmissionContext,
        cancel: &CancellationToken,
    ) -> Result<(), CatalogError> {
        let mut groups: BTreeMap<RelationKey, Vec<&FieldCheckedBatch>> = BTreeMap::new();
        for (port, batch) in candidates {
            groups.entry(inventory[port].key).or_default().push(batch);
        }
        let repeated: BTreeSet<_> = groups
            .iter()
            .filter(|(_, rows)| rows.len() > 1)
            .map(|(key, _)| key.qualified_name())
            .collect();
        if !repeated.is_empty() {
            // Determine which invariant targets are in this invocation. The first
            // batch contributes only its declaration key here; every port's actual
            // values are validated independently below.
            let declarations = groups
                .iter()
                .map(|(key, rows)| (*key, rows[0].clone()))
                .collect();
            let available = membership::complete_context(&self.registry, &declarations, context)?;
            let ambiguous_fk = available.keys().any(|key| {
                self.registry
                    .relation(&key.qualified_name())
                    .is_some_and(|spec| {
                        spec.columns.iter().any(|column| {
                            column.fk().is_some_and(|fk| {
                                repeated.contains(fk.relation)
                                    && fk.relation != key.qualified_name()
                            })
                        })
                    })
            });
            let ambiguous_invariant = self.registry.invariants().iter().any(|invariant| {
                self.registry
                    .relation(&invariant.relation)
                    .is_some_and(|target| {
                        available.contains_key(&target.key)
                            && self.registry.rule(&invariant.rule).is_some_and(|rule| {
                                self.registry.rule_dependencies().iter().any(|dependency| {
                                    dependency.rule_id == rule.id
                                        && repeated.contains(&dependency.relation)
                                        && dependency.relation != invariant.relation
                                })
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
    /// Actual immutable construction and read limits of this context.
    pub fn limits(&self) -> StoreLimits {
        self.limits
    }
    /// Storage provenance; all handles still require semantic admission.
    pub const fn trust_level(&self) -> TrustLevel {
        self.trust
    }
    /// The common platform reservation adapter.
    pub fn reserver(&self) -> &Arc<dyn MemoryReserver> {
        &self.reserver
    }
    /// The actual native engine and shared resources captured by this catalog.
    pub fn session_factory(&self) -> &Arc<crate::session::SessionFactory> {
        &self.sessions
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
        Ok(self
            .prepare_manifest_admission(reference, context, cancel)?
            .execute(cancel)
            .await?
            .into_value())
    }

    /// Capture an exact manifest and its actual parent providers before reading storage.
    /// # Errors
    /// Conflicting source owners, policy or native preparation failure.
    pub fn prepare_manifest_admission(
        &self,
        reference: ManifestRef,
        context: &AdmissionContext,
        cancel: &CancellationToken,
    ) -> Result<super::operation::PreparedStoreOperation<Arc<Snapshot>>, CatalogError> {
        use pse_schema::model::provider::{OperationPurpose, ProviderScope};
        let session = self.context_session(context, cancel)?;
        let context = context.clone();
        self.prepare_store_operation_in(
            &session,
            crate::store::operation::StoreCommand {
                name: "store.admit_manifest_with_parents",
                scope: ProviderScope::Schema("store".into(), "manifests".into()),
                purpose: OperationPurpose::Resolve,
                arguments: vec![datafusion::logical_expr::lit(
                    reference.manifest_checksum.to_string(),
                )],
            },
            Box::new(move |catalog, _session, cancel| {
                Box::pin(async move {
                    let snapshot = catalog
                        .read_manifest_inner(reference, &context, &cancel)
                        .await?;
                    let count = u64::try_from(snapshot.relations().len())
                        .map_err(|_| super::encode::overflow())?;
                    Ok((snapshot, count))
                })
            }),
            cancel,
        )
    }

    async fn read_manifest_inner(
        &self,
        reference: ManifestRef,
        context: &AdmissionContext,
        cancel: &CancellationToken,
    ) -> Result<Arc<Snapshot>, CatalogError> {
        let manifest = self.read_manifest_envelope(reference, cancel).await?;
        let binding = self.read_admission_binding(&manifest, cancel).await?;
        if !binding.matches(context) {
            return Err(membership::refused(
                "explicit context differs from the manifest-selected admission binding",
            ));
        }
        self.admit_manifest(reference, manifest, context, cancel)
            .await
    }

    // Only decoded current envelopes with checked or constructed exact bindings reach
    // this path. Both explicit-context and pinned traversal use the same admission.
    pub(super) async fn admit_manifest(
        &self,
        reference: ManifestRef,
        manifest: crate::snapshot::OwnedManifest,
        context: &AdmissionContext,
        cancel: &CancellationToken,
    ) -> Result<Arc<Snapshot>, CatalogError> {
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
            .load_members(&manifest, &inventory, manifest.lease(), cancel)
            .await?;
        let checked = loaded
            .iter()
            .map(|(port, relation)| (port.clone(), relation.checked().clone()))
            .collect();
        self.admit_ports(
            manifest.snapshot_kind,
            &checked,
            &inventory,
            context,
            cancel,
        )
        .await?;
        self.admit_stage(&candidates, context, cancel).await?;
        let loaded = self.finalize_loaded(loaded, alternatives, cancel)?;
        Ok(Arc::new(Snapshot {
            manifest,
            manifest_ref: reference,
            relations: Arc::new(
                loaded
                    .into_iter()
                    .map(|(key, relation)| (key, Arc::new(relation)))
                    .collect(),
            ),
            parents: Arc::new(context.parents.clone()),
            stage_pass: context.stage_pass,
            invocation: context.invocation.clone(),
            admission: Arc::clone(&self.admission),
        }))
    }

    /// Decode and check only the physical envelope; no snapshot authority is minted.
    pub(super) async fn read_manifest_envelope(
        &self,
        reference: ManifestRef,
        cancel: &CancellationToken,
    ) -> Result<crate::snapshot::OwnedManifest, CatalogError> {
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
        Ok(super::control::OwnedControl::new(manifest, metadata))
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
                if u64::try_from(decoded.batch().num_rows()).ok() != Some(member.rows) {
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
            candidates.insert(member.port.clone(), batch.batch().clone());
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
            let spec = self
                .registry
                .relation_by_id(relation.batch.relation_id())
                .ok_or_else(|| admission("relation", "checked declaration absent"))?;
            let (sorted, canonical) = relation.batch.canonicalize(
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
                let (equivalent, _) = other.canonicalize(
                    &self.registry,
                    spec,
                    self.reserver.as_ref(),
                    CanonicalizeOptions {
                        keep_preimage: false,
                        envelope: self.limits.envelope,
                        cancel: Some(cancel.clone()),
                        ..Default::default()
                    },
                )?;
                if equivalent.batch() != sorted.batch() {
                    return Err(admission(
                        &relation.member.port,
                        "encodings disagree on actual normalized logical values",
                    ));
                }
            }
            relation.batch = sorted;
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
        let result = cancel
            .until_cancelled(self.store.get(path))
            .await?
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
        while let Some(chunk) = cancel
            .until_cancelled(std::future::poll_fn(|cx| stream.as_mut().poll_next(cx)))
            .await?
        {
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

#[cfg(test)]
mod limits_tests {
    use super::StoreLimits;
    use pse_ids::Envelope;

    #[test]
    fn defaults_and_selected_bounds_validate_without_allocating_their_capacity() {
        assert!(StoreLimits::default().validate().is_ok());
        assert!(
            StoreLimits {
                max_object_bytes: 2 << 30,
                max_control_bytes: 128 << 20,
                envelope: Envelope::new(200_000_000, 1 << 30).expect("addressable"),
            }
            .validate()
            .is_ok()
        );
    }

    #[test]
    fn unsupported_representation_and_zero_physical_bounds_are_refused() {
        for limits in [
            StoreLimits {
                max_object_bytes: 0,
                ..StoreLimits::default()
            },
            StoreLimits {
                max_control_bytes: usize::MAX,
                ..StoreLimits::default()
            },
            StoreLimits {
                envelope: Envelope {
                    max_rows: u64::MAX,
                    max_normalized_bytes: 1,
                },
                ..StoreLimits::default()
            },
        ] {
            assert!(limits.validate().is_err());
        }
    }
}
