// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The horizon driver: admission before any effect, the durable attempt, and the loop over
//! one staged sequence (Plan 22 Y5c1).
use super::{
    Horizon, HorizonController, HorizonDecision, HorizonEstimator, HorizonReport, HorizonSignal,
    HorizonStep, WindowInput,
};
use crate::math::MathRuntimeError;
use crate::workflow::{
    ModelingAnalysis, ModelingPackage, ModelingResult, ModelingSolvePreparation, RunHandle,
    RunReport, RunRequest, RunResult, Runtime, WorkflowError, contract,
    durable::DurableAttempt,
    integrated::IntegratedExperiment,
    modeling::assessment::Obligations,
    run::{attempt_for, progress_for},
    staged::{Overlay, Staged},
};
use pse_backend_native::{
    self as native, ProblemError,
    dynamics::DynamicSensitivity,
    kkt::{Fallback, Prediction},
    solve::{Event, Metric, Progress},
};
use pse_ids::{FramedHasher, SemanticId};
use pse_model::generated::identities::RunId;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    sync::{Arc, atomic::AtomicBool},
    time::{Duration, Instant},
};

impl Runtime {
    /// Validate a reconstructed horizon through its owner before study scheduling.
    pub(in crate::workflow) async fn admit_horizon(
        &self,
        horizon: Horizon,
        cancel: &crate::CancelSource,
    ) -> Result<(), WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                if cancel.token().is_cancelled() {
                    return Err(MathRuntimeError::Cancelled.into());
                }
                let admitted = Admitted::new(self, horizon, cancel).await?;
                admitted.close_sources().await
            })
            .await
    }

    /// Start a rolling horizon (Plan 22 Y5c; ADR-0110 Outcome 5): one run of closed-loop
    /// samples over one native session. Everything the loop needs is admitted before any
    /// effect: the plant, its driven inputs and measured outputs, and each stage's
    /// specification, prepared once. An ephemeral run opens its session now and is refused
    /// when none is free; a durable run registers one attempt, queues for a session and runs
    /// under its lease. Its estimator and controller steps are the run's modeling steps, and
    /// [`RunResult::horizon`] reports every sample.
    ///
    /// # Errors
    /// An inconsistent horizon, a stage that does not prepare, cancellation, or no native
    /// session for an ephemeral run.
    pub async fn start_horizon(
        &self,
        horizon: Horizon,
        cancel: &crate::CancelSource,
    ) -> Result<RunHandle, WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async { self.start_horizon_owned(horizon, cancel, None).await })
            .await
    }
    pub(crate) async fn start_horizon_attempt(
        &self,
        horizon: Horizon,
        cancel: &crate::CancelSource,
        attempt: DurableAttempt,
    ) -> Result<RunHandle, WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                self.start_horizon_owned(horizon, cancel, Some(attempt))
                    .await
            })
            .await
    }
    async fn start_horizon_owned(
        &self,
        horizon: Horizon,
        cancel: &crate::CancelSource,
        attempt: Option<DurableAttempt>,
    ) -> Result<RunHandle, WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                let mut attempt = attempt;
                if cancel.token().is_cancelled() {
                    let error = Arc::new(WorkflowError::from(MathRuntimeError::Cancelled));
                    if let Some(attempt) = attempt.take() {
                        let _ = attempt.abandon(&error).await;
                    }
                    return Err(WorkflowError::Shared(error));
                }
                if matches!(self.durability(), crate::workflow::Durability::Ephemeral)
                    && attempt.is_some()
                {
                    let error =
                        Arc::new(contract("a claimed horizon runs only on a durable runtime"));
                    if let Some(attempt) = attempt.take() {
                        let _ = attempt.abandon(&error).await;
                    }
                    return Err(WorkflowError::Shared(error));
                }
                let admitted = match Admitted::new(self, horizon, cancel).await {
                    Ok(admitted) => Arc::new(admitted),
                    Err(error) => {
                        let error = Arc::new(error);
                        if let Some(attempt) = attempt.take() {
                            let _ = attempt.abandon(&error).await;
                        }
                        return Err(WorkflowError::Shared(error));
                    }
                };
                let durable = match attempt_for(self, attempt) {
                    Ok(durable) => durable,
                    Err(error) => {
                        admitted.close_sources().await?;
                        return Err(error);
                    }
                };
                let progress = progress_for(admitted.history, durable.as_ref());
                let opened = match durable {
                    None => match Staged::open(self, Some(progress.clone())) {
                        Ok(staged) => Some(staged),
                        Err(error) => {
                            admitted.close_sources().await?;
                            return Err(error);
                        }
                    },
                    Some(_) => None,
                };
                let cancel = crate::CancelSource::new();
                let checks = cancel.clone();
                let (sender, receiver) = tokio::sync::watch::channel(None);
                let runtime = self.clone();
                let run_id: RunId = durable
                    .as_ref()
                    .and_then(DurableAttempt::claimed_run)
                    .unwrap_or_else(pse_operations::mint_id);
                let attempt_id = durable.as_ref().map(DurableAttempt::attempt_id);
                let canonical_keys = durable
                    .as_ref()
                    .map(|attempt| attempt.canonical_keys(run_id));
                let stream = progress.clone();
                tokio::spawn(crate::workflow::inherit_runtime_activity(async move {
                    use futures_util::FutureExt;
                    let source_owner = admitted.clone();
                    let scopes = admitted
                        .held
                        .iter()
                        .map(|source| source.scope.clone())
                        .collect::<Vec<_>>();
                    let mut operation = async {
                        let mut durable = durable;
                        let staged = match admit(
                            &mut durable,
                            run_id,
                            &admitted,
                            opened,
                            &runtime,
                            &stream,
                            &cancel,
                        )
                        .await
                        {
                            Ok(staged) => staged,
                            Err(error) => {
                                return RunResult::joined(
                                    run_id,
                                    runtime,
                                    RunRequest::Horizon(admitted.request()),
                                    None,
                                    Err(Arc::new(error)),
                                )
                                .refused(durable)
                                .await;
                            }
                        };
                        let original =
                            match super::prefix::OriginalHistory::new(&runtime, &admitted.output) {
                                Ok(original) => original,
                                Err(error) => {
                                    staged.close().await;
                                    return RunResult::joined(
                                        run_id,
                                        runtime,
                                        RunRequest::Horizon(admitted.request()),
                                        None,
                                        Err(Arc::new(error)),
                                    )
                                    .refused(durable)
                                    .await;
                                }
                            };
                        if let Some(attempt) = &mut durable
                            && let Err(error) = attempt
                                .begin_horizon_prefix(admitted.output.clone(), admitted.steps * 3)
                        {
                            staged.close().await;
                            return RunResult::joined(
                                run_id,
                                runtime,
                                RunRequest::Horizon(admitted.request()),
                                None,
                                Err(Arc::new(error)),
                            )
                            .refused(durable)
                            .await;
                        }
                        let mut run = Loop::new(
                            admitted,
                            staged,
                            run_id,
                            cancel.clone(),
                            stream,
                            durable,
                            original,
                        );
                        let outcome = run.run().await;
                        run.finish(runtime, outcome, cancel.token().is_cancelled())
                            .await
                    }
                    .boxed();
                    // The same actual owners stay admitted through native teardown, progress
                    // drain, tables, seeds and final settlement, including every refusal.
                    for scope in scopes {
                        let phase = &cancel;
                        operation = async move {
                            let (mut result, renewal) = scope.joined(phase, operation).await;
                            if let Some(error) = renewal {
                                result.report = Err(Arc::new(error));
                            }
                            result
                        }
                        .boxed();
                    }
                    let mut result = operation.await;
                    if let Err(error) = source_owner.close_sources().await {
                        result.report = Err(Arc::new(error));
                    }
                    sender.send_replace(Some(Arc::new(result)));
                }));
                Ok(RunHandle::staged(
                    checks,
                    receiver,
                    progress,
                    run_id,
                    attempt_id,
                    canonical_keys,
                ))
            })
            .await
    }
}

/// Preserve the actual completed prefix on failure. A failed target ordinal is supplied
/// only by dispatched solve handling; plant/task failures name no mathematical target.
pub(super) fn joined_modeling(
    outcome: Result<(), WorkflowError>,
    completed: Vec<ModelingResult>,
    failed_solve: Option<usize>,
) -> (
    Result<RunReport, Arc<WorkflowError>>,
    Option<crate::workflow::run::ModelingFailure>,
) {
    match outcome {
        Ok(()) => (Ok(RunReport::Modeling(completed)), None),
        Err(error) => (
            Err(Arc::new(error)),
            Some(crate::workflow::run::ModelingFailure {
                completed,
                step: failed_solve,
            }),
        ),
    }
}

/// Register the durable attempt (planned, queued), wait for a native session, and enter
/// running under its lease; an ephemeral run already holds its session.
async fn admit(
    durable: &mut Option<DurableAttempt>,
    run_id: RunId,
    admitted: &Admitted,
    opened: Option<Staged>,
    runtime: &Runtime,
    progress: &Arc<Progress>,
    cancel: &crate::CancelSource,
) -> Result<Staged, WorkflowError> {
    if let Some(attempt) = durable.as_ref() {
        attempt
            .register_horizon(
                run_id,
                runtime,
                &admitted.sources,
                admitted.identity.as_id(),
                &admitted.provenance,
            )
            .await?;
    }
    if let Some(attempt) = durable.as_mut() {
        let stop = cancel.clone();
        attempt.start(Arc::new(move || stop.cancel())).await?;
    }
    let staged = match opened {
        Some(staged) => staged,
        None => tokio::select! {
            staged = Staged::open_queued(runtime, Some(progress.clone()), cancel, None) => staged?,
            () = cancel.cancelled() => return Err(MathRuntimeError::Cancelled.into()),
        },
    };
    if let Some(attempt) = durable.as_mut() {
        let stop = cancel.clone();
        attempt.start(Arc::new(move || stop.cancel())).await?;
    }
    Ok(staged)
}

