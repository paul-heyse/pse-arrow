// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use pse_kernels::{AdmittedProvider, Port, ProviderSpec};
pub use pse_math::implicit::SelectionEquivalence;
use pse_modeling::resolved::{
    Node, NodeIx, PredicateIx, Reference, RegionOutput, RegionRequest, ResolvedPredicate,
    SelectedRegion,
};
/// Authored/admitted mathematical meaning, independent of a numerical initial guess.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImplicitMeaning {
    /// Original residual relation; expression use needs selection or uniqueness evidence.
    Relation,
    /// Library-established unique graph, or the separately proven affine-rate operation.
    Unique,
    /// Explicitly restricted mathematical branch.
    Branch,
    /// Semantic anchor and algorithm/settings reference.
    Operational(String),
    /// Authored minimum-score selection among alternative regimes.
    MinimumScore,
}
/// How the selected function's local derivative neighborhood is established.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionNeighborhood {
    /// No supported uniqueness/selection evidence beyond values.
    Unestablished,
    /// Existing library-established graph or explicitly admitted branch meaning.
    Static,
    /// A validated winning chart must exclude competitive roots across all alternatives.
    RuntimeIsolation,
}
/// One admitted implicit body and its exact current receiving source correspondence.
/// The reusable numerical body itself contains only local source-role tokens.
#[derive(Clone, Debug, PartialEq)]
pub struct ImplicitBody {
    body: Arc<AdmittedBody>,
    bindings: pse_math::SourceBindings,
}
impl std::ops::Deref for ImplicitBody {
    type Target = AdmittedBody;
    fn deref(&self) -> &Self::Target {
        &self.body
    }
}
impl ImplicitBody {
    fn compiled(&self, body: Arc<CompiledBody>) -> Result<pse_math::SourceBody> {
        Ok(pse_math::SourceBody::new(body, self.bindings.clone())?)
    }
}
/// One compiler-issued selector, consumed by evaluation, native realization and export.
#[derive(Clone, Debug, PartialEq)]
pub struct ImplicitSelection {
    /// Selected scientific operation identity; excludes incidental numerical hints.
    pub identity: ContentHash,
    /// Relation or selected function contract.
    pub meaning: ImplicitMeaning,
    /// Semantic anchors in unknown output order, independent of numerical starts.
    pub anchors: Option<Arc<ImplicitBody>>,
    /// Physical branch/neighborhood predicate encoded as a scalar indicator.
    pub restriction: Option<Arc<ImplicitBody>>,
    /// Supported scalar half-line restriction; true is positive.
    pub sign: Option<bool>,
    /// Whether the recognized half-line excludes its zero endpoint.
    pub strict: bool,
    /// Checked equivalence of residual plus restrictions to the selected graph.
    pub equivalence: SelectionEquivalence,
    /// Justified selector-neighborhood derivative capability.
    pub neighborhood: DerivativeOrder,
    /// Proof mechanism; a declared runtime obligation is not a completed certificate.
    pub neighborhood_evidence: SelectionNeighborhood,
}
#[derive(Clone, Debug, PartialEq)]
struct SelectionProjection {
    identity: ContentHash,
    meaning: ImplicitMeaning,
    anchors: Vec<SelectedRegion>,
    restriction: Option<SelectedRegion>,
    sign: Option<bool>,
    strict: bool,
}
fn sign_restriction(
    model: &SpecializedModel,
    predicate: PredicateIx,
    unknowns: &[SemanticId],
) -> Option<bool> {
    use dsl::CompareOp;
    let ResolvedPredicate::Compare { op, lhs, rhs } = model.resolved.predicates.get(predicate)?
    else {
        return None;
    };
    if unknowns.len() != 1 {
        return None;
    }
    let is_unknown = |root: NodeIx| matches!(model.resolved.nodes.get(root),Some(Node::Reference(Reference::Symbol(slot))) if model.symbols.at(*slot).is_some_and(|symbol|symbol.id==unknowns[0]));
    let is_zero = |root: NodeIx| matches!(model.resolved.nodes.get(root),Some(Node::Number(number)) if number.value==0. && number.unit.is_none());
    match (
        is_unknown(*lhs),
        is_zero(*rhs),
        is_zero(*lhs),
        is_unknown(*rhs),
        op,
    ) {
        (true, true, _, _, CompareOp::Gt | CompareOp::Ge)
        | (_, _, true, true, CompareOp::Lt | CompareOp::Le) => Some(true),
        (true, true, _, _, CompareOp::Lt | CompareOp::Le)
        | (_, _, true, true, CompareOp::Gt | CompareOp::Ge) => Some(false),
        _ => None,
    }
}

