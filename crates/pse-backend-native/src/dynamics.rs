// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Finite semi-explicit dynamics. Numerical algorithms belong to Diffsol and SUNDIALS IDAS
//! (ADR-0084, ADR-0093, ADR-0110).
use crate::ProblemError;
use pse_ids::{ContentHash, SemanticId};
use pse_math::index::{OriginalCol, OriginalRow};
use pse_model::{document::Version, scalars::PositiveCount};
use std::{
    collections::BTreeSet,
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};

#[cfg(feature = "idas")]
mod idas;
#[cfg(feature = "diffsol")]
mod integrator;

/// Registry vocabularies of the dynamics settings (ADR-0110, ADR-0115 Outcome 3): the
/// requested integration algorithm (`auto` resolves from trial requirements), the contract
/// for domain errors at internal trial points, Diffsol's library-owned time-stepping scheme
/// and the sparse factorization of its Newton matrices, IDAS's forward-sensitivity
/// corrector (`IDASensInit`), its consistent initialization (`IDACalcIC`) and the declared
/// sign of a normalized state (`IDASetConstraints`).
pub use pse_model::generated::enums::{
    DiffsolLinear, DiffsolMethod, DynamicsMethod as Method, IdasInitialization,
    SensitivityCorrector, StateSign, TrialPolicy,
};
/// Typed Diffsol-only method controls, a versioned boundary document (ADR-0116 Outcome 6):
/// the version is required, and absent fields take these defaults.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct DiffsolSettings {
    /// Document version.
    pub version: Version<1>,
    /// Time-stepping scheme.
    #[serde(default = "DiffsolSettings::default_method")]
    pub method: DiffsolMethod,
    /// Newton linear solver of the implicit schemes.
    #[serde(default = "DiffsolSettings::default_linear")]
    pub linear: DiffsolLinear,
}
impl DiffsolSettings {
    const fn default_method() -> DiffsolMethod {
        DiffsolMethod::Bdf
    }
    const fn default_linear() -> DiffsolLinear {
        DiffsolLinear::FaerLu
    }
}
impl Default for DiffsolSettings {
    /// Variable-order BDF over faer sparse LU.
    fn default() -> Self {
        Self {
            version: Version,
            method: Self::default_method(),
            linear: Self::default_linear(),
        }
    }
}
/// IDAS Newton linear solver.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum IdasLinear {
    /// SuiteSparse KLU over the compiled analytic Jacobian.
    #[default]
    Klu,
    /// Matrix-free GMRES over analytic Jacobian-vector products.
    Spgmr {
        /// Maximum Krylov subspace dimension.
        dimension: PositiveCount,
        /// Left preconditioner built from the compiled Jacobian; none when absent.
        #[serde(default = "unpreconditioned")]
        preconditioner: crate::solve::Preconditioner,
    },
    /// Matrix-free flexible GMRES over analytic Jacobian-vector products.
    Spfgmr {
        /// Maximum Krylov subspace dimension.
        dimension: PositiveCount,
        /// Left preconditioner built from the compiled Jacobian; none when absent.
        #[serde(default = "unpreconditioned")]
        preconditioner: crate::solve::Preconditioner,
    },
}
const fn unpreconditioned() -> crate::solve::Preconditioner {
    crate::solve::Preconditioner::None
}
/// The native IDAS/KINSOL constraint code of a declared state sign.
#[cfg_attr(
    not(feature = "idas"),
    expect(dead_code, reason = "the native constraint codes exist only with IDAS")
)]
pub(crate) const fn state_sign_code(sign: StateSign) -> f64 {
    match sign {
        StateSign::Free => 0.0,
        StateSign::NonNegative => 1.0,
        StateSign::Positive => 2.0,
        StateSign::NonPositive => -1.0,
        StateSign::Negative => -2.0,
    }
}
/// Typed IDAS-only method controls (ADR-0110 item 1), a versioned boundary document
/// (ADR-0116 Outcome 6): the version is required, and absent fields take these defaults.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct IdasSettings {
    /// Document version.
    pub version: Version<1>,
    /// Newton linear solver.
    #[serde(default)]
    pub linear: IdasLinear,
    /// Forward-sensitivity corrector.
    #[serde(default = "IdasSettings::default_sensitivity")]
    pub sensitivity: SensitivityCorrector,
    /// Consistent initialization at the start of the horizon; scheduled changes and resets
    /// always keep their differential states.
    #[serde(default = "IdasSettings::default_initialization")]
    pub initialization: IdasInitialization,
    /// Empty, or one declared sign per state in state order.
    #[serde(default)]
    pub constraints: Vec<StateSign>,
}
impl IdasSettings {
    const fn default_sensitivity() -> SensitivityCorrector {
        SensitivityCorrector::Simultaneous
    }
    const fn default_initialization() -> IdasInitialization {
        IdasInitialization::AlgebraicAndRates
    }
}
impl Default for IdasSettings {
    /// KLU over the analytic Jacobian, the simultaneous corrector, an initialization that
    /// keeps the requested differential states, and no declared signs.
    fn default() -> Self {
        Self {
            version: Version,
            linear: IdasLinear::Klu,
            sensitivity: Self::default_sensitivity(),
            initialization: Self::default_initialization(),
            constraints: Vec::new(),
        }
    }
}
/// Guard crossing detected by an event (`IDASetRootDirection`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Crossing {
    /// Every sign change.
    #[default]
    Either,
    /// Only a guard increasing through zero.
    Rising,
    /// Only a guard decreasing through zero.
    Falling,
}
impl Crossing {
    /// The native IDAS root direction.
    pub const fn code(self) -> i32 {
        match self {
            Self::Either => 0,
            Self::Rising => 1,
            Self::Falling => -1,
        }
    }
}

