// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Current receiving-source bindings, paired with an already admitted numerical body.
use super::{CompiledBody, Evaluation, PreparedBody, PreparedBodyData, TrialEvaluation, Worker};
use crate::{MathError, jets::ProductValues};
use pse_ids::{ContentHash, FramedHasher, SemanticId};
use pse_kernels::{DerivativeOrder, ExecutionScope, ProviderBindings, ProviderError};
use pse_model::diagnostic::SourceLocation;
use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
    sync::{Arc, Weak, atomic::AtomicBool},
};

/// Complete current instance attribution for exactly one admitted original body.
/// Maps are never used to establish reusable arithmetic identity.
#[derive(Clone, Debug)]
pub struct SourceBindings {
    attribution: ExecutionAttribution,
    source: Weak<PreparedBodyData>,
}

/// Immutable diagnostic context for an already checked numerical product.
/// This contains no source witness, execution permission or mutable worker state.
#[derive(Clone, Debug, PartialEq)]
pub struct ExecutionAttribution {
    instance: SemanticId,
    checked_members: Arc<BTreeMap<SemanticId, SemanticId>>,
    checked_sources: Arc<BTreeMap<SemanticId, SemanticId>>,
    occurrences: Arc<BTreeMap<SemanticId, SourceLocation>>,
}
pse_contract::projection_struct! {
/// Complete passive original attribution maps. This record contains no source
/// witness, Root authority or execution permission, even with a matching checksum.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionAttributionSnapshot {
    projection: { name: "MathExecutionAttributionSnapshot", rust_path: crate::ExecutionAttributionSnapshot, version: 1,
        direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Actual original instance whose diagnostic coordinates these maps describe.
    pub instance: SemanticId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Complete local checked-member tokens and original current semantic members.
    pub checked_members: BTreeMap<SemanticId, SemanticId> => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Complete distinct local closure-source tokens and original current sources.
    pub checked_sources: BTreeMap<SemanticId, SemanticId> => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Complete original occurrence locations, including legitimate unused entries.
    pub occurrences: BTreeMap<SemanticId, SourceLocation> => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}

/// Actual attribution populations and a source-backed Rust tree allocation bound.
/// This is not allocator usable size/RSS, source authority or an admission grant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttributionExtents {
    /// Actual entries across all three original maps.
    pub entries: usize,
    /// Actual UTF-8 bytes copied by a metadata clone.
    pub clone_string_bytes: usize,
    /// Actual retained UTF-8 capacities in the existing owned maps.
    pub owned_string_bytes: usize,
    /// Requested node/string backing upper bound for a complete clone.
    pub clone_bytes: usize,
    /// Existing node/string backing upper bound, including actual string capacities.
    pub owned_bytes: usize,
}
/// Passive attribution shape, commitment and checked population refusals.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[diagnostic(code("math.attribution_transfer"))]
pub enum AttributionTransferError {
    /// The original instance/map/location shape is invalid.
    #[error("invalid original attribution shape")]
    Shape,
    /// Full map framing differs from the original parent-issued checksum.
    #[error("original attribution commitment differs")]
    Commitment,
    /// A source-backed backing/population cannot be represented on this platform.
    #[error("original attribution backing extent overflow")]
    Extent,
    /// An unambiguous emitted Closure member token has no original target.
    #[error("original attribution lacks Closure member {0}")]
    MissingMember(SemanticId),
    /// An unambiguous emitted Closure source token has no original target.
    #[error("original attribution lacks Closure source {0}")]
    MissingSource(SemanticId),
}
fn attribution_shape(
    instance: SemanticId,
    members: &BTreeMap<SemanticId, SemanticId>,
    sources: &BTreeMap<SemanticId, SemanticId>,
    occurrences: &BTreeMap<SemanticId, SourceLocation>,
) -> bool {
    instance != SemanticId::NIL
        && members
            .iter()
            .chain(sources)
            .all(|(token, target)| *token != SemanticId::NIL && *target != SemanticId::NIL)
        && occurrences.iter().all(|(token, location)| {
            *token != SemanticId::NIL
                && location.source != SemanticId::NIL
                && location.document != Some(SemanticId::NIL)
                && match (location.start, location.end) {
                    (None, None) => true,
                    (Some(start), Some(end)) => start <= end && location.document.is_some(),
                    _ => false,
                }
        })
}
fn attribution_frame(
    instance: SemanticId,
    members: &BTreeMap<SemanticId, SemanticId>,
    sources: &BTreeMap<SemanticId, SemanticId>,
    occurrences: &BTreeMap<SemanticId, SourceLocation>,
    hash: &mut FramedHasher,
) {
    hash.id(&instance);
    for map in [members, sources] {
        hash.u64(map.len() as u64);
        for (local, current) in map {
            hash.id(local).id(current);
        }
    }
    hash.u64(occurrences.len() as u64);
    for (local, current) in occurrences {
        hash.id(local);
        current.frame(hash);
    }
}
fn attribution_add(a: usize, b: usize) -> Result<usize, AttributionTransferError> {
    a.checked_add(b).ok_or(AttributionTransferError::Extent)
}
fn attribution_tree<K, V>(entries: usize) -> Result<usize, AttributionTransferError> {
    // Pinned Rust alloc/collections/btree/node.rs: B=6, capacity=11,
    // LeafNode has parent pointer + two u16 fields, keys/values; InternalNode
    // adds 12 child pointers. Every populated non-root has >=5 entries.
    // Charge an internal-node bound even for the root/empty retained leaf. No
    // private tree layout is accessed, copied or implemented by the project.
    let header = std::alloc::Layout::new::<[usize; 2]>();
    let keys = std::alloc::Layout::array::<K>(11).map_err(|_| AttributionTransferError::Extent)?;
    let values =
        std::alloc::Layout::array::<V>(11).map_err(|_| AttributionTransferError::Extent)?;
    let (layout, _) = header
        .extend(keys)
        .map_err(|_| AttributionTransferError::Extent)?;
    let (layout, _) = layout
        .extend(values)
        .map_err(|_| AttributionTransferError::Extent)?;
    let (layout, _) = layout
        .extend(std::alloc::Layout::new::<[usize; 12]>())
        .map_err(|_| AttributionTransferError::Extent)?;
    let nodes = attribution_add(entries / 5, 1)?;
    let bytes = nodes
        .checked_mul(layout.pad_to_align().size())
        .ok_or(AttributionTransferError::Extent)?;
    std::alloc::Layout::array::<u8>(bytes)
        .map(|layout| layout.size())
        .map_err(|_| AttributionTransferError::Extent)
}
fn attribution_extents(
    members: &BTreeMap<SemanticId, SemanticId>,
    sources: &BTreeMap<SemanticId, SemanticId>,
    occurrences: &BTreeMap<SemanticId, SourceLocation>,
) -> Result<AttributionExtents, AttributionTransferError> {
    let entries = attribution_add(
        attribution_add(members.len(), sources.len())?,
        occurrences.len(),
    )?;
    let nodes = attribution_add(
        attribution_add(
            attribution_tree::<SemanticId, SemanticId>(members.len())?,
            attribution_tree::<SemanticId, SemanticId>(sources.len())?,
        )?,
        attribution_tree::<SemanticId, SourceLocation>(occurrences.len())?,
    )?;
    let (mut clone_string_bytes, mut owned_string_bytes) = (0, 0);
    for location in occurrences.values() {
        for text in [
            Some(&location.path),
            location.name.as_ref(),
            location.occurrence_role.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            clone_string_bytes = attribution_add(clone_string_bytes, text.len())?;
            owned_string_bytes = attribution_add(owned_string_bytes, text.capacity())?;
        }
    }
    Ok(AttributionExtents {
        entries,
        clone_string_bytes,
        owned_string_bytes,
        clone_bytes: attribution_add(nodes, clone_string_bytes)?,
        owned_bytes: attribution_add(nodes, owned_string_bytes)?,
    })
}
impl ExecutionAttributionSnapshot {
    /// Validate shared original map/location shape; this establishes no source witness.
    pub fn validate_shape(&self) -> Result<(), AttributionTransferError> {
        if attribution_shape(
            self.instance,
            &self.checked_members,
            &self.checked_sources,
            &self.occurrences,
        ) {
            Ok(())
        } else {
            Err(AttributionTransferError::Shape)
        }
    }
    /// Checked passive populations before metadata materialization/adoption.
    pub fn extents(&self) -> Result<AttributionExtents, AttributionTransferError> {
        self.validate_shape()?;
        attribution_extents(
            &self.checked_members,
            &self.checked_sources,
            &self.occurrences,
        )
    }
    /// Full original framing checksum only; a matching caller-controlled replacement
    /// remains passive data. The authenticated private issuer establishes origin.
    pub fn commitment(&self) -> ContentHash {
        let mut hash = FramedHasher::new(pse_ids::Frame::ExecutionAttributionV1);
        attribution_frame(
            self.instance,
            &self.checked_members,
            &self.checked_sources,
            &self.occurrences,
            &mut hash,
        );
        hash.finish_hash()
    }
    pub(crate) fn check_commitment(
        &self,
        expected: ContentHash,
    ) -> Result<(), AttributionTransferError> {
        self.validate_shape()?;
        if self.commitment() == expected {
            Ok(())
        } else {
            Err(AttributionTransferError::Commitment)
        }
    }
    pub(crate) fn into_attribution(self) -> ExecutionAttribution {
        ExecutionAttribution {
            instance: self.instance,
            checked_members: Arc::new(self.checked_members),
            checked_sources: Arc::new(self.checked_sources),
            occurrences: Arc::new(self.occurrences),
        }
    }
}
impl ExecutionAttribution {
    pub(crate) fn snapshot_extents(&self) -> Result<AttributionExtents, AttributionTransferError> {
        if !attribution_shape(
            self.instance,
            &self.checked_members,
            &self.checked_sources,
            &self.occurrences,
        ) {
            return Err(AttributionTransferError::Shape);
        }
        attribution_extents(
            &self.checked_members,
            &self.checked_sources,
            &self.occurrences,
        )
    }
    pub(crate) fn snapshot_after_admission(&self) -> ExecutionAttributionSnapshot {
        ExecutionAttributionSnapshot {
            instance: self.instance,
            checked_members: self.checked_members.as_ref().clone(),
            checked_sources: self.checked_sources.as_ref().clone(),
            occurrences: self.occurrences.as_ref().clone(),
        }
    }
    /// Checksum of the exact original map framing, separate from source/root identities.
    pub fn commitment(&self) -> ContentHash {
        let mut hash = FramedHasher::new(pse_ids::Frame::ExecutionAttributionV1);
        self.frame(&mut hash);
        hash.finish_hash()
    }
}
impl PartialEq for SourceBindings {
    fn eq(&self, other: &Self) -> bool {
        self.attribution == other.attribution
    }
}
impl SourceBindings {
    /// Validate actual token inventories before any compiled body can acquire these bindings.
    pub fn new(
        instance: SemanticId,
        body: &PreparedBody,
        checked_members: BTreeMap<SemanticId, SemanticId>,
        checked_sources: BTreeMap<SemanticId, SemanticId>,
        occurrences: BTreeMap<SemanticId, SourceLocation>,
    ) -> Result<Self, MathError> {
        Self::check_inventory(
            instance,
            body,
            &checked_members,
            &checked_sources,
            &occurrences,
        )?;
        Ok(Self {
            attribution: ExecutionAttribution {
                instance,
                checked_members: Arc::new(checked_members),
                checked_sources: Arc::new(checked_sources),
                occurrences: Arc::new(occurrences),
            },
            source: Arc::downgrade(&body.data),
        })
    }
    pub(crate) fn check_inventory(
        instance: SemanticId,
        body: &PreparedBody,
        checked_members: &BTreeMap<SemanticId, SemanticId>,
        checked_sources: &BTreeMap<SemanticId, SemanticId>,
        occurrences: &BTreeMap<SemanticId, SourceLocation>,
    ) -> Result<(), MathError> {
        if !attribution_shape(instance, checked_members, checked_sources, occurrences)
            || !checked_members.keys().eq(body.checked_members().iter())
            || !checked_sources.keys().eq(body.checked_sources().iter())
            || !occurrences.keys().eq(body.occurrence_tokens().iter())
        {
            return Err(MathError::Contract(
                "receiving source bindings do not cover the admitted body".into(),
            ));
        }
        Ok(())
    }
    /// Actual current instance, never an anonymous default.
    pub fn instance(&self) -> SemanticId {
        self.attribution.instance()
    }
    /// Actual current checked-member mapping, without transferring admission authority.
    pub fn checked_members(&self) -> &BTreeMap<SemanticId, SemanticId> {
        self.attribution.checked_members()
    }
    /// Actual current closure-source mapping, separate from checked members.
    pub fn checked_sources(&self) -> &BTreeMap<SemanticId, SemanticId> {
        self.attribution.checked_sources()
    }
    /// Complete current occurrence DTOs for explicit transport and receiving validation.
    pub fn occurrences(&self) -> &BTreeMap<SemanticId, SourceLocation> {
        self.attribution.occurrences()
    }
    /// Refuse bindings issued for an unrelated admitted body.
    pub fn check(&self, body: &PreparedBody) -> Result<(), MathError> {
        let same = self.source.ptr_eq(&Arc::downgrade(&body.data))
            || self
                .source
                .upgrade()
                .is_some_and(|data| PreparedBody { data, owner: None } == *body);
        if same {
            Ok(())
        } else {
            Err(MathError::Contract(
                "source bindings belong to another admitted body".into(),
            ))
        }
    }
    /// Bind a demanded derivative program to its original body without retaining that body.
    pub fn check_compiled(&self, body: &CompiledBody) -> Result<(), MathError> {
        let source = &body.prepared_support().data.source;
        let same = self.source.ptr_eq(source)
            || self
                .source
                .upgrade()
                .zip(source.upgrade())
                .is_some_and(|(a, b)| {
                    PreparedBody {
                        data: a,
                        owner: None,
                    } == PreparedBody {
                        data: b,
                        owner: None,
                    }
                });
        if same {
            Ok(())
        } else {
            Err(MathError::Contract(
                "source bindings belong to another compiled source".into(),
            ))
        }
    }
    /// Own the current diagnostic projection without changing or globally rewriting its cause.
    pub fn project(&self, cause: MathError) -> MathError {
        self.attribution.project(cause)
    }
    /// Issue passive execution attribution only for this checked compiled source.
    /// The caller must separately bind any delegated use to its admitted attempt.
    pub fn execution_attribution(
        &self,
        body: &CompiledBody,
    ) -> Result<ExecutionAttribution, MathError> {
        self.check_compiled(body)?;
        Ok(self.attribution.clone())
    }
    /// Conservative retained extent, including the original weak-source handle.
    pub fn retained_bytes(&self) -> usize {
        self.attribution
            .retained_bytes()
            .saturating_add(size_of::<Self>() - size_of::<ExecutionAttribution>())
    }
    /// Frame complete current attribution without conferring source authority.
    pub fn frame(&self, hash: &mut FramedHasher) {
        self.attribution.frame(hash);
    }
    pub(crate) fn evaluation_error(&self, error: MathError) -> MathError {
        self.attribution.evaluation_error(error)
    }
}

