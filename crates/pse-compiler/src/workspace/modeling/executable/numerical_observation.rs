// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Compiler-sealed numerical observation demand before symbolic materialization.
use super::*;
use pse_ids::{FrameSink, roles::SemanticBodyHash};
use pse_math::numerical_observation::{NumericalObservationCase, NumericalObservationInstance};
#[cfg(test)]
#[path = "numerical_observation_tests.rs"]
mod tests;

/// One original instance's semantic seal and complete passive binding. A normalized
/// arithmetic key is deliberately absent: it is not original scientific custody.
#[derive(Clone, Debug)]
pub struct NumericalObservationInstanceRequest {
    semantic_identity: SemanticBodyHash,
    binding: NumericalObservationInstance,
}
impl NumericalObservationInstanceRequest {
    /// Complete original compiler-issued semantic body seal.
    pub fn semantic_identity(&self) -> SemanticBodyHash {
        self.semantic_identity
    }
    /// Original physical and diagnostic binding, with no source-bearing math body.
    pub fn binding(&self) -> &NumericalObservationInstance {
        &self.binding
    }
    /// Complete original formal population, distinct from realized compact inputs.
    pub fn formal_count(&self) -> usize {
        self.binding.slots().len()
    }
    /// Selected original output ordinals in retained numerical order.
    pub fn outputs(&self) -> &[usize] {
        self.binding.outputs()
    }
    /// Requested original formal derivative axes in their exact order.
    pub fn coordinates(&self) -> &[usize] {
        self.binding.coordinates()
    }
    /// Complete current source/member/occurrence custody.
    pub fn attribution(&self) -> &pse_math::ExecutionAttribution {
        self.binding.attribution()
    }
}

/// Passive compiler seal, produced before `semantic_body` or typed math admission.
/// This type has no deserializer and grants no receiving resource permission.
#[derive(Clone, Debug)]
pub struct NumericalObservationRequest {
    key: ContentHash,
    request_bytes: Arc<[u8]>,
    deployment_physical: ContentHash,
    consumed_physical: ContentHash,
    environment: ContentHash,
    profile: Profile,
    binding: Arc<NumericalObservationCase>,
    instances: Vec<NumericalObservationInstanceRequest>,
    owner: Option<Arc<dyn pse_math::AllocationOwner>>,
}
impl NumericalObservationRequest {
    /// Attach an actual immutable metadata lease, excluded from scientific meaning.
    /// Shared requests copy only native descriptors; no build recipe/control is retained.
    pub fn with_owner(mut self: Arc<Self>, owner: Arc<dyn pse_math::AllocationOwner>) -> Arc<Self> {
        let value = Arc::make_mut(&mut self);
        value.binding = Arc::clone(&value.binding).with_owner(Arc::clone(&owner));
        value.owner = Some(pse_math::retain_allocation_owner(value.owner.take(), owner));
        self
    }
    /// Separately versioned durable request identity; never a local numerical key.
    pub fn key(&self) -> ContentHash {
        self.key
    }
    /// Complete deterministic request/custody framing bound by the durable key.
    pub fn request_bytes(&self) -> &[u8] {
        &self.request_bytes
    }
    /// Complete current deployment physical meaning, separate from consumed meaning.
    pub fn deployment_physical(&self) -> ContentHash {
        self.deployment_physical
    }
    /// Exact physical premises consumed by this original planning frontier.
    pub fn consumed_physical(&self) -> ContentHash {
        self.consumed_physical
    }
    /// Original selected library implementation and coefficient environment.
    pub fn environment(&self) -> ContentHash {
        self.environment
    }
    /// Explicit demanded derivative ceiling, including genuine Value-only requests.
    pub fn order(&self) -> DerivativeOrder {
        self.binding.order()
    }
    /// Local realization options. Native realization/module budgets do not alter
    /// the portable binary64 request identity and must qualify a local cache separately.
    pub fn profile(&self) -> Profile {
        self.profile
    }
    /// Exact mathematical product selected by this interpretation.
    pub fn product(&self) -> pse_math::jets::ProductKind {
        pse_math::jets::ProductKind::SelectedPartials
    }
    /// Ordered original physical rows and derivative coordinates, without a CasePlan.
    pub fn binding(&self) -> &Arc<NumericalObservationCase> {
        &self.binding
    }
    /// Original instance order, including independently qualified equal arithmetic bodies.
    pub fn instances(&self) -> &[NumericalObservationInstanceRequest] {
        &self.instances
    }
    /// Conservative owned descriptor extent, excluding opaque source build recipes.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.request_bytes.len()
            + self.binding.retained_bytes()
            + self
                .instances
                .iter()
                .map(|instance| {
                    size_of::<NumericalObservationInstanceRequest>()
                        + instance.binding.retained_bytes()
                })
                .sum::<usize>()
    }
}