/// An estimator or controller specification, prepared once at admission: its structure is
/// shared by every step, which rebinds values only.
#[derive(Debug)]
struct Stage {
    package: ModelingPackage,
    /// Exact resolved demand, numerical configuration and selected authored starts.
    provenance: serde_json::Value,
    /// The specification every step composes its overlay over; it demands every path the
    /// loop reads.
    analysis: ModelingAnalysis,
    /// The symbol of every path the loop reads from a solution.
    read: BTreeMap<String, SemanticId>,
    /// The prepared template's request identity.
    identity: pse_ids::roles::LineageRequestHash,
}
impl Stage {
    /// Admit a stage whose steps bind `bound` and whose solutions the loop reads at `read`.
    async fn admit(
        runtime: &Runtime,
        package: &ModelingPackage,
        analysis: &ModelingAnalysis,
        bound: &[&String],
        read: &[&String],
        cancel: &crate::CancelSource,
    ) -> Result<Self, WorkflowError> {
        if !Arc::ptr_eq(&package.runtime.shared, &runtime.shared) {
            return Err(contract("a horizon stage belongs to the horizon's runtime"));
        }
        let mut distinct = BTreeSet::new();
        for path in bound {
            if !distinct.insert(path.as_str()) {
                return Err(contract(format!("a horizon stage binds {path} twice")));
            }
            // A bound path that is not already a case value would change the specialization
            // at every step; as a case value it rebinds a value only (A6).
            if !analysis.case.values.contains_key(path.as_str()) {
                return Err(contract(format!(
                    "a horizon binds only case values of its stages' specifications: {path}"
                )));
            }
        }
        let mut analysis = analysis.clone();
        analysis
            .bindings
            .demand
            .extend(read.iter().map(|p| (*p).clone()));
        analysis.bindings.demand.sort();
        analysis.bindings.demand.dedup();
        let prepared = package.prepare_analysis(&analysis, cancel).await?;
        let paths = &prepared.model.model.compiled().model.paths;
        let read = read
            .iter()
            .map(|path| {
                paths
                    .get(path.as_str())
                    .map(|id| ((*path).clone(), *id))
                    .ok_or_else(|| {
                        contract(format!("horizon path {path} names no symbol of its stage"))
                    })
            })
            .collect::<Result<_, _>>()?;
        let identity = prepared
            .solve
            .request_identity()
            .map_err(MathRuntimeError::from)?;
        let provenance = serde_json::to_value((
            1_u8,
            package.source_attribution(),
            analysis.root,
            analysis.instance,
            &analysis.bindings.demand,
            &read,
            &prepared.starts,
            prepared.solve.numerical_strategy(),
        ))
        .map_err(|error| contract(error.to_string()))?;
        Ok(Self {
            package: package.clone(),
            provenance,
            analysis,
            read,
            identity,
        })
    }
}

/// A controller binding resolved against the plant, the inputs and the estimator.
#[derive(Clone, Debug, serde::Serialize)]
enum Signal {
    /// A plant output, by its position in the contract's outputs.
    Measured(usize),
    /// An estimator symbol.
    Estimated(SemanticId),
    /// A driven input.
    Applied(usize),
    /// One value per step.
    Trajectory(Vec<f64>),
}

#[derive(Debug)]
struct Estimator {
    stage: Stage,
    window: usize,
    /// Plant output positions and their window paths.
    measurements: Vec<(usize, Vec<String>)>,
    inputs: Vec<(usize, WindowInput)>,
    /// Each arrival cost: its prior's case value, the symbol that gives the next prior, and
    /// the first prior.
    arrival: Vec<(String, SemanticId, f64)>,
}

#[derive(Debug)]
struct Controller {
    stage: Stage,
    bindings: Vec<(String, Signal)>,
    /// Each first move's symbol and its driven input.
    moves: Vec<(SemanticId, usize)>,
    advanced: Option<Advanced>,
}

/// An advanced-step controller's sensitivity parameters and predictions (Plan 22 Y5c2).
#[derive(Debug, serde::Serialize)]
struct Advanced {
    /// Each measured or estimated binding and its parameter symbol, in the order of the
    /// stage's sensitivity request.
    parameters: Vec<(String, SemanticId)>,
    /// The controller symbol that predicts a binding one period ahead.
    predictions: BTreeMap<String, SemanticId>,
}

/// The plant: its integrated experiment and the columns the loop drives.
#[derive(Debug)]
struct Plant {
    experiment: IntegratedExperiment,
    worker_bytes: usize,
    /// The integration column of each driven input.
    columns: Vec<usize>,
    /// The differential states, whose ends anchor the next period.
    differential: Vec<usize>,
    start: f64,
    period: f64,
    time_limit: Duration,
}
struct PlantWorker {
    inner: crate::workflow::dynamics::DynamicWorker,
    _charge: Arc<dyn pse_math::AllocationOwner>,
}
impl Plant {
    fn time(&self, k: usize) -> f64 {
        self.start + k as f64 * self.period
    }
    /// The integration vector with the driven inputs at `applied`, held over the period.
    fn integration(&self, applied: &[f64]) -> Vec<f64> {
        let mut parameters = self.experiment.parameters.clone();
        for (column, value) in self.columns.iter().zip(applied) {
            parameters[*column] = *value;
        }
        parameters
    }
    fn execution(
        &self,
        scope: pse_kernels::ExecutionScope,
    ) -> Result<native::solve::Execution, ProblemError> {
        let controls = native::solve::Controls {
            time_limit: self.time_limit,
            ..Default::default()
        };
        native::solve::Execution::within(scope.cancellation().clone(), &controls, scope)
    }
    /// The consistently initialized start at `t₀` under `applied`: the simulation's initial
    /// condition, integrated over the first period sampled at its start only.
    fn initial(
        &self,
        worker: &mut crate::workflow::dynamics::DynamicWorker,
        applied: &[f64],
        execution: &native::solve::Execution,
    ) -> Result<native::dynamics::Sample, ProblemError> {
        use native::dynamics::Oracle;
        let integration = self.integration(applied);
        let contract = &self.experiment.program.contract;
        let initial = worker.evaluate(
            0,
            native::dynamics::Function::Initial,
            self.start,
            &vec![0.; contract.states.len()],
            &self
                .experiment
                .profile
                .parameters_at(&integration, self.start),
            false,
        )?;
        let anchors = self
            .differential
            .iter()
            .map(|i| initial.values[*i])
            .collect::<Vec<_>>();
        let window = self
            .experiment
            .window(self.start, self.time(1), vec![self.start])?;
        let report = self.experiment.integrate_window(
            worker,
            &window,
            &integration,
            &anchors,
            execution,
            DynamicSensitivity::None,
        )?;
        report
            .samples
            .into_iter()
            .next()
            .ok_or_else(|| ProblemError::internal("plant start without its sample"))
    }
    /// Period `k`: from the anchored states at `tₖ` with `applied` held (a one-interval
    /// schedule), sampled at `tₖ₊₁`.
    fn period(
        &self,
        worker: &mut crate::workflow::dynamics::DynamicWorker,
        k: usize,
        applied: &[f64],
        anchors: &[f64],
        execution: &native::solve::Execution,
    ) -> Result<native::dynamics::Sample, ProblemError> {
        let end = self.time(k + 1);
        let window = self.experiment.window(self.time(k), end, vec![end])?;
        let report = self.experiment.integrate_window(
            worker,
            &window,
            &self.integration(applied),
            anchors,
            execution,
            DynamicSensitivity::None,
        )?;
        report
            .samples
            .into_iter()
            .next_back()
            .ok_or_else(|| ProblemError::internal("plant period without its end sample"))
    }
}

#[derive(Debug)]
struct HorizonSourceEngagement {
    source: ModelingPackage,
    scope: Arc<crate::workflow::modeling::engagement::HeldPreparationScope>,
    owned: bool,
}
fn bind_horizon_source(
    source: &ModelingPackage,
    held: &mut Vec<HorizonSourceEngagement>,
) -> ModelingPackage {
    if let Some(current) = held
        .iter()
        .find(|current| crate::workflow::modeling::engagement::same_source(&current.source, source))
    {
        return current.scope.package(source);
    }
    let (scope, owned) = match &source.held_preparation {
        Some(scope) => (scope.clone(), false),
        None => (
            crate::workflow::modeling::engagement::HeldPreparationScope::new(),
            true,
        ),
    };
    let package = scope.package(source);
    held.push(HorizonSourceEngagement {
        source: source.clone(),
        scope,
        owned,
    });
    package
}
async fn close_horizon_sources(held: &[HorizonSourceEngagement]) -> Result<(), WorkflowError> {
    let mut failure = None;
    for source in held {
        if source.owned
            && let Err(error) = source.scope.close().await
            && failure.is_none()
        {
            failure = Some(error);
        }
    }
    if let Some(error) = failure {
        Err(error)
    } else {
        Ok(())
    }
}

