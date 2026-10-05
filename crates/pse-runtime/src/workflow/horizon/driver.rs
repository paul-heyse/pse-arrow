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
    durable::{DurableAttempt, SeedContext},
    integrated::IntegratedExperiment,
    modeling::assessment::Obligations,
    run::{attempt_for, progress_for},
    staged::{Overlay, Staged, Start},
};
use pse_backend_native::{
    self as native, ProblemError,
    dynamics::DynamicSensitivity,
    kkt::{Fallback, Prediction},
    solve::{Event, Metric, Progress},
};
use pse_ids::{FramedHasher, SemanticId};
use pse_model::generated::identities::RunId;
use pse_operations::attempts::AttemptKind;
use std::{
    collections::{BTreeMap, BTreeSet},
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
        if cancel.token().is_cancelled() {
            return Err(MathRuntimeError::Cancelled.into());
        }
        Admitted::new(self, horizon, cancel).await.map(|_| ())
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
        self.start_horizon_owned(horizon, cancel, None).await
    }
    pub(crate) async fn start_horizon_attempt(
        &self,
        horizon: Horizon,
        cancel: &crate::CancelSource,
        attempt: DurableAttempt,
    ) -> Result<RunHandle, WorkflowError> {
        self.start_horizon_owned(horizon, cancel, Some(attempt))
            .await
    }
    async fn start_horizon_owned(
        &self,
        horizon: Horizon,
        cancel: &crate::CancelSource,
        attempt: Option<DurableAttempt>,
    ) -> Result<RunHandle, WorkflowError> {
        if cancel.token().is_cancelled() {
            return Err(MathRuntimeError::Cancelled.into());
        }
        let admitted = Arc::new(Admitted::new(self, horizon, cancel).await?);
        let durable = attempt_for(self, attempt)?;
        let progress = progress_for(admitted.history, durable.as_ref());
        let opened = match durable {
            None => Some(Staged::open(self, Some(progress.clone()))?),
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
        let stream = progress.clone();
        tokio::spawn(async move {
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
                    let refused = RunResult::joined(
                        run_id,
                        runtime,
                        RunRequest::Modeling(Vec::new()),
                        None,
                        Err(Arc::new(error)),
                    )
                    .refused(durable)
                    .await;
                    sender.send_replace(Some(Arc::new(refused)));
                    return;
                }
            };
            let mut run = Loop::new(admitted, staged, run_id, cancel.clone(), stream, durable);
            let outcome = run.run().await;
            let Loop {
                admitted,
                staged,
                durable,
                requests,
                results,
                steps,
                ..
            } = run;
            staged.close().await;
            let report = outcome
                .map(|()| RunReport::Modeling(results))
                .map_err(Arc::new);
            let mut result = RunResult::joined(
                run_id,
                runtime,
                RunRequest::Modeling(requests),
                None,
                report,
            );
            result.horizon = Some(Arc::new(HorizonReport {
                outputs: admitted.outputs.clone(),
                inputs: admitted.inputs.clone(),
                steps,
            }));
            let cancelled = cancel.token().is_cancelled();
            let result = result.finished(durable, cancelled).await;
            sender.send_replace(Some(Arc::new(result)));
        });
        Ok(RunHandle::staged(
            checks, receiver, progress, run_id, attempt_id,
        ))
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
            .register_as(
                run_id,
                AttemptKind::Modeling,
                admitted.identity.as_id(),
                None,
            )
            .await?;
    }
    let staged = match opened {
        Some(staged) => staged,
        None => tokio::select! {
            staged = Staged::open_queued(runtime, Some(progress.clone())) => staged?,
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
        Ok(Self {
            package: package.clone(),
            analysis,
            read,
            identity,
        })
    }
}

