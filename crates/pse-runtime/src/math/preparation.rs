// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact immutable preparation products; current protection and publication stay with callers.
use super::{
    MathRuntimeError, MathService, Workspace,
    modeling::{ModelingPreparation, ModelingRevision},
};
use pse_columnar::{
    AllocationLease,
    flight::{FlightCancellation, FlightError},
};
use pse_compiler::workspace::{ModelingPreparationFrontier, PreparedModeling};
use pse_model::lineage::Solved;
use pse_modeling::{Bindings, DeclarationId, InstanceId, Limits};
use pse_operations::canonical_selection::SelectedDependencies;
use std::{
    collections::BTreeMap,
    hash::{Hash, Hasher},
    sync::Arc,
};

/// An exact instance-qualified request, retaining the owners of local context identities.
/// No selected read, ancestor revision, or attempt cancellation belongs in this key.
#[derive(Debug)]
pub(crate) struct BasisKey {
    request: Arc<crate::workflow::modeling::SelectedRequest>,
    dependencies: Arc<SelectedDependencies>,
    root: DeclarationId,
    instance: InstanceId,
    bindings: Bindings,
    limits: Limits,
    /// Process-local bucket hint; complete fields above remain the equality authority.
    prehash: u64,
    _validation_owner: Arc<pse_engine::session::EngineFactory>,
    _registry_owner: Arc<pse_schema::Registry>,
    _selected_metadata: Arc<AllocationLease>,
    _owner: Arc<AllocationLease>,
}
impl PartialEq for BasisKey {
    fn eq(&self, other: &Self) -> bool {
        self.request == other.request
            && self.dependencies == other.dependencies
            && self.root == other.root
            && self.instance == other.instance
            && self.bindings == other.bindings
            && self.limits == other.limits
    }
}
impl Eq for BasisKey {}
impl Hash for BasisKey {
    fn hash<H: Hasher>(&self, hash: &mut H) {
        hash.write_u64(self.prehash);
    }
}
impl BasisKey {
    fn hash_fields<H: Hasher>(&self, hash: &mut H) {
        self.request.hash(hash);
        self.dependencies.hash(hash);
        self.root.hash(hash);
        self.instance.hash(hash);
        self.bindings.arguments.hash(hash);
        self.bindings.scope.hash(hash);
        self.bindings.facts.hash(hash);
        self.bindings.demand.hash(hash);
        self.bindings.formulation.omitted.hash(hash);
        self.bindings.formulation.elastic.hash(hash);
        self.limits.depth.hash(hash);
        self.limits.items.hash(hash);
        self.limits.members.hash(hash);
        self.limits.body_occurrences.hash(hash);
        self.limits.body_slots.hash(hash);
    }
}
fn bindings_bytes(bindings: &Bindings) -> usize {
    size_of::<Bindings>()
        + bindings
            .arguments
            .iter()
            .chain(&bindings.scope)
            .map(|(name, value)| name.capacity() + value.retained_bytes() + 128)
            .sum::<usize>()
        + bindings
            .facts
            .values()
            .map(|value| value.retained_bytes() + 128)
            .sum::<usize>()
        + bindings.demand.capacity() * size_of::<String>()
        + bindings.demand.iter().map(String::capacity).sum::<usize>()
        + bindings.formulation.omitted.len() * 128
        + bindings
            .formulation
            .elastic
            .values()
            .map(|value| value.retained_bytes() + 128)
            .sum::<usize>()
}
impl BasisKey {
    #[cfg(test)]
    pub(crate) fn force_prehash_for_test(&mut self, prehash: u64) {
        self.prehash = prehash;
    }
    pub(crate) fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + 128
            + self.request.retained_bytes()
            + self.dependencies.retained_bytes()
            + bindings_bytes(&self.bindings)
    }
}

/// Planner output owns every allocation independently of the workspace generation.
#[derive(Debug)]
pub(crate) struct OwnedModelingFrontier {
    product: ModelingPreparationFrontier,
    solved: Solved,
    _owner: Arc<AllocationLease>,
    key: Option<Arc<BasisKey>>,
    generation: u64,
}
impl OwnedModelingFrontier {
    pub(crate) fn body_requests(&self) -> impl Iterator<Item = pse_ids::roles::SemanticBodyHash> {
        self.product.body_requests()
    }
    pub(crate) fn retained_bytes(&self) -> usize {
        size_of::<Self>() + self.product.retained_bytes()
    }
}

