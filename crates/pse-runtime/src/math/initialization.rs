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
    quantities: pse_math::SharedAllocation<pse_quantity::QuantityRegistry>,
    targets: Vec<pse_math::numerics::TargetSpec>,
    requirements: Arc<Vec<pse_math::numerics::SourcedRequirement>>,
    blocks: Vec<ConditionalBlock>,
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
            Arc<pse_model::numerics::ResolvedNumericalPolicy>,
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
        let numerics = Arc::new(pse_math::numerics::resolve(
            &self.quantities,
            &self.targets,
            &self.requirements,
            &solver.numerics,
        )?);
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
/// before iteration, and each evaluation owns a fresh temporary boundary binding.
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
}

#[cfg(feature = "solver-kinsol")]
impl PreparedConditionalUnit {
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
        numerics: Arc<pse_model::numerics::ResolvedNumericalPolicy>,
        driver: &crate::CancelSource,
    ) -> Result<PreparedConditionalUnit, MathRuntimeError> {
        admit_unit_profile(&profile, &numerics)?;
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
        let boundaries: Vec<_> = view
            .plan
            .structure()
            .rows()
            .iter()
            .enumerate()
            .filter_map(|(index, row)| {
                // Typed difference rows retain the original supplied-member identity.
                view.boundary
                    .inputs
                    .iter()
                    .find(|id| {
                        pse_compiler::workspace::ModelingOutput::ConditionalBoundary(**id).row_id()
                            == row.id
                    })
                    .map(|id| (index, *id))
            })
            .collect();
        let mut effective_numerics = numerics.as_ref().clone();
        if !boundaries.is_empty() {
            use pse_model::generated::enums::NumericalTarget;
            let mut projections = Vec::new();
            let mut defaults = Vec::new();
            for (index, _) in &boundaries {
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
                let original = view.boundary.inputs.iter().find(|id| {
                    pse_compiler::workspace::ModelingOutput::ConditionalBoundary(**id).row_id()
                        == row.id
                });
                if let Some(source) = original.and_then(|id| {
                    numerics
                        .targets
                        .iter()
                        .find(|t| t.id == *id && t.kind == NumericalTarget::Observable)
                }) {
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
                let projected =
                    pse_math::numerics::project_difference(&quantities, &numerics, &projections)?;
                h.hash(&projected.key);
                effective_numerics.targets.extend(projected.targets);
            }
            if !defaults.is_empty() {
                let additional =
                    pse_math::numerics::resolve(&quantities, &defaults, &[], &numerics.policy)?;
                h.hash(&additional.key);
                effective_numerics.targets.extend(additional.targets);
            }
            effective_numerics.key = h.finish_hash();
        }
        let numerics = Arc::new(effective_numerics);
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
        })
    }

    /// A nested unit solve on the already admitted causal-map worker. No second CPU
    /// permit is acquired, and temporary inputs/coordinates never modify the original.
    pub(crate) fn evaluate_conditional_unit(
        &self,
        prepared: &PreparedConditionalUnit,
        values: CaseValues,
        providers: &BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
        execution: &Execution,
        budget: &Arc<WorkerBudget>,
    ) -> Result<CaseValues, native::ProblemError> {
        if let Some(stop) = execution.stopped() {
            return Err(native::ProblemError::stopped(
                stop,
                "conditional causal unit stopped",
            ));
        }
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
        let initial: Vec<_> = prepared
            .view
            .boundary
            .members
            .columns
            .iter()
            .map(|id| {
                values.scalars.get(id).copied().ok_or_else(|| {
                    native::ProblemError::Contract("missing conditional unit start".into())
                })
            })
            .collect::<Result<_, _>>()?;
        let _root_support = budget
            .charge(native::assembled::AlgebraicOracle::root_support_allowance(
                worker.assembly(),
                &prepared.view.structure,
            )?)
            .map_err(MathRuntimeError::into_problem)?;
        let oracle = native::assembled::AlgebraicOracle::new(worker, values.clone())?
            .with_structural_analysis(prepared.view.structure.clone().into())?
            .with_normalization(prepared.normalization.clone())?;
        oracle.admit_nle()?;
        let mut nested = execution.clone();
        // Bound the local attempt by both its declared allowance and the inherited
        // enclosing deadline. Cancellation and progress retain the enclosing owner.
        nested.time_limit = nested.time_limit.min(
            nested
                .started
                .elapsed()
                .saturating_add(prepared.profile.controls.time_limit),
        );
        let mut retained = execution::Retained::default();
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
                    data: values.identity(),
                    backend: prepared.backend,
                },
                warm: None,
            },
            &mut retained,
            execution::Roots {
                oracle: Box::new(oracle),
                initial: &initial,
                owner: Some(Box::new((_case, _charge))),
            },
        )?;
        let mut candidate = values;
        if !commit_block(
            &mut candidate,
            &prepared.view.boundary,
            Some(&report),
            &prepared.profile.numerics,
        ) {
            return Err(conditional_failure(&report));
        }
        // Teardown releases native state, temporary evaluator values and its budget
        // charge on every return path, including callback refusal and cancellation.
        drop(retained);
        Ok(candidate)
    }
}

