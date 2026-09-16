// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Durable stage lookup hints. Matching keys never authorize reuse or admission.

mod operation;
pub use operation::OpenedStageHint;

use std::collections::BTreeMap;
use std::sync::Arc;

use object_store::UpdateVersion;

use super::local::ObservedControl;
use datafusion::arrow::array::Array;
use pse_ids::{CancellationToken, ContentHash, ReservationLease, SemanticId, SnapshotKind};
use pse_relations::generated::provenance::pass_records;
use serde::{Deserialize, Serialize};

use super::membership::AdmissionContext;
use super::open::Catalog;
use super::sidecar::{SidecarArtifact, SidecarRef};
pub use super::stage_owned::OwnedStageHint;
use super::verify::admission;
use crate::snapshot::ManifestRef;
use crate::{CatalogError, Snapshot};

/// Explicit optional pass inputs. Every registered input port is present in this map.
pub type StageInputs = BTreeMap<String, Option<Arc<Snapshot>>>;

/// Caller-owned untrusted wire data, containing exact physical input/output references.
/// Catalog reads/writes return [`OwnedStageHint`], whose immutable clones retain memory.
/// Reopening establishes content validity; the compiler separately compares its complete
/// actual admitted inputs and invocation bindings before reuse.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageHint {
    /// Exact registered producing pass.
    pub pass_id: SemanticId,
    /// Every input, with absence distinct from an omitted declaration.
    #[serde(deserialize_with = "super::verify::unique_map")]
    pub inputs: BTreeMap<String, Option<ManifestRef>>,
    /// Exact admitted output encoding choice.
    pub output: ManifestRef,
    /// Exact typed pass-attempt record.
    pub pass_record: SidecarRef,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StageIndex<T> {
    version: u32,
    stage_key: ContentHash,
    entries: Vec<T>,
}
const MAX_ENTRIES: usize = 1024;
const STAGE_INDEX_VERSION: u32 = 4;

impl Catalog {
    /// Read lookup hints, never a validity or reuse certificate. Unknown/malformed
    /// versions and oversized buckets are typed failures; an absent bucket is empty.
    ///
    /// # Errors
    /// Malformed control encoding, resource limits, cancellation or backend failures.
    async fn stage_hints_inner(
        &self,
        key: ContentHash,
        cancel: &CancellationToken,
    ) -> Result<Vec<OwnedStageHint>, CatalogError> {
        Ok(self.stage_index(key, cancel).await?.0.entries)
    }

    /// Record a completed stage only after validating its actual input/output handles
    /// and pass-record content. Hash buckets preserve distinct exact receipts.
    ///
    /// # Errors
    /// Invalid stage/pass ties, exhausted control bounds, a CAS conflict or backend failure.
    async fn write_stage_hint_inner(
        &self,
        key: ContentHash,
        inputs: &StageInputs,
        output: &Snapshot,
        pass_record: &SidecarArtifact,
        cancel: &CancellationToken,
    ) -> Result<OwnedStageHint, CatalogError> {
        let pass_id = output.stage_pass().ok_or_else(|| {
            admission("stage index", "output has no admitted registered producer")
        })?;
        let context = self.stage_context(pass_id, inputs)?;
        if !Arc::ptr_eq(&output.admission, &self.admission)
            || output.manifest.snapshot_kind != SnapshotKind::Stage
            || context.parents.len() != output.parents.len()
            || context.parents.iter().any(|(role, parent)| {
                output
                    .parents
                    .get(role)
                    .is_none_or(|actual| actual.manifest_ref() != parent.manifest_ref())
            })
        {
            return Err(admission(
                "stage index",
                "output is not bound to these exact admitted inputs",
            ));
        }
        self.check_pass_record(pass_record, pass_id, output.snapshot_id())?;
        let hint = self.owned_hint(pass_id, inputs, output, pass_record)?;
        let (mut index, observed) = self.stage_index(key, cancel).await?;
        if index.entries.contains(&hint) {
            return Ok(hint);
        }
        if index.entries.len() >= MAX_ENTRIES {
            return Err(admission(
                "stage index",
                "stage bucket exceeds finite entry limit",
            ));
        }
        let mut entries_reservation = self.reserver.open("store:stage-index-entry-growth");
        entries_reservation.try_grow(
            index
                .entries
                .len()
                .checked_add(1)
                .and_then(|len| len.checked_mul(size_of::<OwnedStageHint>() * 2))
                .ok_or_else(super::encode::overflow)?,
        )?;
        index.entries.push(hint.clone());
        let bytes = super::encode::control(
            &index,
            self.reserver.as_ref(),
            "store:stage-index",
            self.limits.max_control_bytes,
        )?;
        self.put_control(
            &super::layout::stage_path(&key),
            observed.as_ref(),
            bytes,
            cancel,
        )
        .await?;
        drop(index);
        drop(entries_reservation);
        Ok(hint)
    }

