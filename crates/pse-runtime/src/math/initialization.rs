// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Conditional initialization and finite supplied continuation; KINSOL owns iteration.
use super::{
    ExecutableCase, ExecutionWorker, MathRuntimeError, MathService, WorkerBudget, Workspace,
    solves::SolveHandle,
};
use pse_backend_native::{
    self as native,
    execution::{self, BackendExecution, BackendSettings, Retained},
    kinsol,
    quality::Tolerances,
    solve::*,
};
use pse_columnar::flight::FlightCancellation;
use pse_compiler::workspace::Profile;
use pse_ids::{FramedHasher, SemanticId};
use pse_math::binding::CaseValues;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

/// Compiled predecessor-ordered blocks, with explicit conditional input coordinates.
#[derive(Clone, Debug)]
pub struct PreparedInitialization {
    quantities: Arc<pse_quantity::QuantityRegistry>,
    targets: Vec<pse_math::numerics::TargetSpec>,
    requirements: Arc<Vec<pse_math::numerics::SourcedRequirement>>,
    blocks: Vec<(pse_structural::initialization::Block, Arc<ExecutableCase>)>,
    _owner: Arc<super::products::ProductOwner>,
}
impl PreparedInitialization {
    /// Resolve every conditional block before worker acquisition. No fallback follows a failed attempt.
    pub fn strategies(
        &self,
        controls: &Controls,
        selection: SolverSelection,
    ) -> Result<Vec<native::routing::Route>, native::ProblemError> {
        self.blocks
            .iter()
            .map(|(_, case)| {
                let c = native::assembled::contract(&case.assembly);
                let facts = native::routing::oracle_facts(&c, false, true);
                native::routing::Requirements {
                    table: &execution::LINKED,
                    facts: &facts,
                    intent: SolveIntent::Initialize,
                    convex: false,
                    controls,
                }
                .select(selection)
            })
            .collect()
    }
    /// Shared preparation/submission admission for immutable stage overlays.
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
        profile.controls.validate()?;
        if profile.controls.start == StartPolicy::Explicit
            || profile.controls.reuse != ReusePolicy::Fresh
        {
            return Err(native::ProblemError::Contract("initialization uses declared guesses or previous accepted stages, with fresh native allocation".into()).into());
        }
        let strategies = self.strategies(&profile.controls, profile.selection)?;
        // Typed settings must belong to every block's route; KINSOL scales stay per block.
        for route in &strategies {
            if let native::routing::Route::Native(backend) = route {
                execution::adapter(*backend).admit_settings(&profile.backend, &profile.controls)?;
            }
        }
        if profile.controls.threads != 1
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
            &profile.numerics,
        )?);
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
        for (boundary, _) in &self.blocks {
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
        self.blocks.iter().map(|(b, _)| b)
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
    /// Whether its original-quality-validated coordinates were committed.
    pub committed: bool,
}
/// Initialized values and complete bounded attempt history, without an optimum claim.
#[derive(Debug)]
pub struct InitializationReport {
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
/// Explicit finite continuation data and physical acceptance scales.
#[derive(Clone, Debug)]
pub struct InitializationProfile {
    /// Contextual selection applied to each structural block.
    pub selection: SolverSelection,
    /// Same finite execution policy used by other native attempts.
    pub controls: Controls,
    /// Typed backend settings honoured by every block's route, as in a solve; KINSOL
    /// method controls select KLU, bounded dense or matrix-free SPGMR.
    pub backend: BackendSettings,
    /// ID-keyed numerical meaning shared with ordinary solve preparation.
    pub numerics: pse_model::numerics::NumericalPolicy,
    /// Finite prescribed parameter/fixed-coordinate replacements. Use one empty map
    /// for ordinary initialization. Prior values seed a stage only under PreviousAccepted start policy.
    pub stages: Vec<BTreeMap<SemanticId, f64>>,
}
/// Accounted terminal report for an explicitly declared map or Picard splitting.
#[derive(Debug)]
pub struct DeclaredRootReport {
    /// Native root outcome with original residual validation.
    pub report: SolveReport,
    _owner: Arc<pse_columnar::AllocationLease>,
}
/// Inputs of one conditional block attempt.
struct Block<'a> {
    boundary: &'a pse_structural::initialization::Block,
    case: &'a Arc<ExecutableCase>,
    strategy: native::routing::Route,
    values: &'a CaseValues,
    providers: &'a BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
    profile: &'a InitializationProfile,
    numerics: &'a pse_model::numerics::ResolvedNumericalPolicy,
    flag: &'a Arc<AtomicBool>,
    progress: &'a Arc<Progress>,
    started: std::time::Instant,
    previous_attempt: Option<usize>,
    owner: &'a Arc<pse_columnar::AllocationLease>,
    budget: &'a Arc<WorkerBudget>,
}
impl MathService {
    /// Execute a declared root/map factory on its owning admitted worker. Causal
    /// sweeps may use nested native unit calculations on this same admission; they
    /// must not recursively request CPU permits. Mutable oracles never enter Salsa.
    /// The factory charges each evaluator it builds to the job's worker budget.
    pub fn solve_declared_root(
        self: &Arc<Self>,
        contract: native::OracleContract,
        initial: Vec<f64>,
        settings: kinsol::Settings,
        controls: Controls,
        accuracy: ResolvedAccuracy,
        tolerances: Tolerances,
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
        let progress = Arc::new(Progress::new(controls.history));
        let events = progress.clone();
        let service = self.clone();
        let (tx, receiver) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            let bytes = service.policy.worker_bytes;
            // The factory charges every evaluator it builds to this job's reservation.
            let budget = WorkerBudget::new(bytes);
            let result = service
                .job(1, bytes, control, move |flag| {
                    let mut execution = Execution::new(flag, &controls);
                    execution.progress = events;
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
                        profile: accuracy.key()?,
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
                    Ok(DeclaredRootReport {
                        report: report.with_owner(owner.clone()),
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
    /// Prepare conditional demands through the same Salsa database and artifact cache.
    pub async fn prepare_initialization(
        self: &Arc<Self>,
        workspace: Workspace,
        revision: pse_compiler::workspace::Inputs,
        id: SemanticId,
        profile: Profile,
        order: pse_kernels::DerivativeOrder,
    ) -> Result<PreparedInitialization, MathRuntimeError> {
        let foreign = self.policy.foreign_bytes;
        let quantities = revision.quantities.clone();
        let targets = revision
            .cases
            .get(&id)
            .ok_or_else(|| native::ProblemError::Contract("unknown initialization case".into()))?
            .structure
            .numerical_targets(&quantities)?;
        let (products, lease) = self
            .job_retained(
                1,
                self.policy.workspace_bytes,
                FlightCancellation::default(),
                move |_| {
                    let _lease = workspace.lease;
                    let mut compiler = workspace.compiler.lock().map_err(|_| {
                        MathRuntimeError::Infrastructure("compiler lock poisoned".into())
                    })?;
                    compiler.publish(revision)?;
                    let products = compiler.prepare_initialization_blocks(id, profile, order)?;
                    let bytes = products
                        .iter()
                        .try_fold(foreign, |n, p| n.checked_add(p.plan.retained_bytes()))
                        .ok_or(MathRuntimeError::Limit("initialization product extent"))?;
                    Ok((products, bytes))
                },
            )
            .await?;
        self.own_initialization(products, lease, quantities, targets, Vec::new())
            .await
    }
    /// Reuse the same conditional engine with an authored case's selected physical bindings.
    pub async fn prepare_modeling_initialization(
        self: &Arc<Self>,
        workspace: Workspace,
        case: super::Preparation,
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
        let foreign = self.policy.foreign_bytes;
        let control = FlightCancellation::default();
        let operation = self.job_retained(
            1,
            self.policy.workspace_bytes,
            control.clone(),
            move |flag| {
                let _lease = workspace.lease;
                let compiler = workspace.compiler.lock().map_err(|_| {
                    MathRuntimeError::Infrastructure("compiler lock poisoned".into())
                })?;
                let products =
                    compiler.prepare_bound_initialization(case.compiled(), profile, &flag)?;
                let bytes = products
                    .iter()
                    .try_fold(
                        foreign
                            .checked_add(numerical_bytes)
                            .ok_or(MathRuntimeError::Limit("initialization numerical extent"))?,
                        |n, p| n.checked_add(p.plan.retained_bytes()),
                    )
                    .ok_or(MathRuntimeError::Limit("initialization product extent"))?;
                Ok((products, bytes))
            },
        );
        tokio::pin!(operation);
        let (products, lease) = tokio::select! {r=&mut operation=>r?,()=driver.cancelled()=>{control.cancel();let _=operation.await;return Err(MathRuntimeError::Cancelled);}};
        self.own_initialization(products, lease, quantities, targets, numerical.declarations)
            .await
    }
    async fn own_initialization(
        self: &Arc<Self>,
        products: Arc<Vec<pse_compiler::workspace::PreparedBlock>>,
        lease: Arc<pse_columnar::AllocationLease>,
        quantities: Arc<pse_quantity::QuantityRegistry>,
        targets: Vec<pse_math::numerics::TargetSpec>,
        requirements: Vec<pse_math::numerics::SourcedRequirement>,
    ) -> Result<PreparedInitialization, MathRuntimeError> {
        let owner = self.shared_product(
            vec![3, Arc::as_ptr(&products) as usize],
            products.clone(),
            lease,
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
            blocks.push((
                block.boundary.clone(),
                Arc::new(ExecutableCase {
                    assembly,
                    _artifacts: artifacts,
                    _owner: owner.clone(),
                }),
            ));
        }
        Ok(PreparedInitialization {
            quantities,
            targets,
            requirements: Arc::new(requirements),
            blocks,
            _owner: owner,
        })
    }
    /// Admit the complete finite schedule once; each block is solved and validated by
    /// KINSOL before its values become predecessor inputs. No iteration is differentiated.
    pub fn initialize(
        self: &Arc<Self>,
        prepared: PreparedInitialization,
        values: CaseValues,
        providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
        profile: InitializationProfile,
    ) -> Result<SolveHandle<InitializationReport>, MathRuntimeError> {
        let (strategies, numerics) = prepared.validate_profile(&values, &profile)?;
        let size = prepared.blocks.iter().try_fold(
            values.scalars.len().saturating_mul(64),
            |total, (b, _)| {
                b.members
                    .columns
                    .len()
                    .checked_add(b.members.rows.len())
                    .and_then(|v| v.checked_mul(512))
                    .and_then(|v| v.checked_add(profile.controls.report_allowance().ok()?))
                    .and_then(|v| v.checked_mul(profile.stages.len()))
                    .and_then(|v| total.checked_add(v))
                    .ok_or(MathRuntimeError::Limit("initialization result allowance"))
            },
        )?;
        let owner = self.reserve("math:initialization-results", size)?;
        let cancel = FlightCancellation::default();
        let control = cancel.clone();
        let progress = Arc::new(Progress::new(profile.controls.history));
        let events = progress.clone();
        let service = self.clone();
        let (tx, receiver) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            let runner = service.clone();
            let bytes = service.policy.worker_bytes;
            // Each block attempt's evaluator is charged to this job's reservation (F31).
            let budget = WorkerBudget::new(bytes);
            let result = service
                .job(1, bytes, control, move |flag| {
                    runner.run_initialization(
                        prepared, values, providers, profile, strategies, numerics, flag, events,
                        owner, budget,
                    )
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
    fn run_initialization(
        &self,
        prepared: PreparedInitialization,
        original: CaseValues,
        providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
        profile: InitializationProfile,
        strategies: Vec<native::routing::Route>,
        numerics: Arc<pse_model::numerics::ResolvedNumericalPolicy>,
        flag: Arc<AtomicBool>,
        progress: Arc<Progress>,
        owner: Arc<pse_columnar::AllocationLease>,
        budget: Arc<WorkerBudget>,
    ) -> Result<InitializationReport, MathRuntimeError> {
        let adapters: Vec<&dyn BackendExecution> = strategies
            .iter()
            .filter_map(|s| match s {
                native::routing::Route::Native(backend) => Some(execution::adapter(*backend)),
                native::routing::Route::Constant => None,
            })
            .collect();
        execution::scoped(
            &adapters,
            profile.controls.threads,
            self.policy.stack_bytes,
            || {
                self.run_initialization_inner(
                    prepared, original, providers, profile, strategies, numerics, flag, progress,
                    owner, &budget,
                )
            },
        )
    }
    fn run_initialization_inner(
        &self,
        prepared: PreparedInitialization,
        original: CaseValues,
        providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
        profile: InitializationProfile,
        strategies: Vec<native::routing::Route>,
        numerics: Arc<pse_model::numerics::ResolvedNumericalPolicy>,
        flag: Arc<AtomicBool>,
        progress: Arc<Progress>,
        owner: Arc<pse_columnar::AllocationLease>,
        budget: &Arc<WorkerBudget>,
    ) -> Result<InitializationReport, MathRuntimeError> {
        let mut attempts = vec![];
        let mut completed_stages = 0;
        let mut stages = Vec::new();
        let mut committed = CaseValues {
            scalars: BTreeMap::new(),
        };
        let solved: BTreeSet<_> = prepared
            .boundaries()
            .flat_map(|b| b.members.columns.iter().copied())
            .collect();
        let started = std::time::Instant::now();
        for (stage, updates) in profile.stages.iter().enumerate() {
            let mut values = original.clone();
            if profile.controls.start == StartPolicy::PreviousAccepted {
                values
                    .scalars
                    .extend(committed.scalars.iter().map(|(k, v)| (*k, *v)));
            }
            values.scalars.extend(updates.iter().map(|(k, v)| (*k, *v)));
            let attempt_start = attempts.len();
            let mut completed = true;
            for ((boundary, case), strategy) in prepared.blocks.iter().zip(&strategies) {
                if flag.load(Ordering::Acquire) {
                    completed = false;
                    break;
                }
                let previous_attempt = (profile.controls.start == StartPolicy::PreviousAccepted
                    && stage > 0)
                    .then(|| {
                        attempts.iter().rposition(|a: &BlockAttempt| {
                            a.boundary.id == boundary.id && a.committed
                        })
                    })
                    .flatten();
                let result = self
                    .attempt_block(Block {
                        boundary,
                        case,
                        strategy: *strategy,
                        values: &values,
                        providers: &providers,
                        profile: &profile,
                        numerics: &numerics,
                        flag: &flag,
                        progress: &progress,
                        started,
                        previous_attempt,
                        owner: &owner,
                        budget,
                    })
                    .map_err(Arc::new);
                let committed = commit_block(&mut values, boundary, result.as_deref().ok());
                attempts.push(BlockAttempt {
                    stage,
                    strategy: *strategy,
                    boundary: boundary.clone(),
                    result,
                    committed,
                });
                if !committed {
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
                for attempt in &mut attempts[attempt_start..] {
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
            original,
            values: committed,
            stages,
            original_bindings_restored,
            attempts,
            completed_stages,
            cancelled: flag.load(Ordering::Acquire),
            _owner: owner,
        })
    }
    /// One block attempt: its original-coordinate oracle, the route's representation
    /// runner and the submitted-start receipt. A failure commits nothing.
    fn attempt_block(&self, block: Block<'_>) -> Result<Box<SolveReport>, MathRuntimeError> {
        let Block {
            boundary,
            case,
            strategy,
            values,
            providers,
            profile,
            numerics,
            flag,
            progress,
            started,
            previous_attempt,
            owner,
            budget,
        } = block;
        let ExecutionWorker {
            worker,
            _case,
            _charge,
        } = self.worker(case.clone(), providers, flag.clone(), budget)?;
        let facts = Arc::new(case.assembly.presolve_facts(
            &values,
            self.policy.worker_bytes / 256,
            &flag,
        )?);
        let oracle = native::assembled::AlgebraicOracle::new(worker, values.clone())?
            .with_presolve_facts(facts)?;
        oracle.admit_nle()?;
        let tolerances =
            Tolerances::from_policy(&numerics, &boundary.members.columns, &boundary.members.rows)?;
        let initial: Vec<_> = boundary
            .members
            .columns
            .iter()
            .map(|id| values.scalars[id])
            .collect();
        let normalization = pse_math::normalization::Normalization::from_policy(
            &numerics,
            &boundary.members.columns,
            &boundary.members.rows,
        )?;
        let controls = profile.controls.clone();
        let accuracy = ResolvedAccuracy::resolve(&numerics.policy, &tolerances, &normalization)?;
        let mut execution = Execution::new(flag.clone(), &controls);
        execution.progress = progress.clone();
        execution.started = started;
        let mut h = FramedHasher::new("pse.initialization.block.v1");
        for id in &boundary.members.columns {
            h.id(id);
        }
        for id in &boundary.members.rows {
            h.id(id);
        }
        h.hash(&numerics.key).hash(&boundary.id.0);
        let layout = h.finish_hash();
        let mut value_key = FramedHasher::new("pse.initialization.values.v1");
        for (id, value) in &values.scalars {
            value_key.id(id).u64(value.to_bits());
        }
        let data = value_key.finish_hash();
        let mut session = FramedHasher::new("pse.initialization.session.v1");
        session
            .hash(&numerics.key)
            .hash(&controls.identity()?)
            .hash(&profile.backend.identity()?);
        let native::routing::Route::Native(backend) = strategy else {
            return Err(native::ProblemError::Unsupported(
                "unsupported conditional initialization route".into(),
            )
            .into());
        };
        let adapter = execution::adapter(backend);
        let compatibility = Compatibility {
            layout,
            profile: session.finish_hash(),
            data,
            backend,
        };
        // Fresh native allocation per block: nothing is retained across blocks.
        let mut retained = Retained::default();
        let run = execution::Step {
            adapter,
            settings: &profile.backend,
            controls: &controls,
            accuracy: &accuracy,
            execution,
            tolerances: &tolerances,
            normalization: &normalization,
            compatibility: compatibility.clone(),
            warm: None,
        };
        let mut report = match adapter.representation() {
            execution::Representation::Roots => execution::roots(
                run,
                &mut retained,
                execution::Roots {
                    oracle: Box::new(oracle),
                    initial: &initial,
                    owner: None,
                },
            )?,
            execution::Representation::Nlp => execution::nlp(
                run,
                &mut retained,
                execution::Nlp {
                    oracle: Box::new(oracle.with_normalization(normalization.clone())?),
                    initial: &initial,
                    presolve: &native::presolve::Policy::Off,
                    intent: SolveIntent::Initialize,
                    sense: pse_math::binding::ObjectiveSense::Minimize,
                    limit: self.policy.worker_bytes / 256,
                },
            )?,
            execution::Representation::Coefficients
            | execution::Representation::Cone
            | execution::Representation::Factorable
            | execution::Representation::Trajectory => {
                return Err(native::ProblemError::Unsupported(
                    "unsupported conditional initialization route".into(),
                )
                .into());
            }
        };
        drop(retained);
        // The block starts from its staged values. Only a predecessor stage's committed
        // values make that start a seed, submitted as the native initial point; the
        // authored initial point is not a warm start (F25).
        let seed = previous_attempt
            .map(|attempt| -> Result<WarmStart, native::ProblemError> {
                Ok(WarmStart {
                    origin: Some(SeedOrigin { run: None, attempt }),
                    compatibility,
                    payload: adapter.primal_start(initial)?,
                })
            })
            .transpose()?;
        report.start_receipt = Some(StartReceipt {
            previous_attempt,
            transformations: if seed.is_some() {
                SeedTransformation::path(normalization.key(), report.preprocessing.as_ref())
            } else {
                vec![]
            },
            submitted: seed.is_some(),
            seed,
            sparse_seed: None,
        });
        drop(_case);
        drop(_charge);
        Ok(Box::new(report.with_owner(owner.clone())))
    }
}

pub(crate) fn commit_block(
    values: &mut CaseValues,
    b: &pse_structural::initialization::Block,
    report: Option<&SolveReport>,
) -> bool {
    let Some(r) = report else { return false };
    // The native candidate-use decision is the only acceptance rule; commit adds the
    // block's own coordinate and finiteness checks.
    if !crate::workflow::numerics::native_use(r).permits_use() || r.variables != b.members.columns {
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
    fn id(v: u8) -> SemanticId {
        SemanticId::from_bytes([v; 16])
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
        });
        let mut values = CaseValues {
            scalars: BTreeMap::from([(id(1), 1.0)]),
        };
        let accuracy = ResolvedAccuracy::from_policy(&Default::default(), 1e-8).unwrap();
        r.termination.category = Termination::Limit;
        native::quality::qualify(&mut r, &accuracy);
        assert!(!commit_block(&mut values, &boundary, Some(&r)));
        assert_eq!(values.scalars[&id(1)], 1.0);
        r.termination.category = Termination::Success;
        native::quality::qualify(&mut r, &accuracy);
        r.candidate.as_mut().unwrap().primal[0] = f64::NAN;
        assert!(!commit_block(&mut values, &boundary, Some(&r)));
        assert_eq!(values.scalars[&id(1)], 1.0);
        r.candidate.as_mut().unwrap().primal[0] = 2.0;
        assert!(commit_block(&mut values, &boundary, Some(&r)));
        assert_eq!(values.scalars[&id(1)], 2.0);
    }
}