/// A controller binding resolved against the plant, the inputs and the estimator.
#[derive(Clone, Debug)]
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
#[derive(Debug)]
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
    /// The integration column of each driven input.
    columns: Vec<usize>,
    /// The differential states, whose ends anchor the next period.
    differential: Vec<usize>,
    start: f64,
    period: f64,
    time_limit: Duration,
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
    fn execution(&self, flag: &Arc<AtomicBool>) -> native::solve::Execution {
        let mut execution =
            native::solve::Execution::new(flag.clone(), &native::solve::Controls::default());
        execution.time_limit = self.time_limit;
        execution
    }
    /// The consistently initialized start at `t₀` under `applied`: the simulation's initial
    /// condition, integrated over the first period sampled at its start only.
    fn initial(
        &self,
        applied: &[f64],
        execution: &native::solve::Execution,
    ) -> Result<native::dynamics::Sample, ProblemError> {
        use native::dynamics::Oracle;
        let integration = self.integration(applied);
        let contract = &self.experiment.program.contract;
        let mut worker = self.experiment.program.worker(execution.scope()?)?;
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
        k: usize,
        applied: &[f64],
        anchors: &[f64],
        execution: &native::solve::Execution,
    ) -> Result<native::dynamics::Sample, ProblemError> {
        let end = self.time(k + 1);
        let window = self.experiment.window(self.time(k), end, vec![end])?;
        let report = self.experiment.integrate_window(
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

/// A horizon admitted before any effect.
#[derive(Debug)]
struct Admitted {
    plant: Arc<Plant>,
    steps: usize,
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
    async fn new(
        runtime: &Runtime,
        horizon: Horizon,
        cancel: &crate::CancelSource,
    ) -> Result<Self, WorkflowError> {
        let Horizon {
            plant,
            period,
            steps,
            inputs,
            estimator,
            controller,
        } = horizon;
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
        let output = |id: &SemanticId| {
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
                        output(id)?;
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
            Some(e) => {
                Some(Self::estimator(runtime, e, steps, &output, &input, &estimated, cancel).await?)
            }
            None => None,
        };
        let controller = match &controller {
            Some(c) => {
                Some(Self::controller(runtime, c, &output, estimator.as_ref(), cancel).await?)
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
        );
        Ok(Self {
            plant: Arc::new(Plant {
                experiment: IntegratedExperiment {
                    snapshot: plant.snapshot().clone(),
                    program: plant.program(),
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
) -> pse_ids::roles::LineageRequestHash {
    let mut h = FramedHasher::new(pse_ids::Frame::DurableHorizonRequestV1);
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
    pse_ids::roles::LineageRequestHash::from_id(h.finish_hash())
}

#[derive(Clone, Copy, Debug)]
enum Role {
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
struct Loop {
    admitted: Arc<Admitted>,
    staged: Staged,
    durable: Option<DurableAttempt>,
    run_id: RunId,
    cancel: crate::CancelSource,
    progress: Arc<Progress>,
    started: Instant,
    /// The run's modeling steps and their results, in order; the staged sequence records
    /// the same steps, so a step's index is its record's.
    requests: Vec<ModelingSolvePreparation>,
    results: Vec<ModelingResult>,
    steps: Vec<HorizonStep>,
    /// Plant outputs at every sample so far.
    measured: Vec<Vec<f64>>,
    /// Inputs applied over every completed period.
    history: Vec<Vec<f64>>,
    applied: Vec<f64>,
    /// The differential states at the current sample, in the plant's normalized
    /// coordinates.
    anchors: Vec<f64>,
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
    fn new(
        admitted: Arc<Admitted>,
        staged: Staged,
        run_id: RunId,
        cancel: crate::CancelSource,
        progress: Arc<Progress>,
        durable: Option<DurableAttempt>,
    ) -> Self {
        let priors = admitted
            .estimator
            .iter()
            .flat_map(|e| e.arrival.iter().map(|(_, _, initial)| *initial))
            .collect();
        Self {
            applied: admitted.initial.clone(),
            admitted,
            staged,
            durable,
            run_id,
            cancel,
            progress,
            started: Instant::now(),
            requests: Vec::new(),
            results: Vec::new(),
            steps: Vec::new(),
            measured: Vec::new(),
            history: Vec::new(),
            anchors: Vec::new(),
            priors,
            accepted: [None; 2],
            estimate: None,
            background: None,
        }
    }
    async fn run(&mut self) -> Result<(), WorkflowError> {
        let applied = self.applied.clone();
        let start = self
            .plant(move |plant, execution| plant.initial(&applied, execution))
            .await?;
        self.anchor(start);
        for k in 0..self.admitted.steps {
            if self.cancel.token().is_cancelled() {
                return Err(MathRuntimeError::Cancelled.into());
            }
            let estimator = self.estimate(k).await?;
            let control = self.control(k).await?;
            let applied = self.applied.clone();
            let anchors = self.anchors.clone();
            let end = self
                .plant(move |plant, execution| plant.period(k, &applied, &anchors, execution))
                .await?;
            self.history.push(self.applied.clone());
            self.anchor(end);
            let step = HorizonStep {
                time: self.admitted.plant.time(k),
                measured: self.measured[k].clone(),
                estimator,
                controller: control.controller,
                decision: control.decision,
                fallback: control.fallback,
                activity: control.activity,
                advanced: control.advanced,
                applied: self.applied.clone(),
                reached: self.measured[k + 1].clone(),
            };
            self.progress.push(event(k, &step, self.started.elapsed()));
            self.steps.push(step);
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
        self.measured.push(sample.outputs);
    }
    /// Plant work on the session thread beside the solves' retained native state.
    async fn plant<T: Send + 'static>(
        &self,
        work: impl FnOnce(&Plant, &native::solve::Execution) -> Result<T, ProblemError> + Send + 'static,
    ) -> Result<T, WorkflowError> {
        let plant = self.admitted.plant.clone();
        Ok(self
            .staged
            .native(self.admitted.threads, &self.cancel, move |_, flag, _| {
                let execution = plant.execution(flag);
                work(&plant, &execution).map_err(MathRuntimeError::from)
            })
            .await?)
    }
    /// One step of `role`: its overlay of `values` over its specification, seeded from the
    /// last accepted step of the role and offered its native seed (N2), executed and recorded
    /// as the run's next modeling step.
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
    async fn prepare_solve(
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
        let seed = last
            .and_then(|i| self.staged.seed(Start::Accepted(i)))
            .unwrap_or_default();
        let specification = Overlay {
            values,
            ..Overlay::default()
        }
        .compose(&stage.analysis);
        let mut prepared = stage
            .package
            .prepare_analysis_attempt(
                &specification,
                crate::workflow::modeling::cases::CaseOverrides {
                    seed,
                    ..Default::default()
                },
                &self.cancel,
            )
            .await?;
        if retain {
            prepared.solve = prepared
                .solve
                .retaining_factor()
                .map_err(WorkflowError::from)?;
        }
        Ok(prepared)
    }
    async fn execute_solve(
        &mut self,
        role: Role,
        prepared: ModelingSolvePreparation,
    ) -> Result<usize, WorkflowError> {
        let previous = self.accepted[role as usize].and_then(|i| self.staged.predecessor_at(i));
        let attempt = self.requests.len();
        if let Some(durable) = &self.durable {
            let seed = prepared
                .solve
                .compatibility()
                .cloned()
                .zip(prepared.solve.seed_preparation_identity())
                .map(|(compatibility, preparation)| SeedContext {
                    compatibility,
                    preparation,
                });
            durable.set_step(attempt, seed);
        }
        let result = self
            .staged
            .run(
                prepared.clone(),
                Obligations::Final,
                self.run_id,
                attempt,
                previous,
                &self.cancel,
            )
            .await
            .map_err(WorkflowError::Shared)?;
        if result.accepted {
            self.accepted[role as usize] = Some(attempt);
        }
        self.requests.push(prepared);
        self.results.push(result);
        Ok(attempt)
    }
    /// The estimator step at sample `k`, once its window is measured: the window's
    /// measurements, the inputs applied over it and the arrival priors. An accepted estimate
    /// sets the next priors.
    async fn estimate(&mut self, k: usize) -> Result<Option<usize>, WorkflowError> {
        self.estimate = None;
        let admitted = self.admitted.clone();
        let Some(e) = &admitted.estimator else {
            return Ok(None);
        };
        let Some(first) = k.checked_sub(e.window) else {
            return Ok(None);
        };
        let mut values = BTreeMap::new();
        for (o, paths) in &e.measurements {
            for (sample, path) in self.measured[first..=k].iter().zip(paths) {
                values.insert(path.clone(), sample[*o]);
            }
        }
        let periods = &self.history[first..k];
        for (i, how) in &e.inputs {
            match how {
                WindowInput::Constant(path) => {
                    let value = periods[0][*i];
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
        let result = &self.results[index];
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
                Signal::Measured(o) => self.measured[k][*o],
                Signal::Estimated(id) => match self.estimate {
                    Some(i) => solved(&self.results[i], id)?,
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
        let result = &self.results[index];
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
        let admitted = self.admitted.clone();
        let Some(c) = &admitted.controller else {
            return Ok(Control::OPEN);
        };
        let Some(values) = self.bound(c, k)? else {
            return Ok(Control::OPEN);
        };
        let Some(advanced) = &c.advanced else {
            let index = self.solve(Role::Controller, values, false).await?;
            let decision = if self.results[index].accepted {
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
        let proposal_target = prepared.solve.clone();
        // One backsolve against the background solve's retained factor at the actual state.
        // A factor predicts once: the next background solve replaces it.
        let predicted = match self.background.take() {
            None => None,
            Some(from) => {
                let permission = self.results[from].completion.decision.clone();
                let source = match &self.results[from].outcome {
                    crate::math::solves::Outcome::Native(report) => report
                        .candidate
                        .as_ref()
                        .map(|candidate| {
                            self.results[from]
                                .prepared
                                .solve
                                .semantic_point_key(&candidate.primal)
                        })
                        .transpose()
                        .map_err(MathRuntimeError::from)?,
                    _ => None,
                };
                let time_limit = admitted.plant.time_limit;
                let base = self.results[from].prepared.solve.clone();
                let target = proposal_target.clone();
                let parameters = advanced
                    .parameters
                    .iter()
                    .map(|(path, id)| (*id, values[path]))
                    .collect::<Vec<_>>();
                let outcome = self
                    .staged
                    .native(admitted.threads, &self.cancel, move |retained, flag, _| {
                        let outcome = match (retained.advance(), source) {
                            (Some(advance), Some(source))
                                if permission.permits_use()
                                    && target.numerical_strategy().start.policy
                                        != native::solve::StartPolicy::NoPriorStart =>
                            {
                                target.related_target_parameters(&base, advance.parameters())?;
                                let binding = target.original_identity()?;
                                let mut execution = native::solve::Execution::new(
                                    flag.clone(),
                                    &native::solve::Controls::default(),
                                );
                                execution.time_limit = time_limit;
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
                                let activity = target
                                    .composition_request()
                                    .recovery
                                    .contains(&pse_model::strategy::StartOrigin::Predicted)
                                    && target.numerical_strategy().start.policy
                                        != native::solve::StartPolicy::NoPriorStart;
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
                    })
                    .await?;
                Some(outcome.map(|prediction| (from, prediction)))
            }
        };
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
                let accepted = self.results[index].accepted;
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
                let accepted = self.results[index].accepted;
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
            let index = self.solve(Role::Controller, next, true).await?;
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
                        (Some(id), Some(Source::Solved(i))) => solved(&self.results[*i], id)?,
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