impl ExecutionAttribution {
    /// Original instance whose diagnostic coordinates these maps describe.
    pub fn instance(&self) -> SemanticId {
        self.instance
    }
    /// Checked-member mapping, distinct from the closure-source mapping.
    pub fn checked_members(&self) -> &BTreeMap<SemanticId, SemanticId> {
        &self.checked_members
    }
    /// Original closure-source mapping.
    pub fn checked_sources(&self) -> &BTreeMap<SemanticId, SemanticId> {
        &self.checked_sources
    }
    /// Complete original occurrence locations.
    pub fn occurrences(&self) -> &BTreeMap<SemanticId, SourceLocation> {
        &self.occurrences
    }
    /// Preserve the original cause while projecting its diagnostic coordinates.
    pub fn project(&self, cause: MathError) -> MathError {
        MathError::Instance {
            instance: self.instance,
            checked_members: self.checked_members.as_ref().clone(),
            checked_sources: self.checked_sources.as_ref().clone(),
            occurrences: self.occurrences.as_ref().clone(),
            cause: Box::new(cause),
        }
    }
    /// Conservative retained extent of these current-source maps, excluding a second body.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            .saturating_add(
                self.checked_members
                    .len()
                    .saturating_add(self.checked_sources.len())
                    .saturating_mul(size_of::<(SemanticId, SemanticId)>() + 96),
            )
            .saturating_add(
                self.occurrences
                    .len()
                    .saturating_mul(size_of::<(SemanticId, SourceLocation)>() + 96),
            )
            .saturating_add(
                self.occurrences
                    .values()
                    .map(SourceLocation::heap_bytes)
                    .fold(0usize, usize::saturating_add),
            )
            .saturating_add(6 * size_of::<usize>())
    }
    /// Frame complete current attribution only when a caller's identity includes source context.
    pub fn frame(&self, hash: &mut FramedHasher) {
        attribution_frame(
            self.instance,
            &self.checked_members,
            &self.checked_sources,
            &self.occurrences,
            hash,
        );
    }
    pub(crate) fn evaluation_error(&self, error: MathError) -> MathError {
        // These are control/refusal classes consumed directly by the provider boundary.
        match error {
            MathError::Cancelled
            | MathError::Limit(_)
            | MathError::ByteLimit { .. }
            | MathError::SlotLimit { .. }
            | MathError::WorkLimit { .. }
            | MathError::Scope(_) => error,
            MathError::Provider {
                cause: ProviderError::Cancelled | ProviderError::Deadline | ProviderError::Limit(_),
                ..
            } => error,
            error => self.project(error),
        }
    }
}