/// Algorithm selected by source realization or by the compiler's affine rate proof.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImplicitAlgorithm {
    /// Nested solve by the shared native inner solver.
    Native,
    /// Registered accelerator named by the source realization.
    Accelerator(String),
    /// Closed-form solve of rates proven affine by the compiler.
    AffineRates,
}
impl ImplicitAlgorithm {
    fn retained_bytes(&self) -> usize {
        if let Self::Accelerator(id) = self {
            id.capacity()
        } else {
            0
        }
    }
    fn key(&self) -> String {
        match self {
            Self::Native => "nested".into(),
            Self::Accelerator(id) => format!("accelerator:{id}"),
            Self::AffineRates => "affine-rates".into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct AssessmentProjection {
    eligibility: SelectedRegion,
    criterion: Vec<SelectedRegion>,
    quantities: Vec<QuantityTypeId>,
}
/// Selected scaling declaration and its original term range in the nominal program.
#[derive(Clone, Debug, PartialEq)]
pub struct ImplicitScale {
    /// Equation row the scaling applies to.
    pub row: SemanticId,
    /// Declaration that selected the scaling.
    pub source: DeclarationId,
    /// Declared scaling scheme.
    pub scheme: pse_model::generated::enums::ConstraintScalingScheme,
    /// The row's original terms within the residual's nominal-term body.
    pub terms: std::ops::Range<usize>,
}
#[derive(Clone, Debug, PartialEq)]
struct ResidualProjection {
    id: SemanticId,
    rows: Vec<SemanticId>,
    regions: Vec<SelectedRegion>,
    quantities: Vec<QuantityTypeId>,
    assessment: Option<AssessmentProjection>,
    hints: Vec<(
        SemanticId,
        DeclarationId,
        ModelingHint,
        SelectedRegion,
        QuantityTypeId,
    )>,
    terms: Vec<(SelectedRegion, QuantityTypeId)>,
    scales: Vec<ImplicitScale>,
}
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Projection {
    selection: SelectionProjection,
    algorithm: ImplicitAlgorithm,
    pub(super) id: SemanticId,
    unknowns: Vec<SemanticId>,
    inputs: Vec<SemanticId>,
    residuals: Vec<ResidualProjection>,
    spec: ProviderSpec,
}
impl Projection {
    pub(super) fn annotation_variables(&self) -> impl Iterator<Item = SemanticId> + '_ {
        self.unknowns.iter().copied()
    }
    pub(super) fn annotation_rows(
        &self,
    ) -> impl Iterator<Item = (usize, SemanticId, QuantityTypeId, &SelectedRegion)> + '_ {
        self.residuals.iter().flat_map(|residual| {
            residual
                .rows
                .iter()
                .zip(&residual.quantities)
                .zip(&residual.regions)
                .enumerate()
                .map(|(position, ((id, quantity), region))| (position, *id, *quantity, region))
        })
    }
    pub(super) fn annotation_inventory(&self) -> impl Iterator<Item = SemanticId> + '_ {
        self.unknowns.iter().copied().chain(
            self.residuals
                .iter()
                .flat_map(|residual| residual.rows.iter().copied()),
        )
    }
    pub(super) fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            .saturating_add(self.algorithm.retained_bytes())
            .saturating_add(
                self.inputs
                    .capacity()
                    .saturating_add(self.unknowns.capacity())
                    .saturating_mul(size_of::<SemanticId>()),
            )
            .saturating_add(
                self.selection
                    .anchors
                    .iter()
                    .chain(self.selection.restriction.iter())
                    .map(SelectedRegion::retained_bytes)
                    .sum::<usize>(),
            )
            .saturating_add(match &self.selection.meaning {
                ImplicitMeaning::Operational(value) => value.capacity(),
                _ => 0,
            })
            .saturating_add(
                self.residuals
                    .iter()
                    .map(|residual| {
                        size_of::<ResidualProjection>()
                            + residual.rows.capacity() * size_of::<SemanticId>()
                            + residual.quantities.capacity() * size_of::<QuantityTypeId>()
                            + residual
                                .regions
                                .iter()
                                .map(SelectedRegion::retained_bytes)
                                .sum::<usize>()
                            + residual
                                .hints
                                .iter()
                                .map(|hint| hint.3.retained_bytes())
                                .sum::<usize>()
                            + residual
                                .terms
                                .iter()
                                .map(|term| term.0.retained_bytes())
                                .sum::<usize>()
                            + residual.assessment.as_ref().map_or(0, |assessment| {
                                assessment.eligibility.retained_bytes()
                                    + assessment
                                        .criterion
                                        .iter()
                                        .map(SelectedRegion::retained_bytes)
                                        .sum::<usize>()
                            })
                            + residual.scales.capacity() * size_of::<ImplicitScale>()
                    })
                    .sum::<usize>(),
            )
    }
}
/// The criterion and eligibility are separately demanded: an ineligible branch has no score.
#[derive(Clone, Debug, PartialEq)]
pub struct RegimeAssessment {
    pub eligibility: Arc<ImplicitBody>,
    pub criterion: Arc<ImplicitBody>,
}
/// A root problem, with independent equation identities and optional regime assessment.
#[derive(Clone, Debug, PartialEq)]
pub struct AdmittedResidual {
    pub id: SemanticId,
    pub body: Arc<ImplicitBody>,
    pub rows: Vec<SemanticId>,
    pub assessment: Option<RegimeAssessment>,
    /// Selected common/local hints in the residual's lexical scope and formal order.
    pub hints: Option<Arc<ImplicitBody>>,
    pub terms: Option<Arc<ImplicitBody>>,
    pub scales: Vec<ImplicitScale>,
    /// Compiler-owned implicit mode interpretation of these exact ordered hints.
    pub numerical_annotations: Arc<NumericalAnnotations>,
    /// Exact residual, eligibility and criterion for conditional selection certification.
    pub isolation: Option<Arc<pse_math::factorable::RootIsolationProgram>>,
}
/// Injected mathematics and algorithm registrations for one implicit execution.
#[derive(Clone, Debug)]
pub struct ImplicitCapabilities<'a> {
    /// Numerical root solver used by the native algorithm.
    pub solver: Arc<dyn pse_math::implicit::InnerSolver>,
    /// Validated global minimum selection evidence, when nonlinear derivatives need it.
    pub verifier: Option<Arc<dyn pse_math::implicit::SelectionVerifier>>,
    /// Attempt's admitted named accelerator registrations.
    pub accelerators: &'a pse_math::implicit::accelerators::Accelerators,
}
/// Compiler-issued inner demand, independent of case-specific numerical hints.
#[derive(Clone, Debug)]
pub struct ImplicitProgramRequest {
    key: (ContentHash, u8, Vec<usize>, bool),
    artifact: ArtifactRequest,
    source: SemanticId,
}
impl ImplicitProgramRequest {
    /// Use the existing admitted artifact/scalar service and its retention policy.
    pub fn artifact(&self) -> &ArtifactRequest {
        &self.artifact
    }
    /// Reattach the actual consuming residual to a shared compilation refusal.
    /// Other error kinds already retain their original scientific source/cause.
    pub fn attribute_refusal(&self, cause: &MathError) -> Option<MathError> {
        if let MathError::DerivativeDemand {
            outputs,
            coordinates,
            requested,
            available,
            ..
        } = cause
        {
            Some(MathError::DerivativeDemand {
                source_id: Some(self.source),
                body: Some(self.key.0),
                outputs: outputs.clone(),
                coordinates: coordinates.clone(),
                requested: *requested,
                available: *available,
            })
        } else {
            None
        }
    }
    /// Request descriptors retained across asynchronous compilation.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.key.2.capacity() * size_of::<usize>()
            + self.artifact.descriptor_bytes()
            + self.artifact.support.retained_bytes()
            + pse_model::allocation::btree_node_bytes::<
                (ContentHash, u8, Vec<usize>, bool),
                Arc<CompiledBody>,
            >()
            + pse_model::allocation::btree_node_bytes::<usize, ()>()
    }
}
/// One assembly's exact programs and admitted solver descriptors. Mutable native,
/// provider, numerical configuration and cancellation state stays in the fresh factory.
#[derive(Debug)]
pub struct ImplicitPrograms {
    limits: EvaluationLimits,
    bodies: BTreeMap<(ContentHash, u8, Vec<usize>, bool), Arc<CompiledBody>>,
    solvers: BTreeMap<SemanticId, Arc<dyn pse_math::implicit::InnerSolver>>,
}
impl ImplicitPrograms {
    /// Start an empty assembly under the exact requested numeric limits.
    fn new(limits: EvaluationLimits) -> Self {
        Self {
            limits,
            bodies: BTreeMap::new(),
            solvers: BTreeMap::new(),
        }
    }
    /// Assembly-local solver descriptors remain charged across artifact waits.
    pub fn descriptor_bytes(&self) -> usize {
        size_of::<Self>()
            + self
                .solvers
                .values()
                .map(|solver| {
                    pse_model::allocation::btree_node_bytes::<
                        SemanticId,
                        Arc<dyn pse_math::implicit::InnerSolver>,
                    >() + solver.retained_bytes()
                })
                .sum::<usize>()
    }
    /// Bind the result of the compiler-issued request, preserving its artifact owner.
    pub fn bind(&mut self, request: ImplicitProgramRequest, body: Arc<CompiledBody>) -> Result<()> {
        if request.artifact.evaluation != self.limits
            || body.compiled_order() != request.artifact.demand.order
            || body.coordinates() != request.artifact.demand.coordinates
            || body.prepared_support().outputs().len() != request.artifact.demand.outputs.len()
        {
            return Err(CompileError::Missing(
                "implicit artifact binding contract".into(),
            ));
        }
        self.bodies.insert(request.key, body);
        Ok(())
    }
    fn get(
        &self,
        body: &AdmittedBody,
        coordinates: &[usize],
        order: DerivativeOrder,
        local: bool,
    ) -> Result<Arc<CompiledBody>> {
        self.bodies
            .get(&(body.spec.key(), order as u8, coordinates.to_vec(), local))
            .cloned()
            .ok_or_else(|| CompileError::Missing("implicit compiler-issued program missing".into()))
    }
    /// Actual separately charged immutable allocations borrowed by this assembly.
    pub fn allocation_identities(&self) -> BTreeSet<usize> {
        self.bodies
            .values()
            .map(|body| Arc::as_ptr(body) as usize)
            .collect()
    }
}
/// Checked implicit systems over a single ordered unknown set.
#[derive(Clone, Debug, PartialEq)]
pub struct AdmittedImplicit {
    /// Compiler-owned mathematical selection shared across realizations.
    pub selection: ImplicitSelection,
    /// Algorithm that solves the system.
    pub algorithm: ImplicitAlgorithm,
    /// Provider descriptor through which the solved unknowns are called.
    pub descriptor: AdmittedProvider,
    /// Unknowns in the provider's output order.
    pub unknowns: Vec<SemanticId>,
    /// Alternative root problems over the same unknowns.
    pub residuals: Vec<AdmittedResidual>,
    provider_lookups: BTreeMap<String, ProviderCall>,
}
impl AdmittedImplicit {
    pub(super) fn provider_lookups(&self) -> &BTreeMap<String, ProviderCall> {
        &self.provider_lookups
    }
    pub(super) fn owner_attachment_bytes(&self) -> usize {
        size_of::<Self>()
            + 128
            + self.unknowns.len() * size_of::<SemanticId>()
            + self.algorithm.retained_bytes()
            + self
                .provider_lookups
                .iter()
                .map(|(name, provider)| {
                    name.capacity()
                        + size_of::<(String, ProviderCall)>()
                        + 128
                        + provider.descriptor.spec().shapes.retained_bytes()
                        + size_of_val(provider.descriptor.spec().inputs.as_slice())
                        + size_of_val(provider.descriptor.spec().outputs.as_slice())
                })
                .sum::<usize>()
            + match &self.selection.meaning {
                ImplicitMeaning::Operational(value) => value.len(),
                _ => 0,
            }
            + self.residuals.len() * size_of::<AdmittedResidual>()
            + self
                .residuals
                .iter()
                .map(|r| {
                    r.rows.len() * size_of::<SemanticId>()
                        + r.scales.len() * size_of::<ImplicitScale>()
                        + r.numerical_annotations.hints.len()
                            * size_of::<(SemanticId, DeclarationId, ModelingHint)>()
                })
                .sum::<usize>()
            + self
                .bodies()
                .map(|body| body.owner_attachment_bytes())
                .sum::<usize>()
    }
    pub(super) fn with_owner(mut self, owner: Arc<dyn pse_math::AllocationOwner>) -> Self {
        let attach = |body: &mut Arc<ImplicitBody>| {
            *body = Arc::new(ImplicitBody {
                body: Arc::new(body.body.as_ref().clone().with_owner(owner.clone())),
                bindings: body.bindings.clone(),
            });
        };
        for body in self
            .selection
            .anchors
            .iter_mut()
            .chain(self.selection.restriction.iter_mut())
        {
            attach(body);
        }
        for residual in &mut self.residuals {
            attach(&mut residual.body);
            for body in residual.hints.iter_mut().chain(residual.terms.iter_mut()) {
                attach(body);
            }
            if let Some(assessment) = &mut residual.assessment {
                attach(&mut assessment.eligibility);
                attach(&mut assessment.criterion);
            }
        }
        self
    }
    fn residual_requirements(
        &self,
        residual: &AdmittedResidual,
        requested_output: DerivativeOrder,
        inner_minimum: DerivativeOrder,
    ) -> Result<pse_kernels::DerivativeRequirements> {
        let available = residual.body.math.available_order_for_outputs(
            &(0..residual.body.math.output_count()).collect::<Vec<_>>(),
            &(0..residual.body.math.input_count()).collect::<Vec<_>>(),
        )?;
        pse_kernels::DerivativeRequirements::new(
            available,
            available,
            self.selection.neighborhood,
            inner_minimum,
            requested_output,
        )
        .map_err(|cause| self.derivative_refusal(cause))
    }
    fn derivative_refusal(&self, mut cause: pse_kernels::ProviderError) -> CompileError {
        if let pse_kernels::ProviderError::DerivativeUnavailable { members, .. } = &mut cause {
            members.clone_from(&self.unknowns);
        }
        MathError::Provider {
            source_id: self.descriptor.spec().id,
            provider: self.descriptor.spec().id,
            cause,
        }
        .into()
    }
    /// Known capability/dependency analysis population before numerical order is
    /// resolved. Registered accelerator admission has no source-allocation contract;
    /// it and opaque body regions retain conservative runtime admission.
    pub fn requirements_allocation_bound(&self) -> Result<Option<usize>> {
        if matches!(self.algorithm, ImplicitAlgorithm::Accelerator(_)) {
            return Ok(None);
        }
        let overflow = || CompileError::from(MathError::Limit("implicit analysis extent"));
        let mut bytes = self
            .residuals
            .len()
            .checked_mul(1024)
            .ok_or_else(overflow)?;
        for body in self.bodies() {
            let support = if self.algorithm == ImplicitAlgorithm::AffineRates
                && self
                    .residuals
                    .iter()
                    .any(|residual| Arc::ptr_eq(&residual.body.body, body))
            {
                Some(self.unknowns.len())
            } else {
                None
            };
            let Some(bound) = body.math.arithmetic_analysis_allocation_bound(support)? else {
                return Ok(None);
            };
            bytes = bytes.checked_add(bound).ok_or_else(overflow)?;
        }
        Ok(Some(bytes))
    }
    /// Resolve the selected algorithms' minimum and the shared residual/output capability.
    /// Called under the library allocation owner before dependencies receive their demands.
    pub fn requirements(
        &self,
        requested_output: DerivativeOrder,
        native_minimum: DerivativeOrder,
        verifier: Option<&dyn pse_math::implicit::SelectionVerifier>,
        accelerators: &pse_math::implicit::accelerators::Accelerators,
        cancel: &Arc<AtomicBool>,
        limits: EvaluationLimits,
    ) -> Result<pse_kernels::DerivativeRequirements> {
        let mut available = None;
        let mut minimum = DerivativeOrder::Value;
        for residual in &self.residuals {
            let order = residual.body.math.available_order_for_outputs(
                &(0..residual.body.math.output_count()).collect::<Vec<_>>(),
                &(0..residual.body.math.input_count()).collect::<Vec<_>>(),
            )?;
            available = Some(available.map_or(order, |a: DerivativeOrder| a.min(order)));
            let inner = match &self.algorithm {
                ImplicitAlgorithm::Native => native_minimum,
                ImplicitAlgorithm::AffineRates => {
                    use pse_math::implicit::InnerSolver;
                    pse_math::implicit::Affine::new(&residual.body.math, self.unknowns.len())?
                        .minimum_order()
                }
                ImplicitAlgorithm::Accelerator(id) => accelerators
                    .admit(id, &residual.body.math, self.unknowns.len(), limits, cancel)?
                    .minimum_order(),
            };
            minimum = minimum.max(inner);
        }
        let available =
            available.ok_or_else(|| CompileError::Missing("empty implicit residuals".into()))?;
        pse_kernels::DerivativeRequirements::new(
            available,
            available,
            if self.selection.neighborhood_evidence == SelectionNeighborhood::RuntimeIsolation
                && verifier.is_none()
            {
                DerivativeOrder::Value
            } else {
                self.selection.neighborhood
            },
            minimum,
            requested_output,
        )
        .map_err(|cause| self.derivative_refusal(cause))
    }
    /// Actual dependency demands for residual, numerical and selector programs.
    pub fn provider_demands(
        &self,
        requirements: pse_kernels::DerivativeRequirements,
    ) -> Result<BTreeMap<pse_kernels::ProviderKey, DerivativeOrder>> {
        let mut demands = BTreeMap::new();
        let mut add = |body: &AdmittedBody, coordinates: &[usize], order| -> Result<()> {
            for (key, required) in body.math.provider_demands_for_selection(
                &(0..body.math.output_count()).collect::<Vec<_>>(),
                coordinates,
                order,
            )? {
                demands
                    .entry(key)
                    .and_modify(|o: &mut DerivativeOrder| *o = (*o).max(required))
                    .or_insert(required);
            }
            Ok(())
        };
        let local = if requirements.requested_output > DerivativeOrder::Value {
            DerivativeOrder::First
        } else {
            DerivativeOrder::Value
        };
        for residual in &self.residuals {
            add(
                &residual.body,
                &(0..self.unknowns.len()).collect::<Vec<_>>(),
                requirements.inner_minimum,
            )?;
            add(
                &residual.body,
                &(0..residual.body.math.input_count()).collect::<Vec<_>>(),
                requirements.requested_output,
            )?;
            for body in residual.hints.iter().chain(&residual.terms) {
                add(body, &[], DerivativeOrder::Value)?;
            }
            if let Some(assessment) = &residual.assessment {
                add(
                    &assessment.eligibility,
                    &(0..assessment.eligibility.math.input_count()).collect::<Vec<_>>(),
                    local,
                )?;
                add(
                    &assessment.criterion,
                    &(0..assessment.criterion.math.input_count()).collect::<Vec<_>>(),
                    local,
                )?;
            }
        }
        if let Some(body) = &self.selection.anchors {
            add(body, &[], DerivativeOrder::Value)?;
        }
        if let Some(body) = &self.selection.restriction {
            add(
                body,
                &(0..body.math.input_count()).collect::<Vec<_>>(),
                local,
            )?;
        }
        Ok(demands)
    }
    fn source_bodies(&self) -> impl Iterator<Item = &Arc<ImplicitBody>> {
        self.residuals
            .iter()
            .flat_map(|r| {
                std::iter::once(&r.body)
                    .chain(r.hints.iter())
                    .chain(r.terms.iter())
                    .chain(
                        r.assessment
                            .iter()
                            .flat_map(|a| [&a.eligibility, &a.criterion]),
                    )
            })
            .chain(self.selection.anchors.iter())
            .chain(self.selection.restriction.iter())
    }
    /// Every reusable admitted body; its receiving bindings remain on the actual slot.
    pub fn bodies(&self) -> impl Iterator<Item = &Arc<AdmittedBody>> {
        self.source_bodies().map(|body| &body.body)
    }
    /// Approximate heap bytes retained by the system, for cache accounting.
    pub fn retained_bytes(&self) -> usize {
        self.source_bodies()
            .map(|body| size_of::<ImplicitBody>().saturating_add(body.bindings.retained_bytes()))
            .sum::<usize>()
            + size_of_val(self.unknowns.as_slice())
            + self.algorithm.retained_bytes()
            + match &self.selection.meaning {
                ImplicitMeaning::Operational(settings) => settings.capacity(),
                _ => 0,
            }
            + self
                .residuals
                .iter()
                .map(|r| {
                    size_of_val(r.rows.as_slice())
                        + r.numerical_annotations.retained_bytes()
                        + size_of_val(r.scales.as_slice())
                        + size_of::<AdmittedResidual>()
                        + r.isolation
                            .as_ref()
                            .map_or(0, |program| program.retained_bytes())
                })
                .sum::<usize>()
            + self
                .bodies()
                .map(|b| {
                    b.math.retained_bytes()
                        + size_of_val(b.quantities.as_slice())
                        + size_of_val(b.occurrences.as_slice())
                })
                .sum::<usize>()
    }
    /// Initial known payload for this factory's actual compilation population.
    /// The body constructor reserves positive optimized excess before floating allocation. Native
    /// rational optimization/isolation remains runtime's separate foreign allowance.
    /// A control/provider body has no arithmetic-only ceiling and fails closed to the
    /// caller's conservative construction admission.
    pub fn reconstruction_allocation_bound(
        &self,
        configurations: &BTreeMap<SemanticId, pse_math::implicit::Configuration>,
        requirements: pse_kernels::DerivativeRequirements,
        limits: EvaluationLimits,
    ) -> Result<Option<usize>> {
        let overflow = || CompileError::from(MathError::Limit("implicit construction extent"));
        let mut bytes = self
            .residuals
            .len()
            .checked_mul(1024)
            .ok_or_else(overflow)?;
        for residual in &self.residuals {
            let configuration = configurations
                .get(&residual.id)
                .ok_or_else(|| CompileError::Missing("implicit branch configuration".into()))?;
            let local = if requirements.requested_output > DerivativeOrder::Value {
                DerivativeOrder::First
            } else {
                DerivativeOrder::Value
            };
            let mut bodies = vec![
                (&residual.body, DerivativeOrder::Value, false),
                (&residual.body, requirements.inner_minimum, true),
            ];
            if matches!(configuration, pse_math::implicit::Configuration::Hints(_)) {
                bodies.extend(
                    residual
                        .hints
                        .iter()
                        .chain(residual.terms.iter())
                        .map(|body| (body, DerivativeOrder::Value, false)),
                );
            }
            // Match factory compilation: value anchors and branch-local selectors.
            bodies.extend(
                self.selection
                    .anchors
                    .iter()
                    .map(|body| (body, DerivativeOrder::Value, false)),
            );
            bodies.extend(
                self.selection
                    .restriction
                    .iter()
                    .map(|body| (body, local, false)),
            );
            if let Some(assessment) = &residual.assessment {
                bodies.extend([
                    (&assessment.eligibility, local, false),
                    (&assessment.criterion, local, false),
                ]);
            }
            for (body, order, unknown_only) in bodies {
                let outputs = (0..body.quantities.len()).collect::<Vec<_>>();
                let coordinates = if order == DerivativeOrder::Value {
                    Vec::new()
                } else if unknown_only {
                    (0..self.unknowns.len()).collect()
                } else {
                    (0..body.math.input_count()).collect()
                };
                let Some(mut bound) = body.math.arithmetic_selected_compilation_allocation_bound(
                    &outputs,
                    &coordinates,
                    order,
                    limits,
                )?
                else {
                    return Ok(None);
                };
                if self.algorithm == ImplicitAlgorithm::AffineRates
                    && Arc::ptr_eq(body, &residual.body)
                {
                    let Some(proof) = body
                        .math
                        .arithmetic_analysis_allocation_bound(Some(self.unknowns.len()))?
                    else {
                        return Ok(None);
                    };
                    // Affine::new drops its support before numerical compilation.
                    // Its two retained counts and Arc fit the branch metadata above.
                    bound = bound.max(proof);
                }
                bytes = bytes.checked_add(bound).ok_or_else(overflow)?;
            }
            // The reconstruction and regime factories additionally project exact
            // root-isolation descriptors for residual/assessment bodies. Those
            // known Rust vectors are separate from foreign Rational/optimizer memory.
            for body in std::iter::once(&residual.body).chain(
                residual
                    .assessment
                    .iter()
                    .flat_map(|assessment| [&assessment.eligibility, &assessment.criterion]),
            ) {
                let Some(bound) = body
                    .math
                    .root_isolation_allocation_bound(limits.derivative_components)?
                else {
                    return Ok(None);
                };
                bytes = bytes.checked_add(bound).ok_or_else(overflow)?;
            }
            bytes = bytes
                .checked_add(configuration.retained_bytes())
                .and_then(|bytes| {
                    bytes.checked_add(residual.rows.len().checked_mul(size_of::<SemanticId>())?)
                })
                .and_then(|bytes| {
                    bytes.checked_add(
                        self.unknowns
                            .len()
                            .checked_mul(size_of::<pse_math::implicit::Unknown>())?,
                    )
                })
                .ok_or_else(overflow)?;
        }
        Ok(Some(bytes))
    }
    /// Declare every actual inner program before runtime compilation. Configuration
    /// selects hint demand but its numerical values never enter immutable program keys.
    pub fn program_requests(
        &self,
        configurations: &BTreeMap<SemanticId, pse_math::implicit::Configuration>,
        capabilities: &ImplicitCapabilities<'_>,
        requested_output: DerivativeOrder,
        cancel: &Arc<AtomicBool>,
        limits: EvaluationLimits,
    ) -> Result<(Vec<ImplicitProgramRequest>, ImplicitPrograms)> {
        let mut requests = BTreeMap::new();
        let mut programs = ImplicitPrograms::new(limits);
        let profile = Profile {
            optimization: Optimization::default(),
            evaluation: limits,
            ..Profile::default()
        };
        for residual in &self.residuals {
            let configuration = configurations
                .get(&residual.id)
                .ok_or_else(|| CompileError::Missing("implicit branch configuration".into()))?;
            let solver = self.branch_solver(residual, capabilities, limits, cancel)?;
            let requirements =
                self.residual_requirements(residual, requested_output, solver.minimum_order())?;
            programs.solvers.insert(residual.id, solver);
            let local_order = if requested_output > DerivativeOrder::Value {
                DerivativeOrder::First
            } else {
                DerivativeOrder::Value
            };
            let mut add = |body: &AdmittedBody,
                           order: DerivativeOrder,
                           unknown_only: bool,
                           local: bool|
             -> Result<()> {
                let coordinates: Vec<_> = if order == DerivativeOrder::Value && !local {
                    Vec::new()
                } else if unknown_only {
                    (0..self.unknowns.len()).collect()
                } else {
                    (0..body.math.input_count()).collect()
                };
                let outputs = (0..body.quantities.len()).collect::<Vec<_>>();
                let key = (body.spec.key(), order as u8, coordinates.clone(), local);
                if requests.contains_key(&key) {
                    return Ok(());
                }
                let support = Arc::new(
                    body.math
                        .prepare_support(&outputs, &coordinates, order, cancel)
                        .map_err(|error| {
                            error.with_derivative_body(Some(residual.id), body.spec.key())
                        })?,
                );
                let demand = LocalDemand {
                    body: body.spec.key(),
                    outputs,
                    coordinates,
                    order,
                    product: pse_math::jets::ProductKind::SelectedPartials,
                };
                let artifact =
                    artifact_request(&demand, support, profile, &body.spec.physical, local);
                requests.insert(
                    key.clone(),
                    ImplicitProgramRequest {
                        key,
                        artifact,
                        source: residual.id,
                    },
                );
                Ok(())
            };
            add(&residual.body, requirements.inner_minimum, true, false)?;
            add(&residual.body, DerivativeOrder::Value, false, false)?;
            if matches!(configuration, pse_math::implicit::Configuration::Hints(_)) {
                for body in residual.hints.iter().chain(&residual.terms) {
                    add(body, DerivativeOrder::Value, false, false)?;
                }
            }
            if let Some(body) = &self.selection.anchors {
                add(body, DerivativeOrder::Value, false, false)?;
            }
            if let Some(body) = &self.selection.restriction {
                add(body, local_order, false, true)?;
            }
            if let Some(assessment) = &residual.assessment {
                add(&assessment.eligibility, local_order, false, true)?;
                add(&assessment.criterion, local_order, false, true)?;
            }
        }
        Ok((requests.into_values().collect(), programs))
    }
    fn branch_solver(
        &self,
        residual: &AdmittedResidual,
        capabilities: &ImplicitCapabilities<'_>,
        limits: EvaluationLimits,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Arc<dyn pse_math::implicit::InnerSolver>> {
        Ok(match &self.algorithm {
            ImplicitAlgorithm::Accelerator(id) => capabilities.accelerators.admit(
                id,
                &residual.body.math,
                self.unknowns.len(),
                limits,
                cancel,
            )?,
            ImplicitAlgorithm::AffineRates => Arc::new(pse_math::implicit::Affine::new(
                &residual.body.math,
                self.unknowns.len(),
            )?),
            ImplicitAlgorithm::Native => capabilities.solver.clone(),
        })
    }
    /// Build compiler-issued requests directly for service-free functional controls.
    #[cfg(test)]
    pub fn compile_programs(
        &self,
        configurations: &BTreeMap<SemanticId, pse_math::implicit::Configuration>,
        capabilities: &ImplicitCapabilities<'_>,
        requested_output: DerivativeOrder,
        cancel: &pse_diagnostics::Cancellation,
        limits: EvaluationLimits,
    ) -> Result<ImplicitPrograms> {
        let (requests, mut programs) = self.program_requests(
            configurations,
            capabilities,
            requested_output,
            cancel,
            limits,
        )?;
        for request in requests {
            let body = Arc::new(
                request
                    .artifact
                    .build(&pse_kernels::ExecutionScope::new(cancel.clone(), None))
                    .map_err(|error| request.attribute_refusal(&error).unwrap_or(error))?,
            );
            programs.bind(request, body)?;
        }
        Ok(programs)
    }
    /// Retain the actual numerical source with producer-issued residual realization
    /// and an exact single-root proof projection, when representable.
    pub fn reconstruction_factory(
        &self,
        configurations: BTreeMap<SemanticId, pse_math::implicit::Configuration>,
        capabilities: ImplicitCapabilities<'_>,
        requested_output: DerivativeOrder,
        cancel: pse_diagnostics::Cancellation,
        limits: EvaluationLimits,
        programs: &ImplicitPrograms,
    ) -> Result<pse_math::implicit::reconstruction::ReconstructionFactory> {
        use pse_math::implicit::{
            ImplicitFactory,
            reconstruction::{ReconstructionFactory, SelectedResidualRealization},
        };
        let verifier = capabilities.verifier.clone();
        let factory = self.factory_reusing(
            configurations,
            capabilities,
            requested_output,
            cancel.clone(),
            limits,
            programs,
        )?;
        let isolation = match &factory {
            ImplicitFactory::Root(root)
                if requested_output >= DerivativeOrder::First
                    && root.selection.restriction.is_none() =>
            {
                let residual = self
                    .residuals
                    .first()
                    .ok_or_else(|| CompileError::Missing("single-root residual source".into()))?;
                pse_math::factorable::single_root_isolation_program(
                    residual.id,
                    &residual.body.math,
                    &cancel,
                    limits.derivative_components,
                )
                .map_err(|error| match error {
                    pse_math::factorable::FactorableError::Math(error) => CompileError::from(error),
                    error => CompileError::Missing(error.to_string()),
                })?
                .map(Arc::new)
            }
            _ => None,
        };
        // ModelingOutput equality programs lower lhs-rhs with bounds [0,0].
        // Record that producer fact explicitly; consumers never infer offsets from IDs.
        let rows = self
            .residuals
            .first()
            .ok_or_else(|| CompileError::Missing("implicit residual source".into()))?
            .rows
            .clone();
        Ok(ReconstructionFactory::new(
            factory,
            SelectedResidualRealization::ZeroResiduals {
                authored_offsets: rows.into_iter().map(|id| (id, 0.0)).collect(),
            },
            isolation,
            verifier,
        ))
    }
    /// Exercise the production request and factory assembly route without a runtime service.
    #[cfg(test)]
    pub fn factory(
        &self,
        configurations: BTreeMap<SemanticId, pse_math::implicit::Configuration>,
        capabilities: ImplicitCapabilities<'_>,
        requested_output: DerivativeOrder,
        cancel: pse_diagnostics::Cancellation,
        limits: EvaluationLimits,
    ) -> Result<pse_math::implicit::ImplicitFactory> {
        let programs = self.compile_programs(
            &configurations,
            &capabilities,
            requested_output,
            &cancel,
            limits,
        )?;
        self.factory_reusing(
            configurations,
            capabilities,
            requested_output,
            cancel,
            limits,
            &programs,
        )
    }
    /// Every branch receives its own resolved physical hints. The selector has one total deadline.
    pub fn factory_reusing(
        &self,
        mut configurations: BTreeMap<SemanticId, pse_math::implicit::Configuration>,
        capabilities: ImplicitCapabilities<'_>,
        requested_output: DerivativeOrder,
        cancel: pse_diagnostics::Cancellation,
        limits: EvaluationLimits,
        programs: &ImplicitPrograms,
    ) -> Result<pse_math::implicit::ImplicitFactory> {
        if programs.limits != limits {
            return Err(CompileError::Missing(
                "implicit artifact numeric limits changed".into(),
            ));
        }
        use pse_math::implicit::{Factory, ImplicitFactory, RegimeFactory, RegimeFactoryBranch};
        let verifier = capabilities.verifier;
        if configurations.len() != self.residuals.len() || self.residuals.is_empty() {
            return Err(CompileError::Missing(
                "implicit branch configuration extent".into(),
            ));
        }
        if requested_output > DerivativeOrder::Value
            && self.selection.neighborhood_evidence == SelectionNeighborhood::RuntimeIsolation
            && verifier.is_none()
        {
            return Err(CompileError::Missing(
                "nonlinear selection requires a validated isolation capability".into(),
            ));
        }
        let mut branches = Vec::new();
        let mut total_time = None;
        for residual in &self.residuals {
            let configuration = configurations
                .remove(&residual.id)
                .ok_or_else(|| CompileError::Missing("implicit branch configuration".into()))?;
            let unknowns = match &configuration {
                pse_math::implicit::Configuration::Fixed(unknowns, _) => unknowns.clone(),
                pse_math::implicit::Configuration::Hints(_) => self
                    .unknowns
                    .iter()
                    .map(|id| pse_math::implicit::Unknown {
                        id: *id,
                        lower: f64::NEG_INFINITY,
                        upper: f64::INFINITY,
                    })
                    .collect(),
            };
            if unknowns
                .iter()
                .map(|u| u.id)
                .ne(self.unknowns.iter().copied())
            {
                return Err(CompileError::Missing(
                    "implicit unknown binding order".into(),
                ));
            }
            total_time = Some(
                total_time.map_or(configuration.time_limit(), |t: std::time::Duration| {
                    t.min(configuration.time_limit())
                }),
            );
            let compile = |body: &ImplicitBody, order| -> Result<pse_math::SourceBody> {
                let coordinates = if order == DerivativeOrder::Value {
                    Vec::new()
                } else {
                    (0..body.math.input_count()).collect::<Vec<_>>()
                };
                body.compiled(programs.get(body, &coordinates, order, false)?)
            };
            let branch_solver = programs.solvers.get(&residual.id).cloned().ok_or_else(|| {
                CompileError::Missing("implicit admitted branch solver missing".into())
            })?;
            let requirements = self.residual_requirements(
                residual,
                requested_output,
                branch_solver.minimum_order(),
            )?;
            let mut spec = self
                .descriptor
                .restrict_order(requirements.requested_output)
                .map_err(|e| CompileError::Missing(e.to_string()))?
                .spec()
                .clone();
            spec.id = residual.id;
            let restriction = self
                .selection
                .restriction
                .as_ref()
                .map(|body| {
                    body.compiled(programs.get(
                        body,
                        &(0..body.math.input_count()).collect::<Vec<_>>(),
                        if requested_output > DerivativeOrder::Value {
                            DerivativeOrder::First
                        } else {
                            DerivativeOrder::Value
                        },
                        true,
                    )?)
                })
                .transpose()?;
            let unknown_order = branch_solver.minimum_order();
            let unknown_coordinates = if unknown_order == DerivativeOrder::Value {
                Vec::new()
            } else {
                (0..self.unknowns.len()).collect::<Vec<_>>()
            };
            let unknown_body = residual.body.compiled(programs.get(
                &residual.body,
                &unknown_coordinates,
                unknown_order,
                false,
            )?)?;
            let factory = Factory {
                selection: pse_math::implicit::Selection {
                    anchor: self
                        .selection
                        .anchors
                        .as_ref()
                        .map(|b| compile(b, DerivativeOrder::Value))
                        .transpose()?,
                    settings: match &self.selection.meaning {
                        ImplicitMeaning::Operational(s) => Some(s.clone()),
                        _ => None,
                    },
                    restriction,
                    sign: self.selection.sign,
                },
                requirements,
                spec,
                bodies: pse_math::implicit::ResidualBodies {
                    unknown: unknown_body,
                    source: compile(&residual.body, DerivativeOrder::Value)?,
                },
                unknowns,
                rows: residual.rows.clone(),
                hints: if matches!(configuration, pse_math::implicit::Configuration::Hints(_)) {
                    residual
                        .hints
                        .as_ref()
                        .map(|body| compile(body, DerivativeOrder::Value))
                        .transpose()?
                } else {
                    None
                },
                terms: if matches!(configuration, pse_math::implicit::Configuration::Hints(_)) {
                    residual
                        .terms
                        .as_ref()
                        .map(|body| compile(body, DerivativeOrder::Value))
                        .transpose()?
                } else {
                    None
                },
                configuration,
                solver: branch_solver,
                cancel: cancel.clone(),
                max_entries: limits.derivative_components,
                providers: BTreeMap::new(),
            };
            let local = |body: &ImplicitBody| -> Result<pse_math::SourceBody> {
                body.compiled(programs.get(
                    body,
                    &(0..body.math.input_count()).collect::<Vec<_>>(),
                    if requested_output > DerivativeOrder::Value {
                        DerivativeOrder::First
                    } else {
                        DerivativeOrder::Value
                    },
                    true,
                )?)
            };
            match &residual.assessment {
                Some(a) => branches.push(RegimeFactoryBranch {
                    residual: factory,
                    eligibility: local(&a.eligibility)?,
                    criterion: local(&a.criterion)?,
                    isolation: if requested_output > DerivativeOrder::Value
                        && self.selection.neighborhood_evidence
                            == SelectionNeighborhood::RuntimeIsolation
                    {
                        residual.isolation.clone()
                    } else {
                        None
                    },
                }),
                None if self.residuals.len() == 1 => return Ok(ImplicitFactory::Root(factory)),
                None => {
                    return Err(CompileError::Missing(
                        "implicit selection requires every assessment".into(),
                    ));
                }
            }
        }
        Ok(ImplicitFactory::Regimes(RegimeFactory {
            spec: self
                .descriptor
                .restrict_order(requested_output)
                .map_err(|e| CompileError::Missing(e.to_string()))?
                .spec()
                .clone(),
            maximum_regimes: self.residuals.len(),
            alternatives: branches,
            time_limit: total_time
                .ok_or_else(|| CompileError::Missing("empty implicit selector".into()))?,
            cancel,
            verifier,
        }))
    }
}
/// Project exact typed residual/selector products. Actual executable provider descriptors
/// are inserted only after residual admission establishes their final capabilities.
pub(super) fn project(
    db: &dyn CompilerDb,
    model: &SpecializedModel,
    registry: &QuantityRegistry,
    preconditions: &PhysicalPreconditions,
    p: &mut super::Projection,
) -> Result<()> {
    use pse_model::generated::enums::ModelingDeclarationKind as Kind;
    use pse_modeling::specialize::Realization as Policy;
    checkpoint(db);
    let ordinary = model
        .implicit
        .values()
        .any(|policy| matches!(policy, Policy::Nested | Policy::Accelerated(_)));
    let indexed = if ordinary { model.symbols.len() } else { 0 };
    let scratch = model
        .instances
        .len()
        .checked_mul(size_of::<InstanceId>() * 4 + size_of::<usize>() * 24 + 256)
        .and_then(|bytes| {
            bytes.checked_add(
                indexed.checked_mul(size_of::<SemanticId>() * 2 + size_of::<InstanceId>() + 160)?,
            )
        })
        .ok_or(CompileError::Limit("implicit projection scratch extent"))?;
    if scratch > db.projection_scratch_limit() {
        return Err(CompileError::Limit("implicit projection scratch allowance"));
    }
    let context = TypeContext {
        admissions: None,
        formula_authority: None,
        quantities: registry,
        preconditions,
        scope: &p.physical_scope,
    };
    let mut variables = BTreeMap::<InstanceId, Vec<SemanticId>>::new();
    if ordinary {
        for symbol in model.symbols.values() {
            checkpoint(db);
            if symbol.role == Kind::Variable && symbol.definition.is_none() {
                variables
                    .entry(symbol.lineage.instance)
                    .or_default()
                    .push(symbol.id);
            }
        }
    }
    let ports = p
        .inputs
        .iter()
        .zip(&p.formals)
        .map(|(id, formal)| {
            Ok((
                *id,
                Port {
                    id: *id,
                    quantity: formal.quantity,
                    unit: registry
                        .quantity_type(formal.quantity)
                        .map_err(MathError::from)?
                        .canonical_unit,
                },
            ))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    let mut hierarchy = DiGraph::<InstanceId, ()>::new();
    let nodes = model
        .instances
        .keys()
        .map(|id| (*id, hierarchy.add_node(*id)))
        .collect::<BTreeMap<_, _>>();
    for (id, item) in &model.instances {
        checkpoint(db);
        if let Some(parent) = item.parent {
            hierarchy.add_edge(
                nodes[id],
                *nodes
                    .get(&parent)
                    .ok_or_else(|| CompileError::Missing("implicit parent instance".into()))?,
                (),
            );
        }
    }
    let mut stages = toposort(&hierarchy, None)
        .map_err(|_| CompileError::Missing("cyclic instance hierarchy".into()))?
        .into_iter()
        .filter_map(|node| {
            let id = hierarchy[node];
            match model.implicit.get(&id) {
                Some(Policy::Nested) => Some((id.as_id(), ImplicitAlgorithm::Native)),
                Some(Policy::Accelerated(reference)) => Some((
                    id.as_id(),
                    ImplicitAlgorithm::Accelerator(reference.clone()),
                )),
                _ => None,
            }
        })
        .collect::<Vec<_>>();
    if !model.derivatives.is_empty() {
        let axis = model
            .integrated
            .keys()
            .next()
            .ok_or_else(|| CompileError::Missing("rate system axis".into()))?;
        stages.push((
            pse_ids::named_id(*axis, "affine-rate-system"),
            ImplicitAlgorithm::AffineRates,
        ));
    }
    let mut hidden = BTreeSet::new();
    let mut unknown_ids = BTreeSet::new();
    let mut suppliers = BTreeMap::<SemanticId, BTreeSet<SemanticId>>::new();
    let branch_declarations = model
        .regimes
        .values()
        .flat_map(|selection| {
            selection.alternatives.iter().flat_map(|regime| {
                regime
                    .annotations
                    .iter()
                    .map(|annotation| annotation.lineage.declaration)
            })
        })
        .collect::<BTreeSet<_>>();
    for (stage, algorithm) in &stages {
        checkpoint(db);
        let generated = *algorithm == ImplicitAlgorithm::AffineRates;
        let instance = InstanceId::from_id(*stage);
        let unknowns = if generated {
            model
                .derivatives
                .values()
                .map(|derivative| derivative.rate)
                .collect::<Vec<_>>()
        } else {
            variables.get(&instance).cloned().unwrap_or_default()
        };
        let selection = model.regimes.get(&instance);
        let mut operation = SelectionProjection {
            identity: ContentHash::from_bytes([0; 32]),
            meaning: if selection.is_some() {
                ImplicitMeaning::MinimumScore
            } else if generated {
                ImplicitMeaning::Unique
            } else {
                ImplicitMeaning::Relation
            },
            anchors: vec![],
            restriction: None,
            sign: None,
            strict: false,
        };
        if let Some(authored) = model.root_selections.get(&instance) {
            use pse_modeling::specialize::RootSelection;
            let predicate = match authored {
                RootSelection::Branch { predicate, .. } => {
                    operation.meaning = ImplicitMeaning::Branch;
                    Some(*predicate)
                }
                RootSelection::Operational {
                    anchor_roots,
                    settings,
                    neighborhood_root,
                    ..
                } => {
                    operation.meaning = ImplicitMeaning::Operational(settings.clone());
                    if anchor_roots.len() != unknowns.len()
                        || unknowns.iter().any(|id| !anchor_roots.contains_key(id))
                    {
                        return Err(CompileError::Missing(
                            "operational selector requires one anchor per unknown".into(),
                        ));
                    }
                    operation.anchors = unknowns
                        .iter()
                        .map(|id| {
                            model
                                .prepare_region(
                                    RegionRequest {
                                        outputs: &[RegionOutput::Value(anchor_roots[id])],
                                        validity: &[],
                                    },
                                    &context,
                                )
                                .map_err(CompileError::from)
                        })
                        .collect::<Result<_>>()?;
                    *neighborhood_root
                }
            };
            if let Some(predicate) = predicate {
                operation.sign = sign_restriction(model, predicate, &unknowns);
                operation.strict = matches!(
                    model.resolved.predicates.get(predicate),
                    Some(ResolvedPredicate::Compare {
                        op: dsl::CompareOp::Gt | dsl::CompareOp::Lt,
                        ..
                    })
                );
                operation.restriction = Some(model.prepare_region(
                    RegionRequest {
                        outputs: &[RegionOutput::PredicateIndicator(predicate)],
                        validity: &[],
                    },
                    &context,
                )?);
            }
        }
        let branches = if generated {
            let rows = p
                .outputs
                .iter()
                .find_map(|output| {
                    if let ModelingOutput::DynamicRate { equations, .. } = output {
                        Some(equations.clone())
                    } else {
                        None
                    }
                })
                .ok_or_else(|| CompileError::Missing("rate system equations".into()))?;
            if rows.iter().any(|row| hidden.contains(row)) {
                return Err(CompileError::Missing(
                    "time derivatives cannot be owned by a nested algebraic block".into(),
                ));
            }
            vec![(*stage, rows, None)]
        } else if let Some(selection) = selection {
            selection
                .alternatives
                .iter()
                .map(|regime| {
                    (
                        regime.id,
                        regime.equations.iter().map(|row| row.id).collect(),
                        Some(regime),
                    )
                })
                .collect()
        } else {
            vec![(
                *stage,
                model
                    .equations
                    .iter()
                    .filter(|row| row.lineage.instance == instance)
                    .map(|row| row.id)
                    .collect(),
                None,
            )]
        };
        let mut residuals = Vec::new();
        for (id, rows, alternative) in branches {
            if unknowns.is_empty() || rows.len() != unknowns.len() {
                return Err(CompileError::Missing(
                    "nested implicit block must be a nonempty square residual system".into(),
                ));
            }
            let indices=p.outputs.iter().enumerate().filter_map(|(index,output)|matches!(output,ModelingOutput::Equation {id,sense:EquationSense::Eq} if rows.contains(id)).then_some(index)).collect::<Vec<_>>();
            if indices.len() != rows.len() {
                return Err(CompileError::Missing(
                    "implicit block requires equality residuals".into(),
                ));
            }
            let assessment = alternative
                .zip(selection)
                .map(|(alternative, selection)| -> Result<AssessmentProjection> {
                    let eligibility = model.prepare_region(
                        RegionRequest {
                            outputs: &[RegionOutput::PredicateIndicator(
                                alternative.eligibility_root,
                            )],
                            validity: &[],
                        },
                        &context,
                    )?;
                    let criterion = [selection.criterion_root, selection.tolerance_root]
                        .into_iter()
                        .map(|root| {
                            model
                                .prepare_region(
                                    RegionRequest {
                                        outputs: &[RegionOutput::Value(root)],
                                        validity: &[],
                                    },
                                    &context,
                                )
                                .map_err(CompileError::from)
                        })
                        .collect::<Result<Vec<_>>>()?;
                    let quantities = criterion
                        .iter()
                        .map(region_quantity)
                        .collect::<Result<Vec<_>>>()?;
                    Ok(AssessmentProjection {
                        eligibility,
                        criterion,
                        quantities,
                    })
                })
                .transpose()?;
            let local_declarations = alternative
                .into_iter()
                .flat_map(|alternative| {
                    alternative
                        .annotations
                        .iter()
                        .map(|annotation| annotation.lineage.declaration)
                })
                .collect::<BTreeSet<_>>();
            let mut hints = BTreeMap::new();
            for local in [false, true] {
                let mut seen = BTreeSet::new();
                for (index, output) in p.outputs.iter().enumerate() {
                    if let ModelingOutput::Hint {
                        target,
                        declaration,
                        kind,
                    } = output
                        && (unknowns.contains(target) || rows.contains(target))
                        && matches!(
                            kind,
                            ModelingHint::Start
                                | ModelingHint::Lower
                                | ModelingHint::Upper
                                | ModelingHint::Nominal
                        )
                        && if local {
                            local_declarations.contains(declaration)
                        } else {
                            !branch_declarations.contains(declaration)
                        }
                    {
                        if !seen.insert((*target, *kind)) {
                            return Err(CompileError::Missing(
                                "competing implicit numerical hints in one regime".into(),
                            ));
                        }
                        hints.insert(
                            (*target, *kind),
                            (
                                *target,
                                *declaration,
                                *kind,
                                p.regions[index].clone(),
                                p.quantities[index],
                            ),
                        );
                    }
                }
            }
            let mut selected_scales = BTreeMap::new();
            for annotations in [
                model.annotations.as_slice(),
                alternative.map_or(&[][..], |alternative| alternative.annotations.as_slice()),
            ] {
                let mut seen = BTreeSet::new();
                for annotation in annotations {
                    if rows.contains(&annotation.target)
                        && let pse_modeling::annotation::AnnotationValue::Scale(scheme) =
                            annotation.value
                    {
                        if !seen.insert(annotation.target) {
                            return Err(CompileError::Missing(
                                "competing implicit scaling schemes in one regime".into(),
                            ));
                        }
                        selected_scales
                            .insert(annotation.target, (scheme, annotation.lineage.declaration));
                    }
                }
            }
            let mut terms = Vec::new();
            let mut scales = Vec::new();
            for (row, (scheme, source)) in selected_scales {
                let begin = terms.len();
                for (index, output) in p.outputs.iter().enumerate() {
                    if matches!(output,ModelingOutput::Term {equation,..} if *equation==row) {
                        terms.push((p.regions[index].clone(), p.quantities[index]));
                    }
                }
                if begin == terms.len() {
                    return Err(CompileError::Missing(
                        "implicit original scaling terms are absent".into(),
                    ));
                }
                scales.push(ImplicitScale {
                    row,
                    source,
                    scheme,
                    terms: begin..terms.len(),
                });
            }
            residuals.push(ResidualProjection {
                id,
                rows: indices
                    .iter()
                    .map(|index| p.outputs[*index].row_id())
                    .collect(),
                regions: indices
                    .iter()
                    .map(|index| p.regions[*index].clone())
                    .collect(),
                quantities: indices.iter().map(|index| p.quantities[*index]).collect(),
                assessment,
                hints: hints.into_values().collect(),
                terms,
                scales,
            });
            hidden.extend(rows);
        }
        for region in residuals
            .iter()
            .flat_map(|residual| residual.hints.iter().map(|hint| &hint.3))
            .chain(&operation.anchors)
        {
            if expanded_region_reads(region, &suppliers)
                .iter()
                .any(|id| unknowns.contains(id))
            {
                return Err(CompileError::Missing(
                    "implicit numerical hint depends on its own unknowns".into(),
                ));
            }
        }
        let mut needed = BTreeSet::new();
        for region in residuals
            .iter()
            .flat_map(|residual| {
                residual
                    .regions
                    .iter()
                    .chain(residual.hints.iter().map(|hint| &hint.3))
                    .chain(residual.terms.iter().map(|term| &term.0))
                    .chain(residual.assessment.iter().flat_map(|assessment| {
                        std::iter::once(&assessment.eligibility).chain(&assessment.criterion)
                    }))
            })
            .chain(&operation.anchors)
            .chain(operation.restriction.iter())
        {
            needed.extend(expanded_region_inputs(region, &suppliers)?);
        }
        let inputs = p
            .inputs
            .iter()
            .filter(|id| {
                !unknowns.contains(id) && !unknown_ids.contains(*id) && needed.contains(*id)
            })
            .copied()
            .collect::<Vec<_>>();
        let mut hash = FramedHasher::new(pse_ids::Frame::ModelingImplicitResidualV5);
        let mut semantic = FramedHasher::new(pse_ids::Frame::ModelingImplicitOperationV1);
        hash.str("direct-implicit-products")
            .id(stage)
            .str(&algorithm.key());
        semantic.str("direct-implicit-products").id(stage);
        for hasher in [&mut hash, &mut semantic] {
            hasher.str(match &operation.meaning {
                ImplicitMeaning::Relation => "relation",
                ImplicitMeaning::Unique => "unique",
                ImplicitMeaning::Branch => "branch",
                ImplicitMeaning::MinimumScore => "minimum-score",
                ImplicitMeaning::Operational(settings) => settings,
            });
            hasher.u64(operation.anchors.len() as u64);
            for region in &operation.anchors {
                frame_region(region, hasher)?;
            }
            hasher.bool(operation.restriction.is_some());
            if let Some(region) = &operation.restriction {
                frame_region(region, hasher)?;
            }
            hasher.u64(unknowns.len() as u64);
            for id in unknowns.iter().chain(&inputs) {
                hasher.id(id).id(&ports
                    .get(id)
                    .ok_or_else(|| CompileError::Missing("implicit exact formal port".into()))?
                    .quantity
                    .as_id());
            }
        }
        for residual in &residuals {
            for hasher in [&mut hash, &mut semantic] {
                hasher.id(&residual.id).u64(residual.regions.len() as u64);
                for region in &residual.regions {
                    frame_region(region, hasher)?;
                }
                if let Some(assessment) = &residual.assessment {
                    frame_region(&assessment.eligibility, hasher)?;
                    for region in &assessment.criterion {
                        frame_region(region, hasher)?;
                    }
                }
            }
            for scale in &residual.scales {
                hash.id(&scale.row)
                    .id(&scale.source.as_id())
                    .str(scale.scheme.as_str())
                    .u64(scale.terms.start as u64)
                    .u64(scale.terms.end as u64);
            }
            for (region, quantity) in &residual.terms {
                hash.id(&quantity.as_id());
                frame_region(region, &mut hash)?;
            }
            for (target, source, kind, region, quantity) in &residual.hints {
                hash.id(target)
                    .id(&source.as_id())
                    .u64(kind.code())
                    .id(&quantity.as_id());
                frame_region(region, &mut hash)?;
            }
        }
        operation.identity = semantic.finish_hash();
        let revision = hash.finish_hash();
        let spec = ProviderSpec {
            shapes: pse_kernels::ProviderShapes::default(),
            derivative_source: pse_kernels::DerivativeSource::Implicit,
            id: *stage,
            revision,
            data: revision,
            inputs: inputs.iter().map(|id| ports[id].clone()).collect(),
            outputs: unknowns.iter().map(|id| ports[id].clone()).collect(),
            derivatives: DerivativeOrder::Value,
            smoothness: DerivativeOrder::Value,
        };
        for id in &unknowns {
            suppliers.insert(*id, inputs.iter().copied().collect());
            if !p
                .outputs
                .iter()
                .any(|output| matches!(output,ModelingOutput::Member(existing) if existing==id))
            {
                p.outputs.push(ModelingOutput::Member(*id));
                p.regions.push(model.prepare_region(
                    RegionRequest {
                        outputs: &[RegionOutput::Value(model.symbol_root(*id)?)],
                        validity: &[],
                    },
                    &context,
                )?);
                p.quantities.push(ports[id].quantity);
                p.declarations.push(model.symbols[id].lineage.declaration);
            }
        }
        p.implicit.push(Projection {
            selection: operation,
            algorithm: algorithm.clone(),
            id: *stage,
            unknowns: unknowns.clone(),
            inputs,
            residuals,
            spec,
        });
        unknown_ids.extend(unknowns);
    }
    let keep=p.outputs.iter().enumerate().filter_map(|(index,output)|(!matches!(output,ModelingOutput::Equation {id,..}|ModelingOutput::Term {equation:id,..} if hidden.contains(id))).then_some(index)).collect::<Vec<_>>();
    p.outputs = keep.iter().map(|index| p.outputs[*index].clone()).collect();
    p.regions = keep.iter().map(|index| p.regions[*index].clone()).collect();
    p.quantities = keep.iter().map(|index| p.quantities[*index]).collect();
    p.declarations = keep.iter().map(|index| p.declarations[*index]).collect();
    let keep = p
        .inputs
        .iter()
        .enumerate()
        .filter_map(|(index, id)| (!unknown_ids.contains(id)).then_some(index))
        .collect::<Vec<_>>();
    p.inputs = keep.iter().map(|index| p.inputs[*index]).collect();
    p.formals = keep.iter().map(|index| p.formals[*index].clone()).collect();
    model.require_fixed_discrete(
        unknown_ids.iter().copied(),
        pse_modeling::DomainAnalysis::Root,
    )?;
    p.free.retain(|id, _| !unknown_ids.contains(id));
    Ok(())
}
// Semantic reads may omit a formal ignored by a selected function. Composition
// still owes the complete issued input layout, including that formal's original
// scalar/range declaration and any selected supplier's declared input ports.
fn expanded_region_inputs(
    region: &SelectedRegion,
    suppliers: &BTreeMap<SemanticId, BTreeSet<SemanticId>>,
) -> Result<BTreeSet<SemanticId>> {
    let mut pending = region
        .reads
        .numeric
        .union(&region.reads.guards)
        .copied()
        .collect::<Vec<_>>();
    for slot in &region.inputs {
        let binding = region
            .bindings
            .iter()
            .find(|binding| binding.local == *slot)
            .ok_or_else(|| {
                CompileError::Missing("implicit complete input correspondence".into())
            })?;
        pending.push(binding.actual);
    }
    Ok(expanded_supplier_reads(pending, suppliers))
}
fn expanded_region_reads(
    region: &SelectedRegion,
    suppliers: &BTreeMap<SemanticId, BTreeSet<SemanticId>>,
) -> BTreeSet<SemanticId> {
    let pending = region
        .reads
        .numeric
        .union(&region.reads.guards)
        .copied()
        .collect::<Vec<_>>();
    expanded_supplier_reads(pending, suppliers)
}
fn expanded_supplier_reads(
    mut pending: Vec<SemanticId>,
    suppliers: &BTreeMap<SemanticId, BTreeSet<SemanticId>>,
) -> BTreeSet<SemanticId> {
    let mut reads = BTreeSet::new();
    while let Some(id) = pending.pop() {
        if !reads.insert(id) {
            continue;
        }
        if let Some(inputs) = suppliers.get(&id) {
            pending.extend(inputs);
        }
    }
    reads
}
fn frame_region(region: &SelectedRegion, hash: &mut FramedHasher) -> Result<()> {
    hash.hash(&region.syntax.identity()?);
    region.syntax.frame_input_layout(&region.inputs, hash)?;
    region.support.frame(hash);
    Ok(())
}
fn region_quantity(region: &SelectedRegion) -> Result<QuantityTypeId> {
    Ok(region
        .support
        .output(0)?
        .target()
        .require_named()
        .map_err(MathError::from)?)
}
/// Immutable supplier identities and edges; request selection never mutates this graph.
#[derive(Debug)]
pub(super) struct SupplierTopology {
    graph: DiGraph<SemanticId, ()>,
    nodes: BTreeMap<SemanticId, petgraph::graph::NodeIndex>,
}
impl SupplierTopology {
    /// Substitution follows the selected systems and their demanded nested providers.
    /// Other retained descriptors remain reconstruction metadata only.
    pub(super) fn selected_dependencies(
        implicit: &BTreeMap<SemanticId, Arc<AdmittedImplicit>>,
        selected: &BTreeSet<SemanticId>,
    ) -> Result<BTreeSet<SemanticId>> {
        if selected.is_empty() {
            return Ok(BTreeSet::new());
        }
        let topology = Self::build(implicit)?;
        let roots = selected
            .iter()
            .map(|id| {
                topology
                    .nodes
                    .get(id)
                    .copied()
                    .ok_or_else(|| CompileError::Missing("selected implicit descriptor".into()))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(topology
            .order(Some(roots))?
            .into_iter()
            .map(|node| topology.graph[node])
            .collect())
    }

    fn edge_capacity(implicit: &BTreeMap<SemanticId, Arc<AdmittedImplicit>>) -> Result<usize> {
        implicit
            .values()
            .flat_map(|inner| inner.bodies())
            .try_fold(0_usize, |sum, body| {
                sum.checked_add(body.math.providers().len())
                    .ok_or_else(|| MathError::Limit("supplier topology extent").into())
            })
    }
    /// Reserve the maximum retained graph and index before a lazy fill can allocate.
    /// Every edge requires one provider reference; inventory demand can only reduce it.
    pub(super) fn reservation(
        implicit: &BTreeMap<SemanticId, Arc<AdmittedImplicit>>,
        bodies: &BTreeMap<ContentHash, Arc<AdmittedBody>>,
        case: &CaseStructure,
    ) -> Result<usize> {
        let edges = Self::edge_capacity(implicit)?;
        let overflow = || CompileError::from(MathError::Limit("supplier topology traversal"));
        // NodeFiltered retains the complete index space: reserve whole-inventory
        // DFS/toposort visit maps, stacks, result and the selected-identity tree.
        let mut scratch = implicit.len().checked_mul(512).ok_or_else(overflow)?;
        // DFS can queue parallel edges before discovering their shared target.
        // Bound both edge-sized stack capacities, including geometric Vec growth.
        scratch = edges
            .checked_add(1)
            .and_then(|count| count.checked_mul(4 * size_of::<petgraph::graph::NodeIndex>()))
            .and_then(|bytes| scratch.checked_add(bytes))
            .ok_or_else(overflow)?;
        // Value-demand discovery compacts stages and constructs formal/symbol,
        // offset, numeric-slot and reachability inventories. One body is queried
        // at a time, so retain one peak allowance, not a sum of sequential work.
        let mut peak_demand = 0;
        for body in implicit
            .values()
            .flat_map(|inner| inner.bodies())
            .chain(bodies.values())
        {
            let demand = body
                .math
                .slot_count()
                .checked_mul(512)
                .and_then(|bytes| bytes.checked_add(body.math.retained_bytes().checked_mul(2)?))
                .and_then(|bytes| {
                    bytes.checked_add(body.math.output_count().checked_mul(size_of::<usize>())?)
                })
                .and_then(|bytes| bytes.checked_add(body.math.providers().len().checked_mul(192)?))
                .ok_or_else(overflow)?;
            peak_demand = peak_demand.max(demand);
        }
        scratch = scratch.checked_add(peak_demand).ok_or_else(overflow)?;
        // Do not look up observation bodies at admission: a missing selected
        // body must retain its request-time failure boundary. This maximum
        // bounds accumulated roots even for a body shared by many instances.
        let providers = bodies
            .values()
            .map(|body| body.math.providers().len())
            .max()
            .unwrap_or(0);
        let mut peak_outputs = 0;
        for instance in case.instances() {
            peak_outputs = peak_outputs.max(
                instance
                    .contributions
                    .len()
                    .checked_mul(64)
                    .ok_or_else(overflow)?,
            );
            scratch = scratch
                .checked_add(
                    providers
                        .checked_mul(2 * size_of::<petgraph::graph::NodeIndex>())
                        .ok_or_else(overflow)?,
                )
                .ok_or_else(overflow)?;
        }
        scratch = scratch.checked_add(peak_outputs).ok_or_else(overflow)?;
        implicit
            .len()
            .checked_mul(
                size_of::<petgraph::graph::Node<SemanticId>>()
                    + size_of::<(SemanticId, petgraph::graph::NodeIndex)>()
                    + 128,
            )
            .and_then(|bytes| {
                bytes.checked_add(edges.checked_mul(size_of::<petgraph::graph::Edge<()>>())?)
            })
            .and_then(|bytes| bytes.checked_add(size_of::<once_cell::sync::OnceCell<Self>>() + 64))
            .and_then(|bytes| bytes.checked_add(scratch))
            .ok_or_else(|| MathError::Limit("supplier topology allocation").into())
    }
    fn order(
        &self,
        roots: Option<Vec<petgraph::graph::NodeIndex>>,
    ) -> Result<Vec<petgraph::graph::NodeIndex>> {
        let graph = &self.graph;
        let order = if let Some(roots) = roots {
            let mut required = BTreeSet::new();
            let reversed = petgraph::visit::Reversed(graph);
            let mut traversal = petgraph::visit::Dfs::empty(reversed);
            for root in roots {
                traversal.move_to(root);
                while let Some(node) = traversal.next(reversed) {
                    required.insert(graph[node]);
                }
            }
            drop(traversal);
            toposort(
                &petgraph::visit::NodeFiltered::from_fn(graph, |node| {
                    required.contains(&graph[node])
                }),
                None,
            )
        } else {
            toposort(graph, None)
        };
        order.map_err(|_| CompileError::Missing("cyclic implicit provider dependency".into()))
    }
    fn build(implicit: &BTreeMap<SemanticId, Arc<AdmittedImplicit>>) -> Result<Self> {
        let edges = Self::edge_capacity(implicit)?;
        let mut graph = DiGraph::<SemanticId, ()>::with_capacity(implicit.len(), edges);
        let nodes = implicit
            .keys()
            .map(|id| (*id, graph.add_node(*id)))
            .collect::<BTreeMap<_, _>>();
        for (id, inner) in implicit {
            for body in inner.bodies() {
                let demanded = body.math.provider_demands_for_outputs(
                    &(0..body.math.output_count()).collect::<Vec<_>>(),
                    DerivativeOrder::Value,
                )?;
                for provider in body
                    .math
                    .providers()
                    .iter()
                    .filter(|provider| demanded.contains_key(&provider.key()))
                {
                    if let Some(dependency) = nodes.get(&provider.id) {
                        graph.add_edge(*dependency, nodes[id], ());
                    }
                }
            }
        }
        Ok(Self { graph, nodes })
    }
}

impl AdmittedModeling {
    /// Provider demand from the bound case's actual output and formal-coordinate maps.
    /// Fixed inputs remain Value unless an authored partial or explicit response consumes them.
    pub fn provider_demands_for_plan(
        &self,
        plan: &CasePlan,
        order: DerivativeOrder,
    ) -> Result<BTreeMap<pse_kernels::ProviderKey, DerivativeOrder>> {
        let mut demands = BTreeMap::new();
        for demand in plan.demands() {
            let body = self
                .bodies
                .get(&demand.body)
                .ok_or_else(|| CompileError::Missing("bound provider demand body".into()))?;
            if plan
                .bodies()
                .get(&demand.body)
                .is_none_or(|bound| bound.as_ref() != body.math.as_ref())
            {
                return Err(CompileError::Missing(
                    "bound provider demand source correspondence".into(),
                ));
            }
            let order = order.max(match demand.product {
                pse_math::jets::ProductKind::SelectedPartials => demand.order,
                pse_math::jets::ProductKind::JacobianAction
                | pse_math::jets::ProductKind::TransposeJacobianAction => DerivativeOrder::First,
                pse_math::jets::ProductKind::WeightedHessian
                | pse_math::jets::ProductKind::WeightedHessianAction => DerivativeOrder::Second,
            });
            for (key, required) in body.math.provider_demands_for_selection(
                &demand.outputs,
                &demand.coordinates,
                order,
            )? {
                demands
                    .entry(key)
                    .and_modify(|o: &mut DerivativeOrder| *o = (*o).max(required))
                    .or_insert(required);
            }
        }
        Ok(demands)
    }
    /// Provider demand of selected original observations before implicit dependency propagation.
    pub fn provider_demands_for(
        &self,
        rows: Option<&BTreeSet<SemanticId>>,
        order: DerivativeOrder,
    ) -> Result<BTreeMap<pse_kernels::ProviderKey, DerivativeOrder>> {
        let mut demands = BTreeMap::new();
        for instance in self
            .case
            .instances()
            .iter()
            .filter(|i| rows.is_none_or(|r| r.contains(&i.instance)))
        {
            let body = self
                .bodies
                .get(&instance.body)
                .ok_or_else(|| CompileError::Missing("observation body".into()))?;
            let outputs = instance
                .contributions
                .iter()
                .map(|contribution| contribution.output)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            for (key, required) in body.math.provider_demands_for_outputs(&outputs, order)? {
                demands
                    .entry(key)
                    .and_modify(|o: &mut DerivativeOrder| *o = (*o).max(required))
                    .or_insert(required);
            }
        }
        Ok(demands)
    }
    /// Admit only providers reachable from selected observations and their hint/residual dependencies.
    pub(in crate::workspace::modeling) fn implicit_order_for(
        &self,
        rows: Option<&BTreeSet<SemanticId>>,
    ) -> Result<Vec<Arc<AdmittedImplicit>>> {
        let topology = self
            .supplier_topology
            .get_or_try_init(|| SupplierTopology::build(&self.implicit))?;
        let graph = &topology.graph;
        let nodes = &topology.nodes;
        let order = if let Some(rows) = rows {
            let mut pending = Vec::new();
            for instance in self
                .case
                .instances()
                .iter()
                .filter(|i| rows.contains(&i.instance))
            {
                let body = self
                    .bodies
                    .get(&instance.body)
                    .ok_or_else(|| CompileError::Missing("observation body".into()))?;
                let outputs = instance
                    .contributions
                    .iter()
                    .map(|contribution| contribution.output)
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>();
                let demanded = body
                    .math
                    .provider_demands_for_outputs(&outputs, DerivativeOrder::Value)?;
                pending.extend(
                    body.math
                        .providers()
                        .iter()
                        .filter(|provider| demanded.contains_key(&provider.key()))
                        .filter_map(|provider| nodes.get(&provider.id).copied()),
                );
            }
            topology.order(Some(pending))
        } else {
            topology.order(None)
        }?;
        Ok(order
            .into_iter()
            .map(|node| self.implicit[&graph[node]].clone())
            .collect())
    }
}

/// Exact final implicit descriptors and the actual supplier definitions that own them.
type Admitted = (
    BTreeMap<SemanticId, Arc<AdmittedImplicit>>,
    BTreeMap<ContentHash, AdmittedProvider>,
);
/// Apply only final admitted supplier outputs. The model owns typed substitution,
/// original member/range preservation, actual port joins and source correspondence.
pub(super) fn resolve_region(
    model: &SpecializedModel,
    region: &SelectedRegion,
    definitions: &BTreeMap<SemanticId, Arc<AdmittedImplicit>>,
    independent: &BTreeSet<SemanticId>,
    context: &TypeContext<'_>,
) -> Result<SelectedRegion> {
    let mut replacements = Vec::new();
    for supplier in definitions.values() {
        let spec = supplier.descriptor.spec();
        if spec.outputs.len() != supplier.unknowns.len() {
            return Err(CompileError::Missing(
                "implicit descriptor original output correspondence".into(),
            ));
        }
        for (ordinal, (member, port)) in supplier.unknowns.iter().zip(&spec.outputs).enumerate() {
            if port.id != *member {
                return Err(CompileError::Missing(
                    "implicit descriptor original output identity".into(),
                ));
            }
            if independent.contains(member) {
                continue;
            }
            replacements.push(pse_modeling::resolved::ProviderSubstitution {
                member: *member,
                selection: pse_modeling::resolved::ProviderSelection {
                    identity: spec.identity(),
                    output: ordinal,
                },
                inputs: spec
                    .inputs
                    .iter()
                    .map(|port| (port.id, port.quantity))
                    .collect(),
                quantity: port.quantity,
            });
        }
    }
    Ok(region.substitute_providers(model, &replacements, context)?)
}
pub(super) fn admit(
    db: &dyn CompilerDb,
    inventory: Inventory,
    p: &super::Projection,
    supplied: BTreeMap<SemanticId, Arc<AdmittedImplicit>>,
) -> Result<Admitted> {
    let registry = inventory.quantities(db);
    let checker = inventory.preconditions(db);
    let physical = physical_identity(registry, checker);
    let context = TypeContext {
        admissions: None,
        formula_authority: None,
        quantities: registry,
        preconditions: checker.as_ref(),
        scope: &p.physical_scope,
    };
    let mut definitions = supplied;
    let mut calls = BTreeMap::new();
    for (id, supplier) in &definitions {
        let spec = supplier.descriptor.spec();
        if spec.id != *id
            || spec
                .outputs
                .iter()
                .map(|port| port.id)
                .ne(supplier.unknowns.iter().copied())
            || supplier.bodies().any(|body| body.spec.physical != physical)
        {
            return Err(MathError::Contract(
                "supplied implicit system has a different admitted compiler inventory".into(),
            )
            .into());
        }
        calls.insert(spec.identity(), supplier.descriptor.clone());
    }
    let row_declarations = p
        .model
        .equations
        .iter()
        .map(|row| (row.id, row.lineage.declaration))
        .collect::<BTreeMap<_, _>>();
    for implicit in &p.implicit {
        checkpoint(db);
        if definitions.contains_key(&implicit.id) {
            return Err(MathError::Contract(
                "supplied implicit system cannot be admitted again".into(),
            )
            .into());
        }
        let ordered_inputs = implicit
            .unknowns
            .iter()
            .chain(&implicit.inputs)
            .map(|id| {
                p.model.symbols.get(id).ok_or_else(|| {
                    CompileError::Missing("actual implicit source input declaration".into())
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let provider_lookups = std::cell::RefCell::new(BTreeMap::new());
        let independent = implicit
            .unknowns
            .iter()
            .chain(&implicit.inputs)
            .copied()
            .collect();
        let admit_body = |definition, regions: &[SelectedRegion]| -> Result<Arc<ImplicitBody>> {
            // Replace reached child unknowns with their admitted supplier inputs before
            // enforcing the enclosing residual's complete independent input layout.
            let regions = regions
                .iter()
                .map(|region| {
                    resolve_region(&p.model, region, &definitions, &independent, &context)
                })
                .collect::<Result<Vec<_>>>()?;
            let region = SelectedRegion::combine(&regions, &ordered_inputs, &context)?;
            let mut lookup = |name: &str| {
                let selected = provider(db, inventory, name.to_owned()).ok_or_else(|| {
                    MathError::Contract(format!("unregistered external implementation {name}"))
                })?;
                provider_lookups
                    .borrow_mut()
                    .insert(name.to_owned(), selected.clone());
                Ok(selected)
            };
            let providers = crate::typed_math::prepare_selected_providers(
                &region.syntax,
                &region.support,
                &mut lookup,
                &calls,
                &context,
            )?;
            let result = crate::typed_math::Request {
                definition,
                syntax: &region.syntax,
                inputs: &region.inputs,
                support: &region.support,
                providers: &providers,
                physical,
                structure: implicit.spec.revision,
                limits: p.body_limits,
            }
            .admit(registry, checker.as_ref(), db.cancel());
            let body = result.map_err(|cause| MathError::Instance {
                instance: implicit.id,
                checked_members: region.checked_members.clone(),
                checked_sources: region.checked_sources.clone(),
                occurrences: region.occurrences.clone(),
                cause: Box::new(cause),
            })?;
            let bindings = pse_math::SourceBindings::new(
                implicit.id,
                &body.math,
                region.checked_members,
                region.checked_sources,
                region.occurrences,
            )?;
            Ok(Arc::new(ImplicitBody {
                body: Arc::new(body),
                bindings,
            }))
        };
        let mut residuals = Vec::new();
        for r in &implicit.residuals {
            let body = admit_body(r.id, &r.regions)?;
            if implicit.algorithm == ImplicitAlgorithm::AffineRates {
                pse_math::implicit::Affine::new(&body.math, implicit.unknowns.len())?;
            }
            let assessment = r
                .assessment
                .as_ref()
                .map(|assessment| -> Result<RegimeAssessment> {
                    Ok(RegimeAssessment {
                        eligibility: admit_body(
                            pse_ids::named_id(r.id, "eligibility"),
                            std::slice::from_ref(&assessment.eligibility),
                        )?,
                        criterion: admit_body(
                            pse_ids::named_id(r.id, "criterion"),
                            &assessment.criterion,
                        )?,
                    })
                })
                .transpose()?;
            let mut numerical_targets = Vec::new();
            for port in &implicit.spec.outputs {
                let source = p.model.symbols.get(&port.id).ok_or_else(|| {
                    CompileError::Missing("nested numerical coordinate source".into())
                })?;
                numerical_targets.push(NumericalTargetFact {
                    id: port.id,
                    kind: pse_model::generated::enums::NumericalTarget::Variable,
                    role: NumericalTargetRole::NestedUnknown,
                    declaration: source.lineage.declaration,
                    quantity: port.quantity,
                    unit: port.unit,
                    integer: false,
                    location: NumericalLocation::Nested(None),
                    observation: None,
                    support: Some(BTreeSet::from([port.id])),
                    nested: false,
                    selected: true,
                });
            }
            for (position, (id, quantity)) in r.rows.iter().zip(body.quantities()).enumerate() {
                let declaration = row_declarations
                    .get(id)
                    .copied()
                    .ok_or_else(|| CompileError::Missing("nested numerical row source".into()))?;
                let region = r
                    .regions
                    .get(position)
                    .ok_or_else(|| CompileError::Missing("nested numerical row support".into()))?;
                numerical_targets.push(NumericalTargetFact {
                    id: *id,
                    kind: pse_model::generated::enums::NumericalTarget::Row,
                    role: NumericalTargetRole::NestedRow,
                    declaration,
                    quantity: *quantity,
                    unit: registry
                        .quantity_type(*quantity)
                        .map_err(MathError::from)?
                        .canonical_unit,
                    integer: false,
                    location: NumericalLocation::Nested(Some(position)),
                    observation: Some(AnnotationObservation::Position(position)),
                    support: Some(
                        region
                            .reads
                            .numeric
                            .union(&region.reads.guards)
                            .copied()
                            .collect(),
                    ),
                    nested: false,
                    selected: true,
                });
            }
            let numerical_targets = numerical_targets.into();
            residuals.push(AdmittedResidual {
                id: r.id,
                body,
                rows: r.rows.clone(),
                assessment,
                hints: if r.hints.is_empty() {
                    None
                } else {
                    Some(admit_body(
                        pse_ids::named_id(r.id, "numerical-hints"),
                        &r.hints
                            .iter()
                            .map(|hint| hint.3.clone())
                            .collect::<Vec<_>>(),
                    )?)
                },
                terms: if r.terms.is_empty() {
                    None
                } else {
                    Some(admit_body(
                        pse_ids::named_id(r.id, "nominal-terms"),
                        &r.terms
                            .iter()
                            .map(|term| term.0.clone())
                            .collect::<Vec<_>>(),
                    )?)
                },
                scales: r.scales.clone(),
                numerical_annotations: policy::lower(
                    db,
                    policy::AnnotationFacts {
                        mode: NumericalAnnotationMode::Implicit,
                        targets: numerical_targets,
                        hints: r
                            .hints
                            .iter()
                            .enumerate()
                            .map(|(position, hint)| NumericalAnnotation {
                                target: hint.0,
                                declaration: hint.1,
                                kind: hint.2,
                                quantity: hint.4,
                                observation: AnnotationObservation::Position(position),
                                target_kind: if implicit.unknowns.contains(&hint.0) {
                                    pse_model::generated::enums::NumericalTarget::Variable
                                } else {
                                    pse_model::generated::enums::NumericalTarget::Row
                                },
                                nested: false,
                                constant_bound: false,
                                reads: hint
                                    .3
                                    .reads
                                    .numeric
                                    .union(&hint.3.reads.guards)
                                    .copied()
                                    .collect(),
                            })
                            .collect(),
                        scales: r
                            .scales
                            .iter()
                            .map(|scale| NumericalScale {
                                target: scale.row,
                                declaration: scale.source,
                                scheme: scale.scheme,
                                observations: ScalingObservations::Range(scale.terms.clone()),
                                nested: false,
                            })
                            .collect(),
                    },
                ),
                isolation: None,
            });
        }
        if implicit.selection.meaning == ImplicitMeaning::MinimumScore {
            let cancel = pse_diagnostics::Cancellation::new(false);
            for residual in &mut residuals {
                let Some(assessment) = &residual.assessment else {
                    continue;
                };
                let Some(hints) = &residual.hints else {
                    continue;
                };
                // The first validated binding covers constant authored physical
                // boxes. Parameter-dependent bounds need their own uniform bound
                // predicates; resolving them at one input cannot establish a chart.
                let bounds = residual
                    .numerical_annotations
                    .hints
                    .iter()
                    .enumerate()
                    .filter(|(_, hint)| {
                        matches!(hint.kind, ModelingHint::Lower | ModelingHint::Upper)
                    })
                    .map(|(index, _)| index)
                    .collect::<Vec<_>>();
                if implicit.unknowns.iter().any(|unknown| {
                    [ModelingHint::Lower, ModelingHint::Upper]
                        .into_iter()
                        .any(|kind| {
                            !residual
                                .numerical_annotations
                                .hints
                                .iter()
                                .any(|hint| hint.target == *unknown && hint.kind == kind)
                        })
                }) {
                    continue;
                }
                let support = hints.math.incidence(
                    &bounds,
                    &(0..hints.math.input_count()).collect::<Vec<_>>(),
                    &cancel,
                )?;
                if !support.support().controls.is_empty()
                    || support
                        .support()
                        .first
                        .iter()
                        .any(|columns| !columns.is_empty())
                {
                    continue;
                }
                residual.isolation = pse_math::factorable::root_isolation_program(
                    residual.id,
                    &residual.body.math,
                    &assessment.eligibility.math,
                    &assessment.criterion.math,
                    &cancel,
                    BodyLimits::default().occurrences,
                )
                .map_err(|error| match error {
                    pse_math::factorable::FactorableError::Math(error) => CompileError::from(error),
                    error => CompileError::Missing(error.to_string()),
                })?
                .map(Arc::new);
            }
        }
        let available_order = residuals
            .iter()
            .map(|r| {
                r.body.math.available_order_for_outputs(
                    &(0..r.body.math.output_count()).collect::<Vec<_>>(),
                    &(0..r.body.math.input_count()).collect::<Vec<_>>(),
                )
            })
            .collect::<std::result::Result<Vec<_>, _>>()?
            .into_iter()
            .min()
            .ok_or_else(|| CompileError::Missing("empty implicit residuals".into()))?;
        let mut meaning = implicit.selection.meaning.clone();
        let equivalence = if residuals.len() == 1 {
            pse_math::implicit::graph_equivalence(
                &residuals[0].body.math,
                implicit.unknowns.len(),
                implicit.selection.sign,
            )?
        } else {
            SelectionEquivalence::Unestablished
        };
        if meaning == ImplicitMeaning::Relation {
            if equivalence == SelectionEquivalence::NondegenerateAffine {
                meaning = ImplicitMeaning::Unique;
            } else {
                return Err(CompileError::Missing("implicit expression requires an explicit function selector; numerical starts and bounds do not select mathematical meaning".into()));
            }
        }
        // Static graph evidence establishes one root per alternative. Nonlinear
        // alternatives instead require the whole selection's runtime certificate:
        // one native iterate per regime cannot exclude better unobserved roots.
        let unique_regimes = if matches!(meaning, ImplicitMeaning::MinimumScore) {
            let mut unique = true;
            for residual in &residuals {
                if pse_math::implicit::graph_equivalence(
                    &residual.body.math,
                    implicit.unknowns.len(),
                    None,
                )? != SelectionEquivalence::NondegenerateAffine
                {
                    unique = false;
                    break;
                }
            }
            unique
        } else {
            false
        };
        let static_neighborhood = unique_regimes
            || implicit.algorithm == ImplicitAlgorithm::AffineRates
            || equivalence != SelectionEquivalence::Unestablished;
        let isolated_regimes = meaning == ImplicitMeaning::MinimumScore
            && residuals
                .iter()
                .all(|residual| residual.isolation.is_some());
        let mut neighborhood = if static_neighborhood || isolated_regimes {
            available_order
        } else {
            DerivativeOrder::Value
        };
        if residuals
            .iter()
            .filter_map(|r| r.assessment.as_ref())
            .any(|a| {
                a.eligibility.math.branch_local_order() < DerivativeOrder::First
                    || a.criterion.math.branch_local_order() < DerivativeOrder::First
            })
        {
            neighborhood = DerivativeOrder::Value;
        }
        let mut selection = ImplicitSelection {
            neighborhood_evidence: if neighborhood == DerivativeOrder::Value {
                SelectionNeighborhood::Unestablished
            } else if static_neighborhood {
                SelectionNeighborhood::Static
            } else {
                SelectionNeighborhood::RuntimeIsolation
            },
            identity: implicit.selection.identity,
            meaning,
            anchors: if implicit.selection.anchors.is_empty() {
                None
            } else {
                Some(admit_body(
                    pse_ids::named_id(implicit.id, "selection-anchors"),
                    &implicit.selection.anchors,
                )?)
            },
            restriction: implicit
                .selection
                .restriction
                .as_ref()
                .map(|region| {
                    admit_body(
                        pse_ids::named_id(implicit.id, "selection-neighborhood"),
                        std::slice::from_ref(region),
                    )
                })
                .transpose()?,
            sign: implicit.selection.sign,
            strict: implicit.selection.strict,
            equivalence,
            neighborhood,
        };
        if selection
            .restriction
            .as_ref()
            .is_some_and(|b| b.math.branch_local_order() < DerivativeOrder::First)
        {
            selection.neighborhood = DerivativeOrder::Value;
            selection.neighborhood_evidence = SelectionNeighborhood::Unestablished;
        }
        let mut spec = implicit.spec.clone();
        spec.derivatives = available_order.min(selection.neighborhood);
        spec.smoothness = spec.derivatives;
        let descriptor = AdmittedProvider::new(spec, registry)
            .map_err(|e| CompileError::Missing(e.to_string()))?;
        calls.insert(descriptor.spec().identity(), descriptor.clone());
        definitions.insert(
            implicit.id,
            Arc::new(AdmittedImplicit {
                selection,
                algorithm: implicit.algorithm.clone(),
                descriptor,
                unknowns: implicit.unknowns.clone(),
                residuals,
                provider_lookups: provider_lookups.into_inner(),
            }),
        );
    }
    Ok((definitions, calls))
}
#[cfg(test)]
mod topology_tests {
    use super::*;

    #[test]
    fn supplier_topology_parallel_edges_need_edge_sized_traversal_scratch() {
        let mut graph = DiGraph::new();
        let supplier = SemanticId::from_bytes([1; 16]);
        let consumer = SemanticId::from_bytes([2; 16]);
        let first = graph.add_node(supplier);
        let second = graph.add_node(consumer);
        for _ in 0..4096 {
            graph.add_edge(first, second, ());
        }
        let reversed = petgraph::visit::Reversed(&graph);
        let mut traversal = petgraph::visit::Dfs::new(reversed, second);
        assert_eq!(traversal.next(reversed), Some(second));
        assert_eq!(traversal.stack.len(), graph.edge_count());
        assert!(traversal.stack.len() > graph.node_count());
        drop(traversal);
        let topology = SupplierTopology {
            graph,
            nodes: BTreeMap::from([(supplier, first), (consumer, second)]),
        };
        assert_eq!(
            topology.order(Some(vec![second])).unwrap(),
            vec![first, second]
        );
        assert_eq!(topology.order(None).unwrap(), vec![first, second]);
        assert_eq!(topology.graph.edge_count(), 4096);
    }

    #[test]
    fn supplier_topology_selects_ancestors_without_admitting_unrelated_cycles() {
        let mut graph = DiGraph::new();
        let ids = (1..=5)
            .map(|value| SemanticId::from_bytes([value; 16]))
            .collect::<Vec<_>>();
        let nodes = ids
            .iter()
            .map(|id| (*id, graph.add_node(*id)))
            .collect::<BTreeMap<_, _>>();
        graph.add_edge(nodes[&ids[0]], nodes[&ids[1]], ());
        graph.add_edge(nodes[&ids[2]], nodes[&ids[3]], ());
        graph.add_edge(nodes[&ids[3]], nodes[&ids[2]], ());
        let topology = SupplierTopology { graph, nodes };
        let selected = topology.order(Some(vec![topology.nodes[&ids[1]]])).unwrap();
        assert_eq!(
            selected
                .iter()
                .map(|node| topology.graph[*node])
                .collect::<Vec<_>>(),
            ids[..2]
        );
        assert!(topology.order(Some(Vec::new())).unwrap().is_empty());
        assert_eq!(
            topology.order(Some(vec![topology.nodes[&ids[4]]])).unwrap(),
            vec![topology.nodes[&ids[4]]]
        );
        assert!(topology.order(Some(vec![topology.nodes[&ids[2]]])).is_err());
        assert!(topology.order(None).is_err());
        assert_eq!(topology.graph.node_count(), 5);
        assert_eq!(topology.graph.edge_count(), 3);
    }
}