/// Optional original source build recipe. It is separate from the passive seal,
/// is never decoded from an execution hit and preserves each original BodyInput.
#[derive(Clone, Debug)]
pub struct NumericalObservationBuildRecipe {
    request: ContentHash,
    inputs: Vec<grouped::ObservationBodyRecipe>,
}
impl NumericalObservationBuildRecipe {
    /// Conservative original source extent retained only for a possible cold miss.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self
                .inputs
                .iter()
                .map(grouped::ObservationBodyRecipe::retained_bytes)
                .sum::<usize>()
    }
}

/// One warm symbolic request qualified by the original instance's semantic seal.
/// This owner is never needed by a successful execution-only cold hit.
#[derive(Clone, Debug)]
pub struct NumericalObservationArtifact {
    semantic_identity: SemanticBodyHash,
    instance: SemanticId,
    request: ArtifactRequest,
}
impl NumericalObservationArtifact {
    /// Original semantic seal preserved before normalized-key grouping.
    pub fn semantic_identity(&self) -> SemanticBodyHash {
        self.semantic_identity
    }
    /// Original receiving instance, in the passive request's instance order.
    pub fn instance(&self) -> SemanticId {
        self.instance
    }
    /// Ordinary explicitly selected canonical binary64 artifact request.
    pub fn request(&self) -> &ArtifactRequest {
        &self.request
    }
}

fn request_bytes(
    binding: &NumericalObservationCase,
    instances: &[NumericalObservationInstanceRequest],
    deployment: ContentHash,
    consumed: ContentHash,
    environment: ContentHash,
    profile: Profile,
) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    // Preserve the pinned supplier conversion: fixed-width rational terms use
    // native conversion/division, while large terms use MPFR53. This does not
    // claim general correctly rounded rational-to-binary64 conversion.
    bytes.put_len_prefixed(b"numerical-observation-v1;selected-partials;pinned-rational-f64-and-fixed-args-mpfr53;checked-f64-stage;original-ordered-guards;canonical-coefficients");
    bytes.put_u64_le(pse_math::transfer::EXECUTION_SNAPSHOT_VERSION.into());
    for hash in [deployment, consumed, environment] {
        bytes.put_fixed(hash.as_bytes());
    }
    bytes.put_u64_le(binding.order() as u64);
    // The native realization kind and module allowance are local consumers of
    // the same canonical instructions, never portable producer requirements.
    for value in [
        profile.optimization.cores,
        profile.optimization.horner_iterations,
        profile.optimization.cpe_iterations,
        profile.evaluation.derivative_components,
        profile.evaluation.operations,
        profile.evaluation.scratch_bytes,
        profile.evaluation.provider_calls,
        profile.assembly.contributions,
        profile.assembly.native_index,
        profile.assembly.worker_bytes,
    ] {
        bytes.put_u64_le(value as u64);
    }
    bytes.put_u64_le(binding.inputs().variables().len() as u64);
    for variable in binding.inputs().variables().iter() {
        for id in [
            variable.port.id,
            variable.port.quantity.as_id(),
            variable.port.unit.as_id(),
        ] {
            bytes.put_fixed(id.as_bytes());
        }
        bytes.put_u64_le(u64::from(variable.fixed));
        bytes.put_len_prefixed(variable.domain.as_str().as_bytes());
        for value in [
            variable.lower.unwrap_or(f64::NEG_INFINITY),
            variable.upper.unwrap_or(f64::INFINITY),
        ] {
            bytes.put_u64_le(pse_ids::canonical_f64_bits(value));
        }
    }
    bytes.put_u64_le(binding.inputs().parameters().len() as u64);
    for port in binding.inputs().parameters() {
        for id in [port.id, port.quantity.as_id(), port.unit.as_id()] {
            bytes.put_fixed(id.as_bytes());
        }
    }
    bytes.put_u64_le(binding.rows().len() as u64);
    for row in binding.rows() {
        bytes.put_fixed(row.id.as_bytes());
        bytes.put_fixed(row.quantity.as_id().as_bytes());
        bytes.put_u64_le(pse_ids::canonical_f64_bits(row.lower));
        bytes.put_u64_le(pse_ids::canonical_f64_bits(row.upper));
    }
    bytes.put_u64_le(binding.coordinates().len() as u64);
    for id in binding.coordinates() {
        bytes.put_fixed(id.as_bytes());
    }
    bytes.put_u64_le(instances.len() as u64);
    for instance in instances {
        bytes.put_fixed(instance.semantic_identity.as_id().as_bytes());
        let attribution = instance.attribution();
        // The exact source metadata has no floating-point payload. Typed record
        // field order and BTreeMap key order make this complete native custody
        // framing deterministic; no serialized record can construct this seal.
        let maps = pse_math::ExecutionAttributionSnapshot {
            instance: attribution.instance(),
            checked_members: attribution.checked_members().clone(),
            checked_sources: attribution.checked_sources().clone(),
            occurrences: attribution.occurrences().clone(),
        };
        bytes.put_len_prefixed(&serde_json::to_vec(&maps).map_err(|cause| {
            CompileError::Missing(format!("numerical source custody: {cause}"))
        })?);
        bytes.put_u64_le(0); // This interpretation admits no providers.
        bytes.put_u64_le(instance.binding.slots().len() as u64);
        for slot in instance.binding.slots() {
            let port = slot.source_port();
            for id in [port.id, port.quantity.as_id(), port.unit.as_id()] {
                bytes.put_fixed(id.as_bytes());
            }
            bytes.put_u64_le(pse_ids::canonical_f64_bits(slot.scale()));
            bytes.put_u64_le(pse_ids::canonical_f64_bits(slot.offset()));
        }
        for ordinals in [instance.outputs(), instance.coordinates()] {
            bytes.put_u64_le(ordinals.len() as u64);
            for &ordinal in ordinals {
                bytes.put_u64_le(ordinal as u64);
            }
        }
        bytes.put_u64_le(instance.binding.contributions().len() as u64);
        for part in instance.binding.contributions() {
            let Target::Row(row) = part.target else {
                return Err(CompileError::Missing(
                    "numerical observation objective".into(),
                ));
            };
            bytes.put_u64_le(part.output as u64);
            bytes.put_fixed(row.as_bytes());
            bytes.put_u64_le(pse_ids::canonical_f64_bits(part.scale));
        }
    }
    Ok(bytes)
}