/// One compiled product paired with its validated receiving-source context.
#[derive(Clone, Debug)]
pub struct SourceBody {
    compiled: Arc<CompiledBody>,
    bindings: SourceBindings,
}
impl SourceBody {
    /// Prepare and compile all inputs and outputs with explicit receiving-source attribution.
    /// The binding callback runs while the admitted original body is still alive, before
    /// compilation can release it. Compilation policy and execution scope remain caller-owned.
    /// # Errors
    /// Physical preparation, incomplete or crossed attribution, compilation limits or cancellation.
    pub fn prepare(
        builder: crate::typed::BodyBuilder<'_>,
        outputs: &[crate::typed::TypedValue],
        bind: impl FnOnce(&PreparedBody) -> Result<SourceBindings, MathError>,
        order: DerivativeOrder,
        options: crate::library::Optimization,
        limits: crate::jets::EvaluationLimits,
        scope: &ExecutionScope,
    ) -> Result<Self, MathError> {
        let prepared = builder.prepare(outputs)?;
        let bindings = bind(&prepared)?;
        bindings.check(&prepared)?;
        let compiled = prepared.compile(
            &(0..prepared.output_count()).collect::<Vec<_>>(),
            &(0..prepared.input_count()).collect::<Vec<_>>(),
            order,
            options,
            limits,
            scope,
        )?;
        Self::new(Arc::new(compiled), bindings)
    }
    /// Refuse crossed body/context pairs before worker creation.
    pub fn new(compiled: Arc<CompiledBody>, bindings: SourceBindings) -> Result<Self, MathError> {
        bindings.check_compiled(&compiled)?;
        Ok(Self { compiled, bindings })
    }
    /// Retained admitted numerical product, without mutable access to its paired slot.
    pub fn compiled(&self) -> &Arc<CompiledBody> {
        &self.compiled
    }
    /// Source context for new demanded products of this same original body.
    pub fn bindings(&self) -> &SourceBindings {
        &self.bindings
    }
    /// Retain admission through the product; this preserves the original source allocation identity.
    pub fn with_owner(mut self, owner: Arc<dyn crate::AllocationOwner>) -> Self {
        self.compiled = Arc::new(self.compiled.as_ref().clone().with_owner(owner));
        self
    }
    /// Conservative body and current attribution extent.
    pub fn retained_bytes(&self) -> usize {
        self.compiled
            .retained_bytes()
            .saturating_add(self.bindings.retained_bytes())
    }
    pub(crate) fn worker(&self) -> SourceWorker {
        SourceWorker {
            worker: self.compiled.worker(),
            attribution: self.bindings.attribution.clone(),
        }
    }
    pub(crate) fn worker_scoped(&self, scope: ExecutionScope) -> SourceWorker {
        SourceWorker {
            worker: self.compiled.worker_scoped(scope),
            attribution: self.bindings.attribution.clone(),
        }
    }
}
impl Deref for SourceBody {
    type Target = Arc<CompiledBody>;
    fn deref(&self) -> &Self::Target {
        &self.compiled
    }
}