/// Complete mathematics plus reusable description requirements. These premises and bytes
/// carry no protection or acknowledgment; each current consumer rechecks and settles them.
#[derive(Debug)]
pub(crate) struct PreparedBasis {
    product: PreparedModeling,
    root: DeclarationId,
    instance: InstanceId,
    pub(crate) acquisition_dependencies: Arc<SelectedDependencies>,
    pub(crate) descriptions: Vec<Arc<super::portable::BodyDescription>>,
    owner: Arc<super::products::ProductOwner>,
    _metadata: Arc<AllocationLease>,
}
impl PreparedBasis {
    fn new(
        service: &MathService,
        preparation: ModelingPreparation,
        acquisition_dependencies: Arc<SelectedDependencies>,
    ) -> Result<Self, MathRuntimeError> {
        let bytes = size_of::<Self>() + 128 + acquisition_dependencies.retained_bytes();
        let metadata = service.reserve("math:prepared-basis-metadata", bytes)?;
        Ok(Self {
            root: preparation.solved.model(),
            instance: preparation.solved.instance(),
            product: preparation.product,
            owner: preparation._owner,
            acquisition_dependencies,
            descriptions: Vec::new(),
            _metadata: metadata,
        })
    }
    pub(crate) fn compiled(&self) -> &PreparedModeling {
        &self.product
    }
    pub(crate) fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + 128
            + self.product.retained_bytes()
            + self.acquisition_dependencies.retained_bytes()
            + self.descriptions.capacity() * size_of::<Arc<super::portable::BodyDescription>>()
            + self
                .descriptions
                .iter()
                .map(|description| description.retained_bytes())
                .sum::<usize>()
    }
    /// Attach caller-produced immutable material while sharing the mathematical owner once.
    pub(crate) fn with_descriptions(
        &self,
        service: &MathService,
        mut descriptions: Vec<Arc<super::portable::BodyDescription>>,
    ) -> Result<Self, MathRuntimeError> {
        descriptions.sort_unstable_by_key(|description| description.semantic_identity);
        let metadata = service.reserve(
            "math:prepared-basis-description-inventory",
            size_of::<Self>()
                + 128
                + self.acquisition_dependencies.retained_bytes()
                + descriptions.capacity() * size_of::<Arc<super::portable::BodyDescription>>(),
        )?;
        Ok(Self {
            product: self.product.clone(),
            root: self.root,
            instance: self.instance,
            acquisition_dependencies: self.acquisition_dependencies.clone(),
            descriptions,
            owner: self.owner.clone(),
            _metadata: metadata,
        })
    }
    pub(crate) fn description(
        &self,
        identity: pse_ids::roles::SemanticBodyHash,
    ) -> Option<&Arc<super::portable::BodyDescription>> {
        self.descriptions
            .binary_search_by_key(&identity, |description| description.semantic_identity)
            .ok()
            .map(|index| &self.descriptions[index])
    }
    /// Bind the current revision's lineage; consumed source versions are supplied afterward.
    pub(crate) fn bind(
        &self,
        revision: &ModelingRevision,
        root: DeclarationId,
        instance: InstanceId,
    ) -> Result<ModelingPreparation, MathRuntimeError> {
        if root != self.root || instance != self.instance {
            return Err(pse_math::MathError::Contract(
                "prepared basis has a different exact root or instance".into(),
            )
            .into());
        }
        Ok(ModelingPreparation {
            product: self.product.clone(),
            solved: revision.solved(root, instance)?,
            consumed_sources: Arc::new(BTreeMap::new()),
            _source_owner: None,
            _owner: self.owner.clone(),
        })
    }
}
fn flight_error(error: FlightError<MathRuntimeError>) -> MathRuntimeError {
    match error {
        FlightError::Load(error) => MathRuntimeError::Shared(error),
        FlightError::Capacity => MathRuntimeError::Limit("prepared basis flights"),
        FlightError::Retiring => MathRuntimeError::Retiring,
        FlightError::Panicked => {
            MathRuntimeError::Panic("pure modeling preparation task panic".into())
        }
    }
}
impl MathService {
    pub(crate) fn basis_key(
        &self,
        selected: &crate::workflow::modeling::SelectedAdmission,
        root: DeclarationId,
        instance: InstanceId,
        bindings: Bindings,
        limits: Limits,
    ) -> Result<Arc<BasisKey>, MathRuntimeError> {
        let bytes = size_of::<BasisKey>() + 128 + bindings_bytes(&bindings);
        let owner = self.reserve("math:exact-preparation-key", bytes)?;
        let mut key = BasisKey {
            request: selected.request.clone(),
            dependencies: selected.dependencies.clone(),
            root,
            instance,
            bindings,
            limits,
            prehash: 0,
            _validation_owner: selected._validation_owner.clone(),
            _registry_owner: selected._registry_owner.clone(),
            _selected_metadata: selected.metadata.clone(),
            _owner: owner,
        };
        let mut hash = rustc_hash::FxHasher::default();
        key.hash_fields(&mut hash);
        key.prehash = hash.finish();
        Ok(Arc::new(key))
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "the planner receives the exact specialization request and the current private cancellation driver"
    )]
    pub(crate) async fn plan_frontier(
        self: &Arc<Self>,
        workspace: Workspace,
        revision: ModelingRevision,
        root: DeclarationId,
        instance: InstanceId,
        bindings: Bindings,
        limits: Limits,
        driver: &crate::CancelSource,
    ) -> Result<OwnedModelingFrontier, MathRuntimeError> {
        if driver.token().is_cancelled() {
            return Err(MathRuntimeError::Cancelled);
        }
        let control = FlightCancellation::default();
        let operation = self.plan_frontier_controlled(
            workspace,
            revision,
            root,
            instance,
            bindings,
            limits,
            control.clone(),
            None,
        );
        tokio::pin!(operation);
        tokio::select! { result = &mut operation => result, () = driver.cancelled() => {
            control.cancel(); let _ = operation.await; Err(MathRuntimeError::Cancelled)
        }}
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "one pure worker owns the exact specialization request and loader cancellation"
    )]
    async fn plan_frontier_controlled(
        self: &Arc<Self>,
        workspace: Workspace,
        revision: ModelingRevision,
        root: DeclarationId,
        instance: InstanceId,
        bindings: Bindings,
        limits: Limits,
        control: FlightCancellation,
        deadline: Option<std::time::Instant>,
    ) -> Result<OwnedModelingFrontier, MathRuntimeError> {
        let solved = revision.solved(root, instance)?;
        let generation = self.modeling_cache.generation();
        let check = control.flag();
        let (product, owner) = self
            .job_retained_scoped(1, super::WITHIN_WORKSPACE, control, deadline, move |flag| {
                let _workspace_lease = workspace.lease;
                let mut compiler = workspace.compiler.lock().map_err(|_| {
                    MathRuntimeError::Infrastructure("compiler lock poisoned".into())
                })?;
                if flag.load(std::sync::atomic::Ordering::Acquire) {
                    return Err(MathRuntimeError::Cancelled);
                }
                compiler.publish_modeling_revision(revision.admitted.clone())?;
                let product =
                    compiler.plan_modeling_cancellable(root, instance, bindings, limits, flag)?;
                let bytes = size_of::<OwnedModelingFrontier>() + product.retained_bytes();
                Ok((product, bytes))
            })
            .await?;
        if check.load(std::sync::atomic::Ordering::Acquire) {
            return Err(MathRuntimeError::Cancelled);
        }
        Ok(OwnedModelingFrontier {
            product,
            solved,
            _owner: owner,
            key: None,
            generation,
        })
    }
    pub(crate) async fn plan_frontier_keyed(
        self: &Arc<Self>,
        key: Arc<BasisKey>,
        workspace: Workspace,
        revision: ModelingRevision,
        driver: &crate::CancelSource,
        deadline: std::time::Instant,
    ) -> Result<Arc<OwnedModelingFrontier>, MathRuntimeError> {
        if driver.token().is_cancelled() {
            return Err(MathRuntimeError::Cancelled);
        }
        if let Some(frontier) = self.modeling_cache.frontier(&key) {
            return Ok(frontier);
        }
        let service = self.clone();
        let generation = self.modeling_cache.generation();
        let operation = self
            .frontier_flights
            .load_owned(key.clone(), move |control| async move {
                if let Some(frontier) = service.modeling_cache.frontier(&key) {
                    return Ok(frontier);
                }
                let mut frontier = service
                    .plan_frontier_controlled(
                        workspace,
                        revision,
                        key.root,
                        key.instance,
                        key.bindings.clone(),
                        key.limits,
                        control,
                        Some(deadline),
                    )
                    .await?;
                frontier.key = Some(key.clone());
                frontier.generation = generation;
                let frontier = Arc::new(frontier);
                service
                    .modeling_cache
                    .retain_frontier(generation, key, frontier.clone());
                Ok(frontier)
            });
        tokio::pin!(operation);
        tokio::select! { result = &mut operation => result.map_err(flight_error), () = driver.cancelled() => Err(MathRuntimeError::Cancelled) }
    }
    pub(crate) async fn complete_frontier(
        self: &Arc<Self>,
        workspace: Workspace,
        frontier: Arc<OwnedModelingFrontier>,
        driver: &crate::CancelSource,
    ) -> Result<ModelingPreparation, MathRuntimeError> {
        if driver.token().is_cancelled() {
            return Err(MathRuntimeError::Cancelled);
        }
        let control = FlightCancellation::default();
        let operation =
            self.complete_frontier_controlled(workspace, frontier, control.clone(), None);
        tokio::pin!(operation);
        tokio::select! { result = &mut operation => result, () = driver.cancelled() => {
            control.cancel(); let _ = operation.await; Err(MathRuntimeError::Cancelled)
        }}
    }
    async fn complete_frontier_controlled(
        self: &Arc<Self>,
        workspace: Workspace,
        frontier: Arc<OwnedModelingFrontier>,
        control: FlightCancellation,
        deadline: Option<std::time::Instant>,
    ) -> Result<ModelingPreparation, MathRuntimeError> {
        let solved = frontier.solved;
        let check = control.flag();
        let (product, lease) = self
            .job_retained_scoped(1, super::WITHIN_WORKSPACE, control, deadline, move |flag| {
                let _workspace_lease = workspace.lease;
                let mut compiler = workspace.compiler.lock().map_err(|_| {
                    MathRuntimeError::Infrastructure("compiler lock poisoned".into())
                })?;
                if flag.load(std::sync::atomic::Ordering::Acquire) {
                    return Err(MathRuntimeError::Cancelled);
                }
                let product =
                    compiler.complete_modeling_cancellable(frontier.product.clone(), flag)?;
                let bytes = product.retained_bytes();
                Ok((product, bytes))
            })
            .await?;
        if check.load(std::sync::atomic::Ordering::Acquire) {
            return Err(MathRuntimeError::Cancelled);
        }
        let owner = self.own_modeling_product(&product, lease)?;
        Ok(ModelingPreparation {
            product: product.with_owner(owner.clone()),
            solved,
            consumed_sources: Arc::default(),
            _source_owner: None,
            _owner: owner,
        })
    }
    pub(crate) async fn complete_frontier_keyed(
        self: &Arc<Self>,
        key: Arc<BasisKey>,
        acquisition_dependencies: Arc<SelectedDependencies>,
        workspace: Workspace,
        frontier: Arc<OwnedModelingFrontier>,
        driver: &crate::CancelSource,
        deadline: std::time::Instant,
    ) -> Result<Arc<PreparedBasis>, MathRuntimeError> {
        if driver.token().is_cancelled() {
            return Err(MathRuntimeError::Cancelled);
        }
        if frontier.key.as_ref() != Some(&key) {
            return Err(pse_math::MathError::Contract(
                "completion frontier has a different exact preparation request".into(),
            )
            .into());
        }
        if let Some(basis) = self.modeling_cache.basis(&key)
            && basis.acquisition_dependencies == acquisition_dependencies
        {
            return Ok(basis);
        }
        let service = self.clone();
        let generation = frontier.generation;
        let operation = self.basis_flights.load_owned(
            (key.clone(), acquisition_dependencies.clone()),
            move |control| async move {
                if let Some(basis) = service.modeling_cache.basis(&key)
                    && basis.acquisition_dependencies == acquisition_dependencies
                {
                    return Ok(basis);
                }
                let model = service
                    .complete_frontier_controlled(workspace, frontier, control, Some(deadline))
                    .await?;
                let basis = Arc::new(PreparedBasis::new(
                    &service,
                    model,
                    acquisition_dependencies,
                )?);
                // This is immutable mathematics only. A current caller still qualifies and
                // publishes every required description under its own selected read.
                service
                    .modeling_cache
                    .retain_basis(generation, key, basis.clone());
                Ok(basis)
            },
        );
        tokio::pin!(operation);
        tokio::select! { result = &mut operation => result.map_err(flight_error), () = driver.cancelled() => Err(MathRuntimeError::Cancelled) }
    }
}
