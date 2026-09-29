// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Rolling horizons: moving-horizon estimation (MHE) and nonlinear model-predictive control
//! (NMPC) as compositions over the staged-sequence primitive (Plan 22 Y5c; ADR-0110
//! Outcome 5; I16).
//!
//! A horizon is one run and, under a durable runtime, one attempt: its steps are strictly
//! sequential and share one worker-owned native session, so they are not a study's
//! independent points (O7). At every sample `tₖ = t₀ + k·period` the loop
//!
//! 1. **measures** the plant: its outputs at `tₖ`, from the end of the previous period or,
//!    at the first sample, from its consistently initialized start;
//! 2. **estimates** (MHE), once `window` periods are measured: a least-squares problem over
//!    the window whose initial state is free and regularized by an arrival cost. Its prior
//!    is the previous estimate of the state one period into its window, and the first
//!    window takes the declared prior;
//! 3. **controls** (NMPC): a dynamic optimization over the controller's horizon from the
//!    measured or estimated state, whose first moves are applied; a step that is not
//!    accepted holds the last applied inputs;
//! 4. **applies** the moves to the plant for one period, integrated on the same native
//!    session from the previous period's end state (its anchored start) with the moves held
//!    (a one-interval schedule), so the solves' retained native state survives.
//!
//! The estimator and the controller are simultaneous-route (collocation) problems, the
//! primary dynamic-optimization route (ADR-0110 Outcome 5). The MHE's free initial state is
//! simply a state without an initial condition; its arrival cost is a term of the model's
//! objective over a case value, the prior, that the loop binds. Every estimator and
//! controller step composes a value-only overlay (its state, measurements, priors,
//! setpoints and held moves) over its immutable specification, so each prepares its
//! structure once and rebinds values (A6). A step starts from the values of the previous
//! accepted step of its role and offers that step's native seed (N2), which its start
//! policy consumes under `PreviousAccepted`: an interior-point restart, or the SQP working
//! set on POUNCE. The steps are the run's modeling steps: each is recorded and, under a
//! durable runtime, its accepted seed is stored (O6), and every sample streams one
//! `horizon.step` progress event (O5).
//!
//! **Advanced step** (Plan 22 Y5c2; I16). A controller may decide each sample by
//! prediction instead of a solve on the critical path. After applying its moves at `tₖ` it
//! solves at the values it predicts for `tₖ₊₁` (its own solution one period ahead for each
//! mapped state, the next setpoint, the moves just applied), a background solve that keeps
//! its parametric sensitivity factor with respect to the measured or estimated state in the
//! session's retained state, charged to the job's allowance. At `tₖ₊₁` one backsolve against
//! that factor corrects the background solution to the actual state (ADR-0118). A
//! prediction that would change the active set, or a factor that was not kept, falls back to
//! a full solve at the sample, recorded with its reason. While the active set holds, the
//! prediction of a problem whose KKT conditions are linear in the state is its solution.
use pse_ids::SemanticId;

#[cfg(feature = "solver-diffsol")]
mod driver;

/// A loop value that a controller or estimator case value is bound to at each step.
#[derive(Clone, Debug, PartialEq)]
pub enum HorizonSignal {
    /// A plant output measured at the step's sample, by its contract output.
    Measured(SemanticId),
    /// The value the step's estimator solved at this estimator path: the estimate. A
    /// controller bound to one acts once the estimator has.
    Estimated(String),
    /// The value applied to this driven input (an index into [`Horizon::inputs`]) over the
    /// previous period: a held move.
    Applied(usize),
    /// One value per step, such as a setpoint trajectory.
    Trajectory(Vec<f64>),
}

/// A plant input the loop drives.
#[derive(Clone, Debug, PartialEq)]
pub struct HorizonInput {
    /// An unscheduled parameter of the plant's contract.
    pub parameter: SemanticId,
    /// The value applied until a controller move drives it.
    pub initial: f64,
}

/// The controller (NMPC) of a horizon.
#[derive(Clone, Debug)]
pub struct HorizonController {
    /// The package the controller's specification belongs to.
    pub package: super::ModelingPackage,
    /// A dynamic optimization over the controller's horizon, on the simultaneous route.
    /// Every bound path is one of its case values, so its steps rebind values only.
    pub analysis: super::ModelingAnalysis,
    /// The case values bound at every step: the state, setpoints and held moves.
    pub bindings: Vec<(String, HorizonSignal)>,
    /// The first move of each driven input: the controller path whose solved value is
    /// applied over the next period, and the input's index into [`Horizon::inputs`].
    pub moves: Vec<(String, usize)>,
    /// Decide samples by advanced-step prediction (Plan 22 Y5c2); `None` solves every
    /// sample in full.
    pub advanced: Option<AdvancedStep>,
}

/// An advanced-step controller: its measured and estimated state bindings are the
/// parameters of a sensitivity request, and each background solve runs at the predicted
/// next values of its bindings.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AdvancedStep {
    /// Each measured or estimated binding with the controller path whose solved value
    /// predicts it one period ahead. A binding without one is predicted to hold its value.
    pub predictions: Vec<(String, String)>,
}

