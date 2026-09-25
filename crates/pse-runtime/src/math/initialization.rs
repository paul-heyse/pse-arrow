// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Conditional initialization and finite supplied continuation; KINSOL owns iteration.
use super::{
    ExecutableCase, ExecutionWorker, MathRuntimeError, MathService, Workspace, solves::SolveHandle,
};
use pse_backend_native::{self as native, kinsol, quality::Tolerances, solve::*};
use pse_compiler::workspace::Profile;
use pse_engine::cache_service::flight::FlightCancellation;
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
    blocks: Vec<(pse_structural::initialization::Block, Arc<ExecutableCase>)>,
    _owner: Arc<pse_columnar::AllocationLease>,
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
                    available: Some(crate::math::solves::ALGEBRAIC_BACKENDS),
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
        if profile.controls.accuracy != Accuracy::default() {
            return Err(native::ProblemError::Contract(
                "initialization accuracy belongs to the numerical policy".into(),
            )
            .into());
        }
        let numerics = Arc::new(pse_math::numerics::resolve(
            &self.quantities,
            &self.targets,
            &[],
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
    /// KLU, bounded dense or matrix-free SPGMR; no custom factorization.
    pub linear: kinsol::Linear,
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
impl MathService {
    /// Execute a declared root/map factory on its owning admitted worker. Causal
    /// sweeps may use nested native unit calculations on this same admission; they
    /// must not recursively request CPU permits. Mutable oracles never enter Salsa.
    pub fn solve_declared_root(
        self: &Arc<Self>,
        contract: native::OracleContract,
        initial: Vec<f64>,
        settings: kinsol::Settings,
        controls: Controls,
        tolerances: Tolerances,
        factory: impl FnOnce(Execution) -> Result<kinsol::Function, native::ProblemError>
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
            let result = service
                .job(1, service.policy.worker_bytes, control, move |flag| {
                    let mut execution = Execution::new(flag, &controls);
                    execution.progress = events;
                    let function = factory(execution.clone())?;
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
                        return Err(native::ProblemError::Contract(
                            "root factory differs from admitted source contract".into(),
                        )
                        .into());
                    }
                    let stamp = Compatibility {
                        layout: contract.identity,
                        data: contract.identity,
                        backend: Backend::Kinsol,
                    };
                    let mut session =
                        kinsol::Session::new(function, settings, execution.clone(), stamp)?;
                    let mut report =
                        session.solve(&initial, &controls, execution, &tolerances, None)?;
                    native::quality::qualify(&mut report, &controls.accuracy);
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
        let owner = self.reserve("math:initialization-products", self.policy.workspace_bytes)?;
        let quantities = revision.quantities.clone();
        let targets = revision
            .cases
            .get(&id)
            .ok_or_else(|| native::ProblemError::Contract("unknown initialization case".into()))?
            .structure
            .numerical_targets(&quantities)?;
        let products = self
            .job(
                1,
                self.policy.stack_bytes,
                FlightCancellation::default(),
                move |_| {
                    let _lease = workspace.lease;
                    let mut compiler = workspace.compiler.lock().map_err(|_| {
                        MathRuntimeError::Infrastructure("compiler lock poisoned".into())
                    })?;
                    compiler.publish(revision)?;
                    compiler
                        .prepare_initialization_blocks(id, profile, order)
                        .map_err(Into::into)
                },
            )
            .await?;
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
            let result = service
                .job(1, service.policy.worker_bytes, control, move |flag| {
                    runner.run_initialization(
                        prepared, values, providers, profile, strategies, numerics, flag, events,
                        owner,
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
    ) -> Result<InitializationReport, MathRuntimeError> {
        #[cfg(feature = "solver-pounce")]
        if strategies
            .iter()
            .any(|s| *s == native::routing::Route::Native(Backend::Pounce))
        {
            return native::pounce::with_threads(1, self.policy.stack_bytes, || {
                self.run_initialization_inner(
                    prepared, original, providers, profile, strategies, numerics, flag, progress,
                    owner,
                )
            });
        }
        self.run_initialization_inner(
            prepared, original, providers, profile, strategies, numerics, flag, progress, owner,
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
                let result = (|| -> Result<Box<SolveReport>, MathRuntimeError> {
                    let providers = providers
                        .iter()
                        .map(|(k, f)| {
                            f.worker()
                                .map(|v| (*k, v))
                                .map_err(|e| native::ProblemError::Contract(e.to_string()))
                        })
                        .collect::<Result<_, _>>()?;
                    let ExecutionWorker {
                        worker,
                        _case,
                        _lease,
                    } = self.worker(case.clone(), providers, flag.clone())?;
                    let facts = Arc::new(case.assembly.presolve_facts(
                        &values,
                        self.policy.worker_bytes / 256,
                        &flag,
                    )?);
                    let oracle = native::assembled::AlgebraicOracle::new(worker, values.clone())?
                        .with_presolve_facts(facts)?;
                    oracle.admit_nle()?;
                    let tolerances = Tolerances::from_policy(
                        &numerics,
                        &boundary.members.columns,
                        &boundary.members.rows,
                    )?;
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
                    let mut controls = profile.controls.clone();
                    controls.accuracy =
                        Accuracy::resolve(&numerics.policy, &tolerances, &normalization)?;
                    let contract = native::NleOracle::contract(&oracle).clone();
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
                    let mut report = match strategy {
                        native::routing::Route::Native(Backend::Kinsol) => {
                            let initial = normalization.normalized_point(&initial)?;
                            let oracle = native::transport::Roots::new(
                                Box::new(oracle),
                                normalization.clone(),
                            )?;
                            let settings = kinsol::Settings {
                                strategy: kinsol::Strategy::LineSearch,
                                linear: profile.linear,
                                variable_scales: tolerances
                                    .variables
                                    .iter()
                                    .zip(&normalization.variables)
                                    .map(|(v, s)| controls.accuracy.feasibility * s / v)
                                    .collect(),
                                residual_scales: tolerances
                                    .rows
                                    .iter()
                                    .zip(&normalization.rows)
                                    .map(|(v, s)| controls.accuracy.feasibility * s / v)
                                    .collect(),
                                anderson: 0,
                                damping: 1.0,
                                setup_interval: 10,
                                step_tolerance: controls.accuracy.feasibility,
                            };
                            let mut session = kinsol::Session::new(
                                kinsol::Function::Equations(Box::new(oracle)),
                                settings,
                                execution.clone(),
                                Compatibility {
                                    layout,
                                    data,
                                    backend: Backend::Kinsol,
                                },
                            )?;
                            let mut report = session.solve(
                                &initial,
                                &controls,
                                execution,
                                &tolerances.normalized(&normalization)?,
                                None,
                            )?;
                            native::transport::recover(&mut report, &normalization, &contract)?;
                            drop(session);
                            report
                        }
                        native::routing::Route::Native(
                            backend @ (Backend::Ipopt | Backend::Pounce),
                        ) => {
                            let oracle = oracle.with_normalization(normalization.clone())?;
                            let mut pipeline = native::presolve::Pipeline::new(
                                Box::new(native::assembled::FeasibilityOracle(Box::new(oracle))),
                                &initial,
                                &native::presolve::Policy::Off,
                                &tolerances,
                                None,
                                execution.clone(),
                                None,
                                Compatibility {
                                    layout,
                                    data,
                                    backend: *backend,
                                },
                                self.policy.worker_bytes / 256,
                            )?;
                            let mut transport = pipeline.take_oracle()?;
                            let result = match backend {
                                #[cfg(feature = "solver-ipopt")]
                                Backend::Ipopt => native::ipopt::Session::new().solve(
                                    &mut transport,
                                    pipeline.initial(),
                                    pse_math::binding::ObjectiveSense::Minimize,
                                    &controls,
                                    execution.clone(),
                                    &pipeline.tolerances(&tolerances),
                                    None,
                                    None,
                                    pipeline.native_compatibility().clone(),
                                )?,
                                #[cfg(feature = "solver-pounce")]
                                Backend::Pounce => native::pounce::Session::new().solve(
                                    Box::new(transport),
                                    pipeline.initial(),
                                    pse_math::binding::ObjectiveSense::Minimize,
                                    &controls,
                                    native::pounce::Method::InteriorPoint,
                                    Default::default(),
                                    execution.clone(),
                                    &pipeline.tolerances(&tolerances),
                                    None,
                                    pipeline.native_compatibility().clone(),
                                )?,
                                _ => {
                                    return Err(native::ProblemError::Contract(
                                        "initialization NLP adapter not linked".into(),
                                    )
                                    .into());
                                }
                            };
                            pipeline.finish(
                                result,
                                &tolerances,
                                pse_math::binding::ObjectiveSense::Minimize,
                            )
                        }
                        _ => {
                            return Err(native::ProblemError::Contract(
                                "unsupported conditional initialization route".into(),
                            )
                            .into());
                        }
                    };
                    native::quality::record_kkt(&mut report, &normalization, &controls.accuracy);
                    native::quality::qualify(&mut report, &controls.accuracy);
                    let previous_attempt = (profile.controls.start == StartPolicy::PreviousAccepted && stage > 0)
                        .then(|| attempts.iter().rposition(|a: &BlockAttempt| a.boundary.id == boundary.id && a.committed)).flatten();
                    report.start_receipt = Some(StartReceipt {
                        previous_attempt,
                        seed: Some(WarmStart {
                            origin: previous_attempt.map(|attempt| SeedOrigin { run: None, attempt }),
                            compatibility: Compatibility {layout,data,backend:report.backend},
                            payload: if report.backend == Backend::Kinsol { WarmPayload::Root(initial) }
                                else { WarmPayload::Nlp {primal:initial,bounds:None,rows:None} },
                        }),
                        sparse_seed: None,
                        transformations: vec!["stage overlay -> source primal -> shared normalization -> native initial point".into()],
                        submitted: true,
                    });
                    drop(_lease);
                    drop(_case);
                    Ok(Box::new(report.with_owner(owner.clone())))
                })()
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
}

fn commit_block(
    values: &mut CaseValues,
    b: &pse_structural::initialization::Block,
    report: Option<&SolveReport>,
) -> bool {
    let Some(r) = report else { return false };
    if !matches!(
        r.termination.category,
        Termination::Success | Termination::Acceptable
    ) || r.validation_error.is_some()
        || !r.quality.as_ref().is_some_and(|q| q.feasible())
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
        r.termination.category = Termination::Limit;
        assert!(!commit_block(&mut values, &boundary, Some(&r)));
        assert_eq!(values.scalars[&id(1)], 1.0);
        r.termination.category = Termination::Success;
        r.candidate.as_mut().unwrap().primal[0] = f64::NAN;
        assert!(!commit_block(&mut values, &boundary, Some(&r)));
        assert_eq!(values.scalars[&id(1)], 1.0);
        r.candidate.as_mut().unwrap().primal[0] = 2.0;
        assert!(commit_block(&mut values, &boundary, Some(&r)));
        assert_eq!(values.scalars[&id(1)], 2.0);
    }
}