    fn owned_hint(
        &self,
        pass_id: SemanticId,
        inputs: &StageInputs,
        output: &Snapshot,
        pass_record: &SidecarArtifact,
    ) -> Result<OwnedStageHint, CatalogError> {
        let mut reservation = self.reserver.open("store:stage-hint-header");
        let mut extent = super::stage_owned::add(
            4096,
            super::stage_owned::map_extent::<String, Option<ManifestRef>>(inputs.len())?,
        )?;
        for role in inputs.keys() {
            extent = super::stage_owned::add(extent, role.capacity())?;
        }
        extent = super::stage_owned::add(
            extent,
            super::stage_owned::member_extent(pass_record.reference().member())?,
        )?;
        reservation.try_grow(extent.checked_mul(2).ok_or_else(super::encode::overflow)?)?;
        let hint = StageHint {
            pass_id,
            inputs: inputs
                .iter()
                .map(|(role, parent)| {
                    (
                        role.clone(),
                        parent.as_ref().map(|parent| parent.manifest_ref()),
                    )
                })
                .collect(),
            output: output.manifest_ref(),
            pass_record: pass_record.reference().clone(),
        };
        let retained = super::stage_owned::hint_extent(&hint)?;
        if retained > reservation.size() {
            return Err(super::encode::overflow());
        }
        reservation.shrink(reservation.size() - retained);
        Ok(OwnedStageHint::new(
            hint,
            ReservationLease::new(reservation),
        ))
    }

    /// Reopen every output member and the pass record with explicit already admitted
    /// input handles. This is admission only; the compiler must still compare actual
    /// dependency semantics before using a hint as a memo hit.
    ///
    /// # Errors
    /// Missing/foreign input context, corrupt references or semantic admission failures.
    async fn open_stage_hint_inner(
        &self,
        hint: &StageHint,
        inputs: &StageInputs,
        cancel: &CancellationToken,
    ) -> Result<(Arc<Snapshot>, SidecarArtifact), CatalogError> {
        let context = self.stage_context(hint.pass_id, inputs)?;
        let actual: BTreeMap<_, _> = inputs
            .iter()
            .map(|(role, parent)| {
                (
                    role.clone(),
                    parent.as_ref().map(|parent| parent.manifest_ref()),
                )
            })
            .collect();
        if actual != hint.inputs {
            return Err(admission(
                "stage index",
                "explicit input references differ from the lookup receipt",
            ));
        }
        let output = self.read_pinned_manifest(hint.output, cancel).await?;
        if output.stage_pass() != context.stage_pass
            || output.parents().len() != context.parents.len()
            || output.parents().iter().any(|(role, parent)| {
                context
                    .parents
                    .get(role)
                    .is_none_or(|input| input.manifest_ref() != parent.manifest_ref())
            })
        {
            return Err(admission(
                "stage index",
                "output binding differs from selected inputs",
            ));
        }
        let record = self.read_sidecar(&hint.pass_record, cancel).await?;
        self.check_pass_record(&record, hint.pass_id, output.snapshot_id())?;
        Ok((output, record))
    }

    fn stage_context(
        &self,
        pass_id: SemanticId,
        inputs: &StageInputs,
    ) -> Result<AdmissionContext, CatalogError> {
        let pass = self
            .registry
            .passes()
            .iter()
            .find(|pass| pass.id == pass_id)
            .ok_or_else(|| admission("stage index", "producer is not registered"))?;
        if inputs.len() != pass.inputs.len()
            || pass.inputs.iter().any(|port| {
                inputs
                    .get(port.port)
                    .is_none_or(|parent| port.required && parent.is_none())
            })
        {
            return Err(admission(
                "stage index",
                "every declared input must be bound, including explicit optional absence",
            ));
        }
        let mut parents = BTreeMap::new();
        for (role, parent) in inputs {
            if let Some(parent) = parent {
                if !Arc::ptr_eq(&parent.admission, &self.admission) {
                    return Err(admission("stage index", "foreign input must be reopened"));
                }
                parents.insert(role.clone(), Arc::clone(parent));
            }
        }
        Ok(AdmissionContext {
            traversal: Arc::default(),
            invocation: None,
            parents,
            stage_pass: Some(pass_id),
        })
    }

    async fn stage_index(
        &self,
        key: ContentHash,
        cancel: &CancellationToken,
    ) -> Result<(StageIndex<OwnedStageHint>, Option<ObservedControl>), CatalogError> {
        let path = super::layout::stage_path(&key);
        let (bytes, meta) = match self
            .read_versioned(&path, self.limits.max_control_bytes, cancel)
            .await
        {
            Ok(value) => value,
            Err(CatalogError::Infrastructure { source, .. })
                if source
                    .downcast_ref::<object_store::Error>()
                    .is_some_and(|error| matches!(error, object_store::Error::NotFound { .. })) =>
            {
                return Ok((
                    StageIndex {
                        version: STAGE_INDEX_VERSION,
                        stage_key: key,
                        entries: vec![],
                    },
                    None,
                ));
            }
            Err(error) => return Err(error),
        };
        let reservation = super::control::decode_reservation(
            &bytes,
            self.reserver.as_ref(),
            "store:stage-index-decode",
        )?;
        let index: StageIndex<StageHint> = serde_json::from_slice(&bytes)
            .map_err(|error| admission("stage index", &error.to_string()))?;
        if index.version != STAGE_INDEX_VERSION
            || index.stage_key != key
            || index.entries.len() > MAX_ENTRIES
        {
            return Err(admission(
                "stage index",
                "unknown version, wrong lookup identity or excessive entries",
            ));
        }
        if self.local.is_none() && meta.e_tag.is_none() && meta.version.is_none() {
            return Err(admission(
                "stage index",
                "backend provides no conditional update token",
            ));
        }
        let index = self.retain_index(index, reservation)?;
        Ok((
            index,
            Some(ObservedControl {
                bytes,
                version: UpdateVersion {
                    e_tag: meta.e_tag,
                    version: meta.version,
                },
            }),
        ))
    }