/// Dispatch only after checking the complete integration requirements.
#[cfg(any(feature = "diffsol", feature = "idas"))]
pub fn integrate(
    oracle: &mut dyn Oracle,
    profile: &Profile,
    parameters: &[f64],
    cancel: Cancellation,
) -> Result<Report, ProblemError> {
    integrate_with_progress(
        oracle,
        profile,
        parameters,
        cancel,
        Arc::new(crate::solve::Progress::new(256)),
    )
}
/// Execute with the caller-owned bounded progress sink.
#[cfg(any(feature = "diffsol", feature = "idas"))]
pub fn integrate_with_progress(
    oracle: &mut dyn Oracle,
    profile: &Profile,
    parameters: &[f64],
    cancel: Cancellation,
    progress: Arc<crate::solve::Progress>,
) -> Result<Report, ProblemError> {
    profile.validate(oracle.contract(), parameters)?;
    match profile.resolved_method()? {
        #[cfg(feature = "diffsol")]
        Method::Diffsol => {
            integrator::integrate_with_progress(oracle, profile, parameters, cancel, progress)
        }
        #[cfg(feature = "idas")]
        Method::Idas => {
            idas::integrate_with_progress(oracle, profile, parameters, cancel, progress)
        }
        _ => Err(ProblemError::unsupported(
            "requested dynamic backend is not linked",
        )),
    }
}