/// Worker-local numerical scratch with immutable, body-specific diagnostic attribution.
#[derive(Debug)]
pub(crate) struct SourceWorker {
    worker: Worker,
    attribution: ExecutionAttribution,
}
impl Deref for SourceWorker {
    type Target = Worker;
    fn deref(&self) -> &Self::Target {
        &self.worker
    }
}
impl DerefMut for SourceWorker {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.worker
    }
}
impl SourceWorker {
    pub(crate) fn prepare_providers(
        &mut self,
        providers: &mut ProviderBindings,
    ) -> Result<(), MathError> {
        self.worker
            .prepare_providers(providers)
            .map_err(|error| self.attribution.evaluation_error(error))
    }
    pub(crate) fn evaluate(
        &mut self,
        inputs: &[f64],
        order: DerivativeOrder,
        providers: &mut ProviderBindings,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Evaluation, MathError> {
        self.worker
            .evaluate(inputs, order, providers, cancel)
            .map_err(|error| self.attribution.evaluation_error(error))
    }
    pub(crate) fn evaluate_trial_into(
        &mut self,
        inputs: &[f64],
        order: DerivativeOrder,
        providers: &mut ProviderBindings,
        cancel: &Arc<AtomicBool>,
        result: &mut TrialEvaluation,
    ) -> Result<(), MathError> {
        self.worker
            .evaluate_trial_into(inputs, order, providers, cancel, result)
            .map_err(|error| self.attribution.evaluation_error(error))
    }
    pub(crate) fn evaluate_product_trial_into(
        &mut self,
        inputs: &[f64],
        values: ProductValues<'_>,
        providers: &mut ProviderBindings,
        cancel: &Arc<AtomicBool>,
        output: &mut [f64],
        trial: &mut TrialEvaluation,
    ) -> Result<(), MathError> {
        self.worker
            .evaluate_product_trial_into(inputs, values, providers, cancel, output, trial)
            .map_err(|error| self.attribution.evaluation_error(error))
    }
}

#[cfg(test)]
#[path = "source_bindings_tests.rs"]
mod tests;

#[cfg(test)]
mod transfer_tests {
    use super::*;
    fn id(n: u8) -> SemanticId {
        SemanticId::from_bytes([n; 16])
    }
    fn snapshot() -> ExecutionAttributionSnapshot {
        ExecutionAttributionSnapshot {
            instance: id(1),
            checked_members: [(id(2), id(3))].into(),
            checked_sources: [(id(4), id(5))].into(),
            occurrences: [(
                id(6),
                SourceLocation {
                    source: id(7),
                    revision: Some(ContentHash::from_bytes([8; 32])),
                    document: Some(id(9)),
                    occurrence_role: Some("role".into()),
                    occurrence_position: Some(2),
                    path: "original".into(),
                    name: Some("name".into()),
                    start: Some(1),
                    end: Some(3),
                },
            )]
            .into(),
        }
    }
    #[test]
    fn passive_attribution_keeps_original_frame_order_full_maps_and_actual_text_capacities() {
        let mut passive = snapshot();
        let extent = passive.extents().unwrap();
        assert_eq!(extent.entries, 3);
        assert_eq!(extent.clone_string_bytes, 16);
        passive
            .occurrences
            .get_mut(&id(6))
            .unwrap()
            .path
            .reserve_exact(100);
        let enlarged = passive.extents().unwrap();
        assert_eq!(extent.clone_bytes, enlarged.clone_bytes);
        assert!(enlarged.owned_bytes > extent.owned_bytes);
        let mut original = FramedHasher::new(pse_ids::Frame::ExecutionAttributionV1);
        original.id(&passive.instance);
        original.u64(1).id(&id(2)).id(&id(3));
        original.u64(1).id(&id(4)).id(&id(5));
        original.u64(1).id(&id(6));
        passive.occurrences[&id(6)].frame(&mut original);
        assert_eq!(original.finish_hash(), passive.commitment());
        let commitment = passive.commitment();
        let checked = passive.clone().into_attribution();
        assert_eq!(checked.commitment(), commitment);
        assert_eq!(checked.snapshot_after_admission(), passive);
        passive.checked_members.insert(id(10), id(11));
        assert_ne!(commitment, passive.commitment());
        assert!(matches!(
            passive.check_commitment(commitment),
            Err(AttributionTransferError::Commitment)
        ));
    }
    #[test]
    fn passive_attribution_uses_shared_shape_rules_and_checked_tree_bound() {
        let mutations: [fn(&mut ExecutionAttributionSnapshot); 8] = [
            |s| s.instance = SemanticId::NIL,
            |s| {
                s.checked_members.insert(SemanticId::NIL, id(1));
            },
            |s| {
                s.checked_sources.insert(id(1), SemanticId::NIL);
            },
            |s| s.occurrences.get_mut(&id(6)).unwrap().source = SemanticId::NIL,
            |s| s.occurrences.get_mut(&id(6)).unwrap().document = Some(SemanticId::NIL),
            |s| s.occurrences.get_mut(&id(6)).unwrap().document = None,
            |s| s.occurrences.get_mut(&id(6)).unwrap().end = None,
            |s| s.occurrences.get_mut(&id(6)).unwrap().end = Some(0),
        ];
        for mutate in mutations {
            let mut bad = snapshot();
            mutate(&mut bad);
            assert!(matches!(
                bad.extents(),
                Err(AttributionTransferError::Shape)
            ));
        }
        assert!(matches!(
            attribution_tree::<SemanticId, SourceLocation>(usize::MAX),
            Err(AttributionTransferError::Extent)
        ));
        let mut unlocated = snapshot();
        let location = unlocated.occurrences.get_mut(&id(6)).unwrap();
        location.start = None;
        location.end = None;
        location.document = None;
        assert!(unlocated.validate_shape().is_ok());
    }
}