    fn retain_index(
        &self,
        index: StageIndex<StageHint>,
        mut reservation: Box<dyn pse_ids::Reservation>,
    ) -> Result<StageIndex<OwnedStageHint>, CatalogError> {
        let slots = index
            .entries
            .len()
            .checked_mul(size_of::<OwnedStageHint>())
            .ok_or_else(super::encode::overflow)?;
        let retained = index.entries.iter().try_fold(slots, |bytes, hint| {
            super::stage_owned::add(bytes, super::stage_owned::hint_extent(hint)?)
        })?;
        if retained > reservation.size() {
            return Err(super::encode::overflow());
        }
        // The decoded Vec allocation coexists with the explicitly allocated wrapper
        // Vec while DTOs move without cloning. Keep that transition charged separately.
        let mut moving = self.reserver.open("store:stage-index-wrap");
        moving.try_grow(
            index
                .entries
                .capacity()
                .checked_mul(size_of::<StageHint>())
                .ok_or_else(super::encode::overflow)?,
        )?;
        reservation.shrink(reservation.size() - retained);
        let lease = ReservationLease::new(reservation);
        let mut entries = Vec::with_capacity(index.entries.len());
        for hint in index.entries {
            entries.push(OwnedStageHint::new(hint, Arc::clone(&lease)));
        }
        drop(moving);
        Ok(StageIndex {
            version: index.version,
            stage_key: index.stage_key,
            entries,
        })
    }

    fn check_pass_record(
        &self,
        record: &SidecarArtifact,
        pass_id: SemanticId,
        output: pse_ids::SnapshotId,
    ) -> Result<(), CatalogError> {
        if !Arc::ptr_eq(&record.admission, &self.admission) {
            return Err(admission("stage index", "pass record must be reopened"));
        }
        let spec = self
            .registry
            .relation("provenance.pass_records")
            .ok_or_else(|| admission("stage index", "pass records are not declared"))?;
        if record.relation().contract().canonical.relation_id != spec.id {
            return Err(admission(
                "stage index",
                "sidecar is not the pass record relation",
            ));
        }
        let pass = self
            .registry
            .passes()
            .iter()
            .find(|pass| pass.id == pass_id)
            .ok_or_else(|| admission("stage index", "pass is not declared"))?;
        let rows = pass_records::View::from_checked(record.relation().checked())
            .map_err(|error| admission("stage index", &error.to_string()))?;
        let status = rows.status_column();
        if rows.len() != 1
            || rows.pass_id_column().value(0) != pass_id.as_bytes()
            || rows.version_column().value(0) != pass.version
            || rows.snapshot_out_column().is_null(0)
            || rows.snapshot_out_column().value(0) != output.0.as_bytes()
            || !matches!(status.value(0), "ok" | "reused")
        {
            return Err(admission(
                "stage index",
                "actual pass record must name this exact successful producer and output",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use object_store::{ObjectStoreExt, memory::InMemory};

    #[tokio::test]
    async fn stage_lookup_accepts_only_the_current_reference_format() {
        let store = Arc::new(InMemory::new());
        let catalog = Catalog::open(
            store.clone(),
            Arc::new(pse_schema::RegistryBuilder::new().build().unwrap()),
            crate::TrustLevel::Untrusted,
            Arc::new(crate::FixedClock("2026-09-15T00:00:00Z".to_owned())),
            Arc::new(
                crate::session::SessionFactory::new(
                    Arc::new(datafusion::execution::runtime_env::RuntimeEnv::default()),
                    pse_ids::FixedBudget::new(1 << 20),
                    crate::ExecutionSettings::default(),
                    crate::ThreadBudget {
                        pool_threads: std::num::NonZeroUsize::MIN,
                        target_partitions: std::num::NonZeroUsize::MIN,
                    },
                    crate::session::native_engine_profile(),
                )
                .unwrap(),
            ),
        );
        let key = ContentHash::from_bytes([7; 32]);
        let path = super::super::layout::stage_path(&key);
        for version in [1, 2, 3, STAGE_INDEX_VERSION, STAGE_INDEX_VERSION + 1] {
            let bytes = serde_json::to_vec(&StageIndex::<StageHint> {
                version,
                stage_key: key,
                entries: vec![],
            })
            .unwrap();
            store.put(&path, bytes.into()).await.unwrap();
            let result = catalog.stage_hints(key, &CancellationToken::new()).await;
            assert_eq!(result.is_ok(), version == STAGE_INDEX_VERSION);
        }
    }
}