/// One compiled function role, not a second expression representation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Function {
    /// Differential rates followed by the selected algebraic residuals.
    Rhs,
    /// Initial differential values and algebraic guesses.
    Initial,
    /// User-selected physical outputs.
    Output,
    /// Physical integrands evaluated by native output quadrature.
    QuadratureFlux,
    /// Active guard functions, in stable event order.
    Roots,
    /// Complete post-event state for one event.
    Reset(usize),
}
/// One event's already compiled behavior.
#[derive(Clone, Debug)]
pub struct Event {
    /// Stable source identity.
    pub id: SemanticId,
    /// Stop at the event without applying a reset.
    pub terminal: bool,
    /// Post-reset mode, with the same state/parameter layout.
    pub next_mode: usize,
    /// Absolute normalized guard tolerance for ambiguity detection.
    pub tolerance: f64,
    /// Guard crossings that trigger the event; Diffsol detects every sign change.
    pub direction: Crossing,
}
/// One conserved state whose signed flux is integrated by the native solver.
#[derive(Clone, Debug)]
pub struct Balance {
    /// Original balance identity and flux output.
    pub id: SemanticId,
    /// Conserved differential state index.
    pub state: usize,
    /// Normalized coordinate to canonical conserved quantity scale.
    pub scale: f64,
    /// Absolute conserved quantity closure tolerance.
    pub tolerance: f64,
    /// Explicit physical jumps keyed by authored event identity.
    pub impulses: std::collections::BTreeMap<SemanticId, f64>,
}
/// Exact admitted layout. State coordinates are normalized; outputs are physical.
#[derive(Clone, Debug)]
pub struct Contract {
    /// Ordered physical quadrature identities, independent of conservation claims.
    pub quadratures: Vec<SemanticId>,
    /// Independently observed conservation contracts consuming named quadratures.
    pub balances: Vec<Balance>,
    /// Complete physical/source/profile interpretation.
    pub identity: ContentHash,
    /// Ordered state identities.
    pub states: Vec<SemanticId>,
    /// Fixed diagonal mass entries: true = one; false = zero.
    pub differential: Vec<bool>,
    /// Ordered varying parameters; other numeric inputs remain fixed.
    pub parameters: Vec<SemanticId>,
    /// Ordered physical outputs.
    pub outputs: Vec<SemanticId>,
    /// Same-layout modes and their active roots.
    pub events: Vec<Vec<Event>>,
}
impl Contract {
    /// Validate finite layout and the concrete supported mass/event profile.
    pub fn validate(&self) -> Result<(), ProblemError> {
        let unique = |ids: &[SemanticId]| ids.iter().collect::<BTreeSet<_>>().len() == ids.len();
        if !unique(&self.quadratures)
            || !unique(&self.balances.iter().map(|b| b.id).collect::<Vec<_>>())
            || self.balances.iter().any(|b| {
                !self.quadratures.contains(&b.id)
                    || b.state >= self.states.len()
                    || !self.differential.get(b.state).copied().unwrap_or(false)
                    || !positive(b.scale)
                    || !positive(b.tolerance)
                    || b.impulses.values().any(|v| !v.is_finite())
            })
            || self.states.is_empty()
            || self.outputs.is_empty()
            || self.events.is_empty()
            || self.states.len() != self.differential.len()
            || !self.differential.contains(&true)
            || !unique(&self.states)
            || !unique(&self.parameters)
            || !unique(&self.outputs)
            || self.states.iter().any(|s| self.parameters.contains(s))
            || self.events.iter().any(|events| {
                !unique(&events.iter().map(|e| e.id).collect::<Vec<_>>())
                    || events
                        .iter()
                        .any(|e| e.next_mode >= self.events.len() || !positive(e.tolerance))
            })
        {
            return Err(contract(
                "dynamic state, mass, parameter, output or event layout",
            ));
        }
        Ok(())
    }
}
/// Finite integration policy. Tolerances apply to the normalized state coordinates.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    /// Native method selected from the required trial semantics; automatic when absent.
    #[serde(default = "automatic")]
    pub method: Method,
    /// Behavior required for internal domain failures; terminal when absent.
    #[serde(default = "terminal")]
    pub trial_failures: TrialPolicy,
    /// Physical acceptance and ID-keyed nominal requests, distinct from integration error controls.
    #[serde(default)]
    pub numerics: pse_model::numerics::NumericalPolicy,
    /// Initial physical time in seconds.
    pub start: f64,
    /// Requested final physical time in seconds.
    pub end: f64,
    /// Strictly increasing finite output times, within the horizon.
    pub samples: Vec<f64>,
    /// Positive relative integration tolerance.
    pub rtol: f64,
    /// Explicit integrated-flux relative tolerance; required when conservation is declared.
    pub out_rtol: Option<f64>,
    /// Absolute integrated-flux tolerances in canonical conserved quantities.
    pub out_atol: Vec<f64>,
    /// Positive absolute tolerances in normalized state order.
    pub atol: Vec<f64>,
    /// Initial step in seconds.
    pub initial_step: f64,
    /// Maximum native step calls across all segments.
    pub max_steps: usize,
    /// Maximum retained event records.
    pub max_events: usize,
    /// Cooperative outer time allowance, including callbacks and resets.
    pub time_limit: Duration,
    /// Maximum retained scalar cells, including sensitivities and event states.
    pub max_cells: usize,
    /// Integrate smooth forward sensitivities for every selected parameter.
    pub sensitivities: bool,
    /// Positive characteristic parameter scales, also used for sensitivity tolerances.
    pub parameter_scales: Vec<f64>,
    /// Fixed-time replacement of all parameters; carried-state sensitivities remain active.
    pub changes: Vec<InputChange>,
    /// Typed Diffsol scheme and linear solver.
    #[serde(default)]
    pub diffsol: DiffsolSettings,
    /// Typed IDAS linear solver, sensitivity corrector, start and sign constraints.
    #[serde(default)]
    pub idas: IdasSettings,
    /// Native initialization controls, available in the linked profile.
    #[cfg(feature = "diffsol")]
    #[serde(with = "initial_options")]
    pub initialization: Arc<diffsol::InitialConditionSolverOptions<f64>>,
    /// Native BDF/nonlinear/error-control settings.
    #[cfg(feature = "diffsol")]
    #[serde(with = "ode_options")]
    pub native: Arc<diffsol::OdeSolverOptions<f64>>,
}
/// A scheduled replacement of the complete selected parameter vector.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputChange {
    /// Exact physical time.
    pub time: f64,
    /// Values in declared parameter order.
    pub parameters: Vec<f64>,
}
impl Profile {
    /// Resolve algorithm and trial semantics without acquiring a worker.
    pub fn resolved_method(&self) -> Result<Method, ProblemError> {
        let method = match self.method {
            Method::Auto if self.trial_failures == TrialPolicy::Recoverable => Method::Idas,
            Method::Auto => Method::Diffsol,
            m => m,
        };
        if method == Method::Diffsol && self.trial_failures == TrialPolicy::Recoverable {
            return Err(contract(
                "Diffsol cannot recover typed trial failures; request IDAS",
            ));
        }
        if (method == Method::Idas && !cfg!(feature = "idas"))
            || (method == Method::Diffsol && !cfg!(feature = "diffsol"))
        {
            return Err(ProblemError::unsupported(
                "requested dynamic backend is not linked",
            ));
        }
        Ok(method)
    }
    /// Validate before allocation or native construction; arithmetic overflow is a refusal.
    pub fn validate(&self, c: &Contract, p: &[f64]) -> Result<usize, ProblemError> {
        c.validate()?;
        let n = c.states.len();
        match self.resolved_method()? {
            Method::Idas => {
                #[cfg(feature = "diffsol")]
                if settings_identity(self) != settings_identity(&Self::default()) {
                    return Err(contract(
                        "Diffsol-specific controls cannot be applied to IDAS",
                    ));
                }
                // ADR-0110 item 1: Diffsol owns reset sensitivities; IDAS has no saltation.
                if self.sensitivities && c.events.iter().any(|e| !e.is_empty()) {
                    return Err(ProblemError::unsupported(
                        "IDAS forward sensitivities do not cross events; Diffsol owns reset sensitivities",
                    ));
                }
                let i = &self.idas;
                if !(i.constraints.is_empty() || i.constraints.len() == n)
                    || (!i.constraints.is_empty()
                        && i.constraints.iter().all(|s| *s == StateSign::Free))
                    || match i.linear {
                        IdasLinear::Klu => false,
                        IdasLinear::Spgmr { dimension, .. }
                        | IdasLinear::Spfgmr { dimension, .. } => {
                            i32::try_from(dimension.into_inner()).is_err()
                        }
                    }
                {
                    return Err(contract("invalid IDAS constraint or Krylov control"));
                }
            }
            Method::Diffsol => {
                if self.idas != IdasSettings::default() {
                    return Err(contract(
                        "IDAS-specific controls cannot be applied to Diffsol",
                    ));
                }
                if c.events
                    .iter()
                    .flatten()
                    .any(|e| e.direction != Crossing::Either)
                {
                    return Err(ProblemError::unsupported(
                        "Diffsol detects every guard sign change; a directional event needs IDAS",
                    ));
                }
                if self.diffsol.method == DiffsolMethod::Tsit45 {
                    // ADR-0110 item 2: the explicit scheme is admitted for mass-free ODEs only.
                    if c.differential.contains(&false) {
                        return Err(ProblemError::unsupported(
                            "explicit tsit45 integrates mass-free ODEs only; algebraic states need an implicit scheme",
                        ));
                    }
                    if self.diffsol.linear != DiffsolSettings::default().linear {
                        return Err(contract("explicit tsit45 has no Newton linear solver"));
                    }
                }
            }
            Method::Auto => return Err(ProblemError::internal("unresolved dynamic method")),
        }
        if self.sensitivities && c.events.iter().flatten().any(|e| e.terminal) {
            return Err(contract(
                "terminal-event sensitivities require a declared event-time output contract",
            ));
        }
        if c.quadratures.is_empty() {
            if self.out_rtol.is_some() || !self.out_atol.is_empty() {
                return Err(contract("output integration tolerance without quadratures"));
            }
        } else if self.out_rtol.is_none_or(|v| !positive(v))
            || self.out_atol.len() != c.quadratures.len()
            || self.out_atol.iter().any(|v| !positive(*v))
        {
            return Err(contract(
                "explicit physical output integration tolerances required",
            ));
        }
        #[cfg(feature = "diffsol")]
        {
            let i = &self.initialization;
            let n = &self.native;
            if i.max_linesearch_iterations == 0
                || i.max_newton_iterations == 0
                || i.max_linear_solver_setups == 0
                || ![i.step_reduction_factor, i.armijo_constant]
                    .iter()
                    .all(|x| x.is_finite() && *x > 0.0 && *x < 1.0)
                || n.max_nonlinear_solver_iterations == 0
                || n.max_error_test_failures == 0
                || n.max_nonlinear_solver_failures == 0
                || !positive(n.nonlinear_solver_tolerance)
                || !positive(n.min_timestep)
                || [
                    n.threshold_to_update_jacobian,
                    n.threshold_to_update_rhs_jacobian,
                    n.pi_control_proportional,
                    n.pi_control_integral,
                ]
                .iter()
                .any(|x| !x.is_finite() || *x < 0.0)
                || [n.max_timestep_growth, n.min_timestep_growth]
                    .into_iter()
                    .flatten()
                    .any(|x| !positive(x) || x < 1.0)
                || [n.max_timestep_shrink, n.min_timestep_shrink]
                    .into_iter()
                    .flatten()
                    .any(|x| !positive(x) || x > 1.0)
                || n.max_timestep_growth
                    .zip(n.min_timestep_growth)
                    .is_some_and(|(max, min)| max < min)
                || n.max_timestep_shrink
                    .zip(n.min_timestep_shrink)
                    .is_some_and(|(max, min)| max < min)
            {
                return Err(contract("invalid native Diffsol control"));
            }
        }
        let m = c.outputs.len();
        let np = c.parameters.len();
        if !self.start.is_finite()
            || !self.end.is_finite()
            || self.start >= self.end
            || !positive(self.rtol)
            || !positive(self.initial_step)
            || self.atol.len() != n
            || self.atol.iter().any(|v| !positive(*v))
            || p.len() != np
            || p.iter().any(|v| !v.is_finite())
            || self.parameter_scales.len() != np
            || self.parameter_scales.iter().any(|v| !positive(*v))
            || self.max_steps == 0
            || self.max_events == 0
            || self.time_limit.is_zero()
            || self.samples.is_empty()
            || self
                .samples
                .iter()
                .any(|t| !t.is_finite() || *t < self.start || *t > self.end)
            || self.samples.windows(2).any(|w| w[0] >= w[1])
            || self.changes.iter().any(|x| {
                !x.time.is_finite()
                    || x.time <= self.start
                    || x.time > self.end
                    || x.parameters.len() != np
                    || x.parameters.iter().any(|v| !v.is_finite())
            })
            || self.changes.windows(2).any(|w| w[0].time >= w[1].time)
            || (self.sensitivities && np == 0)
            || (self.sensitivities
                && !c.quadratures.is_empty()
                && c.events.iter().any(|e| !e.is_empty()))
        {
            return Err(contract(
                "dynamic horizon, tolerances, budget or smooth sensitivity profile",
            ));
        }
        let cells = self
            .samples
            .len()
            .checked_mul(
                m.checked_add(n)
                    .and_then(|v| v.checked_add(c.quadratures.len()))
                    .ok_or_else(|| contract("dynamic output extent"))?,
            )
            .and_then(|v| {
                v.checked_mul(if self.sensitivities {
                    np.checked_add(1)?
                } else {
                    1
                })
            })
            .and_then(|v| {
                self.max_events
                    .checked_mul(n)?
                    .checked_mul(2)?
                    .checked_add(v)
            })
            .ok_or_else(|| contract("dynamic result extent overflow"))?;
        if cells > self.max_cells {
            return Err(contract("dynamic result cell allowance"));
        }
        Ok(cells)
    }
}
/// Function value and raw partials with respect to state followed by parameters.
#[derive(Clone, Debug)]
pub struct Evaluation {
    /// Ordered function values.
    pub values: Vec<f64>,
    /// Canonical sparse raw partials, present only on derivative demand.
    pub jacobian: Option<faer::sparse::SparseColMat<usize, f64>>,
}
/// One structural entry of a dynamic function's raw partials: a row of the function's
/// values and a coordinate of the state followed by the parameters, both positions as
/// the oracle states them. Integrator adapters convert to native indices at their edge.
///
/// ```
/// use pse_backend_native::dynamics::SupportEntry;
/// use pse_math::index::{OriginalCol, OriginalRow};
///
/// let entry = SupportEntry::new(OriginalRow::new(0), OriginalCol::new(2));
/// assert_eq!((entry.row.get(), entry.col.get()), (0, 2));
/// ```
///
/// ```compile_fail,E0308
/// use pse_backend_native::dynamics::SupportEntry;
/// use pse_math::index::OriginalCol;
///
/// // A coordinate is not a function row.
/// let entry = SupportEntry::new(OriginalCol::new(2), OriginalCol::new(0));
/// ```
pub type SupportEntry = pse_math::index::Entry<OriginalRow, OriginalCol>;