/// How an estimator's model takes a driven input over its window.
#[derive(Clone, Debug, PartialEq)]
pub enum WindowInput {
    /// One case value for the whole window; the input must have been constant over it.
    Constant(String),
    /// One case value per period of the window, oldest first.
    Periods(Vec<String>),
}

/// The arrival cost of one estimated state: a prior on the window's free initial state.
#[derive(Clone, Debug, PartialEq)]
pub struct Arrival {
    /// The estimator's case value that holds the prior of the window's initial state.
    pub prior: String,
    /// The estimator path of that state one period into the window, which starts the next
    /// window: its solved value is the next prior.
    pub next: String,
    /// The prior of the first window.
    pub initial: f64,
}

/// The state estimator (MHE) of a horizon.
#[derive(Clone, Debug)]
pub struct HorizonEstimator {
    /// The package the estimator's specification belongs to.
    pub package: super::ModelingPackage,
    /// A least-squares problem over the last `window` periods, on the simultaneous route,
    /// whose initial state is free and regularized by the arrival cost. Every bound path is
    /// one of its case values.
    pub analysis: super::ModelingAnalysis,
    /// Periods in the estimation window. The estimator first acts once that many periods
    /// have been measured.
    pub window: usize,
    /// Each measured plant output, by contract output, with its `window + 1` case values
    /// over the window, oldest first.
    pub measurements: Vec<(SemanticId, Vec<String>)>,
    /// Each driven input the model takes, by its index into [`Horizon::inputs`].
    pub inputs: Vec<(usize, WindowInput)>,
    /// The arrival costs.
    pub arrival: Vec<Arrival>,
}

/// A closed-loop receding-horizon run over one native session (Plan 22 Y5c).
#[derive(Clone, Debug)]
pub struct Horizon {
    /// The plant: an integrated simulation whose profile covers the loop. Each period
    /// integrates it from the previous period's end state; the first starts from the
    /// simulation's own initial condition at its profile's start.
    pub plant: super::ModelingSimulation,
    /// The sample period, in seconds.
    pub period: f64,
    /// Closed-loop steps.
    pub steps: usize,
    /// The plant inputs the loop drives.
    pub inputs: Vec<HorizonInput>,
    /// The state estimator, when the controller acts on estimates.
    pub estimator: Option<HorizonEstimator>,
    /// The controller; without one the inputs stay at their initial values.
    pub controller: Option<HorizonController>,
}

/// How the inputs applied at a sample were decided.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HorizonDecision {
    /// No controller acted: there is none, or the estimate it binds does not exist (the
    /// estimator has not acted yet, or its step was not accepted). The inputs keep their
    /// last applied values.
    OpenLoop,
    /// The first moves of an accepted controller step.
    Solved,
    /// The controller step was not accepted; the last applied inputs are held.
    Held,
    /// The first moves predicted from the previous sample's background solve (Plan 22
    /// Y5c2).
    Predicted,
    /// The prediction was refused, for [`HorizonStep::fallback`]; the first moves of an
    /// accepted full solve at the sample.
    Fallback,
}
impl HorizonDecision {
    /// Stable snake-case spelling, as the `horizon.step` event records it.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OpenLoop => "open_loop",
            Self::Solved => "solved",
            Self::Held => "held",
            Self::Predicted => "predicted",
            Self::Fallback => "fallback",
        }
    }
}

/// One sample of a horizon run.
#[derive(Clone, Debug, PartialEq)]
pub struct HorizonStep {
    /// The sample time `tₖ`, in seconds.
    pub time: f64,
    /// The plant outputs at `tₖ`, in [`HorizonReport::outputs`] order.
    pub measured: Vec<f64>,
    /// The run's modeling step of this sample's estimator, when it acted.
    pub estimator: Option<usize>,
    /// The run's modeling step of this sample's controller, when it acted: for a
    /// prediction, the background solve whose factor predicted.
    pub controller: Option<usize>,
    /// How the applied inputs were decided.
    pub decision: HorizonDecision,
    /// Why an advanced-step controller did not predict at this sample; a full solve decided
    /// it instead.
    pub fallback: Option<pse_backend_native::kkt::Fallback>,
    /// The run's modeling step of the background solve an advanced-step controller ran at
    /// this sample for the next one.
    pub advanced: Option<usize>,
    /// The inputs applied over `[tₖ, tₖ + period]`, in [`HorizonReport::inputs`] order.
    pub applied: Vec<f64>,
    /// The plant outputs the period reached at `tₖ + period`.
    pub reached: Vec<f64>,
}

/// What a horizon run did at each sample; its estimator and controller steps are the run's
/// modeling steps, in order.
#[derive(Clone, Debug, PartialEq)]
pub struct HorizonReport {
    /// The plant's contract outputs: the columns of every measurement.
    pub outputs: Vec<SemanticId>,
    /// The driven plant parameters: the columns of every applied vector.
    pub inputs: Vec<SemanticId>,
    /// The completed samples, in order.
    pub steps: Vec<HorizonStep>,
}

#[cfg(all(test, feature = "solver-ipopt", feature = "solver-diffsol"))]
#[path = "horizon_tests.rs"]
mod tests;