/// Preserve actual evaluation witnesses before lowering an unqualified native exit.
#[cfg(feature = "solver-kinsol")]
fn conditional_failure(report: &SolveReport) -> native::ProblemError {
    if let Some(cause) = report
        .shared_validation_failure()
        .or_else(|| report.shared_callback_failure())
    {
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
        let job_bytes =
            self.policy
                .worker_bytes
                .checked_add(foreign_bytes)
                .ok_or(MathRuntimeError::Limit(
                    "declared root worker/foreign allowance",
                ))?;
        let run_id = pse_operations::mint_id();
        let (tx, receiver) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            let bytes = service.policy.worker_bytes;
            // The factory charges every evaluator it builds to this job's reservation.
            let budget = WorkerBudget::new(bytes);
            let worker_service = service.clone();
            let result = service
                .job_scoped(1, job_bytes, control, Some(deadline), move |flag| {
                    let original = contract.identity;
                    let start_identity = super::opaque_strategy::point_identity(original, &initial);
                    let (result, trace) = super::opaque_strategy::direct(
                        &worker_service,
                        super::opaque_strategy::Source {
                            original,
                            preparation: original,
                            profile,
                            backend: Some(Backend::Kinsol),
                            controls: &controls,
                            start: pse_model::strategy::StartOrigin::Specification,
                            start_identity: Some(start_identity),
                        },
                        &scope,
                        || {
                            let mut execution = Execution::within(flag, &controls, scope.clone())?;
                            execution.progress = events;
                            execution.memory = Some(foreign_bytes);
                            let function = factory(execution.clone(), budget)?;
                            let actual = function.contract();
                            if actual.identity != contract.identity
                                || actual.rows != contract.rows
                                || actual.variables.len() != n
                                || actual
                                    .variables
                                    .iter()
                                    .zip(&contract.variables)
                                    .any(|(a, b)| {
                                        a.id != b.id
                                            || a.lower.to_bits() != b.lower.to_bits()
                                            || a.upper.to_bits() != b.upper.to_bits()
                                    })
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
                            let mut session =
                                kinsol::Session::new(function, settings, execution.clone(), stamp)?;
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
                        |report| crate::workflow::numerics::native_use(report, &policy),
                        super::strategy::cause_native,
                    )?;
                    let decision = crate::workflow::numerics::native_use(&result, &policy);
                    // The trace owns its independent allocation; native result ownership
                    // survives the joined worker without cloning evaluator storage.
                    Ok(DeclaredRootReport {
                        report: result,
                        run_id,
                        strategy: trace,
                        decision,
                        _owner: owner,
                    })
                })
                .await;
            let _ = tx.send(result);
        });
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
        let quantities = case.compiled().quantities.clone();
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
                let bytes = products
                    .iter()
                    .try_fold(numerical_bytes, |n, p| {
                        n.checked_add(p.plan.retained_bytes())
                    })
                    .ok_or(MathRuntimeError::Limit("initialization product extent"))?;
                Ok((products, bytes))
            });
        tokio::pin!(operation);
        let (products, lease) = tokio::select! {r=&mut operation=>r?,()=driver.cancelled()=>{control.cancel();let _=operation.await;return Err(MathRuntimeError::Cancelled);}};
        self.own_initialization(products, lease, quantities, targets, numerical.declarations)
            .await
    }
    async fn own_initialization(
        self: &Arc<Self>,
        products: Arc<Vec<pse_compiler::workspace::PreparedBlock>>,
        lease: Arc<pse_columnar::AllocationLease>,
        quantities: pse_math::SharedAllocation<pse_quantity::QuantityRegistry>,
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
                    _owner: owner.clone(),
                }),
            });
        }
        Ok(PreparedInitialization {
            snapshot: execution::Snapshot::observe(&execution::LINKED),
            quantities,
            targets,
            requirements: Arc::new(requirements),
            blocks,
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
        let scope = pse_kernels::ExecutionScope::new(Arc::default(), Some(deadline));
        let run_id = pse_operations::mint_id();
        let session = self.open_session()?;
        let service = self.clone();
        let events = progress.clone();
        Ok(SolveHandle::supervise(progress, move |cancel| async move {
            let mut run = Blocks {
                service: &service,
                session: &session,
                prepared: &prepared,
                providers: &providers,
                profile: &profile,
                strategies: &strategies,
                numerics: &numerics,
                progress: &events,
                owner: &owner,
                cancel: &cancel,
                scope,
                run_id,
                bound: vec![None; prepared.blocks.len()],
                attempts: Vec::new(),
            };
            let report = run.stages(values).await;
            session.close().await;
            report
        }))
    }
}
/// The block initialization planner over one native session.
struct Blocks<'a> {
    service: &'a Arc<MathService>,
    session: &'a super::NativeSession,
    prepared: &'a PreparedInitialization,
    providers: &'a BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
    profile: &'a InitializationProfile,
    strategies: &'a [native::routing::Route],
    numerics: &'a Arc<pse_model::numerics::ResolvedNumericalPolicy>,
    progress: &'a Arc<Progress>,
    owner: &'a Arc<pse_columnar::AllocationLease>,
    cancel: &'a crate::CancelSource,
    scope: pse_kernels::ExecutionScope,
    run_id: pse_model::generated::identities::RunId,
    /// Each block's view as last bound; later stages rebind its values (A6).
    bound: Vec<Option<Preparation>>,
    attempts: Vec<BlockAttempt>,
}
impl Blocks<'_> {
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
                );
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
        let bound = match &self.bound[index] {
            Some(view) => {
                self.service
                    .rebind_within(view, values.clone(), self.cancel, self.scope.clone())
                    .await?
            }
            None => {
                self.service
                    .bind_block(
                        block,
                        self.prepared.quantities.clone(),
                        values.clone(),
                        self.cancel,
                        self.scope.deadline(),
                    )
                    .await?
            }
        };
        self.scope.check().map_err(native::ProblemError::Provider)?;
        self.bound[index] = Some(bound.clone());
        let step = self
            .service
            .prepare_conditional(
                bound,
                block.executable.clone(),
                values.clone(),
                self.providers.clone(),
                // Admitted by `validate_profile`: a root or initialize intent, exact
                // convexity and no explicit preprocessing; blocks run the initialize intent
                // on their identity transport.
                SolverProfile {
                    presolve: native::presolve::Policy::Off,
                    intent: SolveIntent::Initialize,
                    ..self.profile.solver.clone()
                },
                self.numerics.clone(),
                self.strategies[index],
                self.prepared.snapshot.clone(),
                &self.scope,
                self.cancel,
            )
            .await?;
        // The block starts from its staged values. Only a predecessor stage's committed
        // values make that start a seed; the authored initial point is not a warm start (F25).
        let step = step.within_task(self.scope.clone())?;
        let previous = previous
            .map(|attempt| step.primal_seed().map(|seed| Predecessor { attempt, seed }))
            .transpose()?;
        let numerical_policy = self.numerics.policy.clone();
        let (outcome, (), strategy) = self
            .session
            .step(
                step,
                previous,
                self.attempts.len(),
                self.progress.clone(),
                self.owner.clone(),
                self.cancel,
                move |outcome, _, _| {
                    (
                        (),
                        super::StepRetention {
                            candidate: outcome.candidate_use(&numerical_policy),
                            session: super::SessionDisposition::RetainCompatible,
                        },
                    )
                },
            )
            .await?;
        match outcome {
            super::solves::Outcome::Native(report) => Ok((report, strategy)),
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
impl MathService {
    /// Bind a conditional block's view to its first values; its programs are the block's own.
    async fn bind_block(
        self: &Arc<Self>,
        block: &ConditionalBlock,
        quantities: pse_math::SharedAllocation<pse_quantity::QuantityRegistry>,
        values: CaseValues,
        driver: &crate::CancelSource,
        deadline: Option<std::time::Instant>,
    ) -> Result<Preparation, MathRuntimeError> {
        let view = block.view.clone();
        let control = FlightCancellation::default();
        let operation = self.job_retained_scoped(
            1,
            self.policy.worker_bytes,
            control.clone(),
            deadline,
            move |flag| {
                let bound = view.bind(quantities, &values, &flag)?;
                // The block plan, structural witness, descriptors and registry already
                // have owners. Only this first binding's products and wrappers escape.
                let bytes = block_binding_bytes(&bound);
                Ok((bound, bytes))
            },
        );
        tokio::pin!(operation);
        let (bound, lease) = tokio::select! {result=&mut operation=>result?,()=driver.cancelled()=>{control.cancel();let _=operation.await;return Err(MathRuntimeError::Cancelled);}};
        let executable = Arc::new(std::sync::OnceLock::from(block.executable.clone()));
        Ok(Self::own_binding(
            block.owner.clone(),
            bound,
            lease,
            executable,
        ))
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

pub(crate) fn commit_block(
    values: &mut CaseValues,
    b: &pse_structural::initialization::Block,
    report: Option<&SolveReport>,
    policy: &pse_model::numerics::NumericalPolicy,
) -> bool {
    let Some(r) = report else { return false };
    // The native candidate-use decision is the only acceptance rule; commit adds the
    // block's own coordinate and finiteness checks.
    if !crate::workflow::numerics::native_use(r, policy).permits_use()
        || r.variables != b.members.columns
    {
        return false;
    }
    let Some(c) = &r.candidate else { return false };
    if c.primal.len() != b.members.columns.len() || c.primal.iter().any(|v| !v.is_finite()) {
        return false;
    }
    values.scalars.extend(
        b.members
            .columns
            .iter()
            .copied()
            .zip(c.primal.iter().copied()),
    );
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;
    fn id(v: u8) -> SemanticId {
        SemanticId::from_bytes([v; 16])
    }
    fn declared_submission(
        service: &Arc<MathService>,
        duration: std::time::Duration,
        called: Arc<AtomicBool>,
    ) -> SolveHandle<DeclaredRootReport> {
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
        };
        service
            .solve_declared_root(
                contract,
                vec![0.],
                settings,
                controls,
                accuracy,
                tolerances,
                policy,
                super::super::solves::profile_key(&profile).unwrap().as_id(),
                move |_, _| {
                    called.store(true, std::sync::atomic::Ordering::Release);
                    Err(native::ProblemError::Internal(
                        "queued declared root factory must not run".into(),
                    ))
                },
            )
            .unwrap()
    }
    #[tokio::test]
    async fn declared_root_deadline_and_cancellation_cover_cpu_wait_without_dispatch() {
        use std::sync::atomic::Ordering;
        let service = super::super::tests::service();
        let baseline = service.pool.reserved();
        let permit = service.cpu.clone().acquire_many_owned(2).await.unwrap();
        let called = Arc::new(AtomicBool::new(false));
        let handle = declared_submission(
            &service,
            std::time::Duration::from_millis(25),
            called.clone(),
        );
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
            declared_submission(&service, std::time::Duration::from_secs(5), called.clone());
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
    #[cfg(feature = "solver-kinsol")]
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
    }
    #[tokio::test]
    async fn first_block_binding_retains_shared_parents_until_last_alias() {
        pse_math::initialize().unwrap();
        let service = super::super::tests::service();
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
            .own_initialization(Arc::new(vec![view]), lease, quantities, vec![], vec![])
            .await
            .unwrap();
        drop(plan);
        drop(registry);
        let parents = service.pool.reserved();
        // Filling the existing test pool must still refuse worker admission, with no
        // retained binding or parent charge lost by the failed first attempt.
        let pressure = service
            .reserve("math:test-pressure", (512 << 20) - parents)
            .unwrap();
        assert!(matches!(
            service
                .bind_block(
                    &prepared.blocks[0],
                    prepared.quantities.clone(),
                    CaseValues {
                        scalars: BTreeMap::new()
                    },
                    &crate::CancelSource::new(),
                    None,
                )
                .await,
            Err(MathRuntimeError::Pool(_))
        ));
        drop(pressure);
        assert_eq!(service.pool.reserved(), parents);
        let bound = service
            .bind_block(
                &prepared.blocks[0],
                prepared.quantities.clone(),
                CaseValues {
                    scalars: BTreeMap::new(),
                },
                &crate::CancelSource::new(),
                None,
            )
            .await
            .unwrap();
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
        drop(prepared);
        drop(bound);
        assert!(service.pool.reserved() > baseline + registry_bytes);
        drop(binding_alias);
        assert!(service.pool.reserved() > baseline);
        drop(registry_alias);
        assert_eq!(service.pool.reserved(), baseline);
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
        assert!(!commit_block(
            &mut values,
            &boundary,
            Some(&r),
            &Default::default()
        ));
        assert_eq!(values.scalars[&id(1)], 1.0);
        r.termination.category = Termination::Success;
        native::quality::qualify(&mut r, &accuracy);
        r.candidate.as_mut().unwrap().primal[0] = f64::NAN;
        assert!(!commit_block(
            &mut values,
            &boundary,
            Some(&r),
            &Default::default()
        ));
        assert_eq!(values.scalars[&id(1)], 1.0);
        r.candidate.as_mut().unwrap().primal[0] = 2.0;
        assert!(commit_block(
            &mut values,
            &boundary,
            Some(&r),
            &Default::default()
        ));
        assert_eq!(values.scalars[&id(1)], 2.0);
    }
}