impl CompilerWorkspace {
    /// Seal selected original observation custody before mathematical body admission.
    /// `None` denotes unsupported providers or unqualified original source custody,
    /// selected before any persisted execution lookup; it never hides an invalid hit.
    pub fn numerical_observation_request(
        &mut self,
        frontier: &ModelingPreparationFrontier,
        rows: Vec<SemanticId>,
        coordinates: Vec<SemanticId>,
        order: DerivativeOrder,
        profile: Profile,
        cancel: Arc<AtomicBool>,
    ) -> Result<
        Option<(
            Arc<NumericalObservationRequest>,
            NumericalObservationBuildRecipe,
        )>,
    > {
        if cancel.load(Ordering::Acquire) {
            return Err(CompileError::Cancelled);
        }
        self.db.cancel = cancel;
        let compatible = match &frontier.physical_premises {
            Some(premises) => premises.observe(&self.inputs.quantities, &self.inputs.preconditions),
            None => {
                frontier.physical
                    == complete_context(
                        &self.inputs.quantities,
                        &self.inputs.preconditions,
                        &frontier.projection.physical_scope,
                    )
            }
        };
        if !compatible || frontier.environment != *self.inventory.environment(&self.db) {
            return Err(MathError::Contract(
                "numerical frontier has a different current physical or implementation context"
                    .into(),
            )
            .into());
        }
        let Some(selected) = frontier
            .primary
            .numerical_observation(&rows, &coordinates)?
        else {
            return Ok(None);
        };
        for recipe in &selected.recipes {
            if !recipe.observe(&self.inputs.quantities, &self.inputs.preconditions)? {
                return Err(MathError::Contract(
                    "numerical original physical premises changed before request sealing".into(),
                )
                .into());
            }
        }
        let binding = Arc::new(NumericalObservationCase::new(
            selected.variables,
            selected.parameters,
            selected.rows,
            coordinates,
            selected.instances,
            order,
            &self.inputs.quantities,
            profile.assembly,
        )?);
        let instances = selected
            .recipes
            .iter()
            .zip(binding.instances())
            .map(|(recipe, binding)| NumericalObservationInstanceRequest {
                semantic_identity: recipe.identity(),
                binding: binding.clone(),
            })
            .collect::<Vec<_>>();
        let deployment_physical =
            crate::physical_identity::complete(&self.inputs.quantities, &self.inputs.preconditions);
        let bytes = request_bytes(
            &binding,
            &instances,
            deployment_physical,
            frontier.physical,
            frontier.environment,
            profile,
        )?;
        let mut hash = FramedHasher::new(pse_ids::Frame::NumericalObservationRequestV1);
        hash.part(&bytes);
        let key = hash.finish_hash();
        let recipe = NumericalObservationBuildRecipe {
            request: key,
            inputs: selected.recipes,
        };
        let request = Arc::new(NumericalObservationRequest {
            key,
            request_bytes: bytes.into(),
            deployment_physical,
            consumed_physical: frontier.physical,
            environment: frontier.environment,
            profile,
            binding,
            instances,
            owner: None,
        });
        if self.db.cancel.load(Ordering::Acquire) {
            return Err(CompileError::Cancelled);
        }
        Ok(Some((request, recipe)))
    }
    /// Materialize only on a numerical execution miss, preserving each original
    /// semantic seal before any equal normalized body keys can be grouped.
    pub fn compile_numerical_observation(
        &mut self,
        request: &NumericalObservationRequest,
        recipe: &NumericalObservationBuildRecipe,
        cancel: Arc<AtomicBool>,
    ) -> Result<Vec<NumericalObservationArtifact>> {
        if cancel.load(Ordering::Acquire) {
            return Err(CompileError::Cancelled);
        }
        self.db.cancel = cancel;
        if request.key != recipe.request
            || request.instances.len() != recipe.inputs.len()
            || request.deployment_physical
                != crate::physical_identity::complete(
                    &self.inputs.quantities,
                    &self.inputs.preconditions,
                )
            || request.environment != *self.inventory.environment(&self.db)
        {
            return Err(MathError::Contract(
                "numerical warm recipe request/context mismatch".into(),
            )
            .into());
        }
        let roots = recipe
            .inputs
            .iter()
            .map(grouped::ObservationBodyRecipe::identity)
            .collect::<BTreeSet<_>>()
            .len();
        let root_bytes = request
            .retained_bytes()
            .checked_add(recipe.retained_bytes())
            .ok_or(CompileError::Limit("numerical observation live root"))?;
        if !self.live_root_fits(roots.max(self.limits.query_values), root_bytes) {
            return Err(CompileError::Limit("numerical observation live root"));
        }
        let (mut capacity, mut count, mut working) = self.live_root_working_set(roots, root_bytes);
        if !self.live_root_fits(count.max(capacity), working) {
            let current_cancel = Arc::clone(&self.db.cancel);
            self.rebuild(self.inputs.clone())?;
            self.db.cancel = current_cancel;
            (capacity, count, working) = self.live_root_working_set(roots, root_bytes);
        }
        if !self.live_root_fits(count.max(capacity), working) {
            return Err(CompileError::Limit("numerical observation live root"));
        }
        self.accept_live_root(capacity, working);
        let mut profile = request.profile;
        profile.optimization.evaluator = pse_math::library::Evaluator::Interpreted;
        let result = salsa::Cancelled::catch(std::panic::AssertUnwindSafe(|| {
            recipe
                .inputs
                .iter()
                .zip(&request.instances)
                .map(|(input, instance)| {
                    checkpoint(&self.db);
                    if input.identity() != instance.semantic_identity
                        || !input.observe(&self.inputs.quantities, &self.inputs.preconditions)?
                    {
                        return Err(MathError::Contract(
                            "numerical original build recipe changed".into(),
                        )
                        .into());
                    }
                    let body = input.materialize(&self.db)?;
                    pse_math::SourceBindings::new(
                        instance.attribution().instance(),
                        &body.math,
                        instance.attribution().checked_members().clone(),
                        instance.attribution().checked_sources().clone(),
                        instance.attribution().occurrences().clone(),
                    )?;
                    let support = Arc::new(body.math.prepare_support(
                        instance.outputs(),
                        instance.coordinates(),
                        request.order(),
                        self.db.cancel(),
                    )?);
                    let demand = LocalDemand {
                        body: body.spec.key(),
                        outputs: instance.outputs().to_vec(),
                        coordinates: instance.coordinates().to_vec(),
                        order: request.order(),
                        product: pse_math::jets::ProductKind::SelectedPartials,
                    };
                    Ok(NumericalObservationArtifact {
                        semantic_identity: input.identity(),
                        instance: instance.attribution().instance(),
                        request: artifact_request(
                            &demand,
                            support,
                            profile,
                            &request.environment,
                            false,
                        ),
                    })
                })
                .collect::<Result<Vec<_>>>()
        }))
        .map_err(|_| compiler_cancelled(&self.db))
        .and_then(std::convert::identity);
        self.trim_queries()?;
        result
    }
}