/// Support entries from `(row, coordinate)` positions, for test oracles.
#[cfg(test)]
pub(crate) fn entries(pairs: impl IntoIterator<Item = (usize, usize)>) -> Vec<SupportEntry> {
    pairs
        .into_iter()
        .map(|(row, column)| SupportEntry::new(OriginalRow::new(row), OriginalCol::new(column)))
        .collect()
}

/// Worker-local compiled mathematical operations. No library state enters compiler queries.
pub trait Oracle: std::fmt::Debug {
    /// Immutable source layout.
    fn contract(&self) -> &Contract;
    /// Complete all-branch sparse support for the selected function and mode.
    fn support(&self, mode: usize, function: Function) -> Vec<SupportEntry>;
    /// Evaluate exact compiled functions and requested raw partials.
    fn evaluate(
        &mut self,
        mode: usize,
        function: Function,
        time: f64,
        state: &[f64],
        parameters: &[f64],
        derivatives: bool,
    ) -> Result<Evaluation, ProblemError>;
}
/// Successfully completed output point; no preallocated placeholder is observable.
#[derive(Clone, Debug)]
pub struct Sample {
    /// Actual active mode at this sample, after any coincident nonterminal reset.
    pub mode: usize,
    /// Cumulative physical quadratures, integrated natively across completed segments.
    pub integrals: Vec<f64>,
    /// Physical time in seconds.
    pub time: f64,
    /// Normalized state coordinates; physical conversion belongs to the declared projection.
    pub state: Vec<f64>,
    /// Ordered physical outputs.
    pub outputs: Vec<f64>,
    /// State-major parameter derivatives, empty when not requested.
    pub state_sensitivities: Vec<f64>,
    /// Output-major parameter derivatives, empty when not requested.
    pub output_sensitivities: Vec<f64>,
}
/// An actual native root or scheduled input transition.
#[derive(Clone, Debug)]
pub struct EventRecord {
    /// Source event, absent for a scheduled input transition.
    pub event: Option<SemanticId>,
    /// Physical event time.
    pub time: f64,
    /// State before consistency/reset.
    pub before: Vec<f64>,
    /// State after consistency/reset, absent for terminal events or failed resets.
    pub after: Option<Vec<f64>>,
}
/// Registry-owned integration outcome, independent of physical acceptance.
pub use pse_model::generated::enums::TrajectoryTermination as Termination;
/// Joined report retaining valid completed data even when a later callback fails.
#[derive(Debug)]
pub struct Report {
    /// Native/profile outcome.
    pub termination: Termination,
    /// Last completed physical time; initial time until initialization succeeds.
    pub completed_time: f64,
    /// Requested initial state and guesses.
    pub requested_initial: Vec<f64>,
    /// Native consistent initial state.
    pub consistent_initial: Vec<f64>,
    /// Completed samples only.
    pub samples: Vec<Sample>,
    /// Actual event transitions only.
    pub events: Vec<EventRecord>,
    /// Native statistics for each finished segment, without invented counters.
    pub statistics: Vec<serde_json::Value>,
    /// Attributable failure; successful reports contain none.
    pub error: Option<ProblemError>,
    /// Bounded actual integration progress.
    pub progress: Vec<crate::solve::Event>,
    /// Actual count discarded by the bounded progress owner.
    pub dropped_progress: u64,
}
impl Report {
    /// Known retained trajectory buffers. Opaque statistics/error/progress storage
    /// is covered separately by the runtime's report allowance.
    pub fn numeric_bytes(&self) -> usize {
        size_of::<Self>()
            + (self.requested_initial.capacity() + self.consistent_initial.capacity())
                * size_of::<f64>()
            + self.samples.capacity() * size_of::<Sample>()
            + self
                .samples
                .iter()
                .map(|s| {
                    (s.integrals.capacity()
                        + s.state.capacity()
                        + s.outputs.capacity()
                        + s.state_sensitivities.capacity()
                        + s.output_sensitivities.capacity())
                        * size_of::<f64>()
                })
                .sum::<usize>()
            + self.events.capacity() * size_of::<EventRecord>()
            + self
                .events
                .iter()
                .map(|e| {
                    (e.before.capacity() + e.after.as_ref().map_or(0, Vec::capacity))
                        * size_of::<f64>()
                })
                .sum::<usize>()
    }
    #[cfg(any(feature = "diffsol", feature = "idas"))]
    pub(crate) fn new(start: f64) -> Self {
        Self {
            termination: Termination::Failed,
            completed_time: start,
            requested_initial: vec![],
            consistent_initial: vec![],
            samples: vec![],
            events: vec![],
            statistics: vec![],
            error: None,
            progress: vec![],
            dropped_progress: 0,
        }
    }
}
pub(crate) fn contract(message: &str) -> ProblemError {
    ProblemError::Contract(message.into())
}
fn positive(x: f64) -> bool {
    x.is_finite() && x > 0.0
}
/// Share the same outer cancellation flag across fitting and integration.
pub type Cancellation = Arc<AtomicBool>;

