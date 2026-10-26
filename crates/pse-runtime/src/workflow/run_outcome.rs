// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Scientific requests and typed joined reports have one owning declaration.
use crate::workflow::{
    FitReport, HorizonOutcome, HorizonRequest, ModelingResult, ModelingSimulation,
    ModelingSolvePreparation, ModelingTrajectory, PreparedFit, WorkflowError,
};
use std::sync::Arc;

type Report<T> = Result<T, Arc<WorkflowError>>;

/// Borrowed execution-input projection; it never stores an independent request.
#[derive(Clone, Copy, Debug)]
pub enum RunRequest<'a> {
    /// Compact horizon inputs.
    Horizon(&'a HorizonRequest),
    /// Actual prepared algebraic targets, including a full horizon's dispatched targets.
    Modeling(&'a Vec<ModelingSolvePreparation>),
    /// Immutable physical simulation.
    Simulation(&'a ModelingSimulation),
    /// Immutable shared-parameter fit.
    Fit(&'a PreparedFit),
    /// Original shooting coordinates.
    #[cfg(feature = "solver-diffsol")]
    Shooting {
        /// Admitted problem owner.
        problem: &'a Arc<crate::workflow::ShootingProblem>,
        /// Explicit original coordinates, when selected.
        initial: &'a Option<Vec<f64>>,
    },
}
/// Kind-owned lifecycle premises. Positional sources retain distinct context cuts.
pub(super) struct RequestFacts<'a> {
    pub sources: Vec<&'a crate::workflow::ModelingPackage>,
    pub identity: Result<(pse_ids::ContentHash, Option<pse_ids::ContentHash>), WorkflowError>,
    pub provenance: Result<Vec<serde_json::Value>, WorkflowError>,
    pub requested_steps: usize,
    pub native_record_coverage: crate::workflow::ReplayNativeRecordCoverage,
    pub replay: Box<
        dyn FnOnce(
                pse_model::generated::identities::RunId,
            ) -> Result<super::replay::ReplayRequestFacts, WorkflowError>
            + Send
            + Sync
            + 'a,
    >,
}
impl<'a> RequestFacts<'a> {
    pub(super) fn context(&self) -> Result<&'a crate::workflow::PhysicalContext, WorkflowError> {
        let source = self
            .sources
            .first()
            .ok_or_else(|| crate::workflow::contract("empty scientific request"))?;
        Ok(&source.physical)
    }
    pub(super) fn inputs(&self) -> Result<Vec<pse_operations::canonical::Revision>, WorkflowError> {
        let mut inputs = self
            .sources
            .iter()
            .map(|source| source.recorded_revision().cloned())
            .collect::<Result<Vec<_>, _>>()?;
        inputs.sort_by(|a, b| a.key.cmp(&b.key));
        inputs.dedup_by(|a, b| a.key == b.key);
        Ok(inputs)
    }
    pub(super) fn source_selections(&self) -> Vec<crate::workflow::OperationSource> {
        self.sources
            .iter()
            .map(|source| crate::workflow::OperationSource::of(source))
            .collect()
    }
}
impl<'a> RunRequest<'a> {
    /// One exhaustive dispatch supplies all lifecycle readers with the same premises.
    pub(super) fn facts(self) -> RequestFacts<'a> {
        use crate::workflow::{ReplayNativeRecordCoverage as Coverage, contract};
        let encode =
            |value| serde_json::to_value(value).map_err(|error| contract(error.to_string()));
        match self {
            Self::Modeling(steps) => {
                let identity = (|| {
                    let mut hash =
                        pse_ids::FramedHasher::new(pse_ids::Frame::DurableModelingRequestV3);
                    hash.u64(steps.len() as u64);
                    for step in steps {
                        hash.hash(
                            &step
                                .solve
                                .request_identity()
                                .map_err(crate::math::MathRuntimeError::from)?
                                .as_id(),
                        );
                    }
                    let preparation = match steps.as_slice() {
                        [one] => Some(
                            one.solve
                                .preparation_identity()
                                .map_err(crate::math::MathRuntimeError::from)?,
                        ),
                        [] | [_, _, ..] => None,
                    };
                    Ok((hash.finish_hash(), preparation))
                })();
                RequestFacts {
                    sources: steps.iter().map(|step| &step.source).collect(),
                    identity,
                    provenance: steps
                        .iter()
                        .map(|step| {
                            serde_json::to_value((
                                &step.starts,
                                step.solve.numerical_strategy(),
                                step.bound_source.as_deref().map(|bound| &**bound),
                            ))
                            .map_err(|error| contract(error.to_string()))
                        })
                        .collect(),
                    requested_steps: steps.len(),
                    native_record_coverage: Coverage::StrategyAttempts,
                    replay: Box::new(move |run| super::replay::modeling_request_facts(steps, run)),
                }
            }
            Self::Simulation(simulation) => RequestFacts {
                sources: vec![&simulation.source],
                identity: Ok((simulation.identity(), Some(simulation.identity()))),
                provenance: encode(simulation.profile()).map(|value| vec![value]),
                requested_steps: 1,
                native_record_coverage: Coverage::IntegrationReportOnly,
                replay: Box::new(move |run| {
                    super::replay::simulation_request_facts(simulation, run)
                }),
            },
            Self::Fit(fit) => RequestFacts {
                sources: vec![&fit.source],
                identity: Ok((fit.problem.key, Some(fit.problem.key))),
                provenance: serde_json::to_value(fit.numerical_strategy())
                    .map(|value| vec![value])
                    .map_err(|error| contract(error.to_string())),
                requested_steps: 1,
                native_record_coverage: Coverage::StrategyAttempts,
                replay: Box::new(move |run| super::replay::fit_request_facts(fit, run)),
            },
            Self::Horizon(horizon) => RequestFacts {
                sources: horizon.sources.iter().collect(),
                identity: Ok((horizon.identity, None)),
                provenance: serde_json::from_slice(&horizon.provenance)
                    .map(|value| vec![value])
                    .map_err(|error| contract(error.to_string())),
                requested_steps: 1,
                native_record_coverage: Coverage::IntegrationReportOnly,
                replay: Box::new(move |run| super::replay::horizon_request_facts(horizon, run)),
            },
            #[cfg(feature = "solver-diffsol")]
            Self::Shooting { problem, initial } => RequestFacts {
                sources: vec![&problem.simulation.source],
                identity: Ok((
                    problem.request_identity(initial.as_deref()).as_id(),
                    Some(problem.request_identity(None).as_id()),
                )),
                provenance: problem
                    .numerical_strategy()
                    .map_err(crate::math::MathRuntimeError::from)
                    .map_err(WorkflowError::from)
                    .and_then(|strategy| {
                        serde_json::to_value(strategy)
                            .map(|value| vec![value])
                            .map_err(|error| contract(error.to_string()))
                    }),
                requested_steps: 1,
                native_record_coverage: Coverage::StrategyAttempts,
                replay: Box::new(move |run| {
                    super::replay::shooting_request_facts(problem, initial, run)
                }),
            },
        }
    }
}
/// Borrowed report projection derived from the paired outcome.
#[derive(Clone, Copy, Debug)]
pub enum RunReport<'a> {
    /// Compact original horizon history.
    Horizon(&'a Arc<HorizonOutcome>),
    /// Completed algebraic targets.
    Modeling(&'a Vec<ModelingResult>),
    /// Physical integration and its original checks.
    Simulation(&'a ModelingTrajectory),
    /// Parameter fitting and its original checks.
    Fit(&'a FitReport),
    /// Composed shooting result.
    #[cfg(feature = "solver-diffsol")]
    Shooting(&'a crate::workflow::ShootingReport),
}
/// The physical horizon result representation selected by its original request.
#[derive(Debug)]
pub enum HorizonRunOutcome {
    /// Full executed modeling history; failed suffix metadata remains on RunResult.
    Full {
        /// Only targets actually prepared by the horizon owner.
        requests: Vec<ModelingSolvePreparation>,
        /// Its typed joined results.
        result: Report<Vec<ModelingResult>>,
    },
    /// Compact history, whose independently retained prefix survives failure.
    Compact(Report<Arc<HorizonOutcome>>),
}
/// The sole owning request/result pair. A family cannot contain another family's report.
#[derive(Debug)]
pub enum RunOutcome {
    /// One finite authored algebraic sequence.
    Modeling {
        /// Original prepared sequence.
        request: Vec<ModelingSolvePreparation>,
        /// Completed sequence, or its typed failure.
        result: Report<Vec<ModelingResult>>,
    },
    /// One physical simulation.
    Simulation {
        /// Original preparation.
        request: Box<ModelingSimulation>,
        /// Its original checked trajectory.
        result: Report<Box<ModelingTrajectory>>,
    },
    /// One shared-parameter fitting attempt.
    Fit {
        /// Original fitting preparation.
        request: Box<PreparedFit>,
        /// Its typed fit result.
        result: Report<Box<FitReport>>,
    },
    /// A horizon always owns its original input, independently of history materialization.
    Horizon {
        /// Actual original horizon input.
        request: HorizonRequest,
        /// Full or compact joined physical history.
        execution: HorizonRunOutcome,
    },
    /// One composed host-only shooting attempt.
    #[cfg(feature = "solver-diffsol")]
    Shooting {
        /// Admitted problem owner.
        problem: Arc<crate::workflow::ShootingProblem>,
        /// The exact original initial coordinates.
        initial: Option<Vec<f64>>,
        /// Its composed physical/numerical result.
        result: Report<Box<crate::workflow::ShootingReport>>,
    },
}
impl RunOutcome {
    pub(super) fn request(&self) -> RunRequest<'_> {
        match self {
            Self::Modeling { request, .. } => RunRequest::Modeling(request),
            Self::Simulation { request, .. } => RunRequest::Simulation(request),
            Self::Fit { request, .. } => RunRequest::Fit(request),
            Self::Horizon {
                execution: HorizonRunOutcome::Full { requests, .. },
                ..
            } => RunRequest::Modeling(requests),
            Self::Horizon {
                request,
                execution: HorizonRunOutcome::Compact(_),
                ..
            } => RunRequest::Horizon(request),
            #[cfg(feature = "solver-diffsol")]
            Self::Shooting {
                problem, initial, ..
            } => RunRequest::Shooting { problem, initial },
        }
    }
    pub(super) fn original_request(&self) -> RunRequest<'_> {
        match self {
            Self::Horizon { request, .. } => RunRequest::Horizon(request),
            Self::Modeling { .. } | Self::Simulation { .. } | Self::Fit { .. } => self.request(),
            #[cfg(feature = "solver-diffsol")]
            Self::Shooting { .. } => self.request(),
        }
    }
    pub(super) fn report(&self) -> Result<RunReport<'_>, &Arc<WorkflowError>> {
        match self {
            Self::Modeling { result, .. } => result.as_ref().map(RunReport::Modeling),
            Self::Simulation { result, .. } => result
                .as_ref()
                .map(|report| RunReport::Simulation(report.as_ref())),
            Self::Fit { result, .. } => result
                .as_ref()
                .map(|report| RunReport::Fit(report.as_ref())),
            Self::Horizon {
                execution: HorizonRunOutcome::Full { result, .. },
                ..
            } => result.as_ref().map(RunReport::Modeling),
            Self::Horizon {
                execution: HorizonRunOutcome::Compact(result),
                ..
            } => result.as_ref().map(RunReport::Horizon),
            #[cfg(feature = "solver-diffsol")]
            Self::Shooting { result, .. } => result
                .as_ref()
                .map(|report| RunReport::Shooting(report.as_ref())),
        }
    }
    pub(super) fn error(&self) -> Option<&Arc<WorkflowError>> {
        match self {
            Self::Modeling { result, .. } => result.as_ref().err(),
            Self::Simulation { result, .. } => result.as_ref().err(),
            Self::Fit { result, .. } => result.as_ref().err(),
            Self::Horizon {
                execution: HorizonRunOutcome::Full { result, .. },
                ..
            } => result.as_ref().err(),
            Self::Horizon {
                execution: HorizonRunOutcome::Compact(result),
                ..
            } => result.as_ref().err(),
            #[cfg(feature = "solver-diffsol")]
            Self::Shooting { result, .. } => result.as_ref().err(),
        }
    }
    pub(super) fn fail(&mut self, error: Arc<WorkflowError>) {
        match self {
            Self::Modeling { result, .. } => *result = Err(error),
            Self::Simulation { result, .. } => *result = Err(error),
            Self::Fit { result, .. } => *result = Err(error),
            Self::Horizon {
                execution: HorizonRunOutcome::Full { result, .. },
                ..
            } => *result = Err(error),
            Self::Horizon {
                execution: HorizonRunOutcome::Compact(result),
                ..
            } => *result = Err(error),
            #[cfg(feature = "solver-diffsol")]
            Self::Shooting { result, .. } => *result = Err(error),
        }
    }
    #[cfg(feature = "solver-diffsol")]
    pub(in crate::workflow) fn horizon_refused(
        request: HorizonRequest,
        error: Arc<WorkflowError>,
    ) -> Self {
        let execution = match &request.original.output {
            crate::workflow::StudyOutputDemand::Full {} => HorizonRunOutcome::Full {
                requests: Vec::new(),
                result: Err(error),
            },
            crate::workflow::StudyOutputDemand::Compact { .. } => {
                HorizonRunOutcome::Compact(Err(error))
            }
        };
        Self::Horizon { request, execution }
    }
}

/// Native dispatch carries its input's report type through the finish boundary.
pub(super) trait PairedOperation: Send + Sync + 'static {
    type Report: Send + 'static;
    fn request(&self) -> RunRequest<'_>;
    fn finish(self, report: Report<Self::Report>) -> RunOutcome;
}
impl PairedOperation for Vec<ModelingSolvePreparation> {
    type Report = Vec<ModelingResult>;
    fn request(&self) -> RunRequest<'_> {
        RunRequest::Modeling(self)
    }
    fn finish(self, result: Report<Self::Report>) -> RunOutcome {
        RunOutcome::Modeling {
            request: self,
            result,
        }
    }
}
impl PairedOperation for Box<ModelingSimulation> {
    type Report = Box<ModelingTrajectory>;
    fn request(&self) -> RunRequest<'_> {
        RunRequest::Simulation(self)
    }
    fn finish(self, result: Report<Self::Report>) -> RunOutcome {
        RunOutcome::Simulation {
            request: self,
            result,
        }
    }
}
impl PairedOperation for Box<PreparedFit> {
    type Report = Box<FitReport>;
    fn request(&self) -> RunRequest<'_> {
        RunRequest::Fit(self)
    }
    fn finish(self, result: Report<Self::Report>) -> RunOutcome {
        RunOutcome::Fit {
            request: self,
            result,
        }
    }
}
#[cfg(feature = "solver-diffsol")]
pub(super) struct ShootingOperation {
    pub problem: Arc<crate::workflow::ShootingProblem>,
    pub initial: Option<Vec<f64>>,
}
#[cfg(feature = "solver-diffsol")]
impl PairedOperation for ShootingOperation {
    type Report = Box<crate::workflow::ShootingReport>;
    fn request(&self) -> RunRequest<'_> {
        RunRequest::Shooting {
            problem: &self.problem,
            initial: &self.initial,
        }
    }
    fn finish(self, result: Report<Self::Report>) -> RunOutcome {
        RunOutcome::Shooting {
            problem: self.problem,
            initial: self.initial,
            result,
        }
    }
}