/// A horizon admitted before any effect.
#[derive(Debug)]
pub(super) struct Admitted {
    held: Vec<HorizonSourceEngagement>,
    _held_owner: Option<Arc<pse_columnar::AllocationLease>>,
    /// Existing operation-owner projections recorded before native effects.
    provenance: Vec<u8>,
    original: Arc<Horizon>,
    sources: Vec<ModelingPackage>,
    plant: Arc<Plant>,
    steps: usize,
    output: crate::workflow::StudyOutputDemand,
    _history_owner: Arc<pse_columnar::AllocationLease>,
    /// The driven plant parameters and their initial values.
    inputs: Vec<SemanticId>,
    initial: Vec<f64>,
    /// The plant's contract outputs.
    outputs: Vec<SemanticId>,
    estimator: Option<Estimator>,
    controller: Option<Controller>,
    /// The horizon's request identity, registered with its durable attempt.
    identity: pse_ids::roles::LineageRequestHash,
    /// Native threads of the stages' solves; the plant runs on as many, so the session keeps
    /// their retained state.
    threads: usize,
    /// Retained events of the run's in-memory stream.
    history: usize,
}
impl Admitted {
    pub(super) fn request(&self) -> super::HorizonRequest {
        super::HorizonRequest {
            identity: self.identity.as_id(),
            sources: self.sources.clone(),
            provenance: self.provenance.clone(),
            original: self.original.clone(),
        }
    }
    pub(super) async fn new(
        runtime: &Runtime,
        mut horizon: Horizon,
        cancel: &crate::CancelSource,
    ) -> Result<Self, WorkflowError> {
        let owner = runtime.shared.math().reserve(
            "horizon:held-source-owners",
            3 * (size_of::<HorizonSourceEngagement>() + 128),
        )?;
        let mut held = Vec::<HorizonSourceEngagement>::with_capacity(3);
        horizon.plant.source = bind_horizon_source(&horizon.plant.source, &mut held);
        if let Some(estimator) = &mut horizon.estimator {
            estimator.package = bind_horizon_source(&estimator.package, &mut held);
        }
        if let Some(controller) = &mut horizon.controller {
            controller.package = bind_horizon_source(&controller.package, &mut held);
        }
        let result = async {
            for source in &held {
                source.scope.check_active().await?;
            }
            for original in horizon.plant.original_preparations() {
                horizon
                    .plant
                    .source
                    .prepare(
                        original.root,
                        original.instance,
                        original.bindings.clone(),
                        original.limits,
                        cancel,
                    )
                    .await?;
            }
            let mut admitted = Self::build(runtime, horizon, cancel).await?;
            for source in &held {
                source.scope.freeze()?;
            }
            admitted._held_owner = Some(owner);
            Ok::<_, WorkflowError>(admitted)
        }
        .await;
        match result {
            Ok(mut admitted) => {
                admitted.held = held;
                Ok(admitted)
            }
            Err(error) => {
                close_horizon_sources(&held).await?;
                Err(error)
            }
        }
    }
    pub(super) async fn close_sources(&self) -> Result<(), WorkflowError> {
        close_horizon_sources(&self.held).await
    }
    async fn build(
        runtime: &Runtime,
        horizon: Horizon,
        cancel: &crate::CancelSource,
    ) -> Result<Self, WorkflowError> {
        let original = Arc::new(horizon.clone());
        let mut sources = vec![horizon.plant.source.clone()];
        if let Some(estimator) = &horizon.estimator {
            sources.push(estimator.package.clone());
        }
        if let Some(controller) = &horizon.controller {
            sources.push(controller.package.clone());
        }
        let Horizon {
            output,
            plant,
            period,
            steps,
            inputs,
            estimator,
            controller,
        } = horizon;
        output.validate(runtime)?;
        if steps == 0 || steps > 4096 || !period.is_finite() || period <= 0. {
            return Err(contract(
                "a horizon has 1 to 4096 steps of a positive, finite period",
            ));
        }
        if estimator.is_none() && controller.is_none() {
            return Err(contract("a horizon needs an estimator or a controller"));
        }
        if !Arc::ptr_eq(&plant.runtime.shared, &runtime.shared) {
            return Err(contract("the plant belongs to the horizon's runtime"));
        }
        let profile = plant.profile();
        let c = plant.contract();
        let np = c.parameters.len();
        let last = profile.start + steps as f64 * period;
        if !last.is_finite() || last > profile.end + 1e-9 * profile.end.abs().max(1.) {
            return Err(contract(
                "the plant's profile covers every period of the horizon",
            ));
        }
        let mut columns = Vec::with_capacity(inputs.len());
        for (k, input) in inputs.iter().enumerate() {
            let parameter = c
                .parameters
                .iter()
                .position(|p| *p == input.parameter)
                .ok_or_else(|| {
                    contract("a horizon input is a parameter of the plant's contract")
                })?;
            if profile.schedule.iter().any(|s| s.parameter == parameter)
                || inputs[..k].iter().any(|i| i.parameter == input.parameter)
                || !input.initial.is_finite()
            {
                return Err(contract(
                    "a horizon input is an unscheduled plant parameter, driven once, from a finite initial value",
                ));
            }
            columns.push(profile.columns_at(np, profile.start)[parameter]);
        }
        let output_index = |id: &SemanticId| {
            c.outputs.iter().position(|o| o == id).ok_or_else(|| {
                contract("a horizon measurement is an output of the plant's contract")
            })
        };
        let input = |i: usize| {
            (i < inputs.len())
                .then_some(i)
                .ok_or_else(|| contract("a horizon names a driven input it does not declare"))
        };
        // The controller's signals, resolved except for the estimates, which the
        // estimator's admission reads.
        let mut estimated: Vec<&String> = Vec::new();
        if let Some(controller) = &controller {
            for (_, signal) in &controller.bindings {
                match signal {
                    HorizonSignal::Measured(id) => {
                        output_index(id)?;
                    }
                    HorizonSignal::Estimated(path) if estimator.is_some() => estimated.push(path),
                    HorizonSignal::Estimated(_) => {
                        return Err(contract(
                            "a controller binds an estimate without an estimator",
                        ));
                    }
                    HorizonSignal::Applied(i) => {
                        input(*i)?;
                    }
                    HorizonSignal::Trajectory(values) => {
                        if values.len() < steps || values.iter().any(|v| !v.is_finite()) {
                            return Err(contract(
                                "a horizon trajectory has a finite value for every step",
                            ));
                        }
                    }
                }
            }
            let mut driven = BTreeSet::new();
            for (_, i) in &controller.moves {
                if !driven.insert(input(*i)?) {
                    return Err(contract("a controller moves each driven input once"));
                }
            }
        }
        let estimator = match &estimator {
            Some(e) => Some(
                Self::estimator(runtime, e, steps, &output_index, &input, &estimated, cancel)
                    .await?,
            ),
            None => None,
        };
        let controller = match &controller {
            Some(c) => {
                Some(Self::controller(runtime, c, &output_index, estimator.as_ref(), cancel).await?)
            }
            None => None,
        };
        // The plant runs on the controller's threads, or the estimator's without one.
        let stages = || {
            controller
                .iter()
                .map(|c| &c.stage)
                .chain(estimator.iter().map(|e| &e.stage))
        };
        let threads = stages()
            .map(|s| s.analysis.solver.controls.threads)
            .next()
            .unwrap_or(1);
        let history = stages()
            .map(|s| s.analysis.solver.controls.history)
            .max()
            .unwrap_or(0);
        let identity = identity(
            &plant,
            period,
            steps,
            &inputs,
            estimator.as_ref(),
            controller.as_ref(),
            &output,
        )?;
        let provenance = serde_json::to_vec(&(
            2_u8,
            &output,
            plant.identity(),
            plant.profile(),
            &plant.parameters,
            period.to_bits(),
            steps,
            inputs
                .iter()
                .map(|input| (input.parameter, input.initial.to_bits()))
                .collect::<Vec<_>>(),
            &c.outputs,
            estimator.as_ref().map(|stage| {
                (
                    &stage.stage.provenance,
                    stage.window,
                    &stage.measurements,
                    &stage.inputs,
                    &stage.arrival,
                )
            }),
            controller.as_ref().map(|stage| {
                (
                    &stage.stage.provenance,
                    &stage.bindings,
                    &stage.moves,
                    &stage.advanced,
                )
            }),
        ))
        .map_err(|error| contract(error.to_string()))?;
        let window = estimator.as_ref().map_or(0, |estimator| estimator.window);
        let history_bytes =
            (window + 4)
                .checked_mul(c.outputs.len() * size_of::<f64>() + size_of::<Vec<f64>>() + 128)
                .and_then(|bytes| {
                    bytes.checked_add((window + 3).checked_mul(
                        inputs.len() * size_of::<f64>() + size_of::<Vec<f64>>() + 128,
                    )?)
                })
                .and_then(|bytes| bytes.checked_add(16 * 1024))
                .ok_or_else(|| contract("horizon rolling state extent"))?;
        let history_owner = runtime
            .shared
            .math()
            .reserve("horizon:rolling-role-state", history_bytes)?;
        Ok(Self {
            original,
            output,
            _history_owner: history_owner,
            held: Vec::new(),
            _held_owner: None,
            sources,
            provenance,
            plant: Arc::new(Plant {
                worker_bytes: plant.bytes,
                experiment: IntegratedExperiment {
                    snapshot: plant.snapshot().clone(),
                    program: plant.program(),
                    snapshots: plant.snapshot_allocator(),
                    profile: profile.clone(),
                    parameters: plant.parameters.clone(),
                    output_ports: Vec::new(),
                    bindings: Vec::new(),
                },
                columns,
                differential: (0..c.states.len()).filter(|i| c.differential[*i]).collect(),
                start: profile.start,
                period,
                time_limit: profile.time_limit,
            }),
            steps,
            inputs: inputs.iter().map(|i| i.parameter).collect(),
            initial: inputs.iter().map(|i| i.initial).collect(),
            outputs: c.outputs.clone(),
            estimator,
            controller,
            identity,
            threads,
            history,
        })
    }
    async fn estimator(
        runtime: &Runtime,
        e: &HorizonEstimator,
        steps: usize,
        output: &impl Fn(&SemanticId) -> Result<usize, WorkflowError>,
        input: &impl Fn(usize) -> Result<usize, WorkflowError>,
        estimated: &[&String],
        cancel: &crate::CancelSource,
    ) -> Result<Estimator, WorkflowError> {
        if e.window == 0 || e.window >= steps {
            return Err(contract(
                "an estimation window spans at least one period and fewer than the horizon's steps",
            ));
        }
        let mut bound = Vec::new();
        let mut measurements = Vec::new();
        for (id, paths) in &e.measurements {
            let o = output(id)?;
            if paths.len() != e.window + 1 || measurements.iter().any(|(m, _)| *m == o) {
                return Err(contract(
                    "an estimator measures each output once, at every sample of its window",
                ));
            }
            bound.extend(paths);
            measurements.push((o, paths.clone()));
        }
        for (k, (i, how)) in e.inputs.iter().enumerate() {
            input(*i)?;
            if e.inputs[..k].iter().any(|(j, _)| j == i) {
                return Err(contract("an estimator takes each driven input once"));
            }
            match how {
                WindowInput::Constant(path) => bound.push(path),
                WindowInput::Periods(paths) if paths.len() == e.window => bound.extend(paths),
                WindowInput::Periods(_) => {
                    return Err(contract(
                        "an estimator takes a varying input at every period of its window",
                    ));
                }
            }
        }
        if e.arrival.iter().any(|a| !a.initial.is_finite()) {
            return Err(contract("an arrival cost starts from a finite prior"));
        }
        bound.extend(e.arrival.iter().map(|a| &a.prior));
        let read = e
            .arrival
            .iter()
            .map(|a| &a.next)
            .chain(estimated.iter().copied())
            .collect::<Vec<_>>();
        let stage = Stage::admit(runtime, &e.package, &e.analysis, &bound, &read, cancel).await?;
        let arrival = e
            .arrival
            .iter()
            .map(|a| (a.prior.clone(), stage.read[&a.next], a.initial))
            .collect();
        Ok(Estimator {
            stage,
            window: e.window,
            measurements,
            inputs: e.inputs.clone(),
            arrival,
        })
    }
    async fn controller(
        runtime: &Runtime,
        c: &HorizonController,
        output: &impl Fn(&SemanticId) -> Result<usize, WorkflowError>,
        estimator: Option<&Estimator>,
        cancel: &crate::CancelSource,
    ) -> Result<Controller, WorkflowError> {
        let bound = c.bindings.iter().map(|(path, _)| path).collect::<Vec<_>>();
        let mut read = c.moves.iter().map(|(path, _)| path).collect::<Vec<_>>();
        // An advanced step differentiates with respect to the measured and estimated
        // bindings, and reads the paths that predict them.
        let states = c
            .bindings
            .iter()
            .filter(|(_, signal)| {
                matches!(
                    signal,
                    HorizonSignal::Measured(_) | HorizonSignal::Estimated(_)
                )
            })
            .map(|(path, _)| path)
            .collect::<Vec<_>>();
        if let Some(advanced) = &c.advanced {
            if states.is_empty() || c.analysis.solver.sensitivity.is_some() {
                return Err(contract(
                    "an advanced-step controller binds a measured or estimated state and requests no sensitivity of its own",
                ));
            }
            let mut mapped = BTreeSet::new();
            for (state, predictor) in &advanced.predictions {
                if !states.contains(&state) || !mapped.insert(state) {
                    return Err(contract(format!(
                        "an advanced step predicts each measured or estimated binding once: {state}"
                    )));
                }
                read.push(predictor);
            }
            read.extend(states.iter().copied());
        }
        let mut stage =
            Stage::admit(runtime, &c.package, &c.analysis, &bound, &read, cancel).await?;
        let advanced = match &c.advanced {
            Some(a) => {
                let parameters = states
                    .iter()
                    .map(|path| ((*path).clone(), stage.read[*path]))
                    .collect::<Vec<_>>();
                // The sensitivity request prepares with the stage, which is refused here
                // when a state binding is not a declared parameter.
                stage.analysis.solver.sensitivity =
                    Some(crate::math::settings::SensitivityRequest {
                        parameters: parameters.iter().map(|(_, id)| *id).collect(),
                        reduced_hessian: false,
                        propagation: None,
                    });
                let prepared = c.package.prepare_analysis(&stage.analysis, cancel).await?;
                stage.identity = prepared
                    .solve
                    .request_identity()
                    .map_err(MathRuntimeError::from)?;
                Some(Advanced {
                    parameters,
                    predictions: a
                        .predictions
                        .iter()
                        .map(|(state, predictor)| (state.clone(), stage.read[predictor]))
                        .collect(),
                })
            }
            None => None,
        };
        let bindings = c
            .bindings
            .iter()
            .map(|(path, signal)| {
                let signal = match signal {
                    HorizonSignal::Measured(id) => Signal::Measured(output(id)?),
                    HorizonSignal::Estimated(p) => Signal::Estimated(
                        estimator
                            .and_then(|e| e.stage.read.get(p).copied())
                            .ok_or_else(|| {
                                contract("a controller binds an estimate without an estimator")
                            })?,
                    ),
                    HorizonSignal::Applied(i) => Signal::Applied(*i),
                    HorizonSignal::Trajectory(values) => Signal::Trajectory(values.clone()),
                };
                Ok((path.clone(), signal))
            })
            .collect::<Result<_, WorkflowError>>()?;
        let moves = c
            .moves
            .iter()
            .map(|(path, i)| (stage.read[path], *i))
            .collect();
        Ok(Controller {
            stage,
            bindings,
            moves,
            advanced,
        })
    }
}