impl std::fmt::Debug for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynamicProfile")
            .field("start", &self.start)
            .field("end", &self.end)
            .field("samples", &self.samples)
            .field("sensitivities", &self.sensitivities)
            .finish_non_exhaustive()
    }
}

/// Every Diffsol-only control, in the serde encoding of the profile (F09): the remote
/// definitions fail to compile when a pinned option type gains a field, and no field
/// list is written by hand.
#[cfg(feature = "diffsol")]
#[derive(serde::Serialize)]
struct DiffsolIdentity<'a> {
    #[serde(with = "initial_options")]
    initialization: Arc<diffsol::InitialConditionSolverOptions<f64>>,
    #[serde(with = "ode_options")]
    native: Arc<diffsol::OdeSolverOptions<f64>>,
    diffsol: &'a DiffsolSettings,
}
/// Exact Diffsol-only controls included in environment identity and reporting.
#[cfg(feature = "diffsol")]
pub fn settings_identity(p: &Profile) -> String {
    encoded(&DiffsolIdentity {
        initialization: p.initialization.clone(),
        native: p.native.clone(),
        diffsol: &p.diffsol,
    })
    .to_string()
}

/// All effective integration controls for durable provenance and profile identity: the
/// complete serde encoding of the profile plus the resolved method (F09).
pub fn profile_json(p: &Profile) -> serde_json::Value {
    let mut value = encoded(p);
    if let serde_json::Value::Object(fields) = &mut value {
        fields.insert("resolved_method".into(), encoded(&p.resolved_method().ok()));
    }
    value
}
/// These encodings have only string map keys, so serialization cannot fail; a failure
/// still yields a distinct value rather than a shared placeholder.
fn encoded<T: serde::Serialize>(value: &T) -> serde_json::Value {
    serde_json::to_value(value)
        .unwrap_or_else(|e| serde_json::Value::String(format!("unencodable: {e}")))
}
/// The typed trajectory transitions at `time` receive their post-transition state. A
/// conserved state may jump only by its declared event impulse (shared by both
/// integrators).
#[cfg(any(feature = "diffsol", feature = "idas"))]
pub(crate) fn settle_transitions(
    layout: &Contract,
    events: &mut [EventRecord],
    time: f64,
    state: &[f64],
) -> Result<(), ProblemError> {
    for event in events
        .iter_mut()
        .rev()
        .take_while(|e| e.time == time && e.after.is_none())
    {
        for balance in &layout.balances {
            let jump = (state[balance.state] - event.before[balance.state]) * balance.scale;
            let declared = event
                .event
                .and_then(|id| balance.impulses.get(&id).copied())
                .unwrap_or(0.0);
            if !jump.is_finite() || (jump - declared).abs() > balance.tolerance {
                return Err(contract(
                    "conserved state jump differs from its declared event impulse",
                ));
            }
        }
        event.after = Some(state.to_vec());
    }
    Ok(())
}
/// Number of guards within their declared ambiguity tolerance.
#[cfg(any(feature = "diffsol", feature = "idas"))]
pub(crate) fn guards_at_zero(guards: &[f64], events: &[Event]) -> usize {
    guards
        .iter()
        .zip(events)
        .filter(|(g, e)| g.abs() <= e.tolerance)
        .count()
}