/// Shared retained completion projections. These observations grant no scientific permission.
pub(super) struct CompletedFacts<'a> {
    pub table_ids: Vec<pse_ids::SemanticId>,
    pub traces: Vec<(usize, &'a Arc<crate::math::strategy::Trace>)>,
    pub native_reports: Vec<(usize, &'a pse_backend_native::solve::SolveReport)>,
    pub completed_modeling: Vec<(&'a ModelingSolvePreparation, &'a ModelingResult)>,
    pub requested: usize,
    pub observed: usize,
    pub completed_time: Option<pse_model::diagnostic::Observation>,
    pub terminal: Option<pse_model::generated::enums::NumericalAttemptObservation>,
    pub non_modeling_accepted: Option<bool>,
}
impl<'a> CompletedFacts<'a> {
    fn modeling(
        &mut self,
        run: &'a super::RunResult,
        requests: &'a [ModelingSolvePreparation],
        result: Option<&'a Vec<ModelingResult>>,
    ) {
        self.requested = requests.len();
        if let Some(results) = result {
            self.completed_modeling.extend(requests.iter().zip(results));
        }
        for step in 0..requests.len() {
            if let Some(result) = run.modeling_result(step) {
                self.observed += 1;
                self.traces
                    .extend(result.strategy.iter().map(|trace| (step, trace)));
                if let crate::math::solves::Outcome::Native(native) = &result.outcome {
                    self.native_reports.push((step, native));
                }
            }
        }
        self.terminal = result
            .and_then(|results| results.last())
            .map(|result| crate::math::strategy::observe(&result.outcome));
    }
}
fn modeling_table_ids() -> Vec<pse_ids::SemanticId> {
    use pse_relations::generated::runtime::*;
    Vec::from([
        scientific_contexts::RELATION_ID,
        solve_runs::RELATION_ID,
        solve_variables::RELATION_ID,
        solve_constraints::RELATION_ID,
        solve_metrics::RELATION_ID,
        solution_pool::RELATION_ID,
        infeasibility_certificates::RELATION_ID,
        incumbents::RELATION_ID,
        modeling_checks::RELATION_ID,
        modeling_reports::RELATION_ID,
        modeling_findings::RELATION_ID,
        solve_strategy_events::RELATION_ID,
        solve_strategy_products::RELATION_ID,
        local_validity::RELATION_ID,
        parametric_sensitivities::RELATION_ID,
        reduced_hessians::RELATION_ID,
        propagated_covariances::RELATION_ID,
        route_decisions::RELATION_ID,
        structural_assessments::RELATION_ID,
    ])
}
impl super::RunResult {
    /// One exhaustive kind dispatch supplies retained lifecycle observations and products.
    pub(super) fn completed_facts(&self) -> CompletedFacts<'_> {
        use pse_relations::generated::runtime::*;
        let mut facts = CompletedFacts {
            table_ids: Vec::new(),
            traces: Vec::new(),
            native_reports: Vec::new(),
            completed_modeling: Vec::new(),
            requested: 1,
            observed: 0,
            completed_time: None,
            terminal: None,
            non_modeling_accepted: None,
        };
        match &self.outcome {
            RunOutcome::Horizon {
                request,
                execution: HorizonRunOutcome::Compact(result),
            } => {
                facts.requested = request.original.steps;
                facts.observed = self
                    .horizon
                    .as_ref()
                    .map_or(0, |report| report.completed_steps);
                if let Ok(report) = result {
                    facts.non_modeling_accepted = Some(self.usable());
                    facts.terminal = report
                        .summary_error()
                        .or_else(|| report.detail_error())
                        .map(crate::math::strategy::workflow_failure);
                }
                facts
                    .table_ids
                    .extend(super::horizon::prefix::summary_ids());
                if let Some(report) = &self.horizon_output {
                    facts.table_ids.extend(report.summaries.keys());
                    facts.table_ids.extend(&report.detail_membership);
                }
            }
            RunOutcome::Modeling { request, result } => {
                facts.modeling(self, request, result.as_ref().ok());
                facts.table_ids.extend(modeling_table_ids());
            }
            RunOutcome::Horizon {
                request,
                execution: HorizonRunOutcome::Full { requests, result },
            } => {
                facts.modeling(self, requests, result.as_ref().ok());
                facts.requested = request.original.steps;
                facts.observed = self
                    .horizon
                    .as_ref()
                    .map_or(0, |report| report.completed_steps);
                facts.table_ids.extend(modeling_table_ids());
            }
            RunOutcome::Simulation { request, result } => {
                facts.requested = request.profile().samples.len();
                if let Ok(trajectory) = result {
                    facts.observed = trajectory.report().coverage.produced();
                    facts.completed_time = Some(pse_model::diagnostic::Observation::number(
                        trajectory.report().completed_time,
                    ));
                    facts.terminal =
                        Some(super::replay::trajectory_observation(trajectory.report()));
                    facts.non_modeling_accepted = Some(self.usable());
                }
                facts.table_ids.push(computation_runs::RELATION_ID);
                if let Ok(trajectory) = result {
                    facts.table_ids.extend(trajectory.table_ids());
                }
            }
            RunOutcome::Fit { result, .. } => {
                if let Ok(report) = result {
                    facts.observed = 1;
                    facts.non_modeling_accepted = Some(self.usable());
                    facts
                        .traces
                        .extend(report.strategy.iter().map(|trace| (0, trace)));
                    facts
                        .native_reports
                        .extend(report.solve.iter().map(|native| (0, native)));
                    facts.terminal = report
                        .validation_error
                        .as_ref()
                        .map(|diagnostic| crate::math::strategy::boundary_failure(diagnostic.class))
                        .or_else(|| {
                            report
                                .solve
                                .as_ref()
                                .map(crate::math::strategy::observe_native)
                        });
                }
                facts.table_ids.extend([
                    scientific_contexts::RELATION_ID,
                    computation_runs::RELATION_ID,
                    fit_variables::RELATION_ID,
                    fit_constraints::RELATION_ID,
                    fit_parameters::RELATION_ID,
                    fit_observations::RELATION_ID,
                    response_sensitivities::RELATION_ID,
                    solve_metrics::RELATION_ID,
                    infeasibility_certificates::RELATION_ID,
                    modeling_checks::RELATION_ID,
                    modeling_reports::RELATION_ID,
                    solve_strategy_events::RELATION_ID,
                    solve_strategy_products::RELATION_ID,
                    local_validity::RELATION_ID,
                    parameter_covariances::RELATION_ID,
                    parameter_intervals::RELATION_ID,
                    profile_points::RELATION_ID,
                    response_directions::RELATION_ID,
                    propagated_covariances::RELATION_ID,
                ]);
            }
            #[cfg(feature = "solver-diffsol")]
            RunOutcome::Shooting { result, .. } => {
                if let Ok(report) = result {
                    facts.observed = 1;
                    facts.non_modeling_accepted = Some(self.usable());
                    facts
                        .traces
                        .extend(report.strategy.iter().map(|trace| (0, trace)));
                    facts
                        .native_reports
                        .extend(report.solve.iter().map(|native| (0, native)));
                    facts.terminal = report
                        .validation_error
                        .as_ref()
                        .map(|diagnostic| crate::math::strategy::boundary_failure(diagnostic.class))
                        .or_else(|| {
                            report
                                .trajectory
                                .as_ref()
                                .map(|trajectory| super::replay::trajectory_observation(trajectory))
                        })
                        .or_else(|| {
                            report
                                .solve
                                .as_ref()
                                .map(crate::math::strategy::observe_native)
                        });
                }
                facts.table_ids.extend([
                    computation_runs::RELATION_ID,
                    solve_variables::RELATION_ID,
                    solve_constraints::RELATION_ID,
                    solve_metrics::RELATION_ID,
                    solve_strategy_events::RELATION_ID,
                    solve_strategy_products::RELATION_ID,
                ]);
                if let Ok(report) = result
                    && report.trajectory.is_some()
                {
                    facts.table_ids.extend(ModelingTrajectory::relation_ids());
                }
            }
        }

        if let Some(error) = self.error() {
            facts.terminal = Some(crate::math::strategy::workflow_failure(error));
            if let Some(trace) = error.strategy_trace() {
                facts.traces.push((
                    self.modeling_failure
                        .as_ref()
                        .and_then(|failure| failure.step)
                        .unwrap_or(0),
                    trace,
                ));
            }
        }
        facts
    }
}
