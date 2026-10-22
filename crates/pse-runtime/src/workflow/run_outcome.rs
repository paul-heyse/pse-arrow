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
            Self::Horizon { execution: HorizonRunOutcome::Full { requests, .. }, .. } => RunRequest::Modeling(requests),
            Self::Horizon { request, execution: HorizonRunOutcome::Compact(_), .. } => RunRequest::Horizon(request),
            #[cfg(feature = "solver-diffsol")]
            Self::Shooting { problem, initial, .. } => RunRequest::Shooting { problem, initial },
        }
    }
    pub(super) fn original_request(&self) -> RunRequest<'_> {
        match self {
            Self::Horizon { request, .. } => RunRequest::Horizon(request),
            _ => self.request(),
        }
    }
    pub(super) fn report(&self) -> Result<RunReport<'_>, &Arc<WorkflowError>> {
        match self {
            Self::Modeling { result, .. } => result.as_ref().map(RunReport::Modeling),
            Self::Simulation { result, .. } => result.as_ref().map(|report| RunReport::Simulation(report.as_ref())),
            Self::Fit { result, .. } => result.as_ref().map(|report| RunReport::Fit(report.as_ref())),
            Self::Horizon { execution: HorizonRunOutcome::Full { result, .. }, .. } => result.as_ref().map(RunReport::Modeling),
            Self::Horizon { execution: HorizonRunOutcome::Compact(result), .. } => result.as_ref().map(RunReport::Horizon),
            #[cfg(feature = "solver-diffsol")]
            Self::Shooting { result, .. } => result.as_ref().map(|report| RunReport::Shooting(report.as_ref())),
        }
    }
    pub(super) fn error(&self) -> Option<&Arc<WorkflowError>> {
        match self {
            Self::Modeling { result, .. } => result.as_ref().err(),
            Self::Simulation { result, .. } => result.as_ref().err(),
            Self::Fit { result, .. } => result.as_ref().err(),
            Self::Horizon { execution: HorizonRunOutcome::Full { result, .. }, .. } => result.as_ref().err(),
            Self::Horizon { execution: HorizonRunOutcome::Compact(result), .. } => result.as_ref().err(),
            #[cfg(feature = "solver-diffsol")]
            Self::Shooting { result, .. } => result.as_ref().err(),
        }
    }
    pub(super) fn fail(&mut self, error: Arc<WorkflowError>) {
        match self {
            Self::Modeling { result, .. } => *result = Err(error),
            Self::Simulation { result, .. } => *result = Err(error),
            Self::Fit { result, .. } => *result = Err(error),
            Self::Horizon { execution: HorizonRunOutcome::Full { result, .. }, .. } => *result = Err(error),
            Self::Horizon { execution: HorizonRunOutcome::Compact(result), .. } => *result = Err(error),
            #[cfg(feature = "solver-diffsol")]
            Self::Shooting { result, .. } => *result = Err(error),
        }
    }
    #[cfg(feature = "solver-diffsol")]
    pub(in crate::workflow) fn horizon_refused(request: HorizonRequest, error: Arc<WorkflowError>) -> Self {
        let execution = match &request.original.output {
            crate::workflow::StudyOutputDemand::Full {} => HorizonRunOutcome::Full { requests: Vec::new(), result: Err(error) },
            crate::workflow::StudyOutputDemand::Compact { .. } => HorizonRunOutcome::Compact(Err(error)),
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
    fn request(&self) -> RunRequest<'_> { RunRequest::Modeling(self) }
    fn finish(self, result: Report<Self::Report>) -> RunOutcome { RunOutcome::Modeling { request: self, result } }
}
impl PairedOperation for Box<ModelingSimulation> {
    type Report = Box<ModelingTrajectory>;
    fn request(&self) -> RunRequest<'_> { RunRequest::Simulation(self) }
    fn finish(self, result: Report<Self::Report>) -> RunOutcome { RunOutcome::Simulation { request: self, result } }
}
impl PairedOperation for Box<PreparedFit> {
    type Report = Box<FitReport>;
    fn request(&self) -> RunRequest<'_> { RunRequest::Fit(self) }
    fn finish(self, result: Report<Self::Report>) -> RunOutcome { RunOutcome::Fit { request: self, result } }
}
#[cfg(feature = "solver-diffsol")]
pub(super) struct ShootingOperation {
    pub problem: Arc<crate::workflow::ShootingProblem>,
    pub initial: Option<Vec<f64>>,
}
#[cfg(feature = "solver-diffsol")]
impl PairedOperation for ShootingOperation {
    type Report = Box<crate::workflow::ShootingReport>;
    fn request(&self) -> RunRequest<'_> { RunRequest::Shooting { problem: &self.problem, initial: &self.initial } }
    fn finish(self, result: Report<Self::Report>) -> RunOutcome { RunOutcome::Shooting { problem: self.problem, initial: self.initial, result } }
}