#[cfg(feature = "diffsol")]
mod initial_options {
    use super::*;
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(
        remote = "diffsol::InitialConditionSolverOptions<f64>",
        deny_unknown_fields
    )]
    struct Remote {
        use_linesearch: bool,
        max_linesearch_iterations: usize,
        max_newton_iterations: usize,
        max_linear_solver_setups: usize,
        step_reduction_factor: f64,
        armijo_constant: f64,
    }
    pub(super) fn serialize<S: serde::Serializer>(
        value: &Arc<diffsol::InitialConditionSolverOptions<f64>>,
        s: S,
    ) -> Result<S::Ok, S::Error> {
        Remote::serialize(value, s)
    }
    pub(super) fn deserialize<'de, D: serde::Deserializer<'de>>(
        d: D,
    ) -> Result<Arc<diffsol::InitialConditionSolverOptions<f64>>, D::Error> {
        Remote::deserialize(d).map(Arc::new)
    }
}

#[cfg(feature = "diffsol")]
mod ode_options {
    use super::*;
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(remote = "diffsol::OdeSolverOptions<f64>", deny_unknown_fields)]
    struct Remote {
        max_nonlinear_solver_iterations: usize,
        max_error_test_failures: usize,
        max_nonlinear_solver_failures: usize,
        nonlinear_solver_tolerance: f64,
        min_timestep: f64,
        max_timestep_growth: Option<f64>,
        min_timestep_growth: Option<f64>,
        max_timestep_shrink: Option<f64>,
        min_timestep_shrink: Option<f64>,
        update_jacobian_after_steps: usize,
        update_rhs_jacobian_after_steps: usize,
        threshold_to_update_jacobian: f64,
        threshold_to_update_rhs_jacobian: f64,
        pi_control_proportional: f64,
        pi_control_integral: f64,
    }
    pub(super) fn serialize<S: serde::Serializer>(
        value: &Arc<diffsol::OdeSolverOptions<f64>>,
        s: S,
    ) -> Result<S::Ok, S::Error> {
        Remote::serialize(value, s)
    }
    pub(super) fn deserialize<'de, D: serde::Deserializer<'de>>(
        d: D,
    ) -> Result<Arc<diffsol::OdeSolverOptions<f64>>, D::Error> {
        Remote::deserialize(d).map(Arc::new)
    }
}

const fn automatic() -> Method {
    Method::Auto
}
const fn terminal() -> TrialPolicy {
    TrialPolicy::Terminal
}
impl Default for Profile {
    fn default() -> Self {
        Self {
            method: automatic(),
            trial_failures: terminal(),
            numerics: Default::default(),
            start: 0.0,
            end: 1.0,
            samples: vec![0.0, 1.0],
            rtol: 1e-6,
            out_rtol: None,
            out_atol: vec![],
            atol: vec![1e-8],
            initial_step: 1e-4,
            max_steps: 100000,
            max_events: 1000,
            time_limit: Duration::from_secs(300),
            max_cells: 1_000_000,
            sensitivities: false,
            parameter_scales: vec![],
            changes: vec![],
            diffsol: DiffsolSettings::default(),
            idas: IdasSettings::default(),
            #[cfg(feature = "diffsol")]
            initialization: Arc::new(Default::default()),
            #[cfg(feature = "diffsol")]
            native: Arc::new(Default::default()),
        }
    }
}

#[cfg(all(test, feature = "diffsol"))]
mod tests;