/// The horizon's request identity: its plant, loop and prepared stage templates, framed so
/// no other request shares it.
fn identity(
    plant: &crate::workflow::ModelingSimulation,
    period: f64,
    steps: usize,
    inputs: &[super::HorizonInput],
    estimator: Option<&Estimator>,
    controller: Option<&Controller>,
    output: &crate::workflow::StudyOutputDemand,
) -> Result<pse_ids::roles::LineageRequestHash, WorkflowError> {
    let mut h = FramedHasher::new(pse_ids::Frame::DurableHorizonRequestV2);
    h.part(&serde_json::to_vec(output).map_err(|error| contract(error.to_string()))?);
    h.hash(&plant.identity())
        .u64(period.to_bits())
        .u64(steps as u64)
        .u64(inputs.len() as u64);
    for input in inputs {
        h.id(&input.parameter).u64(input.initial.to_bits());
    }
    match estimator {
        Some(e) => {
            h.bool(true)
                .hash(&e.stage.identity.as_id())
                .u64(e.window as u64)
                .u64(e.measurements.len() as u64);
            for (o, paths) in &e.measurements {
                h.u64(*o as u64).u64(paths.len() as u64);
                for path in paths {
                    h.str(path);
                }
            }
            h.u64(e.inputs.len() as u64);
            for (i, how) in &e.inputs {
                h.u64(*i as u64);
                match how {
                    WindowInput::Constant(path) => {
                        h.str("constant").str(path);
                    }
                    WindowInput::Periods(paths) => {
                        h.str("periods").u64(paths.len() as u64);
                        for path in paths {
                            h.str(path);
                        }
                    }
                }
            }
            h.u64(e.arrival.len() as u64);
            for (prior, next, initial) in &e.arrival {
                h.str(prior).id(next).u64(initial.to_bits());
            }
        }
        None => {
            h.bool(false);
        }
    }
    match controller {
        Some(c) => {
            h.bool(true)
                .hash(&c.stage.identity.as_id())
                .u64(c.bindings.len() as u64);
            for (path, signal) in &c.bindings {
                h.str(path);
                match signal {
                    Signal::Measured(o) => {
                        h.str("measured").u64(*o as u64);
                    }
                    Signal::Estimated(id) => {
                        h.str("estimated").id(id);
                    }
                    Signal::Applied(i) => {
                        h.str("applied").u64(*i as u64);
                    }
                    Signal::Trajectory(values) => {
                        h.str("trajectory").u64(values.len() as u64);
                        for v in values {
                            h.u64(v.to_bits());
                        }
                    }
                }
            }
            h.u64(c.moves.len() as u64);
            for (id, i) in &c.moves {
                h.id(id).u64(*i as u64);
            }
            match &c.advanced {
                Some(a) => {
                    h.bool(true).u64(a.parameters.len() as u64);
                    for (path, id) in &a.parameters {
                        h.str(path).id(id);
                    }
                    h.u64(a.predictions.len() as u64);
                    for (path, id) in &a.predictions {
                        h.str(path).id(id);
                    }
                }
                None => {
                    h.bool(false);
                }
            }
        }
        None => {
            h.bool(false);
        }
    }
    Ok(pse_ids::roles::LineageRequestHash::from_id(h.finish_hash()))
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Role {
    Estimator,
    Controller,
}

/// The value a solution holds at `id`.
fn solved(result: &ModelingResult, id: &SemanticId) -> Result<f64, WorkflowError> {
    result
        .values
        .scalars
        .get(id)
        .copied()
        .filter(|v| v.is_finite())
        .ok_or_else(|| contract("a horizon path has no finite solved value"))
}

/// One horizon run in progress.
pub(super) struct Loop {
    admitted: Arc<Admitted>,
    staged: Staged,
    durable: Option<DurableAttempt>,
    run_id: RunId,
    cancel: crate::CancelSource,
    progress: Arc<Progress>,
    started: Instant,
    /// Explicit Full history only. Role results below are keyed by original
    /// absolute ordinal and retain no obsolete native/preparation owners.
    requests: Vec<ModelingSolvePreparation>,
    results: BTreeMap<usize, ModelingResult>,
    full_results: Vec<ModelingResult>,
    full_owners: Vec<Arc<pse_columnar::AllocationLease>>,
    original: super::prefix::OriginalHistory,
    next_attempt: usize,
    completed_samples: usize,
    /// An actually attempted solve that returned an error; absent for plant/task failure.
    failed_solve: Option<usize>,
    publication_failure: Option<Arc<WorkflowError>>,
    unmerged: Option<Box<RunResult>>,
    capture_failure: Option<Arc<WorkflowError>>,
    #[cfg(all(test, feature = "solver-ipopt", feature = "solver-diffsol"))]
    pressure_next_export: bool,
    #[cfg(all(test, feature = "solver-ipopt", feature = "solver-diffsol"))]
    original_before_refusal: Option<serde_json::Value>,
    steps: Vec<HorizonStep>,
    /// Plant outputs at every sample so far.
    measured: VecDeque<Vec<f64>>,
    /// Inputs applied over every completed period.
    history: VecDeque<Vec<f64>>,
    applied: Vec<f64>,
    /// The differential states at the current sample, in the plant's normalized
    /// coordinates.
    anchors: Vec<f64>,
    /// The horizon's original provider cancellation/deadline owner. Each plant IVP
    /// has fresh local controls without renewing this enclosing scope.
    plant_scope: pse_kernels::ExecutionScope,
    priors: Vec<f64>,
    /// The last accepted step of each role, which seeds the next one.
    accepted: [Option<usize>; 2],
    /// This sample's accepted estimator step.
    estimate: Option<usize>,
    /// An advanced-step controller's background solve for the next sample, whose factor the
    /// session retains.
    background: Option<usize>,
}

/// What decided a controller's moves at a sample.
enum Source {
    Solved(usize),
    Predicted(Prediction),
}

/// A sample's controller decision.
struct Control {
    controller: Option<usize>,
    decision: HorizonDecision,
    fallback: Option<Fallback>,
    activity: Option<native::kkt::path::PathPrediction>,
    advanced: Option<usize>,
}
impl Control {
    const OPEN: Self = Self {
        controller: None,
        decision: HorizonDecision::OpenLoop,
        fallback: None,
        activity: None,
        advanced: None,
    };
}
impl Loop {
    pub(super) fn new(
        admitted: Arc<Admitted>,
        staged: Staged,
        run_id: RunId,
        cancel: crate::CancelSource,
        progress: Arc<Progress>,
        durable: Option<DurableAttempt>,
        original: super::prefix::OriginalHistory,
    ) -> Self {
        let priors = admitted
            .estimator
            .iter()
            .flat_map(|e| e.arrival.iter().map(|(_, _, initial)| *initial))
            .collect();
        let plant_scope =
            pse_kernels::ExecutionScope::new(Arc::new(AtomicBool::new(false)), cancel.deadline())
                .with_admitted_threads(
                    std::num::NonZeroUsize::new(admitted.threads)
                        .unwrap_or(std::num::NonZeroUsize::MIN),
                );
        Self {
            plant_scope,
            applied: admitted.initial.clone(),
            admitted,
            staged,
            durable,
            run_id,
            cancel,
            progress,
            started: Instant::now(),
            requests: Vec::new(),
            results: BTreeMap::new(),
            full_results: Vec::new(),
            full_owners: Vec::new(),
            original,
            next_attempt: 0,
            completed_samples: 0,
            failed_solve: None,
            publication_failure: None,
            unmerged: None,
            capture_failure: None,
            #[cfg(all(test, feature = "solver-ipopt", feature = "solver-diffsol"))]
            pressure_next_export: false,
            #[cfg(all(test, feature = "solver-ipopt", feature = "solver-diffsol"))]
            original_before_refusal: None,
            steps: Vec::new(),
            measured: VecDeque::new(),
            history: VecDeque::new(),
            anchors: Vec::new(),
            priors,
            accepted: [None; 2],
            estimate: None,
            background: None,
        }
    }
    pub(super) async fn finish(
        self,
        runtime: Runtime,
        outcome: Result<(), WorkflowError>,
        cancelled: bool,
    ) -> RunResult {
        let Loop {
            admitted,
            staged,
            durable,
            run_id,
            requests,
            full_results,
            full_owners,
            original,
            publication_failure,
            unmerged,
            capture_failure,
            failed_solve,
            steps,
            next_attempt,
            completed_samples,
            ..
        } = self;
        staged.close().await;
        let usable = (outcome.is_ok() || publication_failure.is_some())
            && !original.completion.assessments.is_empty()
            && original
                .completion
                .assessments
                .iter()
                .all(|assessment| assessment.permits_result);
        let final_error = if failed_solve.is_none() && publication_failure.is_none() {
            outcome.as_ref().err()
        } else {
            None
        };
        let (
            completion,
            detail,
            summaries,
            mut owners,
            tightenings,
            summary_error,
            detail_membership,
        ) = original.finish(usable, final_error);
        let mut result = if matches!(admitted.output, crate::workflow::StudyOutputDemand::Full {}) {
            let (report, failure) = joined_modeling(outcome, full_results, failed_solve);
            let mut result = RunResult::joined(
                run_id,
                runtime,
                RunRequest::Modeling(requests),
                None,
                report,
            );
            result.modeling_failure = failure;
            result.original_horizon_request = Some(RunRequest::Horizon(admitted.request()));
            result
        } else {
            let report = Arc::new(super::HorizonOutcome {
                solves: next_attempt,
                summaries,
                detail,
                summary_error,
                detail_membership,
            });
            let original = outcome
                .map(|()| RunReport::Horizon(report.clone()))
                .map_err(Arc::new);
            let mut result = RunResult::joined(
                run_id,
                runtime,
                RunRequest::Horizon(admitted.request()),
                None,
                original,
            );
            result.horizon_output = Some(report);
            result
        };
        owners.extend(full_owners);
        owners.push(admitted._history_owner.clone());
        result.assessments = completion.assessments.clone();
        result.completion = match capture_failure {
            Some(error) => Err(error),
            None => Ok(completion),
        };
        result.original_completion_captured = result.completion.is_ok();
        result.horizon_unmerged = unmerged;
        result.horizon_owners = owners;
        result.horizon_tightenings = Some(tightenings);
        result.horizon = Some(Arc::new(HorizonReport {
            outputs: admitted.outputs.clone(),
            inputs: admitted.inputs.clone(),
            steps,
            completed_steps: completed_samples,
        }));
        result.finished(durable, cancelled).await
    }
    pub(super) async fn run(&mut self) -> Result<(), WorkflowError> {
        let observation = pse_diagnostics::observation::current();
        let _phase = observation
            .as_ref()
            .map(|context| context.phase("horizon.run"));
        if self.cancel.token().is_cancelled() {
            return Err(MathRuntimeError::Cancelled.into());
        }
        let applied = self.applied.clone();
        let start = self
            .plant(move |plant, worker, execution| plant.initial(worker, &applied, execution))
            .await?;
        self.anchor(start);
        for k in 0..self.admitted.steps {
            let _sample = observation.as_ref().map(|context| {
                context.count("horizon.sample_attempts", "count", 1);
                context.gauge("horizon.sample_index", "index", k as f64);
                context.phase("horizon.sample")
            });
            if self.cancel.token().is_cancelled() {
                return Err(MathRuntimeError::Cancelled.into());
            }
            let estimator = self.estimate(k).await?;
            let control = self.control(k).await?;
            let applied = self.applied.clone();
            let anchors = self.anchors.clone();
            let end = self
                .plant(move |plant, worker, execution| {
                    plant.period(worker, k, &applied, &anchors, execution)
                })
                .await?;
            if matches!(
                self.admitted.output,
                crate::workflow::StudyOutputDemand::Full {}
            ) {
                let owner = match super::prefix::step_copy_owner(
                    &self.admitted.sources[0].runtime,
                    self.measured
                        .back()
                        .ok_or_else(|| contract("horizon current measurement absent"))?,
                    &self.applied,
                    &end.outputs,
                    control.activity.as_ref(),
                ) {
                    Ok(owner) => owner,
                    Err(error) => return Err(self.refuse_publication(error)),
                };
                self.full_owners.push(owner);
            } else {
                self.steps.clear();
            }
            let measured = self
                .measured
                .back()
                .ok_or_else(|| contract("horizon current measurement absent"))?
                .clone();
            self.history.push_back(self.applied.clone());
            self.anchor(end);
            let step = HorizonStep {
                time: self.admitted.plant.time(k),
                measured,
                estimator,
                controller: control.controller,
                decision: control.decision,
                fallback: control.fallback,
                activity: control.activity,
                advanced: control.advanced,
                applied: self.applied.clone(),
                reached: self
                    .measured
                    .back()
                    .ok_or_else(|| contract("horizon reached measurement absent"))?
                    .clone(),
            };
            super::observe_step(k, &step);
            self.progress.push(event(k, &step, self.started.elapsed()));
            self.steps.push(step);
            self.completed_samples += 1;
            let window = self
                .admitted
                .estimator
                .as_ref()
                .map_or(0, |estimator| estimator.window);
            while self.history.len() > window {
                self.history.pop_front();
            }
            let needed = [
                self.accepted[0],
                self.accepted[1],
                self.estimate,
                self.background,
            ];
            self.results
                .retain(|ordinal, _| needed.contains(&Some(*ordinal)));
        }
        Ok(())
    }
    /// Take a plant sample: its outputs are the next measurement, its differential states
    /// the next period's anchored start.
    fn anchor(&mut self, sample: native::dynamics::Sample) {
        self.anchors = self
            .admitted
            .plant
            .differential
            .iter()
            .map(|i| sample.state[*i])
            .collect();
        self.measured.push_back(sample.outputs);
        let window = self
            .admitted
            .estimator
            .as_ref()
            .map_or(0, |estimator| estimator.window);
        while self.measured.len() > window + 1 {
            self.measured.pop_front();
        }
    }
    /// Plant work on the session thread beside the solves' retained native state.
    async fn plant<T: Send + 'static>(
        &self,
        work: impl FnOnce(
            &Plant,
            &mut crate::workflow::dynamics::DynamicWorker,
            &native::solve::Execution,
        ) -> Result<T, ProblemError>
        + Send
        + 'static,
    ) -> Result<T, WorkflowError> {
        let _phase = pse_diagnostics::observation::phase("horizon.plant");
        let plant = self.admitted.plant.clone();
        let scope = self.plant_scope.clone();
        Ok(self
            .staged
            .native_in_task(
                self.admitted.threads,
                scope.clone(),
                &self.cancel,
                move |retained, _, _| {
                    let execution = plant.execution(scope.clone())?;
                    let (owner, reused) = retained.workflow_owner(|| {
                        let charge = plant.experiment.snapshots.reserve(plant.worker_bytes)?;
                        let inner = plant.experiment.worker_scoped(scope)?;
                        Ok(PlantWorker {
                            inner,
                            _charge: charge,
                        })
                    })?;
                    if let Some(context) = pse_diagnostics::observation::current() {
                        context.count(
                            if reused {
                                "horizon.plant_worker_reuses"
                            } else {
                                "horizon.plant_worker_constructions"
                            },
                            "count",
                            1,
                        );
                    }
                    work(&plant, &mut owner.inner, &execution).map_err(MathRuntimeError::from)
                },
            )
            .await?)
    }
    /// One step of `role`: its overlay of `values` over its specification. Only
    /// `PreviousAccepted` inherits the last accepted role's coordinate/native seed (N2).
    /// The task runs as the next modeling step.
    /// A step that keeps its sensitivity factor for an advanced-step prediction `retain`s it.
    async fn solve(
        &mut self,
        role: Role,
        values: BTreeMap<String, f64>,
        retain: bool,
    ) -> Result<usize, WorkflowError> {
        let prepared = self.prepare_solve(role, values, retain).await?;
        self.execute_solve(role, prepared).await
    }
    pub(super) async fn prepare_solve(
        &mut self,
        role: Role,
        values: BTreeMap<String, f64>,
        retain: bool,
    ) -> Result<ModelingSolvePreparation, WorkflowError> {
        let admitted = self.admitted.clone();
        let stage = match role {
            Role::Estimator => admitted.estimator.as_ref().map(|e| &e.stage),
            Role::Controller => admitted.controller.as_ref().map(|c| &c.stage),
        }
        .ok_or_else(|| contract("a horizon step of an undeclared stage"))?;
        let last = self.accepted[role as usize];
        let seed = if stage.analysis.solver.controls.start
            == native::solve::StartPolicy::PreviousAccepted
        {
            last.and_then(|i| self.results.get(&i))
                .filter(|result| result.completion.decision.permits_use())
                .map(|result| result.values.scalars.clone())
                .unwrap_or_default()
        } else {
            BTreeMap::new()
        };
        let specification = Overlay {
            values,
            ..Overlay::default()
        }
        .compose(&stage.analysis);
        let mut prepared = self
            .staged
            .prepare(stage.package.prepare_analysis_attempt(
                &specification,
                crate::workflow::modeling::cases::CaseOverrides {
                    seed,
                    ..Default::default()
                },
                &self.cancel,
            ))
            .await??;
        if retain {
            prepared.solve = prepared
                .solve
                .retaining_factor()
                .map_err(WorkflowError::from)?;
        }
        Ok(prepared)
    }
    pub(super) async fn execute_solve(
        &mut self,
        role: Role,
        prepared: ModelingSolvePreparation,
    ) -> Result<usize, WorkflowError> {
        let previous = self.accepted[role as usize].and_then(|attempt| {
            self.results
                .get(&attempt)
                .filter(|result| result.completion.decision.permits_use())
                .and_then(|result| match &result.outcome {
                    crate::math::solves::Outcome::Native(report) => report.warm_start.clone(),
                    _ => None,
                })
                .map(|seed| crate::math::solves::Predecessor { attempt, seed })
        });
        let copy_owners = match super::prefix::preparation_copy_owner(
            &self.admitted.sources[0].runtime,
            &prepared,
        ) {
            Ok(owners) => owners,
            Err(error) => return Err(self.refuse_publication(error)),
        };
        let full_owners = if matches!(
            self.admitted.output,
            crate::workflow::StudyOutputDemand::Full {}
        ) {
            match super::prefix::preparation_copy_owner(
                &self.admitted.sources[0].runtime,
                &prepared,
            ) {
                Ok(owners) => owners,
                Err(error) => return Err(self.refuse_publication(error)),
            }
        } else {
            Vec::new()
        };
        if let Err(error) = self
            .original
            .admit_step(&self.admitted.sources[0].runtime, &prepared)
        {
            return Err(self.refuse_publication(error));
        }
        let attempt = self.next_attempt;
        self.next_attempt += 1;
        if let Some(durable) = &self.durable {
            durable.set_step(attempt);
        }
        let request = prepared.clone();
        if matches!(
            self.admitted.output,
            crate::workflow::StudyOutputDemand::Full {}
        ) {
            self.full_owners.extend(full_owners);
            self.requests.push(prepared.clone());
        }
        let result = match self
            .staged
            .run_unrecorded(
                prepared,
                Obligations::Final,
                self.run_id,
                attempt,
                previous,
                &self.cancel,
            )
            .await
        {
            Ok(result) => result,
            Err(error) => {
                self.failed_solve = Some(attempt);
                let mut original = RunResult::joined(
                    self.run_id,
                    self.admitted.sources[0].runtime.clone(),
                    RunRequest::Modeling(vec![request]),
                    None,
                    Err(error.clone()),
                );
                original.modeling_origin = attempt;
                original = original.completed();
                original.horizon_owners = copy_owners;
                self.append_original(original).await?;
                return Err(WorkflowError::Shared(error));
            }
        };
        if result.accepted {
            self.accepted[role as usize] = Some(attempt);
        }
        let mut original = RunResult::joined(
            self.run_id,
            self.admitted.sources[0].runtime.clone(),
            RunRequest::Modeling(vec![request]),
            None,
            Ok(RunReport::Modeling(vec![result.clone()])),
        );
        original.modeling_origin = attempt;
        original = original.completed();
        original.horizon_owners = copy_owners;
        if matches!(
            self.admitted.output,
            crate::workflow::StudyOutputDemand::Full {}
        ) {
            self.full_results.push(result.clone());
        }
        self.results.insert(attempt, result);
        self.append_original(original).await?;
        Ok(attempt)
    }
    fn refuse_publication(&mut self, error: WorkflowError) -> WorkflowError {
        let error = Arc::new(error);
        if let Some(durable) = &mut self.durable {
            durable.refuse_horizon_prefix(error.clone());
        }
        if self.publication_failure.is_none() {
            self.publication_failure = Some(error.clone());
        }
        WorkflowError::Shared(error)
    }
    async fn append_original(&mut self, mut original: RunResult) -> Result<(), WorkflowError> {
        if let Err(error) = self.original.validate_original(&original) {
            let error = Arc::new(error);
            self.capture_failure = Some(error.clone());
            self.unmerged = Some(Box::new(original));
            return Err(self.refuse_publication(WorkflowError::Shared(error)));
        }
        #[cfg(all(test, feature = "solver-ipopt", feature = "solver-diffsol"))]
        let pressure = if std::mem::take(&mut self.pressure_next_export) {
            self.original_before_refusal = Some(
                serde_json::to_value(
                    original
                        .completion
                        .as_ref()
                        .map_err(|error| WorkflowError::Shared(error.clone()))?,
                )
                .map_err(|error| contract(error.to_string()))?,
            );
            let runtime = &self.admitted.sources[0].runtime;
            let pool = runtime.shared.pool();
            let pressure =
                pse_columnar::MemoryConsumer::new("test:horizon-post-science-publication-pressure")
                    .register(&pool);
            pressure
                .try_grow(runtime.shared.budget().memory_limit_bytes.get() - pool.reserved())
                .map_err(pse_engine::EngineError::from)?;
            Some(pressure)
        } else {
            None
        };
        let publication = async {
            self.original.export(&original).await?;
            if let Some(durable) = &mut self.durable {
                durable.publish_horizon_step(&original).await?;
            }
            Ok::<_, WorkflowError>(())
        }
        .await;
        // Even an allocation/encoding/append refusal transfers the latest actual
        // scientific rows. All aggregate capacity and owners predate dispatch.
        self.original.take_original(&mut original);
        #[cfg(all(test, feature = "solver-ipopt", feature = "solver-diffsol"))]
        drop(pressure);
        if let Err(error) = publication {
            return Err(self.refuse_publication(error));
        }
        Ok(())
    }
    fn result(&self, ordinal: usize) -> Result<&ModelingResult, WorkflowError> {
        self.results
            .get(&ordinal)
            .ok_or_else(|| contract("horizon role result is no longer retained"))
    }
    #[cfg(all(test, feature = "solver-ipopt", feature = "solver-diffsol"))]
    pub(super) fn refuse_next_export(&mut self) {
        self.pressure_next_export = true;
    }
    #[cfg(all(test, feature = "solver-ipopt", feature = "solver-diffsol"))]
    pub(super) fn before_export_refusal(&self) -> Option<&serde_json::Value> {
        self.original_before_refusal.as_ref()
    }
    #[cfg(all(test, feature = "canonical-tests", feature = "solver-ipopt"))]
    pub(super) fn cancel_before_first_step(&self) {
        self.cancel.cancel();
    }
    #[cfg(all(test, feature = "solver-ipopt", feature = "solver-diffsol"))]
    pub(super) fn retained_counts(&self) -> (usize, usize, usize, usize, usize) {
        (
            self.results.len(),
            self.requests.len(),
            self.full_results.len(),
            self.measured.len(),
            self.history.len(),
        )
    }
    #[cfg(all(test, feature = "solver-ipopt", feature = "solver-diffsol"))]
    pub(super) fn accepted_role(&self, role: Role) -> Option<usize> {
        self.accepted[role as usize]
    }
    #[cfg(all(test, feature = "solver-ipopt", feature = "solver-diffsol"))]
    pub(super) fn last_sample(&self) -> Option<&HorizonStep> {
        self.steps.last()
    }
    #[cfg(all(test, feature = "solver-ipopt", feature = "solver-diffsol"))]
    pub(super) fn retained_result(&self, ordinal: usize) -> Option<&ModelingResult> {
        self.results.get(&ordinal)
    }
    #[cfg(all(test, feature = "solver-ipopt", feature = "solver-diffsol"))]
    pub(super) async fn close(mut self) -> Result<(), WorkflowError> {
        self.staged.close().await;
        if let Some(attempt) = self.durable.take() {
            let record = attempt
                .abandon(&Arc::new(contract(
                    "horizon control closed before finalization",
                )))
                .await;
            record.attempt.map_err(WorkflowError::Shared)?;
        }
        self.admitted.close_sources().await
    }
    #[cfg(all(test, feature = "solver-ipopt", feature = "solver-diffsol"))]
    pub(super) fn completion(&self) -> &crate::workflow::Completion {
        &self.original.completion
    }
    #[cfg(all(test, feature = "solver-ipopt", feature = "solver-diffsol"))]
    pub(super) fn retained_sample_count(&self) -> usize {
        self.steps.len()
    }
    /// The estimator step at sample `k`, once its window is measured: the window's
    /// measurements, the inputs applied over it and the arrival priors. An accepted estimate
    /// sets the next priors.
    async fn estimate(&mut self, k: usize) -> Result<Option<usize>, WorkflowError> {
        let _phase = pse_diagnostics::observation::phase("horizon.estimate");
        self.estimate = None;
        let admitted = self.admitted.clone();
        let Some(e) = &admitted.estimator else {
            return Ok(None);
        };
        let Some(_) = k.checked_sub(e.window) else {
            return Ok(None);
        };
        let mut values = BTreeMap::new();
        for (o, paths) in &e.measurements {
            for (sample, path) in self.measured.iter().zip(paths) {
                values.insert(path.clone(), sample[*o]);
            }
        }
        let periods = &self.history;
        for (i, how) in &e.inputs {
            match how {
                WindowInput::Constant(path) => {
                    let value = periods
                        .front()
                        .ok_or_else(|| contract("estimator input window absent"))?[*i];
                    if periods.iter().any(|p| p[*i].to_bits() != value.to_bits()) {
                        return Err(contract(format!(
                            "the estimator holds input {i} constant over its window, but it varied"
                        )));
                    }
                    values.insert(path.clone(), value);
                }
                WindowInput::Periods(paths) => {
                    for (period, path) in periods.iter().zip(paths) {
                        values.insert(path.clone(), period[*i]);
                    }
                }
            }
        }
        for ((prior, _, _), value) in e.arrival.iter().zip(&self.priors) {
            values.insert(prior.clone(), *value);
        }
        let index = self.solve(Role::Estimator, values, false).await?;
        let result = self.result(index)?;
        if result.accepted {
            let next = e
                .arrival
                .iter()
                .map(|(_, id, _)| solved(result, id))
                .collect::<Result<Vec<_>, _>>()?;
            self.priors = next;
            self.estimate = Some(index);
        }
        Ok(Some(index))
    }
    /// The controller's bound values at sample `k`: its state, setpoints and held moves;
    /// `None` while an estimate it binds does not exist.
    fn bound(
        &self,
        c: &Controller,
        k: usize,
    ) -> Result<Option<BTreeMap<String, f64>>, WorkflowError> {
        let mut values = BTreeMap::new();
        for (path, signal) in &c.bindings {
            let value = match signal {
                Signal::Measured(o) => self
                    .measured
                    .back()
                    .ok_or_else(|| contract("horizon current measurement absent"))?[*o],
                Signal::Estimated(id) => match self.estimate {
                    Some(i) => solved(self.result(i)?, id)?,
                    None => return Ok(None),
                },
                Signal::Applied(i) => self.applied[*i],
                Signal::Trajectory(values) => values[k],
            };
            values.insert(path.clone(), value);
        }
        Ok(Some(values))
    }
    /// Apply the first moves of the accepted step `index`.
    fn apply_solved(&mut self, c: &Controller, index: usize) -> Result<(), WorkflowError> {
        let result = self.result(index)?;
        let moves = c
            .moves
            .iter()
            .map(|(id, _)| Ok((*id, solved(result, id)?)))
            .collect::<Result<BTreeMap<_, _>, WorkflowError>>()?;
        self.apply(c, |id| Ok(moves[id]))
    }
    /// Apply the first moves `value` reads.
    fn apply(
        &mut self,
        c: &Controller,
        value: impl Fn(&SemanticId) -> Result<f64, WorkflowError>,
    ) -> Result<(), WorkflowError> {
        let moves = c
            .moves
            .iter()
            .map(|(id, i)| Ok((*i, value(id)?)))
            .collect::<Result<Vec<_>, WorkflowError>>()?;
        for (i, value) in moves {
            self.applied[i] = value;
        }
        Ok(())
    }
    /// The controller step at sample `k`: its state, setpoints and held moves bound, and its
    /// first moves applied when it is accepted. An advanced-step controller predicts them
    /// from the previous sample's background solve, or falls back to a full solve, and then
    /// solves in the background for the next sample.
    async fn control(&mut self, k: usize) -> Result<Control, WorkflowError> {
        let _phase = pse_diagnostics::observation::phase("horizon.control");
        let admitted = self.admitted.clone();
        let Some(c) = &admitted.controller else {
            return Ok(Control::OPEN);
        };
        let Some(values) = self.bound(c, k)? else {
            return Ok(Control::OPEN);
        };
        let Some(advanced) = &c.advanced else {
            let index = self.solve(Role::Controller, values, false).await?;
            let decision = if self.result(index)?.accepted {
                self.apply_solved(c, index)?;
                HorizonDecision::Solved
            } else {
                HorizonDecision::Held
            };
            return Ok(Control {
                controller: Some(index),
                decision,
                ..Control::OPEN
            });
        };
        let mut prepared = self
            .prepare_solve(Role::Controller, values.clone(), false)
            .await?;
        let deadline = Instant::now()
            .checked_add(prepared.solve.time_limit())
            .ok_or_else(|| contract("advanced controller task deadline extent"))?;
        let proposal_scope = prepared
            .solve
            .task_scope()
            .unwrap_or_else(|| pse_kernels::ExecutionScope::new(Arc::default(), Some(deadline)));
        prepared.solve = c
            .stage
            .package
            .runtime
            .native()
            .admit_proposal_task(prepared.solve, proposal_scope.clone())
            .map_err(MathRuntimeError::from)?;
        let proposal_target = prepared.solve.clone();
        // One backsolve against the background solve's retained factor at the actual state.
        // A factor predicts once: the next background solve replaces it.
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("horizon.prediction.requests", "count", 1);
        }
        let predicted = crate::observation::child("horizon.prediction", async {
            if let Some(context) = pse_diagnostics::observation::current() {
                context.integer("horizon.sample_index", "index", k as i64);
                context.text("horizon.role", "prediction");
            }
            Ok::<_, WorkflowError>(match self.background.take() {
                None => None,
                Some(from) => {
                    let background = self.result(from)?;
                    let permission = background.completion.decision.clone();
                    if let Some(context) = pse_diagnostics::observation::current() {
                        context.integer(
                            "horizon.prediction.background_ordinal",
                            "index",
                            from as i64,
                        );
                        if let crate::math::solves::Outcome::Native(report) = &background.outcome {
                            match report.evidence.sensitivity.as_ref() {
                                Some(analysis) => {
                                    context.boolean(
                                        "horizon.prediction.background_factor",
                                        analysis.retained.is_some(),
                                    );
                                    match &analysis.sensitivities {
                                        Ok(_) => context
                                            .text("horizon.prediction.background_withheld", "none"),
                                        Err(cause) => context.text(
                                            "horizon.prediction.background_withheld",
                                            &cause.to_string(),
                                        ),
                                    }
                                }
                                None => context.unavailable(
                                    "horizon.prediction.background_factor",
                                    "background has no sensitivity analysis",
                                ),
                            }
                        }
                    }
                    let source = match &background.outcome {
                        crate::math::solves::Outcome::Native(report) => report
                            .candidate
                            .as_ref()
                            .map(|candidate| {
                                background
                                    .prepared
                                    .solve
                                    .semantic_point_key(&candidate.primal)
                            })
                            .transpose()
                            .map_err(MathRuntimeError::from)?,
                        _ => None,
                    };
                    let base = background.prepared.solve.clone();
                    let target = proposal_target.clone();
                    let parameters = advanced
                        .parameters
                        .iter()
                        .map(|(path, id)| (*id, values[path]))
                        .collect::<Vec<_>>();
                    let outcome =
                        self.staged
                            .native_in_task(
                                admitted.threads,
                                proposal_scope.clone(),
                                &self.cancel,
                                move |retained, flag, _| {
                                    let advance = retained.advance();
                                    if let Some(context) = pse_diagnostics::observation::current() {
                                        context.boolean(
                                            "horizon.prediction.factor_retained",
                                            advance.is_some(),
                                        );
                                        context.boolean(
                                            "horizon.prediction.source_candidate",
                                            source.is_some(),
                                        );
                                        context.boolean(
                                            "horizon.prediction.original_permission",
                                            permission.permits_use(),
                                        );
                                        context.boolean(
                                            "horizon.prediction.explicit_start",
                                            target.numerical_strategy().start.policy
                                                == native::solve::StartPolicy::Explicit
                                                || target.entry_origin(false)
                                                    == pse_model::strategy::StartOrigin::Explicit,
                                        );
                                    }
                                    let outcome = match (advance, source) {
                            (Some(advance), Some(source))
                                if permission.permits_use()
                                    && target.numerical_strategy().start.policy
                                        != native::solve::StartPolicy::Explicit
                                    && target.entry_origin(false)
                                        != pse_model::strategy::StartOrigin::Explicit =>
                            {
                                target.related_target_parameters(&base, advance.parameters())?;
                                let binding = target.original_identity()?;
                                let mut execution = native::solve::Execution::within(
                                    flag.clone(), target.controls(), proposal_scope.clone(),
                                )?;
                                execution.work_admission=target.task_admission()
                                    .map(|owner| -> Arc<dyn native::solve::WorkAdmission> {owner});
                                let segments = advance
                                    .variables()
                                    .len()
                                    .saturating_add(advance.rows().len())
                                    .saturating_add(1);
                                let limits = native::kkt::activity::Limits {
                                    backsolves: segments.saturating_mul(8),
                                    refactorizations: segments.saturating_mul(2),
                                    bytes: advance.bytes().saturating_mul(4),
                                };
                                // The original target can still have a minimal direct
                                // declaration here; its authored recovery origins live
                                // in the composition request until the operation binds.
                                let starts = pse_model::strategy::StartRules {
                                    policy: target.controls().start,
                                    recovery: target.composition_request().recovery.clone(),
                                };
                                let activity = starts.recovery
                                    .contains(&pse_model::strategy::StartOrigin::Predicted)
                                    && starts.permits_entry(
                                        pse_model::strategy::StartOrigin::Predicted, false);
                                match crate::math::prediction::select(
                                    crate::math::prediction::SelectionRequest {
                                        mechanism:
                                            crate::math::prediction::ProposalMechanism::Kkt {
                                                advance,
                                                parameters: &parameters,
                                                activity: activity.then_some((segments, limits)),
                                            },
                                        permission: &permission,
                                        source,
                                        target: binding,
                                        branch: target.composition_request().branch,
                                    },
                                    &execution,
                                ) {
                                    Ok(crate::math::prediction::SelectedProposal::Kkt {
                                        prediction,
                                        ..
                                    }) => Ok((prediction, None, execution.scope()?)),
                                    Ok(crate::math::prediction::SelectedProposal::Activity {
                                        proposal,
                                        path,
                                        fallback,
                                    }) => {
                                        // A tracked endpoint still receives independent original correction.
                                        // The original activity refusal remains visible in the horizon decision.
                                        let fallback = fallback.ok_or_else(|| {
                                            ProblemError::internal(
                                                "activity selection lacked its original refusal",
                                            )
                                        })?;
                                        Ok((
                                            path.prediction.clone(),
                                            Some((*proposal, path, fallback)),
                                            execution.scope()?,
                                        ))
                                    }
                                    Ok(_) => {
                                        return Err(ProblemError::internal(
                                            "horizon selector returned another producer",
                                        )
                                        .into());
                                    }
                                    Err(crate::math::prediction::SelectionFailure::Kkt(
                                        fallback,
                                    )) => Err(fallback),
                                    Err(error) => return Err(error.into_problem().into()),
                                }
                            }
                            _ => Err(Fallback::NotRetained),
                        };
                                    retained.release();
                                    Ok(outcome)
                                },
                            )
                            .await?;
                    Some(outcome.map(|prediction| (from, prediction)))
                }
            })
        })
        .await?;
        let (mut control, source) = match predicted {
            Some(Ok((_, (_, Some((proposal, path, fallback)), scope)))) => {
                let screened = admitted
                    .controller
                    .as_ref()
                    .ok_or_else(|| contract("activity controller missing"))?
                    .stage
                    .package
                    .runtime
                    .native()
                    .screen_start(
                        prepared.solve.clone(),
                        proposal,
                        prepared.solve.composition_request().branch,
                        scope.clone(),
                        &self.cancel,
                    )
                    .await?;
                prepared.solve = prepared
                    .solve
                    .within_task(scope)
                    .map_err(MathRuntimeError::from)?
                    .with_screened_start(&screened)
                    .map_err(MathRuntimeError::from)?;
                let index = self
                    .execute_solve(Role::Controller, prepared.clone())
                    .await?;
                let accepted = self.result(index)?.accepted;
                if accepted {
                    self.apply_solved(c, index)?;
                }
                (
                    Control {
                        controller: Some(index),
                        decision: if accepted {
                            HorizonDecision::Fallback
                        } else {
                            HorizonDecision::Held
                        },
                        fallback: Some(fallback),
                        activity: Some(path),
                        advanced: None,
                    },
                    accepted.then_some(Source::Solved(index)),
                )
            }
            Some(Ok((from, (prediction, None, _)))) => {
                self.apply(c, |id| {
                    prediction.value(id).ok_or_else(|| {
                        contract("an advanced-step controller's moves are columns of its solve")
                    })
                })?;
                (
                    Control {
                        controller: Some(from),
                        decision: HorizonDecision::Predicted,
                        ..Control::OPEN
                    },
                    Some(Source::Predicted(prediction)),
                )
            }
            refused => {
                let fallback = match refused {
                    Some(Err(reason)) => Some(reason),
                    _ => None,
                };
                let index = self
                    .execute_solve(Role::Controller, prepared.clone())
                    .await?;
                let accepted = self.result(index)?.accepted;
                if accepted {
                    self.apply_solved(c, index)?;
                }
                let decision = match (accepted, fallback) {
                    (false, _) => HorizonDecision::Held,
                    (true, Some(_)) => HorizonDecision::Fallback,
                    (true, None) => HorizonDecision::Solved,
                };
                (
                    Control {
                        controller: Some(index),
                        decision,
                        fallback,
                        activity: None,
                        advanced: None,
                    },
                    accepted.then_some(Source::Solved(index)),
                )
            }
        };
        if k + 1 < admitted.steps {
            let next = self.next(c, advanced, k, &values, source.as_ref())?;
            if let Some(context) = pse_diagnostics::observation::current() {
                context.count("horizon.background.requests", "count", 1);
            }
            let index = crate::observation::child("horizon.background", async {
                if let Some(context) = pse_diagnostics::observation::current() {
                    context.integer("horizon.sample_index", "index", (k + 1) as i64);
                    context.text("horizon.role", "background");
                }
                self.solve(Role::Controller, next, true).await
            })
            .await?;
            self.background = Some(index);
            control.advanced = Some(index);
        }
        Ok(control)
    }
    /// The bound values an advanced-step controller predicts for sample `k + 1`: each mapped
    /// state from the solution that decided sample `k` (held without one), the next
    /// setpoint, and the moves just applied.
    fn next(
        &self,
        c: &Controller,
        advanced: &Advanced,
        k: usize,
        values: &BTreeMap<String, f64>,
        source: Option<&Source>,
    ) -> Result<BTreeMap<String, f64>, WorkflowError> {
        let mut next = BTreeMap::new();
        for (path, signal) in &c.bindings {
            let value = match signal {
                Signal::Measured(_) | Signal::Estimated(_) => {
                    match (advanced.predictions.get(path), source) {
                        (Some(id), Some(Source::Solved(i))) => solved(self.result(*i)?, id)?,
                        (Some(id), Some(Source::Predicted(p))) => p.value(id).ok_or_else(|| {
                            contract("an advanced step's predicting path is a column of its solve")
                        })?,
                        _ => values[path],
                    }
                }
                Signal::Applied(i) => self.applied[*i],
                Signal::Trajectory(values) => values[k + 1],
            };
            next.insert(path.clone(), value);
        }
        Ok(next)
    }
}

