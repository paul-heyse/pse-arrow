// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Conditional block initialization and finite supplied continuation on the staged
//! sequence primitive (A6): each stage is an overlay over the immutable original values,
//! each block a solve step on one native session, and a stage commits only when every
//! block's candidate is a result. Libraries own iteration.
use super::{
    ExecutableCase, MathRuntimeError, MathService, Preparation, WorkerBudget, Workspace,
    solves::{Predecessor, SolveHandle, SolverProfile},
};
use pse_backend_native::{self as native, execution, kinsol, quality::Tolerances, solve::*};
use pse_columnar::flight::FlightCancellation;
use pse_compiler::workspace::Profile;
use pse_ids::SemanticId;
use pse_math::binding::CaseValues;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

/// Compiled predecessor-ordered blocks, with explicit conditional input coordinates.
#[derive(Clone, Debug)]
pub struct PreparedInitialization {
    snapshot: execution::Snapshot,
    numerical_owner: super::numerics::PolicyResolutionOwner,
    quantities: pse_math::SharedAllocation<pse_quantity::QuantityRegistry>,
    preconditions: pse_math::SharedAllocation<pse_quantity::PhysicalPreconditions>,
    targets: Vec<pse_math::numerics::TargetSpec>,
    requirements: Arc<Vec<pse_math::numerics::SourcedRequirement>>,
    blocks: Vec<ConditionalBlock>,
    /// Complete original evaluator for transactional final block assessment.
    original: Option<Arc<ExecutableCase>>,
    _owner: Arc<super::products::ProductOwner>,
}
/// One conditional block: its boundary, its value-independent view and its assembled
/// programs, prepared once and rebound to each stage's values.
#[derive(Clone, Debug)]
struct ConditionalBlock {
    boundary: pse_structural::initialization::Block,
    view: pse_compiler::workspace::PreparedBlock,
    executable: Arc<ExecutableCase>,
    owner: Arc<super::products::ProductOwner>,
}
impl PreparedInitialization {
    /// Resolve every conditional block before worker acquisition. No fallback follows a failed attempt.
    pub fn strategies(
        &self,
        solver: &SolverProfile,
        numerics: &pse_model::numerics::ResolvedNumericalPolicy,
    ) -> Result<Vec<native::routing::Route>, native::ProblemError> {
        self.blocks
            .iter()
            .map(|block| {
                let c = native::assembled::contract(&block.executable.assembly);
                let mut facts = native::routing::oracle_facts(&c, false, true);
                facts.derivatives = block.view.plan.available_order();
                facts.prepared_derivatives = block.view.plan.order();
                let rows: Vec<_> = block
                    .view
                    .plan
                    .structure()
                    .rows()
                    .iter()
                    .map(|row| row.id)
                    .collect();
                let normalization = pse_math::normalization::Normalization::from_policy(
                    numerics,
                    block.view.plan.columns(),
                    &rows,
                )?;
                let tolerances =
                    Tolerances::from_policy(numerics, block.view.plan.columns(), &rows)?;
                let accuracy =
                    ResolvedAccuracy::resolve(&numerics.policy, &tolerances, &normalization)?;
                native::routing::Requirements {
                    table: &execution::LINKED,
                    facts: &facts,
                    intent: SolveIntent::Initialize,
                    numerical_psd: false,
                    least_squares: false,
                    controls: &solver.controls,
                    settings: &solver.backend,
                    sensitivity: false,
                    context: native::routing::Context {
                        snapshot: self.snapshot.clone(),
                        pending_classes: &[],
                        structure: Some(native::routing::Structure {
                            variables: block.view.plan.columns().to_vec(),
                            equations: block
                                .view
                                .plan
                                .structure()
                                .rows()
                                .iter()
                                .map(|row| pse_structural::incidence::Constraint {
                                    id: row.id,
                                    lower: row.lower.is_finite().then_some(row.lower),
                                    upper: row.upper.is_finite().then_some(row.upper),
                                })
                                .collect(),
                            witness: block.view.structure.clone().into(),
                        }),
                        oracle: Some(&c),
                        guards: &BTreeMap::new(),
                        budgets: Some(execution::Budgets {
                            tolerances: &tolerances,
                            normalization: &normalization,
                            accuracy: &accuracy,
                        }),
                        coefficients: None,
                        cone: None,
                        factorable: None,
                        certificate: None,
                        prepared: &[
                            native::routing::ArtifactDemand::Representation(
                                execution::Representation::Nlp,
                            ),
                            native::routing::ArtifactDemand::Representation(
                                execution::Representation::Roots,
                            ),
                        ],
                        refusals: &BTreeMap::new(),
                    },
                }
                .select(solver.selection)
            })
            .collect()
    }
    /// Shared preparation/submission admission for immutable stage overlays. This is the
    /// one admission of an initialization request (F26): its intent must be a root or
    /// initialization purpose, it carries no explicit preprocessing or numerical convexity
    /// strategy (blocks run without either), and its typed backend settings must belong to
    /// every block's route, so another backend's settings never apply to a block.
    pub fn validate_profile(
        &self,
        values: &CaseValues,
        profile: &InitializationProfile,
    ) -> Result<
        (
            Vec<native::routing::Route>,
            pse_math::SharedAllocation<pse_model::numerics::ResolvedNumericalPolicy>,
        ),
        MathRuntimeError,
    > {
        let solver = &profile.solver;
        solver.controls.validate()?;
        if !matches!(solver.intent, SolveIntent::Initialize | SolveIntent::Root) {
            return Err(native::ProblemError::Contract(
                "initialization requires the root or initialize intent".into(),
            )
            .into());
        }
        if matches!(solver.presolve, native::presolve::Policy::Explicit { .. })
            || solver.convexity != pse_math::convexity::ConvexityPolicy::Exact
        {
            return Err(native::ProblemError::Contract(
                "initialization blocks have no explicit preprocessing or numerical convexity strategy"
                    .into(),
            )
            .into());
        }
        // Blocks differ in coordinates, so reuse cannot be required; a stage's block may
        // reuse its predecessor stage's retained session when allowed.
        if solver.controls.start == StartPolicy::Explicit
            || solver.controls.reuse == ReusePolicy::RequireReuse
        {
            return Err(native::ProblemError::Contract("initialization uses declared guesses or previous accepted stages; blocks cannot require native reuse".into()).into());
        }
        if solver.controls.threads != 1
            || profile.stages.is_empty()
            || profile
                .stages
                .len()
                .checked_mul(self.blocks.len())
                .is_none_or(|v| v > 4096)
        {
            return Err(MathRuntimeError::Limit(
                "serial finite initialization schedule",
            ));
        }
        let numerics = self.numerical_owner.resolve(
            self.quantities.clone(),
            self.preconditions.clone(),
            &self.targets,
            &self.requirements,
            &solver.numerics,
        )?;
        let strategies = self.strategies(solver, &numerics)?;
        let solved: BTreeSet<_> = self
            .boundaries()
            .flat_map(|b| b.members.columns.iter().copied())
            .collect();
        for stage in &profile.stages {
            if stage.iter().any(|(id, v)| {
                solved.contains(id) || !values.scalars.contains_key(id) || !v.is_finite()
            }) {
                return Err(native::ProblemError::Contract(
                    "continuation may only replace declared finite fixed/parameter inputs".into(),
                )
                .into());
            }
        }
        for ConditionalBlock { boundary, .. } in &self.blocks {
            Tolerances::from_policy(&numerics, &boundary.members.columns, &boundary.members.rows)?;
            if boundary
                .inputs
                .iter()
                .chain(&boundary.members.columns)
                .any(|id| !values.scalars.contains_key(id))
            {
                return Err(native::ProblemError::Contract(
                    "missing initialization boundary value".into(),
                )
                .into());
            }
        }
        Ok((strategies, numerics))
    }
    /// Inspect the immutable conditional boundaries before executing native work.
    pub fn boundaries(&self) -> impl Iterator<Item = &pse_structural::initialization::Block> {
        self.blocks.iter().map(|b| &b.boundary)
    }
}
/// One source-attributed block attempt. A failed block never commits trial coordinates.
#[derive(Clone, Debug)]
pub struct BlockAttempt {
    /// Supplied continuation stage, zero-based.
    pub stage: usize,
    /// Selected contextual block route, retained on failure.
    pub strategy: native::routing::Route,
    /// Exact original rows, solved columns and predecessor inputs.
    pub boundary: pse_structural::initialization::Block,
    /// Faithful native report or typed pre-execution failure.
    pub result: Result<Box<SolveReport>, Arc<MathRuntimeError>>,
    /// Actual shared driver events when execution reached an observed strategy.
    pub trace: Option<Arc<super::strategy::Trace>>,
    /// Whether its original-quality-validated coordinates were committed.
    pub committed: bool,
}
/// Initialized values and complete bounded attempt history, without an optimum claim.
#[derive(Debug)]
pub struct InitializationReport {
    /// Actual initialization submission identity shared by its block event rows.
    pub run_id: pse_model::generated::identities::RunId,
    /// Immutable original specification, including its authored guesses.
    pub original: CaseValues,
    /// Committed solved unknowns only; never fixed inputs or temporary overlays.
    pub values: CaseValues,
    /// Stage-local overlays and candidates, retained even after failure or cancellation.
    pub stages: Vec<StageAttempt>,
    /// Whether the last completed stage used the original fixed/parameter bindings.
    pub original_bindings_restored: bool,
    /// Actual block attempts in predecessor and continuation order.
    pub attempts: Vec<BlockAttempt>,
    /// Number of fully committed stages.
    pub completed_stages: usize,
    /// Cancellation request observed before the next block.
    pub cancelled: bool,
    _owner: Arc<pse_columnar::AllocationLease>,
}
/// One transactional continuation stage; intermediate successes are not publication.
#[derive(Clone, Debug)]
pub struct StageAttempt {
    /// Zero-based declared stage.
    pub stage: usize,
    /// Exact replacements of immutable original bindings.
    pub overlay: BTreeMap<SemanticId, f64>,
    /// Stage working values, including failed-stage evidence, never the specification.
    pub candidate: CaseValues,
    /// All required blocks completed with independently accepted coordinates.
    pub completed: bool,
}
/// Explicit finite continuation data and the solve profile of every block.
#[derive(Clone, Debug)]
pub struct InitializationProfile {
    /// The request's solve profile: selection applied to each structural block, finite
    /// controls, route-typed backend settings and ID-keyed numerical meaning. Blocks run
    /// under the initialize intent without preprocessing; [`PreparedInitialization::
    /// validate_profile`] refuses a profile that asks for anything else.
    pub solver: SolverProfile,
    /// Finite prescribed parameter/fixed-coordinate replacements. Use one empty map
    /// for ordinary initialization. Prior values seed a stage only under PreviousAccepted start policy.
    pub stages: Vec<BTreeMap<SemanticId, f64>>,
}
/// Accounted terminal report for an explicitly declared map or Picard splitting.
#[derive(Debug)]
pub struct DeclaredRootReport {
    /// Native root outcome with original residual validation.
    pub report: SolveReport,
    /// Actual submission identity, retained across every trace projection.
    pub run_id: pse_model::generated::identities::RunId,
    /// Shared numerical execution and original candidate assessment events.
    pub strategy: Arc<super::strategy::Trace>,
    /// Complete independently checked original candidate, including non-port local state.
    pub original_candidate: Option<CaseValues>,
    decision: crate::workflow::numerics::CandidateDecision,
    _owner: Arc<pse_columnar::AllocationLease>,
}

impl DeclaredRootReport {
    /// Original residual quality and the declared numerical policy own this permission.
    pub fn candidate_use(&self) -> pse_model::generated::enums::CandidateUse {
        self.decision.usability
    }
}

/// One admitted original-equation unit execution. The profile is interpreted once,
/// before iteration; an enclosing causal map owns its compact binding and native state.
#[cfg(feature = "solver-kinsol")]
#[derive(Clone, Debug)]
pub(crate) struct PreparedConditionalUnit {
    assessment: Option<native::structural::Assessment>,
    snapshot: execution::Snapshot,
    pub(crate) view: pse_compiler::workspace::PreparedBlock,
    pub(crate) executable: Arc<ExecutableCase>,
    profile: SolverProfile,
    backend: Backend,
    normalization: pse_math::normalization::Normalization,
    tolerances: Tolerances,
    accuracy: ResolvedAccuracy,
    profile_key: pse_ids::ContentHash,
    pub(crate) foreign_bytes: usize,
}

#[cfg(feature = "solver-kinsol")]
impl PreparedConditionalUnit {
    pub(crate) fn inner_accuracy_identity(&self) -> pse_ids::ContentHash {
        self.profile_key
    }
    pub(crate) const fn backend(&self) -> Backend {
        self.backend
    }
    #[cfg(test)]
    pub(crate) fn row_magnitudes(&self, id: SemanticId) -> Option<(f64, f64)> {
        let index = self
            .view
            .boundary
            .members
            .rows
            .iter()
            .position(|row| *row == id)?;
        Some((self.normalization.rows[index], self.tolerances.rows[index]))
    }
    pub(crate) const fn route(&self) -> native::routing::Route {
        native::routing::Route::Native(self.backend)
    }
}