/// The O5 progress event of one completed sample.
fn event(k: usize, step: &HorizonStep, elapsed: Duration) -> Event {
    let integer = |n: usize| Metric::Integer(i64::try_from(n).unwrap_or(i64::MAX));
    let mut values = BTreeMap::from([
        ("step".to_owned(), integer(k)),
        ("time".to_owned(), Metric::Real(step.time)),
        (
            "decision".to_owned(),
            Metric::Text(step.decision.as_str().to_owned()),
        ),
    ]);
    if let Some(estimator) = step.estimator {
        values.insert("estimator".to_owned(), integer(estimator));
    }
    if let Some(controller) = step.controller {
        values.insert("controller".to_owned(), integer(controller));
    }
    if let Some(advanced) = step.advanced {
        values.insert("advanced".to_owned(), integer(advanced));
    }
    if let Some(fallback) = step.fallback {
        values.insert(
            "fallback".to_owned(),
            Metric::Text(fallback.as_str().to_owned()),
        );
    }
    for (i, value) in step.applied.iter().enumerate() {
        values.insert(format!("input.{i}"), Metric::Real(*value));
    }
    for (o, value) in step.measured.iter().enumerate() {
        values.insert(format!("output.{o}"), Metric::Real(*value));
    }
    Event {
        phase: "horizon.step".to_owned(),
        elapsed,
        values,
        incumbent: None,
    }
}