#[cfg(feature = "solver-kinsol")]
impl MathService {
    /// Compile the admitted local submodel on the compiler's existing job owner.
    #[expect(
        clippy::too_many_arguments,
        reason = "a unit preparation binds ownership, boundary, selected residuals, compiler and resolved numerical policy"
    )]
    pub(crate) async fn prepare_conditional_unit(
        self: &Arc<Self>,
        workspace: Workspace,
        model: super::modeling::ModelingPreparation,
        source: Preparation,
        node: SemanticId,
        quantities: Arc<pse_quantity::QuantityRegistry>,
        inputs: BTreeSet<SemanticId>,
        outputs: BTreeSet<SemanticId>,
        rows: BTreeSet<SemanticId>,
        unknowns: BTreeSet<SemanticId>,
        compiler_profile: Profile,
        profile: SolverProfile,
        numerics: pse_math::SharedAllocation<pse_model::numerics::ResolvedNumericalPolicy>,
        driver: &crate::CancelSource,
    ) -> Result<PreparedConditionalUnit, MathRuntimeError> {
        admit_unit_profile(&profile, &numerics)?;
        let preconditions = source.compiled().preconditions.clone();
        let control = FlightCancellation::default();
        let operation =
            self.job_retained(1, super::WITHIN_WORKSPACE, control.clone(), move |flag| {
                let _lease = workspace.lease;
                let compiler = workspace.compiler.lock().map_err(|_| {
                    MathRuntimeError::Infrastructure("compiler lock poisoned".into())
                })?;
                let view = compiler.prepare_modeling_conditional_unit(
                    model.compiled(),
                    source.compiled(),
                    node,
                    &inputs,
                    &outputs,
                    &rows,
                    &unknowns,
                    compiler_profile,
                    &flag,
                )?;
                let bytes = view.plan.retained_bytes();
                Ok((view, bytes))
            });
        tokio::pin!(operation);
        let (view, lease) = tokio::select! {
            result = &mut operation => result?,
            () = driver.cancelled() => { control.cancel(); let _ = operation.await; return Err(MathRuntimeError::Cancelled); }
        };
        // Boundary difference identities are a checked projection of supplied members;
        // keep the originating member beside each local row through policy transport.
        let boundary_inputs =
            pse_math::index::CheckedInventory::new(&view.boundary.inputs, |id| {
                pse_compiler::workspace::ModelingOutput::ConditionalBoundary(*id).row_id()
            })?;
        let boundaries: Vec<_> = view
            .plan
            .structure()
            .rows()
            .iter()
            .enumerate()
            .filter_map(|(index, row)| boundary_inputs.get(&row.id).map(|id| (index, *id)))
            .collect();
        let derivation = self
            .policy_resolution_owner()
            .derivation(&numerics, boundaries.len())?;
        let mut effective_numerics = numerics.as_ref().clone();
        if !boundaries.is_empty() {
            use pse_model::generated::enums::NumericalTarget;
            let mut projections = Vec::new();
            let mut defaults = Vec::new();
            let source_access = pse_math::numerics::TargetAccess::new(&numerics)?;
            for (index, original) in &boundaries {
                let row = &view.plan.structure().rows()[*index];
                let target = pse_math::numerics::TargetSpec {
                    id: row.id,
                    kind: NumericalTarget::Row,
                    quantity: row.quantity,
                    unit: quantities
                        .quantity_type(row.quantity)
                        .map_err(pse_math::MathError::from)?
                        .canonical_unit,
                    integer: false,
                    declared_tolerance: None,
                };
                if let Some(source) = source_access.find(NumericalTarget::Observable, *original) {
                    projections.push(pse_math::numerics::TargetProjection {
                        source: source.id,
                        source_kind: source.kind,
                        target,
                    });
                } else {
                    defaults.push(target);
                }
            }
            let mut h = pse_ids::FramedHasher::new(pse_ids::Frame::NumericalProjectionV1);
            h.hash(&numerics.key);
            if !projections.is_empty() {
                let projected = pse_math::numerics::project_difference(
                    &quantities,
                    &preconditions,
                    &numerics,
                    &projections,
                )?;
                h.hash(&projected.key);
                effective_numerics.targets.extend(projected.targets);
            }
            if !defaults.is_empty() {
                let additional = pse_math::numerics::resolve(
                    &quantities,
                    &preconditions,
                    &defaults,
                    &[],
                    &numerics.policy,
                )?;
                h.hash(&additional.key);
                effective_numerics.targets.extend(additional.targets);
            }
            effective_numerics.key = h.finish_hash();
        }
        let numerics = derivation.finish(effective_numerics)?;
        native::structural::admit(&view.structure, native::structural::Mode::Roots)?;
        let executable = self
            .assemble_functions(
                pse_compiler::workspace::PreparedFunctions {
                    plan: view.plan.clone(),
                    artifacts: view.artifacts.clone(),
                },
                lease,
                driver,
            )
            .await?;
        let contract = native::assembled::contract(&executable.assembly);
        let mut facts = native::routing::oracle_facts(&contract, false, true);
        facts.derivatives = view.plan.available_order();
        facts.prepared_derivatives = view.plan.order();
        facts.domains = view
            .plan
            .columns()
            .iter()
            .map(|id| {
                view.plan
                    .structure()
                    .variables()
                    .iter()
                    .find(|v| v.port.id == *id)
                    .map_or(
                        pse_model::generated::enums::ModelingVariableDomain::Continuous,
                        |v| v.domain,
                    )
            })
            .collect();
        let snapshot = execution::Snapshot::observe(&execution::LINKED);
        let normalization = pse_math::normalization::Normalization::from_policy(
            &numerics,
            &view.boundary.members.columns,
            &view.boundary.members.rows,
        )?;
        let tolerances = Tolerances::from_policy(
            &numerics,
            &view.boundary.members.columns,
            &view.boundary.members.rows,
        )?;
        let accuracy = ResolvedAccuracy::resolve(&numerics.policy, &tolerances, &normalization)?;
        let decision = native::routing::Requirements {
            table: &execution::LINKED,
            facts: &facts,
            intent: profile.intent,
            numerical_psd: false,
            least_squares: false,
            controls: &profile.controls,
            settings: &profile.backend,
            sensitivity: false,
            context: native::routing::Context {
                snapshot: snapshot.clone(),
                pending_classes: &[],
                structure: Some(native::routing::Structure {
                    variables: view.plan.columns().to_vec(),
                    equations: view
                        .plan
                        .structure()
                        .rows()
                        .iter()
                        .map(|row| pse_structural::incidence::Constraint {
                            id: row.id,
                            lower: row.lower.is_finite().then_some(row.lower),
                            upper: row.upper.is_finite().then_some(row.upper),
                        })
                        .collect(),
                    witness: view.structure.clone().into(),
                }),
                oracle: Some(&contract),
                guards: &BTreeMap::new(),
                budgets: Some(execution::Budgets {
                    tolerances: &tolerances,
                    normalization: &normalization,
                    accuracy: &accuracy,
                }),
                coefficients: None,
                cone: None,
                factorable: None,
                certificate: None,
                prepared: &[native::routing::ArtifactDemand::Representation(
                    execution::Representation::Roots,
                )],
                refusals: &BTreeMap::new(),
            },
        }
        .decision(profile.selection);
        let route = decision.route()?;
        let native::routing::Route::Native(backend) = route else {
            return Err(native::ProblemError::Unsupported(
                "conditional unit needs a native root capability".into(),
            )
            .into());
        };
        let adapter = execution::adapter(backend);
        if adapter.representation() != execution::Representation::Roots {
            return Err(native::ProblemError::Unsupported("conditional unit requires a selected root representation; request an explicit simultaneous strategy".into()).into());
        }
        let demand = native::routing::derivative_demand(adapter.capability(), &profile.controls)
            .unwrap_or(pse_kernels::DerivativeOrder::Value);
        if executable.assembly.order() < demand {
            return Err(native::ProblemError::Unsupported(
                "conditional residual compilation does not meet selected root derivative demand"
                    .into(),
            )
            .into());
        }
        adapter.admit_settings(&profile.backend, &profile.controls, &snapshot)?;
        adapter.admit_contract(
            &contract,
            &BTreeMap::new(),
            &profile.backend,
            execution::Budgets {
                tolerances: &tolerances,
                normalization: &normalization,
                accuracy: &accuracy,
            },
        )?;
        let profile_key = super::solves::profile_key(&profile)?.as_id();
        let foreign_bytes = self.policy.foreign_allowance(&profile.controls);
        Ok(PreparedConditionalUnit {
            assessment: decision.structure,
            snapshot,
            view,
            executable,
            profile,
            backend,
            normalization,
            tolerances,
            accuracy,
            profile_key,
            foreign_bytes,
        })
    }

    /// Build one private conditional evaluator after the enclosing causal owner has
    /// reserved all native residency. The returned session never leaves that worker.
    pub(crate) fn conditional_unit_session(
        &self,
        prepared: &PreparedConditionalUnit,
        value: impl Fn(SemanticId) -> Option<f64>,
        providers: &BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
        execution: &Execution,
        budget: &Arc<WorkerBudget>,
    ) -> Result<ConditionalUnitSession, native::ProblemError> {
        execution.check()?;
        // Charge duplicate immutable callback inventories, compact inputs, starts and
        // directional support before constructing any of them. Opaque native residency
        // is a separate additive allowance on the enclosing operation.
        let support = native::assembled::AlgebraicOracle::root_support_allowance(
            &prepared.executable.assembly,
            &prepared.view.structure,
        )?;
        let wrapper = prepared
            .view
            .structure
            .contributions
            .len()
            .checked_mul(64)
            .and_then(|bytes| {
                bytes.checked_add(
                    prepared
                        .view
                        .boundary
                        .members
                        .columns
                        .len()
                        .checked_add(prepared.view.boundary.members.rows.len())?
                        .checked_mul(512)?,
                )
            })
            .and_then(|bytes| {
                bytes.checked_add(
                    prepared
                        .executable
                        .assembly
                        .input_ids()
                        .len()
                        .checked_mul(64)?,
                )
            })
            .and_then(|bytes| bytes.checked_add(support)?.checked_add(8192))
            .ok_or_else(|| native::ProblemError::memory("conditional callback wrapper extent"))?;
        let charge = budget
            .charge(wrapper)
            .map_err(MathRuntimeError::into_problem)?;
        let super::ExecutionWorker {
            worker,
            _case,
            _charge,
        } = self
            .worker(
                prepared.executable.clone(),
                providers,
                execution.scope()?,
                budget,
            )
            .map_err(MathRuntimeError::into_problem)?;
        let initial = prepared
            .view
            .boundary
            .members
            .columns
            .iter()
            .map(|id| {
                value(*id).filter(|value| value.is_finite()).ok_or_else(|| {
                    native::ProblemError::Contract("missing conditional authored start".into())
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let oracle = native::assembled::AlgebraicOracle::compact(worker, value)?
            .with_structural_analysis(prepared.view.structure.clone().into())?
            .with_normalization(prepared.normalization.clone())?;
        Ok(ConditionalUnitSession {
            retained: execution::Retained::default(),
            oracle: native::assembled::SharedRootOracle::new(oracle)?,
            initial,
            _case,
            _worker: _charge,
            _wrapper: charge,
            #[cfg(test)]
            calls: 0,
            #[cfg(test)]
            last_reused: false,
            #[cfg(test)]
            last_setup_reused: false,
        })
    }
}

/// Thread-confined conditional state owned by exactly one enclosing causal map.
/// Native callbacks are destroyed before their evaluator/provider storage charges.
#[cfg(feature = "solver-kinsol")]
#[derive(Debug)]
pub(crate) struct ConditionalUnitSession {
    retained: execution::Retained,
    oracle: native::assembled::SharedRootOracle,
    initial: Vec<f64>,
    _case: Arc<ExecutableCase>,
    _worker: super::WorkerCharge,
    _wrapper: super::WorkerCharge,
    #[cfg(test)]
    pub(crate) calls: usize,
    #[cfg(test)]
    pub(crate) last_reused: bool,
    #[cfg(test)]
    pub(crate) last_setup_reused: bool,
}
#[cfg(feature = "solver-kinsol")]
impl Drop for ConditionalUnitSession {
    fn drop(&mut self) {
        self.retained.clear();
        self.retained.release();
    }
}
#[cfg(feature = "solver-kinsol")]
impl ConditionalUnitSession {
    pub(crate) fn evaluate(
        &mut self,
        prepared: &PreparedConditionalUnit,
        updates: &[(SemanticId, f64)],
        execution: &Execution,
    ) -> Result<Vec<(SemanticId, f64)>, native::ProblemError> {
        let result = (|| {
            execution.check()?;
            self.oracle.refill_inputs(updates)?;
            let mut nested = execution.clone();
            nested.time_limit = nested.time_limit.min(
                nested
                    .started
                    .elapsed()
                    .saturating_add(prepared.profile.controls.time_limit),
            );
            nested.memory = Some(prepared.foreign_bytes);
            let report = execution::roots(
                execution::Step {
                    adapter: execution::adapter(prepared.backend),
                    snapshot: &prepared.snapshot,
                    structure: prepared.assessment.as_ref(),
                    settings: &prepared.profile.backend,
                    controls: &prepared.profile.controls,
                    accuracy: &prepared.accuracy,
                    execution: nested,
                    tolerances: &prepared.tolerances,
                    normalization: &prepared.normalization,
                    compatibility: Compatibility {
                        layout: prepared.view.plan.structure().key(),
                        profile: prepared.profile_key,
                        data: self.oracle.input_identity()?,
                        backend: prepared.backend,
                    },
                    warm: None,
                },
                &mut self.retained,
                execution::Roots {
                    oracle: Box::new(self.oracle.clone()),
                    // Allocation retention never substitutes a previous solved root.
                    initial: &self.initial,
                    owner: None,
                },
            )?;
            #[cfg(test)]
            {
                self.calls += 1;
                self.last_reused = report.evidence.reused_native_state;
                self.last_setup_reused =
                    report.metrics.get("setup.initial_reused") == Some(&Metric::Bool(true));
            }
            let primal = block_candidate(
                &prepared.view.boundary,
                Some(&report),
                &prepared.profile.numerics,
            )
            .map_err(|_| conditional_failure(&report))?;
            execution.check()?;
            Ok(prepared
                .view
                .boundary
                .members
                .columns
                .iter()
                .copied()
                .zip(primal.iter().copied())
                .collect())
        })();
        if result.is_err() {
            self.retained.clear();
            self.retained.release();
        }
        result
    }
}

/// Preserve actual evaluation witnesses before lowering an unqualified native exit.
#[cfg(feature = "solver-kinsol")]
fn conditional_failure(report: &SolveReport) -> native::ProblemError {
    if let Some(cause) = report.shared_effective_failure() {
        return native::ProblemError::Math(pse_math::MathError::Typed {
            retained: cause.retained_bytes(),
            cause: pse_model::diagnostic::DiagnosticCause::from_shared(cause),
        });
    }
    native::ProblemError::native(
        native::NativeStatus {
            backend: report.backend,
            code: report.termination.code,
            name: report.termination.name.clone(),
        },
        report.termination.category,
        report
            .termination
            .message
            .clone()
            .unwrap_or_else(|| "conditional unit candidate refused by original assessment".into()),
    )
}

#[cfg(feature = "solver-kinsol")]
fn admit_unit_profile(
    profile: &SolverProfile,
    numerics: &pse_model::numerics::ResolvedNumericalPolicy,
) -> Result<(), MathRuntimeError> {
    profile.controls.validate()?;
    if !matches!(profile.intent, SolveIntent::Root | SolveIntent::Initialize)
        || profile.controls.threads != 1
        || profile.controls.start != StartPolicy::NoPriorStart
        || profile.controls.reuse == ReusePolicy::RequireReuse
        || profile.sensitivity.is_some()
        || matches!(profile.presolve, native::presolve::Policy::Explicit { .. })
        || profile.convexity != pse_math::convexity::ConvexityPolicy::Exact
        || profile.numerics.key() != numerics.policy.key()
    {
        return Err(native::ProblemError::Contract("conditional unit needs one serial declared-root execution with original guesses, shared physical numerical policy and no optimization/required reuse procedure".into()).into());
    }
    Ok(())
}

impl MathService {
    /// Execute a declared root/map factory on its owning admitted worker. Causal
    /// sweeps may use nested native unit calculations on this same admission; they
    /// must not recursively request CPU permits. Mutable oracles never enter Salsa.
    /// The factory charges each evaluator it builds to the job's worker budget.
    #[expect(
        clippy::too_many_arguments,
        reason = "a declared root solve binds its contract, start, settings, controls, accuracy, tolerances and evaluator factory as independent inputs"
    )]
    pub fn solve_declared_root(
        self: &Arc<Self>,
        contract: native::OracleContract,
        initial: Vec<f64>,
        settings: kinsol::Settings,
        controls: Controls,
        accuracy: ResolvedAccuracy,
        tolerances: Tolerances,
        policy: pse_model::numerics::NumericalPolicy,
        profile: pse_ids::ContentHash,
        original_candidate: Option<Arc<std::sync::Mutex<Option<CaseValues>>>>,
        // Known factory containers, excluding evaluator charges drawn as constructed.
        // None preserves conservative entry for opaque external/provider factories.
        factory_bytes: Option<usize>,
        // Actual nested adapter scopes and their simultaneous opaque residency.
        mut native_backends: Vec<Backend>,
        resident_foreign_bytes: usize,
        factory: impl FnOnce(
            Execution,
            Arc<WorkerBudget>,
        ) -> Result<kinsol::Function, native::ProblemError>
        + Send
        + 'static,
    ) -> Result<SolveHandle<DeclaredRootReport>, MathRuntimeError> {
        controls.validate()?;
        contract.validate(pse_kernels::DerivativeOrder::Value)?;
        let n = contract.variables.len();
        tolerances.validate(n, contract.rows.len())?;
        if controls.threads != 1 || initial.len() != n || initial.iter().any(|v| !v.is_finite()) {
            return Err(native::ProblemError::Contract(
                "declared serial root dimensions/controls".into(),
            )
            .into());
        }
        let bytes = n
            .checked_add(contract.rows.len())
            .and_then(|v| v.checked_mul(512))
            .and_then(|v| v.checked_add(controls.report_allowance().ok()?))
            .ok_or(MathRuntimeError::Limit("declared root result allowance"))?;
        let owner = self.reserve("math:declared-root-result", bytes)?;
        let cancel = FlightCancellation::default();
        let control = cancel.clone();
        let deadline = std::time::Instant::now()
            .checked_add(controls.time_limit)
            .ok_or(MathRuntimeError::Limit("declared root deadline extent"))?;
        let scope = pse_kernels::ExecutionScope::new(cancel.flag().clone(), Some(deadline));
        let progress = Arc::new(Progress::new(controls.history));
        let events = progress.clone();
        let service = self.clone();
        let foreign_bytes = self.policy.foreign_allowance(&controls);
        let factory_demand = factory_bytes.unwrap_or(self.policy.worker_bytes);
        if factory_demand > self.policy.worker_bytes {
            return Err(MathRuntimeError::Limit(
                "declared root construction capacity",
            ));
        }
        let stack = self.policy.stack_bytes;
        if !native_backends.contains(&Backend::Kinsol) {
            native_backends.push(Backend::Kinsol);
        }
        let adapters = native_backends
            .iter()
            .map(|backend| execution::adapter(*backend))
            .collect::<Vec<_>>();
        let stacks = execution::scope_stack_bytes(&adapters, 1, stack)?;
        let job_bytes = factory_demand
            .checked_add(foreign_bytes)
            .and_then(|bytes| bytes.checked_add(resident_foreign_bytes))
            .and_then(|bytes| bytes.checked_add(stacks))
            .ok_or(MathRuntimeError::Limit(
                "declared root resident/scope allowance",
            ))?;
        let entry = self
            .admit_entry(1, job_bytes, &control, Some(deadline), None)?
            .with_construction(super::jobs::ConstructionEstimate::from_bound(
                factory_bytes,
                self.policy.worker_bytes,
            ));
        let run_id = pse_operations::mint_id();
        let (tx, receiver) = tokio::sync::oneshot::channel();
        tokio::spawn(pse_operations::canonical::inherit_activity(async move {
            // Known factories draw each live evaluator before allocating it; the
            // entry lease covers their separately bounded container construction.
            // Unknown factories retain the pre-reserved conservative worker extent.
            let budget = if factory_bytes.is_some() {
                WorkerBudget::drawing_for(service.policy.worker_bytes - factory_demand, &service)
            } else {
                WorkerBudget::new(service.policy.worker_bytes)
            };
            let worker_service = service.clone();
            let result = service
                .job_scoped_on_entry(1, job_bytes, control, Some(deadline), entry, move |flag| {
                    let adapters = native_backends
                        .iter()
                        .map(|backend| execution::adapter(*backend))
                        .collect::<Vec<_>>();
                    execution::scoped(&adapters, 1, stack, move || {
                        let original = contract.identity;
                        let start_identity =
                            super::strategy::target::point_identity(original, &initial);
                        let request = pse_model::strategy::CompositionRequest::default();
                        let mut factory = Some(factory);
                        let (result, trace) = super::strategy::target::callable(
                            &worker_service,
                            super::strategy::target::Source {
                                original,
                                preparation: original,
                                profile,
                                profile_projection: None,
                                backend: Some(Backend::Kinsol),
                                solver: None,
                                controls: &controls,
                                request: &request,
                                start: pse_model::strategy::StartOrigin::Specification,
                                start_identity: Some(start_identity),
                            },
                            &scope,
                            |_, admission| {
                                let mut execution =
                                    Execution::within(flag.clone(), &controls, scope.clone())?;
                                let budget = budget.with_admission(admission.clone());
                                execution.work_admission = Some(admission);
                                execution.progress = events.clone();
                                execution.memory = Some(foreign_bytes);
                                let producer = factory.take().ok_or_else(|| {
                                    native::ProblemError::Internal(
                                        "one-shot declared root producer dispatched twice".into(),
                                    )
                                })?;
                                let function = producer(execution.clone(), budget)?;
                                let actual = function.contract();
                                if actual.identity != contract.identity
                                    || actual.rows != contract.rows
                                    || actual.variables.len() != n
                                    || actual.variables.iter().zip(&contract.variables).any(
                                        |(a, b)| {
                                            a.id != b.id
                                                || a.lower.to_bits() != b.lower.to_bits()
                                                || a.upper.to_bits() != b.upper.to_bits()
                                        },
                                    )
                                {
                                    return Err(native::ProblemError::Internal(
                                        "root factory differs from admitted source contract".into(),
                                    )
                                    .into());
                                }
                                let stamp = Compatibility {
                                    layout: contract.identity,
                                    profile,
                                    data: contract.identity,
                                    backend: Backend::Kinsol,
                                };
                                let mut session = kinsol::Session::new(
                                    function,
                                    settings.clone(),
                                    execution.clone(),
                                    stamp,
                                )?;
                                // The declared root runs and qualifies with the budgets its caller
                                // resolved from the numerical policy (F20).
                                let mut report = session.solve(
                                    &initial,
                                    &controls,
                                    &accuracy,
                                    execution,
                                    &tolerances,
                                    None,
                                )?;
                                native::quality::qualify(&mut report, &accuracy);
                                drop(session);
                                Ok(report.with_owner(owner.clone()))
                            },
                            |report| Some(report),
                            super::strategy::observe_native,
                            |report, observed| {
                                super::strategy::target::original_assessment(
                                    crate::workflow::numerics::native_use(report, &policy),
                                    super::strategy::cause_native(report),
                                    None,
                                    observed,
                                )
                            },
                        )?;
                        let decision = crate::workflow::numerics::native_use(&result, &policy);
                        // The trace owns its independent allocation; native result ownership
                        // survives the joined worker without cloning evaluator storage.
                        Ok(DeclaredRootReport {
                            report: result,
                            run_id,
                            strategy: trace,
                            original_candidate: original_candidate
                                .as_ref()
                                .map(|candidate| {
                                    candidate.lock().map(|value| value.clone()).map_err(|_| {
                                        MathRuntimeError::Infrastructure(
                                            "original candidate lock poisoned".into(),
                                        )
                                    })
                                })
                                .transpose()?
                                .flatten(),
                            decision,
                            _owner: owner,
                        })
                    })
                })
                .await;
            let _ = tx.send(result);
        }));
        Ok(SolveHandle {
            cancel,
            receiver: Some(receiver),
            progress,
        })
    }
    /// Reuse the same conditional engine with an authored case's selected physical bindings.
    pub async fn prepare_modeling_initialization(
        self: &Arc<Self>,
        workspace: Workspace,
        case: Preparation,
        profile: Profile,
        numerical: super::solves::NumericalInputs,
        driver: &crate::CancelSource,
    ) -> Result<PreparedInitialization, MathRuntimeError> {
        let original = self.assemble(case.clone()).await?;
        let quantities = case.compiled().quantities.clone();
        let preconditions = case.compiled().preconditions.clone();
        let mut targets = case
            .compiled()
            .plan
            .structure()
            .numerical_targets(&quantities)?;
        targets.extend(numerical.targets);
        use pse_model::HeapUsage;
        let numerical_bytes = numerical
            .declarations
            .iter()
            .map(|r| size_of_val(r) + r.declaration.owned_bytes())
            .sum::<usize>()
            + size_of_val(targets.as_slice());
        let control = FlightCancellation::default();
        let operation =
            self.job_retained(1, super::WITHIN_WORKSPACE, control.clone(), move |flag| {
                let _lease = workspace.lease;
                let compiler = workspace.compiler.lock().map_err(|_| {
                    MathRuntimeError::Infrastructure("compiler lock poisoned".into())
                })?;
                let products =
                    compiler.prepare_bound_initialization(case.compiled(), profile, &flag)?;
                let bytes = pse_compiler::workspace::PreparedBlock::retained_group_bytes(&products)
                    .and_then(|bytes| bytes.checked_add(numerical_bytes))
                    .ok_or(MathRuntimeError::Limit("initialization product extent"))?;
                Ok((products, bytes))
            });
        tokio::pin!(operation);
        let (products, lease) = tokio::select! {r=&mut operation=>r?,()=driver.cancelled()=>{control.cancel();let _=operation.await;return Err(MathRuntimeError::Cancelled);}};
        let mut prepared = self
            .own_initialization(
                products,
                lease,
                quantities,
                preconditions,
                targets,
                numerical.declarations,
            )
            .await?;
        prepared.original = Some(original);
        Ok(prepared)
    }
    async fn own_initialization(
        self: &Arc<Self>,
        products: Arc<Vec<pse_compiler::workspace::PreparedBlock>>,
        lease: Arc<pse_columnar::AllocationLease>,
        quantities: pse_math::SharedAllocation<pse_quantity::QuantityRegistry>,
        preconditions: pse_math::SharedAllocation<pse_quantity::PhysicalPreconditions>,
        targets: Vec<pse_math::numerics::TargetSpec>,
        requirements: Vec<pse_math::numerics::SourcedRequirement>,
    ) -> Result<PreparedInitialization, MathRuntimeError> {
        let owner = self.shared_product(
            vec![3, Arc::as_ptr(&products) as usize],
            products.clone(),
            lease,
            Vec::new(),
        )?;
        let mut blocks = Vec::with_capacity(products.len());
        for block in products.iter() {
            let mut artifacts = vec![];
            for request in block.artifacts.iter() {
                artifacts.push(self.artifact(request.clone()).await?);
            }
            let plan = Arc::new(block.plan.as_ref().clone().with_owner(owner.clone()));
            let layout = self.reserve(
                "case_operation_layout",
                plan.operation_layout_allocation_bound()?,
            )?;
            let assembly =
                Arc::new(plan.assemble(artifacts.iter().map(|a| a.program.clone()).collect())?);
            let mut view = block.clone();
            view.plan = plan;
            blocks.push(ConditionalBlock {
                boundary: block.boundary.clone(),
                view,
                owner: owner.clone(),
                executable: Arc::new(ExecutableCase {
                    assembly,
                    _artifacts: artifacts,
                    _layout: layout,
                    _owner: owner.clone(),
                }),
            });
        }
        Ok(PreparedInitialization {
            numerical_owner: self.policy_resolution_owner(),
            snapshot: execution::Snapshot::observe(&execution::LINKED),
            quantities,
            preconditions,
            targets,
            requirements: Arc::new(requirements),
            blocks,
            original: None,
            _owner: owner,
        })
    }
    /// Admit the complete finite schedule once, then run it as one staged sequence on one
    /// native session: each stage composes its overlay over the immutable original values,
    /// each block is a solve step bound to those values, and a block's solved coordinates
    /// become its successors' inputs only after its candidate is a result.
    pub fn initialize(
        self: &Arc<Self>,
        prepared: PreparedInitialization,
        values: CaseValues,
        providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
        profile: InitializationProfile,
    ) -> Result<SolveHandle<InitializationReport>, MathRuntimeError> {
        let (strategies, numerics) = prepared.validate_profile(&values, &profile)?;
        let controls = &profile.solver.controls;
        let size = prepared.blocks.iter().try_fold(
            values.scalars.len().saturating_mul(64),
            |total, b| {
                b.boundary
                    .members
                    .columns
                    .len()
                    .checked_add(b.boundary.members.rows.len())
                    .and_then(|v| v.checked_mul(512))
                    .and_then(|v| v.checked_add(controls.report_allowance().ok()?))
                    .and_then(|v| v.checked_mul(profile.stages.len()))
                    .and_then(|v| total.checked_add(v))
                    .ok_or(MathRuntimeError::Limit("initialization result allowance"))
            },
        )?;
        let owner = self.reserve("math:initialization-results", size)?;
        let progress = Arc::new(Progress::new(controls.history));
        let deadline = std::time::Instant::now()
            .checked_add(controls.time_limit)
            .ok_or(MathRuntimeError::Limit(
                "initialization task deadline extent",
            ))?;
        let control = FlightCancellation::default();
        let scope = pse_kernels::ExecutionScope::new(control.flag(), Some(deadline));
        let run_id = pse_operations::mint_id();
        let session = self.open_session()?;
        let admission = profile.solver.composition.limits.map(|limits| {
            super::strategy::admission::TaskAdmission::new_for(limits, scope.clone(), self, false)
        });
        let service = self.clone();
        let events = progress.clone();
        Ok(SolveHandle::supervise(progress, move |cancel| async move {
            let operation_cancel = cancel.clone();
            let operation_controls = profile.solver.controls.clone();
            let operation_scope = scope.clone();
            let report = session
                .sequence(operation_scope, operation_controls, &cancel, move |local| {
                    let mut run = Blocks {
                        service: &service,
                        local,
                        prepared: &prepared,
                        providers: &providers,
                        profile: &profile,
                        strategies: &strategies,
                        numerics: &numerics,
                        progress: &events,
                        owner: &owner,
                        cancel: &operation_cancel,
                        scope,
                        admission,
                        run_id,
                        bound: vec![None; prepared.blocks.len()],
                        attempts: Vec::new(),
                    };
                    local.run_ready(run.stages(values))
                })
                .await;
            session.close().await;
            report
        }))
    }
}
/// The block initialization planner over one native session.
struct Blocks<'a, 'native> {
    service: &'a Arc<MathService>,
    local: &'a super::staged::LocalOperation<'native>,
    prepared: &'a PreparedInitialization,
    providers: &'a BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
    profile: &'a InitializationProfile,
    strategies: &'a [native::routing::Route],
    numerics: &'a pse_math::SharedAllocation<pse_model::numerics::ResolvedNumericalPolicy>,
    progress: &'a Arc<Progress>,
    owner: &'a Arc<pse_columnar::AllocationLease>,
    cancel: &'a crate::CancelSource,
    scope: pse_kernels::ExecutionScope,
    admission: Option<Arc<super::strategy::admission::TaskAdmission>>,
    run_id: pse_model::generated::identities::RunId,
    /// Each block's view as last bound; later stages rebind its values (A6).
    bound: Vec<Option<Preparation>>,
    attempts: Vec<BlockAttempt>,
}
impl Blocks<'_, '_> {
    async fn stages(
        &mut self,
        original: CaseValues,
    ) -> Result<InitializationReport, MathRuntimeError> {
        let mut completed_stages = 0;
        let mut stages = Vec::new();
        let mut committed = CaseValues {
            scalars: BTreeMap::new(),
        };
        let solved: BTreeSet<_> = self
            .prepared
            .boundaries()
            .flat_map(|b| b.members.columns.iter().copied())
            .collect();
        let previous_accepted = self.profile.solver.controls.start == StartPolicy::PreviousAccepted;
        for (stage, updates) in self.profile.stages.iter().enumerate() {
            // The stage overlay exists only in this stage's working values; the original
            // specification is never written.
            let mut values = original.clone();
            if previous_accepted {
                values
                    .scalars
                    .extend(committed.scalars.iter().map(|(k, v)| (*k, *v)));
            }
            values.scalars.extend(updates.iter().map(|(k, v)| (*k, *v)));
            let first = self.attempts.len();
            let mut completed = true;
            for index in 0..self.prepared.blocks.len() {
                if self.cancel.token().is_cancelled() {
                    completed = false;
                    break;
                }
                let previous = (previous_accepted && stage > 0)
                    .then(|| {
                        let id = self.prepared.blocks[index].boundary.id;
                        self.attempts
                            .iter()
                            .rposition(|a| a.boundary.id == id && a.committed)
                    })
                    .flatten();
                let attempted = self.attempt(index, &values, previous).await;
                let trace = match &attempted {
                    Ok((_, trace)) => Some(trace.clone()),
                    Err(MathRuntimeError::Strategy { trace, .. }) => Some(trace.clone()),
                    _ => None,
                };
                let result = attempted.map(|(report, _)| report).map_err(Arc::new);
                let boundary = &self.prepared.blocks[index].boundary;
                let committed_block = commit_block(
                    &mut values,
                    boundary,
                    result.as_deref().ok(),
                    &self.numerics.policy,
                )
                .is_ok();
                self.attempts.push(BlockAttempt {
                    stage,
                    strategy: self.strategies[index],
                    boundary: boundary.clone(),
                    result,
                    trace,
                    committed: committed_block,
                });
                if !committed_block {
                    completed = false;
                    break;
                }
            }
            if completed && let Some(original_case) = &self.prepared.original {
                let values = values.clone();
                let numerics = self.numerics.clone();
                let structure = original_case.assembly.structure().clone();
                let admission = self.admission.clone();
                let validation_scope = self.scope.clone();
                self.scope.check().map_err(native::ProblemError::Provider)?;
                let bytes = original_case.assembly.numeric_worker_bytes();
                completed = self.local.construct(bytes, |_storage| {
                    let budget = WorkerBudget::new(bytes);
                    let mut worker = self.service.worker(
                        original_case.clone(),
                        self.providers,
                        validation_scope.clone(),
                        &budget,
                    )?;
                    let mut execution = self.local.execution().clone();
                    execution.work_admission =
                        admission.map(|owner| -> Arc<dyn WorkAdmission> { owner });
                    let rows = execution.counted(
                        WorkEvidence {
                            evaluations: Some(1),
                            iterations: Some(0),
                            factorizations: Some(0),
                            proof_steps: Some(0),
                        },
                        || worker.worker().constraints(&values).map_err(Into::into),
                    )?;
                    let row_ids: Vec<_> = structure.rows().iter().map(|r| r.id).collect();
                    let tolerances = Tolerances::from_policy(
                        &numerics,
                        &structure
                            .variables()
                            .iter()
                            .filter(|v| !v.fixed)
                            .map(|v| v.port.id)
                            .collect::<Vec<_>>(),
                        &row_ids,
                    )?;
                    let rows_ok = rows.iter().zip(structure.rows()).zip(&tolerances.rows).all(
                        |((value, row), tolerance)| {
                            value.is_finite()
                                && *value >= row.lower - tolerance
                                && *value <= row.upper + tolerance
                        },
                    );
                    let bounds_ok = structure.variables().iter().all(|variable| {
                        values.scalars.get(&variable.port.id).is_some_and(|value| {
                            value.is_finite()
                                && variable.lower.is_none_or(|lower| *value >= lower)
                                && variable.upper.is_none_or(|upper| *value <= upper)
                        })
                    });
                    Ok(rows_ok && bounds_ok)
                })?;
            }
            if completed {
                committed.scalars = values
                    .scalars
                    .iter()
                    .filter(|(id, _)| solved.contains(id))
                    .map(|(id, v)| (*id, *v))
                    .collect();
                completed_stages += 1;
            } else {
                for attempt in &mut self.attempts[first..] {
                    attempt.committed = false;
                }
            }
            stages.push(StageAttempt {
                stage,
                overlay: updates.clone(),
                candidate: values,
                completed,
            });
            if !completed {
                break;
            }
        }
        let original_bindings_restored = stages.last().is_some_and(|s| {
            s.completed
                && s.overlay
                    .iter()
                    .all(|(id, v)| original.scalars.get(id) == Some(v))
        });
        Ok(InitializationReport {
            run_id: self.run_id,
            original,
            values: committed,
            stages,
            original_bindings_restored,
            attempts: std::mem::take(&mut self.attempts),
            completed_stages,
            cancelled: self.cancel.token().is_cancelled(),
            _owner: self.owner.clone(),
        })
    }
    /// One block attempt: the block's view rebound to the stage values, the shared step
    /// executor on the session, and, in a later stage, the block's committed predecessor as
    /// its submitted start. A failure commits nothing.
    async fn attempt(
        &mut self,
        index: usize,
        values: &CaseValues,
        previous: Option<usize>,
    ) -> Result<(Box<SolveReport>, Arc<super::strategy::Trace>), MathRuntimeError> {
        self.scope.check().map_err(native::ProblemError::Provider)?;
        let block = &self.prepared.blocks[index];
        let bound_slot = &mut self.bound[index];
        let bound = match bound_slot.as_ref() {
            Some(view) => {
                let binding = view.compiled().rebind_binding_allocation_bound(values)?;
                if binding > self.service.policy.worker_bytes {
                    return Err(MathRuntimeError::Limit("local constructor worker capacity"));
                }
                let bound = view
                    .compiled()
                    .rebind_allocation_bound(values)?
                    .map(|bytes| bytes.max(binding));
                let construction = super::jobs::ConstructionEstimate::from_bound(
                    bound,
                    self.service.policy.worker_bytes,
                );
                self.local.construct_estimated(construction, |lease| {
                    self.service.rebind_local(
                        view,
                        values.clone(),
                        self.scope.cancellation(),
                        lease,
                    )
                })?
            }
            None => {
                let construction = super::jobs::ConstructionEstimate::from_bound(
                    block.view.binding_allocation_bound(values)?,
                    self.service.policy.worker_bytes,
                );
                self.local.construct_estimated(construction, |lease| {
                    self.service.bind_block_local(
                        block,
                        self.prepared.quantities.clone(),
                        self.prepared.preconditions.clone(),
                        values,
                        self.scope.cancellation(),
                        lease,
                    )
                })?
            }
        };
        let profile = SolverProfile {
            presolve: native::presolve::Policy::Off,
            intent: SolveIntent::Initialize,
            ..self.profile.solver.clone()
        };
        let native::routing::Route::Native(selected) = self.strategies[index] else {
            return Err(native::ProblemError::Contract(
                "conditional block needs a native route".into(),
            )
            .into());
        };
        let executable = bound
            .executable
            .get()
            .cloned()
            .unwrap_or_else(|| block.executable.clone());
        // Only genuinely missing compiler artifacts leave the owner through its
        // exclusive quiescent loan. The upgraded immutable programs survive stages.
        let (bound, executable) = self.local.preparation(self.service.conditional_program(
            bound,
            executable,
            &profile,
            selected,
            &self.scope,
            self.cancel,
        ))?;
        self.scope.check().map_err(native::ProblemError::Provider)?;
        *bound_slot = Some(bound.clone());
        let source_terms = executable
            .assembly
            .structure()
            .instances()
            .iter()
            .try_fold(0usize, |total, instance| {
                total.checked_add(instance.contributions.len())
            })
            .ok_or(MathRuntimeError::Limit("block assessment source terms"))?;
        let assessment_bytes = executable
            .assembly
            .numeric_worker_bytes()
            .checked_add(
                values
                    .scalars
                    .len()
                    .checked_mul(64)
                    .ok_or(MathRuntimeError::Limit("block assessment bindings"))?,
            )
            .and_then(|bytes| {
                bytes.checked_add(executable.assembly.input_ids().len().checked_mul(8)?)
            })
            .and_then(|bytes| {
                bytes.checked_add(block.boundary.members.rows.len().checked_mul(512)?)
            })
            .and_then(|bytes| {
                bytes.checked_add(block.boundary.members.columns.len().checked_mul(256)?)
            })
            .and_then(|bytes| bytes.checked_add(source_terms.checked_mul(128)?))
            .and_then(|bytes| bytes.checked_add(4096))
            .ok_or(MathRuntimeError::Limit(
                "block independent assessment extent",
            ))?;
        let assessor = self.local.construct(assessment_bytes, |storage| {
            Ok(BlockAssessor {
                service: self.service.clone(),
                case: executable.clone(),
                values: values.clone(),
                providers: self.providers.clone(),
                numerics: self.numerics.clone(),
                original: self.local.execution().clone(),
                storage,
            })
        })?;
        let step = self.service.prepare_conditional_bound(
            bound,
            executable,
            values.clone(),
            self.providers.clone(),
            profile,
            self.numerics.clone(),
            self.strategies[index],
            self.prepared.snapshot.clone(),
        )?;
        // The block starts from its staged values. Only a predecessor stage's committed
        // values make that start a seed; the authored initial point is not a warm start (F25).
        let step = match &self.admission {
            Some(admission) => step.within_admitted_task(self.scope.clone(), admission.clone())?,
            None => step.within_task(self.scope.clone())?,
        };
        let previous = previous
            .map(|attempt| step.primal_seed().map(|seed| Predecessor { attempt, seed }))
            .transpose()?;
        let (outcome, independent, strategy) = self
            .local
            .step(
                step,
                previous,
                self.attempts.len(),
                self.progress.clone(),
                self.owner.clone(),
                self.cancel,
                move |outcome, scope, budget| assessor.assess(outcome, scope, budget),
            )
            .await?;
        match outcome {
            super::solves::Outcome::Native(mut report) => {
                independent.apply(&mut report);
                Ok((report, strategy))
            }
            super::solves::Outcome::Rejected(error) => Err(MathRuntimeError::Strategy {
                cause: error,
                trace: strategy,
            }),
            super::solves::Outcome::Constant(_) => Err(MathRuntimeError::Strategy {
                cause: Arc::new(
                    native::ProblemError::Internal(
                        "conditional block evaluated without free coordinates".into(),
                    )
                    .into(),
                ),
                trace: strategy,
            }),
        }
    }
}
/// A block's independent original program. Only immutable code and its pre-admitted
/// storage cross the callback boundary; every candidate gets fresh evaluator/providers.
struct BlockAssessor {
    service: Arc<MathService>,
    case: Arc<ExecutableCase>,
    values: CaseValues,
    providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
    numerics: pse_math::SharedAllocation<pse_model::numerics::ResolvedNumericalPolicy>,
    original: Execution,
    storage: Arc<pse_columnar::AllocationLease>,
}
struct BlockObservation {
    observed: Option<(native::quality::Quality, native::quality::Observation)>,
    failure: Option<Arc<native::ProblemError>>,
    storage: Arc<pse_columnar::AllocationLease>,
}
impl BlockObservation {
    fn apply(self, report: &mut SolveReport) {
        if let Some(cause) = self.failure {
            report.record_validation_failure(native::ProblemError::Math(
                pse_math::MathError::Typed {
                    retained: cause.retained_bytes(),
                    cause: pse_model::diagnostic::DiagnosticCause::from_shared(cause),
                },
            ));
            report.retain_failure_owner(self.storage);
        } else if let Some((quality, observation)) = self.observed {
            if !quality.feasible() {
                report.qualification = Qualification::Unqualified;
                report.termination.assurance = Assurance::None;
            }
            report.quality = Some(quality);
            report.observation = Some(observation);
        }
    }
}
impl BlockAssessor {
    fn assess(
        &self,
        outcome: &super::solves::Outcome,
        scope: Option<&pse_kernels::ExecutionScope>,
        budget: &Arc<WorkerBudget>,
    ) -> super::strategy::Assessed<BlockObservation> {
        use super::strategy::{Assessed, OriginalConclusion};
        let mut retention = super::StepRetention {
            candidate: outcome.candidate_use(&self.numerics.policy),
            session: super::SessionDisposition::RetainCompatible,
        };
        let mut evaluations = 0;
        let evaluated = match outcome {
            super::solves::Outcome::Native(report) if report.candidate.is_some() => self
                .evaluate(report, scope, budget, &mut evaluations)
                .map(Some),
            _ => Ok(None),
        };
        let mut product = BlockObservation {
            observed: None,
            failure: None,
            storage: self.storage.clone(),
        };
        let original = match evaluated {
            Ok(Some((quality, observation))) => {
                if !quality.feasible() {
                    retention
                        .candidate
                        .refuse(pse_model::generated::enums::CandidateRefusal::Infeasible);
                }
                product.observed = Some((quality, observation));
                if retention.candidate.permits_use() {
                    OriginalConclusion::Satisfied
                } else {
                    OriginalConclusion::Refused {
                        cause: Arc::new(native::ProblemError::numerical(format!(
                            "independent original block assessment refused: {}",
                            retention.candidate.reason()
                        ))),
                    }
                }
            }
            Ok(None) => Assessed::native((), retention.clone(), outcome).original,
            Err(error) => {
                let cause = Arc::new(error.into_problem());
                retention
                    .candidate
                    .refuse(pse_model::generated::enums::CandidateRefusal::ValidationFailed);
                product.failure = Some(cause.clone());
                OriginalConclusion::Unavailable { cause }
            }
        };
        let mut owner = pse_ids::FramedHasher::new(pse_ids::Frame::NumericalWorkV1);
        owner
            .hash(&self.case.assembly.structure().key())
            .str("independent-original-block-assessment");
        Assessed {
            product,
            original,
            retention,
            work: vec![pse_model::strategy::WorkCharge {
                phase: pse_model::strategy::Phase::Assessment,
                scope: pse_model::strategy::Scope::Task,
                charging_owner: owner.finish_hash(),
                observed: pse_model::strategy::WorkObservation {
                    attempts: 0,
                    evaluations: Some(evaluations),
                    iterations: Some(0),
                    factorizations: Some(0),
                    proof_steps: Some(0),
                },
            }],
            accuracy: Vec::new(),
            accuracy_stop: None,
        }
    }
    fn evaluate(
        &self,
        report: &SolveReport,
        scope: Option<&pse_kernels::ExecutionScope>,
        budget: &Arc<WorkerBudget>,
        evaluations: &mut u64,
    ) -> Result<(native::quality::Quality, native::quality::Observation), MathRuntimeError> {
        self.original.check()?;
        let scope = scope.ok_or_else(|| {
            native::ProblemError::Contract("independent block scope missing".into())
        })?;
        if !Arc::ptr_eq(scope.cancellation(), &self.original.cancel)
            || self
                .original
                .scope()?
                .deadline()
                .is_some_and(|outer| scope.deadline().is_none_or(|deadline| deadline > outer))
        {
            return Err(native::ProblemError::Contract(
                "independent block differs from original task".into(),
            )
            .into());
        }
        let assembly = &self.case.assembly;
        let candidate = report.candidate.as_ref().ok_or_else(|| {
            native::ProblemError::Contract("independent block candidate missing".into())
        })?;
        if report.variables.as_slice() != assembly.columns()
            || candidate.primal.len() != report.variables.len()
        {
            return Err(native::ProblemError::Contract(
                "independent block original coordinate order".into(),
            )
            .into());
        }
        let structure = assembly.structure();
        let contract = native::assembled::contract(assembly);
        let bounds = structure
            .rows()
            .iter()
            .map(|row| (row.lower, row.upper))
            .collect::<Vec<_>>();
        if report.rows != contract.rows {
            return Err(native::ProblemError::Contract(
                "independent block original row order".into(),
            )
            .into());
        }
        let tolerance =
            Tolerances::from_policy(&self.numerics, assembly.columns(), &contract.rows)?;
        let inputs = assembly.bind_inputs(|id| self.values.scalars.get(&id).copied())?;
        let scratch = WorkerBudget::new(assembly.numeric_worker_bytes());
        let mut worker =
            self.service
                .worker(self.case.clone(), &self.providers, scope.clone(), &scratch)?;
        let mut execution = self.original.clone();
        execution.work_admission = budget
            .admission()
            .map(|owner| -> Arc<dyn WorkAdmission> { owner });
        let values = execution.counted(
            WorkEvidence {
                evaluations: Some(1),
                iterations: Some(0),
                factorizations: Some(0),
                proof_steps: Some(0),
            },
            || {
                *evaluations += 1;
                worker
                    .worker()
                    .constraints(inputs.trial(&candidate.primal)?)
                    .map_err(Into::into)
            },
        )?;
        self.original.check()?;
        if worker
            .worker()
            .applicability_observations()
            .iter()
            .any(|observation| observation.required && !observation.admitted)
        {
            return Err(native::ProblemError::numerical(
                "independent original block applicability refused",
            )
            .into());
        }
        let quality =
            native::quality::observed(&contract, &bounds, &candidate.primal, &values, &tolerance)?;
        let mut observation = native::quality::Observation::from_values(None, values, bounds)?;
        observation.sources = worker.worker().constraint_sources()?;
        self.original.check()?;
        Ok((quality, observation))
    }
}

impl MathService {
    /// Bind on the actual serial owner under its pre-admitted construction lease.
    fn bind_block_local(
        &self,
        block: &ConditionalBlock,
        quantities: pse_math::SharedAllocation<pse_quantity::QuantityRegistry>,
        preconditions: pse_math::SharedAllocation<pse_quantity::PhysicalPreconditions>,
        values: &CaseValues,
        flag: &Arc<std::sync::atomic::AtomicBool>,
        lease: Arc<pse_columnar::AllocationLease>,
    ) -> Result<Preparation, MathRuntimeError> {
        let bound = block.view.bind(quantities, preconditions, values, flag)?;
        let bytes = block_binding_bytes(&bound);
        self.own_binding(
            block.owner.clone(),
            bound,
            super::modeling::retained_construction(lease, bytes)?,
            Arc::new(std::sync::OnceLock::from(block.executable.clone())),
        )
    }
}

fn block_binding_bytes(bound: &pse_compiler::workspace::PreparedCase) -> usize {
    // The same wrapper allowance as a value rebind, plus all newly allocated first-bind
    // products. Empty occurrence/derivation/derived containers are new here too.
    2 * size_of::<pse_compiler::workspace::PreparedCase>()
        + 1024
        + bound.binding_bytes()
        + bound.presolve.bytes()
        + 32
        + bound.provenance_bytes()
        + bound.derivation.retained_bytes()
        + size_of::<pse_compiler::workspace::Derivation>()
        + 32
        + bound.derived.retained_bytes()
        + size_of::<pse_compiler::workspace::Derived>()
        + 32
        + bound
            .coefficients
            .as_ref()
            .map_or(0, |c| c.retained_bytes() + 32)
}

/// Commit once, retaining the actual refusal in the production diagnostic surface.
pub(crate) fn commit_block(
    values: &mut CaseValues,
    b: &pse_structural::initialization::Block,
    report: Option<&SolveReport>,
    policy: &pse_model::numerics::NumericalPolicy,
) -> Result<(), Arc<native::ProblemError>> {
    let primal = block_candidate(b, report, policy)?;
    values.scalars.extend(
        b.members
            .columns
            .iter()
            .copied()
            .zip(primal.iter().copied()),
    );
    Ok(())
}
fn block_candidate<'a>(
    b: &pse_structural::initialization::Block,
    report: Option<&'a SolveReport>,
    policy: &pse_model::numerics::NumericalPolicy,
) -> Result<&'a [f64], Arc<native::ProblemError>> {
    let Some(r) = report else {
        return Err(Arc::new(native::ProblemError::Contract(
            "conditional block native report unavailable".into(),
        )));
    };
    // Preserve the shared witness and its classification before deriving presentation.
    if let Some(cause) = r.shared_effective_failure() {
        return Err(cause);
    }
    // The native candidate-use decision is the only acceptance rule; commit adds the
    // block's own coordinate and finiteness checks.
    let decision = crate::workflow::numerics::native_use(r, policy);
    let ordered = r.variables == b.members.columns;
    let candidate_len = r.candidate.as_ref().map(|c| c.primal.len());
    let nonfinite = r
        .candidate
        .as_ref()
        .and_then(|c| c.primal.iter().position(|v| !v.is_finite()));
    let malformed = !ordered
        || (r.candidate.is_none()
            && matches!(
                r.termination.category,
                Termination::Success | Termination::Acceptable | Termination::FeasibleOnly
            ))
        || r.candidate.as_ref().is_some_and(|c| {
            c.primal.len() != b.members.columns.len() || nonfinite.is_some() || r.quality.is_none()
        });
    if !decision.permits_use()
        || !ordered
        || candidate_len != Some(b.members.columns.len())
        || nonfinite.is_some()
    {
        let mut reason = format!(
            "conditional block candidate refused: usability={:?}; reasons={}; termination={:?} ({}: {}); qualification={:?}; candidate_present={}; candidate_length={candidate_len:?}; expected_length={}; variable_order_matches={ordered}; nonfinite_coordinate={nonfinite:?}; original_quality.normalized_max={:?}",
            decision.usability,
            decision.reason(),
            r.termination.category,
            r.termination.code,
            r.termination.name,
            r.qualification,
            r.candidate.is_some(),
            b.members.columns.len(),
            r.quality.as_ref().map(|quality| quality.normalized_max),
        );
        if !ordered {
            let mismatch = r
                .variables
                .iter()
                .zip(&b.members.columns)
                .position(|(actual, expected)| actual != expected)
                .unwrap_or(r.variables.len().min(b.members.columns.len()));
            reason.push_str(&format!(
                "; variable_order_mismatch_at={mismatch}; actual={:?}; expected={:?}",
                r.variables.get(mismatch),
                b.members.columns.get(mismatch),
            ));
        }
        if let Some(quality) = &r.quality {
            // Bounded original-space witnesses; never concatenate every model row.
            for (kind, violation) in quality
                .rows
                .iter()
                .map(|v| ("row", v))
                .chain(quality.bounds.iter().map(|v| ("bound", v)))
                .chain(quality.integrality.iter().map(|v| ("integrality", v)))
                .filter(|(_, v)| v.physical > v.tolerance)
                .take(12)
            {
                reason.push_str(&format!(
                    "; {kind}={} physical={} tolerance={}",
                    violation.id, violation.physical, violation.tolerance,
                ));
            }
        }
        return Err(Arc::new(if malformed {
            native::ProblemError::Contract(reason)
        } else {
            // A refused native iterate is a failed numerical trajectory, not a
            // missing execution capability. Optional composition may try Direct;
            // none of these coordinates have been committed or accepted.
            native::ProblemError::numerical(reason)
        }));
    }
    let c = r.candidate.as_ref().ok_or_else(|| {
        Arc::new(native::ProblemError::Contract(
            "conditional block candidate unavailable".into(),
        ))
    })?;
    Ok(&c.primal)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    fn id(v: u8) -> SemanticId {
        SemanticId::from_bytes([v; 16])
    }
    fn declared_submission(
        service: &Arc<MathService>,
        duration: std::time::Duration,
        called: Arc<AtomicBool>,
    ) -> Result<SolveHandle<DeclaredRootReport>, MathRuntimeError> {
        declared_submission_with_residency(service, duration, called, 0)
    }
    fn declared_submission_with_residency(
        service: &Arc<MathService>,
        duration: std::time::Duration,
        called: Arc<AtomicBool>,
        resident_foreign_bytes: usize,
    ) -> Result<SolveHandle<DeclaredRootReport>, MathRuntimeError> {
        declared_submission_observed(service, duration, called, resident_foreign_bytes, || {})
    }
    fn declared_submission_observed(
        service: &Arc<MathService>,
        duration: std::time::Duration,
        called: Arc<AtomicBool>,
        resident_foreign_bytes: usize,
        entered: impl FnOnce() + Send + 'static,
    ) -> Result<SolveHandle<DeclaredRootReport>, MathRuntimeError> {
        let contract = native::OracleContract {
            identity: pse_ids::ContentHash::from_bytes([72; 32]),
            variables: vec![native::Variable {
                id: id(71),
                lower: f64::NEG_INFINITY,
                upper: f64::INFINITY,
            }],
            rows: vec![id(72)],
            derivatives: pse_kernels::DerivativeOrder::Value,
            smoothness: pse_kernels::DerivativeOrder::Value,
        };
        let policy = pse_model::numerics::NumericalPolicy::default();
        let tolerances = Tolerances {
            variables: vec![1e-8],
            rows: vec![1e-8],
            integrality: 1e-8,
        };
        let normalization = pse_math::normalization::Normalization {
            variables: vec![1.],
            rows: vec![1.],
            objective: 1.,
        };
        let accuracy = ResolvedAccuracy::resolve(&policy, &tolerances, &normalization).unwrap();
        let settings = kinsol::Settings::from_policy(
            kinsol::Method {
                strategy: kinsol::Strategy::FixedPoint,
                ..Default::default()
            },
            &tolerances,
            &normalization,
            accuracy.feasibility,
        );
        let controls = Controls {
            time_limit: duration,
            ..Default::default()
        };
        let profile = SolverProfile {
            selection: SolverSelection::Explicit(Backend::Kinsol),
            backend: execution::BackendSettings::Kinsol(settings.method),
            controls: controls.clone(),
            numerics: policy.clone(),
            intent: SolveIntent::Root,
            presolve: Default::default(),
            convexity: Default::default(),
            sensitivity: None,
            composition: Default::default(),
            reconstruction: None,
        };
        service.solve_declared_root(
            contract,
            vec![0.],
            settings,
            controls,
            accuracy,
            tolerances,
            policy,
            super::super::solves::profile_key(&profile).unwrap().as_id(),
            None,
            Some(4096),
            vec![Backend::Kinsol],
            resident_foreign_bytes,
            move |_, _| {
                called.store(true, Ordering::Release);
                entered();
                Err(native::ProblemError::Internal(
                    "declared root fixture factory completed".into(),
                ))
            },
        )
    }
    #[tokio::test(flavor = "current_thread")]
    async fn declared_root_reserves_all_retained_unit_residency_before_cpu_entry() {
        let (service, _cache) = super::super::tests::service_with_policy(
            256 << 20,
            super::super::MathPolicy {
                foreign_bytes: 1 << 20,
                ..Default::default()
            },
        );
        let baseline = service.pool.reserved();
        let cpu = service.cpu.clone().acquire_many_owned(2).await.unwrap();
        let called = Arc::new(AtomicBool::new(false));
        let mut ordinary_extent = 0;
        for residency in [0, 8 << 20] {
            let released = service.released.notified();
            tokio::pin!(released);
            released.as_mut().enable();
            let handle = declared_submission_with_residency(
                &service,
                std::time::Duration::from_secs(5),
                called.clone(),
                residency,
            )
            .unwrap();
            let stop = handle.cancellation();
            let mut pending = Box::pin(handle.finish());
            // Population admission retains its result owner while active admission
            // releases tentative constructor/native memory before waiting for CPU.
            tokio::time::timeout(std::time::Duration::from_secs(1), released)
                .await
                .expect("blocked declared root must release tentative active memory");
            assert!(
                futures_util::poll!(pending.as_mut()).is_pending(),
                "blocked declared root must remain pending after its real admission attempt"
            );
            let extent = service.pool.reserved() - baseline;
            if residency == 0 {
                ordinary_extent = extent;
            } else {
                assert_eq!(extent, ordinary_extent);
            }
            assert_eq!(service.cpu.available_permits(), 0);
            assert!(!called.load(Ordering::Acquire));
            stop.cancel();
            assert!(matches!(
                tokio::time::timeout(std::time::Duration::from_secs(1), pending)
                    .await
                    .expect("queued declared root cancellation must drain"),
                Err(MathRuntimeError::Cancelled)
            ));
            assert_eq!(service.pool.reserved(), baseline);
        }
        assert!(
            declared_submission_with_residency(
                &service,
                std::time::Duration::from_secs(5),
                called.clone(),
                usize::MAX
            )
            .is_err()
        );
        assert!(!called.load(Ordering::Acquire));
        assert_eq!(service.jobs.available_permits(), service.policy.jobs);
        assert_eq!(service.pool.reserved(), baseline);
        drop(cpu);
        // Once active admission succeeds, the factory observes the complete retained
        // native residency before it may construct any nested unit. Hold that exact
        // entry with a bounded fixture barrier so the measurement cannot race teardown.
        let mut ordinary_extent = 0;
        for residency in [0, 8 << 20] {
            let (entered, observation) = tokio::sync::oneshot::channel();
            let (release, released) = std::sync::mpsc::sync_channel(1);
            let pool = service.pool.clone();
            let handle = declared_submission_observed(
                &service,
                std::time::Duration::from_secs(15),
                Arc::new(AtomicBool::new(false)),
                residency,
                move || {
                    entered.send(pool.reserved()).unwrap();
                    released
                        .recv_timeout(std::time::Duration::from_secs(10))
                        .unwrap();
                },
            )
            .unwrap();
            let extent = tokio::time::timeout(std::time::Duration::from_secs(10), observation)
                .await
                .expect("declared factory must enter within its unchanged clock")
                .unwrap()
                - baseline;
            if residency == 0 {
                ordinary_extent = extent;
            } else {
                assert_eq!(extent, ordinary_extent + residency);
            }
            release.send(()).unwrap();
            assert!(handle.finish().await.is_err());
            assert_eq!(service.pool.reserved(), baseline);
        }
    }
    #[tokio::test]
    async fn declared_root_deadline_and_cancellation_cover_cpu_wait_without_dispatch() {
        let (service, _cache) = super::super::tests::service_with_policy(
            256 << 20,
            super::super::MathPolicy {
                worker_bytes: 16usize << 30,
                workspace_bytes: 16 << 20,
                foreign_bytes: 1 << 20,
                ..Default::default()
            },
        );
        let baseline = service.pool.reserved();
        let permit = service.cpu.clone().acquire_many_owned(2).await.unwrap();
        let called = Arc::new(AtomicBool::new(false));
        let handle = declared_submission(
            &service,
            std::time::Duration::from_millis(25),
            called.clone(),
        )
        .unwrap();
        let cancellation = handle.cancellation();
        let result = handle.finish().await;
        assert!(matches!(
            result,
            Err(MathRuntimeError::Solve(native::ProblemError::Limit {
                kind: native::LimitKind::Time,
                ..
            }))
        ));
        assert!(!cancellation.flag().load(Ordering::Acquire));
        assert!(!called.load(Ordering::Acquire));
        assert_eq!(service.cpu.available_permits(), 0);
        let handle =
            declared_submission(&service, std::time::Duration::from_secs(5), called.clone())
                .unwrap();
        handle.cancel();
        assert!(matches!(
            handle.finish().await,
            Err(MathRuntimeError::Cancelled)
        ));
        assert!(!called.load(Ordering::Acquire));
        drop(permit);
        assert_eq!(service.pool.reserved(), baseline);
        assert_eq!(service.cpu.available_permits(), 2);
    }
    #[tokio::test(flavor = "current_thread")]
    async fn declared_root_burst_owns_ticket_before_spawning_factory_task() {
        let (service, _cache) = super::super::tests::service_with_policy(
            256 << 20,
            super::super::MathPolicy {
                jobs: 32,
                foreign_bytes: 1 << 20,
                ..Default::default()
            },
        );
        let baseline = service.pool.reserved();
        let cpu = service.cpu.clone().acquire_many_owned(2).await.unwrap();
        let called = Arc::new(AtomicBool::new(false));
        let mut handles = Vec::new();
        for _ in 0..32 {
            handles.push(
                declared_submission(&service, std::time::Duration::from_secs(5), called.clone())
                    .unwrap(),
            );
        }
        assert_eq!(service.jobs.available_permits(), 0);
        assert!(matches!(
            declared_submission(&service, std::time::Duration::from_secs(5), called.clone()),
            Err(MathRuntimeError::Limit("native jobs"))
        ));
        assert!(!called.load(Ordering::Acquire));
        for handle in &handles {
            handle.cancel();
        }
        for handle in handles {
            assert!(matches!(
                handle.finish().await,
                Err(MathRuntimeError::Cancelled)
            ));
        }
        assert!(!called.load(Ordering::Acquire));
        assert_eq!(service.jobs.available_permits(), 32);
        assert_eq!(service.pool.reserved(), baseline);
        drop(cpu);
    }
    #[cfg(feature = "solver-kinsol")]
    #[tokio::test]
    async fn declared_root_known_factory_enters_small_pool_under_generous_worker_capacity() {
        let (service, _cache) = super::super::tests::service_with_policy(
            256 << 20,
            super::super::MathPolicy {
                worker_bytes: 16usize << 30,
                foreign_bytes: 1 << 20,
                ..Default::default()
            },
        );
        let baseline = service.pool.reserved();
        let called = Arc::new(AtomicBool::new(false));
        let result =
            declared_submission(&service, std::time::Duration::from_secs(1), called.clone())
                .unwrap()
                .finish()
                .await;
        // The supplied factory's original typed refusal follows dispatch. The
        // unchanged worker ceiling must not refuse this tiny factory at entry.
        assert!(called.load(Ordering::Acquire));
        assert!(result.is_err());
        // The returned refusal owns its retained strategy trace until released.
        drop(result);
        assert_eq!(service.pool.reserved(), baseline);
        assert_eq!(service.jobs.available_permits(), service.policy.jobs);
    }
    #[test]
    fn conditional_failure_preserves_shared_terminal_causes_and_actual_native_status() {
        let controls = Controls::default();
        let execution = Execution::new(Arc::default(), &controls);
        let contract = native::OracleContract {
            identity: pse_ids::ContentHash::from_bytes([71; 32]),
            variables: vec![native::Variable {
                id: id(71),
                lower: f64::NEG_INFINITY,
                upper: f64::INFINITY,
            }],
            rows: vec![id(72)],
            derivatives: pse_kernels::DerivativeOrder::First,
            smoothness: pse_kernels::DerivativeOrder::First,
        };
        let terminal = NativeTermination {
            code: -8,
            name: "KIN_MAXITER_REACHED".into(),
            message: None,
            category: Termination::IterationLimit,
            assurance: Assurance::None,
        };
        let mut report = SolveReport::new(Backend::Kinsol, &contract, terminal, &execution);
        let failure = conditional_failure(&report);
        assert!(matches!(
            failure,
            native::ProblemError::Limit {
                kind: native::LimitKind::Work,
                ..
            }
        ));
        for error in [
            native::ProblemError::memory("original allocation"),
            native::ProblemError::Contract("original unit contract".into()),
            native::ProblemError::Provider(pse_kernels::ProviderError::Deadline),
        ] {
            report.record_validation_failure(error);
            let original = report.shared_validation_failure().unwrap();
            let failure = conditional_failure(&report);
            let native::ProblemError::Math(pse_math::MathError::Typed { cause, .. }) = &failure
            else {
                panic!("actual failure was erased")
            };
            assert!(std::ptr::eq(
                cause
                    .as_error()
                    .downcast_ref::<native::ProblemError>()
                    .unwrap(),
                original.as_ref()
            ));
            assert_eq!(
                native::callback::classify(&failure),
                native::callback::classify(original.as_ref())
            );
        }
        for callback_terminal in [false, true] {
            let mut dual = SolveReport::new(
                Backend::Kinsol,
                &contract,
                NativeTermination {
                    code: -13,
                    name: "callback fixture".into(),
                    message: None,
                    category: Termination::Evaluation,
                    assurance: Assurance::None,
                },
                &execution,
            );
            let mut callbacks = native::callback::CallbackState::new(execution.clone());
            let callback = if callback_terminal {
                native::ProblemError::Contract("terminal callback contract".into())
            } else {
                native::ProblemError::Math(pse_math::MathError::Domain {
                    source_id: id(72),
                    requirement: "callback trial domain",
                })
            };
            assert!(
                callbacks
                    .evaluate::<()>("residual", || Err(callback))
                    .is_none()
            );
            callbacks.finish(&mut dual);
            let callback = dual.shared_callback_failure().unwrap();
            dual.record_validation_failure(native::ProblemError::memory("original validation"));
            let expected = if callback_terminal {
                callback
            } else {
                dual.shared_validation_failure().unwrap()
            };
            let failure = conditional_failure(&dual);
            let native::ProblemError::Math(pse_math::MathError::Typed { cause, .. }) = &failure
            else {
                panic!("selected source was erased")
            };
            assert!(std::ptr::eq(
                cause
                    .as_error()
                    .downcast_ref::<native::ProblemError>()
                    .unwrap(),
                expected.as_ref(),
            ));
        }
    }
    #[tokio::test]
    async fn first_block_binding_retains_shared_parents_until_last_alias() {
        pse_math::initialize().unwrap();
        let (service, _cache) = super::super::tests::service_with_policy(
            256 << 20,
            super::super::MathPolicy {
                worker_bytes: 16usize << 30,
                workspace_bytes: 16 << 20,
                foreign_bytes: 1 << 20,
                ..Default::default()
            },
        );
        let baseline = service.pool.reserved();
        let registry = Arc::new(
            pse_quantity::QuantityRegistryBuilder::new()
                .build()
                .unwrap(),
        );
        let registry_bytes = registry.allocation_extent();
        let registry_lease = service
            .reserve("math:test-registry", registry_bytes)
            .unwrap();
        let quantities =
            pse_math::SharedAllocation::from(registry.clone()).with_owner(registry_lease);
        let preconditions = Arc::new(pse_quantity::PhysicalPreconditions::new(vec![]).unwrap());
        let precondition_lease = service
            .reserve("math:test-prerequisites", preconditions.allocation_extent())
            .unwrap();
        let preconditions =
            pse_math::SharedAllocation::from(preconditions).with_owner(precondition_lease);
        let cancel = Arc::new(AtomicBool::new(false));
        let structure = Arc::new(
            pse_math::binding::CaseStructure::new(
                vec![],
                vec![],
                vec![],
                vec![],
                None,
                Default::default(),
            )
            .unwrap(),
        );
        let plan = Arc::new(
            pse_math::assembly::CasePlan::prepare(
                structure,
                BTreeMap::new(),
                &registry,
                pse_kernels::DerivativeOrder::First,
                Default::default(),
                &cancel,
            )
            .unwrap(),
        );
        let witness = Arc::new(
            pse_structural::incidence::CaseIncidence::new(
                pse_structural::projection::Scope::Whole(id(7)),
                vec![],
                vec![],
                vec![],
                BTreeSet::new(),
                pse_structural::projection::GraphLimits {
                    nodes: 10,
                    edges: 10,
                },
            )
            .unwrap()
            .analyze(&cancel)
            .unwrap(),
        );
        let view = pse_compiler::workspace::PreparedBlock {
            class_proof_work: Profile::default().class_proof_work,
            boundary: pse_structural::initialization::Block {
                id: pse_structural::incidence::BlockId(pse_ids::ContentHash::from_bytes([8; 32])),
                members: pse_structural::incidence::Part {
                    rows: vec![],
                    columns: vec![],
                },
                inputs: vec![],
            },
            plan: plan.clone(),
            structure: witness,
            artifacts: Arc::new(vec![]),
        };
        let lease = service
            .reserve("math:test-initialization", plan.retained_bytes())
            .unwrap();
        let prepared = service
            .own_initialization(
                Arc::new(vec![view]),
                lease,
                quantities,
                preconditions,
                vec![],
                vec![],
            )
            .await
            .unwrap();
        drop(plan);
        drop(registry);
        let parents = service.pool.reserved();
        let session = service.open_session().unwrap();
        let resident = service.pool.reserved() - parents;
        let values = CaseValues {
            scalars: BTreeMap::new(),
        };
        let demand = prepared.blocks[0]
            .view
            .binding_allocation_bound(&values)
            .unwrap()
            .unwrap();
        // The actual local constructor yields active resources while the original
        // operation clock expires under pressure; all shared parents survive.
        let pressure = service
            .reserve("math:test-pressure", (256 << 20) - service.pool.reserved())
            .unwrap();
        let worker_service = service.clone();
        let worker_prepared = prepared.clone();
        let worker_values = values.clone();
        let driver = crate::CancelSource::new();
        let expired = session
            .sequence(
                pse_kernels::ExecutionScope::new(
                    Arc::default(),
                    Some(std::time::Instant::now() + std::time::Duration::from_millis(20)),
                ),
                Controls::default(),
                &driver,
                move |local| {
                    // Move the complete fixture owner, rather than leaving its
                    // uncaptured _owner field alive until this test returns.
                    let prepared = worker_prepared;
                    local.construct(demand, |lease| {
                        worker_service.bind_block_local(
                            &prepared.blocks[0],
                            prepared.quantities.clone(),
                            prepared.preconditions.clone(),
                            &worker_values,
                            &local.execution().cancel,
                            lease,
                        )
                    })
                },
            )
            .await;
        assert!(matches!(
            expired,
            Err(MathRuntimeError::Solve(native::ProblemError::Limit {
                kind: native::LimitKind::Time,
                ..
            }))
        ));
        drop(pressure);
        assert_eq!(service.pool.reserved(), parents + resident);
        let worker_service = service.clone();
        let worker_prepared = prepared.clone();
        let bound = session
            .sequence(
                pse_kernels::ExecutionScope::new(
                    Arc::default(),
                    Some(std::time::Instant::now() + std::time::Duration::from_secs(10)),
                ),
                Controls::default(),
                &driver,
                move |local| {
                    let prepared = worker_prepared;
                    local.construct(demand, |lease| {
                        worker_service.bind_block_local(
                            &prepared.blocks[0],
                            prepared.quantities.clone(),
                            prepared.preconditions.clone(),
                            &values,
                            &local.execution().cancel,
                            lease,
                        )
                    })
                },
            )
            .await
            .unwrap();
        session.close().await;
        assert_eq!(
            service.pool.reserved() - parents,
            block_binding_bytes(bound.compiled())
        );
        assert!(Arc::ptr_eq(
            &bound.compiled().plan,
            &prepared.blocks[0].view.plan
        ));
        assert!(Arc::ptr_eq(
            bound.executable.get().unwrap(),
            &prepared.blocks[0].executable
        ));
        let binding_alias = bound.compiled().coefficient_values.clone();
        let registry_alias = bound.compiled().quantities.clone();
        let prerequisites_alias = bound.compiled().preconditions.clone();
        drop(prepared);
        drop(bound);
        assert!(service.pool.reserved() > baseline + registry_bytes);
        drop(binding_alias);
        assert!(service.pool.reserved() > baseline);
        drop(registry_alias);
        assert!(service.pool.reserved() > baseline);
        drop(prerequisites_alias);
        assert_eq!(service.pool.reserved(), baseline);
    }
    #[tokio::test]
    async fn initialization_latest_block_bindings_plateau_when_only_coefficients_change() {
        use datafusion::execution::cache::Cache;
        let (service, _cache) = super::super::tests::service_with_policy(
            256 << 20,
            super::super::MathPolicy {
                foreign_bytes: 1 << 20,
                worker_bytes: 16usize << 30,
                ..Default::default()
            },
        );
        let (source, mut values, parameter) = super::super::tests::component_fixture_source(
            &service,
            "package p {def Root {param p:Scalar=1;var x:Scalar;eq a:x==1;annotation start x(1);}}",
        );
        let compiled = source.compiled();
        let block = pse_compiler::workspace::PreparedBlock {
            class_proof_work: compiled.class_proof_work,
            boundary: pse_structural::initialization::Block {
                id: pse_structural::incidence::BlockId(pse_ids::ContentHash::from_bytes([10; 32])),
                members: pse_structural::incidence::Part {
                    rows: compiled
                        .plan
                        .structure()
                        .rows()
                        .iter()
                        .map(|row| row.id)
                        .collect(),
                    columns: compiled.plan.columns().to_vec(),
                },
                inputs: Vec::new(),
            },
            plan: compiled.plan.clone(),
            structure: Arc::new(compiled.structure.as_ref().clone()),
            artifacts: Arc::new(compiled.artifacts.as_ref().clone()),
        };
        let products = Arc::new(vec![block]);
        let bytes =
            pse_compiler::workspace::PreparedBlock::retained_group_bytes(&products).unwrap();
        let lease = service
            .reserve("test:initialization-components", bytes)
            .unwrap();
        let prepared = service
            .own_initialization(
                products,
                lease,
                compiled.quantities.clone(),
                compiled.preconditions.clone(),
                Vec::new(),
                Vec::new(),
            )
            .await
            .unwrap();
        drop(source);
        let driver = crate::CancelSource::new();
        let demand = prepared.blocks[0]
            .view
            .binding_allocation_bound(&values)
            .unwrap()
            .unwrap();
        let lease = service
            .reserve("test:initialization-first-binding", demand)
            .unwrap();
        let mut bound = service
            .bind_block_local(
                &prepared.blocks[0],
                prepared.quantities.clone(),
                prepared.preconditions.clone(),
                &values,
                &Arc::new(AtomicBool::new(false)),
                lease,
            )
            .unwrap();
        let mut plateau = None;
        // This is the production first-bind/rebind sequence and latest-view overwrite
        // used by Blocks::attempt; reports and historical aliases are not retained here.
        for index in 0..24 {
            let previous = Arc::downgrade(&bound._owner);
            let presolve = bound.compiled().presolve.clone();
            let derived = bound.compiled().derived.clone();
            values.scalars.insert(parameter, 2.0 + index as f64);
            bound = service
                .rebind(&bound, values.clone(), &driver)
                .await
                .unwrap();
            assert!(previous.upgrade().is_none());
            assert!(pse_math::SharedAllocation::ptr_eq(
                &presolve,
                &bound.compiled().presolve
            ));
            assert!(pse_math::SharedAllocation::ptr_eq(
                &derived,
                &bound.compiled().derived
            ));
            assert!(
                bound
                    .compiled()
                    .coefficient_values
                    .iter()
                    .any(|(id, bits)| *id == parameter && *bits == (2.0 + index as f64).to_bits())
            );
            if index == 0 {
                plateau = Some(service.pool.reserved());
            }
            assert_eq!(service.pool.reserved(), plateau.unwrap());
        }
        drop(bound);
        drop(prepared);
        // A cached program also retains its descriptor and stable plan anchors.
        assert!(service.entries.len() > 0);
        service.clear_program_cache();
        assert_eq!(service.pool.reserved(), 0);
    }
    #[cfg(feature = "solver-kinsol")]
    #[tokio::test]
    async fn actual_block_attempts_plateau_and_escaped_coefficients_keep_their_charge() {
        let (service, _cache) = super::super::tests::service_with_policy(
            256 << 20,
            super::super::MathPolicy {
                foreign_bytes: 1 << 20,
                worker_bytes: 16usize << 30,
                ..Default::default()
            },
        );
        let (source, mut values, parameter) = super::super::tests::component_fixture_source(
            &service,
            "package p {def Root {param p:Scalar=1;var x:Scalar;eq a:x==1;annotation start x(1);}}",
        );
        let compiled = source.compiled();
        assert_eq!(compiled.plan.columns().len(), 1);
        // The pure compiler fixture returns fixed inputs; Blocks also requires
        // its unknown's authored initial value before profile admission.
        values.scalars.insert(compiled.plan.columns()[0], 1.);
        let targets = compiled
            .plan
            .structure()
            .numerical_targets(&compiled.quantities)
            .unwrap();
        let block = pse_compiler::workspace::PreparedBlock {
            class_proof_work: compiled.class_proof_work,
            boundary: pse_structural::initialization::Block {
                id: pse_structural::incidence::BlockId(pse_ids::ContentHash::from_bytes([11; 32])),
                members: pse_structural::incidence::Part {
                    rows: compiled
                        .plan
                        .structure()
                        .rows()
                        .iter()
                        .map(|row| row.id)
                        .collect(),
                    columns: compiled.plan.columns().to_vec(),
                },
                inputs: Vec::new(),
            },
            plan: compiled.plan.clone(),
            structure: Arc::new(compiled.structure.as_ref().clone()),
            artifacts: Arc::new(compiled.artifacts.as_ref().clone()),
        };
        let products = Arc::new(vec![block]);
        let bytes =
            pse_compiler::workspace::PreparedBlock::retained_group_bytes(&products).unwrap();
        let lease = service
            .reserve("test:actual-block-components", bytes)
            .unwrap();
        let prepared = service
            .own_initialization(
                products,
                lease,
                compiled.quantities.clone(),
                compiled.preconditions.clone(),
                targets,
                Vec::new(),
            )
            .await
            .unwrap();
        drop(source);
        let profile = InitializationProfile {
            solver: SolverProfile {
                selection: SolverSelection::Explicit(Backend::Kinsol),
                backend: execution::BackendSettings::Kinsol(kinsol::Method::default()),
                controls: Controls {
                    history: 0,
                    reuse: ReusePolicy::Fresh,
                    ..Default::default()
                },
                intent: SolveIntent::Initialize,
                presolve: Default::default(),
                numerics: Default::default(),
                convexity: Default::default(),
                sensitivity: None,
                composition: Default::default(),
                reconstruction: None,
            },
            stages: vec![BTreeMap::new()],
        };
        let (strategies, numerics) = prepared.validate_profile(&values, &profile).unwrap();
        let providers = BTreeMap::new();
        let progress = Arc::new(Progress::new(0));
        let owner = service
            .reserve(
                "test:actual-block-reports",
                profile.solver.controls.report_allowance().unwrap(),
            )
            .unwrap();
        let cancel = crate::CancelSource::new();
        let control = FlightCancellation::default();
        let deadline = std::time::Instant::now()
            .checked_add(profile.solver.controls.time_limit)
            .unwrap();
        let scope = pse_kernels::ExecutionScope::new(control.flag(), Some(deadline));
        let session = service.open_session().unwrap();
        let mut other_residents = Vec::new();
        for _ in 1..service.policy.jobs {
            other_residents.push(service.open_session().unwrap());
        }
        assert_eq!(service.jobs.available_permits(), 0);
        let worker_service = service.clone();
        let worker_prepared = prepared.clone();
        let worker_owner = owner.clone();
        let worker_progress = progress.clone();
        let operation_cancel = cancel.clone();
        let operation_scope = scope.clone();
        let operation_controls = profile.solver.controls.clone();
        let sequence = session.sequence(operation_scope, operation_controls, &cancel, move |local| {
            let service = worker_service;
            let prepared = worker_prepared;
            let owner = worker_owner;
            let progress = worker_progress;
        let mut run = Blocks {
            service: &service,
            local,
            prepared: &prepared,
            providers: &providers,
            profile: &profile,
            strategies: &strategies,
            numerics: &numerics,
            progress: &progress,
            owner: &owner,
            cancel: &operation_cancel,
            scope,
            admission: None,
            run_id: pse_operations::mint_id(),
            bound: vec![None],
            attempts: Vec::new(),
        };
        let (report, trace) = local.run_ready(run.attempt(0, &values, None)).unwrap();
        // A report can retain a successful same-point solver observation while its
        // candidate no longer satisfies the authored block. Only fresh independent
        // evaluation may authorize the commit or finish the automatic strategy.
        {
            use super::super::strategy::{self, OriginalConclusion};
            let executable = run.bound[0].as_ref().unwrap().executable.get().unwrap().clone();
            let bytes = executable.assembly.numeric_worker_bytes() + 16384;
            let assessor = local.construct(bytes, |storage| Ok(BlockAssessor {
                service: service.clone(), case: executable, values: values.clone(),
                providers: providers.clone(), numerics: numerics.clone(),
                original: local.execution().clone(), storage,
            })).unwrap();
            let mut forged = report.clone();
            forged.candidate.as_mut().unwrap().primal[0] = 2.;
            assert!(crate::workflow::numerics::native_use(&forged, &numerics.policy).permits_use(),
                "revealing control must start with stale native permission");
            let outcome = super::super::solves::Outcome::Native(forged);
            let budget = WorkerBudget::new(bytes);
            let assessed = assessor.assess(&outcome, Some(&run.scope), &budget);
            assert!(matches!(assessed.original, OriginalConclusion::Refused { .. }));
            assert!(!assessed.retention.candidate.permits_use());
            assert_eq!(assessed.work[0].observed.evaluations, Some(1));
            assert!(!assessed.product.observed.as_ref().unwrap().0.feasible());
            let last = strategy::AutoObservation { awaiting_assessment: false,
                native: strategy::observe(&outcome), original: Some(assessed.original),
                permission: Some(assessed.retention.candidate.usability) };
            let recovery = strategy::AutoCandidate { observed: None,
                identity: pse_ids::ContentHash::from_bytes([91; 32]),
                kind: pse_model::strategy::MechanismKind::Direct,
                start: pse_model::strategy::StartOrigin::Specification,
                replacement: false, support: Default::default(), reservation: None, prepared: true,
            };
            // The independently refused observation consumed the first execution;
            // admit exactly one recovery, including that original execution.
            let request = pse_model::strategy::CompositionRequest {
                limits: Some(pse_model::strategy::WorkLimits {
                    attempts: 2, evaluations: None, iterations: None,
                    factorizations: None, proof_steps: None,
                }),
                ..Default::default()
            };
            let starts = pse_model::strategy::StartRules {
                policy: StartPolicy::NoPriorStart, recovery: Vec::new(),
            };
            assert!(matches!(strategy::next_automatic(&Default::default(), &starts,
                std::slice::from_ref(&recovery), &BTreeSet::new(), Some(&last),
                pse_model::strategy::WorkObservation {
                    attempts: 1, evaluations: Some(0), iterations: Some(0),
                    factorizations: Some(0), proof_steps: Some(0),
                }, false, None), strategy::AutoDecision::Stop { .. }),
                "the default single execution cannot silently admit recovery");
            assert!(matches!(strategy::next_automatic(&request, &starts, &[recovery],
                &BTreeSet::new(), Some(&last), pse_model::strategy::WorkObservation {
                    attempts: 1, evaluations: Some(0), iterations: Some(0),
                    factorizations: Some(0), proof_steps: Some(0),
                }, false, None),
                strategy::AutoDecision::Dispatch { candidate: 0 }));
            assert!(matches!(outcome, super::super::solves::Outcome::Native(_)),
                "the forged native observation must retain its original report");
            if let super::super::solves::Outcome::Native(mut refused) = outcome {
                assessed.product.apply(&mut refused);
                let mut untouched = values.clone();
                assert!(commit_block(&mut untouched, &prepared.blocks[0].boundary, Some(&refused),
                    &numerics.policy).is_err());
                assert_eq!(untouched.scalars, values.scalars,
                    "refused independent coordinates cannot become a successor input or stage commit");
            }
            let refreshed = assessor.assess(&super::super::solves::Outcome::Native(report.clone()),
                Some(&run.scope), &budget);
            assert!(refreshed.original.satisfied());
            assert_eq!(refreshed.work[0].observed.evaluations, Some(1));
            assert!(refreshed.product.observed.as_ref().unwrap().0.feasible());
            let wrong_owner = pse_kernels::ExecutionScope::new(Arc::new(AtomicBool::new(false)), run.scope.deadline());
            let refused = assessor.assess(&super::super::solves::Outcome::Native(report.clone()),
                Some(&wrong_owner), &budget);
            assert!(matches!(refused.original, OriginalConclusion::Unavailable { .. }));
            assert_eq!(refused.work[0].observed.evaluations, Some(0));
            let mut original = assessor.original.clone();
            original.time_limit = std::time::Duration::ZERO;
            let expired = BlockAssessor { original, ..assessor };
            let refused = expired.assess(&super::super::solves::Outcome::Native(report.clone()),
                Some(&run.scope), &budget);
            assert!(matches!(refused.original, OriginalConclusion::Unavailable { .. }));
            assert!(!refused.retention.candidate.permits_use());
            assert_eq!(refused.work[0].observed.evaluations, Some(0));
            assert!(!run.scope.cancellation().load(Ordering::Acquire));
        }
        let mut committed = values.clone();
        commit_block(
            &mut committed,
            &prepared.blocks[0].boundary,
            Some(&report),
            &numerics.policy,
        )
        .unwrap();
        assert!(
            (committed.scalars[&prepared.blocks[0].boundary.members.columns[0]] - 1.).abs() < 1e-8
        );
        drop((report, trace));
        let mut plateau = None;
        let mut escaped = Vec::new();
        // The first 24 edits keep only the latest binding. The final 12 deliberately
        // retain distinct old coefficient snapshots, but never old complete bindings.
        for index in 0..36 {
            let old = run.bound[0].as_ref().unwrap();
            let previous = Arc::downgrade(&old._owner);
            let presolve = old.compiled().presolve.clone();
            let derived = old.compiled().derived.clone();
            if index >= 24 {
                let snapshot = old.compiled().coefficient_values.clone();
                assert!(
                    snapshot.iter().any(
                        |(id, bits)| *id == parameter && *bits == (1. + index as f64).to_bits()
                    )
                );
                escaped.push(snapshot);
            }
            values.scalars.insert(parameter, 2. + index as f64);
            let (report, trace) = local.run_ready(run.attempt(0, &values, None)).unwrap();
            let latest = run.bound[0].as_ref().unwrap();
            assert!(
                previous.upgrade().is_none(),
                "actual block attempt retained a preceding complete binding"
            );
            assert!(pse_math::SharedAllocation::ptr_eq(
                &presolve,
                &latest.compiled().presolve
            ));
            assert!(pse_math::SharedAllocation::ptr_eq(
                &derived,
                &latest.compiled().derived
            ));
            assert!(
                latest
                    .compiled()
                    .coefficient_values
                    .iter()
                    .any(|(id, bits)| *id == parameter && *bits == (2. + index as f64).to_bits())
            );
            let fresh = prepared.blocks[0]
                .view
                .bind(
                    prepared.quantities.clone(),
                    prepared.preconditions.clone(),
                    &values,
                    &Arc::new(AtomicBool::new(false)),
                )
                .unwrap();
            assert_eq!(
                latest.compiled().coefficient_values,
                fresh.coefficient_values
            );
            assert_eq!(latest.compiled().presolve.key, fresh.presolve.key);
            assert_eq!(
                latest.compiled().complete(&values).scalars,
                fresh.complete(&values).scalars
            );
            let mut committed = values.clone();
            commit_block(
                &mut committed,
                &prepared.blocks[0].boundary,
                Some(&report),
                &numerics.policy,
            )
            .unwrap();
            assert!(
                (committed.scalars[&prepared.blocks[0].boundary.members.columns[0]] - 1.).abs()
                    < 1e-8
            );
            assert_eq!(committed.scalars[&parameter], 2. + index as f64);
            drop((fresh, report, trace, presolve, derived));
            if index == 0 {
                plateau = Some(service.pool.reserved());
            }
            if index < 24 {
                assert_eq!(service.pool.reserved(), plateau.unwrap());
            }
        }
        assert_eq!(escaped.len(), 12);
        for pair in escaped.windows(2) {
            assert!(!pse_math::SharedAllocation::ptr_eq(&pair[0], &pair[1]));
        }
        assert!(service.pool.reserved() > plateau.unwrap());
        drop(escaped);
        assert_eq!(service.pool.reserved(), plateau.unwrap());
        assert!(
            run.attempts.is_empty(),
            "the latest-only control must not retain attempt history"
        );
        drop(run);
            Ok(())
        });
        #[cfg(feature = "solver-ipopt")]
        {
            let entries = Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let observed = entries.clone();
            super::super::observed_native_entry(
                Arc::new(move |_| {
                    observed.fetch_add(1, Ordering::AcqRel);
                }),
                sequence,
            )
            .await
            .unwrap();
            assert_eq!(
                entries.load(Ordering::Acquire),
                1,
                "all actual block attempts must execute on one native channel entry"
            );
        }
        #[cfg(not(feature = "solver-ipopt"))]
        sequence.await.unwrap();
        session.close().await;
        for resident in other_residents {
            resident.close().await;
        }
        drop((prepared, owner, progress));
        service.clear_program_cache();
        assert_eq!(service.pool.reserved(), 0);
    }
    #[test]
    fn block_commit_is_atomic_and_requires_native_success_plus_original_quality() {
        let boundary = pse_structural::initialization::Block {
            id: pse_structural::incidence::BlockId(pse_ids::ContentHash::from_bytes([2; 32])),
            members: pse_structural::incidence::Part {
                rows: vec![id(2)],
                columns: vec![id(1)],
            },
            inputs: vec![],
        };
        let contract = native::OracleContract {
            identity: pse_ids::ContentHash::from_bytes([1; 32]),
            variables: vec![native::Variable {
                id: id(1),
                lower: f64::NEG_INFINITY,
                upper: f64::INFINITY,
            }],
            rows: vec![id(2)],
            derivatives: pse_kernels::DerivativeOrder::First,
            smoothness: pse_kernels::DerivativeOrder::First,
        };
        let mut r = SolveReport::new(
            Backend::Kinsol,
            &contract,
            kinsol::termination(0),
            &Execution::new(Arc::new(AtomicBool::new(false)), &Controls::default()),
        );
        r.quality = Some(native::quality::Quality::new(vec![], vec![], vec![]).unwrap());
        r.candidate = Some(Candidate {
            kind: CandidateKind::FinalIterate,
            primal: vec![2.0],
            objective: None,
            row_dual: None,
            bound_dual: None,
            reduced_costs: None,
            slacks: None,
            commitment: None,
        });
        let mut values = CaseValues {
            scalars: BTreeMap::from([(id(1), 1.0)]),
        };
        let accuracy = ResolvedAccuracy::from_policy(&Default::default(), 1e-8).unwrap();
        r.termination.category = Termination::Limit;
        native::quality::qualify(&mut r, &accuracy);
        let error =
            commit_block(&mut values, &boundary, Some(&r), &Default::default()).unwrap_err();
        assert!(matches!(
            error.as_ref(),
            native::ProblemError::Numerical { .. }
        ));
        assert!(error.to_string().contains("native_outcome"));
        assert!(error.to_string().contains("termination=Limit"));
        assert_eq!(values.scalars[&id(1)], 1.0);
        r.termination.category = Termination::Success;
        native::quality::qualify(&mut r, &accuracy);
        r.candidate.as_mut().unwrap().primal[0] = f64::NAN;
        let error =
            commit_block(&mut values, &boundary, Some(&r), &Default::default()).unwrap_err();
        assert!(matches!(error.as_ref(), native::ProblemError::Contract(_)));
        assert!(error.to_string().contains("nonfinite_coordinate=Some(0)"));
        assert_eq!(values.scalars[&id(1)], 1.0);
        r.candidate.as_mut().unwrap().primal[0] = 2.0;
        assert!(commit_block(&mut values, &boundary, Some(&r), &Default::default()).is_ok());
        assert_eq!(values.scalars[&id(1)], 2.0);

        // Every true refusal remains atomic; diagnostics distinguish the gate that
        // failed without substituting a different numerical acceptance policy.
        let refuse = |report: Option<&SolveReport>, reason: &str| {
            let mut untouched = CaseValues {
                scalars: BTreeMap::from([(id(1), 1.0)]),
            };
            let cause =
                commit_block(&mut untouched, &boundary, report, &Default::default()).unwrap_err();
            assert_eq!(untouched.scalars, BTreeMap::from([(id(1), 1.0)]));
            assert!(cause.to_string().contains(reason), "{cause}");
            cause
        };
        assert!(matches!(
            refuse(None, "native report unavailable").as_ref(),
            native::ProblemError::Contract(_)
        ));
        let mut rejected = r.clone();
        rejected.candidate = None;
        assert!(matches!(
            refuse(Some(&rejected), "candidate_present=false").as_ref(),
            native::ProblemError::Contract(_)
        ));
        let mut rejected = r.clone();
        rejected.candidate.as_mut().unwrap().primal.clear();
        assert!(matches!(
            refuse(
                Some(&rejected),
                "candidate_length=Some(0); expected_length=1"
            )
            .as_ref(),
            native::ProblemError::Contract(_)
        ));
        let mut rejected = r.clone();
        rejected.variables = vec![id(3)];
        assert!(matches!(
            refuse(Some(&rejected), "variable_order_mismatch_at=0").as_ref(),
            native::ProblemError::Contract(_)
        ));
        let mut rejected = r.clone();
        rejected.quality = None;
        assert!(matches!(
            refuse(Some(&rejected), "feasibility_unavailable").as_ref(),
            native::ProblemError::Contract(_)
        ));
        let mut rejected = r.clone();
        rejected.qualification = Qualification::Unqualified;
        refuse(Some(&rejected), "unqualified");
        let mut rejected = r.clone();
        rejected.quality = Some(
            native::quality::Quality::new(
                (2..=15)
                    .map(|row| native::quality::Violation {
                        id: id(row),
                        physical: 2.0,
                        tolerance: 1.0,
                    })
                    .collect(),
                vec![],
                vec![],
            )
            .unwrap(),
        );
        native::quality::qualify(&mut rejected, &accuracy);
        let cause = refuse(Some(&rejected), "infeasible");
        assert!(matches!(
            cause.as_ref(),
            native::ProblemError::Numerical { .. }
        ));
        let reason = cause.to_string();
        assert!(reason.contains("original_quality.normalized_max=Some(2.0)"));
        assert!(reason.contains(&format!("row={} physical=2 tolerance=1", id(2))));
        assert_eq!(reason.matches("; row=").count(), 12);
        assert!(!reason.contains(&format!("row={}", id(14))));

        let mut rejected = r;
        rejected.variables.clear();
        rejected.record_validation_failure(native::ProblemError::numerical(
            "independent original observation failed",
        ));
        let original = rejected.shared_validation_failure().unwrap();
        let cause = refuse(Some(&rejected), "independent original observation failed");
        assert!(Arc::ptr_eq(&original, &cause));
    }
}
